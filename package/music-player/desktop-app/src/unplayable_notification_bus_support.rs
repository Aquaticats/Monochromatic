//! A private D-Bus session bus and a recording `org.freedesktop.Notifications` service,
//! for tests only.
//!
//! `PrivateBus::start` runs its own `dbus-daemon` on a socket in a scratch folder,
//! so no test ever reaches the user's
//! session bus or notification service.
//! `Recorder` answers `Notify` and keeps every call,
//! so a test can assert the exact arguments.

/// Imports the map type of the hints that a recorded call carries.
use std::collections::HashMap;
/// Imports the line reader that reads the daemon's address.
use std::io::{BufRead, BufReader};
/// Imports the path type of the daemon's socket folder.
use std::path::PathBuf;
/// Imports the process types that start the daemon.
use std::process::{Child, Command, Stdio};
/// Imports the shared ownership and lock types that share the recorded calls.
use std::sync::{Arc, Mutex};
/// Imports the duration type of the service's reply delay.
use std::time::Duration;

/// Imports the stream extension that reads messages one at a time.
use futures_util::StreamExt;
/// Imports the connection,
/// message stream and variant types of the D-Bus client.
use zbus::{Connection, MessageStream, OwnedVariant};

/// One recorded `Notify` call,
/// reduced to what the tests assert.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Call {
    /// Application name that the caller gave.
    pub app_name: String,
    /// Identifier of the notification that the call replaces,
    /// zero for a new one.
    pub replaces_id: u32,
    /// Icon name or path.
    pub app_icon: String,
    /// Headline of the notification.
    pub summary: String,
    /// Body text of the notification.
    pub body: String,
    /// Action keys and labels,
    /// alternating,
    /// as the call sends them.
    pub actions: Vec<String>,
    /// The names of the hints that the call carried,
    /// in sorted order.
    pub hint_names: Vec<String>,
    /// The urgency hint,
    /// when the call carried one.
    pub urgency: Option<u8>,
    /// The desktop entry hint,
    /// when the call carried one.
    pub desktop_entry: Option<String>,
    /// Milliseconds until the notification expires,
    /// or the server default.
    pub expire_timeout: i32,
}

/// The calls a recorder has seen,
/// shared with the test.
pub(crate) type Calls = Arc<Mutex<Vec<Call>>>;

/// What the `Notify` call carries on the wire,
/// in the order the specification gives.
type NotifyBody = (String, u32, String, String, String, Vec<String>, HashMap<String, OwnedVariant>, i32);

/// Reduces a decoded `Notify` body to the fields the tests assert.
fn record(body: NotifyBody) -> Call {
    let (app_name, replaces_id, app_icon, summary, text, actions, hints, expire_timeout) = body;
    let mut hint_names: Vec<String> = hints.keys().cloned().collect();
    hint_names.sort();
    let urgency = hints.get("urgency").and_then(|value| return u8::try_from(value).ok());
    let desktop_entry = hints.get("desktop-entry").and_then(|value| return String::try_from(value.clone()).ok());
    return Call { app_name, replaces_id, app_icon, summary, body: text, actions, hint_names, urgency, desktop_entry, expire_timeout };
}

/// How a recording service misbehaves:
/// a delay before every reply (a wedged service when long),
/// and a number of calls at
/// the start that it refuses with an error reply and does not record.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Behavior {
    /// How long the service waits before it answers each call.
    pub delay: Duration,
    /// How many of the first calls the service refuses with an error reply.
    pub refuse_first: usize,
}

/// Answers `Notify` calls on `connection` until the bus goes away,
/// recording each and replying with the call count
/// after the delay of `behavior`;
/// the first `refuse_first` calls get an error reply instead.
fn answer_notify_calls(connection: &Connection, incoming: MessageStream, calls: &Calls, behavior: Behavior) {
    zbus::block_on(answer_notify_calls_async(connection, incoming, calls, behavior));
}

/// The body of `answer_notify_calls`,
/// run inside one `zbus::block_on` call on the service thread.
/// The delay is a blocking sleep on purpose:
/// it stands for a service that stopped answering.
async fn answer_notify_calls_async(connection: &Connection, mut incoming: MessageStream, calls: &Calls, behavior: Behavior) {
    let mut refused = 0;
    while let Some(received) = incoming.next().await {
        let Ok(message) = received else {
            return;
        };
        let header = message.header();
        if header.member().map(ToString::to_string).as_deref() != Some("Notify") {
            continue;
        }
        let Ok(body) = message.body().deserialize::<NotifyBody>() else {
            continue;
        };
        std::thread::sleep(behavior.delay);
        if refused < behavior.refuse_first {
            refused += 1;
            if connection.reply_error(&header, "org.freedesktop.DBus.Error.Failed", &("refused on purpose",)).await.is_err() {
                return;
            }
            continue;
        }
        let id = {
            let mut recorded = calls.lock().expect("the recorded calls lock is not poisoned");
            recorded.push(record(body));
            u32::try_from(recorded.len()).expect("a test makes few calls")
        };
        if connection.reply(&header, &(id,)).await.is_err() {
            return;
        }
    }
    tracing::debug!("the recording notification service stopped reading its connection");
}

/// A `dbus-daemon` on a private socket,
/// killed and cleaned up on drop.
pub(crate) struct PrivateBus {
    /// The daemon process,
    /// killed on drop.
    child: Child,
    /// The private folder that holds the daemon's socket,
    /// removed on drop.
    folder: PathBuf,
    /// The bus address that clients connect to.
    pub address: String,
}

/// Methods of `PrivateBus` that start the daemon,
/// connect to it and serve the notification service on it.
impl PrivateBus {
    /// Starts the daemon and reads its address.
    ///
    /// # Panics
    /// When `dbus-daemon` is not installed or does not print an address;
    /// the tests need it and say so.
    pub(crate) fn start(label: &str) -> PrivateBus {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock after 1970").as_nanos();
        // The daemon creates its socket inside this folder, and a Unix socket path holds at most 107 bytes, which the
        // build directory's path already exceeds, so the folder lives in the per-user runtime directory.
        let runtime = std::env::var_os("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR holds the short socket directory");
        let folder = std::path::PathBuf::from(runtime).join(format!("mp_bus_{label}_{}_{nanos}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("the scratch folder is made");
        let config = folder.join("bus.conf");
        let text = format!(
            "<!DOCTYPE busconfig PUBLIC \"-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN\" \
             \"http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd\">\n\
             <busconfig><type>session</type><listen>unix:dir={}</listen><auth>EXTERNAL</auth>\
             <policy context=\"default\"><allow send_destination=\"*\" eavesdrop=\"true\"/>\
             <allow eavesdrop=\"true\"/><allow own=\"*\"/></policy></busconfig>\n",
            folder.display()
        );
        std::fs::write(&config, text).expect("the bus config is written");
        let mut child = Command::new("dbus-daemon")
            .arg("--nofork")
            .arg("--print-address=1")
            .arg(format!("--config-file={}", config.display()))
            .stdout(Stdio::piped())
            .spawn()
            .expect("dbus-daemon must be installed to run the notification bus tests");
        let stdout = child.stdout.take().expect("the daemon's stdout is piped");
        let mut printed = String::new();
        BufReader::new(stdout).read_line(&mut printed).expect("the daemon prints its address");
        let address = printed.trim().to_string();
        assert!(address.starts_with("unix:"), "the private bus address is {address:?}");
        return PrivateBus { child, folder, address };
    }

    /// Connects a client to this bus with a bounded reply wait.
    pub(crate) fn connect(&self, timeout: Duration) -> zbus::Result<Connection> {
        return crate::dbus_step_bound::connect(zbus::connection::Builder::address(self.address.as_str()), timeout);
    }

    /// Starts a recording notification service on this bus and returns its connection and calls.
    pub(crate) fn serve_notifications(&self, delay: Duration) -> (Connection, Calls) {
        return self.serve_notifications_with(Behavior { delay, refuse_first: 0 });
    }

    /// Starts a recording notification service that misbehaves as `behavior` says.
    pub(crate) fn serve_notifications_with(&self, behavior: Behavior) -> (Connection, Calls) {
        let calls: Calls = Arc::new(Mutex::new(Vec::new()));
        let connection = crate::dbus_step_bound::connect(
            zbus::connection::Builder::address(self.address.as_str()).name("org.freedesktop.Notifications"),
            crate::dbus_step_bound::DBUS_STEP_BOUND,
        )
        .expect("the recording notification service starts");
        let serving = connection.clone();
        // The iterator is made here, before the thread starts: a call that arrives before an iterator exists is never
        // delivered to it, and a loaded host delayed the thread past the first call, which then timed out.
        let incoming = MessageStream::from(&connection);
        let recorded = Arc::clone(&calls);
        std::thread::spawn(move || answer_notify_calls(&serving, incoming, &recorded, behavior));
        return (connection, calls);
    }
}

/// Kills the daemon and removes its socket folder,
/// so that no test leaves a process behind.
impl Drop for PrivateBus {
    /// What:
    /// Kills the daemon and removes its socket folder.
    /// Why:
    /// A test that panics must still leave no process or folder behind.
    fn drop(&mut self) {
        if let Err(error) = self.child.kill() {
            tracing::debug!(error = %error, "the private bus daemon was already gone");
        }
        if let Err(error) = self.child.wait() {
            tracing::debug!(error = %error, "the private bus daemon could not be waited for");
        }
        if let Err(error) = std::fs::remove_dir_all(&self.folder) {
            tracing::debug!(error = %error, "the private bus folder could not be removed");
        }
    }
}
