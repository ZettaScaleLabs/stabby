use crate as stabby;
use core::ops::{Deref, DerefMut};

/// An ABI stable equivalent of `&'a T`
#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct Slice<'a, T: 'a> {
    start: &'a T,
    len: usize,
}
impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            start: unsafe { &*value.as_ptr() },
            len: value.len(),
        }
    }
}
impl<'a, T> From<&'a mut [T]> for Slice<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        Self {
            start: unsafe { &mut *value.as_mut_ptr() },
            len: value.len(),
        }
    }
}
impl<'a, T> From<Slice<'a, T>> for &'a [T] {
    fn from(value: Slice<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts(value.start, value.len) }
    }
}
impl<'a, T> Deref for Slice<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start, self.len) }
    }
}
impl<'a, T: 'a> Eq for Slice<'a, T> where for<'b> &'b [T]: Eq {}
impl<'a, T: 'a> PartialEq for Slice<'a, T>
where
    for<'b> &'b [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<'a, T: 'a> core::fmt::Debug for Slice<'a, T>
where
    for<'b> &'b [T]: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::fmt::Display for Slice<'a, T>
where
    for<'b> &'b [T]: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::hash::Hash for Slice<'a, T>
where
    for<'b> &'b [T]: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

/// An ABI stable equivalent of `&'a mut T`
#[stabby::stabby]
pub struct SliceMut<'a, T: 'a> {
    pub start: &'a mut T,
    pub len: usize,
}
impl<'a, T> Deref for SliceMut<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start, self.len) }
    }
}
impl<'a, T> DerefMut for SliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start, self.len) }
    }
}
impl<'a, T: 'a> Eq for SliceMut<'a, T> where for<'b> &'b [T]: Eq {}
impl<'a, T: 'a> PartialEq for SliceMut<'a, T>
where
    for<'b> &'b [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<'a, T: 'a> core::fmt::Debug for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::fmt::Display for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::hash::Hash for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}
impl<'a, T> From<&'a mut [T]> for SliceMut<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        Self {
            start: unsafe { &mut *value.as_mut_ptr() },
            len: value.len(),
        }
    }
}
impl<'a, T> From<SliceMut<'a, T>> for Slice<'a, T> {
    fn from(value: SliceMut<'a, T>) -> Self {
        Self {
            start: value.start,
            len: value.len,
        }
    }
}
impl<'a, T> From<SliceMut<'a, T>> for &'a mut [T] {
    fn from(value: SliceMut<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts_mut(value.start, value.len) }
    }
}

impl<'a, T> From<SliceMut<'a, T>> for &'a [T] {
    fn from(value: SliceMut<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts(value.start, value.len) }
    }
}
