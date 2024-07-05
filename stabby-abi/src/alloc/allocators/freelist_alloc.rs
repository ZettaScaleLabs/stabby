use core::{mem::MaybeUninit, ptr::NonNull, sync::atomic::AtomicPtr};

/// A global [`FreelistAlloc`].
///
/// This allocator is 0-sized and thread safe (by spin-lock).
#[crate::stabby]
#[derive(Clone, Copy, Default)]
pub struct FreelistGlobalAlloc {
    inner: [u8; 0],
}
impl core::fmt::Debug for FreelistGlobalAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("FreelistAlloc")
    }
}
impl FreelistGlobalAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}
fn allock() -> FreelistGlobalAllock {
    loop {
        let mut ptr =
            GLOBAL_ALLOC.swap(usize::MAX as *mut _, core::sync::atomic::Ordering::Acquire);
        if ptr as usize == usize::MAX {
            core::hint::spin_loop();
            continue;
        }
        return FreelistGlobalAllock {
            alloc: FreelistAlloc {
                head: unsafe { ptr.as_mut() },
                end: NonNull::new(GLOBAL_ALLOC_END.load(core::sync::atomic::Ordering::Acquire)),
            },
        };
    }
}
static GLOBAL_ALLOC: AtomicPtr<Slot> = AtomicPtr::new(core::ptr::null_mut());
static GLOBAL_ALLOC_END: AtomicPtr<Slot> = AtomicPtr::new(core::ptr::null_mut());
struct FreelistGlobalAllock {
    alloc: FreelistAlloc,
}
impl Drop for FreelistGlobalAllock {
    fn drop(&mut self) {
        GLOBAL_ALLOC_END.store(
            unsafe { core::mem::transmute::<Option<NonNull<Slot>>, *mut Slot>(self.alloc.end) },
            core::sync::atomic::Ordering::Release,
        );
        GLOBAL_ALLOC.store(
            unsafe {
                core::mem::transmute::<Option<&'static mut Slot>, *mut Slot>(self.alloc.head.take())
            },
            core::sync::atomic::Ordering::Release,
        )
    }
}
impl crate::alloc::IAlloc for FreelistGlobalAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        let mut alloc = allock();
        (&mut alloc.alloc).alloc(layout)
    }
    unsafe fn realloc(
        &mut self,
        ptr: *mut (),
        prev_layout: crate::alloc::Layout,
        new_size: usize,
    ) -> *mut () {
        let mut alloc = allock();
        (&mut alloc.alloc).realloc(ptr, prev_layout, new_size)
    }
    unsafe fn free(&mut self, ptr: *mut ()) {
        let mut alloc = allock();
        (&mut alloc.alloc).free(ptr)
    }
}

/// A free-list based allocator.
#[crate::stabby]
#[derive(Default)]
pub struct FreelistAlloc {
    head: Option<&'static mut Slot>,
    end: Option<NonNull<Slot>>,
}
impl crate::alloc::IAlloc for &mut FreelistAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        let layout = layout.for_alloc();
        match self.take(layout) {
            Some(slot) => {
                let this = unsafe { (slot as *mut Slot).add(1).cast::<()>() };
                this
            }
            None => core::ptr::null_mut(),
        }
    }
    unsafe fn realloc(
        &mut self,
        ptr: *mut (),
        prev_layout: crate::alloc::Layout,
        new_size: usize,
    ) -> *mut () {
        let slot = ptr.cast::<Slot>().sub(1);
        let mut slot = unsafe { &mut *slot };
        let prev_size = slot.size;
        match self.try_extend(&mut slot, new_size) {
            Ok(()) => ptr,
            Err(()) => {
                let new_ptr = self.alloc(crate::alloc::Layout {
                    size: new_size,
                    align: prev_layout.align,
                });
                unsafe {
                    core::ptr::copy_nonoverlapping(ptr.cast::<u8>(), new_ptr.cast(), prev_size)
                };
                self.insert(slot);
                new_ptr
            }
        }
    }
    unsafe fn free(&mut self, p: *mut ()) {
        let slot = p.cast::<Slot>().sub(1);
        self.insert(&mut *slot);
    }
}
impl core::fmt::Debug for FreelistAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FreelistAlloc(end={:?})",
            self.end.map_or(core::ptr::null_mut(), |p| p.as_ptr())
        )?;
        let mut f = f.debug_list();
        let mut head = &self.head;
        while let Some(slot) = head {
            f.entry(&slot);
            head = &slot.lower;
        }
        f.finish()
    }
}

#[crate::stabby]
pub struct Slot {
    size: usize,
    lower: Option<&'static mut Slot>,
    start: NonNull<Slot>,
    align: usize,
}
impl core::fmt::Debug for Slot {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Slot")
            .field("start", &self.start())
            .field("end", &self.end())
            .field("size", &self.size)
            .finish()
    }
}
impl core::cmp::Ord for Slot {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self as *const Self).cmp(&(other as *const Self))
    }
}
impl core::cmp::PartialOrd for Slot {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl core::cmp::Eq for Slot {}
impl core::cmp::PartialEq for Slot {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self, other)
    }
}
impl Slot {
    const fn full_size(&self) -> usize {
        core::mem::size_of::<Self>() + self.size
    }
    const fn start(&self) -> NonNull<Slot> {
        self.start
    }
    const fn end(&self) -> NonNull<Slot> {
        unsafe {
            NonNull::new_unchecked(
                (self as *const Self)
                    .cast::<u8>()
                    .add(self.full_size())
                    .cast_mut()
                    .cast(),
            )
        }
    }
    fn end_mut(&mut self) -> NonNull<Slot> {
        unsafe {
            NonNull::new_unchecked(
                (self as *mut Self)
                    .cast::<u8>()
                    .add(self.full_size())
                    .cast(),
            )
        }
    }
    fn shift(&'static mut self, target_align: usize) -> &'static mut Self {
        let start = self.start().as_ptr().cast::<u8>();
        let align_offset = start.align_offset(target_align) as isize;
        let self_offset = unsafe { (self as *const Self).cast::<u8>().offset_from(start) };
        self.align = target_align;
        if align_offset == self_offset {
            return self;
        }
        self.size = (self.size as isize + self_offset - align_offset) as usize;
        let new_addr = unsafe { start.offset(align_offset) };
        unsafe {
            core::ptr::copy(
                (self as *const Self).cast(),
                new_addr,
                core::mem::size_of::<Self>(),
            );
            &mut *new_addr.cast()
        }
    }
    fn split(self: &mut &'static mut Self, at: usize) -> Option<&'static mut Self> {
        let size = self.size;
        (size > at).then(move || {
            self.size = at - core::mem::size_of::<Self>();
            let start = self.end_mut();
            assert_eq!(
                start
                    .as_ptr()
                    .cast::<u8>()
                    .align_offset(core::mem::align_of::<Slot>()),
                0
            );
            let slot = unsafe { start.cast::<MaybeUninit<Slot>>().as_mut() };
            slot.write(Slot {
                size: size - at,
                lower: None,
                start,
                align: 8,
            })
        })
    }
}

impl FreelistAlloc {
    fn insert(&mut self, mut slot: &'static mut Slot) {
        slot = slot.shift(8);
        let mut head = &mut self.head;
        while let Some(h) = head {
            if *h < slot {
                if h.end_mut() == slot.start() {
                    h.size += slot.full_size();
                    return;
                }
                break;
            }
            head = unsafe {
                core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                    &mut h.lower,
                )
            };
        }
        slot.lower = head.take();
        *head = Some(slot)
    }
    fn take(&mut self, layout: crate::alloc::Layout) -> Option<&'static mut Slot> {
        let req = layout.concat(crate::alloc::Layout::of::<Slot>());
        let slot_owner = self.select_slot(req.size)?;
        let mut slot = slot_owner.take()?;
        let lower = slot.lower.take();
        *slot_owner = slot.split(req.size);
        match slot_owner {
            Some(owner) => owner.lower = lower,
            None => *slot_owner = lower,
        }
        Some(slot)
    }
    fn select_slot(&mut self, size: usize) -> Option<&mut Option<&'static mut Slot>> {
        let mut head = unsafe {
            core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                &mut self.head,
            )
        };
        while let Some(h) = head {
            if h.size < size {
                head = unsafe {
                    core::mem::transmute::<
                        &mut Option<&'static mut Slot>,
                        &mut Option<&'static mut Slot>,
                    >(&mut h.lower)
                };
            } else {
                return Some(head);
            }
        }
        let alloc_end = self.end;
        self.grow_take(alloc_end, size)
    }
    fn grow_take(
        &mut self,
        alloc_end: Option<NonNull<Slot>>,
        mut size: usize,
    ) -> Option<&mut Option<&'static mut Slot>> {
        let start = crate::alloc::allocators::paging::memmap(
            alloc_end.map_or(core::ptr::null(), |p| p.as_ptr().cast()),
            &mut size,
        )?;
        let slot = unsafe { start.cast::<MaybeUninit<Slot>>().as_mut() };
        let slot = slot.write(Slot {
            size: size - core::mem::size_of::<Slot>(),
            lower: None,
            start: start.cast(),
            align: 8,
        });
        self.end = Some(slot.end_mut());
        self.insert(slot);
        Some(&mut self.head)
    }
    fn try_extend(&mut self, slot: &mut &'static mut Slot, new_size: usize) -> Result<(), ()> {
        'a: loop {
            let prev_size = slot.size;
            if prev_size >= new_size {
                return Ok(());
            }
            let alloc_end = self.end;
            let slot_end = slot.end_mut();
            if alloc_end == Some(slot_end) {
                if self.grow_take(alloc_end, new_size - prev_size).is_some() {
                    slot.size = new_size;
                    return Ok(());
                }
            }
            let mut head = unsafe {
                core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                    &mut self.head,
                )
            };
            while let Some(h) = head {
                match h.start().cmp(&slot_end) {
                    core::cmp::Ordering::Less => return Err(()),
                    core::cmp::Ordering::Equal => {
                        let lower = h.lower.take();
                        let extension =
                            unsafe { core::mem::replace(head, lower).unwrap_unchecked() };
                        let extension_size = unsafe {
                            extension
                                .end_mut()
                                .as_ptr()
                                .offset_from(extension.start().as_ptr())
                                as usize
                        };
                        slot.size += extension_size;
                        if let Some(remainder) = slot.split(new_size) {
                            remainder.lower = head.take();
                            *head = Some(remainder);
                        }
                        continue 'a;
                    }
                    core::cmp::Ordering::Greater => {
                        head = unsafe {
                            core::mem::transmute::<
                                &mut Option<&'static mut Slot>,
                                &mut Option<&'static mut Slot>,
                            >(&mut h.lower)
                        };
                    }
                }
            }
            return Err(());
        }
    }
}
