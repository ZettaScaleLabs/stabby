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
    ptr: core::mem::ManuallyDrop<P>,
    vtable: &'a Vt,
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
        P::drop(&mut self.ptr, self.vtable.drop_vt().drop)
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

// MYTRAIT

#[stabby::stabby]
pub trait MyTrait {
    type Output;
    extern "C" fn do_stuff<'a>(&'a self, with: &Self::Output) -> &'a u8;
    extern "C" fn gen_stuff(&mut self) -> Self::Output;
}

pub trait DynMyTrait<N, Output> {
    extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8;
}
impl<Vt: TransitiveDeref<StabbyVtableMyTrait<Output>, N>, Output, N> DynMyTrait<N, Output>
    for DynRef<'_, Vt>
{
    extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8 {
        (self.vtable.tderef().do_stuff)(self.ptr, with)
    }
}
impl<
        'c,
        P: IPtrOwned,
        Vt: HasDropVt + TransitiveDeref<StabbyVtableMyTrait<Output>, N>,
        Output,
        N,
    > DynMyTrait<N, Output> for Dyn<'c, P, Vt>
{
    extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8 {
        (self.vtable.tderef().do_stuff)(unsafe { self.ptr.as_ref() }, with)
    }
}
pub trait DynMutMyTrait<N, Output>: DynMyTrait<N, Output> {
    extern "C" fn gen_stuff(&mut self) -> Output;
}
impl<
        'a,
        P: IPtrOwned + IPtrMut,
        Vt: HasDropVt + TransitiveDeref<StabbyVtableMyTrait<Output>, N>,
        Output,
        N,
    > DynMutMyTrait<N, Output> for Dyn<'a, P, Vt>
{
    extern "C" fn gen_stuff(&mut self) -> Output {
        (self.vtable.tderef().gen_stuff)(unsafe { self.ptr.as_mut() })
    }
}

// IMPL

impl MyTrait for u8 {
    type Output = u8;
    extern "C" fn do_stuff<'a>(&'a self, _: &Self::Output) -> &'a u8 {
        self
    }
    extern "C" fn gen_stuff(&mut self) -> Self::Output {
        *self
    }
}
impl MyTrait for u16 {
    type Output = u8;
    extern "C" fn do_stuff<'a>(&'a self, _: &Self::Output) -> &'a u8 {
        &0
    }
    extern "C" fn gen_stuff(&mut self) -> Self::Output {
        *self as u8
    }
}

// MYTRAIT2
#[stabby::stabby]
pub trait MyTrait2 {
    extern "C" fn do_stuff2(&self) -> u8;
}

// IMPL

impl MyTrait2 for u8 {
    extern "C" fn do_stuff2(&self) -> u8 {
        *self
    }
}
impl MyTrait2 for u16 {
    extern "C" fn do_stuff2(&self) -> u8 {
        (*self) as u8
    }
}

impl MyTrait3<Box<()>> for u8 {
    type A = u8;
    type B = u8;
    extern "C" fn do_stuff<'a>(&'a self, _a: &'a Self::A, _b: Self::B) -> Self::B {
        *self
    }
    extern "C" fn gen_stuff(&mut self, _with: Box<()>) -> Self::A {
        *self
    }
}
impl MyTrait3<Box<()>> for u16 {
    type A = u8;
    type B = u8;
    extern "C" fn do_stuff<'a>(&'a self, _a: &'a Self::A, _b: Self::B) -> Self::B {
        (*self) as u8
    }
    extern "C" fn gen_stuff(&mut self, _with: Box<()>) -> Self::A {
        (*self) as u8
    }
}

#[test]
fn test() {
    let boxed = Box::new(6u8);
    let mut dyned = Dyn::<
        _,
        stabby::vtable!(
            MyTrait2 + MyTrait<Output = u8> + MyTrait3<Box<()>, A = u8, B = u8> + Send + Sync
        ),
    >::from(boxed);
    assert_eq!(dyned.downcast_ref::<u8>(), Some(&6));
    assert_eq!(dyned.do_stuff(&0), &6);
    assert_eq!(dyned.gen_stuff(), 6);
    assert!(dyned.downcast_ref::<u16>().is_none());
}

#[stabby::stabby]
pub trait MyTrait3<Hi: core::ops::Deref> {
    type A;
    type B;
    extern "C" fn do_stuff<'a>(&'a self, a: &'a Self::A, b: Self::B) -> Self::B;
    extern "C" fn gen_stuff(&mut self, with: Hi) -> Self::A;
}
