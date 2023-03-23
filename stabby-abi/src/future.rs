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

use core::task::Context;

use crate::enums::IDiscriminantProvider;
use crate::option::Option;
use crate::vtable::HasDropVt;
use crate::{IPtrMut, IPtrOwned};

pub use stable_waker::StableWaker;
#[cfg(feature = "unsafe_wakers")]
mod stable_waker {
    use core::task::Waker;

    use crate::StableLike;
    #[crate::stabby]
    pub struct StableWaker<'a>(StableLike<&'a Waker, &'a ()>);
    impl StableWaker<'_> {
        pub fn with_waker<'a, F: FnOnce(&'a Waker) -> U, U>(&'a self, f: F) -> U {
            f(self.0.value)
        }
    }
    impl<'a> From<&'a Waker> for StableWaker<'a> {
        fn from(value: &'a Waker) -> Self {
            unsafe { Self(StableLike::stable(value)) }
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
        wake_by_ref: StableLike<for<'b> extern "C" fn(&'b Waker), &'static ()>,
        drop: StableLike<extern "C" fn(&mut ManuallyDrop<Waker>), &'static ()>,
    }
    #[crate::stabby]
    pub struct StableWaker<'a> {
        waker: Arc<StableWakerInner>,
        marker: core::marker::PhantomData<&'a ()>,
    }
    impl StableWaker<'_> {
        fn into_waker(waker: Arc<StableWakerInner>) -> Waker {
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
                (this.wake_by_ref)(&this.waker)
            }
            unsafe fn drop(this: *const ()) {
                Arc::from_raw(this as *const StableWakerInner);
            }
            let waker = RawWaker::new(Arc::into_raw(waker) as *const _, &VTABLE);
            unsafe { Waker::from_raw(waker) }
        }
        pub fn with_waker<F: FnOnce(&Waker) -> U, U>(&self, f: F) -> U {
            let waker = Self::into_waker(self.waker.clone());
            f(&waker)
        }
    }
    impl<'a> From<&'a Waker> for StableWaker<'static> {
        fn from(value: &'a Waker) -> Self {
            extern "C" fn drop_waker(waker: &mut ManuallyDrop<Waker>) {
                unsafe { ManuallyDrop::drop(waker) }
            }
            extern "C" fn wake_by_ref(waker: &Waker) {
                waker.wake_by_ref()
            }
            let waker = unsafe {
                StableWakerInner {
                    waker: StableLike {
                        value: ManuallyDrop::new(value.clone()),
                        marker: core::marker::PhantomData,
                    },
                    wake_by_ref: StableLike::stable(wake_by_ref),
                    drop: StableLike::stable(drop_waker),
                }
            };
            StableWaker {
                waker: Arc::new(waker),
                marker: core::marker::PhantomData,
            }
        }
    }
}

#[crate::stabby]
pub trait Future {
    type Output: IDiscriminantProvider<()>;
    extern "C" fn poll(&mut self, waker: StableWaker) -> Option<Self::Output>;
}
impl<T: core::future::Future> Future for T
where
    T::Output: IDiscriminantProvider<()>,
{
    type Output = T::Output;
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
            (this.vtable().head.poll)(this.ptr_mut().as_mut(), cx.waker().into())
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
            (this.vtable().0.head.poll)(this.ptr_mut().as_mut(), cx.waker().into())
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
            (this.vtable().0.head.poll)(this.ptr_mut().as_mut(), cx.waker().into())
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
            (this.vtable().0 .0.head.poll)(this.ptr_mut().as_mut(), cx.waker().into())
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
            (this.vtable().0 .0.head.poll)(this.ptr_mut().as_mut(), cx.waker().into())
                .match_owned(|v| core::task::Poll::Ready(v), || core::task::Poll::Pending)
        }
    }
}
