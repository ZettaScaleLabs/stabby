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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use crate as stabby;
use crate::vtable::*;

pub trait IPtr {
    /// # Safety
    /// This function implies an implicit cast of the reference
    unsafe fn as_ref<U: Sized>(&self) -> &U;
}
pub trait IPtrMut: IPtr {
    /// # Safety
    /// This function implies an implicit cast of the reference
    unsafe fn as_mut<U: Sized>(&mut self) -> &mut U;
}
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
pub trait IPtrOwned: IPtr {
    /// Must return `true` if and only if dropping one instance of
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ()));
}
impl<'a, T> IPtr for &'a T {
    unsafe fn as_ref<U>(&self) -> &U {
        core::mem::transmute(self)
    }
}
impl<'a, T> IPtr for &'a mut T {
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

pub trait IntoDyn {
    type Anonymized;
    type Target;
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
    pub fn ptr(&self) -> &() {
        self.ptr
    }
    pub fn vtable(&self) -> &Vt {
        self.vtable
    }
    /// Downcasts the reference based on vtable equality.
    /// This implies that this downcast will always yield `None` when attempting to downcast values constructed accross an FFI.
    pub fn downcast<T>(&self) -> Option<&T>
    where
        Vt: PartialEq + IConstConstructor<'a, T>,
    {
        (self.vtable == Vt::VTABLE).then(|| unsafe { self.ptr.as_ref() })
    }
}
#[stabby::stabby]
/// A stable trait object (or a stable `&mut dyn`)
pub struct Dyn<'a, P: IPtrOwned + 'a, Vt: HasDropVt + 'static> {
    ptr: core::mem::ManuallyDrop<P>,
    vtable: &'static Vt,
    unsend: core::marker::PhantomData<&'a P>,
}

impl<'a, P: IPtrOwned, Vt: HasDropVt + 'a> Dyn<'a, P, Vt> {
    pub fn ptr(&self) -> &P {
        &self.ptr
    }
    pub fn ptr_mut(&mut self) -> &mut P {
        &mut self.ptr
    }
    pub fn vtable(&self) -> &'a Vt {
        self.vtable
    }
    pub fn as_ref(&self) -> DynRef<'_, Vt> {
        DynRef {
            ptr: unsafe { self.ptr.as_ref() },
            vtable: self.vtable,
            unsend: core::marker::PhantomData,
        }
    }
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

    /// Downcasts the reference based on vtable equality.
    /// This implies that this downcast will always yield `None` when attempting to downcast values constructed accross an FFI.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        Vt: PartialEq + Copy + IConstConstructor<'a, T>,
    {
        (self.vtable == Vt::VTABLE).then(|| unsafe { self.ptr.as_ref() })
    }
    /// Downcasts the mutable reference based on vtable equality.
    /// This implies that this downcast will always yield `None` when attempting to downcast values constructed accross an FFI.
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        Vt: PartialEq + Copy + IConstConstructor<'a, T>,
        P: IPtrMut,
    {
        (self.vtable == Vt::VTABLE).then(|| unsafe { self.ptr.as_mut() })
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
            vtable: Vt::VTABLE,
            unsend: core::marker::PhantomData,
        }
    }
}

impl<'a, P: IPtrOwned, Vt: HasDropVt> Drop for Dyn<'a, P, Vt> {
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
                ptr: core::mem::transmute(value),
                vtable: Vt::VTABLE,
                unsend: core::marker::PhantomData,
            }
        }
    }
}

unsafe impl<'a, Vt: HasSendVt> Send for DynRef<'a, Vt> {}
unsafe impl<'a, Vt: HasSyncVt> Sync for DynRef<'a, Vt> {}

unsafe impl<'a, P: IPtrOwned + Send, Vt: HasSendVt + HasDropVt> Send for Dyn<'a, P, Vt> {}
unsafe impl<'a, P: IPtrOwned + Sync, Vt: HasSyncVt + HasDropVt> Sync for Dyn<'a, P, Vt> {}
