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

/// An extension trait to load symbols from libraries while checking for ABI-compatibility.
pub trait StabbyLibrary {
    /// Gets `symbol` from the library, using stabby's reports to check for compatibility.
    ///
    /// The library must have a symbol with the appropriate type named the same way, and marked with `#[stabby::export]`.
    ///
    /// # Safety
    /// Since this function calls foreign code, it is inherently unsafe.
    ///
    /// # Errors
    /// If the symbol is not found OR reflection indicated an ABI-mismatch.
    ///
    /// The symbol missing can mean that the library was compiled with a different version of stabby, or that the symbol was not exported with `#[stabby::export]`.
    ///
    /// In case of ABI-mismatch, the error will contain a message indicating the expected and found type layouts.
    unsafe fn get_stabbied<'a, T: crate::IStable>(
        &'a self,
        symbol: &[u8],
    ) -> Result<Symbol<'a, T>, Box<dyn std::error::Error + Send + Sync>>;
    /// Gets `symbol` from the library, using stabby's canaries to check for compatibility.
    ///
    /// The library must have a symbol with the appropriate type named the same way, and marked with `#[stabby::export(canaries)]`.
    ///
    /// Note that while canaries greatly improve the chance ABI compatibility, they don't guarantee it.
    ///
    /// # Safety
    /// The symbol on the other side of the FFI boundary cannot be type-checked, and may still have a different
    /// ABI than expected (although the canaries should greatly reduce that risk).
    ///
    /// # Errors
    /// If the symbol is not found OR the canaries were not found.
    unsafe fn get_canaried<'a, T>(
        &'a self,
        symbol: &[u8],
    ) -> Result<libloading::Symbol<'a, T>, Box<dyn std::error::Error + Send + Sync>>;
}
/// A symbol bound to a library's lifetime.
pub struct Symbol<'a, T> {
    inner: T,
    lt: core::marker::PhantomData<&'a ()>,
}
impl<T> core::ops::Deref for Symbol<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
mod canaries {
    stabby_abi::canary_suffixes!();
}
const STABBIED_SUFFIX: &[u8] = b"_stabbied_v3";
const REPORT_SUFFIX: &[u8] = b"_stabbied_v3_report";
impl StabbyLibrary for libloading::Library {
    /// Gets `symbol` from the library, using stabby's reports to check for compatibility.
    ///
    /// The library must have a symbol with the appropriate type named the same way, and marked with `#[stabby::export]`.
    ///
    /// # Safety
    /// Since this function calls foreign code, it is inherently unsafe.
    unsafe fn get_stabbied<'a, T: crate::IStable>(
        &'a self,
        symbol: &[u8],
    ) -> Result<Symbol<'a, T>, Box<dyn std::error::Error + Send + Sync>> {
        let stabbied = self.get::<extern "C" fn(&crate::abi::report::TypeReport) -> Option<T>>(
            [symbol, STABBIED_SUFFIX].concat().as_slice(),
        )?;
        match stabbied(T::REPORT) {
            Some(f) => Ok(Symbol {
                inner: f,
                lt: core::marker::PhantomData,
            }),
            None => {
                let report = self
                    .get::<extern "C" fn() -> &'static crate::abi::report::TypeReport>(
                        [symbol, REPORT_SUFFIX].concat().as_slice(),
                    )?;
                let report = report();
                Err(format!(
                    "Report mismatch: loader({loader}),  lib({report}",
                    loader = T::REPORT
                )
                .into())
            }
        }
    }
    /// Gets `symbol` from the library, using stabby's canaries to check for compatibility.
    ///
    /// The library must have a symbol with the appropriate type named the same way, and marked with `#[stabby::export(canaries)]`.
    ///
    /// Note that while canaries greatly improve the chance ABI compatibility, they don't guarantee it.
    ///
    /// # Safety
    /// The symbol on the other side of the FFI boundary cannot be type-checked, and may still have a different
    /// ABI than expected (although the canaries should greatly reduce that risk).
    unsafe fn get_canaried<'a, T>(
        &'a self,
        symbol: &[u8],
    ) -> Result<libloading::Symbol<'a, T>, Box<dyn std::error::Error + Send + Sync>> {
        let stabbied = self.get::<T>(symbol)?;
        for suffix in [
            canaries::CANARY_RUSTC,
            canaries::CANARY_OPT_LEVEL,
            canaries::CANARY_DEBUG,
            canaries::CANARY_TARGET,
            canaries::CANARY_NUM_JOBS,
        ] {
            if let Err(e) =
                self.get::<extern "C" fn()>([symbol, suffix.as_bytes()].concat().as_slice())
            {
                return Err(format!(
                    "Canary {symbol}{suffix} not found: {e}",
                    symbol = std::str::from_utf8_unchecked(symbol)
                )
                .into());
            }
        }
        Ok(stabbied)
    }
}
