//! What:
//!  The three bounds one pass is held to:
//!  how many kept references it may remove,
//!  how long a record whose reference is gone is kept,
//!  and how long one pass may spend planning and removing.
//! Why:
//!  A pass runs after every commit a repository lands,
//!  inside the service's after-commit upkeep,
//!  so a cap and a time keep one pass from holding that upkeep for long,
//!  and the record's grace keeps a record another command wrote a moment ago
//!  from being swept by a pass that listed the repository before that write.
//!  Each value carries the measurement it rests on,
//!  so a person changing one knows what the count was measured on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const THE_MOST_A_PASS_PRUNES = 16;
//! const A_RECORD_OUTLIVES_ITS_REFERENCE_FOR = 60 * 60 * 1000;
//! const A_PASS_WITHIN = 20 * 1000;
//! ```

/// The length of the record's grace and the pass's own time.
use std::time::Duration;
/// How many kept references one pass may remove.
///  Sixteen:
///  a pass runs after every commit a repository lands,
/// so a burst larger than this is worked through
///  by the passes that follow it,
/// and no single pass holds the service's upkeep for long.
///  Measured:
/// this repository held five kept references in the 5.7 days before 2026-11-12,
///  so no pass has had to remove any yet,
///  and the cap costs nothing until a repository's kept references actually accumulate.
pub const THE_MOST_A_PASS_PRUNES: usize = 16;

/// How long a record whose reference is gone is kept before a pass removes it.
///  One hour:
///  a record names the worktree the commit was built in,
///  and a reference another command wrote after a pass listed the repository has one written a moment ago,
///  so the file is kept long past the moment anything could still be arriving for it.
///  The size it costs is nothing next to the reference it describes.
pub const A_RECORD_OUTLIVES_ITS_REFERENCE_FOR: Duration = Duration::from_secs(60 * 60);

/// How long one pass may spend planning and removing.
///  Twenty seconds:
///  the service's after-commit work starts housekeeping for thirty seconds a round
///  (`HOUSEKEEPING_OF_ONE_ROUND`),
/// and a pass that ends inside that slice leaves the round's own
///  bound intact.
///  Measured:
/// a pass over this repository's five kept references is four Git starts
///  (a `for-each-ref`,
/// a `rev-list`,
/// a `cat-file` and one `update-ref` for each removal),
///  each of them milliseconds on the host of 2026-11-12.
pub const A_PASS_WITHIN: Duration = Duration::from_secs(20);
