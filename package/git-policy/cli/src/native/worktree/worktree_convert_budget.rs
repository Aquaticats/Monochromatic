//! What:
//!  How many conversions one run runs at once:
//! the bytes one conversion holds,
//! the ceiling the filesystem's own metadata work sets,
//! and the workers a host of a measured size allows.
//! Why:
//!  The budget divides the free memory by what one conversion holds and stops at the ceiling measurements
//!  found,
//! and the numbers and the measurement behind each live together here so the command's body reads as
//!  the run it describes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const workers = conversionWorkers(lanes, cap, host);
//! ```

use crate::parallel_budget::{
    BudgetRequest,
    HostResources,
    worker_budget,
};

/// What:
///  The bytes one conversion holds at its peak,
/// rounded up for a worktree of about two hundred thousand files.
/// Why:
///   The budget divides the free memory by it.
/// A conversion holds the index entries and the tree-walk state of one worktree and
///  streams the file data through the kernel's clone call,
/// so its peak follows the number of files and not their size:
/// measured
///  on 2026-11-03 at 7484 KiB resident for a worktree of 16 thousand files (about 0.45 KiB per file),
/// which is about 90 MiB at
///  two hundred thousand files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONVERSION_MEMORY = 128 * MiB;
/// ```
pub const CONVERSION_MEMORY: u64 = 128 << 20;

/// What:
///  The most worktrees converted at once however many CPUs and bytes the host has.
/// Why:
///   The conversions are bound by the filesystem's metadata work,
/// which stops scaling past a point.
/// Measured on 2026-11-03 on a
///  btrfs volume (32 worktrees of about ten thousand files each,
/// 28 threads,
/// other sessions running,
/// medians of three or more
///  runs at 1,
/// 2,
/// 4,
/// 8,
/// 16 and 18 workers:
/// 61,
/// 35,
/// 19,
/// 14,
/// 12 and 12 seconds;
/// further runs at 12,
/// 16 and 24 workers:
/// 11 to 30,
/// 11
///  to 16 and 16 to 18 seconds):
/// more than sixteen workers were never faster.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONVERSION_STORAGE_CEILING = 16;
/// ```
pub const CONVERSION_STORAGE_CEILING: usize = 16;

/// The workers a run of `lanes` independent conversions uses on `host`,
/// with the person's `cap`.
pub fn conversion_workers(
    lanes: usize,
    cap: Option<usize>,
    host: &HostResources,
) -> usize {
    return worker_budget(&BudgetRequest {
        items: lanes,
        per_item_memory: CONVERSION_MEMORY,
        ceiling: Some(CONVERSION_STORAGE_CEILING),
        cap,
        host,
    });
}
