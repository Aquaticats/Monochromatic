//! What:
//!  Every spelling of the ignored-files transform,
//!  and a real-Git control of the token it adds.
//! Why:
//!  The token must stop Git replacing an ignored file without changing what a person can type
//!       when nothing would be lost,
//!  and the caller's own answer must survive.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(keepIgnoredFiles(['checkout', 'other'])).toEqual(['checkout', '--no-overwrite-ignore', 'other']);
//! ```

/// The decision under test,
///  its result type and the real-Git fixture helpers.
use super::{NO_OVERWRITE_IGNORE, keep_ignored_files};
use crate::command_test_support::{fixture, git, git_output, os_arguments, remove, repository};
use crate::rule_argument_rewrite::ArgumentRewrite;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

/// Decide for text arguments.
fn decide(values: &[&str]) -> ArgumentRewrite {
    return keep_ignored_files(os_arguments(values).as_slice());
}

/// The arguments the transform forwards,
///  or the caller's own when it changes nothing.
fn forwarded(values: &[&str]) -> Vec<OsString> {
    return match decide(values) {
        ArgumentRewrite::Rewritten(rewritten) => rewritten,
        ArgumentRewrite::Unchanged => os_arguments(values),
    };
}

/// A `HEAD` move and a forced switch get the token directly after the subcommand;
/// the caller's global options stay where they are.
#[test]
fn a_switch_gets_the_token_directly_after_the_subcommand() {
    assert_eq!(NO_OVERWRITE_IGNORE, &["--no-overwrite-ignore"]);
    for values in [
        vec!["checkout", "other"],
        vec!["checkout", "--quiet", "other"],
        vec!["checkout", "-f", "other"],
        vec!["checkout", "--", "a.txt"],
        vec!["checkout", "--ours", "--", "a.txt"],
        vec!["switch", "other"],
        vec!["switch", "--discard-changes", "other"],
        vec!["switch", "--detach", "HEAD~1"],
    ] {
        let (word, region): (&str, &[&str]) = (values[0], &values[1..]);
        let mut expected: Vec<String> = vec![String::from(word), String::from("--no-overwrite-ignore")];
        for token in region {
            expected.push(String::from(*token));
        }
        let spelled: Vec<&str> = expected.iter().map(String::as_str).collect();
        assert_eq!(forwarded(values.as_slice()), os_arguments(&spelled), "{values:?}");
    }
    // The global options come first, and the token still follows the subcommand.
    assert_eq!(
        forwarded(&["-C", "dir", "--no-pager", "checkout", "other"]),
        os_arguments(&[
            "-C",
            "dir",
            "--no-pager",
            "checkout",
            "--no-overwrite-ignore",
            "other"
        ])
    );
}

/// Nothing is added where Git reaches no ignored file,
///  where the caller already answered,
///  or where Git refuses the region.
#[test]
fn nothing_is_added_where_no_ignored_file_is_at_stake() {
    for values in [
        // Another command entirely.
        vec!["status"],
        vec!["restore", "--", "a.txt"],
        vec!["commit", "--message", "message", "a.txt"],
        // A checkout that changes no file content: no target, or a new branch from `HEAD`.
        vec!["checkout"],
        vec!["checkout", "-b", "topic"],
        vec!["switch", "-c", "topic"],
        // The caller already answered the question, in either direction.
        vec!["checkout", "--no-overwrite-ignore", "other"],
        vec!["checkout", "--overwrite-ignore", "other"],
        vec!["switch", "--no-overwrite-ignore", "--detach", "HEAD"],
        // Git refuses the region, so it runs nothing.
        vec!["checkout", "--no-such-option", "other"],
        vec!["switch", "--no-such-option", "other"],
        // No subcommand at all.
        vec![],
        vec!["--version"],
        vec!["--no-such-global", "checkout", "other"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            ArgumentRewrite::Unchanged,
            "{values:?}"
        );
    }
}

/// Real Git:
///  the added token makes a switch refuse to replace an ignored file,
/// and the same switch runs where the target tree does not carry it.
#[test]
fn real_git_refuses_to_replace_an_ignored_file_and_switches_when_nothing_is_lost() {
    let root: PathBuf = fixture("keep-ignored-files");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b".env\n").expect("ignore file");
    git(repo.as_path(), &["add", "--", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    // A branch whose tree carries the ignored path, and one whose tree does not.
    git(repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(repo.as_path(), &["add", "--force", "--", ".env"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "free", "main"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join(".env"), b"SECRET=the person value\n").expect("ignored file");
    for word in ["checkout", "switch"] {
        let refused: Output = git_output(
            repo.as_path(),
            os_arguments(&[word, "--no-overwrite-ignore", "tracks"]).as_slice(),
        );
        assert!(!refused.status.success(), "{word} tracks replaced the ignored file");
        assert_eq!(
            std::fs::read(repo.join(".env")).expect("ignored file"),
            b"SECRET=the person value\n",
            "{word} tracks"
        );
        // The same token, with a target whose tree carries no such path.
        let allowed: Output = git_output(
            repo.as_path(),
            os_arguments(&[word, "--no-overwrite-ignore", "free"]).as_slice(),
        );
        assert!(allowed.status.success(), "{word} free: {}", String::from_utf8_lossy(allowed.stderr.as_slice()));
        git(repo.as_path(), &["checkout", "--quiet", "main"]);
    }
    remove(root.as_path());
}
