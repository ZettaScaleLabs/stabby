#[stabby_macros::stabby(in_stabby)]
#[derive(Clone, Copy)]
pub struct Slice<'a, T> {
    start: &'a T,
    len: usize,
}
#[stabby_macros::stabby(in_stabby)]
pub struct SliceMut<'a, T> {
    start: &'a mut T,
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
impl<'a, T> From<SliceMut<'a, T>> for Slice<'a, T> {
    fn from(value: SliceMut<'a, T>) -> Self {
        Self {
            start: value.start,
            len: value.len,
        }
    }
}
impl<'a, T> From<Slice<'a, T>> for &'a [T] {
    fn from(value: Slice<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts(value.start, value.len) }
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
