//! What:
//!  The real-Git facts provider against disposable repositories,
//!  with its query count.
//! Why:
//!  Every rule that needs a repository fact trusts these answers,
//!  and the count is the
//!      evidence that one invocation asks Git once for the location and never for a fact
//!      no rule requested.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const facts = gitFacts(git, ['-C', nested], []); expect((await facts.location()).prefix).toEqual(Buffer.from('nested/'));
//! ```
#![cfg(unix)]

/// The provider under test,
///  the fact types and the real-Git fixture helpers.
use super::{GitFacts, RepositoryFacts, git_facts, head_file_exists};
use crate::command_test_support::{
    git, git_status, repository, repository_with_tracked_file, start_conflicted_merge,
};
use crate::repository_location::RepositoryLocation;
use crate::rule_commit_index::IndexVsHead;
use crate::rule_commit_sequencer::SequencerState;
use crate::test_support::{REAL_GIT, executable, fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// A provider for real Git after `-C <directory>` for each given directory.
fn facts_in(directories: &[&Path]) -> GitFacts {
    let mut prefix: Vec<OsString> = Vec::<OsString>::new();
    for directory in directories {
        prefix.push(OsString::from("-C"));
        prefix.push(directory.as_os_str().to_os_string());
    }
    return git_facts(Path::new(REAL_GIT), prefix.as_slice(), &[]);
}

/// The location real Git reports after `-C <directory>` for each given directory.
fn located(directories: &[&Path]) -> RepositoryLocation {
    return facts_in(directories).location().expect("location query");
}

/// Real Git reports the prefix for the top level,
///  a nested directory,
///  a chained `-C`,
///  a link and a linked worktree.
#[test]
fn location_is_what_real_git_reports() {
    let root: PathBuf = fixture("facts-location");
    let main: PathBuf = repository(root.as_path(), "main");
    let nested: PathBuf = main.join("nested/deep \u{e9}");
    std::fs::create_dir_all(&nested).expect("nested directory");
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: main.join(".git"),
        git_dir: main.join(".git"),
        worktree_root: main.clone(),
    };
    assert_eq!(
        located(&[main.as_path()]),
        RepositoryLocation {
            identity: identity.clone(),
            prefix: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        located(&[nested.as_path()]),
        RepositoryLocation {
            identity: identity.clone(),
            prefix: "nested/deep \u{e9}/".as_bytes().to_vec(),
        }
    );
    // Git chains `-C` itself: down into a directory and back up to the top level.
    assert_eq!(
        located(&[main.as_path(), Path::new("nested"), Path::new("..")]).prefix,
        Vec::<u8>::new()
    );
    assert_eq!(
        located(&[main.as_path(), Path::new("nested")]).prefix,
        b"nested/".to_vec()
    );
    // Through a symbolic link Git still reports the path below the real top level.
    std::os::unix::fs::symlink(main.join("nested"), root.join("link")).expect("link");
    assert_eq!(
        located(&[root.join("link").as_path()]),
        RepositoryLocation {
            identity,
            prefix: b"nested/".to_vec(),
        }
    );
    let linked: PathBuf = root.join("linked");
    git(
        main.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            linked.to_str().expect("fixture path is UTF-8"),
        ],
    );
    std::fs::create_dir(linked.join("inner")).expect("inner directory");
    assert_eq!(
        located(&[linked.join("inner").as_path()]),
        RepositoryLocation {
            identity: WorktreeIdentity::LinkedWorktree {
                common_dir: main.join(".git"),
                git_dir: main.join(".git/worktrees/linked"),
                worktree_root: linked,
            },
            prefix: b"inner/".to_vec(),
        }
    );
    remove(root.as_path());
}

/// Outside a repository,
///  in a bare repository and inside `.git` there is no worktree and an empty prefix.
#[test]
fn location_without_a_worktree_has_an_empty_prefix() {
    let root: PathBuf = fixture("facts-absent");
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(
        located(&[plain.as_path()]),
        RepositoryLocation {
            identity: WorktreeIdentity::OutsideWorktree,
            prefix: Vec::<u8>::new(),
        }
    );
    git(root.as_path(), &["init", "--quiet", "--bare", "bare.git"]);
    let bare: PathBuf = root.join("bare.git");
    assert_eq!(
        located(&[bare.as_path()]),
        RepositoryLocation {
            identity: WorktreeIdentity::BareRepository {
                common_dir: bare.clone(),
                git_dir: bare,
            },
            prefix: Vec::<u8>::new(),
        }
    );
    let main: PathBuf = repository(root.as_path(), "main");
    assert_eq!(
        located(&[main.join(".git").as_path()]),
        RepositoryLocation {
            identity: WorktreeIdentity::OutsideWorktree,
            prefix: Vec::<u8>::new(),
        }
    );
    remove(root.as_path());
}

/// A provider starts no process until asked,
///  asks for the location once,
///  and counts every other query.
#[test]
fn queries_are_lazy_and_the_location_is_asked_once() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("facts-count");
    let mut facts: GitFacts = facts_in(&[root.as_path()]);
    assert_eq!(facts.queries, 0);
    let first: RepositoryLocation = facts.location().expect("first answer");
    assert_eq!(facts.queries, 1);
    assert_eq!(facts.location().expect("remembered answer"), first);
    assert_eq!(facts.location().expect("remembered answer"), first);
    assert_eq!(facts.queries, 1);
    assert_eq!(facts.index_vs_head(), Ok(IndexVsHead::Matches));
    assert_eq!(facts.queries, 2);
    assert_eq!(facts.sequencer_state(), Ok(SequencerState::NotInProgress));
    assert_eq!(facts.queries, 3);
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("topic")),
        Ok(false)
    );
    assert_eq!(facts.queries, 4);
    assert_eq!(facts.location().expect("still remembered"), first);
    assert_eq!(facts.queries, 4);
    // A failed location answer is remembered too: the provider does not ask again.
    let mut missing: GitFacts = git_facts(directory.join("no-git").as_path(), &[], &[]);
    let failure: String = missing.location().expect_err("missing executable");
    assert_eq!(missing.location(), Err(failure));
    assert_eq!(missing.queries, 1);
    remove(directory.as_path());
}

/// The index state follows `git diff-index --quiet --cached HEAD`:
///  equal,
///  different,
///  or unanswerable.
#[test]
fn index_state_is_what_real_git_reports() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("facts-index");
    assert_eq!(
        facts_in(&[root.as_path()]).index_vs_head(),
        Ok(IndexVsHead::Matches)
    );
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("edit");
    // A worktree edit that is not staged leaves the index equal to `HEAD`.
    assert_eq!(
        facts_in(&[root.as_path()]).index_vs_head(),
        Ok(IndexVsHead::Matches)
    );
    git(root.as_path(), &["add", "--", "tracked.txt"]);
    assert_eq!(
        facts_in(&[root.as_path()]).index_vs_head(),
        Ok(IndexVsHead::Differs)
    );
    let unborn: PathBuf = directory.join("unborn");
    std::fs::create_dir(&unborn).expect("unborn directory");
    git(unborn.as_path(), &["init", "--quiet"]);
    assert_eq!(
        facts_in(&[unborn.as_path()]).index_vs_head(),
        Ok(IndexVsHead::Unknown)
    );
    let plain: PathBuf = directory.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(
        facts_in(&[plain.as_path()]).index_vs_head(),
        Ok(IndexVsHead::Unknown)
    );
    remove(directory.as_path());
}

/// A conflicted merge,
///  a stopped cherry-pick and a stopped revert each await their commit;
///  a clean repository does not.
#[test]
fn sequencer_state_is_what_the_head_files_say() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("facts-sequencer");
    assert_eq!(
        facts_in(&[root.as_path()]).sequencer_state(),
        Ok(SequencerState::NotInProgress)
    );
    start_conflicted_merge(root.as_path());
    assert!(root.join(".git/MERGE_HEAD").exists());
    assert_eq!(
        facts_in(&[root.as_path()]).sequencer_state(),
        Ok(SequencerState::InProgress)
    );
    git(root.as_path(), &["merge", "--abort"]);
    assert_eq!(
        facts_in(&[root.as_path()]).sequencer_state(),
        Ok(SequencerState::NotInProgress)
    );
    // Each head file alone counts, whatever it holds.
    for name in ["CHERRY_PICK_HEAD", "REVERT_HEAD"] {
        let head: PathBuf = root.join(".git").join(name);
        std::fs::write(&head, b"").expect("head file");
        assert_eq!(
            facts_in(&[root.as_path()]).sequencer_state(),
            Ok(SequencerState::InProgress),
            "{name}"
        );
        std::fs::remove_file(&head).expect("remove head file");
    }
    assert_eq!(
        facts_in(&[root.as_path()]).sequencer_state(),
        Ok(SequencerState::NotInProgress)
    );
    // Git prints the target of a linked head file, not the name asked for, so the answer
    // is refused instead of being read as "nothing in progress".
    let linked_head: PathBuf = root.join(".git/CHERRY_PICK_HEAD");
    std::os::unix::fs::symlink("nowhere", &linked_head).expect("linked head file");
    assert_eq!(
        facts_in(&[root.as_path()]).sequencer_state(),
        Err(String::from(
            "git rev-parse --git-path did not print the three sequencer head paths of one Git directory"
        ))
    );
    std::fs::remove_file(&linked_head).expect("remove linked head file");
    // Outside a repository the path query fails and normal enforcement applies.
    let plain: PathBuf = directory.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(
        facts_in(&[plain.as_path()]).sequencer_state(),
        Ok(SequencerState::NotInProgress)
    );
    remove(directory.as_path());
}

/// The remote guess is answered from one listing of the selected repository.
#[test]
fn remote_guess_is_what_the_refs_say() {
    let directory: PathBuf = fixture("facts-guess");
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(
        root.as_path(),
        &["remote", "add", "origin", "/nonexistent/origin"],
    );
    for reference in [
        "refs/remotes/origin/topic",
        "refs/remotes/origin/local",
        "refs/heads/local",
    ] {
        git(root.as_path(), &["update-ref", reference, "HEAD"]);
    }
    let mut facts: GitFacts = facts_in(&[root.as_path()]);
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("topic")),
        Ok(true)
    );
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("local")),
        Ok(false)
    );
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("absent")),
        Ok(false)
    );
    assert_eq!(facts.queries, 3);
    // Outside a repository the listing fails: nothing can be created.
    let plain: PathBuf = directory.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(
        facts_in(&[plain.as_path()]).remote_guess_creates_branch(OsStr::new("topic")),
        Ok(false)
    );
    // Asking changed nothing: the guessed branch still does not exist.
    assert!(
        !git_status(
            root.as_path(),
            &["show-ref", "--verify", "--quiet", "refs/heads/topic"]
        )
        .status
        .success()
    );
    remove(directory.as_path());
}

/// Every query is one process:
///  the caller's global options first,
///  the overlay in its environment.
#[test]
fn each_fact_is_one_process_with_the_global_prefix_and_overlay() {
    let root: PathBuf = fixture("facts-probe");
    let probe: PathBuf = root.join("probe");
    let log: PathBuf = root.join("log");
    executable(
        probe.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s|%s\\n' \"$MARKER\" \"$*\" >> '{}'\nexit 1\n",
            log.display()
        )
        .as_bytes(),
    );
    let mut facts: GitFacts = git_facts(
        probe.as_path(),
        &[OsString::from("-C"), OsString::from("somewhere")],
        &[(OsString::from("MARKER"), OsString::from("overlay"))],
    );
    // The probe exits 1 with no output: outside a worktree, a dirty index, no operation, no guess.
    assert_eq!(
        facts.location(),
        Ok(RepositoryLocation {
            identity: WorktreeIdentity::OutsideWorktree,
            prefix: Vec::<u8>::new(),
        })
    );
    assert_eq!(facts.index_vs_head(), Ok(IndexVsHead::Differs));
    assert_eq!(facts.sequencer_state(), Ok(SequencerState::NotInProgress));
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("topic")),
        Ok(false)
    );
    assert_eq!(facts.queries, 4);
    assert_eq!(
        std::fs::read_to_string(&log).expect("probe log"),
        "overlay|-C somewhere rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir --show-toplevel --show-prefix\n\
         overlay|-C somewhere diff-index --quiet --cached HEAD --\n\
         overlay|-C somewhere rev-parse --path-format=absolute --git-path MERGE_HEAD --git-path CHERRY_PICK_HEAD --git-path REVERT_HEAD\n\
         overlay|-C somewhere for-each-ref --format=%(refname) refs/heads/topic refs/remotes/*/topic\n"
    );
    remove(root.as_path());
}

/// A Git that cannot be started is a failure naming the executable and the fact,
///  for every fact.
#[test]
fn an_unstartable_git_fails_every_fact() {
    let root: PathBuf = fixture("facts-missing");
    let missing: PathBuf = root.join("no-git");
    let mut facts: GitFacts = git_facts(missing.as_path(), &[], &[]);
    let prefix: String = format!("cli-git could not start {} to read ", missing.display());
    // `.err()` keeps only the failure text of each answer.
    let failures: [(Option<String>, &str); 4] = [
        (facts.location().err(), "the repository location"),
        (facts.index_vs_head().err(), "the index state"),
        (facts.sequencer_state().err(), "the Git directory"),
        (
            facts.remote_guess_creates_branch(OsStr::new("topic")).err(),
            "branch names",
        ),
    ];
    for (failure, fact) in failures {
        let message: String = failure.expect(fact);
        assert!(
            message.starts_with(format!("{prefix}{fact}: ").as_str()),
            "{message}"
        );
    }
    assert_eq!(facts.queries, 4);
    remove(root.as_path());
}

/// Output the sequencer query never produces is a failure,
///  not a guess.
#[test]
fn uninterpretable_answers_are_failures() {
    let root: PathBuf = fixture("facts-garbled");
    let probe: PathBuf = root.join("probe");
    executable(
        probe.as_path(),
        b"#!/bin/sh\nprintf 'unexpected\\n'\nexit 0\n",
    );
    let mut facts: GitFacts = git_facts(probe.as_path(), &[], &[]);
    assert_eq!(
        facts.sequencer_state(),
        Err(String::from(
            "git rev-parse --git-path did not print the three sequencer head paths of one Git directory"
        ))
    );
    let location: String = facts.location().expect_err("garbled location");
    assert!(location.starts_with("cli-git could not "), "{location}");
    // An exit status of 0 from the index query means "equal", whatever was printed.
    assert_eq!(facts.index_vs_head(), Ok(IndexVsHead::Matches));
    assert_eq!(
        facts.remote_guess_creates_branch(OsStr::new("topic")),
        Ok(false)
    );
    remove(root.as_path());
}

/// A head file is present whatever it is,
///  even a link to nowhere;
///  a missing or empty path is absent.
#[test]
fn head_file_presence_does_not_follow_links() {
    let root: PathBuf = fixture("facts-head-file");
    let dangling: PathBuf = root.join("dangling");
    std::os::unix::fs::symlink("nowhere", &dangling).expect("dangling link");
    let regular: PathBuf = root.join("regular");
    std::fs::write(&regular, b"").expect("regular file");
    assert!(head_file_exists(dangling.as_os_str().as_encoded_bytes()));
    assert!(head_file_exists(regular.as_os_str().as_encoded_bytes()));
    assert!(!head_file_exists(
        root.join("missing").as_os_str().as_encoded_bytes()
    ));
    assert!(!head_file_exists(b""));
    remove(root.as_path());
}
