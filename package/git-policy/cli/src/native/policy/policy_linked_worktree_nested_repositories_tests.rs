//! What:
//!  What a clean in a linked worktree takes when it deletes a nested Git repository,
//!  and the one audit line that names it before Git starts.
//! Why:
//!  The owner decided on 2026-11-15 that such a clean deletes the repository as real Git does
//!  and that the wrapper logs the command and what went, with no saved copy
//!  (`doc/handover/cli-git-rust-implementation.md`, "Answers on waking, third round").
//!  So the control has to show two things at once:
//!  the clean runs exactly as typed, and the record names each repository Git takes,
//!  written before Git runs and with the facts of a line that knows it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // allowADiscardingForm({ command: 'CleanIgnoredOnly', region: ['-f','-f','-d','-X'] })
//! // one line: outcome 'logged', facts { nestedRepositories: ['nested'] }, writtenBeforeGitRuns true
//! ```

use super::AllowedForm;
use super::allow_a_discarding_form;
use crate::command_test_support::{fixture, git, remove};
use crate::policy_engine::PolicyOutcome;
use crate::repo_read::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use crate::rule_linked_worktree::GuardedCommand;
use crate::test_support::{REAL_GIT, isolated_overlay};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  A worktree holding one ignored nested repository with its own history and an uncommitted file,
///  and the paths of the fixture.
/// Why:
///  The nested repository's own commit and its uncommitted file are what no other copy of the
///  outer repository holds,
///  so a clean that deletes it is the loss the record exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = worktreeWithANestedRepository('policy-nested-log');
/// ```
struct Places {
    /// The fixture root the whole control is removed with.
    root: PathBuf,
    /// The main checkout,
    /// which holds the audit file every worktree writes to.
    main: PathBuf,
    /// The linked worktree the clean runs in.
    linked: PathBuf,
    /// The nested repository inside that worktree's ignored directory.
    nested: PathBuf,
}

fn worktree_with_a_nested_repository(name: &str) -> Places {
    let root: PathBuf = fixture(name);
    let main: PathBuf = root.join("main");
    let linked: PathBuf = root.join("linked");
    std::fs::create_dir_all(main.as_path()).expect("the main checkout");
    git(main.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(main.join(".gitignore"), b"nested/\n").expect("ignore rules");
    std::fs::write(main.join("tracked.txt"), b"one\n").expect("tracked file");
    git(main.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=base"]);
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    let nested: PathBuf = linked.join("nested");
    std::fs::create_dir_all(nested.as_path()).expect("the nested repository");
    git(nested.as_path(), &["init", "--quiet", "--initial-branch=inner"]);
    std::fs::write(nested.join("tracked.txt"), b"committed\n").expect("its tracked file");
    git(nested.as_path(), &["add", "--", "tracked.txt"]);
    git(nested.as_path(), &["commit", "--quiet", "--message=inner"]);
    std::fs::write(nested.join("untracked.txt"), b"the nested repository's work\n").expect("its work");
    return Places { root, main, linked, nested };
}

/// What:
///  The decision for one clean region, with the facts real Git answers for the linked worktree,
///  and the audit lines it wrote.
/// Why:
///  The record is the whole observable of this reading:
///  the decision lets the clean run, and the line is what a later reader has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { outcome, lines } = decideClean(places, ['-f','-f','-d','-X']);
/// ```
fn decide_clean(places: &Places, region: &[&str]) -> (PolicyOutcome, Vec<String>) {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), places.linked.as_os_str().to_os_string()];
    let mut facts: GitFacts = git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
    let location = facts.location().expect("the linked worktree's identity");
    let region: Vec<OsString> = region.iter().map(OsString::from).collect();
    let mut seen: Vec<OsString> = vec![OsString::from("clean")];
    seen.extend(region.iter().cloned());
    let before: usize = audit_lines(places.main.as_path()).len();
    let outcome: PolicyOutcome = allow_a_discarding_form(AllowedForm {
        facts: &mut facts,
        command: if region.iter().any(|word| return word == "-x") {
            GuardedCommand::CleanIgnored
        } else {
            GuardedCommand::CleanIgnoredOnly
        },
        location: &location,
        region: region.as_slice(),
        seen: seen.as_slice(),
        dirty: Some(false),
    });
    return (outcome, audit_lines(places.main.as_path()).into_iter().skip(before).collect());
}

/// Every audit line of the repository, oldest first.
fn audit_lines(main: &Path) -> Vec<String> {
    let file: PathBuf = main.join(".git/monochromatic-git-policy-cli-audit.jsonl");
    let Ok(text) = std::fs::read_to_string(file.as_path()) else {
        return Vec::<String>::new();
    };
    return text.lines().map(String::from).collect();
}

/// What:
///  A clean with a second force names the nested repository it takes,
///  in one line written before Git runs,
///  and keeps no copy of it.
/// Why:
///  That is the owner's answer of 2026-11-15: the clean deletes the repository as real Git does,
///  and the wrapper logs the command and what went.
///  The line's own field says it was written before Git ran,
///  because Git has not deleted anything yet when it is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(lines).toEqual([expect.stringContaining('"nestedRepositories":["ignored/nested"]')]);
/// ```
#[test]
fn a_clean_names_the_nested_repository_it_takes_before_git_runs() {
    let places: Places = worktree_with_a_nested_repository("policy-nested-log");
    let (outcome, lines): (PolicyOutcome, Vec<String>) = decide_clean(&places, &["-f", "-f", "-d", "-X"]);
    assert!(matches!(outcome, PolicyOutcome::Findings(ref found) if found.is_empty()), "{outcome:?}");
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &String = lines.first().expect("one line");
    for expected in [
        "\"outcome\":\"logged\"",
        "\"form\":\"CleanIgnoredOnly\"",
        "\"writtenBeforeGitRuns\":true",
        "\"nestedRepositories\":[\"nested\"]",
    ] {
        assert!(line.contains(expected), "{expected} missing from {line}");
    }
    // Nothing was moved and nothing was copied: the worktree is exactly as the clean found it.
    assert_eq!(
        std::fs::read_to_string(places.nested.join("untracked.txt")).expect("the nested work"),
        "the nested repository's work\n"
    );
    let discarded: PathBuf = places.main.join(".git/monochromatic-git-policy-cli/discarded");
    assert!(!discarded.exists(), "a copy was kept under {}", discarded.display());
    remove(places.root.as_path());
}

/// What:
///  A clean that takes no repository keeps the one line it always left.
/// Why:
///  A clean of ignored files that meets no nested repository is the ordinary case,
///  and its record is the allowed line and not a logged one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideClean(places, ['-f','-d','-X']) // one 'allowed' line, no nestedRepositories fact
/// ```
#[test]
fn a_clean_that_takes_no_repository_leaves_the_allowed_line() {
    let places: Places = worktree_with_a_nested_repository("policy-nested-nothing");
    // One force is Git's own hold on a repository with a `.git` entry, so the clean takes nothing.
    let (outcome, lines): (PolicyOutcome, Vec<String>) = decide_clean(&places, &["-f", "-d", "-X"]);
    assert!(matches!(outcome, PolicyOutcome::Findings(ref found) if found.is_empty()), "{outcome:?}");
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &String = lines.first().expect("one line");
    assert!(line.contains("\"outcome\":\"allowed\""), "{line}");
    assert!(!line.contains("nestedRepositories"), "{line}");
    remove(places.root.as_path());
}

/// What:
///  A dry run that failed stops the clean instead of deleting repositories no line names.
/// Why:
///  The record is what a later reader uses to attribute the loss,
///  and a clean whose reading failed would delete a nested repository with a line that says
///  nothing about it.
///  The sentence is the reading's own and names what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideClean(places, ['-f','-f','-d','-X']).outcome.kind === 'findings' with the reading's sentence
/// ```
#[test]
fn a_dry_run_that_failed_stops_the_clean() {
    let places: Places = worktree_with_a_nested_repository("policy-nested-failed");
    let shim: PathBuf = places.root.join("git-that-cannot-run-a-dry-run");
    std::fs::write(
        shim.as_path(),
        format!(
            "#!/bin/sh\ncase \"$*\" in\n  *'clean -n'*) echo 'fatal: the dry run did not answer' >&2; exit 128;;\nesac\nexec {REAL_GIT} \"$@\"\n",
            REAL_GIT = REAL_GIT
        )
        .as_bytes(),
    )
    .expect("the shim");
    let mut permissions = std::fs::metadata(shim.as_path()).expect("the shim").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(shim.as_path(), permissions).expect("the shim runs");
    let prefix: Vec<OsString> = vec![OsString::from("-C"), places.linked.as_os_str().to_os_string()];
    let mut facts: GitFacts = git_facts(shim.as_path(), prefix.as_slice(), isolated_overlay().as_slice());
    let location = facts.location().expect("the linked worktree's identity");
    let region: Vec<OsString> = ["-f", "-f", "-d", "-X"].iter().map(OsString::from).collect();
    let seen: Vec<OsString> = ["clean", "-f", "-f", "-d", "-X"].iter().map(OsString::from).collect();
    let before: usize = audit_lines(places.main.as_path()).len();
    let outcome: PolicyOutcome = allow_a_discarding_form(AllowedForm {
        facts: &mut facts,
        command: GuardedCommand::CleanIgnoredOnly,
        location: &location,
        region: region.as_slice(),
        seen: seen.as_slice(),
        dirty: Some(false),
    });
    let said: String = format!("{outcome:?}");
    assert!(
        said.contains("could not read what the clean would delete"),
        "the refusal names the reading that failed: {said}"
    );
    let lines: Vec<String> = audit_lines(places.main.as_path()).into_iter().skip(before).collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("\"outcome\":\"rejected\""), "{lines:?}");
    remove(places.root.as_path());
}
