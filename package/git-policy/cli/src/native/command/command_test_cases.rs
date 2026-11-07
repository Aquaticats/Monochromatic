//! What:
//!  Disposable copies of a pristine fixture and scoped worker threads for independent test cases.
//! Why:
//!  Putting shared state back after each case costs extra Git starts and forces the cases to run one after another;
//!  a fresh copy of the fixture needs only file operations,
//! and cases that share nothing run side by side.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await Promise.all(cases.map(async (item) => run(await cp(pristine, uniqueDirectory()), item)));
//! ```

/// Paths of the fixture trees.
use std::path::Path;
/// Hand out the next unclaimed case to whichever worker asks.
use std::sync::atomic::{AtomicUsize, Ordering};

/// The most worker threads one test starts;
///  the cases are short,
/// so a handful keeps the host from being flooded.
const WORKER_CAP: usize = 8;

/// What:
///  Copy a directory tree with plain file operations,
///  without starting a process.
/// Why:
///   A copy of a small pristine repository costs less than the Git start that would put its refs back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cp(from, to, { recursive: true });
/// ```
pub(crate) fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("copy target directory");
    for found in std::fs::read_dir(from).expect("copy source directory") {
        let entry: std::fs::DirEntry = found.expect("copy source entry");
        let target = to.join(entry.file_name());
        let kind: std::fs::FileType = entry.file_type().expect("copy source entry type");
        assert!(!kind.is_symlink(), "a fixture holds no links: {}", entry.path().display());
        if kind.is_dir() {
            copy_tree(entry.path().as_path(), target.as_path());
        } else {
            std::fs::copy(entry.path(), target).expect("copy source file");
        }
    }
}

/// What:
///  Run `work` for every item on scoped worker threads.
///  Each worker owns one result that `fresh` creates from the worker's number,
///  and the results of all workers come back for the caller to merge.
/// Why:
///   The cases are independent,
/// so they need not wait for each other;
///  a panic in a worker resumes on the calling thread with its own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const results = await Promise.all(workers.map((n) => drain(queue, fresh(n), work)));
/// ```
pub(crate) fn across_workers<T: Sync, R: Send>(
    items: &[T],
    fresh: impl Fn(usize) -> R + Sync,
    work: impl Fn(&T, &mut R) + Sync,
) -> Vec<R> {
    let available: usize = std::thread::available_parallelism().map_or(1, usize::from);
    let workers: usize = available.min(items.len()).min(WORKER_CAP).max(1);
    let next: AtomicUsize = AtomicUsize::new(0);
    return std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|number| {
                let (make, run, claimed) = (&fresh, &work, &next);
                return scope.spawn(move || {
                    let mut result: R = make(number);
                    while let Some(item) = items.get(claimed.fetch_add(1, Ordering::SeqCst)) {
                        run(item, &mut result);
                    }
                    return result;
                });
            })
            .collect();
        return handles
            .into_iter()
            .map(|handle| {
                return match handle.join() {
                    Ok(result) => result,
                    Err(payload) => std::panic::resume_unwind(payload),
                };
            })
            .collect();
    });
}

/// Copies and workers are controlled in the test module.
#[cfg(test)]
#[path = "command_test_cases_tests.rs"]
mod tests;
