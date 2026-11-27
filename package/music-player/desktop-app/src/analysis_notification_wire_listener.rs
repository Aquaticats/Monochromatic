//! Decoding bus signals and forwarding them from their own thread.
//!
//! What:
//! `decode_signal` turns one bus message into the `BusSignal` the indicator acts
//! on,
//! `read_signals` forwards each decoded signal until the stream or the receiver
//! stops,
//! and `spawn_listener` starts the thread that runs it on a bounded queue.
//! Why:
//! a signal stream and the indicator's channel cannot be waited on together,
//! so listening has a thread of its own,
//! split from `src/analysis_notification_wire.rs` to stay within the line budget.

/// What:
///      `use std::num::NonZeroUsize` brings the positive size type that bounds a
///           signal subscription's queue.
/// Why:
///      The queue bound is passed as a non-zero size,
///  so "no queue" cannot be expressed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no NonZeroUsize; a number with a runtime check is the closest thing.
/// ```
use std::num::NonZeroUsize;

/// What:
///      `use std::thread::JoinHandle` brings the handle of the listener thread.
/// Why:
///      The caller keeps it to know the thread is still running,
///  and to wait for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no thread handle; the caller keeps a Promise instead.
/// ```
use std::thread::JoinHandle;

/// What:
///      `use futures_util::StreamExt` brings the extension that reads one item of
///           the signal stream at a time.
/// Why:
///      The stream is polled one signal at a time inside the listener's loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // `for await (const signal of stream)` needs no import in TS.
/// ```
use futures_util::StreamExt;

/// What:
///      `use zbus::match_rule::MatchRule` brings the rule that selects which
///           signals the subscription receives.
/// Why:
///      The listener subscribes only to its own interface,
///  which keeps the queue quiet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MatchRule } from "zbus";
/// ```
use zbus::match_rule::MatchRule;

/// What:
///      `use zbus::message::Type` brings the message-kind constant that selects
///           signals.
/// Why:
///      The match rule must name signals,
///  or it would also receive method calls and returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Type } from "zbus";
/// ```
use zbus::message::Type;

/// What:
///      `use zbus::{...}` brings the connection the subscription is made on,
///  and the stream it produces.
/// Why:
///      Those two are the whole state of the listener thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Connection, MessageStream } from "zbus";
/// ```
use zbus::{
    Connection,
    MessageStream,
};

/// What:
///      `use super::{...}` brings the signal value,
///  the backend that chooses the interface,
///  and the two interface names.
/// Why:
///      A listener watches one interface,
///  and answers with the parent's signal type,
///  so nothing here restates a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Backend, BusSignal, JOB_VIEW_INTERFACE, NOTIFICATIONS_INTERFACE } from "./analysis_notification_wire";
/// ```
use super::{
    Backend,
    BusSignal,
    JOB_VIEW_INTERFACE,
    NOTIFICATIONS_INTERFACE,
};

/// What:
///      `use crate::dbus_step_bound::step` brings the bounded wait that makes the
///           subscription.
/// Why:
///      Subscribing is a bus call and may never return,
///  so it waits under the same bound as every other call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { step } from "./dbus_step_bound";
/// ```
use crate::dbus_step_bound::step;
/// What:
/// `decode_signal` turns a bus message into a `BusSignal`,
/// or `None` for anything else.
/// Why:
/// the listener forwards only the four signals the indicator acts on.
fn decode_signal(message: &zbus::Message) -> Option<BusSignal> {
    let header = message.header();
    let sender = header.sender().map(ToString::to_string).unwrap_or_default();
    let member = header.member().map(ToString::to_string)?;
    let path = header.path().map(ToString::to_string).unwrap_or_default();
    return match member.as_str() {
        "suspendRequested" => Some(BusSignal::JobSuspend {
            sender,
            path,
        }),
        "resumeRequested" => Some(BusSignal::JobResume {
            sender,
            path,
        }),
        "ActionInvoked" => {
            let (id, key): (u32, String) = message.body().deserialize().ok()?;
            Some(BusSignal::ActionInvoked {
                sender,
                id,
                key,
            })
        }
        "NotificationClosed" => {
            let (id, reason): (u32, u32) = message.body().deserialize().ok()?;
            Some(BusSignal::NotificationClosed {
                sender,
                id,
                reason,
            })
        }
        _ => None,
    };
}

/// What:
/// `SIGNAL_QUEUE` is how many signals a listener's subscription holds before it drops the oldest.
/// Why:
/// the indicator gets a button press or a dismissal now and then,
/// so a short queue is never full.
const SIGNAL_QUEUE: usize = 64;

/// What:
/// `read_signals` forwards each decoded signal of `stream` to `forward` until the stream ends,
/// an error arrives,
/// or `forward` returns false.
/// Why:
/// the listener thread's whole body,
/// run inside one `zbus::block_on` call.
async fn read_signals<F>(
    mut stream: MessageStream,
    interface: &str,
    forward: F,
) where
    F: Fn(BusSignal) -> bool,
{
    while let Some(received) = stream.next().await {
        let message = match received {
            Ok(message) => message,
            Err(error) => {
                tracing::info!(error = %error, interface, "analysis signal stream ended with an error");
                return;
            }
        };
        if let Some(signal) = decode_signal(&message)
            && !forward(signal)
        {
            return;
        }
    }
    tracing::debug!(interface, "analysis signal stream ended");
}

/// What:
/// `spawn_listener` starts a thread that forwards the signals of `backend`'s interface to `forward`.
/// Why:
/// a signal stream cannot be waited on together with a channel,
/// so signals get their own thread.
/// The
/// thread ends when `forward` returns false (the indicator's thread is gone),
/// when the stream ends,
/// or when the
/// connection closes.
/// It subscribes before the first request,
/// so an early button press is not lost.
pub fn spawn_listener<F>(
    connection: &Connection,
    backend: Backend,
    forward: F,
) -> zbus::Result<JoinHandle<()>>
where
    F: Fn(BusSignal) -> bool + Send + 'static,
{
    let interface = match backend {
        Backend::Job => JOB_VIEW_INTERFACE,
        Backend::Notification => NOTIFICATIONS_INTERFACE,
    };
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .interface(interface)
        .build()?;
    let stream = step(MessageStream::for_match_rule(
        rule,
        connection,
        NonZeroUsize::new(SIGNAL_QUEUE),
    ))?;
    tracing::debug!(interface, "analysis indicator listening for signals");
    let handle = std::thread::Builder::new()
        .name(format!("analysis-signals-{interface}"))
        .spawn(move || {
            // The read waits for senders and ends with the stream, so it carries no bound of its own.
            zbus::block_on(read_signals(stream, interface, forward));
            tracing::debug!(interface, "analysis signal listener stopped");
        })?;
    return Ok(handle);
}
