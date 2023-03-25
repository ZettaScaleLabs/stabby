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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

//! Provides ZSTs that allow the `StableIf<StableAs<T, Layout>, CompilerVersion>` pattern.
//!
//! `CompilerVersion_MAJ_MIN_PATCH` will only `impl IStable` if compiled with that exact version
//! of the compiler, providing you with a compile-time proof that you are using the expected compiler version.
//!
//! You can also add a `compiler_version: CompilerVersion_VERSION<()>` marker field in your structs to ensure that they are marked as stable only if compiled with the appropriate compiler version.

include!(concat!(env!("OUT_DIR"), "/compiler_versions.rs"));
