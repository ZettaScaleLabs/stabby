use crate as stabby;
use crate::abi::vtable::*;

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
    pub fn downcast<T: IConstConstructor<'a, Vt>>(&self) -> Option<&T>
    where
        Vt: PartialEq,
    {
        (self.vtable == T::VTABLE).then(|| unsafe { self.ptr.as_ref() })
    }
}
#[stabby::stabby]
pub struct Dyn<'a, P: IPtrOwned, Vt: HasDropVt + 'a> {
    pub ptr: core::mem::ManuallyDrop<P>,
    pub vtable: &'a Vt,
    unsend: core::marker::PhantomData<*mut P>,
}

impl<'a, P: IPtrOwned, Vt: HasDropVt + 'a> Dyn<'a, P, Vt> {
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
    pub fn downcast_ref<T: IConstConstructor<'a, Vt>>(&self) -> Option<&T>
    where
        Vt: PartialEq + Copy,
    {
        (self.vtable == T::VTABLE).then(|| unsafe { self.ptr.as_ref() })
    }
    /// Downcasts the mutable reference based on vtable equality.
    /// This implies that this downcast will always yield `None` when attempting to downcast values constructed accross an FFI.
    pub fn downcast_mut<T: IConstConstructor<'a, Vt>>(&mut self) -> Option<&mut T>
    where
        Vt: PartialEq + Copy,
        P: IPtrMut,
    {
        (self.vtable == T::VTABLE).then(|| unsafe { self.ptr.as_mut() })
    }
}

impl<'a, Vt: HasDropVt + Copy + 'a, P: IntoDyn> From<P> for Dyn<'a, P::Anonymized, Vt>
where
    P::Anonymized: IPtrOwned,
    P::Target: IConstConstructor<'a, Vt>,
{
    fn from(value: P) -> Self {
        Self {
            ptr: core::mem::ManuallyDrop::new(value.anonimize()),
            vtable: P::Target::VTABLE,
            unsend: core::marker::PhantomData,
        }
    }
}

impl<'a, P: IPtrOwned, Vt: HasDropVt> Drop for Dyn<'a, P, Vt> {
    fn drop(&mut self) {
        P::drop(&mut self.ptr, *self.vtable.drop_vt().drop)
    }
}

impl<'a, T: IConstConstructor<'a, Vt>, Vt: Copy> From<&'a T> for DynRef<'a, Vt> {
    fn from(value: &'a T) -> Self {
        unsafe {
            DynRef {
                ptr: core::mem::transmute(value),
                vtable: T::VTABLE,
                unsend: core::marker::PhantomData,
            }
        }
    }
}

unsafe impl<'a, Vt: HasSendVt> Send for DynRef<'a, Vt> {}
unsafe impl<'a, Vt: HasSyncVt> Sync for DynRef<'a, Vt> {}

unsafe impl<'a, P: IPtrOwned + Send, Vt: HasSendVt + HasDropVt> Send for Dyn<'a, P, Vt> {}
unsafe impl<'a, P: IPtrOwned + Sync, Vt: HasSyncVt + HasDropVt> Sync for Dyn<'a, P, Vt> {}
