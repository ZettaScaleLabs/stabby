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

#[stabby::import(name = "library")]
extern "C" {
    pub fn stable_fn(v: u8);
}

#[stabby::import(canaries = "", name = "library")]
#[allow(improper_ctypes)]
extern "C" {
    pub fn unstable_fn(v: &[u8]);
}

fn main() {
    stable_fn(5);
    unsafe { unstable_fn(&[1, 2, 3, 4]) };
}
