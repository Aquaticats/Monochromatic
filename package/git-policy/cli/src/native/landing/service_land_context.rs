//! What:
//!  What the commit service needs to land a batch:
//! how to run Git,
//! where the journal lives,
//! a test seam and the lock ceiling.
//! Why:
//!  The landing (`service_land.rs`) and the recovery of open batches (`service_land_finish.rs`) both take it;
//! a file that uses
//!  neither lets each use it without using the other in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LandContext = { git: GitContext; journal: string; onObjectsWritten?: () => void; lockCeiling: number };
//! ```

use crate::transaction_git::GitContext;
use std::path::PathBuf;
/// What the service needs to land:
/// how to run Git and where the journal lives.
pub struct LandContext {
    /// How to run the real Git,
    /// with the bounds of `child_class.rs`.
    pub git: GitContext,
    /// Directory of the journal.
    pub journal: PathBuf,
    /// Test seam:
    /// called after the batch's objects exist and before the branch moves.
    pub on_objects_written: Option<Box<dyn Fn() + Send + Sync>>,
    /// How long a landing waits for a foreign program's `index.lock` before it is withdrawn (`wait_ceilings.rs`).
    pub lock_ceiling: std::time::Duration,
}
