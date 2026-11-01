//! What:
//!  Real Git 2.56.0 controls for the `git commit` table and for each reading where the
//!       incumbent parser disagreed with Git.
//! Why:
//!  A divergence from the incumbent is only justified by what the binary does;
//!  each
//!      control runs the command in a disposable repository and checks the parser's facts
//!      against the observed effect.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await git(repo, ['commit', '-qam', 'msg']); expect(await git(repo, ['status', '--porcelain'])).toBe('');
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{CommitRegion, FixupKind, parse_commit_region};
use crate::command_commit_table::COMMIT_TABLE;
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_status, os_arguments, output_text, remove,
    repository_with_tracked_file,
};
use std::path::{Path, PathBuf};
use std::process::Output;

/// Number of commits reachable from `HEAD`.
fn commit_count(root: &Path) -> String {
    return output_text(&git(root, &["rev-list", "--count", "HEAD"]));
}

/// Subject line of `HEAD`.
fn head_subject(root: &Path) -> String {
    return output_text(&git(root, &["log", "--max-count=1", "--format=%s"]));
}

/// Parse a region with no other wrapper flags.
fn region(values: &[&str]) -> CommitRegion {
    return parse_commit_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// The copied table lists exactly the long options,
///  required values and negations the
/// binary reports.
#[test]
fn table_matches_git_commit_completion_helper() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-table");
    assert_table_invariants(COMMIT_TABLE);
    assert_eq!(
        render_completion(COMMIT_TABLE),
        git_completion(root.as_path(), &["commit"])
    );
    remove(directory.as_path());
}

/// Divergence:
///  `-qam` contains `-a`.
///  The incumbent only split clusters that it recognized.
#[test]
fn clustered_all_stages_tracked_changes_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-cluster");
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
    git(root.as_path(), &["commit", "-qam", "clustered"]);
    let status: Output = git(root.as_path(), &["status", "--porcelain"]);
    assert_eq!(String::from_utf8_lossy(&status.stdout), "");
    assert_eq!(head_subject(root.as_path()), "clustered");
    assert!(region(&["-qam", "clustered"]).all);
    remove(directory.as_path());
}

/// Divergence:
///  after `-u`,
///  the letter `a` is the untracked-files mode,
///  not `--all`.
#[test]
fn letters_after_an_optional_value_letter_are_its_value_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-optarg");
    let output: Output = git_status(root.as_path(), &["commit", "-ua", "-m", "x", "tracked.txt"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Invalid untracked files mode 'a'"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!region(&["-ua", "-m", "x", "tracked.txt"]).all);
    remove(directory.as_path());
}

/// Divergence:
///  a separated token after `--untracked-files` is a pathspec.
#[test]
fn a_separated_token_after_an_optional_value_option_is_a_pathspec_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-optlong");
    let output: Output = git_status(
        root.as_path(),
        &["commit", "--untracked-files", "no", "-m", "x"],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("pathspec 'no' did not match"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        region(&["--untracked-files", "no", "-m", "x"]).pathspecs,
        vec![1]
    );
    remove(directory.as_path());
}

/// Divergence:
///  the last of `--dry-run`/`--no-dry-run` and of the status formats wins.
#[test]
fn the_last_dry_run_choice_wins_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-dry-run");
    let before: String = commit_count(root.as_path());
    let cancelled: &[&str] = &["--allow-empty", "--dry-run", "--no-dry-run", "-m", "real"];
    let mut command: Vec<&str> = vec!["commit"];
    command.extend_from_slice(cancelled);
    git(root.as_path(), command.as_slice());
    let after: String = commit_count(root.as_path());
    assert_ne!(before, after);
    assert!(!region(cancelled).dry_run);
    let implied: &[&str] = &["--allow-empty", "--no-short", "--long", "-m", "dry"];
    let mut dry_command: Vec<&str> = vec!["commit"];
    dry_command.extend_from_slice(implied);
    git_status(root.as_path(), dry_command.as_slice());
    assert_eq!(commit_count(root.as_path()), after);
    assert!(region(implied).dry_run);
    remove(directory.as_path());
}

/// `--fixup=reword:` turns on `--only` itself and refuses an explicit `-o` and paths;
/// `--fixup=amend:` permits a pathless `-o` (builtin/commit.c:1296-1307,
///  1395-1402).
#[test]
fn fixup_suboptions_behave_as_classified_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-fixup");
    git(root.as_path(), &["commit", "--fixup=reword:HEAD"]);
    assert_eq!(head_subject(root.as_path()), "amend! base");
    assert_eq!(
        region(&["--fixup=reword:HEAD"]).fixup,
        Some(FixupKind::Reword)
    );
    for arguments in [
        vec!["commit", "-o", "--fixup=reword:HEAD"],
        vec!["commit", "--fixup=reword:HEAD", "tracked.txt"],
    ] {
        let refused: Output = git_status(root.as_path(), arguments.as_slice());
        assert!(!refused.status.success(), "{arguments:?}");
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("cannot be used together"),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&refused.stderr)
        );
    }
    let before: String = commit_count(root.as_path());
    git(root.as_path(), &["commit", "-o", "--fixup=amend:HEAD"]);
    assert_ne!(commit_count(root.as_path()), before);
    let unknown: Output = git_status(root.as_path(), &["commit", "--fixup=squash:HEAD"]);
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("unknown option: --fixup=squash:HEAD")
    );
    assert_eq!(
        region(&["--fixup=squash:HEAD"]).fixup,
        Some(FixupKind::UnknownSuboption)
    );
    remove(directory.as_path());
}

/// Git refuses a single-dash long spelling and accepts a unique abbreviation.
#[test]
fn typo_refusal_and_abbreviation_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("commit-spelling");
    let typo: Output = git_status(root.as_path(), &["commit", "-all", "-m", "x"]);
    assert_eq!(typo.status.code(), Some(129));
    assert!(String::from_utf8_lossy(&typo.stderr).contains("did you mean `--all`"));
    assert_eq!(
        parse_commit_region(os_arguments(&["-all", "-m", "x"]).as_slice(), &[])
            .expect_err("refused")
            .kind,
        OptionErrorKind::SingleDashLongOption
    );
    let before: String = commit_count(root.as_path());
    git(root.as_path(), &["commit", "--am", "-m", "amended"]);
    assert_eq!(commit_count(root.as_path()), before);
    assert_eq!(head_subject(root.as_path()), "amended");
    assert!(region(&["--am", "-m", "amended"]).amend);
    let ambiguous: Output = git_status(root.as_path(), &["commit", "--in", "-m", "x"]);
    assert_eq!(ambiguous.status.code(), Some(129));
    assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("ambiguous option: in"));
    remove(directory.as_path());
}
