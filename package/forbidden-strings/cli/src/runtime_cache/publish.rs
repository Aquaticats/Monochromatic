//! Reads and atomically publishes sensitive runtime cache artifacts.
//!
//! Readers load complete bytes and close the handle before validation,
//!  so Windows
//! replacement is not blocked by a lingering reader.
//!  Writers flush a private
//! same-directory temporary file before `rename`,
//!  and remove it on failure.

/// Imports filesystem primitives for complete reads and atomic publication.
use std::fs::{self, File, OpenOptions};
/// Imports byte writer used before durable file flush.
use std::io::Write;
/// Imports artifact paths.
use std::path::{Path, PathBuf};
/// Imports process-local collision-free counter for temporary names.
use std::sync::atomic::{AtomicU64, Ordering};

/// Imports resolved protected hierarchy.
use super::path::CacheLocation;

/// Process-local temporary-file sequence.
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Artifact read failure category safe for warning mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArtifactReadError {
    /// Artifact path does not exist.
    Missing,
    /// Metadata or complete-byte read failed for another reason.
    Unreadable,
}

/// Atomic publication failure with no path or operating-system text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PublishError;

/// Reads complete artifact only after enforcing configured size ceiling.
pub(super) fn read_artifact(
    path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, ArtifactReadError> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            return ArtifactReadError::Missing;
        }
        return ArtifactReadError::Unreadable;
    })?;
    if !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(ArtifactReadError::Unreadable);
    }
    return fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            return ArtifactReadError::Missing;
        }
        return ArtifactReadError::Unreadable;
    })
}

/// Applies owner-only mode to one application-owned directory on Unix.
#[cfg(unix)]
fn make_directory_private(path: &Path) -> Result<(), PublishError> {
    /// Imports Unix mode construction for owner-only directory permission.
    use std::os::unix::fs::PermissionsExt;
    return fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| return PublishError)
}

/// Keeps platform-native directory ACL behavior on non-Unix systems.
#[cfg(not(unix))]
fn make_directory_private(_path: &Path) -> Result<(), PublishError> {
    return Ok(())
}

/// Applies owner-only mode to one artifact file on Unix.
#[cfg(unix)]
fn make_file_private(path: &Path) -> Result<(), PublishError> {
    /// Imports Unix mode construction for owner-only artifact permission.
    use std::os::unix::fs::PermissionsExt;
    return fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| return PublishError)
}

/// Keeps platform-native file ACL behavior on non-Unix systems.
#[cfg(not(unix))]
fn make_file_private(_path: &Path) -> Result<(), PublishError> {
    return Ok(())
}

/// Creates every application-owned hierarchy level and enforces private mode.
fn prepare_directories(location: &CacheLocation) -> Result<(), PublishError> {
    for directory in &location.protected_directories {
        fs::create_dir_all(directory).map_err(|_| return PublishError)?;
        make_directory_private(directory)?;
    }
    return Ok(())
}

/// Builds unique same-directory temporary path without random dependency.
fn temporary_path(artifact_path: &Path) -> Result<PathBuf, PublishError> {
    let parent = artifact_path.parent().ok_or(PublishError)?;
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    return Ok(parent.join(format!(
        ".rules.bin.{}.{}.tmp",
        std::process::id(),
        sequence,
    )))
}

/// Writes,
///  flushes,
///  and atomically renames one complete artifact.
pub(super) fn publish_artifact(
    location: &CacheLocation,
    bytes: &[u8],
) -> Result<(), PublishError> {
    prepare_directories(location)?;
    let temporary = temporary_path(&location.artifact_path)?;
    let publication = publish_from_temporary(&temporary, &location.artifact_path, bytes);
    if publication.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    return publication
}

/// Owns temporary-file lifecycle up to final atomic replacement.
fn publish_from_temporary(
    temporary: &Path,
    artifact_path: &Path,
    bytes: &[u8],
) -> Result<(), PublishError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|_| return PublishError)?;
    make_file_private(temporary)?;
    file.write_all(bytes).map_err(|_| return PublishError)?;
    file.sync_all().map_err(|_| return PublishError)?;
    drop(file);
    fs::rename(temporary, artifact_path).map_err(|_| return PublishError)?;
    sync_parent_best_effort(artifact_path);
    return Ok(())
}

/// Flushes renamed directory entry where platform permits directory handles.
fn sync_parent_best_effort(artifact_path: &Path) {
    let Some(parent) = artifact_path.parent() else {
        return;
    };
    let Ok(directory) = File::open(parent) else {
        return;
    };
    let _ = directory.sync_all();
}

/// Registers complete-read,
///  private-mode,
///  and replacement tests.
#[cfg(test)]
#[path = "publish_tests.rs"]
mod tests;
