//! Controls for the watcher of waits that block on a child process:
//!  the limit from its variable, the notices at the cadence, the child stopped at the bound,
//!  and the one-shot run that captures a command's output under a watch.
#![cfg(unix)]

use super::*;
use crate::test_support::{ActionSink, SharedSink};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;

/// A cadence of a 60 ms grace and a 60 ms interval, so a notice comes fast.
const QUICK: Cadence = Cadence {
    grace: Duration::from_millis(60),
    interval: Duration::from_millis(60),
};

/// Terms of a limit, the quick cadence and a shared sink the test reads.
fn terms(limit: Duration, sink: &SharedSink) -> WatchTerms {
    return WatchTerms {
        limit,
        cadence: QUICK,
        notices: Box::new(sink.clone()),
    };
}

/// The limit follows the variable and falls back to the default for anything unusable.
#[test]
fn the_limit_comes_from_the_variable_or_the_default() {
    assert_eq!(child_wait_limit_from(None), DEFAULT_CHILD_WAIT);
    assert_eq!(DEFAULT_CHILD_WAIT, Duration::from_secs(1800));
    assert_eq!(child_wait_limit_from(Some(OsStr::new("90"))), Duration::from_secs(90));
    for value in ["0", "", "-1", "1.5", "soon"] {
        assert_eq!(child_wait_limit_from(Some(OsStr::new(value))), DEFAULT_CHILD_WAIT, "{value:?}");
    }
}

/// The sentence for a stopped wait names it, the bound, the variable and what to do next.
#[test]
fn the_timeout_sentence_names_the_wait_the_bound_and_the_next_step() {
    assert_eq!(
        watched_timeout_text("git add -- a.md", Duration::from_secs(1800)),
        "git add -- a.md did not finish within 1800 seconds and was stopped (CLI_GIT_CHILD_WAIT_SECONDS changes this). \
         Run the command again; if it stalls again, check with `ps` what that process waits for."
    );
}

/// A sink that tells a channel each time a line is finished, and keeps the text for the test.
struct LineSignal {
    /// The text written so far.
    shared: SharedSink,
    /// Told once per finished line.
    lines: std::sync::mpsc::Sender<()>,
}

/// Writes are copied and each finished line is signalled.
impl Write for LineSignal {
    /// Copy the bytes and signal the lines they finish.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut self.shared, bytes)?;
        for _line in bytes.iter().filter(|byte| { return **byte == b'\n'; }) {
            self.lines.send(()).ok();
        }
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// A wait that crosses the bound is announced, then the child is stopped, which ends the blocked wait.
///  The notice that is due goes out before the bound is checked, so at least one notice is out
///  however slowly the host runs the watcher.
#[test]
fn a_wait_past_the_bound_is_announced_then_the_child_is_stopped() {
    let sink: SharedSink = SharedSink::default();
    let mut child: Child = Command::new("sleep").arg("30").spawn().expect("sleeper");
    let watch: ChildWatch = ChildWatch::start("the sleeper", child.id(), terms(Duration::from_millis(450), &sink));
    watch.begin();
    let status = child.wait().expect("the stopped child is collected");
    watch.end();
    assert!(watch.expired());
    assert_eq!(status.signal(), Some(9), "{status:?}");
    let text: String = sink.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty(), "{text}");
    for line in &lines {
        assert!(line.starts_with("cli-git: still waiting for the sleeper (process "), "{text}");
        assert!(line.contains("(CLI_GIT_CHILD_WAIT_SECONDS changes this)"), "{text}");
    }
    assert_eq!(text.matches("Hint:").count(), 1, "{text}");
}

/// A wait that ends inside the grace prints nothing and does not stop the child; an idle watch prints nothing either.
///  The grace is an hour, so no host is slow enough to make the wait outlast it.
#[test]
fn a_wait_that_ends_inside_the_grace_prints_nothing() {
    let sink: SharedSink = SharedSink::default();
    let mut child: Child = Command::new("sleep").arg("30").spawn().expect("sleeper");
    let watch: ChildWatch = ChildWatch::start(
        "the sleeper",
        child.id(),
        WatchTerms {
            limit: Duration::from_secs(7200),
            cadence: Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            notices: Box::new(sink.clone()),
        },
    );
    std::thread::sleep(Duration::from_millis(50));
    watch.begin();
    std::thread::sleep(Duration::from_millis(50));
    watch.end();
    assert!(!watch.expired());
    assert_eq!(sink.text(), "");
    drop(watch);
    child.kill().expect("kill");
    child.wait().expect("collect");
}

/// Each wait gets its own clock: a second wait on the same child is announced from its own start, with its own hint.
///  Each wait ends on its first notice, so every wait has at least one notice, however slowly the host runs the watcher.
#[test]
fn each_wait_is_announced_from_its_own_start() {
    let shared: SharedSink = SharedSink::default();
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let mut child: Child = Command::new("sleep").arg("30").spawn().expect("sleeper");
    let watch: ChildWatch = ChildWatch::start(
        "the sleeper",
        child.id(),
        WatchTerms {
            limit: Duration::from_secs(3600),
            cadence: QUICK,
            notices: Box::new(LineSignal { shared: shared.clone(), lines: sender }),
        },
    );
    for _wait in 0..2 {
        watch.begin();
        receiver.recv().expect("a notice announces the wait");
        watch.end();
    }
    let text: String = shared.text();
    assert_eq!(text.matches("Hint:").count(), 2, "the hint opens each wait: {text}");
    assert!(!watch.expired());
    drop(watch);
    child.kill().expect("kill");
    child.wait().expect("collect");
}

/// A command that runs long has its output collected untouched while the wait is announced.
///  The command waits for a release file that the second notice creates, so the wait ends on that event and at least
///  two notices are out, however slowly the host runs the watcher.
#[test]
fn a_command_that_runs_long_is_announced_and_its_output_is_kept() {
    let shared: SharedSink = SharedSink::default();
    let root: PathBuf = crate::test_support::fixture("child-watch-long");
    let release: PathBuf = root.join("release");
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg("while [ ! -e \"$1\" ]; do sleep 0.02; done; printf out; printf err >&2")
        .arg("sh")
        .arg(&release)
        .stdin(Stdio::null());
    let release_file: PathBuf = release.clone();
    let counted: ActionSink = ActionSink::new(&shared, 2, move || {
        std::fs::write(&release_file, b"").expect("release the command");
    });
    let output: Output = output_watched(
        &mut command,
        "the slow command",
        WatchTerms { limit: Duration::from_secs(3600), cadence: QUICK, notices: Box::new(counted) },
    )
    .expect("finished");
    assert_eq!((output.stdout, output.stderr), (b"out".to_vec(), b"err".to_vec()));
    assert!(output.status.success());
    let text: String = shared.text();
    assert!(text.lines().count() >= 2, "{text}");
    assert!(text.starts_with("cli-git: still waiting for the slow command (process "), "{text}");
    crate::test_support::remove(root.as_path());
}

/// A command that outlasts the bound is stopped and reported as a timeout that names it, after at least one notice.
#[test]
fn a_command_past_the_bound_is_stopped_and_reported() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sleep");
    command.arg("30").stdin(Stdio::null());
    let started: Instant = Instant::now();
    let failure: std::io::Error = output_watched(&mut command, "git rev-parse HEAD", terms(Duration::from_millis(400), &sink))
        .expect_err("stopped at the bound");
    assert!(started.elapsed() < Duration::from_secs(60), "{:?}", started.elapsed());
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut);
    assert!(failure.to_string().starts_with("git rev-parse HEAD did not finish within 0 seconds and was stopped"), "{failure}");
    assert!(sink.text().lines().count() >= 1, "{}", sink.text());
}

/// A command that finishes at once prints nothing; the grace is an hour, so no host is slow enough to outlast it.
#[test]
fn a_command_that_finishes_at_once_prints_nothing() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("true");
    command.stdin(Stdio::null());
    let output: Output = output_watched(
        &mut command,
        "true",
        WatchTerms {
            limit: Duration::from_secs(7200),
            cadence: Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            notices: Box::new(sink.clone()),
        },
    )
    .expect("finished");
    assert!(output.status.success());
    assert_eq!(sink.text(), "");
}
