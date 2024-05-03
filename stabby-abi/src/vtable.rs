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
use core::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
};

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
    /// The vtable.
    const VTABLE: Self;
    /// A reference to the vtable
    const VTABLE_REF: &'a Self = &Self::VTABLE;
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        Self::VTABLE_REF
    }
}

#[cfg(feature = "libc")]
#[rustversion::all(since(1.78.0), not(nightly))]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy + core::hash::Hash + core::fmt::Debug {
    /// The vtable.
    const VTABLE: Self;
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        static VTABLES: core::sync::atomic::AtomicPtr<
            crate::alloc::vec::Vec<(
                u64,
                crate::alloc::AllocPtr<*const (), crate::alloc::DefaultAllocator>,
            )>,
        > = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
        use crate::alloc::{boxed::Box, vec::Vec, AllocPtr, DefaultAllocator};
        let vtable = Self::VTABLE;
        #[allow(deprecated)]
        let hash = {
            let mut hasher = core::hash::SipHasher::new();
            vtable.hash(&mut hasher);
            hasher.finish()
        };
        fn insert_vtable(hash: u64, vtable: &[*const ()]) -> AllocPtr<*const (), DefaultAllocator> {
            let mut search_start = 0;
            let mut allocated_vt = None;
            let mut vtables = VTABLES.load(core::sync::atomic::Ordering::SeqCst);
            loop {
                let vts = match unsafe { vtables.as_ref() } {
                    None => [].as_slice(),
                    Some(vts) => match vts[search_start..]
                        .iter()
                        .find_map(|e| (e.0 == hash).then_some(e.1))
                    {
                        Some(vt) => return vt,
                        None => vts,
                    },
                };
                let vt = allocated_vt.unwrap_or_else(|| {
                    let mut ptr =
                        AllocPtr::alloc_array(&mut DefaultAllocator::new(), vtable.len()).unwrap();
                    unsafe {
                        core::ptr::copy_nonoverlapping(vtable.as_ptr(), ptr.as_mut(), vtable.len())
                    };
                    ptr
                });
                allocated_vt = Some(vt);
                let updated = VTABLES.load(core::sync::atomic::Ordering::SeqCst);
                if !core::ptr::eq(vtables, updated) {
                    vtables = updated;
                    continue;
                }
                let mut vec = Vec::with_capacity(vts.len() + 1);
                vec.copy_extend(vts);
                vec.push((hash, vt));
                let mut vec = Box::into_raw(Box::new(vec));
                match VTABLES.compare_exchange(
                    updated,
                    unsafe { vec.as_mut() },
                    core::sync::atomic::Ordering::SeqCst,
                    core::sync::atomic::Ordering::SeqCst,
                ) {
                    Ok(updated) => {
                        if let Some(updated) = NonNull::new(updated) {
                            unsafe {
                                Box::from_raw(AllocPtr {
                                    ptr: updated,
                                    marker: PhantomData::<DefaultAllocator>,
                                })
                            };
                        }
                        return vt;
                    }
                    Err(new_vtables) => {
                        search_start = vts.len();
                        vtables = new_vtables;
                        unsafe { Box::from_raw(vec) };
                    }
                }
            }
        }
        unsafe {
            let vtable = core::slice::from_raw_parts(
                (&vtable as *const Self).cast(),
                core::mem::size_of::<Self>() / core::mem::size_of::<*const ()>(),
            );
            let vt = insert_vtable(hash, vtable).cast().as_ref();
            debug_assert_eq!(
                core::slice::from_raw_parts(
                    (vt as *const Self).cast::<*const ()>(),
                    core::mem::size_of::<Self>() / core::mem::size_of::<*const ()>(),
                ),
                vtable
            );
            vt
        }
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
    const VTABLE: VTable<Head, Tail> = VTable {
        head: Head::VTABLE,
        tail: Tail::VTABLE,
    };
}
impl<'a, T> IConstConstructor<'a, T> for () {
    const VTABLE: () = ();
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
impl<'a, T> IConstConstructor<'a, T> for VtDrop {
    const VTABLE: VtDrop = VtDrop {
        drop: unsafe {
            core::mem::transmute({
                unsafe extern "C" fn drop<T>(this: &mut T) {
                    core::ptr::drop_in_place(this)
                }
                drop::<T>
            } as unsafe extern "C" fn(&mut T))
        },
    };
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
    const VTABLE: VtSend<Vt> = VtSend(Vt::VTABLE);
}

/// A marker for vtables for types that are `Sync`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VtSync<T>(pub T);
impl CompoundVt for dyn Sync {
    type Vt<T> = VtSync<T>;
}
impl<'a, T: Sync, Vt: IConstConstructor<'a, T>> IConstConstructor<'a, T> for VtSync<Vt> {
    const VTABLE: VtSync<Vt> = VtSync(Vt::VTABLE);
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
