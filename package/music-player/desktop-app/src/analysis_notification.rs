//! The true-peak analysis indicator (D107): a Plasma job, or else a freedesktop notification, never an in-app banner.
//!
//! `AnalysisNotifier` is the handle the app keeps.
//! `report`, `fail` and `shutdown` only post a message to a dedicated thread, so a slow or wedged bus can never block
//! the UI thread or the analysis workers.
//! That thread owns the D-Bus connection and the state machine from `analysis_notification_logic`,
//!  and carries out what the machine returns with the calls in `analysis_notification_wire`.
//!
//! - On KDE (`org.kde.kuiserver` present) the analysis is a Plasma job:
//!   app name and icon in the header, the title, the count as the info line, a determinate bar, and a native Pause
//!   button because the job is suspendable (never killable: D84 makes analysis non-optional).
//! - Elsewhere it is one freedesktop notification replaced in place, with the `value` hint as the bar and Pause as an
//!   action button.
//! - If neither answers, one warning is logged, the analysis runs without an indicator, and the bus is not tried
//!   again for the rest of the run.
//!
//! Pause requests from the indicator are passed to `NotifierConfig::on_pause` on the notifier thread;
//! the callback must return quickly (set a flag and unpark a worker, for example).
//! Off Linux the handle exists and does nothing.

#[cfg(target_os = "linux")]
use std::collections::VecDeque;
#[cfg(target_os = "linux")]
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
#[cfg(target_os = "linux")]
use std::sync::mpsc;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use std::thread::JoinHandle;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use zbus::blocking::Connection;

#[cfg(target_os = "linux")]
use crate::analysis_notification_logic::{Effect, IndicatorView, Machine, Outcome};
#[cfg(target_os = "linux")]
use crate::analysis_notification_wire::{
    ACTION_PAUSE, ACTION_RESUME, Backend, BusSignal, CLOSED_BY_EXPIRY, CLOSED_BY_USER, FdoNotification, JobView,
    KUISERVER, has_owner, spawn_listener,
};

/// What: `METHOD_TIMEOUT` bounds every call that waits for a reply.
/// Why: a service that accepts a call and never answers must not hold the notifier thread (and a shutdown that joins
/// it) for the bus default of twenty-five seconds.
#[cfg(target_os = "linux")]
pub(crate) const METHOD_TIMEOUT: Duration = Duration::from_secs(3);

/// What: `Identity` names the app to the desktop.
/// Why: the header of a Plasma job and the sender of a notification show the app's name and icon; the desktop entry
/// is the `.desktop` file name without its suffix, and the name and icon fill in when that file is not installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    /// The desktop file name without `.desktop`, for example `monochromatic.music-player`.
    pub desktop_entry: String,
    /// The name shown in the header, for example `Music Player`.
    pub app_name: String,
    /// The icon name shown in the header, for example `multimedia-player`.
    pub app_icon: String,
}

/// What: `NotifierConfig` is everything the notifier thread needs.
/// Why: the connection is made on that thread, so a test can aim it at a private bus.
pub struct NotifierConfig {
    /// The app's identity for the desktop.
    pub identity: Identity,
    /// True when the app can pause the analysis; false offers no Pause button at all.
    pub suspendable: bool,
    /// Called with `true` to pause and `false` to resume the analysis; runs on the notifier thread.
    pub on_pause: Box<dyn Fn(bool) + Send + 'static>,
    /// Opens the session bus connection on the notifier thread.
    #[cfg(target_os = "linux")]
    pub connect: Box<dyn FnOnce() -> zbus::Result<Connection> + Send + 'static>,
}

/// Methods of `NotifierConfig`.
impl NotifierConfig {
    /// What: `session` builds a config that uses the user's session bus.
    /// Why: this is what the app passes; the bus address comes from the environment as for every D-Bus client.
    pub fn session(identity: Identity, suspendable: bool, on_pause: Box<dyn Fn(bool) + Send + 'static>) -> NotifierConfig {
        return NotifierConfig {
            identity,
            suspendable,
            on_pause,
            #[cfg(target_os = "linux")]
            connect: Box::new(connect_session),
        };
    }
}

/// What: `connect_session` opens a blocking connection to the session bus with a bounded reply wait.
/// Why: the default wait is long; a bounded one keeps a stalled service from stalling the notifier thread.
/// The unplayable-file notifier (D122) opens its own connection the same way.
#[cfg(target_os = "linux")]
pub(crate) fn connect_session() -> zbus::Result<Connection> {
    return zbus::blocking::connection::Builder::session()?.method_timeout(METHOD_TIMEOUT).build();
}

/// What: `Input` is one message to the notifier thread.
/// Why: the app side and the signal listeners speak to the thread through one channel, so it has one wait point.
#[cfg(target_os = "linux")]
enum Input {
    Progress { done: usize, total: usize },
    Fail(String),
    Signal(BusSignal),
    Stop,
}

/// What: `Shared` is the part of the handle every clone points at.
/// Why: the last clone to go away stops the thread; `Drop` cannot run per clone.
#[cfg(target_os = "linux")]
struct Shared {
    sender: Sender<Input>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

/// Methods of `Shared`.
#[cfg(target_os = "linux")]
impl Drop for Shared {
    /// What: `drop` asks the thread to close the indicator and stop, without waiting for it.
    /// Why: an app that forgets `shutdown` still leaves no job behind, and a destructor must not block on the bus.
    fn drop(&mut self) {
        if self.sender.send(Input::Stop).is_err() {
            tracing::debug!("analysis notifier thread already stopped at drop");
        }
    }
}

/// What: `AnalysisNotifier` is the cloneable handle to the indicator.
/// Why: the sweep's progress callback, the pause callback and the shutdown path each need one.
#[cfg(target_os = "linux")]
#[derive(Clone)]
pub struct AnalysisNotifier {
    shared: Arc<Shared>,
}

/// What: `Worker` is the notifier thread's state.
/// Why: the machine, the connection, the open indicator and the one-warning flag live together on that thread.
#[cfg(target_os = "linux")]
struct Worker {
    config_identity: Identity,
    suspendable: bool,
    on_pause: Box<dyn Fn(bool) + Send + 'static>,
    connect: Option<Box<dyn FnOnce() -> zbus::Result<Connection> + Send + 'static>>,
    connection: Option<Connection>,
    events: Sender<Input>,
    machine: Machine,
    job: Option<JobView>,
    notification: Option<FdoNotification>,
    listening_job: bool,
    listening_notification: bool,
}

/// Methods of `AnalysisNotifier`.
#[cfg(target_os = "linux")]
impl AnalysisNotifier {
    /// What: `spawn` starts the notifier thread.
    /// Why: nothing touches the bus until the first unfinished report, so a fully cached library costs nothing.
    pub fn spawn(config: NotifierConfig) -> AnalysisNotifier {
        let (sender, receiver) = mpsc::channel::<Input>();
        let worker = Worker {
            config_identity: config.identity,
            suspendable: config.suspendable,
            on_pause: config.on_pause,
            connect: Some(config.connect),
            connection: None,
            events: sender.clone(),
            machine: Machine::new(config.suspendable),
            job: None,
            notification: None,
            listening_job: false,
            listening_notification: false,
        };
        let spawned = std::thread::Builder::new()
            .name("analysis-notification".to_string())
            .spawn(move || worker.run(&receiver));
        let thread = match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                tracing::warn!(error = %error, "analysis notifier thread could not start; analysis runs without an indicator");
                None
            }
        };
        tracing::debug!("analysis notifier started");
        return AnalysisNotifier {
            shared: Arc::new(Shared {
                sender,
                thread: Mutex::new(thread),
            }),
        };
    }

    /// What: `post` sends one input to the thread.
    /// Why: a stopped thread is not an error for the caller; the analysis simply has no indicator.
    fn post(&self, input: Input) {
        if self.shared.sender.send(input).is_err() {
            tracing::debug!("analysis notifier input dropped: the thread has stopped");
        }
    }

    /// What: `report` gives the sweep's progress: `done` tracks visited of `total`; `done >= total` means finished.
    /// Why: this is called from the engine's update path for every report, so it only posts a message.
    pub fn report(&self, done: usize, total: usize) {
        self.post(Input::Progress { done, total });
    }

    /// What: `fail` ends the indicator with an engine failure message.
    /// Why: the user is told why the analysis stopped.
    pub fn fail(&self, message: &str) {
        self.post(Input::Fail(message.to_string()));
    }

    /// What: `shutdown` closes the indicator, stops the thread and waits for it.
    /// Why: a job or notification must not outlive the app; the wait is bounded by `METHOD_TIMEOUT` per call.
    pub fn shutdown(&self) {
        self.post(Input::Stop);
        let taken = match self.shared.thread.lock() {
            Ok(mut guard) => guard.take(),
            Err(error) => {
                tracing::warn!(error = %error, "analysis notifier thread handle lock was poisoned");
                None
            }
        };
        if let Some(handle) = taken
            && handle.join().is_err()
        {
            tracing::warn!("analysis notifier thread panicked");
        }
    }
}

/// Methods of `Worker`.
#[cfg(target_os = "linux")]
impl Worker {
    /// What: `run` is the thread body: wait for an input or the held view's deadline, apply it, repeat.
    /// Why: one wait point serves progress, signals and the rate limit's trailing flush.
    fn run(mut self, receiver: &Receiver<Input>) {
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

    /// What: `on_signal` turns a button press or a dismissal into machine input.
    /// Why: only the sender and the job path or notification id this thread opened may act on the analysis.
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

    /// What: `apply` carries out effects in order; a bus failure drops the rest and disables the indicator.
    /// Why: once a call fails the remaining ones would fail the same way, and only the pause release still matters.
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

    /// What: `connection` returns the session connection, making it on first use.
    /// Why: the connection is made once; a failure is final for the run.
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

    /// What: `ensure_listener` starts the signal listener of `backend` once.
    /// Why: it must be subscribed before the first request so an early button press is not lost.
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

    /// What: `open` creates the indicator: a Plasma job if the job server is on the bus, else a notification.
    /// Why: Plasma's own notification server ignores the progress hint, so on KDE the job is the only real bar.
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

    /// What: `update` sends `view` to whichever indicator is open.
    /// Why: the machine only returns an update while open, so a missing indicator means a dismissal raced it.
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

    /// What: `close` ends whichever indicator is open and forgets it.
    /// Why: the next sweep opens a fresh one.
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

/// What: `AnalysisNotifier` off Linux is a handle that does nothing.
/// Why: macOS and Windows get an indicator through their own notification APIs in a later step (D107), and the app
/// code that holds the handle stays the same on every platform.
#[cfg(not(target_os = "linux"))]
#[derive(Clone)]
pub struct AnalysisNotifier;

/// Methods of the off-Linux `AnalysisNotifier`.
#[cfg(not(target_os = "linux"))]
impl AnalysisNotifier {
    /// What: `spawn` builds the empty handle.
    /// Why: it mirrors the Linux constructor.
    pub fn spawn(_config: NotifierConfig) -> AnalysisNotifier {
        return AnalysisNotifier;
    }

    /// What: `report` does nothing.
    /// Why: there is no indicator off Linux yet.
    pub fn report(&self, _done: usize, _total: usize) {}

    /// What: `fail` does nothing.
    /// Why: there is no indicator off Linux yet.
    pub fn fail(&self, _message: &str) {}

    /// What: `shutdown` does nothing.
    /// Why: there is nothing to close off Linux yet.
    pub fn shutdown(&self) {}
}
