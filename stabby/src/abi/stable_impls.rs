use crate::abi::{istable::Or, *};

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
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl<T> IStable for core::marker::PhantomData<T> {
    type Size = U0;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::marker::PhantomPinned {
    type Size = U0;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
macro_rules! illegal_values {
    ($t: ty) => {
        Array<U0, $t, End>
    };
    ($t: ty,) => {
        Array<U0, $t, End>
    };
    ($t: ty, $($tt: tt)*) => {
        Or<Array<U0, $t, End>, illegal_values!($($tt)*)>
    };
    (($($l: tt)*), ($($r: tt)*)) => {
        Or<illegal_values!($($l)*), illegal_values!($($r)*)>
    };
}
unsafe impl IStable for bool {
    type Align = U1;
    type Size = U1;
    type IllegalValues = illegal_values!(
        (
            (
                (
                    U255, U254, U253, U252, U251, U250, U249, U248, U247, U246, U245, U244, U243,
                    U242, U241, U240, U239, U238, U237, U236, U235, U234, U233, U232, U231, U230,
                    U229, U228, U227, U226, U225, U224, U223, U222, U221
                ),
                (
                    U220, U219, U218, U217, U216, U215, U214, U213, U212, U211, U210, U209, U208,
                    U207, U206, U205, U204, U203, U202, U201, U200, U199, U198, U197, U196, U195,
                    U194, U193, U192, U191, U190, U189, U188, U187, U186
                )
            ),
            (
                (
                    U185, U184, U183, U182, U181, U180, U179, U178, U177, U176, U175, U174, U173,
                    U172, U171, U170, U169, U168, U167, U166, U165, U164, U163, U162, U161, U160,
                    U159, U158, U157, U156, U155, U154, U153
                ),
                (
                    U152, U151, U150, U149, U148, U147, U146, U145, U144, U143, U142, U141, U140,
                    U139, U138, U137, U136, U135, U134, U133, U132, U131, U130, U129
                )
            )
        ),
        (
            (
                (
                    U128, U127, U126, U125, U124, U123, U122, U121, U120, U119, U118, U117, U116,
                    U115, U114, U113, U112, U111, U110, U109, U108, U107, U106, U105, U104, U103,
                    U102, U101, U100
                ),
                (
                    U99, U98, U97, U96, U95, U94, U93, U92, U91, U90, U89, U88, U87, U86, U85, U84,
                    U83, U82, U81, U80, U79, U78, U77, U76, U75, U74, U73, U72, U71, U70, U69, U68,
                    U67
                )
            ),
            (
                (
                    U66, U65, U64, U63, U62, U61, U60, U59, U58, U57, U56, U55, U54, U53, U52, U51,
                    U50, U49, U48, U47, U46, U45, U44, U43, U42, U41, U40, U39, U38, U37, U36, U35
                ),
                (
                    U34, U33, U32, U31, U30, U29, U28, U27, U26, U25, U24, U23, U22, U21, U20, U19,
                    U18, U17, U16, U15, U14, U13, U12, U11, U10, U9, U8, U7, U6, U5, U4, U3, U2
                )
            )
        ),
    );
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

unsafe impl IStable for u128 {
    type UnusedBits = End;
    type IllegalValues = End;
    type Size = U16;
    type HasExactlyOneNiche = B0;
    #[cfg(not(any(target_arch = "aarch64")))]
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

macro_rules! fnstable {
    (-> $o: ident) => {
        unsafe impl<$o: IStable > IStable for extern "C" fn() -> $o {
            same_as!(core::num::NonZeroUsize);
        }
        unsafe impl<$o: IStable > IStable for unsafe extern "C" fn() -> $o {
            same_as!(core::num::NonZeroUsize);
        }
    };
    ($t: ident, $($tt: ident, )* -> $o: ident) => {
        unsafe impl< $o , $t, $($tt,)* > IStable for extern "C" fn($t, $($tt,)*) -> $o
        where $o : IStable, $t: IStable, $($tt: IStable,)* {
            same_as!(core::num::NonZeroUsize);
        }
        unsafe impl< $o : IStable, $t: IStable, $($tt: IStable,)* > IStable for unsafe extern "C" fn($t, $($tt,)*) -> $o {
            same_as!(core::num::NonZeroUsize);
        }
        fnstable!($($tt,)* -> $o);
    };
}
fnstable!(I15, I14, I13, I12, I11, I10, I9, I8, I7, I6, I5, I4, I3, I2, I1, -> Output);
