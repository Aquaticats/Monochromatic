//! End-to-end tests of the launcher a tree's exposed link goes through for a repository-published tool,
//! and of the older-lock warning on the toolchain verbs,
//! run against the built `mpm-shadow` binary in disposable directories.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpm_shadow::constants::NOT_FOUND_EXIT_CODE;
use mpm_shadow::toolchain::launcher::{LaunchRecord, record_path, render_record};
use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::ScopeKind;

/// The tool the fixtures install.
const TOOL_ID: &str = "cargo:monochromatic-demo-tool";

/// A shell program standing in for the real binary:
/// prints its arguments and one variable on standard output,
/// a line on standard error,
/// and exits with `TOOL_EXIT` or dies of SIGTERM when `TOOL_KILL` is set.
const REAL_PROGRAM: &str = "#!/bin/sh\nprintf 'out:'\nfor argument in \"$@\"; do printf '[%s]' \"$argument\"; done\nprintf '\\n'\nprintf 'env:%s\\n' \"$TOOL_ENV\"\necho 'err-from-the-tool' >&2\nif [ -n \"$TOOL_KILL\" ]; then kill -TERM $$; fi\nexit ${TOOL_EXIT:-0}\n";

/// The warning text the launcher prints for the given versions,
/// the lock file it read and the program it starts.
fn expected_warning(locked: &str, used: &str, lock_file: &Path, started: &Path) -> String {
    return format!(
        "mpm-shadow: warning: {TOOL_ID} is locked at {locked} in this tree's lock, {}, but {used} runs, because older repository tools are \
         never kept once a newer one is installed; upgrade this tree's lock with `mpm update {TOOL_ID}` or merge main; starting {}\n",
        lock_file.display(),
        started.display()
    );
}

/// A disposable tree:
/// a toolchain root holding the demo tool at `used`,
/// a lock pinning `locked`,
/// and the launcher for the tool.
struct Tree {
    directory: tempfile::TempDir,
}

impl Tree {
    fn new(locked: &str, used: &str) -> Tree {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let tree = Tree { directory };
        let crate_dir = tree.crate_dir();
        let bin = crate_dir.join(used).join("bin");
        fs::create_dir_all(&bin).expect("make the bin directory");
        fs::write(bin.join("tool"), REAL_PROGRAM).expect("write the real program");
        fs::set_permissions(bin.join("tool"), fs::Permissions::from_mode(0o755)).expect("chmod");
        symlink(used, crate_dir.join("current")).expect("link current");
        fs::write(
            tree.lock_file(),
            format!("{{\"schemaVersion\":1,\"tools\":{{\"{TOOL_ID}\":{{\"version\":\"{locked}\",\"route\":\"cargo-binstall\"}}}}}}"),
        )
        .expect("write the lock");
        fs::create_dir_all(tree.launcher().parent().expect("a parent")).expect("make the launcher directory");
        fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), tree.launcher()).expect("copy the binary");
        let record = LaunchRecord {
            tool_id: TOOL_ID.to_string(),
            lock_file: tree.lock_file(),
            real: crate_dir.join("current").join("bin").join("tool"),
            scope: ScopeKind::Repo,
        };
        fs::write(record_path(&tree.launcher()), render_record(&record)).expect("write the record");
        return tree;
    }

    fn crate_dir(&self) -> PathBuf {
        return self.directory.path().join("root").join("cargo").join("monochromatic-demo-tool");
    }

    fn lock_file(&self) -> PathBuf {
        return self.directory.path().join("toolchain.lock.json");
    }

    fn launcher(&self) -> PathBuf {
        return self.directory.path().join("launchers").join("tool");
    }

    fn real(&self) -> PathBuf {
        return self.crate_dir().join("current").join("bin").join("tool");
    }
}

/// Runs `program` with the arguments and a marker variable.
fn run(program: &Path, arguments: &[&str], extra: &[(&str, &str)]) -> Output {
    let mut command = Command::new(program);
    command.args(arguments).env_clear().env("PATH", "/usr/bin:/bin").env("TOOL_ENV", "passed");
    for (name, value) in extra {
        command.env(name, value);
    }
    return command.output().expect("run");
}

#[test]
fn an_older_lock_prints_the_warning_on_standard_error_and_leaves_standard_output_byte_identical() {
    let tree = Tree::new("0.4.0", "0.4.4");
    let arguments = ["a b", "--flag=x", "", "$HOME", "tab\there"];
    let direct = run(&tree.real(), &arguments, &[]);
    let launched = run(&tree.launcher(), &arguments, &[]);
    assert_eq!(launched.stdout, direct.stdout, "standard output is the real program's, byte for byte");
    assert!(!launched.stdout.is_empty());
    let expected_stderr =
        format!("{}{}", expected_warning("0.4.0", "0.4.4", &tree.lock_file(), &tree.real()), String::from_utf8_lossy(&direct.stderr));
    assert_eq!(String::from_utf8_lossy(&launched.stderr), expected_stderr);
    assert_eq!(launched.status.code(), Some(0));
}

#[test]
fn a_matching_lock_prints_no_warning_and_passes_everything_through() {
    let tree = Tree::new("0.4.4", "0.4.4");
    let direct = run(&tree.real(), &["x"], &[]);
    let launched = run(&tree.launcher(), &["x"], &[]);
    assert_eq!(launched.stdout, direct.stdout);
    assert_eq!(launched.stderr, direct.stderr, "no warning when the lock and the install match");
}

#[test]
fn the_exit_code_and_the_environment_pass_through() {
    let tree = Tree::new("0.4.0", "0.4.4");
    let launched = run(&tree.launcher(), &[], &[("TOOL_EXIT", "7"), ("TOOL_ENV", "custom value")]);
    assert_eq!(launched.status.code(), Some(7));
    assert!(String::from_utf8_lossy(&launched.stdout).contains("env:custom value"), "{}", String::from_utf8_lossy(&launched.stdout));
}

#[test]
fn a_signal_that_kills_the_real_program_is_the_launchers_own_status() {
    let tree = Tree::new("0.4.4", "0.4.4");
    let launched = run(&tree.launcher(), &[], &[("TOOL_KILL", "1")]);
    assert_eq!(launched.status.signal(), Some(15), "the launcher replaced itself, so the signal is the process's own");
}

#[test]
fn a_missing_real_program_exits_127_naming_it_and_never_writes_standard_output() {
    let tree = Tree::new("0.4.0", "0.4.4");
    fs::remove_file(tree.real()).expect("remove the program");
    let launched = run(&tree.launcher(), &[], &[]);
    assert_eq!(launched.status.code(), Some(NOT_FOUND_EXIT_CODE));
    assert!(launched.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&launched.stderr);
    assert!(stderr.contains(&format!("not starting {}: no file exists there", tree.real().display())), "{stderr}");
    assert!(!stderr.contains("os error"), "the launcher checked and did not try to start it: {stderr}");
}

#[test]
fn without_a_record_the_same_binary_is_the_ordinary_command_line_tool() {
    let tree = Tree::new("0.4.0", "0.4.4");
    fs::remove_file(record_path(&tree.launcher())).expect("remove the record");
    let output = run(&tree.launcher(), &["--version"], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("mpm-shadow"), "{}", String::from_utf8_lossy(&output.stdout));
}

/// A home whose identity file names the tree as the repository,
/// with a check program that says the drive is not mounted;
/// returns the home.
fn home_where_the_repository_is_gone(tree: &Tree) -> PathBuf {
    let home = tree.directory.path().join("home-with-identity");
    let identity = home.join(".config/monochromatic-dev/repository-link.json");
    fs::create_dir_all(identity.parent().expect("a parent")).expect("make the config folder");
    fs::write(&identity, format!("{{\"linkPath\":\"{}\",\"canonicalRoot\":\"/nowhere\",\"phase\":\"after\"}}", tree.directory.path().display()))
        .expect("write the identity");
    let check = home.join(".local/share/monochromatic-dev/repository-link/repository-link-check");
    fs::create_dir_all(check.parent().expect("a parent")).expect("make the install folder");
    fs::write(&check, "#!/bin/sh\necho 'the 860 drive is not mounted yet' >&2\nexit 75\n").expect("write the check");
    fs::set_permissions(&check, fs::Permissions::from_mode(0o755)).expect("chmod");
    return home;
}

#[test]
fn an_unreadable_lock_under_a_repository_that_is_not_there_names_the_cause_and_still_runs_the_tool() {
    let tree = Tree::new("0.4.0", "0.4.4");
    fs::remove_file(tree.lock_file()).expect("remove the lock");
    let home = home_where_the_repository_is_gone(&tree);
    let direct = run(&tree.real(), &["x"], &[]);
    let launched = run(&tree.launcher(), &["x"], &[("HOME", home.to_str().expect("a text path"))]);
    assert_eq!(launched.stdout, direct.stdout, "the tool ran and its output is its own");
    assert_eq!(launched.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&launched.stderr);
    let first = stderr.lines().next().unwrap_or_default();
    assert!(first.starts_with("mpm-shadow: warning: "), "{stderr}");
    assert!(first.contains("the 860 drive is not mounted yet"), "{stderr}");
    assert!(first.contains("toolchain.lock.json"), "the line names the unreadable file: {stderr}");
}

#[test]
fn an_unreadable_lock_with_no_identity_file_stays_silent_as_before() {
    let tree = Tree::new("0.4.0", "0.4.4");
    fs::remove_file(tree.lock_file()).expect("remove the lock");
    let direct = run(&tree.real(), &["x"], &[]);
    let launched = run(&tree.launcher(), &["x"], &[("HOME", tree.directory.path().to_str().expect("a text path"))]);
    assert_eq!(launched.stderr, direct.stderr, "no line without an identity file");
    assert_eq!(launched.stdout, direct.stdout);
}

/// A tree for the toolchain verbs:
/// a plan and a signed lock pinning the demo tool at `locked`,
/// and a home whose toolchain root holds the tool at `installed`.
struct VerbTree {
    directory: tempfile::TempDir,
}

impl VerbTree {
    fn new(locked: &str, installed: &str) -> VerbTree {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let work = directory.path().join("work");
        fs::create_dir_all(&work).expect("make the worktree");
        fs::write(
            work.join("toolchain.plan.jsonc"),
            format!(
                "{{ \"schemaVersion\": 1, \"tools\": {{ \"{TOOL_ID}\": {{ \"request\": \"cargo:monochromatic-demo-tool@{locked}\", \"route\": \"cargo-binstall\", \"bins\": [\"tool\"] }} }} }}"
            ),
        )
        .expect("write the plan");
        let unsigned = format!(
            "{{\"schemaVersion\":1,\"tools\":{{\"{TOOL_ID}\":{{\"request\":\"cargo:monochromatic-demo-tool@{locked}\",\"version\":\"{locked}\",\"route\":\"cargo-binstall\",\"bins\":[\"tool\"],\"env\":{{}},\"options\":{{}},\"artifacts\":{{}}}}}}}}"
        );
        let parsed = parse_lock(&unsigned, "fixture").expect("the fixture lock parses");
        fs::write(work.join("toolchain.lock.json"), render_lock(&parsed.lock)).expect("write the lock");
        let root = directory
            .path()
            .join("home")
            .join(".local")
            .join("share")
            .join("monochromatic-dev")
            .join("toolchain")
            .join("cargo")
            .join("monochromatic-demo-tool");
        let version_dir = root.join(installed);
        fs::create_dir_all(version_dir.join("bin")).expect("make the install");
        fs::write(
            version_dir.join(".crates.toml"),
            format!("[v1]\n\"monochromatic-demo-tool {installed} (registry+https://example.invalid/index)\" = [\"tool\"]\n"),
        )
        .expect("write cargo's record");
        symlink(installed, root.join("current")).expect("link current");
        return VerbTree { directory };
    }

    fn lock_file(&self) -> PathBuf {
        return self.directory.path().join("work").join("toolchain.lock.json");
    }

    fn run(&self, arguments: &[&str]) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(arguments)
            .current_dir(self.directory.path().join("work"))
            .env_clear()
            .env("HOME", self.directory.path().join("home"))
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run mpm-shadow");
    }
}

/// The warning the verbs print,
/// with the verb's own prefix.
fn verb_warning(verb: &str, locked: &str, used: &str, lock_file: &Path) -> String {
    return format!(
        "toolchain {verb}: warning: {TOOL_ID} is locked at {locked} in this tree's lock, {}, but {used} runs, because older repository tools \
         are never kept once a newer one is installed; upgrade this tree's lock with `mpm update {TOOL_ID}` or merge main\n",
        lock_file.display()
    );
}

#[test]
fn lock_check_and_bin_warn_on_standard_error_and_status_in_its_report() {
    let tree = VerbTree::new("0.0.1", "0.0.2");
    let lock = tree.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(lock.status.code(), Some(0), "{}", String::from_utf8_lossy(&lock.stderr));
    assert!(String::from_utf8_lossy(&lock.stdout).contains("digest verified"));
    assert_eq!(String::from_utf8_lossy(&lock.stderr), verb_warning("install", "0.0.1", "0.0.2", &tree.lock_file()));

    let paths = tree.run(&["bin", "--json"]);
    assert_eq!(paths.status.code(), Some(0));
    serde_json::from_slice::<serde_json::Value>(&paths.stdout).expect("standard output stays plain JSON");
    assert_eq!(String::from_utf8_lossy(&paths.stderr), verb_warning("bin", "0.0.1", "0.0.2", &tree.lock_file()));

    let status = tree.run(&["list"]);
    assert_eq!(status.status.code(), Some(0), "an installed-newer repository tool is not drift");
    assert!(String::from_utf8_lossy(&status.stdout).contains(&verb_warning("list", "0.0.1", "0.0.2", &tree.lock_file())), "{}", String::from_utf8_lossy(&status.stdout));
}

/// A tree whose tool is a release archive of this repository:
/// the real program sits in a folder below `current`,
/// and the lock pins the tool by its plain id.
fn archive_tree(locked: &str, used: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
    let tool_dir = directory.path().join("root").join("demo-release");
    let folder = tool_dir.join(used).join("release-folder");
    fs::create_dir_all(&folder).expect("make the folder");
    fs::write(folder.join("tool"), REAL_PROGRAM).expect("write the real program");
    fs::set_permissions(folder.join("tool"), fs::Permissions::from_mode(0o755)).expect("chmod");
    symlink(used, tool_dir.join("current")).expect("link current");
    let lock_file = directory.path().join("toolchain.lock.json");
    fs::write(&lock_file, format!("{{\"schemaVersion\":1,\"tools\":{{\"demo-release\":{{\"version\":\"{locked}\",\"route\":\"archive\"}}}}}}"))
        .expect("write the lock");
    let launcher = directory.path().join("launchers").join("tool");
    fs::create_dir_all(launcher.parent().expect("a parent")).expect("make the launcher directory");
    fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), &launcher).expect("copy the binary");
    let record = LaunchRecord {
        tool_id: "demo-release".to_string(),
        lock_file,
        real: tool_dir.join("current").join("release-folder").join("tool"),
        scope: ScopeKind::Repo,
    };
    fs::write(record_path(&launcher), render_record(&record)).expect("write the record");
    return (directory, launcher, record.real);
}

#[test]
fn a_release_archive_tool_warns_on_every_run_when_the_lock_pins_an_older_version() {
    let (_directory, launcher, real) = archive_tree("0.2.0", "0.2.1");
    let direct = run(&real, &["x"], &[]);
    let launched = run(&launcher, &["x"], &[]);
    assert_eq!(launched.stdout, direct.stdout, "standard output is the real program's, byte for byte");
    let warning = format!(
        "mpm-shadow: warning: demo-release is locked at 0.2.0 in this tree's lock, {}, but 0.2.1 runs, because older repository tools are \
         never kept once a newer one is installed; upgrade this tree's lock with `mpm update demo-release` or merge main; starting ",
        _directory.path().join("toolchain.lock.json").display()
    );
    assert_eq!(
        String::from_utf8_lossy(&launched.stderr),
        format!("{warning}{}\n{}", real.display(), String::from_utf8_lossy(&direct.stderr))
    );

    let (_matching_directory, matching_launcher, matching_real) = archive_tree("0.2.1", "0.2.1");
    let quiet = run(&matching_launcher, &["x"], &[]);
    assert_eq!(quiet.stderr, run(&matching_real, &["x"], &[]).stderr, "no warning when the lock and the install match");
}

#[test]
fn the_verbs_stay_silent_when_the_lock_and_the_install_match() {
    let tree = VerbTree::new("0.0.2", "0.0.2");
    for arguments in [&["install", "--lockfile-only", "--frozen-lockfile"][..], &["bin", "--json"][..], &["list"][..]] {
        let output = tree.run(arguments);
        assert_eq!(output.status.code(), Some(0), "{arguments:?}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("warning"), "{arguments:?}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("warning"), "{arguments:?}");
    }
}

/// A tree whose release archive folder carries the version in its name,
/// as the archives of this repository's tools do (`monochromatic-cargo-shadow-0.1.4-x86_64-unknown-linux-gnu`):
/// the launcher's record was written while `recorded` was the version behind `current`,
/// then `current` moved to `now` and the folder of `recorded` was deleted,
/// which is what installing a newer release in any tree does.
struct MovedArchive {
    directory: tempfile::TempDir,
    tool_dir: PathBuf,
    launcher: PathBuf,
    lock_file: PathBuf,
}

impl MovedArchive {
    fn new(locked: &str, recorded: &str, now: &str) -> MovedArchive {
        return MovedArchive::in_scope(ScopeKind::Repo, locked, recorded, now);
    }

    fn in_scope(scope: ScopeKind, locked: &str, recorded: &str, now: &str) -> MovedArchive {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let tool_dir = directory.path().join("root").join("demo-release");
        let lock_file = directory.path().join("toolchain.lock.json");
        let launcher = directory.path().join("launchers").join("tool");
        let moved = MovedArchive { directory, tool_dir, launcher, lock_file };
        moved.install(recorded);
        fs::write(
            &moved.lock_file,
            format!("{{\"schemaVersion\":1,\"tools\":{{\"demo-release\":{{\"version\":\"{locked}\",\"route\":\"archive\"}}}}}}"),
        )
        .expect("write the lock");
        fs::create_dir_all(moved.launcher.parent().expect("a parent")).expect("make the launcher directory");
        fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), &moved.launcher).expect("copy the binary");
        let record = LaunchRecord {
            tool_id: "demo-release".to_string(),
            lock_file: moved.lock_file.clone(),
            real: moved.program_of(recorded),
            scope,
        };
        fs::write(record_path(&moved.launcher), render_record(&record)).expect("write the record");
        moved.install(now);
        fs::remove_dir_all(moved.tool_dir.join(recorded)).expect("remove the recorded version");
        return moved;
    }

    /// The program of `version` through the `current` link,
    /// the way a record names it.
    fn program_of(&self, version: &str) -> PathBuf {
        return self.tool_dir.join("current").join(format!("demo-release-{version}-linux")).join("tool");
    }

    /// Installs `version` and points `current` at it.
    fn install(&self, version: &str) {
        let folder = self.tool_dir.join(version).join(format!("demo-release-{version}-linux"));
        fs::create_dir_all(&folder).expect("make the folder");
        fs::write(folder.join("tool"), REAL_PROGRAM).expect("write the real program");
        fs::set_permissions(folder.join("tool"), fs::Permissions::from_mode(0o755)).expect("chmod");
        let link = self.tool_dir.join("current");
        let _ = fs::remove_file(&link);
        symlink(version, &link).expect("link current");
    }
}

#[test]
fn a_launcher_whose_recorded_version_was_deleted_runs_the_version_current_names() {
    let moved = MovedArchive::new("0.1.4", "0.1.4", "0.1.5");
    let direct = run(&moved.program_of("0.1.5"), &["a b", "--flag"], &[]);
    let launched = run(&moved.launcher, &["a b", "--flag"], &[]);
    assert_eq!(launched.status.code(), Some(0), "{}", String::from_utf8_lossy(&launched.stderr));
    assert_eq!(launched.stdout, direct.stdout, "standard output is the new version's program, byte for byte");
    let warning = format!(
        "mpm-shadow: warning: demo-release is locked at 0.1.4 in this tree's lock, {}, but 0.1.5 runs, because older repository tools are \
         never kept once a newer one is installed; upgrade this tree's lock with `mpm update demo-release` or merge main; starting ",
        moved.lock_file.display()
    );
    let started = moved.program_of("0.1.5");
    assert_eq!(
        String::from_utf8_lossy(&launched.stderr),
        format!("{warning}{}\n{}", started.display(), String::from_utf8_lossy(&direct.stderr)),
        "the warning names the program under current, not the deleted folder the record named"
    );
    assert!(!String::from_utf8_lossy(&launched.stderr).contains("0.1.4-linux"), "the deleted folder is never named as the one started");
}

#[test]
fn a_launcher_with_a_stale_record_and_a_matching_lock_prints_nothing_extra() {
    let moved = MovedArchive::new("0.1.5", "0.1.4", "0.1.5");
    let direct = run(&moved.program_of("0.1.5"), &["x"], &[]);
    let launched = run(&moved.launcher, &["x"], &[]);
    assert_eq!(launched.status.code(), Some(0));
    assert_eq!(launched.stdout, direct.stdout);
    assert_eq!(launched.stderr, direct.stderr, "following current is not a warning when the lock names the version that runs");
}

#[test]
fn a_launcher_that_finds_no_program_under_current_names_both_versions_and_the_one_command() {
    let moved = MovedArchive::new("0.1.4", "0.1.4", "0.1.5");
    fs::remove_file(moved.program_of("0.1.5")).expect("remove the program");
    let launched = run(&moved.launcher, &[], &[]);
    assert_eq!(launched.status.code(), Some(NOT_FOUND_EXIT_CODE));
    assert!(launched.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&launched.stderr);
    let tree = moved.lock_file.parent().expect("the tree").display().to_string();
    assert!(stderr.contains("not starting"), "{stderr}");
    assert!(!stderr.contains("os error"), "the launcher checked and did not try to start it: {stderr}");
    assert!(stderr.contains("demo-release"), "the tool is named: {stderr}");
    assert!(stderr.contains("locked at 0.1.4"), "the locked version is named: {stderr}");
    assert!(stderr.contains("0.1.5"), "the version current names is named: {stderr}");
    assert!(stderr.contains(&format!("`mpm install` in {tree}")), "the one command and where to run it: {stderr}");
    let _ = &moved.directory;
}

#[test]
fn a_launcher_rewrites_a_stale_record_to_the_program_it_started_so_a_shadow_behind_it_can_see_where_it_leads() {
    let moved = MovedArchive::new("0.1.5", "0.1.4", "0.1.5");
    let launched = run(&moved.launcher, &["x"], &[]);
    assert_eq!(launched.status.code(), Some(0), "{}", String::from_utf8_lossy(&launched.stderr));
    let text = fs::read_to_string(record_path(&moved.launcher)).expect("the record");
    let record = mpm_shadow::toolchain::launcher::parse_record(&text).expect("a record");
    assert_eq!(record.real, moved.program_of("0.1.5"), "the record names the program that ran");
    assert!(record.real.exists());
    assert_eq!(record.lock_file, moved.lock_file, "only the program changes");
    assert_eq!(record.tool_id, "demo-release");
    let again = run(&moved.launcher, &["x"], &[]);
    assert_eq!(again.stdout, launched.stdout);
    assert_eq!(again.stderr, launched.stderr, "a repeat run changes nothing");
}

#[test]
fn a_record_that_cannot_be_rewritten_is_a_warning_and_the_program_still_runs() {
    let moved = MovedArchive::new("0.1.5", "0.1.4", "0.1.5");
    let directory = moved.launcher.parent().expect("a parent").to_path_buf();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).expect("make the launcher directory read-only");
    let direct = run(&moved.program_of("0.1.5"), &["x"], &[]);
    let launched = run(&moved.launcher, &["x"], &[]);
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).expect("restore");
    assert_eq!(launched.status.code(), Some(0));
    assert_eq!(launched.stdout, direct.stdout);
    let stderr = String::from_utf8_lossy(&launched.stderr);
    assert!(stderr.contains("could not update the launcher record"), "{stderr}");
    assert!(stderr.ends_with(&*String::from_utf8_lossy(&direct.stderr)), "{stderr}");
}

/// A guard for the machine-global launcher (`~/.local/bin/git` on the owner's machine):
/// a global record whose version folder was deleted runs the program under `current`,
/// names that path and the global upgrade command,
/// and keeps the global scope when it rewrites the record.
/// A launcher binary older than the follow-current rule started the deleted folder and failed;
/// only the named path was red before this change.
#[test]
fn a_global_launcher_whose_recorded_version_was_deleted_names_the_path_it_starts_and_the_global_command() {
    let moved = MovedArchive::in_scope(ScopeKind::Global, "0.1.4", "0.1.4", "0.1.5");
    let direct = run(&moved.program_of("0.1.5"), &["x"], &[]);
    let launched = run(&moved.launcher, &["x"], &[]);
    assert_eq!(launched.status.code(), Some(0), "{}", String::from_utf8_lossy(&launched.stderr));
    assert_eq!(launched.stdout, direct.stdout);
    let warning = format!(
        "mpm-shadow: warning: demo-release is locked at 0.1.4 in this machine's global lock, {}, but 0.1.5 runs, because older repository \
         tools are never kept once a newer one is installed; upgrade the global lock with `mpm update -g demo-release`; starting ",
        moved.lock_file.display()
    );
    let started = moved.program_of("0.1.5");
    assert_eq!(String::from_utf8_lossy(&launched.stderr), format!("{warning}{}\n{}", started.display(), String::from_utf8_lossy(&direct.stderr)));
    let record = mpm_shadow::toolchain::launcher::parse_record(&fs::read_to_string(record_path(&moved.launcher)).expect("the record")).expect("a record");
    assert_eq!(record.real, started, "the record names the program that ran");
    assert_eq!(record.scope, ScopeKind::Global, "rewriting the program keeps the scope");
}

#[test]
fn a_launcher_never_prints_the_older_lock_warning_for_a_program_it_does_not_start() {
    let moved = MovedArchive::new("0.1.4", "0.1.4", "0.1.5");
    fs::remove_file(moved.program_of("0.1.5")).expect("remove the program");
    let launched = run(&moved.launcher, &[], &[]);
    let stderr = String::from_utf8_lossy(&launched.stderr);
    assert!(!stderr.contains("; starting"), "a warning that says starting is printed only when a program starts: {stderr}");
    assert!(!stderr.contains("warning:"), "{stderr}");
}
