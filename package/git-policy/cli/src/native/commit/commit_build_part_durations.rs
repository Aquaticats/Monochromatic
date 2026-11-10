//! What:
//!  How long each stretch of a commit's build phase held no Git child,
//!  by name,
//!  kept until the commit's timing record takes the figures.
//! Why:
//!   The build phase is the shadow repository's creation,
//!  the commit Git builds in it,
//!  and the stretch that reads the request out of what the preparation handed back,
//!  and the records put 51.7 percent of a commit's whole time in it.
//!  The record names the Git children of that phase in `steps`,
//!  and it named nothing of the creation,
//!  so a build of 63 444 ms whose longest step was 65 ms explained itself with no figure at all.
//!  `doc/planning/cli-git-commit-latency-under-many-agents.md` holds that reading
//!  in its section on what the build phase waits on.
//!  Each stretch is measured from one clock read at each seam to the next,
//!  and the Git children that ran inside it are taken out,
//!  because those are recorded apart and a stretch that holds one is not waiting.
//!  A name is one of a fixed set compiled into the wrapper,
//!  never a path,
//!  a message or an option value.
//!  This is the build's twin of `commit_prepare_part_durations`,
//!  and the two kept apart on purpose:
//!  a preparation's name and a build's name are different work,
//!  and a reading that added them would answer neither question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parts: { name: string; ms: number }[] = [];
//! const clock = beginBuildPartClock('branch');
//! // ... work ...
//! clock.next('message');
//! export const drainBuildParts = () => parts.splice(0);
//! ```

/// The object a record writes for the stretches,
/// which is the preparation's writer.
use crate::commit_prepare_part_durations::{PreparePart, prepare_parts_json};
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
/// one build marks about a dozen seams,
/// and a commit builds at most `LANDING_ATTEMPTS` (3) times,
/// 36 stretches,
/// so 64 is the next power of two above it,
/// about 40 bytes an entry and under 3 KiB of a one mebibyte timing file.
/// The record states how many went unlisted,
/// so a longer run is counted,
/// not lost silently.
pub const MAX_BUILD_PARTS: usize = 64;

/// Lock kept:
///  the stretches of this process since a record last took them,
///  in the order they ended.
///  Each seam of a build pushes under it and the timing record takes under it,
///  nothing else runs under it:
///  no Git process,
///  no file and no other lock,
///  and the same pattern `commit/commit_prepare_part_durations.rs` keeps.
static PARTS: Mutex<Vec<PreparePart>> = Mutex::new(Vec::new());

/// Lock kept:
///  how many stretches came after the list was full,
///  counted under the same rule as `PARTS`:
///  one number the seam adds to and the record takes,
///  with no Git process,
///  no file and no other lock under it.
static DROPPED: Mutex<usize> = Mutex::new(0);

/// What:
///  Keeps one stretch of a build.
/// Why:
///   Every seam of every build reports here,
///  whether the build went on to land or stopped where it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteBuildPart('shadow links', 12);
/// ```
pub fn note_build_part(name: &str, took: Duration) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of stretches.
    let mut parts = PARTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if parts.len() >= MAX_BUILD_PARTS {
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
/// const { parts, dropped } = drainBuildParts();
/// ```
pub fn drain_build_parts() -> (Vec<PreparePart>, usize) {
    let mut parts = PARTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let taken: Vec<PreparePart> = std::mem::take(&mut *parts);
    let mut dropped = DROPPED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let lost: usize = std::mem::take(&mut *dropped);
    return (taken, lost);
}

/// What:
///  The stretches as one object of the record.
/// Why:
///   Nothing in the figure depends on the field it is written in,
///  so the build's stretches are written by the preparation's writer
///  and a reader finds the same shape under either name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// buildPartsJson(parts) // '{"shadow links":{"ms":12,"calls":1}}'
/// ```
pub fn build_parts_json(parts: &[PreparePart]) -> String {
    return prepare_parts_json(parts);
}

/// What:
///  The stretch of a build that is running,
///  marked at each seam,
///  and kept when it ends.
/// Why:
///   The marks are the seams of the build,
/// one line each,
/// so a stretch is
///  named by the block it covers rather than by a timer every call leaves.
///  A build that leaves early,
///  on a stop or a failure,
///  keeps the stretch it was in when it left,
///  because the value keeps it as it goes out of scope.
///  One clock runs through the whole phase and is handed to the work that owns the next stretch,
///  so a stretch that holds another's work is impossible by construction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using clock = beginBuildPartClock('branch'); clock.next('message');
/// ```
#[derive(Debug)]
pub struct BuildPartClock {
    /// Where the running stretch began.
    began: Instant,
    /// What the build is doing in the running stretch.
    name: &'static str,
}

impl BuildPartClock {
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
        note_build_part(self.name, whole.saturating_sub(children));
        self.began = ended;
    }
}

impl Drop for BuildPartClock {
    /// Ends the stretch the build was in,
    /// however the build left it.
    fn drop(&mut self) {
        self.close();
    }
}

/// The list,
/// its bound and its record are controlled in the test module.
#[cfg(test)]
#[path = "commit_build_part_durations_tests.rs"]
mod tests;
