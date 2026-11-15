//! What:
//!  The words a waiting push record is named with:
//! that the push has not been made,
//! the bound a running service looks at a repository in,
//! and how long the
//!  record has waited when that is longer than the bound.
//! Why:
//!   They are the only place a person learns whether anything is looking at the record,
//! so they name what a running
//!  service does,
//! and the state in which nothing looked,
//! and neither can drift from the constant the service's look uses
//!  (`LOOK_AGAIN_AFTER` in `service_claim_watch.rs`).
//! The line is read from the record itself
//! (`queued_push_notices` in
//!  `service_handoff.rs`),
//! so a command that wrote nothing cannot leave a stale one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! queuedPushLine(branchRef, commit, waited); waitedWords(waited)
//! ```

/// What:
///  The one sentence the worktree says about a push that is queued and has not been made.
/// Why:
///   What is known is that the record is queued and that no service has taken it,
/// and what a running service does with it:
/// it
///  looks at the repositories its journal names by itself every `LOOK_AGAIN_AFTER`,
/// so the record needs no commit and no wake
///  to be run;
/// the sentence names that bound,
/// so a person can tell a push that is about to go from one nothing is looking at.
/// A
///  record that has waited longer than the bound is said with how long it has waited,
/// because that is the case in which no service
///  looked at the repository at all,
/// and the person needs the number that says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queuedPushLine(branchRef, commit, waited)
/// ```
pub fn queued_push_line(reference: &str, commit: &str, waited: std::time::Duration) -> String {
    let bound: String = crate::kept_reference_words::span_words(crate::service_claim_watch::LOOK_AGAIN_AFTER);
    let from: String = match reference.is_empty() {
        true => String::new(),
        false => format!(" from {reference}"),
    };
    if waited > crate::service_claim_watch::LOOK_AGAIN_AFTER {
        return format!(
            "the commit {commit} is queued for its push{from}: the push has not been made yet, and this record has waited {}, longer than the {bound} in \
             which a running commit service looks at this repository's records by itself, so no service has looked at this repository since the \
             record was written; the push needs no further commit",
            waited_words(waited)
        );
    }
    return format!(
        "the commit {commit} is queued for its push{from}: the push has not been made yet, and a running commit service looks at this repository's \
         records by itself every {bound}, so the push needs no further commit"
    );
}

/// What:
///  A wait in the words a person reads.
/// Why:
///   `age_words` is written for days and hours of a kept reference and prints `0 minutes` for a wait of seconds,
/// which says
///  nothing here;
/// a wait this line names is one from fifteen seconds to hours,
/// so seconds are said as seconds and anything longer
///  reads as `age_words` prints it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitedWords = (waited) => waited < 2 * 60_000 ? `${Math.floor(waited / 1000)} seconds` : ageWords(waited);
/// ```
fn waited_words(waited: std::time::Duration) -> String {
    if waited.as_secs() < 120 {
        return format!("{} {}", waited.as_secs(), crate::kept_reference_words::counted(u128::from(waited.as_secs()), "second"));
    }
    return crate::kept_reference_words::age_words(waited.as_millis());
}
