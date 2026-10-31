//! Unit tests of the cleanup of unnamed cargo versions:
//! the pure rule with allowed and rejected cases,
//! the lock reader,
//! the worktree list,
//! and the command against a disposable toolchain root.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use super::cargo_installs::{current_link, running_version, version_directory};
use super::fakes::Rig;
use super::fakes_cargo::finish_install;
use super::prune_cargo::{PresentVersion, named_versions, plan_prune, prune_toolchain, worktree_lock_files};

fn present(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<PresentVersion>> {
    let mut map = BTreeMap::new();
    for (crate_name, versions) in entries {
        map.insert(
            (*crate_name).to_string(),
            versions.iter().map(|version| return PresentVersion { version: (*version).to_string(), finished: true }).collect(),
        );
    }
    return map;
}

fn named(entries: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    let mut map = BTreeMap::new();
    for (crate_name, versions) in entries {
        map.insert((*crate_name).to_string(), versions.iter().map(|version| return (*version).to_string()).collect());
    }
    return map;
}

#[test]
fn a_third_party_version_survives_only_when_some_lock_names_it() {
    let found = present(&[("fd-find", &["1.1.0", "1.2.3", "1.3.0"])]);
    let removals = plan_prune(&found, &named(&[("fd-find", &["1.2.3", "1.3.0"])]));
    assert_eq!(removals, vec![("fd-find".to_string(), "1.1.0".to_string())]);
    let unnamed = plan_prune(&found, &named(&[]));
    assert_eq!(unnamed.len(), 3, "a crate no lock names goes entirely");
}

#[test]
fn a_repository_published_crate_keeps_only_its_newest_finished_version_even_when_locks_name_older_ones() {
    let found = present(&[("monochromatic-lint", &["0.3.0", "0.4.0", "0.4.4"])]);
    let removals = plan_prune(&found, &named(&[("monochromatic-lint", &["0.4.0", "0.3.0"])]));
    assert_eq!(removals, vec![("monochromatic-lint".to_string(), "0.3.0".to_string()), ("monochromatic-lint".to_string(), "0.4.0".to_string())]);
    assert!(plan_prune(&found, &named(&[("monochromatic-lint", &["0.4.4"])])).len() == 2);
    assert_eq!(plan_prune(&found, &named(&[])).len(), 3, "unnamed by any lock: all go");
}

#[test]
fn an_unfinished_newer_version_is_never_deleted_by_the_newest_rule() {
    let mut found = present(&[("monochromatic-lint", &["0.4.0"])]);
    found.get_mut("monochromatic-lint").expect("crate").push(PresentVersion { version: "0.5.0".to_string(), finished: false });
    let removals = plan_prune(&found, &named(&[("monochromatic-lint", &["0.5.0"])]));
    assert_eq!(removals, Vec::<(String, String)>::new(), "0.4.0 is the newest finished, 0.5.0 may be installing");
}

#[test]
fn a_lock_in_either_format_names_its_binstall_crates() {
    let rust_format = r#"{"tools":{"cargo:fd-find":{"request":"cargo:fd-find@latest","version":"1.2.3","route":"cargo-binstall"},
        "node":{"version":"26.0.0","route":"archive"}}}"#;
    let typescript_format = r#"{"tools":{"cargo:dylint-link":{"request":{"track":"latest"},"version":"6.3.2","route":"cargo-binstall","packageId":"dylint-link"}}}"#;
    assert_eq!(named_versions(rust_format, "a").expect("reads"), named(&[("fd-find", &["1.2.3"])]));
    assert_eq!(named_versions(typescript_format, "b").expect("reads"), named(&[("dylint-link", &["6.3.2"])]));
    assert!(named_versions("not json", "c").is_err());
    assert!(named_versions("{}", "d").is_err());
}

#[test]
fn the_worktree_list_gives_each_lock_and_refuses_for_a_missing_directory_that_is_not_prunable() {
    let rig = Rig::new();
    let alive = rig._directory.path().join("alive");
    fs::create_dir_all(&alive).expect("make");
    let porcelain = format!("worktree {}\nHEAD abc\nbranch refs/heads/x\n\nworktree /gone/and/prunable\nHEAD def\ndetached\nprunable gitdir file points to non-existent location\n\n", alive.display());
    rig.runner.answer("git -C", &porcelain, 0);
    let files = worktree_lock_files(&rig.runner, &rig.scope.root).expect("lists");
    assert_eq!(files, vec![alive.join("toolchain.lock.json")]);

    let unmounted = Rig::new();
    unmounted.runner.answer("git -C", "worktree /mnt/offline/tree\nHEAD abc\ndetached\n\n", 0);
    let error = worktree_lock_files(&unmounted.runner, &unmounted.scope.root).expect_err("refuses");
    assert!(error.message.contains("/mnt/offline/tree") && error.message.contains("nothing was deleted"), "{error}");

    let failing = Rig::new();
    failing.runner.fail("git -C", "not a repository");
    assert!(worktree_lock_files(&failing.runner, &failing.scope.root).is_err());
}

/// Writes a lock naming `crate_name` at `version` into `directory` and returns its path.
fn write_lock(directory: &PathBuf, crate_name: &str, version: &str) -> PathBuf {
    fs::create_dir_all(directory).expect("make");
    let path = directory.join("toolchain.lock.json");
    fs::write(&path, format!("{{\"tools\":{{\"cargo:{crate_name}\":{{\"request\":\"cargo:{crate_name}@latest\",\"version\":\"{version}\",\"route\":\"cargo-binstall\"}}}}}}")).expect("write");
    return path;
}

#[test]
fn the_command_keeps_every_version_a_listed_worktree_names_and_removes_the_rest() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    for version in ["1.1.0", "1.2.3", "1.3.0"] {
        finish_install(&version_directory(&root, "fd-find", version), "fd-find", version, &["fd"]);
    }
    finish_install(&version_directory(&root, "cargo-nextest", "0.9.1"), "cargo-nextest", "0.9.1", &["cargo-nextest"]);
    for version in ["0.4.0", "0.4.4"] {
        finish_install(&version_directory(&root, "monochromatic-lint", version), "monochromatic-lint", version, &["monochromatic-lint"]);
    }
    std::os::unix::fs::symlink("0.4.4", current_link(&root, "monochromatic-lint")).expect("current");

    let other_tree = rig._directory.path().join("other-tree");
    write_lock(&other_tree, "fd-find", "1.3.0");
    write_lock(&rig.scope.root, "fd-find", "1.2.3");
    let own_lock = rig.scope.root.join("toolchain.lock.json");
    fs::write(&own_lock, "{\"tools\":{\"cargo:fd-find\":{\"request\":\"cargo:fd-find@latest\",\"version\":\"1.2.3\",\"route\":\"cargo-binstall\"},\"cargo:monochromatic-lint\":{\"request\":\"cargo:monochromatic-lint@latest\",\"version\":\"0.4.0\",\"route\":\"cargo-binstall\"}}}").expect("write");
    rig.runner.answer(
        "git -C",
        &format!("worktree {}\nHEAD a\n\nworktree {}\nHEAD b\n\n", rig.scope.root.display(), other_tree.display()),
        0,
    );

    let planned = prune_toolchain(&rig.scope, &rig.runner, &[], true).expect("dry run");
    assert_eq!(planned.len(), 3, "{planned:?}");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "a dry run deletes nothing");

    let removed = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert_eq!(removed.len(), 3, "{removed:?}");
    assert!(!version_directory(&root, "fd-find", "1.1.0").exists(), "named by no lock");
    assert!(version_directory(&root, "fd-find", "1.2.3").is_dir(), "named by this tree");
    assert!(version_directory(&root, "fd-find", "1.3.0").is_dir(), "named by the other worktree");
    assert!(!root.join("cargo").join("cargo-nextest").exists(), "a crate no lock names goes with its directory");
    assert!(!version_directory(&root, "monochromatic-lint", "0.4.0").exists(), "only the newest repository version stays");
    assert_eq!(running_version(&root, "monochromatic-lint").as_deref(), Some("0.4.4"));
    assert!(!current_link(&root, "fd-find").exists(), "a third-party crate never gets a current link");
}

#[test]
fn an_extra_lock_keeps_the_version_it_names_and_an_unreadable_lock_deletes_nothing() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    fs::write(rig.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");

    let clone_lock = write_lock(&rig._directory.path().join("clone"), "fd-find", "1.1.0");
    assert!(prune_toolchain(&rig.scope, &rig.runner, &[clone_lock], false).expect("pruned").is_empty());
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir());

    fs::write(rig.scope.root.join("toolchain.lock.json"), "this is not json").expect("write");
    let error = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect_err("refuses");
    assert!(error.message.contains("not JSON"), "{error}");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "a lock that cannot be read deletes nothing");
}
