use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

/// A simple free-list based allocator.
///
///
#[crate::stabby]
#[derive(Clone, Copy, Default)]
pub struct FreelistAlloc {
    inner: [u8; 0],
}
impl core::fmt::Debug for FreelistAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("FreelistAlloc")
    }
}
impl FreelistAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}
impl crate::alloc::IAlloc for FreelistAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        let layout = layout.for_alloc();
        let crate::alloc::Layout {
            mut size,
            mut align,
        } = layout;
        match ALLOC.lock().take(size, align) {
            Some(slot) => {
                let this = unsafe { (slot as *mut Slot).add(1).cast() };
                unsafe { this.write_bytes(0, size) };
                this
            }
            None => core::ptr::null_mut(),
        }
    }
    unsafe fn realloc(&mut self, p: *mut c_void, new_size: usize) -> *mut c_void {
        let slot = p.cast::<Slot>().sub(1);
        let this = unsafe { &mut *slot };
        let prev_size = this.size;
        let align = this.align;
        let alloc = ALLOC.lock();
        if alloc.try_extend(this, new_size).is_ok() {
            p
        } else {
            let new_ptr = self.alloc(crate::alloc::Layout {
                size: new_size,
                align,
            });
            unsafe { core::ptr::copy_nonoverlapping(p.cast::<u8>(), this.cast(), this.size) };
            new_ptr
        }
    }
    unsafe fn free(&mut self, p: *mut ()) {
        let slot = p.cast::<Slot>().sub(1);
        ALLOC.lock().insert(&mut *slot);
    }
}

#[repr(C)]
struct Slot {
    size: usize,
    lower: Option<&'static mut Slot>,
    padding: usize,
    align: usize,
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
    const fn start(&self) -> *const u8 {
        unsafe { (self as *const Self).cast::<u8>().sub(self.padding) }
    }
    const fn end(&self) -> *const Slot {
        unsafe { (self as *const Self).cast::<u8>().add(self.full_size()) }
    }
    fn shift(&'static mut self, target_align: usize) -> &'static mut Self {
        let required_padding = target_align - core::mem::size_of::<Self>();
        let padding = self.padding;
        if padding == required_padding {
            return self;
        }
        self.size += padding;
        self.align = target_align;
        self.padding = 0;
        let new_addr = unsafe {
            (self as *mut Self)
                .cast::<u8>()
                .offset(padding as isize - required_padding as isize)
        };
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
        (size > at + core::mem::size_of::<Self>()).then(move || {
            self.size = at;
            let slot = unsafe { &mut *(self.end() as *mut MaybeUninit<Slot>) };
            slot.write(Slot {
                size: size - at + core::mem::size_of::<Self>(),
                lower: None,
                padding: 0,
                align: 8,
            })
        })
    }
}

#[repr(C)]
struct Allocator {
    free_list: AtomicPtr<Slot>,
    end: AtomicPtr<Slot>,
}
struct Slots {
    list: Option<&'static mut Slot>,
}
impl Drop for Slots {
    fn drop(&mut self) {
        ALLOC.free_list.store(
            unsafe {
                core::mem::transmute::<Option<&'static mut Slot>, *mut Slot>(self.list.take())
            },
            Ordering::Release,
        );
    }
}
impl Slots {
    fn insert(&mut self, mut slot: &'static mut Slot) {
        slot = slot.shift(core::mem::size_of::<Slot>());
        let mut head = &mut self.list;
        while let Some(h) = head {
            if *h < slot {
                if core::ptr::eq(h.end(), slot.start()) {
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
    fn take(&mut self, size: usize, align: usize) -> Option<&'static mut Slot> {
        let req = size + align;
        let slot_owner = self.select_slot(req)?;
        let mut slot = slot_owner.take()?;
        let lower = slot.lower.take();
        *slot_owner = slot.split(size);
        match slot_owner {
            Some(owner) => owner.lower = lower,
            None => *slot_owner = lower,
        }
        Some(slot)
    }
    fn select_slot(&mut self, size: usize) -> Option<&mut Option<&'static mut Slot>> {
        let mut head = unsafe {
            core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                &mut self.list,
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
        let alloc_end = ALLOC.end.load(Ordering::Relaxed);
        self.grow_take(alloc_end, size)
    }
    fn grow_take(
        &mut self,
        alloc_end: *mut Slot,
        mut size: usize,
    ) -> Option<&mut Option<&'static mut Slot>> {
        let slot = unsafe {
            crate::alloc::allocators::paging::memmap(alloc_end.cast(), &mut size)?
                .cast::<MaybeUninit<Slot>>()
                .as_mut()
        };
        let slot = slot.write(Slot {
            size: size - core::mem::size_of::<Slot>(),
            lower: None,
            padding: 0,
            align: 8,
        });
        ALLOC.end.store(slot.end().cast_mut(), Ordering::Relaxed);
        self.insert(slot);
        Some(&mut self.list)
    }
    fn try_extend(&mut self, slot: &'static mut Slot, new_size: usize) -> Result<(), ()> {
        let alloc_end = ALLOC.end.load(Ordering::Relaxed);
        let prev_size = slot.size;
        if core::ptr::eq(alloc_end, slot.end()) {
            if self.grow_take(alloc_end, new_size - prev_size).is_some() {
                slot.size = new_size;
                return Ok(());
            }
        }
        let mut head = unsafe {
            core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                &mut self.list,
            )
        };
        while let Some(h) = head {
            match h.start().cmp(&slot.end()) {
                core::cmp::Ordering::Less => return Err(()),
                core::cmp::Ordering::Equal => {
                    let extension_size = unsafe { h.end().offset_from(h.start()) };
                    if prev_size + extension_size >= new_size {
                        todo!("just extending the slot may steal too much capacity, yield some back if so")
                    } else if core::ptr::eq(alloc_end, h.end()) {
                        todo!("we might still be able to extend the newly acquired slot")
                    }
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
        Err(())
    }
}
impl Allocator {
    const fn new() -> Self {
        Self {
            free_list: AtomicPtr::new(core::ptr::null_mut()),
            end: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
    fn lock(&self) -> Slots {
        loop {
            let list = self
                .free_list
                .swap(usize::MAX as *mut Slot, Ordering::AcqRel);
            if list as usize != usize::MAX {
                return Slots {
                    list: unsafe { list.as_mut() },
                };
            }
            core::hint::spin_loop();
        }
    }
}
