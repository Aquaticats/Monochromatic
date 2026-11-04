//! What:
//! The notice handshake between shadow instances:
//! a marker file that tells an outer instance that an inner instance
//! already printed the version or help notice.
//! Why:
//! A launcher can put one shadow instance in front of another.
//! Only the innermost instance knows which cargo really ran,
//! so it prints the notice;
//! the outer instance cannot see that and would print a second,
//! wrong one naming the launcher.
//! The inner instance leaves a marker named for the outer instance's process id,
//! and the outer instance claims it after
//! cargo finishes.
//! Without a runtime directory the handshake is off and an outer instance prints its own notice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // notice-marker.ts
//! export function leaveMarker(dir: string, pid: number): void { writeFileSync(markerPath(dir, pid), ''); }
//! export function claimMarker(dir: string, pid: number): boolean { try { unlinkSync(markerPath(dir, pid)); return true; } catch { return false; } }
//! ```

/// Imports the file-system module.
use std::fs;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports the fixed values.
use crate::constants::{NOTICE_MARKER_PREFIX, RUNTIME_DIR_VARIABLE};

/// What:
/// `pub fn runtime_dir() -> Option<PathBuf>` is the per-user runtime directory from the environment,
/// or `None` when it is
/// unset or empty.
/// Why:
/// The marker lives there and not in a system temporary directory,
/// which this repository's lints forbid and whose device
/// is unknowable;
/// with no runtime directory the handshake is simply off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runtimeDir(): string | undefined { return process.env.XDG_RUNTIME_DIR || undefined; }
/// ```
pub fn runtime_dir() -> Option<PathBuf> {
    match std::env::var(RUNTIME_DIR_VARIABLE) {
        Ok(text) if !text.is_empty() => return Some(PathBuf::from(text)),
        _ => return None,
    }
}

/// What:
/// `pub fn marker_path_in(dir: &Path, pid: u32) -> PathBuf` is the marker file for the instance with this process id.
/// Why:
/// One name rule for the writer and the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markerPathIn(dir: string, pid: number): string { return join(dir, `cargo-shadow-notice-${pid}`); }
/// ```
pub fn marker_path_in(dir: &Path, pid: u32) -> PathBuf {
    return dir.join(format!("{NOTICE_MARKER_PREFIX}{pid}"));
}

/// What:
/// `pub fn leave_marker_in(dir: &Path, pid: u32)` creates the empty marker for the instance with this process id.
/// Why:
/// It tells that instance that a notice was already printed below it.
/// A failure to write is ignored:
/// the worst result is
/// a second notice,
/// never a failed cargo run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function leaveMarkerIn(dir: string, pid: number): void
/// ```
pub fn leave_marker_in(dir: &Path, pid: u32) {
    // `let _ =` discards the write result on purpose.
    let _ = fs::write(marker_path_in(dir, pid), "");
}

/// What:
/// `pub fn claim_marker_in(dir: &Path, pid: u32) -> bool` removes the marker for this process id and says whether it was there.
/// Why:
/// Removing it is the check and the cleanup in one step,
/// so two readers can never both believe they claimed it,
/// and no
/// marker outlives its run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function claimMarkerIn(dir: string, pid: number): boolean
/// ```
pub fn claim_marker_in(dir: &Path, pid: u32) -> bool {
    return fs::remove_file(marker_path_in(dir, pid)).is_ok();
}
