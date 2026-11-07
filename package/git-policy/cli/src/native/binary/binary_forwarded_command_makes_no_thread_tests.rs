//! What:
//!  Controls that a forwarded Git command makes no `clone`, `clone3`, `fork` or `vfork` call of its own before real Git
//!  replaces the wrapper.
//! Why:
//!  The wrapper is moving onto tokio (`doc/planning/git-policy-cli-full-tokio.md`, step T2). The owner's rule is that a
//!  forwarded command pays for nothing it does not use: a runtime, and above all its worker threads, is built only by
//!  code that needs it, never in `main`. A snapshot of `/proc/<pid>/status` cannot show that a thread was never
//!  started, so these controls read the system calls of the whole run under `strace`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const trace = runUnderStrace(['git', 'rev-parse', 'HEAD']);
//! // expect(trace.clonesMadeByTheWrapper).toBe(0);
//! ```

/// The trace reader and the counts it returns.
use super::budget_trace::{Counts, TRACED_CALLS, count};
/// Finding `strace` on the test process's own `PATH`.
use super::strace::{strace_available, strace_program};
/// The fixtures and the wrapped run of every binary-level control.
use super::support::{Fixture, REAL_GIT, fixture, remove, repository, wrapped};
/// Paths of the fixture's files.
use std::path::PathBuf;
/// Building the traced command and reading what it printed.
use std::process::{Command, Output};

/// What:
///  `fn traced_wrapper_spawns(...)` runs one wrapped command under `strace` and returns how many `clone`-family calls
///  the wrapper's own program made, plus what the command printed.
/// Why:
///  Every control below needs the same run; one function keeps the trace options in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tracedWrapperSpawns(fixture: Fixture, repo: string, args: string[]): { spawns: number; output: Output } { ... }
/// ```
fn traced_wrapper_spawns(label: &str, arguments: &[&str]) -> (usize, Output) {
    let fixture: Fixture = fixture(format!("no-thread-{label}").as_str());
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    let trace: PathBuf = fixture.root.join("trace.txt");
    let inner: Command = {
        let mut command: Command = wrapped(&fixture);
        command.current_dir(repo.as_path()).args(arguments);
        command
    };
    let mut outer: Command = Command::new(strace_program().expect("strace is installed"));
    outer
        .arg("--follow-forks")
        .arg("--quiet")
        .arg("--quiet")
        .arg("--string-limit=4096")
        .arg("--no-abbrev")
        .arg(format!("--output={}", trace.display()))
        .arg(format!("--trace={TRACED_CALLS}"))
        .arg(inner.get_program())
        .args(inner.get_args());
    outer.env_clear();
    for (name, setting) in inner.get_envs() {
        if let Some(text) = setting {
            outer.env(name, text);
        }
    }
    if let Some(directory) = inner.get_current_dir() {
        outer.current_dir(directory);
    }
    let output: Output = outer.output().expect("strace starts");
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts: Counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    remove(&fixture);
    return (counts.wrapper_spawns, output);
}

/// What:
///  `rev-parse HEAD` and `status --short` are forwarded commands: the wrapper plans them and replaces itself with Git.
/// Why:
///  Neither needs a runtime, a thread or a child, so the wrapper process makes no `clone`-family call at all.
#[test]
fn a_forwarded_command_makes_no_clone_before_git_replaces_the_wrapper() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no system call was counted");
        return;
    }
    for (label, arguments) in [("rev-parse", ["rev-parse", "HEAD"].as_slice()), ("status", ["status", "--short"].as_slice())] {
        let (spawns, output) = traced_wrapper_spawns(label, arguments);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{label}: the forwarded command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(spawns, 0, "{label}: the wrapper made {spawns} clone-family calls before Git replaced it");
    }
}

/// The planted control: a trace in which the wrapper starts a thread and a child, and Git starts one too.
const TRACE_WITH_WRAPPER_THREADS: &str = "\
100 execve(\"/usr/bin/timeout\", [\"timeout\"], 0x0) = 0
100 clone3({flags=CLONE_VM}, 88) = 101
101 execve(\"/fixture/bin/git\", [\"git\", \"rev-parse\"], 0x0) = 0
101 clone3({flags=CLONE_VM|CLONE_THREAD}, 88) = 102
101 clone3({flags=CLONE_VM|CLONE_VFORK}, 88) = 103
103 execve(\"/usr/bin/git\", [\"git\", \"rev-parse\"], 0x0) = 0
103 clone3({flags=CLONE_VM|CLONE_THREAD}, 88) = 104
";

/// What:
///  The counter attributes a spawn to the program of the process that made it: two by the wrapper, none by the
///  harness's `timeout` and none by Git.
/// Why:
///  A control that cannot count a wrapper thread would pass whatever the wrapper does (the guard must be shown to fail).
#[test]
fn the_count_sees_threads_and_children_made_by_the_wrapper_and_only_those() {
    let counts: Counts = count(TRACE_WITH_WRAPPER_THREADS, "/fixture", "/usr/bin/git");
    assert_eq!(counts.wrapper_spawns, 2);
}
