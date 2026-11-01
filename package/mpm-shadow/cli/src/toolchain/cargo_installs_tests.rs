//! Unit tests of the version-keyed cargo installs:
//! two trees with different locks keep separate third-party versions,
//! a repository-published tool keeps only the newest version,
//! and every command that resolves such a tool warns when a tree's lock pins an older one.
//! Each runs against the fake runner inside disposable homes.

use std::fs;

use super::cargo_installs::{
    current_link, is_repository_published, newest_installed, older_lock_message, older_lock_warnings, running_version,
    version_directory,
};
use super::commands::older_lock_warning_lines;
use super::context::Context;
use super::fakes::Rig;
use super::fakes_cargo::{fake_binstall, finish_install};
use super::lock::{Lock, render_lock};
use super::model::{OsFamily, Route};
use super::observe::{format_status, observe_all};
use super::scope::{Scope, repo_scope};
use super::status_cmd::run_status;
use super::sync::sync_tools;

/// The repository-published tool of the sample lock.
const PUBLISHED: &str = "cargo:monochromatic-mpm-shadow";

/// The third-party cargo tool of the sample lock.
const THIRD_PARTY: &str = "cargo:fd-find";

/// A lock holding only the named tools,
/// each at the given version.
fn lock_of(rig: &Rig, versions: &[(&str, &str)]) -> Lock {
    let mut tools = Vec::new();
    for (id, version) in versions {
        let mut tool = rig.tool(id);
        tool.version = (*version).to_string();
        tools.push(tool);
    }
    return Lock { tools };
}

/// A second tree that shares the rig's home,
/// and so its toolchain root.
fn second_tree(rig: &Rig) -> Scope {
    let root = rig._directory.path().join("work-two");
    fs::create_dir_all(&root).expect("make the second tree");
    return repo_scope(&root, &rig.env).expect("a scope");
}

/// A context for one tree of the rig.
fn context_of<'a>(rig: &'a Rig, scope: &'a Scope, lock: &'a Lock) -> Context<'a> {
    return Context { scope, env: &rig.env, runner: &rig.runner, http: &rig.http, host: &rig.host, tools: &lock.tools };
}

fn binstall_lines(rig: &Rig) -> Vec<String> {
    return rig.runner.lines().into_iter().filter(|line| return line.starts_with("cargo binstall")).collect();
}

#[test]
fn only_a_crate_with_the_repository_prefix_on_the_binstall_route_is_repository_published() {
    let rig = Rig::new();
    assert!(is_repository_published(&rig.tool(PUBLISHED)));
    assert!(!is_repository_published(&rig.tool(THIRD_PARTY)));
    let mut cargo_git = rig.tool(PUBLISHED);
    cargo_git.declaration.route = Route::CargoGit;
    assert!(!is_repository_published(&cargo_git), "another route installs elsewhere");
}

#[test]
fn a_binstall_install_goes_into_its_own_root_and_never_into_the_shared_cargo_home() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["fd"]));
    let lock = lock_of(&rig, &[(THIRD_PARTY, "1.2.3")]);
    let report = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert_eq!(report.installed, vec![THIRD_PARTY.to_string()], "{report:?}");
    let root = version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3");
    assert_eq!(
        binstall_lines(&rig),
        vec![format!("cargo binstall --no-confirm --root {} --version 1.2.3 fd-find", root.display())]
    );
    assert!(root.join("bin").join("fd").is_file());
    assert!(!rig.scope.home.join(".cargo").exists(), "nothing was written below the shared cargo home");
}

#[test]
fn two_trees_with_different_third_party_locks_keep_separate_versions_and_link_their_own() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["fd"]));
    let scope_b = second_tree(&rig);
    let lock_a = lock_of(&rig, &[(THIRD_PARTY, "1.2.3")]);
    let lock_b = lock_of(&rig, &[(THIRD_PARTY, "1.3.0")]);

    // Rejected fixture: with only the other tree's version installed, this tree's lock is not satisfied.
    sync_tools(&context_of(&rig, &scope_b, &lock_b), None);
    let before = observe_all(&context_of(&rig, &rig.scope, &lock_a), &lock_a.tools);
    assert!(!before[0].in_sync, "1.3.0 does not satisfy a lock pinning 1.2.3");
    assert_eq!(before[0].observed, None);

    // Allowed fixture: each tree installs its own and both stay.
    let report_a = sync_tools(&context_of(&rig, &rig.scope, &lock_a), None);
    assert_eq!(report_a.installed, vec![THIRD_PARTY.to_string()]);
    for version in ["1.2.3", "1.3.0"] {
        assert!(version_directory(&rig.scope.toolchain_root, "fd-find", version).join("bin").join("fd").is_file(), "{version}");
    }
    assert_eq!(
        fs::read_link(rig.scope.exposure_dir.join("fd")).expect("a link"),
        version_directory(&rig.scope.toolchain_root, "fd-find", "1.2.3").join("bin").join("fd")
    );
    assert_eq!(
        fs::read_link(scope_b.exposure_dir.join("fd")).expect("a link"),
        version_directory(&rig.scope.toolchain_root, "fd-find", "1.3.0").join("bin").join("fd")
    );

    // A second sync of either tree reinstalls nothing.
    let again_a = sync_tools(&context_of(&rig, &rig.scope, &lock_a), None);
    let again_b = sync_tools(&context_of(&rig, &scope_b, &lock_b), None);
    assert_eq!(again_a.unchanged, vec![THIRD_PARTY.to_string()]);
    assert_eq!(again_b.unchanged, vec![THIRD_PARTY.to_string()]);
    assert_eq!(binstall_lines(&rig).len(), 2, "one install per version");
    assert!(again_a.warnings.is_empty() && again_b.warnings.is_empty(), "nothing shared is replaced");
}

#[test]
fn a_tree_pinning_an_older_repository_tool_keeps_the_newer_install_runs_it_and_is_warned() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["mpm-shadow"]));
    let scope_old = second_tree(&rig);
    let lock_new = lock_of(&rig, &[(PUBLISHED, "0.4.4")]);
    let lock_old = lock_of(&rig, &[(PUBLISHED, "0.4.0")]);

    let report_new = sync_tools(&context_of(&rig, &rig.scope, &lock_new), None);
    assert_eq!(report_new.installed, vec![PUBLISHED.to_string()]);
    assert!(report_new.warnings.is_empty(), "{report_new:?}");

    let report_old = sync_tools(&context_of(&rig, &scope_old, &lock_old), None);
    assert!(report_old.installed.is_empty(), "an older version is never installed beside a newer one");
    assert_eq!(report_old.unchanged, vec![PUBLISHED.to_string()]);
    assert_eq!(binstall_lines(&rig).len(), 1);
    assert!(!version_directory(&rig.scope.toolchain_root, "monochromatic-mpm-shadow", "0.4.0").exists());
    assert_eq!(report_old.warnings, vec![older_lock_message(PUBLISHED, "0.4.0", "0.4.4")]);
    assert!(report_old.warnings[0].contains("older repository tools are never kept"), "{report_old:?}");
    assert!(report_old.warnings[0].contains("mise run toolchain:update"), "{report_old:?}");

    // The old tree's links go through its own launcher to the newest version.
    let launcher = scope_old.launcher_dir.join("mpm-shadow");
    assert_eq!(fs::read_link(scope_old.exposure_dir.join("mpm-shadow")).expect("a link"), launcher);
    assert_eq!(fs::read_link(scope_old.root.join(".cache").join("mpm-shadow-bin").join("mpm")).expect("a link"), launcher);
    let current = current_link(&rig.scope.toolchain_root, "monochromatic-mpm-shadow");
    assert_eq!(running_version(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").as_deref(), Some("0.4.4"));
    assert!(current.join("bin").join("mpm-shadow").is_file());
}

#[test]
fn installing_a_newer_repository_tool_removes_the_older_one_and_leaves_no_dangling_link() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["mpm-shadow"]));
    let scope_old = second_tree(&rig);
    let lock_new = lock_of(&rig, &[(PUBLISHED, "0.4.4")]);
    let lock_old = lock_of(&rig, &[(PUBLISHED, "0.4.0")]);

    sync_tools(&context_of(&rig, &scope_old, &lock_old), None);
    let older = version_directory(&rig.scope.toolchain_root, "monochromatic-mpm-shadow", "0.4.0");
    assert!(older.is_dir(), "the older version installs while nothing newer exists");
    assert_eq!(running_version(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").as_deref(), Some("0.4.0"));

    sync_tools(&context_of(&rig, &rig.scope, &lock_new), None);
    assert!(!older.exists(), "installing 0.4.4 removed 0.4.0");
    assert_eq!(newest_installed(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").as_deref(), Some("0.4.4"));

    // The old tree never synced again, and its link still resolves: it goes through `current`.
    let resolved = fs::canonicalize(scope_old.exposure_dir.join("mpm-shadow")).expect("the link resolves");
    assert!(resolved.exists());
    let through_current = fs::canonicalize(current_link(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").join("bin").join("mpm-shadow"))
        .expect("current resolves");
    assert!(through_current.to_string_lossy().ends_with("0.4.4/bin/mpm-shadow"), "{}", through_current.display());
}

#[test]
fn status_counts_a_newer_repository_tool_as_satisfied_but_not_a_newer_third_party_one() {
    let rig = Rig::new();
    let root = &rig.scope.toolchain_root;
    finish_install(&version_directory(root, "monochromatic-mpm-shadow", "0.4.4"), "monochromatic-mpm-shadow", "0.4.4", &["mpm-shadow"]);
    finish_install(&version_directory(root, "fd-find", "1.3.0"), "fd-find", "1.3.0", &["fd"]);
    let lock = lock_of(&rig, &[(PUBLISHED, "0.4.0"), (THIRD_PARTY, "1.2.3")]);
    let observations = observe_all(&context_of(&rig, &rig.scope, &lock), &lock.tools);
    assert!(observations[0].in_sync, "the repository tool runs the newer install");
    assert_eq!(observations[0].observed.as_deref(), Some("0.4.4"));
    assert!(!observations[1].in_sync, "a third-party lock wants its own version");
    let text = format_status(&observations);
    assert!(text.contains("ok      cargo:monochromatic-mpm-shadow  0.4.0\n"), "{text}");
    assert!(text.contains("drift   cargo:fd-find  locked=1.2.3  observed=(absent)\n"), "{text}");
}

#[test]
fn an_unfinished_install_is_not_counted() {
    let rig = Rig::new();
    let unfinished = version_directory(&rig.scope.toolchain_root, "monochromatic-mpm-shadow", "0.4.4");
    fs::create_dir_all(unfinished.join("bin")).expect("make");
    assert_eq!(newest_installed(&rig.scope.toolchain_root, "monochromatic-mpm-shadow"), None);
}

/// Writes the rig's on-disk lock with the repository tool at `version` and installs `installed`.
fn tree_with_installed(rig: &Rig, locked: &str, installed: &str) {
    // The whole sample lock, so it still agrees with the sample plan on disk.
    let mut lock = rig.lock.clone();
    for tool in &mut lock.tools {
        if tool.declaration.id == PUBLISHED {
            tool.version = locked.to_string();
        }
    }
    fs::write(&rig.scope.lock_file, render_lock(&lock)).expect("write the lock");
    let root = version_directory(&rig.scope.toolchain_root, "monochromatic-mpm-shadow", installed);
    finish_install(&root, "monochromatic-mpm-shadow", installed, &["mpm-shadow"]);
}

#[test]
fn install_update_outdated_and_bin_warn_on_standard_error_with_their_own_verb() {
    let rig = Rig::new();
    tree_with_installed(&rig, "0.4.0", "0.4.4");
    for verb in ["install", "update", "outdated", "bin"] {
        let lines = older_lock_warning_lines(verb, &rig.scope, Some(OsFamily::Linux));
        assert_eq!(
            lines,
            vec![format!("toolchain {verb}: warning: {}", older_lock_message(PUBLISHED, "0.4.0", "0.4.4"))],
            "{verb}"
        );
    }
}

#[test]
fn status_prints_the_warning_and_stays_silent_when_the_lock_and_the_install_match() {
    let rig = Rig::new();
    tree_with_installed(&rig, "0.4.0", "0.4.4");
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("status runs");
    let warning = format!("toolchain list: warning: {}\n", older_lock_message(PUBLISHED, "0.4.0", "0.4.4"));
    assert!(report.text.contains(&warning), "{}", report.text);

    let matching = Rig::new();
    tree_with_installed(&matching, "0.4.4", "0.4.4");
    let quiet = run_status(&matching.scope, &matching.env, &matching.runner, &matching.host).expect("status runs");
    assert!(!quiet.text.contains("warning"), "{}", quiet.text);
    for verb in ["install", "update", "outdated", "bin"] {
        assert!(older_lock_warning_lines(verb, &matching.scope, Some(OsFamily::Linux)).is_empty(), "{verb}");
    }
}

#[test]
fn a_third_party_tool_and_another_operating_system_never_warn() {
    let rig = Rig::new();
    let root = &rig.scope.toolchain_root;
    finish_install(&version_directory(root, "fd-find", "1.3.0"), "fd-find", "1.3.0", &["fd"]);
    finish_install(&version_directory(root, "monochromatic-mpm-shadow", "0.4.4"), "monochromatic-mpm-shadow", "0.4.4", &["mpm-shadow"]);
    let lock = lock_of(&rig, &[(THIRD_PARTY, "1.3.0"), (PUBLISHED, "0.4.0")]);
    let scope: &Scope = &rig.scope;
    assert_eq!(older_lock_warnings(scope, &lock.tools, Some(OsFamily::Macos)), Vec::<String>::new(), "the tool is limited to Linux");
    let linux = older_lock_warnings(scope, &lock.tools, Some(OsFamily::Linux));
    assert_eq!(linux, vec![older_lock_message(PUBLISHED, "0.4.0", "0.4.4")]);
}
