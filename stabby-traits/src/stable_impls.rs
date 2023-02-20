use super::*;
use type_layouts::Stable;

macro_rules! same_as {
    ($t: ty) => {
        type Niches = <$t as Stable>::Niches;
        type Start = <$t as Stable>::Start;
        type Size = <$t as Stable>::Size;
        type Align = <$t as Stable>::Align;
    };
}
unsafe impl Stable for () {
    type Niches = End<Self::Size>;
    type Start = U0;
    type Size = U0;
    type Align = U0;
}

unsafe impl Stable for bool {
    type Niches = Niche<
        U0,
        stabby_macros::holes!([0xfffffffc, 0xffffffff, 0xffffffff, 0xffffffff]),
        End<Self::Size>,
    >;
    type Start = U0;
    type Size = U0;
    type Align = U0;
}

unsafe impl Stable for u8 {
    type Niches = End<Self::Size>;
    type Start = U0;
    type Align = U1;
    type Size = U1;
}
unsafe impl Stable for core::num::NonZeroU8 {
    type Niches = Niche<U0, NonZeroHole, End<Self::Size>>;
    type Start = U0;
    type Align = U1;
    type Size = U1;
}
unsafe impl Stable for u16 {
    type Niches = End<Self::Size>;
    type Start = U0;
    type Align = U2;
    type Size = U2;
}
unsafe impl Stable for core::num::NonZeroU16 {
    type Niches = Niche<U1, NonZeroHole, Niche<U0, NonZeroHole, End<Self::Size>>>;
    type Start = U0;
    type Align = U2;
    type Size = U2;
}
unsafe impl Stable for u32 {
    type Niches = End<Self::Size>;
    type Start = U0;
    type Align = U4;
    type Size = U4;
}
unsafe impl Stable for core::num::NonZeroU32 {
    type Niches = Niche<
        U3,
        NonZeroHole,
        Niche<U2, NonZeroHole, Niche<U1, NonZeroHole, Niche<U0, NonZeroHole, End<Self::Size>>>>,
    >;
    type Start = U0;
    type Align = U4;
    type Size = U4;
}
unsafe impl Stable for u64 {
    type Niches = End<Self::Size>;
    type Start = U0;
    type Align = U8;
    type Size = U8;
}
unsafe impl Stable for core::num::NonZeroU64 {
    type Niches = Niche<
        U7,
        NonZeroHole,
        Niche<
            U6,
            NonZeroHole,
            Niche<
                U5,
                NonZeroHole,
                Niche<
                    U4,
                    NonZeroHole,
                    Niche<
                        U3,
                        NonZeroHole,
                        Niche<
                            U2,
                            NonZeroHole,
                            Niche<U1, NonZeroHole, Niche<U0, NonZeroHole, End<Self::Size>>>,
                        >,
                    >,
                >,
            >,
        >,
    >;
    type Start = U0;
    type Align = U8;
    type Size = U8;
}

// TODO: Support for 128bit types, which are going to be a bit more painful.

unsafe impl Stable for usize {
    #[cfg(target_pointer_width = "64")]
    same_as!(u64);
    #[cfg(target_pointer_width = "32")]
    same_as!(u32);
    #[cfg(target_pointer_width = "16")]
    same_as!(u16);
    #[cfg(target_pointer_width = "8")]
    same_as!(u8);
}
unsafe impl Stable for core::num::NonZeroUsize {
    #[cfg(target_pointer_width = "64")]
    same_as!(core::num::NonZeroU64);
    #[cfg(target_pointer_width = "32")]
    same_as!(core::num::NonZeroU32);
    #[cfg(target_pointer_width = "16")]
    same_as!(core::num::NonZeroU16);
    #[cfg(target_pointer_width = "8")]
    same_as!(core::num::NonZeroU8);
}

unsafe impl<T: Sized> Stable for *const T {
    same_as!(usize);
}
unsafe impl<T: Sized> Stable for *mut T {
    same_as!(usize);
}

unsafe impl<T: Sized> Stable for core::ptr::NonNull<T> {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: Sized> Stable for &T {
    same_as!(core::num::NonZeroUsize);
}
unsafe impl<T: Sized> Stable for &mut T {
    same_as!(core::num::NonZeroUsize);
}

unsafe impl Stable for i8 {
    same_as!(u8);
}
unsafe impl Stable for core::num::NonZeroI8 {
    same_as!(core::num::NonZeroU8);
}
unsafe impl Stable for i16 {
    same_as!(u16);
}
unsafe impl Stable for core::num::NonZeroI16 {
    same_as!(core::num::NonZeroU16);
}
unsafe impl Stable for i32 {
    same_as!(u32);
}
unsafe impl Stable for core::num::NonZeroI32 {
    same_as!(core::num::NonZeroU32);
}
unsafe impl Stable for i64 {
    same_as!(u64);
}
unsafe impl Stable for core::num::NonZeroI64 {
    same_as!(core::num::NonZeroU64);
}

unsafe impl Stable for isize {
    same_as!(usize);
}
unsafe impl Stable for core::num::NonZeroIsize {
    same_as!(core::num::NonZeroUsize);
}

unsafe impl<T: Stable> Stable for core::mem::ManuallyDrop<T> {
    same_as!(T);
}
unsafe impl<T: Stable> Stable for core::mem::MaybeUninit<T> {
    same_as!(T);
}

#[cfg(feature = "alloc")]
mod cfgalloc {
    use super::*;
    unsafe impl<T: Sized> Stable for crate::alloc::boxed::Box<T> {
        same_as!(core::ptr::NonNull<T>);
    }
}
