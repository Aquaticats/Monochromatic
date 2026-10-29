//! The private session bus shared by every feature that must stay off the host desktop.
//!
//! One `dbus-daemon` serves the appearance portal (`--color-scheme`, `--accent-color`),
//!  the taskbar progress log (`--launcher-log`),
//!  and the recording notification and Plasma job services (`--notifications`).
//! The daemon reads a configuration with no service directories,
//!  so no host service can be started through it.
//! The hosted client receives its address as `DBUS_SESSION_BUS_ADDRESS`;
//!  the compositor's own session-bus environment is never used.

/// Imports buffered address read from `dbus-daemon` stdout.
use std::io::{BufRead, BufReader};

/// Imports the owned OS-native string that carries the configuration path to `dbus-daemon`.
use std::ffi::OsString;

/// Imports owned scratch paths for the private bus socket and cleanup.
use std::path::{Path, PathBuf};

/// Imports private-bus process and pipe construction.
use std::process::{Child, ChildStdout, Command, Stdio};

/// What:
///      `use std::sync::{atomic::{AtomicU64, Ordering}, mpsc};`.
///  `AtomicU64` is an integer cell several threads may bump without a lock;
///  `mpsc` is a channel between threads.
/// Why:
///       The sequence counter keeps fixture paths unique,
///  and the channel bounds the private-bus address read.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};

/// Imports bounded startup duration for private daemon address.
use std::time::Duration;

/// Imports shared error context and result channel.
use anyhow::{Context, Result};

/// Bounds time spent waiting for private bus startup address.
const PRIVATE_BUS_ADDRESS_TIMEOUT: Duration = Duration::from_secs(2);

/// Produces collision-free paths when tests start more than one private bus.
static PRIVATE_BUS_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Configuration of the private bus:
///  the stock session bus minus every service directory.
///
/// What:
///      `const PRIVATE_BUS_CONFIG: &str = r#"..."#;`.
///  A string baked into the binary.
///           `r#"..."#` is a raw string literal:
///  backslashes and quotes inside it are literal
///           text,
///  so the XML needs no escaping.
///  It keeps the stock `session.conf` elements that
///           make this a session bus (`<type>`,
///  `<keep_umask/>`,
///  `<auth>`,
///  the allow-all
///           policy) and omits `<standard_session_servicedirs/>`,
///  every `<servicedir>`,
///  and every
///           `<include>`/`<includedir>`,
///  which could add service directories back.
/// Why:
///       Without a service directory the daemon has no `.service` file to read,
///  so a
///           hosted client asking it to start a name (as `slint-viewer` did for `org.a11y.Bus`)
///           gets `org.freedesktop.DBus.Error.ServiceUnknown` at once,
///  and no host program is
///           ever executed for it.
///  `dbus-daemon` refuses a configuration without `<listen>`,
///           so the element names a path that cannot be bound;
///  the `--address` option,
///  which
///           the daemon documents as overriding `<listen>`,
///  supplies the real socket.
/// Gotcha:
///    No path is interpolated into this text,
///  so no XML escaping is needed.
///  Adding one
///           later would make the text a syntax boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRIVATE_BUS_CONFIG = `<busconfig>...</busconfig>`;
/// ```
const PRIVATE_BUS_CONFIG: &str = r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <keep_umask/>
  <listen>unix:path=/dev/null/replaced-by-the-address-option</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"#;

/// Reads one printed private-bus address from daemon stdout.
fn read_bus_address(stdout: ChildStdout) -> std::io::Result<String> {
    let mut address = String::new();
    BufReader::new(stdout).read_line(&mut address)?;
    return Ok(address);
}

/// Owns private message-bus daemon and scratch directory.
#[derive(Debug)]
pub struct PrivateBus {
    /// Running `dbus-daemon --config-file=... --nofork` process with no service directories.
    daemon: Child,
    /// Scratch directory containing bus socket.
    directory: PathBuf,
    /// Address inherited by hosted client.
    address: String,
}

/// Starts the isolated message bus that hosted clients use.
impl PrivateBus {
    /// Starts isolated session bus and reads its printed address.
    ///
    /// What:
    ///      `pub fn start() -> Result<Self>`.
    ///  Spawns the daemon and waits up to two seconds for its address line.
    /// Why:
    ///       The appearance portal, the launcher listener, and the notification services share this one bus,
    ///  so the compositor starts the daemon once.
    pub fn start() -> Result<Self> {
        let sequence = PRIVATE_BUS_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "monochromatic-nested-wayland-session-{}-{sequence}",
            std::process::id(),
        ));
        std::fs::create_dir(&directory)
            .with_context(|| format!("creating private D-Bus directory: {}", directory.display()))?;
        let socket_path = directory.join("bus");
        let config_path = directory.join("bus.conf");
        // Write the service-free configuration beside the socket; clean the directory on failure.
        if let Err(error) = std::fs::write(&config_path, PRIVATE_BUS_CONFIG) {
            if let Err(cleanup_error) = std::fs::remove_dir_all(&directory) {
                tracing::warn!(
                    %cleanup_error,
                    path = %directory.display(),
                    "failed to clean private D-Bus directory after configuration write error",
                );
            }
            return Err(error)
                .with_context(|| format!("writing private D-Bus configuration: {}", config_path.display()));
        }
        // What:     `OsString::from(...)` starts an owned OS-native string (sibling: UTF-8
        //           `String`); `.push(&config_path)` appends the path's raw bytes, borrowed.
        // Why:      The option is built without converting the path to UTF-8, so an unusual
        //           temporary-directory name cannot be altered on its way to the daemon.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const configArgument = "--config-file=" + configPath;
        // ```
        let mut config_argument = OsString::from("--config-file=");
        config_argument.push(&config_path);
        // `--config-file` replaces `--session`, which would load the stock service directories.
        let daemon = Command::new("dbus-daemon")
            .arg(config_argument)
            .arg("--nofork")
            .arg("--nopidfile")
            .arg("--nosyslog")
            .arg("--print-address=1")
            .arg(format!("--address=unix:path={}", socket_path.display()))
            .stdout(Stdio::piped())
            .spawn()
            .inspect_err(|_error| {
                if let Err(cleanup_error) = std::fs::remove_dir_all(&directory) {
                    tracing::warn!(
                        %cleanup_error,
                        path = %directory.display(),
                        "failed to clean private D-Bus directory after spawn error",
                    );
                }
            })
            .context("starting private dbus-daemon for the nested session")?;
        let mut bus = Self { daemon, directory, address: String::new() };
        let stdout = bus
            .daemon
            .stdout
            .take()
            .context("private dbus-daemon did not expose address output")?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let reader = std::thread::Builder::new()
            .name("nested-bus-address".to_owned())
            .spawn(move || {
                if sender.send(read_bus_address(stdout)).is_err() {
                    tracing::warn!("private D-Bus address receiver ended before reader");
                }
            })
            .context("starting private D-Bus address reader")?;
        let received = receiver.recv_timeout(PRIVATE_BUS_ADDRESS_TIMEOUT);
        if let Err(error) = &received {
            bus.stop();
            if reader.join().is_err() {
                tracing::warn!("private D-Bus address reader panicked during cleanup");
            }
            anyhow::bail!("private dbus-daemon did not provide its address in time: {error}")
        }
        let address_result = received.context("receiving private dbus-daemon address")?;
        if reader.join().is_err() {
            anyhow::bail!("private D-Bus address reader panicked")
        }
        let address = address_result.context("reading private dbus-daemon address")?;
        bus.address = address.trim().to_owned();
        if bus.address.is_empty() {
            anyhow::bail!("private dbus-daemon returned an empty address")
        }
        return Ok(bus);
    }

    /// Returns the address hosted clients receive as `DBUS_SESSION_BUS_ADDRESS`.
    #[must_use]
    pub fn address(&self) -> &str {
        return self.address.as_str()
    }

    /// Returns the scratch directory holding the bus socket and configuration.
    #[must_use]
    pub fn directory(&self) -> &Path {
        return self.directory.as_path()
    }

    /// Stops and reaps daemon without warning when it already exited.
    fn stop(&mut self) {
        let status = self.daemon.try_wait();
        if matches!(status, Ok(None)) {
            if let Err(error) = self.daemon.kill() {
                tracing::warn!(%error, "failed to stop private dbus-daemon");
            }
        } else if let Err(error) = status {
            tracing::warn!(%error, "failed to inspect private dbus-daemon status");
        }
        if let Err(error) = self.daemon.wait() {
            tracing::warn!(%error, "failed to reap private dbus-daemon");
        }
    }
}

/// Cleans up daemon and scratch socket even when startup fails.
impl Drop for PrivateBus {
    /// Stops private daemon and removes socket directory.
    fn drop(&mut self) {
        self.stop();
        if let Err(error) = std::fs::remove_dir_all(&self.directory) {
            tracing::warn!(%error, path = %self.directory.display(), "failed to remove private D-Bus directory");
        }
    }
}
