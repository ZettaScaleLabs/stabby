//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use crate::{self as stabby};
use core::hash::Hash;

#[rustversion::nightly]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy + core::marker::Freeze {
    /// The vtable.
    const VTABLE: Self;
    /// A reference to the vtable
    const VTABLE_REF: &'a Self = &Self::VTABLE;
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        Self::VTABLE_REF
    }
}

#[rustversion::before(1.78.0)]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy {
    /// A reference to the vtable
    const VTABLE_REF: &'a Self;
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        Self::VTABLE_REF
    }
}

#[cfg(all(feature = "libc", feature = "test"))]
pub use internal::{VTableRegistry, VtBtree, VtVec};

#[cfg(feature = "libc")]
pub(crate) mod internal {
    use crate::alloc::{boxed::BoxedSlice, collections::arc_btree::AtomicArcBTreeSet};
    use core::ptr::NonNull;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VTable(&'static [*const ()]);
    impl PartialOrd<&[*const ()]> for VTable {
        fn partial_cmp(&self, other: &&[*const ()]) -> Option<core::cmp::Ordering> {
            Some(self.0.cmp(*other))
        }
    }
    impl PartialEq<&[*const ()]> for VTable {
        fn eq(&self, other: &&[*const ()]) -> bool {
            self.0.eq(*other)
        }
    }
    unsafe impl Send for VTable {}
    unsafe impl Sync for VTable {}
    use crate::alloc::{vec::Vec, DefaultAllocator};
    /// A BTree used to store VTables.
    ///
    /// This is an internal API, only publically exposed for benchmarking purposes. It may change arbitrarily without warning.
    pub type VtBtree<const SIZE: usize> = AtomicArcBTreeSet<VTable, false, SIZE>;
    /// A Vec used to store VTables
    ///
    /// This is an internal API, only publically exposed for benchmarking purposes. It may change arbitrarily without warning.
    pub type VtVec =
        crate::alloc::sync::AtomicArc<crate::alloc::vec::Vec<VTable>, DefaultAllocator>;
    /// A registry where VTables can be inserted via interior mutability.
    ///
    /// This is an internal API, only publically exposed for benchmarking purposes. It may change arbitrarily without warning.
    pub trait VTableRegistry {
        /// Inserts a raw vtable in the registry.
        fn insert(&self, vtable: &[*const ()]) -> NonNull<*const ()>;
        /// Inserts a vtable in the registry.
        fn insert_typed<'a, Vt: Copy>(&self, vtable: &Vt) -> &'a Vt {
            unsafe {
                let vtable = core::slice::from_raw_parts(
                    (vtable as *const Vt).cast(),
                    core::mem::size_of::<Vt>() / core::mem::size_of::<*const ()>(),
                );
                let vt = self.insert(vtable).cast().as_ref();
                debug_assert_eq!(
                    core::slice::from_raw_parts(
                        (vt as *const Vt).cast::<*const ()>(),
                        core::mem::size_of::<Vt>() / core::mem::size_of::<*const ()>(),
                    ),
                    vtable
                );
                vt
            }
        }
    }
    impl VTableRegistry for VtVec {
        fn insert(&self, vtable: &[*const ()]) -> NonNull<*const ()> {
            let mut search_start = 0;
            let mut allocated_slice: Option<BoxedSlice<_, DefaultAllocator>> = None;
            let mut vtables = self.load(core::sync::atomic::Ordering::SeqCst);
            loop {
                let vts = match vtables.as_ref() {
                    None => [].as_slice(),
                    Some(vts) => match vts[search_start..].iter().find(|e| **e == vtable) {
                        Some(vt) => {
                            // SAFETY: since `vt.0` is a reference, it is guaranteed to be non-null.
                            return unsafe { NonNull::new_unchecked(vt.0.as_ptr().cast_mut()) };
                        }
                        None => vts,
                    },
                };
                let avt = allocated_slice.unwrap_or_else(|| BoxedSlice::from(vtable));
                // SAFETY::`avt` will be leaked if inserting `vt` succeeds.
                let vt = unsafe { core::mem::transmute::<&[_], &'static [_]>(avt.as_slice()) };
                allocated_slice = Some(avt);
                if let Err(updated) =
                    self.is(vtables.as_ref(), core::sync::atomic::Ordering::SeqCst)
                {
                    vtables = updated;
                    continue;
                }
                let mut vec = Vec::with_capacity(vts.len() + 1);
                vec.copy_extend(vts);
                vec.push(VTable(vt));
                if let Err(updated) =
                    self.is(vtables.as_ref(), core::sync::atomic::Ordering::SeqCst)
                {
                    vtables = updated;
                    continue;
                }
                let vec = Some(crate::alloc::sync::Arc::new(vec));
                match self.compare_exchange(
                    vtables.as_ref(),
                    vec,
                    core::sync::atomic::Ordering::SeqCst,
                    core::sync::atomic::Ordering::SeqCst,
                ) {
                    Ok(_) => {
                        core::mem::forget(allocated_slice);
                        return unsafe { NonNull::new_unchecked(vt.as_ptr().cast_mut()) };
                    }
                    Err(new_vtables) => {
                        search_start = vts.len();
                        vtables = new_vtables;
                    }
                }
            }
        }
    }
    #[allow(unknown_lints)]
    #[allow(clippy::missing_transmute_annotations)]
    impl<const SIZE: usize> VTableRegistry for VtBtree<SIZE> {
        fn insert(&self, vtable: &[*const ()]) -> NonNull<*const ()> {
            let mut ret = None;
            let mut allocated_slice: Option<BoxedSlice<_, DefaultAllocator>> = None;
            self.edit(|tree| {
                let mut tree = tree.clone();
                if let Some(vt) = tree.get(&vtable) {
                    ret = unsafe { Some(NonNull::new_unchecked(vt.0.as_ptr().cast_mut())) };
                    return tree;
                }
                let vt = match &allocated_slice {
                    Some(vt) => unsafe { core::mem::transmute(vt.as_slice()) },
                    None => {
                        let vt = BoxedSlice::from(vtable);
                        let slice = unsafe { core::mem::transmute(vt.as_slice()) };
                        allocated_slice = Some(vt);
                        slice
                    }
                };
                tree.insert(vt);
                tree
            });
            if ret.is_none() {
                let ret = &mut ret;
                self.get(&vtable, move |vt| {
                    let start = unsafe {
                        NonNull::new_unchecked(
                            vt.expect("VTable should've been inserted by now")
                                .0
                                .as_ptr()
                                .cast_mut(),
                        )
                    };
                    if let Some(allocated) = allocated_slice {
                        if allocated.slice.start.ptr == start {
                            core::mem::forget(allocated);
                        }
                    }
                    *ret = Some(start);
                });
            }
            unsafe { ret.unwrap_unchecked() }
        }
    }
    #[cfg(stabby_vtables = "vec")]
    pub(crate) static VTABLES: crate::alloc::sync::AtomicArc<
        crate::alloc::vec::Vec<VTable>,
        DefaultAllocator,
    > = crate::alloc::sync::AtomicArc::new(None);
    #[cfg(any(stabby_vtables = "btree", not(stabby_vtables)))]
    #[rustversion::not(nightly)]
    pub(crate) static VTABLES: AtomicArcBTreeSet<VTable, false, 5> = AtomicArcBTreeSet::new();
    #[rustversion::nightly]
    #[cfg(stabby_vtables = "btree")]
    pub(crate) static VTABLES: AtomicArcBTreeSet<VTable, false, 5> = AtomicArcBTreeSet::new();
}

#[cfg(all(
    feature = "libc",
    any(stabby_vtables = "vec", stabby_vtables = "btree", not(stabby_vtables))
))]
#[rustversion::all(not(nightly), since(1.78.0))]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy {
    /// The vtable.
    const VTABLE: Self;
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        use internal::VTableRegistry;
        internal::VTABLES.insert_typed(&Self::VTABLE)
    }
}
/// Implementation detail for stabby's version of dyn traits.
pub trait TransitiveDeref<Head, N> {
    /// Deref transitiverly.
    fn tderef(&self) -> &Head;
}
/// Implementation detail for stabby's version of dyn traits.
pub struct H;
/// Implementation detail for stabby's version of dyn traits.
pub struct T<T>(T);
/// A recursive type to define sets of v-tables.
/// You should _always_ use `stabby::vtable!(Trait1 + Trait2 + ...)` to generate this type,
/// as this macro will ensure that traits are ordered consistently in the vtable.
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VTable<Head, Tail = VtDrop> {
    /// The rest of the vtable.
    ///
    /// It comes first to allow upcasting vtables.
    pub tail: Tail,
    /// The head of the vtable (the last trait listed in the macros)
    pub head: Head,
}

/// Concatenate vtables
pub trait CompoundVt {
    /// The concatenated vtable.
    type Vt<T>;
}

impl<'a, T, Head: Copy + 'a, Tail: Copy + 'a> IConstConstructor<'a, T> for VTable<Head, Tail>
where
    Head: IConstConstructor<'a, T>,
    Tail: IConstConstructor<'a, T>,
{
    impl_vtable_constructor!(
        const VTABLE_REF: &'a VTable<Head, Tail> = &VTable {
            head: *Head::VTABLE_REF,
            tail: *Tail::VTABLE_REF,
        }; =>
        const VTABLE: VTable<Head, Tail> = VTable {
            head: Head::VTABLE,
            tail: Tail::VTABLE,
        };
    );
}
impl<'a, T> IConstConstructor<'a, T> for () {
    impl_vtable_constructor!(
        const VTABLE_REF: &'a () = &();=>
        const VTABLE: () = (););
}
impl<Head, Tail> TransitiveDeref<Head, H> for VTable<Head, Tail> {
    fn tderef(&self) -> &Head {
        &self.head
    }
}
impl<Head, Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VTable<Head, Tail> {
    fn tderef(&self) -> &Vt {
        self.tail.tderef()
    }
}

/// Allows extracting the dropping vtable from a vtable
pub trait HasDropVt {
    /// Access the [`VtDrop`] section of a vtable.
    fn drop_vt(&self) -> &VtDrop;
}
impl HasDropVt for VtDrop {
    fn drop_vt(&self) -> &VtDrop {
        self
    }
}
impl<Head, Tail: HasDropVt> HasDropVt for VTable<Head, Tail> {
    fn drop_vt(&self) -> &VtDrop {
        self.tail.drop_vt()
    }
}
impl<T: HasDropVt> HasDropVt for VtSend<T> {
    fn drop_vt(&self) -> &VtDrop {
        self.0.drop_vt()
    }
}
impl<T: HasDropVt> HasDropVt for VtSync<T> {
    fn drop_vt(&self) -> &VtDrop {
        self.0.drop_vt()
    }
}

/// Whether or not a vtable includes [`VtSend`]
pub trait HasSendVt {}
impl<T> HasSendVt for VtSend<T> {}
impl<T: HasSendVt> HasSendVt for VtSync<T> {}
impl<Head, Tail: HasSyncVt> HasSendVt for VTable<Head, Tail> {}
/// Whether or not a vtable includes [`VtSync`]
pub trait HasSyncVt {}
impl<T> HasSyncVt for VtSync<T> {}
impl<T: HasSyncVt> HasSyncVt for VtSend<T> {}
impl<Head, Tail: HasSyncVt> HasSyncVt for VTable<Head, Tail> {}

// DROP
/// The vtable to drop a value in place
#[stabby::stabby]
#[derive(Clone, Copy, Eq)]
pub struct VtDrop {
    /// The [`Drop::drop`] function, shimmed with the C calling convention.
    pub drop: crate::StableLike<unsafe extern "C" fn(&mut ()), core::num::NonZeroUsize>,
}
impl core::fmt::Debug for VtDrop {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VtDrop({:p})", unsafe { self.drop.as_ref_unchecked() })
    }
}
impl Hash for VtDrop {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.drop.hash(state)
    }
}
impl PartialEq for VtDrop {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(
            unsafe { self.drop.as_ref_unchecked() } as *const unsafe extern "C" fn(&mut ()),
            unsafe { other.drop.as_ref_unchecked() } as *const unsafe extern "C" fn(&mut ()),
        )
    }
}
unsafe extern "C" fn drop<T>(this: &mut T) {
    core::ptr::drop_in_place(this)
}
#[allow(unknown_lints)]
#[allow(clippy::missing_transmute_annotations)]
impl<'a, T> IConstConstructor<'a, T> for VtDrop {
    impl_vtable_constructor!(
        const VTABLE_REF: &'a VtDrop = &VtDrop {
            drop: unsafe {
                core::mem::transmute(drop::<T> as unsafe extern "C" fn(&mut T))
            },
        }; =>
        const VTABLE: VtDrop = VtDrop {
            drop: unsafe {
                core::mem::transmute(drop::<T> as unsafe extern "C" fn(&mut T))
            },
        };
    );
}

/// A marker for vtables for types that are `Send`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VtSend<T>(pub T);
impl CompoundVt for dyn Send {
    type Vt<T> = VtSend<T>;
}
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, N> for VtSend<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::vtable::VtSend<VTable<Head, Tail>>> for VTable<Head, Tail> {
    fn from(value: VtSend<VTable<Head, Tail>>) -> Self {
        value.0
    }
}
impl<'a, T: Send, Vt: IConstConstructor<'a, T>> IConstConstructor<'a, T> for VtSend<Vt> {
    impl_vtable_constructor!(
        const VTABLE_REF: &'a VtSend<Vt> = &VtSend(*Vt::VTABLE_REF);=>
        const VTABLE: VtSend<Vt> = VtSend(Vt::VTABLE);
    );
}

/// A marker for vtables for types that are `Sync`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VtSync<T>(pub T);
impl CompoundVt for dyn Sync {
    type Vt<T> = VtSync<T>;
}
impl<'a, T: Sync, Vt: IConstConstructor<'a, T>> IConstConstructor<'a, T> for VtSync<Vt> {
    impl_vtable_constructor!(
        const VTABLE_REF: &'a VtSync<Vt> = &VtSync(*Vt::VTABLE_REF);=>
        const VTABLE: VtSync<Vt> = VtSync(Vt::VTABLE);
    );
}
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, N> for VtSync<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::vtable::VtSync<VtSend<VTable<Head, Tail>>>> for VTable<Head, Tail> {
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0 .0
    }
}
impl<Head, Tail> From<crate::vtable::VtSync<VtSend<VTable<Head, Tail>>>>
    for VtSend<VTable<Head, Tail>>
{
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0
    }
}
impl<Head, Tail> From<crate::vtable::VtSync<VTable<Head, Tail>>> for VTable<Head, Tail> {
    fn from(value: VtSync<VTable<Head, Tail>>) -> Self {
        value.0
    }
}

/// An ABI-stable equivalent to [`core::any::Any`]
#[stabby::stabby]
pub trait Any {
    /// The report of the type.
    extern "C" fn report(&self) -> &'static crate::report::TypeReport;
    /// The id of the type.
    extern "C" fn id(&self) -> u64;
}
impl<T: crate::IStable> Any for T {
    extern "C" fn report(&self) -> &'static crate::report::TypeReport {
        Self::REPORT
    }
    extern "C" fn id(&self) -> u64 {
        Self::ID
    }
}
