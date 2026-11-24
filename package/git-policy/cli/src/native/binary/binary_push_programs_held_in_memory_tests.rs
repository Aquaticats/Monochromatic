//! What:
//!  The programs and transport settings a commit's push needs,
//!  carried to the commit service in its request and held only there:
//!  the search that finds no value of them on any disk,
//!  a push that uses the values of the command that asked for it,
//!  a push a restarted service has to try again,
//!  and the record an unreleased branch wrote.
//! Why:
//!  `GIT_SSH`,
//! `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are whole command lines that can hold a password,
//! so the owner's decision of 2026-11-12T11:58:45-05:00 is "Memory only";
//! a push must still search the programs of the shell that asked for it,
//! and a push whose settings are gone with a service restart must stop and say what to run rather than speak a
//!  transport nobody chose.
//! Every value in these controls is plainly fake,
//! the stand-in `git-lfs` of a push reads no variable and writes nothing,
//! and no control prints the environment of any process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await wrappedCommit({ GIT_SSH_COMMAND: FAKE_ONE }); expect(filesHolding(FAKE_ONE)).toEqual([]);
//! service.stopAll(); service.serve(); await pushStops({ words: [branch, 'started again since', 'git push'] });
//! ```

use super::commit::{keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, decline_in_process_lfs_scan, executable, fixture, git, give_service_environment, observe_raw, repository, stderr_of, wrapper_first_path,
    FixtureService,
};
use git_policy_cli::service_fault::AFTER_THE_VERDICT;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{HELD, state_directory};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(120);

/// The plainly fake command line a commit's shell chooses its transport with.
const FAKE_TRANSPORT_ONE: &str = "ssh -i /nowhere/FAKE_KEY_ONE_4b1d";

/// A second fake command line,
/// so two pushes can be told apart.
const FAKE_TRANSPORT_TWO: &str = "ssh -i /nowhere/FAKE_KEY_TWO_9c7e";

/// The plainly fake proxy command a commit's shell names.
const FAKE_PROXY: &str = "nc -x fakeuser:FAKE_PROXY_PASSWORD_2a6f proxy.invalid 1080";

/// What:
///  The folder the stand-in `git-lfs` of one control lives in.
/// Why:
///   It stands in for a program the shell that commits has and the service's own list does not,
/// and it is reached through the list the request carries,
/// so it must not sit anywhere a sweep of the fixture reads:
/// it sits beside the fixture,
/// in the fixture root the run was given,
///  never under the shared `HOME`,
/// where two sessions running this control at once wrote and removed one folder
/// (L3 of the alpha.45 release pass).
/// The name carries the process id,
/// so two runs never share it either,
/// and a control removes its own folder when it ends.
fn stand_in_folder(name: &str) -> PathBuf {
    return super::support::fixture_root().join(format!("lfs-push-programs-held-in-memory-stand-in-git-lfs-{}-{name}", std::process::id()));
}

/// A repository with one tracked file,
/// its bare origin,
/// Git LFS keys,
/// a stand-in `git-lfs` for the push to find,
/// and a service whose own `PATH` holds no `git-lfs`.
struct World {
    /// The fixture root.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The bare origin.
    origin: PathBuf,
    /// The `PATH` the shell that commits in this control searches,
    /// which holds that shell's own stand-in `git-lfs` and not the service's list.
    shell_path: OsString,
    /// The stand-in's own folder,
    /// removed when the control ends.
    stand_in: PathBuf,
}

/// What:
///  The stand-in `git-lfs` of these controls:
///  a program that does nothing and says it did,
///  because the push of a repository with LFS keys runs one before it publishes.
/// Why:
///   A repository that carries `filter.lfs.*` and `lfs.*` makes every push start the upload first,
///  so a shell with no `git-lfs` anywhere is refused and nothing is pushed.
/// Two controls of this file failed that way in the gate of 2026-11-21T13:25:00-05:00,
///  where this program was not on the shell's `PATH`,
///  and it stands in for the program the person's shell holds and the service's own list does not.
/// It reads no variable and writes nothing,
/// so the search that a commit and a typed push leave no command line on any disk still holds.
fn stand_in_source() -> String {
    return String::from("#!/bin/sh\nexit 0\n");
}

fn world(name: &str) -> World {
    let fixture: Fixture = fixture(name);
    // The manager's own list holds no `git-lfs`: only the shell that commits does, through the fixture's own `PATH`.
    give_service_environment(&fixture, "PATH", "/usr/bin:/bin");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", origin.to_str().expect("a root that is text")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    git(&fixture, &repo, &["remote", "add", "origin", origin.to_str().expect("a root that is text")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    git(&fixture, &repo, &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
    decline_in_process_lfs_scan(&fixture, repo.as_path());
    // One folder per control: the controls of this file run side by side in one process, and a shared stand-in would be
    //  written and removed by two of them at once.
    let stand_in: PathBuf = stand_in_folder(name);
    std::fs::create_dir_all(stand_in.as_path()).expect("the stand-in's own folder");
    executable(stand_in.join("git-lfs").as_path(), stand_in_source().as_bytes());
    let shell_path: OsString = shell_path(&fixture, stand_in.as_path());
    // The first wrapped command starts the fixture's own service, in the runtime directory it hands that command, and this
    //  control stops and starts that one (`the_service`).
    return World { fixture, repo, origin, shell_path, stand_in };
}

/// What:
///  The fixture's own service, which every wrapped command of this fixture reaches.
/// Why:
///   The settings a push needs live in the memory of the service the command spoke to,
/// so a control that takes them away has to stop that service.
/// A service the control starts itself sits in a runtime directory of its own,
/// which no command of this fixture is told about:
/// it holds none of the settings,
/// and stopping it proves nothing (issue 6447).
fn the_service(fixture: &Fixture) -> std::sync::MutexGuard<'_, Option<FixtureService>> {
    return fixture.service.lock().expect("the fixture's service");
}

/// The runtime directory the fixture's service listens in,
/// which every wrapped command of this fixture is told.
fn service_runtime_of(fixture: &Fixture) -> PathBuf {
    let service: std::sync::MutexGuard<'_, Option<FixtureService>> = the_service(fixture);
    let started: &FixtureService = service.as_ref().expect("a wrapped command started the fixture's service");
    return started.runtime.clone();
}

/// What the manager does when the worktree's service has gone:
/// ends it and everything it started,
/// and with the process its memory of the settings.
fn stop_the_service(fixture: &Fixture) {
    let service: std::sync::MutexGuard<'_, Option<FixtureService>> = the_service(fixture);
    let started: &FixtureService = service.as_ref().expect("a wrapped command started the fixture's service");
    started.stop_all();
}

/// What:
///  Starts the worktree's service again,
///  in the runtime directory the commands of this fixture already look in.
/// Why:
///   The manager starts one instance per worktree,
/// and the fixture has none,
/// so the control starts the new one where the last one listened and holds nothing of its memory.
fn serve_the_service(fixture: &Fixture, git_dir: &Path) {
    let service: std::sync::MutexGuard<'_, Option<FixtureService>> = the_service(fixture);
    let started: &FixtureService = service.as_ref().expect("a wrapped command started the fixture's service");
    started.serve(fixture, git_dir);
}

/// What:
///  The `PATH` the shell that commits in these controls searches:
///  the fixture's wrapper first,
/// then the stand-in `git-lfs`,
/// then the host's own programs.
/// Why:
///   The push must find `git-lfs` where that shell finds it and nowhere else,
/// and the fixture's wrapper must be what runs `git` for the commands here.
fn shell_path(fixture: &Fixture, stand_in: &Path) -> OsString {
    let mut path: OsString = wrapper_first_path(fixture);
    path.push(":");
    path.push(stand_in);
    path.push(":/usr/bin:/bin");
    return path;
}

/// The Git directory of the working repository.
fn git_dir(world: &World) -> PathBuf {
    return world.repo.join(".git");
}

/// The names of the records waiting in one state of a worktree's queue,
/// sorted,
/// and none when that state was never made.
fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.sort();
    return names;
}

/// Run one wrapped command in the repository,
/// with the variables the shell that runs it sets.
fn wrapped_with(world: &World, arguments: &[&str], variables: &[(&str, &str)]) -> Observed {
    let mut command: Command = wrapped_commit(&world.fixture, world.repo.as_path(), arguments);
    command.env("PATH", &world.shell_path);
    for (name, value) in variables {
        command.env(name, value);
    }
    return observe_raw(&mut command, b"");
}

/// What:
///  Run one wrapped command to its end without waiting for the service's after-commit work.
/// Why:
///   The wait that settles the queue in `observe` is the suite's own patience for every kind of work
/// a commit leaves,
/// and these controls wait for the one record they are about;
/// the command itself is already bounded by the fixture's own supervisor.
fn run_to_its_end(world: &World, arguments: &[&str], variables: &[(&str, &str)]) -> std::process::Output {
    let mut command: Command = wrapped_commit(&world.fixture, world.repo.as_path(), arguments);
    command.env("PATH", &world.shell_path);
    for (name, value) in variables {
        command.env(name, value);
    }
    return command.output().expect("the wrapped command runs");
}

/// Wait until `condition` holds,
/// and fail the control that waited with `what` when it never does.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Every regular file under `root`,
/// with its bytes.
fn files_under(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() {
                continue;
            }
            let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            found.push((path, bytes));
        }
    }
    return found;
}

/// What:
///  The paths whose bytes hold `value`,
///  asked of every file under the fixture and under the runtime directory the service listens in.
/// Why:
///  The values that may hold a password must reach no file at all:
/// the Git directory,
/// the runtime directory,
/// the state directory and the home are all under these two roots,
/// so one walk covers them.
fn files_holding(world: &World, value: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for root in [world.fixture.root.clone(), service_runtime_of(&world.fixture)] {
        for (path, bytes) in files_under(root.as_path()) {
            if bytes.windows(value.len()).any(|window| { return window == value.as_bytes(); }) {
                found.push(path);
            }
        }
    }
    return found;
}

/// What the bare origin's `main` names,
/// or the empty text when it names nothing.
fn origin_main(world: &World) -> String {
    let output: std::process::Output = super::support::direct(&world.fixture).current_dir(world.origin.as_path()).args(["rev-parse", "--verify", "--quiet", "refs/heads/main"]).output().expect("real Git answers");
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// The commit the repository's `main` names.
fn head_of(world: &World) -> String {
    return String::from_utf8_lossy(git(&world.fixture, world.repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice()).trim().to_owned();
}

/// What:
///  A commit whose shell set both command lines,
/// and a push typed in the same shell,
///  leave neither value in any file,
///  and the search that says so finds a value planted on purpose.
/// Why:
///   Those two lines can hold a password,
/// the record an unreleased branch wrote put them on the disk,
/// and this release writes nothing of them:
/// a search that never finds anything would prove nothing,
/// so the planted file is the positive control.
///  Which list and which transport reached the push are proved beside this control,
///  not here:
/// the wrapper starts no Git LFS program for a push that introduces no object (`lfs_object_plan::narrow_plan`),
/// so a stand-in program is no instrument this control can rely on.
#[test]
fn a_commit_and_a_typed_push_write_neither_command_line_anywhere() {
    let world: World = world("push-programs-nothing-written");
    let commit: std::process::Output = run_to_its_end(&world, &["commit", "--quiet", "--message=one", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert!(commit.status.success(), "the commit lands: {}", String::from_utf8_lossy(commit.stderr.as_slice()));
    // The commit has landed and its push record exists and is settled, so the command's own values have been where they go.
    let commit_oid: String = head_of(&world);
    wait_until("the service settles the commit's push record", &|| {
        let listing: String = super::support::work_listing(work_directory(git_dir(&world).as_path()).as_path());
        return listing.contains("push-") && listing.contains("pending=[]") && listing.contains("claimed=[]");
    });
    assert!(!commit_oid.is_empty(), "the commit the shell made is in the repository");
    // A push typed in the same shell, which runs no part of it in the service.
    let typed: Observed = wrapped_with(&world, &["push", "--quiet", "origin", "main"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_TWO), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert_eq!(typed.code, Some(0), "the typed push goes through: {}", stderr_of(&typed));

    // The positive control first: the search must find a value that is on a disk.
    let planted: PathBuf = world.fixture.root.join("planted-value.txt");
    std::fs::write(planted.as_path(), format!("{FAKE_TRANSPORT_ONE}\n")).expect("a planted value");
    assert_eq!(files_holding(&world, FAKE_TRANSPORT_ONE), vec![planted.clone()], "the search finds a value that is on the disk");
    std::fs::remove_file(planted.as_path()).expect("the planted value goes");

    for value in [FAKE_TRANSPORT_ONE, FAKE_TRANSPORT_TWO, FAKE_PROXY] {
        assert!(files_holding(&world, value).is_empty(), "{value} reaches no file: {:?}", files_holding(&world, value));
    }
    stop_the_service(&world.fixture);
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.stand_in.as_path()).expect("the stand-in's own folder goes");
}

/// What:
///  A push a restarted service has to try again stops with the words that say so,
///  and the command those words print pushes the branch.
/// Why:
///   The settings live only in the memory of the service that received them,
/// so after a restart the push must not run through the manager's transport;
/// it stops within its bound,
/// and the person is handed the one command that sends the commit.
///  The scene the control names is made by the control, never by a race:
/// the command is killed once its gate approved the push and before it promoted the record,
/// so the record waits in `held`,
/// which no round drains.
/// The service that holds the settings is stopped while the record waits there,
/// and only then does the control take the step the killed command would have taken.
/// A push a service that still held the settings ran would complete,
/// which removes the scene's premise rather than refuting anything:
/// the control would then wait out its bound for words about a push nothing had taken the settings from.
#[test]
fn a_push_a_restarted_service_has_to_try_again_stops_and_names_the_command() {
    let world: World = world("push-programs-restarted");
    // A wrapper killed after its gate approved the push leaves the record held, and the service that landed the commit still
    //  holds the settings in its memory.
    let mut committing: Command = wrapped_commit(&world.fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--allow-empty"]);
    committing.env("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE);
    committing.env(git_policy_cli::service_fault::FAULT_VARIABLE, AFTER_THE_VERDICT);
    let killed: std::process::Output = committing.output().expect("the wrapped commit runs");
    assert!(!killed.status.success(), "the wrapper was killed at the fault point");
    let commit: String = head_of(&world);
    assert_ne!(origin_main(&world), commit, "the push has not been made");
    // The approved record is the one thing the killed command left behind, and no round lists the state it waits in.
    let work: PathBuf = work_directory(git_dir(&world).as_path());
    let held: Vec<String> = names_in(state_directory(work.as_path(), HELD).as_path());
    assert_eq!(held.len(), 1, "the push the gate approved is the one record held: {held:?}");
    // The values are on no disk even with the client gone.
    assert!(files_holding(&world, FAKE_TRANSPORT_ONE).is_empty(), "a killed client leaves no file holding the value: {:?}", files_holding(&world, FAKE_TRANSPORT_ONE));

    // What the manager does when its service has gone: the service that held the settings ends, and its memory with it.
    stop_the_service(&world.fixture);
    // What the killed command would have done next: the approved record becomes runnable while no service runs to take it.
    assert!(git_policy_cli::work_item_holding::promote(work.as_path(), held[0].as_str()), "the approved record is promoted");
    assert!(names_in(state_directory(work.as_path(), HELD).as_path()).is_empty(), "the promoted record no longer waits where no round looks");
    // The manager's next instance of this worktree, which holds nothing of the memory the last one had.
    serve_the_service(&world.fixture, git_dir(&world).as_path());
    let notice: PathBuf = git_policy_cli::push_failure_notice::notice_directory(git_dir(&world).as_path());
    wait_until("the push records what stopped it", &|| { return std::fs::read_dir(notice.as_path()).is_ok_and(|entries| { return entries.flatten().next().is_some(); }); });
    // A plain `git status` is not a command whose output a program reads, so it shows the record.
    let shown: Observed = wrapped_with(&world, &["status"], &[]);
    let words: String = stderr_of(&shown);
    assert!(words.contains("was not made"), "the push was not made: {words}");
    assert!(words.contains(world.repo.to_str().expect("a path that is text")), "the repository is named: {words}");
    assert!(words.contains("refs/heads/main"), "the branch is named: {words}");
    assert!(words.contains("has been started again since"), "the restart is what took the settings: {words}");
    assert!(words.contains("nothing was sent through another transport"), "no other transport was used: {words}");
    assert_ne!(origin_main(&world), commit, "and nothing reached the remote");

    // The command the words print, run as printed in the shell that has the settings.
    let line: &str = words.lines().find(|line| { return line.trim_start().starts_with("Run this in "); }).expect("the words print a command");
    let command: &str = line.trim_start().split_once(": ").expect("the line names a command").1;
    assert!(command.starts_with("git push "), "the printed command is one a shell runs: {command}");
    let pushed: std::process::Output = std::process::Command::new("/bin/sh")
        .current_dir(world.repo.as_path())
        .arg("-c")
        .arg(command)
        .env_clear()
        .env("PATH", &world.shell_path)
        .env("HOME", world.fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .expect("the printed command runs");
    assert!(pushed.status.success(), "the printed command pushes: {}\n{}", String::from_utf8_lossy(pushed.stdout.as_slice()), String::from_utf8_lossy(pushed.stderr.as_slice()));
    assert_eq!(origin_main(&world), commit, "the commit the stopped push held is on the remote");
    stop_the_service(&world.fixture);
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.stand_in.as_path()).expect("the stand-in's own folder goes");
}

/// What:
///  A transport setting whose value is not text stops the push with words that name that cause,
///  and not a restart that never happened.
/// Why:
///   The value cannot travel in a request line,
/// so the service holds the name and none of the value;
/// the push must not run through the manager's transport,
/// which is the incident the carried list exists for,
/// and the person must learn why their own value did not arrive (L1 of the alpha.45 release pass).
#[test]
fn a_transport_value_that_could_not_travel_stops_the_push_and_names_that_cause() {
    use std::os::unix::ffi::OsStringExt;
    let world: World = world("push-programs-unspellable-transport");
    let unspellable: OsString = OsString::from_vec(b"ssh -i /nowhere/FAKE_UNSPELLABLE_2c7d_\xff".to_vec());
    let mut committing: Command = wrapped_commit(&world.fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--allow-empty"]);
    committing.env("GIT_SSH_COMMAND", &unspellable);
    let landed: std::process::Output = committing.output().expect("the wrapped commit runs");
    assert!(landed.status.success(), "the commit itself succeeds: {}", String::from_utf8_lossy(landed.stderr.as_slice()));
    let commit: String = head_of(&world);
    assert_ne!(origin_main(&world), commit, "the push has not been made");
    // The service holds the record and none of the bytes that could not travel:
    let notice: PathBuf = git_policy_cli::push_failure_notice::notice_directory(git_dir(&world).as_path());
    wait_until("the push records what stopped it", &|| {
        return std::fs::read_dir(notice.as_path()).is_ok_and(|entries| { return entries.flatten().next().is_some(); });
    });
    let shown: Observed = wrapped_with(&world, &["status"], &[]);
    let words: String = stderr_of(&shown);
    assert!(words.contains("was not made"), "the push was not made: {words}");
    assert!(words.contains("not valid text"), "the value that could not travel is named as such: {words}");
    assert!(words.contains("could not travel with the push's request"), "the request line is named as what could not hold it: {words}");
    assert!(!words.contains("started again"), "no restart is blamed for a value that never travelled: {words}");
    assert!(words.contains("nothing was sent through another transport"), "no other transport was used: {words}");
    assert!(words.lines().any(|line| { return line.trim_start().starts_with("Run this in ") && line.contains("git push "); }), "the command that pushes is printed: {words}");
    assert_ne!(origin_main(&world), commit, "and nothing reached the remote");
    stop_the_service(&world.fixture);
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.stand_in.as_path()).expect("the stand-in's own folder goes");
}

/// What:
///  The record an unreleased branch wrote,
///  and a staging file of one of its writes,
///  are removed by the first command that meets them.
/// Why:
///   The record holds command lines that can carry a password;
/// nothing writes or reads it any more,
/// so a command that finds it is the only chance it leaves the disk.
#[test]
fn the_record_an_unreleased_branch_wrote_is_removed_by_the_first_command() {
    let world: World = world("push-programs-record-removed");
    let folder: PathBuf = git_dir(&world).join("monochromatic-git-policy-cli");
    std::fs::create_dir_all(folder.as_path()).expect("the folder the work records use");
    let record: PathBuf = folder.join("push-programs.json");
    let staged: PathBuf = folder.join(".push-programs.json.Q1w2E.partial");
    for path in [&record, &staged] {
        std::fs::write(path, format!("{{\"v\":1,\"variables\":{{\"GIT_SSH_COMMAND\":\"{FAKE_TRANSPORT_ONE}\"}}}}\n")).expect("a record of that branch's shape");
    }
    let met: Observed = wrapped_with(&world, &["status", "--porcelain"], &[]);
    assert_eq!(met.code, Some(0), "the command runs: {}", stderr_of(&met));
    assert!(!record.exists(), "the record is gone after the first command that meets it");
    assert!(!staged.exists(), "and so is the staging file of one of its writes");
    assert!(files_holding(&world, FAKE_TRANSPORT_ONE).is_empty(), "no value of that record is left on any disk: {:?}", files_holding(&world, FAKE_TRANSPORT_ONE));
    assert!(folder.is_dir(), "the work records' own folder stays");
    stop_the_service(&world.fixture);
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.stand_in.as_path()).expect("the stand-in's own folder goes");
}
