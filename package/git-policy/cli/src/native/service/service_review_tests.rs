//! Controls for the service's review handler against a real fixture repository and the loopback mock gateway.
//! No test reaches a real host or sends a real key.

use super::*;
use crate::standards_review_input::{TRACKED_LOG_RELATIVE, shared_log_path};
use crate::standards_review_mock::{Answer, Mock, SILENT_AT_MOST, http_reply};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::WorkKind;
use std::os::unix::fs::OpenOptionsExt;

/// A stand-in credential.
const KEY: &str = "test-key-not-a-secret";

/// A gateway answer with a verdict.
const ANSWER: &str = r#"{"answers":[{"choice":"no","probabilities":[{"value":"yes","probability":0.1},{"value":"no","probability":0.9}]}]}"#;

/// A repository whose worktree holds a committed shadow log.
struct Fixture {
    parent: PathBuf,
    root: PathBuf,
}

/// Make the repository with the log and one base commit.
fn fixture_with_log(name: &str) -> Fixture {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = repository(parent.as_path(), "repo");
    let log: PathBuf = root.join(TRACKED_LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directories");
    std::fs::write(log.as_path(), TRACKED_LOG_CONTENT).expect("log");
    git(root.as_path(), &["add", "--", TRACKED_LOG_RELATIVE]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    return Fixture { parent, root };
}

/// Commit `files` (name,
/// text) and return the work item for the new commit.
fn commit(fixture: &Fixture, files: &[(&str, &str)], message: &str) -> WorkItem {
    for (name, text) in files {
        let path: PathBuf = fixture.root.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(path, text).expect("write");
        git(fixture.root.as_path(), &["add", "--", name]);
    }
    git(fixture.root.as_path(), &["commit", "--quiet", &format!("--message={message}")]);
    let oid: String = git_text(fixture.root.as_path(), &["rev-parse", "HEAD"]);
    return WorkItem {
        kind: WorkKind::Review,
        repository: fixture.root.to_path_buf(),
        git_dir: fixture.root.join(".git").to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid,
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

/// The options the service builds,
/// with the gateway pointed at `endpoint` and the credential set.
fn options(item: &WorkItem, endpoint: &str) -> ReviewOptions {
    return options_with(item, endpoint, ReviewKey::new(KEY).as_ref());
}

/// The options the service builds for a wake message that carried `key` (or none),
/// pointed at `endpoint`.
fn options_with(item: &WorkItem, endpoint: &str, key: Option<&ReviewKey>) -> ReviewOptions {
    let mut built: ReviewOptions = options_for(item, &review_variables(key));
    built.endpoint = String::from(endpoint);
    return built;
}

/// The text of the tracked log in the fixture's worktree,
/// which the service must never change.
fn tracked_log_of(fixture: &Fixture) -> String {
    return std::fs::read_to_string(fixture.root.join(TRACKED_LOG_RELATIVE)).expect("tracked log");
}

/// The tracked log's content as the fixture commits it.
const TRACKED_LOG_CONTENT: &str = "# Log\n";

/// The text of the fixture repository's live log;
/// empty when it does not exist.
fn log_of(fixture: &Fixture) -> String {
    return std::fs::read_to_string(shared_log_path(fixture.root.join(".git").as_path())).unwrap_or_default();
}

/// A code commit is reviewed:
/// the gateway is asked with the commit's own text,
/// and the entry lands in the live log in the
/// common Git directory,
/// which did not exist before,
/// while the tracked log stays exactly as committed.
#[test]
fn a_code_commit_is_reviewed_and_appended() {
    let fixture: Fixture = fixture_with_log("review-code");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "export const a = 1\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    let verdict: Verdict = review_item(&git_context(), &item, options(&item, mock.url.as_str()));
    assert_eq!(verdict, Verdict::Done);
    let seen: String = mock.finish().seen;
    assert!(seen.contains("src/a.ts"), "the request carries the changed file");
    let log: String = log_of(&fixture);
    assert!(log.starts_with("\n- Date:\n"), "the live log holds the entry after the line feed every write begins with: {log}");
    assert_eq!(tracked_log_of(&fixture), TRACKED_LOG_CONTENT, "the tracked log is never written");
    assert!(log.contains("`feat(a): add a`"));
    assert!(log.contains("cost_verdict: no (yes 0.1 no 0.9)"));
    assert!(log.contains(&item.oid[..crate::standards_review_entry::SHORT_SHA_LENGTH]));
    remove(fixture.parent.as_path());
}

/// A repeated item adds nothing and asks nothing,
/// so a crash between the append and the settling is harmless.
#[test]
fn a_repeated_item_adds_nothing() {
    let fixture: Fixture = fixture_with_log("review-repeat");
    let item: WorkItem = commit(&fixture, &[("src/a.rs", "fn a() {}\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    assert_eq!(review_item(&git_context(), &item, options(&item, mock.url.as_str())), Verdict::Done);
    mock.finish();
    let after_first: String = log_of(&fixture);
    assert_eq!(review_item(&git_context(), &item, options(&item, "http://127.0.0.1:1/unused")), Verdict::Done);
    assert_eq!(log_of(&fixture), after_first);
    remove(fixture.parent.as_path());
}

/// A commit with no code gets the skip entry without any gateway call,
/// and a commit that only changes the log gets none.
#[test]
fn skips_and_log_only_commits_make_no_call() {
    let fixture: Fixture = fixture_with_log("review-skip");
    let unused: &str = "http://127.0.0.1:1/unused";
    let doc: WorkItem = commit(&fixture, &[("doc/a.md", "text\n")], "docs(a): text");
    assert_eq!(review_item(&git_context(), &doc, options(&doc, unused)), Verdict::Done);
    let skipped: String = log_of(&fixture);
    assert!(skipped.contains("(no code files):"));
    let carried: String = format!("{TRACKED_LOG_CONTENT}carried\n");
    let only_log: WorkItem = commit(&fixture, &[(TRACKED_LOG_RELATIVE, &carried)], "docs(audit): carry entries");
    let carried_more: String = format!("{carried}carried more\n");
    let only_log_changed: WorkItem = commit(&fixture, &[(TRACKED_LOG_RELATIVE, &carried_more)], "docs(audit): carry more");
    for item in [&only_log, &only_log_changed] {
        assert_eq!(review_item(&git_context(), item, options(item, unused)), Verdict::Done);
    }
    assert_eq!(log_of(&fixture), skipped, "a commit of the tracked log itself leaves no entry");
    remove(fixture.parent.as_path());
}

/// A linked worktree's item names that worktree's own Git directory (`.git/worktrees/<name>`),
/// and its entry still goes to
///  the repository's one log in the common Git directory,
/// which outlives the worktree;
/// nothing is written beside the worktree.
#[test]
fn a_linked_worktrees_entry_goes_to_the_repositorys_one_log() {
    let fixture: Fixture = fixture_with_log("review-linked");
    let linked: PathBuf = fixture.parent.join("linked");
    git(fixture.root.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("UTF-8")]);
    std::fs::create_dir_all(linked.join("doc")).expect("directories");
    std::fs::write(linked.join("doc/b.md"), "text\n").expect("write");
    git(linked.as_path(), &["add", "--", "doc/b.md"]);
    git(linked.as_path(), &["commit", "--quiet", "--message=docs(b): text"]);
    let own_git_dir: String = git_text(linked.as_path(), &["rev-parse", "--absolute-git-dir"]);
    let common: PathBuf = std::fs::canonicalize(fixture.root.join(".git")).expect("common directory");
    assert_ne!(Path::new(own_git_dir.as_str()), common.as_path(), "the control needs a linked worktree's own Git directory");
    assert_eq!(common_dir_of(Path::new(own_git_dir.as_str())), common);
    assert_eq!(common_dir_of(common.as_path()), common, "a main Git directory is its own common directory");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Review,
        repository: linked.to_path_buf(),
        git_dir: PathBuf::from(own_git_dir.clone()),
        branch_ref: String::from("refs/heads/side"),
        oid: git_text(linked.as_path(), &["rev-parse", "HEAD"]),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    let built: ReviewOptions = options(&item, "http://127.0.0.1:1/unused");
    assert_eq!(built.log_path, shared_log_path(common.as_path()));
    assert_eq!(review_item(&git_context(), &item, built), Verdict::Done);
    assert!(log_of(&fixture).contains("(no code files):"), "{}", log_of(&fixture));
    assert!(!shared_log_path(Path::new(own_git_dir.as_str())).exists(), "nothing beside the worktree");
    remove(fixture.parent.as_path());
}

/// A repository without a shadow log is left alone:
/// no call,
/// no file created.
#[test]
fn a_repository_without_a_shadow_log_is_left_alone() {
    let parent: PathBuf = fixture("review-no-log");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("a.ts"), "x\n").expect("write");
    git(root.as_path(), &["add", "--", "a.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=feat: a"]);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Review,
        repository: root.to_path_buf(),
        git_dir: root.join(".git").to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: git_text(root.as_path(), &["rev-parse", "HEAD"]),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
    assert_eq!(review_item(&git_context(), &item, options(&item, "http://127.0.0.1:1/unused")), Verdict::Done);
    assert!(!root.join(TRACKED_LOG_RELATIVE).exists());
    assert!(!shared_log_path(root.join(".git").as_path()).exists(), "no switch, no live log");
    remove(parent.as_path());
}

/// A gateway failure is an entry in the log and the item is done;
/// an unknown commit is a failed item with the reason.
#[test]
fn a_gateway_failure_is_logged_and_an_unreadable_commit_fails_the_item() {
    let fixture: Fixture = fixture_with_log("review-failures");
    let item: WorkItem = commit(&fixture, &[("src/a.js", "let a\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("503 Service Unavailable", "busy")));
    assert_eq!(review_item(&git_context(), &item, options(&item, mock.url.as_str())), Verdict::Done);
    mock.finish();
    assert!(log_of(&fixture).contains("Luna failed: HTTP 503 busy"));
    let unknown: WorkItem = WorkItem { oid: "0".repeat(40), ..item.clone() };
    let verdict: Verdict = review_item(&git_context(), &unknown, options(&unknown, "http://127.0.0.1:1/unused"));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains("git")), "{verdict:?}");
    remove(fixture.parent.as_path());
}

/// A gateway that never answers is ended when it has been silent for the quiet window,
/// and the item is done with the
///  failure in its log entry:
/// no budget around the whole review decides it.
#[test]
fn a_silent_gateway_ends_the_review_when_it_has_been_silent_for_the_quiet_window() {
    let fixture: Fixture = fixture_with_log("review-silent");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "x\n")], "feat(a): add a");
    let window: Duration = Duration::from_millis(500);
    let mock: Mock = Mock::start(Answer::Silent(SILENT_AT_MOST));
    let mut silent: ReviewOptions = options(&item, mock.url.as_str());
    silent.quiet_window = window;
    let verdict: Verdict = review_item(&git_context(), &item, silent);
    // No upper bound by the clock: a call that was not ended would run to the silent gateway's close and be done differently.
    assert_eq!(verdict, Verdict::Done);
    let log: String = log_of(&fixture);
    assert!(log.contains("Luna failed: The operation was aborted due to timeout"), "{log}");
    mock.finish();
    remove(fixture.parent.as_path());
}

/// The options point the log at the live log in the item's common Git directory,
/// never at the tracked log,
/// and give the call its quiet window.
#[test]
fn options_follow_the_item() {
    let fixture: Fixture = fixture_with_log("review-options");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "x\n")], "feat(a): add a");
    let built: ReviewOptions = options(&item, "http://127.0.0.1:1/unused");
    assert_eq!(built.log_path, shared_log_path(item.git_dir.as_path()));
    assert_ne!(built.log_path, fixture.root.join(TRACKED_LOG_RELATIVE));
    assert_eq!(built.quiet_window, CALL_QUIET_WINDOW);
    remove(fixture.parent.as_path());
}

/// The key that came in the work message is the bearer credential the gateway sees.
#[test]
fn the_key_from_the_message_reaches_the_gateway() {
    let fixture: Fixture = fixture_with_log("review-key-reaches");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "export const a = 1\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    let key: ReviewKey = ReviewKey::new(KEY).expect("a key");
    let verdict: Verdict = review_item(&git_context(), &item, options_with(&item, mock.url.as_str(), Some(&key)));
    assert_eq!(verdict, Verdict::Done);
    let seen: String = mock.finish().seen.to_ascii_lowercase();
    assert!(seen.contains(&format!("authorization: bearer {KEY}\r\n")), "{seen}");
    assert!(!log_of(&fixture).contains(KEY), "the credential is never written to the log");
    remove(fixture.parent.as_path());
}

/// A review whose message carried no key records the plain "no key" outcome and contacts no one.
#[test]
fn a_review_without_a_key_records_no_key() {
    let fixture: Fixture = fixture_with_log("review-no-key");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "export const a = 1\n")], "feat(a): add a");
    let unused: &str = "http://127.0.0.1:1/unused";
    let verdict: Verdict = review_item(&git_context(), &item, options_with(&item, unused, None));
    assert_eq!(verdict, Verdict::Done);
    let log: String = log_of(&fixture);
    assert!(log.contains(&format!("Luna failed: {}", crate::standards_review_run::NO_KEY_MESSAGE)), "{log}");
    assert!(log.contains("Named superlinear cost:\n  unjudged\n"));
    remove(fixture.parent.as_path());
}

/// The credential variables answer from the message only:
/// with no key they answer nothing,
/// whatever the service's own
/// environment holds,
/// while any other variable still answers from the environment.
#[test]
fn the_service_environment_never_supplies_a_key() {
    let none = review_variables(None);
    for name in KEY_VARIABLES {
        assert_eq!(none.variable(name), None, "{name}");
    }
    // A positive control: the lookup does reach the environment for a name that is not a credential.
    assert_eq!(none.variable("PATH"), std::env::var("PATH").ok());
    assert!(std::env::var("PATH").is_ok());
    let key: ReviewKey = ReviewKey::new(KEY).expect("a key");
    let held = review_variables(Some(&key));
    assert_eq!(held.variable(KEY_VARIABLES[0]).as_deref(), Some(KEY));
    assert_eq!(held.variable(KEY_VARIABLES[1]), None);
}

/// The clock the review stamps its entry and times the gateway call with is the wall clock:
/// a reading taken between two
///  readings of the system clock lies between them.
#[test]
fn the_review_clock_reads_the_wall_clock() {
    let epoch_milliseconds = || {
        let elapsed: Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("a clock after 1970");
        return u64::try_from(elapsed.as_millis()).expect("milliseconds that fit");
    };
    let before: u64 = epoch_milliseconds();
    let read: u64 = wall_clock_millis();
    let after: u64 = epoch_milliseconds();
    assert!(before <= read && read <= after, "{before} <= {read} <= {after}");
}

/// A review that ran as long as the service lets one run is a failed item whose reason names what was running,
/// the ceiling and
/// the command that does it again.
#[test]
fn a_review_stopped_at_the_ceiling_names_the_review_the_ceiling_and_the_command() {
    let oid: &str = "0123456789abcdef0123456789abcdef01234567";
    let ceiling: Duration = crate::wait_ceilings::REVIEW_CEILING;
    let reason: String = ceiling_reason(oid, ceiling);
    assert!(reason.contains(oid) && reason.contains(ceiling.as_secs().to_string().as_str()), "{reason}");
    assert!(reason.contains("standards review") && reason.contains(format!("SHADOW_REVIEW_SHA={oid} mise run shadow-review").as_str()), "{reason}");
}

/// The shipped review ceiling covers the shipped call ceiling and its quiet window,
/// or the call's own bound could never be reached.
#[test]
fn the_review_ceiling_is_past_the_call_ceiling_and_the_quiet_window() {
    assert!(crate::wait_ceilings::REVIEW_CEILING > crate::wait_ceilings::REVIEW_CALL_CEILING);
    assert!(crate::wait_ceilings::REVIEW_CALL_CEILING > CALL_QUIET_WINDOW);
}

/// A review whose log write stalls (sol-2,
/// B):
/// the log is a named pipe that nobody reads,
/// so the review blocks at the log.
/// The
/// service stops waiting for it at the ceiling and fails it;
/// a second item is refused and starts no thread behind the stalled one;
/// when the pipe is read the stalled review ends,
/// and the same item,
/// reviewed again,
/// is done.
#[test]
fn a_stalled_log_blocks_later_reviews_without_piling_threads_and_the_item_is_reviewed_again_after() {
    static OWN: crate::standards_review_run::Containment = crate::standards_review_run::Containment::new();
    let fixture: Fixture = fixture_with_log("review-log-stall");
    let first: WorkItem = commit(&fixture, &[("src/a.rs", "fn a() {}\n")], "feat(a): add a");
    let second: WorkItem = commit(&fixture, &[("src/b.rs", "fn b() {}\n")], "feat(b): add b");
    let pipe: PathBuf = fixture.parent.join("stalled-log");
    let made = std::ffi::CString::new(pipe.to_str().expect("UTF-8")).expect("no NUL");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode, and touches no memory.
    assert_eq!(unsafe { libc::mkfifo(made.as_ptr(), 0o600) }, 0, "the named pipe");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    let mut stalled: ReviewOptions = options(&first, mock.url.as_str());
    stalled.log_path = pipe.clone();
    let ceiling: Duration = Duration::from_millis(700);
    let verdict: Verdict = review_item_in(&OWN, &git_context(), &first, stalled, ceiling);
    let Verdict::Failed(reason) = verdict else { panic!("{verdict:?}") };
    assert!(reason.contains(&first.oid) && reason.contains("SHADOW_REVIEW_SHA"), "{reason}");
    assert_eq!(OWN.outstanding(), 1, "the stalled review is tracked");
    let refused: Verdict = review_item_in(&OWN, &git_context(), &second, options(&second, mock.url.as_str()), ceiling);
    let Verdict::Failed(busy) = refused else { panic!("{refused:?}") };
    assert!(busy.contains("was not started") && busy.contains(&format!("SHADOW_REVIEW_SHA={}", second.oid)), "{busy}");
    assert_eq!(OWN.outstanding(), 1, "no thread was added behind the stalled one");
    assert_eq!(log_of(&fixture), "", "nothing was written for the refused item");
    // The pipe is replaced at its path by nothing (the review then finds no log and makes one), and a writer is opened on the pipe by
    // its other name, which wakes the stalled review's open of it, so the review runs on and ends.
    let other_name: PathBuf = fixture.parent.join("stalled-log-other-name");
    std::fs::hard_link(&pipe, &other_name).expect("a second name for the pipe");
    std::fs::remove_file(&pipe).expect("the pipe leaves its path");
    let waiting_since: std::time::Instant = std::time::Instant::now();
    while OWN.outstanding() > 0 {
        assert!(waiting_since.elapsed() < Duration::from_secs(30), "the stalled review never ended");
        let _poked = std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(other_name.as_path());
        std::thread::sleep(Duration::from_millis(20));
    }
    // The same item, through the very log path that stalled: the pipe left its path, so the path is a log again.
    let mut again_options: ReviewOptions = options(&first, mock.url.as_str());
    again_options.log_path = pipe.clone();
    let again: Verdict = review_item_in(&OWN, &git_context(), &first, again_options, Duration::from_secs(60));
    assert_eq!(again, Verdict::Done, "the same item is reviewed once the earlier review has ended");
    let written: String = std::fs::read_to_string(&pipe).unwrap_or_default();
    assert!(written.contains(&first.oid[..crate::standards_review_entry::SHORT_SHA_LENGTH]), "the entry is in the log that had stalled: {written}");
    mock.finish();
    remove(fixture.parent.as_path());
}

/// What:
///  A review whose thread the system refuses fails its item with the limit that refused the thread, and never as an
///  internal error of this program.
/// Why:
///  The item is the review of a commit that is being pushed or landed, so a person reads its verdict: a refusal that
///  is reported as "an internal error" sends them to report a defect of this program that does not exist. The child is
///  refused every thread for real (`RLIMIT_NPROC`), and its fixture is made while a process may still start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // under prlimit --nproc=1: expect(reviewItemIn(runner, context, item, options, ceiling).reason).toContain('ulimit -u');
/// ```
#[test]
fn where_no_thread_can_start_the_review_names_the_limit() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_review_names_the_limit") {
        return;
    }
    static OWN: crate::standards_review_run::Containment = crate::standards_review_run::Containment::new();
    // The fixture is made of Git processes, which the refusal refuses as well, so it is made while a process may start.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let fixture: Fixture = fixture_with_log("review-no-thread");
    let item: WorkItem = commit(&fixture, &[("src/a.rs", "fn a() {}\n")], "feat(a): add a");
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let verdict: Verdict = review_item_in(&OWN, &git_context(), &item, options(&item, "http://127.0.0.1:1/unused"), Duration::from_secs(60));
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises again");
    remove(fixture.parent.as_path());
    let Verdict::Failed(said) = verdict else { panic!("a review that never started is a failed item: {verdict:?}") };
    assert!(said.contains(&item.oid), "the item is named: {said}");
    assert!(said.contains("ulimit -u") && said.contains("pids.max"), "the refusal names the limits that refused the thread, and is not an internal error: {said}");
    assert!(!said.contains("internal error"), "a refused thread is not a defect of this program: {said}");
}
