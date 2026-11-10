//! What:
//!  Fold the names an earlier release kept in a Git directory into the current names when both exist:
//! logs by adding the earlier
//!  log's bytes to the current log,
//! folders by moving the earlier folder's entries into the current folder.
//! It runs only when no
//!  earlier program that writes them is running,
//! says so once when it waits,
//! and says once when it cannot finish.
//! Why:
//!  Both names exist in a repository where development builds of the renamed wrapper and the installed earlier release ran side
//!  by side,
//! and every command used to print one line per pair.
//! The owner's rules are that a person need not merge folders by
//!  hand and that no user data is discarded;
//! the fold makes the current name hold everything and leaves no earlier name.
//!  The checks that make it safe sit here:
//! the look at the earlier commit service of the Git directory's worktree (processes
//!  only,
//! because a socket whose unit stays active for good writes nothing),
//! then the fold of each name,
//! which loses nothing on
//!  a kill at any instruction (`earlier_log_lines_added_to_current_log.rs`,
//!  `earlier_folder_entries_moved_into_current_folder.rs`).
//! It runs on the command's own path because a forwarded command
//!  replaces the process and a thread would die with it;
//! the cost when no name exists twice is a few `stat` calls per name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! foldAndTell(gitDir, [DATA_FOLDER, MERGE_INTENT_FOLDER]); // silent after a fold; one line when it waits or fails
//! ```

use crate::earlier_folder_entries_moved_into_current_folder::{FolderFold, fold_folder};
use crate::earlier_log_lines_added_to_current_log::{FoldSteps, LogFold, NeverStopped, fold_log, numbered_copies};
use crate::earlier_service_journal_adopted::earlier_journals_in_state;
use crate::earlier_service_process_look::{Hold, Holding, ServiceLook, earlier_service_process_hold, effective_user};
use crate::rename_that_never_replaces::rename_noreplace;
use crate::service_instance::instance_of;
use crate::wait_ceilings::PROCESS_LOOK_CEILING;
use crate::wrapper_names_old_and_new::{KEPT_BESIDE_SUFFIX, MESSAGE_PREFIX, RenamedName};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// What kind of thing a kept name is,
/// which decides how it folds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeptKind {
    /// A log of lines,
    /// which keeps numbered older copies beside the one under the current name.
    Log,
    /// A folder of saved copies,
    /// records or notices.
    Folder,
}

/// One kept name to fold.
#[derive(Clone, Copy, Debug)]
pub struct KeptName {
    /// The pair of spellings.
    pub name: RenamedName,
    /// How it folds.
    pub kind: KeptKind,
}

/// What the look at the earlier program needs from the process around it.
#[derive(Clone, Debug)]
pub struct FoldSurroundings {
    /// The user's state folder,
    /// where the earlier commit service kept its journals.
    pub state_dir: Option<PathBuf>,
    /// The process table to look at.
    pub proc_root: PathBuf,
}

/// The path an environment variable holds,
/// or nothing when it is unset or empty.
fn non_empty_environment_path(name: &str) -> Option<PathBuf> {
    let value: OsString = std::env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    return Some(PathBuf::from(value));
}

/// The state folder under the home folder,
/// or nothing when no home is set.
fn home_state_dir() -> Option<PathBuf> {
    let home: PathBuf = non_empty_environment_path("HOME")?;
    return Some(home.join(".local").join("state"));
}

impl FoldSurroundings {
    /// The surroundings of this process:
    /// its environment and the live process table.
    pub fn of_this_process() -> Self {
        let named: Option<PathBuf> = non_empty_environment_path("XDG_STATE_HOME");
        let state_dir: Option<PathBuf> = named.or_else(home_state_dir);
        return FoldSurroundings { state_dir, proc_root: PathBuf::from("/proc") };
    }
}

/// How the fold of one kept name ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Nothing existed twice.
    Nothing,
    /// The earlier name is gone and its content is under the current name.
    Folded,
    /// A program still has the moved log open for writing:
    /// every byte it held so far is in the current log,
    /// nothing is said,
    ///  and a later command adds what the program writes next and removes the moved log once the program has closed it.
    LeftForALaterCommand,
    /// Another process is folding it.
    Busy,
    /// Entries of the earlier folder stayed where they were;
    /// each is listed.
    Left(Vec<PathBuf>),
    /// The file system refused;
    /// the message names the cause.
    /// Everything is still in place.
    Failed(String),
    /// A control stopped the fold.
    Stopped(&'static str),
}

/// What happened in one Git directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoldReport {
    /// Why nothing was folded,
    /// when an earlier program may be writing.
    pub held: Option<Hold>,
    /// The outcome of each name that existed twice.
    pub outcomes: Vec<(RenamedName, Outcome)>,
}

/// Whether anything exists at a path,
/// a final symbolic link counting as present.
fn present(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// Whether a kept name needs folding:
/// both names exist,
/// or a fold of a log was interrupted and left its moved log or journal.
fn needs_fold(directory: &Path, kept: KeptName) -> bool {
    let earlier: bool = present(directory.join(kept.name.earlier).as_path());
    let current: bool = present(directory.join(kept.name.current).as_path());
    return match kept.kind {
        KeptKind::Folder => earlier && current,
        KeptKind::Log { .. } => {
            let leftovers: bool = present(directory.join(format!("{}{}", kept.name.current, crate::earlier_log_lines_added_to_current_log::WAITING_SUFFIX)).as_path())
                || present(directory.join(format!("{}{}", kept.name.current, crate::earlier_log_lines_added_to_current_log::JOURNAL_SUFFIX)).as_path());
            (earlier && current) || leftovers
        }
    };
}

/// What:
///  Keep the numbered older copies of an earlier rotating log beside the current ones:
/// each `<earlier>.<number>` goes to
///  `<current>.<number>`,
/// or beside it with the suffix when that name is taken.
/// Why:
///   The audit log's older copies name saved files still on disk,
/// so none is left under a name nothing reads and none replaces
///  another.
/// Each is one rename that never replaces;
/// a copy whose both target names are taken stays where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const copy of numberedCopies(parent, earlier)) await renameNoReplace(copy, current + number) ?? renameNoReplace(copy, current + number + suffix)
/// ```
fn keep_numbered_copies(directory: &Path, name: RenamedName) -> std::io::Result<()> {
    for copy in numbered_copies(directory, name.earlier) {
        let Some(found) = copy.file_name() else {
            continue;
        };
        let file_name: String = found.to_string_lossy().into_owned();
        let Some(number) = file_name.strip_prefix(format!("{}.", name.earlier).as_str()) else {
            continue;
        };
        let target: PathBuf = directory.join(format!("{}.{number}", name.current));
        match rename_noreplace(copy.as_path(), target.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let beside: PathBuf = directory.join(format!("{}.{number}{KEPT_BESIDE_SUFFIX}", name.current));
                match rename_noreplace(copy.as_path(), beside.as_path()) {
                    Ok(()) => {}
                    Err(second) if second.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(second) => return Err(second),
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// Fold one kept name.
fn fold_one(directory: &Path, kept: KeptName, steps: &mut dyn FoldSteps) -> Outcome {
    match kept.kind {
        KeptKind::Folder => match fold_folder(directory, kept.name, steps) {
            Ok(FolderFold::Nothing) => return Outcome::Nothing,
            Ok(FolderFold::Folded { .. }) => return Outcome::Folded,
            Ok(FolderFold::Left { kept: stayed }) => return Outcome::Left(stayed),
            Ok(FolderFold::NotRemoved { reason }) => return Outcome::Failed(format!("every entry moved, but the emptied earlier folder could not be removed: {reason}")),
            Ok(FolderFold::Stopped(step)) => return Outcome::Stopped(step),
            Err(error) => return Outcome::Failed(error.to_string()),
        },
        KeptKind::Log => {
            let folded: Result<LogFold, std::io::Error> = fold_log(directory, kept.name, steps);
            let copies: std::io::Result<()> = keep_numbered_copies(directory, kept.name);
            return match (folded, copies) {
                (Err(error), _) | (_, Err(error)) => Outcome::Failed(error.to_string()),
                (Ok(LogFold::Nothing), Ok(())) => Outcome::Nothing,
                (Ok(LogFold::Folded), Ok(())) => Outcome::Folded,
                (Ok(LogFold::Busy), Ok(())) => Outcome::Busy,
                (Ok(LogFold::LeftForALaterCommand), Ok(())) => Outcome::LeftForALaterCommand,
                (Ok(LogFold::Stopped(step)), Ok(())) => Outcome::Stopped(step),
            };
        }
    }
}

/// The reason a look held a fold back,
/// without the program behind it.
fn reason_of(holding: Holding) -> Hold {
    return holding.because;
}

/// The look at the earlier commit service of the worktree whose Git directory this is,
/// processes only.
fn earlier_program_hold(directory: &Path, surroundings: &FoldSurroundings) -> Option<Hold> {
    let instance: String = instance_of(directory);
    let journals: Vec<PathBuf> = match surroundings.state_dir.as_deref() {
        Some(state) => {
            let earlier = earlier_journals_in_state(state, instance.as_str());
            vec![earlier.per_instance, earlier.shared]
        }
        None => Vec::new(),
    };
    let look: ServiceLook = ServiceLook { instance, journals, own_pid: std::process::id(), own_user: effective_user(), ceiling: PROCESS_LOOK_CEILING };
    return earlier_service_process_hold(look, surroundings.proc_root.as_path()).map(reason_of);
}

/// What:
///  Fold every kept name of `names` that exists twice in `directory`,
/// unless an earlier program may be writing.
/// Why:
///   The look runs only when something exists twice,
/// so a repository with nothing to fold pays only the `stat` calls.
///   `steps` lets a control stop the fold at a step,
/// as a kill would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function foldKeptNames(directory: string, names: KeptName[], surroundings: FoldSurroundings, steps: FoldSteps): FoldReport;
/// ```
pub fn fold_kept_names(directory: &Path, names: &[KeptName], surroundings: &FoldSurroundings, steps: &mut dyn FoldSteps) -> FoldReport {
    let mut wanted: Vec<KeptName> = Vec::new();
    for kept in names.iter().copied() {
        if needs_fold(directory, kept) {
            wanted.push(kept);
        }
    }
    if wanted.is_empty() {
        return FoldReport { held: None, outcomes: Vec::new() };
    }
    if let Some(hold) = earlier_program_hold(directory, surroundings) {
        return FoldReport { held: Some(hold), outcomes: Vec::new() };
    }
    let mut outcomes: Vec<(RenamedName, Outcome)> = Vec::new();
    for kept in wanted {
        let outcome: Outcome = fold_one(directory, kept, steps);
        let stopped: bool = matches!(outcome, Outcome::Stopped(_));
        outcomes.push((kept.name, outcome));
        if stopped {
            break;
        }
    }
    return FoldReport { held: None, outcomes };
}

/// Whether this process has already said that a fold waits.
static WAIT_SAID: AtomicBool = AtomicBool::new(false);

/// Whether this process has already said that a fold could not finish.
static FAILURE_SAID: AtomicBool = AtomicBool::new(false);

/// The line that says a fold waits,
/// with what to do in plain words.
fn waiting_line(directory: &Path, hold: Hold) -> String {
    let instance: String = instance_of(directory);
    let stop: String = match hold {
        Hold::InstanceHelperRuns => format!("stop it with `systemctl --user stop cli-git@{instance}.socket cli-git@{instance}.service`"),
        _ => String::from("stop the commit services of the earlier release (`systemctl --user list-units 'cli-git@*'` lists them)"),
    };
    return format!(
        "{}: names the earlier release gave its saved data still sit beside the new names in {}. Nothing is lost and nothing is \
         needed from you: a later command folds them into the new names once the earlier release's program has stopped, because \
         {}. To let it happen sooner, {stop}.\n",
        MESSAGE_PREFIX.current,
        directory.display(),
        hold.words()
    );
}

/// The line that says a fold could not finish.
fn failure_line(directory: &Path, name: RenamedName, outcome: &Outcome) -> Option<String> {
    let because: String = match outcome {
        Outcome::Failed(message) => message.clone(),
        Outcome::Left(kept) => format!(
            "{} entries could not be moved because both names they would take are already in use; they stay where they are",
            kept.len()
        ),
        _ => return None,
    };
    return Some(format!(
        "{}: {} in {} could not be folded into {} ({because}). Everything is still in place, and the next command tries again.\n",
        MESSAGE_PREFIX.current,
        name.earlier,
        directory.display(),
        name.current
    ));
}

/// What:
///  `fold_kept_names`,
/// with the one line to standard error:
/// once per process when the fold waits for an earlier program,
///  and once per process when a name could not be folded.
/// Why:
///   The fold never stops a command;
/// the lines are the only report of a fold that did not happen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function foldAndTell(directory: string, names: KeptName[]): void;
/// ```
pub fn fold_and_tell(directory: &Path, names: &[KeptName]) {
    fold_kept_names_and_tell(directory, names, &FoldSurroundings::of_this_process(), &mut NeverStopped);
}

/// What:
///  `fold_and_tell` with the surroundings and steps given.
/// Why:
///   The surroundings name the process table and the state folder the look reads,
/// and a control must give its own:
/// a fold that reads the host's processes and services takes a different decision
///  depending on what else runs on the machine
/// (four controls of `kept_names_in_the_git_directories` deferred a fold because the
///  look met this host's own services on 2026-11-08).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// foldKeptNamesAndTell(directory, names, { procRoot: fixtureProc, stateDir: null }, steps);
/// ```
pub fn fold_kept_names_and_tell(directory: &Path, names: &[KeptName], surroundings: &FoldSurroundings, steps: &mut dyn FoldSteps) {
    let report: FoldReport = fold_kept_names(directory, names, surroundings, steps);
    if let Some(hold) = report.held
        && !WAIT_SAID.swap(true, Ordering::Relaxed)
    {
        let _written = std::io::stderr().write_all(waiting_line(directory, hold).as_bytes());
    }
    for (name, outcome) in &report.outcomes {
        if let Some(line) = failure_line(directory, *name, outcome)
            && !FAILURE_SAID.swap(true, Ordering::Relaxed)
        {
            let _written = std::io::stderr().write_all(line.as_bytes());
        }
    }
}

#[cfg(test)]
#[path = "earlier_names_folded_into_current_tests.rs"]
mod tests;
