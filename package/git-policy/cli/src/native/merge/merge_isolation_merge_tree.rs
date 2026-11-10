//! What:
//!  The merge tree of an isolated merge:
//! the merge computed from objects alone,
//! and the word-level second pass over
//!  the conflicts it leaves.
//! Why:
//!  `merge_isolation_apply.rs` writes the objects and moves the checkout;
//! the merge that is computed before any of that,
//! and the pass over its conflicts,
//!  are a file of their own that it imports and that imports nothing from it.
//!  The pass awaits every Git start it makes
//! (`merge_second_pass_git_runner.rs`),
//! so the merge from objects is one entry of the runtime for a synchronous caller
//! and no entry at all for a task of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { tree, merged } = await mergeTreeOrRefuse({ session, shape, ends, places, stagedWork });
//! ```

/// Reading an environment value.
use crate::child_environment::environment_with_overlay;
/// The word-level second pass over the conflicts of the merge tree,
/// and the attribute tree it reads.
use crate::merge_second_pass::{ContextRunner, GitReply, GitRunner, Merged, config_options, conflicts_of_merge_tree};
use crate::merge_second_pass_tree::{AttributeProbe, attribute_tree_variables, resolve_tree};
use crate::path_bytes::path_message_text;
/// The command-line reading.
use crate::merge_options::MergeShape;
/// The refusal findings.
use crate::merge_isolation_report::conflict_finding;
use crate::merge_isolation_run_types::{Bail, refused};
/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// Real Git.
use crate::transaction_git::GitContext;
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Compute the merge from objects and return its tree and the files the word pass merged,
/// or refuse a conflict.
/// Why:
///   `merge-tree --write-tree` exits 1 for a conflicted merge and prints the tree with the conflicted files' stages;
/// nothing
///  is written to the worktree or the index.
/// The conflicts get the word-level second pass,
/// and the merge continues only when
///  every conflicted file resolved;
/// otherwise it is refused as before,
/// naming the paths.
/// The attributes of the first side's tip apply to the merge itself,
/// and the tree the merge produced to the second pass,
/// because no worktree is read.
/// A conflict with a clean index is not computed here at all:
/// native Git writes
///  the markers for the person,
/// and the second pass then runs on the stopped merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeTree({ session, shape, ends, places, stagedWork, carrying }): Promise<{ tree: string; merged: Merged[] }>;
/// ```
pub fn merge_tree(
    session: &Session,
    shape: &MergeShape,
    ends: (&str, &str, &str),
    places: (&Path, &Path, &[(OsString, OsString)]),
    staged_work: bool,
    carrying: bool,
) -> Result<(String, Vec<Merged>), Bail> {
    return match crate::lazy_runtime::run_on_runtime(merge_tree_awaiting(session, shape, ends, places, staged_work, carrying)) {
        Ok(found) => found,
        Err(refusal) => Err(Bail::from(format!("{refusal}"))),
    };
}

/// What:
///  The same merge,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The second pass over the merge's conflicts is a task's work
/// (`merge_second_pass_git_runner.rs`):
/// every start it makes is awaited,
/// so the merge from objects is one entry of the runtime for a synchronous caller
/// and no entry at all for a task.
/// The steps,
/// their order and the refusal they hand back are the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { tree, merged } = await mergeTreeOnTheRuntime({ session, shape, ends, places, stagedWork });
/// ```
pub async fn merge_tree_awaiting(
    session: &Session,
    shape: &MergeShape,
    ends: (&str, &str, &str),
    places: (&Path, &Path, &[(OsString, OsString)]),
    staged_work: bool,
    carrying: bool,
) -> Result<(String, Vec<Merged>), Bail> {
    let (head, target, reference): (&str, &str, &str) = ends;
    let (common_dir, root, environment): (&Path, &Path, &[(OsString, OsString)]) = places;
    let mut arguments: Vec<&str> = vec!["merge-tree", "--write-tree", "-z", "--no-messages"];
    if shape.allow_unrelated_histories {
        arguments.push("--allow-unrelated-histories");
    }
    arguments.push(head);
    arguments.push(target);
    let probing: ContextRunner<'_> = ContextRunner { context: &session.context, cwd: Path::new("."), without_prefix: false, object_directory: None, environment: Vec::new() };
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(environment, session.context.overlay.as_slice());
    let probe: AttributeProbe<'_> = AttributeProbe { directory: root, global_prefix: session.context.global_prefix.as_slice(), variables: variables.as_slice(), runner: &probing };
    let runner: ContextRunner<'_> = ContextRunner {
        context: &session.context,
        cwd: Path::new("."),
        without_prefix: false,
        object_directory: None,
        environment: attribute_tree_variables(head, &probe).await,
    };
    let merged: GitReply = runner.run(arguments.as_slice(), None, None).await?;
    let (tree, conflicts) = conflicts_of_merge_tree(merged.stdout.as_slice());
    match merged.code {
        Some(0) if !tree.is_empty() => return Ok((tree, Vec::new())),
        Some(0) => return Err(Bail::from(String::from("merge-tree printed no tree"))),
        // A conflict with a clean index and nothing to carry is left to native Git, which writes the markers for the person to
        //  resolve; the second pass then runs on the stopped merge in the worktree. A checkout that holds the person's own work
        //  in the paths this merge changes cannot be handed to native Git, whose merge refuses such a path outright, so the pass
        //  runs here and a conflict no resolution settles is refused with the paths named and nothing changed.
        Some(1) if !staged_work && !carrying => return Err(Bail::Forward(None)),
        Some(1) => {
            // The second pass runs from the top of the worktree with the person's configuration options only.
            let top_context: GitContext = GitContext { global_prefix: config_options(&session.context.global_prefix), ..session.context.clone() };
            let top_runner: ContextRunner<'_> = ContextRunner { context: &top_context, cwd: root, ..runner };
            if let Some(resolved) = resolve_tree(&top_runner, common_dir, tree.as_str(), conflicts.as_slice(), &probe).await {
                return Ok((resolved.tree, resolved.merged));
            }
            let mut paths: Vec<String> = Vec::new();
            for conflict in &conflicts {
                paths.push(path_message_text(conflict.path.as_slice()));
            }
            return Err(refused(2, conflict_finding(reference, paths.as_slice())));
        }
        _ => return Err(Bail::from(format!("git merge-tree failed: {}", String::from_utf8_lossy(&merged.stderr).trim()))),
    }
}
