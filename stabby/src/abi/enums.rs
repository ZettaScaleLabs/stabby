use core::marker::PhantomData;

use stabby_macros::tyeval;

use super::{
    istable::{Includes, B2},
    IStable,
};
use crate::{
    abi::{padding::IPadding, *},
    tuple::Tuple2,
};

#[crate::stabby]
pub struct Padded<Padding, T> {
    pub(crate) padding: Padding,
    pub(crate) value: T,
}
impl<Padding: Default, T> From<T> for Padded<Padding, T> {
    fn from(value: T) -> Self {
        Self {
            padding: Default::default(),
            value,
        }
    }
}

pub trait IDiscriminant: IStable {
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: *mut u8) -> Self;
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: *mut u8) -> Self;
    fn is_ok(&self, union: *const u8) -> bool;
}

pub trait IDiscriminantProvider {
    type Ok;
    type Err;
    type Discriminant: IDiscriminant;
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitDiscriminant {
    Err = 0,
    Ok = 1,
}
unsafe impl IStable for BitDiscriminant {
    type Size = U1;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = Array<U0, U254, End>;
    type HasExactlyOneNiche = B2;
}

impl IDiscriminant for BitDiscriminant {
    unsafe fn ok(_: *mut u8) -> Self {
        BitDiscriminant::Ok
    }
    unsafe fn err(_: *mut u8) -> Self {
        BitDiscriminant::Err
    }
    fn is_ok(&self, _: *const u8) -> bool {
        matches!(self, BitDiscriminant::Ok)
    }
}
impl IDiscriminant for End {
    unsafe fn ok(_: *mut u8) -> Self {
        End
    }
    unsafe fn err(_: *mut u8) -> Self {
        End
    }
    fn is_ok(&self, _: *const u8) -> bool {
        true
    }
}
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct ValueIsErr<Offset, Value, Tail>(PhantomData<(Offset, Value)>, Tail);

impl<Offset: Unsigned, Value: Unsigned, Tail: IDiscriminant> IDiscriminant
    for ValueIsErr<Offset, Value, Tail>
where
    ValueIsErr<Offset, Value, Tail>: IStable,
{
    unsafe fn ok(union: *mut u8) -> Self {
        ValueIsErr(PhantomData, Tail::ok(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) = Value::U8;
        ValueIsErr(PhantomData, Tail::err(union))
    }
    fn is_ok(&self, union: *const u8) -> bool {
        let ptr = union as *const _ as *const u8;
        unsafe { *ptr.add(Offset::USIZE) != Value::U8 && self.1.is_ok(union) }
    }
}
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct BitIsErr<Offset, Mask>(PhantomData<(Offset, Mask)>);
impl<Offset: Unsigned, Mask: Unsigned> IDiscriminant for BitIsErr<Offset, Mask> {
    unsafe fn ok(_: *mut u8) -> Self {
        BitIsErr(PhantomData)
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) |= Mask::U8;
        BitIsErr(PhantomData)
    }
    fn is_ok(&self, union: *const u8) -> bool {
        let ptr = union as *const _ as *const u8;
        unsafe { *ptr.add(Offset::USIZE) & Mask::U8 != 0 }
    }
}
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct Not<Discriminant>(Discriminant);
impl<Discriminant: IDiscriminant> IDiscriminant for Not<Discriminant>
where
    Not<Discriminant>: IStable,
{
    unsafe fn ok(union: *mut u8) -> Self {
        Not(Discriminant::err(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        Not(Discriminant::ok(union))
    }
    fn is_ok(&self, union: *const u8) -> bool {
        !self.0.is_ok(union)
    }
}
