//! What:
//!  The wrapper's side of a commit through the resident service:
//! connect,
//! wait honestly while the operating system
//!  starts the service,
//! send the request,
//! show the queue position,
//! survive a service that dies mid-request,
//! and turn
//!  every failure into the plain text of `service_messages.rs`.
//! Why:
//!  A failure the system can resolve is resolved here without the user:
//! a service that is starting is waited for,
//! one
//!  that crashed is reconnected to with the same request identity,
//! one that was started from another release of the
//!  wrapper (older or newer,
//! found by the release its welcome names) is asked to retire and is reached again once the
//!  manager has started the current one.
//! Only what cannot be resolved ends the command.
//!
//! The parts of this side,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `service_client_what_a_request_came_to.rs`:
//!   what a request came to:
//!   landed,
//!   kept behind a reference lock,
//!   prepared again,
//!   or not made.
//! - `service_client_the_failure_a_person_reads.rs`:
//!   the plain failure a person reads for each code the service answers with.
//! - `service_client_withdraw_a_request.rs`:
//!   asks the service to drop a request this command sent,
//!   and turns its answer into the command's end.
//! - `service_client_scratch_renewal.rs`:
//!   the renewal of the command's scratch directory while a commit waits.
//! - `service_client_wait_for_the_answer.rs`:
//!   the wait for the answer to a request that was written,
//!   with what it remembers between two messages.
//! - `service_client_send_and_wait.rs`:
//!   the sends,
//!   the one identity each carries,
//!   and the tries a service that went silent or closed its connection is given.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await submit(config, request, stderr); // landed, or prepare again, or a ClientFailure
//! ```

/// The notification that a request was written,
/// the wait to start the service,
/// and the ask of an older service to retire.
pub use crate::service_client_listen_and_ask_to_retire::{
    START_WAIT,
    ask_to_retire,
};
/// Sends a commit request and waits until it is answered,
/// leaving in its trace how the service was reached and how long the answer took.
pub use crate::service_client_send_and_wait::{
    submit,
    submit_traced,
};
/// The settings of one client run and the watch on the service manager's answer.
pub use crate::service_client_settings_and_watches::{
    Activation,
    ClientConfig,
    FreshScratch,
    ManagerAnswer,
    ManagerWatch,
    START_NOTICE_GRACE,
    manager_watch_for,
};
/// The plain failure a person reads for a code the service answered with,
/// and the read-only one that asks whether a repository move froze this repository.
pub use crate::service_client_the_failure_a_person_reads::{
    failure_for,
    read_only_failure,
};
/// The client's one-way message that work items were written for a repository.
pub use crate::service_client_wake::wake;
/// What a request came to.
pub use crate::service_client_what_a_request_came_to::Submitted;

/// The client's bounds and how it reads the service's lines.
#[cfg(test)]
#[path = "service_client_tests.rs"]
mod tests;
