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

impl crate::IPtr for Box<()> {
    unsafe fn as_ref<U>(&self) -> &U {
        let this: &() = self;
        core::mem::transmute(this)
    }
}
impl crate::IPtrMut for Box<()> {
    unsafe fn as_mut<U>(&mut self) -> &mut U {
        let this: &mut () = self;
        core::mem::transmute(this)
    }
}
impl crate::IPtrOwned for Box<()> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        unsafe {
            (drop)(this);
            core::mem::ManuallyDrop::drop(this);
        }
    }
}

impl<T> crate::IntoDyn for Box<T> {
    type Anonymized = Box<()>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        unsafe { core::mem::transmute(self) }
    }
}
