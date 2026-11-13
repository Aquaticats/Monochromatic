//! What:
//!  Which reference names count as a kept commit,
//!  and the batch and the form each carries.
//! Why:
//!  The pruning removes references and nothing else,
//!  and `refs/cli-git/` holds more than the kept commits:
//!  `refs/cli-git/carried/<n>` holds the carried work of an isolated merge,
//!  and `refs/cli-git/auto-push/fetch-<pid>` holds a private fetch of a push.
//!  A name is a kept reference only when it is
//!  `refs/cli-git/kept/<a number>` or `refs/cli-git/kept/<a number>-rescue`,
//!  the two forms `kept_ref` and `rescue_ref` write
//!  (`service_scratch_ref_of_a_dropped_batch.rs`),
//!  and every other reference is left where it is.
//!  The rescue form is the name beside a batch's own
//!  (`worktree_removal_held_commits.rs`),
//!  so the two forms are one family and the pruning answers for both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! batchAndRescuedOf('refs/cli-git/kept/1794071191963902-rescue') // { batch: 1794071191963902n, rescued: true }
//! ```

/// The folder every kept reference of the wrapper sits under,
/// and the only one the pruning removes a reference from.
pub const KEPT_REFERENCE_PREFIX: &str = "refs/cli-git/kept/";

/// What the name beside a batch's own kept reference ends with.
pub const RESCUE_SUFFIX: &str = "-rescue";

/// What:
///  The batch a kept reference names,
///  and whether it is the name beside the batch's own.
/// Why:
///   The batch is what the person is told and what names the record,
///  and the two forms are told apart because a batch can have both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const batchAndRescuedOf = (name) => isKept(name) ? { batch: BigInt(last), rescued: last.endsWith('-rescue') } : undefined;
/// ```
pub fn batch_and_rescued_of(name: &str) -> Option<(u64, bool)> {
    let last: &str = name.strip_prefix(KEPT_REFERENCE_PREFIX)?;
    if last.is_empty() || last.contains('/') {
        return None;
    }
    // `strip_suffix` leaves the batch's own digits and the flag of which form this is.
    let (digits, rescued): (&str, bool) = match last.strip_suffix(RESCUE_SUFFIX) {
        Some(own) => (own, true),
        None => (last, false),
    };
    if !digits.bytes().all(|byte| { return byte.is_ascii_digit(); }) {
        return None;
    }
    let batch: u64 = digits.parse::<u64>().ok()?;
    return Some((batch, rescued));
}

/// What:
///  Whether `name` is a kept reference the pruning answers for.
/// Why:
///   The reading,
///  the record and the pass all ask it,
///  so one rule decides what the pruning may touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAKeptReference('refs/cli-git/carried/7') // false
/// ```
pub fn is_a_kept_reference(name: &str) -> bool {
    return batch_and_rescued_of(name).is_some();
}

/// Controls for the names stay out of the release executable.
#[cfg(test)]
#[path = "kept_reference_name_tests.rs"]
mod tests;
