//! What:
//!  The text half of replay's subsumption:
//!       which regular-file paths hold prepared bytes that already contain the landed change.
//! Why:
//!  A path is subsumed when the landed change applies in reverse to the prepared bytes with full context,
//!       or when the prepared change of every landed region is the landed change extended on one side.
//!       Hunks come from index-named single-level trees, so no patch header carries a user path,
//!       and binary detection is Git's content rule (a NUL byte in the first 8000 bytes of any of the three blobs).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await subsumedTextPaths({ gitPath, shadowPath, cwd, objectDirectory, candidates });
//! ```

/// Blob batch reader and the index-named tree writer's shadow runs.
use super::blob_batch::load_blob_batch;
/// The one-sided extension and the strict reverse check.
use super::replay_containment::contains_landed_change;
use super::replay_patch::{PatchHunk, parse_indexed_patch, split_keeping_newlines};
use super::replay_reverse_apply::reverse_applies;
use super::replay_shadow::run_shadow_checked;
use super::replay_shared_paths::{SharedPath, TreeEntry};
use super::transaction_git::{GitContext, GitFailure};
/// Hunk maps keyed by entry name.
use std::collections::{BTreeMap, HashMap};
/// `Path` names the shadow and the object directory.
use std::path::Path;

/// Bytes Git inspects for a NUL when deciding that a blob is binary (`FIRST_FEW_BYTES`).
const BINARY_PROBE_BYTES: usize = 8_000;

/// Context lines of the strict reverse check, Git's default.
const STRICT_CONTEXT: u32 = 3;

/// What:
///  The text check's failure:
///       a Git command failed, a blob could not be read, or a patch had an unexpected shape.
/// Why:
///   Replay reports each as `transaction-failed`, never as a silent "not subsumed".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextSubsumptionError {
    /// A Git command failed.
    Git(GitFailure),
    /// A blob batch could not be read.
    Blob(String),
    /// A patch did not parse.
    Format(String),
}

/// What:
///  Whether a blob is binary by Git's rule:
///       a NUL byte among its first 8000 bytes.
/// Why:
///   Attributes never decide binary-ness here, so a `text` attribute cannot make a binary blob a text candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isBinary(bytes); // bytes.subarray(0, 8000).includes(0)
/// ```
fn is_binary(bytes: &[u8]) -> bool {
    return bytes[..bytes.len().min(BINARY_PROBE_BYTES)].contains(&0);
}

/// What:
///  Writes the empty blob when an addition on both sides needs an empty base; returns its ID,
///       or an empty string when no candidate needs it.
/// Why:
///   An addition on both sides is read against an empty base, so the empty blob is the base entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await emptyBlobFor({ gitPath, shadowPath, candidates });
/// ```
fn empty_blob_for(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    candidates: &[SharedPath],
) -> Result<String, TextSubsumptionError> {
    if !candidates.iter().any(|shared| { return shared.base.is_none(); }) {
        return Ok(String::new());
    }
    let output = run_shadow_checked(
        context,
        shadow,
        cwd,
        &["hash-object", "-w", "-t", "blob", "--stdin"],
        Some(&[]),
        None,
        &[],
    )
    .map_err(TextSubsumptionError::Git)?;
    return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
}

/// What:
///  The object ID a side contributes to the index-named trees:
///       its blob, or the empty blob for an absent base.
/// Why:
///   Index-named trees need one blob per candidate per side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sideOid({ entry, emptyBlob });
/// ```
fn side_oid(entry: &Option<TreeEntry>, empty_blob: &str) -> String {
    return match entry {
        Some(present) => present.oid.clone(),
        None => String::from(empty_blob),
    };
}

/// What:
///  Writes a single-level tree of regular-file entries named by their position.
/// Why:
///   Named by index, the tree's paths never carry a user path into the patch headers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await indexedTree({ gitPath, shadowPath, oids });
/// ```
fn indexed_tree(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    oids: &[String],
) -> Result<String, TextSubsumptionError> {
    // `mut` lets the records be appended in index order.
    let mut input: Vec<u8> = Vec::new();
    for (index, oid) in oids.iter().enumerate() {
        input.extend_from_slice(format!("100644 blob {oid}\t{index}\0").as_bytes());
    }
    let output = run_shadow_checked(context, shadow, cwd, &["mktree", "-z"], Some(&input), None, &[])
        .map_err(TextSubsumptionError::Git)?;
    return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
}

/// What:
///  Hunks per entry name between two index-named trees, with the given context lines.
/// Why:
///   Full-context hunks feed the strict check; zero-context hunks feed the containment check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await indexedHunks({ gitPath, shadowPath, from, to, context });
/// ```
fn indexed_hunks(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    from: &str,
    to: &str,
    lines_of_context: u32,
) -> Result<BTreeMap<Vec<u8>, Vec<PatchHunk>>, TextSubsumptionError> {
    let unified: String = format!("--unified={lines_of_context}");
    let output = run_shadow_checked(
        context,
        shadow,
        cwd,
        &[
            "diff-tree",
            "-p",
            "--text",
            unified.as_str(),
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            from,
            to,
        ],
        None,
        None,
        &[],
    )
    .map_err(TextSubsumptionError::Git)?;
    return parse_indexed_patch(&output.stdout)
        .map_err(|error| { return TextSubsumptionError::Format(error.0); });
}

/// What:
///  Decides the text candidates:
///       the paths whose prepared bytes already contain the landed change.
/// Why:
///   Only candidates whose three blobs are all text reach this check; the caller has already removed identical entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await subsumedTextPaths({ gitPath, shadowPath, cwd, objectDirectory, candidates });
/// ```
pub fn subsumed_text_paths(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    object_directory: &Path,
    candidates: &[SharedPath],
) -> Result<Vec<String>, TextSubsumptionError> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let empty_blob: String = empty_blob_for(context, shadow, cwd, candidates)?;
    // Every present blob of every candidate, in candidate order.
    let oids: Vec<String> = candidates
        .iter()
        .flat_map(|shared| { return [&shared.base, &shared.landed, &shared.prepared]; })
        .filter_map(|entry| { return entry.as_ref().map(|present| { return present.oid.clone(); }); })
        .collect();
    let blobs: HashMap<String, Vec<u8>> = load_blob_batch(context, cwd, &oids, Some(object_directory))
        .map_err(TextSubsumptionError::Blob)?;
    let blob_of = |entry: &Option<TreeEntry>| -> Option<Vec<u8>> {
        return entry.as_ref().map(|present| { return blobs.get(&present.oid).cloned().unwrap_or_default(); });
    };
    // Candidates whose present blobs are all text; a blob the batch did not return counts as binary.
    let text: Vec<&SharedPath> = candidates
        .iter()
        .filter(|shared| {
            return [&shared.base, &shared.landed, &shared.prepared].iter().all(|entry| {
                let blob = entry.as_ref().map(|present| { return blobs.get(&present.oid); });
                return match blob {
                    None => true,
                    Some(Some(bytes)) => !is_binary(bytes),
                    Some(None) => false,
                };
            });
        })
        .collect();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let owned: Vec<SharedPath> = text.iter().map(|shared| { return (*shared).clone(); }).collect();
    let base_tree: String = indexed_tree(
        context,
        shadow,
        cwd,
        &owned.iter().map(|shared| { return side_oid(&shared.base, &empty_blob); }).collect::<Vec<String>>(),
    )?;
    let landed_tree: String = indexed_tree(
        context,
        shadow,
        cwd,
        &owned.iter().map(|shared| { return side_oid(&shared.landed, &empty_blob); }).collect::<Vec<String>>(),
    )?;
    let prepared_tree: String = indexed_tree(
        context,
        shadow,
        cwd,
        &owned.iter().map(|shared| { return side_oid(&shared.prepared, &empty_blob); }).collect::<Vec<String>>(),
    )?;
    let strict: BTreeMap<Vec<u8>, Vec<PatchHunk>> =
        indexed_hunks(context, shadow, cwd, &base_tree, &landed_tree, STRICT_CONTEXT)?;
    let landed_zero: BTreeMap<Vec<u8>, Vec<PatchHunk>> =
        indexed_hunks(context, shadow, cwd, &base_tree, &landed_tree, 0)?;
    let prepared_zero: BTreeMap<Vec<u8>, Vec<PatchHunk>> =
        indexed_hunks(context, shadow, cwd, &base_tree, &prepared_tree, 0)?;
    // `mut` lets the subsumed paths be collected in candidate order.
    let mut subsumed: Vec<String> = Vec::new();
    for (index, shared) in owned.iter().enumerate() {
        let name: Vec<u8> = index.to_string().into_bytes();
        let prepared_bytes: Vec<u8> = blob_of(&shared.prepared).unwrap_or_default();
        let base_bytes: Vec<u8> = blob_of(&shared.base).unwrap_or_default();
        let strict_hunks: &[PatchHunk] = strict.get(&name).map(Vec::as_slice).unwrap_or_default();
        let landed_hunks: &[PatchHunk] = landed_zero.get(&name).map(Vec::as_slice).unwrap_or_default();
        let prepared_hunks: &[PatchHunk] = prepared_zero.get(&name).map(Vec::as_slice).unwrap_or_default();
        let base_lines: Vec<Vec<u8>> = split_keeping_newlines(&base_bytes);
        if reverse_applies(strict_hunks, &prepared_bytes)
            || contains_landed_change(landed_hunks, prepared_hunks, &base_lines)
        {
            subsumed.push(shared.path.clone());
        }
    }
    return Ok(subsumed);
}

#[cfg(test)]
#[path = "replay_subsumption_text_tests.rs"]
mod tests;
