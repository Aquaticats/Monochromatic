//! What:
//!  Test-only counts of the durability syncs the migration's shared write steps made,
//!  and of the ones they skipped for a thread that keeps its writes in memory.
//! Why:
//!  Which of the two a step took is what a control of the switch has to show,
//! and no file on disk shows it:
//! a regular file reads back from the page cache either way.
//! The counts are the process's rather than the thread's because the steps run on threads the caller did not make
//! (a rewrite of an index is a job of the blocking pool),
//! which is exactly the answer a control of that plumbing has to read.
//! Nextest runs one test per process,
//! so a count never mixes two controls.
//! The counts are kept in a file of their own so the shipped `subvolume_file_write_steps.rs` holds no inline test
//! module (`rust/tests-in-sidecars`),
//! and the release executable never contains them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const syncCounts = { noteFileSync, noteDirectorySync, takeSyncCounts };
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

/// How many durability syncs a stretch of the run made,
/// and how many it skipped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SyncCounts {
    /// The syncs that asked the kernel to flush.
    pub made: usize,
    /// The syncs the switch skipped.
    pub skipped: usize,
}

static MADE: AtomicUsize = AtomicUsize::new(0);
static SKIPPED: AtomicUsize = AtomicUsize::new(0);

/// What:
///  Count one file flush,
/// made or skipped,
/// as the write step took it.
/// Why:
///   A control reads whether a writer asked the step at all,
/// and which branch it reached there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// skipped ? skippedCount++ : madeCount++;
/// ```
pub(crate) fn note_file_sync(skipped: bool) {
    if skipped {
        SKIPPED.fetch_add(1, Ordering::SeqCst);
        return;
    }
    MADE.fetch_add(1, Ordering::SeqCst);
}

/// Count one directory flush,
/// made or skipped,
/// as the directory step took it.
pub(crate) fn note_directory_sync(skipped: bool) {
    note_file_sync(skipped);
}

/// What:
///  Take the counts since the last call and start again from nothing.
/// Why:
///   A control calls it before the work it measures and after it,
/// so it reads exactly that work's syncs and never another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = madeCount.swap(0); const skipped = skippedCount.swap(0); return { made, skipped };
/// ```
pub(crate) fn take_sync_counts() -> SyncCounts {
    return SyncCounts { made: MADE.swap(0, Ordering::SeqCst), skipped: SKIPPED.swap(0, Ordering::SeqCst) };
}
