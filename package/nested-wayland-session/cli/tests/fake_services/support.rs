//! Shared helpers for the tests that talk to the fake services over a real private bus.
//!
//! The host session bus is never opened:
//!  every client here connects to the address of a
//! disposable `dbus-daemon` that has no service directories.

use std::{
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

use nested_wayland_session::{
    notification_mode::NotificationMode, plasma_services::PlasmaServices, private_bus::PrivateBus,
    protocol::Response,
};
use zbus::blocking::{connection::Builder, Connection, Proxy};

/// Bounds how long a test waits for something that must already be on the bus.
pub const WAIT: Duration = Duration::from_secs(5);

/// A private bus,
///  the fake services on it,
///  and a client connection like a hosted app's.
pub struct Fixture {
    /// The services under test;
    ///  dropped before the bus.
    pub services: PlasmaServices,
    /// A second connection on the same bus,
    ///  standing in for the hosted app.
    pub client: Connection,
    /// The bus,
    ///  kept alive for the whole test.
    pub _bus: Arc<PrivateBus>,
}

/// Starts a private bus and the services `mode` names,
///  and connects a client.
pub fn start(mode: NotificationMode) -> anyhow::Result<Fixture> {
    let bus = Arc::new(PrivateBus::start()?);
    let services = PlasmaServices::start(mode, Arc::clone(&bus), Instant::now())?
        .ok_or_else(|| return anyhow::anyhow!("mode {mode:?} serves nothing"))?;
    let client = Builder::address(bus.address())?.build()?;
    return Ok(Fixture { services, client, _bus: bus });
}

/// Opens a proxy to a service the way a hosted app does:
///  by well-known name.
pub fn proxy(client: &Connection, name: &'static str, path: &str, interface: &'static str) -> anyhow::Result<Proxy<'static>> {
    return Ok(Proxy::new(client, name, path.to_owned(), interface)?);
}

/// Unwraps an `ok <data>` response and returns the data;
///  panics with the response otherwise.
pub fn ok_data(response: Response) -> String {
    return match response {
        Response::OkWith(data) => data,
        Response::Ok => String::new(),
        Response::Err(message) => panic!("expected ok, got err {message}"),
    };
}

/// Unwraps an `err <message>` response and returns the message;
///  panics otherwise.
pub fn err_message(response: Response) -> String {
    let Response::Err(message) = response else {
        panic!("expected err, got {response:?}");
    };
    return message;
}

/// Reads the value of `name=` in a space-separated record.
pub fn field<'a>(record: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}=");
    return record.split(' ').find_map(|part| return part.strip_prefix(prefix.as_str()));
}

/// Polls `condition` until it holds,
///  or fails the test at the timeout.
pub fn wait_until(what: &str, mut condition: impl FnMut() -> bool) {
    let started = Instant::now();
    while started.elapsed() < WAIT {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for {what}");
}

/// Messages of one signal subscription,
///  forwarded from a helper thread so waits have a timeout.
pub struct Signals {
    /// Receives each matching message in arrival order.
    receiver: mpsc::Receiver<zbus::Message>,
}

/// Subscribing and waiting.
impl Signals {
    /// Subscribes to `member` through `proxy`;
    ///  the match rule is registered before this returns.
    pub fn subscribe(proxy: &Proxy<'static>, member: &'static str) -> anyhow::Result<Self> {
        let iterator = proxy.receive_signal(member)?;
        let (sender, receiver) = mpsc::sync_channel(32);
        std::thread::Builder::new().name(format!("signals-{member}")).spawn(move || {
            for message in iterator {
                if sender.send(message).is_err() {
                    return;
                }
            }
        })?;
        return Ok(Self { receiver });
    }

    /// Waits for the next message,
    ///  or fails the test at the timeout.
    pub fn next(&self, what: &str) -> zbus::Message {
        return self
            .receiver
            .recv_timeout(WAIT)
            .unwrap_or_else(|_| panic!("no {what} signal arrived"));
    }

    /// Tells whether another message arrives within a short grace period.
    pub fn arrives_soon(&self) -> bool {
        return self.receiver.recv_timeout(Duration::from_millis(250)).is_ok();
    }
}
