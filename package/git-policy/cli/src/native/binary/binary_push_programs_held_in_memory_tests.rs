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
//! the stand-in program that watches a push writes only whether it saw the fake one,
//! never the value,
//! and no control prints the environment of any process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await wrappedCommit({ GIT_SSH_COMMAND: FAKE_ONE }); expect(filesHolding(FAKE_ONE)).toEqual([]);
//! service.stopAll(); service.serve(); await pushStops({ words: [branch, 'started again since', 'git push'] });
//! ```

use super::commit::{keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, decline_in_process_lfs_scan, executable, fixture, git, give_service_environment, observe_raw, repository, start_fixture_service, stderr_of,
    wrapper_first_path, FixtureService,
};
use git_policy_cli::service_fault::AFTER_PROMOTE;
use git_policy_cli::service_work::work_directory;
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

/// The line the stand-in writes when it runs.
const STAND_IN_RUNS: &str = "lfs run";

/// The folder the stand-in `git-lfs` of these controls lives in,
/// outside the fixture:
/// it is the instrument that says whether a push carried the values,
/// so it must not be provable that it holds one of them.
fn stand_in_folder(name: &str) -> PathBuf {
    return PathBuf::from(std::env::var_os("HOME").expect("a home directory"))
        .join("temp/agent/lfs-push-programs-are-held-in-memory-and-never-written-scratch")
        .join(format!("stand-in-git-lfs-{name}"));
}

/// A repository with one tracked file,
/// its bare origin,
/// Git LFS keys,
/// a stand-in `git-lfs` that reports what the push's environment held,
/// and a service whose own `PATH` holds no `git-lfs`.
struct World {
    /// The fixture root.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The bare origin.
    origin: PathBuf,
    /// Where the stand-in writes what it saw.
    record: PathBuf,
    /// The service of the fixture.
    service: FixtureService,
    /// The `PATH` the shell that commits in this control searches,
    /// which holds that shell's own stand-in `git-lfs` and not the service's list.
    shell_path: OsString,
}

/// What:
///  The stand-in `git-lfs`,
///  which says whether the push searched the fixture's own `PATH` and whether it carried the fake values,
///  and writes no value of them into any file.
/// Why:
///   The control then searches every file for the two command lines and may not find one;
/// a stand-in that recorded them would fail its own search,
/// and a search that is allowed to see them proves nothing.
fn stand_in_source(record: &Path, fixture_bin: &OsStr, stand_in: &Path) -> String {
    return format!(
        "#!/bin/sh\n\
         {{\n\
         printf '{STAND_IN_RUNS}\\n'\n\
         case \"$GIT_SSH_COMMAND\" in\n\
         '{FAKE_TRANSPORT_ONE}') printf 'ssh=one\\n' ;;\n\
         '{FAKE_TRANSPORT_TWO}') printf 'ssh=two\\n' ;;\n\
         '') printf 'ssh=unset\\n' ;;\n\
         *) printf 'ssh=another\\n' ;;\n\
         esac\n\
         case \"$GIT_PROXY_COMMAND\" in\n\
         '{FAKE_PROXY}') printf 'proxy=the-fake\\n' ;;\n\
         '') printf 'proxy=unset\\n' ;;\n\
         *) printf 'proxy=another\\n' ;;\n\
         esac\n\
         case \":$PATH:\" in\n\
         *\":{fixture_bin}:\"*) printf 'path-fixture-bin=yes\\n' ;;\n\
         *) printf 'path-fixture-bin=no\\n' ;;\n\
         esac\n\
         case \":$PATH:\" in\n\
         *\":{stand_in}:\"*) printf 'path-stand-in=yes\\n' ;;\n\
         *) printf 'path-stand-in=no\\n' ;;\n\
         esac\n\
         }} >> '{record}'\n\
         exit 0\n",
        record = record.display(),
        fixture_bin = fixture_bin.to_string_lossy(),
        stand_in = stand_in.display(),
    );
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
    let record: PathBuf = fixture.root.join("lfs-saw.txt");
    // One folder per control: the four controls of this file run side by side in one process, and a shared stand-in would
    //  report one control's push into another control's record.
    let stand_in: PathBuf = stand_in_folder(name);
    std::fs::create_dir_all(stand_in.as_path()).expect("the stand-in's own folder");
    executable(stand_in.join("git-lfs").as_path(), stand_in_source(record.as_path(), fixture.root.join("bin").as_os_str(), stand_in.as_path()).as_bytes());
    let shell_path: OsString = shell_path(&fixture, stand_in.as_path());
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return World { fixture, repo, origin, record, service, shell_path };
}

/// What:
///  The `PATH` the shell that commits in these controls searches:
///  the fixture's wrapper first, then the stand-in `git-lfs`, then the host's own programs.
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

/// Every run the stand-in reported,
/// each as the name/value lines it wrote.
fn stand_in_runs(world: &World) -> Vec<Vec<(String, String)>> {
    let text: String = std::fs::read_to_string(&world.record).unwrap_or_default();
    let mut runs: Vec<Vec<(String, String)>> = Vec::new();
    for line in text.lines() {
        if line == STAND_IN_RUNS {
            runs.push(Vec::new());
            continue;
        }
        let Some((name, value)) = line.split_once('=') else { continue };
        if let Some(last) = runs.last_mut() {
            last.push((name.to_owned(), value.to_owned()));
        }
    }
    return runs;
}

/// What one recorded run said `name` was.
fn saw(run: &[(String, String)], name: &str) -> String {
    return run.iter().find(|(held, _value)| { return held == name; }).map(|(_held, value)| { return value.clone(); }).unwrap_or_default();
}

/// Wait until the stand-in ran `count` times,
/// and answer every run it recorded.
fn wait_for_runs(world: &World, count: usize, what: &str) -> Vec<Vec<(String, String)>> {
    let began: Instant = Instant::now();
    while stand_in_runs(world).len() < count {
        assert!(
            began.elapsed() < PATIENCE,
            "{what}: the stand-in `git-lfs` ran {} of {count} times; the queue holds {}",
            stand_in_runs(world).len(),
            super::support::work_listing(work_directory(git_dir(world).as_path()).as_path())
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    return stand_in_runs(world);
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
    for root in [world.fixture.root.clone(), world.service.runtime.clone()] {
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
///  A commit whose shell set both command lines, and a push typed in the same shell,
///  leave neither value in any file,
///  and the search that says so finds a value planted on purpose.
/// Why:
///   Those two lines can hold a password,
/// the record an unreleased branch wrote put them on the disk,
/// and this release writes nothing of them:
/// a search that never finds anything would prove nothing,
/// so the planted file is the positive control.
#[test]
fn a_commit_and_a_typed_push_write_neither_command_line_anywhere() {
    let world: World = world("push-programs-nothing-written");
    let commit: std::process::Output = run_to_its_end(&world, &["commit", "--quiet", "--message=one", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert!(commit.status.success(), "the commit lands: {}", String::from_utf8_lossy(commit.stderr.as_slice()));
    let runs: Vec<Vec<(String, String)>> = wait_for_runs(&world, 1, "the commit's push ran");
    assert_eq!(saw(runs[0].as_slice(), "path-fixture-bin"), "yes", "the push searched the committing shell's `PATH`, which holds the fixture's own programs: {:?}", runs[0]);
    assert_eq!(saw(runs[0].as_slice(), "path-stand-in"), "yes", "and found `git-lfs` only where that shell keeps it, since the service's own `PATH` does not: {:?}", runs[0]);
    assert_eq!(saw(runs[0].as_slice(), "ssh"), "one", "the push reached the remote the way the committing shell does: {:?}", runs[0]);
    assert_eq!(saw(runs[0].as_slice(), "proxy"), "the-fake", "and named its proxy command: {:?}", runs[0]);
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
    world.service.stop_all();
    remove_settled(&world.fixture);
}

/// What:
///  Two commits of one branch, each from a shell with its own transport,
///  are pushed with the settings of the command that asked for each.
/// Why:
///   The values belong to one request and never to a worktree or a branch:
/// a push that took a later command's settings would reach the remote a way nobody asked for.
#[test]
fn each_push_uses_the_settings_of_the_command_that_asked_for_it() {
    let world: World = world("push-programs-one-each");
    let first: std::process::Output = run_to_its_end(&world, &["commit", "--quiet", "--message=one", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert!(first.status.success(), "the first commit lands: {}", String::from_utf8_lossy(first.stderr.as_slice()));
    let after_first: Vec<Vec<(String, String)>> = wait_for_runs(&world, 1, "the first commit's push ran");
    let second: std::process::Output = run_to_its_end(&world, &["commit", "--quiet", "--message=two", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_TWO)]);
    assert!(second.status.success(), "the second commit lands: {}", String::from_utf8_lossy(second.stderr.as_slice()));
    let runs: Vec<Vec<(String, String)>> = wait_for_runs(&world, 2, "the second commit's push ran");
    assert_eq!(saw(after_first[0].as_slice(), "ssh"), "one", "the first push speaks the first command's transport: {:?}", runs);
    assert_eq!(saw(after_first[0].as_slice(), "proxy"), "the-fake", "and names its proxy command: {:?}", runs);
    assert_eq!(saw(runs[1].as_slice(), "ssh"), "two", "the second push speaks the second command's transport: {:?}", runs);
    assert_eq!(saw(runs[1].as_slice(), "proxy"), "unset", "and the proxy command the second command did not set stays unset: {:?}", runs);
    world.service.stop_all();
    remove_settled(&world.fixture);
}

/// What:
///  A push a restarted service has to try again stops with the words that say so,
///  and the command those words print pushes the branch.
/// Why:
///   The settings live only in the memory of the service that received them,
/// so after a restart the push must not run through the manager's transport;
/// it stops within its bound,
/// and the person is handed the one command that sends the commit.
#[test]
fn a_push_a_restarted_service_has_to_try_again_stops_and_names_the_command() {
    let world: World = world("push-programs-restarted");
    // A wrapper killed after it promoted the push record leaves it waiting, and the service still held the settings.
    let mut committing: Command = wrapped_commit(&world.fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--allow-empty"]);
    committing.env("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE);
    committing.env(git_policy_cli::service_fault::FAULT_VARIABLE, AFTER_PROMOTE);
    let killed: std::process::Output = committing.output().expect("the wrapped commit runs");
    assert!(!killed.status.success(), "the wrapper was killed at the fault point");
    let commit: String = head_of(&world);
    assert_ne!(origin_main(&world), commit, "the push has not been made");
    // The values are on no disk even with the client gone.
    assert!(files_holding(&world, FAKE_TRANSPORT_ONE).is_empty(), "a killed client leaves no file holding the value: {:?}", files_holding(&world, FAKE_TRANSPORT_ONE));

    // What the manager does when its service has gone: a new one, which holds nothing of the old one's memory.
    world.service.stop_all();
    world.service.serve(&world.fixture, git_dir(&world).as_path());
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
    world.service.stop_all();
    remove_settled(&world.fixture);
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
    world.service.stop_all();
    remove_settled(&world.fixture);
}
