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

use crate::{self as stabby, fatptr::AnonymRefMut};
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

#[rustversion::all(not(nightly), since(1.78.0))]
mod const_vtable_fix {
    mod for_each_size {
        include!(concat!(env!("OUT_DIR"), "/const_sizes.rs"));
        pub(super) use for_each_size;
    }
    use for_each_size::for_each_size;

    use core::mem::ManuallyDrop;
    type MaybeFnPtr = core::mem::MaybeUninit<*const ()>;
    const PTR_SIZE: usize = core::mem::size_of::<MaybeFnPtr>();
    const _: () = assert!(PTR_SIZE == core::mem::size_of::<fn()>());

    use super::IConstConstructor;

    #[rustversion::attr(since(1.81.0), expect(
        clippy::absurd_extreme_comparisons,
        reason = "Should compare to usize::MAX at the end"
    ))]
    pub(super) const fn promote_vtable<'a, Source, Vt>() -> &'a Vt
    where
        Vt: IConstConstructor<'a, Source>,
    {
        let perfect_arr_size = size_of::<Vt>() / PTR_SIZE;
        let slice: &'a [MaybeFnPtr] = 'ret: {
            for_each_size! { SIZE,
                if SIZE >= perfect_arr_size {
                    // WARN: Using the associated const here will
                    // crash the compiler from resource exhaustion
                    break 'ret PromotePtrSlice::<_, Vt, SIZE>::promoted_slice();
                }
            }

            #[cold]
            const fn unreachable() -> ! {
                unreachable!()
            }
            unreachable()
        };
        unsafe { &*slice.as_ptr().cast() }
    }
    struct PromotePtrSlice<'a, Source, Vt, const N: usize> {
        _f: (Source, Vt, &'a ()),
    }
    impl<'a, Source, Vt, const N: usize> PromotePtrSlice<'a, Source, Vt, N>
    where
        Vt: IConstConstructor<'a, Source>,
    {
        /// A const function that fulfills the same purpose as the `const` item,
        /// except that because it is a function, it will not cause an eager evaluation
        /// of the huge array
        const fn promoted_slice() -> &'a [MaybeFnPtr]
        where
            Vt: IConstConstructor<'a, Source>,
        {
            Self::PROMOTED_SLICE
        }

        /// A `const` item to perform the promotion.
        const PROMOTED_SLICE: &'a [MaybeFnPtr] = &unsafe {
            #[repr(C)]
            union EmbedBuf<T, const N: usize> {
                src: ManuallyDrop<T>,
                dst: [MaybeFnPtr; N],
            }

            EmbedBuf::<_, N> {
                src: ManuallyDrop::new(Vt::VTABLE),
            }
            .dst
        };
    }
}
#[rustversion::all(not(nightly), since(1.78.0))]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy {
    /// The vtable.
    const VTABLE: Self;
    /// A reference to the vtable
    const VTABLE_REF: &'a Self = const_vtable_fix::promote_vtable::<_, Self>();
    /// Returns the reference to the vtable
    fn vtable() -> &'a Self {
        Self::VTABLE_REF
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
pub trait CompoundVt<'vt_lt> {
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
#[allow(clippy::needless_lifetimes)]
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
    pub drop: crate::StableLike<unsafe extern "C" fn(AnonymRefMut<'_>), core::num::NonZeroUsize>,
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
            unsafe { self.drop.as_ref_unchecked() }
                as *const unsafe extern "C" fn(AnonymRefMut<'_>),
            unsafe { other.drop.as_ref_unchecked() }
                as *const unsafe extern "C" fn(AnonymRefMut<'_>),
        )
    }
}
unsafe extern "C" fn drop<T>(this: AnonymRefMut<'_>) {
    core::ptr::drop_in_place(unsafe { this.cast::<T>().as_mut() })
}
#[allow(unknown_lints)]
#[allow(clippy::missing_transmute_annotations, clippy::needless_lifetimes)]
impl<'a, T> IConstConstructor<'a, T> for VtDrop {
    impl_vtable_constructor!(
        const VTABLE_REF: &'a VtDrop = &VtDrop {
            drop: unsafe {
                core::mem::transmute(drop::<T> as unsafe extern "C" fn(AnonymRefMut<'_>))
            },
        }; =>
        const VTABLE: VtDrop = VtDrop {
            drop: unsafe {
                core::mem::transmute(drop::<T> as unsafe extern "C" fn(AnonymRefMut<'_>))
            },
        };
    );
}

/// A marker for vtables for types that are `Send`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VtSend<T>(pub T);
impl<'a> CompoundVt<'a> for dyn Send {
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
impl<'a> CompoundVt<'a> for dyn Sync {
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
