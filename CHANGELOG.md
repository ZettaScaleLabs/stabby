# 1.0.8
- Fix duplicated bounds on structures that would cause compile errors when a structure had several fields of the same type

# 1.0.7
- Actually expose `stabby::time::{Instant, SystemTime}`

# 1.0.6
- Add trait implementations to `stabby::time::{Duration, Instant, SystemTime}`.
- Improve release process (releases are now based on changelogs, which should become more accurate)

# 1.0.5
- Marked `std::os::fd::{OwnedFd, BorrowedFd}` as stable.
- Added support for `core::time::Duration` and `std::time::{Instant, SystemTime}` through equivalent types.

# 1.0.4
- Added support for `core::iter::Iterator`.
- Made release process more reliable.

# 1.0.3
- Added support for some of `abi_stable`'s types
- Made checks for potential ABI misreports better

# 1.0.2: Accidental repeat of 1.0.1
# 1.0.1
- Fix cyclic trait bounds arising when a stabby trait depended on a dyn-self

# 1.0.0
This is the base release of this CHANGELOG. Please refer to its README for more information.