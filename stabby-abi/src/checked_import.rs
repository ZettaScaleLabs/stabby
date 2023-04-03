use core::sync::atomic::{AtomicU8, Ordering};
#[crate::stabby]
pub struct CheckedImport<F> {
    checked: AtomicU8,
    result: core::cell::UnsafeCell<core::mem::MaybeUninit<F>>,
    checker: unsafe extern "C" fn(&crate::report::TypeReport) -> Option<F>,
    get_report: unsafe extern "C" fn() -> &'static crate::report::TypeReport,
    local_report: &'static crate::report::TypeReport,
}
unsafe impl<F> Send for CheckedImport<F> {}
unsafe impl<F> Sync for CheckedImport<F> {}

#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct ReportMismatch {
    pub local: &'static crate::report::TypeReport,
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
