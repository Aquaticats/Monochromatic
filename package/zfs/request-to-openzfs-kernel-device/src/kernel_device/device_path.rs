//! What:
//!  The one path OpenZFS serves kernel requests on.
//! Why:
//!  `ZfsDevice::open` needs the path,
//! and the handle keeps the path it was opened on,
//! so an open failure prints that stored value and the literal lives here
//! alone.

/// The path of the device file every request travels on.
pub const DEVICE_PATH: &str = "/dev/zfs";
