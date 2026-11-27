//! A cgroup's identity,
//! and the bpffs path its pins live at.
//! The identity is the canonical path while the cgroup exists
//! and the validated lexical path once it has gone,
//! so a detach still finds the pins of a cgroup that was removed.
//! The path is encoded injectively into chunks of lowercase hexadecimal,
//! because bpffs reserves dots and one name is capped at `NAME_MAX`.

/// Lowercase hexadecimal spelling shared with the watcher state names.
use crate::bytes_as_lowercase_hex::{
    DIGITS_PER_BYTE,
    bytes_as_lowercase_hex,
};
/// Standard filesystem and syscall error channel.
use std::io;
/// Raw Unix path bytes used by injective hexadecimal encoding.
use std::os::unix::ffi::OsStrExt;
/// Owned filesystem paths used for canonical cgroup and pin locations.
use std::path::{
    Component,
    Path,
    PathBuf,
};

/// The tool's own root,
/// below which each cgroup's pins sit.
use super::root::PIN_ROOT;

/// Source bytes per encoded component keep every bpffs name within `NAME_MAX`.
const PIN_KEY_CHUNK_BYTES: usize = 100;

/// Encodes canonical path bytes injectively into bpffs-safe hexadecimal components.
fn encoded_pin_key(canonical: &Path) -> io::Result<PathBuf> {
    if !canonical.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cgroup path must be absolute",
        ));
    }
    let encoded: String = bytes_as_lowercase_hex(canonical.as_os_str().as_bytes());
    let mut key = PathBuf::new();
    for chunk in encoded
        .as_bytes()
        .chunks(PIN_KEY_CHUNK_BYTES * DIGITS_PER_BYTE)
    {
        let component = std::str::from_utf8(chunk).map_err(io::Error::other)?;
        key.push(component);
    }
    return Ok(key);
}

/// Validates already-normalized absolute path when target cgroup has disappeared.
fn validate_lexical_cgroup_path(cgroup_dir: &Path) -> io::Result<PathBuf> {
    if !cgroup_dir.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cgroup path must be absolute",
        ));
    }
    for component in cgroup_dir.components() {
        if component == Component::ParentDir || component == Component::CurDir {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing cgroup path must already be lexically normalized",
            ));
        }
    }
    return Ok(cgroup_dir.to_path_buf());
}

/// Returns canonical existing cgroup or validated lexical identity after removal.
pub fn cgroup_identity_path(cgroup_dir: &Path) -> io::Result<PathBuf> {
    match std::fs::canonicalize(cgroup_dir) {
        Ok(canonical) => return Ok(canonical),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return validate_lexical_cgroup_path(cgroup_dir);
        }
        Err(error) => return Err(error),
    }
}

/// Returns collision-free key shared by bpffs pins and runtime keeper state.
pub fn cgroup_key(cgroup_dir: &Path) -> io::Result<PathBuf> {
    return encoded_pin_key(&cgroup_identity_path(cgroup_dir)?);
}

/// Maps cgroup identity injectively without bpffs-reserved dots.
pub fn pin_dir_for(cgroup_dir: &Path) -> io::Result<PathBuf> {
    return Ok(Path::new(PIN_ROOT).join(cgroup_key(cgroup_dir)?));
}
