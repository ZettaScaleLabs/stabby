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
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() {
    let compiler_versions =
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("compiler_versions.rs");
    let mut compiler_versions = BufWriter::new(File::create(compiler_versions).unwrap());
    writeln!(compiler_versions, r"use crate::abi::IStable;").unwrap();
    for version in [
        "1.65.0", "1.66.0", "1.66.1", "1.67.0", "1.67.1", "1.68.0", "1.69.0", "1.70.0", "1.71.0",
        "1.72.0", "1.72.1", "1.73.0", "1.74.0", "1.74.1", "1.75.0", "1.76.0",
    ] {
        let snake_version = version.replace('.', "_");
        writeln!(
            compiler_versions,
            r#"
/// A ZST that allows inserting some information about the expected compiler for a type.
#[allow(non_camel_case_types)]
pub struct CompilerVersion_{snake_version}<Layout: IStable>(core::marker::PhantomData<Layout>);
impl<Layout: IStable> CompilerVersion_{snake_version}<Layout> {{
	/// The constructor for the compiler version.
	pub const UNIT: Self = Self(core::marker::PhantomData);
}}

#[rustversion::stable({version})]
/// This type alias resolves to the compiler that is currently in use to compile the crate
pub type LocalCompiler<Layout> = CompilerVersion_{snake_version}<Layout>;

#[rustversion::stable({version})]
unsafe impl<Layout: IStable> IStable for CompilerVersion_{snake_version}<Layout> {{
	type Size = Layout::Size;
	type Align = Layout::Align;
	type ForbiddenValues = Layout::ForbiddenValues;
	type UnusedBits = Layout::UnusedBits;
	type HasExactlyOneNiche = Layout::HasExactlyOneNiche;
	type ContainsIndirections = Layout::ContainsIndirections;
	const REPORT: &'static crate::abi::report::TypeReport = &crate::abi::report::TypeReport {{
		name: crate::abi::str::Str::new("CompilerVersion_{snake_version}"),
		module: crate::abi::str::Str::new(core::module_path!()),
		fields: crate::abi::StableLike::new(Some(&crate::abi::report::FieldReport {{
			name: crate::abi::str::Str::new("inner"),
			ty: <Layout as crate::abi::IStable>::REPORT,
			next_field: crate::abi::StableLike::new(None),
		}})),
		last_break: crate::abi::report::Version::NEVER,
		tyty: crate::abi::report::TyTy::Struct,
	}};
	const ID: u64 = crate::abi::istable::gen_id(Self::REPORT);
}}
"#
        )
        .unwrap();
    }
}
