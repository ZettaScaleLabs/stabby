/// A stable equivalent to [`core::time::Duration`]
#[crate::stabby]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}
impl core::ops::AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        let nanos = self.nanos + rhs.nanos;
        self.secs += rhs.secs + (nanos / 1_000_000_000) as u64;
        self.nanos = nanos % 1_000_000_000;
    }
}
impl core::ops::Add for Duration {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}
impl From<core::time::Duration> for Duration {
    fn from(value: core::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}
impl From<Duration> for core::time::Duration {
    fn from(value: Duration) -> Self {
        Self::new(value.secs, value.nanos)
    }
}

#[cfg(feature = "std")]
mod impls {
    use super::Duration;
    use std::time::UNIX_EPOCH;
    /// A stable equivalent to [`std::tine::SystemTime`].
    /// # Stability
    /// It is always represented as a duration since [`std::time::UNIX_EPOCH`].
    #[crate::stabby]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SystemTime(pub(crate) Duration);
    impl SystemTime {
        pub const UNIX_EPOCH: Self = SystemTime(Duration { secs: 0, nanos: 0 });
        pub fn now() -> Self {
            std::time::SystemTime::now().into()
        }
    }
    impl core::ops::AddAssign<Duration> for SystemTime {
        fn add_assign(&mut self, rhs: Duration) {
            self.0.add_assign(rhs)
        }
    }
    impl core::ops::Add<Duration> for SystemTime {
        type Output = Self;
        fn add(mut self, rhs: Duration) -> Self {
            self += rhs;
            self
        }
    }

    impl From<std::time::SystemTime> for SystemTime {
        fn from(value: std::time::SystemTime) -> Self {
            Self(
                value
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(core::time::Duration::new(0, 0))
                    .into(),
            )
        }
    }
    impl From<SystemTime> for std::time::SystemTime {
        fn from(value: SystemTime) -> Self {
            UNIX_EPOCH + value.0.into()
        }
    }

    /// A stable equivalent to [`std::time::Instant`].
    ///
    /// # Stability
    /// It is always represented as a duration since a mem-zeroed [`std::time::Instant`].
    ///
    /// ## Verified platforms
    /// Platforms where [`Instant`] is known to be stable accross processes:
    /// - Unix systems use [`libc::CLOCK_MONOTONIC`], which is system-global.
    /// - MacOS use [`libc::CLOCK_UPTIME_RAW`], which is system-global.
    /// - Windows uses performance counters, and [states](https://learn.microsoft.com/en-us/windows/win32/sysinfo/acquiring-high-resolution-time-stamps#guidance-for-acquiring-time-stamps) that said counters are consistent accross processes, except on platforms that don't provide consistent multi-core counters on pre-Vista systems
    ///
    /// Platforms where [`Instant`] is only known to be stable within a process:
    /// - None to date
    ///
    /// Platforms where [`Instant`] is only known to be unstable accross dynamic linkage units:
    /// - None to date, if such a platform is discovered and distinguishable, its support for
    /// [`Instant`] will be retracted until a stable representation is found.
    ///
    /// ## Doubts
    /// While this representation should work on most platforms, it assumes that within a
    /// given process, but accross dynamic linkage units, the OS will use the same clock
    /// to construct [`std::time::Instant`].
    ///
    /// While very likely to be true, this is unverified yet for most platforms.
    /// Please write an issue on [stabby's official repo](https://github.com/ZettaScaleLabs/stabby)
    /// if you have proof either way for your system of choice, and it will be added
    /// to this documentation.
    #[crate::stabby]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(pub(crate) Duration);
    impl Instant {
        pub fn zero() -> Self {
            Self(Duration { secs: 0, nanos: 0 })
        }
    }
    fn instant_epoch() -> std::time::Instant {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }
    impl core::ops::AddAssign<Duration> for Instant {
        fn add_assign(&mut self, rhs: Duration) {
            self.0.add_assign(rhs)
        }
    }
    impl core::ops::Add<Duration> for Instant {
        type Output = Self;
        fn add(mut self, rhs: Duration) -> Self {
            self += rhs;
            self
        }
    }
    impl From<std::time::Instant> for Instant {
        fn from(value: std::time::Instant) -> Self {
            Self(value.duration_since(instant_epoch()).into())
        }
    }
    impl From<Instant> for std::time::Instant {
        fn from(value: Instant) -> Self {
            instant_epoch() + value.0.into()
        }
    }
}
