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

use core::task::Context;

use crate::enums::IDiscriminantProvider;
use crate::option::Option;
use crate::vtable::HasDropVt;
use crate::{IPtrMut, IPtrOwned, IStable};

pub use stable_waker::StableWaker;
#[cfg(unsafe_wakers = "true")]
mod stable_waker {
    use core::task::Waker;

    use crate::StableLike;
    /// A waker that promises to be ABI stable.
    ///
    /// With the `unsafe_wakers` feature enabled, this is actually a lie: the waker is not guaranteed to
    /// be ABI-stable! However, since building ABI-stable wakers that are compatible with Rust's wakers is
    /// costly in terms of runtime, you might want to experiment with unsafe wakers, to bench the cost of
    /// stable wakers if nothing else.
    #[crate::stabby]
    pub struct StableWaker<'a>(StableLike<&'a Waker, &'a ()>);
    impl StableWaker<'_> {
        /// Exposes `self` as a [`core::task::Waker`], letting you use it to run standard futures.
        ///
        /// You generally shouldn't need to do this manually.
        pub fn with_waker<'a, F: FnOnce(&'a Waker) -> U, U>(&'a self, f: F) -> U {
            f(unsafe { self.0.as_ref_unchecked() })
        }
    }
    impl<'a> From<&'a Waker> for StableWaker<'a> {
        fn from(value: &'a Waker) -> Self {
            Self(StableLike::new(value))
        }
    }
}
#[cfg(any(not(unsafe_wakers), unsafe_wakers = "false"))]
mod stable_waker {
    use core::{
        mem::ManuallyDrop,
        ptr::NonNull,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    use crate::alloc::{sync::Arc, AllocPtr, IAlloc};
    use crate::StableLike;

    mod seal {
        use super::*;
        use crate::Tuple as Tuple2;
        #[crate::stabby]
        pub struct StableWakerInner {
            pub waker: StableLike<ManuallyDrop<Waker>, Tuple2<*const (), &'static ()>>,
            pub wake_by_ref: StableLike<for<'b> unsafe extern "C" fn(&'b Waker), &'static ()>,
            pub drop: StableLike<unsafe extern "C" fn(&mut ManuallyDrop<Waker>), &'static ()>,
        }
        unsafe impl Send for StableWakerInner {}
        unsafe impl Sync for StableWakerInner {}
        impl Drop for StableWakerInner {
            fn drop(&mut self) {
                unsafe { (self.drop.as_mut_unchecked())(self.waker.as_mut_unchecked()) }
            }
        }
        #[crate::stabby]
        pub struct SharedStableWaker<Alloc: IAlloc>(pub Arc<StableWakerInner, Alloc>);
        impl<Alloc: IAlloc> SharedStableWaker<Alloc> {
            unsafe fn clone(this: *const ()) -> RawWaker {
                Arc::<_, Alloc>::increment_strong_count(this);
                RawWaker::new(this, &Self::VTABLE)
            }
            unsafe fn drop(this: *const ()) {
                let this = AllocPtr {
                    ptr: NonNull::new(this as *mut _).unwrap(),
                    marker: core::marker::PhantomData,
                };
                let this = Arc::from_raw(this);
                Self(this);
            }
            unsafe fn wake(this: *const ()) {
                Self::wake_by_ref(this);
                Self::drop(this);
            }
            unsafe fn wake_by_ref(this: *const ()) {
                let this = unsafe { &*(this as *const StableWakerInner) };
                (this.wake_by_ref.as_ref_unchecked())(this.waker.as_ref_unchecked())
            }
            const VTABLE: RawWakerVTable =
                RawWakerVTable::new(Self::clone, Self::wake, Self::wake_by_ref, Self::drop);
            pub fn into_raw_waker(self) -> RawWaker {
                RawWaker::new(
                    Arc::into_raw(self.0).ptr.as_ptr() as *const _,
                    &Self::VTABLE,
                )
            }
        }
    }
    use seal::*;
    /// An ABI-stable waker.
    ///
    /// This is done by wrapping a provided [`core::task::Waker`] with calling convention-shims.
    ///
    /// While this is the only way to guarantee ABI-stability when interacting with futures, this does add
    /// a layer of indirection, and cloning this waker will cause an allocation. To bench the performance cost
    /// of this wrapper and decide if you want to risk ABI-unstability on wakers, you may use `RUST_FLAGS='--cfg unsafe_wakers="true"'`, which will turn [`StableWaker`] into a newtype of [`core::task::Waker`].
    #[crate::stabby]
    pub struct StableWaker<'a, Alloc: IAlloc = crate::alloc::DefaultAllocator> {
        waker: StableLike<&'a Waker, &'a ()>,
        #[allow(improper_ctypes_definitions)]
        clone: unsafe extern "C" fn(StableLike<&'a Waker, &'a ()>) -> SharedStableWaker<Alloc>,
        wake_by_ref: StableLike<unsafe extern "C" fn(&Waker), &'a ()>,
    }
    impl<'a, Alloc: IAlloc + Default> StableWaker<'a, Alloc> {
        /// Turns this into a waker whose clone implementation is to clone the underlying waker into a stable Arc.
        pub fn with_waker<F: FnOnce(&Waker) -> U, U>(&self, f: F) -> U {
            const VTABLE: RawWakerVTable =
                RawWakerVTable::new(clone, wake_by_ref, wake_by_ref, drop);
            unsafe fn clone(this: *const ()) -> RawWaker {
                let this = unsafe { &*(this as *const StableWaker) };
                (this.clone)(this.waker).into_raw_waker()
            }
            unsafe fn wake_by_ref(this: *const ()) {
                let this = unsafe { &*(this as *const StableWaker) };
                (this.wake_by_ref.as_ref_unchecked())(this.waker.as_ref_unchecked())
            }
            const unsafe fn drop(_: *const ()) {}
            let waker = RawWaker::new(self as *const Self as *const _, &VTABLE);
            let waker = unsafe { Waker::from_raw(waker) };
            f(&waker)
        }
        unsafe extern "C" fn waker_drop(waker: &mut ManuallyDrop<Waker>) {
            ManuallyDrop::drop(waker)
        }
        unsafe extern "C" fn waker_wake_by_ref(waker: &Waker) {
            waker.wake_by_ref()
        }
        unsafe extern "C" fn clone_borrowed(
            borrowed_waker: StableLike<&Waker, &()>,
        ) -> SharedStableWaker<Alloc> {
            let borrowed_waker = *borrowed_waker.as_ref_unchecked();
            let waker = borrowed_waker.clone();
            let waker = StableWakerInner {
                waker: StableLike::new(ManuallyDrop::new(waker)),
                wake_by_ref: StableLike::new(Self::waker_wake_by_ref),
                drop: StableLike::new(Self::waker_drop),
            };
            let shared = Arc::new_in(waker, Default::default());
            SharedStableWaker(shared)
        }
    }
    impl<'a, Alloc: IAlloc + Default> From<&'a Waker> for StableWaker<'a, Alloc> {
        fn from(value: &'a Waker) -> Self {
            StableWaker {
                waker: StableLike::new(value),
                clone: Self::clone_borrowed,
                wake_by_ref: StableLike::new(Self::waker_wake_by_ref),
            }
        }
    }
}

/// [`core::future::Future`], but ABI-stable.
#[crate::stabby]
pub trait Future {
    /// The output type of the future.
    type Output: IDiscriminantProvider<()>;
    /// Equivalent to [`core::fututre::Future::poll`].
    extern "C" fn poll<'a>(&'a mut self, waker: StableWaker<'a>) -> Option<Self::Output>;
}
impl<T: core::future::Future> Future for T
where
    T::Output: IDiscriminantProvider<()>,
{
    type Output = T::Output;
    #[allow(improper_ctypes_definitions)]
    extern "C" fn poll(&mut self, waker: StableWaker) -> Option<Self::Output> {
        waker.with_waker(|waker| {
            match core::future::Future::poll(
                unsafe { core::pin::Pin::new_unchecked(self) },
                &mut Context::from_waker(waker),
            ) {
                core::task::Poll::Ready(v) => Option::Some(v),
                core::task::Poll::Pending => Option::None(),
            }
        })
    }
}

impl<'a, Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDiscriminantProvider<()>>
    core::future::Future
    for crate::Dyn<'a, P, crate::vtable::VTable<StabbyVtableFuture<Output>, Vt>>
{
    type Output = Output;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        unsafe {
            let this = core::pin::Pin::get_unchecked_mut(self);
            (this.vtable().head.poll.as_ref_unchecked())(this.ptr_mut().as_mut(), cx.waker().into())
                .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}

impl<'a, Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDiscriminantProvider<()>>
    core::future::Future
    for crate::Dyn<
        'a,
        P,
        crate::vtable::VtSend<crate::vtable::VTable<StabbyVtableFuture<Output>, Vt>>,
    >
{
    type Output = Output;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        unsafe {
            let this = core::pin::Pin::get_unchecked_mut(self);
            (this.vtable().0.head.poll.as_ref_unchecked())(
                this.ptr_mut().as_mut(),
                cx.waker().into(),
            )
            .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}

impl<'a, Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDiscriminantProvider<()>>
    core::future::Future
    for crate::Dyn<
        'a,
        P,
        crate::vtable::VtSync<crate::vtable::VTable<StabbyVtableFuture<Output>, Vt>>,
    >
{
    type Output = Output;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        unsafe {
            let this = core::pin::Pin::get_unchecked_mut(self);
            (this.vtable().0.head.poll.as_ref_unchecked())(
                this.ptr_mut().as_mut(),
                cx.waker().into(),
            )
            .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}

impl<'a, Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDiscriminantProvider<()>>
    core::future::Future
    for crate::Dyn<
        'a,
        P,
        crate::vtable::VtSync<
            crate::vtable::VtSend<crate::vtable::VTable<StabbyVtableFuture<Output>, Vt>>,
        >,
    >
{
    type Output = Output;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        unsafe {
            let this = core::pin::Pin::get_unchecked_mut(self);
            (this.vtable().0 .0.head.poll.as_ref_unchecked())(
                this.ptr_mut().as_mut(),
                cx.waker().into(),
            )
            .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}

impl<'a, Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDiscriminantProvider<()>>
    core::future::Future
    for crate::Dyn<
        'a,
        P,
        crate::vtable::VtSend<
            crate::vtable::VtSync<crate::vtable::VTable<StabbyVtableFuture<Output>, Vt>>,
        >,
    >
{
    type Output = Output;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        unsafe {
            let this = core::pin::Pin::get_unchecked_mut(self);
            (this.vtable().0 .0.head.poll.as_ref_unchecked())(
                this.ptr_mut().as_mut(),
                cx.waker().into(),
            )
            .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}

impl<Output> crate::vtable::CompoundVt for dyn core::future::Future<Output = Output>
where
    dyn Future<Output = Output>: crate::vtable::CompoundVt,
{
    type Vt<T> = <dyn Future<Output = Output> as crate::vtable::CompoundVt>::Vt<T>;
}

/// A future that may have already been resolved from the moment it was constructed.
#[crate::stabby]
pub enum MaybeResolved<T, F>
where
    F: core::future::Future<Output = T>,
{
    /// The future was resolved from its construction.
    Resolved(T),
    /// The future has been consumed already.
    Empty,
    /// The future is yet to be resolved.
    Pending(F),
}
impl<T: IStable + Unpin, F: IStable + core::future::Future<Output = T> + Unpin> core::future::Future
    for MaybeResolved<T, F>
where
    F: crate::IStable,
    crate::Result<(), F>: crate::IStable,
    T: crate::IDiscriminantProvider<crate::Result<(), F>>,
    (): crate::IDiscriminantProvider<F>,
{
    type Output = T;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.get_mut();
        let inner = this as *mut _;
        this.match_mut(
            |value| {
                let value = unsafe {
                    let value = core::ptr::read(value);
                    core::ptr::write(inner, MaybeResolved::Empty());
                    value
                };
                core::task::Poll::Ready(value)
            },
            || core::task::Poll::Pending,
            |future| match core::future::Future::poll(core::pin::Pin::new(future), cx) {
                core::task::Poll::Ready(value) => {
                    unsafe {
                        core::ptr::replace(inner, MaybeResolved::Empty());
                    }
                    core::task::Poll::Ready(value)
                }
                core::task::Poll::Pending => core::task::Poll::Pending,
            },
        )
    }
}
