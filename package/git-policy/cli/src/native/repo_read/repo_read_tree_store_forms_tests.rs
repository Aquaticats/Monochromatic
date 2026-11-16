//! What:
//!  The table of object-store forms the commit `HEAD` names and its whole tree are read out of in process:
//!  loose objects,
//!  a pack `git gc` makes,
//!  a pack a fetch makes,
//!  a tree stored as a delta,
//!  a delta chain deeper than a hundred,
//!  a version one pack index,
//!  a multi-pack index and a reverse index beside the packs,
//!  a repository borrowing through `info/alternates`,
//!  a commit graph,
//!  a bitmap,
//!  SHA-256,
//!  and the empty tree.
//! Why:
//!   The cleanliness proof reads the tree of `HEAD` through this reader,
//!  and a form it does not know makes the removal of a worktree refuse or fall back to Git;
//!  every row compares the reader's flat list with `git ls-tree -r --full-tree` byte for byte,
//!  and the rows the reader cannot read (today: a version one pack index,
//!  and a delta chain longer than a hundred) state the words a person meets where it gives up.
//!
//! The fixtures and the two comparisons live in `repo_read_tree_store_forms_fixtures.rs`.

use super::*;
use crate::repo_read_object::to_hex;
use crate::test_support::{git, remove};
use std::path::PathBuf;

#[path = "repo_read_tree_store_forms_fixtures.rs"]
mod fixtures;

use fixtures::{
    FILES, deepest_subtree_commit, delta_bases, format_of, head_tree_name, history, imported_history, pack_files, stored_as, table_fixture,
    the_reader_lists_what_git_lists,
};

/// Loose objects: the plain store,
/// the positive control every packed row is compared against.
#[test]
fn a_loose_tree_of_head_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-loose");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    assert!(pack_files(repo.as_path(), "idx").is_empty(), "nothing was packed yet");
    let compared: usize = the_reader_lists_what_git_lists(repo.as_path(), "loose");
    assert!(compared > FILES, "the whole tree crosses the nested directory: {compared} entries");
    remove(root.as_path());
}

/// A pack `git gc` makes:
/// one pack,
/// a reverse index and a commit graph beside it,
/// and every object of `HEAD` inside it.
#[test]
fn a_tree_of_head_in_a_gc_pack_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-gc");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    let table: String = String::from_utf8(crate::test_support::git_output(repo.as_path(), &["count-objects", "-v"]).stdout).expect("UTF-8");
    assert!(table.lines().any(|line| { return line == "count: 0"; }), "gc left no loose object: {table}");
    assert_eq!(pack_files(repo.as_path(), "idx").len(), 1, "gc made one pack");
    the_reader_lists_what_git_lists(repo.as_path(), "gc");
    remove(root.as_path());
}

/// A pack a fetch makes:
/// the receiving repository borrows no object,
/// so every object of `HEAD` can only come out of the fetched pack.
#[test]
fn a_tree_of_head_in_a_fetched_pack_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-fetched");
    let source: PathBuf = history(root.as_path(), "source", &[]);
    git(source.as_path(), &["gc", "--quiet", "--prune=now"]);
    let url: String = format!("file://{}", source.display());
    let clone: PathBuf = root.join("clone");
    git(root.as_path(), &["clone", "--quiet", "--no-local", url.as_str(), clone.to_str().expect("UTF-8")]);
    assert_eq!(pack_files(clone.as_path(), "idx").len(), 1, "the fetch made one pack");
    the_reader_lists_what_git_lists(clone.as_path(), "fetched");
    remove(root.as_path());
}

/// A pack whose trees are stored as deltas,
/// as `git repack` stores them by default:
/// the positive control shows the fixture really holds a tree stored as a delta.
#[test]
fn a_tree_of_head_stored_as_a_delta_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-delta");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet", "--window=250", "--depth=50"]);
    git(repo.as_path(), &["prune-packed"]);
    let as_deltas: usize = pack_files(repo.as_path(), "idx")
        .iter()
        .map(|pack| {
            let table: String = String::from_utf8(crate::test_support::git_output(repo.as_path(), &["verify-pack", "-v", pack.to_str().expect("UTF-8")]).stdout).expect("UTF-8");
            return table
                .lines()
                .filter(|line| { return line.split_whitespace().nth(1) == Some("tree"); })
                .filter(|line| { return line.split_whitespace().count() == 7; })
                .count();
        })
        .sum();
    assert!(as_deltas > 0, "the repack stored no tree as a delta");
    the_reader_lists_what_git_lists(repo.as_path(), "delta");
    remove(root.as_path());
}

/// What:
///  A tree of `HEAD` stored at the end of a delta chain deeper than a hundred,
///  which `git gc --aggressive` makes (`repack -f --depth=250 --window=250`).
/// Why:
///   Git follows a chain as deep as the pack holds,
/// so a reader with a ceiling of its own refuses a tree Git reads,
/// and the cleanliness proof then answers that the tree of `HEAD` is unreadable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = importedHistory(root, 'repo'); await git(repo, ['repack','-a','-d','-f','--window=250','--depth=250']);
/// const { commit, depth } = deepestSubtreeCommit(repo, 'd'); expect(depth).toBeGreaterThan(100);
/// ```
#[test]
fn a_tree_of_head_at_the_end_of_a_delta_chain_deeper_than_a_hundred_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-deep-chain");
    let repo: PathBuf = imported_history(root.as_path(), "repo");
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet", "--window=250", "--depth=250"]);
    git(repo.as_path(), &["prune-packed"]);
    let bases = delta_bases(repo.as_path());
    let (commit, depth) = deepest_subtree_commit(repo.as_path(), "d", &bases);
    assert!(depth > 100, "the fixture holds a tree below HEAD stored at the end of a chain of {depth} deltas");
    // The commit whose subtree is deepest becomes HEAD, so the whole tree of HEAD must cross that chain.
    git(repo.as_path(), &["checkout", "--quiet", "--detach", commit.as_str()]);
    the_reader_lists_what_git_lists(repo.as_path(), "a delta chain deeper than a hundred");
    remove(root.as_path());
}

/// What:
///  A pack whose index is version 1,
///  which `git -c pack.indexVersion=1 repack` writes and Git reads.
/// Why:
///   Version 1 has no magic and no large-offset table,
///  so a reader of version 2 only declines the whole object database,
///  and the cleanliness proof then answers that the object database has a layout it does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = history(root, 'repo', []); await git(repo, ['-c','pack.indexVersion=1','repack','-a','-d','-f']);
/// ```
#[test]
fn a_tree_of_head_in_a_pack_with_a_version_one_index_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-index-version-1");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["-c", "pack.indexVersion=1", "repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let index: PathBuf = pack_files(repo.as_path(), "idx").into_iter().next().expect("an index");
    let bytes: Vec<u8> = std::fs::read(&index).expect("the index's bytes");
    // Positive control: a version 1 index starts with its fanout table, not with version 2's magic.
    assert_ne!(&bytes[..4], b"\xfftOc", "the fixture's index is version 1");
    let mut counts: [u8; 4] = [0; 4];
    counts.copy_from_slice(&bytes[255 * 4..255 * 4 + 4]);
    let stored: usize = crate::test_support::git_output(repo.as_path(), &["cat-file", "--batch-all-objects", "--batch-check"]).stdout.iter().filter(|byte| { return **byte == b'\n'; }).count();
    assert_eq!(u32::from_be_bytes(counts) as usize, stored, "the index's fanout ends at the number of objects the pack holds");
    the_reader_lists_what_git_lists(repo.as_path(), "index version 1");
    remove(root.as_path());
}

/// A multi-pack index and a reverse index beside two packs,
/// which is what a repository with several packs and a background maintenance has.
#[test]
fn a_tree_of_head_under_a_multi_pack_index_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-multi-pack");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    std::fs::write(repo.join("late.txt"), "late\n").expect("late");
    git(repo.as_path(), &["add", "late.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=late"]);
    git(repo.as_path(), &["repack", "-d", "--quiet"]);
    git(repo.as_path(), &["multi-pack-index", "write"]);
    assert!(repo.join(".git/objects/pack/multi-pack-index").exists(), "the multi-pack index was written");
    // Positive control: the indexes the reader ignores are there, so this row proves it ignores them rather than lacks them.
    assert!(!pack_files(repo.as_path(), "rev").is_empty(), "a reverse index sits beside a pack");
    assert_eq!(pack_files(repo.as_path(), "idx").len(), 2, "two packs");
    the_reader_lists_what_git_lists(repo.as_path(), "multi-pack index");
    remove(root.as_path());
}

/// A bitmap beside the pack of a repository whose history was repacked for serving.
#[test]
fn a_tree_of_head_beside_a_bitmap_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-bitmap");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet", "--write-bitmap-index"]);
    git(repo.as_path(), &["prune-packed"]);
    assert_eq!(pack_files(repo.as_path(), "bitmap").len(), 1, "a bitmap sits beside the pack");
    the_reader_lists_what_git_lists(repo.as_path(), "bitmap");
    remove(root.as_path());
}

/// A commit graph the reader never opens,
/// written by `git commit-graph write` and beside the pack.
#[test]
fn a_tree_of_head_beside_a_commit_graph_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-commit-graph");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    git(repo.as_path(), &["commit-graph", "write", "--reachable"]);
    assert!(repo.join(".git/objects/info/commit-graph").exists(), "the commit graph was written");
    the_reader_lists_what_git_lists(repo.as_path(), "commit graph");
    remove(root.as_path());
}

/// A repository that borrows every object through `info/alternates`:
/// its own object directory holds no pack and no loose object.
#[test]
fn a_tree_of_head_borrowed_through_alternates_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-alternates");
    let source: PathBuf = history(root.as_path(), "source", &[]);
    git(source.as_path(), &["gc", "--quiet", "--prune=now"]);
    let clone: PathBuf = root.join("clone");
    git(root.as_path(), &["clone", "--quiet", "--shared", source.to_str().expect("UTF-8"), clone.to_str().expect("UTF-8")]);
    assert!(clone.join(".git/objects/info/alternates").exists(), "the clone borrows its objects");
    assert!(pack_files(clone.as_path(), "idx").is_empty(), "the clone holds no pack of its own");
    the_reader_lists_what_git_lists(clone.as_path(), "alternates");
    remove(root.as_path());
}

/// A SHA-256 repository whose objects are all in a pack.
#[test]
fn a_sha256_tree_of_head_in_a_pack_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-sha256");
    let repo: PathBuf = history(root.as_path(), "repo", &["--object-format=sha256"]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    assert_eq!(format_of(repo.as_path()), ObjectFormat::Sha256, "the fixture is a SHA-256 repository");
    let tree: String = head_tree_name(repo.as_path());
    let name = crate::repo_read_object::from_hex(tree.as_bytes(), ObjectFormat::Sha256).expect("a SHA-256 name");
    assert_eq!(to_hex(name.as_slice()), tree);
    the_reader_lists_what_git_lists(repo.as_path(), "sha256");
    remove(root.as_path());
}

/// The empty tree of a commit that holds no file,
/// stored in a pack:
/// its entry states a size of zero,
/// which the inflater must still accept.
#[test]
fn a_packed_empty_tree_of_head_is_read() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-empty");
    let repo: PathBuf = fixtures::place(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=empty"]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    let stored: String = stored_as(repo.as_path(), head_tree_name(repo.as_path()).as_str()).expect("the empty tree is in the pack");
    assert!(stored.contains(" tree   0 "), "the tree of HEAD is empty and packed: {stored}");
    let Ok(read) = fixtures::read_head_tree(repo.as_path()) else {
        panic!("the reader declined the empty tree of HEAD");
    };
    assert!(read.is_empty(), "the empty tree has no entries");
    remove(root.as_path());
}

/// The kind and store of the tree of `HEAD` in the packed fixture,
/// printed for the report:
/// this control fails when the fixture stops holding what the table claims.
#[test]
fn the_packed_fixtures_hold_the_store_forms_the_table_names() {
    let root: PathBuf = table_fixture("repo-read-tree-forms-described");
    let repo: PathBuf = history(root.as_path(), "repo", &[]);
    git(repo.as_path(), &["gc", "--quiet", "--prune=now"]);
    let tree: String = head_tree_name(repo.as_path());
    let stored: String = stored_as(repo.as_path(), tree.as_str()).expect("the tree of HEAD is in the gc pack");
    assert!(stored.contains(" tree "), "the tree of HEAD is a tree: {stored}");
    assert_eq!(stored.split_whitespace().count(), 5, "stored whole, not as a delta: {stored}");
    remove(root.as_path());
}
