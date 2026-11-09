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
use super::prune_cargo::{PresentVersion, named_by_all, named_versions, plan_prune, present_versions, prune_toolchain};
use super::worktrees::worktree_lock_files;

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

#[test]
fn a_repository_published_crate_keeps_the_longer_alpha_number_as_its_newest_version() {
    let found = present(&[("monochromatic-lint", &["0.4.5-alpha.9", "0.4.5-alpha.10"])]);
    let removals = plan_prune(&found, &named(&[("monochromatic-lint", &["0.4.5-alpha.9", "0.4.5-alpha.10"])]));
    assert_eq!(removals, vec![("monochromatic-lint".to_string(), "0.4.5-alpha.9".to_string())]);
}

/// A crate directory goes only when it is empty:
/// whatever another run left in it, which the scan does not count as a version, is not carried away
/// with the directory.
#[test]
fn a_crate_directory_that_holds_something_the_scan_does_not_count_stays() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    // The lock names a version that is not installed, so `1.1.0` is the only deletion and nothing remains.
    fs::write(
        rig.scope.root.join("toolchain.lock.json"),
        "{\"tools\":{\"cargo:fd-find\":{\"request\":\"cargo:fd-find@latest\",\"version\":\"1.2.0\",\"route\":\"cargo-binstall\"}}}",
    )
    .expect("write");
    // A link an `atomic_symlink` write of another run leaves behind: a name the scan skips, in the crate directory.
    let leftover = root.join("cargo").join("fd-find").join(".current.4242.tmp");
    fs::write(&leftover, "").expect("drop the leftover");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let removed = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert_eq!(removed.len(), 1, "{removed:?}");
    assert!(!version_directory(&root, "fd-find", "1.1.0").exists(), "the unnamed version goes");
    assert!(leftover.exists(), "the crate directory was removed with something inside it: {removed:?}");
}

/// A prune that cannot repoint `current` deletes nothing:
/// the link is moved to the newest finished version before the older one goes,
/// so a run that stops at the link has never deleted the version the link named.
#[test]
fn a_prune_that_cannot_repoint_current_deletes_no_version() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    for version in ["1.1.0", "1.2.0"] {
        finish_install(
            &version_directory(&root, "monochromatic-lint", version),
            "monochromatic-lint",
            version,
            &["monochromatic-lint"],
        );
    }
    // `current` is a directory here, so the atomic link write cannot land: the prune stops at the repoint.
    fs::create_dir(current_link(&root, "monochromatic-lint")).expect("squat the link");
    fs::write(
        rig.scope.root.join("toolchain.lock.json"),
        "{\"tools\":{\"cargo:monochromatic-lint\":{\"request\":\"cargo:monochromatic-lint@latest\",\"version\":\"1.1.0\",\"route\":\"cargo-binstall\"}}}",
    )
    .expect("write");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let pruned = prune_toolchain(&rig.scope, &rig.runner, &[], false);
    let error = pruned.expect_err("the link cannot be written over a directory");
    assert!(error.message.contains("current"), "the message names the link: {error}");
    assert!(version_directory(&root, "monochromatic-lint", "1.1.0").is_dir(), "the version `current` named was deleted");
    assert!(version_directory(&root, "monochromatic-lint", "1.2.0").is_dir(), "the newest finished version was deleted");
}

/// A version installed while the prune runs is never deleted:
/// the prune lists what is present before it reads any lock,
/// so an install that lands afterwards is not in the list it deletes from.
#[test]
fn an_installation_that_lands_while_the_prune_runs_is_never_deleted() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.0.0"), "fd-find", "1.0.0", &["fd"]);
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    fs::write(
        rig.scope.root.join("toolchain.lock.json"),
        "{\"tools\":{\"cargo:fd-find\":{\"request\":\"cargo:fd-find@latest\",\"version\":\"1.1.0\",\"route\":\"cargo-binstall\"}}}",
    )
    .expect("write");
    // Another run finishes installing 1.2.0 while this prune lists worktrees, before it reads any lock.
    let installed_into = root.clone();
    rig.runner.on(
        "git -C",
        Box::new(move |_request| {
            finish_install(&version_directory(&installed_into, "fd-find", "1.2.0"), "fd-find", "1.2.0", &["fd"]);
        }),
    );
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let removed = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert!(
        version_directory(&root, "fd-find", "1.2.0").is_dir(),
        "the version installed after the prune listed what was present was deleted: {removed:?}"
    );
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "the named version stays");
    assert!(!version_directory(&root, "fd-find", "1.0.0").exists(), "a version that was there and unnamed is still pruned");
}

/// Sets a directory's mode, so a read of it can be made to fail.
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("chmod");
}

/// The scan reports a crate directory it could not read instead of leaving it out:
/// a caller that reads silence as "no versions are left" would delete the whole directory,
/// including a version a lock names.
#[test]
fn a_crate_directory_the_scan_cannot_read_is_an_error_and_not_an_empty_crate() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.2.0"), "fd-find", "1.2.0", &["fd"]);
    let crate_dir = root.join("cargo").join("fd-find");
    set_mode(&crate_dir, 0o000);
    let scanned = present_versions(&root);
    set_mode(&crate_dir, 0o755);
    let error = scanned.expect_err("a directory it cannot read proves nothing");
    assert!(error.message.contains(&crate_dir.display().to_string()), "the message names the directory: {error}");
    assert!(error.message.contains("nothing was deleted"), "{error}");
}

/// A prune that cannot read a crate directory deletes nothing and says so,
/// rather than reporting that there was nothing to prune.
#[test]
fn the_command_refuses_when_it_cannot_read_a_crate_directory() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    finish_install(&version_directory(&root, "fd-find", "1.2.0"), "fd-find", "1.2.0", &["fd"]);
    fs::write(rig.scope.root.join("toolchain.lock.json"), "{\"tools\":{\"cargo:fd-find\":{\"request\":\"cargo:fd-find@latest\",\"version\":\"1.2.0\",\"route\":\"cargo-binstall\"}}}").expect("write");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let crate_dir = root.join("cargo").join("fd-find");
    set_mode(&crate_dir, 0o000);
    let pruned = prune_toolchain(&rig.scope, &rig.runner, &[], false);
    set_mode(&crate_dir, 0o755);
    let error = pruned.expect_err("a crate directory it cannot read proves nothing");
    assert!(error.message.contains(&crate_dir.display().to_string()), "the message names the directory: {error}");
    assert!(version_directory(&root, "fd-find", "1.2.0").is_dir(), "the version the lock names was deleted");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "a version no lock names was deleted");
}

/// A `cargo-binstall` row with no version is not a lock row this command can judge:
/// reading it as naming nothing would delete the crate it installs,
/// so the read refuses and the version stays.
#[test]
fn a_cargo_binstall_row_without_a_version_refuses_the_prune_and_deletes_nothing() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    fs::write(rig.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");
    let broken = rig._directory.path().join("clone").join("toolchain.lock.json");
    fs::create_dir_all(rig._directory.path().join("clone")).expect("make");
    fs::write(&broken, "{\"tools\":{\"cargo:fd-find\":{\"request\":\"cargo:fd-find@latest\",\"route\":\"cargo-binstall\"}}}").expect("write");
    let error = prune_toolchain(&rig.scope, &rig.runner, std::slice::from_ref(&broken), false).expect_err("the row names no version");
    assert!(error.message.contains("cargo:fd-find"), "the message names the row: {error}");
    assert!(error.message.contains("nothing was deleted"), "the message says nothing happened: {error}");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "nothing was deleted");
}

/// An explicit `--lock` whose file is gone by the time the command reads it refuses the prune:
/// a check that runs before the read can be outrun by a concurrent rename,
/// and then the lock would protect nothing while the prune deletes the version it names.
#[test]
fn an_explicit_lock_that_vanishes_before_it_is_read_refuses_the_prune_and_deletes_nothing() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    fs::write(rig.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");
    let clone_lock = write_lock(&rig._directory.path().join("clone"), "fd-find", "1.1.0");
    let moved_aside = clone_lock.with_file_name("toolchain.lock.json.moved");
    let vanished = clone_lock.clone();
    // A concurrent command renames the lock away while this run lists worktrees, before it reads any lock.
    rig.runner.on(
        "git -C",
        Box::new(move |_request| {
            if vanished.exists() {
                fs::rename(&vanished, &moved_aside).expect("move the lock aside");
            }
        }),
    );
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    let error = prune_toolchain(&rig.scope, &rig.runner, std::slice::from_ref(&clone_lock), false).expect_err("the named lock is gone when it is read");
    assert!(error.message.contains(&clone_lock.display().to_string()), "the message names the path: {error}");
    assert!(error.message.contains("cannot be read"), "the refusal comes from the read and not from a check before it: {error}");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "nothing was deleted");
}

/// The read stage itself tells the two kinds of lock apart:
/// one discovery reported may be absent,
/// one the caller named may not.
#[test]
fn the_read_stage_refuses_a_missing_explicit_lock_and_skips_a_missing_discovered_one() {
    let rig = Rig::new();
    let absent: PathBuf = rig._directory.path().join("absent").join("toolchain.lock.json");
    assert!(named_by_all(std::slice::from_ref(&absent), &[]).is_ok(), "a lock discovery reports may be absent");
    let error = named_by_all(&[], std::slice::from_ref(&absent)).expect_err("a lock the caller named must be readable");
    assert!(error.message.contains(&absent.display().to_string()), "the message names the path: {error}");
    assert!(error.message.contains("nothing was deleted"), "the message says nothing happened: {error}");
}

/// An explicit `--lock` path that never existed refuses the prune too:
/// a deletion command must not read a mistyped path as protecting nothing.
#[test]
fn an_explicit_lock_file_that_does_not_exist_refuses_the_prune_and_deletes_nothing() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    finish_install(&version_directory(&root, "fd-find", "1.1.0"), "fd-find", "1.1.0", &["fd"]);
    fs::write(rig.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");
    let missing: PathBuf = rig._directory.path().join("clone").join("toolchain.lock.json");
    let error = prune_toolchain(&rig.scope, &rig.runner, std::slice::from_ref(&missing), false).expect_err("the named lock does not exist");
    assert!(error.message.contains(&missing.display().to_string()), "the message names the path: {error}");
    assert!(version_directory(&root, "fd-find", "1.1.0").is_dir(), "nothing was deleted");
}
