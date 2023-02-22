pub use stabby_macros::stabby;
pub use stabby_traits;
pub mod slice;
pub mod tuple {
    pub use stabby_traits::type_layouts::Tuple2;
}
#[test]
fn layouts() {
    macro_rules! test {
        () => {};
        ($t: ty) => {
            <$t as stabby_traits::Stable>::layout_test();
        };
        ($t: ty, $($tt: tt)*) => {
            <$t as stabby_traits::Stable>::layout_test();
            test!($($tt)*);
        };
    }
    test!(
        u8,
        u16,
        u32,
        u64,
        usize,
        &'static u8,
        slice::Slice<'static, u8>,
        tuple::Tuple2<usize, usize>
    );
}
