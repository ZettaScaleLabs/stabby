use crate::{vtable::HasDropVt, IDeterminantProvider, IPtrMut, IPtrOwned};

/// [`core::iter::Iterator`], but ABI-stable.
#[crate::stabby]
pub trait Iterator {
    /// The type of the elements of the iterator.
    type Item: IDeterminantProvider<()>;
    /// Returns the next element in the iterator if it exists.
    extern "C" fn next(&mut self) -> crate::Option<Self::Item>;
    /// See [`core::iter::Iterator::size_hint`]
    extern "C" fn size_hint(&self) -> crate::Tuple<usize, crate::Option<usize>>;
}

impl<T: core::iter::Iterator> Iterator for T
where
    T::Item: IDeterminantProvider<()>,
{
    type Item = T::Item;
    extern "C" fn next(&mut self) -> crate::Option<Self::Item> {
        core::iter::Iterator::next(self).into()
    }
    extern "C" fn size_hint(&self) -> crate::Tuple<usize, crate::Option<usize>> {
        let (min, max) = core::iter::Iterator::size_hint(self);
        crate::Tuple(min, max.into())
    }
}

impl<Vt: HasDropVt, P: IPtrOwned + IPtrMut, Output: IDeterminantProvider<()>> core::iter::Iterator
    for crate::Dyn<'_, P, crate::vtable::VTable<StabbyVtableIterator<'_, Output>, Vt>>
{
    type Item = Output;
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: we're accessing a `StableLike` that was unsafely but properly constructed.
        unsafe {
            (self.vtable().head.next.as_ref_unchecked())(
                self.ptr_mut().as_mut(),
                core::marker::PhantomData,
            )
            .into()
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let crate::Tuple(min, max) = // SAFETY: we're accessing a `StableLike` that was unsafely but properly constructed.
            unsafe { (self.vtable().head.size_hint.as_ref_unchecked())(self.ptr().as_ref(), core::marker::PhantomData) };
        (min, max.into())
    }
}

impl<'a, Output> crate::vtable::CompoundVt<'a> for dyn core::iter::Iterator<Item = Output>
where
    dyn Iterator<Item = Output>: crate::vtable::CompoundVt<'a>,
{
    type Vt<T> = <dyn Iterator<Item = Output> as crate::vtable::CompoundVt<'a>>::Vt<T>;
}
