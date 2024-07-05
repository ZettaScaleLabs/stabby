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

use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn u(mut i: u128) -> String {
    let mut result = "UTerm".into();
    let mut ids = Vec::new();
    while i > 0 {
        let bit = i & 1;
        ids.push(bit as u8);
        i >>= 1;
    }
    for bit in ids.into_iter().rev() {
        result = format!("UInt<{result}, B{bit}>");
    }
    result
}

fn typenum_unsigned() -> std::io::Result<()> {
    const SEQ_MAX: u128 = 1000;
    let filename = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("unsigned.rs");
    let mut file = BufWriter::new(File::create(filename).unwrap());
    for i in 0..=SEQ_MAX {
        let u = u(i);
        writeln!(file, "/// {i}\npub type U{i} = {u};")?;
        writeln!(file, "/// {i}\npub type Ux{i:X} = {u};")?;
        writeln!(file, "/// {i}\npub type Ub{i:b} = {u};")?;
    }
    for i in 0..39 {
        let ipow = 10u128.pow(i);
        let u = u(ipow);
        writeln!(file, "/// {i}\npub type U10pow{i} = {u};")?;
        if ipow > SEQ_MAX {
            writeln!(file, "/// {i}\npub type U{ipow} = {u};")?;
            writeln!(file, "/// {i}\npub type Ux{ipow:X} = {u};")?;
            writeln!(file, "/// {i}\npub type Ub{ipow:b} = {u};")?;
        }
    }
    for i in 0..128 {
        let p = 1 << i;
        let u = u(p);
        writeln!(file, "/// {i}\npub type U2pow{i} = {u};")?;
        if p > SEQ_MAX {
            writeln!(file, "/// {i}\npub type U{p} = {u};")?;
        }
    }
    Ok(())
}

fn tuples(max_tuple: usize) -> std::io::Result<()> {
    let filename = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("tuples.rs");
    let mut file = BufWriter::new(File::create(filename).unwrap());
    for i in 0..=max_tuple {
        writeln!(
            file,
            r##"/// An ABI stable tuple of {i} elements.
#[crate::stabby]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Tuple{i}<{generics}>({fields});
impl<{generics}> From<({generics})> for Tuple{i}<{generics}> {{
    fn from(value: ({generics})) -> Self {{
        let ({named_fields}) = value;
        Self({named_fields})
    }}
}}
#[allow(clippy::unused_unit)]
impl<{generics}> From<Tuple{i}<{generics}>> for ({generics}) {{
    fn from(value: Tuple{i}<{generics}>) -> Self {{
        let Tuple{i}({named_fields}) = value;
        ({named_fields})
    }}
}}
"##,
            generics = (0..i).fold(String::new(), |mut acc, it| {
                write!(acc, "T{it}, ").unwrap();
                acc
            }),
            fields = (0..i).fold(String::new(), |mut acc, it| {
                write!(acc, "pub T{it}, ").unwrap();
                acc
            }),
            named_fields = (0..i).fold(String::new(), |mut acc, it| {
                write!(acc, "field{it}, ").unwrap();
                acc
            }),
        )?;
    }
    Ok(())
}

fn main() {
    typenum_unsigned().unwrap();
    println!("cargo:rustc-check-cfg=cfg(stabby_max_tuple, values(any()))");
    let max_tuple = std::env::var("CARGO_CFG_STABBY_MAX_TUPLE")
        .map_or(32, |s| s.parse().unwrap_or(32))
        .max(10);
    tuples(max_tuple).unwrap();
    println!("cargo:rustc-check-cfg=cfg(stabby_nightly, values(none()))");
    println!(
        r#"cargo:rustc-check-cfg=cfg(stabby_default_alloc, values(none(), "RustAlloc", "LibcAlloc"))"#
    );
    println!(
        r#"cargo:rustc-check-cfg=cfg(stabby_check_unreachable, values(none(), "true", "false"))"#
    );
    println!(r#"cargo:rustc-check-cfg=cfg(stabby_unsafe_wakers, values(none(), "true", "false"))"#);
    println!(
        r#"cargo:rustc-check-cfg=cfg(stabby_vtables, values(none(), "vec", "btree", "no_alloc"))"#
    );
    if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
        if toolchain.starts_with("nightly") {
            println!("cargo:rustc-cfg=stabby_nightly");
        }
    }
}
