//! What:
//!  The index step of the snapshot rung:
//!  compare the copy with the source's index (settling by content the entries its stat data cannot
//!  vouch for),
//!  install it in the new worktree,
//!  or keep the snapshot's own index when the source changed under the copy.
//! Why:
//!  Every tracked file of the copy must equal the entry the new worktree will carry,
//! and an entry that cannot be proven
//!  that way is either settled by content or,
//!  for a main checkout whose own index the copy carries,
//!  left to the restore that follows.
//! The index an entry was not proven for keeps the stat fields its own index recorded,
//! never the copy's:
//!  fields
//! taken from the file would vouch for bytes no walk proved,
//! and Git would then take an edit its own check cannot see for
//!  the entry's content.
//! The code sits in a file of its own so `worktree_snapshot.rs` stays under the line limit,
//! and that file re-exports what its caller needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = installIndex({ source, stage, admin, ownIndex, environment }); // 'agrees' | 'changed-under-copy'
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::fs_beneath::{open_directory_at, open_file_at, open_root};
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index::{StatFields, StatLookup};
use crate::subvolume_index_files::install_index_in;
use crate::worktree_clean::{ContentTarget, StatEntry, stored_stats, tree_fields_by_content};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

/// What:
///  What the snapshot's index step settled about the copy.
/// Why:
///   A clean source's index is compared with the snapshot;
/// a source that changed under the copy keeps the snapshot
///  and is
/// restored to the new `HEAD` instead of leaving the worktree to Git's own checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyState = 'agrees' | 'changed-under-copy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyState {
    /// Every tracked file of the snapshot equals the index the new worktree received.
    Agrees,
    /// The source changed while it was copied,
    /// so the snapshot's own index was installed and the copy must be
    ///  restored to the new `HEAD`.
    ChangedUnderCopy,
}

/// What:
///  Whether the snapshot carries the index a restore starts from:
/// the main checkout's own index,
/// which lies inside its
///  directory.
/// Why:
///   A linked worktree's index lies outside its directory and is not in the snapshot;
/// the clone rung and the Git rung never
///  look at one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// source.gitDir === source.commonDir && source.gitDir === join(source.root, '.git')
/// ```
fn own_index_in_reach(source: &SnapshotSource) -> bool {
    return source.git_dir == source.common_dir && source.git_dir == source.root.join(".git");
}

/// What:
///  Check the snapshot against the source's index,
/// then install that index in the new worktree's administrative
///  directory with stat fields refreshed and the accelerator extensions of the source dropped;
/// a source that changed in content keeps the
///  snapshot and its own index when that index is in reach.
/// Why:
///   Every tracked file of the snapshot must equal its entry,
/// which proves the snapshot and the index describe the same
///  state whatever the source did meanwhile.
/// An entry the stat data cannot vouch for is settled by content,
/// as the conversion
///  settles one.
/// A file that really changed ends Git's own checkout for a linked worktree's snapshot;
/// a main checkout's
///  snapshot is restored to the new `HEAD` from its own index instead,
/// which is one instant with the copied files.
/// The
///  extensions (`UNTR`,
/// `FSMN`) name the source's worktree.
/// What:
///  The walk of one copy against a source's index:
/// the entries that index names,
/// and the stat fields the copy's files
///  have now,
/// by path.
/// Why:
///   The same walk answers two questions:
/// whether a snapshot a service made ahead is still the source's
///  (`worktree_snapshot_ahead.rs`),
/// and what the new worktree's index records for a copy this add made itself.
/// One
///  walk answers both,
/// so a copy that was proven before it was claimed is never walked twice,
/// and the fields
///  installed are the ones the verdict was given on.
#[derive(Debug)]
pub struct Proof {
    /// The entries of the source's index,
    /// in its own order.
    pub entries: Vec<StatEntry>,
    /// The stat fields of the copy's files,
    /// one per entry.
    pub fields: Vec<StatFields>,
}

/// What:
///  Walk the copy at `root` against the source's index and answer the proof,
/// or the reason the copy is not the
///  source's.
/// Why:
///   Every tracked file of the copy must equal its entry.
/// An entry the stat data cannot vouch for is settled by
///  content,
/// as the conversion settles one;
/// a file that really differs ends the walk at once,
/// which is what makes
///  proving a stale copy cheap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await proveTheCopy({ source, root, environment }); // the entries and the fields, or the reason they are not the same state
/// ```
pub(crate) fn prove_the_copy(source: &SnapshotSource, root: &Path, environment: &ConversionEnvironment) -> Result<Proof, String> {
    let format: ObjectFormat = if source.clean.hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let seed: &[u8] = source.clean.index_bytes.as_slice();
    let Some(entries) = stored_stats(seed, source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let Some(indexed) = parse_index(seed, format) else {
        return Err(String::from("the source's index has a layout the reader does not know"));
    };
    let target: ContentTarget<'_> = ContentTarget { root, git_dir: source.git_dir.as_path(), common_dir: source.common_dir.as_path(), environment, format };
    return match tree_fields_by_content(&target, entries.as_slice(), indexed.as_slice(), source.clean.index_seconds) {
        Ok(fields) => Ok(Proof { entries, fields }),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  Check the snapshot against the source's index,
/// then install that index in the new worktree's administrative
///  directory with the stat fields the walk proved and the accelerator extensions of the source dropped;
/// a source that changed in content keeps the
///  snapshot and its own index when that index is in reach.
/// Why:
///   Every tracked file of the snapshot must equal its entry,
/// which proves the snapshot and the index describe the same
///  state whatever the source did meanwhile.
/// An entry the stat data cannot vouch for is settled by content,
/// as the conversion
///  settles one.
/// A file that really changed ends Git's own checkout for a linked worktree's snapshot;
/// a main checkout's
///  snapshot is restored to the new `HEAD` from its own index instead,
/// which is one instant with the copied files.
/// The
///  extensions (`UNTR`,
/// `FSMN`) name the source's worktree.
/// A copy that was already proven equal to the index (`proven`) is not walked
///  again:
/// the fields it was given on are the ones the new index records.
/// An entry no walk covered keeps the fields its own index recorded,
/// so Git hashes the file instead of trusting
///  fields taken from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await installIndex({ source, stage, admin, ownIndex, environment, proven }); // the state of the copy
/// ```
pub(crate) fn install_index(source: &SnapshotSource, stage: &Path, admin: &File, own_index: Option<&[u8]>, environment: &ConversionEnvironment, proven: Option<Proof>) -> Result<CopyState, String> {
    let format: ObjectFormat = if source.clean.hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    // A clean source proved its index equal to its files, so it is compared with the snapshot. A source with changes has no such
    //  proof: the snapshot's own index, one instant with its files, is installed as it is and the restore settles the rest.
    let mut seed: &[u8] = match own_index {
        Some(bytes) => bytes,
        None => source.clean.index_bytes.as_slice(),
    };
    let mut state: CopyState = CopyState::Agrees;
    let mut restored: Option<Vec<u8>> = None;
    let mut entries: Vec<StatEntry> = Vec::new();
    let mut walked: Vec<StatFields> = Vec::new();
    if own_index.is_none() {
        match proven {
            Some(proof) => {
                entries = proof.entries;
                walked = proof.fields;
            }
            None => {
                let Some(stats) = stored_stats(seed, source.clean.hash_length) else {
                    return Err(String::from("the source's index changed shape"));
                };
                let Some(indexed) = parse_index(seed, format) else {
                    return Err(String::from("the source's index has a layout the reader does not know"));
                };
                let target: ContentTarget<'_> = ContentTarget { root: stage, git_dir: source.git_dir.as_path(), common_dir: source.common_dir.as_path(), environment, format };
                // One walk of the snapshot proves it equal to the source's index and returns the stat fields the new index takes: every
                //  file is looked at once, and the fields are the ones the verdict was given on.
                let walk_snapshot_entered: bool = begin_step("walkSnapshot");
                let walk_snapshot_outcome = tree_fields_by_content(&target, stats.as_slice(), indexed.as_slice(), source.clean.index_seconds);
                end_step(walk_snapshot_entered);
                match walk_snapshot_outcome {
                    Ok(fields) => {
                        entries = stats;
                        walked = fields;
                    }
                    Err(refusal) => {
                        if !own_index_in_reach(source) || !refusal.kind.restorable() {
                            return Err(format!("the source changed while it was copied ({})", refusal.reason));
                        }
                        let read_own_index_entered: bool = begin_step("readOwnIndex");
                        let read_own_index_outcome = read_own_index(stage);
                        end_step(read_own_index_entered);
                        let Ok(own) = read_own_index_outcome else {
                            return Err(format!("the source changed while it was copied ({})", refusal.reason));
                        };
                        restored = Some(own);
                        state = CopyState::ChangedUnderCopy;
                    }
                }
            }
        }
    }
    // A copy the source changed under carries the snapshot's own index, which is one instant with its files; the source's
    //  proven index is the seed every other copy installs.
    if let Some(own) = restored.as_deref() {
        seed = own;
    }
    let mut known: std::collections::HashMap<&[u8], StatFields> = std::collections::HashMap::new();
    for (entry, fields) in entries.iter().zip(walked) {
        known.insert(entry.path.as_slice(), fields);
    }
    // An entry the walk did not cover (a source with changes has no walk) keeps the stat fields the index it came from recorded:
    //  refreshed from the file, they would vouch for bytes no walk proved, and Git would trust them over the content the entry
    //  names. The restore settles those entries by content.
    let install_index_file_entered: bool = begin_step("installIndexFile");
    let install_index_file_outcome = install_index_in(admin, seed, &mut WalkedStats { known: &known }, source.clean.hash_length);
    end_step(install_index_file_entered);
    let report: String = install_index_file_outcome?;
    crate::diagnostic_log::debug("takeSnapshot", report.as_str());
    return Ok(state);
}

/// What:
///  The stat fields of the snapshot's files that the walk proved,
/// by path,
/// and none for an entry it did not cover.
/// Why:
///   One walk of the snapshot gave the fields of the files it covered;
/// a source with changes has no walk,
/// so every
///  one of its entries keeps the fields its index recorded and Git hashes its file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WalkedStats implements StatLookup { statNow(path) { return this.known.get(path); } }
/// ```
struct WalkedStats<'a> {
    /// The fields the walk returned,
    /// by path.
    known: &'a std::collections::HashMap<&'a [u8], StatFields>,
}

impl StatLookup for WalkedStats<'_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        return self.known.get(path).copied();
    }
}

/// What:
///  The bytes of the index inside the snapshot's own `.git`,
/// read below handles that follow no link.
/// Why:
///   A source with uncommitted changes has no index that was proven equal to its files;
/// the copy's index and the copy's files
///  are one instant,
/// whatever the source does next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = await readOwnIndex(stage); // throws a reason
/// ```
pub(crate) fn read_own_index(stage: &Path) -> Result<Vec<u8>, String> {
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    let git_dir: File = subject_failure(open_directory_at(&root, OsStr::new(".git")), &"the snapshot", "has no .git directory")?;
    let mut index: File = subject_failure(open_file_at(&git_dir, OsStr::new("index")), &"the snapshot", "has no index")?;
    let mut bytes: Vec<u8> = Vec::new();
    subject_failure(std::io::Read::read_to_end(&mut index, &mut bytes), &"the snapshot's index", "is unreadable")?;
    return Ok(bytes);
}

/// Index-step controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_index_step_tests.rs"]
mod tests;
