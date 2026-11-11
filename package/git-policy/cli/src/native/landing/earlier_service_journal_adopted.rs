//! What:
//!  Finish,
//! when the person asks,
//! the batches that the commit service of an earlier release left open for a worktree.
//! Why:
//!  The unit this release writes keeps the service journal in a state folder named after the renamed program,
//! so a batch that
//!  the earlier helper had accepted and not finished (its branch moved and its index update lost,
//! for example after a logout
//!  or a kill) sits in the earlier folder (`<state>/cli-git/service/<instance>` and the shared `<state>/cli-git/service`),
//! where
//!  nothing of this release looks on its own.
//! Left there,
//! the person's index would show the landed commit as reverted.
//! The
//!  owner's rule is "never discard user data",
//! so
//!  `git monochromatic-git-policy-cli service settle-earlier-batches` finishes those batches with the recovery the service
//!  already has,
//! in place:
//! nothing is moved or copied,
//! a batch that landed stays remembered where it is,
//! and a batch that looks
//!  as if it never landed keeps its intent on disk (a landing in progress writes its intent first).
//! It does so only when the
//!  earlier helper can neither be running nor start,
//! because a running service finishes its own batches and two services must
//!  never settle one batch.
//! Why not automatically:
//!  The earlier helper takes no lock on its journal (tag `monochromatic-git-policy-cli-v0.5.34`,
//! `service_main.rs`:
//! it creates
//!  the folder,
//! binds,
//! recovers and serves),
//! so there is no exclusion this program could also take;
//! the only gate on its
//!  start is the manager's socket activation.
//! The proof is a look (`earlier_service_process_look.rs`):
//! no socket of the earlier
//!  units listens (a connection to one starts the helper at any moment) and no process has the start arguments of the
//!  worktree's service;
//! a look that cannot be made counts as "may run".
//! A wrapper of the earlier release that runs in the same
//!  worktree writes its units again and starts its socket,
//! which can start the earlier helper between the look and the end of
//!  the recovery,
//! and nothing stops that:
//! the fifth and sixth cross-vendor reviews (2026-11-07) rated it a defect.
//! The guard
//!  systemd would enforce,
//! a mask of the earlier unit,
//! makes that wrapper fail its commit (tag `service_provision.rs` line 453
//!  starts the socket and line 435 returns the refusal;
//! `service_commit.rs` lines 386 to 388 stop the commit with it;
//!  `commit_command.rs` lines 105 to 108 exit 1),
//! so it was not built.
//! The owner picked "Off,
//! settle on request" (2026-11-07):
//!  the service never runs this,
//! no unit file and no start option turns it on,
//! and the command says plainly,
//! before it acts,
//!  that it is safe only when no wrapper of the earlier release is in use in that worktree at that moment.
//! What the provisioning does about the earlier socket:
//!  When a commit sets up this release's service for a worktree,
//! it also stops the earlier release's socket unit of the same
//!  worktree (`cli-git@<instance>.socket`,
//! the same instance text;
//! owner,
//! 2026-11-07,
//! "Stop the old socket"),
//! so that nothing
//!  starts the earlier helper by a connection (`service_provision.rs`,
//! `stop_earlier_instance_socket`).
//! A helper of the earlier
//!  release that is still running is left as it is,
//! with its socket:
//! its service `Requires` the socket,
//! so a stop of the socket
//!  in the usual job mode would stop it mid-batch (measured with throwaway units).
//! The socket keeps listening,
//! so the look keeps
//!  holding;
//! the helper finishes its batches and exits when idle (three hours after its last client,
//! `IDLE_EXIT` at the tag),
//!  and the next commit that sets the service up stops the socket.
//! The socket file is never unlinked:
//! an earlier wrapper can
//!  make its unit listen again at that path at any moment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!(await earlierServiceHold(instance))) await recoverInstance(earlierJournal, instance);
//! ```

use crate::earlier_service_process_look::{Holding, ServiceLook, earlier_service_hold, effective_user};
use crate::earlier_service_unsure_program::UnsureProgram;
/// How long that look may take.
use crate::wait_ceilings::PROCESS_LOOK_CEILING;
/// The context a recovery runs in,
///  and the recovery itself.
use crate::service_land::{Handled, LandContext, RecoveryReport, recover_instance_without_drops};
use crate::service_instance::instance_of;
use crate::transaction_git::GitContext;
/// The log of the service.
use crate::journald_native::Priority;
use crate::service_journal::{OPEN_EXTENSION, Recovery, batch_path, list_intents_read_only};
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
    /// The journal above every instance's own,
    /// which the single service of the first releases used.
    pub shared: PathBuf,
}

/// What:
///  Where the earlier release kept the journals that belong to the journal of this release's service,
///  or nothing when the journal is not in the place the unit gives it (a development run names any folder).
///  `Option<EarlierJournals>` is "the two folders or nothing".
/// Why:
///   The unit passes `<state>/monochromatic-git-policy-cli/service/<instance>`;
/// the earlier unit passed
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
    return Some(earlier_journals_in_state(data_folder.parent()?, instance));
}

/// The earlier journals of an instance below a state folder (`<state>/cli-git/service/<instance>` and `<state>/cli-git/service`).
pub fn earlier_journals_in_state(state: &Path, instance: &str) -> EarlierJournals {
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join(SERVICE_FOLDER);
    return EarlierJournals { per_instance: shared.join(instance), shared };
}

/// What:
///  How many batches the earlier journals of a worktree hold open,
/// counted without changing a file of them.
/// Why:
///   The count decides whether anything needs looking at,
/// and the doctor reports it.
/// A batch is the worktree's when the Git
///  directory its intent records names this instance,
/// which is how the recovery selects it too;
/// a file that cannot be read is
///  nobody's.
/// The journals may belong to a helper that is alive,
/// so nothing in them is tidied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openBatchCount(earlier: EarlierJournals, instance: string): number;
/// ```
pub fn open_batch_count(earlier: &EarlierJournals, instance: &str) -> usize {
    return open_batches_of(earlier.per_instance.as_path(), instance).len() + open_batches_of(earlier.shared.as_path(), instance).len();
}

/// The numbers of the open batches of the worktree that one journal holds,
/// read without changing a file.
fn open_batches_of(journal: &Path, instance: &str) -> Vec<u64> {
    let mut seqs: Vec<u64> = Vec::new();
    for (_path, parsed) in list_intents_read_only(journal, OPEN_EXTENSION) {
        if let Some(intent) = parsed
            && instance_of(intent.git_dir.as_path()) == instance
        {
            seqs.push(intent.seq);
        }
    }
    return seqs;
}

/// One batch an on-request settling dealt with,
/// and the journal it was found in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettledBatch {
    /// The journal the batch is in.
    pub journal: PathBuf,
    /// What was decided and done.
    pub report: RecoveryReport,
}

/// An open batch of the worktree that the recovery could not examine,
/// with the journal it is in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnansweredBatch {
    /// The journal the batch is in.
    pub journal: PathBuf,
    /// The batch.
    pub seq: u64,
}

/// What a settling on request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestedSettlement {
    /// No batch of the worktree is open in the earlier journals;
    /// nothing was looked at.
    NothingOpen,
    /// A batch is open and an earlier helper may run or start,
    /// so nothing was settled.
    Held {
        /// How many batches are open.
        open: usize,
        /// Why nothing was settled,
        /// with the program behind it when a program that cannot be ruled out is.
        because: Holding,
    },
    /// The batches were dealt with by the finish-only recovery,
    /// and the batches it could not examine are named.
    Settled {
        /// The batches the recovery dealt with.
        batches: Vec<SettledBatch>,
        /// The open batches of the worktree that Git gave no answer for;
        /// their intents stay on disk.
        unanswered: Vec<UnansweredBatch>,
    },
}

/// The look at the sockets and processes of the earlier helper of an instance,
/// for journals in `earlier`.
pub fn hold_on_earlier_helper(earlier: &EarlierJournals, instance: &str, proc_root: &Path) -> Option<Holding> {
    let look: ServiceLook = ServiceLook {
        instance: instance.to_string(),
        journals: vec![earlier.per_instance.clone(), earlier.shared.clone()],
        own_pid: std::process::id(),
        own_user: effective_user(),
        ceiling: PROCESS_LOOK_CEILING,
    };
    return earlier_service_hold(look, proc_root);
}

/// The log text that names the program behind a hold:
/// its process,
/// the program,
/// the journal folder it names and why the folder was not resolved.
fn program_note(program: &UnsureProgram) -> String {
    return format!(" (process {}, program `{}`, journal folder `{}`: {}; its other arguments are not logged)", program.pid, program.program, program.journal_word, program.reason);
}

/// Whether the recovery's reports include the batch `seq`.
fn was_reported(reports: &[RecoveryReport], seq: u64) -> bool {
    for report in reports {
        if report.seq == seq {
            return true;
        }
    }
    return false;
}

/// What:
///  Settles the open batches of the worktree's earlier journals when the person asks,
/// if no earlier helper can be running or
///  start.
/// `RequestedSettlement` says which of nothing open,
/// held back (with the reason) and settled happened.
/// Why:
///   The recovery decides each batch by the branch and the index,
/// finishes the index update that may be missing,
/// and marks the
///  finished batch done in the folder it found it in.
/// It never drops the intent of a batch that looks unlanded:
/// the look can
///  miss a live helper,
/// and a live landing writes its intent before it moves the branch,
/// so that intent stays on disk (and is
///  reported as left) for the earlier helper's own start-up recovery.
/// Each batch is logged.
/// Calling it again finds nothing new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleOnRequest(git, earlier, instance, procRoot, lockCeiling): RequestedSettlement;
/// ```
pub fn settle_on_request(git: &GitContext, earlier: &EarlierJournals, instance: &str, proc_root: &Path, lock_ceiling: std::time::Duration) -> RequestedSettlement {
    let open: usize = open_batch_count(earlier, instance);
    if open == 0 {
        return RequestedSettlement::NothingOpen;
    }
    if let Some(because) = hold_on_earlier_helper(earlier, instance, proc_root) {
        crate::service_log!(
            "the earlier service's journals {} and {} were not settled, because {}{}; {open} open batch(es) stay where they are",
            earlier.per_instance.display(),
            earlier.shared.display(),
            because.because.words(),
            because.program.as_ref().map_or_else(String::new, program_note)
        );
        return RequestedSettlement::Held { open, because };
    }
    let mut settled: Vec<SettledBatch> = Vec::new();
    let mut unanswered: Vec<UnansweredBatch> = Vec::new();
    for journal in [earlier.per_instance.clone(), earlier.shared.clone()] {
        if !journal.is_dir() {
            continue;
        }
        let selected: Vec<u64> = open_batches_of(journal.as_path(), instance);
        let earlier_context: LandContext = LandContext { git: git.clone(), journal: journal.clone(), on_objects_written: None, lock_ceiling };
        let reports: Vec<RecoveryReport> = recover_instance_without_drops(&earlier_context, instance);
        log_adopted_batches(reports.as_slice(), journal.as_path());
        // The recovery leaves out a batch Git gave no answer for. It is named, with its intent still on disk, so the person is
        //  never told that everything was dealt with when a batch was not even examined.
        for seq in selected {
            let reported: bool = was_reported(reports.as_slice(), seq);
            if !reported && batch_path(journal.as_path(), seq, OPEN_EXTENSION).exists() {
                unanswered.push(UnansweredBatch { journal: journal.clone(), seq });
            }
        }
        for report in reports {
            settled.push(SettledBatch { journal: journal.clone(), report });
        }
    }
    return RequestedSettlement::Settled { batches: settled, unanswered };
}

/// What:
///  The request identities,
/// each with the commit it became,
/// that the earlier release's service finished for this worktree,
///  read from its journals (empty when the journal is not in the place the unit gives it).
///  `HashMap<String, String>` maps an identity to a commit name.
/// Why:
///   A client that the earlier helper never answered sends its request again to the service of this release;
/// that service
///  answers a request it knows to be landed and lands nothing twice,
/// so it must know what the earlier helper landed.
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
///  Says in the service's log what the adoption did with each batch of one earlier journal,
/// as the settling of the service's own
///  journal does:
/// a Warning with the batch number in the `CLI_GIT_BATCH` field.
/// Why:
///   The batches come from a service that no longer runs or runs elsewhere;
/// a person looking for what became of a commit has
///  only this log.
/// A batch that stays (open,
/// or with its intent on disk) says why and who settles it.
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
            Handled::Held => "left the batch the earlier service wrote held behind a reference lock: its branch has not moved and the next command that finds the lock gone moves it",
        };
        log_entry(
            Priority::Warning,
            format!(
                "{what} {} batch={} decision={} journal={}",
                crate::service_timing::in_the_repository_at(report.repository.as_path()),
                report.seq,
                if report.decision == Recovery::Finish { "finish" } else { "drop" },
                journal.display()
            )
            .as_str(),
            &[("CLI_GIT_BATCH", report.seq.to_string())],
        );
    }
}
