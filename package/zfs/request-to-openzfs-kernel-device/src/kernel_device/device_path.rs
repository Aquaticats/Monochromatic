//! What:
//!  The one path OpenZFS serves kernel requests on.
//! Why:
//!  Opening the device and formatting a diagnostic both need the path,
//! and a
//!  literal in two places is one that can disagree with itself.

/// The path of the device file every request travels on.
pub const DEVICE_PATH: &str = "/dev/zfs";
