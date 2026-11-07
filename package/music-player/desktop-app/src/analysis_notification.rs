//! The true-peak analysis indicator (D107):
//! a Plasma job,
//! or else a freedesktop notification,
//! never an in-app banner.
//!
//! `AnalysisNotifier` is the handle the app keeps.
//! `report`,
//! `fail` and `shutdown` only post a message to a dedicated thread,
//! so a slow or wedged bus can never block
//! the UI thread or the analysis workers.
//! That thread owns the D-Bus connection and the state machine from `analysis_notification_logic`,
//!  and carries out what the machine returns with the calls in `analysis_notification_wire`.
//!
//! - On KDE (`org.kde.kuiserver` present) the analysis is a Plasma job:
//!   app name and icon in the header,
//!   the title,
//!   the count as the info line,
//!   a determinate bar,
//!   and a native Pause
//!   button because the job is suspendable (never killable:
//!   D84 makes analysis non-optional).
//! - Elsewhere it is one freedesktop notification replaced in place,
//!   with the `value` hint as the bar and Pause as an
//!   action button.
//! - If neither answers,
//!   one warning is logged,
//!   the analysis runs without an indicator,
//!   and the bus is not tried
//!   again for the rest of the run.
//!
//! Pause requests from the indicator are passed to `NotifierConfig::on_pause` on the notifier thread;
//! the callback must return quickly (set a flag and unpark a worker,
//! for example).
//! Off Linux the handle exists and does nothing.

/// Imports the channel module whose `channel` function creates the worker's input queue,
/// and the sender that the handle posts through.
#[cfg(target_os = "linux")]
use std::sync::mpsc::{self, Sender};
/// Imports the shared ownership and lock types that let every clone of the notifier reach one worker.
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
/// Imports the handle through which the last notifier clone joins the worker thread.
#[cfg(target_os = "linux")]
use std::thread::JoinHandle;
/// Imports the duration type that bounds every call that waits for a reply.
#[cfg(target_os = "linux")]
use std::time::Duration;

/// Imports the D-Bus session connection that the indicator's calls travel over.
#[cfg(target_os = "linux")]
use zbus::Connection;

/// Imports the thread's input type and state,
/// which the handle posts to and starts.
#[cfg(target_os = "linux")]
use crate::analysis_notification_worker::{Input, Worker};

/// What:
/// `METHOD_TIMEOUT` bounds every call that waits for a reply.
/// Why:
/// a service that accepts a call and never answers must not hold the notifier thread (and a shutdown that joins
/// it) for the bus default of twenty-five seconds.
#[cfg(target_os = "linux")]
pub(crate) const METHOD_TIMEOUT: Duration = Duration::from_secs(3);

/// What:
/// `Identity` names the app to the desktop.
/// Why:
/// the header of a Plasma job and the sender of a notification show the app's name and icon;
/// the desktop entry
/// is the `.desktop` file name without its suffix,
/// and the name and icon fill in when that file is not installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    /// The desktop file name without `.desktop`,
    /// for example `monochromatic.music-player`.
    pub desktop_entry: String,
    /// The name shown in the header,
    /// for example `Music Player`.
    pub app_name: String,
    /// The icon name shown in the header,
    /// for example `multimedia-player`.
    pub app_icon: String,
}

/// What:
/// `NotifierConfig` is everything the notifier thread needs.
/// Why:
/// the connection is made on that thread,
/// so a test can aim it at a private bus.
pub struct NotifierConfig {
    /// The app's identity for the desktop.
    pub identity: Identity,
    /// True when the app can pause the analysis;
    /// false offers no Pause button at all.
    pub suspendable: bool,
    /// Called with `true` to pause and `false` to resume the analysis;
    /// runs on the notifier thread.
    pub on_pause: Box<dyn Fn(bool) + Send + 'static>,
    /// Opens the session bus connection on the notifier thread.
    #[cfg(target_os = "linux")]
    pub connect: Box<dyn FnOnce() -> zbus::Result<Connection> + Send + 'static>,
}

/// Methods of `NotifierConfig`.
impl NotifierConfig {
    /// What:
    /// `session` builds a config that uses the user's session bus.
    /// Why:
    /// this is what the app passes;
    /// the bus address comes from the environment as for every D-Bus client.
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

/// What:
/// `connect_session` opens a connection to the session bus with a bounded connect and a bounded reply wait.
/// Why:
/// the default wait is long;
/// a bounded one keeps a stalled service from stalling the notifier thread.
/// The unplayable-file notifier (D122) opens its own connection the same way.
#[cfg(target_os = "linux")]
pub(crate) fn connect_session() -> zbus::Result<Connection> {
    return crate::dbus_step_bound::connect(zbus::connection::Builder::session(), METHOD_TIMEOUT);
}

/// What:
/// `Shared` is the part of the handle every clone points at.
/// Why:
/// the last clone to go away stops the thread;
/// `Drop` cannot run per clone.
#[cfg(target_os = "linux")]
struct Shared {
    /// Queue through which the app and the signal listeners send the thread its inputs.
    sender: Sender<Input>,
    /// Handle of the notifier thread,
    /// kept so that shutdown can join it.
    thread: Mutex<Option<JoinHandle<()>>>,
}

/// Methods of `Shared`.
#[cfg(target_os = "linux")]
impl Drop for Shared {
    /// What:
    /// `drop` asks the thread to close the indicator and stop,
    /// without waiting for it.
    /// Why:
    /// an app that forgets `shutdown` still leaves no job behind,
    /// and a destructor must not block on the bus.
    fn drop(&mut self) {
        if self.sender.send(Input::Stop).is_err() {
            tracing::debug!("analysis notifier thread already stopped at drop");
        }
    }
}

/// What:
/// `AnalysisNotifier` is the cloneable handle to the indicator.
/// Why:
/// the sweep's progress callback,
/// the pause callback and the shutdown path each need one.
#[cfg(target_os = "linux")]
#[derive(Clone)]
pub struct AnalysisNotifier {
    /// The part of the handle that every clone points at.
    shared: Arc<Shared>,
}

/// Methods of `AnalysisNotifier`.
#[cfg(target_os = "linux")]
impl AnalysisNotifier {
    /// What:
    /// `spawn` starts the notifier thread.
    /// Why:
    /// nothing touches the bus until the first unfinished report,
    /// so a fully cached library costs nothing.
    pub fn spawn(config: NotifierConfig) -> AnalysisNotifier {
        let (sender, receiver) = mpsc::channel::<Input>();
        let worker = Worker::new(config, sender.clone());
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

    /// What:
    /// `post` sends one input to the thread.
    /// Why:
    /// a stopped thread is not an error for the caller;
    /// the analysis simply has no indicator.
    fn post(&self, input: Input) {
        if self.shared.sender.send(input).is_err() {
            tracing::debug!("analysis notifier input dropped: the thread has stopped");
        }
    }

    /// What:
    /// `report` gives the sweep's progress:
    /// `done` tracks visited of `total`;
    /// `done >= total` means finished.
    /// Why:
    /// this is called from the engine's update path for every report,
    /// so it only posts a message.
    pub fn report(&self, done: usize, total: usize) {
        self.post(Input::Progress { done, total });
    }

    /// What:
    /// `fail` ends the indicator with an engine failure message.
    /// Why:
    /// the user is told why the analysis stopped.
    pub fn fail(&self, message: &str) {
        self.post(Input::Fail(message.to_string()));
    }

    /// What:
    /// `shutdown` closes the indicator,
    /// stops the thread and waits for it.
    /// Why:
    /// a job or notification must not outlive the app;
    /// the wait is bounded by `METHOD_TIMEOUT` per call.
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

/// What:
/// `AnalysisNotifier` off Linux is a handle that does nothing.
/// Why:
/// macOS and Windows get an indicator through their own notification APIs in a later step (D107),
/// and the app
/// code that holds the handle stays the same on every platform.
#[cfg(not(target_os = "linux"))]
#[derive(Clone)]
pub struct AnalysisNotifier;

/// Methods of the off-Linux `AnalysisNotifier`.
#[cfg(not(target_os = "linux"))]
impl AnalysisNotifier {
    /// What:
    /// `spawn` builds the empty handle.
    /// Why:
    /// it mirrors the Linux constructor.
    pub fn spawn(_config: NotifierConfig) -> AnalysisNotifier {
        return AnalysisNotifier;
    }

    /// What:
    /// `report` does nothing.
    /// Why:
    /// there is no indicator off Linux yet.
    pub fn report(&self, _done: usize, _total: usize) {}

    /// What:
    /// `fail` does nothing.
    /// Why:
    /// there is no indicator off Linux yet.
    pub fn fail(&self, _message: &str) {}

    /// What:
    /// `shutdown` does nothing.
    /// Why:
    /// there is nothing to close off Linux yet.
    pub fn shutdown(&self) {}
}
