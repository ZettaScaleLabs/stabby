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
#[cfg(feature = "unsafe_wakers")]
mod stable_waker {
    use core::task::Waker;

    use crate::StableLike;
    #[crate::stabby]
    pub struct StableWaker<'a>(StableLike<&'a Waker, &'a ()>);
    impl StableWaker<'_> {
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
#[cfg(all(feature = "alloc", not(feature = "unsafe_wakers")))]
mod stable_waker {
    use core::{
        mem::ManuallyDrop,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    use alloc::sync::Arc;

    use crate::StableLike;
    #[crate::stabby]
    pub struct Tuple2<A, B>(A, B);
    #[crate::stabby]
    pub struct StableWakerInner {
        waker: StableLike<ManuallyDrop<Waker>, Tuple2<*const (), &'static ()>>,
        wake_by_ref: StableLike<for<'b> unsafe extern "C" fn(&'b Waker), &'static ()>,
        drop: StableLike<unsafe extern "C" fn(&mut ManuallyDrop<Waker>), &'static ()>,
    }
    unsafe impl Send for StableWakerInner {}
    unsafe impl Sync for StableWakerInner {}
    impl Drop for StableWakerInner {
        fn drop(&mut self) {
            unsafe { (self.drop.as_mut_unchecked())(self.waker.as_mut_unchecked()) }
        }
    }
    #[crate::stabby]
    pub struct SharedStableWaker(Arc<StableWakerInner>);
    impl SharedStableWaker {
        fn into_raw_waker(self) -> RawWaker {
            const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
            unsafe fn clone(this: *const ()) -> RawWaker {
                Arc::increment_strong_count(this);
                RawWaker::new(this, &VTABLE)
            }
            unsafe fn wake(this: *const ()) {
                wake_by_ref(this);
                drop(this);
            }
            unsafe fn wake_by_ref(this: *const ()) {
                let this = unsafe { &*(this as *const StableWakerInner) };
                (this.wake_by_ref.as_ref_unchecked())(this.waker.as_ref_unchecked())
            }
            unsafe fn drop(this: *const ()) {
                Arc::from_raw(this as *const StableWakerInner);
            }
            RawWaker::new(Arc::into_raw(self.0) as *const _, &VTABLE)
        }
    }
    #[crate::stabby]
    pub struct StableWaker<'a> {
        waker: StableLike<&'a Waker, &'a ()>,
        #[allow(improper_ctypes_definitions)]
        clone: unsafe extern "C" fn(StableLike<&'a Waker, &'a ()>) -> SharedStableWaker,
        wake_by_ref: StableLike<unsafe extern "C" fn(&Waker), &'a ()>,
    }
    impl<'a> StableWaker<'a> {
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
            unsafe fn drop(_: *const ()) {}
            let waker = RawWaker::new(self as *const Self as *const _, &VTABLE);
            let waker = unsafe { Waker::from_raw(waker) };
            f(&waker)
        }
    }
    impl<'a> From<&'a Waker> for StableWaker<'a> {
        fn from(value: &'a Waker) -> Self {
            unsafe extern "C" fn drop(waker: &mut ManuallyDrop<Waker>) {
                ManuallyDrop::drop(waker)
            }
            unsafe extern "C" fn wake_by_ref(waker: &Waker) {
                waker.wake_by_ref()
            }
            #[allow(improper_ctypes_definitions)]
            unsafe extern "C" fn clone(
                borrowed_waker: StableLike<&Waker, &()>,
            ) -> SharedStableWaker {
                let waker = (*borrowed_waker.as_ref_unchecked()).clone();
                let waker = StableWakerInner {
                    waker: StableLike::new(ManuallyDrop::new(waker)),
                    wake_by_ref: StableLike::new(wake_by_ref),
                    drop: StableLike::new(drop),
                };
                SharedStableWaker(Arc::new(waker))
            }
            StableWaker {
                waker: StableLike::new(value),
                clone,
                wake_by_ref: StableLike::new(wake_by_ref),
            }
        }
    }
}

#[crate::stabby]
pub trait Future {
    type Output: IDiscriminantProvider<()>;
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

#[crate::stabby]
pub enum MaybeResolved<T, F>
where
    F: core::future::Future<Output = T>,
{
    Resolved(T),
    Empty,
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
