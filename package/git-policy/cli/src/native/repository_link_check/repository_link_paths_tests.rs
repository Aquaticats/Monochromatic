//! Controls for the paths an invocation names and for the test of whether one lies under the repository link:
//! every spelling of the repository reaches it,
//! and no neighbour does.

use super::*;
use crate::repository_link_identity::LinkPhase;
use crate::test_support::{fixture, remove};

/// An identity whose link path is `<root>/repolink`,
/// a link to a folder that is not there.
fn identity_in(root: &Path) -> LinkIdentity {
    let link: PathBuf = root.join("repolink");
    return LinkIdentity {
        link_path: link.to_string_lossy().into_owned(),
        link_text: root.join("mnt/repository/Monochromatic").to_string_lossy().into_owned(),
        canonical_root: root.join("mnt/repository/Monochromatic").to_string_lossy().into_owned(),
        mount_point: root.join("mnt").to_string_lossy().into_owned(),
        drive_uuid: String::from("DRIVE"),
        nvme_uuid: String::from("NVME"),
        root_commit: String::from("abc"),
        phase: LinkPhase::After,
    };
}

/// Makes the dangling link of an identity.
fn dangle(identity: &LinkIdentity) {
    std::os::unix::fs::symlink(identity.link_text.as_str(), identity.link_path.as_str()).expect("dangling link");
}

fn text(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(|value| return OsString::from(value)).collect();
}

fn pair(name: &str, value: &str) -> (OsString, OsString) {
    return (OsString::from(name), OsString::from(value));
}

/// With no option the invocation works in the current directory.
#[test]
fn a_plain_invocation_names_its_current_directory() {
    assert_eq!(named_paths(Path::new("/work/here"), &[], &[]), vec![PathBuf::from("/work/here")]);
}

/// Each `-C` moves the directory,
/// a relative one from the one before,
/// and the current directory is no longer named.
#[test]
fn every_dash_c_moves_the_directory_the_way_git_chains_them() {
    let prefix: Vec<OsString> = text(&["-C", "/a/b", "-C", "c", "-c", "x=y", "-C", "../d"]);
    assert_eq!(named_paths(Path::new("/work/here"), prefix.as_slice(), &[]), vec![PathBuf::from("/a/b/c/../d")]);
}

/// The directory a command runs in follows the `-C` chain and nothing else:
///  `--work-tree` names a tree,
///  never the directory a relative path is read from,
///  and a token that looks like `-C` inside another option's value is no move.
#[test]
fn the_directory_a_command_runs_in_follows_only_the_dash_c_chain() {
    assert_eq!(chain_directory(Path::new("/work/here"), &[]), PathBuf::from("/work/here"));
    let prefix: Vec<OsString> = text(&["-C", "/a/b", "--work-tree", "/elsewhere", "--git-dir", "-C", "-C", "c"]);
    assert_eq!(chain_directory(Path::new("/work/here"), prefix.as_slice()), PathBuf::from("/a/b/c"));
}

/// The Git directory and work tree are named whether they come as options or as environment,
/// relative ones from the directory after the `-C` chain.
#[test]
fn the_git_directory_work_tree_and_common_directory_are_named() {
    let prefix: Vec<OsString> = text(&["-C", "/w", "--git-dir=/g/dir", "--work-tree", "tree"]);
    let environment: Vec<(OsString, OsString)> = vec![pair("GIT_DIR", "env-git"), pair("GIT_WORK_TREE", "/env/tree"), pair("GIT_COMMON_DIR", "/env/common")];
    let paths: Vec<PathBuf> = named_paths(Path::new("/cwd"), prefix.as_slice(), environment.as_slice());
    for expected in ["/w", "/g/dir", "/w/tree", "/w/env-git", "/env/tree", "/env/common"] {
        assert!(paths.contains(&PathBuf::from(expected)), "{expected} is named in {paths:?}");
    }
}

/// A path under a link that dangles,
/// under the approved root,
/// through an alias of the parent,
/// with dot-dot segments,
/// or under the renamed old copy,
/// lies under the repository;
/// a neighbour with a longer name does not.
#[test]
fn every_spelling_of_the_repository_is_under_it_and_no_neighbour_is() {
    let root: PathBuf = fixture("link-paths-spellings").canonicalize().expect("root");
    let identity: LinkIdentity = identity_in(root.as_path());
    dangle(&identity);
    std::fs::create_dir_all(root.join("elsewhere")).expect("elsewhere");
    std::os::unix::fs::symlink(root.as_path(), root.join("alias")).expect("alias of the parent");
    let link: PathBuf = root.join("repolink");
    for under in [
        link.clone(),
        link.join("sub/deeper"),
        link.join(".git"),
        root.join("alias/repolink/sub"),
        root.join("elsewhere/../repolink/sub"),
        root.join("mnt/repository/Monochromatic/.git/worktrees/x"),
        root.join("repolink.nvme-before-860-2026-11-07T20:00:00-05:00/sub"),
    ] {
        assert!(touches_repository(&identity, under.as_path()), "{} lies under the repository", under.display());
    }
    for outside in [root.join("repolink2"), root.join("elsewhere"), root.join("mnt/repository/Monochromatic2"), root.clone(), PathBuf::from("/usr/bin")] {
        assert!(!touches_repository(&identity, outside.as_path()), "{} does not lie under it", outside.display());
    }
    remove(root.as_path());
}

/// A link that resolves (the drive is mounted) is under the repository too:
/// the path resolves into the approved root.
#[test]
fn a_link_that_resolves_is_under_the_repository() {
    let root: PathBuf = fixture("link-paths-resolving").canonicalize().expect("root");
    let identity: LinkIdentity = identity_in(root.as_path());
    std::fs::create_dir_all(identity.canonical_root.as_str()).expect("the 860 folder");
    dangle(&identity);
    assert!(touches_repository(&identity, root.join("repolink/sub").as_path()));
    assert!(touches_repository(&identity, root.join("repolink").as_path()));
    remove(root.as_path());
}

/// An identity that recorded no link path names no repository.
#[test]
fn an_identity_without_a_link_path_touches_nothing() {
    let mut identity: LinkIdentity = identity_in(Path::new("/x"));
    identity.link_path = String::new();
    identity.canonical_root = String::new();
    assert!(!touches_repository(&identity, Path::new("/x/repolink")));
    assert!(!touches_repository(&identity, Path::new("/")));
}
