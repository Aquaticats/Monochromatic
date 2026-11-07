//! What:
//!  A closed terminal and the other signals a session sends:
//! `SIGHUP`,
//! `SIGINT`,
//! `SIGTERM` and `SIGQUIT` to the wrapper alone
//!  while its Git commit waits,
//! and `SIGHUP` to the wrapper's whole process group while the commit sits in the editor.
//! Why:
//!  The README promises that a closed terminal does not strand the repository.
//! A signal to the wrapper alone must reach the
//!  Git it waits for through the wrapper's own relay (`signal_relay.rs`),
//! and the wrapper must end with the signal's status.
//!  The earlier controls signalled `timeout`,
//! which forwards the signal to its whole group,
//! so Git and the editor were
//!  signalled directly and the relay was never the thing tested;
//! the `SIGQUIT` core dumps of Git on the development host came
//!  from that shape (Git's cleanup re-raises the quit it got,
//! and the quit's default action dumps core).
//! Here the wrapper
//!  runs with no `timeout` (`binary_fault_run.rs`),
//! the signal goes to its process ID alone,
//! and the Git it starts is a
//!  stand-in script that records which signals reached it and then dies of that signal with core dumps off.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wrapper = spawn(git, ['commit'], { detached: true }); process.kill(wrapper.pid, 'SIGQUIT'); // not the group
//! expect(standInGit.receivedSignals).toEqual(['QUIT']); expect(await wrapper.exit).toBe(128 + 3);
//! ```

use super::commit::{remove_settled, run_commit};
use super::fault::{staged_repository, verify_recovery};
use super::fault_run::{is_running, run_watched, send, spawn_watched, unbounded_wrapper};
use super::support::{Fixture, executable, fixture, stalling_editor};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// How long a control waits for a process it expects to be gone:
/// a hang guard,
/// never part of a verdict.
const GONE_PATIENCE: Duration = Duration::from_secs(30);

/// A `git` placed after the wrapper on `PATH`,
/// so it is the real Git the wrapper starts.
/// It runs the real Git for every call
///  except the commit of the preparation (`commit --quiet`),
/// which it turns into a stalled Git:
/// it announces itself,
/// records
///  its process ID,
/// waits,
/// and on a signal records the signal's name and ends with the status Git gives a child it lost to that
///  signal (`128 + signal`;
/// a shell cannot die of a quit it has reset,
/// which is why it exits).
///  Returns the directory to add to `PATH`.
fn stand_in_git(fixture: &Fixture, log: &Path, started: &Path, process_id: &Path) -> PathBuf {
    let directory: PathBuf = fixture.root.join("shim");
    std::fs::create_dir(&directory).expect("shim directory");
    let script: String = String::from(
        "#!/bin/sh
case \" $* \" in
  *\" commit --quiet\"*)
    ulimit -c 0
    stall=
    for pair in INT:130 TERM:143 HUP:129 QUIT:131; do
      name=${pair%%:*}
      code=${pair##*:}
      trap \"/usr/bin/kill \\$stall 2>/dev/null; echo $name >> 'LOG'; exit $code\" $name
    done
    echo \"$$\" > 'PID'
    printf started > 'STARTED'
    ( while LIVES; do /usr/bin/sleep 0.05; done ) </dev/null >/dev/null 2>&1 &
    stall=$!
    wait $stall
    exit 1
    ;;
esac
exec /usr/bin/git \"$@\"
",
    )
    .replace("LOG", &log.display().to_string())
    .replace("PID", &process_id.display().to_string())
    .replace("STARTED", &started.display().to_string())
    // Last, because the condition holds a path that no other name may be looked for in.
    .replace("LIVES", &super::support::while_test_lives(fixture.root.as_path()));
    executable(directory.join("git").as_path(), script.as_bytes());
    return directory;
}

/// Waits until the process is gone,
/// or fails the control naming it.
fn assert_gone(pid: i32, what: &str) {
    let began: Instant = Instant::now();
    while is_running(pid) {
        assert!(began.elapsed() < GONE_PATIENCE, "{what} (process {pid}) still runs {GONE_PATIENCE:?} after the wrapper ended");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Sends `signal` to the wrapper alone while its Git waits,
/// and requires the relay's whole effect:
/// the stand-in Git received that
///  one signal and ended with it,
/// the wrapper ended with `128 + signal`,
/// nothing runs on,
/// and the repository recovers.
fn relayed_to_git(name: &str, label: &str, signal: i32) {
    let fixture: Fixture = fixture(label);
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let log: PathBuf = fixture.root.join("received.log");
    let started: PathBuf = fixture.root.join("git-started");
    let process_id: PathBuf = fixture.root.join("git-pid");
    let shim: PathBuf = stand_in_git(&fixture, log.as_path(), started.as_path(), process_id.as_path());
    // The wrapper first, then the stand-in. `/usr/bin` stays off `PATH`: the wrapper tries the conventional locations of Git
    //  ahead of the rest of `PATH`, so a `/usr/bin/git` on it would be chosen over the stand-in (the stand-in runs the real one
    //  by its absolute path).
    let path: String = format!("{}:{}", fixture.root.join("bin").display(), shim.display());
    let mut command: Command = unbounded_wrapper(&fixture, repo.as_path(), &["commit", "--quiet", "--", "a.txt"]);
    // An editor is named so the wrapper takes the editor route; the stand-in stalls before Git would start it.
    command.env("PATH", &path).env("GIT_EDITOR", "true");
    // SAFETY: between fork and exec only `setrlimit`, an async-signal-safe call, runs.
    unsafe {
        command.pre_exec(|| { return super::support::LimitKind::CoreFile.apply(0, 0); });
    }
    let mut running = spawn_watched(&fixture, &mut command);
    running.wait_for(started.as_path());
    send(running.id(), signal);
    let finished: Output = running.finish();
    let said: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(128 + signal), "SIG{name}: {said}");
    let received: String = std::fs::read_to_string(&log).expect("the stand-in Git recorded what reached it");
    assert_eq!(received, format!("{name}\n"), "exactly the one signal sent to the wrapper reaches Git");
    let git_pid: i32 = std::fs::read_to_string(&process_id).expect("the stand-in Git's process ID").trim().parse().expect("a process ID");
    assert_gone(git_pid, "the Git the wrapper started");
    let verdict: Result<(), String> = verify_recovery(&fixture, repo.as_path(), label, said.as_str());
    assert_eq!(verdict, Ok(()));
    remove_settled(&fixture);
}

/// A hangup sent to the wrapper alone reaches its Git,
/// and the wrapper ends with status 129.
#[test]
fn a_hangup_sent_to_the_wrapper_alone_is_relayed_to_git() {
    relayed_to_git("HUP", "relay-hup", libc::SIGHUP);
}

/// An interrupt sent to the wrapper alone reaches its Git,
/// and the wrapper ends with status 130.
#[test]
fn an_interrupt_sent_to_the_wrapper_alone_is_relayed_to_git() {
    relayed_to_git("INT", "relay-int", libc::SIGINT);
}

/// A termination signal sent to the wrapper alone reaches its Git,
/// and the wrapper ends with status 143.
#[test]
fn a_termination_sent_to_the_wrapper_alone_is_relayed_to_git() {
    relayed_to_git("TERM", "relay-term", libc::SIGTERM);
}

/// A quit sent to the wrapper alone reaches its Git,
/// and the wrapper ends with status 131.
#[test]
fn a_quit_sent_to_the_wrapper_alone_is_relayed_to_git() {
    relayed_to_git("QUIT", "relay-quit", libc::SIGQUIT);
}

/// A hangup of the whole process group,
/// as a closing terminal sends,
/// while the real Git waits in the editor:
/// the commit ends
///  with status 129,
/// the editor is gone,
/// and nothing is left behind.
#[test]
fn a_hangup_of_the_whole_group_during_the_editor_ends_the_commit_and_leaves_nothing() {
    let fixture: Fixture = fixture("signal-group-hup");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let started: PathBuf = fixture.root.join("editor-started");
    let editor: PathBuf = stalling_editor(&fixture, started.as_path());
    let mut command: Command = unbounded_wrapper(&fixture, repo.as_path(), &["commit", "--quiet", "--", "a.txt"]);
    command.env("GIT_EDITOR", &editor);
    let mut running = spawn_watched(&fixture, &mut command);
    running.wait_for(started.as_path());
    send(-running.id(), libc::SIGHUP);
    let finished: Output = running.finish();
    let said: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(128 + libc::SIGHUP), "{said}");
    assert_eq!(verify_recovery(&fixture, repo.as_path(), "signal-group-hup", said.as_str()), Ok(()));
    remove_settled(&fixture);
}

/// The control that the signal shape is sound:
/// a wrapper whose Git is not signalled ends normally,
/// so a status of `128 + signal`
///  above comes from the signal and not from the stand-in.
#[test]
fn an_unsignalled_commit_through_the_same_shape_lands() {
    let fixture: Fixture = fixture("signal-shape");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let output: Output = run_watched(&fixture, &mut unbounded_wrapper(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]));
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(output.stderr.as_slice()));
    let next = run_commit(&fixture, repo.as_path(), &["status", "--porcelain"]);
    assert_eq!(next.code, Some(0));
    remove_settled(&fixture);
}
