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
//! Where that attribute tree is read from lives in `merge_second_pass_tree_attributes.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const resolved = await resolveTree(runner, { commonDir, tree, conflicts }); // { tree, merged } | undefined
//! ```

/// The jobs of the blocking pool that make this step's file calls,
/// so a disk that makes them wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::diagnostic_log::debug;
use crate::merge_second_pass::{
    AttributeSource,
    Conflict,
    GitRunner,
    INPUTS_HINT,
    Left,
    Merged,
    Verdict,
    journal_merged,
    report_lines,
    second_pass,
    tree_with_files,
    write_blob,
};
use crate::path_bytes::path_message_text;
use crate::random_id::random_uuid;
use crate::scratch_location::prepare_repository_scratch;
use std::path::Path;

/// The tree a merge from objects reads its `merge` attribute from,
/// and the attribute source that merge names,
/// reachable at this module's path as they were before the split.
pub use crate::merge_second_pass_tree_attributes::{
    ATTRIBUTE_SOURCE_VARIABLE,
    AttributeProbe,
    attribute_tree_variables,
    attribute_tree_variables_on_the_runtime,
    merge_attribute_source,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeTree";

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
pub async fn resolve_tree(
    runner: &dyn GitRunner,
    common_dir: &Path,
    tree: &str,
    conflicts: &[Conflict],
    probe: &AttributeProbe<'_>,
) -> Option<Resolved> {
    if conflicts.is_empty() {
        return None;
    }
    let source: AttributeSource<'_> = merge_attribute_source(tree, conflicts, probe).await;
    let verdicts: Vec<Verdict> = match second_pass(runner, conflicts, source).await {
        Ok(found) => found,
        Err(reason) => {
            debug(
                TAG,
                format!("the second pass could not run, so the conflicts stay: {reason}").as_str(),
            );
            return None;
        }
    };
    let mut merged: Vec<Merged> = Vec::new();
    let mut left: Vec<String> = Vec::new();
    for verdict in verdicts {
        match verdict {
            Verdict::Merged(file) => merged.push(*file),
            Verdict::Left {
                path,
                why,
            } => left.push(format!(
                "{} ({})",
                path_message_text(path.as_slice()),
                left_words(&why)
            )),
        }
    }
    if !left.is_empty() {
        debug(
            TAG,
            format!(
                "the merge stays conflicted; left to the person: {}",
                left.join(", ")
            )
            .as_str(),
        );
        return None;
    }
    let mut oids: Vec<String> = Vec::new();
    for file in &merged {
        match write_blob(runner, file.bytes.as_slice()).await {
            Ok(oid) => oids.push(oid),
            Err(reason) => {
                debug(
                    TAG,
                    format!(
                        "the merged blob of {} could not be written: {reason}",
                        path_message_text(file.path.as_slice())
                    )
                    .as_str(),
                );
                return None;
            }
        }
    }
    // Making the scratch directory, and the sweep of the repository scratch it makes, are file calls and a walk that can wait
    //  on a disk, so a job of the blocking pool makes them.
    let common_directory: std::path::PathBuf = common_dir.to_path_buf();
    let scratch = match run_blocking_job_within_the_budget(move || {
        return prepare_repository_scratch(common_directory.as_path());
    })
    .await
    {
        Ok(Ok(found)) => found,
        Ok(Err(reason)) => {
            debug(
                TAG,
                format!("no scratch directory for the second pass: {reason}").as_str(),
            );
            return None;
        }
        // The system gave the pool no thread, so no scratch directory was made and the second pass has none.
        Err(refused) => {
            debug(
                TAG,
                format!("no scratch directory for the second pass: {refused}").as_str(),
            );
            return None;
        }
    };
    let name: String = match random_uuid() {
        Ok(found) => found,
        Err(reason) => {
            debug(
                TAG,
                format!("no unique name for the scratch index: {}", reason.0).as_str(),
            );
            return None;
        }
    };
    let index = scratch.join(format!("second-pass-{name}.index"));
    let mut files: Vec<(&[u8], u32, &str)> = Vec::new();
    for (file, oid) in merged.iter().zip(oids.iter()) {
        files.push((file.path.as_slice(), file.mode, oid.as_str()));
    }
    let rewritten: Result<String, String> =
        tree_with_files(runner, tree, files.as_slice(), index.as_path()).await;
    // The removal of the scratch index is a file call that can wait on a disk, so a job of the blocking pool makes it.
    let removing: std::path::PathBuf = index.clone();
    let removed = run_blocking_job_within_the_budget(move || {
        return std::fs::remove_file(removing.as_path());
    })
    .await;
    match removed {
        // The index went, which is all this step asks of the removal.
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            debug(TAG, format!("the scratch index could not be removed ({error}); the repository scratch sweep collects it").as_str());
        }
        // The name may still be there, and the repository scratch sweep is what collects it.
        Err(refused) => {
            debug(TAG, format!("the scratch index could not be removed ({refused}); the repository scratch sweep collects it").as_str());
        }
    }
    return match rewritten {
        Ok(found) => Some(Resolved {
            tree: found,
            merged,
        }),
        Err(reason) => {
            debug(
                TAG,
                format!("the merged tree could not be built: {reason}").as_str(),
            );
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
pub fn resolve_tree_on_the_runtime(
    runner: &dyn GitRunner,
    common_dir: &Path,
    tree: &str,
    conflicts: &[Conflict],
    probe: &AttributeProbe<'_>,
) -> Option<Resolved> {
    return match crate::lazy_runtime::run_on_runtime(resolve_tree(
        runner, common_dir, tree, conflicts, probe,
    )) {
        Ok(found) => found,
        Err(refusal) => {
            crate::diagnostic_log::warn(
                TAG,
                format!("the second pass could not be run, so the conflicts stay: {refusal}")
                    .as_str(),
            );
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
        Left::Overlap(count) => format!(
            "{count} regions hold two changes of the same words, or a word merge of one would write a doubled word or a line no side holds, or a region's split-shaped line is not the only one that could be Git's"
        ),
        Left::UnknownAttributes => {
            String::from("a .gitattributes of the merge is itself conflicted")
        }
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
pub fn report(
    prefix: &str,
    merged: &[Merged],
) -> Vec<String> {
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
pub fn announce(
    prefix: &str,
    sender: (&str, &str),
    merged: &[Merged],
) -> Vec<String> {
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
