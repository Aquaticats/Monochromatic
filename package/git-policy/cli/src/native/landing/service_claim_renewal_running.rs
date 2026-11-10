//! What:
//!  A running renewal of the claims one drain holds:
//! the task that renames them every third of the lease,
//! the way a control drives it,
//! and its stop.
//! Why:
//!  `service_claim_renewal.rs` holds what a renewal changes (the held claims and the renewer that renames them);
//! this file holds the task that runs it on the runtime,
//! which depends on that file and not the other way round.
//! The task waits on its channel and its timer with `.await`,
//! the renames run on the blocking pool,
//! and a renewal is stopped by a wait of the runtime,
//! bounded by `RENEWAL_STOP_CEILING`;
//! dropping one only tells its task to stop,
//!  because a synchronous wait on a task holds the worker the task needs to end.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const renewal = ClaimRenewal.start(renewer, every); renewal.stop();
//! ```

use crate::blocking_job::run_blocking_job;
use crate::claim_time::ClaimTime;
use crate::lazy_runtime::spawn_on_runtime;
use crate::service_claim_renewal::Renewer;
use std::sync::OnceLock;
use std::sync::mpsc::{SyncSender, sync_channel};
use tokio::sync::oneshot::{Receiver, channel as oneshot_channel};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::Instant;

/// How long dropping a renewal waits for its task to end:
/// the wrapper's worst case with the owner's margin
/// (`foundation/worst_delay_seen_on_this_host.rs`,
/// "3x worst case":
/// three times the longest delay of a check and a rename seen in any measurement,
/// and a renewal is one rename of a claim file).
/// A task still inside a rename after it is left behind;
/// the claim then expires by its lease,
/// and the rename it finishes later finds the claim moved on and changes nothing (a renewal renames only from the name it last
/// wrote).
/// Its own time is measured in `service_claim_renewal_tests.rs`.
pub const RENEWAL_STOP_CEILING: Duration = crate::worst_delay_seen_on_this_host::WORST_CASE_WITH_MARGIN;

/// What a controller tells the renewal task.
pub(crate) enum Control {
    /// Renew every claim held now at this reading,
    /// and say so on the channel when done,
    /// if one is given.
    Renew(ClaimTime, Option<SyncSender<()>>),
    /// Stop the task.
    Stop,
}

/// How the claims of one drain are kept fresh.
#[derive(Clone, Copy, Debug)]
pub enum Renewal<'a> {
    /// A thread of the drain renews every third of the lease by the real clock.
    Timed,
    /// Nothing renews until the driver is told to,
    /// with the reading it is given;
    /// a control uses this to be independent of how
    /// the host schedules threads,
    /// and to let any number of leases pass without waiting.
    Driven(&'a DrivenRenewal),
}

/// What:
///  The way a control renews the claims of a drain it started,
/// at the reading it chooses.
/// Why:
///   The renewal thread's timing is real time,
/// and a control of what a renewal protects cannot be told apart from a slow host by waiting.
/// A driven renewal does the same renaming at moments the control chooses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const driver = new DrivenRenewal(); drain(..., { driven: driver }); driver.renew(clockAfterTwoLeases);
/// ```
#[derive(Debug, Default)]
pub struct DrivenRenewal {
    /// The way to the renewal thread of the drain that registered,
    /// once it has.
    /// A driver serves one drain.
    ticks: OnceLock<UnboundedSender<Control>>,
}

/// Methods over a driver.
impl DrivenRenewal {
    /// Register the renewal thread of a drain;
    /// the first drain to register is the one the driver serves.
    fn attach(&self, control: UnboundedSender<Control>) {
        let _first: Result<(), UnboundedSender<Control>> = self.ticks.set(control);
    }

    /// Renew every claim the registered drain holds at the reading `now`,
    /// and return once that is done;
    /// false when no drain has registered or its thread is gone.
    pub fn renew(&self, now: &ClaimTime) -> bool {
        let Some(control) = self.ticks.get() else { return false };
        let (done, finished) = sync_channel::<()>(1);
        return control.send(Control::Renew(now.clone(), Some(done))).is_ok() && finished.recv().is_ok();
    }
}

/// How many times a running renewal has renewed its claims,
/// for the controls;
/// nothing in the release executable.
/// Why:
///   A control of "the claims this drain does not hold are left alone"
/// otherwise sleeps for a few intervals and cannot tell a renewal thread that left them alone
/// from one that never ran.
#[cfg(test)]
pub static RENEWALS_MADE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How many renewal tasks have ended,
/// for the controls;
/// nothing in the release executable.
/// Why:
///   A control of "the renewal is gone when it is dropped" otherwise waits and hopes,
/// where the task itself can say it returned.
#[cfg(test)]
pub static TASKS_ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How many renewal tasks have begun,
/// for the controls;
/// nothing in the release executable.
/// Why:
///   A task is spawned when its renewal is started and picked up by the runtime's workers after that,
/// so a control that counts the threads a
///  renewal adds otherwise reads a process whose renewals have not begun yet and passes.
#[cfg(test)]
pub static TASKS_STARTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// What:
///  A running renewal:
/// a thread that stamps every held claim again each interval until this value is dropped.
/// Why:
///   Dropping stops and joins the thread,
/// so a drain that returns or unwinds leaves no thread behind,
/// and a drain that ended
///  renews nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renewal = ClaimRenewal.start(renewer, every); renewal.stop();
/// ```
pub struct ClaimRenewal {
    /// The way to tell the task to stop.
    pub(crate) control: UnboundedSender<Control>,
    /// What the task sends when it has ended,
    /// taken by the face that waits for it;
    /// nothing when no task could be started.
    ended: Option<Receiver<()>>,
    /// How long a stop waits for the task to end.
    stop_ceiling: Duration,
}

/// Does what a controller asked of the renewal task;
/// the renewer after it,
/// or nothing when the task ends (a stop,
/// or a renewal that could not run).
/// The renaming is files of the work directory,
/// which can wait on storage,
/// so it runs on the blocking pool and the renewer goes there and back.
async fn obey(renewer: Renewer, control: Control) -> Option<Renewer> {
    return match control {
        Control::Renew(now, done) => {
            let renewed: Option<Renewer> = renew_off_the_workers(renewer, now).await;
            if let Some(finished) = done {
                let _told = finished.send(());
            }
            renewed
        }
        Control::Stop => None,
    };
}

/// Renews every claim the renewer holds at the reading `now` on the blocking pool;
/// nothing back when the pool could not run it,
/// which ends the renewal:
/// the claims then expire with their lease,
/// as for a service that died.
async fn renew_off_the_workers(mut renewer: Renewer, now: ClaimTime) -> Option<Renewer> {
    return match run_blocking_job(move || {
        renewer.renew(&now);
        return renewer;
    })
    .await
    {
        Ok(renewed) => {
            // A pass of the renewals has ended, which a control waits for instead of guessing at the host's speed.
            #[cfg(test)]
            RENEWALS_MADE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(renewed)
        }
        Err(error) => {
            crate::service_error!("claims could not be renewed, so they expire with their lease: {error}");
            None
        }
    };
}

/// What:
///  The task of one renewal:
/// obeys what the controller sends,
/// and,
/// for a timed renewal,
/// renews by the real clock every `every`.
/// Why:
///  The task waits on the channel and on the timer with `.await`,
/// so a renewal holds no thread while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const request = await Promise.race([requests.next(), timer(every)]); if (request === STOP) return; renew(request ?? clockNow()); }
/// ```
async fn renew_until_stopped(mut renewer: Renewer, mut requests: UnboundedReceiver<Control>, every: Option<Duration>) {
    // The task says it began, which a control waits for before it reads what the renewal costs the process.
    #[cfg(test)]
    TASKS_STARTED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut due: Option<Instant> = every.map(due_after);
    loop {
        // What the task woke for: `Some(request)` when a controller asked (nothing when every controller is gone), `None` when the
        //  interval of a timed renewal passed.
        let woke_for: Option<Option<Control>> = match due {
            Some(at) => tokio::select! {
                request = requests.recv() => Some(request),
                () = tokio::time::sleep_until(at) => None,
            },
            None => Some(requests.recv().await),
        };
        let renewed: Option<Renewer> = match woke_for {
            // Every controller is gone: nothing asks for anything any more.
            Some(None) => return,
            Some(Some(request)) => obey(renewer, request).await,
            // A claim that was settled is no longer held; one that was taken over is found gone and marked.
            None => {
                let after: Option<Renewer> = renew_off_the_workers(renewer, ClaimTime::read()).await;
                due = every.map(due_after);
                after
            }
        };
        let Some(kept) = renewed else { return };
        renewer = kept;
    }
}

/// The moment a renewal that began now is due again after `interval`.
fn due_after(interval: Duration) -> Instant {
    return Instant::now() + interval;
}

/// Starts the task of a renewal on the process's runtime;
/// nothing back when the runtime has no worker to run it,
/// where the claims expire with their lease,
/// as for a service that died.
fn start_task(renewer: Renewer, requests: UnboundedReceiver<Control>, every: Option<Duration>) -> Option<Receiver<()>> {
    let (ended_sender, ended) = oneshot_channel::<()>();
    let started = spawn_on_runtime(async move {
        renew_until_stopped(renewer, requests, every).await;
        // The task says it returned before it says so on the channel the drop waits on, so a control can read that it ended.
        #[cfg(test)]
        TASKS_ENDED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let _told = ended_sender.send(());
    });
    if started.is_none() {
        crate::service_error!("no worker to renew claims on, so they expire with their lease");
    }
    return if started.is_some() { Some(ended) } else { None };
}

/// Methods over a renewal.
impl ClaimRenewal {
    /// Start renewing the claims the drain tells `renewer` of,
    /// every `every`,
    /// by the real clock.
    pub fn start(renewer: Renewer, every: Duration) -> ClaimRenewal {
        let (control, requests) = unbounded_channel::<Control>();
        let ended: Option<Receiver<()>> = start_task(renewer, requests, Some(every));
        return ClaimRenewal { control, ended, stop_ceiling: RENEWAL_STOP_CEILING };
    }

    /// Start the task of a driven renewal:
    /// it renews only when `driver` is told to,
    /// and registers with it.
    pub fn driven(renewer: Renewer, driver: &DrivenRenewal) -> ClaimRenewal {
        let (control, requests) = unbounded_channel::<Control>();
        driver.attach(control.clone());
        let ended: Option<Receiver<()>> = start_task(renewer, requests, None);
        return ClaimRenewal { control, ended, stop_ceiling: RENEWAL_STOP_CEILING };
    }

    /// `driven` with the wait of the stop set by the control,
    /// so a control shows the wait is bounded in a moment.
    #[cfg(test)]
    pub(crate) fn driven_with_stop_ceiling(renewer: Renewer, driver: &DrivenRenewal, stop_ceiling: Duration) -> ClaimRenewal {
        let mut renewal: ClaimRenewal = ClaimRenewal::driven(renewer, driver);
        renewal.stop_ceiling = stop_ceiling;
        return renewal;
    }

    /// What:
    ///  Stop the renewal and wait for its task,
    ///  for at most the stop ceiling,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   A renewal in the middle of its renames finishes them first,
    /// as the thread it replaced was joined,
    ///  and a wait that does not end is never kept
    /// ("nothing should ever wait forever").
    /// The wait is the runtime's timer,
    /// so the worker that waits holds nothing while the task it waits for runs:
    /// a synchronous wait on a task from inside the runtime would hold the very worker the task needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await renewal.stopAwaiting();
    /// ```
    pub async fn stop_awaiting(mut self) {
        let _told = self.control.send(Control::Stop);
        let Some(ended) = self.ended.take() else { return };
        match tokio::time::timeout(self.stop_ceiling, ended).await {
            // A task that panicked has nothing left to report (its sender is dropped, which ends this wait): the claims it held
            //  expire with the lease.
            Ok(_answered) => {}
            Err(_elapsed) => {
                crate::service_warn!("the renewal of claims did not stop within {:?}; it is left behind and the claims expire with their lease", self.stop_ceiling);
            }
        }
    }

    /// Stop the renewal and wait for its task from a caller outside the runtime,
    /// entering the runtime once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// renewal.stopOnTheCallingThread();
    /// ```
    pub fn stop_on_the_calling_thread(self) {
        match crate::lazy_runtime::run_on_runtime(self.stop_awaiting()) {
            Ok(()) => {}
            // The runtime refused the wait, so the task is told to stop and left to end on its own, which is what a drop does.
            Err(refusal) => crate::service_warn!("the renewal of claims was not waited for: {refusal}"),
        }
    }
}

/// Tells the task to stop when the renewal goes out of scope,
/// and waits for nothing.
/// Why:
///   A synchronous wait on a task holds the thread that waits,
/// and a drain that is a task would hold a worker the task it waits for needs to end;
/// so the ending is waited for by `stop_awaiting` or `stop_on_the_calling_thread`,
/// and a renewal nobody stopped leaves a task that reads its stop and ends on its own,
/// whose claims expire by their lease if it cannot.
impl Drop for ClaimRenewal {
    fn drop(&mut self) {
        let _told = self.control.send(Control::Stop);
    }
}
