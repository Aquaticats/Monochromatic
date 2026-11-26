//! What:
//!  The three steps of one workspace's filling:
//!  clone the source's build directory into the staging copy,
//!  clean the worktree's own crates out of the copy,
//!  publish the copy under the worktree's name by a rename that never replaces,
//!  with the lock the clean must not write,
//!  and the words a failure is reported in.
//! Why:
//!  `package_repair_prewarm_job.rs` would otherwise pass the file-size limit,
//!  and the filling is the one role that starts a tool,
//!  so it is separable from the names it works in,
//!  and from the record the service reads,
//!  and it lives in a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const filled = await fillOne(fill, entry);
//! ```

/// Import the log tag and the names one filling works in.
use super::names::{
    TAG,
    TheNames,
};
/// Import how one workspace's names are taken and given back.
use super::names::{
    give_back,
    give_back_what_a_killed_filling_left,
    make_the_destinations_own_directory,
    names_for,
    remove_if_any,
};
/// Import the names of the worktree's own staged state,
/// when the add moved it.
use super::staged_state::staged_names;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the clean arguments and the command a person runs instead.
use crate::package_repair_cargo::{
    locked_clean_arguments,
    manual_command,
};
/// Import the restore of only the clean's own bytes over the checked-out lock.
use crate::package_repair_cargo_lock::restore_only_the_cleans_own;
/// Import the lock file's name and the read of it.
use crate::package_repair_cargo_lock::{
    LOCK_FILE,
    read_lock,
};
/// Import the command and the bounds of one tool.
use crate::package_repair_command::{
    ToolCommand,
    ToolLimits,
};
/// Import the end of a run and the awaited run of one tool.
use crate::package_repair_command::{
    ToolEnd,
    ToolRun,
    run_tool_awaiting,
};
/// Import the two variables a clean is given and the record's entry.
use crate::package_repair_prewarm::{
    BUILD_DIRECTORY_VARIABLE,
    PrewarmEntry,
    TARGET_DIRECTORY_VARIABLE,
};
/// Import the clone of a directory tree with its modes,
/// times and links.
use crate::package_repair_tree_clone::{
    Cloned,
    clone_tree_awaiting,
};
/// Import the rename that never replaces,
/// and the test for a filesystem that cannot do it.
use crate::private_storage::rename_noreplace;
use crate::rename_that_never_replaces::rename_unsupported;
/// Import the owned argument a command is started with.
use std::ffi::OsString;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  What every workspace of one record needs before it copies anything.
/// Why:
///  The program lookup and the bounds are the same for every workspace of one record,
/// so they are read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fill = { cargo: string; limits: ToolLimits };
/// ```
pub(crate) struct Fill {
    /// The Cargo that cleans the staging copy.
    pub(crate) cargo: PathBuf,
    /// The bounds it runs under.
    pub(crate) limits: ToolLimits,
    /// The Git directory of the worktree whose lock the clean may displace,
    /// or nothing when no Git directory is known,
    /// in which case a lock the clean changed is left as it is found.
    pub(crate) kept_in: Option<PathBuf>,
}

/// What the clean is told about the worktree's lock,
/// so the report of a clean it stopped names the flag to look up.
/// Why:
///   The switch itself is one place,
/// in `package_repair_cargo::locked_clean_arguments`,
/// and the words a person reads are another,
/// so the report spells the same flag these words name.
const LOCK_FLAG: &str = "--locked";

/// What:
///  Why a tool ended as it did,
/// in the words a person reads.
/// Why:
///   A report names what stopped the clean,
/// and the tool's own last words name the manifest or lock it refused,
/// so a variant's name alone is not the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endReason = (end) => end.kind === 'failed' ? end.reason : end.kind === 'timedOut' ? 'it showed no sign of work for its quiet window and was stopped' : ceilingReason(Tool.cargo, end.ceiling);
/// ```
fn end_reason(end: &ToolEnd) -> String {
    return match end {
        ToolEnd::Succeeded => String::from("it succeeded"),
        ToolEnd::Failed(reason) => reason.clone(),
        ToolEnd::TimedOut => {
            String::from("it showed no sign of work for its quiet window and was stopped")
        }
        ToolEnd::CeilingReached(ceiling) => crate::package_repair_report::ceiling_reason(
            crate::package_repair_report::Tool::Cargo,
            *ceiling,
        ),
    };
}

/// What:
///  What a person does next when the filling's clean stopped,
/// in the words they read.
/// Why:
///   The one failure this job causes by itself is a lock Cargo would have had to change,
/// and its remedy is the lock's own command or the clean the add would have run;
/// any other reason is the tool's,
/// and the tool's own words name it.
/// A person is told where their bytes went either way,
/// so a stopped filling never leaves them guessing whether the worktree was written.
pub(crate) fn what_to_do_next(
    entry: &PrewarmEntry,
    reason: &str,
) -> String {
    if !reason.contains(LOCK_FILE) {
        return String::from(
            "The reason above names what to fix, and the next add fills the copy.",
        );
    }
    return format!(
        "Update the lock in {} (for example `cargo update --offline --workspace` there) and the next add fills the copy, or clean the worktree's \
         own crates with `{}`, which may update the lock as your own clean would",
        entry.root.display(),
        manual_command(entry.root.as_path(), entry.members.as_slice()),
    );
}

/// What:
///  Clean the worktree's own crates out of the staging copy.
/// Why:
///   The source's own artifacts name the source's paths,
/// so they must not reach the worktree,
/// while the dependency builds hold nothing path-specific and are the reason the copy exists.
/// Both directories are this job's own,
/// which is what keeps the clean away from the workspace's live state,
/// and the clean itself is told to leave `Cargo.lock` alone,
/// which is what keeps the clean out of the worktree's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanTheCopy({ fill, entry, names }): ToolRun
/// ```
async fn clean_the_copy(
    fill: &Fill,
    entry: &PrewarmEntry,
    names: &TheNames,
) -> ToolRun {
    let command: ToolCommand = ToolCommand {
        arguments: locked_clean_arguments(entry.members.as_slice()),
        directory: entry.root.clone(),
        set: vec![
            (
                OsString::from(BUILD_DIRECTORY_VARIABLE),
                names.staging.as_os_str().to_os_string(),
            ),
            (
                OsString::from(TARGET_DIRECTORY_VARIABLE),
                names.target.as_os_str().to_os_string(),
            ),
        ],
    };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// What:
///  Ask Cargo to resolve the worktree's workspace,
/// which is what writes the `Cargo.lock` the clean itself must not write.
/// Why:
///   `cargo clean` resolves the workspace,
/// and for a lock that no longer matches its manifest that rewrite is part of the clean;
///  `--locked` keeps it out of the clean,
/// so the resolution is asked for as a step of its own.
/// The bytes it writes are Cargo's own,
/// which is what lets the caller tell Cargo's rewrite from a person's newer write
/// (`package_repair_cargo_lock.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTool(cargo, { arguments: ['metadata', '--offline', '--format-version', '1'], directory: entry.root });
/// ```
async fn resolve_the_lock(
    fill: &Fill,
    entry: &PrewarmEntry,
) -> ToolRun {
    let arguments: Vec<OsString> = ["metadata", "--offline", "--format-version", "1"]
        .iter()
        .map(OsString::from)
        .collect();
    let command: ToolCommand = ToolCommand {
        arguments,
        directory: entry.root.clone(),
        set: Vec::new(),
    };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// What:
///  Publish the staging copy under the workspace's build directory name,
/// or say that something else is already there.
/// Why:
///   The rename never replaces,
/// so a build that started while the copy was being made keeps every artifact it wrote and the
/// staging copy is the only thing removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const publish = (staging, destination) => renameNoReplace(staging, destination);
/// ```
pub(crate) fn publish(
    staging: &Path,
    destination: &Path,
) -> Result<bool, String> {
    return match rename_noreplace(staging, destination) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) if rename_unsupported(&error) => Err(format!(
            "{} cannot be published, because this filesystem cannot rename without replacing ({error})",
            destination.display()
        )),
        Err(error) => Err(format!(
            "{} could not be published ({error})",
            destination.display()
        )),
    };
}

/// What a workspace's filling came to.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Filled {
    /// The copy was published under the workspace's build directory name.
    Published,
    /// The worktree already had a build directory,
    /// so nothing was published.
    Kept,
}

/// What:
///  Fill one workspace's build directory:
/// take the names this job fills under,
/// clone the source's,
/// clean the copy,
/// publish it,
/// and give the names back.
/// Why:
///   Each workspace's state is in a directory of its own,
/// so one workspace's failure is reported and the next one still runs.
/// The names are taken before anything is written,
/// so nothing this job did not make is removed,
/// and given back whatever the filling came to,
/// so a successful one leaves no directory behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillOne({ fill, entry }): Promise<Filled>;
/// ```
pub(super) async fn fill_one(
    fill: &Fill,
    entry: &PrewarmEntry,
) -> Result<Filled, String> {
    // A destination that exists is a build that got there first: the worktree keeps everything it made. A filling of this
    //  workspace that was killed after it published left its marker and an empty staging copy beside that directory, and
    //  this job's marker is the proof those names are its own, so the next filling gives them back.
    if entry.to.exists() {
        give_back_what_a_killed_filling_left(entry);
        return Ok(Filled::Kept);
    }
    make_the_destinations_own_directory(entry)?;
    let names: TheNames = match entry.staged_in.as_ref() {
        // The add moved the worktree's own copied state aside before it answered, so there is nothing to clone: the state to
        //  clean is already there.
        Some(staging) => match staged_names(staging)? {
            Some(names) => names,
            None => return Ok(Filled::Kept),
        },
        None => names_for(entry.to.as_path())?,
    };
    let filled: Result<Filled, String> = fill_the_staging(fill, entry, &names).await;
    give_back(&names);
    return filled;
}

/// What:
///  The three steps of one workspace's filling:
/// clone the source's build directory into the staging copy,
/// clean the worktree's own crates out of the copy,
/// and publish it under the worktree's name.
/// Why:
///   A copy that could not be finished is this job's own,
/// and nothing else reads it,
/// so each step that fails removes it and says which step it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillTheStaging({ fill, entry, names }): Promise<Filled>;
/// ```
async fn fill_the_staging(
    fill: &Fill,
    entry: &PrewarmEntry,
    names: &TheNames,
) -> Result<Filled, String> {
    if entry.staged_in.is_none() {
        let Some(from) = entry.from.as_ref() else {
            return Err(String::from(
                "a filling that clones was handed no source build directory, so nothing was cleaned",
            ));
        };
        let made: Result<Cloned, String> =
            clone_tree_awaiting(from.as_path(), names.staging.as_path(), 1).await;
        if let Err(reason) = made {
            let _removed = remove_if_any(names.staging.as_path());
            return Err(format!(
                "the build state of {} could not be cloned ({reason})",
                from.display()
            ));
        }
    }
    // A build that started during the clone owns the destination now, and this job touches nothing of it.
    if entry.to.exists() {
        let _removed = remove_if_any(names.staging.as_path());
        return Ok(Filled::Kept);
    }
    // Cargo resolves the worktree's workspace for this clean even though it deletes only from directories of this job's
    //  own, and that rewrites a `Cargo.lock` that no longer matches its manifest or creates one that is missing. This job
    //  runs after the add returned, when the person may be working in the worktree, so the clean itself must never write
    //  that lock (`--locked`): a lock it refuses is resolved by Cargo first, and the checked-out lock goes back only when
    //  the file still holds exactly the bytes that resolution wrote (`package_repair_cargo_lock.rs`).
    let lock_before: Option<Vec<u8>> = read_lock(entry.root.as_path());
    let mut run: ToolRun = clean_the_copy(fill, entry, names).await;
    if !matches!(run.end, ToolEnd::Succeeded) {
        let resolved: ToolRun = resolve_the_lock(fill, entry).await;
        if matches!(resolved.end, ToolEnd::Succeeded) {
            let the_cleans_own: Option<Vec<u8>> = read_lock(entry.root.as_path());
            run = clean_the_copy(fill, entry, names).await;
            if let Some(written) = the_cleans_own.as_deref() {
                match restore_only_the_cleans_own(
                    entry.root.as_path(),
                    lock_before.as_deref(),
                    fill.kept_in.as_deref(),
                    written,
                ) {
                    None => {}
                    Some(said) => debug(TAG, format!("{}: {said}", entry.root.display()).as_str()),
                }
            }
        }
    }
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_if_any(names.staging.as_path());
        let reason: String = end_reason(&run.end);
        return Err(format!(
            "the copy of {} could not be cleaned ({reason}), so nothing was published and everything staged for it was removed. The clean itself \
             never writes the worktree's {LOCK_FILE} (`{LOCK_FLAG}`): a lock it refuses is resolved by Cargo first, and the checked-out bytes go \
             back over bytes only that resolution wrote, so a write of yours stays yours. {}",
            entry
                .from
                .as_deref()
                .unwrap_or(names.staging.as_path())
                .display(),
            what_to_do_next(entry, reason.as_str()),
        ));
    }
    return match publish(names.staging.as_path(), entry.to.as_path()) {
        Ok(true) => Ok(Filled::Published),
        Ok(false) => {
            let _removed = remove_if_any(names.staging.as_path());
            Ok(Filled::Kept)
        }
        Err(reason) => {
            let _removed = remove_if_any(names.staging.as_path());
            Err(reason)
        }
    };
}
