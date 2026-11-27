//! What:
//!  Writes a client message or a service message as one JSON object on one line.
//! Why:
//!  The line form needs no framing library;
//! the types come from `service_protocol_messages_and_name_checks.rs`
//!  and the decoder that reads these lines is in `service_protocol_decode_lines_to_messages.rs`.
//!
//! The parts of these writers,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `service_protocol_encode_the_fields_of_a_message.rs`:
//!   one field,
//!   one changed path,
//!   one cleanup,
//!   one sync,
//!   the figures of a landing,
//!   the programs a command carried,
//!   and the one object every message is written as.
//! - `service_protocol_encode_the_commit_line.rs`:
//!   the line for one commit request,
//!   and whether it fits the ceiling both sides accept.
//! - `service_protocol_encode_client_messages.rs`:
//!   a retire at a named version,
//!   and every message a client writes.
//! - `service_protocol_encode_service_messages.rs`:
//!   every message a service writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socket.write(JSON.stringify(encodeRequest(request)) + '\n');
//! ```

/// The line that asks a service to retire,
/// and the line for every client message.
pub use crate::service_protocol_encode_client_messages::{
    encode_client,
    encode_retire,
};
/// The line for one service message.
pub use crate::service_protocol_encode_service_messages::encode_service;
/// The line for one commit request,
/// and whether that line is one the service accepts.
pub use crate::service_protocol_encode_the_commit_line::{
    commit_line,
    commit_request_fits,
};
