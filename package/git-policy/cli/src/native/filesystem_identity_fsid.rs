//! What:
//!  The `f_fsid` of the filesystem holding a path,
//!  asked of the kernel in-process and spelled exactly as GNU `stat --file-system --format=%i` spells it.
//! Why:
//!  A filesystem without a UUID (overlayfs, tmpfs) is identified by this text,
//!  and the identity of a real `index.lock` is resolved again when the lock is released.
//!  Starting `stat` for it made that release depend on the host granting a process start,
//!  so a host that refused every start left the lock behind and a plain `git add` stuck on it;
//!  the same call in-process needs no process, and also saves the `stat` starts of every commit on such a filesystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stdout } = await nanoSpawn('stat', ['--file-system', '--format=%i', path]);
//! ```

/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The text GNU `stat --file-system --format=%i` prints for a path,
///  or nothing when the kernel refuses the call.
/// Why:
///   coreutils joins the two 32-bit words of `f_fsid` into one number with the first word on top
///  and prints it in lowercase hexadecimal without leading zeros;
///  the recorded identity must stay byte-identical to that spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await nanoSpawn('stat', ['--file-system', '--format=%i', path])).stdout.trim()
/// ```
pub fn f_fsid_text(path: &Path) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;
    let c_path: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `statfs` is plain data that the call fills in, so all-zero bytes are a valid value.
    let mut buffer: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a NUL-terminated string that outlives the call, and `buffer` is a valid
    // `statfs` the kernel writes at most once.
    let status: libc::c_int = unsafe { libc::statfs(c_path.as_ptr(), &mut buffer) };
    if status != 0 {
        return None;
    }
    // SAFETY: `fsid_t` is two 32-bit integers, the same size and alignment as `[u32; 2]`, and every bit pattern
    // of either is valid.
    let words: [u32; 2] = unsafe { std::mem::transmute_copy::<libc::fsid_t, [u32; 2]>(&buffer.f_fsid) };
    return Some(format!("{:x}", (u64::from(words[0]) << 32) | u64::from(words[1])));
}

/// Controls for the spelling stay out of the release executable.
#[cfg(test)]
#[path = "filesystem_identity_fsid_tests.rs"]
mod tests;
