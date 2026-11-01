//! Refusals of the taskbar verbs when no private bus runs.

/// Brings the verbs, the refusal text, and the wire formatter into scope.
use super::*;

/// Wire formatter, so the test checks the exact line a client reads.
use crate::protocol::format_response;

/// Without a private bus every taskbar verb answers the same `err` line.
#[test]
fn every_taskbar_verb_refuses_without_a_private_bus() {
    let line = "err no private session bus (start with --launcher-log or --color-scheme)";
    assert_eq!(format_response(&latest(None, Instant::now())), line);
    assert_eq!(format_response(&log(None, 64)), line);
    assert_eq!(format_response(&clear(None)), line);
}
