//! Controls for the in-process shadow skeleton:
//!  it is compared with what real `git init --bare --template=` creates,
//! and real Git must accept it as a repository.

use super::*;
use crate::test_support::{fixture, git, git_text, remove};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Every entry under `root` as a path relative to it and whether it is a directory.
fn entries_under(root: &Path) -> BTreeSet<(String, bool)> {
    let mut found: BTreeSet<(String, bool)> = BTreeSet::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("read directory") {
            let path: PathBuf = entry.expect("directory entry").path();
            let is_directory: bool = path.is_dir();
            let relative: String = path.strip_prefix(root).expect("under root").display().to_string();
            found.insert((relative, is_directory));
            if is_directory {
                pending.push(path);
            }
        }
    }
    return found;
}

/// What:
///  The skeleton holds every directory a creation needs and nothing real Git does not make,
///  and `objects/pack`, `refs/heads` and `refs/tags` are left to Git.
/// Why:
///   Leaving those three out is only worth it if every entry the creation makes is one real Git would have made:
///  a directory Git has no shape for would be one the claim's own check could never see.
///  What Git makes of the three on its own is the other control here and the commit in
///  `shadow_pool_tests.rs`
///  ("the commit a member builds is the commit a creation builds").
#[test]
fn the_skeleton_holds_what_a_creation_needs_and_leaves_pack_and_the_ref_folders_to_git() {
    let root: PathBuf = fixture("shadow-skeleton-differential");
    let made: PathBuf = root.join("made");
    let reference: PathBuf = root.join("reference");
    create_shadow_skeleton_in_new_directory(made.as_path());
    git(
        root.as_path(),
        &["init", "--bare", "--quiet", "--template=", "--object-format=sha1", reference.to_str().expect("UTF-8 path")],
    );
    let ours: BTreeSet<(String, bool)> = entries_under(made.as_path());
    let mut theirs: BTreeSet<(String, bool)> = entries_under(reference.as_path());
    assert!(theirs.remove(&(String::from("config"), false)), "real Git writes a config file");
    for entry in &ours {
        assert!(theirs.contains(entry), "the creation makes {}, which real Git does not", entry.0);
    }
    for left_to_git in ["objects/pack", "refs/heads", "refs/tags"] {
        assert!(!ours.contains(&(String::from(left_to_git), true)), "the creation leaves {left_to_git} to Git");
    }
    assert_eq!(ours, BTreeSet::from([
        (String::from("HEAD"), false),
        (String::from("objects"), true),
        (String::from("objects/info"), true),
        (String::from("refs"), true),
    ]));
    remove(root.as_path());
}

/// Real Git reads the skeleton,
/// given the config the shadow writes,
/// as a bare repository whose `HEAD` is symbolic.
#[test]
fn real_git_accepts_the_skeleton_as_a_bare_repository() {
    let root: PathBuf = fixture("shadow-skeleton-accepted");
    let made: PathBuf = root.join("made");
    create_shadow_skeleton_in_new_directory(made.as_path());
    std::fs::write(made.join("config"), "[core]\n\trepositoryformatversion = 0\n\tbare = true\n").expect("config");
    let directory: String = format!("--git-dir={}", made.display());
    assert_eq!(git_text(root.as_path(), &[directory.as_str(), "rev-parse", "--is-bare-repository"]), "true");
    let head: String = git_text(root.as_path(), &[directory.as_str(), "symbolic-ref", "HEAD"]);
    assert!(head.starts_with("refs/heads/"), "HEAD names a branch: {head}");
    remove(root.as_path());
}

/// A directory another run already made is not an error,
/// as with `git init`,
/// and `HEAD` is written again.
#[test]
fn skeleton_over_an_existing_repository_directory_succeeds() {
    let root: PathBuf = fixture("shadow-skeleton-existing");
    let made: PathBuf = root.join("made");
    create_shadow_skeleton_in_new_directory(made.as_path());
    std::fs::write(made.join("HEAD"), "torn").expect("torn HEAD");
    create_shadow_skeleton(made.as_path()).expect("second creation");
    assert_eq!(std::fs::read_to_string(made.join("HEAD")).expect("HEAD"), PLACEHOLDER_HEAD);
    remove(root.as_path());
}

/// A shadow directory whose parent is missing is reported,
/// never created implicitly.
#[test]
fn skeleton_without_a_parent_directory_fails() {
    let root: PathBuf = fixture("shadow-skeleton-orphan");
    let error: std::io::Error =
        create_shadow_skeleton(root.join("missing").join("shadow").as_path()).expect_err("no parent");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    remove(root.as_path());
}

/// Creates the skeleton in a directory that does not exist yet,
/// as the shadow creation does.
fn create_shadow_skeleton_in_new_directory(directory: &Path) {
    create_shadow_skeleton(directory).expect("skeleton");
}
