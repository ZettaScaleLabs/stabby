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

use crate as stabby;

#[rustversion::nightly]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy + core::marker::Freeze {
    /// The vtable.
    const VTABLE: &'a Self;
}
#[rustversion::not(nightly)]
/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Source>: 'a + Copy {
    /// The vtable.
    const VTABLE: &'a Self;
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
    const VTABLE: &'a VTable<Head, Tail> = &VTable {
        head: *Head::VTABLE,
        tail: *Tail::VTABLE,
    };
}
impl<'a, T> IConstConstructor<'a, T> for () {
    const VTABLE: &'a () = &();
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
#[derive(Clone, Copy)]
pub struct VtDrop {
    /// The [`Drop::drop`] function, shimmed with the C calling convention.
    pub drop: crate::StableLike<unsafe extern "C" fn(&mut ()), core::num::NonZeroUsize>,
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
    const VTABLE: &'a VtDrop = &VtDrop {
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
    const VTABLE: &'a VtSend<Vt> = &VtSend(*Vt::VTABLE);
}

/// A marker for vtables for types that are `Sync`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VtSync<T>(pub T);
impl CompoundVt for dyn Sync {
    type Vt<T> = VtSync<T>;
}
impl<'a, T: Sync, Vt: IConstConstructor<'a, T>> IConstConstructor<'a, T> for VtSync<Vt> {
    const VTABLE: &'a VtSync<Vt> = &VtSync(*Vt::VTABLE);
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
