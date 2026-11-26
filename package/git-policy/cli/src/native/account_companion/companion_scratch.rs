//! What:
//!  The run's scratch directory under the account's runtime directory:
//!  the folder the caller resolves,
//!  the run directory's name,
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
//!  The runtime directory is an input the add route resolves once,
//!  and not a variable this module reads:
//!  a control passes a folder under its own fixture root,
//!  so no control reads or changes the environment of the process it runs in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const runtime = RuntimeFolder.fromEnvironment(); const run = makeRunDirectory(runtime);
//! sweepOlderRuns(parent); ... removeRunDirectory(run);
//! ```

use super::companion_process_calls::is_alive;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// The variable that names the account's runtime directory.
pub const RUNTIME_VARIABLE: &str = "XDG_RUNTIME_DIR";

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
///  The account's runtime directory,
///  which the add route resolves once and hands to the companion.
/// Why:
///  The folder is an input of the run rather than a variable the run reads,
///  so the route is the one place the environment is read
///  and a control passes the folder it made under its own fixture root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runtime = RuntimeFolder.fromEnvironment(); // RuntimeFolder | ScratchRefusal
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFolder {
    /// The directory the variable named.
    path: PathBuf,
}

impl RuntimeFolder {
    /// What:
    ///  The account's runtime directory as this process's environment names it,
    ///  which the add route reads once before the check.
    /// Why:
    ///  A variable that names nothing is the refusal a person already reads,
    ///  and it is answered here rather than by the call site.
    ///
    /// # Errors
    /// The variable is unset,
    ///  or what it names is no directory.
    pub fn from_environment() -> Result<RuntimeFolder, ScratchRefusal> {
        return RuntimeFolder::from_name(std::env::var_os(RUNTIME_VARIABLE));
    }

    /// What:
    ///  The folder one name resolves to,
    ///  which is the whole of what the environment contributes.
    /// Why:
    ///  A control proves the words an unset variable prints,
    ///  and the words a name that is no directory prints,
    ///  by passing the name rather than by changing the environment of the process
    ///  it runs in.
    ///
    /// # Errors
    /// Nothing is named,
    ///  or what is named is no directory.
    pub fn from_name(named: Option<OsString>) -> Result<RuntimeFolder, ScratchRefusal> {
        let Some(name) = named else {
            return Err(ScratchRefusal {
                message: format!(
                    "{RUNTIME_VARIABLE} is not set, so the account companion has no runtime directory to keep its scratch in. Ways forward: log in again, or set {RUNTIME_VARIABLE} to this account's runtime directory, then run the add again"
                ),
            });
        };
        return RuntimeFolder::at(Path::new(name.as_os_str()));
    }

    /// What:
    ///  The folder a caller names,
    ///  which is where a control keeps its run.
    /// Why:
    ///  Every directory the companion makes lives under this one,
    ///  and a run that cannot name it refuses rather than mounting elsewhere.
    ///
    /// # Errors
    /// The path is no directory.
    pub fn at(path: &Path) -> Result<RuntimeFolder, ScratchRefusal> {
        if !path.is_dir() {
            return Err(ScratchRefusal {
                message: format!(
                    "{RUNTIME_VARIABLE} names {}, which is no directory, so the account companion cannot keep its scratch there. Ways forward: log in again so the session makes it, then run the add again",
                    path.display()
                ),
            });
        }
        return Ok(RuntimeFolder {
            path: path.to_path_buf(),
        });
    }

    /// The directory the variable named.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }
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
/// A directory could not be made.
pub fn make_run_directory(runtime: &RuntimeFolder) -> Result<RunDirectory, ScratchRefusal> {
    let folder: PathBuf = runtime.path().join(COMPANION_FOLDER);
    std::fs::create_dir_all(folder.as_path())
        .map_err(|error| return refused(folder.as_path(), error))?;
    sweep(folder.as_path());
    let root: PathBuf = folder.join(run_name());
    std::fs::create_dir(root.as_path()).map_err(|error| return refused(root.as_path(), error))?;
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path())
        .map_err(|error| return refused(scratch.as_path(), error))?;
    return Ok(RunDirectory {
        root,
        scratch,
    });
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
    let named: Option<String> = std::fs::read_link("/proc/self/ns/pid")
        .ok()
        .map(|path| return path.to_string_lossy().into_owned());
    let Some(text) = named else {
        return String::from("0");
    };
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    return if digits.is_empty() {
        String::from("0")
    } else {
        digits
    };
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
        let Some(rest) = name.strip_prefix(RUN_PREFIX) else {
            continue;
        };
        let (Some(pid), Some(namespace)) = (
            rest.split('-')
                .next()
                .and_then(|text| return text.parse::<libc::pid_t>().ok()),
            rest.split('-').nth(1),
        ) else {
            continue;
        };
        if namespace != ours || is_alive(pid) {
            continue;
        }
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(listed.path().as_path());
    }
}

/// The refusal one directory that could not be made answers.
fn refused(
    path: &Path,
    error: std::io::Error,
) -> ScratchRefusal {
    return ScratchRefusal {
        message: format!(
            "the account companion's scratch directory {} cannot be made ({error}), so the managed attach is refused as unproven. Ways forward: free some space under {RUNTIME_VARIABLE} or check that it is writable, then run the add again",
            path.display()
        ),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_scratch_tests.rs"]
mod tests;
