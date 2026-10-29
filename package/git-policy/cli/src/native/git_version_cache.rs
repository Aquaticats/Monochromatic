//! What:
//!  A private, per-user cache of the text `git version` prints,
//!  keyed by the identity of the Git executable that printed it.
//! Why:
//!  Every commit starts `git version` to learn whether Git is new enough.
//!  The text is fixed by the executable,
//!  so while the executable is byte-for-byte the same file the answer is the same.
//!  The identity is the canonical path,
//!  size,
//!  modification time,
//!  change time,
//!  device and inode:
//!  an upgrade replaces the file (new inode) or rewrites it (new size, times),
//!  so a replaced Git is noticed on the next start and a stale entry is never trusted.
//!  The cache is used only for a real compiled executable named `git`:
//!  a script,
//!  or a shim such as a version manager's `git` link to a dispatcher,
//!  picks another Git by directory or environment,
//!  so its own file proves nothing about the version it will run.
//!  The cache lives in the user's cache directory because the version check runs before any repository is known.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = await binaryIdentity(gitPath); // undefined: do not cache
//! const text = identity && (await readCache(dir, identity)) ?? (await runGitVersion());
//! ```

/// Debug lines for cache problems,
///  which never fail a command.
use super::diagnostic_log::debug;
/// The environment lookup shared by the wrapper.
use super::child_environment::environment_value;
/// OS strings and paths.
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The first line of every cache file;
///  a file in another layout is ignored.
const FILE_TAG: &str = "cli-git git-version cache 1\n";

/// The most `git version` bytes the cache keeps;
///  a longer answer is not cached.
const TEXT_LIMIT: usize = 1024;

/// The tag of this module's diagnostics.
const TAG: &str = "git-version-cache";

/// What:
///  The identity of one Git executable,
///  as the text that goes into a cache file and the canonical path that names the file.
/// Why:
///   The text is compared whole against what the cache file recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BinaryIdentity = { path: string; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinaryIdentity {
    /// The canonical path of the executable.
    pub path: String,
    /// The path, size, times, device and inode joined into one line.
    pub text: String,
}

/// What:
///  The identity of the executable at `real_git`,
///  or `None` when its answer cannot be cached safely.
/// Why:
///   Only a compiled ELF file named `git` is identified:
///  its version is a property of its bytes.
///  A path or a field that would need escaping in the one-line text is refused too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const s = await stat(realpath(gitPath)); return `${path}\t${s.size}\t${s.mtimeNs}\t${s.ctimeNs}\t${s.dev}\t${s.ino}`;
/// ```
#[cfg(unix)]
pub fn binary_identity(real_git: &Path) -> Option<BinaryIdentity> {
    // `Read` gives `read_exact`; `MetadataExt` gives the times, device and inode.
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let canonical: PathBuf = std::fs::canonicalize(real_git).ok()?;
    if canonical.file_name()?.to_str()? != "git" {
        return None;
    }
    let path: &str = canonical.to_str()?;
    if path.contains('\n') || path.contains('\t') {
        return None;
    }
    // One handle supplies both the fields and the first bytes, so they describe the same file.
    let mut file: std::fs::File = std::fs::File::open(&canonical).ok()?;
    let metadata: std::fs::Metadata = file.metadata().ok()?;
    if !metadata.is_file() {
        return None;
    }
    let mut magic: [u8; 4] = [0; 4];
    file.read_exact(&mut magic).ok()?;
    if magic != *b"\x7fELF" {
        return None;
    }
    let text: String = format!(
        "{path}\t{}\t{}.{}\t{}.{}\t{}\t{}",
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
        metadata.dev(),
        metadata.ino()
    );
    return Some(BinaryIdentity { path: String::from(path), text });
}

/// What:
///  No identity on a platform without Unix file metadata.
/// Why:
///   The cache is not offered there,
///  so every start asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return undefined;
/// ```
#[cfg(not(unix))]
pub fn binary_identity(real_git: &Path) -> Option<BinaryIdentity> {
    let _ = real_git;
    return None;
}

/// What:
///  The directory that holds the cache files:
///  `cli-git` under `XDG_CACHE_HOME`,
///  or under `.cache` in the home directory.
/// Why:
///   The injected environment decides the location,
///  never a fixed path; a relative `XDG_CACHE_HOME` is ignored as the XDG rules say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = isAbsolute(env.XDG_CACHE_HOME ?? '') ? env.XDG_CACHE_HOME : env.HOME && join(env.HOME, '.cache');
/// ```
pub fn cache_directory(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    if let Some(cache_home) = environment_value(environment, "XDG_CACHE_HOME") {
        let cache_path: PathBuf = PathBuf::from(cache_home);
        if cache_path.is_absolute() {
            return Some(cache_path.join("cli-git"));
        }
    }
    let home: OsString = environment_value(environment, "HOME")?;
    if home.is_empty() {
        return None;
    }
    return Some(PathBuf::from(home).join(".cache").join("cli-git"));
}

/// What:
///  The cache file of one executable inside `directory`.
/// Why:
///   One file per canonical path keeps concurrent commits of different Gits apart;
///  the name is a hash, and the file records the full identity to catch a collision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(directory, `git-version-${hash(identity.path)}`)
/// ```
fn cache_file(directory: &Path, identity: &BinaryIdentity) -> PathBuf {
    // The hashing traits give `.hash` and `.finish`.
    use std::hash::{Hash, Hasher};
    let mut hasher: std::collections::hash_map::DefaultHasher = std::collections::hash_map::DefaultHasher::new();
    identity.path.hash(&mut hasher);
    return directory.join(format!("git-version-{:016x}", hasher.finish()));
}

/// What:
///  Whether the cache file at `path` is a regular file of this user that no one else can write.
/// Why:
///   A file another account could have planted or edited is not trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const s = await lstat(path); return s.isFile() && s.uid === process.geteuid() && (s.mode & 0o077) === 0;
/// ```
#[cfg(unix)]
fn is_private_file_of_this_user(path: &Path) -> bool {
    // `MetadataExt` gives the owner and the mode bits.
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    // SAFETY: `geteuid` has no preconditions.
    let owner: u32 = unsafe { libc::geteuid() };
    return metadata.is_file() && metadata.uid() == owner && metadata.mode() & 0o077 == 0;
}

/// What:
///  No file is trusted on a platform without Unix ownership.
/// Why:
///   The cache is not offered there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return false;
/// ```
#[cfg(not(unix))]
fn is_private_file_of_this_user(path: &Path) -> bool {
    let _ = path;
    return false;
}

/// What:
///  The `git version` text recorded for exactly this identity,
///  or `None` for a missing,
///  foreign,
///  unreadable or stale entry.
/// Why:
///   Anything but an exact match of the whole identity line is stale,
///  and the caller then asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = await readFile(cacheFile(dir, identity)); return file.startsWith(TAG + identity.text + '\n') ? file.slice(...) : undefined;
/// ```
pub fn read_version_text(directory: &Path, identity: &BinaryIdentity) -> Option<Vec<u8>> {
    let path: PathBuf = cache_file(directory, identity);
    if !is_private_file_of_this_user(path.as_path()) {
        return None;
    }
    let bytes: Vec<u8> = super::private_storage::read_regular_file(path.as_path(), 4096).ok()?;
    let rest: &[u8] = bytes.strip_prefix(FILE_TAG.as_bytes())?;
    let rest_after_identity: &[u8] = rest.strip_prefix(identity.text.as_bytes())?;
    let text: &[u8] = rest_after_identity.strip_prefix(b"\n")?;
    return Some(text.to_vec());
}

/// What:
///  Record `text` for this identity,
///  replacing any earlier entry,
///  and report nothing when recording is impossible.
/// Why:
///   The write goes to a private temporary file and is renamed over the entry,
///  so a reader sees the old entry or the new one whole;
///  it is not synced because a lost entry only costs one `git version` start.
///  A failure to record is logged and never fails the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dir, { recursive: true, mode: 0o700 }); await writeFile(tmp, bytes, { mode: 0o600 }); await rename(tmp, file);
/// ```
pub fn store_version_text(directory: &Path, identity: &BinaryIdentity, text: &[u8]) {
    if text.len() > TEXT_LIMIT {
        return;
    }
    if let Some(parent) = directory.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        debug(TAG, format!("could not create {}: {error}", parent.display()).as_str());
        return;
    }
    if let Err(error) = super::private_storage::create_private_directory(directory)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        debug(TAG, format!("could not create {}: {error}", directory.display()).as_str());
        return;
    }
    let unique: String = match super::random_id::random_uuid() {
        Ok(id) => id,
        Err(error) => {
            debug(TAG, format!("no random name for the cache file: {}", error.0).as_str());
            return;
        }
    };
    let temporary: PathBuf = directory.join(format!(".tmp-{unique}"));
    let mut bytes: Vec<u8> = Vec::with_capacity(FILE_TAG.len() + identity.text.len() + 1 + text.len());
    bytes.extend_from_slice(FILE_TAG.as_bytes());
    bytes.extend_from_slice(identity.text.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(text);
    if let Err(error) = super::private_storage::write_exclusive_scratch(temporary.as_path(), bytes.as_slice()) {
        debug(TAG, format!("could not write {}: {error}", temporary.display()).as_str());
        return;
    }
    if let Err(error) = std::fs::rename(&temporary, cache_file(directory, identity)) {
        debug(TAG, format!("could not publish the cache file: {error}").as_str());
        if let Err(removal) = std::fs::remove_file(&temporary) {
            debug(TAG, format!("could not remove {}: {removal}", temporary.display()).as_str());
        }
    }
}

/// Cache controls stay out of the release executable.
#[cfg(test)]
#[path = "git_version_cache_tests.rs"]
mod tests;
