//! The notifier thread of the true-peak analysis indicator (D107),
//! Linux only.
//!
//! What:
//!  `Input` is the message the app and the signal listeners send the thread,
//!  and `Worker` is the thread's state and body:
//!  it waits for an input or the held view's deadline,
//!  feeds the state machine of `analysis_notification_logic`,
//!  and carries out the effects with the calls of `analysis_notification_wire`.
//! Why:
//!  The handle and its configuration in `analysis_notification.rs` stay small and readable,
//!  and the thread body,
//! which is the part that talks to the bus,
//!  lives in one file of its own.

/// Imports the queue that holds the indicator updates waiting for the next send slot.
use std::collections::VecDeque;

/// Imports the channel ends and the receive error through which the worker thread takes its inputs.
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};

/// Imports the instant type that paces the indicator updates.
use std::time::Instant;

/// Imports the D-Bus session connection that the indicator's calls travel over.
use zbus::Connection;

/// Imports the identity and the configuration that the thread is started from.
use crate::analysis_notification::{Identity, NotifierConfig};

/// Imports the pure state machine and the effects,
/// views and outcomes it exchanges with this thread.
use crate::analysis_notification_logic::{Effect, IndicatorView, Machine, Outcome};

/// Imports the D-Bus names,
/// signals and call wrappers that carry the indicator over the bus.
use crate::analysis_notification_wire::{
    ACTION_PAUSE, ACTION_RESUME, Backend, BusSignal, CLOSED_BY_EXPIRY, CLOSED_BY_USER, FdoNotification, JobView, KUISERVER,
    has_owner, spawn_listener,
};

/// What:
/// `Input` is one message to the notifier thread.
/// Why:
/// the app side and the signal listeners speak to the thread through one channel,
/// so it has one wait point.
pub(crate) enum Input {
    /// The sweep reported progress.
    Progress {
        /// Tracks visited so far.
        done: usize,
        /// Tracks the sweep will visit.
        total: usize,
    },
    /// The engine failed.
    Fail(
        /// The message to show the user.
        String,
    ),
    /// A listener read a bus signal.
    Signal(
        /// The signal that the listener read.
        BusSignal,
    ),
    /// Close the indicator and end the thread.
    Stop,
}

/// What:
/// `Worker` is the notifier thread's state.
/// Why:
/// the machine,
/// the connection,
/// the open indicator and the one-warning flag live together on that thread.
pub(crate) struct Worker {
    /// The names with which the player introduces itself to the Plasma job and the notification service.
    config_identity: Identity,
    /// Whether the app has a pause to offer,
    /// which decides whether the indicator shows a Pause button.
    suspendable: bool,
    /// Pauses (`true`) or resumes (`false`) the engine's analysis when the user presses the indicator's button.
    on_pause: Box<dyn Fn(bool) + Send + 'static>,
    /// Opens the session bus connection;
    /// taken on first use so that the connect happens once,
    /// on this thread.
    connect: Option<Box<dyn FnOnce() -> zbus::Result<Connection> + Send + 'static>>,
    /// The open session bus connection,
    /// once the connect succeeded.
    connection: Option<Connection>,
    /// Sender into this thread's own queue,
    /// handed to the signal listeners.
    events: Sender<Input>,
    /// The pure state machine that decides what the indicator shows and when.
    machine: Machine,
    /// The Plasma job,
    /// while the job backend shows the indicator.
    job: Option<JobView>,
    /// The freedesktop notification,
    /// while the notification backend shows the indicator.
    notification: Option<FdoNotification>,
    /// Whether a listener for the job's signals is already running,
    /// so that it starts once.
    listening_job: bool,
    /// Whether a listener for the notification's signals is already running,
    /// so that it starts once.
    listening_notification: bool,
}

/// Methods of `Worker`.
impl Worker {
    /// What:
    /// `new` builds the thread's state from the configuration and the sender into its own queue.
    /// Why:
    /// the fields are private to this file,
    /// so the handle in `analysis_notification.rs` starts the thread through this one call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const worker = new Worker(config, events);
    /// ```
    pub(crate) fn new(config: NotifierConfig, events: Sender<Input>) -> Worker {
        return Worker {
            config_identity: config.identity,
            suspendable: config.suspendable,
            on_pause: config.on_pause,
            connect: Some(config.connect),
            connection: None,
            events,
            machine: Machine::new(config.suspendable),
            job: None,
            notification: None,
            listening_job: false,
            listening_notification: false,
        };
    }

    /// What:
    /// `run` is the thread body:
    /// wait for an input or the held view's deadline,
    /// apply it,
    /// repeat.
    /// Why:
    /// one wait point serves progress,
    /// signals and the rate limit's trailing flush.
    pub(crate) fn run(mut self, receiver: &Receiver<Input>) {
        tracing::debug!("analysis notifier thread running");
        loop {
            let received = match self.machine.deadline() {
                Some(due) => receiver.recv_timeout(due.saturating_duration_since(Instant::now())),
                None => receiver.recv().map_err(|_| return RecvTimeoutError::Disconnected),
            };
            let now = Instant::now();
            let effects = match received {
                Ok(Input::Progress { done, total }) => self.machine.progress(now, done, total),
                Ok(Input::Fail(message)) => self.machine.failed(message),
                Ok(Input::Signal(signal)) => self.on_signal(now, &signal),
                Err(RecvTimeoutError::Timeout) => self.machine.tick(now),
                Ok(Input::Stop) | Err(RecvTimeoutError::Disconnected) => {
                    let closing = self.machine.stop();
                    self.apply(closing);
                    break;
                }
            };
            self.apply(effects);
        }
        // The connection is not closed explicitly: closing it while a listener holds a message stream makes zbus log a
        // warning about the match rule it can no longer remove, and the process is about to exit anyway.
        tracing::debug!("analysis notifier thread stopped");
    }

    /// What:
    /// `on_signal` turns a button press or a dismissal into machine input.
    /// Why:
    /// only the sender and the job path or notification id this thread opened may act on the analysis.
    fn on_signal(&mut self, now: Instant, signal: &BusSignal) -> Vec<Effect> {
        match signal {
            BusSignal::JobSuspend { sender, path } | BusSignal::JobResume { sender, path } => {
                let ours = self.job.as_ref().is_some_and(|job| return job.owns(sender, path));
                if !ours {
                    tracing::debug!(sender = %sender, path = %path, "job signal ignored: not our job");
                    return Vec::new();
                }
                let pause = matches!(signal, BusSignal::JobSuspend { .. });
                return self.machine.set_paused(now, pause);
            }
            BusSignal::ActionInvoked { sender, id, key } => {
                let ours = self.notification.as_ref().is_some_and(|open| return open.owns(sender, *id));
                if !ours {
                    tracing::debug!(id, "action ignored: not our notification");
                    return Vec::new();
                }
                return match key.as_str() {
                    ACTION_PAUSE => self.machine.set_paused(now, true),
                    ACTION_RESUME => self.machine.set_paused(now, false),
                    other => {
                        tracing::debug!(action = other, "notification action ignored: unknown key");
                        Vec::new()
                    }
                };
            }
            BusSignal::NotificationClosed { sender, id, reason } => {
                let ours = self.notification.as_ref().is_some_and(|open| return open.owns(sender, *id));
                if !ours || (*reason != CLOSED_BY_USER && *reason != CLOSED_BY_EXPIRY) {
                    tracing::debug!(id, reason, "close signal ignored: not ours or not a user action");
                    return Vec::new();
                }
                self.notification = None;
                return self.machine.dismissed();
            }
        }
    }

    /// What:
    /// `apply` carries out effects in order;
    /// a bus failure drops the rest and disables the indicator.
    /// Why:
    /// once a call fails the remaining ones would fail the same way,
    /// and only the pause release still matters.
    fn apply(&mut self, effects: Vec<Effect>) {
        let mut queue: VecDeque<Effect> = VecDeque::from(effects);
        while let Some(effect) = queue.pop_front() {
            let outcome = match effect {
                Effect::SetEnginePause(paused) => {
                    tracing::debug!(paused, "asking the engine to change the analysis pause");
                    (self.on_pause)(paused);
                    Ok(())
                }
                Effect::Open(view) => self.open(&view),
                Effect::Update(view) => self.update(&view),
                Effect::Close(result) => self.close(&result),
            };
            if let Err(error) = outcome {
                tracing::warn!(
                    error = %error,
                    "analysis progress indicator unavailable (no notification service answered, or a call failed); analysis runs without one for the rest of this run"
                );
                self.job = None;
                self.notification = None;
                let mut kept: VecDeque<Effect> = queue
                    .into_iter()
                    .filter(|left| return matches!(left, Effect::SetEnginePause(_)))
                    .collect();
                kept.extend(self.machine.bus_failed());
                queue = kept;
            }
        }
    }

    /// What:
    /// `connection` returns the session connection,
    /// making it on first use.
    /// Why:
    /// the connection is made once;
    /// a failure is final for the run.
    fn connection(&mut self) -> zbus::Result<Connection> {
        if let Some(existing) = self.connection.as_ref() {
            return Ok(existing.clone());
        }
        let Some(connect) = self.connect.take() else {
            return Err(zbus::Error::Failure("the session bus connection was already attempted".to_string()));
        };
        let made = connect()?;
        tracing::info!("analysis notifier connected to the session bus");
        self.connection = Some(made.clone());
        return Ok(made);
    }

    /// What:
    /// `ensure_listener` starts the signal listener of `backend` once.
    /// Why:
    /// it must be subscribed before the first request so an early button press is not lost.
    fn ensure_listener(&mut self, connection: &Connection, backend: Backend) -> zbus::Result<()> {
        let already = match backend {
            Backend::Job => self.listening_job,
            Backend::Notification => self.listening_notification,
        };
        if already {
            return Ok(());
        }
        let events = self.events.clone();
        spawn_listener(connection, backend, move |signal| return events.send(Input::Signal(signal)).is_ok())?;
        match backend {
            Backend::Job => self.listening_job = true,
            Backend::Notification => self.listening_notification = true,
        }
        return Ok(());
    }

    /// What:
    /// `open` creates the indicator:
    /// a Plasma job if the job server is on the bus,
    /// else a notification.
    /// Why:
    /// Plasma's own notification server ignores the progress hint,
    /// so on KDE the job is the only real bar.
    fn open(&mut self, view: &IndicatorView) -> zbus::Result<()> {
        let connection = self.connection()?;
        if has_owner(&connection, KUISERVER)? {
            self.ensure_listener(&connection, Backend::Job)?;
            match JobView::open(&connection, &self.config_identity, self.suspendable, view) {
                Ok(job) => {
                    self.job = Some(job);
                    return Ok(());
                }
                Err(error) => {
                    tracing::info!(error = %error, "job server refused the analysis job; trying a notification");
                }
            }
        } else {
            tracing::info!("no job server on the session bus; trying a notification");
        }
        self.ensure_listener(&connection, Backend::Notification)?;
        let posted = FdoNotification::open(&connection, &self.config_identity, self.suspendable, view)?;
        self.notification = Some(posted);
        return Ok(());
    }

    /// What:
    /// `update` sends `view` to whichever indicator is open.
    /// Why:
    /// the machine only returns an update while open,
    /// so a missing indicator means a dismissal raced it.
    fn update(&mut self, view: &IndicatorView) -> zbus::Result<()> {
        if let Some(job) = self.job.as_ref() {
            return job.update(view, false);
        }
        if let Some(open) = self.notification.as_mut() {
            return open.update(&self.config_identity, self.suspendable, view);
        }
        tracing::debug!("analysis update dropped: no indicator is open");
        return Ok(());
    }

    /// What:
    /// `close` ends whichever indicator is open and forgets it.
    /// Why:
    /// the next sweep opens a fresh one.
    fn close(&mut self, outcome: &Outcome) -> zbus::Result<()> {
        if let Some(job) = self.job.take() {
            return job.terminate(outcome);
        }
        if let Some(open) = self.notification.take() {
            return open.close();
        }
        tracing::debug!("analysis close dropped: no indicator is open");
        return Ok(());
    }
}
