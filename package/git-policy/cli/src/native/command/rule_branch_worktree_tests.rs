//! What:
//!  Branch-worktree decisions,
//!  the rejection text,
//!  and the one-query remote guess with
//!       real Git 2.56.0 controls.
//! Why:
//!  The guess decides whether `git switch <name>` is a harmless switch or a branch
//!      creation.
//!  Its listing is read line by line,
//!  so a prefix match or a miscounted remote
//!      would reject a switch to an existing branch or let a creation through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(remoteGuessCreatesBranch('refs/remotes/origin/topic\n', 'topic')).toBe(true);
//! ```

/// The decision,
///  its message,
///  the guess query and the real-Git fixture helpers.
use super::{
    BRANCH_CREATION_CODE, BranchWorktreeDecision, branch_creation_message, decide_branch_worktree,
    remote_guess_creates_branch, remote_guess_query_arguments,
};
use crate::command_branch_create::BranchCreationCommand;
use crate::command_test_support::{
    byte_argument, fixture, git, git_output, git_status, os_arguments, remove, repository,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Output;

/// Decide one invocation written as text.
fn decide(values: &[&str]) -> BranchWorktreeDecision {
    return decide_branch_worktree(os_arguments(values).as_slice());
}

/// Explicit creation is decided from the arguments,
///  behind global options too.
#[test]
fn explicit_creation_is_decided_from_arguments() {
    assert_eq!(BRANCH_CREATION_CODE, "branch-creation-requires-worktree");
    for (values, command) in [
        (vec!["branch", "topic"], BranchCreationCommand::Branch),
        (
            vec!["branch", "-c", "main", "topic"],
            BranchCreationCommand::Branch,
        ),
        (
            vec!["checkout", "-b", "topic"],
            BranchCreationCommand::Checkout,
        ),
        (
            vec!["checkout", "--orphan", "topic"],
            BranchCreationCommand::Checkout,
        ),
        (vec!["switch", "-c", "topic"], BranchCreationCommand::Switch),
        (
            vec!["switch", "--create", "topic"],
            BranchCreationCommand::Switch,
        ),
        (
            vec!["-C", "dir", "--no-pager", "switch", "-c", "topic"],
            BranchCreationCommand::Switch,
        ),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            BranchWorktreeDecision::Creates(command),
            "{values:?}"
        );
    }
}

/// A bare name after `switch` or `checkout` needs the repository's branches;
///  the name keeps its bytes.
#[test]
fn a_bare_name_needs_the_remote_guess() {
    assert_eq!(
        decide(&["switch", "topic"]),
        BranchWorktreeDecision::NeedsRemoteGuess {
            command: BranchCreationCommand::Switch,
            target: OsString::from("topic"),
        }
    );
    assert_eq!(
        decide(&["-C", "dir", "checkout", "-q", "feature/x"]),
        BranchWorktreeDecision::NeedsRemoteGuess {
            command: BranchCreationCommand::Checkout,
            target: OsString::from("feature/x"),
        }
    );
    let raw: OsString = byte_argument(b"t\xffpic");
    assert_eq!(
        decide_branch_worktree(&[OsString::from("switch"), raw.clone()]),
        BranchWorktreeDecision::NeedsRemoteGuess {
            command: BranchCreationCommand::Switch,
            target: raw,
        }
    );
}

/// Other commands,
///  forms that create nothing,
///  and regions Git refuses pass.
#[test]
fn everything_else_passes() {
    for values in [
        vec![],
        vec!["--version"],
        vec!["status"],
        vec!["worktree", "add", "-b", "topic", "../topic"],
        vec!["branch"],
        vec!["branch", "--list", "topic"],
        vec!["branch", "-d", "topic"],
        vec!["branch", "-m", "old", "new"],
        vec!["checkout", "--", "file"],
        vec!["checkout", "--detach", "topic"],
        vec!["switch", "--detach", "topic"],
        vec!["switch", "--no-guess", "topic"],
        vec!["switch", "--no-such-option", "-c", "topic"],
        vec!["checkout", "-b"],
        vec!["--no-such-global", "switch", "-c", "topic"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            BranchWorktreeDecision::Pass,
            "{values:?}"
        );
    }
}

/// The rejection names the command,
///  the guessed branch when known,
///  the worktree form and the bypass.
#[test]
fn rejection_text_is_the_incumbent_message() {
    assert_eq!(
        branch_creation_message(BranchCreationCommand::Switch, None),
        "cli-git: git switch branch creation is rejected in the current worktree. Use `git \
         worktree add -b <branch> <path> [<start-point>]` so new branch work starts in its own \
         checkout, or pass --no-enforce-worktree-branch to bypass for this invocation."
    );
    assert_eq!(
        branch_creation_message(BranchCreationCommand::Checkout, Some(OsStr::new("topic"))),
        "cli-git: git checkout for topic branch creation is rejected in the current worktree. \
         Use `git worktree add -b <branch> <path> [<start-point>]` so new branch work starts in \
         its own checkout, or pass --no-enforce-worktree-branch to bypass for this invocation."
    );
    assert!(
        branch_creation_message(BranchCreationCommand::Branch, None)
            .starts_with("cli-git: git branch branch creation is rejected"),
    );
    assert!(
        branch_creation_message(
            BranchCreationCommand::Switch,
            Some(byte_argument(b"t\xffp").as_os_str())
        )
        .starts_with("cli-git: git switch for t\u{fffd}p branch creation"),
    );
}

/// The guess is one listing of the local branch and the remote branches of that name,
///  after the global options.
#[test]
fn the_guess_query_lists_local_and_remote_names_once() {
    assert_eq!(
        remote_guess_query_arguments(
            os_arguments(&["-C", "dir"]).as_slice(),
            OsStr::new("feature/x")
        ),
        os_arguments(&[
            "-C",
            "dir",
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads/feature/x",
            "refs/remotes/*/feature/x",
        ])
    );
    let raw: OsString = byte_argument(b"t\xff");
    let arguments: Vec<OsString> = remote_guess_query_arguments(&[], raw.as_os_str());
    assert_eq!(arguments[2], byte_argument(b"refs/heads/t\xff"));
    assert_eq!(arguments[3], byte_argument(b"refs/remotes/*/t\xff"));
}

/// Exactly one remote branch of that exact name and no local one means Git creates the branch.
#[test]
fn the_listing_is_compared_line_by_line() {
    for (stdout, target, creates) in [
        (
            b"refs/remotes/origin/topic\n".as_slice(),
            b"topic".as_slice(),
            true,
        ),
        (b"refs/remotes/origin/feature/x\n", b"feature/x", true),
        (b"", b"topic", false),
        (b"\n", b"topic", false),
        // A local branch of that name: Git switches to it.
        (
            b"refs/heads/topic\nrefs/remotes/origin/topic\n",
            b"topic",
            false,
        ),
        (
            b"refs/remotes/origin/topic\nrefs/heads/topic\n",
            b"topic",
            false,
        ),
        // Two remotes have it: Git refuses to guess.
        (
            b"refs/remotes/origin/topic\nrefs/remotes/fork/topic\n",
            b"topic",
            false,
        ),
        // Refs that only start with the name, or sit one level deeper, are other branches.
        (
            b"refs/heads/topic/sub\nrefs/remotes/origin/topic\n",
            b"topic",
            true,
        ),
        (b"refs/remotes/origin/topic/sub\n", b"topic", false),
        (b"refs/remotes/origin/nested/topic\n", b"topic", false),
        (b"refs/remotes/topic\n", b"topic", false),
        (b"refs/tags/topic\n", b"topic", false),
        (b"refs/remotes/origin/topic", b"topic", true),
        (b"refs/remotes/origin/topics\n", b"topic", false),
        // A remote's HEAD pointer is never a branch to guess.
        (b"refs/remotes/origin/HEAD\n", b"HEAD", false),
        (b"refs/remotes/origin/t\xffpic\n", b"t\xffpic", true),
    ] {
        assert_eq!(
            remote_guess_creates_branch(stdout, target),
            creates,
            "{:?} for {:?}",
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(target)
        );
    }
}

/// Run the guess query through real Git and read its listing.
fn guessed(root: &Path, target: &str) -> bool {
    let listing: Output = git_output(
        root,
        remote_guess_query_arguments(&[], OsStr::new(target)).as_slice(),
    );
    assert!(listing.status.success(), "{target}");
    return remote_guess_creates_branch(listing.stdout.as_slice(), target.as_bytes());
}

/// Whether a local branch of that name exists after `git switch <name>` in real Git;
///  the branch is removed again.
fn switch_created(root: &Path, target: &str) -> bool {
    let before: bool = git_status(
        root,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            format!("refs/heads/{target}").as_str(),
        ],
    )
    .status
    .success();
    git_status(root, &["switch", "--quiet", target]);
    let after: bool = git_status(
        root,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            format!("refs/heads/{target}").as_str(),
        ],
    )
    .status
    .success();
    git(root, &["checkout", "--quiet", "--force", "main"]);
    if after && !before {
        git(root, &["branch", "--quiet", "-D", target]);
    }
    return after && !before;
}

/// The one-query guess agrees with what real Git 2.56.0 does for `git switch <name>`.
#[test]
fn the_guess_matches_what_real_git_creates() {
    let directory: PathBuf = fixture("branch-worktree-guess");
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(
        root.as_path(),
        &["remote", "add", "origin", "/nonexistent/origin"],
    );
    git(
        root.as_path(),
        &["remote", "add", "fork", "/nonexistent/fork"],
    );
    for reference in [
        "refs/remotes/origin/topic",
        "refs/remotes/origin/feature/x",
        "refs/remotes/origin/both",
        "refs/remotes/fork/both",
        "refs/remotes/origin/local",
        "refs/remotes/origin/deep/er",
        "refs/heads/local",
    ] {
        git(root.as_path(), &["update-ref", reference, "HEAD"]);
    }
    for (target, creates) in [
        ("topic", true),
        ("feature/x", true),
        ("both", false),
        ("local", false),
        ("absent", false),
        ("deep", false),
        ("er", false),
        ("main", false),
    ] {
        assert_eq!(
            guessed(root.as_path(), target),
            creates,
            "guess for {target}"
        );
        assert_eq!(
            switch_created(root.as_path(), target),
            creates,
            "git switch {target}"
        );
    }
    remove(directory.as_path());
}
