//! A minimal AT-SPI accessibility bus on the private session bus,
//!  so a hosted toolkit publishes its accessibility tree where a test can read it.
//!
//! Toolkits built on AccessKit (Slint's winit backend among them) look up the accessibility bus
//!  through `org.a11y.Bus.GetAddress` on the session bus,
//!  watch the `IsEnabled` property of `org.a11y.Status`,
//!  and,
//!  once it is true,
//!  connect to the bus it names,
//!  register their application root with `org.a11y.atspi.Socket.Embed` on the registry,
//!  and serve `org.a11y.atspi.Accessible` and related interfaces from their own bus name.
//!  This service answers exactly those three requests and remembers who registered;
//!  it implements no screen reader and no event routing.

/// Standard library items.
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

/// Error handling with context.
use anyhow::{Context, Result};
/// Structured logging.
use tracing::info;
/// The D-Bus client and server library.
use zbus::{blocking::Connection, zvariant::OwnedObjectPath};

/// Sibling modules of this crate.
use crate::private_bus::PrivateBus;

/// Well-known name of the accessibility bus launcher.
///
/// What:
///      `pub const BUS_NAME: &str = "org.a11y.Bus";`.
/// Why:
///       Toolkits ask this name for the accessibility bus address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BUS_NAME = "org.a11y.Bus";
/// ```
pub const BUS_NAME: &str = "org.a11y.Bus";

/// Well-known name of the AT-SPI registry.
///
/// What:
///      `pub const REGISTRY_NAME: &str = "org.a11y.atspi.Registry";`.
/// Why:
///       Toolkits register their application root with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REGISTRY_NAME = "org.a11y.atspi.Registry";
/// ```
pub const REGISTRY_NAME: &str = "org.a11y.atspi.Registry";

/// Object path of the registry root,
///  which is also where applications register.
///
/// What:
///      `pub const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";`.
/// Why:
///       Applications expose their own root at the same path on their own bus name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROOT_PATH = "/org/a11y/atspi/accessible/root";
/// ```
pub const ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";

/// A registered application:
///  the bus name and object path of its accessibility root.
///
/// What:
///      `pub struct Application { pub bus_name: String, pub path: String }`.
/// Why:
///       The tree reader starts from these roots.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Application = { busName: string; path: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    /// Unique bus name of the application's accessibility connection.
    pub bus_name: String,
    /// Object path of its accessibility root.
    pub path: String,
}

/// The shared list of registered applications.
///
/// What:
///      `type Applications = Arc<Mutex<Vec<Application>>>;`.
/// Why:
///       The registry handler (on the service's thread) writes it and the control command reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Applications = Application[];
/// ```
type Applications = Arc<Mutex<Vec<Application>>>;

/// `org.a11y.Bus`:
///  tells a toolkit where the accessibility bus is.
///
/// What:
///      `struct BusLauncher { address: String }`.
///  The accessibility bus is the private bus itself.
/// Why:
///       Keeping one bus means one daemon to start and one place to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BusLauncher { getAddress() { return this.address; } }
/// ```
struct BusLauncher {
    /// Address of the private bus.
    address: String,
}

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The `GetAddress` method toolkits call.
    #[zbus::interface(name = "org.a11y.Bus")]
    impl BusLauncher {
        /// Return the accessibility bus address.
        fn get_address(&self) -> String {
            return self.address.clone()
        }
    }
};

/// `org.a11y.Status`:
///  reports that accessibility is wanted.
///
/// What:
///      `struct Status;`.
///  Both properties are always true.
/// Why:
///       Toolkits publish their tree only while a screen reader is said to run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Status { isEnabled = true; screenReaderEnabled = true; }
/// ```
struct Status;

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The two properties toolkits read.
    #[zbus::interface(name = "org.a11y.Status")]
    impl Status {
        /// Accessibility is enabled.
        #[zbus(property)]
        fn is_enabled(&self) -> bool {
            return true
        }

        /// A screen reader is running.
        #[zbus(property)]
        fn screen_reader_enabled(&self) -> bool {
            return true
        }
    }
};

/// `org.a11y.atspi.Socket`:
///  the registry's `Embed` request.
///
/// What:
///      `struct Registry { applications: Applications }`.
/// Why:
///       The call carries the application's bus name and root path,
///  and the answer is the desktop root they become children of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Registry { embed(plug: [string, string]): [string, string] { ... } }
/// ```
struct Registry {
    /// Where registered applications are remembered.
    applications: Applications,
    /// Unique bus name of this service,
    ///  which becomes known only once the connection exists.
    unique_name: Arc<OnceLock<String>>,
}

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The `Embed` method applications call.
    #[zbus::interface(name = "org.a11y.atspi.Socket")]
    impl Registry {
        /// Remember the application and return the desktop root.
        fn embed(&self, plug: (String, OwnedObjectPath)) -> (String, OwnedObjectPath) {
            let (bus_name, path) = plug;
            info!(%bus_name, path = %path.as_str(), "an application registered its accessibility root");
            let mut applications = self.applications.lock().unwrap_or_else(PoisonError::into_inner);
            let entry = Application { bus_name, path: path.as_str().to_string() };
            if !applications.contains(&entry) {
                applications.push(entry);
            }
            drop(applications);
            let desktop = OwnedObjectPath::try_from(ROOT_PATH).unwrap_or_else(|_| return OwnedObjectPath::default());
            // The desktop reference must name a unique bus name,
            //  which is what a real registry daemon answers with.
            let owner = self.unique_name.get().cloned().unwrap_or_else(|| return REGISTRY_NAME.to_string());
            return (owner, desktop)
        }
    }
};

/// The running accessibility service.
///
/// What:
///      `pub struct A11yService { connection: Connection, applications: Applications, bus: Arc<PrivateBus> }`.
/// Why:
///       Owns the D-Bus connection that carries the three services,
///  and keeps the private bus alive while the control commands read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class A11yService { connection; applications; bus; }
/// ```
pub struct A11yService {
    /// The service connection,
    ///  also used to read the application's tree.
    connection: Connection,
    /// Applications that registered.
    applications: Applications,
    /// Keeps the private bus alive.
    _bus: Arc<PrivateBus>,
}

/// Starting the service and reading what registered.
impl A11yService {
    /// Start serving the accessibility bus protocol on the private bus.
    ///
    /// What:
    ///      `pub fn start(bus: Arc<PrivateBus>) -> Result<Self>`.
    /// Why:
    ///       Called once at startup when `--accessibility` is given.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function start(bus: PrivateBus): A11yService { ... }
    /// ```
    pub fn start(bus: Arc<PrivateBus>) -> Result<Self> {
        let applications: Applications = Arc::new(Mutex::new(Vec::new()));
        let unique_name: Arc<OnceLock<String>> = Arc::new(OnceLock::new());
        let connection = zbus::blocking::connection::Builder::address(bus.address())?
            .name(BUS_NAME)?
            .name(REGISTRY_NAME)?
            .serve_at("/org/a11y/bus", BusLauncher { address: bus.address().to_string() })?
            .serve_at("/org/a11y/bus", Status)?
            .serve_at(ROOT_PATH, Registry { applications: Arc::clone(&applications), unique_name: Arc::clone(&unique_name) })?
            .build()
            .context("starting the private accessibility bus services")?;
        if let Some(name) = connection.unique_name() {
            let _ = unique_name.set(name.as_str().to_string());
        }
        info!("started the private accessibility bus services");
        return Ok(Self { connection, applications, _bus: bus })
    }

    /// The registered applications,
    ///  first registered first.
    ///
    /// What:
    ///      `pub fn applications(&self) -> Vec<Application>`.
    /// Why:
    ///       The tree reader walks each of them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// applications(): Application[] { return [...this.applications]; }
    /// ```
    #[must_use]
    pub fn applications(&self) -> Vec<Application> {
        let guard: MutexGuard<'_, Vec<Application>> = self.applications.lock().unwrap_or_else(PoisonError::into_inner);
        return guard.clone()
    }

    /// The connection to query applications with.
    ///
    /// What:
    ///      `pub fn connection(&self) -> &Connection`.
    /// Why:
    ///       The tree reader needs a client connection on the same bus.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// connection(): Connection { return this.connection; }
    /// ```
    pub fn connection(&self) -> &Connection {
        return &self.connection
    }
}
