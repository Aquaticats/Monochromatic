//! What:
//!  The finished record a landing writes,
//!  how long a control waits for it,
//!  and what a control says when it does not arrive.
//! Why:
//!  A landing writes its finished record after the index it installs,
//!  and a wrapper whose answer was lost answers the landed commit from the branch that landing has already moved
//!  (`SPEC.md`,
//!  "Before that read,
//!  and for every failure that means an answer was lost after the request was sent").
//!  So a control that counted the journal's finished records the instant its command ended
//!  counted a landing that had not finished (issue 6572),
//!  and the wait for that record belongs beside the count.
//!  Both stand in this file rather than in the harness that calls them,
//!  which is at the per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // awaitTheFinishedRecord(fixture, before, name, observed) // returns once the journal holds a newer record
//! ```

use super::support::{
    Fixture,
    Observed,
};
use git_policy_cli::service_journal::{
    DONE_EXTENSION,
    OPEN_EXTENSION,
};
use std::path::PathBuf;
use std::time::{
    Duration,
    Instant,
};

/// How long a control waits for the finished record of a landing whose commit it has answered,
/// counted from the moment the wrapped command ended.
/// Why:
///  The record is written after the index the landing installs,
///  and that index update is granted no ceiling:
///  it costs one read of every path the commit changed.
///  Measured on 2026-11-27 in this control's own run,
///  at a load average of 36 to 48,
///  the two landings of 4,097 paths took 17.5 s and 25.8 s,
///  which the service states as `index_update_ms` in its answered line.
///  At a load average of 123 to 186 the same two took 30.3 s and 32.1 s
///  (`doc/troubleshooting/git-policy-cli-landing-index-reset-reads-every-changed-path.md`).
///  So this window is a backstop that only a landing which never writes its record reaches:
///  3.7 times the longest of those four measurements,
///  twice the 60 s `NATIVE_GIT_BOUND_SECONDS` bounds one command of this harness with,
///  and inside the four minutes the awkward change list control gives its child.
const FINISHED_RECORD_WAIT: Duration = Duration::from_secs(2 * 60);

/// How often the wait above reads the journal.
/// Why:
///  A landing that finishes writes its record once,
///  and the wait only has to notice it before its bound:
///  a tenth of a second is far finer than the seconds a landing takes,
///  and costs one directory listing.
const FINISHED_RECORD_LOOK_EVERY: Duration = Duration::from_millis(100);

/// The number of batches the service has finished in the fixture.
pub(super) fn finished(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| {
                return entry.path().extension().is_some_and(|extension| {
                    return extension == DONE_EXTENSION;
                });
            })
            .count();
    });
}

/// What:
///  Waits, within `FINISHED_RECORD_WAIT`,
///  until the journal holds more finished records than it held before the command,
///  and fails at that bound with the records it does hold and what the wrapper printed.
/// Why:
///  This wait is where a control learns that its landing finished,
///  so the words of a failure name the journal's own batch records,
///  which tell a landing that never settled (an open intent) from one whose record came late (a finished record),
///  and both streams of the wrapper,
///  whose standard error names a lost answer and the silence it was lost to.
pub(super) fn await_the_finished_record(
    fixture: &Fixture,
    before: usize,
    entry: &str,
    observed: &Observed,
) {
    let began: Instant = Instant::now();
    loop {
        let now: usize = finished(fixture);
        if now > before {
            return;
        }
        let waited: Duration = began.elapsed();
        if waited >= FINISHED_RECORD_WAIT {
            panic!(
                "{entry}: the service landed it, and its finished record did not reach the journal within \
{FINISHED_RECORD_WAIT:?}: the journal held {before} finished records before the commit and {now} after it, and its batch \
records are [{}]\nthe wrapper's standard error, which names an answer it lost, was:\n{}\nthe wrapper's standard output \
was:\n{}",
                the_batch_records(fixture),
                String::from_utf8_lossy(observed.stderr.as_slice()),
                String::from_utf8_lossy(observed.stdout.as_slice()),
            );
        }
        std::thread::sleep(FINISHED_RECORD_LOOK_EVERY.min(FINISHED_RECORD_WAIT - waited));
    }
}

/// What:
///  The batch records the journal holds,
///  by name and in order:
///  the open intent of a landing that has not finished,
///  and the finished record of one that has.
/// Why:
///  A failure that names them tells those two apart
///  (`OPEN_EXTENSION` and `DONE_EXTENSION`),
///  and every other entry of the journal (a claim, a temporary file, a note for the next service)
///  says nothing about a landing of this control.
fn the_batch_records(fixture: &Fixture) -> String {
    let journal: PathBuf = fixture.root.join("journal");
    let Ok(entries) = std::fs::read_dir(&journal) else {
        return String::from("(no journal)");
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| {
            return entry.path().extension().is_some_and(|extension| {
                return extension == DONE_EXTENSION || extension == OPEN_EXTENSION;
            });
        })
        .map(|entry| {
            return entry.file_name().to_string_lossy().into_owned();
        })
        .collect();
    names.sort();
    return names.join(", ");
}
