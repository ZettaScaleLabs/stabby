use crate as stabby;
use crate::abi::enums::{IDiscriminant, IDiscriminantProvider};
use crate::abi::{IStable, Union};

#[stabby::stabby]
pub struct Discriminant<Ok: IStable, Err: IStable>(
    <Ok as IDiscriminantProvider>::Discriminant<Err>,
);
impl<Ok: IStable, Err: IStable> Copy for Discriminant<Ok, Err> {}
impl<Ok: IStable, Err: IStable> Clone for Discriminant<Ok, Err> {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}
impl<Ok: IStable, Err: IStable> Discriminant<Ok, Err> {
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: &mut Union<Ok, Err>) -> Self {
        Self(<<Ok as IDiscriminantProvider>::Discriminant<Err>>::ok(
            union,
        ))
    }
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: &mut Union<Ok, Err>) -> Self {
        Self(<<Ok as IDiscriminantProvider>::Discriminant<Err>>::err(
            union,
        ))
    }
    fn is_ok(&self, union: &Union<Ok, Err>) -> bool {
        self.0.is_ok(union)
    }
}
#[stabby::stabby]
pub struct Result<Ok: IStable, Err: IStable> {
    discriminant: Discriminant<Ok, Err>,
    union: Union<Ok, Err>,
}

impl<Ok: Clone + IStable, Err: Clone + IStable> Clone for Result<Ok, Err> {
    fn clone(&self) -> Self {
        self.match_ref(|ok| Self::Ok(ok.clone()), |err| Self::Err(err.clone()))
    }
}
impl<Ok: IStable, Err: IStable> core::fmt::Debug for Result<Ok, Err>
where
    Ok: core::fmt::Debug,
    Err: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl<Ok: IStable, Err: IStable> core::hash::Hash for Result<Ok, Err>
where
    Ok: core::hash::Hash,
    Err: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        if self.is_ok() {
            true.hash(state);
            unsafe { &self.union._0 }.hash(state);
        } else {
            false.hash(state);
            unsafe { &self.union._1 }.hash(state);
        }
    }
}
impl<Ok: IStable, Err: IStable> core::cmp::PartialEq for Result<Ok, Err>
where
    Ok: core::cmp::PartialEq,
    Err: core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.is_ok(), other.is_ok()) {
            (true, true) => unsafe { self.union._0.eq(&other.union._0) },
            (false, false) => unsafe { self.union._1.eq(&other.union._1) },
            _ => false,
        }
    }
}
impl<Ok: IStable, Err: IStable> core::cmp::Eq for Result<Ok, Err>
where
    Ok: core::cmp::Eq,
    Err: core::cmp::Eq,
{
}
impl<Ok: IStable, Err: IStable> From<core::result::Result<Ok, Err>> for Result<Ok, Err> {
    fn from(value: core::result::Result<Ok, Err>) -> Self {
        match value {
            Ok(value) => Self::Ok(value),
            Err(value) => Self::Err(value),
        }
    }
}
impl<Ok: IStable, Err: IStable> From<Result<Ok, Err>> for core::result::Result<Ok, Err> {
    fn from(value: Result<Ok, Err>) -> Self {
        value.match_owned(Ok, Err)
    }
}
impl<Ok: IStable, Err: IStable> Drop for Result<Ok, Err> {
    fn drop(&mut self) {
        self.match_mut(
            |ok| unsafe { core::ptr::drop_in_place(ok) },
            |err| unsafe { core::ptr::drop_in_place(err) },
        )
    }
}
impl<Ok: IStable, Err: IStable> Result<Ok, Err> {
    #[allow(non_snake_case)]
    pub fn Ok(value: Ok) -> Self {
        let mut union = Union {
            _0: core::mem::ManuallyDrop::new(value),
        };
        Self {
            discriminant: unsafe { Discriminant::ok(&mut union) },
            union,
        }
    }
    #[allow(non_snake_case)]
    pub fn Err(value: Err) -> Self {
        let mut union = Union {
            _1: core::mem::ManuallyDrop::new(value),
        };
        Self {
            discriminant: unsafe { Discriminant::err(&mut union) },
            union,
        }
    }
    pub fn as_ref(&self) -> core::result::Result<&Ok, &Err> {
        self.match_ref(Ok, Err)
    }
    pub fn as_mut(&mut self) -> core::result::Result<&mut Ok, &mut Err> {
        self.match_mut(Ok, Err)
    }
    pub fn match_ref<'a, U, FnOk: FnOnce(&'a Ok) -> U, FnErr: FnOnce(&'a Err) -> U>(
        &'a self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        if self.is_ok() {
            unsafe { ok(&self.union._0) }
        } else {
            unsafe { err(&self.union._1) }
        }
    }
    pub fn match_mut<'a, U, FnOk: FnOnce(&'a mut Ok) -> U, FnErr: FnOnce(&'a mut Err) -> U>(
        &'a mut self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        if self.is_ok() {
            unsafe { ok(&mut self.union._0) }
        } else {
            unsafe { err(&mut self.union._1) }
        }
    }
    pub fn match_owned<U, FnOk: FnOnce(Ok) -> U, FnErr: FnOnce(Err) -> U>(
        self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let is_ok = self.is_ok();
        let union = self.union.clone();
        core::mem::forget(self);
        if is_ok {
            ok(std::mem::ManuallyDrop::<Ok>::into_inner(unsafe {
                union._0
            }))
        } else {
            err(std::mem::ManuallyDrop::<Err>::into_inner(unsafe {
                union._1
            }))
        }
    }
    pub fn is_ok(&self) -> bool {
        self.discriminant.is_ok(&self.union)
    }
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
    pub fn ok(self) -> Option<Ok> {
        self.match_owned(|ok| Some(ok), |_| None)
    }
    pub fn err(self) -> Option<Err> {
        self.match_owned(|_| None, |err| Some(err))
    }
    pub fn and_then<F: FnOnce(Ok) -> U, U: IStable>(self, f: F) -> Result<U, Err> {
        self.match_owned(move |x| Result::Ok(f(x)), |x| Result::Err(x))
    }
    pub fn unwrap_or_else<F: FnOnce(Err) -> Ok>(self, f: F) -> Ok {
        self.match_owned(|x| x, f)
    }
    /// # Safety
    /// May trigger Undefined Behaviour if called on an Err variant.
    pub unsafe fn unwrap_unchecked(self) -> Ok {
        self.unwrap_or_else(|_| core::hint::unreachable_unchecked())
    }
    pub fn unwrap(self) -> Ok
    where
        Err: core::fmt::Debug,
    {
        self.unwrap_or_else(|e| panic!("Result::unwrap called on Err variant: {e:?}"))
    }
    pub fn unwrap_err_or_else<F: FnOnce(Ok) -> Err>(self, f: F) -> Err {
        self.match_owned(f, |x| x)
    }
    /// # Safety
    /// May trigger Undefined Behaviour if called on an Ok variant.
    pub unsafe fn unwrap_err_unchecked(self) -> Err {
        self.unwrap_err_or_else(|_| core::hint::unreachable_unchecked())
    }
    pub fn unwrap_err(self) -> Err
    where
        Ok: core::fmt::Debug,
    {
        self.unwrap_err_or_else(|e| panic!("Result::unwrap_err called on Ok variant: {e:?}"))
    }
}
