//! What:
//! The step that takes a finished copy a dead install left under a hidden name of its own,
//! so that only one install publishes from it.
//! Why:
//! The recovery of a killed install is the one place a copy is moved before anything is published from it,
//! and the claim is what keeps two installs from publishing the same directory twice.
//! It sits in a file of its own because the algorithm next to it is longer than a person reads at once,
//! and this step is the whole reason that algorithm has an order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-claim-of-a-finished-copy.ts
//! export function claimTheFinishedCopy(versionDir: string, candidates: Leftover[], firstBin: string): ClaimedCopy | undefined;
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `io` from `std` for the error a rename answers with when another install claimed the copy first.
use std::io;
/// Imports what this file uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `ToolchainError` from the toolchain error module.
use super::super::super::error::ToolchainError;
/// Imports the hidden names,
/// the purpose a claim carries,
/// and the question that tells a finished install from an unfinished one.
use super::super::{
    RECOVERED_PURPOSE,
    hidden_sibling,
    is_finished,
};
/// Imports the owner facts a name is written with,
/// the hidden sibling this file reads,
/// what a hidden sibling may hold,
/// the record a lock's row makes,
/// and the question whether a marker records that row.
use super::{
    CopyRecord,
    Leftover,
    Owner,
    marker_vouches,
    may_be_published,
};

/// What:
/// The finished copy this run claimed:
/// the name it holds it at now,
/// and the hidden name an earlier install left it at,
/// which the report names.
/// Why:
/// The claim's own name is gone once the copy is published,
/// and the path a person could have seen the copy at is the one an earlier install wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimedCopy = { claim: string; leftAt: string };
/// ```
pub(super) struct ClaimedCopy {
    /// The name this run holds the copy at until it publishes it.
    pub(super) claim: PathBuf,
    /// The hidden name an earlier install left the copy at.
    pub(super) left_at: PathBuf,
}

/// What:
/// Takes a finished copy a dead install left under a name of this process's own,
/// so no other install publishes from the same directory.
/// Why:
/// The rename is the claim,
/// and it is atomic:
/// two installs that recover the same copy cannot both win,
/// and the one that loses finds the name gone and publishes nothing.
/// A copy this process already claimed is taken as it is,
/// because a second install of the same tool in one process is the same install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimTheFinishedCopy(versionDir: string, candidates: Leftover[], firstBin: string): ClaimedCopy | undefined
/// ```
pub(super) fn claim_the_finished_copy(
    version_dir: &Path,
    of_this_version: &[&Leftover],
    first_bin: &str,
    expected: Option<&CopyRecord>,
) -> Result<Option<ClaimedCopy>, ToolchainError> {
    let claim: PathBuf = hidden_sibling(version_dir, RECOVERED_PURPOSE, Owner::here());
    if fs::symlink_metadata(&claim).is_ok() {
        return Ok(Some(ClaimedCopy {
            left_at: claim.clone(),
            claim,
        }));
    }
    let candidate = of_this_version.iter().copied().find(|leftover| {
        return may_be_published(leftover.holds)
            && is_finished(&leftover.path, first_bin)
            && expected.is_some_and(|record| return marker_vouches(&leftover.path, record));
    });
    let Some(finished) = candidate else {
        return Ok(None);
    };
    return match fs::rename(&finished.path, &claim) {
        Ok(()) => Ok(Some(ClaimedCopy {
            left_at: finished.path.clone(),
            claim,
        })),
        // Another install claimed it first, so its copy is the one that will be published.
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ToolchainError::failure(format!(
            "cannot claim the finished install at {} that a killed install left: {error}; that copy is still there and nothing was published from it",
            finished.path.display()
        ))),
    };
}
