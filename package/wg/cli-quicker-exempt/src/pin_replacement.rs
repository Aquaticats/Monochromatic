//! One complete set of new pins replacing the old one,
//! and the removal of a pin set a detach asks for.
//! New links are made in a staging directory,
//! which is then exchanged with the final one,
//! and only the exchanged-away links are removed,
//! so a cgroup never has no working pin set in between.

/// The exact hook names every pin set holds.
use crate::bpf::HOOK_NAMES;
/// Standard filesystem and syscall error channel.
use std::io;
/// Owned filesystem paths used for canonical cgroup and pin locations.
use std::path::{
    Path,
    PathBuf,
};
/// Atomic counter makes staging names unique within one process.
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// The staging root this tool creates in bpffs.
use super::root::STAGING_ROOT;

/// Linux `renameat2` flag atomically exchanging two existing paths.
const RENAME_EXCHANGE: libc::c_uint = 2;
/// Process-local sequence for staging directory names.
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Produces non-hidden staging directory because bpffs rejects dot-prefixed names.
pub fn staging_dir_for() -> PathBuf {
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    return Path::new(STAGING_ROOT).join(format!("stage-{}-{sequence}", std::process::id()));
}

/// Removes each expected link pin,
///  then removes directory only when empty.
pub fn remove_exact_pin_dir(pin_dir: &Path) -> io::Result<()> {
    let mut first_error: Option<io::Error> = None;
    for hook_name in HOOK_NAMES {
        let path = pin_dir.join(hook_name);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    continue;
                }
                if first_error.is_none() {
                    first_error = Some(io::Error::new(
                        error.kind(),
                        format!("remove exact pin {}: {error}", path.display()),
                    ));
                }
            }
        }
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    match std::fs::remove_dir(pin_dir) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("remove pin directory {}: {error}", pin_dir.display()),
            ));
        }
    }
}

/// Removes abandoned transactions while global lifecycle lock excludes active attach commands.
pub fn cleanup_stale_staging() -> io::Result<()> {
    for entry_result in std::fs::read_dir(STAGING_ROOT)? {
        let entry = entry_result?;
        remove_exact_pin_dir(&entry.path())?;
    }
    return Ok(());
}

/// Reports the first exchange path error for an embedded NUL byte.
fn first_exchange_path_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(
        io::ErrorKind::InvalidInput,
        "first exchange path contains NUL",
    );
}

/// Reports the second exchange path error for an embedded NUL byte.
fn second_exchange_path_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(
        io::ErrorKind::InvalidInput,
        "second exchange path contains NUL",
    );
}

/// Invokes Linux `renameat2(RENAME_EXCHANGE)` for two existing bpffs siblings.
fn rename_exchange(
    first: &Path,
    second: &Path,
) -> io::Result<()> {
    let first_c = std::ffi::CString::new(first.as_os_str().as_encoded_bytes())
        .map_err(first_exchange_path_contains_nul)?;
    let second_c = std::ffi::CString::new(second.as_os_str().as_encoded_bytes())
        .map_err(second_exchange_path_contains_nul)?;
    // SAFETY: both paths are NUL-terminated and `AT_FDCWD` resolves absolute names directly.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            first_c.as_ptr(),
            libc::AT_FDCWD,
            second_c.as_ptr(),
            RENAME_EXCHANGE,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}

/// Combines primary transaction error with any failure to remove staged links.
pub fn rollback_staging_error(
    staging_dir: &Path,
    error: io::Error,
) -> io::Error {
    if let Err(cleanup_error) = remove_exact_pin_dir(staging_dir) {
        return io::Error::new(
            error.kind(),
            format!("{error}; staging rollback also failed: {cleanup_error}"),
        );
    }
    return error;
}

/// Commits complete staging pins and removes exchanged prior pins afterward.
pub fn commit_staging(
    staging_dir: &Path,
    final_dir: &Path,
) -> io::Result<()> {
    if final_dir.exists() {
        rename_exchange(staging_dir, final_dir)?;
        return remove_exact_pin_dir(staging_dir);
    }
    return std::fs::rename(staging_dir, final_dir);
}

/// Exposes exact removal to sibling unit tests without widening production API.
#[cfg(test)]
pub fn remove_exact_pin_dir_for_test(pin_dir: &Path) -> io::Result<()> {
    return remove_exact_pin_dir(pin_dir);
}

/// Exposes replacement transaction to sibling unit tests without widening production API.
#[cfg(test)]
pub fn commit_staging_for_test(
    staging_dir: &Path,
    final_dir: &Path,
) -> io::Result<()> {
    return commit_staging(staging_dir, final_dir);
}
