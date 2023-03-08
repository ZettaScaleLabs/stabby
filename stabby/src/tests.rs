use crate as stabby;

#[stabby::stabby]
pub union UTest {
    u8: u8,
    usize: usize,
}

#[stabby::stabby]
#[repr(u32)]
pub enum NoFields {
    _A,
    _B,
}
#[stabby::stabby]
#[repr(C)]
pub enum Fields {
    _A(usize),
    _B,
}

#[stabby::stabby(no_opt)]
pub struct WeirdStructBadLayout {
    fields: Fields,
    no_fields: NoFields,
    utest: UTest,
    u32: u32,
}

#[stabby::stabby]
pub struct WeirdStruct {
    fields: Fields,
    no_fields: NoFields,
    u32: u32,
    utest: UTest,
}

#[stabby::stabby]
fn somefunc(_: u8) -> u8 {
    0
}

#[test]
fn layouts() {
    assert!(WeirdStruct::has_optimal_layout());
    assert!(!WeirdStructBadLayout::has_optimal_layout());

    macro_rules! test {
        () => {};
        ($t: ty) => {
            assert_eq!(core::mem::size_of::<$t>(), <$t as crate::abi::IStable>::size(), "Size mismatch for {}", std::any::type_name::<$t>());
            assert_eq!(core::mem::align_of::<$t>(), <$t as crate::abi::IStable>::align(), "Align mismatch for {}", std::any::type_name::<$t>());
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
        u128,
        usize,
        core::num::NonZeroU8,
        core::num::NonZeroU16,
        core::num::NonZeroU32,
        core::num::NonZeroU64,
        core::num::NonZeroU128,
        core::num::NonZeroUsize,
        i8,
        i16,
        i32,
        i64,
        i128,
        isize,
        core::num::NonZeroI8,
        core::num::NonZeroI16,
        core::num::NonZeroI32,
        core::num::NonZeroI64,
        core::num::NonZeroI128,
        core::num::NonZeroIsize,
        &'static u8,
        &'static mut u8,
        crate::slice::Slice<'static, u8>,
        crate::tuple::Tuple2<usize, usize>,
        crate::tuple::Tuple2<usize, u8>,
        crate::tuple::Tuple2<u8, usize>,
        crate::abi::Union<u8, usize>,
        crate::abi::Union<u8, ()>,
        crate::abi::Union<(), u8>,
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

// MYTRAIT

#[stabby::stabby]
pub trait MyTrait {
    type Output;
    extern "C" fn do_stuff<'a>(&'a self, with: &Self::Output) -> &'a u8;
    extern "C" fn gen_stuff(&mut self) -> Self::Output;
}

// IMPL

impl MyTrait for u8 {
    type Output = u8;
    extern "C" fn do_stuff<'a>(&'a self, _: &Self::Output) -> &'a u8 {
        self
    }
    extern "C" fn gen_stuff(&mut self) -> Self::Output {
        *self
    }
}
impl MyTrait for u16 {
    type Output = u8;
    extern "C" fn do_stuff<'a>(&'a self, _: &Self::Output) -> &'a u8 {
        &0
    }
    extern "C" fn gen_stuff(&mut self) -> Self::Output {
        *self as u8
    }
}

// MYTRAIT2
#[stabby::stabby]
pub trait MyTrait2 {
    extern "C" fn do_stuff2(&self) -> u8;
}

// IMPL

impl MyTrait2 for u8 {
    extern "C" fn do_stuff2(&self) -> u8 {
        *self
    }
}
impl MyTrait2 for u16 {
    extern "C" fn do_stuff2(&self) -> u8 {
        (*self) as u8
    }
}

#[stabby::stabby]
pub trait MyTrait3<Hi: core::ops::Deref> {
    type A;
    type B;
    extern "C" fn do_stuff3<'a>(&'a self, a: &'a Self::A, b: Self::B) -> Self::B;
    extern "C" fn gen_stuff3(&mut self, with: Hi) -> Self::A;
}

impl MyTrait3<Box<()>> for u8 {
    type A = u8;
    type B = u8;
    extern "C" fn do_stuff3<'a>(&'a self, _a: &'a Self::A, _b: Self::B) -> Self::B {
        *self
    }
    extern "C" fn gen_stuff3(&mut self, _with: Box<()>) -> Self::A {
        *self
    }
}
impl MyTrait3<Box<()>> for u16 {
    type A = u8;
    type B = u8;
    extern "C" fn do_stuff3<'a>(&'a self, _a: &'a Self::A, _b: Self::B) -> Self::B {
        (*self) as u8
    }
    extern "C" fn gen_stuff3(&mut self, _with: Box<()>) -> Self::A {
        (*self) as u8
    }
}

#[test]
fn dyn_traits() {
    let boxed = Box::new(6u8);
    let mut dyned = crate::abi::Dyn::<
        _,
        stabby::vtable!(
            Send + MyTrait2 + MyTrait3<Box<()>, A = u8, B = u8> + Sync + MyTrait<Output = u8>
        ),
    >::from(boxed);
    assert_eq!(dyned.downcast_ref::<u8>(), Some(&6));
    assert_eq!(dyned.do_stuff(&0), &6);
    assert_eq!(dyned.gen_stuff(), 6);
    assert_eq!(dyned.gen_stuff3(Box::new(())), 6);
    assert!(dyned.downcast_ref::<u16>().is_none());
    fn trait_assertions<T: Send + Sync + stabby::abi::IStable>(_t: T) {}
    trait_assertions(dyned);
}
