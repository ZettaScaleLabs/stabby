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

use stabby::tuple::{Tuple2, Tuple3};

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
    D(u8),
    E,
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
    macro_rules! test {
        () => {};
        ($t: ty) => {
            dbg!(core::mem::size_of::<$t>());
            assert_eq!(core::mem::size_of::<$t>(), <$t as stabby::abi::IStable>::size(), "Size mismatch for {}", std::any::type_name::<$t>());
            assert_eq!(core::mem::align_of::<$t>(), <$t as stabby::abi::IStable>::align(), "Align mismatch for {}", std::any::type_name::<$t>());
        };
        ($t: ty, $($tt: tt)*) => {
            test!($t);
            test!($($tt)*);
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
        stabby::slice::Slice<'static, u8>,
        stabby::tuple::Tuple2<usize, usize>,
        stabby::tuple::Tuple2<usize, u8>,
        stabby::tuple::Tuple2<u8, usize>,
        stabby::abi::Union<u8, usize>,
        stabby::abi::Union<u8, ()>,
        stabby::abi::Union<(), u8>,
        UTest,
        FieldsC,
        FieldsStabby,
        MultiFieldsC,
        MultiFieldsStabby,
        stabby::tuple::Tuple2<(), usize>,
        stabby::tuple::Tuple2<usize, ()>,
        NoFields,
        WeirdStruct,
        WeirdStructBadLayout,
        Option<&'static u8>,
        Option<&'static mut u8>,
        Option<core::num::NonZeroI8>,
    );
}
