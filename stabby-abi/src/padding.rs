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

use crate::*;

use super::istable::{IBitMask, IForbiddenValues, Saturator};

// #[crate::stabby]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Padded<Left: Unsigned, T> {
    pub lpad: Left::Padding,
    pub value: T,
}
unsafe impl<Left: Unsigned, T: IStable> IStable for Padded<Left, T> {
    type Size = Left::Add<T::Size>;
    type Align = T::Align;
    type ForbiddenValues = <T::ForbiddenValues as IForbiddenValues>::Shift<Left>;
    type UnusedBits = <<Left::Padding as IStable>::UnusedBits as IBitMask>::BitOr<
        <T::UnusedBits as IBitMask>::Shift<Left>,
    >;
    type HasExactlyOneNiche = Saturator;
    const REPORT: &'static report::TypeReport = T::REPORT;
}
impl<Left: Unsigned, T> From<T> for Padded<Left, T> {
    fn from(value: T) -> Self {
        Self {
            lpad: Default::default(),
            value,
        }
    }
}
impl<Left: Unsigned, T> core::ops::Deref for Padded<Left, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<Left: Unsigned, T> core::ops::DerefMut for Padded<Left, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
