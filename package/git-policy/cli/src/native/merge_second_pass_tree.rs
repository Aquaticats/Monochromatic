//! What:
//!  The word-level second pass for a merge computed from objects: given the tree `git merge-tree` wrote and the stages of
//!  its conflicted paths, the tree the merge would have if every conflicted file resolves by word, or nothing.
//!  It also names the attribute tree those merges read, so the `union` line of `.gitattributes` applies without a worktree.
//! Why:
//!  The isolated merge, the auto-push reconcile merge and the commit replay merge never touch a worktree. They publish or
//!  commit a merge only when it is whole, so a tree is returned only when every conflicted path resolved; one path left over
//!  keeps the conflict as Git reported it, and nothing is written for it except the unreferenced objects.
//!  Without a worktree Git finds no `.gitattributes` unless it is told where to read them (`GIT_ATTR_SOURCE`,
//!  `git --attr-source`, `attr.tree`; `doc/planning/cli-git-merge-auto-resolution.md`, "Does git merge-tree honour a merge
//!  driver"). The tree of the first side's tip is the one the person's own checkout of that branch reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const resolved = await resolveTree(runner, { commonDir, tree, conflicts }); // { tree, merged } | undefined
//! ```

use super::diagnostic_log::debug;
use super::merge_second_pass::{Conflict, GitRunner, Left, Merged, Verdict, journal_merged, report_lines, second_pass, tree_with_files, write_blob};
use super::path_bytes::path_message_text;
use super::random_id::random_uuid;
use super::scratch_location::prepare_repository_scratch;
use std::ffi::OsString;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeTree";

/// The variable that names the tree Git reads `.gitattributes` from when there is no worktree.
pub const ATTRIBUTE_SOURCE_VARIABLE: &str = "GIT_ATTR_SOURCE";

/// What:
///  The variable pair that makes a merge from objects read the attributes of `tip`, unless the caller's own environment
///  already names an attribute source.
/// Why:
///   A person who set `GIT_ATTR_SOURCE` meant it; the wrapper adds a source only where there was none.
///   The source is the first side's tip: the tree whose `.gitattributes` a checkout of that branch would apply.
///   A `.gitattributes` that only the other side has is not read, as Git offers one tree only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const variables = env.GIT_ATTR_SOURCE === undefined ? { GIT_ATTR_SOURCE: tip } : {};
/// ```
pub fn attribute_source_variables(tip: &str, environment: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    if environment.iter().any(|(name, _)| { return name == ATTRIBUTE_SOURCE_VARIABLE; }) {
        return Vec::new();
    }
    return vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from(tip))];
}

/// What:
///  The same variable pair for a process that holds the environment itself: the service's own, and the overlay its Git
///  starts add.
/// Why:
///   The commit replay runs in the service, whose Git children inherit the service's environment plus the overlay; a person's
///  `GIT_ATTR_SOURCE` that reached either is kept there too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// attributeSourceVariablesOfProcess(tip, overlay) // [] when process.env or the overlay names GIT_ATTR_SOURCE
/// ```
pub fn attribute_source_variables_of_process(tip: &str, overlay: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    if std::env::var_os(ATTRIBUTE_SOURCE_VARIABLE).is_some() {
        return Vec::new();
    }
    return attribute_source_variables(tip, overlay);
}

/// A merge tree whose every conflicted file resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolved {
    /// The tree with the merged files in place of the conflicted ones.
    pub tree: String,
    /// The files that were merged, with their parts.
    pub merged: Vec<Merged>,
}

/// What:
///  The tree of a merge whose conflicted files all resolve by word, or `None` when any does not or Git fails.
/// Why:
///   See the module comment. Every reason for `None` goes to the diagnostic log with the path it concerns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveTree(runner, commonDir, tree, conflicts): Promise<Resolved | undefined>;
/// ```
pub fn resolve_tree(runner: &dyn GitRunner, common_dir: &Path, tree: &str, conflicts: &[Conflict]) -> Option<Resolved> {
    if conflicts.is_empty() {
        return None;
    }
    let verdicts: Vec<Verdict> = match second_pass(runner, conflicts) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("the second pass could not run, so the conflicts stay: {reason}").as_str());
            return None;
        }
    };
    let mut merged: Vec<Merged> = Vec::new();
    let mut left: Vec<String> = Vec::new();
    for verdict in verdicts {
        match verdict {
            Verdict::Merged(file) => merged.push(*file),
            Verdict::Left { path, why } => left.push(format!("{} ({})", path_message_text(path.as_slice()), left_words(&why))),
        }
    }
    if !left.is_empty() {
        debug(TAG, format!("the merge stays conflicted; left to the person: {}", left.join(", ")).as_str());
        return None;
    }
    let mut oids: Vec<String> = Vec::new();
    for file in &merged {
        match write_blob(runner, file.bytes.as_slice()) {
            Ok(oid) => oids.push(oid),
            Err(reason) => {
                debug(TAG, format!("the merged blob of {} could not be written: {reason}", path_message_text(file.path.as_slice())).as_str());
                return None;
            }
        }
    }
    let scratch = match prepare_repository_scratch(common_dir) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("no scratch directory for the second pass: {reason}").as_str());
            return None;
        }
    };
    let name: String = match random_uuid() {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("no unique name for the scratch index: {}", reason.0).as_str());
            return None;
        }
    };
    let index = scratch.join(format!("second-pass-{name}.index"));
    let files: Vec<(&[u8], u32, &str)> = merged.iter().zip(oids.iter()).map(|(file, oid)| { return (file.path.as_slice(), file.mode, oid.as_str()); }).collect();
    let rewritten: Result<String, String> = tree_with_files(runner, tree, files.as_slice(), index.as_path());
    if let Err(error) = std::fs::remove_file(index.as_path()) {
        debug(TAG, format!("the scratch index could not be removed ({error}); the repository scratch sweep collects it").as_str());
    }
    return match rewritten {
        Ok(found) => Some(Resolved { tree: found, merged }),
        Err(reason) => {
            debug(TAG, format!("the merged tree could not be built: {reason}").as_str());
            None
        }
    };
}

/// Why a path was left, in words for the diagnostic log.
fn left_words(why: &Left) -> String {
    return match why {
        Left::MissingStage => String::from("a stage is missing"),
        Left::NotAPlainFile => String::from("not a plain file with one mode"),
        Left::Binary => String::from("binary"),
        Left::TooLarge => String::from("too large"),
        Left::Driver(value) => format!("merge attribute {value}"),
        Left::Overlap(count) => format!("{count} regions hold two changes of the same words"),
    };
}

/// The lines that list the parts of the merged files, without sending anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = merged.flatMap((file) => reportLines(prefix, file));
/// ```
pub fn report(prefix: &str, merged: &[Merged]) -> Vec<String> {
    return merged.iter().flat_map(|file| { return report_lines(prefix, file); }).collect();
}

/// What:
///  Send the journal entries for the merged files and return the lines that list their parts.
/// Why:
///   The entries are sent only when the merge is used, so the journal never says a merge happened that was discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function announce(prefix: string, operation: string, merged: Merged[]): string[];
/// ```
pub fn announce(prefix: &str, operation: &str, merged: &[Merged]) -> Vec<String> {
    for file in merged {
        journal_merged(operation, file);
    }
    return report(prefix, merged);
}

/// Controls for the attribute source and for the tree of a resolved merge.
#[cfg(test)]
#[path = "merge_second_pass_tree_tests.rs"]
mod tests;
