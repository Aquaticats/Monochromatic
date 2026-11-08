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

use super::archive_staging::write_complete_marker;
use super::cargo_installs::{is_repository_published, older_lock_message, older_lock_warnings};
use super::commands::older_lock_warning_lines;
use super::context::Context;
use super::fakes::{Rig, write_executable};
use super::hash::sha256_hex;
use super::launcher::{LaunchRecord, prepare_launch, warning_naming_program};
use super::lock::{Lock, render_lock};
use super::model::{Artifact, LockedTool, OsFamily, Request, Route};
use super::observe::observe_all;
use super::prune_cargo::{PresentVersion, prune_toolchain};
use super::layout::tool_directory;
use super::published_archive::{
    current_link_of, is_published_archive, named_archives, newest_installed_archive, plan_archive_prune, running_version_archive,
};
use super::runner::RunRequest;
use super::scope::{Scope, ScopeKind, repo_scope};
use super::status_cmd::run_status;
use super::sync::sync_tools;

/// The tool id.
pub(super) const ID: &str = "mpm-shadow";

/// The program the release archive holds
/// (not `mpm-shadow`,
/// which the sample plan's crate tool already exposes under that name).
pub(super) const BIN: &str = "mpm-shadow-release";

/// The tag prefix that names this program's releases.
pub(super) const PREFIX: &str = "monochromatic-mpm-shadow-v";

/// Where a release of the tool is downloaded from.
pub(super) fn release_url(version: &str) -> String {
    return format!("https://example.invalid/releases/{PREFIX}{version}/mpm-shadow-{version}.tar.gz");
}

/// The bytes of a release archive (the fake `tar` does not read them).
pub(super) fn release_body(version: &str) -> Vec<u8> {
    return format!("release archive {version}").into_bytes();
}

/// The release-archive tool at `version`,
/// with the archive row of the host.
pub(super) fn release_tool(rig: &Rig, version: &str) -> LockedTool {
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
pub(super) fn lock_at(rig: &Rig, version: &str) -> Lock {
    return Lock { tools: vec![release_tool(rig, version)] };
}

/// Makes the fake network serve a release and the fake `tar` unpack it as `<folder>/mpm-shadow`.
pub(super) fn serve_releases(rig: &Rig, versions: &[&str]) {
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
pub(super) fn tar_lines(rig: &Rig) -> Vec<String> {
    return rig.runner.lines().into_iter().filter(|line| return line.starts_with("tar --extract")).collect();
}

/// A second tree that shares the rig's home,
/// and so its toolchain root.
pub(super) fn second_tree(rig: &Rig) -> Scope {
    let root = rig._directory.path().join("work-two");
    fs::create_dir_all(&root).expect("make the second tree");
    return repo_scope(&root, &rig.env).expect("a scope");
}

/// A context for one tree of the rig.
pub(super) fn context_of<'a>(rig: &'a Rig, scope: &'a Scope, lock: &'a Lock) -> Context<'a> {
    return Context { scope, env: &rig.env, runner: &rig.runner, http: &rig.http, host: &rig.host, tools: &lock.tools };
}

/// Leaves what an unpacking that was cut short,
/// or an earlier release that unpacked in place,
/// leaves:
/// the program inside the folder an archive would hold,
/// and no completion marker.
pub(super) fn unmarked_archive(root: &Path, version: &str) -> PathBuf {
    let program = tool_directory(root, ID).expect("a plain name").join(version).join("release-folder").join(BIN);
    write_executable(&program);
    return program;
}

/// Finishes an install of `version` by hand:
/// the program inside the folder an archive would hold,
/// and the completion marker the installer writes last.
pub(super) fn finish_archive(root: &Path, version: &str) -> PathBuf {
    let program = unmarked_archive(root, version);
    write_complete_marker(&tool_directory(root, ID).expect("a plain name").join(version), ID, version).expect("write the marker");
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

    let mut other_repository = release_tool(&rig, "0.2.0");
    other_repository.declaration.request = Request::parse("github:other-owner/Monochromatic@latest").expect("parses");
    assert!(!is_published_archive(&other_repository), "the prefix of another repository's tool does not make it this repository's");
    assert!(!is_repository_published(&other_repository));

    // GitHub owner and repository names are case-insensitive, so another spelling of this repository still counts.
    for spelling in ["github:aquaticats/monochromatic@latest", "github:AQUATICATS/MONOCHROMATIC@latest", "github:aquaticats/Monochromatic@0.2.0"] {
        let mut respelled = release_tool(&rig, "0.2.0");
        respelled.declaration.request = Request::parse(spelling).expect("parses");
        assert!(is_published_archive(&respelled), "{spelling}: the same repository in another letter case");
        assert!(is_repository_published(&respelled), "{spelling}");
    }
    for spelling in [
        "github:Aquaticats/Monochromatic-fork@latest",
        "github:Aquaticats/Monochromatic2@latest",
        "github:Aquaticats/Monochromati@latest",
        "github:Aquaticats2/Monochromatic@latest",
        "github:aquaticats/monochromatic/x@latest",
    ] {
        let mut near_miss = release_tool(&rig, "0.2.0");
        near_miss.declaration.request = Request::parse(spelling).expect("parses");
        assert!(!is_published_archive(&near_miss), "{spelling}: a different repository, whatever its letter case");
    }
    let mut pinned = release_tool(&rig, "0.2.0");
    pinned.declaration.request = Request::parse("github:Aquaticats/Monochromatic@0.2.0").expect("parses");
    assert!(is_published_archive(&pinned), "the range of the request does not matter");

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
    let old_download = root.join("downloads").join(ID).join("0.2.0").join("mpm-shadow-0.2.0.tar.gz");
    assert!(old_download.is_file());

    let second = sync_tools(&context_of(&rig, &rig.scope, &lock_new), None);
    assert_eq!(second.installed, vec![ID.to_string()], "{second:?}");
    assert!(!tool_directory(&root, ID).expect("a plain name").join("0.2.0").exists(), "installing 0.2.1 removed 0.2.0");
    assert!(!old_download.exists(), "and the archive downloaded for it");
    assert!(root.join("downloads").join(ID).join("0.2.1").join("mpm-shadow-0.2.1.tar.gz").is_file());
    assert!(!root.join("downloads").join(ID).join("0.2.0").exists(), "the superseded version's download directory is gone too");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"));

    // This tree's exposed link goes through its launcher to the program behind `current`, which resolves.
    let launcher = rig.scope.launcher_dir.join(BIN);
    assert_eq!(fs::read_link(rig.scope.exposure_dir.join(BIN)).expect("a link"), launcher);
    let through_current = fs::canonicalize(current_link_of(&root, ID).expect("a plain name").join("release-folder").join(BIN)).expect("current resolves");
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
    assert!(!tool_directory(&root, ID).expect("a plain name").join("0.2.0").exists());
    assert_eq!(report_old.warnings, vec![older_lock_message(ID, "0.2.0", "0.2.1", &scope_old.lock_file)], "{report_old:?}");
    assert!(report_old.warnings[0].contains("older repository tools are never kept"));
    assert!(report_old.warnings[0].contains(&format!("`mpm update {ID}`")));

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
    fs::create_dir_all(tool_directory(&root, ID).expect("a plain name").join("0.2.2").join("release-folder")).expect("make");
    assert_eq!(newest_installed_archive(&root, ID, BIN), None, "an unfinished install is not counted");
    let before = observe_all(&context_of(&rig, &rig.scope, &lock), &lock.tools);
    assert!(!before[0].in_sync);
    assert_eq!(before[0].observed, None);

    finish_archive(&root, "0.2.1");
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some("0.2.1"), "the unfinished 0.2.2 is skipped");
    assert_eq!(newest_installed_archive(&root, ID, "").as_deref(), Some("0.2.1"), "a tool that declares no program finishes on its marker alone");
    unmarked_archive(&root, "0.2.3");
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some("0.2.1"), "a program without the marker is not finished");
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
        assert_eq!(
            lines,
            vec![format!("toolchain {verb}: warning: {}", older_lock_message(ID, "0.2.0", "0.2.1", &rig.scope.lock_file))],
            "{verb}"
        );
    }
    let report = run_status(&rig.scope, &rig.env, &rig.runner, &rig.host).expect("status runs");
    let warning = format!("toolchain list: warning: {}\n", older_lock_message(ID, "0.2.0", "0.2.1", &rig.scope.lock_file));
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
        scope: ScopeKind::Repo,
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
        assert_eq!(
            launch.warning,
            Some(warning_naming_program(&older_lock_message(ID, "0.2.0", "0.2.1", &record.lock_file), &record.real)),
            "depth {depth}"
        );
        assert_eq!(prepare_launch(&record, Some(&lock_text("0.2.1"))).warning, None, "depth {depth}: matching");
    }
    let unlinked = LaunchRecord {
        tool_id: ID.to_string(),
        lock_file: rig._directory.path().join("toolchain.lock.json"),
        real: rig._directory.path().join("elsewhere").join("folder").join(BIN),
        scope: ScopeKind::Repo,
    };
    assert_eq!(prepare_launch(&unlinked, Some(&lock_text("0.1.0"))).warning, None, "no current link above the program");
}

#[test]
fn an_archive_folder_named_current_does_not_hide_the_link_that_names_the_version() {
    let rig = Rig::new();
    let tool_dir = rig._directory.path().join("folder-named-current").join(ID);
    fs::create_dir_all(tool_dir.join("0.2.1").join("current")).expect("make the version and its folder");
    std::os::unix::fs::symlink("0.2.1", tool_dir.join("current")).expect("link current");
    let record = LaunchRecord {
        tool_id: ID.to_string(),
        lock_file: rig._directory.path().join("toolchain.lock.json"),
        real: tool_dir.join("current").join("current").join(BIN),
        scope: ScopeKind::Repo,
    };
    let launch = prepare_launch(&record, Some(&lock_text("0.2.0")));
    assert_eq!(
        launch.warning,
        Some(warning_naming_program(&older_lock_message(ID, "0.2.0", "0.2.1", &record.lock_file), &record.real)),
        "the nearest ancestor that is a link wins, not the folder"
    );
    assert_eq!(prepare_launch(&record, Some(&lock_text("0.2.1"))).warning, None);
}

/// A lock text naming the release tool at `version`,
/// with its prefix and first bin.
pub(super) fn named_lock(version: &str) -> String {
    return format!(
        "{{\"tools\":{{\"{ID}\":{{\"request\":\"github:Aquaticats/Monochromatic@latest\",\"version\":\"{version}\",\"route\":\"archive\",\"bins\":[\"{BIN}\"],\"options\":{{\"tagPrefix\":\"{PREFIX}\"}}}}}}}}"
    );
}

#[test]
fn a_lock_names_only_the_release_archives_of_this_repository() {
    let named = named_archives(&named_lock("0.2.1"), "the lock").expect("a lock that names a release archive");
    assert_eq!(named.len(), 1);
    assert_eq!(named[ID].versions.iter().cloned().collect::<Vec<_>>(), vec!["0.2.1".to_string()]);
    assert_eq!(named[ID].first_bin, BIN);

    for spelling in ["aquaticats/monochromatic", "AQUATICATS/MONOCHROMATIC"] {
        let respelled = named_lock("0.2.1").replace("Aquaticats/Monochromatic", spelling);
        let in_other_case = named_archives(&respelled, "the lock").expect("reads");
        assert_eq!(in_other_case.keys().cloned().collect::<Vec<_>>(), vec![ID.to_string()], "{spelling}: the same repository in another letter case");
    }
    let other_repository = named_lock("0.2.1").replace("Aquaticats/Monochromatic", "other/repo");
    assert!(named_archives(&other_repository, "the lock").expect("reads").is_empty(), "a prefix of this repository's style on another repository's tool");
    let no_request = named_lock("0.2.1").replace("\"request\":\"github:Aquaticats/Monochromatic@latest\",", "");
    assert!(named_archives(&no_request, "the lock").expect("reads").is_empty(), "a lock entry without a request names nothing");

    let third_party = r#"{"tools":{"node":{"version":"26.0.0","route":"archive","bins":["node"]},
        "bin":{"version":"1.0.0","route":"archive","bins":["bin"],"options":{"tagPrefix":"other-v"}},
        "cargo:x":{"version":"1.0.0","route":"cargo-binstall","bins":["x"],"options":{"tagPrefix":"monochromatic-x-v"}}}}"#;
    assert!(named_archives(third_party, "the lock").expect("reads").is_empty(), "no prefix, a foreign prefix and another route name nothing");
    let typescript_format = r#"{"tools":{"node":{"request":{"track":"latest"},"version":"26.0.0","route":"archive"}}}"#;
    assert!(named_archives(typescript_format, "the lock").expect("reads").is_empty());
    assert!(named_archives("not json", "the lock").expect("reads").is_empty());
    assert!(named_archives("{}", "the lock").expect("reads").is_empty());
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
    std::os::unix::fs::symlink("0.2.0", current_link_of(&root, ID).expect("a plain name")).expect("a stale current");
    let third_party = root.join("node").join("1.0.0");
    fs::create_dir_all(&third_party).expect("make");
    fs::write(rig.scope.root.join("toolchain.lock.json"), named_lock("0.2.0")).expect("write");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);

    let planned = prune_toolchain(&rig.scope, &rig.runner, &[], true).expect("dry run");
    assert_eq!(planned, vec![tool_directory(&root, ID).expect("a plain name").join("0.2.0")]);
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.0").is_dir(), "a dry run deletes nothing");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.0"), "and moves no link");

    let removed = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned");
    assert_eq!(removed, vec![tool_directory(&root, ID).expect("a plain name").join("0.2.0")]);
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.1").is_dir());
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"), "current follows the survivor");
    assert!(third_party.is_dir(), "an archive directory no lock names as a release archive is never judged");

    // Rejected fixture: with no lock naming the tool, its directories are not judged at all.
    let unnamed = Rig::new();
    finish_archive(&unnamed.scope.toolchain_root, "0.2.0");
    finish_archive(&unnamed.scope.toolchain_root, "0.2.1");
    fs::write(unnamed.scope.root.join("toolchain.lock.json"), "{\"tools\":{}}").expect("write");
    unnamed.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", unnamed.scope.root.display()), 0);
    assert!(prune_toolchain(&unnamed.scope, &unnamed.runner, &[], false).expect("pruned").is_empty());
    assert!(tool_directory(&unnamed.scope.toolchain_root, ID).expect("a plain name").join("0.2.0").is_dir());
}

/// The release-archive tool at `version`,
/// but of another repository that tags its releases with this repository's style of prefix.
fn other_repository_tool(rig: &Rig, version: &str) -> LockedTool {
    let mut tool = release_tool(rig, version);
    tool.declaration.request = Request::parse("github:other-owner/other-repo@latest").expect("parses");
    return tool;
}

#[test]
fn a_tool_of_another_repository_with_this_repositorys_style_of_prefix_keeps_its_versions_and_is_never_warned_about() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.0", "0.2.1"]);
    let root = rig.scope.toolchain_root.clone();
    let old_lock = Lock { tools: vec![other_repository_tool(&rig, "0.2.0")] };
    let new_lock = Lock { tools: vec![other_repository_tool(&rig, "0.2.1")] };

    sync_tools(&context_of(&rig, &rig.scope, &old_lock), None);
    let report = sync_tools(&context_of(&rig, &rig.scope, &new_lock), None);
    assert_eq!(report.installed, vec![ID.to_string()], "{report:?}");
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.0").is_dir(), "installing 0.2.1 removes no older version");
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.1").is_dir());
    assert_eq!(running_version_archive(&root, ID), None, "no current link is made for it");

    // A newer install does not satisfy an older pin as drift-free, and no warning names the tool.
    let behind = observe_all(&context_of(&rig, &rig.scope, &old_lock), &old_lock.tools);
    assert_eq!(behind[0].observed.as_deref(), Some("0.2.0"), "the locked version's own directory is what is read");
    let warnings = older_lock_warnings(&rig.scope, &old_lock.tools, Some(OsFamily::Linux));
    assert!(warnings.is_empty(), "{warnings:?}");

    // The cleanup does not judge it, even when a known lock names it with the same prefix.
    let named = named_lock("0.2.1").replace("Aquaticats/Monochromatic", "other-owner/other-repo");
    fs::write(rig.scope.root.join("toolchain.lock.json"), named).expect("write");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);
    assert!(prune_toolchain(&rig.scope, &rig.runner, &[], false).expect("pruned").is_empty());
    assert!(tool_directory(&root, ID).expect("a plain name").join("0.2.0").is_dir());
}

#[test]
fn a_tool_whose_request_spells_this_repository_in_another_case_is_kept_newest_only() {
    let rig = Rig::new();
    serve_releases(&rig, &["0.2.0", "0.2.1"]);
    let root = rig.scope.toolchain_root.clone();
    let respelled = |version: &str| {
        let mut tool = release_tool(&rig, version);
        tool.declaration.request = Request::parse("github:aquaticats/monochromatic@latest").expect("parses");
        return Lock { tools: vec![tool] };
    };
    let old_lock = respelled("0.2.0");
    let new_lock = respelled("0.2.1");
    sync_tools(&context_of(&rig, &rig.scope, &old_lock), None);
    let report = sync_tools(&context_of(&rig, &rig.scope, &new_lock), None);
    assert_eq!(report.installed, vec![ID.to_string()], "{report:?}");
    assert!(!tool_directory(&root, ID).expect("a plain name").join("0.2.0").exists(), "installing 0.2.1 removed 0.2.0");
    assert_eq!(running_version_archive(&root, ID).as_deref(), Some("0.2.1"));

    // A tree pinning the older version is warned, as for the exact spelling.
    let warnings = older_lock_warnings(&rig.scope, &old_lock.tools, Some(OsFamily::Linux));
    assert_eq!(warnings, vec![older_lock_message(ID, "0.2.0", "0.2.1", &rig.scope.lock_file)]);
}

/// Two pre-releases of one version whose numbers differ in length:
/// `.10` is newer than `.9` by semver.
const EARLIER_ALPHA: &str = "0.2.2-alpha.9";

/// The newer pre-release of the same version as `EARLIER_ALPHA`.
const LATER_ALPHA: &str = "0.2.2-alpha.10";

#[test]
fn the_newest_archive_and_the_removal_of_older_ones_read_a_longer_alpha_number_as_the_newer_one() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    for version in [EARLIER_ALPHA, LATER_ALPHA] {
        finish_archive(&root, version);
    }
    assert_eq!(newest_installed_archive(&root, ID, BIN).as_deref(), Some(LATER_ALPHA));
    let removed = super::published_archive::remove_older_archives(&root, ID, BIN).expect("removed");
    assert_eq!(removed.len(), 1, "{removed:?}");
    assert!(removed[0].ends_with(EARLIER_ALPHA), "{removed:?}");
}

#[test]
fn the_archive_prune_rule_keeps_the_longer_alpha_number() {
    let removals = plan_archive_prune(&versions_present(&[(EARLIER_ALPHA, true), (LATER_ALPHA, true)]));
    assert_eq!(removals, vec![(ID.to_string(), EARLIER_ALPHA.to_string())]);
}
