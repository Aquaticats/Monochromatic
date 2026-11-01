//! What:
//!  Real Git 2.56.0 controls for the three copied tables and a differential of the
//!       `git branch` creation fact against what the binary does to a repository.
//! Why:
//!  Whether `git branch` creates depends on a count of action options that no test
//!      written by hand covers;
//!  here Git itself decides every case.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const args of cases) expect(parse(args).createsBranch).toBe(await gitCreatedABranch(args));
//! ```

/// The parser,
///  the tables,
///  the oracles and the real-Git fixture helpers.
use super::BranchCreationCommand::Branch;
use super::{BranchCreationRegion, parse_branch_creation_region};
use crate::command_branch_table::BRANCH_TABLE;
use crate::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
use crate::command_options::OptionError;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, fixture, git, git_status, git_with_input, os_arguments, output_text,
    remove, repository,
};
use std::path::{Path, PathBuf};
use std::process::Output;

/// One entry per `;`:
///  no option,
///  every `git branch` option in a representative spelling,
/// and three spellings Git refuses.
///  A space separates an option from its detached value.
const OPTIONS: &str = ";-v;-q;-t;--track=inherit;--no-track;--set-upstream;--no-set-upstream;\
    -u main;--set-upstream-to=main;--no-set-upstream-to;--unset-upstream;--color;\
    --color=never;-r;-a;--contains;--contains=main;--no-contains=main;--with=main;\
    --without=main;--abbrev;--abbrev=7;-d;-D;--no-delete;-m;-M;--no-move;--omit-empty;-c;-C;\
    --no-copy;-l;--no-list;--show-current;--create-reflog;--edit-description;\
    --delete-merged=nomatch;--dry-run;--no-dry-run;-f;--merged;--merged=main;\
    --no-merged=main;--forked=main;--column;--sort=refname;--points-at=main;--no-points-at;\
    -i;--recurse-submodules;--no-recurse-submodules;--format=x;--bogus;--no-remotes;-x";

/// Entries also compared with zero and with three names,
///  where the name count decides.
const EVERY_COUNT: &str = ";-q;-t;-f;-c;-C;-l;-d;-m;--show-current";

/// Options that select an action,
///  write a variable another option writes,
///  or clear one.
const INTERACTING: &str = "-t;--no-track;--set-upstream;--no-set-upstream;-u main;\
    --no-set-upstream-to;-r;-a;--contains=main;-d;-D;--no-delete;-m;--no-move;-c;-C;\
    --no-copy;-l;--no-list;--show-current;--edit-description;--unset-upstream;\
    --delete-merged=nomatch;--dry-run;--no-dry-run;-f;--points-at=main;--no-points-at;\
    --recurse-submodules;--no-recurse-submodules;-x";

/// Options whose effect depends on a second option:
///  each is paired with every entry of
/// `INTERACTING`,
///  in both orders.
const PARTNERS: &str = "-c;-a;--set-upstream;--dry-run;--recurse-submodules";

/// How many argument lists were compared,
///  created a branch,
///  and were refused.
struct Tally {
    runs: usize,
    created: usize,
    refused: usize,
}

/// The copied tables match what the binary prints for each command.
#[test]
fn tables_match_git() {
    let directory: PathBuf = fixture("branch-tables");
    let root: PathBuf = repository(directory.as_path(), "repository");
    for (table, command) in [
        (BRANCH_TABLE, "branch"),
        (CHECKOUT_TABLE, "checkout"),
        (SWITCH_TABLE, "switch"),
    ] {
        assert_table_invariants(table);
        assert_eq!(
            render_completion(table),
            git_completion(root.as_path(), &[command]),
            "{command}"
        );
    }
    remove(directory.as_path());
}

/// Names of the branches stored as loose refs,
///  sorted.
fn heads(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::<String>::new();
    for entry in std::fs::read_dir(root.join(".git/refs/heads")).expect("branch directory") {
        let name: std::ffi::OsString = entry.expect("branch entry").file_name();
        names.push(name.to_string_lossy().into_owned());
    }
    names.sort();
    return names;
}

/// A repository on `main` with an older branch `other`.
///  Returns the fixture directory,
///  the
/// repository root and the `git update-ref --stdin` lines that put both branches back.
fn branch_fixture(name: &str) -> (PathBuf, PathBuf, String) {
    let directory: PathBuf = fixture(name);
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(root.as_path(), &["branch", "other"]);
    git(
        root.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=second"],
    );
    // With this setting off Git refuses `--recurse-submodules`, which the parser cannot see.
    git(
        root.as_path(),
        &["config", "submodule.propagateBranches", "true"],
    );
    let restore: String = format!(
        "update refs/heads/main {}\nupdate refs/heads/other {}\n",
        output_text(&git(root.as_path(), &["rev-parse", "main"])),
        output_text(&git(root.as_path(), &["rev-parse", "other"])),
    );
    // Positive control of the oracle: both branches are visible as loose refs.
    assert_eq!(heads(root.as_path()), ["main", "other"]);
    return (directory, root, restore);
}

/// Run `git branch <arguments>`.
///  Report whether a branch appeared beside `main` and `other`
/// (a rename or a deletion removes one of them) and Git's exit code,
///  then put both back.
fn observe(root: &Path, restore: &str, arguments: &[&str]) -> (bool, Option<i32>) {
    let mut full: Vec<&str> = vec!["branch"];
    full.extend_from_slice(arguments);
    let output: Output = git_status(root, full.as_slice());
    let after: Vec<String> = heads(root);
    if after == ["main", "other"] {
        return (false, output.status.code());
    }
    let mut script: String = restore.to_owned();
    for name in &after {
        if name != "main" && name != "other" {
            script.push_str(format!("delete refs/heads/{name}\n").as_str());
        }
    }
    let kept: bool =
        after.contains(&String::from("main")) && after.contains(&String::from("other"));
    if !kept {
        // A rename of the current branch moved `HEAD` with it.
        git(root, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    }
    let restored: Output = git_with_input(
        root,
        os_arguments(&["update-ref", "--stdin"]).as_slice(),
        script.as_bytes(),
    );
    assert!(
        restored.status.success(),
        "{}",
        String::from_utf8_lossy(&restored.stderr)
    );
    return (kept, output.status.code());
}

/// Require the parser's answer for one argument list to be what Git did.
fn check(arguments: &[&str], created: bool, code: Option<i32>, tally: &mut Tally) {
    let parsed: Result<BranchCreationRegion, OptionError> =
        parse_branch_creation_region(Branch, os_arguments(arguments).as_slice(), &[]);
    tally.runs += 1;
    match parsed {
        Ok(region) => {
            assert_eq!(region.creates_branch, created, "{arguments:?}");
            if created {
                tally.created += 1;
            }
        }
        Err(error) => {
            assert!(!created, "{arguments:?}");
            assert_eq!(code, Some(129), "{arguments:?}: {error}");
            tally.refused += 1;
        }
    }
}

/// Two argument groups as one list.
fn joined<'a>(first: &[&'a str], second: &[&'a str]) -> Vec<&'a str> {
    let mut all: Vec<&'a str> = first.to_vec();
    all.extend_from_slice(second);
    return all;
}

/// Run one argument list and compare it.
fn compare(root: &Path, restore: &str, arguments: &[&str], tally: &mut Tally) {
    let (created, code): (bool, Option<i32>) = observe(root, restore, arguments);
    check(arguments, created, code, tally);
}

/// Compare one option list followed by two names.
///  A creation takes `<new> <start>` and a
/// copy takes `<old> <new>`,
///  so Git creates in one of the two orders;
///  the parser reads
/// counts and must give both orders that same answer.
fn compare_two_names(root: &Path, restore: &str, options: &[&str], tally: &mut Tally) {
    let forward: Vec<&str> = joined(options, &["fresh", "other"]);
    let backward: Vec<&str> = joined(options, &["other", "fresh"]);
    let (created_forward, code_forward): (bool, Option<i32>) =
        observe(root, restore, forward.as_slice());
    let (created_backward, code_backward): (bool, Option<i32>) =
        observe(root, restore, backward.as_slice());
    let created: bool = created_forward || created_backward;
    check(forward.as_slice(), created, code_forward, tally);
    check(backward.as_slice(), created, code_backward, tally);
}

/// Every option alone,
///  before and after one name and before two names,
///  and the options
/// where the count decides with zero and three names,
///  creates exactly when the parser says.
#[test]
fn single_branch_options_match_git() {
    let (directory, root, restore): (PathBuf, PathBuf, String) = branch_fixture("branch-single");
    let mut tally: Tally = Tally {
        runs: 0,
        created: 0,
        refused: 0,
    };
    let entries: Vec<&str> = OPTIONS.split(';').collect();
    for entry in &entries {
        let options: Vec<&str> = entry.split_whitespace().collect();
        let before: Vec<&str> = joined(options.as_slice(), &["fresh"]);
        // After the name an optional or last-argument value is read differently.
        let after: Vec<&str> = joined(&["fresh"], options.as_slice());
        compare(
            root.as_path(),
            restore.as_str(),
            before.as_slice(),
            &mut tally,
        );
        compare(
            root.as_path(),
            restore.as_str(),
            after.as_slice(),
            &mut tally,
        );
        compare_two_names(
            root.as_path(),
            restore.as_str(),
            options.as_slice(),
            &mut tally,
        );
    }
    for entry in EVERY_COUNT.split(';') {
        let options: Vec<&str> = entry.split_whitespace().collect();
        let three: Vec<&str> = joined(options.as_slice(), &["fresh", "other", "extra"]);
        compare(
            root.as_path(),
            restore.as_str(),
            options.as_slice(),
            &mut tally,
        );
        compare(
            root.as_path(),
            restore.as_str(),
            three.as_slice(),
            &mut tally,
        );
        assert!(entries.contains(&entry), "{entry}");
    }
    // Positive control: the comparison reached creation, refusal and neither.
    assert_eq!((tally.runs, tally.created, tally.refused), (248, 116, 12));
    remove(directory.as_path());
}

/// Every partner with every interacting option,
///  in both orders,
///  before one name.
#[test]
fn paired_branch_options_match_git() {
    let (directory, root, restore): (PathBuf, PathBuf, String) = branch_fixture("branch-paired");
    let mut tally: Tally = Tally {
        runs: 0,
        created: 0,
        refused: 0,
    };
    for partner in PARTNERS.split(';') {
        for entry in INTERACTING.split(';') {
            let options: Vec<&str> = entry.split_whitespace().collect();
            let before: Vec<&str> = joined(&[partner], options.as_slice());
            let after: Vec<&str> = joined(options.as_slice(), &[partner]);
            for pair in [before, after] {
                let arguments: Vec<&str> = joined(pair.as_slice(), &["fresh"]);
                compare(
                    root.as_path(),
                    restore.as_str(),
                    arguments.as_slice(),
                    &mut tally,
                );
            }
        }
    }
    // Positive control: the comparison reached creation, refusal and neither.
    assert_eq!((tally.runs, tally.created, tally.refused), (310, 72, 10));
    remove(directory.as_path());
}
