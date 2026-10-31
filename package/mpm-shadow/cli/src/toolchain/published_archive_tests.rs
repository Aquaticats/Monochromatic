//! Unit tests of the newest-only layout for archive-route tools this repository publishes (`mpm-shadow` itself):
//! which tools count,
//! install and replacement,
//! the no-downgrade guard,
//! the warnings of every command,
//! the launcher's version,
//! and the cleanup.
//! Each runs against the fake runner and the fake network inside disposable homes.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::cargo_installs::{is_repository_published, older_lock_message, older_lock_warnings};
use super::commands::older_lock_warning_lines;
use super::context::Context;
use super::fakes::{Rig, write_executable};
use super::hash::sha256_hex;
use super::launcher::{LaunchRecord, prepare_launch, usable_source};
use super::lock::{Lock, render_lock};
use super::model::{Artifact, LockedTool, OsFamily, Request, Route};
use super::observe::observe_all;
use super::prune_cargo::{PresentVersion, prune_toolchain};
use super::published_archive::{
    current_link_of, is_published_archive, named_archives, newest_installed_archive, plan_archive_prune, running_version_archive,
    tool_directory,
};
use super::runner::RunRequest;
use super::scope::{Scope, repo_scope};
use super::status_cmd::run_status;
use super::sync::sync_tools;

/// The tool id.
const ID: &str = "mpm-shadow";

/// The program the release archive holds
/// (not `mpm-shadow`,
/// which the sample plan's crate tool already exposes under that name).
const BIN: &str = "mpm-shadow-release";

/// The tag prefix that names this program's releases.
const PREFIX: &str = "monochromatic-mpm-shadow-v";

/// Where a release of the tool is downloaded from.
fn release_url(version: &str) -> String {
    return format!("https://example.invalid/releases/{PREFIX}{version}/mpm-shadow-{version}.tar.gz");
}

/// The bytes of a release archive (the fake `tar` does not read them).
fn release_body(version: &str) -> Vec<u8> {
    return format!("release archive {version}").into_bytes();
}

/// The release-archive tool at `version`,
/// with the archive row of the host.
fn release_tool(rig: &Rig, version: &str) -> LockedTool {
    let mut tool = rig.tool("node");
    tool.declaration.id = ID.to_string();
    tool.declaration.request = Request::parse("github:Aquaticats/Monochromatic@latest").expect("parses");
    tool.declaration.bins = vec![BIN.to_string()];
    tool.declaration.options.tag_prefix = Some(PREFIX.to_string());
    tool.version = version.to_string();
    tool.artifacts = vec![(rig.host.platform.clone(), Artifact { url: release_url(version), sha256: sha256_hex(&release_body(version)) })];
    return tool;
}

/// A lock holding only the release-archive tool at `version`.
fn lock_at(rig: &Rig, version: &str) -> Lock {
    return Lock { tools: vec![release_tool(rig, version)] };
}

/// Makes the fake network serve a release and the fake `tar` unpack it as `<folder>/mpm-shadow`.
fn serve_releases(rig: &Rig, versions: &[&str]) {
    for version in versions {
        rig.http.serve(&release_url(version), &release_body(version));
    }
    rig.runner.on(
        "tar --extract",
        Box::new(|request: &RunRequest| {
            let position: usize = request.args.iter().position(|argument| return argument == "--directory").expect("a directory");
            let directory: PathBuf = PathBuf::from(&request.args[position + 1]);
            write_executable(&directory.join("release-folder").join(BIN));
        }),
    );
}

/// The `tar` lines the fake runner saw.
fn tar_lines(rig: &Rig) -> Vec<String> {
    return rig.runner.lines().into_iter().filter(|line| return line.starts_with("tar --extract")).collect();
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

/// Finishes an install of `version` by hand:
/// the program,
/// inside the folder an archive would hold.
fn finish_archive(root: &Path, version: &str) -> PathBuf {
    let program = tool_directory(root, ID).join(version).join("release-folder").join(BIN);
    write_executable(&program);
    return program;
}

#[test]
fn only_an_archive_tool_with_a_repository_tag_prefix_is_repository_published() {
    let rig = Rig::new();
    let published = release_tool(&rig, "0.2.0");
    assert!(is_published_archive(&published));
    assert!(is_repository_published(&published));

    let mut without_prefix = release_tool(&rig, "0.2.0");
    without_prefix.declaration.options.tag_prefix = None;
    assert!(!is_repository_published(&without_prefix), "a plain archive tool is third-party");

    let mut other_prefix = release_tool(&rig, "0.2.0");
    other_prefix.declaration.options.tag_prefix = Some("other-project-v".to_string());
    assert!(!is_repository_published(&other_prefix), "a prefix that does not start with monochromatic- names another project");

    let mut wrong_route = release_tool(&rig, "0.2.0");
    wrong_route.declaration.route = Route::BinSourced;
    assert!(!is_repository_published(&wrong_route), "the prefix only counts on the archive route");

    let mut crate_with_prefix = rig.tool("cargo:fd-find");
    crate_with_prefix.declaration.options.tag_prefix = Some(PREFIX.to_string());
    assert!(!is_repository_published(&crate_with_prefix), "a third-party crate is decided by its crate name");
}

#[test]
fn installing_a_newer_release_removes_the_older_one_its_download_and_leaves_no_dangling_link() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.0", "0.2.1"]);
    let lock_old = lock_at(&rig, "0.2.0");
    let lock_new = lock_at(&rig, "0.2.1");
    let root = rig.scope.toolchain_root.clone();

    let first = sync_tools(&context_of(&rig, &rig.scope, &lock_old), None);
    assert_eq!(first.installed, vec![ID.to_string()], "{first:?}");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.0"));
    let old_download = root.join("downloads").join(format!("{ID}-0.2.0-mpm-shadow-0.2.0.tar.gz"));
    assert!(old_download.is_file());

    let second = sync_tools(&context_of(&rig, &rig.scope, &lock_new), None);
    assert_eq!(second.installed, vec![ID.to_string()], "{second:?}");
    assert!(!tool_directory(&root, ID).join("0.2.0").exists(), "installing 0.2.1 removed 0.2.0");
    assert!(!old_download.exists(), "and the archive downloaded for it");
    assert!(root.join("downloads").join(format!("{ID}-0.2.1-mpm-shadow-0.2.1.tar.gz")).is_file());
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"));

    // This tree's exposed link goes through its launcher to the program behind `current`, which resolves.
    let launcher = rig.scope.launcher_dir.join(BIN);
    assert_eq!(fs::read_link(rig.scope.exposure_dir.join(BIN)).expect("a link"), launcher);
    let through_current = fs::canonicalize(current_link_of(&root, ID).join("release-folder").join(BIN)).expect("current resolves");
    assert!(through_current.to_string_lossy().ends_with("0.2.1/release-folder/mpm-shadow-release"), "{}", through_current.display());
}

#[test]
fn a_tree_pinning_an_older_release_keeps_the_newer_install_runs_it_and_is_warned() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.0", "0.2.1"]);
    let scope_old = second_tree(&rig);
    let lock_new = lock_at(&rig, "0.2.1");
    let lock_old = lock_at(&rig, "0.2.0");
    let root = rig.scope.toolchain_root.clone();

    let report_new = sync_tools(&context_of(&rig, &rig.scope, &lock_new), None);
    assert_eq!(report_new.installed, vec![ID.to_string()]);
    assert!(report_new.warnings.is_empty(), "{report_new:?}");

    let report_old = sync_tools(&context_of(&rig, &scope_old, &lock_old), None);
    assert!(report_old.installed.is_empty(), "an older version is never installed beside a newer one");
    assert_eq!(report_old.unchanged, vec![ID.to_string()]);
    assert_eq!(tar_lines(&rig).len(), 1, "0.2.0 was not even downloaded unpacked");
    assert!(!tool_directory(&root, ID).join("0.2.0").exists());
    assert_eq!(report_old.warnings, vec![older_lock_message(ID, "0.2.0", "0.2.1")], "{report_old:?}");
    assert!(report_old.warnings[0].contains("older repository tools are never kept"));
    assert!(report_old.warnings[0].contains("mise run toolchain:update"));

    // The old tree's link goes through its own launcher, so every run of the tool warns there.
    assert_eq!(fs::read_link(scope_old.exposure_dir.join(BIN)).expect("a link"), scope_old.launcher_dir.join(BIN));
    assert!(scope_old.launcher_dir.join(format!("{BIN}.launch")).is_file());
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"));
}

#[test]
fn a_release_is_observed_as_the_newest_finished_install_and_an_unfinished_one_is_not_counted() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let lock = lock_at(&rig, "0.2.0");
    assert_eq!(newest_installed_archive(&root, ID, BIN), None, "nothing installed");

    // An interrupted unpacking: the version directory exists, the program does not.
    fs::create_dir_all(tool_directory(&root, ID).join("0.2.2").join("release-folder")).expect("make");
    assert_eq!(newest_installed_archive(&root, ID, BIN), None, "an unfinished install is not counted");
    let before = observe_all(&context_of(&rig, &rig.scope, &lock), &lock.tools);
    assert!(!before[0].in_sync);
    assert_eq!(before[0].observed, None);

    finish_archive(&root, "0.2.1");
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some("0.2.1"), "the unfinished 0.2.2 is skipped");
    assert_eq!(newest_installed_archive(&root, ID, ""), None, "a tool that declares no program never finishes");
    let after = observe_all(&context_of(&rig, &rig.scope, &lock), &lock.tools);
    assert!(after[0].in_sync, "the newer install satisfies an older lock");
    assert_eq!(after[0].observed.as_deref(), Some("0.2.1"));

    // Rejected fixture: an older install does not satisfy a newer lock.
    let lock_newer = lock_at(&rig, "0.3.0");
    let behind = observe_all(&context_of(&rig, &rig.scope, &lock_newer), &lock_newer.tools);
    assert!(!behind[0].in_sync);
}

/// Writes the rig's on-disk plan and lock with the release tool first,
/// the lock pinning `locked`,
/// and installs `installed`.
fn tree_with_installed(rig: &Rig, locked: &str, installed: &str) {
    let entry = r#""mpm-shadow": { "request": "github:Aquaticats/Monochromatic@latest", "route": "archive", "bins": ["mpm-shadow-release"], "options": { "tagPrefix": "monochromatic-mpm-shadow-v" } },"#;
    let plan = fs::read_to_string(&rig.scope.plan_file).expect("read the plan");
    fs::write(&rig.scope.plan_file, plan.replacen("\"tools\": {", &format!("\"tools\": {{\n{entry}"), 1)).expect("write the plan");
    let mut lock = rig.lock.clone();
    lock.tools.insert(0, release_tool(rig, locked));
    fs::write(&rig.scope.lock_file, render_lock(&lock)).expect("write the lock");
    finish_archive(&rig.scope.toolchain_root, installed);
}

#[test]
fn every_command_warns_about_an_older_lock_for_a_release_archive_and_stays_silent_when_they_match() {
    let rig = Rig::new();
    tree_with_installed(&rig, "0.2.0", "0.2.1");
    for verb in ["install", "update", "outdated", "bin"] {
        let lines = older_lock_warning_lines(verb, &rig.scope, Some(OsFamily::Linux));
        assert_eq!(lines, vec![format!("toolchain {verb}: warning: {}", older_lock_message(ID, "0.2.0", "0.2.1"))], "{verb}");
    }
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("status runs");
    let warning = format!("toolchain list: warning: {}\n", older_lock_message(ID, "0.2.0", "0.2.1"));
    assert!(report.text.contains(&warning), "{}", report.text);

    let matching = Rig::new();
    tree_with_installed(&matching, "0.2.1", "0.2.1");
    for verb in ["install", "update", "outdated", "bin"] {
        assert!(older_lock_warning_lines(verb, &matching.scope, Some(OsFamily::Linux)).is_empty(), "{verb}");
    }
    let lock = lock_at(&matching, "0.2.1");
    assert!(older_lock_warnings(&matching.scope, &lock.tools, Some(OsFamily::Linux)).is_empty());
}

/// A record for a launcher whose real program sits `depth` folders below the `current` link.
fn record_below_current(directory: &Path, current: &str, depth: usize) -> LaunchRecord {
    let tool_dir = directory.join(ID);
    fs::create_dir_all(tool_dir.join(current)).expect("make");
    std::os::unix::fs::symlink(current, tool_dir.join("current")).expect("link current");
    let mut real = tool_dir.join("current");
    for level in 0..depth {
        real = real.join(format!("folder-{level}"));
    }
    return LaunchRecord {
        tool_id: ID.to_string(),
        lock_file: directory.join("toolchain.lock.json"),
        real: real.join(BIN),
    };
}

/// A lock text pinning the release tool at `version`.
fn lock_text(version: &str) -> String {
    return format!("{{\"schemaVersion\":1,\"tools\":{{\"{ID}\":{{\"version\":\"{version}\",\"route\":\"archive\"}}}}}}");
}

#[test]
fn the_launcher_finds_the_version_of_a_release_archive_below_any_number_of_folders() {
    let rig = Rig::new();
    for depth in [0, 1, 3] {
        let record = record_below_current(&rig._directory.path().join(format!("depth-{depth}")), "0.2.1", depth);
        let launch = prepare_launch(&record, Some(&lock_text("0.2.0")));
        assert_eq!(launch.warning, Some(older_lock_message(ID, "0.2.0", "0.2.1")), "depth {depth}");
        assert_eq!(prepare_launch(&record, Some(&lock_text("0.2.1"))).warning, None, "depth {depth}: matching");
    }
    let unlinked = LaunchRecord {
        tool_id: ID.to_string(),
        lock_file: rig._directory.path().join("toolchain.lock.json"),
        real: rig._directory.path().join("elsewhere").join("folder").join(BIN),
    };
    assert_eq!(prepare_launch(&unlinked, Some(&lock_text("0.1.0"))).warning, None, "no current link above the program");
}

#[test]
fn a_launcher_is_made_from_proc_when_the_running_file_was_deleted() {
    let rig = Rig::new();
    let present = rig._directory.path().join("present");
    fs::write(&present, "x").expect("write");
    assert_eq!(usable_source(present.clone()), present, "a file that exists is used as it is");
    let gone = rig._directory.path().join("deleted (deleted)");
    assert_eq!(usable_source(gone), PathBuf::from("/proc/self/exe"), "a replaced running binary is read through /proc");
}

/// A lock text naming the release tool at `version`,
/// with its prefix and first bin.
fn named_lock(version: &str) -> String {
    return format!(
        "{{\"tools\":{{\"{ID}\":{{\"request\":\"github:o/r@latest\",\"version\":\"{version}\",\"route\":\"archive\",\"bins\":[\"{BIN}\"],\"options\":{{\"tagPrefix\":\"{PREFIX}\"}}}}}}}}"
    );
}

#[test]
fn a_lock_names_only_the_release_archives_of_this_repository() {
    let named = named_archives(&named_lock("0.2.1"));
    assert_eq!(named.len(), 1);
    assert_eq!(named[ID].versions.iter().cloned().collect::<Vec<_>>(), vec!["0.2.1".to_string()]);
    assert_eq!(named[ID].first_bin, BIN);

    let third_party = r#"{"tools":{"node":{"version":"26.0.0","route":"archive","bins":["node"]},
        "bin":{"version":"1.0.0","route":"archive","bins":["bin"],"options":{"tagPrefix":"other-v"}},
        "cargo:x":{"version":"1.0.0","route":"cargo-binstall","bins":["x"],"options":{"tagPrefix":"monochromatic-x-v"}}}}"#;
    assert!(named_archives(third_party).is_empty(), "no prefix, a foreign prefix and another route name nothing");
    let typescript_format = r#"{"tools":{"node":{"request":{"track":"latest"},"version":"26.0.0","route":"archive"}}}"#;
    assert!(named_archives(typescript_format).is_empty());
    assert!(named_archives("not json").is_empty());
    assert!(named_archives("{}").is_empty());
}

fn versions_present(entries: &[(&str, bool)]) -> BTreeMap<String, Vec<PresentVersion>> {
    let mut map = BTreeMap::new();
    map.insert(
        ID.to_string(),
        entries.iter().map(|(version, finished)| return PresentVersion { version: (*version).to_string(), finished: *finished }).collect(),
    );
    return map;
}

#[test]
fn the_archive_prune_rule_keeps_the_newest_finished_version_and_never_judges_by_an_unfinished_one() {
    let removals = plan_archive_prune(&versions_present(&[("0.1.0", true), ("0.2.0", true), ("0.2.1", true)]));
    assert_eq!(removals, vec![(ID.to_string(), "0.1.0".to_string()), (ID.to_string(), "0.2.0".to_string())]);
    let installing = plan_archive_prune(&versions_present(&[("0.2.0", true), ("0.3.0", false)]));
    assert_eq!(installing, Vec::<(String, String)>::new(), "0.2.0 is the newest finished, 0.3.0 may be installing");
    let none_finished = plan_archive_prune(&versions_present(&[("0.2.0", false), ("0.2.1", false)]));
    assert!(none_finished.is_empty(), "with no finished version nothing is deleted");
}

#[test]
fn the_prune_command_removes_older_release_archives_a_lock_names_and_leaves_other_archive_tools_alone() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    for version in ["0.2.0", "0.2.1"] {
        finish_archive(&root, version);
    }
    std::os::unix::fs::symlink("0.2.0", current_link_of(&root, ID)).expect("a stale current");
    let third_party = root.join("node").join("1.0.0");
    fs::create_dir_all(&third_party).expect("make");
    fs::write(rig.scope.root.join("toolchain.lock.json"), named_lock("0.2.0")).expect("write");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);

    let planned = prune_toolchain(&rig.scope, &rig.runner, &[], true).expect("dry run");
    assert_eq!(planned, vec![tool_directory(&root, ID).join("0.2.0")]);
    assert!(tool_directory(&root, ID).join("0.2.0").is_dir(), "a dry run deletes nothing");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.0"), "and moves no link");

    let removed = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert_eq!(removed, vec![tool_directory(&root, ID).join("0.2.0")]);
    assert!(tool_directory(&root, ID).join("0.2.1").is_dir());
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"), "current follows the survivor");
    assert!(third_party.is_dir(), "an archive directory no lock names as a release archive is never judged");

    // Rejected fixture: with no lock naming the tool, its directories are not judged at all.
    let unnamed = Rig::new();
    finish_archive(&unnamed.scope.toolchain_root, "0.2.0");
    finish_archive(&unnamed.scope.toolchain_root, "0.2.1");
    fs::write(unnamed.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");
    unnamed.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", unnamed.scope.root.display()), 0);
    assert!(prune_toolchain(&unnamed.scope, &unnamed.runner, &[], false).expect("pruned").is_empty());
    assert!(tool_directory(&unnamed.scope.toolchain_root, ID).join("0.2.0").is_dir());
}
