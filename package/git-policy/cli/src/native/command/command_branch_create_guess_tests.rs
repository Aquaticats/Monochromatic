//! What:
//!  Real Git 2.56.0 controls for branch creation by `git checkout` and `git switch`,
//!       explicit and by matching the single remote branch `origin/topic`.
//! Why:
//!  Which argument Git turns into a new local branch is decided deep inside
//!      `parse_branchname_arg`;
//!  here the binary creates or does not create,
//!  and the parser's
//!      answer is compared with what happened to the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await branchesAfter(['checkout', 'topic'])).toEqual(['topic']);
//! ```

/// The parser,
///  the command names and the real-Git fixture helpers.
use super::BranchCreationCommand::{Checkout, Switch};
use super::{BranchCreationCommand, BranchCreationRegion, parse_branch_creation_region};
use crate::command_test_support::{
    fixture, git, git_status, os_arguments, output_text, remove, repository,
};
use std::path::{Path, PathBuf};
use std::process::Output;

/// A repository on `main` whose only remote `origin` has the branches `topic` and `main`,
/// and an empty file `list`;
///  returns the fixture directory and the repository root.
fn guess_fixture(name: &str) -> (PathBuf, PathBuf) {
    let directory: PathBuf = fixture(name);
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(
        root.as_path(),
        &["remote", "add", "origin", "/nonexistent/origin"],
    );
    for remote_branch in ["refs/remotes/origin/topic", "refs/remotes/origin/main"] {
        git(root.as_path(), &["update-ref", remote_branch, "HEAD"]);
    }
    std::fs::write(root.join("list"), b"").expect("write empty pathspec file");
    return (directory, root);
}

/// Run `git <word> <line>`.
///  Report every branch other than `main` that exists afterwards,
/// and the unborn branch `HEAD` names after `--orphan`;
///  then return to `main` and delete them.
fn branches_after(root: &Path, word: &str, line: &str) -> Vec<String> {
    let mut full: Vec<&str> = vec![word];
    full.extend(line.split_whitespace());
    // Git's own verdict is read from the repository, so its exit status is not judged here.
    git_status(root, full.as_slice());
    let listing: String = output_text(&git(
        root,
        &["for-each-ref", "--format=%(refname)", "refs/heads"],
    ));
    let mut created: Vec<String> = Vec::<String>::new();
    for reference in listing.lines() {
        if reference != "refs/heads/main" {
            created.push(reference.trim_start_matches("refs/heads/").to_owned());
        }
    }
    let existing: Vec<String> = created.clone();
    let head: Output = git_status(root, &["symbolic-ref", "--short", "HEAD"]);
    let current: String = output_text(&head);
    if head.status.success() && current != "main" && !created.contains(&current) {
        created.push(current);
    }
    git(root, &["checkout", "--quiet", "--force", "main"]);
    for name in &existing {
        git(root, &["branch", "--quiet", "-D", name.as_str()]);
    }
    return created;
}

/// Whether the parser says this region creates a branch in the fixture:
///  explicitly,
///  or by
/// naming `topic`,
///  the only name the fixture's remote can supply.
fn predicted(command: BranchCreationCommand, line: &str) -> bool {
    let values: Vec<&str> = line.split_whitespace().collect();
    let parsed: BranchCreationRegion =
        parse_branch_creation_region(command, os_arguments(values.as_slice()).as_slice(), &[])
            .expect("valid region");
    if parsed.creates_branch {
        return true;
    }
    if let Some(index) = parsed.implicit_creation_target {
        return values[index] == "topic";
    }
    return false;
}

/// Require Git to create exactly the named branches and the parser to agree.
fn assert_cases(root: &Path, word: &str, command: BranchCreationCommand, cases: &[(&str, &str)]) {
    for (line, expected) in cases {
        let created: Vec<String> = branches_after(root, word, line);
        let wanted: Vec<&str> = expected.split_whitespace().collect();
        assert_eq!(created, wanted, "git {word} {line}");
        assert_eq!(
            predicted(command, line),
            !wanted.is_empty(),
            "git {word} {line}"
        );
    }
}

/// `git checkout`:
///  the guess,
///  the forms that stop it,
///  and explicit creation.
#[test]
fn checkout_creation_matches_git() {
    let (directory, root): (PathBuf, PathBuf) = guess_fixture("checkout-guess");
    assert_cases(
        root.as_path(),
        "checkout",
        Checkout,
        &[
            ("topic", "topic"),
            ("topic --", "topic"),
            ("-q topic", "topic"),
            ("-l topic", "topic"),
            ("--end-of-options topic", "topic"),
            ("--conflict merge topic", "topic"),
            ("--recurse-submodules topic", "topic"),
            ("--no-guess --guess topic", "topic"),
            ("--detach --no-detach topic", "topic"),
            ("--orphan new --no-orphan topic", "topic"),
            // Divergence: an empty pathspec file leaves Git switching branches.
            ("--pathspec-from-file=list topic", "topic"),
            // Divergence: `--no-track` also derives the new name from the argument.
            ("--no-track origin/topic", "topic"),
            ("--track origin/topic", "topic"),
            ("-t origin/topic", "topic"),
            ("--track=inherit origin/topic", "topic"),
            ("-b new", "new"),
            ("-bnew", "new"),
            ("-B new", "new"),
            ("-qb new origin/topic", "new"),
            ("--orphan new", "new"),
            ("", ""),
            ("main", ""),
            ("absent", ""),
            ("-", ""),
            ("-- topic", ""),
            ("topic extra", ""),
            ("topic -- extra", ""),
            ("top*", ""),
            ("--no-guess topic", ""),
            ("--guess --no-guess topic", ""),
            ("--detach topic", ""),
            ("-d topic", ""),
            ("--detach", ""),
            ("-p topic", ""),
            ("--ours topic", ""),
            ("-3 topic", ""),
            ("--overlay topic", ""),
            ("--no-overlay topic", ""),
        ],
    );
    remove(directory.as_path());
}

/// `git switch`:
///  one reference,
///  no paths,
///  `--` does not stop the guess.
#[test]
fn switch_creation_matches_git() {
    let (directory, root): (PathBuf, PathBuf) = guess_fixture("switch-guess");
    assert_cases(
        root.as_path(),
        "switch",
        Switch,
        &[
            ("topic", "topic"),
            ("-- topic", "topic"),
            ("--end-of-options topic", "topic"),
            ("-q topic", "topic"),
            ("--discard-changes topic", "topic"),
            ("--create new --no-create topic", "topic"),
            ("--no-track origin/topic", "topic"),
            ("-t origin/topic", "topic"),
            ("-c new", "new"),
            ("-cnew", "new"),
            ("--create=new", "new"),
            ("--cre new", "new"),
            ("-C new", "new"),
            ("--force-c new", "new"),
            ("--orphan new", "new"),
            ("", ""),
            ("main", ""),
            ("absent", ""),
            ("-", ""),
            ("topic extra", ""),
            ("--no-guess topic", ""),
            ("--detach topic", ""),
            ("-d topic", ""),
        ],
    );
    remove(directory.as_path());
}

/// The parser reads names and counts only:
///  where Git refuses for a value or a combination
/// of options,
///  it still answers "creates".
///  Each line is refused by the binary.
#[test]
fn refusals_git_makes_later_are_still_reported_as_creation() {
    let (directory, root): (PathBuf, PathBuf) = guess_fixture("guess-over-report");
    for (word, command, line) in [
        ("checkout", Checkout, "--track topic"),
        ("checkout", Checkout, "--track=bogus origin/topic"),
        ("checkout", Checkout, "-b new -B other"),
        ("checkout", Checkout, "-f -m topic"),
        ("checkout", Checkout, "--pathspec-file-nul topic"),
        ("switch", Switch, "--orphan new --track origin/topic"),
    ] {
        let created: Vec<String> = branches_after(root.as_path(), word, line);
        assert!(created.is_empty(), "git {word} {line}: {created:?}");
        assert!(predicted(command, line), "git {word} {line}");
    }
    remove(directory.as_path());
}
