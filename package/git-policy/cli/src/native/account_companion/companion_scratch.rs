//! What:
//!  The run's scratch directory under the account's runtime directory:
//!  its name,
//!  its making,
//!  the sweep of the directories a killed run left,
//!  and its removal.
//! Why:
//!  The companion mounts its tmpfs inside a directory of its own under
//!  `$XDG_RUNTIME_DIR`,
//!  never under the checkout,
//! the slot tree,
//! the staging folder or the clone
//!  parent,
//! so no path mount or unmount touches the tree the plan's rule forbids
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Where its scratch directory lives").
//!  A run that is killed leaves its directory,
//!  and only the next add removes it,
//!  after proving the process that made it is gone from this PID namespace:
//!  a directory of another namespace is left alone and named,
//!  because a live mount of a live foreign child would be detached by a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const run = makeRunDirectory(); sweepOlderRuns(parent); ... removeRunDirectory(run);
//! ```

use super::companion_process_calls::is_alive;
use std::path::{Path, PathBuf};

/// The folder under `$XDG_RUNTIME_DIR` the companion keeps its runs in.
pub const COMPANION_FOLDER: &str = "monochromatic-git-policy-cli/account-companion";

/// The word every run directory's name begins with.
pub const RUN_PREFIX: &str = "run-";

/// What:
///  One run's own directory,
///  and the mount point inside it.
/// Why:
///  The mount point is a directory the tmpfs is mounted over,
///  so the run's own name is what the sweep reads and the removal takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunDirectory = { root: string; scratch: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunDirectory {
    /// The run's directory.
    pub root: PathBuf,
    /// The scratch mount point inside it.
    pub scratch: PathBuf,
}

/// What:
///  Why a run has no scratch directory:
///  the runtime directory is unset or unwritable,
///  or one of them could not be made.
/// Why:
///  The refusal names the variable and what to do,
///  in the wording the wrapper's provisioning already uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScratchRefusal = { message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScratchRefusal {
    /// The sentence a person reads.
    pub message: String,
}

/// What:
///  Make this run's directory and its scratch mount point,
///  after sweeping what a killed run left.
/// Why:
///  The sweep runs on every add rather than on a clock,
///  so a directory a kill left is removed by the next add that can prove it
///  dead.
///
/// # Errors
/// The runtime directory is unset or unwritable,
///  or a directory could not be made.
pub fn make_run_directory() -> Result<RunDirectory, ScratchRefusal> {
    let base: PathBuf = runtime_base()?;
    let folder: PathBuf = base.join(COMPANION_FOLDER);
    std::fs::create_dir_all(folder.as_path()).map_err(|error| return refused(folder.as_path(), error))?;
    sweep(folder.as_path());
    let root: PathBuf = folder.join(run_name());
    std::fs::create_dir(root.as_path()).map_err(|error| return refused(root.as_path(), error))?;
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).map_err(|error| return refused(scratch.as_path(), error))?;
    return Ok(RunDirectory { root, scratch });
}

/// What:
///  Remove one run's directory,
///  which the parent does only after it has reaped its child.
/// Why:
///  A child a signal cannot end keeps its namespace and its mount,
///  so a directory removed before the wait returns would detach a live mount
///  from that namespace.
pub fn remove_run_directory(run: &RunDirectory) {
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(run.root.as_path());
}

/// What:
///  The account's runtime directory.
/// Why:
///  Every directory the companion makes lives under it,
///  and a run that cannot name it refuses rather than mounting elsewhere.
fn runtime_base() -> Result<PathBuf, ScratchRefusal> {
    let Some(base) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return Err(ScratchRefusal {
            message: String::from(
                "XDG_RUNTIME_DIR is not set, so the account companion has no runtime directory to keep its scratch in. Ways forward: log in again, or set XDG_RUNTIME_DIR to this account's runtime directory, then run the add again",
            ),
        });
    };
    let path: PathBuf = PathBuf::from(base);
    if !path.is_dir() {
        return Err(ScratchRefusal {
            message: format!(
                "XDG_RUNTIME_DIR names {}, which is no directory, so the account companion cannot keep its scratch there. Ways forward: log in again so the session makes it, then run the add again",
                path.display()
            ),
        });
    }
    return Ok(path);
}

/// What:
///  This run's directory name,
///  which carries the wrapper's pid and this PID namespace.
/// Why:
///  The next add proves a dead run by that pid,
///  and a name written in another PID namespace names another process here,
///  so the namespace is part of the name a sweep reads.
fn run_name() -> String {
    // SAFETY: `getpid` takes no argument and reads one number.
    let pid: libc::pid_t = unsafe { libc::getpid() };
    // Two runs of one process happen at once in this crate's own controls, so each run's name carries a number of its own.
    let order: u64 = RUNS_OF_THIS_PROCESS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    return format!("{RUN_PREFIX}{pid}-{}-{order}", namespace_digits());
}

/// How many runs this process has made,
/// which keeps two of them apart.
static RUNS_OF_THIS_PROCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The digits of this process's PID namespace,
/// and zero where the kernel names none.
fn namespace_digits() -> String {
    let named: Option<String> = std::fs::read_link("/proc/self/ns/pid").ok().map(|path| return path.to_string_lossy().into_owned());
    let Some(text) = named else {
        return String::from("0");
    };
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    return if digits.is_empty() { String::from("0") } else { digits };
}

/// What:
///  Remove every run directory whose process is gone from this PID namespace,
///  and leave every other one.
/// Why:
///  The sweep removes only what it can prove dead,
///  and a directory of another namespace is never proved dead here,
///  so it is left and named rather than guessed at.
fn sweep(folder: &Path) {
    let Ok(listing) = std::fs::read_dir(folder) else {
        return;
    };
    let ours: String = namespace_digits();
    for listed in listing.flatten() {
        let name: String = listed.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix(RUN_PREFIX) else { continue };
        let (Some(pid), Some(namespace)) = (rest.split('-').next().and_then(|text| return text.parse::<libc::pid_t>().ok()), rest.split('-').nth(1)) else {
            continue;
        };
        if namespace != ours || is_alive(pid) {
            continue;
        }
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(listed.path().as_path());
    }
}

/// The refusal one directory that could not be made answers.
fn refused(path: &Path, error: std::io::Error) -> ScratchRefusal {
    return ScratchRefusal {
        message: format!(
            "the account companion's scratch directory {} cannot be made ({error}), so the managed attach is refused as unproven. Ways forward: free some space under XDG_RUNTIME_DIR or check that it is writable, then run the add again",
            path.display()
        ),
    };
}
