//! What:
//!  A commit whose prepared content was removed from the repository before it landed (for example by
//!  `git gc --prune=now` running in the same repository),
//! through the real wrapper and the real service.
//! Why:
//!  `git gc --prune=now` deletes every unreferenced object at once,
//! and the objects a commit prepares are unreferenced until
//!  the branch moves (`service_pruned.rs` quotes the Git manual).
//! The person must never meet a machine event or an internal
//!  failure for it:
//! the commit is prepared again once,
//! and if the content is still gone it fails plainly with nothing
//!  changed.
//! The wrapper is paused at a named phase marker (`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`),
//! the content is removed
//!  while it waits,
//! and it is released;
//! nothing is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pauseAt('service-prepared'); await deleteLooseObject(blob); release(); expect(await commit()).toLand();
//! ```

use super::commit::{
    EDITOR_WRITES_A_MESSAGE,
    git_text,
    remove_settled,
    wrapped_commit,
};
use super::support::{
    Fixture,
    FixtureService,
    Observed,
    fixture,
    git,
    observe,
    start_fixture_service,
};
use git_policy_cli::service_messages::{
    ClientFailure,
    render,
};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::time::{
    Duration,
    Instant,
};

/// The longest a control waits for the paused wrapper to arrive,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// The commit most controls of this file run:
/// an ordinary commit of the one edited file.
const THE_PLAIN_COMMIT: &[&str] = &["commit", "--quiet", "-m", "more", "--", "a.txt"];

/// The amend a control of the tree question runs:
/// it embeds the whole tree and names no list of blobs.
const AN_AMEND: &[&str] = &[
    "commit", "--quiet", "--amend", "-m", "amended", "--", "a.txt",
];

/// An editor commit Git builds in the shadow,
/// whose request names the built commit and nothing else.
const A_COMMIT_GIT_BUILDS: &[&str] = &["commit", "--quiet", "--", "a.txt"];

/// A repository with one tracked file and an edit to it that the next commit names.
struct Case {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    /// The name of the blob of the edited file's new content,
    /// which the commit writes and a pruning cleanup would delete.
    blob: String,
    /// The name of the blob of a file below a directory,
    /// which no commit of a control changes and a pruning cleanup would delete.
    under_the_tree: String,
    /// Where the paused wrapper leaves its markers and waits for its release files.
    markers: PathBuf,
}

fn case(name: &str) -> Case {
    return case_in(name, &[], false);
}

/// What:
///  `case` in the ref store `format` names,
/// with a replace ref where `replace` asks for one.
/// Why:
///  A reftable store and a files store holding a replace ref are the two stores the landing's in-process
///  object reader declines,
///  so the arms that show the declined reader's cost live where each of them stands.
fn case_in(
    name: &str,
    format: &[&str],
    replace: bool,
) -> Case {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    let mut init: Vec<&str> = vec!["init", "--quiet", "--initial-branch=main"];
    init.extend_from_slice(format);
    git(&fixture, &repo, init.as_slice());
    git(
        &fixture,
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir").join("gone.txt"), "gone\n").expect("gone");
    git(&fixture, &repo, &["add", "a.txt", "dir/gone.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    if replace {
        let older: String = git_text(&fixture, &repo, &["rev-parse", "HEAD~1"])
            .trim()
            .to_string();
        let head: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"])
            .trim()
            .to_string();
        git(&fixture, &repo, &["replace", older.as_str(), head.as_str()]);
    }
    git(
        &fixture,
        &repo,
        &["config", "maintenance.autoDetach", "false"],
    );
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let blob: String = git_text(&fixture, &repo, &["hash-object", "a.txt"])
        .trim()
        .to_string();
    let under_the_tree: String = git_text(&fixture, &repo, &["rev-parse", "HEAD:dir/gone.txt"])
        .trim()
        .to_string();
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("markers");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Case {
        fixture,
        repo,
        service,
        blob,
        under_the_tree,
        markers,
    };
}

/// Removes the loose file of the commit's blob,
/// as a pruning cleanup would;
/// a packed or absent object is left alone.
fn remove_blob(
    repo: &Path,
    blob: &str,
) {
    let _removed =
        std::fs::remove_file(repo.join(".git/objects").join(&blob[..2]).join(&blob[2..]));
}

/// Answers the paused wrapper at `phase` for each of `occurrences` times in turn:
/// waits for it to arrive,
/// removes the object `doomed` names
/// when `remove` holds for that occurrence,
/// and releases it.
fn answer_phase(
    case: &Case,
    phase: &'static str,
    occurrences: usize,
    remove: fn(usize) -> bool,
    doomed: fn(&Case) -> String,
) -> std::thread::JoinHandle<Vec<bool>> {
    let markers: PathBuf = case.markers.clone();
    let repo: PathBuf = case.repo.clone();
    let name: String = doomed(case);
    return std::thread::spawn(move || {
        let mut removed: Vec<bool> = Vec::new();
        for number in 1..=occurrences {
            let reached: PathBuf = markers.join(format!("{phase}-{number}.reached"));
            let began: Instant = Instant::now();
            // The wrapper may finish before a later occurrence: a commit that landed needs no further answer.
            while !reached.exists() {
                if began.elapsed() > PATIENCE || markers.join("finished").exists() {
                    return removed;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            let blot_out: bool = remove(number);
            if blot_out {
                remove_blob(repo.as_path(), name.as_str());
            }
            removed.push(blot_out);
            std::fs::write(markers.join(format!("{phase}-{number}.release")), "").expect("release");
        }
        return removed;
    });
}

/// Runs the commit of `a.txt` with the wrapper paused at `phase`,
/// answered by `answer`,
/// and returns what the caller saw.
fn commit_paused(
    case: &Case,
    phase: &str,
    arguments: &[&str],
    answer: std::thread::JoinHandle<Vec<bool>>,
) -> (Observed, Vec<bool>) {
    let mut command: Command = wrapped_commit(&case.fixture, &case.repo, arguments);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env(
        PHASE_SIGNAL_VARIABLE,
        format!("{phase}:pause:{}", case.markers.display()),
    );
    let seen: Observed = observe(&mut command, b"");
    std::fs::write(case.markers.join("finished"), "").expect("finished");
    let removed: Vec<bool> = answer.join().expect("the answering thread");
    return (seen, removed);
}

fn stderr_text(seen: &Observed) -> String {
    return String::from_utf8_lossy(seen.stderr.as_slice()).into_owned();
}

/// The branch tip's subject,
/// which says whether the commit landed.
fn tip_subject(case: &Case) -> String {
    return git_text(&case.fixture, &case.repo, &["log", "-1", "--format=%s"])
        .trim()
        .to_string();
}

/// The crash reports a failed import leaves in the Git directory,
/// which a failure that says what happened never leaves.
fn crash_reports(case: &Case) -> Vec<String> {
    return std::fs::read_dir(case.repo.join(".git"))
        .expect("git dir")
        .flatten()
        .map(|entry| {
            return entry.file_name().to_string_lossy().into_owned();
        })
        .filter(|name| {
            return name.contains("crash");
        })
        .collect();
}

/// What a person must be able to read when the content stays gone:
/// no machine event,
/// no internal failure,
/// and no crash report left behind.
/// Why:
///  A crash report is what an import that died on a missing blob leaves,
///  and the person who meets the plain words must not find one either.
fn assert_plain_failure(
    case: &Case,
    seen: &Observed,
) {
    assert_eq!(
        crash_reports(case),
        Vec::<String>::new(),
        "the failed import left its crash report behind"
    );
    let all: String = stderr_text(seen);
    // The phase marker a test arms logs its own warning lines; they are not part of what the wrapper tells the person.
    let text: String = all
        .lines()
        .filter(|line| {
            return !line.starts_with("[warn] ");
        })
        .collect::<Vec<&str>>()
        .join("\n");
    assert_eq!(seen.code, Some(1), "{text}");
    assert_eq!(
        text.trim_end(),
        render(&ClientFailure::ContentPruned),
        "{text}"
    );
    for leaked in ["schemaVersion", "engine-failure", "{\""] {
        assert!(
            !text.contains(leaked),
            "{leaked} reached the person: {text}"
        );
    }
    assert!(
        text.contains("--prune=now") && text.contains("Nothing in the repository changed"),
        "{text}"
    );
    assert_eq!(
        tip_subject(case),
        "base",
        "a failed commit changed the branch"
    );
    assert_eq!(
        std::fs::read_to_string(case.repo.join("a.txt")).expect("a"),
        "base\nmore\n",
        "the person's edit is intact"
    );
}

/// Content removed after it was staged and before the policies read it:
/// the bytes the command captured are gone,
/// and the
///  worktree is not read again to replace them,
/// so the commit fails plainly with nothing changed.
#[test]
fn content_removed_while_staged_fails_plainly_without_reading_the_worktree_again() {
    let case: Case = case("pruned-staged");
    let answer = answer_phase(
        &case,
        "service-staged",
        2,
        |_| {
            return true;
        },
        |whole| {
            return whole.blob.clone();
        },
    );
    let (seen, removed) = commit_paused(&case, "service-staged", THE_PLAIN_COMMIT, answer);
    assert_eq!(removed, vec![true], "the commit was not prepared again");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}

/// Content removed after the commit was prepared and before the service landed it:
/// the same plain failure,
/// not a failure of
///  the service,
/// and nothing is prepared again.
#[test]
fn content_removed_before_landing_fails_plainly_without_reading_the_worktree_again() {
    let case: Case = case("pruned-landing");
    let answer = answer_phase(
        &case,
        "service-prepared",
        2,
        |_| {
            return true;
        },
        |whole| {
            return whole.blob.clone();
        },
    );
    let (seen, removed) = commit_paused(&case, "service-prepared", THE_PLAIN_COMMIT, answer);
    assert_eq!(removed, vec![true], "the commit was not prepared again");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}

/// What:
///  Content removed before a reftable repository's commit landed fails plainly,
///  and no crash report is left.
/// Why:
///  A reftable reflog makes the landing's in-process object reader decline the database,
///  and taking that decline as "nothing is gone" ran `fast-import` over the missing blob:
///  the person met the service's general words and the repository kept `fast_import_crash_<pid>`.
/// Git answers the question the reader declined.
#[test]
fn content_removed_before_landing_fails_plainly_in_a_reftable_repository() {
    let case: Case = case_in("pruned-landing-reftable", &["--ref-format=reftable"], false);
    let answer = answer_phase(
        &case,
        "service-prepared",
        2,
        |_| {
            return true;
        },
        |whole| {
            return whole.blob.clone();
        },
    );
    let (seen, removed) = commit_paused(&case, "service-prepared", THE_PLAIN_COMMIT, answer);
    assert_eq!(removed, vec![true], "the commit was not prepared again");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}

/// What:
///  The same plain failure and the same clean repository beside a replace ref,
///  the other store the reader declines.
/// Why:
///  A files repository that holds a replace ref declined the database on `main` too,
///  and its landing gave the general failure and left the crash report there as well;
///  the control keeps that arm honest now that every caller asks Git.
#[test]
fn content_removed_before_landing_fails_plainly_beside_a_replace_ref() {
    let case: Case = case_in("pruned-landing-replace-ref", &[], true);
    let answer = answer_phase(
        &case,
        "service-prepared",
        2,
        |_| {
            return true;
        },
        |whole| {
            return whole.blob.clone();
        },
    );
    let (seen, removed) = commit_paused(&case, "service-prepared", THE_PLAIN_COMMIT, answer);
    assert_eq!(removed, vec![true], "the commit was not prepared again");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}

/// What:
///  A blob a pruning cleanup removed from below an amended tree fails plainly where the reader declines,
///  and no crash report is left.
/// Why:
///  An amendment embeds a whole tree and no list of blobs,
///  so a blob that the amendment did not change is named only by the tree's own question.
///  Taking the reader's decline as "nothing is gone" imported over the loss and moved the branch to a commit whose
///  tree names an object no store holds.
#[test]
fn a_blob_removed_below_an_amended_tree_fails_plainly_where_the_reader_declines() {
    let case: Case = case_in(
        "pruned-amend-tree-reftable",
        &["--ref-format=reftable"],
        false,
    );
    let answer = answer_phase(
        &case,
        "service-prepared",
        2,
        |_| {
            return true;
        },
        |whole| {
            return whole.under_the_tree.clone();
        },
    );
    let (seen, removed) = commit_paused(&case, "service-prepared", AN_AMEND, answer);
    assert_eq!(removed, vec![true], "the amend was not prepared again");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}

/// What:
///  A commit Git built whose object a pruning cleanup removed fails plainly where the reader declines,
///  and no crash report is left.
/// Why:
///  A request for a commit Git built names that commit and nothing else,
///  so its presence is the only question the landing can ask about the content it captured.
///  A decline taken as "not gone" moved the branch to a commit no object holds.
#[test]
fn a_commit_git_builds_that_a_prune_removed_fails_plainly_where_the_reader_declines() {
    let case: Case = case_in("pruned-built-commit-replace-ref", &[], true);
    git(
        &case.fixture,
        &case.repo,
        &["config", "core.editor", EDITOR_WRITES_A_MESSAGE],
    );
    let mut command: Command = wrapped_commit(&case.fixture, &case.repo, A_COMMIT_GIT_BUILDS);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env(
        PHASE_SIGNAL_VARIABLE,
        format!("service-prepared:pause:{}", case.markers.display()),
    );
    let (seen, pruned) = std::thread::scope(|scope| {
        let answering = scope.spawn(|| {
            let reached: PathBuf = case.markers.join("service-prepared-1.reached");
            let began: Instant = Instant::now();
            while !reached.exists() {
                if began.elapsed() > PATIENCE || case.markers.join("finished").exists() {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            // The cleanup that removes the commit Git built: no reference names it yet.
            git(&case.fixture, &case.repo, &["gc", "--prune=now"]);
            std::fs::write(case.markers.join("service-prepared-1.release"), "").expect("release");
            return true;
        });
        let seen: Observed = observe(&mut command, b"");
        std::fs::write(case.markers.join("finished"), "").expect("finished");
        let pruned: bool = answering.join().expect("the answering thread");
        return (seen, pruned);
    });
    assert!(pruned, "the wrapper reached the phase before its landing");
    assert_plain_failure(&case, &seen);
    remove_settled(&case.fixture);
}
