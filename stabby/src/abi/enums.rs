use super::{istable::B2, IStable, Union};
use crate as stabby;
use stabby::abi::*;

pub trait IDiscriminant<Ok, Err> {
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: &mut Union<Ok, Err>) -> Self;
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: &mut Union<Ok, Err>) -> Self;
    fn is_ok(&self, union: &Union<Ok, Err>) -> bool;
}

pub trait IDiscriminantProvider: IStable {
    type Discriminant<Err>: IDiscriminant<Self, Err> + Clone + Copy;
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
    type UnusedBits = End;
    type HasExactlyOneNiche = B2;
}

impl<Ok: IStable> IDiscriminantProvider for Ok {
    type Discriminant<Err> = BitDiscriminant;
}

impl<Ok, Err> IDiscriminant<Ok, Err> for BitDiscriminant {
    unsafe fn ok(_: &mut Union<Ok, Err>) -> Self {
        BitDiscriminant::Ok
    }
    unsafe fn err(_: &mut Union<Ok, Err>) -> Self {
        BitDiscriminant::Err
    }
    fn is_ok(&self, _: &Union<Ok, Err>) -> bool {
        matches!(self, BitDiscriminant::Ok)
    }
}
