//! What:
//!  The resident service's loop:
//! accept connections,
//! queue requests in arrival order,
//! land what is queued as a batch,
//!  answer each client,
//! and exit when told to retire.
//! Why:
//!  One writer lands every commit,
//! so there is no lock to take,
//! hold or wait for:
//! order is the order of the queue,
//! the
//!  cost of the one durable write is shared by the batch,
//! and a client that waits blocks on its socket instead of
//!  polling (`doc/planning/cli-git-resident-service.md`).
//! The service never starts itself:
//! the operating system hands it
//!  the listening socket (socket activation),
//! or a development run binds one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const batch of queue.batches()) { const outcomes = await landBatch(batch); for (...) reply(outcome); }
//! ```

pub use crate::service_server_work_loop::work_loop;
pub use crate::service_server_socket_binding::{Bind, INHERITED_DESCRIPTOR, bind, socket_passed_in};
#[cfg(test)]
use crate::service_server_socket_binding::ensure_socket_passed_in;
use crate::service_server_queue_types_and_send::{GroupKey, Job, Queued, id_of_queued, index_of_queued, place_in_groups};
use crate::service_server_land_group::{land_group, settle_open_batches};
use crate::service_server_connection_handling::ConnectionShared;
use crate::service_queue_received_on_the_runtime::{Received, receive_for_at_most};
use crate::service_server_accept_connections_as_tasks::start_acceptor_or_exit;
use crate::stop_of_the_service_noticed_by_its_children::{begin_unfinished_work, is_stopping};
use crate::service_journal::{DONE_RETAINED, landed_commits, prune_done};
use crate::service_index_lock::unsettled_locks;
use crate::service_index_prestage::{PrestageGuard, start as start_prestage};
use crate::service_captures::CaptureMemory;
use crate::service_land::LandContext;
use crate::service_timing::{ExitReason, exit_line, log};
use crate::service_claim_watch::{ClaimWatch, WATCH_DIRECTORY};
use crate::push_retry::{PLAN_DIRECTORY, PushRetries};
use crate::service_work::{GitHandlers, Wake};
use crate::service_busy::{GroupAdmission, LandingBoard};
use crate::wait_ceilings::NEVER_ENDED_STEP_SLOW_AFTER;
use crate::service_protocol::{
    FailureCode, RELEASE, ServiceMessage,
};
use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The largest batch whose landing still costs about what its size predicts: 256.
/// Measured with the service binary on throwaway
///  repositories,
/// 2026-11-03,
/// bursts of simultaneous commit requests (`MAX_BATCH` raised to 4096 for the probe,
/// three rounds
///  alternating over 64,
/// 128,
/// 256,
/// 512 and 1024 clients,
/// other sessions running,
/// a debug build):
/// a landing took 71 to 76 ms for
///  62 to 63 requests,
/// 82 to 85 ms for 126,
/// 104 to 109 ms for 253 to 255,
/// 193 to 204 ms for 509 to 511 (once 2,002 ms,
/// a host stall)
///  and 525 to 547 ms for 1,021 to 1,023.
/// The cost of each added request is 0.16 ms up to 254 requests,
/// 0.36 ms from there to
///  510,
/// and 0.66 ms to 1,022,
/// so the cost per request has not yet doubled at 256.
/// The time a client waits for a burst of 128 was
///  176 to 206 ms in one landing and 243 to 249 ms in three at the old cap of 64.
pub const BATCH_COST_KNEE: usize = 256;

/// The most requests landed as one batch:
/// `BATCH_COST_KNEE`.
/// A batch is answered all at once and holds the index lock while it
///  lands,
/// so the cap bounds both.
/// The prototype's burst reached 64,
/// and production never reached more than one request in a
///  batch (1,302 answered requests,
/// 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z),
/// so the cap bites only in a burst.
/// Option B of the
///  batch size question,
/// picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups").
pub const MAX_BATCH: usize = BATCH_COST_KNEE;

/// How long the service waits with nothing to do before it exits;
/// the manager's socket stays,
/// so the next connection starts it
///  again.
/// Bazel's server does the same with `--max_idle_secs`,
/// whose default is three hours,
/// which is this value's precedent.
///  Measured cost of exceeding it:
/// of 1,051 commits in the commit timing log (2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z),
/// the 10
///  that followed an idle gap of more than three hours reached the service in at most 70 ms (`reachMs`) after a start wait of at
///  most 47 ms (`startWaitMs`);
/// 110 idle exits happened in that window.
/// The 28 running services held 212 MiB together (7.9 MiB each,
///  `systemctl --user show --property=MemoryCurrent`).
/// A shorter exit would trade at most that start for that memory;
/// the owner's
///  bound is that a commit should not take 5 s longer than native Git (`SPEC.md`).
/// Kept at three hours as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const IDLE_EXIT: Duration = Duration::from_secs(3 * 60 * 60);

/// How often a serving service asks whether the worktree whose units are its own is still there.
/// Why:
///   A removal Git ran with the wrapper replaced and a worktree deleted without Git leave the socket unit listening,
/// and the service that serves it is already running:
/// no start happens for a gone worktree,
/// so nothing would ever ask,
/// and the socket would listen for that directory until a person remembered the sweep.
/// At this interval the socket stops at most this long after the removal,
/// and the price is one wake of the landing thread and two stat calls per service per interval:
/// one `symlink_metadata` on the Git directory and one `is_dir` on the place that holds it
/// (`service_units_of_removed_worktrees::worktree_is_gone`),
/// measured on this host at 11.4 microseconds for the pair (five rounds of a million,
/// 11.32 to 11.74),
/// so 355 services cost 0.07 milliseconds a second between them.
/// A minute rather than an hour because the leftover is a listening socket,
/// and a minute rather than a second
/// because the loop's own wake is what it costs;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const OWN_WORKTREE_CHECK: Duration = Duration::from_secs(60);

/// How a service run behaves apart from where it listens and what it lands.
/// No equality:
/// the stop of a service's own units is a function pointer,
/// whose addresses are not comparable.
#[derive(Clone, Debug)]
pub struct ServeSettings {
    /// How long the service waits with nothing to do before it exits.
    pub idle: Duration,
    /// The release the service announces to every client.
    /// It is this build's release (`RELEASE`) except in a test that
    ///  needs a service of another release.
    pub release: String,
    /// The worktree this instance serves,
    /// as `service_instance::instance_of` names it;
    /// every message about another worktree
    ///  is refused.
    /// Nothing in a development run that serves any worktree.
    pub instance: Option<String>,
    /// How long a write the service never ends runs before the clients behind it are told,
    /// and may be withdrawn while the
    ///  service is inside it (`wait_ceilings.rs`,
    /// `NEVER_ENDED_STEP_SLOW_AFTER`);
    /// a control shortens it.
    pub stuck_after: Duration,
    /// How long a batch's first request must have waited before it is told it is next,
    /// which is how a person sees the service
    ///  take the commit after a wait (`wait_ceilings.rs`,
    /// `WAITING_NOTICE_AFTER`);
    /// a control shortens it.
    pub place_after: Duration,
    /// The Git directory of the worktree whose units are this service's,
    /// as its note or its journal named it when the service started,
    /// and nothing when it was not there or could not be told.
    /// The serving loop asks about it every `own_worktree_check`.
    pub own_git_dir: Option<PathBuf>,
    /// How often the serving loop asks whether `own_git_dir` is still there.
    pub own_worktree_check: Duration,
    /// What the service does when that directory is gone:
    /// the shipped one stops this service's own socket through the user manager
    ///  (`service_units_of_removed_worktrees::stop_own_socket_when_the_worktree_is_gone`),
    /// and a control passes its own,
    /// so the loop is checked without a manager.
    pub own_units_stop: fn(&str, &Path) -> bool,
}

impl ServeSettings {
    /// The settings of an ordinary run:
    /// the given idle exit and this build's release.
    pub fn new(idle: Duration) -> Self {
        return Self {
            idle,
            release: String::from(RELEASE),
            instance: None,
            stuck_after: NEVER_ENDED_STEP_SLOW_AFTER,
            place_after: crate::wait_ceilings::WAITING_NOTICE_AFTER,
            own_git_dir: None,
            own_worktree_check: OWN_WORKTREE_CHECK,
            own_units_stop: crate::service_units_of_removed_worktrees::stop_own_socket_when_the_worktree_is_gone,
        };
    }
}

/// Starts a thread the service cannot do without,
/// or ends the service with a plain line and status 4:
/// a service that goes on
///  without its acceptor or its work thread answers no one,
/// while an ended service is started again by the manager.
fn start_thread_or_exit(what: &str, body: impl FnOnce() + Send + 'static) {
    if let Err(error) = std::thread::Builder::new().spawn(body) {
        crate::service_error!("{}", crate::scoped_start::refused_thread_sentence(format!("run {what}").as_str(), &error));
        std::process::exit(4);
    }
}

/// How many withdrawn identities the service remembers before it forgets them all:
/// the same count as the finished batches it
///  remembers (`DONE_RETAINED`),
/// since a client that withdrew sends nothing more for its request.
const WITHDRAWN_RETAINED: usize = DONE_RETAINED;

/// What:
///  Whether this service should stop serving because the worktree whose units are its own is gone.
/// Why:
///   Every wake of a service whose worktree is known asks this:
/// the removal that takes a worktree while its service runs is what leaves a socket listening for a directory that is gone.
/// The answer is the same rule the sweep and `doctor` use (`worktree_is_gone`),
/// so an unmounted device and an instance nothing names keep the service serving.
/// The idle bound alone ends the service,
/// and that is answered before this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (ownWorktreeGoneNow(settings, untilIdle)) stopOwnUnitsAndEnd();
/// ```
fn own_worktree_gone_now(settings: &ServeSettings, until_idle: Duration) -> bool {
    if settings.own_git_dir.is_none() || until_idle <= settings.own_worktree_check {
        return false;
    }
    return settings.own_git_dir.as_deref().is_some_and(crate::service_units_of_removed_worktrees::worktree_is_gone);
}

/// What:
///  Runs the service until it is told to retire.
/// Why:
///   The acceptor starts before the recovery of the batch an earlier service left open,
/// so a client that connects
///  while that recovery runs is greeted and beaten instead of reading as a service that stopped;
/// the loop then takes everything queued as
///  one batch,
/// groups it by branch in order of first arrival,
/// lands each group,
/// and answers.
/// Nothing is landed before
///  the recovery is over:
/// only this thread lands,
/// and it lands nothing until it reads the queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serve(listener, context)
/// ```
pub fn serve(listener: UnixListener, context: &LandContext, settings: &ServeSettings) {
    // The loop is the one landing: what a landing remembers (its board, its queued syncs, its unsettled locks) belongs to this
    //  scope and ends with it, and no other thread of the service reads it (`landing_state_carried_by_its_task.rs`).
    crate::landing_state_carried_by_its_task::in_a_landing_with((listener, context, settings), serve_in_a_landing);
}

/// `serve`,
/// inside the scope of the landing's state.
fn serve_in_a_landing((listener, context, settings): (UnixListener, &LandContext, &ServeSettings)) {
    let serving_since: Instant = Instant::now();
    let idle: Duration = settings.idle;
    // The identities of the requests their clients withdrew; a request with one of these is never landed.
    let mut withdrawn: std::collections::HashSet<String> = std::collections::HashSet::new();
    // What the landed commits were captured at: owned by this loop, the only thread that lands and answers.
    let mut captures: CaptureMemory = CaptureMemory::default();
    let (sender, mut receiver): (UnboundedSender<Job>, UnboundedReceiver<Job>) = unbounded_channel();
    let acceptor_queue: UnboundedSender<Job> = sender.clone();
    // After-commit work (push, upkeep) runs on its own thread, so a slow network never delays a landing.
    let (work_sender, mut work_receiver): (UnboundedSender<Wake>, UnboundedReceiver<Wake>) = unbounded_channel();
    let handlers: GitHandlers = GitHandlers::new(context.git.clone());
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let worker_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    // What this landing thread is inside, for the threads that talk to clients (`service_busy.rs`).
    let board: Arc<LandingBoard> = LandingBoard::new(context.journal.clone(), settings.stuck_after);
    crate::service_busy_step_record_on_this_thread::use_on_this_thread(Arc::clone(&board));
    // The plan of the pushes waiting for a new try is kept in this instance's own journal, so a restarted service rebuilds it.
    let plan_directory: PathBuf = context.journal.join(PLAN_DIRECTORY);
    // The repositories this instance was woken for are kept in the journal the same way, so a restarted service looks at them.
    let watch_directory: PathBuf = context.journal.join(WATCH_DIRECTORY);
    let watch_instance: Option<String> = settings.instance.clone();
    start_thread_or_exit("the after-commit work", move || { work_loop(&mut work_receiver, &handlers, &worker_busy, PushRetries::restored_from_environment(plan_directory.as_path()), ClaimWatch::restored(watch_directory.as_path(), watch_instance.as_deref())); });
    // What:
    //  The acceptor starts here,
    // before the recovery of the batch an earlier service left open.
    // Why:
    //  The recovery writes the journal's records and the index,
    // which waits on the disk,
    // and finding C of the audit
    //  `commit-kills-and-concurrency` (2026-11-09) is a service whose slow recovery made every waiting client exit 1
    // with "not known whether the commit was made" although the service landed their commits:
    // a connection accepted before the
    //  recovery is greeted and beaten every second,
    // so a slow recovery reads as a service that works.
    // A request that arrives
    //  meanwhile waits in the queue,
    // which only the landing thread reads,
    // and that thread reads it after the recovery,
    // so nothing lands before the recovery is over.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // startAcceptor(listener); const reports = await settleOpenBatches(context);
    // ```
    start_acceptor_or_exit(
        listener,
        ConnectionShared {
            queue: acceptor_queue,
            work: work_sender,
            board: Arc::clone(&board),
            release: settings.release.clone(),
            instance: settings.instance.clone(),
        },
    );
    let _reports = settle_open_batches(context, settings.instance.as_deref());
    // The memory of landed requests is bounded: the oldest finished batches go before the rest are read.
    let pruned: usize = prune_done(context.journal.as_path(), DONE_RETAINED);
    if pruned > 0 {
        crate::service_log!("removed the records of {pruned} old finished batches from {}", context.journal.display());
    }
    let mut landed: HashMap<String, String> = landed_commits(context.journal.as_path());
    // What the earlier release's helper of this worktree finished counts as landed too: a client it never answered asks again.
    if let Some(instance) = settings.instance.as_deref() {
        landed.extend(crate::earlier_service_journal_adopted::earlier_commits_landed(context.journal.as_path(), instance));
    }
    // The service's own shutdown: a stop is recorded by the handler and ended here, after the Gits the service waits on are ended.
    crate::service_server_shutdown_after_a_stop::start_shutdown_after_a_stop_or_exit();
    // Microseconds, so that instances started at different times count from different numbers; the number each batch takes is
    //  still claimed in the journal (`claim_batch`), because equal readings and counts that overtake another's are possible.
    let clock_seed: u64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => u64::try_from(elapsed.as_micros()).unwrap_or(1),
        Err(_before_the_epoch) => 1,
    };
    let mut seq: u64 = crate::service_fault::pinned_first_batch().unwrap_or(clock_seed);
    // The marker made ahead is removed whichever way this loop ends.
    let _prestaged = PrestageGuard;
    let mut retiring: bool = false;
    // When the service last took work, so that the idle exit stays what `IDLE_EXIT` says while a known worktree is asked about
    //  far more often.
    let mut last_work: Instant = Instant::now();
    while !retiring {
        // A service whose worktree is known wakes at least every `own_worktree_check`: nothing else asks whether that worktree
        //  is still there, and the removal that takes it may happen while this service runs.
        let until_idle: Duration = idle.saturating_sub(last_work.elapsed());
        let wait: Duration = if settings.own_git_dir.is_some() { until_idle.min(settings.own_worktree_check) } else { idle };
        let first: Job = match receive_for_at_most(&mut receiver, Some(wait)) {
            Received::Item(job) => job,
            // Work in flight keeps the service; otherwise it exits and the manager starts it on the next connection.
            Received::Timeout if busy.load(Ordering::SeqCst) > 0 => {
                last_work = Instant::now();
                continue;
            }
            // The interval of a service whose worktree is known, with the idle bound still ahead: the worktree may have been
            //  removed by another Git, and then no start of this service ever comes again to notice it.
            Received::Timeout if own_worktree_gone_now(settings, until_idle) => {
                let instance: &str = settings.instance.as_deref().unwrap_or_default();
                let _asked = (settings.own_units_stop)(instance, context.journal.as_path());
                // Leaving as the idle exit does, so the syncs of the landings that were answered are made first.
                crate::service_durable::drain();
                log(exit_line(ExitReason::WorktreeGone, serving_since.elapsed()).as_str());
                return;
            }
            // The interval of a service whose worktree is known, and that worktree is still there: serve on.
            Received::Timeout if settings.own_git_dir.is_some() && until_idle > settings.own_worktree_check => continue,
            Received::Timeout => {
                // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
                crate::service_durable::drain();
                log(exit_line(ExitReason::Idle, serving_since.elapsed()).as_str());
                return;
            }
            Received::Closed => {
                // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
                crate::service_durable::drain();
                log(exit_line(ExitReason::Error, serving_since.elapsed()).as_str());
                return;
            }
        };
        // Nothing is landed after a stop was recorded: the landing in flight unwinds and the shutdown ends the process; a request that
        //  arrives now is not answered, and its client asks again when the service is started (the journal knows what landed).
        if is_stopping() {
            continue;
        }
        last_work = Instant::now();
        let mut jobs: Vec<Job> = vec![first];
        while jobs.len() < MAX_BATCH {
            match receiver.try_recv() {
                Ok(job) => jobs.push(job),
                Err(_) => break,
            }
        }
        // Withdrawals are answered before anything lands, on this one thread, so each is either after a landing that is over (the
        //  commit is on the branch, and that is the answer) or before the landing of its request begins (the request is dropped
        //  and remembered, so it can never land, not even when it reaches the queue after the withdrawal).
        if withdrawn.len() > WITHDRAWN_RETAINED {
            withdrawn.clear();
        }
        for job in &jobs {
            if let Job::Withdraw(id, reply) = job {
                match landed.get(id) {
                    Some(commit) => reply.send(&ServiceMessage::Landed { id: id.clone(), commit: commit.clone(), already: true, landing: None }),
                    None => {
                        withdrawn.insert(id.clone());
                        reply.send(&ServiceMessage::Withdrawn { id: id.clone() });
                    }
                }
            }
        }
        // Branches in order of first arrival; within a branch, arrival order.
        let mut groups: Vec<(GroupKey, Vec<Queued>)> = Vec::new();
        for job in jobs {
            match job {
                Job::Withdraw(..) => {}
                Job::Commit(request, _dropped, _arrival) if withdrawn.contains(&request.id) => {
                    crate::service_log!("dropped the request {} that its client withdrew", request.id);
                }
                Job::Retire => retiring = true,
                Job::Activity => {}
                Job::Commit(request, stream, arrival) => {
                    // An amend is a group of its own, and what arrives after it starts another group, so the order on
                    //  one branch is the order of arrival.
                    // `HEAD` is a head of one worktree, so two detached worktrees of one repository are two branches.
                    let queue_reference: String = if request.reference == "HEAD" {
                        format!("HEAD@{}", request.repository.display())
                    } else {
                        request.reference.clone()
                    };
                    let key: GroupKey = (request.common_dir.clone(), queue_reference, request.lands_alone());
                    place_in_groups(&mut groups, key, (request, stream, arrival));
                }
            }
        }
        for (_, queued_group) in groups.drain(..) {
            // The trees this group's requests name: each is the commit's own scratch directory, which nothing may remove before the
            //  landing that reads it, and which the service removes once that landing is over.
            //  A request the service did not land keeps its tree in its client's hands instead (see `remove_the_trees`).
            let mut trees: Vec<(String, std::path::PathBuf)> = Vec::new();
            for queued in &queued_group {
                if let Some(tree) = crate::service_scratch_release_removal::the_tree_of(queued.0.scratch.as_path(), queued.0.git_dir.as_path()) {
                    trees.push((queued.0.id.clone(), tree));
                }
            }
            // The shutdown waits for this landing and refuses one that would begin after the stop.
            let Ok(_landing_in_flight) = begin_unfinished_work(false, "a landing") else {
                break;
            };
            let index: Option<std::path::PathBuf> = queued_group.first().map(index_of_queued);
            // A request whose client was answered "withdrawn" while this thread was stuck in a write is dropped here, and the
            //  rest are marked as being landed, under one lock (`service_busy.rs`).
            let ids: Vec<String> = queued_group.iter().map(id_of_queued).collect();
            let admission: GroupAdmission = board.begin_group(ids.as_slice());
            for id in &admission.dropped {
                crate::service_log!("dropped the request {id} that its client withdrew while the service was stuck in a write");
            }
            for id in &admission.unavailable {
                crate::service_error!("the request {id} could not be admitted to its group (the record of the landing thread cannot be written), so it is answered with a failure and not landed");
            }
            // A dropped request is not answered here; an unavailable one is answered with a failure; the rest are landed.
            let mut unavailable: Vec<Queued> = Vec::new();
            let mut admitted: Vec<Queued> = Vec::new();
            for queued in queued_group {
                if admission.dropped.contains(&queued.0.id) {
                    continue;
                }
                if admission.unavailable.contains(&queued.0.id) {
                    unavailable.push(queued);
                } else {
                    admitted.push(queued);
                }
            }
            for (request, stream, _) in &unavailable {
                stream.send(&ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Unavailable });
            }
            let group: Vec<Queued> = admitted;
            if group.is_empty() {
                board.end_group(ids.as_slice());
                remove_the_trees(trees.as_slice(), &std::collections::HashSet::new());
                continue;
            }
            let mut handed_back: std::collections::HashSet<String> = std::collections::HashSet::new();
            land_group(context, settings.instance.as_deref(), &mut seq, group, &mut landed, &mut captures, settings.place_after, &mut handed_back);
            board.end_group(ids.as_slice());
            // The requests are landed and answered, so no landing reads these any more; each removal runs beside this loop, bounded,
            //  and says so in the journal when it cannot finish (`service_scratch_release_removal.rs`).
            remove_the_trees(trees.as_slice(), &handed_back);
            // The batch after this one is numbered one more and lands on this index (a worktree has one), so its marker is
            //  made and flushed now, while the service is idle, and the next commit only places it.
            if let Some(next_index) = index {
                start_prestage(next_index.as_path(), seq + 1);
            }
        }
        // A lock this thread could not settle names this process: the restart reclaims it once this process is gone.
        if unsettled_locks() > 0 {
            crate::service_error!("ending, because an index lock could not be settled; the next connection starts the service again and its recovery reclaims the lock");
            return;
        }
    }
    // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
    crate::service_durable::drain();
    log(exit_line(ExitReason::Retire, serving_since.elapsed()).as_str());
}

/// What:
///  Starts the removal of the scratch trees of the requests a group has just finished with,
///  beside the loop,
///  except those the service handed back to their clients.
/// Why:
///   The tree holds the private index and the conclusion copies a landing reads,
///   so only the landing that named it may decide that it is done with it.
///  A tree whose client is gone waits for that landing instead of for its connection to close,
///  which is what makes a killed client's queued commit land on the tree it made.
///  A request the service did not land (a commit sent back to be prepared again,
/// a branch that moved) has no landing
///  that read its tree:
/// its client reads the change it prepared from that tree to make the commit again,
/// so the client
///  releases it beside its own command.
///  Removing it here took that read from under every rebuild,
/// which failed with a missing private index
///  (2026-11-09,
///  `service_prepare.rs`,
///  `retry_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// trees.filter(([id]) => !handedBack.has(id)).forEach(startTheRemovalBesideTheLoop);
/// ```
fn remove_the_trees(trees: &[(String, std::path::PathBuf)], handed_back: &std::collections::HashSet<String>) {
    for (id, tree) in trees {
        if handed_back.contains(id) {
            continue;
        }
        crate::service_scratch_release_removal::start(tree.clone());
    }
}

/// Socket controls stay out of the release executable.
#[cfg(test)]
#[path = "service_server_tests.rs"]
mod tests;

/// Controls for a service whose own worktree is deleted while it serves stay out of the release executable.
#[cfg(test)]
#[path = "service_server_own_worktree_tests.rs"]
mod own_worktree_tests;

/// Controls for a service that is still recovering the batch an earlier service left open stay out of the release executable.
#[cfg(test)]
#[path = "service_server_beats_while_recovering_tests.rs"]
mod beats_while_recovering_tests;
