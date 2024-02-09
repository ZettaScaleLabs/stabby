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

fn main() {
    use stabby::libloading::StabbyLibrary;
    unsafe {
        let path = if cfg!(target_os = "linux") {
            "./target/debug/liblibrary.so"
        } else if cfg!(target_os = "windows") {
            "./target/debug/library.dll"
        } else if cfg!(target_os = "macos") {
            "./target/debug/library.dylib"
        } else {
            ""
        };
        let lib = libloading::Library::new(path).unwrap_or_else(|e| {
            panic!(
                "{e}\n\nreaddir(./target/debug)={:?}",
                std::fs::read_dir("./target/debug")
            )
        });
        let stable_fn = lib.get_stabbied::<extern "C" fn(u8)>(b"stable_fn").unwrap();
        let unstable_fn = lib
            .get_canaried::<extern "C" fn(&[u8])>(b"unstable_fn")
            .unwrap();
        stable_fn(5);
        unstable_fn(&[1, 2, 3, 4]);
    }
}
