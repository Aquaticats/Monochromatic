//! What:
//!  The repository link check:
//! phase `after` checks the link to the 860,
//! phase `before` checks the real folder on the NVMe,
//! and a deadline gives up on a drive that does not answer.
//! Why:
//!  The answers and texts equal the TypeScript check's
//! (`package/repository-link/check/src/check-link.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await checkRepositoryLink({ read, probe, deadlineSeconds });
//! ```

use crate::repository_link_answer::LinkAnswer;
use crate::repository_link_identity::LinkIdentity;
use crate::repository_link_probe::LinkProbe;
use std::sync::Arc;
use std::time::Duration;

/// How long the check waits for the filesystem.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(5);

/// What:
///  Checks the repository link without a deadline.
/// Why:
///  The table of fixture states runs this directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await answerWithoutDeadline({ read, probe })
/// ```
pub fn check_repository_link(identity: &LinkIdentity, probe: &dyn LinkProbe) -> LinkAnswer {
    todo!("{identity:?} {}", std::mem::size_of_val(probe))
}

/// What:
///  The answer when no identity file was read.
/// Why:
///  Nothing can be checked without the recorded identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusal({ kind: 'identity-missing', message })
/// ```
pub fn answer_when_identity_is_missing() -> LinkAnswer {
    todo!()
}

/// What:
///  Checks the repository link and gives up after a deadline.
/// Why:
///  A stalled `stat` on a hung drive must not hold a Git command forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await checkRepositoryLink({ read, probe, deadlineSeconds })
/// ```
pub fn check_repository_link_within(identity: &LinkIdentity, probe: Arc<dyn LinkProbe + Send + Sync>, deadline: Duration) -> LinkAnswer {
    todo!("{identity:?} {} {deadline:?}", Arc::strong_count(&probe))
}

/// Check controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_check_walk_tests.rs"]
mod tests;
