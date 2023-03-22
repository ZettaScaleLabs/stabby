use core::ops::{Deref, DerefMut};

use crate::boxed::BoxedStr;

#[crate::stabby]
pub struct String {
    pub slice: BoxedStr,
    pub capacity: usize,
}
impl Clone for String {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice.clone(),
            capacity: self.capacity,
        }
    }
}
impl Eq for String {}
impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl core::fmt::Debug for String {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl core::fmt::Display for String {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl core::hash::Hash for String {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}
impl Deref for String {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.slice.deref()
    }
}
impl DerefMut for String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice.deref_mut()
    }
}
impl From<alloc::string::String> for String {
    fn from(value: alloc::string::String) -> Self {
        let capacity = value.capacity();
        let slice = value.into_boxed_str().into();
        Self { slice, capacity }
    }
}
impl From<String> for alloc::string::String {
    fn from(value: String) -> Self {
        let slice = BoxedStr::leak(value.slice);
        unsafe {
            alloc::string::String::from_raw_parts(
                slice.inner.start,
                slice.inner.len,
                value.capacity,
            )
        }
    }
}
