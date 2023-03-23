use crate::{istable::Or, *};

macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type ForbiddenValues = <$t as IStable>::ForbiddenValues;
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
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl<T> IStable for core::marker::PhantomData<T> {
    type Size = U0;
    type Align = U1;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::marker::PhantomPinned {
    type Size = U0;
    type Align = U1;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}
macro_rules! illegal_values {
    ([$($l: tt)*], [$($r: tt)*]) => {
        Or<illegal_values!($($l)*), illegal_values!($($r)*)>
    };
    ($t: ty, $($tt: tt)*) => {
        Or<Array<U0, $t, End>, illegal_values!($($tt)*)>
    };
    ($t: ty) => {
        Array<U0, $t, End>
    };
}
unsafe impl IStable for bool {
    type Align = U1;
    type Size = U1;
    type ForbiddenValues = illegal_values!(
        [
            [
                [
                    [
                        [[[U2], [U3, U4]], [[U5, U6], [U7, U8]]],
                        [[[U9, U10], [U11, U12]], [[U13, U14], [U15, U16]]]
                    ],
                    [
                        [[[U17, U18], [U19, U20]], [[U21, U22], [U23, U24]]],
                        [[[U25, U26], [U27, U28]], [[U29, U30], [U31, U32]]]
                    ]
                ],
                [
                    [
                        [[[U33, U34], [U35, U36]], [[U37, U38], [U39, U40]]],
                        [[[U41, U42], [U43, U44]], [[U45, U46], [U47, U48]]]
                    ],
                    [
                        [[[U49, U50], [U51, U52]], [[U53, U54], [U55, U56]]],
                        [[[U57, U58], [U59, U60]], [[U61, U62], [U63, U64]]]
                    ]
                ]
            ],
            [
                [
                    [
                        [[[U65, U66], [U67, U68]], [[U69, U70], [U71, U72]]],
                        [[[U73, U74], [U75, U76]], [[U77, U78], [U79, U80]]]
                    ],
                    [
                        [[[U81, U82], [U83, U84]], [[U85, U86], [U87, U88]]],
                        [[[U89, U90], [U91, U92]], [[U93, U94], [U95, U96]]]
                    ]
                ],
                [
                    [
                        [[[U97, U98], [U99, U100]], [[U101, U102], [U103, U104]]],
                        [[[U105, U106], [U107, U108]], [[U109, U110], [U111, U112]]]
                    ],
                    [
                        [[[U113, U114], [U115, U116]], [[U117, U118], [U119, U120]]],
                        [[[U121, U122], [U123, U124]], [[U125, U126], [U127, U128]]]
                    ]
                ]
            ]
        ],
        [
            [
                [
                    [
                        [[[U129], [U130, U131]], [[U132, U133], [U134, U135]]],
                        [[[U136, U137], [U138, U139]], [[U140, U141], [U142, U143]]]
                    ],
                    [
                        [[[U144, U145], [U146, U147]], [[U148, U149], [U150, U151]]],
                        [[[U152, U153], [U154, U155]], [[U156, U157], [U158, U159]]]
                    ]
                ],
                [
                    [
                        [[[U160, U161], [U162, U163]], [[U164, U165], [U166, U167]]],
                        [[[U168, U169], [U170, U171]], [[U172, U173], [U174, U175]]]
                    ],
                    [
                        [[[U176, U177], [U178, U179]], [[U180, U181], [U182, U183]]],
                        [[[U184, U185], [U186, U187]], [[U188, U189], [U190, U191]]]
                    ]
                ]
            ],
            [
                [
                    [
                        [[[U192, U193], [U194, U195]], [[U196, U197], [U198, U199]]],
                        [[[U200, U201], [U202, U203]], [[U204, U205], [U206, U207]]]
                    ],
                    [
                        [[[U208, U209], [U210, U211]], [[U212, U213], [U214, U215]]],
                        [[[U216, U217], [U218, U219]], [[U220, U221], [U222, U223]]]
                    ]
                ],
                [
                    [
                        [[[U224, U225], [U226, U227]], [[U228, U229], [U230, U231]]],
                        [[[U232, U233], [U234, U235]], [[U236, U237], [U238, U239]]]
                    ],
                    [
                        [[[U240, U241], [U242, U243]], [[U244, U245], [U246, U247]]],
                        [[[U248, U249], [U250, U251]], [[U252, U253], [U254, U255]]]
                    ]
                ]
            ]
        ]
    );
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}

unsafe impl IStable for u8 {
    type UnusedBits = End;
    type ForbiddenValues = End;
    type Align = U1;
    type Size = U1;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU8 {
    type Align = U1;
    type Size = U1;
    type UnusedBits = End;
    type ForbiddenValues = nz_holes!(U0);
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u16 {
    type UnusedBits = End;
    type ForbiddenValues = End;
    type Align = U2;
    type Size = U2;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU16 {
    type ForbiddenValues = nz_holes!(U0, U1);
    type UnusedBits = End;
    type Align = U2;
    type Size = U2;
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u32 {
    type UnusedBits = End;
    type ForbiddenValues = End;
    type Align = U4;
    type Size = U4;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU32 {
    type ForbiddenValues = nz_holes!(U0, U1, U2, U3);
    type UnusedBits = End;
    type Align = U4;
    type Size = U4;
    type HasExactlyOneNiche = B1;
}
unsafe impl IStable for u64 {
    type UnusedBits = End;
    type ForbiddenValues = End;
    type Align = U8;
    type Size = U8;
    type HasExactlyOneNiche = B0;
}
unsafe impl IStable for core::num::NonZeroU64 {
    type UnusedBits = End;
    type ForbiddenValues = nz_holes!(U0, U1, U2, U3, U4, U5, U6, U7);
    type Align = U8;
    type Size = U8;
    type HasExactlyOneNiche = B1;
}

unsafe impl IStable for u128 {
    type UnusedBits = End;
    type ForbiddenValues = End;
    type Size = U16;
    type HasExactlyOneNiche = B0;
    #[cfg(not(any(target_arch = "aarch64")))]
    type Align = U8;
    #[cfg(any(target_arch = "aarch64"))]
    type Align = U16;
}
unsafe impl IStable for core::num::NonZeroU128 {
    type UnusedBits = End;
    type ForbiddenValues =
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
unsafe impl<T: IStable> IStable for core::option::Option<T>
where
    HasExactlyOneNiche<core::option::Option<T>, T::HasExactlyOneNiche>: IStable,
{
    same_as!(HasExactlyOneNiche<core::option::Option<T>, T::HasExactlyOneNiche>);
}
unsafe impl<T: IStable> IStable for HasExactlyOneNiche<core::option::Option<T>, B1> {
    type Size = T::Size;
    type Align = T::Align;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
}

#[cfg(feature = "alloc")]
mod cfgalloc {
    use super::*;
    unsafe impl<T: IStable> IStable for alloc::boxed::Box<T> {
        same_as!(core::ptr::NonNull<T>);
    }
    unsafe impl<T: IStable> IStable for alloc::sync::Arc<T> {
        same_as!(core::ptr::NonNull<T>);
    }
    unsafe impl<T: IStable> IStable for alloc::sync::Weak<T> {
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
