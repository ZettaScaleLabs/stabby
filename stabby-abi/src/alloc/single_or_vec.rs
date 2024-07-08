use super::vec::Vec;
use crate::alloc::{AllocationError, DefaultAllocator, IAlloc};
use crate::num::NonMaxUsize;
use crate::result::OkGuard;
use crate::{IDeterminantProvider, IStable};
mod seal {
    #[crate::stabby]
    pub struct Single<T, Alloc> {
        pub value: T,
        pub alloc: Alloc,
    }
}
pub(crate) use seal::*;
/// A vector that doesn't need to allocate for its first value.
///
/// Once a second value is pushed, or if greater capacity is reserved,
/// the allocated vector will be used regardless of how the vector's
/// number of elements evolves.
#[crate::stabby]
pub struct SingleOrVec<T: IStable, Alloc: IAlloc + IStable = DefaultAllocator>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    inner: crate::Result<Single<T, Alloc>, Vec<T, Alloc>>,
}

#[cfg(not(stabby_default_alloc = "disabled"))]
impl<T: IStable> SingleOrVec<T>
where
    Single<T, DefaultAllocator>: IDeterminantProvider<Vec<T, DefaultAllocator>>,
    Vec<T, DefaultAllocator>: IStable,
    crate::Result<Single<T, DefaultAllocator>, Vec<T, DefaultAllocator>>: IStable,
{
    /// Constructs a new vector. This doesn't actually allocate.
    pub fn new() -> Self {
        Self::new_in(DefaultAllocator::new())
    }
}

impl<T: IStable, Alloc: IAlloc + IStable + Default> Default for SingleOrVec<T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn default() -> Self {
        Self::new_in(Alloc::default())
    }
}
impl<T: IStable, Alloc: IAlloc + IStable> SingleOrVec<T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    /// Constructs a new vector in `alloc`. This doesn't actually allocate.
    pub fn new_in(alloc: Alloc) -> Self {
        Self {
            inner: crate::Result::Err(Vec::new_in(alloc)),
        }
    }
    /// Constructs a new vector in `alloc`, allocating sufficient space for `capacity` elements.
    ///
    /// # Panics
    /// If the allocator failed to provide a large enough allocation.
    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        let mut this = Self::new_in(alloc);
        this.reserve(capacity);
        this
    }
    /// Constructs a new vector, allocating sufficient space for `capacity` elements.
    ///
    /// # Panics
    /// If the allocator failed to provide a large enough allocation.
    pub fn with_capacity(capacity: usize) -> Self
    where
        Alloc: Default,
    {
        Self::with_capacity_in(capacity, Alloc::default())
    }
    /// Constructs a new vector in `alloc`, allocating sufficient space for `capacity` elements.
    ///
    /// # Errors
    /// Returns an [`AllocationError`] if the allocator couldn't provide a sufficient allocation.
    pub fn try_with_capacity_in(capacity: usize, alloc: Alloc) -> Result<Self, Alloc> {
        Vec::try_with_capacity_in(capacity, alloc).map(|vec| Self {
            inner: crate::Result::Err(vec),
        })
    }
    /// Constructs a new vector, allocating sufficient space for `capacity` elements.
    ///
    /// # Errors
    /// Returns an [`AllocationError`] if the allocator couldn't provide a sufficient allocation.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, Alloc>
    where
        Alloc: Default,
        Self: IDeterminantProvider<AllocationError>,
    {
        Self::try_with_capacity_in(capacity, Alloc::default())
    }
    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.inner.match_ref(|_| 1, |vec| vec.len())
    }
    /// Returns `true` if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.match_ref(|_| false, |vec| vec.is_empty())
    }
    /// Adds `value` at the end of `self`.
    ///
    /// # Panics
    /// This function panics if the vector tried to grow due to
    /// being full, and the allocator failed to provide a new allocation.
    pub fn push(&mut self, value: T) {
        if self.try_push(value).is_err() {
            panic!("Failed to push because reallocation failed.")
        }
    }
    /// Adds `value` at the end of `self`.
    ///
    /// # Errors
    /// This function gives back the `value` if the vector tried to grow due to
    /// being full, and the allocator failed to provide a new allocation.
    ///
    /// `self` is still valid should that happen.
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        // Safety: either `this` or `*self` MUST be leaked to prevent double-frees.
        let this = unsafe { core::ptr::read(self) };
        // Safety: Any path that returns `Err` MUST leak the contents of the second argument.
        let this = this.inner.match_owned_ctx(
            item,
            |item, single| {
                // either `inner` must be leaked and overwritten by the new owner of `value` and `alloc`,
                // or these two must be leaked to prevent double frees.
                let Single { value, alloc } = single;
                match Vec::try_with_capacity_in(8, alloc) {
                    Ok(mut vec) => {
                        vec.push(value);
                        vec.push(item);
                        Ok(crate::Result::Err(vec))
                    }
                    Err(alloc) => {
                        // Safety: leak both `value` and `alloc` since `*self` won't be leaked
                        core::mem::forget((value, alloc));
                        Err(item)
                    }
                }
            },
            |item, mut vec| {
                if vec.capacity() == 0 {
                    unsafe {
                        let alloc = core::ptr::read(&vec.inner.alloc);
                        core::mem::forget(vec);
                        Ok(crate::Result::Ok(Single { value: item, alloc }))
                    }
                } else {
                    match vec.try_push(item) {
                        Ok(()) => Ok(crate::Result::Err(vec)),
                        Err(item) => {
                            // Safety: `vec` since `*self` won't be leaked
                            core::mem::forget(vec);
                            Err(item)
                        }
                    }
                }
            },
        );
        match this {
            Ok(inner) => unsafe {
                // Safety: this leaks `*self`, preventing it from being unduely destroyed
                core::ptr::write(self, Self { inner });
                Ok(())
            },
            Err(item) => Err(item),
        }
    }
    /// The total capacity of the vector.
    pub fn capacity(&self) -> usize {
        self.inner.match_ref(|_| 1, |vec| vec.capacity())
    }
    /// The remaining number of elements that can be pushed before reallocating.
    pub fn remaining_capacity(&self) -> usize {
        self.inner.match_ref(|_| 0, |vec| vec.remaining_capacity())
    }
    /// Ensures that `additional` more elements can be pushed on `self` without reallocating.
    ///
    /// This may reallocate once to provide this guarantee.
    ///
    /// # Panics
    /// This function panics if the allocator failed to provide an appropriate allocation.
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap();
    }
    /// Ensures that `additional` more elements can be pushed on `self` without reallocating.
    ///
    /// This may reallocate once to provide this guarantee.
    ///
    /// # Errors
    /// Returns Ok(new_capacity) if succesful (including if no reallocation was needed),
    /// otherwise returns Err(AllocationError)
    pub fn try_reserve(&mut self, additional: usize) -> Result<NonMaxUsize, AllocationError> {
        let inner = &mut self.inner as *mut _;
        self.inner.match_mut(
            |value| unsafe {
                let new_capacity = 1 + additional;
                // either `inner` must be leaked and overwritten by the new owner of `value` and `alloc`,
                // or these two must be leaked to prevent double frees.
                let Single { value, alloc } = core::ptr::read(&*value);
                match Vec::try_with_capacity_in(new_capacity, alloc) {
                    Ok(mut vec) => {
                        vec.push(value);
                        // overwrite `inner` with `value` and `alloc` with their new owner without freeing `inner`.
                        core::ptr::write(inner, crate::Result::Err(vec));
                        NonMaxUsize::new(new_capacity).ok_or(AllocationError())
                    }
                    Err(alloc) => {
                        // leak both `value` and `alloc` since `inner` can't be overwritten
                        core::mem::drop((value, alloc));
                        Err(AllocationError())
                    }
                }
            },
            |mut vec| vec.try_reserve(additional),
        )
    }
    /// Removes all elements from `self` from the `len`th onward.
    ///
    /// Does nothing if `self.len() <= len`
    pub fn truncate(&mut self, len: usize) {
        if self.len() <= len {
            return;
        }
        let inner = &mut self.inner as *mut _;
        self.inner.match_mut(
            |value| unsafe {
                let Single { value, alloc } = core::ptr::read(&*value);
                core::mem::drop(value); // drop `value` to prevent leaking it since we'll overwrite `inner` with something that doesn't own it
                                        // overwrite `inner` with the new owner of `alloc`
                core::ptr::write(inner, crate::Result::Err(Vec::new_in(alloc)))
            },
            |mut vec| vec.truncate(len),
        )
    }
    /// Returns a slice of the elements in the vector.
    pub fn as_slice(&self) -> &[T] {
        self.inner.match_ref(
            |value| core::slice::from_ref(&value.value),
            |vec| vec.as_slice(),
        )
    }
    /// Returns a mutable slice of the elements in the vector.
    pub fn as_slice_mut(&mut self) -> SliceGuardMut<T, Alloc> {
        self.inner.match_mut(
            |value| SliceGuardMut { inner: Ok(value) },
            |mut vec| SliceGuardMut {
                inner: Err(unsafe {
                    core::mem::transmute::<&mut [T], &mut [T]>(vec.as_slice_mut())
                }),
            },
        )
    }
    // pub fn iter(&self) -> core::slice::Iter<'_, T> {
    //     self.into_iter()
    // }
    // pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
    //     self.into_iter()
    // }
}

/// A mutable accessor to [`SingleOrVec`]'s inner slice.
///
/// Failing to drop this guard may cause Undefined Behaviour
pub struct SliceGuardMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    #[allow(clippy::type_complexity)]
    inner: Result<OkGuard<'a, Single<T, Alloc>, Vec<T, Alloc>>, &'a mut [T]>,
}
impl<'a, T, Alloc> core::ops::Deref for SliceGuardMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            Ok(v) => core::slice::from_ref(&v.value),
            Err(v) => v,
        }
    }
}
impl<'a, T, Alloc> core::ops::DerefMut for SliceGuardMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            Ok(v) => core::slice::from_mut(&mut v.value),
            Err(v) => v,
        }
    }
}

impl<T: Clone, Alloc: IAlloc + Clone> Clone for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn clone(&self) -> Self {
        self.inner.match_ref(
            |Single { value, alloc }| Self {
                inner: crate::Result::Ok(Single {
                    value: value.clone(),
                    alloc: alloc.clone(),
                }),
            },
            |vec| Self {
                inner: crate::Result::Err(vec.clone()),
            },
        )
    }
}
impl<T: PartialEq, Alloc: IAlloc, Rhs: AsRef<[T]>> PartialEq<Rhs> for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn eq(&self, other: &Rhs) -> bool {
        self.as_slice() == other.as_ref()
    }
}
impl<T: Eq, Alloc: IAlloc> Eq for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
}
impl<T: PartialOrd, Alloc: IAlloc, Rhs: AsRef<[T]>> PartialOrd<Rhs> for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn partial_cmp(&self, other: &Rhs) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}
impl<T: Ord, Alloc: IAlloc> Ord for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
impl<T, Alloc: IAlloc> core::ops::Deref for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, Alloc: IAlloc> core::convert::AsRef<[T]> for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, Alloc: IAlloc> core::iter::Extend<T> for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        let iter = iter.into_iter();
        let min = iter.size_hint().0;
        self.reserve(min);
        for item in iter {
            self.push(item);
        }
    }
}

impl<'a, T, Alloc: IAlloc> IntoIterator for &'a SingleOrVec<T, Alloc>
where
    T: IStable + 'a,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a mut SingleOrVec<T, Alloc>
where
    T: IStable + 'a,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, Alloc>;
    fn into_iter(self) -> Self::IntoIter {
        let inner = self.as_slice_mut();
        IterMut {
            start: 0,
            end: inner.len(),
            inner,
        }
    }
}

/// An iterator over a [`SliceGuardMut`].
pub struct IterMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    inner: SliceGuardMut<'a, T, Alloc>,
    start: usize,
    end: usize,
}

impl<'a, T, Alloc> Iterator for IterMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let r = unsafe {
                core::mem::transmute::<&mut T, &mut T>(self.inner.get_unchecked_mut(self.start))
            };
            self.start += 1;
            Some(r)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.start += n;
        self.next()
    }
}

impl<'a, T, Alloc> DoubleEndedIterator for IterMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end -= 1;
            let r = unsafe {
                core::mem::transmute::<&mut T, &mut T>(self.inner.get_unchecked_mut(self.end))
            };
            Some(r)
        } else {
            None
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.end = self.end.saturating_sub(n);
        self.next_back()
    }
}
impl<'a, T, Alloc> ExactSizeIterator for IterMut<'a, T, Alloc>
where
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
    Alloc: IAlloc,
{
    fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

impl<T, Alloc: IAlloc> From<Vec<T, Alloc>> for SingleOrVec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn from(value: Vec<T, Alloc>) -> Self {
        Self {
            inner: crate::Result::Err(value),
        }
    }
}

impl<T, Alloc: IAlloc> From<SingleOrVec<T, Alloc>> for Vec<T, Alloc>
where
    T: IStable,
    Alloc: IStable,
    Single<T, Alloc>: IDeterminantProvider<Vec<T, Alloc>>,
    Vec<T, Alloc>: IStable,
    crate::Result<Single<T, Alloc>, Vec<T, Alloc>>: IStable,
{
    fn from(value: SingleOrVec<T, Alloc>) -> Self {
        value.inner.match_owned(
            |Single { value, alloc }| {
                let mut vec = Vec::new_in(alloc);
                vec.push(value);
                vec
            },
            |vec| vec,
        )
    }
}

#[test]
fn test() {
    use rand::Rng;
    const LEN: usize = 20;
    let mut std = std::vec::Vec::with_capacity(LEN);
    let mut new: SingleOrVec<u8> = SingleOrVec::new();
    let mut capacity: SingleOrVec<u8> = SingleOrVec::with_capacity(LEN);
    let mut rng = rand::thread_rng();
    let n: u8 = rng.gen();
    new.push(n);
    capacity.push(n);
    std.push(n);
    assert!(new.inner.is_ok());
    assert!(capacity.inner.is_err());
    for _ in 0..LEN {
        let n: u8 = rng.gen();
        new.push(n);
        capacity.push(n);
        std.push(n);
    }
    assert_eq!(new.as_slice(), std.as_slice());
    assert_eq!(new.as_slice(), capacity.as_slice());
    let clone = new.clone();
    assert_eq!(new.as_slice(), clone.as_slice());
}
