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

fn main() {
    use std::{
        fs::File,
        io::{BufWriter, Write},
        path::PathBuf,
    };
    const SEQ_MAX: u128 = 1000;
    let padding_rs = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("unsigned.rs");
    let mut padding_file = BufWriter::new(File::create(padding_rs).unwrap());
    for i in 0..=SEQ_MAX {
        let u = u(i);
        writeln!(padding_file, "/// {i}\npub type U{i} = {u};").unwrap();
        writeln!(padding_file, "/// {i}\npub type Ux{i:X} = {u};").unwrap();
        writeln!(padding_file, "/// {i}\npub type Ub{i:b} = {u};").unwrap();
    }
    for i in 0..39 {
        let ipow = 10u128.pow(i);
        let u = u(ipow);
        writeln!(padding_file, "/// {i}\npub type U10pow{i} = {u};").unwrap();
        if ipow > SEQ_MAX {
            writeln!(padding_file, "/// {i}\npub type U{ipow} = {u};").unwrap();
            writeln!(padding_file, "/// {i}\npub type Ux{ipow:X} = {u};").unwrap();
            writeln!(padding_file, "/// {i}\npub type Ub{ipow:b} = {u};").unwrap();
        }
    }
    for i in 0..128 {
        let u = u(1 << i);
        writeln!(padding_file, "/// {i}\npub type U2pow{i} = {u};").unwrap();
    }
    if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
        if toolchain.starts_with("nightly") {
            println!("cargo:rustc-cfg=stabby_nightly");
        }
    }
}
