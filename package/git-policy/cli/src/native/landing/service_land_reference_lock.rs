//! What:
//!  What a landing does when its branch write is refused while the branch is still where the batch found it.
//! Why:
//!  Git refuses a reference update with "cannot lock ref ...: Unable to create ... File exists" when a lock file
//!  stands in the way,
//!  and the same command answers the same failure when the branch moved.
//!  The wrapper told both apart by asking the branch again:
//!  a branch that still names the commit the batch was built on did not move,
//!  so a lock is what refused the write.
//!  A landing that treated that as a moved branch retried it,
//!  and the person's next commit ended with "the branch changed 3 times while this commit was being prepared",
//!  which was false:
//!  nothing in the repository had changed
//!  (2026-11-09,
//!  a `git update-ref --stdin` of the service killed while it held `refs/heads/main.lock`).
//!  A live holder lets go within its step,
//!  so the wait is bounded by the age a lock is judged at (`stale_ref_lock_judge::MARGIN`),
//!  and the locks nothing proves gone are named,
//!  with the command that clears them,
//!  by the judgement every fetch and push already uses (`stale_ref_locks.rs`).
//!  Nothing is removed on a claim:
//!  the proof, the notice and the set-aside removal are that module's,
//!  and a lock a live process may hold is never removed
//!  (`SPEC.md`,
//!  "Reference locks left by a Git that was killed").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = await waitOutTheReferenceLocks(roots, deadline); // { gone: true } | { blocked: words }
//! ```

/// The reference locks of the repository, judged and cleared by the one module that owns the proofs.
use crate::stale_ref_locks::{Judge, clear_stale_ref_locks};/// The files that are candidates at all, so a wait knows whether anything stands.
use crate::stale_ref_lock_walk::ref_lock_candidates;
/// What a pass over the locks found, and how one of them was judged.
use crate::stale_ref_lock_found::Outcome;
use crate::stale_ref_lock_judge::Verdict;
/// The shell quoting the notice gives the command that clears a lock.
use crate::stale_ref_lock_message::shell_quote;
/// How often the wait looks again while a lock stands.
use std::time::Duration;
use std::path::PathBuf;
use std::time::Instant;

/// How often the wait looks at the locks again.
/// Why:
///   A live holder lets go within its step (a reference step of the service's journal took 3,438 ms at most),
/// so a wait that looks this often finds a released lock at once,
/// and the price is one walk of the Git directory and one judgement per look:
/// 50 looks a second, each reading a few directory entries and the process table.
pub(crate) const REFERENCE_LOCK_POLL: Duration = Duration::from_millis(20);

/// What waiting out the reference locks came to.
pub(crate) enum Waiting {
    /// Nothing stands in the way any more:
    /// the write may be tried again.
    Gone,
    /// Locks stand that nothing proves gone,
    /// and `words` names them with the command that clears each.
    Blocked(String),
}

/// What:
///  Waits until the reference locks of the repository are gone or provably stale,
///  and names the ones that are neither.
/// Why:
///   The first judgement clears what a proof reaches (the staging file of the packed references),
///  and a lock a live process holds is waited for,
///  because it lets go on its own within its step.
///  A lock younger than the margin is not judged yet,
///  so a wait that finds only those looks again rather than calling them unproven.
///  At `deadline` the judgement is made once more and whatever is left is named:
///  nothing here removes a lock it cannot prove,
///  and nothing waits past the deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (true) {
///   const standing = candidates(roots);
///   if (standing.length === 0) return { gone: true };
///   const outcome = clearStaleRefLocks(roots, judge);
///   if (outcome.cleared.length > 0) return { gone: true };
///   if (outcome.left.length > 0 || outcome.displaced.length > 0) return { blocked: personWords(outcome) };
///   if (now() >= deadline) return { blocked: personWords(clearStaleRefLocks(roots, judge)) };
///   sleep(POLL);
/// }
/// ```
pub(crate) fn wait_out_the_reference_locks(roots: &[PathBuf], judge: &Judge, deadline: Instant) -> Waiting {
    loop {
        let standing: Vec<PathBuf> = ref_lock_candidates(roots).into_iter().map(|candidate| { return candidate.lock; }).collect();
        if standing.is_empty() {
            return Waiting::Gone;
        }
        let outcome: Outcome = clear_stale_ref_locks(roots, judge);
        if !outcome.cleared.is_empty() {
            return Waiting::Gone;
        }
        if !outcome.left.is_empty() || !outcome.displaced.is_empty() {
            return Waiting::Blocked(words_of(&outcome, standing.as_slice()));
        }
        // Every lock standing is younger than the margin, so a live holder may still be at work: look again.
        if Instant::now() >= deadline {
            let last: Outcome = clear_stale_ref_locks(roots, judge);
            if !last.cleared.is_empty() {
                return Waiting::Gone;
            }
            let left: Vec<PathBuf> = ref_lock_candidates(roots).into_iter().map(|candidate| { return candidate.lock; }).collect();
            return Waiting::Blocked(words_of(&last, left.as_slice()));
        }
        std::thread::sleep(REFERENCE_LOCK_POLL);
    }
}

/// What:
///  The words for the locks that stand,
///  for the person reading the commit's failure.
/// Why:
///   A lock that is not proven stale is never removed,
/// so the person is told what was found and given a command that is safe to paste.
///  The judgement words the same facts for the journal;
///  these are the sentence a command ends with,
///  so they name every lock that stands,
///  what is known about its holder,
///  and the one command that clears them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = `A program is using ${locksNamed(held)}. Nothing proves that a Git process still holds ${locksNamed(unproven)}. If no Git process is running in this repository, remove them with \`rm -- ${quotedList(unproven)}\`.`;
/// ```
fn words_of(outcome: &Outcome, standing: &[PathBuf]) -> String {
    let mut held: Vec<(PathBuf, String)> = Vec::new();
    let mut unproven: Vec<PathBuf> = Vec::new();
    for found in &outcome.left {
        match &found.verdict {
            // The process that has the lock open is the holder the person needs, and its words name it.
            Verdict::Held(who) => held.push((found.lock.clone(), who.clone())),
            Verdict::Unproven(_) => unproven.push(found.lock.clone()),
            Verdict::TooNew | Verdict::Stale(_) => {}
        }
    }
    if held.is_empty() && unproven.is_empty() {
        // No judgement was old enough to word anything, which a lock made during the wait leaves: name what stands.
        unproven.extend_from_slice(standing);
    }
    let mut sentences: Vec<String> = Vec::new();
    for (lock, who) in &held {
        sentences.push(format!("A program is using {}: {who}.", locks_named(std::slice::from_ref(lock))));
    }
    if !unproven.is_empty() {
        sentences.push(format!("Nothing proves that a Git process still holds {}.", locks_named(unproven.as_slice())));
        sentences.push(format!("If no Git process is running in this repository, remove {} with `rm -- {}`.", if unproven.len() == 1 { "it" } else { "them" }, quoted_list(unproven.as_slice())));
    }
    return sentences.join(" ");
}

/// The locks as the person reads them:
/// one in full,
/// several as a list.
///
/// In TS you would write (pseudocode):
/// ```ts
/// const locksNamed = (locks) => locks.length === 1 ? `the lock \`${locks[0]}\`` : `these locks: ${locks.map((lock) => `\`${lock}\``).join(', ')}`;
/// ```
fn locks_named(locks: &[PathBuf]) -> String {
    let mut names: Vec<String> = Vec::new();
    for lock in locks {
        names.push(format!("`{}`", lock.display()));
    }
    return match names.as_slice() {
        [only] => format!("the lock {only}"),
        many => format!("these locks: {}", many.join(", ")),
    };
}

/// The locks as the words of one `rm --`,
/// each quoted so a shell reads exactly that path.
///
/// In TS you would write (pseudocode):
/// ```ts
/// const quotedList = (locks) => locks.map((lock) => shellQuote(lock)).join(' ');
/// ```
fn quoted_list(locks: &[PathBuf]) -> String {
    let mut words: Vec<String> = Vec::new();
    for lock in locks {
        words.push(shell_quote(lock.as_path()));
    }
    return words.join(" ");
}

/// The wait is controlled in the test module.
#[cfg(test)]
#[path = "service_land_reference_lock_tests.rs"]
mod tests;
