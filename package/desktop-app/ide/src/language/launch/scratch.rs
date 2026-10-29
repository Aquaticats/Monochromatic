//! The private temporary directory of one process for one server.

use std::path::{Path, PathBuf};

/// What:
///  The lock file of a scratch directory,
///  beside it.
pub(super) fn lock_path(directory: &Path) -> PathBuf {
    let mut name = directory.file_name().unwrap_or_default().to_os_string();
    name.push(".lock");
    return directory.with_file_name(name);
}

/// What:
///  The name of this process's scratch directory below a server's private state.
/// Why:
///  Placeholder: every process shares one name.
pub fn scratch_name() -> &'static str {
    return "tmp";
}

/// What:
///  Take the right to use a scratch directory.
/// Why:
///  Placeholder: nothing is claimed.
pub(super) fn claim(_directory: &Path) -> Result<(), String> {
    return Ok(());
}

/// What:
///  Remove the scratch directories of ended processes beside this one's.
/// Why:
///  Placeholder: nothing is removed.
pub(super) fn reap_ended(_directory: &Path) {}

/// The ownership and cleanup rules of the scratch directories.
#[cfg(test)]
#[path = "scratch_tests.rs"]
mod tests;
