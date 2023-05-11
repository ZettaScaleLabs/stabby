pub trait StabbyLibrary {
    /// Gets `symbol` from the library, using stabby's reports to check for compatibility.
    ///
    /// The library must have a symbol with the appropriate type named the same way, and marked with `#[stabby::export]`.
    ///
    /// # Safety
    /// Since this function calls foreign code, it is inherently unsafe.
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
    unsafe fn get_canaried<'a, T>(
        &'a self,
        symbol: &[u8],
    ) -> Result<libloading::Symbol<'a, T>, Box<dyn std::error::Error + Send + Sync>>;
}
pub struct Symbol<'a, T> {
    inner: T,
    lt: core::marker::PhantomData<&'a ()>,
}
impl<'a, T> core::ops::Deref for Symbol<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
mod canaries {
    stabby_abi::canary_suffixes!();
}
const STABBIED_SUFFIX: &[u8] = b"_stabbied";
const REPORT_SUFFIX: &[u8] = b"_stabbied_report";
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
            &[symbol, STABBIED_SUFFIX].concat(),
        )?;
        match stabbied(T::REPORT) {
            Some(f) => Ok(Symbol {
                inner: f,
                lt: core::marker::PhantomData,
            }),
            None => {
                let report = self
                    .get::<extern "C" fn() -> &'static crate::abi::report::TypeReport>(
                        &[symbol, REPORT_SUFFIX].concat(),
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
            if let Err(e) = self.get::<extern "C" fn()>(&[symbol, suffix.as_bytes()].concat()) {
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
