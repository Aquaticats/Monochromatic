//! What:
//!  The one Git run that writes the paths a fence leaves to Git:
//! the paths the destination's index still holds,
//! a zeroing of their entries,
//! and the `git checkout-index` that writes them with the destination's own conversion.
//! Why:
//!  Git owns the conversion,
//! so asking it for these bytes is the only answer that needs no model of a filter,
//! an encoding or an ending.
//! `git checkout-index` writes a path only when its entry's stat data no longer matches the file,
//! so a copied or snapshotted file that is the file the entry recorded is zeroed first,
//! and only the paths the destination's index holds are sent,
//! because one it lacks would be injected into the index and then fail the whole run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeThroughGit(admin, destination, context, paths, hashLength); // throws the reason
//! checkoutIndexThroughGit(admin, destination, context, paths, hashLength);
//! ```

use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
use crate::worktree_clone_conversion_decline::{
    MODE_SYMLINK,
    MODE_TYPE_MASK,
};
use crate::worktree_clone_conversion_git_reads::{
    run_in,
    run_reason,
    words,
};
use crate::worktree_ladder_target::{
    checkout_environment,
    checkout_environment_with_index,
};
use std::collections::HashSet;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The paths of the index at `index`,
///  as the set a write step intersects a fence's answer with.
/// Why:
///   The tree difference can remove a path the new `HEAD` lacks,
/// and `git checkout-index` fails the whole rung on a path its index does not hold
///  ("is not in the cache"),
///  so both write steps ask the index they are pointed at which of the fenced paths are still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsTheIndexHolds(index, hashLength): Set<string>; // throws the reason
/// ```
fn paths_the_index_holds(
    index: &Path,
    hash_length: usize,
) -> Result<HashSet<Vec<u8>>, String> {
    let bytes: Vec<u8> = std::fs::read(index)
        .map_err(|error| return format!("the new worktree's index cannot be read ({error})"))?;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let Some(entries) = parse_index(bytes.as_slice(), format) else {
        return Err(String::from(
            "the new worktree's index has a layout the reader does not know",
        ));
    };
    return Ok(entries
        .iter()
        .map(|entry| return entry.path.clone())
        .collect());
}

/// What:
///  Zero the instance data of the index entries for `paths`,
/// with one `git update-index -z --index-info`,
/// so the entry no longer vouches for whatever the rung put in the work tree,
/// and answer the paths the destination's index actually holds.
/// Why:
///   `git checkout-index` in place writes a path only when its entry's stat data no longer matches the file
/// (`entry.c`,
///  `checkout_entry_ca`,
/// `if (!changed) return 0;`),
/// and both rungs can put bytes in the work tree that the entry's own fields still describe:
/// the clone rung installs the source's index over a copy made here,
/// and a btrfs snapshot keeps the source's inode and times,
/// so the copied file *is* the file the source's index recorded.
/// A zeroed entry matches nothing (`read-tree` leaves entries this way),
/// so Git rewrites it and the first status hashes it once and finds it clean.
/// The intersection is a correctness matter,
/// not a saving:
/// `git update-index --index-info` *adds* an entry a path lacks,
/// so a fenced path the new `HEAD` does not carry would be injected into the destination's index,
/// and `git checkout-index` would then fail the whole rung ("is not in the cache").
/// This is option A′'s write step;
///  the fence beside the clone installs its zeroes with the index itself and needs only the checkout below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function zeroTheEntries(admin, destination, context, paths, hashLength); // throws the reason
/// ```
fn zero_the_entries(
    admin: &Path,
    destination: &Path,
    context: &GitContext,
    paths: &[Vec<u8>],
    hash_length: usize,
) -> Result<Vec<Vec<u8>>, String> {
    let index_path: PathBuf = admin.join("index");
    let bytes: Vec<u8> = std::fs::read(index_path.as_path())
        .map_err(|error| return format!("the new worktree's index cannot be read ({error})"))?;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let Some(entries) = parse_index(bytes.as_slice(), format) else {
        return Err(String::from(
            "the new worktree's index has a layout the reader does not know",
        ));
    };
    let wanted: HashSet<&[u8]> = paths.iter().map(|path| return path.as_slice()).collect();
    let mut input: Vec<u8> = Vec::new();
    let mut zeroed: Vec<Vec<u8>> = Vec::new();
    for entry in &entries {
        if !wanted.contains(entry.path.as_slice()) || entry.mode & MODE_TYPE_MASK == MODE_SYMLINK {
            continue;
        }
        let object: String = crate::repo_read_object::to_hex(entry.id.as_slice());
        input.extend_from_slice(format!("{:o} {object}\t", entry.mode).as_bytes());
        input.extend_from_slice(entry.path.as_slice());
        input.push(0);
        zeroed.push(entry.path.clone());
    }
    if input.is_empty() {
        return Ok(zeroed);
    }
    let output: GitOutput = run_in(
        context,
        destination,
        words(&["update-index", "-z", "--index-info"]).as_slice(),
        Some(input),
        checkout_environment(admin, destination),
    )?;
    return if output.code == Some(0) {
        Ok(zeroed)
    } else {
        Err(run_reason("`git update-index --index-info`", &output))
    };
}

/// What:
///  Write the paths Git must write,
/// in the destination,
/// through one `git checkout-index --force --stdin`.
/// Why:
///   Git owns the conversion,
/// so asking it for these bytes is the only answer that needs no model of a filter,
///  an encoding or an ending;
/// the runs name the destination's Git directory and work tree,
/// as the ladder's other runs do,
/// so the destination's own settings decide.
/// The entries are zeroed first (`zero_the_entries`),
/// because an entry that still matches the file is left alone,
/// and only the paths the destination's index holds are sent,
/// because one it lacks would be injected into the index and then fail the run.
/// `-u` records the stat data of each file Git writes,
/// so the new worktree's first `git status` compares instead of rehashing every fenced path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeThroughGit(admin, destination, context, paths, hashLength); // throws the reason
/// ```
pub fn write_through_git(
    admin: &Path,
    destination: &Path,
    context: &GitContext,
    paths: &[Vec<u8>],
    hash_length: usize,
) -> Result<(), String> {
    let zeroed: Vec<Vec<u8>> = zero_the_entries(admin, destination, context, paths, hash_length)?;
    let index: PathBuf = admin.join("index");
    return checkout_paths(
        admin,
        destination,
        context,
        zeroed.as_slice(),
        index.as_path(),
    );
}

/// What:
///  Run the one `git checkout-index --force -u --stdin` of `paths`,
/// against the index at `index`.
/// Why:
///   Both write steps end in the same run,
/// and each has already decided which paths its index holds,
///  so the run itself asks nothing else;
/// `GIT_INDEX_FILE` names the index,
/// so a rung whose index awaits the worktree's own name records its `-u`
///  stat data in the file that will become that index,
/// and the worktree itself holds no index while the paths are written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutPaths(admin, destination, context, paths, index); // throws the reason
/// ```
fn checkout_paths(
    admin: &Path,
    destination: &Path,
    context: &GitContext,
    paths: &[Vec<u8>],
    index: &Path,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let output: GitOutput = run_in(
        context,
        destination,
        words(&["checkout-index", "--force", "-u", "-z", "--stdin"]).as_slice(),
        Some(input),
        checkout_environment_with_index((admin, destination), Some(index)),
    )?;
    return if output.code == Some(0) {
        Ok(())
    } else {
        Err(run_reason("`git checkout-index`", &output))
    };
}

/// What:
///  Write the paths Git must write through one `git checkout-index --force --stdin`,
/// without zeroing anything first,
/// and only for the paths the destination's index holds.
/// Why:
///   This is the fence beside the clone's write step:
/// the seed the index was installed from holds the written paths' stat data already zeroed
/// (`worktree_clone_conversion_beside_the_clone::seed_without_stat`),
///  so a zeroing run would only rewrite zeroes;
/// the intersection stays,
/// because the tree difference can have removed a path the new `HEAD` lacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexThroughGit(admin, destination, context, paths, hashLength); // throws the reason
/// ```
pub fn checkout_index_through_git(
    admin: &Path,
    destination: &Path,
    context: &GitContext,
    paths: &[Vec<u8>],
    hash_length: usize,
) -> Result<(), String> {
    let index: PathBuf = admin.join("index");
    return checkout_index_through_the_index_at(
        (admin, destination, context),
        index.as_path(),
        paths,
        hash_length,
    );
}

/// What:
///  `checkout_index_through_git` with the index named,
///  for a rung whose index awaits the worktree's own name
/// (`worktree_clone_index::PendingIndex`).
/// Why:
///   Git is pointed at that file with `GIT_INDEX_FILE`,
/// so its `-u` records the stat data of the bytes it wrote in the file that will become the worktree's index,
///  and the worktree holds no index at all until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexThroughTheIndexAt({ admin, destination, context }, pendingPath, paths, hashLength); // throws the reason
/// ```
pub fn checkout_index_through_the_index_at(
    (admin, destination, context): (&Path, &Path, &GitContext),
    index: &Path,
    paths: &[Vec<u8>],
    hash_length: usize,
) -> Result<(), String> {
    let held: HashSet<Vec<u8>> = paths_the_index_holds(index, hash_length)?;
    let present: Vec<Vec<u8>> = paths
        .iter()
        .filter(|path| return held.contains(path.as_slice()))
        .cloned()
        .collect();
    return checkout_paths(admin, destination, context, present.as_slice(), index);
}
