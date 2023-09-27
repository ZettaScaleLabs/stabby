use super::{
    boxed::BoxedSlice,
    sync::{ArcSlice, WeakSlice},
    vec::Vec,
    AllocationError, IAlloc,
};
use core::hash::Hash;

#[crate::stabby]
#[derive(Clone)]
pub struct String<Alloc: IAlloc = super::DefaultAllocator> {
    pub(crate) inner: Vec<u8, Alloc>,
}

impl<Alloc: IAlloc> String<Alloc> {
    pub const fn new_in(alloc: Alloc) -> Self {
        Self {
            inner: Vec::new_in(alloc),
        }
    }
    pub fn new() -> Self
    where
        Alloc: Default,
    {
        Self { inner: Vec::new() }
    }
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
    }
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
        Self::new()
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
        Self::new() + value
    }
}

#[crate::stabby]
pub struct ArcStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: ArcSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> ArcStr<Alloc> {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
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
        self.as_str().partial_cmp(other.as_str())
    }
}
impl<Alloc: IAlloc> Hash for ArcStr<Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

#[crate::stabby]
pub struct WeakStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: WeakSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> WeakStr<Alloc> {
    pub fn upgrade(&self) -> Option<ArcStr<Alloc>> {
        self.inner.upgrade().map(|inner| ArcStr { inner })
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

#[crate::stabby]
pub struct BoxedStr<Alloc: IAlloc = super::DefaultAllocator> {
    inner: BoxedSlice<u8, Alloc>,
}
impl<Alloc: IAlloc> BoxedStr<Alloc> {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.inner.as_slice()) }
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
        self.as_str().partial_cmp(other.as_str())
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
