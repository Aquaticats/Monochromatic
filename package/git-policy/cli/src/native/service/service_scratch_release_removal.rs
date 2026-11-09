//! What:
//!  The removal of a commit's scratch directory once the landing that read it is over.
//! Why:
//!  A request names the private index inside its tree,
//!  and the landing reads the tree's contents,
//!  so only that landing says when the tree is no one's
//!  (`service_server.rs`,
//! `remove_the_trees`).
//!  Tying the removal to a connection instead took it from under a client killed while its request was still queued.
//!  The removal is a recursive one,
//!  and the record holds an `unlinkat` of a shaped tree at 2.5 s
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`),
//!  so waiting for it inside the landing loop would make every commit queued behind it wait for it too.
//!  It runs beside the loop,
//!  bounded by `SCRATCH_REMOVAL_CEILING`,
//!  and a removal that cannot finish says so in the journal rather than nowhere.
//!  Whatever is left of such a directory is inside the repository scratch directory,
//!  where the sweep of a later command removes it.
//!  A tree of a request the landing answered as not landed is not removed here:
//!  that request's command still reads the private index of the attempt that captured its change,
//!  so it is the command's to remove
//!  (`service_commit_tree_ownership.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const start = (path: string) => { void (async () => { ... })(); };
//! ```

/// The bounded removal and what it came to.
use crate::scratch_entry_removal::{Removal, SCRATCH_REMOVAL_CEILING, remove_within};
/// The service's own runtime,
/// which runs the removal beside the landing loop.
use crate::lazy_runtime::spawn_on_runtime;
/// One step of blocking work,
/// off the runtime's threads.
use crate::blocking_job::run_blocking_job;
/// A path and the span the removal is reported in.
use std::path::PathBuf;
use std::time::Duration;

/// What:
///  The commit's scratch tree that a request names,
///  when the name it gives is one of the wrapper's own trees of the worktree.
/// Why:
///   A request carries the tree it prepared in its own field,
///  and deriving the tree from another path it carries names a directory that is not one,
///  so the removal of a landed commit's tree never started.
///  The service must still never remove a directory the wrapper did not make,
///  so the name is checked here even though the decoder checks it too:
///  a request built in this process reaches this function without that decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = theTreeOf(request.scratch, request.gitDir); // undefined when it is not the wrapper's
/// ```
pub fn the_tree_of(named: &std::path::Path, git_dir: &std::path::Path) -> Option<PathBuf> {
    if !crate::service_protocol_messages_and_name_checks::is_scratch_path(named, git_dir) {
        return None;
    }
    return Some(named.to_path_buf());
}

/// What:
///  Starts the removal of one released scratch directory,
///  beside the service's landing loop.
/// Why:
///   The loop must be free to land the next batch while a recursive removal waits on the device,
///  and nothing after the release needs the directory.
///  A process whose runtime cannot take the task leaves the directory to the sweep,
///  and says so:
///  a directory left behind silently is one nobody looks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(path) // returns at once
/// ```
pub fn start(path: PathBuf) {
    let named: String = path.display().to_string();
    let started: Option<tokio::task::JoinHandle<()>> = spawn_on_runtime(remove(path));
    if started.is_none() {
        crate::service_warn!("the released scratch directory {named} is left to the sweep: this service has no multi-threaded runtime to remove it on");
    }
}

/// One released scratch directory removed,
/// bounded,
/// with a line when it does not finish.
async fn remove(path: PathBuf) {
    let named: String = path.display().to_string();
    let outcome: Result<Removal, crate::blocking_job::BlockingJobError> =
        run_blocking_job(move || { return remove_within(path.as_path(), SCRATCH_REMOVAL_CEILING); }).await;
    let ceiling: Duration = SCRATCH_REMOVAL_CEILING;
    match outcome {
        Ok(Removal::Finished) => {}
        Ok(Removal::Unfinished) => {
            crate::service_warn!(
                "the released scratch directory {named} was still being removed after {ceiling:?}; the sweep of the repository scratch directory removes what is left"
            );
        }
        Ok(Removal::Failed(error)) => {
            crate::service_warn!("the released scratch directory {named} could not be removed: {error}");
        }
        Err(error) => {
            crate::service_warn!("the removal of the released scratch directory {named} could not be started: {error}");
        }
    }
}

/// The removal and its line are controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_release_removal_tests.rs"]
mod tests;
