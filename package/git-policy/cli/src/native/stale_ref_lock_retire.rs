//! What:
//!  Judge one candidate and, when it is proven stale, set it aside, check that it is the file that was judged, and remove it
//!  with its companions.
//! Why:
//!  Two wrappers may judge the same lock at the same moment, and a live Git may create a new lock at the same path the
//!  instant the old one is gone. A plain `unlink` by path would then remove the live Git's lock. One `rename` to a private
//!  name takes the file atomically; what was taken is then checked against what was judged, and put back if it differs.
//!  Nothing here removes a path after a separate look at it: the lock, its pid file and the packed-references staging file
//!  all go through that one step, each against the identity it had when it was judged.
//!  No lock of the wrapper's own is taken (`Use as few locks as possible.`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! judgeAndClear(candidate, judge, outcome); // rename(lock, aside); same(aside, judged) ? unlink(aside) : link(aside, lock)
//! ```

use super::diagnostic_log::warn;
use super::stale_ref_lock_found::{Found, Outcome};
use super::stale_ref_lock_judge::{Examined, Identity, Judge, Verdict, identity_of};
use super::service_index_placement::rename_noreplace;
use super::stale_ref_lock_walk::{Candidate, companions_of};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// The start of the private name a lock is set aside under; it ends in no `.lock`, so Git and the walk ignore it.
const ASIDE_PREFIX: &str = "cli-git-stale-lock";

/// Makes two names set aside by one process differ.
static ASIDE_COUNT: AtomicU64 = AtomicU64::new(0);

/// A private name in `root` that no other process uses.
fn aside_name(root: &Path) -> PathBuf {
    let count: u64 = ASIDE_COUNT.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    return root.join(format!("{ASIDE_PREFIX}-{}-{count}-{nanos:x}", std::process::id()));
}

/// Take the file at `lock` by renaming it into `root`; remove it if it is the file that was judged, otherwise put it back.
///  Returns whether the judged file was removed. A file that already differs from the judged one is not moved at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeIfUnchanged(root: string, lock: string, judged: Identity, outcome: Outcome): boolean;
/// ```
pub fn remove_if_unchanged(root: &Path, lock: &Path, judged: &Identity, outcome: &mut Outcome) -> Result<bool, String> {
    // A second look just before the move: whatever was judged may have been replaced while the judgement ran, and a file
    //  that is visibly another one is not taken even for the moment the move lasts.
    if identity_of(lock).as_ref() != Some(judged) {
        return Ok(false);
    }
    return take_and_check(root, lock, judged, outcome);
}

/// The step after that look: take the file at `lock` into `root` under a name nobody has, compare what was taken with
///  `judged`, then remove it or put it back where it was found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeAndCheck(root: string, lock: string, judged: Identity, outcome: Outcome): boolean;
/// ```
pub fn take_and_check(root: &Path, lock: &Path, judged: &Identity, outcome: &mut Outcome) -> Result<bool, String> {
    let aside: PathBuf = aside_name(root);
    // `RENAME_NOREPLACE`: the private name is never written over, whatever is there.
    match rename_noreplace(lock, aside.as_path()) {
        Ok(()) => {}
        // Someone else took it first, which is the same end.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("it could not be set aside: {error}")),
    }
    if identity_of(aside.as_path()).as_ref() == Some(judged) {
        return match std::fs::remove_file(aside.as_path()) {
            Ok(()) => Ok(true),
            Err(error) => Err(format!("the file that was set aside could not be removed ({error}); it is at {}", aside.display())),
        };
    }
    // It was not the judged file: a live Git made a new lock there in between. Put it back whole.
    match std::fs::hard_link(aside.as_path(), lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(aside.as_path());
            warn(TAG, &format!("{} was taken again after it was judged; the new lock was put back", lock.display()));
        }
        Err(error) => {
            warn(TAG, &format!("{} was taken again after it was judged and could not be put back ({error}); it is left at {}", lock.display(), aside.display()));
            outcome.displaced.push(aside);
        }
    }
    return Ok(false);
}

/// Judge `candidate` and record the result in `outcome`; remove it when it is proven stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeAndClear(candidate: Candidate, judge: Judge, outcome: Outcome): void;
/// ```
pub fn judge_and_clear(candidate: &Candidate, judge: &Judge, outcome: &mut Outcome) {
    // The files that go with the lock are looked at before the judgement, so that each is removed only if it is still the
    //  file that was there while the lock was judged.
    let companions: Vec<(PathBuf, Identity)> = if candidate.orphan_staging {
        Vec::new()
    } else {
        companions_of(candidate.lock.as_path()).into_iter().filter_map(|path| { return identity_of(path.as_path()).map(|identity| { return (path, identity); }); }).collect()
    };
    let maybe: Option<Examined> = if candidate.orphan_staging { judge.examine_staging(candidate.lock.as_path()) } else { judge.examine(candidate.lock.as_path()) };
    let Some(Examined { verdict, pid_file, identity, born }) = maybe else {
        return;
    };
    let mut found: Found = Found {
        lock: candidate.lock.clone(),
        verdict: verdict.clone(),
        size: identity.size,
        born,
        pid_file,
        companions: companions.iter().map(|(path, _)| { return path.clone(); }).collect(),
    };
    match verdict {
        Verdict::TooNew => {}
        Verdict::Held(_) | Verdict::Unproven(_) => outcome.left.push(found),
        // The staging file exists only while its lock does, so the lock is looked for again, after the file was judged: a
        //  lock that is back means a Git holds both, and the file is left.
        Verdict::Stale(_) if candidate.orphan_staging && candidate.lock.with_file_name("packed-refs.lock").symlink_metadata().is_ok() => {}
        Verdict::Stale(_) => match remove_if_unchanged(candidate.root.as_path(), candidate.lock.as_path(), &identity, outcome) {
            Ok(true) => {
                for (companion, judged) in &companions {
                    if let Err(reason) = remove_if_unchanged(candidate.root.as_path(), companion.as_path(), judged, outcome) {
                        warn(TAG, &format!("{} went with a lock that was removed, and could not be removed: {reason}", companion.display()));
                    }
                }
                outcome.cleared.push(found);
            }
            Ok(false) => {}
            Err(reason) => {
                found.verdict = Verdict::Unproven(reason);
                outcome.left.push(found);
            }
        },
    }
}
