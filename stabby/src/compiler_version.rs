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

//!	Provides ZSTs that only implement `IStable` when built with their corresponding version of the compiler.
//!
//! This allow the `StableAs<T, CompilerVersion<Layout>>` pattern.
//!
//! `CompilerVersion_MAJ_MIN_PATCH<Layout>` will only `impl IStable` as if it was `Layout`, but only if
//! compiled with the specified version of the compiler, providing you with a compile-time proof that you
//! are using the expected compiler version.
//!
//! Note that it is EXTREMELY memory-unsafe to lie about `Layout` if any type that contains this is
//! used in a `#[repr(stabby)]` enum, since `CompilerVersion<Layout>` is ALWAYS a ZST, and non-`()`
//! layouts should only be used in combination with `StableAs<T, Layout>`.
//!
//! You can also add a `compiler_version: CompilerVersion_VERSION<()>` marker field in your structs to ensure
//! that they are marked as stable only if compiled with the appropriate compiler version, however since the
//! rest of the fields of the struct need to bi ABI-stable for IStable to be implemented, I think the
//! applications are few and far between.

include!(concat!(env!("OUT_DIR"), "/compiler_versions.rs"));
