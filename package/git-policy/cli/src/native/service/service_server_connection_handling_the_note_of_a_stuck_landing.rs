//! What:
//!  The note a connection's beat sends about what the landing thread is stuck in:
//!  the words of a write that is never ended,
//!  said again only when they change,
//!  with the read of the journal that outlives the beat which started it.
//! Why:
//!  A client queued behind a write that never returns is told why it waits,
//!  and the words become the blocker its error names;
//!  a read of the journal can wait on storage,
//!  so one read is kept and the beats behind it wait for that same read
//!  instead of each holding a thread of the blocking pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class StuckNotice { board; request; last; stillReading; async note() { ... } }
//! ```

use crate::blocking_job::run_blocking_job;
use crate::service_busy::LandingBoard;
use crate::service_protocol::ServiceMessage;
use crate::service_reply::NoteSource;
use std::sync::Arc;
/// What:
///  The point at which a beat's note of `request` reads the journal,
///  for a control that holds that read (`service_fault_hooks_test_support.rs`).
/// Why:
///   The point names the connection's own request,
///  so two controls of one process hold the note read of their own connection and neither answers the other
///  (issue 6497).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteReadPoint = (request?: string) => `the-note-read-of-${request ?? ''}`;
/// ```
#[cfg(test)]
pub(crate) fn note_read_point(request: Option<&str>) -> String {
    return format!("the-note-read-of-{}", request.unwrap_or_default());
}
/// What:
///  The note for a connection's beat:
/// what the landing thread is stuck in when it is inside a write that is never ended,
/// said again
///  only when the words change (they name whole minutes).
/// Why:
///   A client queued behind a write that never returns is told why it waits,
/// and the words become the blocker its error names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// () => { const words = board.describe(); return words !== last ? (last = words, { type: 'waiting', text: words }) : undefined; }
/// ```
pub(crate) fn stuck_notice(
    board: Arc<LandingBoard>,
    request: Option<String>,
) -> StuckNotice {
    return StuckNotice {
        board,
        request,
        last: None,
        still_reading: None,
    };
}
/// The answer of one read of the journal for a beat's note:
/// what the connection has to say,
/// and the key of the words said last.
type NoteFromTheBoard =
    Result<(Option<ServiceMessage>, Option<String>), crate::blocking_job::BlockingJobError>;

/// What:
///  The read of one note while it is still running.
/// Why:
///  A read of the journal can outlive the beat that started it,
/// and a source that started another read every beat would queue one read a second behind a storage that is late,
///  each of them holding a thread of the blocking pool that the commits of every other client need;
///  one read is kept here and the next beat waits for that same read,
/// which is also the only way the words of a read that took several beats are ever written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NoteRead = Promise<[ServiceMessage | undefined, string | undefined]>;
/// ```
type NoteRead = std::pin::Pin<Box<dyn std::future::Future<Output = NoteFromTheBoard> + Send>>;

/// What:
///  The note source of a connection's beat:
/// what the landing thread is stuck in,
/// said again only when the words change,
///  with the read that is still running kept for the beats behind it.
/// Why:
///   The note is read from the journal on every beat,
/// and one read that storage holds must cost a read and never a beat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StuckNotice { board; request; last; stillReading; async note() { ... } }
/// ```
pub(crate) struct StuckNotice {
    /// What the landing thread reports about itself.
    board: Arc<LandingBoard>,
    /// The identity of the request the connection serves,
    /// when it serves one.
    request: Option<String>,
    /// The key of the words said last.
    last: Option<String>,
    /// The read of this connection's note that is still running,
    /// and nothing when no read is.
    still_reading: Option<NoteRead>,
}

/// The note the board has for a connection and the key of the words said last after it,
/// given the key said before.
/// The board is files of the journal,
/// which can wait on storage,
/// so the reads run on the blocking pool.
fn note_from_the_board(
    board: &LandingBoard,
    request: Option<&str>,
    last: Option<String>,
) -> (Option<ServiceMessage>, Option<String>) {
    // A control holds this read on whichever thread runs it, which is a pool thread and not the thread that set the hold.
    #[cfg(test)]
    {
        let held_point: String = note_read_point(request);
        crate::service_fault::hooks::run_on_any_thread(held_point.as_str());
    }
    // A client whose own request is in the group being landed is told so when the write starts, because its commit may be on
    // the branch already; the key changes when the longer words of a minute's stuck write replace the first ones.
    let inside: Option<(String, String)> = match request {
        Some(id) => board.inside_notice(id),
        None => None,
    };
    if let Some((key, words)) = inside {
        if last.as_deref() == Some(key.as_str()) {
            return (None, last);
        }
        return (
            Some(ServiceMessage::InsideWrite {
                text: words,
            }),
            Some(key),
        );
    }
    let Some(words) = board.describe() else {
        return (None, last);
    };
    if last.as_deref() == Some(words.as_str()) {
        return (None, last);
    }
    return (
        Some(ServiceMessage::Waiting {
            text: words.clone(),
        }),
        Some(words),
    );
}

impl NoteSource for StuckNotice {
    async fn note(&mut self) -> Option<ServiceMessage> {
        if self.still_reading.is_none() {
            let board: Arc<LandingBoard> = Arc::clone(&self.board);
            let request: Option<String> = self.request.clone();
            // Cloned rather than taken: the words said last stay the words that were said until a read answers with the next
            //  ones, and a read that outlives its beat answers only beats later.
            let last: Option<String> = self.last.clone();
            self.still_reading = Some(Box::pin(run_blocking_job(move || {
                return note_from_the_board(&board, request.as_deref(), last);
            })));
        }
        let running: &mut NoteRead = self
            .still_reading
            .as_mut()
            .expect("the read of this connection's note was just started");
        // A wait that passes here leaves the read in `still_reading`, so the beat behind this one waits for the same read instead
        //  of starting a second one behind a storage that is already late.
        let answer: NoteFromTheBoard = running.await;
        // This read is over, whatever it answered: the beat behind this one starts the next read.
        self.still_reading = None;
        return match answer {
            Ok((message, last_said)) => {
                self.last = last_said;
                message
            }
            Err(error) => {
                // No note this beat: the beat itself is the proof of life and is still written.
                crate::service_warn!(
                    "the note of what the landing is inside could not be read for a waiting client: {error}"
                );
                None
            }
        };
    }
}
