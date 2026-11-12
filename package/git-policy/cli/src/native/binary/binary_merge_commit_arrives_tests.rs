//! What:
//!  A commit that arrives while an isolated merge is part-way installed,
//!  through the worktree's commit service.
//! Why:
//!  `F12` of the alpha.12 audit is a second command arriving in a merge's install,
//!  and a commit is the one arriving command that moves the branch the way the merge does:
//! it takes a snapshot of the index and the worktree and lands it.
//! Held at one phase of the install,
//!  a commit must finish soundly beside the merge or step aside without harm,
//!  and the worktree must end in one of the two whole states.
//! The audit measured a merge and a `git status` only,
//! and the first run of this branch could not measure a commit at all,
//!  because its fixtures could not start the commit service.
//! The fixture here starts the service the way production's manager does,
//!  so the arriving command is the real one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdTheMergeAt(fixture, repo, 'merge-cleared'); const commit = wrapped(['commit', '--all', '--message=second']);
//! expect(held.release()).toBe(0); expect(whole()).toBe(true); expect(records()).toBe(0);
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_isolation::records_in;
use super::support::{
    Fixture, FixtureService, Observed, fixture, git, observe, stderr_of, start_fixture_service,
};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits for a paused merge to reach its marker.
const MARKER_PATIENCE: Duration = Duration::from_secs(120);

/// The commit that arrives in the person's worktree:
/// their staged edit,
/// named by its path,
/// which is the form the wrapper's own policy asks for
/// (`git commit` rejects `-a/--all` and a commit with no path).
const STAGED_PATH_COMMIT: &[&str] = &["commit", "--message=second", "--", "b.txt"];

/// The commit that arrives where the merge writes no path the person has work in:
/// the other session's staged file,
/// named by its path.
const UNRELATED_PATH_COMMIT: &[&str] = &["commit", "--message=second", "--", "unrelated.txt"];

/// The shape both findings were measured in:
/// `feature` changes `a.txt` and `b.txt` and adds `c.txt` and `d.txt`,
/// the last two force-added although `d.txt` is ignored,
///  and the person's own work stands at all four paths.
fn world(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = the_repo(fixture, name);
    persons_work(fixture, repo.as_path());
    return repo;
}

/// The repository the controls start from:
/// `feature` changes `a.txt` and `b.txt` and adds `c.txt` and `d.txt`,
/// the last two force-added although `d.txt` is ignored,
///  and `main` holds a commit the merge does not touch.
fn the_repo(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&repo).expect("repository directory");
    git(fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), b"l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"b1\nb2\nb3\nb4\nb5\nb6\nb7\nb8\n").expect("file");
    std::fs::write(repo.join(".gitignore"), b"d.txt\n").expect("file");
    std::fs::write(repo.join("other.txt"), b"o\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), b"l1\nl2\nl3\nl4\nl5\nl6\nl7\nFEATURE-A\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"b1\nb2\nb3\nb4\nb5\nb6\nb7\nFEATURE-B\n").expect("file");
    std::fs::write(repo.join("c.txt"), b"feature c\n").expect("file");
    std::fs::write(repo.join("d.txt"), b"feature d\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--force", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("other.txt"), b"o2\n").expect("file");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    return repo;
}

/// The person's own work at every path the merge writes:
/// one unstaged edit,
/// one staged edit,
/// one untracked file and one ignored file.
fn persons_work(fixture: &Fixture, repo: &Path) {
    std::fs::write(repo.join("a.txt"), b"MARK-A-UNSTAGED\nl2\nl3\nl4\nl5\nl6\nl7\nl8\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"MARK-B-STAGED\nb2\nb3\nb4\nb5\nb6\nb7\nb8\n").expect("file");
    git(fixture, repo, &["add", "--", "b.txt"]);
    std::fs::write(repo.join("c.txt"), b"MARK-C-UNTRACKED\n").expect("file");
    std::fs::write(repo.join("d.txt"), b"MARK-D-IGNORED\n").expect("file");
}

/// The same repository with the person's own work staged at a path the merge does not write,
/// which is a commit the service lands while the install runs,
///  and with nothing of theirs at the paths the merge writes.
fn clean_world(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = the_repo(fixture, name);
    std::fs::write(repo.join("unrelated.txt"), b"staged by another session\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--", "unrelated.txt"]);
    return repo;
}

/// A first merge held at one of its own phases,
/// and the directory its markers are in.
struct Held {
    /// The merge that is part-way.
    child: Child,
    /// The directory its marker files are in.
    markers: PathBuf,
    /// The phase it is held at.
    phase: String,
}

impl Held {
    /// Release the merge wherever it pauses and wait for it,
    /// answering its exit status.
    /// Why:
    ///  A merge that is met at its phase again pauses again,
    ///  so every occurrence is released at once,
    ///  the way the audit's own harness does it.
    fn release(mut self) -> Option<i32> {
        for stem in released_stems(self.phase.as_str()) {
            std::fs::write(self.markers.join(format!("{stem}.release")), b"").expect("the release");
        }
        return self.child.wait().ok().and_then(|status| { return status.code(); });
    }
}

/// The marker names a held merge is released by:
/// the phase itself and every occurrence after it.
fn released_stems(phase: &str) -> Vec<String> {
    let mut stems: Vec<String> = vec![String::from(phase)];
    for occurrence in 0..40 {
        stems.push(format!("{phase}-{occurrence}"));
    }
    return stems;
}

impl Drop for Held {
    fn drop(&mut self) {
        let _ended = self.child.kill();
        let _waited = self.child.wait();
    }
}

/// Hold a merge of `feature` at `phase`,
/// and answer once it is there.
fn hold_the_merge_at(fixture: &Fixture, repo: &Path, phase: &str) -> Held {
    let markers: PathBuf = fixture.root.join(format!("markers-{phase}-{}", std::process::id()));
    std::fs::create_dir_all(markers.as_path()).expect("the marker directory");
    let said: PathBuf = fixture.root.join("held.err");
    let mut child: Child = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"])
        .env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::from(std::fs::File::create(said.as_path()).expect("the merge's log")))
        .spawn()
        .expect("the merge starts");
    let began: Instant = Instant::now();
    while began.elapsed() < MARKER_PATIENCE {
        let reached: bool = std::fs::read_dir(markers.as_path()).is_ok_and(|mut entries| {
            return entries.any(|entry| {
                let Ok(found) = entry else { return false };
                return found.file_name().to_string_lossy().ends_with(".reached");
            });
        });
        if reached {
            return Held { child, markers, phase: String::from(phase) };
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ended = child.kill();
    let _waited = child.wait();
    let words: String = std::fs::read_to_string(fixture.root.join("held.err")).unwrap_or_default();
    panic!("the merge never reached {phase}: {words}");
}

/// Every path the commit at `HEAD` holds that the index does not.
fn paths_the_index_lacks(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let committed: Vec<String> =
        git_text(fixture, repo, &["ls-tree", "-r", "--name-only", "HEAD"]).lines().map(String::from).collect();
    let indexed: String = git_text(fixture, repo, &["ls-files", "--cached"]);
    let mut lacking: Vec<String> = Vec::new();
    for path in committed {
        if !indexed.lines().any(|listed| { return listed == path; }) {
            lacking.push(path);
        }
    }
    return lacking;
}

/// Run a wrapped command with the fixture's service reachable.
fn wrapped_at(fixture: &Fixture, service: &FixtureService, repo: &Path, arguments: &[&str]) -> Observed {
    let mut command = wrapped_commit(fixture, repo, arguments);
    command.env("XDG_RUNTIME_DIR", &service.runtime);
    return observe(&mut command, b"");
}

/// A commit that arrives while the merge is held at `phase`:
/// what the commit does,
/// what the merge then does,
/// and the state the worktree is left in.
fn one(phase: &str, commit_arguments: &[&str]) {
    one_of_a_worktree(phase, commit_arguments, false);
}

/// The same arrival in a repository whose merge writes no path the person has work in:
/// the commit is the only command that writes the branch,
/// so the row measures the commit against the install and nothing else.
fn one_of_a_clean_repository(phase: &str, commit_arguments: &[&str]) {
    one_of_a_worktree(phase, commit_arguments, true);
}

/// One arrival,
/// in either repository.
fn one_of_a_worktree(phase: &str, commit_arguments: &[&str], clean: bool) {
    let name: String = match clean {
        true => format!("merge-commit-clean-{phase}"),
        false => format!("merge-commit-arrives-{phase}"),
    };
    let fixture: Fixture = fixture(name.as_str());
    let repo: PathBuf = match clean {
        true => clean_world(&fixture, "repo"),
        false => world(&fixture, "repo"),
    };
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    let held: Held = hold_the_merge_at(&fixture, repo.as_path(), phase);
    let commit: Observed = wrapped_at(&fixture, &service, repo.as_path(), commit_arguments);
    let first: Option<i32> = held.release();
    let status: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain"]);
    let later: Observed = wrapped_at(&fixture, &service, repo.as_path(), &["status", "--porcelain"]);
    let log: String = git_text(&fixture, repo.as_path(), &["log", "--oneline", "--format=%s", "--max-count=5"]);
    println!("=== {phase}: commit exit={:?} said={:?}", commit.code, stderr_of(&commit));
    println!("=== {phase}: merge exit={first:?}");
    println!("=== {phase}: status={status:?}");
    println!("=== {phase}: later status exit={:?} said={:?}", later.code, stderr_of(&later));
    println!("=== {phase}: log={log:?}");
    println!("=== {phase}: index lacks={:?}", paths_the_index_lacks(&fixture, repo.as_path()));
    println!("=== {phase}: records={:?}", records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()));
    assert_eq!(first, Some(0), "the merge a person typed never fails");
    assert_eq!(later.code, Some(0), "the next command reads the worktree");
    assert_eq!(paths_the_index_lacks(&fixture, repo.as_path()), Vec::<String>::new(), "the index holds every path the merge commit holds");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record is left");
    assert!(!repo.join(".git/monochromatic-git-policy-cli-merges/.installing").exists(), "the mark went with the install");
    assert!(!repo.join(".git/index.lock").exists(), "no index lock is left");
    let tree: String = git_text(&fixture, repo.as_path(), &["ls-tree", "-r", "--name-only", "HEAD"]);
    let subjects: String = git_text(&fixture, repo.as_path(), &["log", "--format=%s", "--max-count=4"]);
    println!("=== {phase}: status={status:?} tree={tree:?} subjects={subjects:?}");
    assert_eq!(commit.code, Some(0), "the commit lands: {}", stderr_of(&commit));
    // Both commands are in the branch, whichever of the two landed first.
    assert!(subjects.contains("second"), "the commit is on the branch: {subjects:?}");
    assert!(subjects.contains("Merge branch 'feature'"), "the merge is on the branch: {subjects:?}");
    assert!(tree.contains("c.txt") && tree.contains("d.txt"), "the merge's own paths are in the branch: {tree:?}");
    match clean {
        true => {
            assert_eq!(status, "", "nothing is left in the worktree after both landed: {status:?}");
            assert!(tree.contains("unrelated.txt"), "the commit's own path is in the branch: {tree:?}");
        }
        false => {
            for (file, mark) in [("a.txt", "MARK-A-UNSTAGED"), ("c.txt", "MARK-C-UNTRACKED"), ("d.txt", "MARK-D-IGNORED")] {
                let words: String = std::fs::read_to_string(repo.join(file)).unwrap_or_default();
                assert!(words.contains(mark), "the person's own file {file} holds {mark}: {words:?}");
            }
        }
    }
    remove_settled(&fixture);
}

/// What:
///  The commit row of the phase table,
/// in the worktree the audit measured F12 in:
/// the person's staged edit,
/// committed by its path,
///  arriving while the merge is part-way in the install.
/// Why:
///  These two phases are the ones the audit's second merge went wrong in
///  (`merge-cleared` left an index short of two paths,
/// `merge-clear-file` wedged the worktree),
///  so the same two phases are where the commit row matters most.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdTheMergeAt(fixture, repo, 'merge-cleared'); expect(run(['commit', '--', 'b.txt']).code).toBe(0);
/// expect(held.release()).toBe(0); expect(whole()).toBe(true);
/// ```
#[test]
fn a_commit_arrives_at_merge_cleared() {
    one("merge-cleared", STAGED_PATH_COMMIT);
}

/// The commit arrives while a path is being cleared.
#[test]
fn a_commit_arrives_at_merge_clear_file() {
    one("merge-clear-file", STAGED_PATH_COMMIT);
}

/// What:
///  The same arrival where the merge writes no path the person has work in.
/// Why:
///  It is the row that can be measured on a host where the merge's copy proofs
///  cannot prove themselves,
///  and it answers what the commit does against the install mark:
/// it lands,
///  the merge lands on it or after it,
///  and the worktree ends whole.
#[test]
fn a_clean_commit_arrives_at_merge_intent() {
    one_of_a_clean_repository("merge-intent", UNRELATED_PATH_COMMIT);
}

/// The clean commit arrives while Git writes the merge into the checkout.
#[test]
fn a_clean_commit_arrives_at_merge_applied() {
    one_of_a_clean_repository("merge-applied", UNRELATED_PATH_COMMIT);
}

/// The clean commit arrives after the person's capture and the record are written.
#[test]
fn a_clean_commit_arrives_at_merge_carried() {
    one_of_a_clean_repository("merge-carried", UNRELATED_PATH_COMMIT);
}
