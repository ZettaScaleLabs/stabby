//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use core::ops::{Deref, DerefMut};

impl<Source> super::AccessAs for Source {
    fn ref_as<T: ?Sized>(&self) -> <Self as IGuardRef<T>>::Guard<'_>
    where
        Self: IGuardRef<T>,
    {
        self.guard_ref_inner()
    }
    fn mut_as<T: ?Sized>(&mut self) -> <Self as IGuardMut<T>>::GuardMut<'_>
    where
        Self: IGuardMut<T>,
    {
        self.guard_mut_inner()
    }
}

/// Allows exposing an immutable reference to a temporary conversion.
pub trait IGuardRef<T: ?Sized> {
    /// The type of the guard which will clean up the temporary.
    type Guard<'a>: Deref<Target = T>
    where
        Self: 'a;
    /// Construct the temporary and guard it through an immutable reference.
    fn guard_ref_inner(&self) -> Self::Guard<'_>;
}

/// Allows exposing an mutable reference to a temporary conversion.
pub trait IGuardMut<T: ?Sized>: IGuardRef<T> {
    /// The type of the guard which will clean up the temporary after applying its changes to the original.
    type GuardMut<'a>: DerefMut<Target = T>
    where
        Self: 'a;
    /// Construct the temporary and guard it through a mutable reference.
    fn guard_mut_inner(&mut self) -> Self::GuardMut<'_>;
}

/// A guard exposing an immutable reference to `T` as an immutable reference to `As`
pub struct RefAs<'a, T, As> {
    source: core::marker::PhantomData<&'a T>,
    target: core::mem::ManuallyDrop<As>,
}
impl<'a, T, As> Deref for RefAs<'a, T, As> {
    type Target = As;
    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
impl<T: Into<As>, As: Into<T>> IGuardRef<As> for T {
    type Guard<'a> = RefAs<'a, T, As> where Self: 'a;

    fn guard_ref_inner(&self) -> Self::Guard<'_> {
        RefAs {
            source: core::marker::PhantomData,
            target: core::mem::ManuallyDrop::new(unsafe { core::ptr::read(self).into() }),
        }
    }
}

/// A guard exposing an mutable reference to `T` as an mutable reference to `As`
///
/// Failing to destroy this guard will cause `T` not to receive any of the changes applied to the guard.
pub struct MutAs<'a, T, As: Into<T>> {
    source: &'a mut T,
    target: core::mem::ManuallyDrop<As>,
}
impl<'a, T, As: Into<T>> Deref for MutAs<'a, T, As> {
    type Target = As;
    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
impl<'a, T, As: Into<T>> DerefMut for MutAs<'a, T, As> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.target
    }
}
impl<'a, T, As: Into<T>> Drop for MutAs<'a, T, As> {
    fn drop(&mut self) {
        unsafe { core::ptr::write(self.source, core::ptr::read(&*self.target).into()) }
    }
}
impl<T: Into<As>, As: Into<T>> IGuardMut<As> for T {
    type GuardMut<'a> = MutAs<'a, T, As> where Self: 'a;

    fn guard_mut_inner(&mut self) -> Self::GuardMut<'_> {
        MutAs {
            target: core::mem::ManuallyDrop::new(unsafe { core::ptr::read(self).into() }),
            source: self,
        }
    }
}
