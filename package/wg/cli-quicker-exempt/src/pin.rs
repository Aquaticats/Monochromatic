//! Owns bpffs validation,
//!  collision-free cgroup pin paths,
//!  and atomic link replacement.

/// Raw BPF attachment function and exact hook pin names.
use crate::bpf::{attach_marker, HOOK_NAMES};
/// Debug-only typed pin failure constructor for deterministic fallback tests.
#[cfg(debug_assertions)]
use crate::bpf_error::pin_object_invalid;
/// Standard filesystem and syscall error channel.
use std::io;
/// Unix metadata device identity and raw descriptor access.
use std::os::fd::AsRawFd;
/// Raw Unix path bytes used by injective hexadecimal encoding.
use std::os::unix::ffi::OsStrExt;
/// Unix filesystem device number proves bpffs mount boundaries.
use std::os::unix::fs::MetadataExt;
/// Owned filesystem paths used for canonical cgroup and pin locations.
use std::path::{Component, Path, PathBuf};
/// Atomic counter makes staging names unique within one process.
use std::sync::atomic::{AtomicU64, Ordering};

/// Root mount path that must be a distinct BPF filesystem.
pub const BPF_FS: &str = "/sys/fs/bpf";
/// Tool-owned root containing mirrored cgroup paths and link pins.
pub const PIN_ROOT: &str = "/sys/fs/bpf/wg-quicker-exempt";
/// Dedicated non-hidden directory for incomplete replacement transactions.
const STAGING_ROOT: &str = "/sys/fs/bpf/wg-quicker-exempt/staging";
/// Linux `statfs` magic identifying bpffs.
const BPF_FS_MAGIC: libc::c_long = 0xcafe4a11;
/// Linux `renameat2` flag atomically exchanging two existing paths.
const RENAME_EXCHANGE: libc::c_uint = 2;
/// Process-local sequence for staging directory names.
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);
/// Hex digits avoid dots,
///  which bpffs reserves for future extensions.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
/// Source bytes per encoded component keep every bpffs name within `NAME_MAX`.
const PIN_KEY_CHUNK_BYTES: usize = 100;
/// Debug-only environment flag simulating affected kernel's typed object-pin failure.
#[cfg(debug_assertions)]
const FORCE_PIN_INVALID_ENV: &str = "WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID";

/// Keeps the global lifecycle lock file open until attach or detach command ends.
pub struct LifecycleLock {
    /// Open file owns the descriptor carrying kernel advisory lock.
    _file: std::fs::File,
}

/// Acquires process-crash-safe global lock for pin directory mutations.
pub fn acquire_lifecycle_lock() -> io::Result<LifecycleLock> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open("/run/wg-quicker-exempt.lock")?;
    // SAFETY: `file` owns a valid descriptor and `flock` changes only its kernel lock state.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(LifecycleLock { _file: file });
}

/// Calls `statfs` for a NUL-free filesystem path.
fn filesystem_type(path: &str) -> io::Result<libc::c_long> {
    let c_path = std::ffi::CString::new(path).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "filesystem path contains NUL");
    })?;
    // SAFETY: zeroed `statfs` is output-only storage initialized by successful syscall.
    let mut info: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: pointers reference a NUL-terminated path and writable output structure.
    let result = unsafe { libc::statfs(c_path.as_ptr(), &mut info) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(info.f_type);
}

/// Verifies `/sys/fs/bpf` is a bpffs mount boundary,
///  then creates tool root.
pub fn ensure_pin_root() -> io::Result<()> {
    let filesystem_magic = filesystem_type(BPF_FS)?;
    if filesystem_magic != BPF_FS_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{BPF_FS} is not bpffs: magic={filesystem_magic:#x}"),
        ));
    }
    let mount_metadata = std::fs::metadata(BPF_FS)?;
    let parent_metadata = std::fs::metadata("/sys/fs")?;
    if mount_metadata.dev() == parent_metadata.dev() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{BPF_FS} is not a distinct mount point"),
        ));
    }
    std::fs::create_dir_all(PIN_ROOT)?;
    return std::fs::create_dir_all(STAGING_ROOT);
}

/// Encodes canonical path bytes injectively into bpffs-safe hexadecimal components.
fn encoded_pin_key(canonical: &Path) -> io::Result<PathBuf> {
    if !canonical.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cgroup path must be absolute",
        ));
    }
    let encoded: String = canonical
        .as_os_str()
        .as_bytes()
        .iter()
        .flat_map(|byte| {
            return [
                HEX_DIGITS[(byte >> 4) as usize] as char,
                HEX_DIGITS[(byte & 0x0f) as usize] as char,
            ];
        })
        .collect();
    let mut key = PathBuf::new();
    for chunk in encoded.as_bytes().chunks(PIN_KEY_CHUNK_BYTES * 2) {
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

/// Produces non-hidden staging directory because bpffs rejects dot-prefixed names.
fn staging_dir_for() -> PathBuf {
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    return Path::new(STAGING_ROOT).join(format!(
        "stage-{}-{sequence}",
        std::process::id()
    ));
}

/// Removes each expected link pin,
///  then removes directory only when empty.
fn remove_exact_pin_dir(pin_dir: &Path) -> io::Result<()> {
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
fn cleanup_stale_staging() -> io::Result<()> {
    for entry_result in std::fs::read_dir(STAGING_ROOT)? {
        let entry = entry_result?;
        remove_exact_pin_dir(&entry.path())?;
    }
    return Ok(());
}

/// Invokes Linux `renameat2(RENAME_EXCHANGE)` for two existing bpffs siblings.
fn rename_exchange(first: &Path, second: &Path) -> io::Result<()> {
    let first_c = std::ffi::CString::new(first.as_os_str().as_encoded_bytes()).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "first exchange path contains NUL");
    })?;
    let second_c = std::ffi::CString::new(second.as_os_str().as_encoded_bytes()).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "second exchange path contains NUL");
    })?;
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
fn rollback_staging_error(staging_dir: &Path, error: io::Error) -> io::Error {
    if let Err(cleanup_error) = remove_exact_pin_dir(staging_dir) {
        return io::Error::new(
            error.kind(),
            format!("{error}; staging rollback also failed: {cleanup_error}"),
        );
    }
    return error;
}

/// Commits complete staging pins and removes exchanged prior pins afterward.
fn commit_staging(staging_dir: &Path, final_dir: &Path) -> io::Result<()> {
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
pub fn commit_staging_for_test(staging_dir: &Path, final_dir: &Path) -> io::Result<()> {
    return commit_staging(staging_dir, final_dir);
}

/// Attaches all hooks in staging and commits them without dropping prior working pins.
pub fn attach_cgroup(mark: u32, cgroup_dir: &Path) -> io::Result<usize> {
    ensure_pin_root()?;
    let final_dir = pin_dir_for(cgroup_dir)?;
    let parent = final_dir.parent().ok_or_else(|| {
        return io::Error::new(io::ErrorKind::InvalidInput, "pin path lacks parent");
    })?;
    std::fs::create_dir_all(parent)?;
    cleanup_stale_staging()?;
    let staging_dir = staging_dir_for();
    let staging_device = std::fs::metadata(STAGING_ROOT)?.dev();
    let final_device = std::fs::metadata(parent)?.dev();
    if staging_device != final_device {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "staging and final pin directories are on different filesystems",
        ));
    }
    if staging_dir.exists() {
        remove_exact_pin_dir(&staging_dir)?;
    }
    std::fs::create_dir(&staging_dir)?;
    let cgroup = std::fs::File::open(cgroup_dir)?;
    let staging_text = staging_dir.to_str().ok_or_else(|| {
        return io::Error::new(io::ErrorKind::InvalidInput, "staging path is not UTF-8");
    })?;
    #[cfg(debug_assertions)]
    if std::env::var_os(FORCE_PIN_INVALID_ENV).is_some() {
        let source = io::Error::from_raw_os_error(libc::EINVAL);
        let error = pin_object_invalid(staging_text.to_string(), source);
        return Err(rollback_staging_error(&staging_dir, error));
    }
    let pinned = match attach_marker(cgroup.as_raw_fd(), mark, staging_text) {
        Ok(paths) => paths,
        Err(error) => return Err(rollback_staging_error(&staging_dir, error)),
    };
    if let Err(error) = commit_staging(&staging_dir, &final_dir) {
        let commit_error = io::Error::new(
            error.kind(),
            format!("atomic bpffs link commit failed: {error}"),
        );
        return Err(rollback_staging_error(&staging_dir, commit_error));
    }
    return Ok(pinned.len());
}

/// Detaches only four exact persisted links for one canonical cgroup.
pub fn detach_cgroup(cgroup_dir: &Path) -> io::Result<()> {
    ensure_pin_root()?;
    let final_dir = pin_dir_for(cgroup_dir)?;
    return remove_exact_pin_dir(&final_dir);
}
