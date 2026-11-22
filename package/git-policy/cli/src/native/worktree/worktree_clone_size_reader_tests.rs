//! What:
//!  Controls for the fence's size readers (a prototype for measurement):
//! every reader answers the sizes the tree lists,
//! on loose objects,
//! on packed ones,
//! on subdirectory trees,
//! on an alternate object store,
//! on a replaced blob,
//! on a shallow history,
//! on a partial clone's promisor,
//! and on a repository whose paths need no size at all.
//! Why:
//!  The fence's decision is the map,
//! so a reader that answers a different map is a different fence;
//! each control compares the maps entry by entry,
//!  and the shapes are the ones where Git itself answers from somewhere other than a local pack entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const same = (a: Map<string, number>, b: Map<string, number>) => …;
//! ```

use crate::repo_read_layout::ObjectFormat;
use crate::test_support::{fixture, git, git_context, git_output, repository};
use crate::transaction_git::GitContext;
use crate::worktree_clone_size_reader::{SizeRead, SizeReader, read_sizes, wanted_paths};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The sizes as sorted path text and size,
/// so two maps compare as the report reads them.
fn sorted(sizes: &HashMap<Vec<u8>, u64>) -> Vec<(String, u64)> {
    let mut listed: Vec<(String, u64)> = sizes.iter().map(|(path, size)| return (String::from_utf8_lossy(path).to_string(), *size)).collect();
    listed.sort();
    return listed;
}

/// A Git context whose common directory is `repo`'s own,
/// so a reader opens that repository's object database.
fn context_of(repo: &Path) -> GitContext {
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: crate::test_support::isolated_overlay(),
        global_prefix: Vec::new(),
        common_dir: Some(repo.join(".git")),
    };
}

/// The sizes of `wanted` as `reader` answers them.
fn sizes_of(reader: SizeReader, repo: &Path, wanted: &[(Vec<u8>, Vec<u8>)]) -> HashMap<Vec<u8>, u64> {
    let context: GitContext = context_of(repo);
    let common: PathBuf = repo.join(".git");
    let read: SizeRead<'_> = SizeRead {
        context: &context,
        root: repo,
        common_dir: common.as_path(),
        format: ObjectFormat::Sha1,
        head_commit: "HEAD",
    };
    return read_sizes(reader, &read, wanted).expect("the size read answered");
}

/// Every index entry of `repo`,
/// as the path and the id a size is needed for.
fn wanted_of(repo: &Path) -> Vec<(Vec<u8>, Vec<u8>)> {
    let bytes: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the fixture's index");
    let wanted: Vec<(Vec<u8>, Vec<u8>)> = wanted_paths(bytes.as_slice(), ObjectFormat::Sha1, &|_path, _mode| return true).expect("the index parsed");
    assert!(!wanted.is_empty(), "the fixture tracks no path");
    return wanted;
}

/// Assert every reader answers the shipped read's map for `repo`.
fn assert_every_reader_agrees(repo: &Path) {
    let wanted: Vec<(Vec<u8>, Vec<u8>)> = wanted_of(repo);
    let shipped: HashMap<Vec<u8>, u64> = sizes_of(SizeReader::LsTree, repo, wanted.as_slice());
    assert!(!shipped.is_empty(), "the shipped read answered no size for {}", repo.display());
    for reader in [SizeReader::CatFile, SizeReader::ObjectDatabase] {
        let answered: HashMap<Vec<u8>, u64> = sizes_of(reader, repo, wanted.as_slice());
        assert_eq!(sorted(&answered), sorted(&shipped), "{} answered another map for {}", reader.name(), repo.display());
    }
}

/// Write one file of `repo` at `name` with `bytes`.
fn write(repo: &Path, name: &str, bytes: &[u8]) {
    let path: PathBuf = repo.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("fixture directory");
    }
    std::fs::write(path, bytes).expect("fixture file");
}

/// A repository with the shapes a size read meets:
/// a small blob,
/// a large one,
/// an empty one,
/// a subdirectory tree,
/// a symlink,
/// and a file the index marks executable.
fn repository_with_shapes(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), "small.txt", b"one line\n");
    write(repo.as_path(), "large.bin", &vec![0x7eu8; 300_000]);
    write(repo.as_path(), "empty.txt", b"");
    write(repo.as_path(), "deep/er/still.txt", b"three levels\n");
    write(repo.as_path(), "deep/er/deeper/x.bin", &[0u8, 1, 2, 3, 0xff]);
    write(repo.as_path(), "run.sh", b"#!/bin/sh\nexit 0\n");
    std::os::unix::fs::symlink("small.txt", repo.join("link")).expect("fixture link");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["update-index", "--chmod=+x", "run.sh"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=shapes"]);
    return repo;
}

#[test]
fn every_reader_answers_the_sizes_the_tree_lists() {
    let repo: PathBuf = repository_with_shapes("size-reader-shapes");
    assert!(repo.join(".git/objects").exists(), "the fixture has no object directory");
    assert_every_reader_agrees(repo.as_path());
    // The same repository with every object packed answers from another read of the same objects.
    git(repo.as_path(), &["gc", "--quiet"]);
    assert_every_reader_agrees(repo.as_path());
}

#[test]
fn a_loose_object_and_its_packed_copy_answer_the_same_size() {
    let repo: PathBuf = repository_with_shapes("size-reader-loose-and-packed");
    let loose: HashMap<Vec<u8>, u64> = sizes_of(SizeReader::ObjectDatabase, repo.as_path(), wanted_of(repo.as_path()).as_slice());
    git(repo.as_path(), &["gc", "--quiet"]);
    let packed: HashMap<Vec<u8>, u64> = sizes_of(SizeReader::ObjectDatabase, repo.as_path(), wanted_of(repo.as_path()).as_slice());
    assert_eq!(sorted(&packed), sorted(&loose), "a packed object answered another size than its loose copy");
}

#[test]
fn an_alternate_object_store_answers_the_same_sizes() {
    let root: PathBuf = fixture("size-reader-alternate");
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let borrower: PathBuf = repository_with_shapes("size-reader-alternate/repo");
    let donor: PathBuf = repository(root.as_path(), "donor");
    // Every object of the borrower moves into the donor, and the borrower keeps only an alternates file,
    //  so each of its blobs is read through `info/alternates` or not at all.
    git(borrower.as_path(), &["gc", "--quiet"]);
    let packs: PathBuf = borrower.join(".git/objects/pack");
    let into: PathBuf = donor.join(".git/objects/pack");
    std::fs::create_dir_all(into.as_path()).expect("donor packs");
    for entry in std::fs::read_dir(packs.as_path()).expect("the borrower's packs") {
        let from: PathBuf = entry.expect("a pack file").path();
        let to: PathBuf = into.join(from.file_name().expect("a name"));
        std::fs::rename(from.as_path(), to.as_path()).expect("a pack moved");
    }
    let info: PathBuf = borrower.join(".git/objects/info");
    std::fs::create_dir_all(info.as_path()).expect("the alternates directory");
    std::fs::write(info.join("alternates"), format!("{}\n", donor.join(".git/objects").display())).expect("the alternates file");
    let found: Output = git_output(borrower.as_path(), &["cat-file", "-e", "HEAD"]);
    assert!(found.status.success(), "the borrower reads nothing through its alternate");
    assert_every_reader_agrees(borrower.as_path());
}

#[test]
fn a_replaced_blob_answers_the_size_git_answers_for_it() {
    let repo: PathBuf = repository_with_shapes("size-reader-replaced");
    let small: String = String::from_utf8_lossy(&git_output(repo.as_path(), &["rev-parse", "HEAD:small.txt"]).stdout).trim().to_string();
    let large: String = String::from_utf8_lossy(&git_output(repo.as_path(), &["rev-parse", "HEAD:large.bin"]).stdout).trim().to_string();
    git(repo.as_path(), &["replace", small.as_str(), large.as_str()]);
    // Git answers the replacement's size for the replaced blob, so a reader that reads the files must say the same.
    assert_eq!(git_size_of(repo.as_path(), small.as_str()), git_size_of(repo.as_path(), large.as_str()));
    assert_every_reader_agrees(repo.as_path());
}

/// The size Git reports for one object,
/// through the query the readers take.
fn git_size_of(repo: &Path, id: &str) -> String {
    let output: Output = git_output(repo, &["cat-file", "--batch-check=%(objectsize)", id]);
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

#[test]
fn a_shallow_history_answers_the_same_sizes() {
    let root: PathBuf = fixture("size-reader-shallow");
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let origin: PathBuf = repository_with_shapes("size-reader-shallow/origin");
    let clone: PathBuf = root.join("clone");
    let url: String = format!("file://{}", origin.display());
    git(root.as_path(), &["clone", "--quiet", "--depth=1", url.as_str(), clone.to_str().expect("a path")]);
    assert!(clone.join(".git/shallow").exists(), "the clone is not shallow");
    assert_every_reader_agrees(clone.as_path());
}

#[test]
fn a_partial_clones_promisor_answers_the_same_sizes() {
    let root: PathBuf = fixture("size-reader-partial");
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let origin: PathBuf = repository_with_shapes("size-reader-partial/origin");
    let clone: PathBuf = root.join("clone");
    let url: String = format!("file://{}", origin.display());
    git(root.as_path(), &["clone", "--quiet", "--filter=blob:none", "--no-checkout", url.as_str(), clone.to_str().expect("a path")]);
    // An index over the tree and no checkout at all, so every blob is still absent until it is asked for.
    git(clone.as_path(), &["read-tree", "HEAD"]);
    assert!(promised_objects(clone.as_path()), "the clone holds no promisor pack");
    assert_every_reader_agrees(clone.as_path());
}

/// Whether a repository's packs include one a promisor remote named.
fn promised_objects(repo: &Path) -> bool {
    let packs: PathBuf = repo.join(".git/objects/pack");
    let Ok(listing) = std::fs::read_dir(packs) else {
        return false;
    };
    for entry in listing {
        let Ok(found) = entry else { continue };
        if found.path().extension() == Some(std::ffi::OsStr::new("promisor")) {
            return true;
        }
    }
    return false;
}

#[test]
fn a_repository_whose_paths_need_no_size_reads_nothing() {
    let root: PathBuf = fixture("size-reader-nothing");
    std::fs::create_dir_all(root.as_path()).expect("fixture root");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), "a.txt", b"a\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let bytes: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the fixture's index");
    let wanted: Vec<(Vec<u8>, Vec<u8>)> = wanted_paths(bytes.as_slice(), ObjectFormat::Sha1, &|_path, _mode| return false).expect("the index parsed");
    assert!(wanted.is_empty(), "a read that needs nothing still named paths");
    for reader in SizeReader::ALL {
        let answered: HashMap<Vec<u8>, u64> = sizes_of(reader, repo.as_path(), wanted.as_slice());
        assert!(answered.is_empty(), "{} read a size nothing needed", reader.name());
    }
    let _ = git_context();
}
