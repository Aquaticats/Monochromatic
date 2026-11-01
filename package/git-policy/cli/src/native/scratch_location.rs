//! What:
//!  Where the wrapper keeps short-lived private files:
//!  the per-user scratch directory (`<state>/cli-git/scratch`) for work that has no repository
//!       in hand,
//!  and the repository scratch directory (`<common-dir>/cli-git-scratch`) for work inside one.
//!  Both are checked as private (a real directory of this user, mode `0700`) and swept of entries
//!       older than a day each time they are prepared.
//! Why:
//!  The system temporary directory is never used (issue 1809, repo rule XTD):
//!  which device backs it is unknowable (a small tmpfs, a per-user quota, a slow disk),
//!       and a full one broke every session.
//!  A file left by a killed process must not accumulate without bound,
//!       so every preparation of a directory also collects what is stale in it.
//!  `doc/planning/cli-git-scratch-locations.md` records the choice per site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scratch = join(env.XDG_STATE_HOME ?? join(env.HOME, '.local/state'), 'cli-git', 'scratch');
//! await mkdir(scratch, { recursive: true, mode: 0o700 });
//! await sweepOlderThan(scratch, 24 * 60 * 60 * 1000);
//! ```

/// The environment lookup shared by the wrapper.
use super::child_environment::environment_value;
/// Debug lines for problems that never fail a command.
use super::diagnostic_log::debug;
/// The private creation primitives shared by every durable record.
use super::private_storage::{create_private_directory, create_private_directory_all};
/// The mode a private directory ends with, and the call that sets it, which only Unix has.
#[cfg(unix)]
use super::private_storage::{PRIVATE_DIRECTORY_MODE, protect_path};
/// OS strings, paths and time.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The tag of this module's diagnostics.
const TAG: &str = "cli-git";

/// The entry in a Git common directory that holds repository scratch.
pub const REPOSITORY_SCRATCH_NAME: &str = "cli-git-scratch";

/// How old an entry must be before a preparation removes it:
///  far longer than any command of the wrapper runs (its longest bound is minutes),
///  so a live process never loses a file.
pub const SCRATCH_STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

/// What:
///  The per-user state directory of the wrapper:
///  `cli-git` under an absolute `XDG_STATE_HOME`,
///  or under `.local/state` in the home directory.
/// Why:
///   The injected environment decides the location,
///  never a fixed path;
///  a relative `XDG_STATE_HOME` is ignored as the XDG rules say.
///  It mirrors the cache directory of the version cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = isAbsolute(env.XDG_STATE_HOME ?? '') ? env.XDG_STATE_HOME : env.HOME && join(env.HOME, '.local/state');
/// ```
pub fn state_directory(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    if let Some(state_home) = environment_value(environment, "XDG_STATE_HOME") {
        let state_path: PathBuf = PathBuf::from(state_home);
        if state_path.is_absolute() {
            return Some(state_path.join("cli-git"));
        }
    }
    let home: OsString = environment_value(environment, "HOME")?;
    if home.is_empty() {
        return None;
    }
    return Some(PathBuf::from(home).join(".local").join("state").join("cli-git"));
}

/// What:
///  The per-user scratch directory's path,
///  or the sentence that says what the environment lacks.
///  Nothing is created.
/// Why:
///   Callers that run without a repository learn the location first and create it only when they need it,
///  so a command with nothing to write never touches the disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function userScratchPath(env): string; // throws string
/// ```
pub fn user_scratch_path(environment: &[(OsString, OsString)]) -> Result<PathBuf, String> {
    let Some(state) = state_directory(environment) else {
        return Err(String::from(
            "cli-git keeps its temporary files in a private directory under XDG_STATE_HOME (or HOME/.local/state) and \
             never in the system temporary directory, and this command's environment has neither an absolute \
             XDG_STATE_HOME nor a non-empty HOME; set one of them",
        ));
    };
    return Ok(state.join("scratch"));
}

/// What:
///  Create the per-user scratch directory with its missing parents,
///  check that it is private,
///  sweep its stale entries,
///  and return it.
/// Why:
///   The linter and the push probe write there;
///  the check refuses a directory another account could write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareUserScratch(env): Promise<string>; // throws string
/// ```
pub fn prepare_user_scratch(environment: &[(OsString, OsString)]) -> Result<PathBuf, String> {
    let given: PathBuf = user_scratch_path(environment)?;
    // A relative home would name another place once a child changes directory.
    let directory: PathBuf = std::path::absolute(&given).unwrap_or_else(|_| { return given.clone(); });
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!(
            "cli-git could not create its private scratch directory {}: {error}",
            directory.display()
        ));
    }
    if let Err(error) = assert_private_directory(directory.as_path()) {
        return Err(format!(
            "cli-git will not use {} for temporary files: {error}",
            directory.display()
        ));
    }
    sweep_stale(directory.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER);
    return Ok(directory);
}

/// What:
///  An anonymous file (no name in the directory) in the per-user scratch directory of `environment`,
///  open for reading and writing.
/// Why:
///   Child output goes to a file so that no reader thread can outlive a stopped child;
///  the file has no name from the start (an unnamed temporary file where the system has them,
///  otherwise a name removed at once),
///  so a killed process leaves nothing to collect,
///  and the system temporary directory is never used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(await prepareUserScratch(env), 'wx+'); // unlinked at once
/// ```
pub fn scratch_file(environment: &[(OsString, OsString)]) -> std::io::Result<std::fs::File> {
    let directory: PathBuf = prepare_user_scratch(environment).map_err(std::io::Error::other)?;
    return tempfile::tempfile_in(directory);
}

/// What:
///  Create the repository scratch directory beside the common Git directory's other private storage,
///  check that it is private,
///  sweep its stale entries,
///  and return it.
/// Why:
///   Git input for a commit transaction is written there:
///  it is on the repository's own device,
///  needs no environment variable,
///  and is collected with the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareRepositoryScratch(commonDir: string): Promise<string>;
/// ```
pub fn prepare_repository_scratch(common_dir: &Path) -> std::io::Result<PathBuf> {
    let directory: PathBuf = common_dir.join(REPOSITORY_SCRATCH_NAME);
    match create_private_directory(directory.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    assert_private_directory(directory.as_path())?;
    sweep_stale(directory.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER);
    return Ok(directory);
}

/// What:
///  Accept `path` only if it is a real directory (not a link) of this user that no one else can enter,
///  tightening a looser mode of a directory this user owns.
/// Why:
///   Another account that could write into the directory could plant a link or read the files,
///  and a link in place of the directory could redirect every write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const s = await lstat(path); if (!s.isDirectory() || s.uid !== process.geteuid()) throw ...; if (s.mode & 0o077) await chmod(path, 0o700);
/// ```
#[cfg(unix)]
fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    // `MetadataExt` gives the owner and the mode bits.
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Err(std::io::Error::other("it is a link or a file, not a directory"));
    }
    // SAFETY: `geteuid` has no preconditions.
    let owner: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != owner {
        return Err(std::io::Error::other("it belongs to another account"));
    }
    if metadata.mode() & 0o077 != 0 {
        protect_path(path, PRIVATE_DIRECTORY_MODE)?;
    }
    return Ok(());
}

/// What:
///  Accept `path` only if it is a real directory.
/// Why:
///   Ownership and modes are Unix notions;
///  other platforms keep their inherited access rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(await lstat(path)).isDirectory()) throw ...;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    if !std::fs::symlink_metadata(path)?.is_dir() {
        return Err(std::io::Error::other("it is a link or a file, not a directory"));
    }
    return Ok(());
}

/// What:
///  Remove every entry of `directory` last modified more than `max_age` before `now`,
///  and return how many it removed.
///  A link is removed itself and never followed;
///  a directory is removed with its contents.
/// Why:
///   A process killed between making a scratch file and removing it leaves the file behind,
///  and nothing else would ever collect it.
///  The directory belongs to the wrapper alone,
///  so every entry in it is the wrapper's;
///  an entry modified in the future is kept,
///  because a wrong clock must not delete live files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of await readdir(dir)) if (now - (await lstat(entry)).mtimeMs > maxAgeMs) await rm(entry, { recursive: true });
/// ```
pub fn sweep_stale(directory: &Path, now: SystemTime, max_age: Duration) -> usize {
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => {
            debug(TAG, format!("could not list {} for stale entries: {error}", directory.display()).as_str());
            return 0;
        }
    };
    let mut removed: usize = 0;
    for listed in entries {
        let entry: std::fs::DirEntry = match listed {
            Ok(found) => found,
            Err(error) => {
                debug(TAG, format!("could not read an entry of {}: {error}", directory.display()).as_str());
                continue;
            }
        };
        let path: PathBuf = entry.path();
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(&path) {
            Ok(found) => found,
            Err(error) => {
                debug(TAG, format!("could not inspect {}: {error}", path.display()).as_str());
                continue;
            }
        };
        let age: Option<Duration> = metadata
            .modified()
            .ok()
            .and_then(|modified| { return now.duration_since(modified).ok(); });
        if !age.is_some_and(|found| { return found > max_age; }) {
            continue;
        }
        let outcome: std::io::Result<()> = if metadata.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match outcome {
            Ok(()) => removed += 1,
            Err(error) => debug(TAG, format!("could not remove the stale {}: {error}", path.display()).as_str()),
        }
    }
    return removed;
}

/// Environment resolution, preparation, privacy checks and the sweep are controlled in the test module.
#[cfg(test)]
#[path = "scratch_location_tests.rs"]
mod tests;
