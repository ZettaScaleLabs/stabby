#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod allocs;

pub mod typenum2;
#[doc(hidden)]
pub use typenum2::*;

#[macro_export]
macro_rules! assert_optimal_layout {
    ($t: ty) => {
        const _: () = {
            assert!(<$t>::has_optimal_layout());
        };
    };
}
pub use crate::enums::IDiscriminantProvider;
pub use stabby_macros::stabby;
// pub use crate::Result;

pub use fatptr::*;
mod fatptr;
// pub use istabilize::IStabilize;
// mod istabilize;
pub mod future;
mod stable_impls;
pub mod vtable;

// #[allow(type_alias_bounds)]
// pub type Stable<Source: IStabilize> = Source::Stable;

pub struct AssertStable<T: IStable>(pub core::marker::PhantomData<T>);
impl<T: IStable> AssertStable<T> {
    pub const fn assert() -> Self {
        Self(core::marker::PhantomData)
    }
}

/// Lets you tell `stabby` that `T` has the same stable layout as `As`.
///
/// Lying about this link between `T` and `As` will cause UB.
pub struct StableLike<T, As> {
    pub value: T,
    marker: core::marker::PhantomData<As>,
}
impl<T: Clone, As> Clone for StableLike<T, As> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            marker: self.marker,
        }
    }
}
impl<T: Copy, As> Copy for StableLike<T, As> {}
impl<T, As: IStable> StableLike<T, As> {
    /// # Safety
    /// Refer to type documentation
    pub const unsafe fn stable(value: T) -> Self {
        Self {
            value,
            marker: core::marker::PhantomData,
        }
    }
}

impl<T, As: IStable> core::ops::Deref for StableLike<T, As> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T, As: IStable> core::ops::DerefMut for StableLike<T, As> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
unsafe impl<T, As: IStable> IStable for StableLike<T, As> {
    type Size = As::Size;
    type Align = As::Align;
    type ForbiddenValues = As::ForbiddenValues;
    type UnusedBits = As::UnusedBits;
    type HasExactlyOneNiche = As::HasExactlyOneNiche;
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FieldPair<A, B>(core::marker::PhantomData<(A, B)>);
#[repr(transparent)]
pub struct Struct<T>(T);

#[repr(C)]
pub union Union<A, B> {
    pub ok: core::mem::ManuallyDrop<A>,
    pub err: core::mem::ManuallyDrop<B>,
}
impl<A, B> Clone for Union<A, B> {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}

pub mod enums;
pub mod padding;
pub mod result;
pub use result::Result;
pub mod option;
pub use option::Option;

pub use istable::{Array, End, IStable};

pub(crate) mod istable;
pub type NonZeroHole = U0;
