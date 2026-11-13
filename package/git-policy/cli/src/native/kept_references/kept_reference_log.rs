//! What:
//!  The line one pruned kept reference leaves in the repository's log,
//!  where that log is,
//!  and how it is written.
//! Why:
//!  Pruning a kept reference whose commit no branch reaches destroys the only handle on that commit:
//!  once the reference is gone the objects are unreachable,
//!  and `git gc` takes them.
//!  The person who built the commit has one way to find out where it went,
//!  so the wrapper writes one line before the reference goes,
//!  carrying the commit,
//!  the reference,
//!  its age,
//!  the worktree and batch it came from,
//!  and why it was pruned.
//!  The log is `<common Git directory>/monochromatic-git-policy-cli-kept-references.jsonl`,
//!  beside the wrapper's other logs,
//!  and it is written through the appender every log of the wrapper uses
//!  (`log_lines_appended_without_a_lock.rs`),
//!  so two wrappers pruning at once neither lose a line nor take a rotation from each other.
//!  Its generations are bounded by the bound every log shares
//!  (`LOG_HISTORY_BYTES`),
//!  which is more history than any repository's kept references have needed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! appendLine(join(commonDir, 'monochromatic-git-policy-cli-kept-references.jsonl'), JSON.stringify({ timeMs: Date.now(), reference, commit, ageSeconds, worktree, batch, why }) + '\n', UNPINNED_LOG);
//! ```

use crate::iso_time::format_iso_milliseconds;
use crate::kept_reference_pass::{Pruned, WhyPruned};
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// The log of the kept references a pass pruned,
/// inside the common Git directory,
/// beside the wrapper's other logs.
/// This release is the first to keep one,
/// so it has no earlier spelling (`wrapper_names_old_and_new.rs`).
pub const PRUNED_KEPT_REFERENCE_LOG: &str = "monochromatic-git-policy-cli-kept-references.jsonl";

/// What:
///  Where the line about a pruned kept reference is.
/// Why:
///   A person who looks for a commit that vanished needs the path the listing does not have to
///  guess,
///  and the log lives with the repository's other logs so the commands that read them find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const logPath = (commonDir) => join(commonDir, PRUNED_KEPT_REFERENCE_LOG);
/// ```
pub fn log_path(common_dir: &Path) -> PathBuf {
    return common_dir.join(PRUNED_KEPT_REFERENCE_LOG);
}

/// What:
///  Why a pruned reference went,
///  in the words the log and the listing both use.
/// Why:
///   A line is read by a person looking for a commit they lost,
///  and "reached-a-branch" alone does not say whether anything was lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whyWords = (why) => ({ 'reached-a-branch': 'a branch reaches its commit, so the reference held nothing', 'too-old': ..., 'too-much-kept': ... })[why];
/// ```
pub fn why_words(why: WhyPruned, limits: crate::kept_reference_limits::KeptReferenceLimits) -> String {
    let days: u64 = u64::try_from(limits.max_age.as_secs() / (24 * 60 * 60)).unwrap_or(0);
    return match why {
        WhyPruned::ReachedABranch => String::from("a branch, a remote-tracking reference or a tag reaches its commit, so nothing was lost"),
        WhyPruned::TooOld => format!("it reached no branch and was older than the {days}-day age limit"),
        WhyPruned::TooMuchKept => format!(
            "it reached no branch, and the objects only kept references reached weighed more than the {} bytes the size limit allows, so the oldest went first",
            limits.max_bytes
        ),
    };
}

/// What:
///  One line of the log,
///  as it is written.
/// Why:
///   The fields are the ones a person needs,
///  and the shape is stated in the file that writes it,
///  so a reader of the log knows what each field is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineOf = (pruned, now, limits) => JSON.stringify({ time_ms, time, reference, commit, batch, rescued, age_seconds, kept_at, worktree, why, why_words }) + '\n';
/// ```
pub fn line_of(pruned: &Pruned, now: u128, limits: crate::kept_reference_limits::KeptReferenceLimits) -> String {
    let mut object: Map<String, Value> = Map::new();
    object.insert(String::from("time_ms"), Value::from(u64::try_from(now).unwrap_or(u64::MAX)));
    object.insert(String::from("time"), Value::from(format_iso_milliseconds(i64::try_from(now).unwrap_or(i64::MAX))));
    object.insert(String::from("reference"), Value::from(pruned.reference.as_str()));
    object.insert(String::from("commit"), Value::from(pruned.commit.as_str()));
    object.insert(String::from("batch"), Value::from(pruned.batch));
    object.insert(String::from("rescued"), Value::from(pruned.rescued));
    object.insert(String::from("age_seconds"), Value::from(u64::try_from(pruned.age_ms / 1000).unwrap_or(u64::MAX)));
    object.insert(String::from("kept_at"), Value::from(format_iso_milliseconds(i64::try_from(pruned.kept_at_ms).unwrap_or(i64::MAX))));
    object.insert(String::from("worktree"), pruned.worktree.as_ref().map_or(Value::Null, |path| { return Value::from(path.as_str()); }));
    object.insert(String::from("why"), Value::from(pruned.why.word()));
    object.insert(String::from("why_words"), Value::from(why_words(pruned.why, limits)));
    return format!("{}\n", Value::Object(object));
}

/// What:
///  Write the one line a pruned kept reference leaves.
/// Why:
///   The line is written before the reference goes and never after,
///  so a person reading the log knows that a commit named there is gone or about to be,
///  and a log that cannot be written never stops the pruning:
///  a reference the limits allow to go is removed even when no log takes it,
///  which is the choice the owner's rule about unbounded growth asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { appendLine(logPath(commonDir), lineOf(pruned), UNPINNED_LOG); } catch (error) { debug(TAG, error) }
/// ```
pub fn log_one_pruning(common_dir: &Path, pruned: &Pruned, now: u128) {
    let limits = crate::kept_reference_limits::limits_from_the_environment();
    let line: String = line_of(pruned, now, limits);
    if let Err(error) = append_line(log_path(common_dir).as_path(), line.as_bytes(), &UNPINNED_LOG) {
        crate::diagnostic_log::warn(
            "keptReferenceLog",
            format!("the line about the pruned kept reference {} could not be written: {error}", pruned.reference).as_str(),
        );
    }
}

/// Controls for the log stay out of the release executable.
#[cfg(test)]
#[path = "kept_reference_log_tests.rs"]
mod tests;
