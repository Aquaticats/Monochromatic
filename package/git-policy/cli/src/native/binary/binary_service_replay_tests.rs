//! What:
//!  Two wrapped commits of one file in one worktree,
//! the second landing behind the first,
//! through the real wrapper and the
//!  real service.
//! Why:
//!  A commit that waits behind another commit of the same file must land the bytes it captured when the person ran it,
//!  merged onto the branch the first commit left,
//! never the bytes the file holds by the time the wait ends
//!  (`doc/decision/cli-git-concurrent-commits.md`,
//! "Serial landing").
//! The first commit is held at the index lock,
//! which
//!  the control owns;
//! the second is paused at a named phase marker after it has captured the file,
//! so the control can
//!  write to the file at the one instant the wrong behaviour needs;
//! both are then released.
//! Nothing is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdIndexLock(); const first = commit('one'); const second = commitPausedAfterCapture('two'); writeLate(); release();
//! expect(await second).toLandItsCapturedLinesOntoTheFirst();
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::service_fault::QUEUED_DIRECTORY_VARIABLE;
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// The file both commits name.
const FILE: &str = "f.txt";

/// A second file,
/// which a commit that holds the service busy changes.
const OTHER: &str = "g.txt";

/// The number of lines the file has.
const LINES: usize = 9;

/// How many arrivals at the marker the control releases in advance,
/// after the first it holds:
/// the bound on landing attempts.
const LATER_ARRIVALS: usize = 4;

/// A repository with one tracked file,
/// a running service and a directory for the paused wrapper's markers.
struct Scene {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    markers: PathBuf,
    /// Where the service leaves a file for each commit request it has queued.
    queued: PathBuf,
}

/// A wrapped commit that was started and is not over.
struct Started {
    child: Child,
    /// Everything the commit wrote to its standard error,
    /// once it has ended.
    stderr: std::thread::JoinHandle<String>,
    /// Fires when the commit printed that it waits for the index.
    waiting: Receiver<()>,
}

/// What a finished wrapped commit came to.
struct Ended {
    code: Option<i32>,
    stderr: String,
}

/// The file as committed first,
/// and that text with the lines named replaced by text that says which edit wrote it.
fn lines_with(edits: &[(usize, &str)]) -> String {
    let mut text: String = String::new();
    for number in 1..=LINES {
        let line: String = edits.iter().find(|(at, _)| { return *at == number; }).map_or_else(|| { return format!("line {number}"); }, |(_, edit)| { return (*edit).to_string(); });
        text.push_str(&line);
        text.push('\n');
    }
    return text;
}

/// A scene whose commits the service lands as built by native Git when `native` holds:
/// a commit template in the configuration
///  is something the service does not embed,
/// so Git builds the commit in a scratch repository and the service lands it whole.
fn scene(name: &str, native: bool) -> Scene {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    if native {
        let template: PathBuf = fixture.root.join("template");
        std::fs::write(&template, "").expect("template");
        git(&fixture, &repo, &["config", "commit.template", template.to_str().expect("UTF-8")]);
    }
    std::fs::write(repo.join(FILE), lines_with(&[])).expect("the file");
    std::fs::write(repo.join(OTHER), "other\n").expect("the other file");
    git(&fixture, &repo, &["add", "--", FILE, OTHER]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("markers");
    let queued: PathBuf = fixture.root.join("queued");
    std::fs::create_dir(&queued).expect("queued");
    super::support::give_service_environment(&fixture, QUEUED_DIRECTORY_VARIABLE, queued.to_str().expect("UTF-8"));
    let service: FixtureService = start_fixture_service(&fixture);
    // One service per worktree, as in production: the manager would start it on the first connection.
    service.serve(&fixture, repo.join(".git").as_path());
    return Scene { fixture, repo, service, markers, queued };
}

fn start(scene: &Scene, message: &str, paused: bool) -> Started {
    return start_on(scene, message, FILE, paused.then_some(scene.markers.as_path()));
}

/// Starts a wrapped commit of `path`,
/// paused after it captured when `markers` names the directory of its markers.
fn start_on(scene: &Scene, message: &str, path: &str, markers: Option<&Path>) -> Started {
    return start_with_phase(scene, message, path, "service-prepared", markers);
}

/// Starts a wrapped commit of `path`,
/// paused at `phase` when `markers` names the directory of its markers.
fn start_with_phase(scene: &Scene, message: &str, path: &str, phase: &str, markers: Option<&Path>) -> Started {
    let mut command: Command = wrapped_commit(&scene.fixture, &scene.repo, &["commit", "--quiet", "-m", message, "--", path]);
    command.env("XDG_RUNTIME_DIR", &scene.service.runtime).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    if let Some(directory) = markers {
        command.env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", directory.display()));
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

/// Runs the scenario:
/// the first commit holds at the index lock with `first` in the file,
/// the second captures `second`,
///  `late` is written after the second captured,
/// and then the lock is let go.
fn run(scene: &Scene, first: &[(usize, &str)], second: &[(usize, &str)], late: &[(usize, &str)]) -> (Ended, Ended) {
    let file: PathBuf = scene.repo.join(FILE);
    let lock: PathBuf = scene.repo.join(".git/index.lock");
    // Only the first arrival at the marker is held for the control; a commit sent back reaches it again, and goes on.
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(scene.markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    // The lock is the control's, so the service waits for it exactly as it waits for a foreign Git.
    std::fs::write(&lock, "a foreign program's lock\n").expect("the index lock");
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

/// Edits of different regions both land,
/// and what is written after the second command started is not in its commit.
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

/// A commit Git built for the service lands the lines it captured,
/// merged onto the branch,
/// and a write made after the second
///  command started is not in it:
/// the commit is built again from the captured tree,
/// not from the worktree.
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

/// A second command that captured the file after the first one did,
/// from the same worktree,
/// holds the first command's edit in
///  its bytes:
/// when it rewrites inside that edit the two cannot be merged line by line,
/// and the later capture's bytes land
///  (`doc/decision/cli-git-concurrent-commits.md`,
/// "Serial landing"),
/// as two commits one after the other would record them.
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

/// Edits of the same lines that no command of this worktree made cannot both land:
/// the second fails with nothing changed,
/// its
///  message saved,
/// and says what and why.
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
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("monochromatic-git-policy-cli-saved-message-"); }); })
        .collect();
    assert_eq!(saved.len(), 1, "the message is saved once: {saved:?}");
    assert!(text.contains(saved[0].to_string_lossy().as_ref()), "the saved message is named: {text}");
    assert_eq!(std::fs::read_to_string(&saved[0]).expect("the saved message").trim_end(), "two");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["diff", "--cached", "--raw"]), "", "the index matches the branch the other commit left");
    assert_eq!(std::fs::read_to_string(&file).expect("the file"), lines_with(&late), "the person's file is intact");
    assert!(!scene.repo.join(".git/index.lock").exists(), "no lock is left");
    remove_settled(&scene.fixture);
}

/// A commit of a name that is not UTF-8 that another commit overtakes while it is prepared is prepared again and lands with the
///  exact bytes of its name and its captured content,
/// behind the commit that landed first.
#[test]
fn a_commit_of_a_name_that_is_not_utf8_is_prepared_again_and_lands_behind_another() {
    use std::os::unix::ffi::OsStringExt;
    let scene: Scene = scene("replay-non-utf8", false);
    let name: OsString = OsString::from_vec(b"caf\xe9.txt".to_vec());
    let named: PathBuf = scene.repo.join(&name);
    std::fs::write(&named, b"one\n").expect("the base file");
    git(&scene.fixture, &scene.repo, &[OsStr::new("add"), OsStr::new("--"), name.as_os_str()]);
    git(&scene.fixture, &scene.repo, &["commit", "--quiet", "--message=base two"]);
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(scene.markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    std::fs::write(&named, b"one\ntwo\n").expect("the edit");
    let mut slow: Command = wrapped_commit(&scene.fixture, &scene.repo, &["commit", "--quiet", "-m", "slow", "--"]);
    slow.arg(&name)
        .env("XDG_RUNTIME_DIR", &scene.service.runtime)
        .env(PHASE_SIGNAL_VARIABLE, format!("service-prepared:pause:{}", scene.markers.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let child: Child = slow.spawn().expect("the slow commit starts");
    let reached: PathBuf = scene.markers.join("service-prepared-1.reached");
    wait_for("the slow commit prepared", || { return reached.exists(); });
    // Another commit lands on the branch while the slow one waits, so its preparation is out of date.
    std::fs::write(scene.repo.join(OTHER), "other edit\n").expect("the other edit");
    let mut winner: Command = wrapped_commit(&scene.fixture, &scene.repo, &["commit", "--quiet", "-m", "winner", "--", OTHER]);
    winner.env("XDG_RUNTIME_DIR", &scene.service.runtime);
    let landed = winner.output().expect("the winner runs");
    assert!(landed.status.success(), "{}", String::from_utf8_lossy(&landed.stderr));
    std::fs::write(scene.markers.join("service-prepared-1.release"), "").expect("release");
    let ended = child.wait_with_output().expect("the slow commit ends");
    let stderr: String = String::from_utf8_lossy(&ended.stderr).into_owned();
    assert_eq!(ended.status.code(), Some(0), "{stderr}");
    let listed = git(&scene.fixture, &scene.repo, &["ls-tree", "-z", "--name-only", "HEAD"]);
    assert!(listed.stdout.split(|byte| { return *byte == 0; }).any(|entry| { return entry == name.as_encoded_bytes(); }), "the exact name is in the tree");
    let blob = git(&scene.fixture, &scene.repo, &[OsString::from("show"), {
        let mut spec: OsString = OsString::from("HEAD:");
        spec.push(&name);
        spec
    }]);
    assert_eq!(blob.stdout, b"one\ntwo\n", "the captured content landed");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-3"]), "slow\nwinner\nbase two\n");
    remove_settled(&scene.fixture);
}

/// Whether a process started with `marker` in its environment holds a socket,
/// which a wrapper does only once it has connected
///  to the service and is sending its request.
fn holds_socket(marker: &Path) -> bool {
    let needle: Vec<u8> = marker.to_string_lossy().as_bytes().to_vec();
    let Ok(processes) = std::fs::read_dir("/proc") else { return false };
    return processes.flatten().any(|process| {
        let Ok(environment) = std::fs::read(process.path().join("environ")) else { return false };
        if !environment.windows(needle.len()).any(|window| { return window == needle.as_slice(); }) {
            return false;
        }
        let Ok(descriptors) = std::fs::read_dir(process.path().join("fd")) else { return false };
        return descriptors.flatten().any(|descriptor| { return std::fs::read_link(descriptor.path()).is_ok_and(|target| { return target.to_string_lossy().starts_with("socket:"); }); });
    });
}

/// How many commit requests the service has put in its queue.
fn queued_requests(scene: &Scene) -> usize {
    return std::fs::read_dir(&scene.queued).map_or(0, Iterator::count);
}

/// Three commits with the service held busy by the first:
/// the other two reach it while it waits,
/// so they land as one batch on
///  one base.
/// The earlier one is captured first and the later one after it,
/// from one worktree,
/// so the later one's bytes hold
///  the earlier one's edit;
/// `later_arrives_first` says which of the two reaches the service first.
fn run_one_batch(scene: &Scene, later_arrives_first: bool) -> (Ended, Ended, Ended) {
    let file: PathBuf = scene.repo.join(FILE);
    let lock: PathBuf = scene.repo.join(".git/index.lock");
    let earlier_markers: PathBuf = scene.markers.join("earlier");
    let later_markers: PathBuf = scene.markers.join("later");
    for directory in [&earlier_markers, &later_markers] {
        std::fs::create_dir(directory).expect("markers of one commit");
        for number in 2..=LATER_ARRIVALS {
            std::fs::write(directory.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
        }
    }
    std::fs::write(&lock, "a foreign program's lock\n").expect("the index lock");
    std::fs::write(scene.repo.join(OTHER), "other edit\n").expect("edit of the other file");
    let holder: Started = start_on(scene, "holder", OTHER, None);
    holder.waiting.recv_timeout(PATIENCE).expect("the first commit waits for the index");
    std::fs::write(&file, lines_with(&[(1, "earlier edit")])).expect("earlier edit");
    let earlier: Started = start_on(scene, "earlier", FILE, Some(earlier_markers.as_path()));
    wait_for("the earlier commit captured", || { return earlier_markers.join("service-prepared-1.reached").exists(); });
    std::fs::write(&file, lines_with(&[(1, "earlier edit"), (5, "later edit")])).expect("later edit");
    let later: Started = start_on(scene, "later", FILE, Some(later_markers.as_path()));
    wait_for("the later commit captured", || { return later_markers.join("service-prepared-1.reached").exists(); });
    let order: [&Path; 2] = if later_arrives_first { [later_markers.as_path(), earlier_markers.as_path()] } else { [earlier_markers.as_path(), later_markers.as_path()] };
    // The service has the holder's request already; each release is followed by the service queueing one more.
    let mut queued_before: usize = 1;
    wait_for("the holder's request queued", || { return queued_requests(scene) >= queued_before; });
    for directory in order {
        std::fs::write(directory.join("service-prepared-1.release"), "").expect("release");
        wait_for("the commit connected to the service", || { return holds_socket(directory); });
        // The service leaves a file when it has put the request in its queue: that, not a pause, orders the arrivals.
        queued_before += 1;
        wait_for("the service queued the request", || { return queued_requests(scene) >= queued_before; });
    }
    std::fs::remove_file(&lock).expect("the lock is let go");
    return (finish(holder), finish(earlier), finish(later));
}

/// The commit captured later reaches the service first and the one captured earlier second,
/// in one batch:
/// the later capture
///  holds the earlier edit and more,
/// so its edit must not be replaced by the earlier capture's bytes.
/// The earlier one has
///  nothing left to commit,
/// as native Git says for a path whose bytes an earlier commit took.
#[test]
fn an_earlier_capture_that_arrives_second_in_one_batch_does_not_replace_the_later_one() {
    let scene: Scene = scene("batch-inverted", false);
    let (holder, earlier, later) = run_one_batch(&scene, true);
    assert_eq!(holder.code, Some(0), "{}", holder.stderr);
    assert_eq!(later.code, Some(0), "{}", later.stderr);
    assert_eq!(show(&scene, &format!("HEAD:{FILE}")), lines_with(&[(1, "earlier edit"), (5, "later edit")]), "the later edit is on the branch");
    assert_eq!(earlier.code, Some(1), "{}", earlier.stderr);
    assert!(earlier.stderr.contains("concurrent-commit/already-committed"), "{}", earlier.stderr);
    remove_settled(&scene.fixture);
}

/// The natural order,
/// the earlier capture first,
/// lands both commits in one batch,
/// one after the other,
/// as they were run.
#[test]
fn captures_that_arrive_in_order_in_one_batch_both_land() {
    let scene: Scene = scene("batch-in-order", false);
    let (holder, earlier, later) = run_one_batch(&scene, false);
    for ended in [&holder, &earlier, &later] {
        assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    }
    assert_eq!(show(&scene, &format!("HEAD:{FILE}")), lines_with(&[(1, "earlier edit"), (5, "later edit")]));
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-4"]), "later\nearlier\nholder\nbase\n");
    remove_settled(&scene.fixture);
}

/// The scratch trees of the repository the scene's commits make:
/// one directory per attempt,
/// named after the request and the attempt,
/// inside the repository scratch directory.
fn scratch_trees(scene: &Scene) -> Vec<PathBuf> {
    let directory: PathBuf = scene.repo.join(".git").join("monochromatic-git-policy-cli-scratch");
    let Ok(entries) = std::fs::read_dir(&directory) else { return Vec::new() };
    let mut trees: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("monochromatic-git-policy-cli-service-") {
            trees.push(entry.path());
        }
    }
    trees.sort();
    return trees;
}

/// The scratch tree that appeared since `before`:
/// the one the commit that was started after that look made.
fn the_tree_that_appeared(scene: &Scene, before: &[PathBuf]) -> PathBuf {
    let mut appeared: Vec<PathBuf> = Vec::new();
    for tree in scratch_trees(scene) {
        if !before.contains(&tree) {
            appeared.push(tree);
        }
    }
    assert_eq!(appeared.len(), 1, "one attempt made one tree: {appeared:?}");
    return appeared.remove(0);
}

/// A commit Git built that the service answers as not landed is sent back to capture its change:
/// the private index of the attempt it captured is still there for that read,
/// because the service leaves a tree it answered to the command that made it,
/// and the commit then lands on the branch as it moved.
/// Why:
///  The service removes the tree of an answered request beside its landing loop,
/// and the command reads the private index of that tree after the answer;
/// a removal in that window made the read fail with "No such file or directory",
/// or,
/// when only the index file was gone,
/// rebuilt the commit from the empty tree and reported a conflict naming files the commit never changed
/// (reproduction `r02-index-commit-fails-when-foreign-commit-moves-branch.ts`,
/// 2026-11-09).
#[test]
fn a_commit_the_service_sent_back_still_has_the_private_index_of_the_attempt_it_captured() {
    let scene: Scene = scene("replay-sent-back", true);
    let file: PathBuf = scene.repo.join(FILE);
    // Only the first arrival at the marker is held for the control; a commit sent back reaches it again, and goes on.
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(scene.markers.join(format!("service-sent-back-{number}.release")), "").expect("release of a later arrival");
    }
    std::fs::write(scene.repo.join(OTHER), "the other edit\n").expect("the other edit");
    std::fs::write(&file, lines_with(&[(1, "the edit")])).expect("the edit");
    // The holder keeps the service busy on the index, so the commit under control is prepared on the tip before the holder
    //  lands and reaches the service behind it.
    let lock: PathBuf = scene.repo.join(".git/index.lock");
    std::fs::write(&lock, "a foreign program's lock\n").expect("the index lock");
    let holder: Started = start_on(&scene, "holder", OTHER, None);
    holder.waiting.recv_timeout(PATIENCE).expect("the holder waits for the index");
    let before: Vec<PathBuf> = scratch_trees(&scene);
    let sent_back: Started = start_with_phase(&scene, "sent back", FILE, "service-sent-back", Some(scene.markers.as_path()));
    // The commit under control is prepared and its request is queued while the branch is still where the holder found it.
    wait_for("the service queued the commit under control", || { return queued_requests(&scene) >= 2; });
    std::fs::remove_file(&lock).expect("the lock is let go");
    let reached: PathBuf = scene.markers.join("service-sent-back-1.reached");
    wait_for("the service sent the commit back", || { return reached.exists(); });
    let tree: PathBuf = the_tree_that_appeared(&scene, before.as_slice());
    let index: PathBuf = tree.join("commit.index");
    // The service removes the trees it answered beside its landing loop; a removal of this one would show up here.
    let began: Instant = Instant::now();
    while index.exists() && began.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(index.exists(), "the private index of the attempt the service sent back was removed: {}", index.display());
    std::fs::write(scene.markers.join("service-sent-back-1.release"), "").expect("release");
    let ended: Ended = finish(sent_back);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(show(&scene, &format!("HEAD:{FILE}")), lines_with(&[(1, "the edit")]), "the captured bytes landed on the moved branch");
    assert_eq!(git_text(&scene.fixture, &scene.repo, &["log", "--format=%s", "-3"]), "sent back\nholder\nbase\n");
    assert_eq!(std::fs::read_to_string(scene.repo.join(FILE)).expect("the file"), lines_with(&[(1, "the edit")]), "the person's file is intact");
    assert_eq!(finish(holder).code, Some(0), "the holder landed too");
    remove_settled(&scene.fixture);
}
