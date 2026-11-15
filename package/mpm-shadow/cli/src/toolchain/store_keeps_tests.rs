//! What:
//! Unit tests of the keeps every removal asks before it takes a version away:
//! a version a lock names,
//! a version a running process of this account uses,
//! a lock that cannot be read,
//! and a version nothing keeps.
//! Why:
//! Both install-time removals and `prune --toolchain` decide by this rule,
//! so an answer of "nothing keeps it" that came too easily would remove a version a service runs,
//! and an answer of "kept" that came too easily would let the store grow without bound.
//! The fixtures install a tool no lock of the sample plan names and no process on this machine runs,
//! so an answer is about the fixture and never about this machine's own processes.

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Child` and `Command` from `std::process`,
/// which start the stand-in program a holder test needs.
use std::process::{Child, Command};

/// Imports `Rig` from `super::fakes`.
use super::fakes::Rig;
/// Imports the rule under test.
use super::store_keeps::{InstallKeeps, tool_directory_name};

/// The id of the tool the fixtures install.
pub(super) const FIXTURE_ID: &str = "store-keeps-fixture";

/// The store directory of that tool,
/// which is what a lock row for it becomes.
const FIXTURE_DIRECTORY: &str = "store-keeps-fixture";

/// The program the fixtures install.
const FIXTURE_BIN: &str = "store-keeps-fixture";

/// The tag prefix that makes the fixture a release archive of this repository.
pub(super) const FIXTURE_PREFIX: &str = "monochromatic-store-keeps-fixture-v";

/// The lock text that names the fixture at `version`,
/// in the shape another worktree's lock has.
pub(super) fn lock_naming(version: &str) -> String {
    return format!(
        "{{\"tools\":{{\"{FIXTURE_ID}\":{{\"request\":\"github:Aquaticats/Monochromatic@latest\",\"version\":\"{version}\",\"route\":\"archive\",\"bins\":[\"{FIXTURE_BIN}\"],\"options\":{{\"tagPrefix\":\"{FIXTURE_PREFIX}\"}}}}}}}}"
    );
}

/// What:
/// A program of this test's own,
/// started from a path under a store,
/// so the process table holds that store path.
/// Why:
/// The rule under test reads the real process table,
/// and a version a service runs is the case it exists for;
/// a guard stops the program however the test ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StandIn {
///   static startedFrom(program: string): StandIn;
///   pid(): number;
///   stop(): void;
/// }
/// ```
pub(super) struct StandIn {
    /// The started program.
    child: Child,
}

/// The methods of `StandIn`.
impl StandIn {
    /// What:
    /// Copies the system's `sleep` to `program` and starts it there long enough for any test.
    /// Why:
    /// A store path in the process table has to be a program this account really runs,
    /// and `sleep` is a program that holds its file and does nothing.
    pub(super) fn started_from(program: &Path) -> StandIn {
        if let Some(parent) = program.parent() {
            fs::create_dir_all(parent).expect("make the program's directory");
        }
        fs::copy("/bin/sleep", program).expect("copy a program into the store");
        // What: the program is started by the spelling its links resolved to.
        // Why: `/proc/<pid>/exe` reports the resolved path, and a fixture started by another spelling would test nothing.
        let physical: PathBuf = fs::canonicalize(program).expect("the copy resolves");
        let child: Child = Command::new(&physical).arg("300").spawn().expect("start the stand-in");
        return StandIn { child };
    }

    /// The process id of the program.
    pub(super) fn pid(&self) -> u32 {
        return self.child.id();
    }

    /// Stops the program and waits for it,
    /// so nothing of this test outlives it.
    pub(super) fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Stops the program when the guard drops,
/// for a test that ends early.
impl Drop for StandIn {
    /// Kills and reaps the program.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The version directory of the fixture at `version` under the rig's repository root.
pub(super) fn version_directory(rig: &Rig, version: &str) -> PathBuf {
    return rig.scope.toolchain_root.join(FIXTURE_DIRECTORY).join(version);
}

/// Makes git list both trees,
/// so the other tree's lock is one an install can see.
pub(super) fn list_both_trees(rig: &Rig, other: &Path) {
    let text: String = format!("worktree {}\nHEAD 0000\n\nworktree {}\nHEAD 0000\n\n", rig.scope.root.display(), other.display());
    rig.runner.answer("git -C", &text, 0);
}

/// A second worktree of the rig's home whose lock names the fixture at `version`.
pub(super) fn second_tree_naming(rig: &Rig, version: &str) -> PathBuf {
    let other = super::published_archive_tests::second_tree(rig);
    fs::write(&other.lock_file, lock_naming(version)).expect("write the other tree's lock");
    list_both_trees(rig, &other.root);
    return other.lock_file;
}

//region what keeps a version

#[test]
fn a_version_another_worktree_lock_names_is_kept_and_the_reason_names_that_lock() {
    if crate::toolchain::fixtures::machine_hides_processes() {
        return;
    }
    let rig = Rig::new();
    fs::create_dir_all(version_directory(&rig, "0.2.0")).expect("make the version directory");
    // Control: with this tree's lock alone the version is nothing to anyone, so the test can fail.
    let alone: InstallKeeps = InstallKeeps::of(&rig.context());
    assert_eq!(alone.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &version_directory(&rig, "0.2.0")), None, "this tree's lock pins no 0.2.0");

    let other_lock: PathBuf = second_tree_naming(&rig, "0.2.0");
    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    let reason: String = keeps
        .keep_reason(FIXTURE_DIRECTORY, "0.2.0", &version_directory(&rig, "0.2.0"))
        .expect("the other worktree's lock names it");
    assert!(reason.contains("the lock") && reason.contains("names it"), "{reason}");
    assert!(reason.contains(&other_lock.display().to_string()), "the line names the lock file: {reason}");
    assert_eq!(keeps.keep_reason(FIXTURE_DIRECTORY, "0.2.1", &version_directory(&rig, "0.2.1")), None, "a version no lock names is kept by nothing");
}

#[test]
fn a_version_no_lock_names_and_no_process_runs_is_kept_by_nothing() {
    if crate::toolchain::fixtures::machine_hides_processes() {
        return;
    }
    let rig = Rig::new();
    fs::create_dir_all(version_directory(&rig, "0.2.0")).expect("make the version directory");
    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    assert_eq!(keeps.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &version_directory(&rig, "0.2.0")), None);
    // Control: the same rule keeps this very directory once another tree's lock names it, so the answer above is the
    // store's state and not a rule that keeps nothing.
    second_tree_naming(&rig, "0.2.0");
    let after: InstallKeeps = InstallKeeps::of(&rig.context());
    assert!(after.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &version_directory(&rig, "0.2.0")).is_some());
}

#[test]
fn a_version_a_process_of_this_account_runs_is_kept_and_the_reason_names_the_process() {
    let rig = Rig::new();
    let directory: PathBuf = version_directory(&rig, "0.2.0");
    let program: PathBuf = directory.join("release-folder").join(FIXTURE_BIN);
    fs::create_dir_all(&directory).expect("make the version directory");
    // What: the store is there and nothing of this test runs it yet.
    // Why: the same path then reports a holder only because a program was started from it, which is what proves the
    // scan reached this process rather than answering for another reason.
    // A machine with a process the kernel will not describe keeps every version already, so the strict "nothing
    // keeps it" is only asked where the scan described every process.
    let before: InstallKeeps = InstallKeeps::of(&rig.context());
    let scan = super::store_path_holders::holders_of(
        std::slice::from_ref(&rig.scope.toolchain_root),
        std::time::Instant::now() + std::time::Duration::from_secs(2),
    );
    if scan.opaque.is_empty() {
        assert_eq!(before.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &directory), None, "nothing runs it yet");
    }

    let stand_in: StandIn = StandIn::started_from(&program);
    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    let reason: String = keeps.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &directory).expect("a process of this account runs it");
    assert!(reason.contains("it is in use"), "{reason}");
    assert!(reason.contains("runs"), "{reason}");
    assert!(reason.contains(&format!("pid {}", stand_in.pid())), "the line names the process: {reason}");
    stand_in.stop();
}

#[test]
fn a_lock_that_cannot_be_read_keeps_every_version() {
    let rig = Rig::new();
    fs::create_dir_all(version_directory(&rig, "0.2.0")).expect("make the version directory");
    fs::create_dir_all(version_directory(&rig, "0.2.1")).expect("make the second version directory");
    let other = super::published_archive_tests::second_tree(&rig);
    // A directory where the lock file belongs: git lists it, and reading it as a lock fails.
    fs::create_dir_all(&other.lock_file).expect("make a directory in the lock's place");
    list_both_trees(&rig, &other.root);

    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    for version in ["0.2.0", "0.2.1"] {
        let reason: String = keeps
            .keep_reason(FIXTURE_DIRECTORY, version, &version_directory(&rig, version))
            .expect("a lock that cannot be read might name any version");
        assert!(reason.contains("could not be read"), "{reason}");
        assert!(reason.contains(&other.lock_file.display().to_string()), "the line names the file: {reason}");
    }
}

#[test]
fn a_route_that_keeps_no_version_directory_has_no_directory_name() {
    let rig = Rig::new();
    assert_eq!(tool_directory_name(&rig.tool("cargo:fd-find")), Some("cargo/fd-find".to_string()));
    assert_eq!(tool_directory_name(&rig.tool("node")), Some("node".to_string()));
    assert_eq!(tool_directory_name(&rig.tool("npm:wrangler")), None, "pnpm keeps its payload in its own home");
    assert_eq!(tool_directory_name(&rig.tool("rust")), None, "rustup keeps its payload in its own home");
}

//endregion

//region what a scan that could not finish keeps

/// What:
/// A process scan that ran out of its time keeps the version:
/// "the scan did not finish" is carried to the removal as its own answer,
/// never as "nothing runs it".
/// Why:
/// The review of this change ran an install with `MPM_SHADOW_STORE_WORK_LIMIT=0` while an older version of the tool was
/// running,
/// and the install deleted that running version:
/// `holders_of` answered `incomplete`,
/// and the answer was dropped on the way to the decision.
/// The module's own rule says a process table that cannot be read keeps every version.
#[test]
fn a_process_scan_that_could_not_finish_keeps_the_version() {
    let mut rig = Rig::new();
    // The process scan's deadline is spent before its first look at the table.
    rig.env = rig.env.with(crate::toolchain::store_trim::WORK_LIMIT_ENV, "0");
    let directory: std::path::PathBuf = rig.scope.toolchain_root.join("node").join("26.11.0");

    let keeps = super::store_keeps::InstallKeeps::of(&rig.context());
    let kept: Option<String> = keeps.keep_reason("node", "26.11.0", &directory);

    assert!(kept.is_some(), "a scan that could not tell what is in use was read as nothing in use");
    let line: String = kept.expect("the reason");
    assert!(line.contains("could not be read"), "the line says which answer was missing: {line}");
    assert!(keeps.in_use_reason(&directory).is_some(), "the newest-only rule asks this alone too");
}

//endregion

/// What:
/// A version a process of this account holds is kept even when the kernel will not describe that process,
/// and the line says an unspecified process may run it.
/// Why:
/// The review of this change ran a same-user process that opened a file under a version, moved into the version and
/// made itself undumpable: the scan could read neither its programs nor its folders, so no held path named the
/// version, and the newest-only removals would have deleted a version in active use.
#[test]
fn a_version_a_process_hides_its_use_of_is_kept() {
    let rig = Rig::new();
    let directory: PathBuf = version_directory(&rig, "0.2.0");
    let program: PathBuf = directory.join("release-folder").join(FIXTURE_BIN);
    fs::create_dir_all(directory.join("release-folder")).expect("make the version directory");
    fs::write(&program, b"#!/bin/sh\n").expect("write a file of the version");
    let _hiding: HiddenUse = HiddenUse::started_in(&directory, &program);

    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    let kept: Option<String> = keeps.keep_reason(FIXTURE_DIRECTORY, "0.2.0", &directory);

    let line: String = kept.expect("an opaque process holding this version keeps it");
    assert!(line.contains("it is in use"), "the line says the version is in use: {line}");
    assert!(line.contains("runs"), "the line says what the process may run: {line}");
    assert!(line.contains("may"), "the line says the scan cannot tell which version: {line}");
}

/// What:
/// A process of this account that the kernel will not describe:
/// it works inside `directory`,
/// holds `program` open,
/// and has made itself undumpable,
/// so the scan can read neither.
/// Why:
/// This is the case the rule exists for and the one no ordinary program produces:
/// it is the only way a version in use leaves no held path in the process table.
struct HiddenUse {
    /// The started process.
    child: Child,
}

/// The methods of `HiddenUse`.
impl HiddenUse {
    /// Starts the hiding process in `directory`, holding `program` open, and waits until it is hidden.
    fn started_in(directory: &Path, program: &Path) -> HiddenUse {
        let ready: PathBuf = directory.join(".hidden-use-ready");
        let child: Child = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "import ctypes,os,time; os.chdir({:?}); f=open({:?},'rb'); ctypes.CDLL(None).prctl(4,0,0,0,0); open({:?},'w').close(); time.sleep(300)",
                directory,
                program,
                ready,
            ))
            .spawn()
            .expect("start the hidden stand-in");
        let give_up: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !ready.exists() {
            assert!(std::time::Instant::now() < give_up, "the stand-in never became opaque");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        return HiddenUse { child };
    }
}

/// Stops the hidden process and waits for it, so nothing of this test outlives it.
impl Drop for HiddenUse {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
