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

/// A scene whose commits the service lands as built by native Git when `native` holds: a commit template in the configuration
///  is something the service does not embed, so Git builds the commit in a scratch repository and the service lands it whole.
fn scene(name: &str, native: bool) -> Scene {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    if native {
        let template: PathBuf = fixture.root.join("template");
        std::fs::write(&template, "").expect("template");
        git(&fixture, &repo, &["config", "commit.template", template.to_str().expect("UTF-8")]);
    }
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
    let scene: Scene = scene("replay-merge", false);
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

/// A commit Git built for the service lands the lines it captured, merged onto the branch, and a write made after the second
///  command started is not in it: the commit is built again from the captured tree, not from the worktree.
#[test]
fn a_commit_git_builds_that_waits_behind_another_lands_its_captured_lines_merged_onto_it() {
    let scene: Scene = scene("replay-native", true);
    let first: [(usize, &str); 1] = [(1, "first edit")];
    let second: [(usize, &str); 1] = [(7, "second edit")];
    let late: [(usize, &str); 2] = [(4, "late edit"), (7, "second edit")];
    let (one, two) = run(&scene, &first, &second, &late);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    assert_eq!(two.code, Some(0), "{}", two.stderr);
    assert_eq!(show(&scene, &format!("HEAD:{FILE}")), lines_with(&[(1, "first edit"), (7, "second edit")]), "both edits landed and the late write did not");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-3"]), "two\none\nbase\n");
    assert_eq!(std::fs::read_to_string(scene.repo.join(FILE)).expect("the file"), lines_with(&late), "the person's file is intact");
    remove_settled(&scene.fixture);
}

/// A second command that captured the file after the first one did, from the same worktree, holds the first command's edit in
///  its bytes: when it rewrites inside that edit the two cannot be merged line by line, and the later capture's bytes land
///  (`doc/decision/cli-git-concurrent-commits.md`, "Serial landing"), as two commits one after the other would record them.
#[test]
fn the_later_capture_of_one_worktree_lands_its_bytes_over_an_overlapping_earlier_one() {
    let scene: Scene = scene("replay-order", false);
    let first: [(usize, &str); 1] = [(1, "first edit")];
    let second: [(usize, &str); 1] = [(1, "second edit")];
    let late: [(usize, &str); 2] = [(1, "second edit"), (4, "late edit")];
    let (one, two) = run(&scene, &first, &second, &late);
    assert_eq!(one.code, Some(0), "{}", one.stderr);
    assert_eq!(two.code, Some(0), "{}", two.stderr);
    assert_eq!(show(&scene, &format!("HEAD:{FILE}")), lines_with(&second), "the bytes the second command captured landed, not the late write");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-3"]), "two\none\nbase\n");
    assert_eq!(std::fs::read_to_string(scene.repo.join(FILE)).expect("the file"), lines_with(&late), "the person's file is intact");
    remove_settled(&scene.fixture);
}

/// Edits of the same lines that no command of this worktree made cannot both land: the second fails with nothing changed, its
///  message saved, and says what and why.
#[test]
fn overlapping_edits_of_a_commit_made_elsewhere_fail_the_second_commit_with_nothing_changed_and_its_message_saved() {
    let scene: Scene = scene("replay-conflict", false);
    let file: PathBuf = scene.repo.join(FILE);
    let second: [(usize, &str); 1] = [(1, "second edit")];
    let late: [(usize, &str); 2] = [(1, "second edit"), (4, "late edit")];
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(scene.markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    std::fs::write(&file, lines_with(&second)).expect("second edit");
    let waiting: Started = start(&scene, "two", true);
    let reached: PathBuf = scene.markers.join("service-prepared-1.reached");
    wait_for("the second commit prepared", || { return reached.exists(); });
    // Another program commits the same line while the second command waits: the service has no capture of it.
    std::fs::write(&file, lines_with(&[(1, "foreign edit")])).expect("foreign edit");
    git(&scene.fixture, &scene.repo, &["commit", "--quiet", "--message=one", "--", FILE]);
    std::fs::write(&file, lines_with(&late)).expect("late edit");
    std::fs::write(scene.markers.join("service-prepared-1.release"), "").expect("release");
    let two: Ended = finish(waiting);
    assert_eq!(two.code, Some(1), "{}", two.stderr);
    let short: String = git_text(&scene.fixture, &scene.repo, &["log", "-1", "--format=%h"]).trim().to_string();
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-2"]), "one\nbase\n", "only the other commit landed");
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
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["diff", "--cached", "--raw"]), "", "the index matches the branch the other commit left");
    assert_eq!(std::fs::read_to_string(&file).expect("the file"), lines_with(&late), "the person's file is intact");
    assert!(!scene.repo.join(".git/index.lock").exists(), "no lock is left");
    remove_settled(&scene.fixture);
}
