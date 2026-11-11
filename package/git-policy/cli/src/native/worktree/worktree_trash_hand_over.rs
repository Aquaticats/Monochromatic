//! What:
//!  Hand the directories a snapshot's leave-out set aside to the commit service,
//! as one durable work item,
//!  and wake the service,
//! without waiting for anything to be emptied.
//! Why:
//!  The leave-out renames the other checkouts,
//! the caches and the run records out of the new worktree in
//!  about a second,
//! but what it set aside is hundreds of thousands of entries,
//! and every `unlink` of them starts a btrfs
//!  transaction.
//! A trash emptied inside the add made ten concurrent adds spend more than ten minutes each on
//  2026-11-10,
//! with 2,182 of 2,796 blocked samples inside `unlink` or `rmdir` and the drive 99% busy
//!  (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`).
//! So the add writes the record,
//! tells the service,
//! and returns.
//! The record lives in the
//!  repository's common work directory rather than in the new worktree's,
//! because that is what makes one repository empty its trashes
//!  one at a time:
//! a round drains one lane per repository,
//! and the drain of one work directory empties its trashes in
//!  sequence (`service_work_drain_pending_claims.rs`),
//! so concurrent adds enqueue beside each other and never delete beside
//!  each other.
//! The claim the drain takes is the lease every other kind of work already uses,
//! so no new lock
//!  appears for this.
//! A record that outlives the command is the whole point:
//! a killed add or a stopped service leaves
//!  the trash named,
//! the next run of the repository's service empties it,
//! and a trash that cannot be emptied is reported
//!  and named by every later command in the worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await handOverTrash({ context, trash, reach }); // enqueue the trash item in the common work dir; provision(); wake(commonDir)
//! ```

use crate::diagnostic_log::debug;
use crate::iso_time::now_iso;
use crate::service_client::{Activation, ClientConfig, START_WAIT, manager_watch_for, wake};
use crate::service_messages::{ClientFailure, render};
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::{ProvisionHost, provision};
use crate::service_provision_host_and_manager::units_record_dir_of;
use crate::service_route::{Route, route_from_environment};
use crate::service_work::work_directory;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{ANSWER_CEILING, SHORT_EXTRA_WAIT, START_CEILING, WITHDRAW_CEILING, waiting_notice_after};
use crate::work_item::{PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::worktree_trash::{PendingTrash, TRASH_PARENT, TRASH_REFERENCE};
use std::io::Write;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashHandOver";

/// How the service's address is found for a Git directory;
/// injected so a control never reaches the real service.
pub type Router<'a> = &'a dyn Fn(&Path, &Path) -> Option<Route>;

/// Sets up what the service needs before it can be reached,
/// as a commit does for its first message;
/// injected so a control
///  never reaches the session bus.
pub type Provisioner<'a> = &'a dyn Fn(&ProvisionHost) -> Result<(), ClientFailure>;

/// What:
///  How the service is reached:
/// where its address is found,
/// how it is set up,
/// and how long its start is waited for.
/// Why:
///   A control must be able to hand a trash over with no session bus and no service,
/// which is the state of a machine
///  whose units are not installed,
/// and the wait for a start is the one bound this step keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reach = { router: (git: string, gitDir: string) => Route | undefined; provisioner: (host: ProvisionHost) => void; startWait: number };
/// ```
pub struct Reach<'a> {
    /// Finds the service's address for a Git directory.
    pub router: Router<'a>,
    /// Sets up the repository's socket activation,
    /// which a repository whose units were never installed does not have.
    pub provisioner: Provisioner<'a>,
    /// The longest wait for the service to start.
    pub start_wait: Duration,
}

/// What:
///  Hand this add's trash to the commit service of the repository,
/// and answer the reason it stays when the service
///  could not be told about it.
/// Why:
///   The production reach is the one a commit uses,
/// so a trash is handed over exactly as a push record is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, trash, notices });
/// ```
pub fn hand_over_to_the_service(context: &GitContext, trash: &PendingTrash, notices: &mut dyn Write) -> Result<(), String> {
    return hand_over_with(context, trash, notices, &Reach { router: &route_from_environment, provisioner: &provision, start_wait: START_WAIT });
}

/// What:
///  The record of one trash:
/// the worktree that holds it,
/// the repository's common directory whose service empties it,
/// and what the
///  leave-out made and moved.
/// Why:
///   The record and the wake name the same directory,
/// so the two must not be built in two places;
/// the path goes in as
///  its bytes as well as its text (`path_bytes_in_records.rs`),
/// so a worktree whose path is not text is named like any
///  other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashItem = (trash) => ({ kind: 'trash', repository: trash.destination, gitDir: trash.commonDir, ref: TRASH_REFERENCE, oid: trash.manifest.name, parent: encodeManifest(trash.manifest) });
/// ```
fn trash_item(trash: &PendingTrash) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Trash,
        repository: trash.destination.clone(),
        git_dir: trash.common_dir.clone(),
        branch_ref: String::from(TRASH_REFERENCE),
        oid: trash.manifest.name.clone(),
        parent: trash.manifest.encode(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  Write this trash as a durable work item now,
/// and answer the reason it could not be written.
/// Why:
///   The record is what outlives the command,
/// so it is written the moment its trash stands in the published worktree,
///  before the add does anything else:
/// a command killed after that leaves a record a service empties,
/// where a manifest that
///  lived only in the add's memory left a trash nothing would ever empty.
/// Writing it twice is one record:
/// the queue's id
///  is the trash's name,
/// so a second write finds the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordTheTrash(trash) // enqueue(workDir(trash.commonDir), trashItem(trash))
/// ```
pub fn record_the_trash(trash: &PendingTrash) -> Result<(), String> {
    return match enqueue(work_directory(trash.common_dir.as_path()).as_path(), &trash_item(trash)) {
        Ok(_added) => Ok(()),
        Err(reason) => Err(reason),
    };
}

/// What:
///  What the provisioner answered for the host this repository needs,
/// or nothing when the repository needs no setup at all.
/// Why:
///   The answer is asked before the service is woken,
/// so a setup that fails ends the handover with the reason the person can act on,
/// and a repository whose units are already installed pays nothing for it.
fn the_provisioners_answer(provisioner: Provisioner<'_>, host: Option<&ProvisionHost>) -> Option<Result<(), ClientFailure>> {
    let setup: &ProvisionHost = host?;
    return Some(provisioner(setup));
}

/// What:
///  `hand_over_to_the_service` with the route finder and the provisioner injected,
/// so a control reaches no
///  service and no session bus.
/// Why:
///   The item is written before the service is reached,
/// so a failed wake leaves the trash named rather than
///  anonymous,
/// and the reason in the answer says so to the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, trash, notices, reach });
/// ```
pub fn hand_over_with(context: &GitContext, trash: &PendingTrash, notices: &mut dyn Write, reach: &Reach<'_>) -> Result<(), String> {
    if let Err(reason) = record_the_trash(trash) {
        debug(TAG, format!("the work item could not be written: {reason}").as_str());
        return Err(stays_words(trash, format!("the work item could not be written: {reason}").as_str(), None));
    }
    let record: PathBuf = state_directory(work_directory(trash.common_dir.as_path()).as_path(), PENDING).join(trash_item(trash).id());
    let told: Result<(), String> = match (reach.router)(context.real_git.as_path(), trash.common_dir.as_path()) {
        // The record stays for the next wake a command of this repository gives, which is every commit and every add.
        None => Err(String::from("no commit service can be started from this session")),
        Some(Route { address, provision: host }) => match the_provisioners_answer(reach.provisioner, host.as_ref()) {
            Some(Err(failure)) => Err(render(&failure)),
            _ => {
                let config: ClientConfig = ClientConfig {
                    address,
                    activation: Activation::Managed,
                    start_wait: reach.start_wait,
                    heartbeat_silence: HEARTBEAT_SILENCE,
                    start_watch: host.as_ref().map(manager_watch_for),
                    keep_fresh: None,
                    answer_ceiling: ANSWER_CEILING,
                    withdraw_ceiling: WITHDRAW_CEILING,
                    waiting_notice_after: waiting_notice_after(),
                    extra_wait: SHORT_EXTRA_WAIT,
                    start_ceiling: START_CEILING,
                    units_record_dir: host.as_ref().map(units_record_dir_of),
                };
                // The wake's own lines are not the add's report: the add says what became of the trash once.
                let mut silent: std::io::Sink = std::io::sink();
                match wake(&config, trash.common_dir.as_path(), None, &mut silent) {
                    Ok(()) => Ok(()),
                    Err(failure) => Err(render(&failure)),
                }
            }
        },
    };
    return match told {
        Ok(()) => {
            // A routine hand-over says nothing: every add hands over the stand-ins and the `.git` copy it made itself, so the
            //  line is kept for the one trash nothing of this add would ever have emptied.
            if !trash.quiet && trash.taken_over > 0 {
                let _told = writeln!(notices, "{}", handed_words(trash));
            }
            Ok(())
        }
        // The record stays and the service it names runs when the repository is next worked on, so this is a report and never
        //  the add's failure: the worktree is whole either way.
        Err(why) => Err(stays_words(trash, why.as_str(), Some(record.as_path()))),
    };
}


/// What:
///  The line that tells the person about directories the hand-over took over from trash an earlier add left.
/// Why:
///   Every add hands the snapshot's stand-ins and its own `.git` copy over,
/// so a line about the add's own trash
///  would print on every add and tell nobody anything.
/// What the person cannot learn anywhere else is a trash an earlier add
///  left:
/// nothing of this add would ever have emptied it,
/// and the line names it,
/// says where its directories went,
/// and
///  who empties them now.
/// One directory is said in the singular,
/// and every verb and pronoun agrees with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handedWords = (trash) => { const n = trash.manifest.entries.length; const one = n === 1; return `... took over ${trash.takenOver === 1 ? 'one directory' : `${trash.takenOver} directories`} an earlier add left ... The ${n} ${one ? 'directory' : 'directories'} left out of the new worktree ${trash.destination} ${one ? 'was' : 'were'} handed ..., which empties ${one ? 'it' : 'them'} ...`; };
/// ```
fn handed_words(trash: &PendingTrash) -> String {
    let count: usize = trash.manifest.entries.len();
    let (directories, verb, pronoun): (&str, &str, &str) = match count {
        1 => ("directory", "was", "it"),
        _ => ("directories", "were", "them"),
    };
    let taken_over: String = match trash.taken_over {
        1 => String::from("one directory"),
        many => format!("{many} directories"),
    };
    return format!(
        "monochromatic-git-policy-cli: the new worktree {} took over {taken_over} that an earlier add left in the trash of the source. The {count} {directories} left out of the new worktree {} {verb} handed to the commit service, which empties {pronoun} after this command returns, one trash at a time for this repository and at the lowest I/O priority; the add did not wait for that.",
        trash.destination.display(),
        trash.destination.display()
    );
}

/// What:
///  What stays when the service could not be told,
///  named with the reason,
///  the trash and the record that holds it.
/// Why:
///   The trash outlives the command in that case,
/// so the person is told which directory to look at and why nothing empties
///  it now;
/// a record that was written means the service empties it when it next runs for the repository,
///  and no record means only the person will.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staysWords = (trash, why, record) => record === undefined ? `... no record names them, so only you will remove them` : `... the record ${record} names them, so the commit service empties them when it next runs for this repository`;
/// ```
fn stays_words(trash: &PendingTrash, why: &str, record: Option<&Path>) -> String {
    let trash_shown: PathBuf = trash.destination.join(TRASH_PARENT).join(&trash.manifest.name);
    let quoted: String = crate::shell_quote::shell_quote(trash_shown.as_os_str().as_bytes());
    let next: String = match record {
        Some(path) => format!("the record {} names them, so the commit service empties them when it next runs for this repository", path.display()),
        None => String::from("no record names them, so only you will remove them"),
    };
    return format!(
        "the directories left out of the new worktree {} were not handed to the commit service ({why}); the trash {quoted} stays inside the worktree, {next}, and every later command in this worktree names it and deletes nothing from it.",
        trash.destination.display()
    );
}

/// The hand-over's controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_hand_over_tests.rs"]
mod tests;
