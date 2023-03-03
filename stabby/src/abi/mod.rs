use core::ops::*;
pub use typenum::*;

use stabby_macros::holes;

pub mod holes {
    include!(concat!(env!("OUT_DIR"), "/holes.rs"));
}
pub use fatptr::*;
mod fatptr;
pub use istabilize::IStabilize;
mod istabilize;
mod stable_impls;
pub mod vtable;

pub struct AssertStable<T: IStable>(pub core::marker::PhantomData<T>);
impl<T: IStable> AssertStable<T> {
    pub const fn assert() -> Self {
        Self(core::marker::PhantomData)
    }
}

#[repr(C)]
pub struct Tuple2<A, B> {
    _0: A,
    _1: B,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Union<A: Copy, B: Copy> {
    _0: A,
    _1: B,
}
pub use istable::{Array, End, IStable};
mod istable;
pub type NonZeroHole = holes!([1, 0, 0, 0]);
