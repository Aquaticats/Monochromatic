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

use crate::diagnostic_log::debug;
use crate::merge_second_pass::{Conflict, GitRunner, Left, Merged, Verdict, journal_merged, report_lines, second_pass, tree_with_files, write_blob};
use crate::path_bytes::path_message_text;
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::read_layout_after_success_with;
use crate::random_id::random_uuid;
use crate::scratch_location::prepare_repository_scratch;
use std::ffi::OsString;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeTree";

/// The variable that names the tree Git reads `.gitattributes` from when there is no worktree.
pub const ATTRIBUTE_SOURCE_VARIABLE: &str = "GIT_ATTR_SOURCE";

/// The configuration key that names that tree when neither `--attr-source` nor the variable does (`Documentation/config/attr.adoc`).
const ATTRIBUTE_TREE_KEY: &str = "attr.tree";

/// What the attribute-tree choice needs to know about where a merge runs.
pub struct AttributeProbe<'a> {
    /// The worktree (or a directory inside the repository) the merge belongs to.
    pub directory: &'a Path,
    /// The invocation's global options, among which `--attr-source` may be.
    pub global_prefix: &'a [OsString],
    /// The complete environment the Git children get: the process's own and the overlay.
    pub variables: &'a [(OsString, OsString)],
    /// How to start Git, to resolve a configured `attr.tree`.
    pub runner: &'a dyn GitRunner,
}

/// What:
///  The value of `attr.tree`, read in process when the readers take the repository and from Git when they decline it; `None` when
///  neither can say, and `Some(None)` when it is not configured.
/// Why:
///   The in-process reader declines an environment that holds a `GIT_*` variable it does not know (`GIT_EDITOR` is one many
///  people set), and a declined repository must not silently lose the person's attribute tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configuredAttributeTree(probe: AttributeProbe): string | null | undefined;
/// ```
fn configured_attribute_tree(probe: &AttributeProbe<'_>) -> Option<Option<String>> {
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(probe.variables);
    let listed = match read_layout_after_success_with(probe.directory, probe.variables, &scope) {
        Some(layout) => list_config(layout.common_dir.as_path(), layout.git_dir.as_path(), &scope, probe.variables),
        None => None,
    };
    if let Some(entries) = listed {
        // The last entry of the key is the value Git uses; empty text and bytes that are not text count as no value.
        let mut value: Option<String> = None;
        for item in entries.iter().rev() {
            if item.entry.key == ATTRIBUTE_TREE_KEY {
                if let Some(bytes) = item.entry.value.as_deref()
                    && let Ok(text) = std::str::from_utf8(bytes)
                    && !text.is_empty()
                {
                    value = Some(text.to_owned());
                }
                break;
            }
        }
        return Some(value);
    }
    debug(TAG, "the in-process readers decline the repository, so Git is asked for attr.tree");
    let reply = probe.runner.run(&["config", "--get", ATTRIBUTE_TREE_KEY], None, None).ok()?;
    return match reply.code {
        Some(0) => {
            let text: String = String::from_utf8_lossy(&reply.stdout).trim().to_owned();
            Some(if text.is_empty() { None } else { Some(text) })
        }
        Some(1) => Some(None),
        _ => None,
    };
}

/// What:
///  The variable pair that makes a merge from objects read the attributes of `tip`, only where Git itself would read no
///  attribute tree.
/// Why:
///   Git takes the attribute tree from `--attr-source`, else from `GIT_ATTR_SOURCE`, else from the configuration `attr.tree`
///  (`attr.c`, `compute_default_attr_source`, lines 1202 to 1231); a value of the configuration that does not resolve to a tree
///  is ignored, as Git ignores it, while one of the other two that does not resolve is an error of Git's own. So the first
///  side's tip is named only when none of the three applies: the person's choice is never overridden. A layout or a
///  configuration the in-process readers decline counts as a person's choice that cannot be seen, and nothing is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeTreeVariables(tip: string, probe: AttributeProbe): [string, string][];
/// ```
pub fn attribute_tree_variables(tip: &str, probe: &AttributeProbe<'_>) -> Vec<(OsString, OsString)> {
    for (name, _value) in probe.variables {
        if name == ATTRIBUTE_SOURCE_VARIABLE {
            return Vec::new();
        }
    }
    for token in probe.global_prefix {
        if let Some(text) = token.to_str()
            && (text == "--attr-source" || text.starts_with("--attr-source="))
        {
            return Vec::new();
        }
    }
    let Some(configured) = configured_attribute_tree(probe) else {
        debug(TAG, "attr.tree cannot be read, so no attribute tree is named");
        return Vec::new();
    };
    if let Some(value) = configured {
        let spec: String = format!("{value}^{{tree}}");
        let resolves: bool = match probe.runner.run(&["rev-parse", "--verify", "--quiet", spec.as_str()], None, None) {
            Ok(reply) => reply.code == Some(0),
            Err(_) => false,
        };
        if resolves {
            return Vec::new();
        }
        debug(TAG, format!("attr.tree names {value}, which is no tree; Git ignores it, so the first side's tip is named").as_str());
    }
    return vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from(tip))];
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
    let mut files: Vec<(&[u8], u32, &str)> = Vec::new();
    for (file, oid) in merged.iter().zip(oids.iter()) {
        files.push((file.path.as_slice(), file.mode, oid.as_str()));
    }
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
    let mut lines: Vec<String> = Vec::new();
    for file in merged {
        lines.extend(report_lines(prefix, file));
    }
    return lines;
}

/// What:
///  Send the journal entries for the merged files and return the lines that list their parts.
/// Why:
///   The entries are sent only when the merge is used, so the journal never says a merge happened that was discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function announce(prefix: string, sender: [identifier: string, operation: string], merged: Merged[]): string[];
/// ```
pub fn announce(prefix: &str, sender: (&str, &str), merged: &[Merged]) -> Vec<String> {
    let (identifier, operation): (&str, &str) = sender;
    for file in merged {
        journal_merged(identifier, operation, file);
    }
    return report(prefix, merged);
}

/// Controls for the attribute source and for the tree of a resolved merge.
#[cfg(test)]
#[path = "merge_second_pass_tree_tests.rs"]
mod tests;
