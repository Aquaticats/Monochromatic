//! Controls for the service's review handler against a real fixture repository and the loopback mock gateway.
//! No test reaches a real host or sends a real key.

use super::*;
use crate::standards_review_input::LOG_RELATIVE;
use crate::standards_review_mock::{Answer, Mock, http_reply};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::WorkKind;

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
    let log: PathBuf = root.join(LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directories");
    std::fs::write(log.as_path(), "# Log\n").expect("log");
    git(root.as_path(), &["add", "--", LOG_RELATIVE]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    return Fixture { parent, root };
}

/// Commit `files` (name, text) and return the work item for the new commit.
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
        repository: fixture.root.display().to_string(),
        git_dir: fixture.root.join(".git").display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid,
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
}

/// The options the service builds, with the gateway pointed at `endpoint` and the credential set.
fn options(item: &WorkItem, endpoint: &str) -> ReviewOptions {
    let variable = |name: &str| -> Option<String> { return (name == "SHADOW_REVIEW_LLMGATEWAY_API_KEY").then(|| return String::from(KEY)); };
    let mut built: ReviewOptions = options_for(item, &variable);
    built.endpoint = String::from(endpoint);
    return built;
}

/// The log text of the fixture.
fn log_of(fixture: &Fixture) -> String {
    return std::fs::read_to_string(fixture.root.join(LOG_RELATIVE)).expect("log");
}

/// A code commit is reviewed: the gateway is asked with the commit's own text, and the entry lands after the old content.
#[test]
fn a_code_commit_is_reviewed_and_appended() {
    let fixture: Fixture = fixture_with_log("review-code");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "export const a = 1\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    let verdict: Verdict = review_item(&git_context(), &item, options(&item, mock.url.as_str()), BUDGET);
    assert_eq!(verdict, Verdict::Done);
    let seen: String = mock.finish();
    assert!(seen.contains("src/a.ts"), "the request carries the changed file");
    let log: String = log_of(&fixture);
    assert!(log.starts_with("# Log\n"), "existing content is untouched");
    assert!(log.contains("`feat(a): add a`"));
    assert!(log.contains("cost_verdict: no (yes 0.1 no 0.9)"));
    assert!(log.contains(&item.oid[..crate::standards_review_entry::SHORT_SHA_LENGTH]));
    remove(fixture.parent.as_path());
}

/// A repeated item adds nothing and asks nothing, so a crash between the append and the settling is harmless.
#[test]
fn a_repeated_item_adds_nothing() {
    let fixture: Fixture = fixture_with_log("review-repeat");
    let item: WorkItem = commit(&fixture, &[("src/a.rs", "fn a() {}\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", ANSWER)));
    assert_eq!(review_item(&git_context(), &item, options(&item, mock.url.as_str()), BUDGET), Verdict::Done);
    mock.finish();
    let after_first: String = log_of(&fixture);
    assert_eq!(review_item(&git_context(), &item, options(&item, "http://127.0.0.1:1/unused"), BUDGET), Verdict::Done);
    assert_eq!(log_of(&fixture), after_first);
    remove(fixture.parent.as_path());
}

/// A commit with no code gets the skip entry without any gateway call, and a commit that only changes the log gets none.
#[test]
fn skips_and_log_only_commits_make_no_call() {
    let fixture: Fixture = fixture_with_log("review-skip");
    let unused: &str = "http://127.0.0.1:1/unused";
    let doc: WorkItem = commit(&fixture, &[("doc/a.md", "text\n")], "docs(a): text");
    assert_eq!(review_item(&git_context(), &doc, options(&doc, unused), BUDGET), Verdict::Done);
    let skipped: String = log_of(&fixture);
    assert!(skipped.contains("(no code files):"));
    let only_log: WorkItem = commit(&fixture, &[(LOG_RELATIVE, &skipped)], "docs(audit): carry entries");
    let changed: String = format!("{skipped}carried\n");
    let only_log_changed: WorkItem = commit(&fixture, &[(LOG_RELATIVE, &changed)], "docs(audit): carry more");
    for item in [&only_log, &only_log_changed] {
        assert_eq!(review_item(&git_context(), item, options(item, unused), BUDGET), Verdict::Done);
    }
    assert_eq!(log_of(&fixture), changed, "a commit of the log itself leaves no entry");
    remove(fixture.parent.as_path());
}

/// A repository without a shadow log is left alone: no call, no file created.
#[test]
fn a_repository_without_a_shadow_log_is_left_alone() {
    let parent: PathBuf = fixture("review-no-log");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("a.ts"), "x\n").expect("write");
    git(root.as_path(), &["add", "--", "a.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=feat: a"]);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Review,
        repository: root.display().to_string(),
        git_dir: root.join(".git").display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid: git_text(root.as_path(), &["rev-parse", "HEAD"]),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
    assert_eq!(review_item(&git_context(), &item, options(&item, "http://127.0.0.1:1/unused"), BUDGET), Verdict::Done);
    assert!(!root.join(LOG_RELATIVE).exists());
    remove(parent.as_path());
}

/// A gateway failure is an entry in the log and the item is done; an unknown commit is a failed item with the reason.
#[test]
fn a_gateway_failure_is_logged_and_an_unreadable_commit_fails_the_item() {
    let fixture: Fixture = fixture_with_log("review-failures");
    let item: WorkItem = commit(&fixture, &[("src/a.js", "let a\n")], "feat(a): add a");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("503 Service Unavailable", "busy")));
    assert_eq!(review_item(&git_context(), &item, options(&item, mock.url.as_str()), BUDGET), Verdict::Done);
    mock.finish();
    assert!(log_of(&fixture).contains("Luna failed: HTTP 503 busy"));
    let unknown: WorkItem = WorkItem { oid: "0".repeat(40), ..item.clone() };
    let verdict: Verdict = review_item(&git_context(), &unknown, options(&unknown, "http://127.0.0.1:1/unused"), BUDGET);
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains("git")), "{verdict:?}");
    remove(fixture.parent.as_path());
}

/// A gateway that never answers cannot hold the work thread past the budget: the item fails with the budget named.
#[test]
fn the_budget_bounds_the_whole_review() {
    let fixture: Fixture = fixture_with_log("review-budget");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "x\n")], "feat(a): add a");
    let budget: Duration = Duration::from_millis(500);
    let mock: Mock = Mock::start(Answer::Silent(budget * 8));
    let mut slow: ReviewOptions = options(&item, mock.url.as_str());
    // A call deadline far past the budget, so only the budget can end the review in time.
    slow.deadline = budget * 8;
    let started: std::time::Instant = std::time::Instant::now();
    let verdict: Verdict = review_item(&git_context(), &item, slow, budget);
    let elapsed: Duration = started.elapsed();
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains("did not finish")), "{verdict:?}");
    assert!(elapsed < budget * 4, "{elapsed:?}");
    remove(fixture.parent.as_path());
}

/// The options read the log from the item's worktree, bound the call, and keep the spool beside the item's Git directory.
#[test]
fn options_follow_the_item() {
    let fixture: Fixture = fixture_with_log("review-options");
    let item: WorkItem = commit(&fixture, &[("src/a.ts", "x\n")], "feat(a): add a");
    let built: ReviewOptions = options(&item, "http://127.0.0.1:1/unused");
    assert_eq!(built.log_path, fixture.root.join(LOG_RELATIVE));
    assert_eq!(built.deadline, CALL_DEADLINE);
    assert!(CALL_DEADLINE < BUDGET);
    assert_eq!(built.spool, Some(spool_directory(Path::new(item.git_dir.as_str()), item.repository.as_str())));
    remove(fixture.parent.as_path());
}
