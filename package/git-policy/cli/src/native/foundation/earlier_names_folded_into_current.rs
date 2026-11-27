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
//! What a fold is given and what it says are two sibling files:
//!  `what_a_fold_is_given_and_what_it_answers.rs` and `what_a_fold_says_to_a_person.rs`.//!
//! In TS you'd write (pseudocode):
//! ```ts
//! foldAndTell(gitDir, [DATA_FOLDER, MERGE_INTENT_FOLDER]); // silent after a fold; one line when it waits or fails
//! ```

/// The fold of a folder kept under an earlier name,
/// and how it ended.
use crate::earlier_folder_entries_moved_into_current_folder::{
    FolderFold,
    fold_folder,
};
/// The fold of a log kept under an earlier name,
/// the steps it reports through and its numbered copies.
use crate::earlier_log_lines_added_to_current_log::{
    FoldSteps,
    LogFold,
    fold_log,
    numbered_copies,
};
/// The journals an earlier commit service kept in the state folder,
/// and their names.
use crate::earlier_service_journal_adopted::earlier_journals_in_state;
/// The look at an earlier program of this Git directory,
/// and the ceiling on that look.
use crate::earlier_service_process_look::{
    Hold,
    Holding,
    ServiceLook,
    earlier_service_process_hold,
    effective_user,
};
/// The rename that never replaces an existing name.
use crate::private_storage::rename_noreplace;
/// The instance a Git directory belongs to.
use crate::service_instance::instance_of;
/// The ceiling on a look at the process table.
use crate::wait_ceilings::PROCESS_LOOK_CEILING;
/// What a fold is given and what it answers:
/// the names and their kinds,
/// the surroundings it reads,
/// how one name's fold ended,
/// and one Git directory's report.
pub use crate::what_a_fold_is_given_and_what_it_answers::{
    FoldReport,
    FoldSurroundings,
    KeptKind,
    KeptName,
    Outcome,
};
/// The two lines a fold says,
/// and the switches that say each once in a process.
use crate::what_a_fold_says_to_a_person::{
    FAILURE_SAID,
    WAIT_SAID,
    failure_line,
    waiting_line,
};
/// The suffix a copy of a moving log takes when its name is taken.
use crate::wrapper_names_old_and_new::KEPT_BESIDE_SUFFIX;
/// The pair of spellings of each name the folds above work on.
use crate::wrapper_names_old_and_new::RenamedName;
/// Writing a line to standard error.
use std::io::Write;
/// Paths of the Git directory,
/// its entries and the state folder.
use std::path::{
    Path,
    PathBuf,
};
/// The order a switch's swap takes.
use std::sync::atomic::Ordering;

/// Whether anything exists at a path,
/// a final symbolic link counting as present.
fn present(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// Whether a kept name needs folding:
/// both names exist,
/// or a fold of a log was interrupted and left its moved log or journal.
fn needs_fold(
    directory: &Path,
    kept: KeptName,
) -> bool {
    let earlier: bool = present(directory.join(kept.name.earlier).as_path());
    let current: bool = present(directory.join(kept.name.current).as_path());
    return match kept.kind {
        KeptKind::Folder => earlier && current,
        KeptKind::Log => {
            let leftovers: bool = present(
                directory
                    .join(format!(
                        "{}{}",
                        kept.name.current,
                        crate::earlier_log_lines_added_to_current_log::WAITING_SUFFIX
                    ))
                    .as_path(),
            ) || present(
                directory
                    .join(format!(
                        "{}{}",
                        kept.name.current,
                        crate::earlier_log_lines_added_to_current_log::JOURNAL_SUFFIX
                    ))
                    .as_path(),
            );
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
fn keep_numbered_copies(
    directory: &Path,
    name: RenamedName,
) -> std::io::Result<()> {
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
                let beside: PathBuf =
                    directory.join(format!("{}.{number}{KEPT_BESIDE_SUFFIX}", name.current));
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
fn fold_one(
    directory: &Path,
    kept: KeptName,
    steps: &mut dyn FoldSteps,
) -> Outcome {
    match kept.kind {
        KeptKind::Folder => match fold_folder(directory, kept.name, steps) {
            Ok(FolderFold::Nothing) => return Outcome::Nothing,
            Ok(FolderFold::Folded {
                ..
            }) => return Outcome::Folded,
            Ok(FolderFold::Left {
                kept: stayed,
            }) => return Outcome::Left(stayed),
            Ok(FolderFold::NotRemoved {
                reason,
            }) => {
                return Outcome::Failed(format!(
                    "every entry moved, but the emptied earlier folder could not be removed: {reason}"
                ));
            }
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
fn earlier_program_hold(
    directory: &Path,
    surroundings: &FoldSurroundings,
) -> Option<Hold> {
    let instance: String = instance_of(directory);
    let journals: Vec<PathBuf> = match surroundings.state_dir.as_deref() {
        Some(state) => {
            let earlier = earlier_journals_in_state(state, instance.as_str());
            vec![earlier.per_instance, earlier.shared]
        }
        None => Vec::new(),
    };
    let look: ServiceLook = ServiceLook {
        instance,
        journals,
        own_pid: std::process::id(),
        own_user: effective_user(),
        ceiling: PROCESS_LOOK_CEILING,
    };
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
pub fn fold_kept_names(
    directory: &Path,
    names: &[KeptName],
    surroundings: &FoldSurroundings,
    steps: &mut dyn FoldSteps,
) -> FoldReport {
    let mut wanted: Vec<KeptName> = Vec::new();
    for kept in names.iter().copied() {
        if needs_fold(directory, kept) {
            wanted.push(kept);
        }
    }
    if wanted.is_empty() {
        return FoldReport {
            held: None,
            outcomes: Vec::new(),
        };
    }
    if let Some(hold) = earlier_program_hold(directory, surroundings) {
        return FoldReport {
            held: Some(hold),
            outcomes: Vec::new(),
        };
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
    return FoldReport {
        held: None,
        outcomes,
    };
}

/// What:
///  The fold of one Git directory's kept names,
/// with the surroundings and steps given,
/// and its one line to standard error.
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
pub fn fold_kept_names_and_tell(
    directory: &Path,
    names: &[KeptName],
    surroundings: &FoldSurroundings,
    steps: &mut dyn FoldSteps,
) {
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
