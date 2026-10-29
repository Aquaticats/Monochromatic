//! The fake notification and Plasma job services,
//!  driven by real clients on real private buses.
//!
//! One test crate holds every case,
//!  so the shared helpers in `support` are used by the crate as a whole.

/// Helpers:
///  a private bus,
///  the services,
///  a client,
///  and signal subscriptions.
mod support;

/// The freedesktop notification server.
mod notifications;

/// The Plasma job view server.
mod jobs;

/// The default mode,
///  which serves nothing.
mod none;

/// The fake's introspected interfaces against Plasma's own interface XML.
mod contract;
