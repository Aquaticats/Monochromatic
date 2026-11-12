//! What:
//!  Read the real Git that the wrapper's commit service was started with,
//! from the one note the wrapper writes beside its sockets.
//! Why:
//!  Nothing the wrapper installs records the real Git it shadows.
//! The file that names a real Git is the note the wrapper writes when it starts a server:
//!  `<runtime>/monochromatic-git-policy-cli/real-git`,
//! whose single line is the absolute Git the service runs.
//! Until this release the record was the service unit's `ExecStart` line,
//! which held `--git <path>`;
//! the unit is gone,
//! and the note holds the path itself.
//! When no real Git follows the wrapper on `PATH`,
//! that path is the only
//!  recorded one,
//! so the resolver (`real_git.rs`) falls back to it,
//! after the same checks a `PATH` candidate passes.
//!  This is the coordinator's pick under the owner's delegation,
//! open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`,
//! section "Falling back to the Git the commit service was set up
//!  with").
//! The record is one per account's runtime directory, not one per worktree,
//! because it is read before the worktree is known:
//! the real Git is what the wrapper resolves first,
//! and the worktree's own Git directory is asked with it.
//! It lives in the runtime directory,
//! so it is gone after the account's last logout,
//! and no record then
//!  means no fallback.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recordedGit = readFileSync(join(runtimeDir, 'monochromatic-git-policy-cli', 'real-git'), 'utf8').split('\n')[0] || undefined;
//! ```

use crate::real_git_candidate::MAX_SCRIPT_INSPECTION_BYTES;
use crate::service_instance::SOCKET_DIRECTORY;
/// `Read` supplies `read_to_end` and `take`,
/// the bounded read below.
use std::io::Read;
/// `Path` is a borrowed filesystem path;
///  `PathBuf` is the owned sibling a function returns.
use std::path::{Path, PathBuf};

/// The name of the note that records the real Git the service runs.
pub const RECORD_NAME: &str = "real-git";

/// What:
///  The note that records the real Git,
/// below a runtime directory.
/// Why:
///  The wrapper writes it there and nowhere else is searched:
///  a record found by looking through other places would be a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordFile(runtimeDir: string): string { return join(runtimeDir, 'monochromatic-git-policy-cli', 'real-git'); }
/// ```
pub fn record_file(runtime_dir: &Path) -> PathBuf {
    return runtime_dir.join(SOCKET_DIRECTORY).join(RECORD_NAME);
}

/// What:
///  The absolute path a record's text names,
/// or nothing when it names none.
/// Why:
///  The note holds one line and that line is the path,
/// so a record with more lines,
/// or with a line that is not an absolute path,
/// names nothing this wrapper can trust.
///  A relative path is no record,
/// because it would mean a different file for each directory the wrapper runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordedGitPath(text: string): string | undefined { const line = text.split('\n')[0].trim(); return isAbsolute(line) ? line : undefined; }
/// ```
pub fn recorded_git_path(text: &str) -> Option<PathBuf> {
    let line: &str = text.lines().next()?.trim();
    let path: PathBuf = PathBuf::from(line);
    return if path.is_absolute() { Some(path) } else { None };
}

/// What:
///  The Git a record names,
/// read with the bound a launcher record is read with.
/// Why:
///  The note is a path and a newline,
/// so a file past the bound is not the wrapper's record and names nothing.
///  An unreadable or missing file names nothing either,
/// and the caller keeps its stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRecordedGit(recordFile: string): string | undefined;
/// ```
pub fn read_recorded_git(record: &Path) -> Option<PathBuf> {
    let file: std::fs::File = std::fs::File::open(record).ok()?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    // One byte more than the limit is read, so that a file over the limit is told from one exactly at it.
    let bound: u64 = u64::try_from(MAX_SCRIPT_INSPECTION_BYTES).unwrap_or(u64::MAX).saturating_add(1);
    let read: usize = file.take(bound).read_to_end(&mut bytes).ok()?;
    if read > MAX_SCRIPT_INSPECTION_BYTES {
        return None;
    }
    // A file that is not text is no record: a path with a replacement character names another place.
    let text: &str = std::str::from_utf8(bytes.as_slice()).ok()?;
    return recorded_git_path(text);
}

/// What:
///  Records the real Git beside the sockets,
/// so a later command that cannot find one on `PATH` can fall back to it.
/// Why:
///  The service runs this Git,
/// so it is the one path that is certainly right for this account.
/// The write is a temporary file and a rename,
/// so a reader never sees half a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAtomic(recordFile(runtimeDir), `${git}\n`);
/// ```
pub fn remember(runtime_dir: &Path, git: &Path) -> std::io::Result<()> {
    let directory: PathBuf = runtime_dir.join(SOCKET_DIRECTORY);
    std::fs::create_dir_all(&directory)?;
    let record: PathBuf = directory.join(RECORD_NAME);
    let temporary: PathBuf = directory.join(format!("{RECORD_NAME}.writing"));
    std::fs::write(&temporary, format!("{}\n", git.display()))?;
    return std::fs::rename(&temporary, record);
}

/// Controls of the record stay out of the release executable.
#[cfg(test)]
#[path = "real_git_record_tests.rs"]
mod tests;
