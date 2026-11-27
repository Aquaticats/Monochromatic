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

use crate::push_retry::{
    PLAN_DIRECTORY,
    PushRetries,
};
use crate::service_busy::LandingBoard;
use crate::service_captures::CaptureMemory;
use crate::service_claim_watch::{
    ClaimWatch,
    WATCH_DIRECTORY,
};
use crate::service_index_lock::unsettled_locks;
use crate::service_index_prestage::PrestageGuard;
use crate::service_journal::{
    DONE_RETAINED,
    landed_commits,
    prune_done,
};
use crate::service_land::LandContext;
use crate::service_queue_received_on_the_runtime::{
    Received,
    receive_for_at_most,
};
use crate::service_server_accept_connections_as_tasks::start_acceptor_or_exit;
use crate::service_server_after_commit_work_start::start_the_after_commit_work_or_exit;
use crate::service_server_batch_read::{
    ReadBatch,
    answer_the_withdrawals,
    group_the_batch,
};
use crate::service_server_connection_handling::ConnectionShared;
use crate::service_server_land_group::settle_open_batches;
use crate::service_server_landing_one_group::{
    TheGroupLanding,
    land_the_group,
};
use crate::service_server_own_worktree_question::own_worktree_gone_now;
use crate::service_server_queue_types_and_send::Job;
use crate::service_timing::{
    ExitReason,
    exit_line,
    log,
};
use crate::service_work::{
    GitHandlers,
    Wake,
};
use crate::stop_of_the_service_noticed_by_its_children::is_stopping;
use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::time::{
    Duration,
    Instant,
};
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
    unbounded_channel,
};

/// What one service run is held to:
/// the batch cap,
/// the batch whose cost still grows as its size does,
/// the idle exit,
/// and how often the worktree it serves is asked about.
pub use crate::service_server_batch_and_idle_limits::{
    BATCH_COST_KNEE,
    IDLE_EXIT,
    MAX_BATCH,
    OWN_WORKTREE_CHECK,
};
/// What one service run is configured with.
pub use crate::service_server_serve_settings::ServeSettings;
#[cfg(test)]
use crate::service_server_socket_binding::ensure_socket_passed_in;
pub use crate::service_server_socket_binding::{
    Bind,
    INHERITED_DESCRIPTOR,
    address_text,
    bind,
    socket_passed_in,
};
pub use crate::service_server_work_loop::work_loop;
pub use crate::service_server_work_loop_awaiting::work_loop_awaiting;

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
pub fn serve(
    listener: UnixListener,
    context: &LandContext,
    settings: &ServeSettings,
) {
    // The loop is the one landing: what a landing remembers (its board, its queued syncs, its unsettled locks) belongs to this
    //  scope and ends with it, and no other thread of the service reads it (`landing_state_carried_by_its_task.rs`).
    crate::landing_state_carried_by_its_task::in_a_landing_with(
        (listener, context, settings),
        serve_in_a_landing,
    );
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
    let (sender, mut receiver): (UnboundedSender<Job>, UnboundedReceiver<Job>) =
        unbounded_channel();
    let acceptor_queue: UnboundedSender<Job> = sender.clone();
    // After-commit work (push, upkeep) runs as a task of its own, so a slow network never delays a landing.
    let (work_sender, work_receiver): (UnboundedSender<Wake>, UnboundedReceiver<Wake>) =
        unbounded_channel();
    let handlers: GitHandlers = GitHandlers::new(context.git.clone());
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let worker_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    // The finished shadows this service drew ahead of the commits that ask for one, and the answers that take them: memory of
    //  this run only, so a restarted service hands out nothing it did not draw (`service_shadow_draw.rs`).
    let drawn: Arc<crate::service_shadow_draw::DrawnShadows> =
        crate::service_shadow_draw::DrawnShadows::none_yet();
    // What this landing thread is inside, for the threads that talk to clients (`service_busy.rs`).
    let board: Arc<LandingBoard> = LandingBoard::new(context.journal.clone(), settings.stuck_after);
    crate::service_busy_step_record_on_this_thread::use_on_this_thread(Arc::clone(&board));
    // The plan of the pushes waiting for a new try is kept in this instance's own journal, so a restarted service rebuilds it.
    let plan_directory: PathBuf = context.journal.join(PLAN_DIRECTORY);
    // The repositories this instance was woken for are kept in the journal the same way, so a restarted service looks at them.
    let watch_directory: PathBuf = context.journal.join(WATCH_DIRECTORY);
    let watch_instance: Option<String> = settings.instance.clone();
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
    // The plan and the watch of the after-commit work are read after this,
    // and on a thread of their own,
    // so a
    //  journal that stopped answering delays them and never the greeting either.
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
            drawn: Arc::clone(&drawn),
            board: Arc::clone(&board),
            release: settings.release.clone(),
            instance: settings.instance.clone(),
            // The worktree this instance was named for, so a request naming it exactly is answered without a resolution.
            own_git_dir: settings.own_git_dir.clone(),
        },
    );
    // The plan and the watch are read here, after the acceptor started and before the task that runs the loop starts, so the
    //  loop's task begins with what a service that stopped last left and neither reading delays a client's greeting.
    let (started_retries, mut started_watch): (PushRetries, ClaimWatch) =
        crate::service_journal_restored_at_start::restored_within(
            settings.journal_restore_ceiling,
            plan_directory,
            watch_directory,
            watch_instance,
        );
    // What: The worktree this service serves is one of the repositories the loop looks at by itself.
    // Why:  Every record a client leaves for this instance is written under this worktree's own Git directory, so a record whose
    //       wake was lost is found by the look even when no landing of this process wrote a note for it and no client ever woke
    //       it (`service_claim_watch.rs`, `LOOK_AGAIN_AFTER`).
    if let Some(own) = settings.own_git_dir.as_deref() {
        started_watch.remember(own);
    }
    start_the_after_commit_work_or_exit(
        work_receiver,
        handlers,
        worker_busy,
        started_retries,
        started_watch,
    );
    let _reports = settle_open_batches(context, settings.instance.as_deref());
    // The memory of landed requests is bounded: a finished record goes only when no client of it may still read it,
    //  and only past the newest `DONE_RETAINED` of the journal, before the rest are read into that memory.
    let pruned: usize = prune_done(
        context.journal.as_path(),
        DONE_RETAINED,
        &crate::claim_time::ClaimTime::read(),
    );
    if pruned > 0 {
        crate::service_log!(
            "removed the records of {pruned} old finished batches from {}",
            context.journal.display()
        );
    }
    let mut landed: HashMap<String, String> = landed_commits(context.journal.as_path());
    // What the earlier release's helper of this worktree finished counts as landed too: a client it never answered asks again.
    if let Some(instance) = settings.instance.as_deref() {
        landed.extend(
            crate::earlier_service_journal_adopted::earlier_commits_landed(
                context.journal.as_path(),
                instance,
            ),
        );
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
    // What: How many commits this process was asked to land.
    // Why:  A service retired before it served anything has nothing to finish, and its socket is stopped so the client that
    //       retired it cannot start it again and again (`stop_own_socket_before_serving_anything`).
    let mut served: usize = 0;
    // When the service last took work, so that the idle exit stays what `IDLE_EXIT` says while a known worktree is asked about
    //  far more often.
    let mut last_work: Instant = Instant::now();
    while !retiring {
        // A service whose worktree is known wakes at least every `own_worktree_check`: nothing else asks whether that worktree
        //  is still there, and the removal that takes it may happen while this service runs.
        let until_idle: Duration = idle.saturating_sub(last_work.elapsed());
        let wait: Duration = if settings.own_git_dir.is_some() {
            until_idle.min(settings.own_worktree_check)
        } else {
            idle
        };
        let first: Job = match receive_for_at_most(&mut receiver, Some(wait)) {
            Received::Item(job) => job,
            // Work: The worktree whose units are this service's may have been removed while the service runs, and that
            //       question is answered before any other answer to a timeout.
            // Why:  The after-commit work counts itself busy while a push waits for a new try, and the busy arm below keeps the
            //       service alive for that work's sake; a service of a repository that was deleted, with a plan nothing can run,
            //       therefore never reached this question and never ended (measured 2026-11-11: units retrying at try 149 for a
            //       repository deleted hours before). Nothing can be delivered to a directory that is gone, so nothing waits for
            //       the work to finish first.
            Received::Timeout if own_worktree_gone_now(settings, until_idle) => {
                let instance: &str = settings.instance.as_deref().unwrap_or_default();
                let _asked = (settings.own_units_stop)(instance, context.journal.as_path());
                // Leaving as the idle exit does, so the syncs of the landings that were answered are made first.
                crate::service_durable::drain();
                log(exit_line(ExitReason::WorktreeGone, serving_since.elapsed()).as_str());
                return;
            }
            // Work in flight keeps the service; otherwise it exits and the manager starts it on the next connection.
            Received::Timeout if busy.load(Ordering::SeqCst) > 0 => {
                last_work = Instant::now();
                continue;
            }
            // The interval of a service whose worktree is known, and that worktree is still there: serve on.
            Received::Timeout
                if settings.own_git_dir.is_some() && until_idle > settings.own_worktree_check =>
            {
                continue;
            }
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
        answer_the_withdrawals(jobs.as_slice(), &landed, &mut withdrawn);
        let ReadBatch {
            groups,
            retiring: asked_to_retire,
            commits,
        } = group_the_batch(jobs, &withdrawn);
        retiring = asked_to_retire;
        served += commits;
        for (_, queued_group) in groups {
            let landed_one: TheGroupLanding = land_the_group(
                context,
                settings.instance.as_deref(),
                &mut seq,
                queued_group,
                &mut landed,
                &mut captures,
                settings.place_after,
                settings.note_ceiling,
                &drawn,
                &board,
            );
            if matches!(landed_one, TheGroupLanding::TheStopBegan) {
                break;
            }
        }
        // A lock this thread could not settle names this process: the restart reclaims it once this process is gone.
        if unsettled_locks() > 0 {
            crate::service_error!(
                "ending, because an index lock could not be settled; the next connection starts the service again and its recovery reclaims the lock"
            );
            return;
        }
    }
    // What: A service retired before it served anything stops its own socket, so the connection that retired it does not
    //       start another one.
    // Why:  A client of another release retires every service it meets, and each of its connections starts one; the manager's
    //       start limit then leaves the unit failed for as long as no command in that worktree runs
    //       (`stop_own_socket_before_serving_anything`).
    if served == 0
        && let Some(instance) = settings.instance.as_deref()
    {
        let _stopped: bool = crate::service_own_socket_of_a_retire_that_served_nothing::stop_own_socket_before_serving_anything(instance);
    }
    // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
    crate::service_durable::drain();
    log(exit_line(ExitReason::Retire, serving_since.elapsed()).as_str());
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

/// Controls for the greeting of a client while the journal of the after-commit work gives no answer stay out of the
/// release executable.
#[cfg(test)]
#[path = "service_server_greets_before_the_journal_is_restored_tests.rs"]
mod greets_before_the_journal_is_restored_tests;
