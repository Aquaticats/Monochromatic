//! The bpffs check and the roots this tool creates there.
//! A pin is only worth as much as the filesystem behind it:
//! `/sys/fs/bpf` must be a bpffs mount of its own,
//! and the tool then creates one root for its pins
//! and one non-hidden staging root,
//! which bpffs needs because it reserves dots.

/// Standard filesystem and syscall error channel.
use std::io;
/// Unix filesystem device number proves bpffs mount boundaries.
use std::os::unix::fs::MetadataExt;

/// Root mount path that must be a distinct BPF filesystem.
pub const BPF_FS: &str = "/sys/fs/bpf";
/// Tool-owned root containing mirrored cgroup paths and link pins.
pub const PIN_ROOT: &str = "/sys/fs/bpf/wg-quicker-exempt";
/// Dedicated non-hidden directory for incomplete replacement transactions.
pub const STAGING_ROOT: &str = "/sys/fs/bpf/wg-quicker-exempt/staging";
/// Linux `statfs` magic identifying bpffs.
const BPF_FS_MAGIC: libc::c_long = 0xcafe4a11;

/// Reports the filesystem path error for an embedded NUL byte.
fn filesystem_path_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidInput, "filesystem path contains NUL");
}

/// Calls `statfs` for a NUL-free filesystem path.
fn filesystem_type(path: &str) -> io::Result<libc::c_long> {
    let c_path = std::ffi::CString::new(path).map_err(filesystem_path_contains_nul)?;
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
