//! The bound ends a stalled step,
//!  and a healthy step on a real private bus finishes far inside it.

/// Brings the bound helpers into scope.
use super::*;

/// Imports `Instant` for the measurements.
use std::time::Instant;

/// Imports the private bus and the test client.
use crate::{bus_test_client, private_bus::PrivateBus};

/// A short bound for the tests that must see a step end,
///  so each takes a fraction of a second where the shipped bound takes seconds.
const SHORT_BOUND: Duration = Duration::from_millis(300);

/// How many times a short bound the tests allow before calling an ended step late.
const LATE_FACTOR: u32 = 10;

/// What the answering method returns.
const PONG: u32 = 7;

/// Well-known name of the test service whose method never answers.
const SILENT_NAME: &str = "org.example.MonochromaticSilentPeer";

/// Object path of the test service.
const SILENT_PATH: &str = "/org/example/SilentPeer";

/// Interface of the test service.
const SILENT_INTERFACE: &str = "org.example.SilentPeer";

/// A service with one method that answers and one that never does.
struct SilentPeer;

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The methods the tests call.
    #[zbus::interface(name = "org.example.SilentPeer")]
    impl SilentPeer {
        /// Answers at once.
        async fn ping(&self) -> u32 {
            return PONG;
        }

        /// Never answers:
        ///  the future stays pending for the life of the connection.
        async fn never(&self) -> u32 {
            std::future::pending::<()>().await;
            return 0;
        }
    }
};

/// Starts a private bus and the silent peer on it.
fn start_silent_peer() -> anyhow::Result<(PrivateBus, zbus::Connection)> {
    let bus = PrivateBus::start()?;
    let builder = zbus::connection::Builder::address(bus.address()).name(SILENT_NAME).serve_at(SILENT_PATH, SilentPeer);
    let peer = build_bounded(builder, "starting the silent test peer")?;
    return Ok((bus, peer));
}

/// Opens a proxy to the silent peer on `connection`.
fn silent_proxy(connection: &zbus::Connection) -> anyhow::Result<bus_test_client::Proxy<'static>> {
    return Ok(bus_test_client::Proxy::new(connection, SILENT_NAME, SILENT_PATH, SILENT_INTERFACE)?);
}

/// A future that never finishes ends with an error naming the step and the bound.
#[test]
fn a_pending_step_ends_with_a_timeout_error() {
    let started = Instant::now();
    let outcome = block_on_within(SHORT_BOUND, "a step that never ends", std::future::pending::<Result<(), zbus::Error>>());
    let elapsed = started.elapsed();
    let message = format!("{:#}", outcome.expect_err("a pending step cannot succeed"));
    assert!(message.contains("a step that never ends did not finish within"), "{message}");
    assert!(elapsed >= SHORT_BOUND, "ended early: {elapsed:?}");
    assert!(elapsed < SHORT_BOUND * LATE_FACTOR, "ended late: {elapsed:?}");
}

/// A step that finishes before the bound returns its value,
///  and a failing step keeps its own error.
#[test]
fn a_finished_step_returns_its_result_and_a_failed_step_its_error() -> anyhow::Result<()> {
    let value = block_on_within(SHORT_BOUND, "an immediate step", async { return Ok::<u32, zbus::Error>(5) })?;
    assert_eq!(value, 5);
    let failure = block_on_within(SHORT_BOUND, "a failing step", async { return Err::<(), zbus::Error>(zbus::Error::Failure("boom".to_owned())) });
    let message = format!("{:#}", failure.expect_err("the step fails"));
    assert!(message.contains("a failing step failed") && message.contains("boom"), "{message}");
    return Ok(());
}

/// Without a method bound a call to a peer that never answers does not end by itself;
///  this is the hang the connection's `method_timeout` removes.
#[test]
fn a_call_without_a_method_bound_waits_for_the_outer_bound() -> anyhow::Result<()> {
    let (bus, _peer) = start_silent_peer()?;
    // Built without `method_timeout`, as the zbus 5 clients of this crate were.
    let unbounded = zbus::block_on(zbus::connection::Builder::address(bus.address()).build())?;
    assert_eq!(unbounded.method_timeout(), None);
    let proxy = zbus::block_on(zbus::Proxy::new(&unbounded, SILENT_NAME, SILENT_PATH, SILENT_INTERFACE))?;
    // Positive control: the same proxy gets an answer from the method that does reply.
    let pong: u32 = zbus::block_on(proxy.call("Ping", &()))?;
    assert_eq!(pong, PONG);
    let outcome = block_on_within(SHORT_BOUND, "an unbounded call to a silent peer", proxy.call::<_, _, u32>("Never", &()));
    let message = format!("{:#}", outcome.expect_err("the silent peer never answers"));
    assert!(message.contains("did not finish within"), "{message}");
    return Ok(());
}

/// A connection built with `method_timeout` ends the same call with `TimedOut` at that bound,
///  without the caller racing a timer of its own.
#[test]
fn a_call_with_a_method_bound_ends_with_timed_out() -> anyhow::Result<()> {
    let (bus, _peer) = start_silent_peer()?;
    let bounded = zbus::block_on(zbus::connection::Builder::address(bus.address()).method_timeout(SHORT_BOUND).build())?;
    assert_eq!(bounded.method_timeout(), Some(SHORT_BOUND));
    let proxy = zbus::block_on(zbus::Proxy::new(&bounded, SILENT_NAME, SILENT_PATH, SILENT_INTERFACE))?;
    let pong: u32 = zbus::block_on(proxy.call("Ping", &()))?;
    assert_eq!(pong, PONG);
    let started = Instant::now();
    let outcome = zbus::block_on(proxy.call::<_, _, u32>("Never", &()));
    let elapsed = started.elapsed();
    let Err(zbus::Error::InputOutput(error)) = outcome else {
        panic!("expected a timed-out I/O error, got {outcome:?}");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(elapsed >= SHORT_BOUND && elapsed < SHORT_BOUND * LATE_FACTOR, "ended at {elapsed:?}");
    return Ok(());
}

/// Every connection `build_bounded` makes carries the method bound.
#[test]
fn built_connections_carry_the_method_bound() -> anyhow::Result<()> {
    let (_bus, peer) = start_silent_peer()?;
    assert_eq!(peer.method_timeout(), Some(BUS_STEP_BOUND));
    return Ok(());
}

/// A connect,
///  a signal send,
///  and a method call on a private bus finish far inside the shipped bound.
///  The measured times are printed so a run records them.
#[test]
fn bus_steps_finish_far_inside_the_bound() -> anyhow::Result<()> {
    let bus = PrivateBus::start()?;
    let connect_started = Instant::now();
    let peer = build_bounded(
        zbus::connection::Builder::address(bus.address()).name(SILENT_NAME).serve_at(SILENT_PATH, SilentPeer),
        "connecting the measured peer",
    )?;
    let connect = connect_started.elapsed();

    let emit_started = Instant::now();
    block_on_bounded(
        "emitting the measured signal",
        peer.emit_signal(None::<&str>, SILENT_PATH, SILENT_INTERFACE, "Measured", &()),
    )?;
    let emit = emit_started.elapsed();

    let client = bus_test_client::connect(bus.address())?;
    let proxy = silent_proxy(&client)?;
    let call_started = Instant::now();
    let pong: u32 = proxy.call("Ping", &())?;
    let call = call_started.elapsed();
    assert_eq!(pong, PONG);

    println!("bus step times: connect and claim name {connect:?}, emit signal {emit:?}, method call {call:?}");
    for (step, elapsed) in [("connect", connect), ("emit", emit), ("call", call)] {
        assert!(elapsed < BUS_STEP_BOUND / LATE_FACTOR, "{step} took {elapsed:?} against a bound of {BUS_STEP_BOUND:?}");
    }
    return Ok(());
}
