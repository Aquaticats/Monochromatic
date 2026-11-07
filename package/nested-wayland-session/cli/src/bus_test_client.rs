//! A blocking D-Bus client for the tests,
//!  standing in for the `zbus::blocking` types zbus 6 removed.
//!
//! Each call runs one `zbus::block_on`,
//!  which is the replacement zbus's upgrade guide names.
//!  Every connection carries the method-call bound and every connect is bounded,
//!  so a test whose peer stopped answering fails at the bound instead of hanging.
//!  The shipped compositor does not call this module.

/// Imports `Future` from `std`.
use std::future::Future;

/// Imports `StreamExt` for `next` on a signal stream.
use futures_util::StreamExt;
/// Imports the zbus client types this module wraps.
use zbus::{
    connection::Builder,
    names::{BusName, InterfaceName, MemberName},
    proxy::SignalStream,
    Connection, DynamicType, Message,
};

/// Imports the bound shared with the compositor's own steps.
use crate::bus_step_bound::{block_on_bounded, BUS_STEP_BOUND};

/// Connects to the bus at `address` with the method-call bound set,
///  failing after one step bound.
///
/// What:
///      `pub fn connect(address: &str) -> anyhow::Result<Connection>`.
/// Why:
///       Tests connect clients the way a hosted app does,
///  by the private bus address alone.
///
/// # Errors
///
/// Fails when the address is invalid,
///  the bus does not answer within the bound,
///  or authentication fails.
pub fn connect(address: &str) -> anyhow::Result<Connection> {
    return block_on_bounded("connecting a test client", Builder::address(address).method_timeout(BUS_STEP_BOUND).build());
}

/// Runs `work` on the calling thread and returns its result.
///
/// What:
///      `pub fn block_on<T>(work: impl Future<Output = T>) -> T`.
/// Why:
///       One name for tests that need an async zbus call with its raw `zbus::Result`,
///  such as a test that matches the error variant.
pub fn block_on<T>(work: impl Future<Output = T>) -> T {
    return zbus::block_on(work);
}

/// A blocking view of one remote object's interface.
pub struct Proxy<'a> {
    /// The async proxy every method forwards to.
    inner: zbus::Proxy<'a>,
}

/// Calls and subscriptions,
///  each one `block_on` call.
impl<'a> Proxy<'a> {
    /// Opens a proxy for `interface` of `path` on `destination`.
    ///
    /// # Errors
    ///
    /// Fails on an invalid name or path.
    pub fn new<D, P, I>(connection: &Connection, destination: D, path: P, interface: I) -> zbus::Result<Self>
    where
        D: TryInto<BusName<'a>>,
        D::Error: Into<zbus::Error>,
        P: TryInto<zbus::ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
        I: TryInto<InterfaceName<'a>>,
        I::Error: Into<zbus::Error>,
    {
        let inner = zbus::block_on(zbus::Proxy::new(connection, destination, path, interface))?;
        return Ok(Self { inner });
    }

    /// Calls `method` and returns its reply.
    ///
    /// # Errors
    ///
    /// Fails with the bus error,
    ///  or with `ErrorKind::TimedOut` after the connection's method bound.
    pub fn call<'m, M, B, R>(&self, method: M, body: &B) -> zbus::Result<R>
    where
        M: TryInto<MemberName<'m>>,
        M::Error: Into<zbus::Error>,
        B: zbus::export::serde::Serialize + DynamicType,
        R: for<'d> zbus::wire::DynamicDeserialize<'d>,
    {
        return zbus::block_on(self.inner.call(method, body));
    }

    /// Calls `method` without waiting for a reply.
    ///
    /// # Errors
    ///
    /// Fails when the call cannot be sent.
    pub fn call_noreply<'m, M, B>(&self, method: M, body: &B) -> zbus::Result<()>
    where
        M: TryInto<MemberName<'m>>,
        M::Error: Into<zbus::Error>,
        B: zbus::export::serde::Serialize + DynamicType,
    {
        return zbus::block_on(self.inner.call_noreply(method, body));
    }

    /// Subscribes to `signal` and returns its messages as an iterator that blocks for the next one.
    ///
    /// # Errors
    ///
    /// Fails when the match rule cannot be registered.
    pub fn receive_signal<'m, M>(&self, signal: M) -> zbus::Result<SignalIterator<'m>>
    where
        M: TryInto<MemberName<'m>>,
        M::Error: Into<zbus::Error>,
    {
        let stream = zbus::block_on(self.inner.receive_signal(signal))?;
        return Ok(SignalIterator { stream });
    }
}

/// The messages of one signal subscription;
///  `next` blocks until one arrives or the connection closes.
pub struct SignalIterator<'a> {
    /// The subscription the iterator drains.
    stream: SignalStream<'a>,
}

/// Blocks the calling thread for each message.
impl Iterator for SignalIterator<'_> {
    /// One received signal message.
    type Item = Message;

    /// Blocks until the next signal arrives,
    ///  and returns `None` once the connection closes.
    fn next(&mut self) -> Option<Message> {
        return zbus::block_on(self.stream.next());
    }
}
