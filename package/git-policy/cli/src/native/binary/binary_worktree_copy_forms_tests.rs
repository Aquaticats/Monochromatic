//! What:
//!  The ignored-state copy for every form of `git worktree add` that Git itself accepts,
//! through the built executable:
//!  abbreviated long options,
//! `--git-dir` and `--work-tree`,
//! a destination whose parent Git has to create;
//! and no copy,
//! with
//!  Git's own error kept,
//! for each form Git rejects.
//! Why:
//!  The owner decided on 2026-11-03 to keep the lock-free copy and read more forms,
//! so that fewer adds skip the copy.
//! The
//!  wrapper resolves an abbreviation exactly as Git's `parse_long_opt` does (`parse-options.c` of Git 2.56.0):
//! an exact name
//!  wins,
//! a unique prefix is the option,
//! an ambiguous prefix is Git's error,
//! and `--no-` forms negate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(run(wrappedGit, ['worktree', 'add', '--det', '../one']).stderr).toMatch(/^monochromatic-git-policy-cli: copied /);
//! ```

use super::support::{Fixture, Observed, fixture, remove, run_wrapped, stderr_of};
use super::worktree_copy::{needle, repository_with_rules};
use std::path::PathBuf;

/// Where the generated rules file lives,
/// as the repository's configuration names it.
const RULES_FILE: &str = ".cache/forbidden-strings.rules.txt";

/// The text the copy line of a successful add starts with.
const COPIED: &str = "monochromatic-git-policy-cli: copied ";

/// Git's status for a usage error.
const USAGE_ERROR_STATUS: i32 = 129;

/// An add that succeeded and copied the rules file into the worktree at `destination` (relative to the fixture root).
fn assert_copied(fixture: &Fixture, observed: &Observed, destination: &str) {
    assert_eq!(observed.code, Some(0), "{}", stderr_of(observed));
    assert!(stderr_of(observed).starts_with(COPIED), "{}", stderr_of(observed));
    assert_eq!(
        std::fs::read(fixture.root.join(destination).join(RULES_FILE)).expect("copied rules"),
        format!("{}\n", needle()).into_bytes(),
        "{destination}"
    );
}

/// An add Git rejected:
/// its own error,
/// no worktree,
/// no copy line.
fn assert_rejected(fixture: &Fixture, observed: &Observed, destination: &str) {
    assert_ne!(observed.code, Some(0), "{}", stderr_of(observed));
    assert!(!stderr_of(observed).contains(COPIED), "{}", stderr_of(observed));
    assert!(!stderr_of(observed).contains("does not read"), "Git's error stands alone: {}", stderr_of(observed));
    assert!(!fixture.root.join(destination).exists(), "{destination} must not exist");
}

/// Abbreviations Git accepts,
/// with the value of an abbreviated valued option in the next token or after `=`,
/// and the
/// negated abbreviation,
/// are copied into like the full spelling.
#[test]
fn an_abbreviated_long_option_gets_the_copy() {
    let fixture: Fixture = fixture("forms-abbreviated");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    for (arguments, destination) in [
        (vec!["worktree", "add", "--quiet", "--det", "../one"], "one"),
        (vec!["worktree", "add", "--quiet", "--lock", "--reas", "why", "../two"], "two"),
        (vec!["worktree", "add", "--quiet", "--lock", "--reas=why", "../three"], "three"),
        (vec!["worktree", "add", "--quiet", "--no-che", "../four"], "four"),
        (vec!["worktree", "add", "--qu", "--rel", "../five"], "five"),
        (vec!["worktree", "add", "--quiet", "--orp", "-b", "six", "../six"], "six"),
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_copied(&fixture, &observed, destination);
    }
    remove(&fixture);
}

/// An exact name wins over longer names it is a prefix of,
/// and a `--no-` form of an abbreviation negates it.
#[test]
fn an_exact_name_wins_and_a_negated_abbreviation_negates() {
    let fixture: Fixture = fixture("forms-exact");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    // `--lock` is a prefix of nothing else here, `--quiet` is exact; `--no-lo` negates the abbreviated `--lock`.
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "--lock", "--no-lo", "../one"]);
    assert_copied(&fixture, &observed, "one");
    remove(&fixture);
}

/// A prefix that fits two options is Git's error:
/// the wrapper does not guess,
/// adds nothing and copies nothing.
#[test]
fn an_ambiguous_prefix_is_gits_error_and_gets_no_copy() {
    let fixture: Fixture = fixture("forms-ambiguous");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    for (arguments, destination) in [
        (vec!["worktree", "add", "--quiet", "--re", "../one"], "one"),
        (vec!["worktree", "add", "--quiet", "--n", "../two"], "two"),
        (vec!["worktree", "add", "--quiet", "--no-", "../three"], "three"),
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_rejected(&fixture, &observed, destination);
        assert!(stderr_of(&observed).contains("ambiguous option") || observed.code == Some(USAGE_ERROR_STATUS), "{}", stderr_of(&observed));
    }
    remove(&fixture);
}

/// Forms Git rejects for another reason keep Git's own error:
/// a value on a flag,
/// a double negation,
/// an unknown option,
/// and
/// an option between `worktree` and `add`.
#[test]
fn forms_git_rejects_keep_gits_error_and_get_no_copy() {
    let fixture: Fixture = fixture("forms-rejected");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    for (arguments, destination) in [
        (vec!["worktree", "add", "--quiet", "--detach=yes", "../one"], "one"),
        (vec!["worktree", "add", "--quiet", "--no-no-checkout", "../two"], "two"),
        (vec!["worktree", "add", "--quiet", "--nonsense", "../three"], "three"),
        (vec!["worktree", "--quiet", "add", "../four"], "four"),
        (vec!["worktree", "add", "--quiet", "--reason", "../five"], "five"),
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_rejected(&fixture, &observed, destination);
    }
    remove(&fixture);
}

/// `--git-dir` and `--work-tree` do not move a relative destination:
/// it lies against the working directory,
/// as Git resolves
/// it,
/// from outside the work tree and from inside it,
/// in the attached and the separate spelling.
#[test]
fn git_dir_and_work_tree_globals_get_the_copy() {
    let fixture: Fixture = fixture("forms-git-dir");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let common: String = repo.join(".git").to_string_lossy().into_owned();
    let work_tree: String = repo.to_string_lossy().into_owned();
    let git_dir_option: String = format!("--git-dir={common}");
    let work_tree_option: String = format!("--work-tree={work_tree}");
    let outside: Observed = run_wrapped(
        &fixture,
        fixture.root.as_path(),
        &[git_dir_option.as_str(), work_tree_option.as_str(), "worktree", "add", "--quiet", "-b", "one", "one"],
    );
    assert_copied(&fixture, &outside, "one");
    let separate: Observed = run_wrapped(
        &fixture,
        fixture.root.as_path(),
        &["--git-dir", common.as_str(), "--work-tree", work_tree.as_str(), "worktree", "add", "--quiet", "-b", "two", "two"],
    );
    assert_copied(&fixture, &separate, "two");
    let inside: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &[git_dir_option.as_str(), work_tree_option.as_str(), "worktree", "add", "--quiet", "-b", "three", "../three"],
    );
    assert_copied(&fixture, &inside, "three");
    remove(&fixture);
}

/// A form the ladder does not serve
/// (`-f` is not one of the words the add plan reads),
/// whose branch Git refuses:
/// the wrapper makes the destination before real Git runs,
/// so it must take it away again,
/// because a Git command run in an empty folder inside a repository acts on the enclosing checkout.
#[test]
fn a_refused_add_in_a_form_the_ladder_does_not_serve_leaves_no_destination() {
    let fixture: Fixture = fixture("forms-refused-leftover");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    // The repository's own branch, so `-b main` is the branch Git refuses.
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--force", "-b", "main", "../taken"]);
    assert_rejected(&fixture, &observed, "taken");
    assert!(
        stderr_of(&observed).contains("a branch named 'main' already exists"),
        "Git's own refusal stands alone: {}",
        stderr_of(&observed)
    );
    remove(&fixture);
}

/// A destination whose parent directories do not exist yet is the destination Git creates them for.
#[test]
fn a_destination_below_missing_directories_gets_the_copy() {
    let fixture: Fixture = fixture("forms-missing-parents");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "deep", ".claude/worktrees/deep"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(stderr_of(&observed).starts_with(COPIED), "{}", stderr_of(&observed));
    assert!(repo.join(".claude/worktrees/deep").join(RULES_FILE).exists());
    remove(&fixture);
}
