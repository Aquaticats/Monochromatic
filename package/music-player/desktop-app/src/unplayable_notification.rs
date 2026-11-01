//! The "cannot be played" report as a system notification (D122), never a message in the window.
//!
//! `UnplayableNotifier` is the handle the app keeps.
//! `report` and `shutdown` only post a message to a dedicated thread, so a slow or missing notification service can
//! never block the engine thread that reports the files or the window.
//! That thread owns its own session bus connection, made on the first report that has something new to say,
//! so a library with no broken file costs no connection.
//! It takes the wording and the "told before" rule from `unplayable_notification_logic` and makes the call in
//! `unplayable_notification_wire`.
//!
//! - If the bus or the notification service does not answer, one warning is logged and nothing is drawn in the window;
//!   the row mark still shows the file.
//!   Later reports try again, because a service that was not up yet at launch may be up at the next report.
//! - The analysis indicator (D107) has its own thread and connection with its own failure rule, so a failure of one
//!   never disables the other.
//!
//! Off Linux the handle exists and does nothing; macOS and Windows have no notification code yet (D122).

#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use std::thread::JoinHandle;

#[cfg(target_os = "linux")]
use zbus::blocking::Connection;

use crate::analysis_notification::Identity;
#[cfg(target_os = "linux")]
use crate::unplayable_notification_logic::{Ledger, Notice};

/// What: `UnplayableConfig` is everything the notifier thread needs.
/// Why: the connection is made on that thread, so a test can aim it at a private bus.
pub struct UnplayableConfig {
    /// The app's identity for the desktop, the same value the analysis indicator uses.
    pub identity: Identity,
    /// Opens a session bus connection on the notifier thread; called again after a failure.
    #[cfg(target_os = "linux")]
    pub connect: Box<dyn Fn() -> zbus::Result<Connection> + Send + 'static>,
}

/// Methods of `UnplayableConfig`.
impl UnplayableConfig {
    /// What: `session` builds a config that uses the user's session bus.
    /// Why: this is what the app passes; the bus address comes from the environment as for every D-Bus client.
    pub fn session(identity: Identity) -> UnplayableConfig {
        return UnplayableConfig {
            identity,
            #[cfg(target_os = "linux")]
            connect: Box::new(crate::analysis_notification::connect_session),
        };
    }
}

/// What: `Input` is one message to the notifier thread.
/// Why: the app side speaks to the thread through one channel, so the thread has one wait point.
#[cfg(target_os = "linux")]
enum Input {
    Report(Vec<PathBuf>),
    Stop,
}

/// What: `Shared` is the part of the handle every clone points at.
/// Why: the last clone to go away drops the only sender, which ends the thread after it has handled the reports
/// already posted; `Drop` cannot run per clone, and a destructor must not block on the bus.
#[cfg(target_os = "linux")]
struct Shared {
    sender: Sender<Input>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

/// What: `UnplayableNotifier` is the cloneable handle to the notification.
/// Why: the engine's update callback needs one, and the shutdown path needs another.
#[cfg(target_os = "linux")]
#[derive(Clone)]
pub struct UnplayableNotifier {
    shared: Arc<Shared>,
}

/// What: `Worker` is the notifier thread's state.
/// Why: the connection, the told-before ledger and the one-warning flag live together on that thread.
#[cfg(target_os = "linux")]
struct Worker {
    identity: Identity,
    connect: Box<dyn Fn() -> zbus::Result<Connection> + Send + 'static>,
    connection: Option<Connection>,
    ledger: Ledger,
    warned: bool,
}

/// Methods of `UnplayableNotifier`.
#[cfg(target_os = "linux")]
impl UnplayableNotifier {
    /// What: `spawn` starts the notifier thread.
    /// Why: nothing touches the bus until a report has a file not told before.
    pub fn spawn(config: UnplayableConfig) -> UnplayableNotifier {
        let (sender, receiver) = mpsc::channel::<Input>();
        let worker = Worker {
            identity: config.identity,
            connect: config.connect,
            connection: None,
            ledger: Ledger::new(),
            warned: false,
        };
        let spawned = std::thread::Builder::new()
            .name("unplayable-notification".to_string())
            .spawn(move || worker.run(&receiver));
        let thread = match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                tracing::warn!(error = %error, "unplayable notifier thread could not start; broken files are marked on their rows only");
                None
            }
        };
        tracing::debug!("unplayable notifier started");
        return UnplayableNotifier {
            shared: Arc::new(Shared { sender, thread: Mutex::new(thread) }),
        };
    }

    /// What: `report` gives the files that could not be played; they become one notification unless all were told.
    /// Why: this is called on the engine thread for every report, so it only copies the paths and posts a message.
    pub fn report(&self, paths: &[PathBuf]) {
        if self.shared.sender.send(Input::Report(paths.to_vec())).is_err() {
            tracing::debug!(files = paths.len(), "unplayable report dropped: the notifier thread has stopped");
        }
    }

    /// What: `shutdown` stops the thread after it has handled the reports already posted, and waits for it.
    /// Why: a report posted just before the app closes should still reach the desktop; the wait is bounded by the
    /// method timeout of the one call that may be in flight.
    pub fn shutdown(&self) {
        if self.shared.sender.send(Input::Stop).is_err() {
            tracing::debug!("unplayable notifier thread already stopped at shutdown");
        }
        let taken = match self.shared.thread.lock() {
            Ok(mut guard) => guard.take(),
            Err(error) => {
                tracing::warn!(error = %error, "unplayable notifier thread handle lock was poisoned");
                None
            }
        };
        if let Some(handle) = taken
            && handle.join().is_err()
        {
            tracing::warn!("unplayable notifier thread panicked");
        }
    }
}

/// Methods of `Worker`.
#[cfg(target_os = "linux")]
impl Worker {
    /// What: `run` is the thread body: wait for a report, handle it, repeat until told to stop.
    /// Why: one wait point; reports are handled in the order they were posted.
    fn run(mut self, receiver: &Receiver<Input>) {
        tracing::debug!("unplayable notifier thread running");
        loop {
            match receiver.recv() {
                Ok(Input::Report(paths)) => self.handle(&paths),
                Ok(Input::Stop) | Err(_) => break,
            }
        }
        tracing::debug!("unplayable notifier thread stopped");
    }

    /// What: `handle` turns one report into a notification, if it has a file not told before.
    /// Why: a failed post is logged once and nothing else is shown; the row mark is the fallback.
    fn handle(&mut self, paths: &[PathBuf]) {
        let Some(notice) = self.ledger.admit(paths) else {
            return;
        };
        if let Err(error) = self.deliver(&notice) {
            self.warn_once(&error);
        }
    }

    /// What: `deliver` posts `notice` on the session bus, making the connection on first use.
    /// Why: a failed connection is dropped from the state so the next report tries again.
    fn deliver(&mut self, notice: &Notice) -> zbus::Result<()> {
        let connection = match self.connection.as_ref() {
            Some(existing) => existing.clone(),
            None => {
                let made = (self.connect)()?;
                tracing::info!("unplayable notifier connected to the session bus");
                self.connection = Some(made.clone());
                made
            }
        };
        crate::unplayable_notification_wire::post(&connection, &self.identity, notice)?;
        return Ok(());
    }

    /// What: `warn_once` logs the first failure as a warning and later ones as debug lines.
    /// Why: a desktop with no notification service would otherwise repeat the same warning for every report.
    fn warn_once(&mut self, error: &zbus::Error) {
        if self.warned {
            tracing::debug!(error = %error, "unplayable-file notification could not be posted again");
            return;
        }
        self.warned = true;
        tracing::warn!(
            error = %error,
            "unplayable-file notification could not be posted (no notification service answered, or the call failed); \
             broken files are marked on their rows only"
        );
    }
}

/// What: `UnplayableNotifier` off Linux is a handle that does nothing.
/// Why: macOS and Windows have no notification code yet (D122), and the app code that holds the handle stays the same
/// on every platform; the row mark is all they show.
#[cfg(not(target_os = "linux"))]
#[derive(Clone)]
pub struct UnplayableNotifier;

/// Methods of the off-Linux `UnplayableNotifier`.
#[cfg(not(target_os = "linux"))]
impl UnplayableNotifier {
    /// What: `spawn` builds the empty handle.
    /// Why: it mirrors the Linux constructor.
    pub fn spawn(_config: UnplayableConfig) -> UnplayableNotifier {
        return UnplayableNotifier;
    }

    /// What: `report` does nothing.
    /// Why: there is no notification off Linux yet.
    pub fn report(&self, _paths: &[std::path::PathBuf]) {}

    /// What: `shutdown` does nothing.
    /// Why: there is nothing to stop off Linux yet.
    pub fn shutdown(&self) {}
}

#[cfg(all(test, target_os = "linux"))]
#[path = "unplayable_notification_tests.rs"]
mod tests;
