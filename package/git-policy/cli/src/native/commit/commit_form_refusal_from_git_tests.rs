//! What:
//!  The words a commit form's refusal carries:
//!  where the question's `--dry-run` goes,
//!  that a form Git accepts answers with nothing,
//!  and that a form Git refuses answers with Git's own status and message.
//! Why:
//!  The wrapper promises Git's own wording,
//!  so the question is answered by the installed Git here,
//!  never by a message this program keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = await askGit(context, repo, ['--include', '--only', '--message=x', '--', 'a.txt']);
//! expect(refusal.code).toBe(128);
//! ```

use super::{Refusal, ask_git, unavailable_words, with_dry_run};
use crate::testing::test_support::{fixture, git, git_context, repository};
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
///  `--dry-run` is added after the person's options,
///  and before the separator that ends them.
/// Why:
///   A `--dry-run` after `--` is a path to Git,
/// which would make the question ask about a file called
///  `--dry-run` instead of about the form.
#[test]
fn a_dry_run_goes_before_the_separator_and_a_path_stays_a_path() {
    let asked: Vec<String> = text(with_dry_run(tokens(&["--message=x", "--", "a.txt"]).as_slice()).as_slice());
    assert_eq!(asked, ["--message=x", "--dry-run", "--", "a.txt"]);
    let plain: Vec<String> = text(with_dry_run(tokens(&["--message=x"]).as_slice()).as_slice());
    assert_eq!(plain, ["--message=x", "--dry-run"]);
}

/// What:
///  A command line with no separator keeps every token where the person wrote it.
/// Why:
///   The question must be the person's own command line,
///  or Git would answer about another one.
#[test]
fn a_region_without_a_separator_keeps_its_order_and_gains_the_option_at_the_end() {
    let asked: Vec<String> = text(with_dry_run(tokens(&["--include", "--only", "--message=x", "a.txt"]).as_slice()).as_slice());
    assert_eq!(asked, ["--include", "--only", "--message=x", "a.txt", "--dry-run"]);
}

/// What:
///  A form Git refuses comes back with Git's status and Git's own sentence,
///  and a form Git accepts comes back with nothing.
/// Why:
///   This is the whole promise of the refusal:
///  the words are the installed Git's,
///  and a command that is fine is never refused.
#[test]
fn git_s_own_answer_is_what_comes_back_and_an_accepted_form_answers_nothing() {
    let repo: PathBuf = a_repository_with_a_staged_change("commit-form-refusal-asked");
    let context: crate::transaction_git::GitContext = git_context();
    let commits_before: String = crate::testing::test_support::git_text(repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let refused: Option<Refusal> = ask_git(
        &context,
        repo.as_path(),
        tokens(&["--include", "--only", "--message=x", "--", "a.txt"]).as_slice(),
    );
    let refusal: Refusal = refused.expect("Git refuses -i with -o");
    assert_eq!(refusal.code, 128);
    let words: String = String::from_utf8_lossy(refusal.stderr.as_slice()).into_owned();
    assert!(words.contains("cannot be used together"), "Git's own words: {words}");
    // The question left the repository as it was: the change is still staged, `HEAD` never moved, and no commit appeared.
    assert!(git(repo.as_path(), &["diff", "--cached", "--name-only"]).status.success());
    assert_eq!(
        crate::testing::test_support::git_text(repo.as_path(), &["rev-list", "--count", "HEAD"]),
        commits_before,
        "the question made no commit"
    );
    let accepted: Option<Refusal> = ask_git(&context, repo.as_path(), tokens(&["--message=x"]).as_slice());
    assert_eq!(accepted, None, "Git accepts a plain message");
}

/// What:
///  The sentence used when Git cannot be asked names the form and says whose words these are.
/// Why:
///   A person whose Git will not start must still learn which form is refused,
///  and must not be told the words are Git's when they are the wrapper's.
#[test]
fn the_wrapper_s_own_sentence_names_the_form_and_whose_words_they_are() {
    let words: String = unavailable_words("selection-conflict");
    assert!(words.contains("selection-conflict"), "the form: {words}");
    assert!(words.contains("Real Git could not be started"), "why: {words}");
    assert!(words.contains("the wrapper's"), "whose words: {words}");
    assert!(words.ends_with('\n'), "one line: {words}");
}
