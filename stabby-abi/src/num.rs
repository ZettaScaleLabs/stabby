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
            /// Constructs `Self`, returning `None` if `n == MAX`
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
            /// Returns the inner value.
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
                Some(self.cmp(other))
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
            type ContainsIndirections = <$NonZeroU8 as crate::IStable>::ContainsIndirections;
            const ID: u64 = $crate::report::gen_id(Self::REPORT);
            const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
                name: $crate::str::Str::new(stringify!($NonMaxU8)),
                module: $crate::str::Str::new(core::module_path!()),
                fields: $crate::StableLike::new(None),
                version: 0,
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
            /// Construct `Self`, returning `None` if `n` is the `FORBIDDEN` value.
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
            /// Get the inner value.
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
                Some(self.cmp(other))
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
            type ContainsIndirections = <$NonZeroU8 as crate::IStable>::ContainsIndirections;
            const ID: u64 = $crate::report::gen_id(Self::REPORT);
            const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
                name: $crate::str::Str::new(stringify!($NonMaxU8)),
                module: $crate::str::Str::new(core::module_path!()),
                fields: $crate::StableLike::new(None),
                version: 0,
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

macro_rules! makeumask {
    ($name: ident, $base: ident, $bits: ty, $docs: literal) => {
        #[doc = $docs]
        ///
        /// Its memory layout from Rust's point of view is that of the smallest unsigned type
        /// that can contain it.
        ///
        /// However, `stabby` can tell that it's most significant bits are unused, and use that knowledge to make
        /// `#[repr(stabby)]` `enum`s smaller.
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
        pub struct $name($base);
        impl $name {
            const MASK: $base = $base::MAX
                >> (8 * ::core::mem::size_of::<$base>() as $base
                    - <$bits as $crate::typenum2::Unsigned>::U128 as $base);
            /// The maximum value for this type.
            pub const MAX: Self = Self(Self::MASK);
            /// The maximum value for this type.
            pub const MIN: Self = Self(0);
            /// Construct a new value if it can fit.
            pub const fn new(value: $base) -> Option<Self> {
                match value <= Self::MASK {
                    true => Some(Self(value)),
                    false => None,
                }
            }
            /// Get the inner value.
            pub const fn get(&self) -> $base {
                self.0 & Self::MASK
            }
        }
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::LowerHex::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::UpperHex::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::LowerExp for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::LowerExp::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::UpperExp for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::UpperExp::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Binary for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Binary::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Octal for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Octal::fmt(&self.0, f)
            }
        }
        impl ::core::ops::BitAnd<$base> for $name {
            type Output = Self;
            fn bitand(self, rhs: $base) -> Self {
                Self(self.0 & rhs)
            }
        }
        impl ::core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }
        impl<T> ::core::ops::BitAndAssign<T> for $name
        where
            Self: ::core::ops::BitAnd<T, Output = Self>,
        {
            fn bitand_assign(&mut self, rhs: T) {
                *self = *self & rhs;
            }
        }
        impl ::core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl ::core::ops::BitOr<$base> for $name {
            type Output = $base;
            fn bitor(self, rhs: $base) -> $base {
                self.get() | rhs
            }
        }
        impl<T> ::core::ops::BitOrAssign<T> for $name
        where
            Self: ::core::ops::BitOr<T, Output = Self>,
        {
            fn bitor_assign(&mut self, rhs: T) {
                *self = *self | rhs;
            }
        }
        impl ::core::ops::Add for $name {
            type Output = $base;
            fn add(self, rhs: Self) -> $base {
                unsafe { self.get().checked_add(rhs.get()).unwrap_unchecked() }
            }
        }
        impl ::core::ops::Sub<$base> for $name {
            type Output = Self;
            fn sub(self, rhs: $base) -> Self {
                Self(self.get() - rhs)
            }
        }
        impl ::core::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.get() - rhs.get())
            }
        }
    };
}

macro_rules! makeimask {
    ($name: ident, $base: ident, $bits: ty, $docs: literal) => {
        #[doc = $docs]
        ///
        /// Its memory layout from Rust's point of view is that of the smallest unsigned type
        /// that can contain it.
        ///
        /// However, `stabby` can tell that it's most significant bits are unused, and use that knowledge to make
        /// `#[repr(stabby)]` `enum`s smaller.
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
        pub struct $name($base);
        impl $name {
            const MASK: $base = ($base::MAX
                >> (8 * ::core::mem::size_of::<$base>() as i32
                    - <$bits as $crate::typenum2::Unsigned>::U128 as i32))
                | (1 << (<$bits as $crate::typenum2::Unsigned>::U128 as i32 - 1));
            /// The maximum value for this type.
            pub const MAX: Self =
                Self($base::MAX >> (<$bits as $crate::typenum2::Unsigned>::U128 as $base));
            /// The maximum value for this type.
            pub const MIN: Self =
                Self(1 << (<$bits as $crate::typenum2::Unsigned>::U128 as i32 - 1));
            /// Construct a new value if it can fit.
            pub const fn new(value: $base) -> Option<Self> {
                match value <= Self::MAX.get() && value >= Self::MIN.get() {
                    true => Some(Self(value)),
                    false => None,
                }
            }
            const fn sign_extend(value: $base) -> $base {
                const SHIFT: i32 = (8 * ::core::mem::size_of::<$base>() as i32
                    - <$bits as $crate::typenum2::Unsigned>::U128 as i32);
                value | ((value & Self::MIN.0) << SHIFT) >> (SHIFT - 1)
            }
            /// Get the inner value.
            pub const fn get(&self) -> $base {
                Self::sign_extend(self.0 & Self::MASK)
            }
        }
        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::LowerHex::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::UpperHex::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::LowerExp for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::LowerExp::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::UpperExp for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::UpperExp::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Binary for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Binary::fmt(&self.0, f)
            }
        }
        impl ::core::fmt::Octal for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                ::core::fmt::Octal::fmt(&self.0, f)
            }
        }
        impl ::core::ops::BitAnd<$base> for $name {
            type Output = Self;
            fn bitand(self, rhs: $base) -> Self {
                Self(self.0 & rhs)
            }
        }
        impl ::core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }
        impl<T> ::core::ops::BitAndAssign<T> for $name
        where
            Self: ::core::ops::BitAnd<T, Output = Self>,
        {
            fn bitand_assign(&mut self, rhs: T) {
                *self = *self & rhs;
            }
        }
        impl ::core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl<T> ::core::ops::BitOrAssign<T> for $name
        where
            Self: ::core::ops::BitOr<T, Output = Self>,
        {
            fn bitor_assign(&mut self, rhs: T) {
                *self = *self | rhs;
            }
        }
    };
}

#[test]
fn numbers() {
    macro_rules! deftest {
        ($name: ty, $base: ty) => {
            for i in (<$base>::MIN..<$base>::MAX)
                .rev()
                .step_by(<$base>::MAX as usize / 127)
            {
                let Some(n) = <$name>::new(dbg!(i)) else {
                    assert!(i > <$name>::MAX.get() || i < <$name>::MIN.get());
                    continue;
                };
                assert_eq!(n.get(), i);
                assert!(i <= dbg!(<$name>::MAX.get()));
                assert!(i >= dbg!(<$name>::MIN.get()));
            }
        };
    }
    deftest!(u6, u8);
    deftest!(u26, u32);
    deftest!(i6, i8);
    deftest!(i26, i32);
}

makeumask!(u1, u8, crate::typenum2::U1, "A 1 bit unsigned integer");
makeumask!(u2, u8, crate::typenum2::U2, "A 2 bit unsigned integer");
makeumask!(u3, u8, crate::typenum2::U3, "A 3 bit unsigned integer");
makeumask!(u4, u8, crate::typenum2::U4, "A 4 bit unsigned integer");
makeumask!(u5, u8, crate::typenum2::U5, "A 5 bit unsigned integer");
makeumask!(u6, u8, crate::typenum2::U6, "A 6 bit unsigned integer");
makeumask!(u7, u8, crate::typenum2::U7, "A 7 bit unsigned integer");

makeumask!(u9, u16, crate::typenum2::U9, "A 9 bit unsigned integer");
makeumask!(u10, u16, crate::typenum2::U10, "A 10 bit unsigned integer");
makeumask!(u11, u16, crate::typenum2::U11, "A 11 bit unsigned integer");
makeumask!(u12, u16, crate::typenum2::U12, "A 12 bit unsigned integer");
makeumask!(u13, u16, crate::typenum2::U13, "A 13 bit unsigned integer");
makeumask!(u14, u16, crate::typenum2::U14, "A 14 bit unsigned integer");
makeumask!(u15, u16, crate::typenum2::U15, "A 15 bit unsigned integer");

makeumask!(u17, u32, crate::typenum2::U17, "A 17 bit unsigned integer");
makeumask!(u18, u32, crate::typenum2::U18, "A 18 bit unsigned integer");
makeumask!(u19, u32, crate::typenum2::U19, "A 19 bit unsigned integer");
makeumask!(u20, u32, crate::typenum2::U20, "A 20 bit unsigned integer");
makeumask!(u21, u32, crate::typenum2::U21, "A 21 bit unsigned integer");
makeumask!(u22, u32, crate::typenum2::U22, "A 22 bit unsigned integer");
makeumask!(u23, u32, crate::typenum2::U23, "A 23 bit unsigned integer");
makeumask!(u24, u32, crate::typenum2::U24, "A 24 bit unsigned integer");
makeumask!(u25, u32, crate::typenum2::U25, "A 25 bit unsigned integer");
makeumask!(u26, u32, crate::typenum2::U26, "A 26 bit unsigned integer");
makeumask!(u27, u32, crate::typenum2::U27, "A 27 bit unsigned integer");
makeumask!(u28, u32, crate::typenum2::U28, "A 28 bit unsigned integer");
makeumask!(u29, u32, crate::typenum2::U29, "A 29 bit unsigned integer");
makeumask!(u30, u32, crate::typenum2::U30, "A 30 bit unsigned integer");
makeumask!(u31, u32, crate::typenum2::U31, "A 31 bit unsigned integer");

makeumask!(u33, u64, crate::typenum2::U33, "A 33 bit unsigned integer");
makeumask!(u34, u64, crate::typenum2::U34, "A 34 bit unsigned integer");
makeumask!(u35, u64, crate::typenum2::U35, "A 35 bit unsigned integer");
makeumask!(u36, u64, crate::typenum2::U36, "A 36 bit unsigned integer");
makeumask!(u37, u64, crate::typenum2::U37, "A 37 bit unsigned integer");
makeumask!(u38, u64, crate::typenum2::U38, "A 38 bit unsigned integer");
makeumask!(u39, u64, crate::typenum2::U39, "A 39 bit unsigned integer");
makeumask!(u40, u64, crate::typenum2::U40, "A 40 bit unsigned integer");
makeumask!(u41, u64, crate::typenum2::U41, "A 41 bit unsigned integer");
makeumask!(u42, u64, crate::typenum2::U42, "A 42 bit unsigned integer");
makeumask!(u43, u64, crate::typenum2::U43, "A 43 bit unsigned integer");
makeumask!(u44, u64, crate::typenum2::U44, "A 44 bit unsigned integer");
makeumask!(u45, u64, crate::typenum2::U45, "A 45 bit unsigned integer");
makeumask!(u46, u64, crate::typenum2::U46, "A 46 bit unsigned integer");
makeumask!(u47, u64, crate::typenum2::U47, "A 47 bit unsigned integer");
makeumask!(u48, u64, crate::typenum2::U48, "A 48 bit unsigned integer");
makeumask!(u49, u64, crate::typenum2::U49, "A 49 bit unsigned integer");
makeumask!(u50, u64, crate::typenum2::U50, "A 50 bit unsigned integer");
makeumask!(u51, u64, crate::typenum2::U51, "A 51 bit unsigned integer");
makeumask!(u52, u64, crate::typenum2::U52, "A 52 bit unsigned integer");
makeumask!(u53, u64, crate::typenum2::U53, "A 53 bit unsigned integer");
makeumask!(u54, u64, crate::typenum2::U54, "A 54 bit unsigned integer");
makeumask!(u55, u64, crate::typenum2::U55, "A 55 bit unsigned integer");
makeumask!(u56, u64, crate::typenum2::U56, "A 56 bit unsigned integer");
makeumask!(u57, u64, crate::typenum2::U57, "A 57 bit unsigned integer");
makeumask!(u58, u64, crate::typenum2::U58, "A 58 bit unsigned integer");
makeumask!(u59, u64, crate::typenum2::U59, "A 59 bit unsigned integer");
makeumask!(u60, u64, crate::typenum2::U60, "A 60 bit unsigned integer");
makeumask!(u61, u64, crate::typenum2::U61, "A 61 bit unsigned integer");
makeumask!(u62, u64, crate::typenum2::U62, "A 62 bit unsigned integer");
makeumask!(u63, u64, crate::typenum2::U63, "A 63 bit unsigned integer");

makeumask!(u65, u128, crate::typenum2::U65, "A 65 bit unsigned integer");
makeumask!(u66, u128, crate::typenum2::U66, "A 66 bit unsigned integer");
makeumask!(u67, u128, crate::typenum2::U67, "A 67 bit unsigned integer");
makeumask!(u68, u128, crate::typenum2::U68, "A 68 bit unsigned integer");
makeumask!(u69, u128, crate::typenum2::U69, "A 69 bit unsigned integer");
makeumask!(u70, u128, crate::typenum2::U70, "A 70 bit unsigned integer");
makeumask!(u71, u128, crate::typenum2::U71, "A 71 bit unsigned integer");
makeumask!(u72, u128, crate::typenum2::U72, "A 72 bit unsigned integer");
makeumask!(u73, u128, crate::typenum2::U73, "A 73 bit unsigned integer");
makeumask!(u74, u128, crate::typenum2::U74, "A 74 bit unsigned integer");
makeumask!(u75, u128, crate::typenum2::U75, "A 75 bit unsigned integer");
makeumask!(u76, u128, crate::typenum2::U76, "A 76 bit unsigned integer");
makeumask!(u77, u128, crate::typenum2::U77, "A 77 bit unsigned integer");
makeumask!(u78, u128, crate::typenum2::U78, "A 78 bit unsigned integer");
makeumask!(u79, u128, crate::typenum2::U79, "A 79 bit unsigned integer");
makeumask!(u80, u128, crate::typenum2::U80, "A 80 bit unsigned integer");
makeumask!(u81, u128, crate::typenum2::U81, "A 81 bit unsigned integer");
makeumask!(u82, u128, crate::typenum2::U82, "A 82 bit unsigned integer");
makeumask!(u83, u128, crate::typenum2::U83, "A 83 bit unsigned integer");
makeumask!(u84, u128, crate::typenum2::U84, "A 84 bit unsigned integer");
makeumask!(u85, u128, crate::typenum2::U85, "A 85 bit unsigned integer");
makeumask!(u86, u128, crate::typenum2::U86, "A 86 bit unsigned integer");
makeumask!(u87, u128, crate::typenum2::U87, "A 87 bit unsigned integer");
makeumask!(u88, u128, crate::typenum2::U88, "A 88 bit unsigned integer");
makeumask!(u89, u128, crate::typenum2::U89, "A 89 bit unsigned integer");
makeumask!(u90, u128, crate::typenum2::U90, "A 90 bit unsigned integer");
makeumask!(u91, u128, crate::typenum2::U91, "A 91 bit unsigned integer");
makeumask!(u92, u128, crate::typenum2::U92, "A 92 bit unsigned integer");
makeumask!(u93, u128, crate::typenum2::U93, "A 93 bit unsigned integer");
makeumask!(u94, u128, crate::typenum2::U94, "A 94 bit unsigned integer");
makeumask!(u95, u128, crate::typenum2::U95, "A 95 bit unsigned integer");
makeumask!(u96, u128, crate::typenum2::U96, "A 96 bit unsigned integer");
makeumask!(u97, u128, crate::typenum2::U97, "A 97 bit unsigned integer");
makeumask!(u98, u128, crate::typenum2::U98, "A 98 bit unsigned integer");
makeumask!(u99, u128, crate::typenum2::U99, "A 99 bit unsigned integer");
makeumask!(
    u100,
    u128,
    crate::typenum2::U100,
    "A 100 bit unsigned integer"
);
makeumask!(
    u101,
    u128,
    crate::typenum2::U101,
    "A 101 bit unsigned integer"
);
makeumask!(
    u102,
    u128,
    crate::typenum2::U102,
    "A 102 bit unsigned integer"
);
makeumask!(
    u103,
    u128,
    crate::typenum2::U103,
    "A 103 bit unsigned integer"
);
makeumask!(
    u104,
    u128,
    crate::typenum2::U104,
    "A 104 bit unsigned integer"
);
makeumask!(
    u105,
    u128,
    crate::typenum2::U105,
    "A 105 bit unsigned integer"
);
makeumask!(
    u106,
    u128,
    crate::typenum2::U106,
    "A 106 bit unsigned integer"
);
makeumask!(
    u107,
    u128,
    crate::typenum2::U107,
    "A 107 bit unsigned integer"
);
makeumask!(
    u108,
    u128,
    crate::typenum2::U108,
    "A 108 bit unsigned integer"
);
makeumask!(
    u109,
    u128,
    crate::typenum2::U109,
    "A 109 bit unsigned integer"
);
makeumask!(
    u110,
    u128,
    crate::typenum2::U110,
    "A 110 bit unsigned integer"
);
makeumask!(
    u111,
    u128,
    crate::typenum2::U111,
    "A 111 bit unsigned integer"
);
makeumask!(
    u112,
    u128,
    crate::typenum2::U112,
    "A 112 bit unsigned integer"
);
makeumask!(
    u113,
    u128,
    crate::typenum2::U113,
    "A 113 bit unsigned integer"
);
makeumask!(
    u114,
    u128,
    crate::typenum2::U114,
    "A 114 bit unsigned integer"
);
makeumask!(
    u115,
    u128,
    crate::typenum2::U115,
    "A 115 bit unsigned integer"
);
makeumask!(
    u116,
    u128,
    crate::typenum2::U116,
    "A 116 bit unsigned integer"
);
makeumask!(
    u117,
    u128,
    crate::typenum2::U117,
    "A 117 bit unsigned integer"
);
makeumask!(
    u118,
    u128,
    crate::typenum2::U118,
    "A 118 bit unsigned integer"
);
makeumask!(
    u119,
    u128,
    crate::typenum2::U119,
    "A 119 bit unsigned integer"
);
makeumask!(
    u120,
    u128,
    crate::typenum2::U120,
    "A 120 bit unsigned integer"
);
makeumask!(
    u121,
    u128,
    crate::typenum2::U121,
    "A 121 bit unsigned integer"
);
makeumask!(
    u122,
    u128,
    crate::typenum2::U122,
    "A 122 bit unsigned integer"
);
makeumask!(
    u123,
    u128,
    crate::typenum2::U123,
    "A 123 bit unsigned integer"
);
makeumask!(
    u124,
    u128,
    crate::typenum2::U124,
    "A 124 bit unsigned integer"
);
makeumask!(
    u125,
    u128,
    crate::typenum2::U125,
    "A 125 bit unsigned integer"
);
makeumask!(
    u126,
    u128,
    crate::typenum2::U126,
    "A 126 bit unsigned integer"
);
makeumask!(
    u127,
    u128,
    crate::typenum2::U127,
    "A 127 bit unsigned integer"
);

makeimask!(i1, i8, crate::typenum2::U1, "A 1 bit signed integer");
makeimask!(i2, i8, crate::typenum2::U2, "A 2 bit signed integer");
makeimask!(i3, i8, crate::typenum2::U3, "A 3 bit signed integer");
makeimask!(i4, i8, crate::typenum2::U4, "A 4 bit signed integer");
makeimask!(i5, i8, crate::typenum2::U5, "A 5 bit signed integer");
makeimask!(i6, i8, crate::typenum2::U6, "A 6 bit signed integer");
makeimask!(i7, i8, crate::typenum2::U7, "A 7 bit signed integer");

makeimask!(i9, i16, crate::typenum2::U9, "A 9 bit signed integer");
makeimask!(i10, i16, crate::typenum2::U10, "A 10 bit signed integer");
makeimask!(i11, i16, crate::typenum2::U11, "A 11 bit signed integer");
makeimask!(i12, i16, crate::typenum2::U12, "A 12 bit signed integer");
makeimask!(i13, i16, crate::typenum2::U13, "A 13 bit signed integer");
makeimask!(i14, i16, crate::typenum2::U14, "A 14 bit signed integer");
makeimask!(i15, i16, crate::typenum2::U15, "A 15 bit signed integer");

makeimask!(i17, i32, crate::typenum2::U17, "A 17 bit signed integer");
makeimask!(i18, i32, crate::typenum2::U18, "A 18 bit signed integer");
makeimask!(i19, i32, crate::typenum2::U19, "A 19 bit signed integer");
makeimask!(i20, i32, crate::typenum2::U20, "A 20 bit signed integer");
makeimask!(i21, i32, crate::typenum2::U21, "A 21 bit signed integer");
makeimask!(i22, i32, crate::typenum2::U22, "A 22 bit signed integer");
makeimask!(i23, i32, crate::typenum2::U23, "A 23 bit signed integer");
makeimask!(i24, i32, crate::typenum2::U24, "A 24 bit signed integer");
makeimask!(i25, i32, crate::typenum2::U25, "A 25 bit signed integer");
makeimask!(i26, i32, crate::typenum2::U26, "A 26 bit signed integer");
makeimask!(i27, i32, crate::typenum2::U27, "A 27 bit signed integer");
makeimask!(i28, i32, crate::typenum2::U28, "A 28 bit signed integer");
makeimask!(i29, i32, crate::typenum2::U29, "A 29 bit signed integer");
makeimask!(i30, i32, crate::typenum2::U30, "A 30 bit signed integer");
makeimask!(i31, i32, crate::typenum2::U31, "A 31 bit signed integer");

makeimask!(i33, i64, crate::typenum2::U33, "A 33 bit signed integer");
makeimask!(i34, i64, crate::typenum2::U34, "A 34 bit signed integer");
makeimask!(i35, i64, crate::typenum2::U35, "A 35 bit signed integer");
makeimask!(i36, i64, crate::typenum2::U36, "A 36 bit signed integer");
makeimask!(i37, i64, crate::typenum2::U37, "A 37 bit signed integer");
makeimask!(i38, i64, crate::typenum2::U38, "A 38 bit signed integer");
makeimask!(i39, i64, crate::typenum2::U39, "A 39 bit signed integer");
makeimask!(i40, i64, crate::typenum2::U40, "A 40 bit signed integer");
makeimask!(i41, i64, crate::typenum2::U41, "A 41 bit signed integer");
makeimask!(i42, i64, crate::typenum2::U42, "A 42 bit signed integer");
makeimask!(i43, i64, crate::typenum2::U43, "A 43 bit signed integer");
makeimask!(i44, i64, crate::typenum2::U44, "A 44 bit signed integer");
makeimask!(i45, i64, crate::typenum2::U45, "A 45 bit signed integer");
makeimask!(i46, i64, crate::typenum2::U46, "A 46 bit signed integer");
makeimask!(i47, i64, crate::typenum2::U47, "A 47 bit signed integer");
makeimask!(i48, i64, crate::typenum2::U48, "A 48 bit signed integer");
makeimask!(i49, i64, crate::typenum2::U49, "A 49 bit signed integer");
makeimask!(i50, i64, crate::typenum2::U50, "A 50 bit signed integer");
makeimask!(i51, i64, crate::typenum2::U51, "A 51 bit signed integer");
makeimask!(i52, i64, crate::typenum2::U52, "A 52 bit signed integer");
makeimask!(i53, i64, crate::typenum2::U53, "A 53 bit signed integer");
makeimask!(i54, i64, crate::typenum2::U54, "A 54 bit signed integer");
makeimask!(i55, i64, crate::typenum2::U55, "A 55 bit signed integer");
makeimask!(i56, i64, crate::typenum2::U56, "A 56 bit signed integer");
makeimask!(i57, i64, crate::typenum2::U57, "A 57 bit signed integer");
makeimask!(i58, i64, crate::typenum2::U58, "A 58 bit signed integer");
makeimask!(i59, i64, crate::typenum2::U59, "A 59 bit signed integer");
makeimask!(i60, i64, crate::typenum2::U60, "A 60 bit signed integer");
makeimask!(i61, i64, crate::typenum2::U61, "A 61 bit signed integer");
makeimask!(i62, i64, crate::typenum2::U62, "A 62 bit signed integer");
makeimask!(i63, i64, crate::typenum2::U63, "A 63 bit signed integer");

makeimask!(i65, i128, crate::typenum2::U65, "A 65 bit signed integer");
makeimask!(i66, i128, crate::typenum2::U66, "A 66 bit signed integer");
makeimask!(i67, i128, crate::typenum2::U67, "A 67 bit signed integer");
makeimask!(i68, i128, crate::typenum2::U68, "A 68 bit signed integer");
makeimask!(i69, i128, crate::typenum2::U69, "A 69 bit signed integer");
makeimask!(i70, i128, crate::typenum2::U70, "A 70 bit signed integer");
makeimask!(i71, i128, crate::typenum2::U71, "A 71 bit signed integer");
makeimask!(i72, i128, crate::typenum2::U72, "A 72 bit signed integer");
makeimask!(i73, i128, crate::typenum2::U73, "A 73 bit signed integer");
makeimask!(i74, i128, crate::typenum2::U74, "A 74 bit signed integer");
makeimask!(i75, i128, crate::typenum2::U75, "A 75 bit signed integer");
makeimask!(i76, i128, crate::typenum2::U76, "A 76 bit signed integer");
makeimask!(i77, i128, crate::typenum2::U77, "A 77 bit signed integer");
makeimask!(i78, i128, crate::typenum2::U78, "A 78 bit signed integer");
makeimask!(i79, i128, crate::typenum2::U79, "A 79 bit signed integer");
makeimask!(i80, i128, crate::typenum2::U80, "A 80 bit signed integer");
makeimask!(i81, i128, crate::typenum2::U81, "A 81 bit signed integer");
makeimask!(i82, i128, crate::typenum2::U82, "A 82 bit signed integer");
makeimask!(i83, i128, crate::typenum2::U83, "A 83 bit signed integer");
makeimask!(i84, i128, crate::typenum2::U84, "A 84 bit signed integer");
makeimask!(i85, i128, crate::typenum2::U85, "A 85 bit signed integer");
makeimask!(i86, i128, crate::typenum2::U86, "A 86 bit signed integer");
makeimask!(i87, i128, crate::typenum2::U87, "A 87 bit signed integer");
makeimask!(i88, i128, crate::typenum2::U88, "A 88 bit signed integer");
makeimask!(i89, i128, crate::typenum2::U89, "A 89 bit signed integer");
makeimask!(i90, i128, crate::typenum2::U90, "A 90 bit signed integer");
makeimask!(i91, i128, crate::typenum2::U91, "A 91 bit signed integer");
makeimask!(i92, i128, crate::typenum2::U92, "A 92 bit signed integer");
makeimask!(i93, i128, crate::typenum2::U93, "A 93 bit signed integer");
makeimask!(i94, i128, crate::typenum2::U94, "A 94 bit signed integer");
makeimask!(i95, i128, crate::typenum2::U95, "A 95 bit signed integer");
makeimask!(i96, i128, crate::typenum2::U96, "A 96 bit signed integer");
makeimask!(i97, i128, crate::typenum2::U97, "A 97 bit signed integer");
makeimask!(i98, i128, crate::typenum2::U98, "A 98 bit signed integer");
makeimask!(i99, i128, crate::typenum2::U99, "A 99 bit signed integer");
makeimask!(
    i100,
    i128,
    crate::typenum2::U100,
    "A 100 bit signed integer"
);
makeimask!(
    i101,
    i128,
    crate::typenum2::U101,
    "A 101 bit signed integer"
);
makeimask!(
    i102,
    i128,
    crate::typenum2::U102,
    "A 102 bit signed integer"
);
makeimask!(
    i103,
    i128,
    crate::typenum2::U103,
    "A 103 bit signed integer"
);
makeimask!(
    i104,
    i128,
    crate::typenum2::U104,
    "A 104 bit signed integer"
);
makeimask!(
    i105,
    i128,
    crate::typenum2::U105,
    "A 105 bit signed integer"
);
makeimask!(
    i106,
    i128,
    crate::typenum2::U106,
    "A 106 bit signed integer"
);
makeimask!(
    i107,
    i128,
    crate::typenum2::U107,
    "A 107 bit signed integer"
);
makeimask!(
    i108,
    i128,
    crate::typenum2::U108,
    "A 108 bit signed integer"
);
makeimask!(
    i109,
    i128,
    crate::typenum2::U109,
    "A 109 bit signed integer"
);
makeimask!(
    i110,
    i128,
    crate::typenum2::U110,
    "A 110 bit signed integer"
);
makeimask!(
    i111,
    i128,
    crate::typenum2::U111,
    "A 111 bit signed integer"
);
makeimask!(
    i112,
    i128,
    crate::typenum2::U112,
    "A 112 bit signed integer"
);
makeimask!(
    i113,
    i128,
    crate::typenum2::U113,
    "A 113 bit signed integer"
);
makeimask!(
    i114,
    i128,
    crate::typenum2::U114,
    "A 114 bit signed integer"
);
makeimask!(
    i115,
    i128,
    crate::typenum2::U115,
    "A 115 bit signed integer"
);
makeimask!(
    i116,
    i128,
    crate::typenum2::U116,
    "A 116 bit signed integer"
);
makeimask!(
    i117,
    i128,
    crate::typenum2::U117,
    "A 117 bit signed integer"
);
makeimask!(
    i118,
    i128,
    crate::typenum2::U118,
    "A 118 bit signed integer"
);
makeimask!(
    i119,
    i128,
    crate::typenum2::U119,
    "A 119 bit signed integer"
);
makeimask!(
    i120,
    i128,
    crate::typenum2::U120,
    "A 120 bit signed integer"
);
makeimask!(
    i121,
    i128,
    crate::typenum2::U121,
    "A 121 bit signed integer"
);
makeimask!(
    i122,
    i128,
    crate::typenum2::U122,
    "A 122 bit signed integer"
);
makeimask!(
    i123,
    i128,
    crate::typenum2::U123,
    "A 123 bit signed integer"
);
makeimask!(
    i124,
    i128,
    crate::typenum2::U124,
    "A 124 bit signed integer"
);
makeimask!(
    i125,
    i128,
    crate::typenum2::U125,
    "A 125 bit signed integer"
);
makeimask!(
    i126,
    i128,
    crate::typenum2::U126,
    "A 126 bit signed integer"
);
makeimask!(
    i127,
    i128,
    crate::typenum2::U127,
    "A 127 bit signed integer"
);
