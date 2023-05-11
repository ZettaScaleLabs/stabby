use crate::{str::Str, StableLike};

#[crate::stabby]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl TypeReport {
    pub fn fields(&self) -> Fields {
        Fields(self.fields.value)
    }
}
#[crate::stabby]
pub struct Fields(Option<&'static FieldReport>);
impl Iterator for Fields {
    type Item = &'static FieldReport;
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.0.take()?;
        self.0 = field.next_field.value;
        Some(field)
    }
}
