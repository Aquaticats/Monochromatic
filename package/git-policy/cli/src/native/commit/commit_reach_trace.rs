//! What:
//!  What reaching a commit service cost the client:
//!  the connection attempts,
//!  the waits for a start,
//!  the services of other releases asked to retire,
//!  and what a waiting notice said of a foreign index holder.
//! Why:
//!  A slow commit that began by reaching a service of another release must be told apart from a slow landing,
//!  so the record states the tries,
//!  the waits and the releases that answered;
//!  and a wait for another program's hold on the index is only a wait for a proven holder if the notice proved one.
//!  These live beside the record that writes them,
//! in a file of their own,
//!  because the record's own file is at the length this package allows
//!  (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ReachTrace = { tries: number; refused: number; startWaited: number; retiresSent: number; releasesSeen: string[]; answeredBy?: string };
//! ```

use std::time::Duration;

/// The word written for a release a service did not announce (the services of the earliest releases sent no release).
pub const UNANNOUNCED: &str = "unannounced";

/// What:
///  What the client did to reach a service that speaks its protocol and was started from its release.
/// Why:
///   The 17-second commit began with a service of another release being asked to retire;
/// the tries,
/// the wait for a start,
/// the
///  retires sent and the releases that answered tell that apart from a slow landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReachTrace = { tries: number; refused: number; startWaited: number; retiresSent: number; releasesSeen: string[]; answeredBy?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReachTrace {
    /// Connection attempts made,
    /// each one a `connect` call.
    pub tries: usize,
    /// Attempts that found nothing listening or no greeting,
    /// which is the service being started by the manager.
    pub refused: usize,
    /// Time from the first attempt of a reach to the greeting,
    /// summed over the reaches that needed more than one attempt.
    pub start_waited: Duration,
    /// Retire messages sent to services of another release.
    pub retires_sent: usize,
    /// The release of every greeting,
    /// in the order received,
    /// without repeats.
    pub releases_seen: Vec<String>,
    /// The release of the service that finally took the request.
    pub answered_by: Option<String>,
    /// Time spent reaching,
    /// retire rounds included.
    pub elapsed: Duration,
}

impl ReachTrace {
    /// Notes the release a greeting named.
    pub fn saw_release(&mut self, release: Option<&str>) {
        let word: String = release.unwrap_or(UNANNOUNCED).to_string();
        if !self.releases_seen.contains(&word) {
            self.releases_seen.push(word);
        }
    }
}

/// What:
///  What a waiting notice of the service says of the holder of the index lock.
/// Why:
///   The notice is the only thing about the holder that reaches the client,
/// and the record should say whether the wait was for a process proven to hold the lock or for one nobody could prove alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderKindOf(text) // 'proven-alive' | 'unproven'
/// ```
pub fn holder_kind_of(text: &str) -> &'static str {
    return if text.contains("(proven by ") { "proven-alive" } else { "unproven" };
}

/// What:
///  The kind of holder a wait was for after one more waiting notice,
/// given the kind the earlier notices gave.
/// Why:
///  A proof counts when it comes and is never undone:
/// the notices after the one that proved the holder (the way out of a wait that reached its ceiling,
/// the service's own notice that it is inside a write the moment the holder lets go) say nothing of the holder,
/// and the last notice must not decide what kind of holder the wait was for
/// (the heartbeat of the connection and the holder's release meet at the same instant in the control that releases it after a
/// whole number of beats).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderKindAfter(earlier, text) // earlier === 'proven-alive' ? 'proven-alive' : holderKindOf(text)
/// ```
pub fn holder_kind_after(earlier: Option<&'static str>, text: &str) -> &'static str {
    return match earlier {
        Some("proven-alive") => "proven-alive",
        Some(_) | None => holder_kind_of(text),
    };
}
