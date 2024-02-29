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

#[test]
fn enums() {
    use core::num::{NonZeroU16, NonZeroU8};
    use stabby::{
        abi::{typenum2, IDeterminantProvider, IStable},
        result::Result,
        tuple::{Tuple2, Tuple3},
    };
    fn inner<A, B>(a: A, b: B, expected_size: usize)
    where
        A: Clone + PartialEq + core::fmt::Debug + IStable,
        B: Clone + PartialEq + core::fmt::Debug + IStable,
        A: IDeterminantProvider<B>,
        <A as IDeterminantProvider<B>>::Determinant: core::fmt::Debug,
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
            "discriminant: {}, OkShift: {}, ErrShift: {}, Debug: {}",
            core::any::type_name::<<A as IDeterminantProvider<B>>::Determinant>(),
            <<A as IDeterminantProvider<B>>::OkShift as typenum2::Unsigned>::USIZE,
            <<A as IDeterminantProvider<B>>::ErrShift as typenum2::Unsigned>::USIZE,
            0 // core::any::type_name::<<A as IDeterminantProvider<B>>::Debug>(),
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
    // inner(
    //     Tuple2(1u8, 2u16),
    //     Tuple2(3u8, NonZeroU8::new(4).unwrap()),
    //     4,
    // );
    // inner(
    //     Tuple2(3u8, NonZeroU8::new(4).unwrap()),
    //     Tuple2(1u8, 2u16),
    //     4,
    // );
    // inner(
    //     Tuple3(3u8, NonZeroU8::new(4).unwrap(), 6u16),
    //     Tuple2(1u8, 2u16),
    //     4,
    // );
    // inner(Tuple2(3u8, 4u16), Tuple2(1u8, 2u16), 4);
    // inner(3u16, Tuple2(1u8, 2u16), 4);
    // inner(1u8, NonZeroU16::new(6).unwrap(), 4);
    // let _: typenum2::U2 = <stabby::option::Option<NonZeroU16> as IStable>::Size::default();
    // let _: typenum2::U2 = <stabby::option::Option<u8> as IStable>::Size::default();
    // let _: typenum2::U1 = <stabby::option::Option<bool> as IStable>::Size::default();
    // inner(true, (), 1);
    // let string: stabby::string::String = stabby::string::String::from("Hi");
    // inner(
    //     string,
    //     stabby::str::Str::from("there"),
    //     core::mem::size_of::<stabby::string::String>(),
    // );
}
