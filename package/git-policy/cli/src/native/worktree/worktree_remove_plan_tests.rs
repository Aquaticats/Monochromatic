//! Controls for the plan of `git worktree remove`:
//! the one accepted shape and every declined one.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, remove};

/// The accepted forms name exactly one existing directory and at most one `-f`/`--force`;
/// every other option and every other count is declined.
#[test]
fn the_plan_names_one_directory_and_no_option() {
    let root: PathBuf = fixture("remove-plan");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    for values in [vec!["worktree", "remove", "wt"], vec!["worktree", "remove", "--", "wt"], vec!["worktree", "remove", "./wt/"]] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None, None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(values[values.len() - 1]), force: false }), "{values:?}");
    }
    for values in [
        vec!["worktree", "remove"],
        vec!["worktree", "remove", "wt", "extra"],
        vec!["worktree", "remove", "absent"],
        vec!["worktree", "add", "wt"],
        vec!["worktree", "prune"],
    ] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None, None), None, "{values:?}");
    }
    remove(root.as_path());
}

/// What:
///  A forced removal names the same directory and is a plan,
///  so the wrapper destroys the worktree's subvolume instead of leaving Git to
///  unlink every entry of it.
/// Why:
///   Git deletes a dirty worktree only with `-f`/`--force` (`remove_worktree`,
///  `if (!force) check_clean_worktree(...)`),
///  so before this the forced removal,
/// which the wrapper's own reports tell a person to run,
///  always left every entry of the tree to real Git to unlink.
/// How much a worktree of that shape holds was not measured here;
/// the page
///  `doc/planning/cli-git-worktree-removal-cost-and-disk-time.md` is where it is measured.
#[test]
fn a_forced_removal_names_the_directory_and_the_force() {
    let root: PathBuf = fixture("remove-plan-forced");
    std::fs::create_dir(root.join("wt")).expect("directory");
    let canonical: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical");
    // Every spelling Git's `OPT__FORCE` reads as a force, the second one and the abbreviations included: the ladder
    // serves them as Git does, and a locked worktree is still Git's (`lock_state` leaves it to Git).
    for values in [
        vec!["worktree", "remove", "-f", "wt"],
        vec!["worktree", "remove", "--force", "wt"],
        vec!["worktree", "remove", "--force", "--", "wt"],
        vec!["worktree", "remove", "-f", "--", "wt"],
        vec!["worktree", "remove", "-f", "-f", "wt"],
        vec!["worktree", "remove", "-ff", "wt"],
        vec!["worktree", "remove", "--forc", "wt"],
    ] {
        let plan = parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None, None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from("wt"), force: true }), "{values:?}");
    }
    // An option Git does not know, and a switch with no directory, name no removal: Git answers those with its usage.
    for values in [vec!["worktree", "remove", "-fq", "wt"], vec!["worktree", "remove", "-f"]] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None, None), None, "{values:?}");
    }
    remove(root.as_path());
}

/// An empty operand names no directory:
/// joined to the start directory it would name the current one,
/// and Git answers that
///  it is not a working tree.
#[test]
fn an_empty_operand_names_no_directory() {
    let root: PathBuf = fixture("remove-plan-empty");
    for values in [vec!["worktree", "remove", ""], vec!["worktree", "remove", "--", ""]] {
        assert_eq!(parse_remove_plan(os_arguments(values.as_slice()).as_slice(), 0, root.as_path(), None, None), None, "{values:?}");
    }
    // The current directory spelled out is a name, and Git removes it.
    let plan = parse_remove_plan(os_arguments(&["worktree", "remove", "."]).as_slice(), 0, root.as_path(), None, None);
    assert_eq!(plan.map(|found| { return found.target; }), Some(std::fs::canonicalize(root.as_path()).expect("canonical")));
    remove(root.as_path());
}

/// A name that ends one registered worktree path at a directory boundary names that worktree,
/// because Git matches the last path components before it reads the name as a path.
/// The comparison is byte for byte,
/// as Git's own is unless `core.ignorecase` folds the case;
/// a name that differs only by case is therefore Git's own decision and is left to Git.
/// A name two registrations end in is Git's ambiguous case and is left to Git,
/// and so is a name no registration ends in.
#[test]
fn a_name_git_picks_by_its_last_component_names_the_registered_worktree() {
    let root: PathBuf = fixture("remove-plan-suffix");
    let repo: PathBuf = root.join("repo");
    let common: PathBuf = repo.join(".git");
    let topic: PathBuf = root.join("topic");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("registration");
    std::fs::create_dir_all(topic.as_path()).expect("worktree");
    std::fs::write(common.join("worktrees/topic/gitdir"), format!("{}/.git\n", topic.display())).expect("gitdir");
    let canonical: PathBuf = std::fs::canonicalize(topic.as_path()).expect("canonical");
    for (arguments, typed) in [
        (vec!["worktree", "remove", "topic"], "topic"),
        (vec!["worktree", "remove", "--", "topic"], "topic"),
    ] {
        let plan = parse_remove_plan(os_arguments(arguments.as_slice()).as_slice(), 0, repo.as_path(), Some(common.as_path()), None);
        assert_eq!(plan, Some(RemovePlan { target: canonical.clone(), operand: OsString::from(typed), force: false }), "{arguments:?}");
    }
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "TOPIC"]).as_slice(), 0, repo.as_path(), Some(common.as_path()), None),
        None,
        "Git's byte comparison picks no worktree for a name that differs only by case, and no directory stands at it"
    );
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "absent"]).as_slice(), 0, repo.as_path(), Some(common.as_path()), None),
        None,
        "no registration ends in that name and no directory stands at it"
    );
    // The force the person typed reaches a name the registrations picked too, because the plan carries it whichever way it read
    // the directory.
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "-f", "topic"]).as_slice(), 0, repo.as_path(), Some(common.as_path()), None),
        Some(RemovePlan { target: canonical.clone(), operand: OsString::from("topic"), force: true }),
        "a forced removal of a worktree the registrations picked"
    );
    let other: PathBuf = root.join("other/topic");
    std::fs::create_dir_all(common.join("worktrees/other")).expect("second registration");
    std::fs::create_dir_all(other.as_path()).expect("second worktree");
    std::fs::write(common.join("worktrees/other/gitdir"), format!("{}/.git\n", other.display())).expect("gitdir");
    assert_eq!(
        parse_remove_plan(os_arguments(&["worktree", "remove", "topic"]).as_slice(), 0, repo.as_path(), Some(common.as_path()), None),
        None,
        "Git's suffix match is unique or nothing, and no directory here stands at the name"
    );
    remove(root.as_path());
}

/// What:
///  A plan given the worktree Git's own answer names is that worktree,
///  whatever the plan's own resolving would have said.
/// Why:
///  The ladder destroys the plan's target before Git is ever started,
///  so a target read another way could destroy a worktree the guard never judged:
///  the plan names the guard's answer instead, and what was judged and what is destroyed are one path
///  (a review of `e615fb599b`, finding 1).
#[test]
fn the_plan_names_the_worktree_the_guard_read() {
    let root: PathBuf = fixture("remove-plan-picked");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir_all(repo.as_path()).expect("repository");
    let judged: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical");
    // A name the plan cannot resolve on its own: no registration ends in it, and no directory stands at it.
    let plan = parse_remove_plan(os_arguments(&["worktree", "remove", "case-differing-name"]).as_slice(), 0, repo.as_path(), None, Some(judged.as_path()));
    assert_eq!(
        plan,
        Some(RemovePlan { target: judged.clone(), operand: OsString::from("case-differing-name"), force: false }),
        "the plan names the worktree the guard read"
    );
    // A forced removal carries the same answer, because the ladder serves that form too.
    let forced = parse_remove_plan(os_arguments(&["worktree", "remove", "--force", "whatever"]).as_slice(), 0, repo.as_path(), None, Some(judged.as_path()));
    assert_eq!(forced, Some(RemovePlan { target: judged, operand: OsString::from("whatever"), force: true }), "and so does a forced one");
    remove(root.as_path());
}


//region every spelling of the removal's one option, in every position Git reads it

/// What:
///  The directory a row's words name, left for the row to fill in.
/// Why:
///   Each row needs a worktree no earlier row took,
///  and the shape of the words is the same for every spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PLACEHOLDER = '{the directory}'
/// ```
const PLACEHOLDER: &str = "{the directory}";

/// What:
///  Every spelling of the one option `git worktree remove` takes,
///  made from the option's own name rather than written out:
///  its short letter in a cluster, its long name at each length that still names it alone,
///  and each of those with the `no-` prefix that turns it off.
/// Why:
///   A hand-written list of spellings is a magic list:
///  one spelling left out is a spelling no control covers,
///  and the name Git's own table gives the option is the constant the spellings follow from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forceSpellings() // ['-f', '-ff', '--f', '--fo', '--for', '--forc', '--force', '--no-f', ..., '--no-force']
/// ```
fn force_spellings() -> Vec<String> {
    let name: &str = "force";
    let mut spellings: Vec<String> = Vec::new();
    // The short letter in a cluster, as `-f` and `-ff` are.
    for letters in 1..=2 {
        spellings.push(format!("-{}", "f".repeat(letters)));
    }
    // The long name at every length, and the negated name at every length: the last three of those end inside the `no-`
    // itself, which Git's reader matches against no option at all, so the walk must see them too.
    for length in 1..=name.len() {
        spellings.push(format!("--{}", &name[..length]));
    }
    let negated: String = format!("no-{name}");
    for length in 1..=negated.len() {
        spellings.push(format!("--{}", &negated[..length]));
    }
    return spellings;
}

/// What:
///  Every spelling of the usage answer this command has,
///  made from `help` the same way.
/// Why:
///   A word that makes Git print its usage removes nothing,
///  so the guard must not read it as a removal either,
///  and Git's own reader takes the long name at every length that names it alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// usageSpellings() // ['-h', '--h', '--he', '--hel', '--help']
/// ```
fn usage_spellings() -> Vec<String> {
    let name: &str = "help";
    let mut spellings: Vec<String> = vec![String::from("-h")];
    for length in 1..=name.len() {
        spellings.push(format!("--{}", &name[..length]));
    }
    return spellings;
}

/// What:
///  The four places a person writes an option, as the words they stand in:
///  before the directory, after it, before a `--`, and after one.
/// Why:
///  Git's `parse_options` reads the whole argument list,
///  so an option stands anywhere before a `--`;
///  the fourth place shows that a word after `--` is a directory and never an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rowsWith('--force', wt) // [['--force', wt], [wt, '--force'], ['--force', '--', wt], ['--', wt, '--force']]
/// ```
fn rows_with(spelling: &str, operand: &str) -> Vec<Vec<String>> {
    let double_dash: String = String::from("--");
    return vec![
        vec![spelling.to_owned(), operand.to_owned()],
        vec![operand.to_owned(), spelling.to_owned()],
        vec![spelling.to_owned(), double_dash.clone(), operand.to_owned()],
        vec![double_dash, operand.to_owned(), spelling.to_owned()],
    ];
}

/// What:
///  Whether Git itself removes the worktree at `worktree` when these words follow `worktree remove`.
/// Why:
///   The property is about what Git does, not about what its usage text says:
///  a row where Git removes and this program reads anything but a removal is the defect the walk exists for,
///  and a row where Git answers with its usage and this program reads a removal is a destroy Git never runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitRemoves(repository, ['/tmp/wt', '--no-force'], '/tmp/wt') // true
/// ```
fn git_removes(repository: &Path, words: &[String], worktree: &Path) -> bool {
    let mut arguments: Vec<&std::ffi::OsStr> = vec![std::ffi::OsStr::new("worktree"), std::ffi::OsStr::new("remove")];
    arguments.extend(words.iter().map(|one| return std::ffi::OsStr::new(one.as_str())));
    let _answered: std::process::Output = crate::test_support::git_output(repository, arguments.as_slice());
    return !worktree.exists();
}

/// What:
///  Take the worktree one row made away, whichever way the row ended.
/// Why:
///  A row Git answered with its usage leaves the worktree and its registration behind,
///  and the next row needs a directory of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takeTheRowAway(repository, '/tmp/wt')
/// ```
fn take_the_row_away(repository: &Path, worktree: &Path) {
    if worktree.exists() {
        let directory: String = worktree.display().to_string();
        crate::test_support::git(repository, &["worktree", "remove", "--force", "--", directory.as_str()]);
    }
    crate::test_support::git(repository, &["worktree", "prune"]);
}

/// What:
///  The reading of one shape, for a row's message.
/// Why:
///   A row that disagrees with Git must say what this program read instead,
///  and "not a removal" alone hides whether the word was refused or taken for Git's usage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describe(shape) // 'a removal, forced' | 'a removal' | 'not a removal' | 'a word it cannot read (...)'
/// ```
fn described(shape: &RemovalShape) -> String {
    return match shape {
        RemovalShape::Served { force: true, .. } => String::from("a removal, forced"),
        RemovalShape::Served { force: false, .. } => String::from("a removal"),
        RemovalShape::NotARemoval => String::from("not a removal"),
        RemovalShape::Unprovable(reason) => format!("a word it cannot read ({reason})"),
    };
}

/// What:
///  Every spelling and every position of the option `git worktree remove` takes,
///  each row run against real Git and held against this program's reading of the same words.
/// Why:
///   `removal_shape` decides whether a removal is judged at all,
///  and a spelling it reads as "not a removal" hands the command to Git with no look at the worktree:
///  an option written after the directory did exactly that, and this walk is the control for it
///  (a review of `d351b65746`).
///  The property it holds: Git removes a worktree if and only if this program reads a removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const spelling of forceSpellings()) for (const words of rowsWith(spelling, PLACEHOLDER)) { assert(gitRemoves(...) === isRemoval(removalShape([...words]))) }
/// ```
#[test]
fn every_spelling_of_the_removal_option_in_every_position_reads_as_git_reads_it() {
    let root: PathBuf = fixture("remove-plan-walk");
    let repository: PathBuf = crate::test_support::repository(root.as_path(), "repository");
    // The tracked file every row edits, so a row's dirty worktree holds an edit Git itself sees.
    std::fs::write(repository.join("tracked.txt"), "base\n").expect("the tracked file");
    crate::test_support::git(repository.as_path(), &["add", "--", "tracked.txt"]);
    crate::test_support::git(repository.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let mut walked: usize = 0;
    let mut disagreements: Vec<String> = Vec::new();
    let mut misspoken: Vec<String> = Vec::new();
    let mut spellings: Vec<String> = force_spellings();
    spellings.extend(usage_spellings());
    for (index, spelling) in spellings.iter().enumerate() {
        for (position, words) in rows_with(spelling, PLACEHOLDER).into_iter().enumerate() {
            // A directory that needs quoting: a space, an apostrophe and a dollar sign, which Git reads as bytes.
            let worktree: PathBuf = root.join(format!("linked worktree {index} {position} 'quoted $odd"));
            let operand: String = worktree.display().to_string();
            crate::test_support::git(repository.as_path(), &["worktree", "add", "--quiet", "--detach", operand.as_str()]);
            let words: Vec<String> = words.into_iter().map(|one| return one.replace(PLACEHOLDER, operand.as_str())).collect();
            let mut arguments: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove")];
            arguments.extend(words.iter().map(|one| return OsString::from(one.as_str())));
            let removed: bool = git_removes(repository.as_path(), words.as_slice(), worktree.as_path());
            let shape: RemovalShape = removal_shape(arguments.as_slice(), 0);
            let says_removal: bool = matches!(shape, RemovalShape::Served { .. });
            walked += 1;
            if removed != says_removal {
                disagreements.push(format!(
                    "{words:?}: Git {} and this program reads {}",
                    match removed {
                        true => "removed the worktree",
                        false => "removed nothing",
                    },
                    described(&shape)
                ));
            }
            if let RemovalShape::Served { force, operand_at, operand } = &shape {
                if !(*operand_at < arguments.len() && arguments[*operand_at] == *operand) {
                    misspoken.push(format!("{words:?}: the directory it names is not the word at {operand_at}"));
                }
                // The force the words carry, as Git shows it: the same words on a worktree holding an edit Git sees,
                // which an unforced removal refuses and a forced one takes. Nothing here says which spelling is which.
                let dirty: PathBuf = root.join(format!("dirty worktree {index} {position} 'quoted $odd"));
                let dirty_operand: String = dirty.display().to_string();
                crate::test_support::git(repository.as_path(), &["worktree", "add", "--quiet", "--detach", dirty_operand.as_str()]);
                std::fs::write(dirty.join("tracked.txt"), "an edit Git sees\n").expect("the edit");
                let elsewhere: Vec<String> = words.iter().map(|one| return one.replace(operand.to_string_lossy().as_ref(), dirty_operand.as_str())).collect();
                // A worktree holding an edit Git sees is removed when the force is on and refused when it is off, so what
                // Git did with it is the force these words carry.
                let git_force: bool = git_removes(repository.as_path(), elsewhere.as_slice(), dirty.as_path());
                if git_force != *force {
                    misspoken.push(format!("{words:?}: this program reads force={force} and Git {}", if git_force { "removed the dirty worktree" } else { "refused it" }));
                }
                take_the_row_away(repository.as_path(), dirty.as_path());
            }
            take_the_row_away(repository.as_path(), worktree.as_path());
        }
    }
    assert!(walked >= spellings.len() * 4, "every spelling ran in every position: {walked} rows");
    assert!(disagreements.is_empty(), "{} of {walked} rows disagree with Git:\n{}", disagreements.len(), disagreements.join("\n"));
    assert!(misspoken.is_empty(), "the force and the directory of a removal this program reads:\n{}", misspoken.join("\n"));
    remove(root.as_path());
}

//endregion
