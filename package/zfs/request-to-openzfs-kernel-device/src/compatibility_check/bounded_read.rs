//! What:
//!  The one read the gate performs on a file root controls:
//! open without
//!  following a link,
//! read to a bound,
//! and refuse a file bigger than the bound.
//! Why:
//!  Every file the gate reads is root's:
//! `/sys/module/zfs/version`,
//! `srcversion`,
//!  the build-id note,
//! and the two type-information files.
//! A link below `/sys` would be
//!  a path this process follows on root's instruction,
//! so the open refuses one,
//! and
//!  the read bound keeps a file that grew since it was measured from filling
//! this
//!  process's memory before a single byte is judged.
//!  The three bounds are the writer's,
//! pending the design's owed start-cost measurement,
//!  which sets the base bound and the peak-memory threshold together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function readBounded({ path, bound }: ReadRequest): { bytes: Uint8Array };
//! ```

/// The step vocabulary and the refusal builders.
use super::refusal::{Refusal, Step, refusal_at};
/// The read trait the bounded loop uses.
use std::io::Read;
/// The open options that carry the flags.
use std::fs::OpenOptions;
/// The borrowed path type of one read.
use std::path::Path;
/// The one open flag set:
/// no link below the path,
/// no descriptor inherited.
use std::os::unix::fs::OpenOptionsExt;

/// The most bytes one identity file may hold:
/// the sysfs page it is created as.
pub const IDENTITY_FILE_BOUND: u64 = 4_096;
/// The most bytes the kernel's base type information may hold.
///
/// The bound is the writer's,
/// a multiple of the 6,579,400 bytes this host exposes,
/// and the
/// design's owed start-cost measurement sets it with the peak-memory threshold.
pub const BASE_BTF_BOUND: u64 = 64 * 1024 * 1024;
/// The most bytes a loaded module's type information may hold.
///
/// The bound is the writer's,
/// a multiple of the 570,234 bytes this host's module
/// exposes,
/// and the design's owed start-cost measurement sets it with the base bound.
pub const MODULE_BTF_BOUND: u64 = 16 * 1024 * 1024;

/// What:
///  One file the gate reads.
/// Why:
///   The path and the bound are all a read needs;
/// the step says which refusal a failure
///  is reported as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadRequest<'a> = { path: &'a Path; bound: u64; step: Step };
/// ```
pub struct ReadRequest<'a> {
    /// The file to read.
    pub path: &'a Path,
    /// The most bytes it may hold.
    pub bound: u64,
    /// The step a failure of this read is reported as.
    pub step: Step,
}

/// Read one file root controls,
/// refusing a link,
/// a failure and a file past the bound.
///
/// # Errors
/// A refusal naming the step,
/// the path and why the read could not finish.
pub fn read_bounded(request: &ReadRequest<'_>) -> Result<Vec<u8>, Refusal> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(request.path)
        .map_err(|error| {
            return refusal_at(
                request.step,
                request.path,
                format!("it could not be opened: {error}"),
            );
        })?;
    let bytes: Vec<u8> = read_to_the_end(&mut file, request)?;
    return Ok(bytes);
}

/// Read the whole file,
/// or refuse the byte past the bound.
///
/// # Errors
/// A refusal naming the step,
/// the path and the bound the file passed.
fn read_to_the_end(file: &mut std::fs::File, request: &ReadRequest<'_>) -> Result<Vec<u8>, Refusal> {
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    let mut limited = file.take(request.bound.saturating_add(1));
    let read: u64 = limited.read_to_end(&mut bytes).map_err(|error| {
            return refusal_at(
                request.step,
                request.path,
                format!("it could not be read: {error}"),
            );
        })? as u64;
    if read > request.bound {
        return Err(refusal_at(
            request.step,
            request.path,
            format!(
                "it holds more than the {} bytes this read accepts",
                request.bound
            ),
        ));
    }
    return Ok(bytes);
}
