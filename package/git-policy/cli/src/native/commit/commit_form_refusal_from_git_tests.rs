//! What:
//!  The question's shape and what each answer decides:
//!  `--dry-run` is the question's first token whatever the person wrote,
//!  a fatal status is the refusal,
//!  a status of zero lets the commit go on,
//!  and each way of not answering says what happened.
//! Why:
//!  The wrapper promises Git's own wording for the forms Git refuses,
//!  and that it never refuses a commit Git makes;
//!  a lost `--dry-run` once made this question a real commit in another
//!  repository,
//!  so the placement is a property of the arguments and not of the cases
//!  someone thought of.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theQuestion(['-m', '--', '-i'])).toEqual(['commit', '--dry-run', '-m', '--', '-i']);
//! expect({ status: 0, stderr: [] }).decide().kind).toBe('accept');
//! ```

use super::{Answer, Decision, Unanswered, ask_git, the_question, unavailable_words};
use crate::testing::test_support::{fixture, git, git_context, git_text, repository};
use std::ffi::OsString;
use std::path::PathBuf;

/// The tokens of a command line as one list.
fn tokens(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The tokens of a command line as readable text,
/// for the comparison a test makes.
fn text(values: &[OsString]) -> Vec<String> {
    return values.iter().map(|value| { return value.to_string_lossy().into_owned(); }).collect();
}

/// A repository with one commit and one staged change,
/// so a commit command has something to commit.
fn a_repository_with_a_staged_change(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "a changed\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// What:
///  `--dry-run` is the question's first token,
/// and every token the person wrote follows it unchanged.
/// Why:
///  The first token after `commit` is in option position whatever it spells,
///  so no option of the person's can take `--dry-run` as its value,
///  and no `--` of the person's can put it after the end of the options.
#[test]
fn a_dry_run_is_the_questions_first_token_and_the_persons_tokens_follow_it() {
    let asked: Vec<String> = text(the_question(tokens(&["-m", "--", "-i"]).as_slice()).as_slice());
    assert_eq!(asked, ["commit", "--dry-run", "-m", "--", "-i"]);
    let heading_a_path: Vec<String> = text(the_question(tokens(&["--", "a.txt"]).as_slice()).as_slice());
    assert_eq!(heading_a_path, ["commit", "--dry-run", "--", "a.txt"]);
    let empty: Vec<String> = text(the_question(&[]).as_slice());
    assert_eq!(empty, ["commit", "--dry-run"]);
    // A message file named `--` is a value; `--dry-run` still comes first.
    let file_named_dashdash: Vec<String> = text(the_question(tokens(&["-F", "--", "-i"]).as_slice()).as_slice());
    assert_eq!(file_named_dashdash, ["commit", "--dry-run", "-F", "--", "-i"]);
}

/// What:
///  Git's own status and bytes end the command,
///  a status of zero is no refusal at all,
///  and no answer is the wrapper's own sentence.
/// Why:
///  "Commits should never fail":
/// a rule that is wider than Git must not refuse a commit Git makes,
///  and a Git that started and answered must not be reported as a Git that could not start.
#[test]
fn each_answer_decides_the_command_it_means() {
    let refused: Decision = Answer::Answered { status: 128, stderr: b"fatal: No paths with --include/--only does not make sense.\n".to_vec() }.decide();
    assert_eq!(
        refused,
        Decision::EndWithGit { status: 128, stderr: b"fatal: No paths with --include/--only does not make sense.\n".to_vec() }
    );
    assert_eq!(Answer::Answered { status: 0, stderr: Vec::new() }.decide(), Decision::Accept);
    let other: Decision = Answer::Answered { status: 1, stderr: b"error: pathspec 'x' did not match any file(s) known to git\n".to_vec() }.decide();
    assert_eq!(
        other,
        Decision::EndWithGit { status: 1, stderr: b"error: pathspec 'x' did not match any file(s) known to git\n".to_vec() },
        "an answer that is not a refusal is still Git's answer, with Git's status"
    );
    for why in [Unanswered::NotStarted, Unanswered::HeldPastItsBound, Unanswered::EndedWithoutAStatus] {
        assert_eq!(Answer::Unanswered(why).decide(), Decision::RefuseInOurWords(why));
    }
}

/// What:
///  The sentence used when Git gives no answer names the form,
///  says which of the three happened,
///  and says the words are the wrapper's.
/// Why:
///  A person whose Git will not start,
///  or whose Git was held past its bound,
///  must learn both what is refused and what became of the question,
///  and must not be told the words are Git's.
#[test]
fn the_wrappers_own_sentence_says_which_way_git_did_not_answer() {
    let not_started: String = unavailable_words("selection-conflict", Unanswered::NotStarted);
    assert!(not_started.contains("selection-conflict"), "the form: {not_started}");
    assert!(not_started.contains("could not be started"), "what happened: {not_started}");
    assert!(not_started.contains("the wrapper's"), "whose words: {not_started}");
    assert!(not_started.ends_with('\n'), "one line: {not_started}");
    let held: String = unavailable_words("selection-conflict", Unanswered::HeldPastItsBound);
    assert!(held.contains("showed no sign of work for the bound"), "what happened: {held}");
    assert!(!held.contains("could not be started"), "not a start failure: {held}");
    let ended: String = unavailable_words("selection-conflict", Unanswered::EndedWithoutAStatus);
    assert!(ended.contains("ended without an exit status"), "what happened: {ended}");
    assert_ne!(not_started, held, "each reason has its own sentence");
}

/// What:
///  The question about a command whose `--` is the value of `-m` is a dry run:
///  Git answers its own refusal and nothing in the repository changes.
/// Why:
///  This is the shape that once made a commit outside the service,
///  with the subject `--dry-run`,
///  in a repository the command never named.
#[test]
fn a_message_named_dashdash_is_asked_as_a_dry_run_and_changes_nothing() {
    let repo: PathBuf = a_repository_with_a_staged_change("commit-question-dashdash");
    std::fs::write(repo.join("-i"), "dash-i\n").expect("file");
    git(repo.as_path(), &["add", "--", "-i"]);
    let context: crate::transaction_git::GitContext = git_context();
    let before: (String, String, Vec<u8>, String) = (
        git_text(repo.as_path(), &["rev-parse", "HEAD"]),
        git_text(repo.as_path(), &["for-each-ref", "--format=%(refname) %(objectname)"]),
        std::fs::read(repo.join(".git/index")).expect("index"),
        git_text(repo.as_path(), &["rev-list", "--count", "HEAD"]),
    );
    let answer: Answer = ask_git(&context, repo.as_path(), tokens(&["-m", "--", "-i"]).as_slice());
    let Answer::Answered { status, stderr } = answer else {
        panic!("real Git answered: {answer:?}");
    };
    assert_eq!(status, 128, "Git refuses it: {}", String::from_utf8_lossy(stderr.as_slice()));
    assert!(
        String::from_utf8_lossy(stderr.as_slice()).starts_with("fatal: No paths with --include/--only does not make sense."),
        "Git's own words: {}",
        String::from_utf8_lossy(stderr.as_slice())
    );
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), before.0, "no commit");
    assert_eq!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname) %(objectname)"]), before.1, "no reference");
    assert_eq!(std::fs::read(repo.join(".git/index")).expect("index"), before.2, "the person's index, byte for byte");
    assert!(!repo.join(".git/index.lock").exists(), "no lock is left beside the person's index");
    assert_eq!(git_text(repo.as_path(), &["rev-list", "--count", "HEAD"]), before.3, "no commit was added");
}
