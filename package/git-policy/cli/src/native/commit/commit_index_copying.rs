//! What:
//!  The decision of whether an index may seed an explicit-path commit,
//! and the copy a reset to a tree follows.
//! Why:
//!  `commit_index.rs` initializes the private commit index for each selection mode and stays under the file-size
//!  limit;
//! the seed decision and the reset copy need nothing from it,
//! so they live in a file of their own that it imports.
//! The one read every private copy is written from,
//! and the failure a read or a copy reports,
//! live beside them in `commit_index_bytes.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seed = seedOfIndexBytes(index.bytes()); await writeIndexForReset({ index, destination, seed });
//! ```

/// What marks an index carries.
use crate::index_flags::{IndexFlags, read_index_flags};
/// The bytes one read of an index returned,
/// and the copy a reset follows.
use crate::commit_index_bytes::{IndexBytes, copy_index_failure};
/// The request built against a private index.
use crate::private_index_request::indexed_request;
/// The failure a step of the commit reports.
use crate::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  How the private index of a commit is built from the captured real index.
/// Why:
///   Two marks stop the copy and one does not:
///  an assume-unchanged or skip-worktree entry makes `git add --all` skip or refuse a file the commit named,
///  where native Git stages it,
///  while an intent-to-add entry carries no stat data and copies as it is.
///  The copy is then reset to the tree the commit starts from,
/// as native Git's own partial commit does:
/// the reset leaves a mark whose mode and object name match the tree's entry,
///  replaces one that differs,
///  and drops one the tree does not hold,
///  and the copy's marked paths are cleared before it (`write_index_for_reset`),
///  so the commit keeps the real index's cached stat data
///  and the tree it writes holds every path the tree holds
/// (`commit_index.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateIndexSeed = 'copy' | 'copy-with-intent-to-add' | 'base';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivateIndexSeed {
    /// Copy the real index as it is.
    Copy,
    /// Copy the real index as it is,
    /// marks included:
    /// its intent-to-add entries are selections a later step stages,
    /// so no early return may skip them,
    /// and these are the paths they name,
    /// which the copy a reset follows clears before it (`write_index_for_reset`).
    CopyWithIntentToAdd(Vec<Vec<u8>>),
    /// Build the private index from the base alone.
    Base,
}

/// What:
///  Whether a build seeded this way keeps the cached stat data of the entries its tree shares with the real index.
/// Why:
///  The commit's timing record states this fact,
/// and only a build that starts from a copy keeps any.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// seed !== 'base'
/// ```
impl PrivateIndexSeed {
    /// The answer the timing record carries for a build seeded this way.
    pub fn keeps_stat_data(&self) -> bool {
        return !matches!(self, PrivateIndexSeed::Base);
    }
}

/// What:
///  Which way an index's bytes may seed the private index of an explicit-path commit.
/// Why:
///   Only bytes whose every entry was read and that carry no assume-unchanged or skip-worktree mark may seed it,
/// and bytes this crate cannot read are treated like marked ones.
///  An index whose only marks are intent-to-add entries seeds it too,
///  and is copied as it is:
/// its marks carry no stat data to lose,
///  and the copy a reset follows clears them,
/// so the reset puts the tree's own entry back.
///  The marks are read from the bytes one read of the index returned,
///  never from a copy of it on disk,
///  so the answer describes the state every private copy of the commit holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// seedOfIndexBytes(indexBytes)
/// ```
pub fn seed_of_index_bytes(bytes: &[u8]) -> PrivateIndexSeed {
    // Finding a mark walks every entry, and an index of a large worktree holds tens of thousands, so the walk is named.
    let began: std::time::Instant = std::time::Instant::now();
    let seed: PrivateIndexSeed = seed_of_flags(read_index_flags(bytes));
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::INDEX_SEED, began.elapsed());
    return seed;
}

/// What:
///  `seed_of_index_bytes` without the note of how long the walk took.
/// Why:
///   The note covers the walk that found nothing as well as the one that found a mark.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// seedOfFlags(readIndexFlags(bytes))
/// ```
fn seed_of_flags(flags: IndexFlags) -> PrivateIndexSeed {
    return match flags {
        IndexFlags::Plain => PrivateIndexSeed::Copy,
        IndexFlags::IntentToAdd(paths) => PrivateIndexSeed::CopyWithIntentToAdd(paths),
        IndexFlags::Marked(_) | IndexFlags::Unreadable => PrivateIndexSeed::Base,
    };
}

/// What:
///  Writes one private index that a reset to a tree follows from the bytes one read returned,
///  clearing the intent-to-add marks that reset would keep.
/// Why:
///   A reset leaves an intent-to-add mark whose mode and object name match the tree's own entry,
/// and a tree built from an index holding such a mark leaves that path out once anything invalidates the cached tree:
///  staging a selected path does,
///  and a tracked file nobody named then becomes a deletion in the tree the commit writes.
///  Every way of building the private index from a copy resets it,
///  the first preparation and a rebuild after the branch moved,
///  so both copy here and neither keeps a rule of its own.
///  The copy is written from the bytes the caller already read,
///  which is what keeps it the same state as every other copy of the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await index.writeCopy(commitIndex); if (seed.paths) await dropIntentToAdd(seed.paths);
/// ```
pub fn write_index_for_reset(
    context: &GitContext,
    cwd: &Path,
    index: &IndexBytes,
    destination: &Path,
    object_directory: &Path,
    seed: &PrivateIndexSeed,
    oid_length: usize,
) -> Result<(), GitFailure> {
    index.write_copy(destination).map_err(copy_index_failure)?;
    let PrivateIndexSeed::CopyWithIntentToAdd(paths) = seed else {
        return Ok(());
    };
    return drop_intent_to_add(context, cwd, destination, object_directory, paths.as_slice(), oid_length);
}

/// What:
///  Writes the requested tree's own entry over each given intent-to-add entry,
///  so the index holds that tree and not a mark that leaves the path out.
/// Why:
///   A reset leaves an intent-to-add mark whose mode and object name match the tree's entry,
/// and a tree built from an index holding such a mark leaves that path out:
///  staging a selected path or a scratch-commit hook invalidates the cached tree,
///  and a tracked file nobody named would then become a deletion in the tree the index writes.
///  `update-index --index-info` reads these names as index paths,
///  so a name relative to the repository root keeps that meaning under an invocation started in a subdirectory,
///  and the record names the object with the repository's own hash length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git update-index -z --index-info <<< paths.map((path) => `0 ${'0'.repeat(oidLength)} 0\t${path}\0`).join('')
/// ```
fn drop_intent_to_add(
    context: &GitContext,
    cwd: &Path,
    private_index: &Path,
    object_directory: &Path,
    paths: &[Vec<u8>],
    oid_length: usize,
) -> Result<(), GitFailure> {
    let zero: Vec<u8> = vec![b'0'; oid_length];
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(b"0 ");
        input.extend_from_slice(zero.as_slice());
        input.extend_from_slice(b" 0\t");
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let mut request: GitRequest = indexed_request(
        context,
        cwd,
        &["update-index", "-z", "--index-info"],
        Some(private_index),
        Some(object_directory),
    );
    request.input = Some(input);
    run_git_checked(context, &request)?;
    return Ok(());
}
