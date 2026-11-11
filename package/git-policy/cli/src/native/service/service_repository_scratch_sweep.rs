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
//!  One collection runs at a time for each repository scratch directory:
//!  a second landing of the same directory that begins meanwhile leaves the collection to the first,
//!  and a landing of another directory that begins meanwhile collects its own,
//!  because nothing would collect that one after the first ends
//!  (the development service serves several common Git directories at once,
//!  and two linked worktrees are two services over one scratch directory each).
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
/// and what keeps one collection at a time for each directory.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
///  the common directories whose collection this service has started and not finished,
///  which it adds to and removes from and nothing else touches:
/// one collection at a time is all a repository scratch directory needs,
/// and a landing of another directory never waits for more than one search,
/// one addition and one removal of an entry,
/// because a landing that finds its own directory here leaves the collection to the run already under way.
static COLLECTING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// What:
///  Claims the collection of `common_dir` for this caller,
///  and answers whether it is this caller's to run.
/// Why:
///   A landing that finds its own directory already being collected starts nothing,
///  and one of another directory starts its own collection:
///  the entries of one scratch directory say nothing about another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim(commonDir) // whether this caller runs the collection
/// ```
fn claim(common_dir: &Path) -> bool {
    let mut collecting = COLLECTING.lock().expect("the collections this service runs");
    if place_of(collecting.as_slice(), common_dir).is_some() {
        return false;
    }
    collecting.push(common_dir.to_path_buf());
    return true;
}

/// What:
///  Gives the collection of `common_dir` back,
///  so the next landing of that repository starts one.
/// Why:
///   A directory whose collection ended is one no thread of this service is listing,
///  and a reference that stayed would leave every later landing of that repository without a collection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// release(commonDir) // void
/// ```
fn release(common_dir: &Path) {
    let mut collecting = COLLECTING.lock().expect("the collections this service runs");
    if let Some(collected) = place_of(collecting.as_slice(), common_dir) {
        let _released = collecting.remove(collected);
    }
}

/// What:
///  The place of `common_dir` among the directories this service is collecting,
///  or nothing when it is not there.
/// Why:
///   One directory has one collection,
///  so the claim and the release ask the same question and remove the entry they found:
///  neither borrows the record for a removal nor hides the search in a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeOf(collected, commonDir) // the index of that directory, or nothing
/// ```
fn place_of(collected: &[PathBuf], common_dir: &Path) -> Option<usize> {
    let mut place: usize = 0;
    while place < collected.len() {
        if collected[place].as_path() == common_dir {
            return Some(place);
        }
        place += 1;
    }
    return None;
}

/// What:
///  Starts the collection of one repository's scratch directory,
///  beside the landing loop,
///  unless that same directory is already being collected by this service.
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
    if !claim(common_dir) {
        return;
    }
    let owned: PathBuf = common_dir.to_path_buf();
    let named: String = common_dir.display().to_string();
    let started = spawn_named("repository-scratch-sweep", move || {
        // A control holds this collection at its beginning, on the thread that runs it, so "while it collects" is a state
        //  and not a duration.
        #[cfg(test)]
        crate::service_fault::hooks::run_on_any_thread(collection_point(owned.as_path()).as_str());
        // The directory is given back however the collection ends, so one refused listing does not stop every later one.
        let removed: usize = sweep_repository_scratch(owned.as_path());
        release(owned.as_path());
        crate::diagnostic_log::debug(
            "repositoryScratchSweep",
            format!("collected {} stale entries of the repository scratch directory of {named}", removed).as_str(),
        );
    });
    if let Err(error) = started {
        release(common_dir);
        crate::service_warn!("{}", refused_thread_sentence("collect the repository scratch directory", &error));
    }
}

/// The collection,
/// the directories it holds and the point a control holds it at
/// are controlled in the test module.
#[cfg(test)]
#[path = "service_repository_scratch_sweep_tests.rs"]
mod tests;
