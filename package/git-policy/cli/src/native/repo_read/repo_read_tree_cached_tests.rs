//! Controls for the comparison of a tree with an index beside its cache-tree: the answers equal real Git's in every state of the
//!  cache-tree (valid everywhere, invalid along a path, absent, naming another tree than the one compared), the directories that
//!  are equal are not read, and every state the cache-tree says nothing of is read in full or declined.

use super::*;
use crate::repo_read_index::parse_index_with_tree;
use crate::repo_read_query::tests::{answered, declined, mine, text};
use crate::test_support::{fixture, git, git_output, remove};
use std::path::{Path, PathBuf};

/// A repository of `directories` directories, each `d<n>/sub/f.txt`, in two commits (the second edits `d1/sub/f.txt`), with the
/// objects packed, so `HEAD~1` is a tree the cache-tree of the committed index does not name.
fn committed_repository(root: &Path, name: &str, format: &str, directories: usize) -> PathBuf {
    return repository_with(root, name, format, directories, true);
}

/// `committed_repository`, packed or left as the loose objects Git wrote.
fn repository_with(root: &Path, name: &str, format: &str, directories: usize, packed: bool) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
    for directory in 0..directories {
        std::fs::create_dir_all(repo.join(format!("d{directory}/sub"))).expect("dirs");
        std::fs::write(repo.join(format!("d{directory}/sub/f.txt")), format!("{directory}\n")).expect("file");
        std::fs::write(repo.join(format!("d{directory}/g.txt")), format!("g{directory}\n")).expect("file");
    }
    std::fs::write(repo.join("root.txt"), "root\n").expect("root");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("d1/sub/f.txt"), "changed\n").expect("edit");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=second"]);
    if packed {
        git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
        git(repo.as_path(), &["prune-packed"]);
    }
    return repo;
}

/// The cached comparisons of one state, against `HEAD` and against the commit before it, in the spellings the wrapper issues.
fn cached_shapes(root: &Path, repo: &Path) {
    let head = text(repo, &["rev-parse", "HEAD"]);
    let before = text(repo, &["rev-parse", "HEAD~1"]);
    for object in [head.as_str(), before.as_str()] {
        answered(root, repo, &["diff-index", "--cached", "--name-only", "-z", "--no-renames", object]);
        answered(root, repo, &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", object]);
        answered(root, repo, &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", object, "--", "d3"]);
        answered(root, repo, &["diff-index", "--cached", "--quiet", object]);
        answered(root, repo, &["diff-index", "--cached", "--quiet", object, "--", "d1/sub/f.txt"]);
    }
}

/// An index of the same entries with no cache-tree: a fresh index has none until something writes a tree.
fn without_cache_tree(repo: &Path) {
    let listing = git_output(repo, &["ls-files", "--stage", "-z"]);
    let mut update = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", repo.join(".git/fresh-index"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["update-index", "-z", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("update-index starts");
    std::io::Write::write_all(&mut update.stdin.take().expect("stdin"), listing.stdout.as_slice()).expect("entries");
    assert!(update.wait().expect("update-index ends").success());
    std::fs::rename(repo.join(".git/fresh-index"), repo.join(".git/index")).expect("installed");
}

/// Whether the repository's index file holds a cache-tree.
fn has_cache_tree(repo: &Path, format: crate::repo_read_layout::ObjectFormat) -> bool {
    let bytes = std::fs::read(repo.join(".git/index")).expect("index");
    return parse_index_with_tree(bytes.as_slice(), format).expect("a plain index").tree.is_some();
}

/// The answers equal Git's whatever the cache-tree holds.
#[test]
fn the_answers_equal_git_in_every_state_of_the_cache_tree() {
    let root = fixture("repo-read-cached-states");
    for format in ["sha1", "sha256"] {
        let object_format = if format == "sha256" { crate::repo_read_layout::ObjectFormat::Sha256 } else { crate::repo_read_layout::ObjectFormat::Sha1 };
        let repo = committed_repository(root.as_path(), format, format, 6);
        assert!(has_cache_tree(repo.as_path(), object_format), "a committed index has a cache-tree");
        // Valid everywhere: nothing is staged.
        cached_shapes(root.as_path(), repo.as_path());
        // Invalid along one path only.
        std::fs::write(repo.join("d3/sub/f.txt"), "staged\n").expect("edit");
        git(repo.as_path(), &["add", "d3/sub/f.txt"]);
        cached_shapes(root.as_path(), repo.as_path());
        // The root invalid and the directories valid.
        std::fs::write(repo.join("top.txt"), "top\n").expect("top");
        git(repo.as_path(), &["add", "top.txt"]);
        cached_shapes(root.as_path(), repo.as_path());
        // A directory added, a directory removed, a file whose mode changed.
        std::fs::create_dir_all(repo.join("fresh/inner")).expect("dirs");
        std::fs::write(repo.join("fresh/inner/n.txt"), "new\n").expect("new");
        git(repo.as_path(), &["add", "fresh"]);
        git(repo.as_path(), &["rm", "--cached", "--quiet", "-r", "d4"]);
        std::fs::set_permissions(repo.join("d2/g.txt"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
        git(repo.as_path(), &["add", "d2/g.txt"]);
        cached_shapes(root.as_path(), repo.as_path());
        // A cache-tree written for this state names new trees for the changed directories and the old ones elsewhere.
        git(repo.as_path(), &["write-tree"]);
        cached_shapes(root.as_path(), repo.as_path());
        // Index versions 3 and 4 keep the extension.
        for version in ["4", "3"] {
            git(repo.as_path(), &["update-index", "--index-version", version]);
            cached_shapes(root.as_path(), repo.as_path());
        }
        // No cache-tree at all.
        without_cache_tree(repo.as_path());
        assert!(!has_cache_tree(repo.as_path(), object_format), "the fresh index has none");
        cached_shapes(root.as_path(), repo.as_path());
    }
    remove(root.as_path());
}

/// A path that became a directory, an intent-to-add entry and an unmerged path stay with Git or equal it: the cache-tree says
/// nothing of them.
#[test]
fn states_the_cache_tree_says_nothing_of_are_read_in_full_or_left_to_git() {
    let root = fixture("repo-read-cached-odd");
    let repo = committed_repository(root.as_path(), "odd", "sha1", 4);
    // A file that became a directory.
    git(repo.as_path(), &["rm", "--cached", "--quiet", "root.txt"]);
    std::fs::remove_file(repo.join("root.txt")).expect("rm");
    std::fs::create_dir_all(repo.join("root.txt")).expect("dir");
    std::fs::write(repo.join("root.txt/inner"), "x\n").expect("inner");
    git(repo.as_path(), &["add", "root.txt"]);
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    for shape in [vec!["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", head.as_str()], vec!["diff-index", "--cached", "--quiet", head.as_str()]] {
        if let Some(found) = mine(root.as_path(), repo.as_path(), shape.as_slice()) {
            let expected = git_output(repo.as_path(), shape.as_slice());
            assert_eq!(found.stdout, expected.stdout, "{shape:?}");
            assert_eq!(Some(found.code), expected.status.code(), "{shape:?}");
        }
    }
    // An intent-to-add entry in a directory the cache-tree still holds valid elsewhere.
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD"]);
    std::fs::write(repo.join("d2/new.txt"), "ita\n").expect("ita");
    git(repo.as_path(), &["add", "--intent-to-add", "d2/new.txt"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", "--no-renames", head.as_str()]);
    answered(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", head.as_str()]);
    // An unmerged path.
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD"]);
    git(repo.as_path(), &["branch", "side", "HEAD~1"]);
    git(repo.as_path(), &["checkout", "--quiet", "side"]);
    std::fs::write(repo.join("d1/sub/f.txt"), "side\n").expect("side");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    let merge = git_output(repo.as_path(), &["merge", "--no-commit", "main"]);
    assert!(!merge.status.success(), "the merge conflicts");
    let side = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", "--no-renames", side.as_str()]);
    answered(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", side.as_str()]);
    remove(root.as_path());
}

/// A sparse index is left to Git.
#[test]
fn a_sparse_index_is_left_to_git() {
    let root = fixture("repo-read-cached-sparse");
    let repo = committed_repository(root.as_path(), "sparse", "sha1", 4);
    git(repo.as_path(), &["sparse-checkout", "init", "--cone", "--sparse-index"]);
    git(repo.as_path(), &["sparse-checkout", "set", "d1"]);
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", "--no-renames", head.as_str()]);
    declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", head.as_str()]);
    remove(root.as_path());
}

/// With one staged file the directories that did not change are not read: the entries decoded are those of the trees along the
/// changed path, not one per directory, and the answer still equals Git's. Positive control: a state with no cache-tree reads every
/// before `HEAD`, whose trees the cache-tree does not name, reads far more.
#[test]
fn the_directories_the_cache_tree_proves_equal_are_not_read() {
    let root = fixture("repo-read-cached-reads");
    let directories: usize = 40;
    let repo = committed_repository(root.as_path(), "reads", "sha1", directories);
    std::fs::write(repo.join("d3/sub/f.txt"), "staged\n").expect("edit");
    git(repo.as_path(), &["add", "d3/sub/f.txt"]);
    let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
    let decoded_by = |shape: &[&str]| -> u64 {
        let started = crate::repo_read_pack::DECODED_TOTAL.load(std::sync::atomic::Ordering::Relaxed);
        answered(root.as_path(), repo.as_path(), shape);
        return crate::repo_read_pack::DECODED_TOTAL.load(std::sync::atomic::Ordering::Relaxed) - started;
    };
    let raw = decoded_by(&["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", head.as_str()]);
    let quiet = decoded_by(&["diff-index", "--cached", "--quiet", head.as_str()]);
    let directories_u64: u64 = directories as u64;
    assert!(raw > 0 && raw < directories_u64, "only the trees along the staged path are read: {raw} entries decoded for {directories} directories");
    assert!(quiet > 0 && quiet < directories_u64, "the quiet comparison reads as little: {quiet} entries decoded");
    // Positive control: the commit before HEAD has other trees than the cache-tree names where `d1` changed, and a state with no
    // cache-tree reads every directory.
    without_cache_tree(repo.as_path());
    let whole = decoded_by(&["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", head.as_str()]);
    assert!(whole >= directories_u64 * 2, "without a cache-tree every directory is read: {whole} entries decoded");
    remove(root.as_path());
}

/// A node whose count is not the number of index entries below the directory is not trusted: the walk declines, and the caller
/// reads the whole tree.
#[test]
fn a_count_that_does_not_match_the_index_declines_the_walk() {
    let root = fixture("repo-read-cached-count");
    let repo = committed_repository(root.as_path(), "count", "sha1", 5);
    let format = crate::repo_read_layout::ObjectFormat::Sha1;
    let bytes = std::fs::read(repo.join(".git/index")).expect("index");
    let parsed = parse_index_with_tree(bytes.as_slice(), format).expect("index");
    let mut cache = parsed.tree.expect("a committed index has a cache-tree");
    let index: Vec<FlatEntry> = parsed.entries.iter().map(|entry| { return FlatEntry { path: entry.path.clone(), mode: entry.mode, id: entry.id.clone() }; }).collect();
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("database");
    let head_tree: Vec<u8> = crate::repo_read_object::from_hex(text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]).as_bytes(), format).expect("name");
    assert_eq!(changes_with_cache(&database, head_tree.as_slice(), format, &cache, index.as_slice()).map(|changes| { return changes.len(); }), Some(0), "the cache-tree as written gives no change");
    cache.entry_count += 1;
    assert!(changes_with_cache(&database, head_tree.as_slice(), format, &cache, index.as_slice()).is_none(), "a root count that is off declines");
    remove(root.as_path());
}

/// A matching cache-tree must not hide a tree object that is gone: Git reads the tree it compares with and fails when it is
/// missing, so the readers decline (both diff paths, both object formats) and Git gives the error.
#[test]
fn a_root_tree_that_is_gone_is_not_hidden_by_a_matching_cache_tree() {
    let root = fixture("repo-read-cached-missing-root");
    for format in ["sha1", "sha256"] {
        let repo = repository_with(root.as_path(), format, format, 4, false);
        let tree = text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let head = text(repo.as_path(), &["rev-parse", "HEAD"]);
        // The unchanged index still holds a cache-tree that names this very tree.
        answered(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", head.as_str()]);
        std::fs::remove_file(repo.join(".git/objects").join(&tree[..2]).join(&tree[2..])).expect("the tree is loose");
        declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", head.as_str()]);
        declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--name-only", "-z", "--no-renames", head.as_str()]);
        declined(root.as_path(), repo.as_path(), &["diff-index", "--cached", "--quiet", head.as_str()]);
    }
    remove(root.as_path());
}
