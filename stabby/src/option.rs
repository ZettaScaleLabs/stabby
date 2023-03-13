use crate as stabby;
use crate::abi::enums::IDiscriminantProvider;
use crate::abi::IStable;

#[stabby::stabby]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Option<T: IStable>
where
    (T, ()): IDiscriminantProvider,
{
    inner: crate::result::Result<T, ()>,
}
impl<T: IStable> core::fmt::Debug for Option<T>
where
    (T, ()): IDiscriminantProvider,
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl<T: IStable> From<core::option::Option<T>> for Option<T>
where
    (T, ()): IDiscriminantProvider,
{
    fn from(value: core::option::Option<T>) -> Self {
        match value {
            Some(value) => Self {
                inner: crate::result::Result::Ok(value),
            },
            None => Self {
                inner: crate::result::Result::Err(()),
            },
        }
    }
}
impl<T: IStable> From<Option<T>> for core::option::Option<T>
where
    (T, ()): IDiscriminantProvider,
{
    fn from(value: Option<T>) -> Self {
        value.inner.ok()
    }
}
impl<T: IStable> Option<T>
where
    (T, ()): IDiscriminantProvider,
{
    pub fn as_ref(&self) -> core::option::Option<&T> {
        self.match_ref(Some, || None)
    }
    pub fn as_mut(&mut self) -> core::option::Option<&mut T> {
        self.match_mut(Some, || None)
    }
    pub fn match_ref<'a, U, FnSome: FnOnce(&'a T) -> U, FnNone: FnOnce() -> U>(
        &'a self,
        ok: FnSome,
        err: FnNone,
    ) -> U {
        self.inner.match_ref(ok, |_| err())
    }
    pub fn match_mut<'a, U, FnSome: FnOnce(&'a mut T) -> U, FnNone: FnOnce() -> U>(
        &'a mut self,
        ok: FnSome,
        err: FnNone,
    ) -> U {
        self.inner.match_mut(ok, |_| err())
    }
    pub fn match_owned<U, FnSome: FnOnce(T) -> U, FnNone: FnOnce() -> U>(
        self,
        ok: FnSome,
        err: FnNone,
    ) -> U {
        self.inner.match_owned(ok, |_| err())
    }
    pub fn is_some(&self) -> bool {
        self.inner.is_ok()
    }
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        self.match_owned(|x| x, f)
    }
    /// # Safety
    /// May trigger Undefined Behaviour if called on an Err variant.
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.unwrap_or_else(|| core::hint::unreachable_unchecked())
    }
    pub fn unwrap(self) -> T {
        self.unwrap_or_else(|| panic!("Option::unwrap called on None"))
    }
}
