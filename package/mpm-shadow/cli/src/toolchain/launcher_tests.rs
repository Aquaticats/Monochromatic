//! Unit tests of the launcher's decision:
//! the record on disk,
//! and when the warning is printed.
//! Starting a program and passing its streams and exit status through are covered by `tests/launcher.rs`,
//! which runs the built binary.

use std::fs;
use std::path::{Path, PathBuf};

use super::cargo_installs::{older_lock_message, older_lock_message_for};
use super::fakes::Rig;
use super::launcher::{LaunchRecord, ensure_launcher, parse_record, prepare_launch, record_path, render_record, warning_naming_program};
use super::scope::ScopeKind;

/// The id the tests use for the tool.
const TOOL: &str = "cargo:monochromatic-git-policy-cli";

/// A lock text pinning the tool at `version`.
fn lock_text(version: &str) -> String {
    return format!("{{\"schemaVersion\":1,\"tools\":{{\"{TOOL}\":{{\"version\":\"{version}\",\"route\":\"cargo-binstall\"}}}}}}");
}

/// A crate directory with versions `installed` and `current` pointing at `current`,
/// and the record a launcher in it would carry.
fn record_with_current(directory: &Path, installed: &[&str], current: &str) -> LaunchRecord {
    let crate_dir: PathBuf = directory.join("cargo").join("monochromatic-git-policy-cli");
    for version in installed {
        fs::create_dir_all(crate_dir.join(version).join("bin")).expect("make");
    }
    std::os::unix::fs::symlink(current, crate_dir.join("current")).expect("link current");
    return LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: directory.join("toolchain.lock.json"),
        real: crate_dir.join("current").join("bin").join("cli-git-native"),
        scope: ScopeKind::Repo,
    };
}

#[test]
fn a_record_round_trips_and_a_bad_one_is_refused() {
    let record = LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: PathBuf::from("/work/toolchain.lock.json"),
        real: PathBuf::from("/root/cargo/x/current/bin/x"),
        scope: ScopeKind::Repo,
    };
    assert_eq!(parse_record(&render_record(&record)), Some(record));
    assert_eq!(parse_record("not json"), None);
    assert_eq!(parse_record("{\"toolId\":\"a\"}"), None);
    assert_eq!(record_path(Path::new("/a/b/git")), PathBuf::from("/a/b/git.launch"));
}

#[test]
fn a_global_record_round_trips_and_a_record_without_a_scope_belongs_to_a_tree() {
    let record = LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: PathBuf::from("/config/toolchain.lock.json"),
        real: PathBuf::from("/root/cargo/x/current/bin/x"),
        scope: ScopeKind::Global,
    };
    assert_eq!(parse_record(&render_record(&record)), Some(record));
    let older = parse_record("{\"toolId\":\"a\",\"lockFile\":\"/l\",\"real\":\"/r\"}").expect("a record written before the field existed");
    assert_eq!(older.scope, ScopeKind::Repo);
}

#[test]
fn a_global_launcher_warns_with_the_global_upgrade_command() {
    let rig = Rig::new();
    let record = LaunchRecord { scope: ScopeKind::Global, ..record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4") };
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    assert_eq!(
        launch.warning,
        Some(warning_naming_program(&older_lock_message_for(ScopeKind::Global, TOOL, "0.4.0", "0.4.4", &record.lock_file), &record.real))
    );
    assert!(launch.warning.unwrap_or_default().contains(&format!("`mpm-shadow update -g {TOOL}`")));
}

#[test]
fn a_current_link_that_forwards_to_the_other_roots_current_still_names_the_version_that_runs() {
    let rig = Rig::new();
    let winner = record_with_current(&rig._directory.path().join("winner-root"), &["0.4.5"], "0.4.5");
    let emptied_crate = rig._directory.path().join("emptied-root").join("cargo").join("monochromatic-git-policy-cli");
    fs::create_dir_all(&emptied_crate).expect("make the emptied root's crate directory");
    std::os::unix::fs::symlink(winner.real.parent().and_then(Path::parent).expect("the winner's current link"), emptied_crate.join("current"))
        .expect("forward current");
    let record = LaunchRecord { real: emptied_crate.join("current").join("bin").join("cli-git-native"), ..winner };
    let launch = prepare_launch(&record, Some(&lock_text("0.4.4")));
    assert_eq!(
        launch.warning,
        Some(warning_naming_program(&older_lock_message(TOOL, "0.4.4", "0.4.5", &record.lock_file), &record.real)),
        "the version behind the chain, not the word current"
    );
}

#[test]
fn the_launcher_warns_when_the_version_that_runs_is_newer_than_the_lock() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    assert_eq!(launch.warning, Some(warning_naming_program(&older_lock_message(TOOL, "0.4.0", "0.4.4", &record.lock_file), &record.real)));
    assert_eq!(launch.program, record.real);
}

#[test]
fn the_older_lock_warning_names_the_lock_file_it_compared() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    let warning = launch.warning.expect("the lock is older than the version that runs");
    assert!(
        warning.contains(&record.lock_file.display().to_string()),
        "the warning names the lock file it read, so a reader can act on it: {warning}"
    );
}

#[test]
fn the_launcher_is_silent_when_the_lock_and_the_version_that_runs_match() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    assert_eq!(prepare_launch(&record, Some(&lock_text("0.4.4"))).warning, None);
}

#[test]
fn the_launcher_never_blocks_a_run_for_an_unreadable_lock_a_missing_tool_or_a_missing_link() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    assert_eq!(prepare_launch(&record, None).warning, None, "no lock file");
    assert_eq!(prepare_launch(&record, Some("{")).warning, None, "a broken lock");
    assert_eq!(prepare_launch(&record, Some("{\"tools\":{}}")).warning, None, "the tool is not in the lock");
    let unlinked = LaunchRecord { real: rig._directory.path().join("elsewhere").join("bin").join("x"), ..record };
    assert_eq!(prepare_launch(&unlinked, Some(&lock_text("0.1.0"))).warning, None, "no current link");
}

#[test]
fn a_launcher_is_made_with_its_record_beside_it() {
    let rig = Rig::new();
    let real = rig.scope.toolchain_root.join("cargo").join("x").join("current").join("bin").join("x");
    let launcher = ensure_launcher(&rig.scope, TOOL, "git", &real).expect("made");
    assert_eq!(launcher, rig.scope.launcher_dir.join("git"));
    assert!(launcher.is_file());
    let record = parse_record(&fs::read_to_string(record_path(&launcher)).expect("a record")).expect("parses");
    assert_eq!(record.real, real);
    assert_eq!(record.lock_file, rig.scope.lock_file);
    // A second call (the exposure link and a postinstall link both ask) leaves no temporary name behind.
    ensure_launcher(&rig.scope, TOOL, "git", &real).expect("made again");
    let mut names: Vec<String> = fs::read_dir(&rig.scope.launcher_dir)
        .expect("list")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["git".to_string(), "git.launch".to_string()]);
}

/// The environment variable that makes a copy of the test binary act as the child of the test below;
/// it holds the file the child writes its report to,
/// and its presence is the guard against the child starting another child.
#[cfg(target_os = "linux")]
const DELETED_BINARY_CHILD: &str = "MPM_SHADOW_TEST_DELETED_BINARY_CHILD";

/// The full test name the child is asked to run.
#[cfg(target_os = "linux")]
const DELETED_BINARY_TEST: &str = "toolchain::launcher_tests::a_launcher_is_made_from_the_running_binary_after_its_file_was_deleted";

/// The child half of the test below:
/// deletes the file this process runs from,
/// asks `ensure_launcher` for a launcher,
/// and reports the launcher's size.
#[cfg(target_os = "linux")]
fn make_a_launcher_after_deleting_the_running_binary(report: &Path) {
    let rig = Rig::new();
    let running = std::env::current_exe().expect("the running binary");
    fs::remove_file(&running).expect("delete the running binary");
    assert!(!running.exists(), "the file the process runs from is gone");
    let real = rig.scope.toolchain_root.join("cargo").join("x").join("current").join("bin").join("x");
    let launcher = ensure_launcher(&rig.scope, TOOL, "git", &real).expect("a launcher is made through /proc/self/exe");
    fs::write(report, fs::metadata(&launcher).expect("the launcher exists").len().to_string()).expect("write the report");
}

#[cfg(target_os = "linux")]
#[test]
fn a_launcher_is_made_from_the_running_binary_after_its_file_was_deleted() {
    if let Ok(report) = std::env::var(DELETED_BINARY_CHILD) {
        make_a_launcher_after_deleting_the_running_binary(Path::new(&report));
        return;
    }
    // The parent copies this test binary, starts the copy as the child and lets it delete itself.
    let rig = Rig::new();
    let own = std::env::current_exe().expect("the test binary");
    let copy = rig._directory.path().join("test-binary-copy");
    fs::copy(&own, &copy).expect("copy the test binary");
    let report = rig._directory.path().join("child-report");
    let output = std::process::Command::new(&copy)
        .args(["--exact", DELETED_BINARY_TEST, "--nocapture", "--test-threads", "1"])
        .env(DELETED_BINARY_CHILD, &report)
        .output()
        .expect("start the copy");
    assert!(
        output.status.success(),
        "the child failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let launcher_size: u64 = fs::read_to_string(&report).expect("the child reported").parse().expect("a size");
    assert_eq!(launcher_size, fs::metadata(&own).expect("size").len(), "the launcher is a whole copy of the deleted binary");
}

/// A release archive tool (the folder below `current` carries the version) whose record names `recorded`,
/// with `current` now pointing at `now` and the folder of `recorded` deleted.
fn moved_archive_record(directory: &Path, recorded: &str, now: &str) -> LaunchRecord {
    let tool_dir: PathBuf = directory.join("demo-release");
    for version in [recorded, now] {
        fs::create_dir_all(tool_dir.join(version).join(format!("demo-release-{version}-linux"))).expect("make");
    }
    let program: PathBuf = tool_dir.join(now).join(format!("demo-release-{now}-linux")).join("tool");
    fs::write(&program, "x").expect("write");
    fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("make the program executable");
    std::os::unix::fs::symlink(now, tool_dir.join("current")).expect("link current");
    fs::remove_dir_all(tool_dir.join(recorded)).expect("remove the recorded version");
    return LaunchRecord {
        tool_id: "demo-release".to_string(),
        lock_file: directory.join("toolchain.lock.json"),
        real: tool_dir.join("current").join(format!("demo-release-{recorded}-linux")).join("tool"),
        scope: ScopeKind::Repo,
    };
}

#[test]
fn a_recorded_program_that_was_deleted_is_replaced_by_the_one_under_current() {
    let rig = Rig::new();
    let record = moved_archive_record(rig._directory.path(), "0.1.4", "0.1.5");
    let launch = prepare_launch(&record, Some("{\"schemaVersion\":1,\"tools\":{\"demo-release\":{\"version\":\"0.1.4\"}}}"));
    assert_eq!(launch.program, rig._directory.path().join("demo-release").join("current").join("demo-release-0.1.5-linux").join("tool"));
    assert!(launch.warning.is_some_and(|text| return text.contains("0.1.4") && text.contains("0.1.5")));
}

#[test]
fn a_recorded_program_that_still_exists_is_never_replaced() {
    let rig = Rig::new();
    let record = moved_archive_record(rig._directory.path(), "0.1.4", "0.1.5");
    let kept = rig._directory.path().join("demo-release").join("current").join("demo-release-0.1.5-linux").join("tool");
    let mut named = record;
    named.real = kept.clone();
    assert_eq!(prepare_launch(&named, None).program, kept);
}

#[test]
fn a_recorded_program_with_nothing_under_current_stays_as_recorded() {
    let rig = Rig::new();
    let record = moved_archive_record(rig._directory.path(), "0.1.4", "0.1.5");
    fs::remove_file(rig._directory.path().join("demo-release").join("0.1.5").join("demo-release-0.1.5-linux").join("tool")).expect("remove");
    assert_eq!(prepare_launch(&record, None).program, record.real);
}

/// Records an identity naming the rig's worktree as the repository,
/// and installs a check program that says `answer` and exits `status`.
fn repository_on_a_drive_that_is_gone(rig: &Rig, answer: &str, status: u32) {
    use std::os::unix::fs::PermissionsExt;
    let identity: PathBuf = rig.scope.home.join(".config/monochromatic-dev/repository-link.json");
    fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    fs::write(&identity, format!("{{\"linkPath\":\"{}\",\"canonicalRoot\":\"/nowhere/Monochromatic\",\"phase\":\"after\"}}", rig.scope.root.display())).expect("identity");
    let check: PathBuf = rig.scope.home.join(".local/share/monochromatic-dev/repository-link/repository-link-check");
    fs::create_dir_all(check.parent().expect("parent")).expect("install folder");
    fs::write(&check, format!("#!/bin/sh\necho '{answer}' >&2\nexit {status}\n")).expect("check program");
    fs::set_permissions(&check, fs::Permissions::from_mode(0o755)).expect("chmod");
}

/// Installing a launcher under a repository whose link to the drive does not resolve fails with the check's plain message,
/// before anything is written.
#[test]
fn a_launcher_is_not_made_under_a_repository_that_is_not_there() {
    let rig = Rig::new();
    repository_on_a_drive_that_is_gone(&rig, "the 860 drive is not mounted yet", 75);
    let real = rig.scope.toolchain_root.join("cargo").join("x").join("current").join("bin").join("x");
    let error = ensure_launcher(&rig.scope, TOOL, "git", &real).expect_err("refused");
    assert!(error.to_string().contains("the 860 drive is not mounted yet"), "{error}");
    assert!(!rig.scope.launcher_dir.exists(), "nothing was created");
}

/// A repository the check says is usable gets its launcher as before.
#[test]
fn a_launcher_is_made_under_a_repository_the_check_says_is_ready() {
    let rig = Rig::new();
    repository_on_a_drive_that_is_gone(&rig, "ready", 0);
    let real = rig.scope.toolchain_root.join("cargo").join("x").join("current").join("bin").join("x");
    assert!(ensure_launcher(&rig.scope, TOOL, "git", &real).is_ok());
}

#[test]
fn the_warning_names_the_program_under_current_when_the_recorded_folder_was_deleted() {
    let rig = Rig::new();
    let directory: &Path = rig._directory.path();
    let crate_dir: PathBuf = directory.join("cargo").join("monochromatic-git-policy-cli");
    let recorded: PathBuf = crate_dir.join("current").join("0.4.0-folder").join("cli-git-native");
    let now_folder: PathBuf = crate_dir.join("0.4.4").join("0.4.4-folder");
    fs::create_dir_all(&now_folder).expect("make the folder of the version that runs");
    fs::write(now_folder.join("cli-git-native"), "#!/bin/sh\n").expect("write the program");
    fs::set_permissions(now_folder.join("cli-git-native"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
    std::os::unix::fs::symlink("0.4.4", crate_dir.join("current")).expect("link current");
    let record = LaunchRecord {
        tool_id: TOOL.to_string(),
        lock_file: directory.join("toolchain.lock.json"),
        real: recorded.clone(),
        scope: ScopeKind::Global,
    };
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    let started: PathBuf = crate_dir.join("current").join("0.4.4-folder").join("cli-git-native");
    assert_eq!(launch.program, started);
    let warning: String = launch.warning.expect("the lock is older than the version that runs");
    assert!(warning.ends_with(&format!("; starting {}", started.display())), "{warning}");
    assert!(!warning.contains(&recorded.display().to_string()), "the deleted path is never named as the one started: {warning}");
}

/// The environment variable that makes a copy of the test binary act as the installed `mpm-shadow` of the test below;
/// it holds the file the child writes the launcher's bytes to.
#[cfg(target_os = "linux")]
const INSTALLED_SHADOW_CHILD: &str = "MPM_SHADOW_TEST_INSTALLED_SHADOW_CHILD";

/// The worktree root the child builds its scope from.
#[cfg(target_os = "linux")]
const INSTALLED_SHADOW_ROOT: &str = "MPM_SHADOW_TEST_INSTALLED_SHADOW_ROOT";

/// The full test name the child is asked to run.
#[cfg(target_os = "linux")]
const INSTALLED_SHADOW_TEST: &str = "toolchain::launcher_tests::a_launcher_is_a_copy_of_the_newest_installed_mpm_shadow";

/// The bytes of the newest installed `mpm-shadow` the test below places.
#[cfg(target_os = "linux")]
const NEWEST_SHADOW: &[u8] = b"the newest installed mpm-shadow\n";

/// Where an install of `mpm-shadow` at `version` puts the program,
/// in the layout a release archive unpacks into.
#[cfg(target_os = "linux")]
fn shadow_install_program(toolchain_root: &Path, version: &str) -> PathBuf {
    return toolchain_root.join("mpm-shadow").join(version).join(format!("monochromatic-mpm-shadow-{version}-linux")).join("mpm-shadow");
}

/// Writes `content` as the program of `mpm-shadow` at `version` in `toolchain_root`
/// and points that root's `current` link at the version.
#[cfg(target_os = "linux")]
fn install_shadow(toolchain_root: &Path, version: &str, content: &[u8]) -> PathBuf {
    let program: PathBuf = shadow_install_program(toolchain_root, version);
    fs::create_dir_all(program.parent().expect("a parent")).expect("make the install");
    fs::write(&program, content).expect("write the program");
    fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("make the program executable");
    // What the install leaves beside the folder: a link named after the program, which the search finds through `current`.
    let named: PathBuf = toolchain_root.join("mpm-shadow").join(version).join("mpm-shadow");
    let _ = fs::remove_file(&named);
    std::os::unix::fs::symlink(format!("monochromatic-mpm-shadow-{version}-linux/mpm-shadow"), &named).expect("link the program");
    let link: PathBuf = toolchain_root.join("mpm-shadow").join("current");
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(version, &link).expect("link current");
    return program;
}

/// The child half of the test below:
/// asks `ensure_launcher` for a launcher and writes the bytes it placed.
#[cfg(target_os = "linux")]
fn make_a_launcher_from_wherever_it_runs(report: &Path) {
    use super::scope::{EnvMap, Scope, repo_scope};
    let env: EnvMap = EnvMap::from_process();
    let root: PathBuf = PathBuf::from(env.get(INSTALLED_SHADOW_ROOT).expect("the worktree root this child is told about"));
    let scope: Scope = repo_scope(&root, &env).expect("a scope");
    let real: PathBuf = scope.toolchain_root.join("demo").join("current").join("bin").join("tool");
    let launcher: PathBuf = ensure_launcher(&scope, TOOL, "git", &real).expect("a launcher is made");
    fs::write(report, fs::read(&launcher).expect("the launcher's bytes")).expect("write the report");
}

/// What:
/// A launcher is a copy of the newest installed `mpm-shadow`,
/// not of the `mpm-shadow` that ran `install`.
/// Why:
/// The installer is whichever copy the tree (or the machine) ran,
/// so a launcher copied from it can be older than the newest installed shadow
/// and then follows no `current` link and never rewrites its record;
/// that is how the machine-global `git` launcher stayed a 0.2.7-alpha.2 copy and started a deleted 0.6.0 folder.
/// The child runs from an old version folder of the tree's own root while `current` names a newer one,
/// which sits in the tree's root first and in the machine-global root second.
#[cfg(target_os = "linux")]
#[test]
fn a_launcher_is_a_copy_of_the_newest_installed_mpm_shadow() {
    if let Ok(report) = std::env::var(INSTALLED_SHADOW_CHILD) {
        make_a_launcher_from_wherever_it_runs(Path::new(&report));
        return;
    }
    for newest_in_the_other_root in [false, true] {
        let rig = Rig::new();
        let own_root: PathBuf = rig.scope.toolchain_root.clone();
        let other_root: PathBuf = rig.scope.other_toolchain_root.clone();
        let newest_root: &Path = if newest_in_the_other_root { &other_root } else { &own_root };
        let newest: PathBuf = install_shadow(newest_root, "0.2.1", NEWEST_SHADOW);
        // The running copy: this test binary, placed where an older install of the same tool lives.
        let running: PathBuf = shadow_install_program(&own_root, "0.2.0");
        fs::create_dir_all(running.parent().expect("a parent")).expect("make the old install");
        fs::copy(std::env::current_exe().expect("the test binary"), &running).expect("copy the test binary");
        fs::set_permissions(&running, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("make it executable");
        if newest_in_the_other_root {
            std::os::unix::fs::symlink("0.2.0", own_root.join("mpm-shadow").join("current")).expect("the running copy is what this root's current names");
        }
        let report: PathBuf = rig._directory.path().join("launcher-report");
        let output: std::process::Output = std::process::Command::new(&running)
            .args(["--exact", INSTALLED_SHADOW_TEST, "--nocapture", "--test-threads", "1"])
            .env(INSTALLED_SHADOW_CHILD, &report)
            .env(INSTALLED_SHADOW_ROOT, &rig.scope.root)
            .env("HOME", &rig.scope.home)
            .output()
            .expect("start the copy");
        assert!(
            output.status.success(),
            "the child failed with {:?} (newest in the other root: {newest_in_the_other_root}):\n{}\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::read(&report).expect("the child reported the launcher's bytes"),
            NEWEST_SHADOW,
            "the launcher is a copy of {}, not of the installer that ran (newest in the other root: {newest_in_the_other_root})",
            newest.display()
        );
        let launcher: PathBuf = rig.scope.launcher_dir.join("git");
        assert!(
            !fs::symlink_metadata(&launcher).expect("the launcher exists").file_type().is_symlink(),
            "a launcher is the program's own file, never a link whose relative target resolves beside the launcher"
        );
    }
}
