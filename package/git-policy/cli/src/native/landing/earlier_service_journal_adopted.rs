//! What:
//!  Finish the batches that the commit service of an earlier release left open, when the service of this release starts for
//!  the same worktree.
//! Why:
//!  The unit this release writes keeps the service journal in a state folder named after the renamed program, so a batch that
//!  the earlier helper had accepted and not finished (its branch moved and its index update lost, for example after a logout
//!  or a kill) sits in the earlier folder, where nothing of this release looks. Left there, the person's index would show the
//!  landed commit as reverted. The owner's rule is "never discard user data", so the new service finishes those batches
//!  with the recovery it already has, in place and before it accepts a connection: nothing is moved or copied, and a batch
//!  that landed stays remembered where it is. It does so only when no process is running the service of that worktree, because
//!  a running service finishes its own batches and two services must never settle one batch. The proof is a look at the
//!  running processes (`/proc`); a look that cannot be made counts as "may be running", the adoption is tried again at the
//!  next start, and nothing waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!(await serviceMayRun(instance))) await recoverInstance(earlierJournal, instance);
//! ```

/// Whether a service of the worktree may be running.
use crate::earlier_service_process_look::{ServiceLook, effective_user, service_of_instance_may_run};
/// How long that look may take.
use crate::wait_ceilings::PROCESS_LOOK_CEILING;
/// The context a recovery runs in,
///  and the recovery itself.
use crate::service_land::{Handled, LandContext, RecoveryReport, recover_instance_without_drops};
/// The log of the service.
use crate::journald_native::Priority;
use crate::service_journal::Recovery;
use crate::service_timing::log_entry;
/// The names of the folders.
use crate::wrapper_names_old_and_new::DATA_FOLDER;
/// Paths.
use crate::service_journal::landed_commits_read_only;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The name of the folder below the state folder that holds the service journals.
const SERVICE_FOLDER: &str = "service";

/// The two folders an earlier release kept journals in for one worktree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EarlierJournals {
    /// The journal of the earlier release's service for this instance.
    pub per_instance: PathBuf,
    /// The journal above every instance's own, which the single service of the first releases used.
    pub shared: PathBuf,
}

/// What:
///  Where the earlier release kept the journals that belong to the journal of this release's service,
///  or nothing when the journal is not in the place the unit gives it (a development run names any folder).
///  `Option<EarlierJournals>` is "the two folders or nothing".
/// Why:
///   The unit passes `<state>/monochromatic-git-policy-cli/service/<instance>`; the earlier unit passed
///  `<state>/cli-git/service/<instance>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function earlierJournalsOf(journal: string, instance: string): EarlierJournals | undefined;
/// ```
pub fn earlier_journals_of(journal: &Path, instance: &str) -> Option<EarlierJournals> {
    if journal.file_name()? != instance {
        return None;
    }
    let service_folder: &Path = journal.parent()?;
    if service_folder.file_name()? != SERVICE_FOLDER {
        return None;
    }
    let data_folder: &Path = service_folder.parent()?;
    if data_folder.file_name()? != DATA_FOLDER.current {
        return None;
    }
    let state: &Path = data_folder.parent()?;
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join(SERVICE_FOLDER);
    return Some(EarlierJournals { per_instance: shared.join(instance), shared });
}

/// What:
///  The request identities, each with the commit it became, that the earlier release's service finished for this worktree,
///  read from its journals (empty when the journal is not in the place the unit gives it).
///  `HashMap<String, String>` maps an identity to a commit name.
/// Why:
///   A client that the earlier helper never answered sends its request again to the service of this release; that service
///  answers a request it knows to be landed and lands nothing twice, so it must know what the earlier helper landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function earlierCommitsLanded(journal: string, instance: string): Map<string, string>;
/// ```
pub fn earlier_commits_landed(journal: &Path, instance: &str) -> HashMap<String, String> {
    let mut landed: HashMap<String, String> = HashMap::new();
    let Some(earlier) = earlier_journals_of(journal, instance) else {
        return landed;
    };
    for folder in [earlier.shared, earlier.per_instance] {
        landed.extend(landed_commits_read_only(folder.as_path()));
    }
    return landed;
}

/// What:
///  Finish the open batches of the worktree's instance in the journals the earlier release kept,
///  when no service of the instance is running.
///  `&LandContext` is the new service's context: its Git and the new journal's place.
/// Why:
///   The recovery decides each batch by the branch and the index, finishes the index update that may be missing, and marks
///  the finished batch done in the folder it found it in. It never drops the intent of a batch that looks unlanded: the look at
///  the processes can miss a live helper, and a live landing writes its intent before it moves the branch, so that intent stays
///  on disk (and is reported as left) for the earlier helper's own start-up recovery.
///  Calling it again finds nothing new to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function adoptOpenBatches(context: LandContext, instance: string, procRoot: string): RecoveryReport[];
/// ```
pub fn adopt_open_batches(context: &LandContext, instance: &str, proc_root: &Path) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let Some(earlier) = earlier_journals_of(context.journal.as_path(), instance) else {
        return reports;
    };
    if !earlier.per_instance.is_dir() && !earlier.shared.is_dir() {
        return reports;
    }
    let look: ServiceLook = ServiceLook {
        instance: instance.to_string(),
        journals: vec![earlier.per_instance.clone(), earlier.shared.clone()],
        own_pid: std::process::id(),
        own_user: effective_user(),
        ceiling: PROCESS_LOOK_CEILING,
    };
    if service_of_instance_may_run(look, proc_root) {
        crate::service_log!("the earlier service's journals {} and {} are left for now: a service of this worktree may be running, or the processes cannot be looked at", earlier.per_instance.display(), earlier.shared.display());
        return reports;
    }
    for journal in [earlier.per_instance, earlier.shared] {
        if !journal.is_dir() {
            continue;
        }
        let earlier_context: LandContext = LandContext { git: context.git.clone(), journal, on_objects_written: None, lock_ceiling: context.lock_ceiling };
        let settled: Vec<RecoveryReport> = recover_instance_without_drops(&earlier_context, instance);
        log_adopted_batches(settled.as_slice(), earlier_context.journal.as_path());
        reports.extend(settled);
    }
    return reports;
}

/// What:
///  Says in the service's log what the adoption did with each batch of one earlier journal, as the settling of the service's own
///  journal does: a Warning with the batch number in the `CLI_GIT_BATCH` field.
/// Why:
///   The batches come from a service that no longer runs or runs elsewhere; a person looking for what became of a commit has
///  only this log. A batch that stays (open, or with its intent on disk) says why and who settles it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const report of reports) log(Warning, `${verb(report.handled)} batch=${report.seq} journal=${journal}`, { CLI_GIT_BATCH: report.seq })
/// ```
fn log_adopted_batches(reports: &[RecoveryReport], journal: &Path) {
    for report in reports {
        let what: &str = match report.handled {
            Handled::Finished => "finished the batch the earlier service left open",
            Handled::StillOpen => "left open the batch the earlier service left open, which could not be finished now (the index lock is held or a record cannot be read); the next start tries again",
            Handled::Left => "left on disk the intent of a batch the earlier service wrote: the branch shows it as not landed, and a landing in progress writes its intent first, so the earlier service's own start settles it",
            Handled::Dropped => "dropped the intent of a batch the earlier service wrote",
        };
        log_entry(
            Priority::Warning,
            format!("{what} batch={} decision={} journal={}", report.seq, if report.decision == Recovery::Finish { "finish" } else { "drop" }, journal.display()).as_str(),
            &[("CLI_GIT_BATCH", report.seq.to_string())],
        );
    }
}
