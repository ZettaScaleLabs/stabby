use core::borrow::Borrow;

use crate::{
    abi::{IDiscriminantProvider, IStable},
    str::Str,
    string::String,
};

#[crate::stabby]
pub enum Cow<'a, Borrowed: IStable + ToOwned>
where
    <Borrowed as ToOwned>::Owned: IStable,
{
    Borrowed(&'a Borrowed),
    Owned(<Borrowed as ToOwned>::Owned),
}

impl<Borrowed: IStable + ToOwned> Cow<'_, Borrowed>
where
    <Borrowed as ToOwned>::Owned: IStable,
    for<'a> (&'a Borrowed, <Borrowed as ToOwned>::Owned): IDiscriminantProvider,
{
    pub fn into_owned(self) -> <Borrowed as ToOwned>::Owned {
        self.match_owned(|b| b.to_owned(), |o| o)
    }
    pub fn to_owned(self) -> Cow<'static, Borrowed> {
        Cow::Owned(self.into_owned())
    }
}
impl<Borrowed: IStable + ToOwned> Borrow<Borrowed> for Cow<'_, Borrowed>
where
    <Borrowed as ToOwned>::Owned: IStable,
    for<'a> (&'a Borrowed, <Borrowed as ToOwned>::Owned): IDiscriminantProvider,
{
    fn borrow(&self) -> &Borrowed {
        self.match_ref(|&b| b, |o| o.borrow())
    }
}

#[crate::stabby]
pub enum CowStr<'a> {
    Borrowed(Str<'a>),
    Owned(String),
}
fn test() {
    let _: crate::abi::typenum2::U24 = <CowStr as IStable>::Size::default();
}
