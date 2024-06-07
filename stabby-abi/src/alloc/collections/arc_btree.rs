#![allow(missing_docs, clippy::missing_panics_doc)]

use core::{
    borrow::Borrow, marker::PhantomData, mem::MaybeUninit, ptr::NonNull, sync::atomic::AtomicPtr,
};

use crate::{
    alloc::{sync::Arc, AllocPtr, DefaultAllocator, IAlloc},
    IStable,
};

#[cfg(feature = "libc")]
pub struct AtomicArcBTreeSet<T: Ord, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>(
    AtomicPtr<ArcBTreeSetNodeInner<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
);
#[cfg(feature = "libc")]
impl<T: Ord + Clone, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> Default
    for AtomicArcBTreeSet<T, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(feature = "libc")]
impl<T: Ord + Clone, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    AtomicArcBTreeSet<T, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    pub const fn new() -> Self {
        Self(AtomicPtr::new(unsafe {
            core::mem::transmute::<
                ArcBTreeSet<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
                *mut ArcBTreeSetNodeInner<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
            >(ArcBTreeSet::<
                T,
                DefaultAllocator,
                REPLACE_ON_INSERT,
                SPLIT_LIMIT,
            >::new_in(DefaultAllocator::new()))
        }))
    }
    pub fn edit(
        &self,
        mut f: impl FnMut(
            ArcBTreeSet<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
        ) -> ArcBTreeSet<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
    ) {
        let mut current = self.0.load(core::sync::atomic::Ordering::Acquire);
        loop {
            let new = f(ArcBTreeSet::copy_from_ptr(current));
            let new_ptr = new.as_ptr();
            if core::ptr::eq(current, new_ptr) {
                return;
            }
            match self.0.compare_exchange(
                current,
                new_ptr,
                core::sync::atomic::Ordering::Release,
                core::sync::atomic::Ordering::Acquire,
            ) {
                Ok(old) => unsafe {
                    core::mem::forget(new);
                    ArcBTreeSet::take_ownership_from_ptr(old);
                    return;
                },
                Err(new_old) => {
                    current = new_old;
                }
            }
        }
    }
    pub fn get<K>(&self, value: &K, f: impl FnOnce(Option<&T>))
    where
        T: PartialOrd<K>,
    {
        let set = ArcBTreeSet::copy_from_ptr(self.0.load(core::sync::atomic::Ordering::Relaxed));
        f(set.get(value))
    }
}
#[crate::stabby]
#[derive(Debug, Clone)]
pub struct Entry<K, V> {
    key: K,
    value: V,
}

impl<K: Ord, V> PartialEq for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}
impl<K: Ord, V> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<K: Ord, V> PartialEq<K> for Entry<K, V> {
    fn eq(&self, other: &K) -> bool {
        self.key.eq(other)
    }
}
impl<K: Ord, V> PartialOrd<K> for Entry<K, V> {
    fn partial_cmp(&self, other: &K) -> Option<core::cmp::Ordering> {
        Some(self.key.cmp(other))
    }
}
impl<K: Ord, V> Eq for Entry<K, V> {}
impl<K: Ord, V> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
#[crate::stabby]
#[derive(Clone)]
pub struct ArcBTreeMap<K, V, Alloc: IAlloc = DefaultAllocator, const SPLIT_LIMIT: usize = { 5 }>(
    ArcBTreeSet<Entry<K, V>, Alloc, true, SPLIT_LIMIT>,
);
impl<K: Ord, V, Alloc: IAlloc, const SPLIT_LIMIT: usize> ArcBTreeMap<K, V, Alloc, SPLIT_LIMIT> {
    pub const fn new_in(alloc: Alloc) -> Self {
        Self(ArcBTreeSet::new_in(alloc))
    }
    pub fn get<Q: Borrow<K>>(&self, key: &Q) -> Option<&V> {
        self.0.get(key.borrow()).map(|entry| &entry.value)
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Clone,
        V: Clone,
        Alloc: Clone,
    {
        self.0.insert(Entry { key, value }).map(|entry| entry.value)
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct ArcBTreeSet<
    T,
    Alloc: IAlloc = DefaultAllocator,
    const REPLACE_ON_INSERT: bool = { false },
    const SPLIT_LIMIT: usize = { 5 },
>(Result<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>, Alloc>);

unsafe impl<T: IStable, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    IStable for ArcBTreeSet<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
where
    Alloc: IStable<Size = crate::U0>,
    Arc<T, Alloc>: IStable<HasExactlyOneNiche = crate::B1>,
{
    type Size = <*const T as IStable>::Size;
    type Align = <*const T as IStable>::Align;
    type ForbiddenValues = <*const T as IStable>::ForbiddenValues;
    type UnusedBits = <*const T as IStable>::UnusedBits;
    type HasExactlyOneNiche = crate::B0;
    type ContainsIndirections = crate::B1;
    const REPORT: &'static crate::report::TypeReport = &crate::report::TypeReport {
        name: crate::str::Str::new("ArcBTreeSet"),
        module: crate::str::Str::new("stabby_abi::alloc::collections::arc_btree"),
        fields: crate::StableLike::new(Some(&crate::report::FieldReport {
            name: crate::str::Str::new("T"),
            ty: T::REPORT,
            next_field: crate::StableLike::new(Some(&crate::report::FieldReport {
                name: crate::str::Str::new("Alloc"),
                ty: Alloc::REPORT,
                next_field: crate::StableLike::new(None),
            })),
        })),
        tyty: crate::report::TyTy::Struct,
        version: 0,
    };
    const ID: u64 = crate::report::gen_id(Self::REPORT);
}

#[cfg(feature = "libc")]
impl<T: Ord> Default for ArcBTreeSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(feature = "libc")]
impl<T: Ord> ArcBTreeSet<T> {
    pub const fn new() -> Self {
        Self::new_in(DefaultAllocator::new())
    }
}
impl<
        T: Ord + core::fmt::Debug,
        Alloc: IAlloc,
        const REPLACE_ON_INSERT: bool,
        const SPLIT_LIMIT: usize,
    > core::fmt::Debug for ArcBTreeSet<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ArcBTreeSet")?;
        match &self.0 {
            Err(_) => f.write_str("{}"),
            Ok(set) => set.fmt(f),
        }
    }
}
impl<
        T: Ord + core::fmt::Debug,
        Alloc: IAlloc,
        const REPLACE_ON_INSERT: bool,
        const SPLIT_LIMIT: usize,
    > core::fmt::Debug for ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = f.debug_set();
        for entry in self.0.entries() {
            if let Some(lesser) = &entry.smaller {
                f.entry(&lesser);
            }
            f.entry(&entry.value);
        }
        if let Some(greater) = &self.0.greater {
            f.entry(greater);
        }
        f.finish()
    }
}
impl<T: Ord + Clone, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSet<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    const fn as_ptr(
        &self,
    ) -> *mut ArcBTreeSetNodeInner<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT> {
        unsafe { core::mem::transmute_copy(self) }
    }
    fn copy_from_ptr(
        ptr: *const ArcBTreeSetNodeInner<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
    ) -> Self {
        let owner: Self = unsafe { core::mem::transmute(ptr) };
        let this = owner.clone();
        core::mem::forget(owner);
        this
    }
    unsafe fn take_ownership_from_ptr(
        ptr: *mut ArcBTreeSetNodeInner<T, DefaultAllocator, REPLACE_ON_INSERT, SPLIT_LIMIT>,
    ) -> Self {
        unsafe { core::mem::transmute(ptr) }
    }
}
impl<T: Ord, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSet<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    const CHECK: () = if SPLIT_LIMIT % 2 == 0 {
        panic!("SPLIT_LIMIT on BTreeSet/BTreeMap must be odd (it is the number of elements at which a node will split)");
    };
    #[allow(clippy::let_unit_value)]
    pub const fn new_in(alloc: Alloc) -> Self {
        _ = Self::CHECK;
        Self(Err(alloc))
    }
    pub fn get<K>(&self, key: &K) -> Option<&T>
    where
        T: PartialOrd<K>,
    {
        self.0.as_ref().ok().and_then(|set| set.get(key))
    }
    pub fn insert(&mut self, value: T) -> Option<T>
    where
        T: Clone,
        Alloc: Clone,
    {
        match &mut self.0 {
            Ok(inner) => inner.insert(value),
            Err(alloc) => {
                self.0 = Ok(ArcBTreeSetNode(Arc::new_in(
                    ArcBTreeSetNodeInner::new(
                        Some(ArcBTreeSetEntry {
                            value,
                            smaller: None,
                        }),
                        None,
                    ),
                    alloc.clone(),
                )));
                None
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn for_each(&self, mut f: impl FnMut(&T)) {
        if let Ok(this) = &self.0 {
            this.for_each(&mut f)
        }
    }
    pub fn len(&self) -> usize {
        match &self.0 {
            Err(_) => 0,
            Ok(node) => node.len(),
        }
    }
    pub const fn is_empty(&self) -> bool {
        self.0.is_err()
    }
}

// #[stabby::stabby]
/// An immutable ArcBTreeMap.
struct ArcBTreeSetNode<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>(
    Arc<ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>, Alloc>,
);
struct ArcBTreeSetNodeInner<
    T,
    Alloc: IAlloc,
    const REPLACE_ON_INSERT: bool,
    const SPLIT_LIMIT: usize,
> {
    entries: [MaybeUninit<ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>; SPLIT_LIMIT],
    len: usize,
    greater: Option<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
}
impl<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn new(
        entries: impl IntoIterator<Item = ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
        greater: Option<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
    ) -> Self {
        let mut this = ArcBTreeSetNodeInner {
            entries: [(); SPLIT_LIMIT].map(|_| MaybeUninit::uninit()),
            len: 0,
            greater,
        };
        for entry in entries {
            if this.len >= SPLIT_LIMIT - 1 {
                panic!("Attempted to construct an node with too many entries");
            }
            this.entries[this.len].write(entry);
            this.len += 1;
        }
        this
    }
}
impl<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> Clone
    for ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> Drop
    for ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.entries_mut()) }
    }
}
impl<T: Clone, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> Clone
    for ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn clone(&self) -> Self {
        let mut entries: [MaybeUninit<ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>;
            SPLIT_LIMIT] = [(); SPLIT_LIMIT].map(|_| core::mem::MaybeUninit::uninit());
        unsafe {
            for (i, entry) in self.entries().iter().enumerate() {
                *entries.get_unchecked_mut(i) = MaybeUninit::new(entry.clone())
            }
        }
        Self {
            entries,
            len: self.len,
            greater: self.greater.clone(),
        }
    }
}
// #[stabby::stabby]
/// A node of an immutable ArcBTreeMap.
struct ArcBTreeSetEntry<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> {
    smaller: Option<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
    value: T,
}
impl<T: Clone, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize> Clone
    for ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            smaller: self.smaller.clone(),
        }
    }
}

impl<T: Ord, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn len(&self) -> usize {
        self.0.entries().iter().fold(0, |acc, it| {
            acc + 1 + it.smaller.as_ref().map_or(0, |n| n.len())
        }) + self.0.greater.as_ref().map_or(0, |n| n.len())
    }
    fn get<K>(&self, key: &K) -> Option<&T>
    where
        T: PartialOrd<K>,
    {
        use core::cmp::Ordering;
        for entry in self.0.entries() {
            match entry.value.partial_cmp(key)? {
                Ordering::Equal => return Some(&entry.value),
                Ordering::Greater => return entry.smaller.as_ref()?.get(key),
                _ => {}
            }
        }
        self.0.greater.as_ref()?.get(key)
    }
    fn insert(&mut self, value: T) -> Option<T>
    where
        T: Clone,
        Alloc: Clone,
    {
        if !REPLACE_ON_INSERT && self.get(&value).is_some() {
            return Some(value);
        }
        match self.insert_inner(value) {
            Err(done) => done,
            Ok((right, pivot)) => {
                let entry = ArcBTreeSetEntry {
                    value: pivot,
                    smaller: Some(self.clone()),
                };
                let mut inner = ArcBTreeSetNodeInner {
                    entries: [(); SPLIT_LIMIT].map(|_| MaybeUninit::uninit()),
                    len: 1,
                    greater: Some(right),
                };
                inner.entries[0].write(entry);
                self.0 = Arc::new_in(inner, Arc::allocator(&self.0).clone());
                None
            }
        }
    }
    fn insert_inner(&mut self, value: T) -> Result<(Self, T), Option<T>>
    where
        T: Clone,
        Alloc: Clone,
    {
        use core::cmp::Ordering;
        let inner = Arc::make_mut(&mut self.0);
        let alloc = unsafe {
            AllocPtr {
                ptr: NonNull::new_unchecked(inner),
                marker: PhantomData,
            }
        };
        let alloc = &unsafe { alloc.prefix() }.alloc;
        let entries = inner.entries_mut();
        for (i, entry) in entries.iter_mut().enumerate() {
            match entry.value.cmp(&value) {
                Ordering::Equal => return Err(Some(core::mem::replace(&mut entry.value, value))),
                Ordering::Greater => match entry.smaller.as_mut() {
                    Some(smaller) => {
                        let (right, pivot) = smaller.insert_inner(value)?;
                        return match inner.insert(i, pivot, Some(right), alloc) {
                            None => Err(None),
                            Some(splits) => Ok(splits),
                        };
                    }
                    None => {
                        return match inner.insert(i, value, None, alloc) {
                            None => Err(None),
                            Some(splits) => Ok(splits),
                        }
                    }
                },
                _ => {}
            }
        }
        match inner.greater.as_mut() {
            Some(greater) => {
                let (right, pivot) = greater.insert_inner(value)?;
                if let Some(splits) = inner.push(pivot, Some(right), alloc) {
                    return Ok(splits);
                }
            }
            None => {
                if let Some(splits) = inner.push(value, None, alloc) {
                    return Ok(splits);
                }
            }
        }
        Err(None)
    }
    #[cfg(test)]
    fn for_each(&self, f: &mut impl FnMut(&T)) {
        for ArcBTreeSetEntry { value, smaller } in self.0.entries() {
            if let Some(smaller) = smaller {
                smaller.for_each(f);
            }
            f(value)
        }
        if let Some(greater) = self.0.greater.as_ref() {
            greater.for_each(f)
        }
    }
}
impl<T: Ord, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn insert(
        &mut self,
        i: usize,
        value: T,
        greater: Option<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
        alloc: &Alloc,
    ) -> Option<(ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>, T)>
    where
        Alloc: Clone,
    {
        unsafe {
            for j in (i..self.len).rev() {
                *self.entries.get_unchecked_mut(j + 1) =
                    MaybeUninit::new(self.entries.get_unchecked(j).assume_init_read());
            }
            self.len += 1;
            *self.entries.get_unchecked_mut(i) = MaybeUninit::new(ArcBTreeSetEntry {
                value,
                smaller: core::mem::replace(
                    &mut self
                        .entries
                        .get_unchecked_mut(i + 1)
                        .assume_init_mut()
                        .smaller,
                    greater,
                ),
            });
        }
        self.split(alloc)
    }
    fn push(
        &mut self,
        value: T,
        greater: Option<ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>>,
        alloc: &Alloc,
    ) -> Option<(ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>, T)>
    where
        Alloc: Clone,
    {
        unsafe {
            self.entries
                .get_unchecked_mut(self.len)
                .write(ArcBTreeSetEntry {
                    value,
                    smaller: core::mem::replace(&mut self.greater, greater),
                });
            self.len += 1;
        }
        self.split(alloc)
    }
    fn split(
        &mut self,
        alloc: &Alloc,
    ) -> Option<(ArcBTreeSetNode<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>, T)>
    where
        Alloc: Clone,
    {
        unsafe {
            if self.len == SPLIT_LIMIT {
                let ArcBTreeSetEntry {
                    value: pivot,
                    smaller,
                } = self
                    .entries
                    .get_unchecked(SPLIT_LIMIT / 2)
                    .assume_init_read();
                let mut right = Self {
                    entries: [(); SPLIT_LIMIT].map(|_| core::mem::MaybeUninit::uninit()),
                    len: SPLIT_LIMIT / 2,
                    greater: self.greater.take(),
                };
                core::ptr::copy_nonoverlapping(
                    self.entries.get_unchecked(SPLIT_LIMIT / 2 + 1),
                    right.entries.get_unchecked_mut(0),
                    SPLIT_LIMIT / 2,
                );
                self.greater = smaller;
                self.len = SPLIT_LIMIT / 2;
                let right = ArcBTreeSetNode(Arc::new_in(right, alloc.clone()));
                Some((right, pivot))
            } else {
                None
            }
        }
    }
}

impl<T, Alloc: IAlloc, const REPLACE_ON_INSERT: bool, const SPLIT_LIMIT: usize>
    ArcBTreeSetNodeInner<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>
{
    fn entries(&self) -> &[ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>] {
        unsafe { core::mem::transmute(self.entries.get_unchecked(..self.len)) }
    }
    fn entries_mut(&mut self) -> &mut [ArcBTreeSetEntry<T, Alloc, REPLACE_ON_INSERT, SPLIT_LIMIT>] {
        unsafe { core::mem::transmute(self.entries.get_unchecked_mut(..self.len)) }
    }
}
#[test]
fn btree_insert() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let mut vec = crate::alloc::vec::Vec::new();
        let mut btree = ArcBTreeSet::new();
        for _ in 0..rng.gen_range(0..800) {
            let val = rng.gen_range(0..100);
            if vec.binary_search(&val).is_ok() {
                assert_eq!(btree.insert(val), Some(val));
            } else {
                vec.push(val);
                vec.sort();
                assert_eq!(btree.insert(val), None);
            }
        }
        vec.sort();
        assert_eq!(vec.len(), btree.len());
        let mut iter = vec.into_iter();
        btree.for_each(|i| assert_eq!(Some(*i), iter.next()));
        assert_eq!(iter.next(), None);
    }
}
