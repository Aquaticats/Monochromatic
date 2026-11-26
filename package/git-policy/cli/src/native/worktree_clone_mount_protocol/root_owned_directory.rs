//! What:
//!  The root-owned directory a `/run` record lives in:
//!  made when it is absent,
//!  given its designed mode,
//!  and proved owned by root through its own descriptor before anything below
//!  it is read or written.
//! Why:
//!  The results and readiness directories are `/run` entries,
//!  and a fresh boot has neither,
//!  so the writer makes its own directory rather than refusing every publish
//!  until someone else does
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Where the verdict is enforced,
//! and where the record lives":
//!  a mode process creates the root-owned `0755` directory itself because a
//!  sweep can reach its first attach before anything else wrote).
//!  The open follows no link and the owner is checked before the mode is
//!  changed,
//!  so a link or another account's folder is refused without being written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const directory = makeAndProveDirectory('/run/x', 0, 0o755); // throws when another account owns it
//! ```

use std::ffi::CString;
use std::os::fd::{
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::{
    MetadataExt,
    PermissionsExt,
};
use std::path::Path;

/// What:
///  Why a root-owned directory could not be made or proved.
/// Why:
///   Each caller names the directory in its own refusal,
///  so this type carries only the cause and the values it compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootDirectoryRefusal = { kind: 'open' | 'owner' | 'mode'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootDirectoryRefusal {
    /// The directory could not be made,
    /// opened or examined.
    Open {
        /// What the call answered.
        reason: String,
    },
    /// The directory is not owned by the expected account.
    Owner {
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The directory does not carry its designed mode,
    /// even after the writer set it.
    Mode {
        /// The mode found.
        found: u32,
        /// The mode the route requires.
        wanted: u32,
    },
}

/// What:
///  Make `path` when it is absent,
///  set it to `mode`,
///  and prove it is a directory owned by `owner` at that mode.
/// Why:
///  The descriptor this returns is the one every later call is relative to,
///  so a path replaced after the proof cannot steer a read or a write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = makeAndProveDirectory(path, owner, mode);
/// ```
pub(super) fn make_and_prove(
    path: &Path,
    owner: u32,
    mode: u32,
) -> Result<std::fs::File, RootDirectoryRefusal> {
    if let Err(error) = std::fs::create_dir_all(path) {
        return Err(RootDirectoryRefusal::Open {
            reason: error.to_string(),
        });
    }
    let text: CString = CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_nul| {
        return RootDirectoryRefusal::Open {
            reason: String::from("the path holds a NUL byte"),
        };
    })?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call.
    let descriptor: libc::c_int = unsafe {
        libc::open(
            text.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        return Err(RootDirectoryRefusal::Open {
            reason: std::io::Error::last_os_error().to_string(),
        });
    }
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    let directory: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
    let facts: std::fs::Metadata = directory.metadata().map_err(|error| {
        return RootDirectoryRefusal::Open {
            reason: error.to_string(),
        };
    })?;
    if facts.uid() != owner {
        return Err(RootDirectoryRefusal::Owner {
            found: facts.uid(),
            wanted: owner,
        });
    }
    directory
        .set_permissions(std::fs::Permissions::from_mode(mode))
        .map_err(|error| {
            return RootDirectoryRefusal::Open {
                reason: error.to_string(),
            };
        })?;
    let after: std::fs::Metadata = directory.metadata().map_err(|error| {
        return RootDirectoryRefusal::Open {
            reason: error.to_string(),
        };
    })?;
    let permission_bits: u32 = after.mode() & 0o7777;
    if permission_bits != mode {
        return Err(RootDirectoryRefusal::Mode {
            found: permission_bits,
            wanted: mode,
        });
    }
    return Ok(directory);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "root_owned_directory_tests.rs"]
mod tests;
