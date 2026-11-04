//! What:
//!  The events of a settled commit with the fix summary the policies' corrections earned.
//! Why:
//!  The summary appears only after some pass changed candidate bytes, and a normalization that removed every change always
//!  reports it. The conclusion that once landed the commit itself (hook dispatcher, native preparation in the shadow, the
//!  landing loop) is gone: the resident service is the only writer of a commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return withChangedFixSummary({ result: pass, trigger: 'pre-forward', passes, changedPaths });
//! ```

use crate::policy_events::PolicyEvent;

/// What:
///  The settled pass's events with the fix summary the corrections earned.
/// Why:
///   The summary appears only after some pass changed candidate bytes, naming each changed path once in byte order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withChangedFixSummary({ result: pass, trigger: 'pre-forward', passes, changedPaths });
/// ```
pub fn events_with_changed_summary(events: Vec<PolicyEvent>, passes: u64, changed_paths: &[Vec<u8>]) -> Vec<PolicyEvent> {
    if passes == 0 {
        return events;
    }
    return events_with_summary(events, passes, changed_paths);
}

/// What:
///  The events with a fix summary appended whatever the pass count.
/// Why:
///   A normalization that removed every change always reports its summary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withFixSummary({ result, trigger: 'pre-forward', passes, changedPaths });
/// ```
pub fn events_with_summary(mut events: Vec<PolicyEvent>, passes: u64, changed_paths: &[Vec<u8>]) -> Vec<PolicyEvent> {
    let mut sorted: Vec<Vec<u8>> = changed_paths.to_vec();
    sorted.sort();
    events.push(PolicyEvent::FixSummary {
        trigger: crate::policy_trigger::Trigger::PreForward,
        passes,
        changed_paths: sorted
            .iter()
            .map(|path| { return crate::event_path::EventPath::from_git_bytes(path.as_slice()); })
            .collect(),
    });
    return events;
}

/// Summary controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_conclusion_tests.rs"]
mod tests;
