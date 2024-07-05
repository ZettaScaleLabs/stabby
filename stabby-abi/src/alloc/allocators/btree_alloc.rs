use core::{
    cell::UnsafeCell, mem::MaybeUninit, ops::DerefMut, ptr::NonNull, sync::atomic::AtomicPtr,
};

use crate::num::NonMaxUsize;

/// A simple btree based allocator.
#[crate::stabby]
#[derive(Clone, Copy, Default)]
pub struct BTreeAlloc {
    inner: [u8; 0],
}
impl core::fmt::Debug for BTreeAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BTreeAlloc")
    }
}
impl BTreeAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}
impl crate::alloc::IAlloc for BTreeAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        allock().expect("Allocator not found").alloc(layout)
    }
    unsafe fn realloc(&mut self, p: *mut (), new_layout: crate::alloc::Layout) -> *mut () {
        allock()
            .expect("Allocator not found")
            .realloc(p, new_layout)
    }
    unsafe fn free(&mut self, p: *mut ()) {
        allock().expect("Allocator not found").free(p);
    }
}

static ALLOC: AtomicPtr<Node> = AtomicPtr::new(core::ptr::null_mut());
#[repr(transparent)]
struct BTreeAllocGuard<'a> {
    root: &'a Node,
}
fn allock<'a>() -> Option<BTreeAllocGuard<'a>> {
    loop {
        let ptr = ALLOC.swap(usize::MAX as *mut _, core::sync::atomic::Ordering::Acquire);
        if ptr as usize == usize::MAX {
            core::hint::spin_loop();
            continue;
        }
        if let Some(root) = unsafe { ptr.as_mut() } {
            return Some(BTreeAllocGuard { root });
        }
        let mut allocated = crate::alloc::allocators::paging::PAGESIZE;
        let mut root = unsafe {
            crate::alloc::allocators::paging::memmap(core::ptr::null(), &mut allocated)?
                .cast::<MaybeUninit<Node>>()
                .as_mut()
        };
        unsafe {
            let start = NonNull::new_unchecked(
                root.as_ptr().cast::<u8>().add(core::mem::size_of::<Node>()),
            );
            let end = root.as_ptr().cast::<u8>().add(allocated);
            let root = root.write(Node(UnsafeCell::new(NodeInner {
                ends: [core::ptr::null(); NODE_SIZE],
                blocks: [None; NODE_SIZE],
            })));
            root.0.get_mut().ends[0] = NonNull::new(end.cast());
            root.0.get_mut().blocks[0] = Some(BlockRest {
                start: start.cast(),
                max_contiguous: end.offset_from(start) as usize,
                children: None,
            });
            return Some(BTreeAllocGuard { root });
        }
    }
}
impl Drop for BTreeAllocGuard<'_> {
    fn drop(&mut self) {
        ALLOC.store(self.root, core::sync::atomic::Ordering::Release)
    }
}

const NODE_SIZE: usize = 8;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct BlockRest {
    start: NonNull<()>,
    children: Option<&'static Node>,
    max_contiguous: usize,
}
struct Block {
    end: NonNull<()>,
    rest: BlockRest,
}
#[repr(C)]
struct NodeInner {
    ends: [Option<NonNull<()>>; NODE_SIZE],
    blocks: [Option<BlockRest>; NODE_SIZE],
}
#[repr(transparent)]
struct Node(UnsafeCell<NodeInner>);

impl crate::alloc::IAlloc for BTreeAllocGuard<'_> {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        let layout = layout.for_alloc();
        todo!()
    }
    unsafe fn free(&mut self, mut ptr: *mut ()) {
        todo!()
    }
}
macro_rules! segfault {
    () => {
        #[cfg(target_family = "unix")]
        unsafe {
            libc::signal(libc::SIGSEGV, libc::SIG_DFL)
        };
        return None;
    };
}
impl Node {
    fn alloc(&self, layout: crate::alloc::Layout, result: &mut *mut ()) -> Option<Block> {
        let inner = unsafe { &mut *self.0.get() };
        for i in 0..NODE_SIZE {
            {
                let Some(block) = &inner.blocks[i] else {
                    break;
                };
                if block.max_contiguous < layout.size {
                    continue;
                }
                if let Some(children) = block.children {
                    let extra_block = children.alloc(layout, result);
                    if result.is_null() {
                        continue;
                    }
                    block.max_contiguous = unsafe { (*children.0.get()).blocks.iter() }
                        .map_while(|x| x.as_ref())
                        .fold(0, |acc, it| acc.max(it.max_contiguous));
                    todo!()
                }
                let misalign = block.start.as_ptr() as usize % layout.align;
                let new_block = if misalign != 0 {
                    // start isn't aligned as we want, let's split from the end to reduce fragmentation
                    todo!()
                } else {
                    let end = unsafe { block.start.as_ptr().add(layout.size) };
                    if core::ptr::eq(end, inner.ends[i]) {
                        // this block is just right, we take it
                        block.max_contiguous = 0;
                        *result = block.start.as_ptr();
                        return Default::default();
                    } else {
                        // start is aligned, let's split from the start to
                        todo!()
                    }
                };
            }
        }
        Default::default()
    }
    fn free(
        &self,
        start: *mut (),
        end: *mut (),
        mut max_contig: usize,
        root: &Self,
        parent: Option<&Self>,
    ) -> Option<&'static Self> {
        let blocks = self.blocks.deref_mut();
        let Some(block_idx) = self.ends.iter().position(|block| block.end > start) else {
            segfault!();
        };
        // SAFETY: Since we found the owner, we know its index to be valid and contain `Some(block)`
        let mut block = unsafe {
            self.blocks
                .get_unchecked_mut(block_idx)
                .as_mut()
                .unwrap_unchecked()
        };
        if let Some(children) = &block.children {
            todo!("Free in children")
        } else if core::ptr::eq(block.start.as_ptr(), start) {
            if block.max_contiguous != 0 {
                segfault!();
            }
            block.max_contiguous = block.len();
            let (block, left, right) = self.merge_around(block_idx);
            unsafe {
                Some(NonMaxUsize::new_unchecked(
                    max_contig.max(block.max_contiguous),
                ))
            }
        } else {
            segfault!();
        }
    }
    fn merge_around(&mut self, index: usize) -> (&mut Block, Option<&mut Node>, Option<&mut Node>) {
        todo!()
    }
}
