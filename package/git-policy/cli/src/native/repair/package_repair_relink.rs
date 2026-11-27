//! What:
//!  Hand the relinking of a new worktree's copied package state to the commit service instead of
//!  waiting for it inside the add.
//! Why:
//!  pnpm and mpm-shadow both rewrite state that a copy left naming another location:
//! pnpm's `node_modules` for the new depth,
//! and the toolchain launchers for the new checkout.
//! A traced `git worktree add` of the installed wrapper spent 49.6 s in `pnpm install` and 11.8 s in
//!  `mpm-shadow install`
//! (`doc/planning/cli-git-worktree-add-package-repair.md`),
//! and the second of them started `pnpm list --global` eleven times.
//! Neither run needs the worktree's files:
//! both rewrite link layouts in place,
//! so a person's first command in the worktree can come before or after them,
//! and the add does not have to wait while dozens of adds queue on one disk.
//! The record names the worktree and the tools it needs,
//! so the service runs each tool's own frozen,
//! offline install there,
//! and the add returns after one record and one wake.
//! A record that cannot reach a service stays,
//! and the line the add prints names the command that does the same work by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! handOverRelink({ context, commonDir, destination, entry }); // enqueue one `relink` item, wake the service
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_relink_record.rs`:
//!   the record one worktree's relink is queued as,
//!   and the tools it may ask for.
//! - `package_repair_relink_work_item.rs`:
//!   the work item that record becomes,
//!   its object name,
//!   and the add's own directories a relative search path is read from.

/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the record's entry,
/// the tool names it carries,
/// the entry's object name,
/// and the item the queue stores.
pub use crate::package_repair_relink_record::{
    RELINK_REFERENCE,
    RelinkEntry,
    RelinkTool,
    SearchPathOfTheAdd,
    decode,
    encode,
};
pub use crate::package_repair_relink_work_item::{
    item_of,
    object_name_of,
};
/// Import the client that wakes a service,
/// and the configuration a wake is given.
use crate::service_client::wake;
/// Import the client configuration every session of this wrapper reaches a service with.
use crate::service_commit_input_notes_and_embedded_commit::managed_client_config;
/// Import the words and the rendering of a client's failure.
use crate::service_messages::{
    ClientFailure,
    render,
};
/// Import the one step that makes a service of a session's own.
use crate::service_provision::provision;
/// Import the route a session's environment names.
use crate::service_route::route_from_environment;
/// Import the address a wake is sent to.
use crate::service_server::Bind;
/// Import the directory the queue's states live in.
use crate::service_work::work_directory;
/// Import the real Git and its overlay a hand-over is made for.
use crate::transaction_git::GitContext;
/// Import the bound a hand-over waits for a greeting.
use crate::wait_ceilings::HAND_OVER_GREETING_WAIT;
/// Import the queue's own names:
/// its record,
/// its pending state,
/// the work to add,
/// and the directory a state lives in.
use crate::work_item::{
    Enqueued,
    PENDING,
    WorkItem,
    enqueue,
    state_directory,
};
/// Import what a hand-over may reach:
/// the route finder,
/// the provisioner,
/// and the bound the greeting is waited with.
use crate::worktree_trash_hand_over::Reach;
/// Import the borrowed and owned filesystem paths a record names.
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairRelink";
/// What:
///  Write the relink's record and wake the service that runs it,
/// or take the record back when no service holds it.
/// Why:
///   The record is written first,
/// so a wake that cannot reach the service leaves the work named rather than lost,
/// and the add never waits for the relinking.
/// A record nobody took owes nothing,
/// because the add relinks the worktree itself,
/// and it must go:
/// a record names a path and the tools for it,
/// and a service that drained it later would run those tools in whatever directory holds that path by then,
/// which need not be the worktree the add relinked.
/// Taking the pending record back is one unlink,
/// which the queue's claim answers by finding nothing to take,
/// so the two never both do the work.
///
/// Ok:
///  A service holds the record,
/// so the relinking is its work.
/// Err:
///  No service holds the record and none can be given it,
/// so the add relinks the worktree itself;
/// the answer says why,
/// and the record it leaves is taken back after that run finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, commonDir, destination, entry });
/// ```
pub fn hand_over_to_the_service(
    context: &GitContext,
    common_dir: &Path,
    destination: &Path,
    entry: &RelinkEntry,
    start: &crate::start_directory::StartDirectory,
) -> Result<(), NotTaken> {
    return hand_over_with(
        context,
        common_dir,
        destination,
        entry,
        &Reach {
            router: &route_from_environment,
            provisioner: &provision,
            start_wait: HAND_OVER_GREETING_WAIT,
        },
        start,
    );
}

/// What:
///  `hand_over_to_the_service` with the route finder and the provisioner injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The same reach a prewarm is handed over with,
/// so a relink is queued and woken exactly as that work is,
/// and a hand-over that reaches nothing answers as `hand_over_to_the_service` documents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, destination, entry, reach });
/// ```
pub fn hand_over_with(
    context: &GitContext,
    common_dir: &Path,
    destination: &Path,
    entry: &RelinkEntry,
    reach: &Reach<'_>,
    start: &crate::start_directory::StartDirectory,
) -> Result<(), NotTaken> {
    let item: WorkItem = item_of(common_dir, destination, entry, start).map_err(not_written)?;
    let work_dir: PathBuf = work_directory(common_dir);
    let id: String = item.id();
    match enqueue(work_dir.as_path(), &item) {
        Ok(Enqueued::Added) => {}
        Ok(Enqueued::Known) => {
            // The id says this add's own record is already queued: the moment is part of the name, so no earlier add's
            //  record answers for this one. A record of this add that is not pending any more was claimed by a service,
            //  which knows what became of it.
            return if pending_record(&work_dir, id.as_str()).exists() {
                Ok(())
            } else {
                Err(NotTaken {
                    reason: format!(
                        "the record {id} of this relink was already taken by a service, which holds it"
                    ),
                    queued: None,
                })
            };
        }
        Err(reason) => {
            debug(
                TAG,
                format!("the work item could not be written: {reason}").as_str(),
            );
            return Err(not_written(format!(
                "the work item could not be written: {reason}"
            )));
        }
    }
    let reason: String = match (reach.router)(context.real_git.as_path(), common_dir) {
        None => String::from("no commit service can be started from this session"),
        Some(route) => {
            let crate::service_route::Route {
                address,
                provision: host,
            } = route;
            let reached: Result<(), String> = match host.as_ref() {
                Some(setup) => match (reach.provisioner)(setup) {
                    Ok(()) => {
                        wake_the_service(address, host.as_ref(), common_dir, reach.start_wait)
                    }
                    Err(failure) => Err(render(&failure)),
                },
                None => wake_the_service(address, host.as_ref(), common_dir, reach.start_wait),
            };
            match reached {
                Ok(()) => return Ok(()),
                Err(reason) => reason,
            }
        }
    };
    debug(
        TAG,
        format!(
            "{}: no service took the relink ({reason})",
            destination.display()
        )
        .as_str(),
    );
    // The record stays queued until the add's own run has finished: a record taken back first would leave nothing for a
    //  service to recover the repair from after a kill, and a record left queued only names a path the proof above
    //  confines to the worktree the record was written for.
    return Err(NotTaken {
        reason,
        queued: Some(QueuedRelink {
            work_dir,
            id,
        }),
    });
}

/// The answer of a hand-over whose record was never written.
fn not_written(reason: String) -> NotTaken {
    return NotTaken {
        reason,
        queued: None,
    };
}

/// What:
///  Tell the service at `address` that the repository has work to drain,
///  waiting `start_wait` for its greeting and no longer than that for a start.
/// Why:
///   The wake's lines are not the add's:
/// it says once what became of the relinking,
/// so the client's output is dropped here,
/// and the wait is the reach's,
/// which a control sets short enough that a service which never answers cannot hold it.
/// The record is written before this wake,
/// so the bound is the hand-over's own greeting wait and never the commit's start wait:
/// a greeting that does not come within it means this command does the relinking itself
/// (`wait_ceilings.rs`,
/// `HAND_OVER_GREETING_WAIT`),
/// and the ceiling moves with it so no other bound lengthens the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wakeTheService = (address, host, commonDir, startWait) => { const config = { ...managedClientConfig(address, host), startWait, startCeiling: startWait }; wake(config, commonDir, undefined, silent) };
/// ```
fn wake_the_service(
    address: Bind,
    host: Option<&crate::service_provision::ProvisionHost>,
    common_dir: &Path,
    start_wait: std::time::Duration,
) -> Result<(), String> {
    let config: ClientConfig = ClientConfig {
        start_wait,
        start_ceiling: start_wait,
        ..managed_client_config(address, host)
    };
    let mut silent: std::io::Sink = std::io::sink();
    return wake(&config, common_dir, None, &[], &mut silent).map_err(render_failure);
}

/// What:
///  What a hand-over that no service took leaves the add:
///  the words the person reads,
///  and the record still queued.
/// Why:
///   The add relinks the worktree itself,
/// but the record may only be taken back once that run has finished:
/// a record removed first leaves nothing for a service to recover the repair from after a kill,
/// and the worktree then keeps copied links that name the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotTaken = { reason: string; queued?: QueuedRelink };
/// ```
#[derive(Debug)]
pub struct NotTaken {
    /// Why no service took the record.
    pub reason: String,
    /// The record still queued when one was written,
    /// taken back by the add after its own run.
    pub queued: Option<QueuedRelink>,
}

/// What:
///  A record still queued for a service,
///  which the add's own run makes superfluous when it ends.
/// Why:
///   Taking the record back is one unlink by its own id,
/// and the claim a service makes moves the file out of `pending` in one rename,
/// so this either wins and the claim finds nothing,
/// or loses and the service owns the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QueuedRelink = { workDir: string; id: string };
/// ```
#[derive(Debug)]
pub struct QueuedRelink {
    /// The work directory the record sits in.
    work_dir: PathBuf,
    /// The record's id.
    id: String,
}

impl QueuedRelink {
    /// What:
    ///  Take the record back,
    ///  now that the add's own run has finished.
    /// Why:
    ///   The record owes nothing from here on,
    /// and a record that cannot be removed stays while the service may relink this worktree again,
    /// which is said here rather than failing the add:
    /// a worktree whose copied links were relinked twice is right either way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const takeBack = ({ workDir, id }) => { try { unlink(pending(workDir, id)) } catch (error) { log(`the record ${id} stays: ${error.message}`) } };
    /// ```
    pub fn take_back(&self) {
        match std::fs::remove_file(pending_record(self.work_dir.as_path(), self.id.as_str()).as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                debug(TAG, format!("the record {} was taken by a service, whose run this add's own run stood beside", self.id).as_str())
            }
            Err(error) => debug(TAG, format!("the record {} stays ({error}), so the commit service may relink this worktree again", self.id).as_str()),
        }
    }
}

/// The pending record of one id.
fn pending_record(
    work_dir: &Path,
    id: &str,
) -> PathBuf {
    return state_directory(work_dir, PENDING).join(id);
}

/// What:
///  The words a client failure is reported with.
/// Why:
///   The wake's failure is the person's,
/// and a named function stands where a closure that only forwarded its argument stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renderFailure = (failure) => render(failure);
/// ```
fn render_failure(failure: ClientFailure) -> String {
    return render(&failure);
}

/// The client configuration type the wake takes,
/// named here so the import list stays one word per line.
type ClientConfig = crate::service_client::ClientConfig;

/// What:
///  The hand-over of a relink record as a job of the process's blocking pool.
/// Why:
///  Writing a record and waking a service that may have to be started are storage and socket waits,
/// and the task that runs the Cargo repair beside it must never do either:
/// a service that never answered would stop that repair's waits with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handOverInAJob = ([context, commonDir, destination, entry]) => handOverToTheService(context, commonDir, destination, entry);
/// ```
pub(super) fn hand_over_in_a_job(state: TheHandOver) -> Result<(), NotTaken> {
    let (context, common_dir, destination, entry, start): TheHandOver = state;
    return hand_over_to_the_service(
        &context,
        common_dir.as_path(),
        destination.as_path(),
        &entry,
        &start,
    );
}

/// What a hand-over job reads:
/// the Git context,
/// the repository's common directory,
/// the new worktree,
/// and the entry.
/// Why:
///   A job reads only what it was handed,
/// so the four travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHandOver = [GitContext, string, string, RelinkEntry];
/// ```
pub(super) type TheHandOver = (
    GitContext,
    PathBuf,
    PathBuf,
    RelinkEntry,
    crate::start_directory::StartDirectory,
);

/// Relink controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_tests.rs"]
mod tests;
