//! What:
//!  What recovery reads to decide whose checkout this is and whether a merge began:
//!  the branch `HEAD` names,
//!  the commit a ref names,
//!  and whether the index or the worktree holds the merge.
//! Why:
//!  These are the reads a person's command pays for on every merge record it meets,
//! and they are reads only;
//!  the decision they feed lives in `merge_isolation_recovery.rs`,
//! which keeps the file under the package's line
//!  limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checkedOut = await symbolicRef({ place, root }); const current = await resolveCommit({ place, root, reference });
//! ```

use crate::merge_isolation_recovery_steps::{Judged, RecoveryPlace, Standing};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::{GitRequest, run_git};
use std::path::Path;

/// What:
///  Whether an index entry is the merge's own,
/// which is the only evidence that Git began to install it.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMergesEntry = (standing: Standing) => standing === 'new';
/// ```
pub fn holds_the_merges_entry(standing: &Standing) -> bool {
    return *standing == Standing::New;
}

/// What:
///  Whether the worktree holds the merge's content at this path,
/// which is the only evidence that Git began.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMerge = (judged: Judged) => judged.standing === 'new';
/// ```
pub fn holds_the_merge(judged: &Judged) -> bool {
    return judged.standing == Standing::New;
}

/// What:
///  The branch `HEAD` names in `root`,
/// or nothing when it names a commit itself.
/// Why:
///   The checkout a merge installs into is the one its branch is checked out in;
/// `symbolic-ref` says which branch that is,
/// and fails
///  on a detached `HEAD`,
/// which the record names with no target ref of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = await git(['symbolic-ref', '--quiet', 'HEAD']); return named.succeeded ? named.stdout.trim() : undefined;
/// ```
pub fn symbolic_ref(place: &RecoveryPlace, root: &Path) -> Result<Option<String>, RecoveryError> {
    let request: GitRequest = GitRequest::new(root, &["symbolic-ref", "--quiet", "HEAD"]);
    let output = match run_git(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git symbolic-ref could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  The commit a ref names,
/// or nothing when it names none.
/// Why:
///   The branch value decides whether the merge landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCommit(reference: string): Promise<string | undefined>;
/// ```
pub fn resolve_commit(place: &RecoveryPlace, root: &Path, reference: &str) -> Result<Option<String>, RecoveryError> {
    let revision: String = format!("{reference}^{{commit}}");
    let request: GitRequest = GitRequest::new(root, &["rev-parse", "--verify", "--quiet", revision.as_str()]);
    let output = match run_git(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git rev-parse could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}
