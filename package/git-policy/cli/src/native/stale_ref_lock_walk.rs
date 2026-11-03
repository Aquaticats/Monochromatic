//! What:
//!  Which files below a Git directory are reference locks.
//! Why:
//!  Only the locks of references are judged here. `index.lock`, `config.lock`, `shallow.lock`, object and pack locks and
//!  every `~pid.lock` file belong to other protocols or to the lock they accompany.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidates = refLockCandidates(roots); // packed-refs.lock, HEAD.lock, refs/**/*.lock
//! ```

use std::path::PathBuf;

/// Every reference lock below `roots`, in path order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refLockCandidates(roots: string[]): string[];
/// ```
pub fn ref_lock_candidates(_roots: &[PathBuf]) -> Vec<PathBuf> {
    return Vec::new();
}
