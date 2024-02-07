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

use crate::{str::Str, StableLike};
use sha2_const_stable::Sha256;

#[crate::stabby]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub dirty: bool,
}
impl Version {
    pub const NEVER: Self = Self {
        major: 0,
        minor: 0,
        patch: 0,
        dirty: false,
    };
}
impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            major,
            minor,
            patch,
            dirty,
        } = self;
        if *dirty {
            write!(f, "*")?;
        }
        write!(f, "{major}.{minor}.{patch}")
    }
}

type NextField = StableLike<Option<&'static FieldReport>, usize>;

#[crate::stabby]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TypeReport {
    pub name: Str<'static>,
    pub module: Str<'static>,
    pub fields: NextField,
    pub last_break: Version,
    pub tyty: TyTy,
}

impl core::fmt::Display for TypeReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            name,
            module,
            last_break,
            tyty,
            ..
        } = self;
        write!(f, "{tyty:?} {module} :: {name} (last_break{last_break}) {{")?;
        for FieldReport { name, ty, .. } in self.fields() {
            write!(f, "{name}: {ty}, ")?
        }
        write!(f, "}}")
    }
}
impl core::hash::Hash for TypeReport {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.module.hash(state);
        for field in self.fields() {
            field.hash(state);
        }
        self.last_break.hash(state);
        self.tyty.hash(state);
    }
}

impl TypeReport {
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.name == other.name
            && self.module == other.module
            && self.last_break == other.last_break
            && self.tyty == other.tyty
            && self
                .fields()
                .zip(other.fields())
                .all(|(s, o)| s.name == o.name && s.ty.is_compatible(o.ty))
    }
}

#[crate::stabby]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TyTy {
    Struct,
    Enum(Str<'static>),
    Union,
}

#[crate::stabby]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FieldReport {
    pub name: Str<'static>,
    pub ty: &'static TypeReport,
    pub next_field: NextField,
}
impl core::hash::Hash for FieldReport {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ty.hash(state);
    }
}

impl TypeReport {
    pub const fn fields(&self) -> Fields {
        Fields(self.fields.value)
    }
}
#[crate::stabby]
pub struct Fields(Option<&'static FieldReport>);
impl Fields {
    pub const fn next_const(self) -> (Self, Option<&'static FieldReport>) {
        match self.0 {
            Some(field) => (Self(*field.next_field.as_ref()), Some(field)),
            None => (self, None),
        }
    }
}
impl Iterator for Fields {
    type Item = &'static FieldReport;
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.0.take()?;
        self.0 = field.next_field.value;
        Some(field)
    }
}

const fn hash_report(report: &TypeReport) -> Sha256 {
    let mut hash = Sha256::new()
        .update(report.module.as_str().as_bytes())
        .update(report.name.as_str().as_bytes());
    hash = match report.tyty {
        crate::report::TyTy::Struct => hash.update(&[0]),
        crate::report::TyTy::Union => hash.update(&[1]),
        crate::report::TyTy::Enum(s) => hash.update(s.as_str().as_bytes()),
    };
    let mut fields = report.fields();
    while let (new, Some(next)) = fields.next_const() {
        fields = new;
        hash = hash
            .update(next.name.as_str().as_bytes())
            .update(&hash_report(next.ty).finalize())
    }
    hash
}
/// Generates an ID based on a [`TypeReport`] using [`Sha256`]
pub const fn gen_id(report: &TypeReport) -> u64 {
    let [hash @ .., _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] =
        hash_report(report).finalize();
    u64::from_le_bytes(hash)
}
