//! What:
//!  The word-level second pass over the files a Git merge left conflicted:
//! which conflicted files may be re-merged,
//! their
//!  re-merge by `word_merge`,
//! the objects and the tree that carry the result,
//! and the words that list every merged part
//!  on standard error and in the journal.
//! Why:
//!  The owner decided on 2026-11-03 (`doc/planning/cli-git-merge-auto-resolution.md`) that the merges a person runs and the
//!  wrapper's own unattended merges get a second pass that accepts a file only when every part resolves and shows each part.
//!  Four callers need the same answer from the same stage blobs:
//! the forwarded merge,
//! rebase,
//! cherry-pick and revert
//!  (`merge_second_pass_worktree.rs`),
//! the isolated merge,
//! the auto-push reconcile merge and the commit replay merge,
//! which
//!  work from objects (`merge_second_pass_tree.rs`).
//! They differ in how Git is started,
//! so they hand in a `GitRunner`,
//! whose start is awaited:
//! a reconcile merges as a task of the runtime,
//! and a person's own merge reaches the same pass through one synchronous face.
//!  Only a plain file whose three stages exist,
//! with one mode on all three,
//! that is text,
//! small,
//! and not claimed by a merge
//!  driver or the `binary` attribute,
//! is re-merged;
//! everything else is left to the person exactly as Git left it.
//!
//! The limits,
//! the reasons a path is left and what became of it
//! live in `merge_second_pass_verdicts.rs`;
//! the blobs and the tree a merge from objects is carried by live in `merge_second_pass_objects.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await secondPass(runner, conflicts); // [{ merged: { path, mode, bytes, parts } } | { left: { path, why } }]
//! ```

use crate::merge::merge_second_pass_git_lines::merge_with_git_lines;
pub use crate::merge_second_pass_conflicts::{
    Conflict,
    Stage,
    conflicts_of_merge_tree,
};
use crate::merge_second_pass_conflicts::{
    is_nul_byte,
    split_once,
};
pub use crate::merge_second_pass_git_runner::{
    ContextRunner,
    GitReply,
    GitRunner,
    config_options,
};
/// The objects a pass over a merge from objects makes live in a file of their own;
/// named here so every `merge_second_pass::name` path stays the same.
pub use crate::merge_second_pass_objects::{
    resolved_record,
    tree_with_files,
    write_blob,
};
/// The report words and the journal entry live in a file of their own;
/// named here so every `merge_second_pass::name` path stays the same.
pub use crate::merge_second_pass_report::{
    BLOBS_FIELD,
    EXCERPT_CHARACTERS,
    FILE_FIELD,
    INPUTS_HINT,
    OPERATION_FIELD,
    PARTS_FIELD,
    REPORT_MARK,
    journal_entry,
    journal_merged,
    part_text,
    report_lines,
};
use crate::merge_second_pass_verdicts::shared_plain_mode;
/// Whether a stage is binary or too large stays the pass's own question,
/// asked of the file that holds the limits;
/// named here so `merge_isolation_carry_layers.rs` keeps its import path.
pub(crate) use crate::merge_second_pass_verdicts::unfit;
/// What the pass accepts and what it leaves live in a file of their own;
/// named here so every `merge_second_pass::name` path stays the same.
pub use crate::merge_second_pass_verdicts::{
    AttributeSource,
    LARGEST_STAGE_BYTES,
    Left,
    Merged,
    Verdict,
    is_attributes_path,
};
use crate::path_bytes::path_message_text;
use crate::word_merge::WordMerge;
use std::collections::BTreeMap;

/// The values of the `merge` attribute that leave the text merge to Git's own algorithm:
/// none given,
/// `merge` set,
/// `text`.
const TEXT_MERGE_VALUES: &[&str] = &["unspecified", "set", "text"];

/// The fields `git check-attr -z` prints for each answer:
/// the path,
/// the attribute and its value.
const ATTRIBUTE_FIELDS: usize = 3;

/// What:
///  The value of the `merge` attribute of each path,
/// from one `git check-attr` start.
/// Why:
///   A path with a driver,
/// with `-merge` or with `merge=binary` must be merged the way its setting says,
/// not by a second
///  pass that ignores the setting.
/// `source` decides where the setting is read:
/// the invocation's own source,
/// or the tree the merge
///  produced,
/// whose `--source` outranks `--attr-source` and `GIT_ATTR_SOURCE` in Git 2.56.0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeAttributes(runner, paths: Uint8Array[], source: AttributeSource): Promise<Map<string, string>>;
/// ```
async fn merge_attributes(
    runner: &dyn GitRunner,
    paths: &[&[u8]],
    source: AttributeSource<'_>,
) -> Result<BTreeMap<Vec<u8>, String>, String> {
    if paths.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path);
        input.push(0);
    }
    // The `--source` value is built here and lives until the start below reads it.
    let tree: String = match source {
        AttributeSource::Tree(found) => format!("--source={found}"),
        AttributeSource::Invocation | AttributeSource::Unknown => String::new(),
    };
    let mut arguments: Vec<&str> = vec!["check-attr", "-z"];
    if !tree.is_empty() {
        arguments.push(tree.as_str());
    }
    arguments.extend(["--stdin", "merge"]);
    let reply: GitReply = runner
        .run(arguments.as_slice(), Some(input.as_slice()), None)
        .await?;
    if reply.code != Some(0) {
        return Err(format!(
            "git check-attr failed: {}",
            String::from_utf8_lossy(&reply.stderr).trim()
        ));
    }
    let fields: Vec<&[u8]> = reply.stdout.split(is_nul_byte).collect();
    let mut values: BTreeMap<Vec<u8>, String> = BTreeMap::new();
    for triple in fields.as_chunks::<ATTRIBUTE_FIELDS>().0 {
        values.insert(
            triple[0].to_vec(),
            String::from_utf8_lossy(triple[2]).into_owned(),
        );
    }
    return Ok(values);
}

/// What:
///  The bytes of the named blobs,
/// from one `git cat-file --batch` start.
/// Why:
///   One start reads every stage of every candidate file;
/// the answer for a name is its header line and then its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readBlobs(runner, oids: string[]): Promise<Map<string, Uint8Array>>;
/// ```
async fn read_blobs(
    runner: &dyn GitRunner,
    oids: &[&str],
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut input: Vec<u8> = Vec::new();
    for oid in oids {
        input.extend_from_slice(oid.as_bytes());
        input.push(b'\n');
    }
    let reply: GitReply = runner
        .run(&["cat-file", "--batch"], Some(input.as_slice()), None)
        .await?;
    if reply.code != Some(0) {
        return Err(format!(
            "git cat-file failed: {}",
            String::from_utf8_lossy(&reply.stderr).trim()
        ));
    }
    let mut blobs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut rest: &[u8] = reply.stdout.as_slice();
    for oid in oids {
        let Some((header_bytes, after)) = split_once(rest, b'\n') else {
            return Err(format!("git cat-file ended before {oid}"));
        };
        let header: String = String::from_utf8_lossy(header_bytes).into_owned();
        let words: Vec<&str> = header.split(' ').collect();
        let [name, _kind, size_text] = words.as_slice() else {
            return Err(format!("git cat-file could not read {oid}: {header}"));
        };
        let Ok(size) = size_text.parse::<usize>() else {
            return Err(format!(
                "git cat-file printed an unreadable size for {oid}: {header}"
            ));
        };
        let Some(content) = after.get(..size) else {
            return Err(format!("git cat-file ended inside {oid}"));
        };
        blobs.insert((*name).to_owned(), content.to_vec());
        rest = after.get(size + 1..).unwrap_or_default();
    }
    return Ok(blobs);
}

/// What:
///  Re-merge every conflicted path that may be,
/// and say what became of each.
/// Why:
///   A path is merged only when all three stages exist with one plain mode,
/// are small text,
/// and the `merge` attribute
///  that `source` names leaves the text merge to Git;
/// and only when every line region resolves.
/// Anything else is `Left`,
///  so the caller keeps the conflict.
/// `Err` means a Git start failed;
/// the caller then leaves every conflict as Git left it.
///   A merge whose own `.gitattributes` is conflicted reads as `Unknown`:
/// no rule of it can be trusted,
///  so every candidate keeps its conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function secondPass(runner: GitRunner, conflicts: Conflict[], source: AttributeSource): Promise<Verdict[]>;
/// ```
pub async fn second_pass(
    runner: &dyn GitRunner,
    conflicts: &[Conflict],
    source: AttributeSource<'_>,
) -> Result<Vec<Verdict>, String> {
    let mut verdicts: BTreeMap<usize, Verdict> = BTreeMap::new();
    let mut candidates: Vec<(usize, u32)> = Vec::new();
    for (index, conflict) in conflicts.iter().enumerate() {
        let (Some(base), Some(ours), Some(theirs)) =
            (&conflict.base, &conflict.ours, &conflict.theirs)
        else {
            verdicts.insert(
                index,
                Verdict::Left {
                    path: conflict.path.clone(),
                    why: Left::MissingStage,
                },
            );
            continue;
        };
        match shared_plain_mode([base, ours, theirs]) {
            Some(mode) => candidates.push((index, mode)),
            None => {
                verdicts.insert(
                    index,
                    Verdict::Left {
                        path: conflict.path.clone(),
                        why: Left::NotAPlainFile,
                    },
                );
            }
        }
    }
    if source == AttributeSource::Unknown {
        for (index, _) in &candidates {
            verdicts.insert(
                *index,
                Verdict::Left {
                    path: conflicts[*index].path.clone(),
                    why: Left::UnknownAttributes,
                },
            );
        }
        return Ok(verdicts.into_values().collect());
    }
    let mut candidate_paths: Vec<&[u8]> = Vec::new();
    for (index, _) in &candidates {
        candidate_paths.push(conflicts[*index].path.as_slice());
    }
    let attributes: BTreeMap<Vec<u8>, String> =
        merge_attributes(runner, candidate_paths.as_slice(), source).await?;
    let mut wanted: Vec<&str> = Vec::new();
    for (index, _) in &candidates {
        let conflict: &Conflict = &conflicts[*index];
        for stage in [&conflict.base, &conflict.ours, &conflict.theirs]
            .into_iter()
            .flatten()
        {
            if !wanted.contains(&stage.oid.as_str()) {
                wanted.push(stage.oid.as_str());
            }
        }
    }
    let blobs: BTreeMap<String, Vec<u8>> = if wanted.is_empty() {
        BTreeMap::new()
    } else {
        read_blobs(runner, wanted.as_slice()).await?
    };
    for (index, mode) in candidates {
        let conflict: &Conflict = &conflicts[index];
        if let Some(value) = attributes.get(&conflict.path)
            && !TEXT_MERGE_VALUES.contains(&value.as_str())
        {
            verdicts.insert(
                index,
                Verdict::Left {
                    path: conflict.path.clone(),
                    why: Left::Driver(value.clone()),
                },
            );
            continue;
        }
        let mut sides: Vec<&Vec<u8>> = Vec::new();
        for stage in [&conflict.base, &conflict.ours, &conflict.theirs] {
            if let Some(found) = stage.as_ref()
                && let Some(bytes) = blobs.get(&found.oid)
            {
                sides.push(bytes);
            }
        }
        let [base, ours, theirs] = sides.as_slice() else {
            return Err(format!(
                "a stage of {} was not read",
                path_message_text(conflict.path.as_slice())
            ));
        };
        let mut reason: Option<Left> = None;
        for bytes in [base, ours, theirs] {
            reason = unfit(bytes);
            if reason.is_some() {
                break;
            }
        }
        let verdict: Verdict = match reason {
            Some(why) => Verdict::Left {
                path: conflict.path.clone(),
                why,
            },
            None => match merge_with_git_lines(
                runner,
                conflict,
                [base.as_slice(), ours.as_slice(), theirs.as_slice()],
            )
            .await?
            {
                WordMerge::Resolved {
                    merged,
                    parts,
                } => Verdict::Merged(Box::new(Merged {
                    path: conflict.path.clone(),
                    mode,
                    bytes: merged,
                    parts,
                    stages: conflict.clone(),
                })),
                WordMerge::Conflict {
                    unresolved,
                } => Verdict::Left {
                    path: conflict.path.clone(),
                    why: Left::Overlap(unresolved),
                },
            },
        };
        verdicts.insert(index, verdict);
    }
    return Ok(verdicts.into_values().collect());
}

/// Controls for the parsing,
/// the eligibility rules,
/// the objects and the report words.
#[cfg(test)]
#[path = "merge_second_pass_tests.rs"]
pub(crate) mod tests;
