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
use crate::vtable::*;

/// Indicates that `Self` can be used as a pointer in dynptrs.
pub trait IPtr {
    /// # Safety
    /// This function implies an implicit cast of the reference
    unsafe fn as_ref<U: Sized>(&self) -> &U;
}
/// Indicates that `Self` can be used as an unconditionally mutable pointer in dynptrs.
pub trait IPtrMut: IPtr {
    /// # Safety
    /// This function implies an implicit cast of the reference
    unsafe fn as_mut<U: Sized>(&mut self) -> &mut U;
}
/// Indicates that `Self` can be used as a conditionally mutable pointer in dynptrs.
pub trait IPtrTryAsMut {
    /// # Safety
    /// This function implies an implicit cast of the reference
    unsafe fn try_as_mut<U: Sized>(&mut self) -> Option<&mut U>;
}
impl<T: IPtrMut> IPtrTryAsMut for T {
    unsafe fn try_as_mut<U>(&mut self) -> Option<&mut U> {
        Some(self.as_mut())
    }
}
/// Provides drop support in dynptr for pointers that have at least partial ownership of their pointee.
///
/// `drop` is the drop function of the pointee.
pub trait IPtrOwned {
    /// Called instead of `Drop::drop` when the dynptr is dropped.
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ()));
}
/// Provides Clone support for smart pointers that allow it.
pub trait IPtrClone: IPtrOwned {
    /// Clone the pointer.
    fn clone(this: &Self) -> Self;
}
impl<T> IPtr for &T {
    unsafe fn as_ref<U>(&self) -> &U {
        core::mem::transmute(self)
    }
}
impl<T> IPtr for &mut T {
    unsafe fn as_ref<U>(&self) -> &U {
        core::mem::transmute(self)
    }
}
impl<T> IPtrMut for &mut T {
    unsafe fn as_mut<U>(&mut self) -> &mut U {
        core::mem::transmute(self)
    }
}
impl<T> IPtrOwned for &mut T {
    fn drop(_: &mut core::mem::ManuallyDrop<Self>, _: unsafe extern "C" fn(&mut ())) {}
}

/// Used to turn a pointer into a dynamic pointer.
pub trait IntoDyn {
    /// A pointer type with type information discarded.
    type Anonymized;
    /// The original pointee.
    type Target;
    /// Drop the type information from the pointer.
    fn anonimize(self) -> Self::Anonymized;
}
impl<'a, T> IntoDyn for &'a T {
    type Anonymized = &'a ();
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        unsafe { core::mem::transmute(self) }
    }
}
impl<'a, T> IntoDyn for &'a mut T {
    type Anonymized = &'a mut ();
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        unsafe { core::mem::transmute(self) }
    }
}

#[stabby::stabby]
#[derive(Clone, Copy)]
/// A stable `&'a dyn Traits`
pub struct DynRef<'a, Vt: 'static> {
    ptr: &'a (),
    vtable: &'a Vt,
    unsend: core::marker::PhantomData<*mut ()>,
}

impl<'a, Vt: Copy + 'a> DynRef<'a, Vt> {
    /// Access the data pointer.
    pub const fn ptr(&self) -> &() {
        self.ptr
    }
    /// Access the vtable.
    pub const fn vtable(&self) -> &Vt {
        self.vtable
    }

    /// Allows casting a `dyn A + B` into `dyn A`.
    ///
    /// Note that you can only remove the outermost (rightmost in dyn syntax) trait at a time,
    /// except `Send` and `Sync` that may both be kept, or both be removed.
    pub fn into_super<Super>(self) -> Super
    where
        Self: IntoSuperTrait<Super>,
    {
        IntoSuperTrait::into_super(self)
    }
    /// Downcasts the reference based on vtable equality.
    ///
    /// This implies that this downcast will always yield `None` when attempting to downcast
    /// values constructed accross an FFI.
    ///
    /// # Safety
    /// This may have false positives if all of the following applies:
    /// - `self` was built from `&U`, within the same FFI-boundary,
    /// - `T` and `U` have identical implementations for all methods of the vtable,
    /// - the compiler chose to merge these implementations, making `T` and `U` share
    ///   their function pointers.
    ///
    /// While all of these factors aligning is unlikely, you should be aware of this if you
    /// plan on using methods of `T` that wouldn't be valid for `U`.
    pub unsafe fn downcast<T>(&self) -> Option<&T>
    where
        Vt: PartialEq + IConstConstructor<'a, T>,
    {
        (self.vtable == Vt::vtable()).then(|| unsafe { self.ptr.as_ref() })
    }
    /// Downcasts the reference based on its reflection report.
    pub fn stable_downcast<T: crate::IStable, Path>(&self) -> Option<&T>
    where
        Vt: TransitiveDeref<crate::vtable::StabbyVtableAny, Path>,
    {
        (self.report() == T::REPORT).then(|| unsafe { self.ptr.as_ref() })
    }
}
#[stabby::stabby]
/// A stable trait object (or a stable `&mut dyn`)
pub struct Dyn<'a, P: IPtrOwned + 'a, Vt: HasDropVt + 'static> {
    pub(crate) ptr: core::mem::ManuallyDrop<P>,
    pub(crate) vtable: &'static Vt,
    pub(crate) unsend: core::marker::PhantomData<&'a P>,
}

/// Allows casting a `dyn A + B` into `dyn A`.
pub trait IntoSuperTrait<Super> {
    /// Cast `dyn A + B` into `dyn A`.
    fn into_super(this: Self) -> Super;
}
macro_rules! impl_super {
    ($from: ty, $to: ty, $($generics: tt)*) => {
        impl<'a, P: IPtrOwned + 'a + Sized, $($generics)*> IntoSuperTrait<Dyn<'a, P, $to>> for Dyn<'a, P, $from>
        {
            fn into_super(this: Self) -> Dyn<'a, P, $to> {
                let ptr = &this as *const _;
                core::mem::forget(this);
                unsafe { core::ptr::read(ptr as *const _) }
            }
        }
        impl<'a,  $($generics)*> IntoSuperTrait<DynRef<'a, $to>> for DynRef<'a, $from>
        {
            fn into_super(this: Self) -> DynRef<'a, $to> {
                let ptr = &this as *const _;
                unsafe { core::ptr::read(ptr as *const _) }
            }
        }
    };
}
impl_super!(VTable<Head, Tail>, Tail, Head, Tail: HasDropVt + 'static);
impl_super!(VtSend<Vt>, Vt, Vt: HasDropVt + 'static);
impl_super!(VtSync<Vt>, Vt, Vt: HasDropVt + 'static);
impl_super!(VtSync<VtSend<Vt>>, Vt, Vt: HasDropVt + 'static);
impl_super!(VtSend<VtSync<Vt>>, Vt, Vt: HasDropVt + 'static);
impl_super!(VtSync<VtSend<Vt>>, VtSync<Vt>, Vt: HasDropVt + 'static);
impl_super!(VtSend<VtSync<Vt>>, VtSend<Vt>, Vt: HasDropVt + 'static);
impl_super!(VtSend<VTable<Head, Tail>>, Tail, Head, Tail: HasDropVt + 'static);
impl_super!(VtSync<VTable<Head, Tail>>, Tail, Head, Tail: HasDropVt + 'static);
impl_super!(VtSync<VtSend<VTable<Head, Tail>>>, Tail, Head, Tail: HasDropVt + 'static);
impl_super!(VtSend<VtSync<VTable<Head, Tail>>>, Tail, Head, Tail: HasDropVt + 'static);
impl_super!(VtSend<VTable<Head, Tail>>, VtSend<Tail>, Head, Tail: HasDropVt + 'static);
impl_super!(VtSync<VTable<Head, Tail>>, VtSync<Tail>, Head, Tail: HasDropVt + 'static);
impl_super!(VtSync<VtSend<VTable<Head, Tail>>>, VtSync<VtSend<Tail>>, Head, Tail: HasDropVt + 'static);
impl_super!(VtSend<VtSync<VTable<Head, Tail>>>, VtSend<VtSync<Tail>>, Head, Tail: HasDropVt + 'static);

impl<'a, P: IPtrOwned + IPtrClone, Vt: HasDropVt + 'a> Clone for Dyn<'a, P, Vt> {
    fn clone(&self) -> Self {
        Self {
            ptr: core::mem::ManuallyDrop::new(IPtrClone::clone(&self.ptr)),
            vtable: self.vtable,
            unsend: self.unsend,
        }
    }
}
impl<'a, P: IPtrOwned + IPtr, Vt: HasDropVt + 'a> Dyn<'a, P, Vt> {
    /// Access the data pointer immutably.
    pub fn ptr(&self) -> &P {
        &self.ptr
    }
    /// Access the data pointer mutably.
    pub fn ptr_mut(&mut self) -> &mut P {
        &mut self.ptr
    }
    /// Access the vtable.
    pub const fn vtable(&self) -> &'a Vt {
        self.vtable
    }
    /// Borrow into an ABI-stable `&dyn Traits`
    pub fn as_ref(&self) -> DynRef<'_, Vt> {
        DynRef {
            ptr: unsafe { self.ptr.as_ref() },
            vtable: self.vtable,
            unsend: core::marker::PhantomData,
        }
    }
    /// Borrow into an ABI-stable `&mut dyn Traits`
    pub fn as_mut(&mut self) -> Dyn<&mut (), Vt>
    where
        P: IPtrMut,
    {
        Dyn {
            ptr: unsafe { core::mem::ManuallyDrop::new(self.ptr.as_mut()) },
            vtable: self.vtable,
            unsend: core::marker::PhantomData,
        }
    }
    /// Attempt to borrow into an ABI-stable `&mut dyn Traits`
    pub fn try_as_mut(&mut self) -> Option<Dyn<&mut (), Vt>>
    where
        P: IPtrTryAsMut,
    {
        Some(Dyn {
            ptr: unsafe { core::mem::ManuallyDrop::new(self.ptr.try_as_mut()?) },
            vtable: self.vtable,
            unsend: core::marker::PhantomData,
        })
    }

    /// Allows casting a `dyn A + B` into `dyn A`.
    ///
    /// Note that you can only remove the outermost (rightmost in dyn syntax) trait at a time,
    /// except `Send` and `Sync` that may both be kept, or both be removed.
    pub fn into_super<Super>(self) -> Super
    where
        Self: IntoSuperTrait<Super>,
    {
        IntoSuperTrait::into_super(self)
    }

    /// Downcasts the reference based on vtable equality.
    ///
    /// This implies that this downcast will always yield `None` when attempting to downcast
    /// values constructed accross an FFI.
    ///
    /// Note that the compiler may chose to have multiple copies of the vtable, notably in optimized builds.
    /// This means that even within a same compile unit, this function may fail to downcast a value even if
    /// the type should have matched.
    ///
    /// In general, you should prefer [`Self::stable_downcast_ref`]
    ///
    /// # Safety
    /// This may have false positives if all of the following applies:
    /// - `self` was built from `&U`, within the same FFI-boundary,
    /// - `T` and `U` have identical implementations for all methods of the vtable,
    /// - the compiler chose to merge these implementations, making `T` and `U` share
    ///   their function pointers.
    ///
    /// While all of these factors aligning is unlikely, you should be aware of this if you
    /// plan on using methods of `T` that wouldn't be valid for `U`.
    pub unsafe fn downcast_ref<T>(&self) -> Option<&T>
    where
        Vt: PartialEq + Copy + IConstConstructor<'a, T>,
    {
        (self.vtable == Vt::vtable()).then(|| unsafe { self.ptr.as_ref() })
    }
    /// Downcasts the reference based on its reflection report.
    pub fn stable_downcast_ref<T: crate::IStable, Path>(&self) -> Option<&T>
    where
        Vt: TransitiveDeref<crate::vtable::StabbyVtableAny, Path> + IConstConstructor<'a, T>,
    {
        (self.id() == T::ID && self.report() == T::REPORT).then(|| unsafe { self.ptr.as_ref() })
    }
    /// Downcasts the mutable reference based on vtable equality.
    ///
    /// This implies that this downcast will always yield `None` when attempting to downcast
    /// values constructed accross an FFI.
    ///
    /// Note that the compiler may chose to have multiple copies of the vtable, notably in optimized builds.
    /// This means that even within a same compile unit, this function may fail to downcast a value even if
    /// the type should have matched.
    ///
    /// In general, you should prefer [`Self::stable_downcast_mut`]
    ///
    /// # Safety
    /// This may have false positives if all of the following applies:
    /// - `self` was built from `&U`, within the same FFI-boundary,
    /// - `T` and `U` have identical implementations for all methods of the vtable,
    /// - the compiler chose to merge these implementations, making `T` and `U` share
    ///   their function pointers.
    ///
    /// While all of these factors aligning is unlikely, you should be aware of this if you
    /// plan on using methods of `T` that wouldn't be valid for `U`.
    pub unsafe fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        Vt: PartialEq + Copy + IConstConstructor<'a, T>,
        P: IPtrMut,
    {
        (self.vtable == Vt::vtable()).then(|| unsafe { self.ptr.as_mut() })
    }
    /// Downcasts the mutable reference based on its reflection report.
    pub fn stable_downcast_mut<T: crate::IStable, Path>(&mut self) -> Option<&mut T>
    where
        Vt: TransitiveDeref<crate::vtable::StabbyVtableAny, Path> + IConstConstructor<'a, T>,
        P: IPtrMut,
    {
        (self.id() == T::ID && self.report() == T::REPORT).then(|| unsafe { self.ptr.as_mut() })
    }
}

impl<
        'a,
        Vt: HasDropVt + Copy + IConstConstructor<'static, P::Target> + 'static,
        P: IntoDyn + 'a,
    > From<P> for Dyn<'a, P::Anonymized, Vt>
where
    P::Anonymized: IPtrOwned,
{
    fn from(value: P) -> Self {
        Self {
            ptr: core::mem::ManuallyDrop::new(value.anonimize()),
            vtable: Vt::vtable(),
            unsend: core::marker::PhantomData,
        }
    }
}

impl<P: IPtrOwned, Vt: HasDropVt> Drop for Dyn<'_, P, Vt> {
    fn drop(&mut self) {
        P::drop(&mut self.ptr, *unsafe {
            self.vtable.drop_vt().drop.as_ref_unchecked()
        })
    }
}

impl<'a, T, Vt: Copy + IConstConstructor<'a, T>> From<&'a T> for DynRef<'a, Vt> {
    fn from(value: &'a T) -> Self {
        unsafe {
            DynRef {
                ptr: core::mem::transmute::<&'a T, &'a ()>(value),
                vtable: Vt::vtable(),
                unsend: core::marker::PhantomData,
            }
        }
    }
}

// SAFETY: This is analogous to a reference, and Vt proves the rest
unsafe impl<Vt: HasSendVt> Send for DynRef<'_, Vt> {}
// SAFETY: This is analogous to a reference, and Vt proves the rest
unsafe impl<Vt: HasSyncVt> Sync for DynRef<'_, Vt> {}

// SAFETY: The pointer must be `Send` and the pointee must me `Send` and `Sync`.
unsafe impl<P: IPtrOwned + Send, Vt: HasSendVt + HasDropVt> Send for Dyn<'_, P, Vt> {}
// SAFETY: The pointer must be `Sync` and the pointee must me `Send` and `Sync`.
unsafe impl<P: IPtrOwned + Sync, Vt: HasSyncVt + HasDropVt> Sync for Dyn<'_, P, Vt> {}
