//! What:
//!  The small JSON words one timing record is written with:
//!  a duration in whole milliseconds,
//!  the word for a field that has no value,
//!  and the landing's own figures as one object.
//! Why:
//!  A reader of the file parses every record the same way,
//!  so one absent figure is one word and one figure is one number,
//!  and the record of a slow commit explains its landing without a second file
//!  read by request identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landingJson = (l) => l === null ? 'null' : JSON.stringify({ lockHoldMs: l.lockHoldMs, ... });
//! ```

/// The one quote every name in a record is written with.
use crate::json_record::quote;
/// The landing figures a service of this release answers with.
use crate::service_protocol::LandingFigures;
/// `Duration` is an elapsed stretch of time from the monotonic clock.
use std::time::Duration;

/// The whole milliseconds of a duration.
pub(crate) fn whole_milliseconds(elapsed: Duration) -> u128 {
    return elapsed.as_millis();
}

/// The JSON word for a field that has no value.
pub(crate) fn null_word() -> String {
    return String::from("null");
}

/// What:
///  The landing's own figures as one JSON object,
/// or the JSON word for no value when the service sent none.
/// Why:
///   The record of a slow commit then explains its landing on its own:
/// which read spent the time,
/// how long the landing held what others queued on,
/// how many commits queued behind it
/// and how many syncs it made with the time of each,
/// without a second file read by request identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landingJson = (l) => l === null ? 'null' : JSON.stringify({ lockHoldMs: l.lockHoldMs, queuedBehind: l.queuedBehind, readsMs: l.readsMs, reads: Object.fromEntries(l.reads), syncs: l.syncs });
/// ```
pub(crate) fn landing_json(landing: Option<&LandingFigures>) -> String {
    let Some(figures) = landing else {
        return null_word();
    };
    let mut reads: Vec<String> = Vec::new();
    for (name, ms) in &figures.reads {
        reads.push(format!("{}:{ms}", quote(name.as_str())));
    }
    let mut syncs: Vec<String> = Vec::new();
    for sync in &figures.syncs {
        syncs.push(format!(
            "{{\"what\":{},\"waited\":{},\"ms\":{}}}",
            quote(sync.what.as_str()),
            sync.waited,
            sync.ms
        ));
    }
    return format!(
        "{{\"lockHoldMs\":{},\"queuedBehind\":{},\"readsMs\":{},\"reads\":{{{}}},\"syncs\":[{}]}}",
        figures.lock_hold_ms,
        figures.queued_behind,
        figures.reads_ms,
        reads.join(","),
        syncs.join(","),
    );
}
