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
    let padding_rs = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("unsigned.rs");
    let mut padding_file = BufWriter::new(File::create(padding_rs).unwrap());
    const SEQ_MAX: u128 = 1000;
    for i in 0..=SEQ_MAX {
        let u = u(i);
        writeln!(padding_file, "pub type U{i} = {u};").unwrap();
        writeln!(padding_file, "pub type Ux{i:X} = {u};").unwrap();
        writeln!(padding_file, "pub type Ub{i:b} = {u};").unwrap();
    }
    for i in 0..39 {
        let ipow = 10u128.pow(i);
        let u = u(ipow);
        writeln!(padding_file, "pub type U10pow{i} = {u};").unwrap();
        if ipow > SEQ_MAX {
            writeln!(padding_file, "pub type U{ipow} = {u};").unwrap();
            writeln!(padding_file, "pub type Ux{ipow:X} = {u};").unwrap();
            writeln!(padding_file, "pub type Ub{ipow:b} = {u};").unwrap();
        }
    }
    for i in 0..128 {
        let u = u(1 << i);
        writeln!(padding_file, "pub type U2pow{i} = {u};").unwrap();
    }

    let compiler_versions =
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("compiler_versions.rs");
    let mut compiler_versions = BufWriter::new(File::create(compiler_versions).unwrap());
    writeln!(compiler_versions, r"use crate::IStable;").unwrap();
    for version in ["1.65.0", "1.66.0", "1.66.1", "1.67.0", "1.67.1", "1.68.0"] {
        let snake_version = version.replace('.', "_");
        writeln!(
            compiler_versions,
            r#"
#[allow(non_camel_case_types)]
pub struct CompilerVersion_{snake_version}<Layout: IStable>(core::marker::PhantomData<Layout>);
impl<Layout: IStable> CompilerVersion_{snake_version}<Layout> {{
    pub const UNIT: Self = Self(core::marker::PhantomData);
}}
#[rustversion::stable({version})]
unsafe impl<Layout: IStable> crate::IStable for CompilerVersion_{snake_version}<Layout> {{
    type Size = Layout::Size;
    type Align = Layout::Align;
    type ForbiddenValues = Layout::ForbiddenValues;
    type UnusedBits = Layout::UnusedBits;
    type HasExactlyOneNiche = Layout::HasExactlyOneNiche;
    primitive_report!("CompilerVersion_{snake_version}");
}}
"#
        )
        .unwrap();
    }
}
