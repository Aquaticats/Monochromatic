//! What:
//!  The synchronization of one created worktree with its source's ignored state:
//! the stage of that state,
//! its journal,
//!  the installation into the worktree the command registered,
//! and the notice naming what the copy left out.
//! Why:
//!  The destination of a copy is a path,
//! and the registration behind a path can be removed and replaced by another command
//!  between the moment real Git returned and the moment the first entry is written.
//! The binding the creating command proved is
//!  carried beside the staged payload and checked before staging and again before the first entry,
//! so a copy only ever writes into
//!  the worktree it was made for.
//! A command that ends its copy before it is whole leaves the destination to the person,
//!  who is told what to do there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copied = await synchronizeCreatedWorktreeNoting({ commonDir, sourceRoot, destinationRoot, binding, recorded, selection, notice });
//! ```

use crate::diagnostic_log::debug;
use crate::json_record::quote;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::Binding;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure_of};
use crate::worktree_copy_journal::create_journal;
use crate::worktree_copy_journal_encoding_and_validation::PendingJournal;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{StagedSnapshot, stage_ignored_snapshot};
use crate::worktree_copy_stage_binding::{record_stage_binding, verify_stage_binding};
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage};
use crate::worktree_copy::{Invocation, summary_line};
use crate::worktree_copy_registry::{CreatedWorktrees, WorktreeCopyObservation, find_created_worktree};
use crate::worktree_copy_transaction::{Outcome, SnapshotSource, complete_journal};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  The refusal of a copy whose destination is no longer the registration it was made for.
/// Why:
///   Nothing in a worktree is overwritten or removed,
/// so the copy stops before the first entry,
/// and the person gets both registrations and the commands that show one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`monochromatic-git-policy-cli: the ignored state for ${destination} was not copied: ${reason}. ...`)
/// ```
fn registration_changed(destination: &Path, reason: &str) -> WorktreeCopyError {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let message: String = format!(
        "monochromatic-git-policy-cli: the ignored state for {shown} was not copied: {reason}. Nothing in that worktree was changed; `git worktree list` shows what is registered there now, and a `git worktree add` into the same path copies the ignored state afresh."
    );
    return crate::worktree_copy_error::failure_with_cause(message.as_str(), None);
}

/// What:
///  Stages,
/// journals and installs the ignored state of one newly created worktree.
/// Why:
///   The stage is removed if the journal cannot be created,
/// and a process that dies while staging leaves a claim,
///  so no stage survives without a record that recovery can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktree({ commonDir, sourceRoot, destinationRoot, registeredRoots, gitPath }): Promise<number>;
/// ```
pub fn synchronize_created_worktree(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<usize, WorktreeCopyError> {
    return synchronize_created_worktree_noting(
        context,
        common_dir,
        source_root,
        destination_root,
        registered_roots,
        selection,
        None,
        None,
        &mut std::io::sink(),
    );
}

/// How many unreadable paths the notice names before it says how many more there are.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const NOTICE_PATHS: usize = 3;

/// What:
///  The one notice line for ignored entries the user could not read and the copy therefore left out.
/// Why:
///   The new worktree lacks them,
/// so the person sees how many and which first ones,
/// without a flood for a large set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: left out ${count} unreadable ignored entries from ${source}: ${first.join(', ')}${more}.\n`
/// ```
fn unreadable_notice(source_root: &Path, unreadable: &[String]) -> String {
    let named: Vec<String> = unreadable.iter().take(NOTICE_PATHS).map(String::as_str).map(quote).collect();
    let rest: usize = unreadable.len().saturating_sub(NOTICE_PATHS);
    let more: String = if rest == 0 { String::new() } else { format!(" and {rest} more") };
    let noun: &str = if unreadable.len() == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: left out {} unreadable ignored {noun} from {}: {}{more}.\n",
        unreadable.len(),
        quote(source_root.to_string_lossy().as_ref()),
        named.join(", ")
    );
}

/// What:
///  Stages one source's ignored state,
/// installs it into one created worktree,
/// and writes a notice naming the
///  entries the user could not read.
/// Why:
///   The notice goes to the writer the lifecycle holds for the person;
/// the count of copied entries is the result.
/// A `recorded` claim is the record the creating command wrote before real Git ran,
/// which names the stage
///  this copy fills;
/// without one the claim is made here,
/// after Git,
/// so an interruption before it is unrecorded.
/// The `binding` is the registration real Git made for the destination;
/// every write is checked against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktreeNoting({ commonDir, sourceRoot, destinationRoot, registeredRoots, selection, recorded, binding, notice }): Promise<number>;
/// ```
pub fn synchronize_created_worktree_noting(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
    recorded: Option<StageClaim>,
    binding: Option<&Binding>,
    notice: &mut dyn std::io::Write,
) -> Result<usize, WorktreeCopyError> {
    let started: std::time::Instant = std::time::Instant::now();
    let claim: StageClaim = match recorded {
        Some(written) => written,
        None => claim_stage(common_dir, destination_root)?,
    };
    // The registration is checked before the stage costs anything, so a worktree that was removed or replaced meanwhile
    // stops the copy before a byte is staged for it.
    if let Some(bound) = binding
        && let Err(reason) = verify_stage_binding(common_dir, bound)
    {
        claim.settle()?;
        return Err(registration_changed(destination_root, reason.as_str()));
    }
    // `match` on staging: a failure has already removed its container, so only the claim is left to settle.
    let snapshot: StagedSnapshot = match stage_ignored_snapshot(
        context,
        source_root,
        claim.container(),
        registered_roots,
        selection,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            claim.settle()?;
            return Err(error);
        }
    };
    // The registration travels beside the payload, so a recovery checks the pair the copy was made for and not the path
    // alone.
    if let Some(bound) = binding {
        record_stage_binding(claim.container(), bound)?;
    }
    let staged_done: std::time::Duration = started.elapsed();
    let journaling: std::time::Instant = std::time::Instant::now();
    let pending: PendingJournal = match create_journal(common_dir, destination_root, &snapshot) {
        Ok(pending) => pending,
        Err(error) => {
            // A failed removal replaces the journal failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(snapshot.stage_container.as_path())
                .map_err(raw_failure_of)?;
            claim.settle()?;
            return Err(error);
        }
    };
    debug(
        "worktreeCopyTransaction",
        format!("journal created in {} ms after the stage at {} ms", journaling.elapsed().as_millis(), staged_done.as_millis()).as_str(),
    );
    // The journal names the stage from here on, so the claim has done its work.
    claim.release()?;
    // The last check before the first entry is written: staging takes minutes, and the registration it was staged for
    // can be gone by now. The journal and stage stay pending, so the next command's recovery reports them.
    if let Some(bound) = binding
        && let Err(reason) = verify_stage_binding(common_dir, bound)
    {
        return Err(registration_changed(destination_root, reason.as_str()));
    }
    let staged_after: std::time::Duration = started.elapsed();
    let staged: StagedSnapshot = snapshot.clone();
    let outcome: Outcome = complete_journal(pending, &SnapshotSource::Staged(&staged))?;
    debug(
        "worktreeCopyTransaction",
        format!(
            "settled {} entries in {} ms: stage and journal {} ms, install and completion {} ms",
            snapshot.entries.len(),
            started.elapsed().as_millis(),
            staged_after.as_millis(),
            (started.elapsed() - staged_after).as_millis()
        )
        .as_str(),
    );
    return match outcome {
        Outcome::Installed(copied) => {
            if !snapshot.unreadable.is_empty() {
                // A closed standard error leaves nowhere to print; the log line of the stage still names the entries.
                let _written = notice.write_all(unreadable_notice(source_root, snapshot.unreadable.as_slice()).as_bytes());
            }
            Ok(copied)
        }
        Outcome::Ended(error) => Err(error),
    };
}

/// The line a successful `git worktree add` prints when its command line has a form whose destination is not read.
pub(crate) const UNKNOWN_DESTINATION_NOTICE: &str = "monochromatic-git-policy-cli: this `git worktree add` names its worktree in a form monochromatic-git-policy-cli does not read, so the worktree received no ignored-state copy. Name the path as a plain operand (`git worktree add [-b <branch>] <path> [<commit-ish>]`, with `-C <directory>` or `-c <name>=<value>` before `worktree`) to have it copied.\n";

/// What:
///  The line for a destination Git made whose registration could not be confirmed.
/// Why:
///   A worktree that gets no copy must say so;
/// here the pointer in its `.git` file and the registration it names do not
///  name each other,
/// so monochromatic-git-policy-cli cannot tell the worktree is the command's own and copies nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} was made by git worktree add, but ...`
/// ```
fn unconfirmed_destination_notice(destination: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} was made, but its registration could not be confirmed as this command's, so it received no ignored-state copy. `git worktree list` shows whether Git registered it.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// What:
///  Synchronizes the worktree this command registered at its destination and writes the summary line for it;
/// the root of
///  the created worktree,
/// a snapshot's included,
/// comes back for the package-manager repair.
/// Why:
///   The destination is the one the command line named,
/// so a worktree another add registered at the same time is never
///  copied into here;
/// a failure stops the copy with no summary written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = await findCreatedWorktree(...); const copied = await synchronizeCreatedWorktree(...); process.stderr.write(renderSummary(copied));
/// ```
pub(crate) fn synchronize_after(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    registered_before: bool,
    git_succeeded: bool,
    carried: Option<&Path>,
    recorded: &mut Option<StageClaim>,
    stderr: &mut dyn Write,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    // A main checkout that names no path hands over nothing: no stage, no journal and no summary line.
    if matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty()) {
        return Ok(carried.map(Path::to_path_buf).into_iter().collect());
    }
    let Some(destination) = invocation.destination.as_deref() else {
        stderr
            .write_all(UNKNOWN_DESTINATION_NOTICE.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    };
    let mut found: CreatedWorktrees =
        find_created_worktree(&invocation.context, observation, destination, registered_before)?;
    // A destination Git made but whose registration cannot be confirmed is told, not skipped without a word.
    if found.created.is_empty()
        && git_succeeded
        && !registered_before
        && std::fs::symlink_metadata(destination.join(".git")).is_ok()
    {
        let line: String = unconfirmed_destination_notice(destination);
        stderr
            .write_all(line.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    let mut created_roots: Vec<PathBuf> = Vec::new();
    for worktree in &found.created {
        created_roots.push(worktree.root.clone());
    }
    // A destination that received the source's ignored state with a snapshot needs no copy.
    let mut kept = Vec::new();
    for worktree in found.created {
        if carried != Some(worktree.root.as_path()) {
            kept.push(worktree);
        }
    }
    found.created = kept;
    if found.created.is_empty() {
        return Ok(created_roots);
    }
    let mut copied: usize = 0;
    // The record made before real Git names the stage this copy fills, so the copy keeps the one record it has.
    let mut held: Option<StageClaim> = recorded.take();
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            for created in &found.created {
                copied += synchronize_created_worktree_noting(
                    &invocation.context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    created.root.as_path(),
                    found.registered_roots.as_slice(),
                    &observation.selection,
                    held.take(),
                    Some(&created.binding),
                    stderr,
                )?;
            }
        }
    }
    // A copy that did not happen leaves the record to the caller, which settles it.
    *recorded = held;
    let line: String = summary_line(
        copied,
        found.created.len(),
        observation.source_root.as_deref(),
    );
    stderr
        .write_all(line.as_bytes())
        .map_err(raw_failure_of)?;
    return Ok(created_roots);
}

/// Synchronization controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_synchronize_tests.rs"]
mod tests;
