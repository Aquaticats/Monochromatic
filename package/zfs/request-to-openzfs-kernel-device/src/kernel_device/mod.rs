//! What:
//!  The live calls:
//! the opened `/dev/zfs` handle,
//! the one `ioctl` call,
//!  the bounded growth of a read's reply,
//! and one typed method per request the crate
//!  can express.
//! Why:
//!  A packed request is only half of a conversation:
//! the kernel owns the other
//!  half,
//! and it answers in one call whose result is an error number and,
//!  for a read,
//! a list written into the caller's buffer.
//! Keeping the unsafe call in one
//!  file and every retry rule in another lets the device-free tests prove the
//!  decisions,
//! while the guest fixture proves the calls against the running module.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `device_error.rs`:
//!   why a live call could not be made or answered.
//! - `call_outcome.rs`:
//!   what one call's error number and reported size mean.
//! - `ioctl_call.rs`:
//!   the one unsafe `libc::ioctl`,
//!   and nothing else.
//! - `read_reply.rs`:
//!   how a read grows its reply and is asked again.
//! - `listed_object.rs`:
//!   what one step of a listing returns.
//! - `device_path.rs`:
//!   the one path OpenZFS serves requests on.
//! - `device.rs`:
//!   the open handle,
//!   one attempt,
//!   and the read and write paths.
//! - `read_requests.rs`:
//!   the five reads.
//! - `write_requests.rs`:
//!   the six writes,
//!   promote included for the tests only.

/// The type an answer's error number and size resolve to.
mod call_outcome;
/// The open handle and the two call paths.
mod device;
/// Why a live call could not be made or answered.
mod device_error;
/// The device file's path.
mod device_path;
/// The one unsafe call.
mod ioctl_call;
/// What one step of a listing returns.
mod listed_object;
/// How a read grows its reply and is asked again.
mod read_reply;
/// The five reads.
mod read_requests;
/// The six writes.
mod write_requests;

/// The open `/dev/zfs` handle and its typed methods.
pub use device::ZfsDevice;
/// The failure type every live call returns.
pub use device_error::DeviceError;
/// The path of the one device file OpenZFS serves requests on.
pub use device_path::DEVICE_PATH;
/// The one step of a listing.
pub use listed_object::{DatasetIdentity, ListedObject};

/// The guest fixture the live tests build their pools in,
/// compiled for tests only.
#[cfg(test)]
mod fixture;
/// The encrypted pool a live test builds,
/// compiled for tests only.
#[cfg(test)]
mod fixture_pool;
/// The readers the live tests use on a kernel answer,
/// compiled for tests only.
#[cfg(test)]
mod fixture_reply;

/// The device-free tests of the call outcome.
#[cfg(test)]
#[path = "call_outcome_tests.rs"]
mod call_outcome_tests;
/// The device-free tests of the live failure type.
#[cfg(test)]
#[path = "device_error_tests.rs"]
mod device_error_tests;
/// The guest fixture's own checks.
#[cfg(test)]
#[path = "fixture_tests.rs"]
mod fixture_tests;
/// The delegated fixture the root run prepares.
#[cfg(test)]
#[path = "live_account_fixture_tests.rs"]
mod live_account_fixture_tests;
/// The live delegated-account tests.
#[cfg(test)]
#[path = "live_account_tests.rs"]
mod live_account_tests;
/// The arm that prints what the plan's items returned.
#[cfg(test)]
#[path = "live_measurement_record_tests.rs"]
mod live_measurement_record_tests;
/// The live batch tests:
/// per-name errors and GUID preservation.
#[cfg(test)]
#[path = "live_batch_tests.rs"]
mod live_batch_tests;
/// The `zfs_cmd` and statistics field tables,
/// compiled for tests only.
#[cfg(test)]
mod live_btf_fields;
/// The field tables of the records `zfs_cmd` holds,
/// compiled for tests only.
#[cfg(test)]
mod live_btf_nested_fields;
/// The live layout comparison against the running module's type information.
#[cfg(test)]
#[path = "live_btf_tests.rs"]
mod live_btf_tests;
/// The live promote controls.
#[cfg(test)]
#[path = "live_promote_tests.rs"]
mod live_promote_tests;
/// The live read tests:
/// cursor,
/// property sources and reply growth.
#[cfg(test)]
#[path = "live_read_tests.rs"]
mod live_read_tests;
/// The live write tests:
/// encoding,
/// properties,
/// a head destroy and the unloaded key.
#[cfg(test)]
#[path = "live_write_tests.rs"]
mod live_write_tests;
/// The device-free tests of the read retry rule.
#[cfg(test)]
#[path = "read_reply_tests.rs"]
mod read_reply_tests;
