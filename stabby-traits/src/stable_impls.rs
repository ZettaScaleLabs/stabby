use crate as stabby_traits;
use crate::type_layouts::*;

macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type IllegalValues = <$t as IStable>::IllegalValues;
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
}

unsafe impl IStable for bool {
    type Align = U1;
    type Size = U1;
    type IllegalValues =
        Array<U0, stabby_macros::holes!([0xfffffffc, 0xffffffff, 0xffffffff, 0xffffffff]), End>;
    type UnusedBits = End;
}

unsafe impl IStable for u8 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U1;
    type Size = U1;
}
unsafe impl IStable for core::num::NonZeroU8 {
    type Align = U1;
    type Size = U1;
    type UnusedBits = End;
    type IllegalValues = nz_holes!(U0);
}
unsafe impl IStable for u16 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U2;
    type Size = U2;
}
unsafe impl IStable for core::num::NonZeroU16 {
    type IllegalValues = nz_holes!(U0, U1);
    type UnusedBits = End;
    type Align = U2;
    type Size = U2;
}
unsafe impl IStable for u32 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U4;
    type Size = U4;
}
unsafe impl IStable for core::num::NonZeroU32 {
    type IllegalValues = nz_holes!(U0, U1, U2, U3);
    type UnusedBits = End;
    type Align = U4;
    type Size = U4;
}
unsafe impl IStable for u64 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Align = U8;
    type Size = U8;
}
unsafe impl IStable for core::num::NonZeroU64 {
    type UnusedBits = End;
    type IllegalValues = nz_holes!(U0, U1, U2, U3, U4, U5, U6, U7);
    type Align = U8;
    type Size = U8;
}

// TODO: Support for 128bit types, which are going to be a bit more painful.

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

unsafe impl<T: IStable> IStable for *const T {
    same_as!(usize);
}
unsafe impl<T: IStable> IStable for *mut T {
    same_as!(usize);
}

unsafe impl<T: IStable> IStable for core::ptr::NonNull<T> {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: IStable> IStable for &T {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: IStable> IStable for &mut T {
    same_as!(core::num::NonZeroUsize);
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

#[cfg(feature = "alloc")]
mod cfgalloc {
    use super::*;
    unsafe impl<T: IStable> IStable for crate::alloc::boxed::Box<T> {
        same_as!(core::ptr::NonNull<T>);
    }
}
