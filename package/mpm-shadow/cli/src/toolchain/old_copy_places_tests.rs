//! Unit tests of the places searched for links to an old copy and of the scan:
//! links match by resolved path (two spellings of one directory),
//! a foreign link,
//! a dangling link and a regular file are never reported,
//! and a place git cannot list is recorded as a reason to remove nothing.
//! Every fixture sits in a disposable directory.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use super::fakes::{FakeRunner, Rig, write_executable};
use super::old_copy_places::{Place, PlaceKind, Places, known_places, links_to_copy};
use super::scope::{ScopeKind, global_scope};

/// A link at `link` pointing at `target`, making the link's directory.
fn link(link: &Path, target: &Path) {
    fs::create_dir_all(link.parent().expect("a parent")).expect("make the directory");
    symlink(target, link).expect("make the link");
}

/// The answer of `git worktree list --porcelain` for these directories.
fn listing(directories: &[&Path]) -> String {
    let mut text: String = String::new();
    for directory in directories {
        text.push_str(&format!("worktree {}\nHEAD 0000\nbranch refs/heads/x\n\n", directory.display()));
    }
    return text;
}

/// A place of one kind.
fn place(kind: PlaceKind, directory: PathBuf) -> Place {
    return Place { kind, directory };
}

#[test]
fn a_link_is_found_by_resolved_path_even_when_it_spells_the_directory_another_way() {
    let rig = Rig::new();
    let root = rig._directory.path();
    // The owner's machine: `/home` is a link to `var/home`, and one link spells the copy each way.
    fs::create_dir_all(root.join("var").join("home").join(".cargo").join("bin")).expect("make the real home");
    symlink("var/home", root.join("alias")).expect("make the alias");
    let copy = root.join("var").join("home").join(".cargo").join("bin").join("search-shadow");
    write_executable(&copy);
    let directory = root.join("links");
    link(&directory.join("by-alias"), &root.join("alias").join(".cargo").join("bin").join("search-shadow"));
    link(&directory.join("by-real-path"), &copy);
    let places = Places { directories: vec![place(PlaceKind::Checkout, directory.clone())], incomplete: Vec::new() };
    let scan = links_to_copy(&places, &copy);
    let names: Vec<String> = scan.found.iter().map(|found| return found.path.file_name().expect("a name").to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["by-alias".to_string(), "by-real-path".to_string()], "{scan:?}");
}

#[test]
fn a_foreign_link_a_dangling_link_a_regular_file_and_a_directory_are_not_reported() {
    let rig = Rig::new();
    let root = rig._directory.path();
    let copy = root.join("cargo-bin").join("cli-git-native");
    write_executable(&copy);
    let other = root.join("somewhere").join("tool");
    write_executable(&other);
    let directory = root.join("links");
    link(&directory.join("to-the-copy"), &copy);
    link(&directory.join("foreign"), &other);
    link(&directory.join("dangling"), &root.join("gone"));
    write_executable(&directory.join("regular-file"));
    fs::create_dir_all(directory.join("a-directory")).expect("make a directory");
    let places = Places { directories: vec![place(PlaceKind::LocalBin, directory.clone())], incomplete: Vec::new() };
    let scan = links_to_copy(&places, &copy);
    assert_eq!(scan.found.len(), 1, "{scan:?}");
    assert_eq!(scan.found[0].path, directory.join("to-the-copy"));
    assert_eq!(scan.found[0].kind, PlaceKind::LocalBin);
}

#[test]
fn a_chain_of_links_is_reported_at_every_link_that_resolves_to_the_copy() {
    let rig = Rig::new();
    let root = rig._directory.path();
    let copy = root.join("cargo-bin").join("mpm-shadow");
    write_executable(&copy);
    let directory = root.join("links");
    link(&directory.join("first"), &copy);
    link(&directory.join("second"), &directory.join("first"));
    let places = Places { directories: vec![place(PlaceKind::GlobalFarm, directory)], incomplete: Vec::new() };
    assert_eq!(links_to_copy(&places, &copy).found.len(), 2);
}

#[test]
fn a_missing_place_is_no_links_and_an_unlistable_one_is_reported() {
    let rig = Rig::new();
    let root = rig._directory.path();
    let copy = root.join("cargo-bin").join("mpm-shadow");
    write_executable(&copy);
    let not_a_directory = root.join("a-file");
    fs::write(&not_a_directory, "x").expect("write a file");
    let places = Places {
        directories: vec![place(PlaceKind::Checkout, root.join("missing")), place(PlaceKind::Checkout, not_a_directory.clone())],
        incomplete: Vec::new(),
    };
    let scan = links_to_copy(&places, &copy);
    assert!(scan.found.is_empty());
    assert_eq!(scan.unreadable, vec![not_a_directory], "a file where a directory should be cannot be listed");
}

#[test]
fn the_places_of_a_repository_run_are_local_bin_the_global_farm_and_every_checkouts_bin_directories() {
    let rig = Rig::new();
    let root = rig._directory.path();
    let second = root.join("work-two");
    for checkout in [&rig.scope.root, &second] {
        fs::create_dir_all(checkout.join(".cache").join("cli-git-bin")).expect("make a link directory");
        fs::create_dir_all(checkout.join(".cache").join("toolchain-bin")).expect("make the exposure directory");
        fs::create_dir_all(checkout.join(".cache").join("toolchain-launchers")).expect("not a link directory");
        fs::write(checkout.join(".cache").join("forbidden-strings-bin"), "a file, not a directory").expect("write a file");
    }
    rig.runner.answer("git -C", &listing(&[&rig.scope.root, &second]), 0);
    let places = known_places(&rig.scope, &rig.env, &rig.runner);
    assert!(places.incomplete.is_empty(), "{places:?}");
    let global = global_scope(&rig.env).expect("a global scope");
    let kinds: Vec<(PlaceKind, PathBuf)> = places.directories.iter().map(|found| return (found.kind, found.directory.clone())).collect();
    assert!(kinds.contains(&(PlaceKind::LocalBin, global.link_dir.clone())), "{kinds:?}");
    assert!(kinds.contains(&(PlaceKind::GlobalFarm, global.exposure_dir.clone())), "{kinds:?}");
    for checkout in [&rig.scope.root, &second] {
        assert!(kinds.contains(&(PlaceKind::Checkout, checkout.join(".cache").join("cli-git-bin"))), "{kinds:?}");
        assert!(kinds.contains(&(PlaceKind::Checkout, checkout.join(".cache").join("toolchain-bin"))), "{kinds:?}");
    }
    assert_eq!(kinds.len(), 6, "no launchers directory and no plain file: {kinds:?}");
    assert_eq!(rig.runner.lines().len(), 1, "git was asked once");
}

#[test]
fn a_worktree_git_lists_but_cannot_be_found_is_unseen_unless_git_marks_it_prunable() {
    let rig = Rig::new();
    let gone = rig._directory.path().join("gone");
    let gone_text = gone.display().to_string();
    rig.runner.answer("git -C", &format!("worktree {gone_text}\nHEAD 0000\n\n"), 0);
    let unmarked = known_places(&rig.scope, &rig.env, &rig.runner);
    assert_eq!(unmarked.incomplete.len(), 1, "{unmarked:?}");
    assert!(unmarked.incomplete[0].contains(&gone_text), "{unmarked:?}");

    let marked_rig = Rig::new();
    let gone_marked = marked_rig._directory.path().join("gone");
    marked_rig.runner.answer("git -C", &format!("worktree {}\nHEAD 0000\nprunable gitdir file points to non-existent location\n\n", gone_marked.display()), 0);
    let marked = known_places(&marked_rig.scope, &marked_rig.env, &marked_rig.runner);
    assert!(marked.incomplete.is_empty(), "{marked:?}");
}

#[test]
fn git_failing_to_list_the_worktrees_makes_the_places_incomplete_but_keeps_the_scopes_own_checkout() {
    let rig = Rig::new();
    fs::create_dir_all(rig.scope.root.join(".cache").join("cli-git-bin")).expect("make a link directory");
    rig.runner.fail("git -C", "fatal: not a git repository");
    let places = known_places(&rig.scope, &rig.env, &rig.runner);
    assert_eq!(places.incomplete.len(), 1, "{places:?}");
    assert!(places.incomplete[0].contains("not a git repository"), "{places:?}");
    assert!(
        places.directories.iter().any(|found| return found.kind == PlaceKind::Checkout),
        "the checkout the run is in is still searched"
    );
}

#[test]
fn a_global_run_searches_no_checkout_and_says_why() {
    let rig = Rig::new();
    let mut scope = global_scope(&rig.env).expect("a global scope");
    assert_eq!(scope.kind, ScopeKind::Global);
    scope.home = rig.scope.home.clone();
    let runner = FakeRunner::new();
    let places = known_places(&scope, &rig.env, &runner);
    assert_eq!(places.incomplete.len(), 1, "{places:?}");
    assert!(places.incomplete[0].contains("inside a checkout"), "{places:?}");
    assert!(places.directories.iter().all(|found| return found.kind.is_outside_checkout()));
    assert!(runner.lines().is_empty(), "a global run never asks git");
}
