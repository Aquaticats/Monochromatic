//! What:
//!  The word-level second pass for a merge computed from objects:
//! given the tree `git merge-tree` wrote and the stages of
//!  its conflicted paths,
//! the tree the merge would have if every conflicted file resolves by word,
//! or nothing.
//!  It also names the attribute tree those merges read,
//! so the `union` line of `.gitattributes` applies without a worktree,
//! and the tree the second pass reads the `merge` attribute from.
//! Why:
//!  The isolated merge,
//! the auto-push reconcile merge and the commit replay merge never touch a worktree.
//! They publish or
//!  commit a merge only when it is whole,
//! so a tree is returned only when every conflicted path resolved;
//! one path left over
//!  keeps the conflict as Git reported it,
//! and nothing is written for it except the unreferenced objects.
//!  Without a worktree Git finds no `.gitattributes` unless it is told where to read them (`GIT_ATTR_SOURCE`,
//!  `git --attr-source`,
//! `attr.tree`;
//! `doc/planning/cli-git-merge-auto-resolution.md`,
//! "Does git merge-tree honour a merge
//!  driver").
//! The tree of the first side's tip is the one the person's own checkout of that branch reads,
//! and it is what `git merge-tree`
//!  itself reads.
//! The tree the merge produced is what the second pass reads
//! (`git check-attr --source=<tree>`),
//! so a `merge`
//!  rule the merge itself introduces governs the pass;
//! a `.gitattributes` that merge left conflicted names no source,
//! so no
//!  candidate of it is merged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const resolved = await resolveTree(runner, { commonDir, tree, conflicts }); // { tree, merged } | undefined
//! ```

use crate::child_environment::environment_value;
use crate::diagnostic_log::debug;
use crate::merge_second_pass::{AttributeSource, Conflict, GitRunner, INPUTS_HINT, Left, Merged, Verdict, is_attributes_path, journal_merged, report_lines, second_pass, tree_with_files, write_blob};
use crate::path_bytes::path_message_text;
use crate::repo_read_config_list::{canonical_key, list_config};
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
    /// The invocation's global options,
    /// among which `--attr-source` may be.
    pub global_prefix: &'a [OsString],
    /// The complete environment the Git children get:
    /// the process's own and the overlay.
    pub variables: &'a [(OsString, OsString)],
    /// How to start Git,
    /// to resolve a configured `attr.tree`.
    pub runner: &'a dyn GitRunner,
}

/// What:
///  The value of `attr.tree`,
/// read in process when the readers take the repository and from Git when they decline it;
/// `None` when
///  neither can say,
/// and `Some(None)` when it is not configured.
/// Why:
///   The in-process reader declines an environment that holds a `GIT_*` variable it does not know (`GIT_EDITOR` is one many
///  people set),
/// and a declined repository must not silently lose the person's attribute tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configuredAttributeTree(probe: AttributeProbe): string | null | undefined;
/// ```
async fn configured_attribute_tree(probe: &AttributeProbe<'_>) -> Option<Option<String>> {
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(probe.variables);
    let listed = match read_layout_after_success_with(probe.directory, probe.variables, &scope) {
        Some(layout) => list_config(layout.common_dir.as_path(), layout.git_dir.as_path(), &scope, probe.variables),
        None => None,
    };
    if let Some(entries) = listed {
        // The last entry of the key is the value Git uses; empty text and bytes that are not text count as no value.
        let mut value: Option<String> = None;
        for item in entries.iter().rev() {
            if canonical_key(item.entry.key.as_str()).as_deref() == Some(ATTRIBUTE_TREE_KEY) {
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
    let reply = probe.runner.run(&["config", "--get", ATTRIBUTE_TREE_KEY], None, None).await.ok()?;
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
///  The value the invocation's own global options give `attr.tree`,
/// the last one winning,
/// or nothing when the command line names no attribute tree.
/// Why:
///   Git takes a `-c name=value` and a `--config-env=name=variable` above every configuration file
/// (`git_config_from_parameters`),
/// so a value given there is the one Git uses
///  whether a file sets the key or not;
/// a value no tree resolves to is ignored,
/// as a configured one is,
/// and is told apart from
///  none at all here.
/// `--config-env` takes its value from the environment,
/// and Git stops before it reads anything when
///  the variable it names is not there,
/// so that case is told apart from "nothing names the key".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandLineAttributeTree(prefix: string[], variables: [string, string][]): 'not-set' | { value: string | null } | 'unknown';
/// ```
fn command_line_attribute_tree(prefix: &[OsString], variables: &[(OsString, OsString)]) -> CommandLineTree {
    let mut named: CommandLineTree = CommandLineTree::NotSet;
    let mut tokens = prefix.iter();
    while let Some(token) = tokens.next() {
        let Some(text) = token.to_str() else { continue };
        // The entry an option carries: `-c` and a bare `--config-env` take the next token, and `--config-env=` carries it.
        let (entry, from_environment): (&str, bool) = if let Some(carried) = text.strip_prefix("--config-env=") {
            (carried, true)
        } else if text == "-c" || text == "--config-env" {
            let Some(next) = tokens.next() else { continue };
            let Some(carried) = next.to_str() else { continue };
            (carried, text == "--config-env")
        } else {
            continue;
        };
        // `-c name=value` splits at the first `=` and `--config-env=name=variable` at the last (`config.c` lines 495 and 511):
        // the variable name may hold no `=` while the key may, and Git refuses a `--config-env` entry with none at all.
        let split: Option<(&str, &str)> = if from_environment { entry.rsplit_once('=') } else { Some(entry.split_once('=').unwrap_or((entry, ""))) };
        let Some((key, given)) = split else { continue };
        if canonical_key(key).as_deref() != Some(ATTRIBUTE_TREE_KEY) {
            continue;
        }
        if !from_environment {
            // `-c name` alone sets `true`, which no tree resolves to.
            named = if given.is_empty() { CommandLineTree::Set(None) } else { CommandLineTree::Set(Some(given.to_owned())) };
            continue;
        }
        let Some(value) = environment_value(variables, given) else {
            named = CommandLineTree::Unknown;
            continue;
        };
        let from_the_variable: String = value.to_string_lossy().into_owned();
        // An empty value is no value, which Git ignores as it ignores a configured one.
        named = if from_the_variable.is_empty() { CommandLineTree::Set(None) } else { CommandLineTree::Set(Some(from_the_variable)) };
    }
    return named;
}

/// What the invocation's global options say about `attr.tree`.
enum CommandLineTree {
    /// No option sets the key.
    NotSet,
    /// An option sets it to this value,
    /// or to no value Git can resolve (`-c attr.tree` alone,
    /// or an empty value).
    Set(Option<String>),
    /// An option names the key,
    /// but takes its value from a variable the environment does not hold,
    /// so Git stops before it reads anything.
    Unknown,
}

/// What the attribute-tree precedence came to for a merge that must name a tree.
enum AttributeChoice {
    /// The person named one (`--attr-source`,
    /// `GIT_ATTR_SOURCE`,
    /// a resolving `attr.tree`),
    /// or nothing can say whether one is named.
    ThePerson,
    /// Nothing names one.
    Free,
}

/// What:
///  The attribute-tree precedence Git itself applies,
/// without naming a tree.
/// Why:
///   Git takes the attribute tree from `--attr-source`,
/// else from `GIT_ATTR_SOURCE`,
/// else from the configuration `attr.tree`
///  (`attr.c`,
/// `compute_default_attr_source`,
/// lines 1202 to 1231);
/// a value of the configuration that does not resolve to a tree
///  is ignored,
/// as Git ignores it,
/// while one of the other two that does not resolve is an error of Git's own.
/// A layout or a
///  configuration the in-process readers decline counts as a person's choice that cannot be seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeChoice(probe: AttributeProbe): 'the-person' | 'free';
/// ```
async fn attribute_choice(probe: &AttributeProbe<'_>) -> AttributeChoice {
    for (name, _value) in probe.variables {
        if name == ATTRIBUTE_SOURCE_VARIABLE {
            return AttributeChoice::ThePerson;
        }
    }
    for token in probe.global_prefix {
        if let Some(text) = token.to_str()
            && (text == "--attr-source" || text.starts_with("--attr-source="))
        {
            return AttributeChoice::ThePerson;
        }
    }
    let read: Option<Option<String>> = match command_line_attribute_tree(probe.global_prefix, probe.variables) {
        // The command line names the key and Git stops on the value it cannot find, so no tree is named over it.
        CommandLineTree::Unknown => {
            debug(TAG, "an attr.tree of the command line takes its value from a variable the environment does not hold, and Git stops on it, so no attribute tree is named");
            return AttributeChoice::ThePerson;
        }
        CommandLineTree::Set(from_the_command_line) => Some(from_the_command_line),
        CommandLineTree::NotSet => configured_attribute_tree(probe).await,
    };
    let Some(configured) = read else {
        debug(TAG, "attr.tree cannot be read, so no attribute tree is named");
        return AttributeChoice::ThePerson;
    };
    if let Some(value) = configured {
        let spec: String = format!("{value}^{{tree}}");
        let resolves: bool = match probe.runner.run(&["rev-parse", "--verify", "--quiet", spec.as_str()], None, None).await {
            Ok(reply) => reply.code == Some(0),
            Err(_) => false,
        };
        if resolves {
            return AttributeChoice::ThePerson;
        }
        debug(TAG, format!("attr.tree names {value}, which is no tree; Git ignores it, so the tree is named").as_str());
    }
    return AttributeChoice::Free;
}

/// What:
///  The variable pair that makes a merge from objects read the attributes of `tip`,
/// only where Git itself would read no
///  attribute tree.
/// Why:
///   So the first side's tip is named only when none of the three sources applies:
/// the person's choice is never
///  overridden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attributeTreeVariables(tip: string, probe: AttributeProbe): Promise<[string, string][]>;
/// ```
pub async fn attribute_tree_variables(tip: &str, probe: &AttributeProbe<'_>) -> Vec<(OsString, OsString)> {
    return match attribute_choice(probe).await {
        AttributeChoice::ThePerson => Vec::new(),
        AttributeChoice::Free => vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from(tip))],
    };
}

/// What:
///  The same variables for a caller that is not inside the runtime,
/// entered on the runtime once.
/// Why:
///   The isolated merge,
/// the commit replay merge and the shadow replay read the attribute tree
/// on a thread of their own,
/// and the face above is refused there only inside the runtime;
/// the choice,
/// the questions and the words are the awaited face's,
/// so the two cannot name different trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(() => attributeTreeVariables(tip, probe))
/// ```
pub fn attribute_tree_variables_on_the_runtime(tip: &str, probe: &AttributeProbe<'_>) -> Vec<(OsString, OsString)> {
    return match crate::lazy_runtime::run_on_runtime(attribute_tree_variables(tip, probe)) {
        Ok(found) => found,
        Err(refusal) => {
            crate::diagnostic_log::warn(TAG, format!("the attribute tree could not be read, so none is named: {refusal}").as_str());
            Vec::new()
        }
    };
}

/// What:
///  Whether one of the conflicts of a merge from objects is a `.gitattributes` file.
/// Why:
///   The tree the merge produced carries Git's markers in a rule file both sides changed,
/// so no rule of it can be trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anyAttributesConflict = (conflicts: Conflict[]) => conflicts.some((conflict) => isAttributesPath(conflict.path));
/// ```
fn any_attributes_conflict(conflicts: &[Conflict]) -> bool {
    for conflict in conflicts {
        if is_attributes_path(conflict.path.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Where the second pass of a merge from objects reads the `merge` attribute of its candidates.
/// Why:
///   The tree the merge produced carries the `.gitattributes` of both sides as the merge left them,
/// so a rule the
///  merge itself introduces applies to the files of that same merge;
/// reading the first side's tip alone let such a
///  file be merged by word and committed.
/// The tree is named only where the person named none,
/// and only when the merge
///  left no `.gitattributes` conflicted:
/// a conflicted one holds Git's markers,
/// and a rule read from
///  between them is no side's rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeAttributeSource(tree: string, conflicts: Conflict[], probe: AttributeProbe): AttributeSource;
/// ```
pub async fn merge_attribute_source<'a>(tree: &'a str, conflicts: &[Conflict], probe: &AttributeProbe<'_>) -> AttributeSource<'a> {
    return match attribute_choice(probe).await {
        AttributeChoice::ThePerson => AttributeSource::Invocation,
        AttributeChoice::Free if any_attributes_conflict(conflicts) => AttributeSource::Unknown,
        AttributeChoice::Free => AttributeSource::Tree(tree),
    };
}

/// A merge tree whose every conflicted file resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolved {
    /// The tree with the merged files in place of the conflicted ones.
    pub tree: String,
    /// The files that were merged,
    /// with their parts.
    pub merged: Vec<Merged>,
}

/// What:
///  The tree of a merge whose conflicted files all resolve by word,
/// or `None` when any does not or Git fails.
/// Why:
///   See the module comment.
/// Every reason for `None` goes to the diagnostic log with the path it concerns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveTree(runner, commonDir, tree, conflicts): Promise<Resolved | undefined>;
/// ```
pub async fn resolve_tree(runner: &dyn GitRunner, common_dir: &Path, tree: &str, conflicts: &[Conflict], probe: &AttributeProbe<'_>) -> Option<Resolved> {
    if conflicts.is_empty() {
        return None;
    }
    let source: AttributeSource<'_> = merge_attribute_source(tree, conflicts, probe).await;
    let verdicts: Vec<Verdict> = match second_pass(runner, conflicts, source).await {
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
        match write_blob(runner, file.bytes.as_slice()).await {
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
    let rewritten: Result<String, String> = tree_with_files(runner, tree, files.as_slice(), index.as_path()).await;
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

/// What:
///  The same tree for a caller that is not inside the runtime,
/// entered on the runtime once.
/// Why:
///   The isolated merge and the commit replay merge compute their merge on a thread of their own,
/// which enters the runtime for every Git start it makes;
/// this face is the same one entry,
/// and the pass,
/// the questions and the logs are the awaited face's,
/// so a merge from objects resolves the same way however it was reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(() => resolveTree(runner, commonDir, tree, conflicts, probe))
/// ```
pub fn resolve_tree_on_the_runtime(runner: &dyn GitRunner, common_dir: &Path, tree: &str, conflicts: &[Conflict], probe: &AttributeProbe<'_>) -> Option<Resolved> {
    return match crate::lazy_runtime::run_on_runtime(resolve_tree(runner, common_dir, tree, conflicts, probe)) {
        Ok(found) => found,
        Err(refusal) => {
            crate::diagnostic_log::warn(TAG, format!("the second pass could not be run, so the conflicts stay: {refusal}").as_str());
            None
        }
    };
}

/// Why a path was left,
/// in words for the diagnostic log.
fn left_words(why: &Left) -> String {
    return match why {
        Left::MissingStage => String::from("a stage is missing"),
        Left::NotAPlainFile => String::from("not a plain file with one mode"),
        Left::Binary => String::from("binary"),
        Left::TooLarge => String::from("too large"),
        Left::Driver(value) => format!("merge attribute {value}"),
        Left::Overlap(count) => format!("{count} regions hold two changes of the same words"),
        Left::UnknownAttributes => String::from("a .gitattributes of the merge is itself conflicted"),
    };
}

/// The lines that list the parts of the merged files,
/// and the sentence that says how to read an input they name,
/// without sending anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = [...merged.flatMap((file) => reportLines(prefix, file)), ...(merged.some((file) => file.parts.length > 0) ? [`${prefix}${INPUTS_HINT}`] : [])];
/// ```
pub fn report(prefix: &str, merged: &[Merged]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut parts: usize = 0;
    for file in merged {
        parts += file.parts.len();
        lines.extend(report_lines(prefix, file));
    }
    if parts > 0 {
        lines.push(format!("{prefix}{INPUTS_HINT}"));
    }
    return lines;
}

/// What:
///  Send the journal entries for the merged files and return the lines that list their parts.
/// Why:
///   The entries are sent only when the merge is used,
/// so the journal never says a merge happened that was discarded.
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
