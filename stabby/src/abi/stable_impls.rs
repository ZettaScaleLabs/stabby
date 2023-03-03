use crate::abi::*;

macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type IllegalValues = <$t as IStable>::IllegalValues;
        type HasExactlyOneNiche = <$t as IStable>::HasExactlyOneNiche;
    };
}
macro_rules! nz_holes {
    ($t: ty) => {
        Array<$t, NonZeroHole, End>
    };
    ($t: ty, $($tt: tt)*) => {
        Array<$t, NonZeroHole, nz_holes!($($tt)*)>
    };
}
unsafe impl IStable for () {
    type Size = U0;
    type Align = U0;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl<T> IStable for core::marker::PhantomData<T> {
    type Size = U0;
    type Align = U0;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::marker::PhantomPinned {
    type Size = U0;
    type Align = U0;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}

unsafe impl IStable for bool {
    type Align = U1;
    type Size = U1;
    type IllegalValues =
        Array<U0, stabby_macros::holes!([0xfffffffc, 0xffffffff, 0xffffffff, 0xffffffff]), End>;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}

unsafe impl IStable for u8 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U1;
    type Size = U1;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU8 {
    type Align = U1;
    type Size = U1;
    type UnusedBits = End;
    type IllegalValues = nz_holes!(U0);
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u16 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U2;
    type Size = U2;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU16 {
    type IllegalValues = nz_holes!(U0, U1);
    type UnusedBits = End;
    type Align = U2;
    type Size = U2;
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u32 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U4;
    type Size = U4;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU32 {
    type IllegalValues = nz_holes!(U0, U1, U2, U3);
    type UnusedBits = End;
    type Align = U4;
    type Size = U4;
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u64 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U8;
    type Size = U8;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU64 {
    type UnusedBits = End;
    type IllegalValues = nz_holes!(U0, U1, U2, U3, U4, U5, U6, U7);
    type Align = U8;
    type Size = U8;
    type HasExactlyOneNiche = B1;
}

// TODO: Support for 128bit types, which are going to be a bit more painful.
unsafe impl IStable for u128 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Size = U16;
    type HasExactlyOneNiche = B0;
    #[cfg(any(target_arch = "x86_64", target_arch = "arm"))]
    type Align = U8;
    #[cfg(any(target_arch = "aarch64"))]
    type Align = U16;
}
unsafe impl IStable for core::num::NonZeroU128 {
    type UnusedBits = End;
    type IllegalValues =
        nz_holes!(U0, U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15);
    type Size = U16;
    type HasExactlyOneNiche = B1;
    type Align = <u128 as IStable>::Align;
}

unsafe impl IStable for usize {
    #[cfg(target_pointer_width = "64")]
    same_as!(u64);
    #[cfg(target_pointer_width = "32")]
    same_as!(u32);
    #[cfg(target_pointer_width = "16")]
    same_as!(u16);
    #[cfg(target_pointer_width = "8")]
    same_as!(u8);
}
unsafe impl IStable for core::num::NonZeroUsize {
    #[cfg(target_pointer_width = "64")]
    same_as!(core::num::NonZeroU64);
    #[cfg(target_pointer_width = "32")]
    same_as!(core::num::NonZeroU32);
    #[cfg(target_pointer_width = "16")]
    same_as!(core::num::NonZeroU16);
    #[cfg(target_pointer_width = "8")]
    same_as!(core::num::NonZeroU8);
}

unsafe impl IStable for i8 {
    same_as!(u8);
}
unsafe impl IStable for core::num::NonZeroI8 {
    same_as!(core::num::NonZeroU8);
}
unsafe impl IStable for i16 {
    same_as!(u16);
}
unsafe impl IStable for core::num::NonZeroI16 {
    same_as!(core::num::NonZeroU16);
}
unsafe impl IStable for i32 {
    same_as!(u32);
}
unsafe impl IStable for core::num::NonZeroI32 {
    same_as!(core::num::NonZeroU32);
}
unsafe impl IStable for i64 {
    same_as!(u64);
}
unsafe impl IStable for core::num::NonZeroI64 {
    same_as!(core::num::NonZeroU64);
}
unsafe impl IStable for i128 {
    same_as!(u128);
}
unsafe impl IStable for core::num::NonZeroI128 {
    same_as!(core::num::NonZeroU128);
}
unsafe impl IStable for isize {
    same_as!(usize);
}
unsafe impl IStable for core::num::NonZeroIsize {
    same_as!(core::num::NonZeroUsize);
}

unsafe impl<T: IStable> IStable for core::mem::ManuallyDrop<T> {
    same_as!(T);
}
unsafe impl<T: IStable> IStable for core::mem::MaybeUninit<T> {
    same_as!(T);
}
unsafe impl<T: IStable> IStable for core::cell::UnsafeCell<T> {
    same_as!(T);
}

unsafe impl<T: IStable> IStable for *const T {
    same_as!(usize);
}
unsafe impl<T: IStable> IStable for *mut T {
    same_as!(usize);
}
unsafe impl<T: IStable> IStable for core::ptr::NonNull<T> {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: IStable> IStable for core::sync::atomic::AtomicPtr<T> {
    same_as!(*mut T);
}
unsafe impl IStable for core::sync::atomic::AtomicBool {
    same_as!(bool);
}
unsafe impl IStable for core::sync::atomic::AtomicI8 {
    same_as!(i8);
}
unsafe impl IStable for core::sync::atomic::AtomicI16 {
    same_as!(i16);
}
unsafe impl IStable for core::sync::atomic::AtomicI32 {
    same_as!(i32);
}
unsafe impl IStable for core::sync::atomic::AtomicI64 {
    same_as!(i64);
}
unsafe impl IStable for core::sync::atomic::AtomicIsize {
    same_as!(isize);
}
unsafe impl IStable for core::sync::atomic::AtomicU8 {
    same_as!(u8);
}
unsafe impl IStable for core::sync::atomic::AtomicU16 {
    same_as!(u16);
}
unsafe impl IStable for core::sync::atomic::AtomicU32 {
    same_as!(u32);
}
unsafe impl IStable for core::sync::atomic::AtomicU64 {
    same_as!(u64);
}
unsafe impl IStable for core::sync::atomic::AtomicUsize {
    same_as!(usize);
}
unsafe impl<T: IStable> IStable for &T {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: IStable> IStable for &mut T {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: IStable> IStable for core::pin::Pin<T> {
    same_as!(T);
}

pub struct HasExactlyOneNiche<A, B>(core::marker::PhantomData<(A, B)>);
unsafe impl<T: IStable> IStable for Option<T>
where
    HasExactlyOneNiche<Option<T>, T::HasExactlyOneNiche>: IStable,
{
    same_as!(HasExactlyOneNiche<Option<T>, T::HasExactlyOneNiche>);
}
unsafe impl<T: IStable> IStable for HasExactlyOneNiche<Option<T>, B1> {
    type Size = T::Size;
    type Align = T::Align;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}

#[cfg(feature = "alloc")]
mod cfgalloc {
    use super::*;
    unsafe impl<T: IStable> IStable for crate::alloc::boxed::Box<T> {
        same_as!(core::ptr::NonNull<T>);
    }
    unsafe impl<T: IStable> IStable for crate::alloc::sync::Arc<T> {
        same_as!(core::ptr::NonNull<T>);
    }
    unsafe impl<T: IStable> IStable for crate::alloc::sync::Weak<T> {
        same_as!(core::ptr::NonNull<T>);
    }
}
