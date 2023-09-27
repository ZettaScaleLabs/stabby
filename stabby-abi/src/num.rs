macro_rules! define_non_max {
    ($NonMaxU8:ident: $u8: ty = $NonZeroU8: ty ) => {
        /// A number whose bit pattern is guaranteed not to be only 1s.
        ///
        /// `x` is stored as `NonZero(!x)`, so transmuting results in wrong values.
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $NonMaxU8 {
            inner: $NonZeroU8,
        }
        impl $NonMaxU8 {
            pub const fn new(n: $u8) -> Option<Self> {
                match <$NonZeroU8>::new(!n) {
                    Some(n) => Some(Self { inner: n }),
                    None => None,
                }
            }
            /// Constructs `Self` without checking whether or not it is only 1s.
            ///
            /// # Safety
            /// Calling this with the illegal value may result in undefined behaviour.
            pub const unsafe fn new_unchecked(n: $u8) -> Self {
                Self {
                    inner: <$NonZeroU8>::new_unchecked(!n),
                }
            }
            pub const fn get(self) -> $u8 {
                !self.inner.get()
            }
        }
        impl From<$NonMaxU8> for $u8 {
            fn from(value: $NonMaxU8) -> Self {
                value.get()
            }
        }
        impl TryFrom<$u8> for $NonMaxU8 {
            type Error = ();
            fn try_from(value: $u8) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(())
            }
        }
        impl PartialOrd for $NonMaxU8 {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.get().partial_cmp(&other.get())
            }
        }
        impl Ord for $NonMaxU8 {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.get().cmp(&other.get())
            }
        }
        impl core::hash::Hash for $NonMaxU8 {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.get().hash(state)
            }
        }
        unsafe impl crate::IStable for $NonMaxU8 {
            type Size = <$NonZeroU8 as crate::IStable>::Size;
            type Align = <$NonZeroU8 as crate::IStable>::Align;
            type ForbiddenValues = <$NonZeroU8 as crate::IStable>::ForbiddenValues;
            type UnusedBits = <$NonZeroU8 as crate::IStable>::UnusedBits;
            type HasExactlyOneNiche = <$NonZeroU8 as crate::IStable>::HasExactlyOneNiche;
            const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
                name: $crate::str::Str::new(stringify!($NonMaxU8)),
                module: $crate::str::Str::new(core::module_path!()),
                fields: $crate::StableLike::new(None),
                last_break: $crate::report::Version::NEVER,
                tyty: $crate::report::TyTy::Struct,
            };
        }
    };
}
macro_rules! define_non_x {
    ($NonMaxU8:ident: $u8: ty = $NonZeroU8: ty ) => {
        /// A number whose value is guaranteed not to be `FORBIDDEN`.
        ///
        /// `x` is stored as `NonZero(x.wrapping_sub(FORBIDDEN))`, so transmuting results in wrong values.
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $NonMaxU8<const FORBIDDEN: $u8> {
            inner: $NonZeroU8,
        }
        impl<const FORBIDDEN: $u8> $NonMaxU8<{ FORBIDDEN }> {
            pub const fn new(n: $u8) -> Option<Self> {
                match <$NonZeroU8>::new(n.wrapping_sub(FORBIDDEN)) {
                    Some(n) => Some(Self { inner: n }),
                    None => None,
                }
            }
            /// Constructs `Self` without checking whether or not it is only 1s.
            ///
            /// # Safety
            /// Calling this with the illegal value may result in undefined behaviour.
            pub const unsafe fn new_unchecked(n: $u8) -> Self {
                Self {
                    inner: <$NonZeroU8>::new_unchecked(n.wrapping_sub(FORBIDDEN)),
                }
            }
            pub const fn get(self) -> $u8 {
                self.inner.get().wrapping_add(FORBIDDEN)
            }
        }
        impl<const FORBIDDEN: $u8> From<$NonMaxU8<{ FORBIDDEN }>> for $u8 {
            fn from(value: $NonMaxU8<{ FORBIDDEN }>) -> Self {
                value.get()
            }
        }
        impl<const FORBIDDEN: $u8> TryFrom<$u8> for $NonMaxU8<{ FORBIDDEN }> {
            type Error = ();
            fn try_from(value: $u8) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(())
            }
        }
        impl<const FORBIDDEN: $u8> PartialOrd for $NonMaxU8<{ FORBIDDEN }> {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.get().partial_cmp(&other.get())
            }
        }
        impl<const FORBIDDEN: $u8> Ord for $NonMaxU8<{ FORBIDDEN }> {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.get().cmp(&other.get())
            }
        }
        impl<const FORBIDDEN: $u8> core::hash::Hash for $NonMaxU8<{ FORBIDDEN }> {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.get().hash(state)
            }
        }
        unsafe impl<const FORBIDDEN: $u8> crate::IStable for $NonMaxU8<{ FORBIDDEN }> {
            type Size = <$NonZeroU8 as crate::IStable>::Size;
            type Align = <$NonZeroU8 as crate::IStable>::Align;
            type ForbiddenValues = <$NonZeroU8 as crate::IStable>::ForbiddenValues;
            type UnusedBits = <$NonZeroU8 as crate::IStable>::UnusedBits;
            type HasExactlyOneNiche = <$NonZeroU8 as crate::IStable>::HasExactlyOneNiche;
            const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
                name: $crate::str::Str::new(stringify!($NonMaxU8)),
                module: $crate::str::Str::new(core::module_path!()),
                fields: $crate::StableLike::new(None),
                last_break: $crate::report::Version::NEVER,
                tyty: $crate::report::TyTy::Struct,
            };
        }
    };
}

define_non_max!(NonMaxU8: u8 = core::num::NonZeroU8);
define_non_max!(NonMaxU16: u16 = core::num::NonZeroU16);
define_non_max!(NonMaxU32: u32 = core::num::NonZeroU32);
define_non_max!(NonMaxU64: u64 = core::num::NonZeroU64);
define_non_max!(NonMaxU128: u128 = core::num::NonZeroU128);
define_non_max!(NonMaxUsize: usize = core::num::NonZeroUsize);

define_non_x!(NonXU8: u8 = core::num::NonZeroU8);
define_non_x!(NonXU16: u16 = core::num::NonZeroU16);
define_non_x!(NonXU32: u32 = core::num::NonZeroU32);
define_non_x!(NonXU64: u64 = core::num::NonZeroU64);
define_non_x!(NonXU128: u128 = core::num::NonZeroU128);
define_non_x!(NonXUsize: usize = core::num::NonZeroUsize);
define_non_x!(NonXI8: i8 = core::num::NonZeroI8);
define_non_x!(NonXI16: i16 = core::num::NonZeroI16);
define_non_x!(NonXI32: i32 = core::num::NonZeroI32);
define_non_x!(NonXI64: i64 = core::num::NonZeroI64);
define_non_x!(NonXI128: i128 = core::num::NonZeroI128);
define_non_x!(NonXIsize: isize = core::num::NonZeroIsize);

macro_rules! makeutest {
    ($u8: ident, $NonMaxU8: ident, $NonXU8: ident) => {
        #[test]
        fn $u8() {
            for i in 0..255 {
                assert_eq!($NonMaxU8::new(i).unwrap().get(), i);
                assert_eq!($NonXU8::<{ $u8::MAX }>::new(i).unwrap().get(), i);
            }
            assert!($NonMaxU8::new($u8::MAX).is_none());
            assert!($NonXU8::<{ $u8::MAX }>::new($u8::MAX).is_none());
            assert!($NonXU8::<72>::new(72).is_none());
            for i in 0..=255 {
                if i != 72 {
                    assert_eq!($NonXU8::<72>::new(i).unwrap().get(), i);
                }
            }
        }
    };
}
makeutest!(u8, NonMaxU8, NonXU8);
makeutest!(u16, NonMaxU16, NonXU16);
makeutest!(u32, NonMaxU32, NonXU32);
makeutest!(u64, NonMaxU64, NonXU64);
makeutest!(u128, NonMaxU128, NonXU128);
makeutest!(usize, NonMaxUsize, NonXUsize);
macro_rules! makeitest {
    ($i8: ident,  $NonXI8: ident) => {
        #[test]
        fn $i8() {
            for i in -127..=127 {
                assert_eq!($NonXI8::<{ $i8::MIN }>::new(i).unwrap().get(), i);
            }
            assert!($NonXI8::<{ $i8::MIN }>::new($i8::MIN).is_none());
            assert!($NonXI8::<72>::new(72).is_none());
            for i in -128..=127 {
                if i != 72 {
                    assert_eq!($NonXI8::<72>::new(i).unwrap().get(), i);
                }
            }
        }
    };
}
makeitest!(i8, NonXI8);
makeitest!(i16, NonXI16);
makeitest!(i32, NonXI32);
makeitest!(i64, NonXI64);
makeitest!(i128, NonXI128);
makeitest!(isize, NonXIsize);
