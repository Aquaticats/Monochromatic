//! What:
//!  Real Git 2.56.0 controls for the three copied tables and a differential of the
//!       `git branch` creation fact against what the binary does to a repository.
//! Why:
//!  Whether `git branch` creates depends on a count of action options that no test
//!      written by hand covers;
//!  here Git itself decides every case.
//!  Each case runs against its own copy of a pristine repository,
//!  so no case puts anything back and the cases run side by side.
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
use crate::command_test_cases::{across_workers, copy_tree};
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, fixture, git, git_status, os_arguments, remove, repository,
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

/// One worker's disposable space and what its cases found.
struct Lane {
    pristine: PathBuf,
    scratch: PathBuf,
    tally: Tally,
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

/// A repository on `main` with an older branch `other` that no case touches.
///  Returns the fixture directory and the pristine repository root that every case copies.
fn branch_fixture(name: &str) -> (PathBuf, PathBuf) {
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
    // Positive control of the oracle: both branches are visible as loose refs.
    assert_eq!(heads(root.as_path()), ["main", "other"]);
    return (directory, root);
}

/// The space of one worker: its own scratch directory and an empty tally.
fn lane(directory: &Path, pristine: &Path, number: usize) -> Lane {
    let scratch: PathBuf = directory.join(format!("lane-{number}"));
    std::fs::create_dir_all(scratch.as_path()).expect("lane directory");
    return Lane {
        pristine: pristine.to_path_buf(),
        scratch,
        tally: Tally {
            runs: 0,
            created: 0,
            refused: 0,
        },
    };
}

/// Run `git branch <arguments>` in a fresh copy of the pristine repository.
///  Report whether a branch appeared beside `main` and `other`
/// (a rename or a deletion removes one of them) and Git's exit code,
///  then drop the copy.
fn observe(lane: &Lane, arguments: &[&str]) -> (bool, Option<i32>) {
    let case: PathBuf = lane.scratch.join("case");
    copy_tree(lane.pristine.as_path(), case.as_path());
    let mut full: Vec<&str> = vec!["branch"];
    full.extend_from_slice(arguments);
    let output: Output = git_status(case.as_path(), full.as_slice());
    let after: Vec<String> = heads(case.as_path());
    remove(case.as_path());
    let kept: bool =
        after.contains(&String::from("main")) && after.contains(&String::from("other"));
    return (kept && after != ["main", "other"], output.status.code());
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
fn compare(lane: &mut Lane, arguments: &[&str]) {
    let (created, code): (bool, Option<i32>) = observe(lane, arguments);
    check(arguments, created, code, &mut lane.tally);
}

/// Compare one option list followed by two names.
///  A creation takes `<new> <start>` and a
/// copy takes `<old> <new>`,
///  so Git creates in one of the two orders;
///  the parser reads
/// counts and must give both orders that same answer.
fn compare_two_names(lane: &mut Lane, options: &[&str]) {
    let forward: Vec<&str> = joined(options, &["fresh", "other"]);
    let backward: Vec<&str> = joined(options, &["other", "fresh"]);
    let (created_forward, code_forward): (bool, Option<i32>) = observe(lane, forward.as_slice());
    let (created_backward, code_backward): (bool, Option<i32>) = observe(lane, backward.as_slice());
    let created: bool = created_forward || created_backward;
    check(forward.as_slice(), created, code_forward, &mut lane.tally);
    check(backward.as_slice(), created, code_backward, &mut lane.tally);
}

/// The tallies of all workers as one.
fn merged(lanes: Vec<Lane>) -> Tally {
    let mut total: Tally = Tally {
        runs: 0,
        created: 0,
        refused: 0,
    };
    for lane in lanes {
        total.runs += lane.tally.runs;
        total.created += lane.tally.created;
        total.refused += lane.tally.refused;
    }
    return total;
}

/// Positive control: the comparison reached creation, refusal and neither.
fn assert_reached_every_outcome(tally: &Tally) {
    assert!(tally.created > 0, "no case created a branch");
    assert!(tally.refused > 0, "no case was refused");
    assert!(tally.created + tally.refused < tally.runs, "every case created or was refused");
}

/// Every option alone,
///  before and after one name and before two names,
///  and the options
/// where the count decides with zero and three names,
///  creates exactly when the parser says.
#[test]
fn single_branch_options_match_git() {
    let (directory, pristine): (PathBuf, PathBuf) = branch_fixture("branch-single");
    let entries: Vec<&str> = OPTIONS.split(';').collect();
    let counted: Vec<&str> = EVERY_COUNT.split(';').collect();
    // Each item says whether the entry is also compared with zero and three names.
    let items: Vec<(bool, &str)> = entries
        .iter()
        .map(|entry| return (false, *entry))
        .chain(counted.iter().map(|entry| return (true, *entry)))
        .collect();
    let lanes: Vec<Lane> = across_workers(
        &items,
        |number| return lane(directory.as_path(), pristine.as_path(), number),
        |&(every_count, entry), lane| {
            let options: Vec<&str> = entry.split_whitespace().collect();
            if every_count {
                let three: Vec<&str> = joined(options.as_slice(), &["fresh", "other", "extra"]);
                compare(lane, options.as_slice());
                compare(lane, three.as_slice());
            } else {
                let before: Vec<&str> = joined(options.as_slice(), &["fresh"]);
                // After the name an optional or last-argument value is read differently.
                let after: Vec<&str> = joined(&["fresh"], options.as_slice());
                compare(lane, before.as_slice());
                compare(lane, after.as_slice());
                compare_two_names(lane, options.as_slice());
            }
        },
    );
    let tally: Tally = merged(lanes);
    // Four lists per option entry and two per every-count entry, whatever the entries are.
    assert_eq!(tally.runs, 4 * entries.len() + 2 * counted.len());
    assert!(counted.iter().all(|entry| return entries.contains(entry)));
    assert_reached_every_outcome(&tally);
    remove(directory.as_path());
}

/// Every partner with every interacting option,
///  in both orders,
///  before one name.
#[test]
fn paired_branch_options_match_git() {
    let (directory, pristine): (PathBuf, PathBuf) = branch_fixture("branch-paired");
    let partners: Vec<&str> = PARTNERS.split(';').collect();
    let interacting: Vec<&str> = INTERACTING.split(';').collect();
    let items: Vec<(&str, &str)> = partners
        .iter()
        .flat_map(|partner| return interacting.iter().map(|entry| return (*partner, *entry)))
        .collect();
    let lanes: Vec<Lane> = across_workers(
        &items,
        |number| return lane(directory.as_path(), pristine.as_path(), number),
        |&(partner, entry), lane| {
            let options: Vec<&str> = entry.split_whitespace().collect();
            let before: Vec<&str> = joined(&[partner], options.as_slice());
            let after: Vec<&str> = joined(options.as_slice(), &[partner]);
            for pair in [before, after] {
                let arguments: Vec<&str> = joined(pair.as_slice(), &["fresh"]);
                compare(lane, arguments.as_slice());
            }
        },
    );
    let tally: Tally = merged(lanes);
    // Both orders for every partner and interacting entry.
    assert_eq!(tally.runs, 2 * partners.len() * interacting.len());
    assert_reached_every_outcome(&tally);
    remove(directory.as_path());
}
