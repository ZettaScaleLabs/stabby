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
//   Pierre Avital, <pierre.avital@me.com>
//

use core::num::{NonZeroU16, NonZeroU32};
use std::num::NonZeroU8;

use stabby::tuple::{Tuple2, Tuple3, Tuple8};
use stabby_abi::{typenum2::*, Array, End, Result};

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

#[stabby::stabby]
pub enum MultiFieldsStabby {
    A(NonZeroU16),
    B,
    C(Tuple2<u8, u16>),
    D(u32),
    E,
}
#[stabby::stabby]
pub enum SameFieldsFourTimes<T> {
    A(T),
    B(T),
    C(T),
    D(T),
}

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
    use stabby::abi::istable::IForbiddenValues;
    macro_rules! test {
        () => {};
        ($t: ty, $unused: ty, $illegal: ty) => {
            test!($t);
            let _: core::mem::MaybeUninit<$unused> =
                core::mem::MaybeUninit::<<$t as stabby::abi::IStable>::UnusedBits>::uninit();
            let _: core::mem::MaybeUninit<$illegal> = core::mem::MaybeUninit::<
                <<$t as stabby::abi::IStable>::ForbiddenValues as IForbiddenValues>::SelectOne,
            >::uninit();
        };
        ($t: ty) => {
            dbg!(core::mem::size_of::<$t>());
            assert_eq!(
                core::mem::size_of::<$t>(),
                <$t as stabby::abi::IStable>::size(),
                "Size mismatch for {}",
                std::any::type_name::<$t>()
            );
            assert_eq!(
                core::mem::align_of::<$t>(),
                <$t as stabby::abi::IStable>::align(),
                "Align mismatch for {}",
                std::any::type_name::<$t>()
            );
        };
    }

    let value = MultiFieldsStabby::D(5);
    value.match_ref(
        |_| panic!(),
        || panic!(),
        |_| panic!(),
        |&v| assert_eq!(v, 5),
        || panic!(),
    );
    value.match_owned(
        |_| panic!(),
        || panic!(),
        |_| panic!(),
        |v| assert_eq!(v, 5),
        || panic!(),
    );

    test!(bool, End, Array<U0, U2, End>);
    test!(u8, End, End);
    test!(u16, End, End);
    test!(u32, End, End);
    test!(u64, End, End);
    test!(u128, End, End);
    test!(usize, End, End);
    test!(core::num::NonZeroU8, End, Array<U0, U0, End>);
    test!(core::num::NonZeroU16, End, Array<U0, U0, Array<U1, U0, End>>);
    test!(core::num::NonZeroU32, End, _);
    test!(core::num::NonZeroU64, End, _);
    test!(core::num::NonZeroU128, End, _);
    test!(core::num::NonZeroUsize, End, _);
    test!(i8, End, End);
    test!(i16, End, End);
    test!(i32, End, End);
    test!(i64, End, End);
    test!(i128, End, End);
    test!(isize, End, End);
    test!(core::num::NonZeroI8, End, _);
    test!(core::num::NonZeroI16, End, _);
    test!(core::num::NonZeroI32, End, _);
    test!(core::num::NonZeroI64, End, _);
    test!(core::num::NonZeroI128, End, _);
    test!(core::num::NonZeroIsize, End, _);
    test!(&'static u8, End, _);
    test!(&'static mut u8, End, _);
    test!(stabby::slice::Slice<'static, u8>, End, _);
    test!(stabby::tuple::Tuple2<usize, usize>, End, End);
    test!(stabby::tuple::Tuple2<u32, u8>, Array<U5, UxFF, Array<U6, UxFF, Array<U7, UxFF, End>>>, End);
    test!(stabby::tuple::Tuple2<u8, u32>, Array<U1, UxFF, Array<U2, UxFF, Array<U3, UxFF, End>>>, End);
    test!(stabby::tuple::Tuple2<u8, Tuple2<u8, u32>>, Array<U1, UxFF, Array<U2, UxFF, Array<U3, UxFF, Array<U5, UxFF, Array<U6, UxFF, Array<U7, UxFF, End>>>>>>, End);
    test!(stabby::abi::Union<u8, usize>, End, End);
    test!(stabby::abi::Union<u8, ()>, End, End);
    test!(stabby::abi::Union<(), u8>, End, End);
    test!(stabby::result::Result<(), ()>, Array<U0, Ub11111110, End>, End);
    test!(UTest, End, End);
    test!(FieldsC, Array<U1, UxFF, Array<U2, UxFF, Array<U3, UxFF, End>>>, End);
    test!(FieldsStabby, End, End);
    test!(MultiFieldsC, Array<U1, UxFF, End>, End);
    test!(Result<u32, ()>, Array<U0, Ub11111110, Array<U1, UxFF, Array<U2, UxFF, Array<U3, UxFF, End>>>>, End);
    test!(Result<Tuple2<u8, u16>, Result<u32, ()>>, Array<U1, Ub11111110, End>, End);
    test!(Result<NonZeroU16, ()>, End, End);
    test!(MultiFieldsStabby, Array<U1, Ub11111100, _>, End);
    test!(stabby::tuple::Tuple2<(), usize>, End, End);
    test!(stabby::tuple::Tuple2<usize, ()>, End, End);
    test!(NoFields);
    test!(WeirdStruct);
    test!(WeirdStructBadLayout);
    test!(Option<&'static u8>, End, End);
    test!(Option<&'static mut u8>, End, End);
    test!(Option<core::num::NonZeroI8>, End, End);
    // Ensure that only 8 positions are tried before giving switching to external tags
    assert_eq!(core::mem::size_of::<Tuple2<u64, Align128>>(), 2 * 16);
    assert_eq!(
        core::mem::size_of::<
            Tuple2<
                Tuple8<NonZeroU8, u8, u8, u8, u8, u8, u8, u8>,
                Tuple8<u8, u8, u8, u8, u8, u8, u8, u8>,
            >,
        >(),
        16
    );
    assert_eq!(
        core::mem::size_of::<
            Result<
                Tuple2<u64, Align128>,
                Tuple2<
                    Tuple8<NonZeroU8, u8, u8, u8, u8, u8, u8, u8>,
                    Tuple8<u8, u8, u8, u8, u8, u8, u8, u8>,
                >,
            >,
        >(),
        3 * 16
    );
}
#[repr(align(16))]
struct Align128(u128);
unsafe impl stabby::abi::IStable for Align128 {
    type Size = U16;
    type Align = U16;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
    type ContainsIndirections = B0;
    const REPORT: &'static stabby::abi::report::TypeReport = &stabby::abi::report::TypeReport {
        name: stabby::abi::str::Str::new("Align128"),
        module: stabby::abi::str::Str::new(core::module_path!()),
        fields: stabby::abi::StableLike::new(None),
        version: 0,
        tyty: stabby::abi::report::TyTy::Struct,
    };
    const ID: u64 = stabby::abi::report::gen_id(Self::REPORT);
}
