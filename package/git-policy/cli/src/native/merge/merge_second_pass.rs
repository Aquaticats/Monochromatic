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
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = await secondPass(runner, conflicts); // [{ merged: { path, mode, bytes, parts } } | { left: { path, why } }]
//! ```

use crate::merge_second_pass_conflicts::{is_nul_byte, split_once};
pub use crate::merge_second_pass_conflicts::{Conflict, Stage, conflicts_of_merge_tree};
pub use crate::merge_second_pass_git_runner::{ContextRunner, GitReply, GitRunner, config_options};
/// The report words and the journal entry live in a file of their own;
/// named here so every `merge_second_pass::name` path stays the same.
pub use crate::merge_second_pass_report::{BLOBS_FIELD, EXCERPT_CHARACTERS, FILE_FIELD, INPUTS_HINT, OPERATION_FIELD, PARTS_FIELD, REPORT_MARK, journal_entry, journal_merged, part_text, report_lines};
use crate::path_bytes::path_message_text;
use crate::word_merge::{MergedPart, WordMerge, merge_by_word};
use std::collections::BTreeMap;
use std::path::Path;

/// The number of leading bytes in which Git looks for a NUL byte to call a file binary:
/// `FIRST_FEW_BYTES` in Git 2.56.0's
///  `xdiff-interface.c` line 197,
/// used by `buffer_is_binary` and by the merge driver choice in `merge-ll.c` lines 120 to 122.
const BINARY_PROBE_BYTES: usize = 8000;

/// The size above which Git's own merge treats a file as binary and does not merge it by text:
/// `MAX_XDIFF_SIZE`,
/// 1023 MiB,
/// in
///  `xdiff-interface.h` line 14,
/// tested in `merge-ll.c` lines 117 to 119.
const GIT_MAX_XDIFF_SIZE: usize = 1024 * 1024 * 1023;

/// The size above which this pass leaves a file to the person,
/// although Git would still merge it by text.
/// Git's value cannot
///  bound the cost of this pass:
/// measured on three texts that share no line,
/// one 1 MiB of lines took 0.08 s and 45 MB,
/// 8 MiB took
///  0.73 s and 352 MB,
/// 32 MiB took 4.12 s and 1320 MB (release build,
/// Myers diff,
/// one core),
/// so a file at Git's limit would take
///  minutes and tens of gigabytes.
/// 8 MiB keeps the worst case under a second.
const PASS_COST_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// The largest blob the second pass re-merges:
/// the smaller of Git's own limit and the one measured above.
const LARGEST_BLOB_BYTES: usize = if PASS_COST_LIMIT_BYTES < GIT_MAX_XDIFF_SIZE { PASS_COST_LIMIT_BYTES } else { GIT_MAX_XDIFF_SIZE };

/// The largest stage the pass reads,
/// in the bytes a caller's answer ceiling counts.
/// Why:
///   The pass asks `git cat-file --batch` for the three stages of every candidate and refuses a file past this size,
/// so an answer larger than one
///  of these per name asked for is one the pass would refuse after reading it.
pub const LARGEST_STAGE_BYTES: u64 = LARGEST_BLOB_BYTES as u64;

/// The modes of a plain file,
/// with and without the executable bit.
const PLAIN_FILE_MODES: [u32; 2] = [0o100_644, 0o100_755];

/// The values of the `merge` attribute that leave the text merge to Git's own algorithm:
/// none given,
/// `merge` set,
/// `text`.
const TEXT_MERGE_VALUES: [&str; 3] = ["unspecified", "set", "text"];

/// The fields `git check-attr -z` prints for each answer:
/// the path,
/// the attribute and its value.
const ATTRIBUTE_FIELDS: usize = 3;

/// Why a conflicted path is left to the person.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Left {
    /// A stage is missing:
    /// both sides added the path,
    /// or one side deleted it.
    MissingStage,
    /// The stages differ in mode,
    /// or the file is a link,
    /// a submodule or not a plain file.
    NotAPlainFile,
    /// A stage holds a NUL byte near its start.
    Binary,
    /// A stage is larger than the pass takes.
    TooLarge,
    /// The `merge` attribute names a driver,
    /// `binary` or is unset,
    /// so the merge belongs to that setting.
    Driver(String),
    /// At least one line region holds two different changes of the same words.
    Overlap(usize),
    /// A `.gitattributes` of the merge is itself conflicted,
    /// so no `merge` setting can be read for the path.
    UnknownAttributes,
}

/// Where the pass reads the `merge` attribute of the paths it may merge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeSource<'a> {
    /// Whatever the invocation itself names:
    /// the worktree of a merge a person runs,
    /// or the tree that `--attr-source`,
    /// `GIT_ATTR_SOURCE` or `attr.tree` names.
    Invocation,
    /// The tree the merge produced,
    /// which carries a `merge` rule the merge itself introduces.
    Tree(&'a str),
    /// A `.gitattributes` of the merge is itself conflicted,
    /// so no rule of it can be trusted and nothing may be merged.
    Unknown,
}

/// A conflicted file that the word pass merged completely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merged {
    /// The path as bytes.
    pub path: Vec<u8>,
    /// The mode all three stages share.
    pub mode: u32,
    /// The merged bytes.
    pub bytes: Vec<u8>,
    /// The line regions that needed the word pass.
    pub parts: Vec<MergedPart>,
    /// The three stages the file was merged from,
    /// kept to put the conflict back if a later step fails.
    pub stages: Conflict,
}

/// What became of one conflicted path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Merged completely.
    Merged(Box<Merged>),
    /// Left as Git left it.
    Left {
        /// The path as bytes.
        path: Vec<u8>,
        /// Why.
        why: Left,
    },
}

/// What:
///  Whether the bytes are text the pass may merge:
/// no NUL byte near the start,
/// and not larger than the limit.
/// Why:
///   Git's own binary test looks at the first bytes only;
/// a file it would call binary is never a candidate for a
///  line-and-word merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = (bytes) => bytes.length > LARGEST ? 'too-large' : bytes.subarray(0, PROBE).includes(0) ? 'binary' : 'text';
/// ```
pub(crate) fn unfit(bytes: &[u8]) -> Option<Left> {
    if bytes.len() > LARGEST_BLOB_BYTES {
        return Some(Left::TooLarge);
    }
    if bytes[..bytes.len().min(BINARY_PROBE_BYTES)].contains(&0) {
        return Some(Left::Binary);
    }
    return None;
}

/// The mode all three stages share,
/// when it is a plain file's.
fn shared_plain_mode(stages: [&Stage; 3]) -> Option<u32> {
    let mode: u32 = stages[0].mode;
    for stage in stages {
        if stage.mode != mode {
            return None;
        }
    }
    if PLAIN_FILE_MODES.contains(&mode) {
        return Some(mode);
    }
    return None;
}

/// What:
///  Whether a conflicted path is a `.gitattributes` file.
/// Why:
///   A `.gitattributes` names the rules of its own folder and of everything below it,
/// so one a merge left conflicted can change any rule that bears on a candidate,
/// and a copy of it that holds Git's markers is no side's rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAttributesPath = (path: string) => path === '.gitattributes' || path.endsWith('/.gitattributes');
/// ```
pub fn is_attributes_path(path: &[u8]) -> bool {
    return path == b".gitattributes" || path.ends_with(b"/.gitattributes");
}

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
async fn merge_attributes(runner: &dyn GitRunner, paths: &[&[u8]], source: AttributeSource<'_>) -> Result<BTreeMap<Vec<u8>, String>, String> {
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
    let reply: GitReply = runner.run(arguments.as_slice(), Some(input.as_slice()), None).await?;
    if reply.code != Some(0) {
        return Err(format!("git check-attr failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    let fields: Vec<&[u8]> = reply.stdout.split(is_nul_byte).collect();
    let mut values: BTreeMap<Vec<u8>, String> = BTreeMap::new();
    for triple in fields.as_chunks::<ATTRIBUTE_FIELDS>().0 {
        values.insert(triple[0].to_vec(), String::from_utf8_lossy(triple[2]).into_owned());
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
async fn read_blobs(runner: &dyn GitRunner, oids: &[&str]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut input: Vec<u8> = Vec::new();
    for oid in oids {
        input.extend_from_slice(oid.as_bytes());
        input.push(b'\n');
    }
    let reply: GitReply = runner.run(&["cat-file", "--batch"], Some(input.as_slice()), None).await?;
    if reply.code != Some(0) {
        return Err(format!("git cat-file failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    let mut blobs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut rest: &[u8] = reply.stdout.as_slice();
    for oid in oids {
        let Some((header_bytes, after)) = split_once(rest, b'\n') else { return Err(format!("git cat-file ended before {oid}")) };
        let header: String = String::from_utf8_lossy(header_bytes).into_owned();
        let words: Vec<&str> = header.split(' ').collect();
        let [name, _kind, size_text] = words.as_slice() else { return Err(format!("git cat-file could not read {oid}: {header}")) };
        let Ok(size) = size_text.parse::<usize>() else {
            return Err(format!("git cat-file printed an unreadable size for {oid}: {header}"));
        };
        let Some(content) = after.get(..size) else { return Err(format!("git cat-file ended inside {oid}")) };
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
pub async fn second_pass(runner: &dyn GitRunner, conflicts: &[Conflict], source: AttributeSource<'_>) -> Result<Vec<Verdict>, String> {
    let mut verdicts: BTreeMap<usize, Verdict> = BTreeMap::new();
    let mut candidates: Vec<(usize, u32)> = Vec::new();
    for (index, conflict) in conflicts.iter().enumerate() {
        let (Some(base), Some(ours), Some(theirs)) = (&conflict.base, &conflict.ours, &conflict.theirs) else {
            verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::MissingStage });
            continue;
        };
        match shared_plain_mode([base, ours, theirs]) {
            Some(mode) => candidates.push((index, mode)),
            None => {
                verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::NotAPlainFile });
            }
        }
    }
    if source == AttributeSource::Unknown {
        for (index, _) in &candidates {
            verdicts.insert(*index, Verdict::Left { path: conflicts[*index].path.clone(), why: Left::UnknownAttributes });
        }
        return Ok(verdicts.into_values().collect());
    }
    let mut candidate_paths: Vec<&[u8]> = Vec::new();
    for (index, _) in &candidates {
        candidate_paths.push(conflicts[*index].path.as_slice());
    }
    let attributes: BTreeMap<Vec<u8>, String> = merge_attributes(runner, candidate_paths.as_slice(), source).await?;
    let mut wanted: Vec<&str> = Vec::new();
    for (index, _) in &candidates {
        let conflict: &Conflict = &conflicts[*index];
        for stage in [&conflict.base, &conflict.ours, &conflict.theirs].into_iter().flatten() {
            if !wanted.contains(&stage.oid.as_str()) {
                wanted.push(stage.oid.as_str());
            }
        }
    }
    let blobs: BTreeMap<String, Vec<u8>> = if wanted.is_empty() { BTreeMap::new() } else { read_blobs(runner, wanted.as_slice()).await? };
    for (index, mode) in candidates {
        let conflict: &Conflict = &conflicts[index];
        if let Some(value) = attributes.get(&conflict.path)
            && !TEXT_MERGE_VALUES.contains(&value.as_str())
        {
            verdicts.insert(index, Verdict::Left { path: conflict.path.clone(), why: Left::Driver(value.clone()) });
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
        let [base, ours, theirs] = sides.as_slice() else { return Err(format!("a stage of {} was not read", path_message_text(conflict.path.as_slice()))) };
        let mut reason: Option<Left> = None;
        for bytes in [base, ours, theirs] {
            reason = unfit(bytes);
            if reason.is_some() {
                break;
            }
        }
        let verdict: Verdict = match reason {
            Some(why) => Verdict::Left { path: conflict.path.clone(), why },
            None => match merge_by_word(base, ours, theirs) {
                WordMerge::Resolved { merged, parts } => Verdict::Merged(Box::new(Merged { path: conflict.path.clone(), mode, bytes: merged, parts, stages: conflict.clone() })),
                WordMerge::Conflict { unresolved } => Verdict::Left { path: conflict.path.clone(), why: Left::Overlap(unresolved) },
            },
        };
        verdicts.insert(index, verdict);
    }
    return Ok(verdicts.into_values().collect());
}

/// What:
///  Write one merged file as a blob and return its object name.
/// Why:
///   `--no-filters` keeps the bytes exactly as merged;
/// the checkout filters apply when a worktree file is made,
/// not here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeBlob(runner: GitRunner, bytes: Uint8Array): Promise<string>;
/// ```
pub async fn write_blob(runner: &dyn GitRunner, bytes: &[u8]) -> Result<String, String> {
    let reply: GitReply = runner.run(&["hash-object", "-w", "--no-filters", "--stdin"], Some(bytes), None).await?;
    if reply.code != Some(0) {
        return Err(format!("git hash-object failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
    }
    return Ok(String::from_utf8_lossy(&reply.stdout).trim().to_owned());
}

/// The `update-index --index-info` record that puts a blob at a path as a resolved entry.
pub fn resolved_record(mode: u32, oid: &str, path: &[u8]) -> Vec<u8> {
    let mut record: Vec<u8> = format!("{mode:06o} {oid}\t").into_bytes();
    record.extend_from_slice(path);
    record.push(0);
    return record;
}

/// What:
///  The tree that is `tree` with the named files replaced,
/// from a scratch index.
/// Why:
///   The tree `git merge-tree` wrote holds the conflicted file with markers;
/// replacing it by the merged blob makes the tree
///  of the merge as if Git had merged cleanly.
/// A scratch index is the way to edit a tree with Git's own commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function treeWithFiles(runner, tree: string, files: { path, mode, oid }[], scratchIndex: string): Promise<string>;
/// ```
pub async fn tree_with_files(runner: &dyn GitRunner, tree: &str, files: &[(&[u8], u32, &str)], scratch_index: &Path) -> Result<String, String> {
    let read: GitReply = runner.run(&["read-tree", tree], None, Some(scratch_index)).await?;
    if read.code != Some(0) {
        return Err(format!("git read-tree failed: {}", String::from_utf8_lossy(&read.stderr).trim()));
    }
    let mut records: Vec<u8> = Vec::new();
    for (path, mode, oid) in files {
        records.extend_from_slice(resolved_record(*mode, oid, path).as_slice());
    }
    let updated: GitReply = runner.run(&["update-index", "-z", "--index-info"], Some(records.as_slice()), Some(scratch_index)).await?;
    if updated.code != Some(0) {
        return Err(format!("git update-index failed: {}", String::from_utf8_lossy(&updated.stderr).trim()));
    }
    let written: GitReply = runner.run(&["write-tree"], None, Some(scratch_index)).await?;
    if written.code != Some(0) {
        return Err(format!("git write-tree failed: {}", String::from_utf8_lossy(&written.stderr).trim()));
    }
    return Ok(String::from_utf8_lossy(&written.stdout).trim().to_owned());
}

/// Controls for the parsing,
/// the eligibility rules,
/// the objects and the report words.
#[cfg(test)]
#[path = "merge_second_pass_tests.rs"]
pub(crate) mod tests;
