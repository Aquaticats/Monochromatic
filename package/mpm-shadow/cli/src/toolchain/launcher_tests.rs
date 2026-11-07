//! Unit tests of the launcher's decision:
//! the record on disk,
//! and when the warning is printed.
//! Starting a program and passing its streams and exit status through are covered by `tests/launcher.rs`,
//! which runs the built binary.

use std::fs;
use std::path::{Path, PathBuf};

use super::cargo_installs::{older_lock_message, older_lock_message_for};
use super::fakes::Rig;
use super::launcher::{LaunchRecord, ensure_launcher, parse_record, prepare_launch, record_path, render_record};
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
    assert_eq!(launch.warning, Some(older_lock_message_for(ScopeKind::Global, TOOL, "0.4.0", "0.4.4")));
    assert!(launch.warning.unwrap_or_default().contains(&format!("`mpm update -g {TOOL}`")));
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
    assert_eq!(launch.warning, Some(older_lock_message(TOOL, "0.4.4", "0.4.5")), "the version behind the chain, not the word current");
}

#[test]
fn the_launcher_warns_when_the_version_that_runs_is_newer_than_the_lock() {
    let rig = Rig::new();
    let record = record_with_current(rig._directory.path(), &["0.4.4"], "0.4.4");
    let launch = prepare_launch(&record, Some(&lock_text("0.4.0")));
    assert_eq!(launch.warning, Some(older_lock_message(TOOL, "0.4.0", "0.4.4")));
    assert_eq!(launch.program, record.real);
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
