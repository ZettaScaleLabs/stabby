use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::alloc::IAlloc;

/// The arena used by an [`ArenaAlloc`] to allocate data.
#[crate::stabby]
pub struct Arena<const SIZE: usize> {
    start: AtomicUsize,
    size: usize,
    largest_failure: AtomicUsize,
    total_failure: AtomicUsize,
    buffer: UnsafeCell<[MaybeUninit<u8>; SIZE]>,
}

impl<const SIZE: usize> core::fmt::Debug for Arena<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Arena")
            .field("start", &self.start)
            .field("size", &self.size)
            .finish()
    }
}

impl<const SIZE: usize> Arena<SIZE> {
    /// Returns the amount of memory the arena can still provide.
    pub fn remaining_capacity(&self) -> usize {
        (self.size).saturating_sub(self.start.load(Ordering::Relaxed))
    }

    /// Returns the size of the largest allocation attempt to have failed.
    ///
    /// Note that layouts that would have not just exceeded arena-capacity, but
    /// also yielded allocations that spanned beyond pointer-representable memory
    /// will not be tallied.
    pub fn largest_failure(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.largest_failure.load(Ordering::Relaxed))
    }

    /// Returns the the sum of the sizes of allocation attempts that have failed.
    ///
    /// Note that layouts that would have not just exceeded arena-capacity, but
    /// also yielded allocations that spanned beyond pointer-representable memory
    /// will not be tallied.
    pub fn total_failures(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.total_failure.load(Ordering::Relaxed))
    }
}

/// An allocator based on an [`Arena`].
///
/// This type of allocator is useful when performing batched work:
/// - Allocation is performed through a small compare-and-swap loop on an integer.
/// - Deallocation is... a noop: the memory will be released when the corresponding [`Arena`] is destroyed.
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct ArenaAlloc<'a> {
    arena: &'a Arena<0>,
}

impl core::cmp::PartialEq for ArenaAlloc<'_> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(&self.arena, &other.arena)
    }
}
impl core::cmp::Eq for ArenaAlloc<'_> {}

impl core::fmt::Debug for ArenaAlloc<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArenaAlloc")
            .field("arena_addr", &(&self.arena as *const _ as *const u8))
            .field("arena", &self.arena)
            .finish()
    }
}

impl<'a, const SIZE: usize> From<&'a Arena<SIZE>> for ArenaAlloc<'a> {
    fn from(value: &'a Arena<SIZE>) -> Self {
        Self::new(value)
    }
}

impl<'a> ArenaAlloc<'a> {
    /// Constructs a new allocator around `arena`.
    ///
    /// Note that all `ArenaAlloc` constructed from a same arena are identical.
    pub const fn new<const SIZE: usize>(arena: &'a Arena<SIZE>) -> Self {
        Self {
            // SAFETY: `value.size == SIZE` by construction
            arena: unsafe { core::mem::transmute::<&'a Arena<SIZE>, &'a Arena<0>>(arena) },
        }
    }

    /// Reinforces the arena allocator with the passed backup allocator.
    ///
    /// Once the arena runs out, the backup allocator will take over allocation.
    pub const fn backup_with<Reinforcements>(
        self,
        backup_alloc: Reinforcements,
    ) -> ReinforcedArenaAlloc<'a, Reinforcements> {
        ReinforcedArenaAlloc {
            arena_alloc: self,
            backup_alloc,
        }
    }

    /// Returns the amount of memory the arena can still provide.
    pub fn remaining_capacity(&self) -> usize {
        self.arena.remaining_capacity()
    }

    /// Returns the size of the largest allocation attempt to have failed.
    ///
    /// Note that layouts that would have not just exceeded arena-capacity, but
    /// also yielded allocations that spanned beyond pointer-representable memory
    /// will not be tallied.
    pub fn largest_failure(&self) -> Option<NonZeroUsize> {
        self.arena.largest_failure()
    }

    /// Returns the the sum of the sizes of allocation attempts that have failed.
    ///
    /// Note that layouts that would have not just exceeded arena-capacity, but
    /// also yielded allocations that spanned beyond pointer-representable memory
    /// will not be tallied.
    pub fn total_failures(&self) -> Option<NonZeroUsize> {
        self.arena.total_failures()
    }

    /// Returns `true` if the `ptr` came from this allocator's arena
    fn ptr_came_from_arena(&self, ptr: *mut ()) -> bool {
        (ptr as usize)
            .checked_sub(self.arena.buffer.get() as usize)
            .unwrap_or(usize::MAX)
            < self.arena.size
    }
}

impl IAlloc for ArenaAlloc<'_> {
    fn alloc(&mut self, mut layout: crate::alloc::Layout) -> *mut () {
        layout = layout.realign(layout.align.max(8));
        let buffer_start = self.arena.buffer.get().cast::<u8>();
        let Some(buffer_end) = (buffer_start as usize).checked_add(self.arena.size) else {
            return core::ptr::null_mut();
        };
        loop {
            let start = self.arena.start.load(Ordering::Relaxed);
            // SAFETY: `ret` is only used once the following factors are validated:
            // 1. `ret`'s alignment is correct (`layout.next_matching`)
            // 2. `ret`'s allocation doesn't exceed `buffer_end`
            // 3. `ret`'s allocation won't be overlapped with other allocations

            // VALIDATING 1
            let ret = unsafe { layout.next_matching(buffer_start.add(start)) };

            // VALIDATING 2
            let Some(end) = (ret as usize).checked_add(layout.size) else {
                return core::ptr::null_mut();
            };
            if end > buffer_end {
                self.arena
                    .largest_failure
                    .fetch_max(layout.size, Ordering::Relaxed);
                self.arena
                    .total_failure
                    .fetch_add(layout.size, Ordering::Relaxed);
                return core::ptr::null_mut();
            }

            // VALIDATING 3
            if self
                .arena
                .start
                .compare_exchange(start, end, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return ret.cast();
            }
        }
    }

    unsafe fn free(&mut self, _ptr: *mut ()) {}
}

/// An allocator that uses an arena until it runs out of space.
///
/// Once it does, the backup allocator is used instead of the arena.
///
/// This allows the use of arenas in settings where total memory consumption
/// isn't known fully ahead of time, but has a low enough average case that
/// arena allocation provides performance benefits in such cases.
#[crate::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ReinforcedArenaAlloc<'a, Backup> {
    arena_alloc: ArenaAlloc<'a>,
    backup_alloc: Backup,
}

impl<'a, Reinforcements> ReinforcedArenaAlloc<'a, Reinforcements> {
    /// Returns the total amount of memory allocated by the backup allocator
    /// instead of the arena.
    ///
    /// This is technically a lower-bound of how much bigger the arena should've been:
    /// alignment constraint could have forced gaps between allocations had they succeeded,
    /// which are not tallied by the arena allocator's tracking.
    ///
    /// Still, collecting statistics on this value can provide valuable insights as to how much
    /// bigger the underlying arena allocator should have been to support all allocations by
    /// itself.
    ///
    /// Conversely, collecting statistics on [`Arena::remaining_capacity`] before destroying
    /// that Arena can provide insights on how much smaller that [`Arena`] could be and
    /// still support all allocations on its own.
    pub fn total_offloaded(&self) -> Option<NonZeroUsize> {
        self.arena_alloc.total_failures()
    }

    /// The underlying [`ArenaAlloc`].
    pub const fn arena(&self) -> ArenaAlloc<'a> {
        self.arena_alloc
    }

    /// The underlying backup allocator.
    pub const fn backup_alloc(&self) -> &Reinforcements {
        &self.backup_alloc
    }
}

impl<Reinforcements: IAlloc> IAlloc for ReinforcedArenaAlloc<'_, Reinforcements> {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        let ret = self.arena_alloc.alloc(layout);
        if ret.is_null() {
            self.backup_alloc.alloc(layout)
        } else {
            ret
        }
    }

    unsafe fn free(&mut self, ptr: *mut ()) {
        if !self.arena_alloc.ptr_came_from_arena(ptr) {
            // SAFETY: we've validated that `ptr` didn't come from the arena, so it must come from `backup_alloc`
            unsafe {
                self.backup_alloc.free(ptr);
            }
        }
    }
}
