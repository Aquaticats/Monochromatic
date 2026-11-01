//! What:
//!  Two wrapped commits of one file in one worktree, the second landing behind the first, through the real wrapper and the
//!  real service.
//! Why:
//!  A commit that waits behind another commit of the same file must land the bytes it captured when the person ran it,
//!  merged onto the branch the first commit left, never the bytes the file holds by the time the wait ends
//!  (`doc/decision/cli-git-concurrent-commits.md`, "Serial landing"). The first commit is held at the index lock, which
//!  the control owns; the second is paused at a named phase marker after it has captured the file, so the control can
//!  write to the file at the one instant the wrong behaviour needs; both are then released. Nothing is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdIndexLock(); const first = commit('one'); const second = commitPausedAfterCapture('two'); writeLate(); release();
//! expect(await second).toLandItsCapturedLinesOntoTheFirst();
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// The longest a control waits for anything, far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// The file both commits name.
const FILE: &str = "f.txt";

/// The number of lines the file has.
const LINES: usize = 9;

/// How many arrivals at the marker the control releases in advance, after the first it holds: the bound on landing attempts.
const LATER_ARRIVALS: usize = 4;

/// A repository with one tracked file, a running service and a directory for the paused wrapper's markers.
struct Scene {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    markers: PathBuf,
}

/// A wrapped commit that was started and is not over.
struct Started {
    child: Child,
    /// Everything the commit wrote to its standard error, once it has ended.
    stderr: std::thread::JoinHandle<String>,
    /// Fires when the commit printed that it waits for the index.
    waiting: Receiver<()>,
}

/// What a finished wrapped commit came to.
struct Ended {
    code: Option<i32>,
    stderr: String,
}

/// The file as committed first, and that text with the lines named replaced by text that says which edit wrote it.
fn lines_with(edits: &[(usize, &str)]) -> String {
    let mut text: String = String::new();
    for number in 1..=LINES {
        let line: String = edits.iter().find(|(at, _)| { return *at == number; }).map_or_else(|| { return format!("line {number}"); }, |(_, edit)| { return (*edit).to_string(); });
        text.push_str(&line);
        text.push('\n');
    }
    return text;
}

fn scene(name: &str) -> Scene {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(FILE), lines_with(&[])).expect("the file");
    git(&fixture, &repo, &["add", "--", FILE]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("markers");
    let service: FixtureService = start_fixture_service(&fixture);
    return Scene { fixture, repo, service, markers };
}

fn start(scene: &Scene, message: &str, paused: bool) -> Started {
    let mut command: Command = wrapped_commit(&scene.fixture, &scene.repo, &["commit", "--quiet", "-m", message, "--", FILE]);
    command.env("XDG_RUNTIME_DIR", &scene.service.runtime).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    if paused {
        command.env(PHASE_SIGNAL_VARIABLE, format!("service-prepared:pause:{}", scene.markers.display()));
    }
    let mut child: Child = command.spawn().expect("the wrapped commit starts");
    let stderr = child.stderr.take().expect("piped standard error");
    let (sender, waiting) = channel::<()>();
    let reader = std::thread::spawn(move || {
        let mut all: String = String::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if line.contains("waiting for") {
                let _told = sender.send(());
            }
            all.push_str(&line);
            all.push('\n');
        }
        return all;
    });
    return Started { child, stderr: reader, waiting };
}

fn finish(mut started: Started) -> Ended {
    let status = started.child.wait().expect("the commit ends");
    return Ended { code: status.code(), stderr: started.stderr.join().expect("the reader") };
}

fn wait_for(what: &str, present: impl Fn() -> bool) {
    let began: Instant = Instant::now();
    while !present() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Runs the scenario: the first commit holds at the index lock with `first` in the file, the second captures `second`,
///  `late` is written after the second captured, and then the lock is let go.
fn run(scene: &Scene, first: &[(usize, &str)], second: &[(usize, &str)], late: &[(usize, &str)]) -> (Ended, Ended) {
    let file: PathBuf = scene.repo.join(FILE);
    let lock: PathBuf = scene.repo.join(".git/index.lock");
    // Only the first arrival at the marker is held for the control; a commit sent back reaches it again, and goes on.
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(scene.markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    // The lock is the control's, so the service waits for it exactly as it waits for a foreign Git.
    std::fs::write(&lock, "").expect("the index lock");
    std::fs::write(&file, lines_with(first)).expect("first edit");
    let one: Started = start(scene, "one", false);
    one.waiting.recv_timeout(PATIENCE).expect("the first commit waits for the index");
    std::fs::write(&file, lines_with(second)).expect("second edit");
    let two: Started = start(scene, "two", true);
    let reached: PathBuf = scene.markers.join("service-prepared-1.reached");
    wait_for("the second commit prepared", || { return reached.exists(); });
    // The second commit has captured its file. What is written now was not part of the command.
    std::fs::write(&file, lines_with(late)).expect("late edit");
    std::fs::write(scene.markers.join("service-prepared-1.release"), "").expect("release");
    std::fs::remove_file(&lock).expect("the lock is let go");
    let first_end: Ended = finish(one);
    let second_end: Ended = finish(two);
    return (first_end, second_end);
}

fn show(scene: &Scene, revision: &str) -> String {
    return git_text(&scene.fixture, &scene.repo, &["show", revision]);
}

/// Edits of different regions both land, and what is written after the second command started is not in its commit.
#[test]
fn a_commit_that_waits_behind_another_lands_its_captured_lines_merged_onto_it() {
    let scene: Scene = scene("replay-merge");
    let first: [(usize, &str); 1] = [(1, "first edit")];
    let second: [(usize, &str); 1] = [(7, "second edit")];
    let late: [(usize, &str); 2] = [(4, "late edit"), (7, "second edit")];
    let (one, two) = run(&scene, &first, &second, &late);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    assert_eq!(two.code, Some(0), "{}", two.stderr);
    let landed: String = show(&scene, &format!("HEAD:{FILE}"));
    assert_eq!(landed, lines_with(&[(1, "first edit"), (7, "second edit")]), "both edits landed and the late write did not");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-3"]), "two\none\nbase\n");
    let second_change: String = git_text(&scene.fixture, &scene.repo, &["diff", "--numstat", "HEAD~1", "HEAD"]);
    assert_eq!(second_change, format!("1\t1\t{FILE}\n"), "the second commit changes its own line only");
    assert_eq!(std::fs::read_to_string(scene.repo.join(FILE)).expect("the file"), lines_with(&late), "the person's file is intact");
    remove_settled(&scene.fixture);
}

/// Edits of the same lines cannot both land: the second fails with nothing changed, its message saved, and says what and why.
#[test]
fn overlapping_edits_fail_the_second_commit_with_nothing_changed_and_its_message_saved() {
    let scene: Scene = scene("replay-conflict");
    let first: [(usize, &str); 1] = [(1, "first edit")];
    let second: [(usize, &str); 1] = [(1, "second edit")];
    let late: [(usize, &str); 2] = [(1, "second edit"), (4, "late edit")];
    let (one, two) = run(&scene, &first, &second, &late);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    assert_eq!(two.code, Some(1), "{}", two.stderr);
    let short: String = git_text(&scene.fixture, &scene.repo, &["log", "-1", "--format=%h"]).trim().to_string();
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-2"]), "one\nbase\n", "only the first commit landed");
    let text: String = two.stderr.lines().filter(|line| { return !line.starts_with("[warn] "); }).collect::<Vec<&str>>().join("\n");
    assert!(text.contains(FILE), "the file is named: {text}");
    assert!(text.contains(short.as_str()) && text.contains("one"), "the commit that landed first is named: {text}");
    let saved: Vec<PathBuf> = std::fs::read_dir(scene.repo.join(".git"))
        .expect("the git directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("cli-git-saved-message-"); }); })
        .collect();
    assert_eq!(saved.len(), 1, "the message is saved once: {saved:?}");
    assert!(text.contains(saved[0].to_string_lossy().as_ref()), "the saved message is named: {text}");
    assert_eq!(std::fs::read_to_string(&saved[0]).expect("the saved message").trim_end(), "two");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["diff", "--cached", "--raw"]), "", "the index matches the branch the first commit left");
    assert_eq!(std::fs::read_to_string(scene.repo.join(FILE)).expect("the file"), lines_with(&late), "the person's file is intact");
    assert!(!scene.repo.join(".git/index.lock").exists(), "no lock is left");
    remove_settled(&scene.fixture);
}
