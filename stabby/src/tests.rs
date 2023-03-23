//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use core::num::{NonZeroU16, NonZeroU32};

use crate::{
    self as stabby,
    slice::SliceMut,
    tuple::{Tuple2, Tuple3},
};

#[stabby::stabby]
pub union UTest {
    u8: u8,
    usize: usize,
}
#[stabby::stabby]
pub union UTest2 {
    usize: usize,
    u32s: Tuple3<u32, u32, u32>,
}

#[stabby::stabby]
#[repr(u32)]
pub enum NoFields {
    _A,
    _B,
}
#[stabby::stabby]
#[repr(C)]
pub enum FieldsC {
    _A(NonZeroU32),
    _B,
}
#[stabby::stabby]
pub enum FieldsStabby {
    _A(NonZeroU32),
    _B,
}
#[stabby::stabby]
#[repr(C)]
#[allow(dead_code)]
pub enum MultiFieldsC {
    A(NonZeroU16),
    B,
    C(Tuple2<u8, u16>),
    D(u8),
    E,
}
// #[stabby::stabby]
// pub enum MultiFieldsStabby {
//     A(NonZeroU16),
//     B,
//     C(Tuple2<u8, u16>),
//     D(u8),
//     E,
// }

#[stabby::stabby(no_opt)]
pub struct WeirdStructBadLayout {
    fields: FieldsC,
    no_fields: NoFields,
    utest: UTest,
    u32: u32,
}

#[stabby::stabby]
pub struct WeirdStructBadLayout2 {
    fields: FieldsC,
    no_fields: NoFields,
    utest: UTest,
}

#[stabby::stabby]
pub struct WeirdStruct {
    fields: FieldsC,
    no_fields: NoFields,
    u32: u32,
    utest: UTest,
}

#[stabby::stabby]
fn somefunc(_: u8) -> u8 {
    0
}
#[stabby::stabby]
pub struct Test {
    b: u8,
    a: u32,
}

#[test]
fn layouts() {
    macro_rules! test {
        () => {};
        ($t: ty) => {
            dbg!(core::mem::size_of::<$t>());
            assert_eq!(core::mem::size_of::<$t>(), <$t as crate::abi::IStable>::size(), "Size mismatch for {}", std::any::type_name::<$t>());
            assert_eq!(core::mem::align_of::<$t>(), <$t as crate::abi::IStable>::align(), "Align mismatch for {}", std::any::type_name::<$t>());
        };
        ($t: ty, $($tt: tt)*) => {
            test!($t);
            test!($($tt)*);
        };
    }

    // let value = MultiFieldsStabby::D(5);
    // value.match_ref(
    //     |_| panic!(),
    //     || panic!(),
    //     |_| panic!(),
    //     |&v| assert_eq!(v, 5),
    //     || panic!(),
    // );
    // value.match_owned(
    //     |_| panic!(),
    //     || panic!(),
    //     |_| panic!(),
    //     |v| assert_eq!(v, 5),
    //     || panic!(),
    // );

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
        FieldsC,
        FieldsStabby,
        MultiFieldsC,
        // MultiFieldsStabby,
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

#[stabby::stabby]
pub trait AsyncRead {
    extern "C" fn read<'a>(
        &'a mut self,
        buffer: crate::slice::SliceMut<'a, u8>,
    ) -> crate::future::DynFuture<'a, usize>;
}
impl<'b> AsyncRead for SliceMut<'b, u8> {
    extern "C" fn read<'a>(
        &'a mut self,
        mut buffer: stabby::slice::SliceMut<'a, u8>,
    ) -> stabby::future::DynFuture<'a, usize> {
        Box::new(async move {
            let len = self.len().min(buffer.len());
            let (l, r) = self.split_at_mut(len);
            let r = unsafe { core::mem::transmute::<_, &mut [u8]>(r) };
            buffer[..len].copy_from_slice(l);
            *self = r.into();
            len
        })
        .into()
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

#[test]
fn enums() {
    use crate::{
        abi::{typenum2, IDiscriminantProvider, IStable},
        result::Result,
        tuple::Tuple2,
    };
    use core::num::{NonZeroU16, NonZeroU8};
    fn inner<A, B>(a: A, b: B, expected_size: usize)
    where
        A: Clone + PartialEq + core::fmt::Debug + IStable,
        B: Clone + PartialEq + core::fmt::Debug + IStable,
        A: IDiscriminantProvider<B>,
        <A as IDiscriminantProvider<B>>::Discriminant: core::fmt::Debug,
        Result<A, B>: IStable,
    {
        println!(
            "Testing: {}({a:?}) | {}({b:?})",
            core::any::type_name::<A>(),
            core::any::type_name::<B>()
        );
        let ac = a.clone();
        let bc = b.clone();
        let a: core::result::Result<A, B> = Ok(a);
        let b: core::result::Result<A, B> = Err(b);
        let a: Result<_, _> = a.into();
        println!(
            "discriminant: {}, OkShift: {}, ErrShift: {}",
            core::any::type_name::<<A as IDiscriminantProvider<B>>::Discriminant>(),
            <<A as IDiscriminantProvider<B>>::OkShift as typenum2::Unsigned>::USIZE,
            <<A as IDiscriminantProvider<B>>::ErrShift as typenum2::Unsigned>::USIZE,
        );
        assert!(a.is_ok());
        let b: Result<_, _> = b.into();
        assert!(b.is_err());
        assert_eq!(a, Result::Ok(ac.clone()));
        assert_eq!(a.unwrap(), ac);
        assert_eq!(b, Result::Err(bc.clone()));
        assert_eq!(b.unwrap_err(), bc);
        assert_eq!(<Result<A, B> as IStable>::size(), expected_size);
        println!()
    }
    inner(8u8, 2u8, 2);
    let _: typenum2::U2 = <Result<u8, u8> as IStable>::Size::default();
    let _: typenum2::U2 = <Result<Result<u8, u8>, Result<u8, u8>> as IStable>::Size::default();
    inner(Tuple2(1u8, 2u16), Tuple2(3u16, 4u16), 6);
    inner(
        Tuple2(1u8, 2u16),
        Tuple2(3u8, NonZeroU8::new(4).unwrap()),
        4,
    );
    inner(
        Tuple2(3u8, NonZeroU8::new(4).unwrap()),
        Tuple2(1u8, 2u16),
        4,
    );
    inner(
        Tuple3(3u8, NonZeroU8::new(4).unwrap(), 6u16),
        Tuple2(1u8, 2u16),
        4,
    );
    inner(Tuple2(3u8, 4u16), Tuple2(1u8, 2u16), 4);
    inner(3u16, Tuple2(1u8, 2u16), 4);
    inner(1u8, NonZeroU16::new(6).unwrap(), 4);
    let _: typenum2::U2 = <crate::option::Option<NonZeroU16> as IStable>::Size::default();
    let _: typenum2::U2 = <crate::option::Option<u8> as IStable>::Size::default();
    let _: typenum2::U1 = <crate::option::Option<bool> as IStable>::Size::default();
    inner(true, (), 1);
    inner(
        crate::string::String::from("Hi".to_owned()),
        crate::str::Str::from("there"),
        core::mem::size_of::<crate::string::String>(),
    );
}
