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

use core::{
    ops::Deref,
    sync::atomic::{AtomicU8, Ordering},
};
/// Used in `#[stabby::import(canaries)]`
#[crate::stabby]
pub struct CanariedImport<F> {
    result: F,
    checked: AtomicU8,
    canary: extern "C" fn(),
}
unsafe impl<F> Send for CanariedImport<F> {}
unsafe impl<F> Sync for CanariedImport<F> {}
impl<F> CanariedImport<F> {
    /// Used in `#[stabby::import(canaries)]`
    pub const fn new(source: F, canary_caller: extern "C" fn()) -> Self {
        Self {
            result: source,
            checked: AtomicU8::new(0),
            canary: canary_caller,
        }
    }
}
impl<F> Deref for CanariedImport<F> {
    type Target = F;
    fn deref(&self) -> &Self::Target {
        if self.checked.swap(1, Ordering::Relaxed) == 0 {
            (self.canary)()
        }
        &self.result
    }
}

/// Used in `#[stabby::import]`
#[crate::stabby]
pub struct CheckedImport<F> {
    result: core::cell::UnsafeCell<core::mem::MaybeUninit<F>>,
    checked: AtomicU8,
    #[allow(improper_ctypes_definitions)]
    checker: unsafe extern "C" fn(&crate::report::TypeReport) -> Option<F>,
    get_report: unsafe extern "C" fn() -> &'static crate::report::TypeReport,
    local_report: &'static crate::report::TypeReport,
}
unsafe impl<F> Send for CheckedImport<F> {}
unsafe impl<F> Sync for CheckedImport<F> {}

/// When reports mismatch between loader and loadee, both reports are exposed to allow debuging the issue.
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct ReportMismatch {
    /// The report on loader side.
    pub local: &'static crate::report::TypeReport,
    /// The report on loadee side.
    pub loaded: &'static crate::report::TypeReport,
}
impl core::fmt::Display for ReportMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ReportMismatch {}

const UNCHECKED: u8 = 0;
const VALIDATED: u8 = 1;
const INVALIDATED: u8 = 2;
const LOCKED: u8 = 3;
impl<F> CheckedImport<F> {
    /// Used by `#[stabby::import]` proc-macro
    #[allow(improper_ctypes_definitions)]
    pub const fn new(
        checker: unsafe extern "C" fn(&crate::report::TypeReport) -> Option<F>,
        get_report: unsafe extern "C" fn() -> &'static crate::report::TypeReport,
        local_report: &'static crate::report::TypeReport,
    ) -> Self {
        Self {
            checked: AtomicU8::new(UNCHECKED),
            checker,
            get_report,
            local_report,
            result: core::cell::UnsafeCell::new(core::mem::MaybeUninit::uninit()),
        }
    }
    fn error_report(&self) -> ReportMismatch {
        ReportMismatch {
            local: self.local_report,
            loaded: unsafe { (self.get_report)() },
        }
    }
    /// # Errors
    /// Returns a [`ReportMismatch`] if the local and loaded reports differ.
    pub fn as_ref(&self) -> Result<&F, ReportMismatch> {
        loop {
            match self.checked.load(Ordering::Relaxed) {
                UNCHECKED => match unsafe { (self.checker)(self.local_report) } {
                    Some(result) => {
                        if self
                            .checked
                            .compare_exchange_weak(
                                UNCHECKED,
                                LOCKED,
                                Ordering::SeqCst,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            unsafe {
                                (*self.result.get()).write(result);
                                self.checked.store(VALIDATED, Ordering::SeqCst);
                                return Ok((*self.result.get()).assume_init_ref());
                            }
                        }
                    }
                    None => {
                        self.checked.store(INVALIDATED, Ordering::Relaxed);
                        return Err(self.error_report());
                    }
                },
                VALIDATED => return Ok(unsafe { (*self.result.get()).assume_init_ref() }),
                INVALIDATED => return Err(self.error_report()),
                _ => {}
            }
            core::hint::spin_loop();
        }
    }
}
impl<F> core::ops::Deref for CheckedImport<F> {
    type Target = F;
    fn deref(&self) -> &Self::Target {
        self.as_ref().unwrap()
    }
}
impl<F> Drop for CheckedImport<F> {
    fn drop(&mut self) {
        if self.checked.load(Ordering::Relaxed) == VALIDATED {
            unsafe { self.result.get_mut().assume_init_drop() }
        }
    }
}
