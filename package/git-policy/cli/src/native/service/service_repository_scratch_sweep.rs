//! What:
//!  The collection of the repository scratch directory,
//!  run beside the service's landing loop instead of on a commit's path.
//! Why:
//!  A process killed between making a scratch tree and removing it leaves the tree behind,
//!  and nothing else would ever collect it,
//!  so a command that prepares scratch must not be the one that pays for the collection:
//!  it lists the directory,
//!  asks the file system about every entry,
//!  and may remove up to `SCRATCH_TREES_REMOVED_PER_COMMAND` trees.
//!  On this volume those calls stall:
//!  the collection was 13,482 ms of the 48 wrapped commits of 2026-11-11T12:33:40Z to 13:15:27Z,
//!  4,775 ms in one,
//!  and it was the largest single cause in the records of that window
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
//!  "The causes,
//! ranked by the time they cost").
//!  So the commit makes its own tree and starts nothing else,
//!  and the service collects when it begins a landing:
//!  the work is one thread of its own,
//!  never the runtime's blocking pool,
//!  because a sweep may hold a place there for a recursive removal the landing itself would wait for.
//!  One collection runs at a time:
//!  a second landing that begins meanwhile leaves the collection to the first,
//!  which covers the same directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! start(commonDir); // returns at once
//! ```

/// One thread for this collection,
/// and the sentence a machine writes when it refuses one.
use crate::scoped_start::{refused_thread_sentence, spawn_named};
/// What a collection does,
/// and the rules it applies.
use crate::scratch_location::sweep_repository_scratch;
/// A path,
/// and the flag that keeps one collection at a time.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// The point at which a collection begins,
/// for a control that holds one repository's collection open while another repository's is asked for
/// (`service_fault_hooks_test_support.rs`).
/// It names the directory,
/// so a control of one repository never holds a collection of another.
#[cfg(test)]
pub(crate) fn collection_point(common_dir: &Path) -> String {
    return format!("a-repository-scratch-collection-of-{}", common_dir.display());
}

/// Lock kept:
///  whether this service is already collecting,
///  which it sets and clears and nothing else touches:
/// one collection at a time is all a repository scratch directory needs,
/// and a landing never waits for this flag,
/// because a landing that finds it set leaves the collection to the run already under way.
static COLLECTING: AtomicBool = AtomicBool::new(false);

/// What:
///  Starts the collection of one repository's scratch directory,
///  beside the landing loop,
///  unless one is already running in this service.
/// Why:
///   Every answer the service gives is a commit the person waits for,
///  so the collection is started beside the loop and never inside it.
///  An empty directory costs one listing,
///  and what a killed command left is removed by the age rules of
///  `scratch_location.rs`,
///  which keep a live command's tree however old it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(commonDir) // void
/// ```
pub fn start(common_dir: &Path) {
    if COLLECTING.swap(true, Ordering::AcqRel) {
        return;
    }
    let owned: PathBuf = common_dir.to_path_buf();
    let named: String = common_dir.display().to_string();
    let started = spawn_named("repository-scratch-sweep", move || {
        // A control holds this collection at its beginning, on the thread that runs it, so "while it collects" is a state
        //  and not a duration.
        #[cfg(test)]
        crate::service_fault::hooks::run_on_any_thread(collection_point(owned.as_path()).as_str());
        // The flag is cleared however the collection ends, so one refused listing does not stop every later one.
        let removed: usize = sweep_repository_scratch(owned.as_path());
        COLLECTING.store(false, Ordering::Release);
        crate::diagnostic_log::debug(
            "repositoryScratchSweep",
            format!("collected {} stale entries of the repository scratch directory of {named}", removed).as_str(),
        );
    });
    if let Err(error) = started {
        COLLECTING.store(false, Ordering::Release);
        crate::service_warn!("{}", refused_thread_sentence("collect the repository scratch directory", &error));
    }
}

/// The collection and its flag are controlled in the test module.
#[cfg(test)]
#[path = "service_repository_scratch_sweep_tests.rs"]
mod tests;
