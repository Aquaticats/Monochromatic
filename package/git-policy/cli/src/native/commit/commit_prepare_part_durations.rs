//! What:
//!  How long each stretch of a commit's preparation held no Git child,
//!  by name,
//!  kept until the commit's timing record takes the figures.
//! Why:
//!   The preparation is the largest bucket the wrapper owns outright:
//!  21.7 percent of the uncovered time of 866 commits,
//!  median 332 ms,
//!  p99 11 884 ms
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//!  "The preparation's private policy index and its in-process work").
//!  The record named Git steps and phases,
//!  not the in-process stretches between them,
//!  so no reading could say which stretch spends the milliseconds.
//!  Each stretch is measured from one clock read at each seam to the next,
//!  and the Git steps that ran inside it are taken out of it,
//!  because those are recorded apart and a stretch that holds one is not waiting.
//!  A name is one of a fixed set compiled into the wrapper,
//!  never a path,
//!  a message or an option value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parts: { name: string; ms: number }[] = [];
//! const clock = beginPartClock('layout');
//! // ... work ...
//! clock.next('capture');
//! export const drainPrepareParts = () => parts.splice(0);
//! ```

/// Escaping of a name inside a JSON string.
use crate::json_record::quote;
/// The Git steps a stretch holds,
/// whose time is recorded apart from the stretch.
use crate::transaction_step_durations::step_time_between;
/// A lock around the shared list.
use std::sync::Mutex;
/// A moment and a span of time.
use std::time::{Duration, Instant};

/// The most stretches one record lists.
///
/// Measured on the shapes of the code rather than on a run:
/// one preparation marks about fifteen seams,
/// and a commit prepares at most `LANDING_ATTEMPTS` (3) times,
/// 45 stretches,
/// so 64 is the next power of two above it,
/// about 40 bytes an entry and under 3 KiB of a one mebibyte timing file.
/// The record states how many went unlisted,
/// so a longer run is counted,
/// not lost silently.
pub const MAX_PREPARE_PARTS: usize = 64;

/// One stretch of a preparation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparePart {
    /// What the preparation was doing,
    /// as one of the names compiled into the wrapper.
    pub name: String,
    /// How long it did that without a Git child.
    pub took: Duration,
}

/// The stretches of this process since a record last took them,
/// in the order they ended.
static PARTS: Mutex<Vec<PreparePart>> = Mutex::new(Vec::new());

/// How many stretches came after the list was full.
static DROPPED: Mutex<usize> = Mutex::new(0);

/// What:
///  Keeps one stretch of a preparation.
/// Why:
///   Every seam of every preparation reports here,
///  whether the preparation went on to land or stopped where it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// notePreparePart('layout', 12);
/// ```
pub fn note_prepare_part(name: &str, took: Duration) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of stretches.
    let mut parts = PARTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if parts.len() >= MAX_PREPARE_PARTS {
        let mut dropped = DROPPED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *dropped += 1;
        return;
    }
    parts.push(PreparePart { name: String::from(name), took });
}

/// What:
///  Every stretch kept and the number that did not fit,
/// in the order they ended;
/// the list is emptied.
/// Why:
///   One record takes the stretches of the command it describes;
/// the next command starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { parts, dropped } = drainPrepareParts();
/// ```
pub fn drain_prepare_parts() -> (Vec<PreparePart>, usize) {
    let mut parts = PARTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let taken: Vec<PreparePart> = std::mem::take(&mut *parts);
    let mut dropped = DROPPED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let lost: usize = std::mem::take(&mut *dropped);
    return (taken, lost);
}

/// The stretches of one name added together,
/// with the number of times the name was marked,
/// in the order the names first appeared.
fn totals_of(parts: &[PreparePart]) -> Vec<(String, Duration, usize)> {
    let mut totals: Vec<(String, Duration, usize)> = Vec::new();
    for part in parts {
        let mut found: bool = false;
        for (name, sum, calls) in &mut totals {
            if *name != part.name {
                continue;
            }
            *sum += part.took;
            *calls += 1;
            found = true;
            break;
        }
        if !found {
            totals.push((part.name.clone(), part.took, 1));
        }
    }
    return totals;
}

/// What:
///  The stretches as a JSON object,
/// `{"layout":{"ms":12,"calls":1},...}`,
/// one member per name,
/// with the times of a name marked more than once added together.
/// Why:
///   A retry marks the same names again,
///  and a reading wants the whole of what the command spent in each.
///  `calls` says how many stretches a sum is made of,
///  so a name that was marked twice is not read as one long stretch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(Object.fromEntries(totals.map(([name, sum, calls]) => [name, { ms: sum, calls }])))
/// ```
pub fn prepare_parts_json(parts: &[PreparePart]) -> String {
    let mut members: Vec<String> = Vec::new();
    for (name, sum, calls) in totals_of(parts) {
        members.push(format!("{}:{{\"ms\":{},\"calls\":{}}}", quote(name.as_str()), sum.as_millis(), calls));
    }
    return format!("{{{}}}", members.join(","));
}

/// What:
///  The stretch of a preparation that is running,
///  marked at each seam,
///  and kept when it ends.
/// Why:
///   The marks are the seams of the preparation,
/// one line each,
/// so a stretch is
///  named by the block it covers rather than by a timer every call leaves.
///  A preparation that leaves early,
///  on a stop or a failure,
///  keeps the stretch it was in when it left,
///  because the value keeps it as it goes out of scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using clock = beginPartClock('layout'); clock.next('capture');
/// ```
#[derive(Debug)]
pub struct PreparePartClock {
    /// Where the running stretch began.
    began: Instant,
    /// What the preparation is doing in the running stretch.
    name: &'static str,
}

impl PreparePartClock {
    /// Starts measuring,
    /// with the name of the stretch that is about to run.
    pub fn begin(name: &'static str) -> Self {
        return Self { began: Instant::now(), name };
    }

    /// Ends the running stretch and starts the next one,
    /// named `name`.
    pub fn next(&mut self, name: &'static str) {
        self.close();
        self.name = name;
    }

    /// Ends the running stretch,
    /// keeping what it spent without a Git child.
    fn close(&mut self) {
        let ended: Instant = Instant::now();
        let whole: Duration = ended.saturating_duration_since(self.began);
        let children: Duration = step_time_between(self.began, ended);
        note_prepare_part(self.name, whole.saturating_sub(children));
        self.began = ended;
    }
}

impl Drop for PreparePartClock {
    /// Ends the stretch the preparation was in,
    /// however the preparation left it.
    fn drop(&mut self) {
        self.close();
    }
}

/// The list,
/// its bound and its record are controlled in the test module.
#[cfg(test)]
#[path = "commit_prepare_part_durations_tests.rs"]
mod tests;
