//! Binding of the control socket so that only the owning user can ever connect to it.

/// What:
///  The file and directory calls the staging steps use,
///  and the Unix socket type itself.
/// Why:
///  The socket is bound inside a private directory,
///  restricted to owner-only access,
///  then renamed to the requested path.
use std::{
    fs::{DirBuilder, File},
    io::ErrorKind,
    os::unix::{
        fs::{DirBuilderExt, PermissionsExt},
        io::AsRawFd,
        net::UnixListener,
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

/// Context names the failed step and the requested socket path in the error `run` reports.
use anyhow::{Context, Result};

/// The log line that makes the address discoverable.
use tracing::{info, warn};

/// Mode of the control socket file:
///  read and write for the owner,
///  nothing for the group or anyone else.
///
/// What:
///      `pub const SOCKET_MODE: u32 = 0o600;`.
/// Why:
///       Connecting to a Unix socket needs write permission on its file.
///  The protocol behind it is unauthenticated and can type,
///  click,
///  and capture the hosted window,
///  so every connection must come from the user who started the session.
pub const SOCKET_MODE: u32 = 0o600;

/// Mode of the staging directory:
///  owner only.
///
/// What:
///      `const STAGING_MODE: u32 = 0o700;`.
/// Why:
///       Nobody else can traverse it,
///  so the socket inside is unreachable until it is renamed.
const STAGING_MODE: u32 = 0o700;

/// Name of the socket file inside the staging directory.
const STAGED_NAME: &str = "control.sock";

/// Counter that keeps staging directory names distinct within one process.
static STAGING_COUNTER: AtomicU32 = AtomicU32::new(0);

/// How many staging names to try before giving up.
const STAGING_ATTEMPTS: u32 = 16;

/// A private directory next to the final socket path,
///  removed when dropped.
///
/// What:
///      `struct Staging { directory: PathBuf, handle: File }`.
///  Holds the directory path and an open descriptor for it.
/// Why:
///       The descriptor lets the socket bind through `/proc/self/fd/<n>/` when the staging path
///           is too long for a Unix socket address.
///  Dropping removes the socket file if it is still there,
///  then the directory,
///  on every error path.
struct Staging {
    /// Absolute or relative path of the staging directory.
    directory: PathBuf,
    /// Open descriptor of the directory.
    handle: File,
}

impl Staging {
    /// Create a fresh owner-only directory in `parent`.
    ///
    /// What:
    ///      `fn create(parent: &Path) -> Result<Staging>`.
    ///  Tries a few names built from the process id and a counter.
    /// Why:
    ///       The directory is created with mode 0700,
    ///  so the umask can only remove permissions from it,
    ///  never add any;
    ///  the explicit chmod afterwards restores 0700 for an unusually strict umask.
    fn create(parent: &Path) -> Result<Staging> {
        for _attempt in 0..STAGING_ATTEMPTS {
            let number = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
            let directory = parent.join(format!(".nws-control-{}-{number}", std::process::id()));
            match DirBuilder::new().mode(STAGING_MODE).create(&directory) {
                Ok(()) => {
                    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(STAGING_MODE))
                        .with_context(|| format!("restricting the staging directory {}", directory.display()))?;
                    let handle = File::open(&directory)
                        .with_context(|| format!("opening the staging directory {}", directory.display()))?;
                    return Ok(Staging { directory, handle })
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| {
                        return format!(
                            "creating the private staging directory {}; the directory for the socket must exist \
                             and be writable by you",
                            directory.display()
                        )
                    })
                }
            }
        }
        anyhow::bail!("no unused staging directory name was found in {}", parent.display())
    }

    /// Path of the socket file inside the directory.
    fn socket_path(&self) -> PathBuf {
        return self.directory.join(STAGED_NAME)
    }

    /// Path through the open descriptor that always fits a Unix socket address.
    fn descriptor_socket_path(&self) -> PathBuf {
        return PathBuf::from(format!("/proc/self/fd/{}/{STAGED_NAME}", self.handle.as_raw_fd()))
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        // A renamed socket leaves nothing behind in the directory.
        if let Err(error) = std::fs::remove_file(self.socket_path())
            && error.kind() != ErrorKind::NotFound
        {
            warn!("removing the staged control socket in {} failed: {error}", self.directory.display());
        }
        if let Err(error) = std::fs::remove_dir(&self.directory) {
            warn!("removing the staging directory {} failed: {error}", self.directory.display());
        }
    }
}

/// Bind (and clean up any stale) the control Unix socket,
///  reachable only by the owning user.
///
/// What:
///      `pub fn bind_listener(path: &Path) -> Result<UnixListener>`.
///  Removes a stale socket file if present,
///  binds a new socket inside a private 0700 directory beside `path`,
///  sets its mode to 0600,
///  and renames it to `path`.
/// Why:
///       `bind` creates the file with mode 0777 minus the umask,
///  and a chmod after `bind` would leave a window where a looser mode is reachable.
///  Changing the process umask would need a libc call and would race every other thread.
///  Here the socket is unreachable until its mode is final,
///  and the rename is atomic,
///  so a client either finds no socket or one only the owner can open.
///
/// Errors:
///       The staging directory cannot be created next to `path`,
///  the bind fails,
///  or the rename fails.
///  Every message names `path`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bindListener(path): Listener {
///   rmSync(path, { force: true });
///   const staging = mkdirSync(join(dirname(path), ".nws-control-N"), { mode: 0o700 });
///   const listener = listen(join(staging, "control.sock"));
///   chmodSync(join(staging, "control.sock"), 0o600);
///   renameSync(join(staging, "control.sock"), path);
///   rmdirSync(staging);
///   return listener;
/// }
/// ```
pub fn bind_listener(path: &Path) -> Result<UnixListener> {
    // What:     `if path.exists() { std::fs::remove_file(path).with_context(...)?; }`.
    //           Remove a pre-existing socket file.
    // Why:      A leftover socket file from a previous run is replaced by the new one.
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("removing stale control socket {}", path.display()))?;
    }

    // What:     The directory that will hold the socket,
    //           the current directory for a bare file name.
    // Why:      The staging directory and the final path must share a file system for the rename.
    let parent = match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory,
        _ => Path::new("."),
    };
    let staging = Staging::create(parent)
        .with_context(|| format!("binding control socket {}", path.display()))?;

    // What:     Bind at the staged path,
    //           and again through the directory descriptor when the staged path is too long.
    // Why:      The staged path is longer than `path`,
    //           so a path that just fits a socket address needs the descriptor form.
    let listener = match UnixListener::bind(staging.socket_path()) {
        Err(error) if error.kind() == ErrorKind::InvalidInput => UnixListener::bind(staging.descriptor_socket_path()),
        outcome => outcome,
    }
    .with_context(|| format!("binding control socket {}", path.display()))?;

    // What:     Set the socket file to the owner-only mode.
    // Why:      The directory keeps the socket unreachable until this and the rename are done.
    std::fs::set_permissions(staging.socket_path(), std::fs::Permissions::from_mode(SOCKET_MODE))
        .with_context(|| format!("restricting control socket {} to its owner", path.display()))?;

    // What:     Move the socket to the requested path.
    // Why:      The rename is atomic and keeps the mode and the listening state.
    std::fs::rename(staging.socket_path(), path)
        .with_context(|| format!("moving the bound control socket to {}", path.display()))?;

    // What:     `info!(...)`. Log where the control socket lives.
    // Why:      Make the address discoverable.
    info!("control socket listening at {}", path.display());

    // What:     `Ok(listener)`. Return the bound listener.
    // Why:      Hand it to the control thread.
    return Ok(listener)
}

/// Unit tests for the socket modes and the staging directory.
#[cfg(test)]
#[path = "control_socket_tests.rs"]
mod tests;
