//! What:
//!  The messages between the wrapper and the resident commit service,
//! one JSON object per line.
//! Why:
//!  A client sends a prepared change with a request identity and waits;
//! the service answers with a landing,
//! a queue
//!  position,
//! a request to prepare again,
//! or a failure from a closed list.
//! The line form needs no framing library and
//!  every field is checked on read,
//! so a damaged or hostile line is refused rather than guessed
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Protocol and request identity").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socket.write(JSON.stringify(encodeRequest(request)) + '\n'); const reply = decodeResponse(await readLine(socket));
//! ```
/// The message types and name checks,
/// re-exported under the path they had in this file.
pub use crate::service_protocol_messages_and_name_checks::{
    PROTOCOL_VERSION, RELEASE, HEARTBEAT_EVERY, HEARTBEAT_SILENCE, NOTE_OF_A_BEAT_CEILING, P99_CHANGED_PATH_CHARACTERS, MAX_LINE_BYTES, MAX_CHANGES, MAX_ID_BYTES, ALLOWED_MODES, Change, Cleanup, CommitRequest, LandedCapture, LandingFigures, LandingSync, is_zero_object, FailureCode, ClientMessage, ServiceMessage, is_object_name, is_plain_path, SCRATCH_PREFIX, is_scratch_path,
};
/// The line writers,
/// re-exported under the path they had in this file.
pub use crate::service_protocol_encode_messages_to_lines::{commit_line, commit_request_fits, encode_client, encode_retire, encode_service};
/// The line readers,
/// re-exported under the path they had in this file.
pub use crate::service_protocol_decode_lines_to_messages::{decode_client, decode_service};

/// Protocol controls stay out of the release executable.
#[cfg(test)]
#[path = "service_protocol_tests.rs"]
mod tests;
