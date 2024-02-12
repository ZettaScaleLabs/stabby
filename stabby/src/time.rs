use core::sync::atomic::{AtomicI64, Ordering};

/// A stable equivalent to [`core::time::Duration`]
#[crate::stabby]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    /// The number of seconds elapsed.
    pub secs: u64,
    /// The number of subsecond nanos elapsed.
    pub nanos: u32,
}
impl Duration {
    /// Construct a new [`Duration`].
    pub const fn new(secs: u64, subsec_nanos: u32) -> Self {
        Self {
            secs,
            nanos: subsec_nanos,
        }
    }
    /// Construct a new [`Duration`].
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / 1000,
            nanos: ((millis % 1000) * 1000000) as u32,
        }
    }
    /// Construct a new [`Duration`].
    pub const fn from_micros(micros: u64) -> Self {
        Self {
            secs: micros / 1000000,
            nanos: ((micros % 1000000) * 1000) as u32,
        }
    }
    /// Construct a new [`Duration`].
    /// # Panics
    /// if `secs` is negative.
    pub fn from_secs_f64(secs: f64) -> Self {
        assert!(secs >= 0.);
        Self {
            secs: secs.floor() as u64,
            nanos: ((secs % 1.) * 1_000_000_000.) as u32,
        }
    }
    /// Returns the number of seconds in the duration.
    pub const fn as_secs(&self) -> u64 {
        self.secs
    }
    /// Returns the total number of nanoseconds in the duration.
    pub const fn as_nanos(&self) -> u128 {
        self.secs as u128 * 1_000_000_000 + self.nanos as u128
    }
    /// Returns the number of seconds in the duration, including sub-seconds.
    pub fn as_secs_f64(&self) -> f64 {
        self.as_nanos() as f64 / 1_000_000_000.
    }
    /// Returns the number of nanoseconds after the last second of the duration.
    pub const fn subsec_nanos(&self) -> u32 {
        self.nanos
    }
    /// Returns the number of microseconds after the last second of the duration.
    pub const fn subsec_micros(&self) -> u32 {
        self.subsec_nanos() / 1000
    }
    /// Returns the number of milliseconds after the last second of the duration.
    pub const fn subsec_millis(&self) -> u32 {
        self.subsec_nanos() / 1000000
    }
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

/// A signed [`Duration`] represented as a single [`AtomicI64`], allowing to change its value atomically.
///
/// Its resolution is 1μs, and the maximum encodable duration is 278737 years.
#[crate::stabby]
pub struct AtomicDuration {
    t: AtomicI64,
}
const SHIFT: i64 = 20;
const MASK: i64 = 0xfffff;
/// A sign to be paired with a [`Duration`].
#[crate::stabby]
#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    #[default]
    /// +
    Positive,
    /// -
    Negative,
}
impl AtomicDuration {
    const fn i64_to_duration(mut t: i64) -> (Duration, Sign) {
        let sign = if t.is_negative() {
            t = -t;
            Sign::Negative
        } else {
            Sign::Positive
        };
        let micros = (t & MASK) as u32;
        let secs = t >> SHIFT;
        (Duration::new(secs as u64, micros * 1000), sign)
    }
    const fn duration_to_i64(t: Duration, sign: Sign) -> i64 {
        let t = ((t.as_secs() as i64) << SHIFT) + (t.subsec_micros() as i64);
        match sign {
            Sign::Positive => t,
            Sign::Negative => -t,
        }
    }
    /// This type's time resolution.
    pub const RESOLUTION: Duration = Self::i64_to_duration(1).0;
    /// This type's maximum value.
    pub const MAX: Duration = Self::i64_to_duration(i64::MAX).0;
    /// Atomically loads the stored value, converting it to a duration-sign tuple.
    ///
    /// The [`Ordering`] is used in a single `load` operation.
    pub fn load(&self, ord: Ordering) -> (Duration, Sign) {
        Self::i64_to_duration(self.t.load(ord))
    }
    /// Converts the duration-sign tuple into before storing it atomically.
    ///
    /// The [`Ordering`] is used in a single `store` operation.
    pub fn store(&self, duration: Duration, sign: Sign, ord: Ordering) {
        self.t.store(Self::duration_to_i64(duration, sign), ord)
    }
    /// Perform a [`AtomicDuration::load`] and [`AtomicDuration::store`] in a single atomic operation
    pub fn swap(&self, duration: Duration, sign: Sign, ord: Ordering) -> (Duration, Sign) {
        Self::i64_to_duration(self.t.swap(Self::duration_to_i64(duration, sign), ord))
    }
    /// Construct a new [`AtomicDuration`]
    pub const fn new(duration: Duration, sign: Sign) -> Self {
        Self {
            t: AtomicI64::new(Self::duration_to_i64(duration, sign)),
        }
    }
}

#[cfg(feature = "std")]
pub use impls::{AtomicInstant, Instant, SystemTime};

#[cfg(feature = "std")]
mod impls {
    use super::{AtomicDuration, Duration};
    use core::sync::atomic::Ordering;
    use std::time::UNIX_EPOCH;
    /// A stable equivalent to [`std::time::SystemTime`].
    /// # Stability
    /// It is always represented as a duration since [`std::time::UNIX_EPOCH`].
    #[crate::stabby]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SystemTime(pub(crate) Duration);
    impl SystemTime {
        /// An anchor in time which can be used to create new SystemTime instances or learn about where in time a SystemTime lies.
        ///
        /// This constant is defined to be "1970-01-01 00:00:00 UTC" on all systems with respect to the system clock. Using duration_since on an existing SystemTime instance can tell how far away from this point in time a measurement lies, and using UNIX_EPOCH + duration can be used to create a SystemTime instance to represent another fixed point in time.
        pub const UNIX_EPOCH: Self = SystemTime(Duration { secs: 0, nanos: 0 });
        /// Measure the current [`SystemTime`].
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
    /// - Unix systems use [`libc::CLOCK_MONOTONIC`](https://docs.rs/libc/latest/libc/constant.CLOCK_MONOTONIC.html), which is system-global.
    /// - MacOS use [`libc::CLOCK_UPTIME_RAW`](https://docs.rs/libc/latest/libc/constant.CLOCK_UPTIME_RAW.html), which is system-global.
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
    /// While very likely to be true, this is unverified yet for niche platforms.
    /// Please write an issue on [stabby's official repo](https://github.com/ZettaScaleLabs/stabby)
    /// if you have proof either way for your system of choice, and it will be added
    /// to this documentation.
    #[crate::stabby]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(pub(crate) Duration);
    impl Instant {
        /// The epoch of the [`Instant`] type.
        pub const fn zero() -> Self {
            Self(Duration { secs: 0, nanos: 0 })
        }
        /// Measure the current [`Instant`].
        pub fn now() -> Self {
            std::time::Instant::now().into()
        }
    }
    #[rustversion::attr(since(1.75), const)]
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

    /// An [`Instant`] stored as an [`AtomicDuration`] since [`Instant::zero`]
    #[crate::stabby]
    pub struct AtomicInstant(pub(crate) AtomicDuration);
    impl AtomicInstant {
        /// Measure the current time into a new [`AtomicInstant`].
        pub fn now() -> Self {
            Self(AtomicDuration::new(
                instant_epoch().elapsed().into(),
                super::Sign::Positive,
            ))
        }
        /// Construct the epoch for [`AtomicInstant`].
        pub const fn epoch() -> Self {
            Self(AtomicDuration::new(
                Duration::new(0, 0),
                super::Sign::Positive,
            ))
        }
        /// Atomically update `self` to [`Instant::now`] while returning its previous value.
        pub fn update(&self, ordering: Ordering) -> Instant {
            Instant(
                self.0
                    .swap(
                        instant_epoch().elapsed().into(),
                        super::Sign::Positive,
                        ordering,
                    )
                    .0,
            )
        }
        /// Atomically update `self` to `instant` while returning its previous value.
        pub fn swap(&self, instant: Instant, ordering: Ordering) -> Instant {
            Instant(self.0.swap(instant.0, super::Sign::Positive, ordering).0)
        }
        /// Atomically read `self`.
        pub fn load(&self, ordering: Ordering) -> Instant {
            Instant(self.0.load(ordering).0)
        }
        /// Atomically write `instant` to `self`.
        pub fn store(&self, instant: Instant, ordering: Ordering) {
            self.0.store(instant.0, super::Sign::Positive, ordering)
        }
    }
}
