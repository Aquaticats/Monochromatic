//! What:
//!  The programs and transport settings a commit's push needs,
//!  carried to the commit service in its request and held only there:
//!  the search that finds no value of them on any disk,
//!  a push that uses the values of the command that asked for it,
//!  a push a restarted service has to try again,
//!  and the record an unreleased branch wrote.
//! Why:
//!  `GIT_SSH` and `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are whole command lines that can hold a password,
//! so the owner's decision of 2026-11-12T11:58:45-05:00 is "Memory only";
//! a push must still search the programs of the shell that asked for it,
//! and a push whose settings are gone with a service restart must stop and say what to run rather than speak a
//!  transport nobody chose.
//! Every value in these controls is plainly fake,
//! and no control prints the environment of any process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await wrappedCommit({ GIT_SSH_COMMAND: FAKE_ONE }); expect(filesHolding(FAKE_ONE)).toEqual([]);
//! service.stopAll(); await pushStops({ words: [branch, 'started again since', 'git push'] });
//! ```

use super::commit::{keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, decline_in_process_lfs_scan, executable, fixture, git, give_service_environment, observe, repository, start_fixture_service, stderr_of, FixtureService};
use git_policy_cli::service_fault::AFTER_PROMOTE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(90);

/// The plainly fake command line a commit's shell chooses its transport with.
const FAKE_TRANSPORT_ONE: &str = "ssh -i /nowhere/FAKE_KEY_ONE_4b1d";

/// A second fake command line,
/// so two pushes can be told apart.
const FAKE_TRANSPORT_TWO: &str = "ssh -i /nowhere/FAKE_KEY_TWO_9c7e";

/// The plainly fake proxy command a commit's shell names.
const FAKE_PROXY: &str = "nc -x fakeuser:FAKE_PROXY_PASSWORD_2a6f proxy.invalid 1080";

/// A repository with one tracked file,
/// its bare origin,
/// Git LFS keys,
/// a stand-in `git-lfs` that records the environment it was given,
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
}

/// The line the stand-in writes when it runs.
const STAND_IN_RUNS: &str = "lfs run";

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
    // The stand-in records the five variables the push's children receive, and nothing else of its environment.
    executable(
        fixture.root.join("bin/git-lfs").as_path(),
        format!(
            "#!/bin/sh\nprintf '{STAND_IN_RUNS}\\nPATH=%s\\nGIT_SSH_COMMAND=%s\\nGIT_PROXY_COMMAND=%s\\nGIT_SSH=%s\\nGIT_EXEC_PATH=%s\\n' \
             \"$PATH\" \"$GIT_SSH_COMMAND\" \"$GIT_PROXY_COMMAND\" \"$GIT_SSH\" \"$GIT_EXEC_PATH\" >> '{record}'\nexit 0\n",
            record = record.display()
        )
        .as_bytes(),
    );
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return World { fixture, repo, origin, record, service };
}

/// Run one wrapped command in the repository,
/// with the variables the shell that runs it sets.
fn wrapped_with(world: &World, arguments: &[&str], variables: &[(&str, &str)]) -> Observed {
    let mut command: Command = wrapped_commit(&world.fixture, world.repo.as_path(), arguments);
    for (name, value) in variables {
        command.env(name, value);
    }
    return observe(&mut command, b"");
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

/// Every run the stand-in recorded,
/// each as the five variables it was given.
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
fn wait_for_runs(world: &World, count: usize) -> Vec<Vec<(String, String)>> {
    wait_until(format!("the stand-in `git-lfs` ran {count} times").as_str(), &|| { return stand_in_runs(world).len() >= count; });
    return stand_in_runs(world);
}

/// Every regular file under `root`,
/// read,
/// with the paths that could not be read left to the caller's search.
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
    let mut roots: Vec<PathBuf> = vec![world.fixture.root.clone(), world.service.runtime.clone()];
    let mut found: Vec<PathBuf> = Vec::new();
    for root in roots.drain(..) {
        for (path, bytes) in files_under(root.as_path()) {
            if bytes.windows(value.len()).any(|window| { return window == value.as_bytes(); }) {
                found.push(path);
            }
        }
    }
    return found;
}

/// Whether the bare origin holds `commit` on `main`.
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
    let commit: Observed = wrapped_with(&world, &["commit", "--quiet", "--message=one", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert_eq!(commit.code, Some(0), "the commit lands: {}", stderr_of(&commit));
    let runs: Vec<Vec<(String, String)>> = wait_for_runs(&world, 1);
    assert_eq!(saw(runs[0].as_slice(), "GIT_SSH_COMMAND"), FAKE_TRANSPORT_ONE, "the push reached the remote the way the committing shell does");
    // A push typed in the same shell, which runs no part of it in the service.
    let typed: Observed = wrapped_with(&world, &["push", "--quiet", "origin", "main"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_TWO), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert_eq!(typed.code, Some(0), "the typed push goes through: {}", stderr_of(&typed));

    // The positive control first: the search must find a value that is on a disk.
    let planted: PathBuf = world.fixture.root.join("planted-value.txt");
    std::fs::write(planted.as_path(), format!("{FAKE_TRANSPORT_ONE}\n")).expect("a planted value");
    assert_eq!(files_holding(&world, FAKE_TRANSPORT_ONE), vec![planted.clone()], "the search finds a value that is on the disk");
    std::fs::remove_file(planted.as_path()).expect("the planted value goes");

    for value in [FAKE_TRANSPORT_ONE, FAKE_TRANSPORT_TWO, FAKE_PROXY] {
        assert!(files_holding(&world, value).is_empty(), "{value} reaches no file");
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
    let first: Observed = wrapped_with(&world, &["commit", "--quiet", "--message=one", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE), ("GIT_PROXY_COMMAND", FAKE_PROXY)]);
    assert_eq!(first.code, Some(0), "the first commit lands: {}", stderr_of(&first));
    let after_first: Vec<Vec<(String, String)>> = wait_for_runs(&world, 1);
    let second: Observed = wrapped_with(&world, &["commit", "--quiet", "--message=two", "--allow-empty"], &[("GIT_SSH_COMMAND", FAKE_TRANSPORT_TWO)]);
    assert_eq!(second.code, Some(0), "the second commit lands: {}", stderr_of(&second));
    let runs: Vec<Vec<(String, String)>> = wait_for_runs(&world, 2);
    assert_eq!(saw(after_first[0].as_slice(), "GIT_SSH_COMMAND"), FAKE_TRANSPORT_ONE, "the first push speaks the first command's transport");
    assert_eq!(saw(after_first[0].as_slice(), "GIT_PROXY_COMMAND"), FAKE_PROXY, "and names its proxy command");
    assert_eq!(saw(runs[1].as_slice(), "GIT_SSH_COMMAND"), FAKE_TRANSPORT_TWO, "the second push speaks the second command's transport");
    assert_eq!(saw(runs[1].as_slice(), "GIT_PROXY_COMMAND"), "", "and the proxy command the second command did not set stays unset");
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
    // A wrapper killed after it promoted the push record leaves it waiting, and the service still holds the settings.
    let mut committing: Command = wrapped_commit(&world.fixture, world.repo.as_path(), &["commit", "--quiet", "--message=one", "--allow-empty"]);
    committing.env("GIT_SSH_COMMAND", FAKE_TRANSPORT_ONE);
    committing.env(git_policy_cli::service_fault::FAULT_VARIABLE, AFTER_PROMOTE);
    let killed: std::process::Output = committing.output().expect("the wrapped commit runs");
    assert!(!killed.status.success(), "the wrapper was killed at the fault point");
    let commit: String = head_of(&world);
    assert_ne!(origin_main(&world), commit, "the push has not been made");
    // The values are on no disk even with the client gone.
    assert!(files_holding(&world, FAKE_TRANSPORT_ONE).is_empty(), "a killed client leaves no file holding the value");

    // What the manager does when its service has gone: the next command starts one that holds nothing of the old one.
    world.service.stop_all();
    let woken: Observed = wrapped_with(&world, &["status", "--porcelain"], &[]);
    assert_eq!(woken.code, Some(0), "the command runs: {}", stderr_of(&woken));
    let notice: PathBuf = git_policy_cli::push_failure_notice::notice_directory(world.repo.join(".git").as_path());
    wait_until("the push records what stopped it", &|| { return std::fs::read_dir(notice.as_path()).is_ok_and(|entries| { return entries.flatten().next().is_some(); }); });
    let shown: Observed = wrapped_with(&world, &["status", "--porcelain"], &[]);
    let words: String = stderr_of(&shown);
    assert!(words.contains("was not made"), "the push was not made: {words}");
    assert!(words.contains(world.repo.to_str().expect("a path that is text")), "the repository is named: {words}");
    assert!(words.contains("refs/heads/main"), "the branch is named: {words}");
    assert!(words.contains("has been started again since"), "the restart is what took the settings: {words}");
    assert!(words.contains("nothing was sent through another transport"), "no other transport was used: {words}");
    assert_ne!(origin_main(&world), commit, "and nothing reached the remote");

    // The command the words print, run as printed in the shell that has the settings.
    let line: &str = words.lines().find(|line| { return line.starts_with("Run this in "); }).expect("the words print a command");
    let command: &str = line.split_once(": ").expect("the line names a command").1;
    let pushed: std::process::Output = std::process::Command::new("/bin/sh")
        .current_dir(world.repo.as_path())
        .arg("-c")
        .arg(command)
        .env("PATH", format!("{}:/usr/bin:/bin", world.fixture.root.join("bin").display()))
        .env("HOME", world.fixture.root.join("home"))
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
    let folder: PathBuf = world.repo.join(".git/monochromatic-git-policy-cli");
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
    assert!(files_holding(&world, FAKE_TRANSPORT_ONE).is_empty(), "no value of that record is left on any disk");
    world.service.stop_all();
    remove_settled(&world.fixture);
}
