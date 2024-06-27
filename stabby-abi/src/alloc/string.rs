use super::{
    boxed::BoxedSlice,
    sync::{ArcSlice, WeakSlice},
    vec::Vec,
    AllocationError, IAlloc,
};
use core::hash::Hash;

/// A growable owned string.
#[crate::stabby]
#[derive(Clone)]
pub struct String<Alloc: IAlloc = super::DefaultAllocator> {
    pub(crate) inner: Vec<u8, Alloc>,
}

#[cfg(feature = "libc")]
impl String {
    /// Constructs a new string using the default allocator.
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }
}
impl<Alloc: IAlloc> String<Alloc> {
    /// Constructs a new string using the provided allocator.
    pub const fn new_in(alloc: Alloc) -> Self {
        Self {
            inner: Vec::new_in(alloc),
        }
    }
    /// Returns self as a borrowed string
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
    }
    /// Returns self as a mutably borrowed string
    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.inner.as_slice_mut()) }
    }
    fn try_concat_str(&mut self, s: &str) -> Result<(), AllocationError> {
        self.inner.try_copy_extend(s.as_bytes())
    }
    /// Attempts to concatenate `s` to `self`
    /// # Errors
    /// This returns an [`AllocationError`] if reallocation was needed and failed to concatenate.
    pub fn try_concat<S: AsRef<str> + ?Sized>(&mut self, s: &S) -> Result<(), AllocationError> {
        self.try_concat_str(s.as_ref())
    }
}
impl<Alloc: IAlloc + Default> Default for String<Alloc> {
    fn default() -> Self {
        Self {
            inner: Vec::default(),
        }
    }
}
impl<S: AsRef<str> + ?Sized, Alloc: IAlloc> core::ops::Add<&S> for String<Alloc> {
    type Output = Self;
    fn add(mut self, rhs: &S) -> Self::Output {
        self += rhs.as_ref();
        self
    }
}
impl<S: AsRef<str> + ?Sized, Alloc: IAlloc> core::ops::AddAssign<&S> for String<Alloc> {
    fn add_assign(&mut self, rhs: &S) {
        self.inner.copy_extend(rhs.as_ref().as_bytes())
    }
}

impl<Alloc: IAlloc> From<String<Alloc>> for Vec<u8, Alloc> {
    fn from(value: String<Alloc>) -> Self {
        value.inner
    }
}

impl<Alloc: IAlloc> TryFrom<Vec<u8, Alloc>> for String<Alloc> {
    type Error = core::str::Utf8Error;
    fn try_from(value: Vec<u8, Alloc>) -> Result<Self, Self::Error> {
        core::str::from_utf8(value.as_slice())?;
        Ok(Self { inner: value })
    }
}

impl<Alloc: IAlloc> core::ops::Deref for String<Alloc> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<Alloc: IAlloc> core::convert::AsRef<str> for String<Alloc> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl<Alloc: IAlloc> core::ops::DerefMut for String<Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl<Alloc: IAlloc> core::fmt::Debug for String<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}
impl<Alloc: IAlloc> core::fmt::Display for String<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}
impl<Alloc: IAlloc> Hash for String<Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}
impl<Alloc: IAlloc, Rhs: AsRef<str>> PartialEq<Rhs> for String<Alloc> {
    fn eq(&self, other: &Rhs) -> bool {
        self.as_str() == other.as_ref()
    }
}
impl<Alloc: IAlloc> Eq for String<Alloc> {}
impl<Alloc: IAlloc, Rhs: AsRef<str>> PartialOrd<Rhs> for String<Alloc> {
    fn partial_cmp(&self, other: &Rhs) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_ref())
    }
}
impl<Alloc: IAlloc> Ord for String<Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<Alloc: IAlloc + Default> From<&str> for String<Alloc> {
    fn from(value: &str) -> Self {
        Self::default() + value
    }
}

impl<Alloc: IAlloc + Default> From<crate::str::Str<'_>> for String<Alloc> {
    fn from(value: crate::str::Str<'_>) -> Self {
        Self::default() + value.as_ref()
    }
}

/// A reference counted boxed string.
#[crate::stabby]
pub struct ArcStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: ArcSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> ArcStr<Alloc> {
    /// Returns a borrow to the inner string.
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
    }
    /// Returns a mutably borrow to the inner str.
    /// # Safety
    /// [`Self::is_unique`] must be true.
    pub unsafe fn as_str_mut_unchecked(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.inner.as_slice_mut_unchecked()) }
    }
    /// Returns a mutably borrow to the inner str if no other borrows of it can exist.
    pub fn as_str_mut(&mut self) -> Option<&mut str> {
        Self::is_unique(self).then(|| unsafe { self.as_str_mut_unchecked() })
    }
    /// Whether or not `this` is the sole owner of its data, including weak owners.
    pub fn is_unique(this: &Self) -> bool {
        ArcSlice::is_unique(&this.inner)
    }
}
impl<Alloc: IAlloc> AsRef<str> for ArcStr<Alloc> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<Alloc: IAlloc> core::fmt::Debug for ArcStr<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}
impl<Alloc: IAlloc> core::fmt::Display for ArcStr<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}
impl<Alloc: IAlloc> core::ops::Deref for ArcStr<Alloc> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl<Alloc: IAlloc> From<String<Alloc>> for ArcStr<Alloc> {
    fn from(value: String<Alloc>) -> Self {
        Self {
            inner: value.inner.into(),
        }
    }
}
impl<Alloc: IAlloc> TryFrom<ArcStr<Alloc>> for String<Alloc> {
    type Error = ArcStr<Alloc>;
    fn try_from(value: ArcStr<Alloc>) -> Result<Self, ArcStr<Alloc>> {
        match value.inner.try_into() {
            Ok(vec) => Ok(String { inner: vec }),
            Err(slice) => Err(ArcStr { inner: slice }),
        }
    }
}
impl<Alloc: IAlloc> Clone for ArcStr<Alloc> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<Alloc: IAlloc> Eq for ArcStr<Alloc> {}
impl<Alloc: IAlloc> PartialEq for ArcStr<Alloc> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl<Alloc: IAlloc> Ord for ArcStr<Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl<Alloc: IAlloc> PartialOrd for ArcStr<Alloc> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Alloc: IAlloc> Hash for ArcStr<Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

/// A weak reference counted boxed string.
#[crate::stabby]
pub struct WeakStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: WeakSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> WeakStr<Alloc> {
    /// Returns a strong reference if the strong count hasn't reached 0 yet.
    pub fn upgrade(&self) -> Option<ArcStr<Alloc>> {
        self.inner.upgrade().map(|inner| ArcStr { inner })
    }
    /// Returns a strong reference to the string.
    ///
    /// If you're using this, there are probably design issues in your program...
    pub fn force_upgrade(&self) -> ArcStr<Alloc> {
        ArcStr {
            inner: self.inner.force_upgrade(),
        }
    }
}
impl<Alloc: IAlloc> From<&ArcStr<Alloc>> for WeakStr<Alloc> {
    fn from(value: &ArcStr<Alloc>) -> Self {
        Self {
            inner: (&value.inner).into(),
        }
    }
}
impl<Alloc: IAlloc> Clone for WeakStr<Alloc> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// A boxed string.
#[crate::stabby]
pub struct BoxedStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: BoxedSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> BoxedStr<Alloc> {
    /// Returns a borrow to the inner string.
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
    }
    /// Returns a mutable borrow to the inner string.
    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(self.inner.as_slice_mut()) }
    }
}
impl<Alloc: IAlloc> AsRef<str> for BoxedStr<Alloc> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl<Alloc: IAlloc> core::ops::Deref for BoxedStr<Alloc> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl<Alloc: IAlloc> From<String<Alloc>> for BoxedStr<Alloc> {
    fn from(value: String<Alloc>) -> Self {
        Self {
            inner: value.inner.into(),
        }
    }
}
impl<Alloc: IAlloc> From<BoxedStr<Alloc>> for String<Alloc> {
    fn from(value: BoxedStr<Alloc>) -> Self {
        String {
            inner: value.inner.into(),
        }
    }
}
impl<Alloc: IAlloc> Eq for BoxedStr<Alloc> {}
impl<Alloc: IAlloc> PartialEq for BoxedStr<Alloc> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl<Alloc: IAlloc> Ord for BoxedStr<Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl<Alloc: IAlloc> PartialOrd for BoxedStr<Alloc> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Alloc: IAlloc> core::hash::Hash for BoxedStr<Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<Alloc: IAlloc> core::fmt::Debug for BoxedStr<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}
impl<Alloc: IAlloc> core::fmt::Display for BoxedStr<Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}

impl core::fmt::Write for String {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.try_concat(s).map_err(|_| core::fmt::Error)
    }
}

#[cfg(feature = "std")]
mod std_impl {
    use crate::alloc::IAlloc;
    impl<Alloc: IAlloc + Default> From<std::string::String> for crate::alloc::string::String<Alloc> {
        fn from(value: std::string::String) -> Self {
            Self::from(value.as_ref())
        }
    }
    impl<Alloc: IAlloc + Default> From<crate::alloc::string::String<Alloc>> for std::string::String {
        fn from(value: crate::alloc::string::String<Alloc>) -> Self {
            Self::from(value.as_ref())
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use crate::alloc::IAlloc;
    use serde::{Deserialize, Serialize};
    impl<Alloc: IAlloc> Serialize for String<Alloc> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice: &str = self;
            slice.serialize(serializer)
        }
    }
    impl<'a, Alloc: IAlloc + Default> Deserialize<'a> for String<Alloc> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'a>,
        {
            crate::str::Str::deserialize(deserializer).map(Into::into)
        }
    }
}
