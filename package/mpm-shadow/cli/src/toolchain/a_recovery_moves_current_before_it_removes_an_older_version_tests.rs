//! What:
//! The order a recovery's newest-only rule keeps:
//! the tool's `current` link is moved to the newest finished version before any version directory older than it is
//! removed,
//! which is the order an install keeps,
//! and the sentences that removal leaves in the report.
//! Why:
//! A start made between the two steps of the wrong order finds `current` naming a version this run has just removed,
//! and a run killed there leaves every start failing until the next `install`;
//! the two controls below stand in that instant and end a run there,
//! and each fails when the two steps are swapped back.
//! The sentences are read from a run here because none named them before,
//! so an edit to either wording kept every test green.
//! Everything runs against the fake runner and the fake network inside disposable homes,
//! and a program these tests start is a copy of this test binary started as a child,
//! so nothing of the test binary's own process is replaced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-recovery-moves-current-before-it-removes-an-older-version.tests.ts
//! test('a start between the two steps runs the recovered version', () => { ... });
//! test('a run killed between the two steps leaves a link that resolves', () => { ... });
//! test('the sentence names the removed version for what it was', () => { ... });
//! test('the sentence names the recovered version when that version is the older one', () => { ... });
//! test('a run that publishes after a failed recovery says what it then did', () => { ... });
//! test('nothing is removed when the link cannot be moved', () => { ... });
//! test('a link move that fails once ends one and keeps the older version', () => { ... });
//! test('a put back also points current at the newest finished version', () => { ... });
//! test('a failed listing says no copy was read when the install put the version there', () => { ... });
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports the trait that changes a directory's mode for the failed-listing control.
use std::os::unix::fs::PermissionsExt;
/// Imports the trait that reads the signal a killed process ended with.
use std::os::unix::process::ExitStatusExt;
/// Imports what these tests use from `std::path`.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the types that start the child processes these controls run.
use std::process::{
    Command,
    ExitStatus,
    Output,
};

/// Imports the crate that answers an operating system error's own number,
/// which a planted refusal is given as one.
use rustix::io::Errno;

/// Imports the hidden name a killed install leaves its finished copy under,
/// the owner pair such a name carries,
/// the read of a process's start time that pair is built from,
/// and the marker an install writes last.
use super::archive_staging::{
    Owner,
    Plant,
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    RenameRefused,
    STAGING_PURPOSE,
    clear_the_plant,
    hidden_sibling,
    plant_these_steps,
    start_time_of_a_process,
    write_complete_marker,
};
/// Imports the link name a repository-published tool's version is chosen through.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports the bundle of seams an installer receives,
/// and the machine a test pretends to be.
use super::context::{
    Context,
    Host,
};
/// Imports the two fakes and the rig a test builds them into.
use super::fakes::{
    FakeHttp,
    FakeRunner,
    Rig,
};
/// Imports the writer of a program's bytes,
/// which hands them to a child so no other test's `fork` inherits the write.
use super::fixture_programs::write_program;
/// Imports the digest of an archive a control serves.
use super::hash::sha256_hex;
/// Imports the step that gives a release archive's programs their version-independent names.
use super::install_archive::link_stable_programs;
/// Imports the one link write the install and these fixtures share,
/// the refusal a control plants at its rename,
/// and how many moves that refusal answers.
use super::install_support::{
    HowOften,
    atomic_symlink,
    refusing_the_next_link_move,
};
/// Imports the launcher a tree runs a repository-published tool through,
/// the record it reads,
/// and the read of the record beside this process's own file.
use super::launcher::{
    ensure_launcher,
    launch_record_of_current_exe,
    run_launcher,
};
/// Imports the directory of one tool below the toolchain root.
use super::layout::tool_directory;
/// Imports the lock,
/// its text,
/// and the read of its text.
use super::lock::{
    Lock,
    parse_lock,
    render_lock,
};
/// Imports the operating-system family an install is judged against.
use super::model::OsFamily;
/// Imports the locked tool a failed-listing control rewrites.
use super::model::{
    Artifact,
    LockedTool,
};
/// Imports the link `current`,
/// the version it names,
/// and the question that tells a finished install from an unfinished one.
use super::published_archive::{
    current_link_of,
    is_finished,
    running_version_archive,
};
/// Imports what these tests share with the tests of the newest-only layout:
/// the tool id and its program,
/// the folder a release archive unpacks into,
/// the lock that pins one version,
/// the finished copy and the release a fake network serves,
/// a digest no lock has to vouch for,
/// and the context built over a rig.
use super::published_archive_tests::{
    A_TEST_SHA256,
    BIN,
    ID,
    context_of,
    finish_archive,
    lock_at,
    release_digest,
    release_folder,
    serve_releases,
};
/// Imports the request a fake `tar` answers.
use super::runner::RunRequest;
/// Imports the scope of a tree,
/// its environment,
/// and the scope one tree's root gives.
use super::scope::{
    EnvMap,
    Scope,
    repo_scope,
};
/// Imports the hold a control plants at the newest-only rule's one instant,
/// and the install itself.
use super::sync::{
    Hold,
    SyncReport,
    holding_the_next_instant,
    sync_tools,
};

/// The version these tests install before the killed upgrade,
/// which the recovery then removes as the older one.
const THE_VERSION_INSTALLED_BEFORE: &str = "1.0.0";

/// The version the killed install left a finished copy of,
/// which the tree's lock pins.
const THE_VERSION_A_KILL_LEFT_FINISHED: &str = "2.0.0";

/// The name the version's program prints,
/// which is how a start says which version ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatTheProgramPrints(version: string): string { return `${ID} ${version} (fixture)`; }
/// ```
fn what_the_program_prints(version: &str) -> String {
    return format!("{ID} {version} (fixture)");
}

/// The number `signal(7)` gives `SIGKILL`,
/// which no handler catches and after which no cleanup runs.
const SIGKILL_NUMBER: i32 = 9;

/// The answer a plant gives a rename or an exchange,
/// the operating system's number for an input/output error,
/// so what it shows is a failure outside the "cannot exchange" list.
const AN_INPUT_OUTPUT_ERROR: i32 = Errno::IO.raw_os_error();

/// What:
/// The test this file's control starts as the tree's launcher.
/// Why:
/// The child has to be a process of its own:
/// a launcher replaces itself with the tool's program,
/// and that inside the test binary would end every test in this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_LAUNCHER_A_CONTROL_STARTS = 'toolchain::a_recovery_moves_current_before_it_removes_an_older_version_tests::the_launcher_a_control_starts';
/// ```
const THE_LAUNCHER_A_CONTROL_STARTS: &str = "toolchain::a_recovery_moves_current_before_it_removes_an_older_version_tests::the_launcher_a_control_starts";

/// What:
/// The test this file's control starts as the run it ends at the instant between the two steps.
/// Why:
/// The kill must end a whole run and not this process,
/// and this test binary is the only program that carries the fake runner and the fake network a run needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_RUN_A_CONTROL_KILLS = 'toolchain::a_recovery_moves_current_before_it_removes_an_older_version_tests::the_run_a_control_kills_between_the_two_steps';
/// ```
const THE_RUN_A_CONTROL_KILLS: &str = "toolchain::a_recovery_moves_current_before_it_removes_an_older_version_tests::the_run_a_control_kills_between_the_two_steps";

/// The variable the run a control kills reads the fixture's home directory from.
const HOME_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_HOME";

/// The variable that same run reads the fixture's tree from.
const TREE_VARIABLE: &str = "MPM_SHADOW_TEST_FIXTURE_TREE";

/// What:
/// The tool directory below the toolchain root of `root`,
/// which every version and every hidden name of the tool sits in.
/// Why:
/// The tests build and inspect the tool's directory on disk,
/// and the layout is the one answer for where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theToolDirectory(root: string): string { return toolDirectory(root, ID)!; }
/// ```
fn the_tool_directory(root: &Path) -> PathBuf {
    return tool_directory(root, ID).expect("a plain directory name");
}

/// What:
/// An installer's pair for a process the system no longer runs:
/// the number of the first process with a start time one tick after its own,
/// which no process ever carried.
/// Why:
/// It is a stand-in for the pair a real kill leaves,
/// and a later install may remove a hidden name only when its installer provably ended,
/// so the name has to be one the system answers `gone` for.
/// This is the shape `the_killed_installs_finished_copy` writes,
/// and it stands for a real kill;
/// a real kill of a real install is the report's evidence,
/// not this file's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anInstallerThatNoLongerRuns(): Owner { return { ...Owner.here(), pid: 1, started: startTimeOf(1)! + 1 }; }
/// ```
fn an_installer_that_no_longer_runs() -> Owner {
    let here: Owner = Owner::here();
    let started: u64 =
        start_time_of_a_process(1).expect("the system reports the first process's start");
    return Owner {
        pid: 1,
        started: started + 1,
        boot: here.boot,
        namespace: here.namespace,
    };
}

/// What:
/// Writes the finished copy of `version` into `into`,
/// the way an install leaves one:
/// the program inside the folder the release archive unpacks into,
/// the program's own name inside the copy,
/// and the completion marker recording the lock's row last.
/// Why:
/// A recovery publishes a copy only when its marker records the lock's tool,
/// version and digest,
/// so a copy built any other way would be refused and the fixture would show nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeAFinishedCopy(rig: Rig, version: string, into: string): void {
///   const tool = lockAt(rig, version).tool(ID)!;
///   writeProgram(join(into, releaseFolder(version), BIN), `#!/bin/sh\necho ${whatTheProgramPrints(version)}\n`);
///   linkStablePrograms(tool, into);
///   writeCompleteMarker(into, ID, version, releaseDigest(version));
/// }
/// ```
fn write_a_finished_copy(
    rig: &Rig,
    version: &str,
    into: &Path,
) {
    let lock: Lock = lock_at(rig, version);
    let tool = lock.tool(ID).expect("the lock names the tool");
    fs::create_dir_all(into.join(release_folder(version)))
        .expect("make the folder a release archive unpacks into");
    write_program(
        &into.join(release_folder(version)).join(BIN),
        format!("#!/bin/sh\necho '{}'\n", what_the_program_prints(version)).as_bytes(),
    );
    link_stable_programs(tool, into).expect("the program's own name inside the copy");
    write_complete_marker(into, ID, version, &release_digest(version))
        .expect("write the completion marker last");
}

/// What:
/// The finished copy of `version` a killed install leaves under a hidden name beside the version directory,
/// and answers the tool directory it sits in.
/// Why:
/// This is the state a recovery starts from when nothing is published yet,
/// and every control here needs it built the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theKilledInstallsFinishedCopy(rig: Rig, version: string): string {
///   const toolDir = theToolDirectory(rig.scope.toolchainRoot);
///   const hidden = hiddenSibling(join(toolDir, version), STAGING_PURPOSE, anInstallerThatNoLongerRuns());
///   writeAFinishedCopy(rig, version, hidden);
///   return toolDir;
/// }
/// ```
fn the_killed_installs_finished_copy(
    rig: &Rig,
    version: &str,
) -> PathBuf {
    let tool_dir: PathBuf = the_tool_directory(&rig.scope.toolchain_root);
    // What: `hidden_sibling` builds the name a kill leaves a finished copy under, out of the version and the owner pair.
    // Why: the recovery finds a copy by that name and by nothing else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hidden = hiddenSibling(join(toolDir, version), STAGING_PURPOSE, anInstallerThatNoLongerRuns());
    // ```
    let hidden: PathBuf = hidden_sibling(
        &tool_dir.join(version),
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_a_finished_copy(rig, version, &hidden);
    return tool_dir;
}

/// What:
/// The claim a failed recovery left in the tool's directory,
/// found by the version and the purpose its name carries.
/// Why:
/// The name also carries the pair of the process that made it,
/// which this file cannot know,
/// so the claim is found rather than built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theClaimTheFailedRecoveryLeft(rig: Rig, version: string): string {
///   return readdirSync(theToolDirectory(rig.scope.toolchainRoot)).find((name) => name.startsWith(`.${version}.recovered.`));
/// }
/// ```
fn the_claim_the_failed_recovery_left(
    rig: &Rig,
    version: &str,
) -> PathBuf {
    let tool_dir: PathBuf = the_tool_directory(&rig.scope.toolchain_root);
    let prefix: String = format!(".{version}.recovered.");
    for entry in fs::read_dir(&tool_dir)
        .expect("the tool's directory")
        .flatten()
    {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) {
            return entry.path();
        }
    }
    panic!(
        "the failed recovery left no claim in {}",
        tool_dir.display()
    );
}

/// What:
/// Gives that claim the pair of an installer that has ended,
/// which is what a later run reads once the run that claimed it is gone.
/// Why:
/// Two runs of one test are one process,
/// so the claim a real later run finds dead still names this process;
/// this stand-in is the only part of the fixture that is not a real state,
/// and the report's transcripts show the same removal from two real runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asAfterTheClaimingRunEnded(rig: Rig, version: string): string {
///   const claim = theClaimTheFailedRecoveryLeft(rig, version);
///   const ended = hiddenSibling(join(theToolDirectory(rig.scope.toolchainRoot), version), RECOVERED_PURPOSE, anInstallerThatNoLongerRuns());
///   renameSync(claim, ended);
///   return ended;
/// }
/// ```
fn as_after_the_claiming_run_ended(
    rig: &Rig,
    version: &str,
) -> PathBuf {
    let claim: PathBuf = the_claim_the_failed_recovery_left(rig, version);
    let tool_dir: PathBuf = the_tool_directory(&rig.scope.toolchain_root);
    let ended: PathBuf = hidden_sibling(
        &tool_dir.join(version),
        RECOVERED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::rename(&claim, &ended).expect("give the claim the pair of an installer that ended");
    return ended;
}

/// What:
/// This tree's launcher for the tool and the exposed name that points at it,
/// which is what a person starts.
/// Why:
/// A launcher replaces itself with the program its record names,
/// or with the program under `current` when that file is gone,
/// so starting it is how a test asks whether `current` resolves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theTreesLauncher(rig: Rig): string {
///   const real = join(theToolDirectory(rig.scope.toolchainRoot), CURRENT_LINK_NAME, BIN);
///   const launcher = ensureLauncher(rig.scope, ID, BIN, real);
///   const exposed = join(rig.scope.exposureDir, BIN);
///   atomicSymlink(exposed, launcher);
///   return exposed;
/// }
/// ```
fn the_trees_launcher(rig: &Rig) -> PathBuf {
    let real: PathBuf = the_tool_directory(&rig.scope.toolchain_root)
        .join(CURRENT_LINK_NAME)
        .join(BIN);
    let launcher: PathBuf = ensure_launcher(&rig.scope, ID, BIN, &real)
        .expect("this tree's launcher for the tool is made");
    let exposed: PathBuf = rig.scope.exposure_dir.join(BIN);
    atomic_symlink(&exposed, &launcher).expect("the exposed name points at the launcher");
    return exposed;
}

/// What:
/// The state a killed upgrade leaves,
/// and the lock the tree installs from:
/// the older version installed and finished with `current` naming it,
/// the killed install's finished copy of the newer version under a hidden name,
/// this tree's launcher,
/// and the lock pinning the newer version.
/// Why:
/// The run this file judges recovers the newer version and then settles the newest-only rule,
/// which is the pair of steps whose order is the finding;
/// the older version has to be installed for the rule to have anything to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aKilledUpgradeOf(rig: Rig, older: string, newer: string): [string, Lock] {
///   finishArchive(rig.scope.toolchainRoot, older);
///   const lock = lockAt(rig, newer);
///   atomicSymlink(currentLinkOf(rig.scope.toolchainRoot, ID), older);
///   const toolDir = theKilledInstallsFinishedCopy(rig, newer);
///   writeFileSync(rig.scope.lockFile, renderLock(lock));
///   theTreesLauncher(rig);
///   return [toolDir, lock];
/// }
/// ```
fn a_killed_upgrade_of(
    rig: &Rig,
    older: &str,
    newer: &str,
) -> (PathBuf, Lock) {
    let root: PathBuf = rig.scope.toolchain_root.clone();
    finish_archive(&root, older);
    let current: PathBuf = current_link_of(&root, ID).expect("a plain directory name");
    // What: `atomic_symlink` writes the `current` link through a temporary name, so no reader sees it half written.
    // Why: an install writes the link the same way, so this fixture's link is the shape a run meets.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // atomicSymlink(current, older);
    // ```
    atomic_symlink(&current, Path::new(older)).expect("`current` names the older version");
    let tool_dir: PathBuf = the_killed_installs_finished_copy(rig, newer);
    let lock: Lock = lock_at(rig, newer);
    fs::write(&rig.scope.lock_file, render_lock(&lock)).expect("write the lock this tree installs");
    the_trees_launcher(rig);
    return (tool_dir, lock);
}

/// What:
/// Starts the tool through the exposed name that points at this tree's launcher,
/// in a child process,
/// and answers what that child wrote and how it ended.
/// Why:
/// The launcher replaces itself with the tool's program,
/// which in this process would end every test in this file,
/// and a child's exit status is what a person's shell sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTheTool(exposed: string): { status: ExitStatus; stdout: string; stderr: string } {
///   return spawnSync(exposed, ['--exact', THE_LAUNCHER_A_CONTROL_STARTS, '--ignored', '--test-threads=1']);
/// }
/// ```
fn start_the_tool(exposed: &Path) -> Output {
    return Command::new(exposed)
        .arg("--exact")
        .arg(THE_LAUNCHER_A_CONTROL_STARTS)
        .arg("--ignored")
        .arg("--test-threads=1")
        // What: `--nocapture` keeps the test harness from taking the lines the launcher writes for its own report.
        // Why: a captured line is printed only when the child's own test fails,
        //  and this child reports the refused start by exiting with its status.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // childArgs.push('--nocapture');
        // ```
        .arg("--nocapture")
        .output()
        .expect("start the tool through this tree's launcher");
}

/// What:
/// Starts the run this file's killer ends at the instant between the two steps,
/// and answers how that run ended.
/// Why:
/// A kill inside this process would end the whole test binary,
/// so the run is a process of its own over the same fixture directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheRunAControlKills(rig: Rig): ExitStatus {
///   return spawnSync(process.execPath, ['--exact', THE_RUN_A_CONTROL_KILLS, '--ignored', '--test-threads=1'], {
///     env: { ...process.env, MPM_SHADOW_TEST_FIXTURE_HOME: rig.scope.home, MPM_SHADOW_TEST_FIXTURE_TREE: rig.scope.root },
///   }).status;
/// }
/// ```
fn run_the_run_a_control_kills(rig: &Rig) -> ExitStatus {
    return Command::new(std::env::current_exe().expect("the test binary's own path"))
        .arg("--exact")
        .arg(THE_RUN_A_CONTROL_KILLS)
        .arg("--ignored")
        .arg("--test-threads=1")
        .env(HOME_VARIABLE, &rig.scope.home)
        .env(TREE_VARIABLE, &rig.scope.root)
        .output()
        .expect("start the run this control kills")
        .status;
}

/// What:
/// A start of the tool made while a run stands between the recovery's two steps,
/// which the first control plants and the run itself makes.
/// Why:
/// The assertion inside the run is the control:
/// a start there reads `current`,
/// so it runs the newest finished version when the link was moved first and finds no program when it was not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aStartWhereTheRunStands(scope: Scope): void {
///   const started = startTheTool(join(scope.exposureDir, BIN));
///   assert(started.status.success(), ...);
///   assert(started.stdout.includes(whatTheProgramPrints(THE_VERSION_A_KILL_LEFT_FINISHED)));
/// }
/// ```
fn a_start_where_the_run_stands(scope: &Scope) {
    let started: Output = start_the_tool(&scope.exposure_dir.join(BIN));
    assert!(
        started.status.success(),
        "a start between the two steps runs the newest finished version: {started:?}"
    );
    assert!(
        String::from_utf8_lossy(&started.stdout)
            .contains(&what_the_program_prints(THE_VERSION_A_KILL_LEFT_FINISHED)),
        "the start ran {} and not the version the run removed: {}",
        THE_VERSION_A_KILL_LEFT_FINISHED,
        String::from_utf8_lossy(&started.stdout)
    );
}

/// What:
/// A newer version of the tool finished in the tool directory of `final_dir`,
/// which stands for the install that lands between this run's check and its publish.
/// Why:
/// The rule against keeping an older version is decided before the publish,
/// so only an install made during the publish can put a newer version there while the older locked one is published;
/// the plant makes that instant exact instead of timed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anotherInstallPutsTheNewerVersionInPlace(staging: string, finalDir: string): void {
///   writeTheNewerVersionByHand(dirname(finalDir), THE_VERSION_A_KILL_LEFT_FINISHED);
/// }
/// ```
fn another_install_puts_the_newer_version_in_place(
    _staging: &Path,
    final_dir: &Path,
) {
    let tool_dir: &Path = final_dir.parent().expect("the tool's directory");
    let newer: PathBuf = tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED);
    fs::create_dir_all(newer.join(release_folder(THE_VERSION_A_KILL_LEFT_FINISHED)))
        .expect("make the folder a release archive unpacks into");
    write_program(
        &newer
            .join(release_folder(THE_VERSION_A_KILL_LEFT_FINISHED))
            .join(BIN),
        format!(
            "#!/bin/sh\necho '{}'\n",
            what_the_program_prints(THE_VERSION_A_KILL_LEFT_FINISHED)
        )
        .as_bytes(),
    );
    write_complete_marker(
        &newer,
        ID,
        THE_VERSION_A_KILL_LEFT_FINISHED,
        &release_digest(THE_VERSION_A_KILL_LEFT_FINISHED),
    )
    .expect("write the completion marker last");
}

/// What:
/// The launcher a control starts:
/// it reads the record beside its own file and starts the tool through it.
/// Why:
/// It is the whole of what a person's shell runs,
/// so the start it makes is the one the finding is about,
/// and it is a fixture a control starts rather than a test of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the launcher a control starts', () => {
///   const record = launchRecordOfCurrentExe()!;
///   process.exit(runLauncher(record, []));
/// });
/// ```
#[test]
#[ignore = "a fixture a control starts: this copy is the tree's launcher and starts the tool through `current`"]
fn the_launcher_a_control_starts() {
    let record = launch_record_of_current_exe().expect("this launcher has its record beside it");
    let status: i32 = run_launcher(&record, &[]);
    std::process::exit(status);
}

/// What:
/// The run a control kills:
/// it installs from the fixture's own lock,
/// which recovers the killed install's finished copy and then settles the newest-only rule,
/// and it ends this process at the instant between that rule's two steps.
/// Why:
/// The state a kill in that instant leaves is the second thing the order decides,
/// and only a process that really ends shows it;
/// this test refuses rather than passes when the run returned instead of being killed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the run a control kills between the two steps', () => {
///   const scope = repoScope(process.env.MPM_SHADOW_TEST_FIXTURE_TREE!, homeOnly(process.env));
///   const held = holdingTheNextInstant({ killAtTheInstant: true });
///   const report = syncTools(contextOverTheFixturesLock(scope));
///   fail(`the run returned (${report}), so no kill between the two steps happened`);
/// });
/// ```
#[test]
#[ignore = "a fixture a control starts and kills: it ends this process at the instant between the two steps"]
fn the_run_a_control_kills_between_the_two_steps() {
    let home: PathBuf = std::env::var_os(HOME_VARIABLE)
        .map(PathBuf::from)
        .expect("a control names the fixture's home directory");
    let tree: PathBuf = std::env::var_os(TREE_VARIABLE)
        .map(PathBuf::from)
        .expect("a control names the fixture's tree");
    let env: EnvMap = EnvMap::from_pairs(&[("HOME", home.to_str().expect("a utf-8 path"))]);
    let scope: Scope = repo_scope(&tree, &env).expect("the fixture's tree is a scope");
    let text: String = fs::read_to_string(&scope.lock_file).expect("the fixture's lock");
    let label: String = scope.lock_file.to_string_lossy().to_string();
    let lock: Lock = parse_lock(&text, &label)
        .expect("the fixture's lock is a lock")
        .lock;
    let runner: FakeRunner = FakeRunner::new();
    let http: FakeHttp = FakeHttp::new();
    let host: Host = Host {
        platform: "linux-x64".to_string(),
        family: Some(OsFamily::Linux),
    };
    let context: Context = Context {
        scope: &scope,
        env: &env,
        runner: &runner,
        http: &http,
        host: &host,
        tools: &lock.tools,
    };
    let _hold = holding_the_next_instant(Hold {
        acts: None,
        kill_at_the_instant: true,
    });
    let report: SyncReport = sync_tools(&context, None);
    panic!(
        "the run returned ({report:?}), so it was not killed at the instant between the two steps"
    );
}

#[test]
fn a_start_between_the_two_steps_runs_the_version_the_recovery_published() {
    let rig = Rig::new();
    let (tool_dir, lock) = a_killed_upgrade_of(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        THE_VERSION_A_KILL_LEFT_FINISHED,
    );
    let _hold = holding_the_next_instant(Hold {
        acts: Some(a_start_where_the_run_stands),
        kill_at_the_instant: false,
    });
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        !tool_dir.join(THE_VERSION_INSTALLED_BEFORE).exists(),
        "the rule removed the older version once the link was moved: {report:?}"
    );
    assert_eq!(
        running_version_archive(&rig.scope.toolchain_root, ID).as_deref(),
        Some(THE_VERSION_A_KILL_LEFT_FINISHED),
        "the run ends with the link naming the version the lock pins"
    );
    assert!(
        is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "the recovered version is finished: {report:?}"
    );
}

#[test]
fn a_run_killed_between_the_two_steps_leaves_a_link_that_resolves() {
    let rig = Rig::new();
    a_killed_upgrade_of(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        THE_VERSION_A_KILL_LEFT_FINISHED,
    );
    let ended: ExitStatus = run_the_run_a_control_kills(&rig);
    assert_eq!(
        ended.signal(),
        Some(SIGKILL_NUMBER),
        "the run has to end at the instant between the two steps, not after it: {ended:?}"
    );
    // What the kill left: the link names the version the killed run published, and that version is there.
    assert_eq!(
        running_version_archive(&rig.scope.toolchain_root, ID).as_deref(),
        Some(THE_VERSION_A_KILL_LEFT_FINISHED),
        "a run killed between the two steps leaves `current` naming a version that is there"
    );
    let started: Output = start_the_tool(&rig.scope.exposure_dir.join(BIN));
    assert!(
        started.status.success(),
        "the tool starts after the run was killed: {started:?}"
    );
    assert!(
        String::from_utf8_lossy(&started.stdout)
            .contains(&what_the_program_prints(THE_VERSION_A_KILL_LEFT_FINISHED)),
        "the start after the kill runs the recovered version: {}",
        String::from_utf8_lossy(&started.stdout)
    );
}

#[test]
fn the_sentence_after_a_recovery_names_the_removed_version_for_what_it_was() {
    let rig = Rig::new();
    let (tool_dir, lock) = a_killed_upgrade_of(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        THE_VERSION_A_KILL_LEFT_FINISHED,
    );
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(
        report.recovered.iter().any(|sentence| {
            return sentence.starts_with(&format!(
                "finished the publish of {ID} {THE_VERSION_A_KILL_LEFT_FINISHED} from the finished copy an earlier install left at "
            ));
        }),
        "the report says which publish this run finished: {report:?}"
    );
    let removed: &String = report
        .recovered
        .iter()
        .find(|sentence| return sentence.starts_with("removed the older"))
        .unwrap_or_else(|| panic!("the report says nothing was removed: {report:?}"));
    assert_eq!(
        removed,
        &format!(
            "removed the older {ID} {THE_VERSION_INSTALLED_BEFORE} that was installed before this run, because a newer one is installed and an older repository tool is never kept"
        ),
        "the sentence names the older version for what it was, not for what this run recovered"
    );
    assert!(
        !removed.contains("this run recovered"),
        "this run recovered {THE_VERSION_A_KILL_LEFT_FINISHED} and removed {THE_VERSION_INSTALLED_BEFORE}: {removed}"
    );
    assert!(!tool_dir.join(THE_VERSION_INSTALLED_BEFORE).exists());
}

#[test]
fn the_sentence_names_the_recovered_version_when_that_version_is_the_older_one() {
    let rig = Rig::new();
    // The lock pins the older version, and a finished copy of it is what the killed install left.
    let lock: Lock = lock_at(&rig, THE_VERSION_INSTALLED_BEFORE);
    let tool_dir: PathBuf = the_killed_installs_finished_copy(&rig, THE_VERSION_INSTALLED_BEFORE);
    // What: the plant runs only in a publish, and this publish is the recovery's own.
    // Why: a newer version can land between the rule's check and this publish, and no fixture can time that.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plantTheseSteps({ anotherInstallActs: anotherInstallPutsTheNewerVersionInPlace });
    // ```
    plant_these_steps(Plant {
        another_install_acts: Some(another_install_puts_the_newer_version_in_place),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    clear_the_plant();
    let removed: &String = report
        .recovered
        .iter()
        .find(|sentence| return sentence.starts_with("removed the older"))
        .unwrap_or_else(|| panic!("the report says nothing was removed: {report:?}"));
    assert_eq!(
        removed,
        &format!(
            "removed the older {ID} {THE_VERSION_INSTALLED_BEFORE} that this run recovered, because a newer one is installed and an older repository tool is never kept"
        ),
        "the version this run published is the one the rule removed, and the sentence says so"
    );
    assert!(
        !tool_dir.join(THE_VERSION_INSTALLED_BEFORE).exists(),
        "the rule removes the version this run published once a newer one is installed"
    );
    assert!(
        is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "the newer version the other install made is the one that stays: {report:?}"
    );
    // What: the run ends warning that its lock pins the version the rule just removed.
    // Why: a version newer than the locked one counts as satisfying it,
    //  so the run reports the newer one as what runs and warns about the pin rather than downloading anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(report.warnings.some((warning) => warning.includes('is locked at 1.0.0')));
    // ```
    assert!(
        report.warnings.iter().any(|warning| {
            return warning.contains(&format!("is locked at {THE_VERSION_INSTALLED_BEFORE}"))
                && warning.contains(&format!("but {THE_VERSION_A_KILL_LEFT_FINISHED} runs"));
        }),
        "the run ends warning that its lock pins the version the rule removed: {report:?}"
    );
}

#[test]
fn a_run_that_publishes_after_a_failed_recovery_says_what_it_then_did() {
    let rig = Rig::new();
    let lock: Lock = lock_at(&rig, THE_VERSION_A_KILL_LEFT_FINISHED);
    let tool_dir: PathBuf =
        the_killed_installs_finished_copy(&rig, THE_VERSION_A_KILL_LEFT_FINISHED);
    serve_releases(&rig, &[THE_VERSION_A_KILL_LEFT_FINISHED]);
    // What: the recovery's own publish is refused where it gives the version its name.
    // Why: the bug this test is about is a run that then downloads and publishes the version itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plantTheseSteps({ refuseTheRename: ['the-finished-copy-goes-in', AN_INPUT_OUTPUT_ERROR] });
    // ```
    plant_these_steps(Plant {
        refuse_the_rename: Some((RenameRefused::GivesTheVersionItsName, AN_INPUT_OUTPUT_ERROR)),
        ..Plant::default()
    });
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    clear_the_plant();
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(
        report.installed,
        vec![ID.to_string()],
        "the run downloaded and installed the version after the recovery failed: {report:?}"
    );
    let warning: &String = report
        .warnings
        .iter()
        .find(|warning| {
            return warning.contains("running the command again publishes it without downloading");
        })
        .unwrap_or_else(|| panic!("the recovery's own words are not reported: {report:?}"));
    assert!(
        warning.contains(&format!(
            "the install this run went on to make put {ID} {THE_VERSION_A_KILL_LEFT_FINISHED} at its name"
        )),
        "the warning says what the run then did: {warning}"
    );
    assert!(
        !warning.ends_with("and running the command again publishes it without downloading"),
        "no line says the next run publishes a copy this run already replaced: {warning}"
    );
    assert!(
        is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "{report:?}"
    );
    // What: the copy the failed recovery kept gets the pair of an installer that has ended, which is what a later run
    //  reads once this run's process is gone.
    // Why: both runs here are one process, so the claim carries this process's own pair and a real later run would leave
    //  it alone; this is a stand-in for the first run having ended, and the run after a killed install is the state the
    //  transcript in the report shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // asAfterTheClaimingRunEnded(rig, THE_VERSION_A_KILL_LEFT_FINISHED);
    // const later = syncTools(context_of(rig, scope, lock));
    // ```
    as_after_the_claiming_run_ended(&rig, THE_VERSION_A_KILL_LEFT_FINISHED);
    let later: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(later.failed.is_empty(), "{later:?}");
    assert_eq!(
        later.recovered,
        vec![format!(
            "removed 1 hidden directory left behind by an install of {ID} that has ended, \
             1 of which held a second finished copy of {THE_VERSION_A_KILL_LEFT_FINISHED}; \
             {THE_VERSION_A_KILL_LEFT_FINISHED} holds its completion marker and its program at its name, \
             so that copy was a second one"
        )],
        "a later run publishes nothing from that copy and removes it, and the version is still installed: {later:?}"
    );
    assert!(
        rig.http.requests().len() == 1,
        "only the first run downloaded, so the later one published nothing: {:?}",
        rig.http.requests()
    );
}

/// What:
/// Every link move of one run refused,
/// and the state the run then leaves.
/// Why:
/// A version removed under a link that still names it is the state the order of the two steps exists to prevent,
/// and no test made this move fail before,
/// so deleting the return after a failed move kept every test green;
/// this control fails when the removal runs after a failed move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('nothing is removed when the link cannot be moved', () => {
///   const [toolDir, lock] = aKilledUpgradeOf(rig, OLD, NEW);
///   using refusal = refusingTheNextLinkMove(AN_INPUT_OUTPUT_ERROR, 'every-time');
///   const report = syncTools(contextOf(rig, rig.scope, lock));
///   expect(report.failed.some(([id, message]) => id === ID && message.includes('cannot place the link'))).toBe(true);
///   expect(existsSync(join(toolDir, OLD))).toBe(true);
///   expect(runningVersionArchive(rig.scope.toolchainRoot, ID)).toBe(OLD);
/// });
/// ```
#[test]
fn nothing_is_removed_when_the_link_cannot_be_moved() {
    let rig = Rig::new();
    let (tool_dir, lock) = a_killed_upgrade_of(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        THE_VERSION_A_KILL_LEFT_FINISHED,
    );
    let _refusal = refusing_the_next_link_move(AN_INPUT_OUTPUT_ERROR, HowOften::EveryTime);
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(
        report.failed.iter().any(|(id, message)| {
            return id.as_str() == ID && message.contains("cannot place the link");
        }),
        "the run reports that the link could not be placed: {report:?}"
    );
    assert!(
        tool_dir.join(THE_VERSION_INSTALLED_BEFORE).exists(),
        "nothing is removed under a link that does not name the newest version yet: {report:?}"
    );
    assert_eq!(
        running_version_archive(&rig.scope.toolchain_root, ID).as_deref(),
        Some(THE_VERSION_INSTALLED_BEFORE),
        "the failed move left the link naming the older version, so nothing may have been removed under it"
    );
    assert!(
        is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "the recovery published the newer version before the move failed: {report:?}"
    );
}

/// What:
/// One link move of one run refused,
/// so the run's later step moves the link,
/// and the older version stays beside the newer one.
/// Why:
/// A transient failure of one rename costs a status 1 and leaves a retired version on disk,
/// because the later step moves the link but nothing removes the older version after it;
/// the README says what happens and this control holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a link move that fails once ends one and keeps the older version', () => {
///   using refusal = refusingTheNextLinkMove(AN_INPUT_OUTPUT_ERROR, 'once');
///   const report = syncTools(...);
///   expect(report.failed).not.toEqual([]);
///   expect(runningVersionArchive(root, ID)).toBe(NEW);
///   expect(existsSync(join(toolDir, OLD))).toBe(true);
///   expect(startTheTool(exposed).stdout).toContain(whatTheProgramPrints(NEW));
/// });
/// ```
#[test]
fn a_link_move_that_fails_once_ends_one_and_keeps_the_older_version() {
    let rig = Rig::new();
    let (tool_dir, lock) = a_killed_upgrade_of(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        THE_VERSION_A_KILL_LEFT_FINISHED,
    );
    let _refusal = refusing_the_next_link_move(AN_INPUT_OUTPUT_ERROR, HowOften::Once);
    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(
        !report.failed.is_empty(),
        "the run ends 1 although the version is at its name: {report:?}"
    );
    assert!(
        report.failed.iter().any(|(id, message)| {
            return id.as_str() == ID && message.contains("cannot place the link");
        }),
        "the failure names the link it could not place: {report:?}"
    );
    assert_eq!(
        running_version_archive(&rig.scope.toolchain_root, ID).as_deref(),
        Some(THE_VERSION_A_KILL_LEFT_FINISHED),
        "a later step of the same run moved the link: {report:?}"
    );
    assert!(
        is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "the version is at its name after the failed move: {report:?}"
    );
    assert!(
        tool_dir.join(THE_VERSION_INSTALLED_BEFORE).exists(),
        "nothing was removed under the failed move, so the older version stays: {report:?}"
    );
    let started: Output = start_the_tool(&rig.scope.exposure_dir.join(BIN));
    assert!(
        started.status.success(),
        "the tool starts after the failed move: {started:?}"
    );
    assert!(
        String::from_utf8_lossy(&started.stdout)
            .contains(&what_the_program_prints(THE_VERSION_A_KILL_LEFT_FINISHED)),
        "a start runs the version the link now names: {}",
        String::from_utf8_lossy(&started.stdout)
    );
}

/// What:
/// A recovery that puts back the unfinished copy a killed install had moved aside,
/// and the link it also moves.
/// Why:
/// A put-back gives the version's name a copy,
/// so the rule that points `current` at the newest finished version runs for it too,
/// and the newest finished version may be an older one than the lock names;
/// nothing said so before,
/// and whether it stays is the owner's to decide,
/// so this control holds what the code does today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a put back also points current at the newest finished version', () => {
///   writeAFinishedCopy(rig, OLD, join(toolDir, OLD));
///   const replaced = hiddenSibling(join(toolDir, NEW), REPLACED_PURPOSE, endedInstaller());
///   writeAnUnfinishedCopy(rig, NEW, replaced);
///   const staging = hiddenSibling(join(toolDir, NEW), STAGING_PURPOSE, endedInstaller());
///   writeAFinishedCopy(rig, NEW, staging);
///   writeCompleteMarker(staging, ID, NEW, A_TEST_SHA256);
///   const report = syncTools(contextOf(rig, rig.scope, lockAt(rig, NEW)));
///   expect(report.recovered).toContain('put back the copy ...');
///   expect(runningVersionArchive(root, ID)).toBe(OLD);
///   expect(report.failed.some(([, message]) => message.includes('download failed'))).toBe(true);
///   expect(startTheTool(exposed).stdout).toContain(whatTheProgramPrints(OLD));
/// });
/// ```
#[test]
fn a_put_back_also_points_current_at_the_newest_finished_version() {
    let rig = Rig::new();
    let root: PathBuf = rig.scope.toolchain_root.clone();
    let tool_dir: PathBuf = the_tool_directory(&root);
    // The older finished version, and no `current`: what a kill between the marker and the link leaves.
    write_a_finished_copy(
        &rig,
        THE_VERSION_INSTALLED_BEFORE,
        &tool_dir.join(THE_VERSION_INSTALLED_BEFORE),
    );
    // The version's name empty, the copy that was there at its aside name: a kill between the two renames.
    let replaced: PathBuf = hidden_sibling(
        &tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED),
        REPLACED_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    fs::create_dir_all(replaced.join(release_folder(THE_VERSION_A_KILL_LEFT_FINISHED)))
        .expect("make the folder the copy that was there unpacked into");
    write_program(
        &replaced
            .join(release_folder(THE_VERSION_A_KILL_LEFT_FINISHED))
            .join(BIN),
        format!(
            "#!/bin/sh\necho '{}'\n",
            what_the_program_prints(THE_VERSION_A_KILL_LEFT_FINISHED)
        )
        .as_bytes(),
    );
    // The finished copy whose marker records no digest this lock names, so the recovery cannot publish it.
    let staging: PathBuf = hidden_sibling(
        &tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED),
        STAGING_PURPOSE,
        an_installer_that_no_longer_runs(),
    );
    write_a_finished_copy(&rig, THE_VERSION_A_KILL_LEFT_FINISHED, &staging);
    write_complete_marker(
        &staging,
        ID,
        THE_VERSION_A_KILL_LEFT_FINISHED,
        A_TEST_SHA256,
    )
    .expect("write a marker the lock does not vouch for");
    the_trees_launcher(&rig);
    let lock: Lock = lock_at(&rig, THE_VERSION_A_KILL_LEFT_FINISHED);

    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert!(
        report.recovered.iter().any(|sentence| {
            return sentence.starts_with(&format!(
                "put back the copy of {ID} {THE_VERSION_A_KILL_LEFT_FINISHED} that an earlier install had moved aside"
            ));
        }),
        "the recovery puts back the copy the killed install had moved aside: {report:?}"
    );
    assert!(
        tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED).is_dir(),
        "the copy is back at the version's name: {report:?}"
    );
    assert!(
        !is_finished(&tool_dir.join(THE_VERSION_A_KILL_LEFT_FINISHED), BIN),
        "the copy put back is unfinished, which is why `current` moves to an older version"
    );
    assert_eq!(
        running_version_archive(&root, ID).as_deref(),
        Some(THE_VERSION_INSTALLED_BEFORE),
        "a put-back also points `current` at the newest finished version: {report:?}"
    );
    assert!(
        report.failed.iter().any(|(id, message)| {
            return id.as_str() == ID && message.contains("download failed");
        }),
        "the lock's version is not at its name and the install that follows fails: {report:?}"
    );
    let started: Output = start_the_tool(&rig.scope.exposure_dir.join(BIN));
    assert!(
        started.status.success(),
        "a start through the launcher runs the older version `current` names: {started:?}"
    );
    assert!(
        String::from_utf8_lossy(&started.stdout)
            .contains(&what_the_program_prints(THE_VERSION_INSTALLED_BEFORE)),
        "the start runs {} and not the version the lock names: {}",
        THE_VERSION_INSTALLED_BEFORE,
        String::from_utf8_lossy(&started.stdout)
    );
}

/// What:
/// The mode a control found on a directory,
/// put back when the control's body leaves,
/// however it leaves.
/// Why:
/// A directory a control makes unreadable must not keep the fixture from being removed after a failed assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheModePutBack { [Symbol.dispose]() { chmodSync(this.path, this.mode); } }
/// ```
struct TheModePutBack {
    /// The directory whose mode the control changed.
    path: PathBuf,
    /// The mode it had before the control changed it.
    mode: u32,
}

impl TheModePutBack {
    /// Makes `path` possible to write and enter but not to list,
    /// and answers the guard that restores its mode.
    fn making_unreadable(path: &Path) -> TheModePutBack {
        let mode: u32 = fs::metadata(path)
            .expect("the directory whose mode this control changes")
            .permissions()
            .mode();
        fs::set_permissions(path, fs::Permissions::from_mode(0o311))
            .expect("leave the directory writable and enterable but not listable");
        return TheModePutBack {
            path: path.to_path_buf(),
            mode,
        };
    }
}

impl Drop for TheModePutBack {
    /// Puts the mode back.
    fn drop(&mut self) {
        // Best effort: the fixture is removed afterwards either way, and the error below is what matters.
        let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(self.mode));
    }
}

/// What:
/// A run that cannot list the tool's directory,
/// whose install then puts the version at its name,
/// and the warning it prints for the recovery that failed.
/// Why:
/// The suffix used to say a later run publishes nothing from the copy and removes it,
/// and a failed listing read no copy at all;
/// the warning must claim nothing about a copy it did not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a failed listing says no copy was read when the install put the version there', () => {
///   const tool = rig.tool('bin');
///   serve(rig.http, tool.artifacts[0].url, body);
///   tool.artifacts = [{ platform: 'linux-x64', url, sha256: sha256Hex(body) }];
///   mkdirSync(toolDir);
///   using mode = TheModePutBack.makingUnreadable(toolDir);
///   const report = syncTools(contextOf(rig, rig.scope, { tools: [tool] }));
///   expect(report.installed).toEqual(['bin']);
///   expect(report.failed).toEqual([]);
///   const warning = report.warnings.find((line) => line.includes('cannot list'));
///   expect(warning).toContain('so the run does not fail for it');
///   expect(warning).not.toContain('from that copy');
/// });
/// ```
#[test]
fn a_failed_listing_says_no_copy_was_read_when_the_install_put_the_version_there() {
    let rig = Rig::new();
    let mut tool: LockedTool = rig.tool("bin");
    let url: String = tool.artifacts[0].1.url.clone();
    let body: &[u8] = b"the bytes a release of bin holds";
    rig.http.serve(&url, body);
    tool.artifacts = vec![(
        rig.host.platform.clone(),
        Artifact {
            url: url.clone(),
            sha256: sha256_hex(body),
        },
    )];
    let version: String = tool.version.clone();
    let lock: Lock = Lock {
        tools: vec![tool],
    };
    // What: the release unpacks into nothing this test reads; the marker and the rename are what the run needs.
    // Why: the tool is not one this repository publishes, so no program is required for it to finish.
    rig.runner
        .on("tar --extract", Box::new(|_request: &RunRequest| {}));
    let tool_dir: PathBuf = tool_directory(&rig.scope.toolchain_root, "bin").expect("a plain name");
    fs::create_dir_all(&tool_dir).expect("make the tool's directory");
    let _mode = TheModePutBack::making_unreadable(&tool_dir);

    let report: SyncReport = sync_tools(&context_of(&rig, &rig.scope, &lock), None);
    assert_eq!(
        report.installed,
        vec!["bin".to_string()],
        "the install put the version at its name: {report:?}"
    );
    assert!(
        report.failed.is_empty(),
        "the recovery's failure does not fail the run: {report:?}"
    );
    let warning: &String = report
        .warnings
        .iter()
        .find(|warning| return warning.contains("cannot list"))
        .unwrap_or_else(|| panic!("the recovery's own words are reported: {report:?}"));
    assert!(
        warning.ends_with(&format!(
            "; the install this run went on to make put bin {version} at its name, so the run does not fail for it"
        )),
        "the suffix says what this run then did: {warning}"
    );
    assert!(
        !warning.contains("from that copy"),
        "no copy was read, so the warning claims nothing about one: {warning}"
    );
}
