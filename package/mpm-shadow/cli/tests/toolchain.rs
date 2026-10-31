//! End-to-end tests of the toolchain verbs of the built `mpm-shadow` binary,
//! run against a disposable worktree and a
//! disposable home.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;

use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The plan the tests write:
/// two tools,
/// one with a pinned version so no network is needed.
const PLAN: &str = r#"{
  // Two tools: a pinned archive and a binstall crate.
  "schemaVersion": 1,
  "tools": {
    "bin": { "request": "github:marcosnils/bin@0.29.4", "route": "archive", "bins": ["bin"] },
    "cargo:fd-find": { "request": "cargo:fd-find@latest", "route": "cargo-binstall", "packageId": "fd-find", "bins": ["fd"] },
  },
}
"#;

/// A lock without a digest,
/// as the TypeScript tool wrote it.
const OLD_LOCK: &str = r#"{
  "schemaVersion": 1,
  "generatedFrom": "package/file-enforcer/core/src/toolchain/plan.ts",
  "tools": {
    "bin": {
      "request": "github:marcosnils/bin@0.29.4", "version": "0.29.4", "route": "archive", "bins": ["bin"], "env": {}, "options": {},
      "artifacts": { "linux-x64": { "url": "https://example.invalid/bin.tar.gz", "sha256": "439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791" } }
    },
    "cargo:fd-find": {
      "request": "cargo:fd-find@latest", "version": "10.5.0", "route": "cargo-binstall", "packageId": "fd-find", "bins": ["fd"], "env": {}, "options": {}, "artifacts": {}
    }
  }
}
"#;

struct Fixture {
    directory: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        fs::create_dir_all(directory.path().join("home")).expect("make the home");
        fs::create_dir_all(directory.path().join("work").join("sub")).expect("make the worktree");
        fs::write(directory.path().join("work").join("toolchain.plan.jsonc"), PLAN).expect("write the plan");
        fs::write(directory.path().join("work").join("toolchain.lock.json"), OLD_LOCK).expect("write the lock");
        return Fixture { directory };
    }

    /// Rewrites the lock in `directory` with its digest, the way `update` would have written it.
    fn sign_lock(&self, directory: &Path) {
        let path = directory.join("toolchain.lock.json");
        let parsed = parse_lock(&fs::read_to_string(&path).expect("read the lock"), "lock").expect("the fixture lock parses");
        fs::write(&path, render_lock(&parsed.lock)).expect("write the signed lock");
    }

    fn work(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    fn run(&self, arguments: &[&str]) -> Output {
        return self.run_in(&self.work().join("sub"), arguments);
    }

    fn run_in(&self, directory: &Path, arguments: &[&str]) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(arguments)
            .current_dir(directory)
            .env_clear()
            .env("HOME", self.directory.path().join("home"))
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run mpm-shadow");
    }
}

fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

#[test]
fn lock_check_refuses_a_lock_without_a_digest_and_passes_once_it_is_signed() {
    let fixture = Fixture::new();
    let refused = fixture.run(&["lock", "--check"]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("no digest"), "{}", text(&refused.stderr));

    fixture.sign_lock(&fixture.work());
    let checked = fixture.run(&["lock", "--check"]);
    assert_eq!(checked.status.code(), Some(0), "{}", text(&checked.stderr));
    assert!(text(&checked.stdout).contains("digest verified"), "{}", text(&checked.stdout));
}

#[test]
fn lock_after_a_plan_edit_keeps_the_resolved_versions_and_passes_the_gate() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join("toolchain.plan.jsonc"), edited).expect("edit the plan");
    assert_eq!(fixture.run(&["lock", "--check"]).status.code(), Some(1));
    let rewritten = fixture.run(&["lock"]);
    assert_eq!(rewritten.status.code(), Some(0), "{}", text(&rewritten.stderr));
    let lock: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fixture.work().join("toolchain.lock.json")).expect("read")).expect("JSON");
    assert_eq!(lock["tools"]["cargo:fd-find"]["version"], "10.5.0");
    assert_eq!(lock["tools"]["cargo:fd-find"]["bins"][1], "fdfind");
    assert_eq!(fixture.run(&["lock", "--check"]).status.code(), Some(0));
}

#[test]
fn a_broken_plan_exits_with_code_two_naming_the_file() {
    let fixture = Fixture::new();
    fs::write(fixture.work().join("toolchain.plan.jsonc"), "{ \"tools\": ").expect("break the plan");
    let output = fixture.run(&["lock", "--check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("toolchain.plan.jsonc"), "{}", text(&output.stderr));
}

#[test]
fn outside_a_worktree_the_command_names_what_to_do() {
    let fixture = Fixture::new();
    let lonely = fixture.directory.path().join("lonely");
    fs::create_dir_all(&lonely).expect("make the directory");
    let output = fixture.run_in(&lonely, &["lock", "--check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("MONOCHROMATIC_WORKTREE_ROOT"), "{}", text(&output.stderr));
}

#[test]
fn global_reads_the_per_user_config_directory() {
    let fixture = Fixture::new();
    let output = fixture.run(&["lock", "--check", "--global"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        text(&output.stderr).contains(".config/monochromatic-dev/toolchain/toolchain.plan.jsonc"),
        "{}",
        text(&output.stderr)
    );
    let config = fixture.directory.path().join("home").join(".config").join("monochromatic-dev").join("toolchain");
    fs::create_dir_all(&config).expect("make the config directory");
    fs::write(config.join("toolchain.plan.jsonc"), PLAN).expect("write the global plan");
    fs::write(config.join("toolchain.lock.json"), OLD_LOCK).expect("write the global lock");
    fixture.sign_lock(&config);
    assert_eq!(fixture.run(&["lock", "--check", "--global"]).status.code(), Some(0));
}

#[test]
fn bin_paths_prints_the_pinned_shape_for_the_ide_launcher() {
    let fixture = Fixture::new();
    let exposure = fixture.work().join(".cache").join("toolchain-bin");
    fs::create_dir_all(&exposure).expect("make the exposure directory");
    fs::write(exposure.join("fd"), "").expect("write a link stand-in");
    let output = fixture.run(&["bin-paths"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let document: serde_json::Value = serde_json::from_str(&text(&output.stdout)).expect("JSON on standard output");
    assert_eq!(document["exposureDir"], ".cache/toolchain-bin");
    assert_eq!(document["bins"]["fd"], exposure.join("fd").to_str().expect("utf-8"));
    assert!(document["bins"].get("bin").is_none(), "a binary without a link is not listed");
    assert_eq!(document["pathDirs"][0], exposure.to_str().expect("utf-8"));
}

#[test]
fn bin_paths_needs_no_lock() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.work().join("toolchain.lock.json")).expect("remove the lock");
    let output = fixture.run(&["bin-paths"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
}

#[test]
fn help_lists_the_toolchain_verbs() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--help"]);
    let help = text(&output.stdout);
    for verb in ["lock", "update", "sync", "status", "verify-archive", "bin-paths", "prune"] {
        assert!(help.contains(verb), "{verb} missing from:\n{help}");
    }
}

#[test]
fn sync_and_status_refuse_when_the_plan_and_lock_disagree_and_name_the_remedy() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let edited = PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]");
    fs::write(fixture.work().join("toolchain.plan.jsonc"), edited).expect("edit the plan");
    for verb in ["sync", "status"] {
        let output = fixture.run(&[verb]);
        assert_eq!(output.status.code(), Some(1), "{verb}: {}", text(&output.stderr));
        assert!(text(&output.stderr).contains("mpm-shadow lock"), "{verb}: {}", text(&output.stderr));
        assert!(text(&output.stdout).is_empty(), "{verb} printed nothing before refusing: {}", text(&output.stdout));
    }
    assert!(!fixture.directory.path().join("home").join(".local").exists(), "nothing was installed under the home");
}

#[test]
fn update_refuses_a_hand_edited_lock_before_any_network_use() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let path = fixture.work().join("toolchain.lock.json");
    let text_before = fs::read_to_string(&path).expect("read");
    fs::write(&path, text_before.replace("\"version\": \"10.5.0\"", "\"version\": \"1.0.0\"")).expect("hand edit");
    let output = fixture.run(&["update", "--check"]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("edited by hand"), "{}", text(&output.stderr));
}

#[test]
fn verify_archive_accepts_the_recorded_hash_and_refuses_anything_else() {
    let fixture = Fixture::new();
    fixture.sign_lock(&fixture.work());
    let download = fixture.directory.path().join("bin.tar.gz");
    fs::write(&download, "not the recorded bytes").expect("write the download");
    let refused = fixture.run(&["verify-archive", "bin", download.to_str().expect("utf-8"), "--platform", "linux-x64"]);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("SHA-256 mismatch"), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791"), "{}", text(&refused.stderr));
    let unknown = fixture.run(&["verify-archive", "bin", download.to_str().expect("utf-8"), "--platform", "plan9-mips"]);
    assert_eq!(unknown.status.code(), Some(2));
}
