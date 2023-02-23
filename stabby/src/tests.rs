#[stabby_macros::stabby(in_stabby)]
pub union UTest {
    u8: u8,
    usize: usize,
}

#[stabby_macros::stabby(in_stabby)]
#[repr(u32)]
pub enum NoFields {
    _A,
    _B,
}
#[stabby_macros::stabby(in_stabby)]
#[repr(u8)]
pub enum Fields {
    _A(usize),
    _B,
}

#[stabby_macros::stabby(in_stabby)]
pub struct WeirdStructBadLayout {
    fields: Fields,
    no_fields: NoFields,
    utest: UTest,
    u32: u32,
}

#[stabby_macros::stabby(in_stabby)]
pub struct WeirdStruct {
    fields: Fields,
    no_fields: NoFields,
    u32: u32,
    utest: UTest,
}

#[stabby_macros::stabby(in_stabby)]
pub trait MyTrait {
    fn do_stuff<'a>(&'a self) -> &'a Self;
}

#[test]
fn layouts() {
    macro_rules! test {
    () => {};
    ($t: ty) => {
        assert_eq!(core::mem::size_of::<$t>(), <$t as stabby_traits::Stable>::size(), "Size mismatch for {}", std::any::type_name::<$t>());
        assert_eq!(core::mem::align_of::<$t>(), <$t as stabby_traits::Stable>::align(), "Align mismatch for {}", std::any::type_name::<$t>());
    };
    ($t: ty, $($tt: tt)*) => {
        test!($t);
        test!($($tt)*);
    };
}

    test!(
        u8,
        u16,
        u32,
        u64,
        usize,
        core::num::NonZeroU8,
        core::num::NonZeroU16,
        core::num::NonZeroU32,
        core::num::NonZeroU64,
        core::num::NonZeroUsize,
        i8,
        i16,
        i32,
        i64,
        isize,
        core::num::NonZeroI8,
        core::num::NonZeroI16,
        core::num::NonZeroI32,
        core::num::NonZeroI64,
        core::num::NonZeroIsize,
        &'static u8,
        &'static mut u8,
        crate::slice::Slice<'static, u8>,
        crate::tuple::Tuple2<usize, usize>,
        crate::tuple::Tuple2<usize, u8>,
        crate::tuple::Tuple2<u8, usize>,
        stabby_traits::type_layouts::Union<u8, usize>,
        stabby_traits::type_layouts::Union<u8, ()>,
        stabby_traits::type_layouts::Union<(), u8>,
        UTest,
        Fields,
        crate::tuple::Tuple2<(), usize>,
        crate::tuple::Tuple2<usize, ()>,
        NoFields,
        WeirdStruct,
        WeirdStructBadLayout,
        Option<&'static u8>,
        Option<&'static mut u8>,
        Option<core::num::NonZeroI8>,
    );
}
