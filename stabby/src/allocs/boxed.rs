use crate::slice::SliceMut;
use crate::str::StrMut;
use crate::{self as stabby, type_layouts::IntoDyn};
use core::ops::{Deref, DerefMut};

#[stabby::stabby]
pub struct BoxedSlice<T> {
    start: &'static mut (),
    len: usize,
    marker: core::marker::PhantomData<Box<T>>,
}
impl<T> BoxedSlice<T> {
    pub(crate) fn leak<'a>(self) -> SliceMut<'a, T> {
        let r = SliceMut {
            start: unsafe { core::mem::transmute_copy(&self.start) },
            len: self.len,
        };
        core::mem::forget(self);
        r
    }
}
impl<T: Clone> Clone for BoxedSlice<T> {
    fn clone(&self) -> Self {
        Box::from_iter(self.iter().cloned()).into()
    }
}
impl<T> Deref for BoxedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start as *const () as *const T, self.len) }
    }
}
impl<T> DerefMut for BoxedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start as *mut () as *mut T, self.len) }
    }
}
impl<T> From<Box<[T]>> for BoxedSlice<T> {
    fn from(value: Box<[T]>) -> Self {
        let value = Box::leak(value);
        let len = value.len();
        let start = unsafe { &mut *(value.as_ptr() as *mut ()) };
        Self {
            start,
            len,
            marker: core::marker::PhantomData,
        }
    }
}
impl<T> From<BoxedSlice<T>> for Box<[T]> {
    fn from(value: BoxedSlice<T>) -> Self {
        unsafe { Box::from_raw(&mut *value.leak()) }
    }
}
impl<T> Drop for BoxedSlice<T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(&mut *self) };
    }
}

#[stabby::stabby]
#[derive(Clone)]
pub struct BoxedStr {
    inner: BoxedSlice<u8>,
}
impl BoxedStr {
    pub(crate) fn leak(self) -> StrMut<'static> {
        StrMut {
            inner: self.inner.leak(),
        }
    }
}
impl Deref for BoxedStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}
impl DerefMut for BoxedStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.inner) }
    }
}
impl From<Box<str>> for BoxedStr {
    fn from(value: Box<str>) -> Self {
        let value: Box<[u8]> = value.into();
        Self {
            inner: value.into(),
        }
    }
}
impl From<BoxedStr> for Box<str> {
    fn from(value: BoxedStr) -> Self {
        unsafe { Box::from_raw(&mut *value.leak()) }
    }
}

impl stabby::type_layouts::IPtr for Box<()> {
    unsafe fn as_ref<U>(&self) -> &U {
        let this: &() = self;
        core::mem::transmute(this)
    }
}
impl stabby::type_layouts::IPtrMut for Box<()> {
    unsafe fn as_mut<U>(&mut self) -> &mut U {
        let this: &mut () = self;
        core::mem::transmute(this)
    }
}
impl stabby::type_layouts::IPtrOwned for Box<()> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        unsafe {
            (drop)(this);
            core::mem::ManuallyDrop::drop(this);
        }
    }
}

impl<T> IntoDyn for Box<T> {
    type Anonymized = Box<()>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        unsafe { core::mem::transmute(self) }
    }
}
