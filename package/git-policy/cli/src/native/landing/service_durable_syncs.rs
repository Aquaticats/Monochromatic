//! What:
//!  The syncs one landing of a commit made,
//!  as its answer states them:
//!  what each was for and how long it took.
//! Why:
//!   A landing that made more syncs and a landing whose syncs met a stalling device
//!  look alike in the phases;
//!  the report of one landing in an fsync test spent 107.9 s
//!  and nothing said how many syncs made it.
//!  The record and the words for it live beside the code that makes the syncs,
//!  so the landing's answer states both from one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SyncFigure = { what: string; waited: boolean; took: number };
//! ```

use std::time::Duration;

/// What:
///  One file or directory sync a landing made,
/// or handed to the background,
/// as the landing's answer states it.
/// Why:
///   The report of one landing in an fsync test spent 107.9 s,
/// of which the index was 57.8 s
/// and the journal 20.6 s;
/// nothing said how many syncs the landing made
/// or how long each took,
/// so a landing that slowed down when the device stalled could not be told from
/// one that made more syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SyncFigure = { what: string; waited: boolean; took: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SyncFigure {
    /// What the sync is for,
    /// as `flush_file` and `flush_directory` name it.
    pub what: &'static str,
    /// Whether the landing waited for the sync (`true`) or handed it to the background task (`false`).
    pub waited: bool,
    /// How long the sync took,
    /// which is zero for one the landing handed to the background and never waited for.
    pub took: Duration,
}

/// The words an answer gives for the syncs of its landing:
/// `what=queued` for one the landing handed to the background,
/// `what=<ms>` for one it waited for,
/// in the order it asked for them,
/// and nothing when the landing made none.
pub fn syncs_word(syncs: &[SyncFigure]) -> String {
    let mut words: Vec<String> = Vec::new();
    for sync in syncs {
        if sync.waited {
            words.push(format!("{}={}", sync.what, sync.took.as_millis()));
        } else {
            words.push(format!("{}=queued", sync.what));
        }
    }
    return words.join(";");
}

/// How many syncs of the landing the landing waited for.
pub fn syncs_waited_count(syncs: &[SyncFigure]) -> usize {
    let mut waited: usize = 0;
    for sync in syncs {
        if sync.waited {
            waited += 1;
        }
    }
    return waited;
}

/// What the landing spent waiting for its own syncs,
/// which is the part of the landing's phases a stalled device shows in.
pub fn syncs_held(syncs: &[SyncFigure]) -> Duration {
    let mut total: Duration = Duration::ZERO;
    for sync in syncs {
        if sync.waited {
            total += sync.took;
        }
    }
    return total;
}
