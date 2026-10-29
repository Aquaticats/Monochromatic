//! Waiting in the native language tests:
//!  advancing timers until a state holds,
//!  and waiting for
//! the scripted server to finish its start within the product's start deadline.

/// The reader whose binding reports the server's state,
///  and the server's name.
use super::test_support::{LanguageReader, SERVER};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Server states as the binding reports them.
use ide_app::language::status::ServerState;
/// Deadline for the server's start.
use std::time::Duration;

/// What:
///  Longest wait for the scripted server to become ready or to fail its start.
/// Why:
///  This is the product's start deadline for the server these tests define:
///  `initialize`
///      gets three request timeouts of 5 s (`START_FACTOR` in `src/language/config.rs`),
///  and the
///      worker reports the start failed 2 s after that (`START_MARGIN` in
///      `src/language/attach/start.rs`).
///  A shorter wait fails a start the product still accepts.
///      If either constant or the `timeout` in `definitions` changes,
///  this follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const READY_PATIENCE_MS = 3 * 5_000 + 2_000;
/// ```
const READY_PATIENCE: Duration = Duration::from_secs(17);

/// What:
///  The scripted server's state as the binding last polled it,
///  if it is listed.
///       `Option<ServerState>` is "a state,
///  or nothing" (sibling:
///  a borrowed `&ServerState`).
/// Why:
///  The readiness wait stops on a failed start and names that state in its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serverState(reader: LanguageReader): ServerState | undefined
/// ```
fn server_state(reader: &LanguageReader) -> Option<ServerState> {
    // The trailing `?` leaves with nothing after the binding was closed.
    let binding = reader.binding.as_ref()?;
    let language = binding.language.borrow();
    // The first row for the server decides; its state is copied out of the borrow.
    for row in &language.status.servers {
        if row.server.name == SERVER {
            return Some(row.state.clone());
        }
    }
    return None;
}

/// Wait until the scripted server is ready for the displayed file,
///  as the binding polled it.
/// A failed start ends the wait at once,
///  and the message names the state last seen.
pub(super) fn ready(reader: &LanguageReader) {
    let mut waiting = Waiting::within(READY_PATIENCE);
    while !matches!(
        server_state(reader),
        Some(ServerState::Ready | ServerState::FailedToStart { .. })
    ) {
        waiting
            .step("the scripted server neither became ready nor failed within its start deadline");
    }
    assert_eq!(
        server_state(reader),
        Some(ServerState::Ready),
        "the scripted server did not become ready"
    );
}
