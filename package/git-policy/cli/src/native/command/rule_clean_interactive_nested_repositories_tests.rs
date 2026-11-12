//! What:
//!  The plan of an interactive clean,
//!  the one force it is capped to,
//!  the spellings it is left alone in,
//!  and a real-Git control that `--no-force -f` really lands Git on one force.
//! Why:
//!  The cap changes what Git runs,
//!  so every clean that carries no menu,
//!  no descent,
//!  no ignored mode or only one force must reach Git untouched,
//!  and the token pair must be written where Git reads options and never where it reads paths.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(keepNestedRepositoriesOutOfAnInteractiveMenu(['clean', '-iffdX'], facts).arguments)
//! //   .toEqual(['clean', '-iffdX', '--no-force', '-f']);
//! ```

use super::{CapDecision, ONE_FORCE, keep_nested_repositories_out_of_an_interactive_menu};
use crate::command_test_support::{fixture, git, git_output, os_arguments, remove};
use crate::policy_test_support::{ScriptedFacts, linked_worktree, scripted_facts};
use std::path::PathBuf;

/// A linked worktree at `<root>/linked` of the repository at `<root>/main`,
///  holding the ignored nested repository `cache/nested` and the ignored directory `cache`.
struct Worktree {
    /// The fixture root the whole control is removed with,
    ///  once every path of it is named.
    root: PathBuf,
    /// The scripted facts whose location is that linked worktree.
    facts: ScriptedFacts,
}

fn worktree(name: &str, with_repository: bool) -> Worktree {
    let root: PathBuf = fixture(name);
    let linked: PathBuf = root.join("linked");
    let main: PathBuf = root.join("main");
    std::fs::create_dir_all(linked.join("cache")).expect("the ignored directory");
    std::fs::write(linked.join("cache/data.txt"), b"ignored data\n").expect("ignored work");
    if with_repository {
        std::fs::create_dir_all(linked.join("cache/nested/.git")).expect("a nested repository");
        std::fs::write(linked.join("cache/nested/untracked.txt"), b"untracked user data\n").expect("its work");
    }
    std::fs::create_dir_all(main.join(".git")).expect("the common Git directory");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(linked_worktree(
        main.to_str().expect("utf-8 fixture path"),
        linked.to_str().expect("utf-8 fixture path"),
    ));
    facts.clean_deletes = Ok(vec![b"Would remove cache/".to_vec()]);
    return Worktree { root, facts };
}

/// What:
///  An interactive clean whose menu would offer a nested repository runs with one force,
///  and the token pair lands at the end of its own options.
/// Why:
///   Git resets its force count on `--no-force`,
///  so the pair lands on one force whatever the person typed,
///  and no path may follow it as though it were an option.
#[test]
fn an_interactive_clean_runs_with_one_force_and_names_the_repository_it_keeps() {
    let found: Worktree = worktree("rule-interactive-nested", true);
    let mut facts: ScriptedFacts = found.facts.clone();
    for (spelling, expected) in [
        (
            vec!["clean", "-i", "-f", "-f", "-d", "-X"],
            vec!["clean", "-i", "-f", "-f", "-d", "-X", "--no-force", "-f"],
        ),
        (
            vec!["clean", "-iffdX"],
            vec!["clean", "-iffdX", "--no-force", "-f"],
        ),
        (
            vec!["clean", "-i", "-ffdX", "-e", "notes.txt", "--", "cache"],
            vec!["clean", "-i", "-ffdX", "-e", "notes.txt", "--no-force", "-f", "--", "cache"],
        ),
        (
            vec!["clean", "--interactive", "--force", "--force", "-d", "-X", "cache"],
            vec!["clean", "--interactive", "--force", "--force", "-d", "-X", "--no-force", "-f", "cache"],
        ),
    ] {
        facts.asked.clear();
        match keep_nested_repositories_out_of_an_interactive_menu(os_arguments(spelling.as_slice()).as_slice(), &mut facts) {
            CapDecision::Capped { arguments, repositories } => {
                assert_eq!(arguments, os_arguments(expected.as_slice()), "{spelling:?}");
                assert_eq!(repositories, vec![b"cache/nested".to_vec()], "{spelling:?}");
            }
            CapDecision::Unchanged => panic!("{spelling:?} was left alone"),
            CapDecision::Stopped { message } => panic!("{spelling:?} stopped: {message}"),
        }
    }
    assert_eq!(ONE_FORCE, vec!["--no-force", "-f"]);
    remove(found.root.as_path());
}

/// What:
///  A clean with no menu,
///  no descent,
///  no ignored mode or only one force is left to Git,
///  and nothing is asked about it.
/// Why:
///   Git keeps every nested repository by itself at one force,
///  and only an interactive clean cannot be asked what it will delete;
///  a transform that asked about every clean would cost a Git process for nothing.
#[test]
fn a_clean_that_offers_no_menu_reaches_git_untouched() {
    let found: Worktree = worktree("rule-interactive-nested-leaves", true);
    let mut facts: ScriptedFacts = found.facts.clone();
    for spelling in [
        vec!["clean", "-f", "-f", "-d", "-X"],
        vec!["clean", "-i", "-f", "-d", "-X"],
        vec!["clean", "-i", "-f", "-f", "-X"],
        vec!["clean", "-i", "-f", "-f", "-d"],
        vec!["status", "-i", "-f", "-f", "-d", "-X"],
        vec!["clean", "-i", "-f", "-f", "-d", "-X", "--no-such-option"],
    ] {
        facts.asked.clear();
        let decision: CapDecision =
            keep_nested_repositories_out_of_an_interactive_menu(os_arguments(spelling.as_slice()).as_slice(), &mut facts);
        assert!(matches!(decision, CapDecision::Unchanged), "{spelling:?}");
        assert!(facts.asked.is_empty(), "{spelling:?} asked {:?}", facts.asked);
    }
    remove(found.root.as_path());
}

/// What:
///  An interactive clean started in a subdirectory reads the names Git prints below that directory,
///  so the menu it caps is the menu the person is looking at.
/// Why:
///  Git prints a removed path relative to the directory the clean runs in,
///  with a `.` in front of one named from inside a directory it keeps,
///  so a transform that read the name from the worktree root would cap nothing and the menu would offer the
///  repository after all.
#[test]
fn an_interactive_clean_names_a_repository_below_the_command() {
    let root: PathBuf = fixture("rule-interactive-nested-subdirectory");
    let linked: PathBuf = root.join("linked");
    let main: PathBuf = root.join("main");
    std::fs::create_dir_all(linked.join("sub/container/nested/.git")).expect("a nested repository");
    std::fs::write(linked.join("sub/container/nested/untracked.txt"), b"untracked user data\n").expect("its work");
    std::fs::create_dir_all(main.join(".git")).expect("the common Git directory");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(crate::repository_location::RepositoryLocation {
        identity: crate::worktree_identity::WorktreeIdentity::LinkedWorktree {
            common_dir: main.join(".git"),
            git_dir: main.join(".git/worktrees/linked"),
            worktree_root: linked.clone(),
        },
        prefix: b"sub/".to_vec(),
    });
    facts.clean_deletes = Ok(vec![b"Would remove ./container".to_vec()]);
    match keep_nested_repositories_out_of_an_interactive_menu(
        os_arguments(&["clean", "-i", "-f", "-f", "-d", "-X"]).as_slice(),
        &mut facts,
    ) {
        CapDecision::Capped { repositories, .. } => assert_eq!(repositories, vec![b"sub/container/nested".to_vec()]),
        CapDecision::Unchanged => panic!("the repository below the command was not found"),
        CapDecision::Stopped { message } => panic!("the reading stopped: {message}"),
    }
    remove(root.as_path());
}

/// What:
///  A menu that would offer no repository is left alone,
///  and so is a clean Git itself refuses.
/// Why:
///   A clean that deletes nothing of a repository needs no cap,
///  and `-x` written with `-X` is the one region Git refuses before it reads anything,
///  so Git's own refusal is the answer the person gets and no dry run is asked.
#[test]
fn nothing_to_keep_and_gits_own_refusal_leave_the_clean_alone() {
    let empty: Worktree = worktree("rule-interactive-nested-empty", false);
    let mut facts: ScriptedFacts = empty.facts.clone();
    let arguments = os_arguments(&["clean", "-i", "-f", "-f", "-d", "-X"]);
    assert!(matches!(
        keep_nested_repositories_out_of_an_interactive_menu(arguments.as_slice(), &mut facts),
        CapDecision::Unchanged
    ));
    let found: Worktree = worktree("rule-interactive-nested-refused", true);
    let mut refused: ScriptedFacts = found.facts.clone();
    refused.clean_deletes = Err(String::from("a clean Git refuses itself must not be read"));
    refused.asked.clear();
    let both_modes: Vec<std::ffi::OsString> = os_arguments(&["clean", "-i", "-f", "-f", "-d", "-x", "-X"]);
    assert!(matches!(
        keep_nested_repositories_out_of_an_interactive_menu(both_modes.as_slice(), &mut refused),
        CapDecision::Unchanged
    ));
    assert!(refused.asked.is_empty(), "a clean Git refuses was read anyway: {:?}", refused.asked);
    remove(empty.root.as_path());
    remove(found.root.as_path());
}

/// What:
///  A dry run that failed stops the command with its own sentence,
///  whatever words Git wrote on standard error.
/// Why:
///   A nonzero exit is no proof that the clean which runs afterwards refuses,
///  and a repository the read could not see may be one the menu would have offered;
///  a failed read followed by the clean deleted a repository no copy held.
#[test]
fn a_failed_reading_stops_the_command() {
    let found: Worktree = worktree("rule-interactive-nested-failed", true);
    let mut facts: ScriptedFacts = found.facts.clone();
    let sentence: &str = "Git could not read what the clean would delete: fatal: transient dry-run read failure";
    facts.clean_deletes = Err(String::from(sentence));
    match keep_nested_repositories_out_of_an_interactive_menu(
        os_arguments(&["clean", "-i", "-f", "-f", "-d", "-X"]).as_slice(),
        &mut facts,
    ) {
        CapDecision::Stopped { message } => assert_eq!(message, sentence),
        CapDecision::Unchanged => panic!("a failed reading was read as Git's refusal"),
        CapDecision::Capped { .. } => panic!("a failed reading was capped"),
    }
    remove(found.root.as_path());
}

/// What:
///  A reading that never answered stops the command with its own sentence.
/// Why:
///   A repository the read could not see may be one the menu would have offered,
///  and a menu answered after Git starts is one no cap can be decided for.
#[test]
fn a_reading_that_never_answered_stops_the_command() {
    let found: Worktree = worktree("rule-interactive-nested-unreadable", true);
    let mut facts: ScriptedFacts = found.facts.clone();
    let sentence: &str =
        "Git never answered what the clean would delete: the read ended without an exit status, so the clean was not judged";
    facts.clean_deletes = Err(String::from(sentence));
    match keep_nested_repositories_out_of_an_interactive_menu(
        os_arguments(&["clean", "-i", "-f", "-f", "-d", "-X"]).as_slice(),
        &mut facts,
    ) {
        CapDecision::Stopped { message } => assert_eq!(message, sentence),
        CapDecision::Unchanged => panic!("a reading that never answered was read as nothing to keep"),
        CapDecision::Capped { .. } => panic!("a reading that never answered was capped"),
    }
    remove(found.root.as_path());
}

/// Whether one of `answer`'s lines is exactly `wanted`,
///  which is how a dry run names one path per line.
fn says_a_line(answer: &str, wanted: &str) -> bool {
    return answer.lines().any(|line| { return line == wanted; });
}

/// Whether one of `answer`'s lines begins with `prefix`.
fn says_a_line_starting(answer: &str, prefix: &str) -> bool {
    return answer.lines().any(|line| { return line.starts_with(prefix); });
}

/// What:
///  Real Git:
///  a clean typed with two forces and the token pair removes everything except the nested repository,
///  which Git keeps and names.
/// Why:
///  The transform's whole claim is that `--no-force -f` is one force to Git;
///  a control that only read the arguments would pass on a flag Git does not count.
#[test]
fn real_git_reads_the_pair_as_one_force() {
    let found: Worktree = worktree("rule-interactive-nested-git", true);
    // A repository Git can run in: the fixture's directories are what Git's own answer is read from.
    let repo: PathBuf = found.root.join("repository");
    std::fs::create_dir_all(repo.as_path()).expect("a repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("ignore file");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=initial"]);
    std::fs::create_dir_all(repo.join("cache/nested")).expect("a nested repository");
    git(repo.join("cache/nested").as_path(), &["init", "--quiet", "--initial-branch=inner"]);
    std::fs::write(repo.join("cache/nested/untracked.txt"), b"untracked user data\n").expect("its work");
    std::fs::create_dir_all(repo.join("cache/plain")).expect("an ignored directory");
    std::fs::write(repo.join("cache/plain/data.txt"), b"ignored data\n").expect("ignored work");
    let typed: std::process::Output = git_output(repo.as_path(), os_arguments(&["clean", "-n", "-f", "-f", "-d", "-X"]).as_slice());
    let typed_said: String = String::from_utf8_lossy(typed.stdout.as_slice()).into_owned();
    // Two forces remove the whole ignored directory, and Git says nothing about the repository inside it.
    assert!(says_a_line(&typed_said, "Would remove cache/"), "{typed_said}");
    assert!(!says_a_line_starting(&typed_said, "Would skip repository"), "{typed_said}");
    let capped: std::process::Output = git_output(
        repo.as_path(),
        os_arguments(&["clean", "-n", "-f", "-f", "-d", "-X", "--no-force", "-f"]).as_slice(),
    );
    let capped_said: String = String::from_utf8_lossy(capped.stdout.as_slice()).into_owned();
    // The pair is one force to Git: the repository is kept and named, so the menu cannot offer it.
    assert!(says_a_line(&capped_said, "Would skip repository cache/nested"), "{capped_said}");
    assert!(!says_a_line(&capped_said, "Would remove cache/"), "{capped_said}");
    remove(found.root.as_path());
}
