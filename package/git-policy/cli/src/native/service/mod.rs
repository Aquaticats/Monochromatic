//! What:
//!  The resident commit service:
//!  protocol,
//!  server,
//!  client,
//!  journal,
//!  provisioning and logging.
//! Why:
//!  The service is a second program with its own life.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // service/index.ts: one `export * from './file'` per module below.
//! ```

/// Messages between the wrapper and the resident commit service.
pub mod service_protocol;

/// The longest the wrapper waits for each thing it waits for, with the measurement beside each number.
pub mod wait_ceilings;

/// What the one landing thread is doing, shared with the threads that talk to waiting clients.
pub mod service_busy;

/// The connection of one waiting client, answered by the landing thread and kept alive by a beat from the reading thread.
pub mod service_reply;

/// The commit service's journal and its restart decisions.
pub mod service_journal;

/// Test-only fault points of the commit service.
pub mod service_fault;

/// The native journal protocol and the standard error fallback the service's entries go through.
pub mod journald_native;

/// The lines the commit service writes about its life and about the time each request took.
pub mod service_timing;

/// The line the commit service writes when a signal ends it.
pub mod service_signal;

/// The commit service's loop.
pub mod service_server;

/// The wrapper's side of a commit through the commit service.
pub mod service_client;

/// The wrapper's own provisioning of the commit service's socket activation.
pub mod service_provision;

/// A minimal D-Bus client for the user session bus, for the provisioning's two calls.
pub mod systemd_dbus;

/// Which commit service instance belongs to a worktree: its name, socket and unit instances.
pub mod service_instance;

/// One entry from the wrapper a person ran to the journal, sent without waiting.
#[cfg(target_os = "linux")]
pub mod client_journal;

/// The service's handler for a review work item, bounded by a call deadline and a total budget.
pub mod service_review;

/// The standards-review credential: redacted when printed, read from the wrapper's environment, sent in the work message.
pub mod service_review_key;
