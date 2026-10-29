//! Ratchet budgets for the cost of one wrapped commit: how many processes it starts and how many fsync-class calls it
//!  makes, so a change that makes a commit dearer fails a test that names the cause.
//! Each control runs the real wrapped binary under `strace` on a disposable repository (an explicit-path commit,
//!  `git commit --quiet --message=x -- a.txt`), reads every `execve` and sync call of the whole process tree, and
//!  compares exact per-subcommand counts with the budget constants below.
//! The counts are exact, not upper bounds: a commit that got cheaper fails too, so the constant is lowered in the
//!  same change and the saving cannot be given back unnoticed.
//! The controls need `strace`; a host without it reports that and checks nothing, so the host suite is where they run.

use super::budget_trace::{Counts, TRACED_CALLS, count};
use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::fork_failure::strace_available;
use super::support::{Fixture, REAL_GIT, fixture, git, repository};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

//region budgets: the one place the numbers live
// The ratchet rule: lower a number whenever a commit gets cheaper, and never raise one without a recorded reason in
//  the commit message and in `doc/planning/cli-git-spawn-and-sync-reduction.md`, section "Budgets".
//  A test fails on any difference, so a saving shows up as a failure that asks for the lower number.
// Each list names every program the commit may start or sync from; a zero entry documents a start that was removed
//  (for example `findmnt` and `hash-object`) and must not return, and makes the failure message print that name.
// Measured with host Git 2.56.0 on Linux, maintenance kept in the foreground (see `prepared_repository`), over five
//  runs of every scenario; the counts were identical in every run.
// Variance sources: `stat` depends on the filesystem under the temporary directory (see
//  `STAT_WITHOUT_FILESYSTEM_UUID`); another Git than 2.56.0 may start other children (`gc`, `repack`) in the
//  foreground maintenance of a commit, and the gate's Git is the authority for the budget.

/// The `stat` starts of a commit on a filesystem that has no UUID (tmpfs, overlayfs): three filesystem identities
///  fall back to `stat --file-system`. The test expects none on a filesystem with a UUID.
const STAT_WITHOUT_FILESYSTEM_UUID: usize = 3;

/// What one scenario may cost.
struct Budget {
    /// Starts of Git by subcommand.
    git_starts: &'static [(&'static str, usize)],
    /// Starts of other programs by file name (the test's own `timeout` and the wrapper are not counted).
    other_starts: &'static [(&'static str, usize)],
    /// Sync-class calls made by the wrapper process and its threads.
    wrapper_syncs: usize,
    /// Sync-class calls made by Git processes, by the subcommand that made them.
    git_syncs: &'static [(&'static str, usize)],
}

/// A commit with no hooks.
const NO_HOOKS: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 3),
        ("commit", 1),
        ("config", 1),
        ("diff", 1),
        ("diff-index", 4),
        ("diff-tree", 1),
        ("for-each-ref", 1),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 1),
        ("init", 1),
        ("ls-files", 4),
        ("maintenance run", 1),
        ("pack-objects", 1),
        ("read-tree", 1),
        ("remote", 1),
        ("reset", 1),
        ("rev-parse", 8),
        ("show", 1),
        ("symbolic-ref", 1),
        ("update-index", 1),
        ("update-ref", 2),
        ("version", 1),
        ("write-tree", 1),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", STAT_WITHOUT_FILESYSTEM_UUID)],
    wrapper_syncs: 33,
    git_syncs: &[("index-pack", 3)],
};

/// A commit with one hook defined in the configuration (`hook.early.event` and `hook.early.command`):
///  the hook runs under `sh`, and each of the four events has a `hook run` start.
const ONE_CONFIG_HOOK: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 3),
        ("commit", 1),
        ("config", 1),
        ("diff", 1),
        ("diff-index", 4),
        ("diff-tree", 1),
        ("for-each-ref", 1),
        ("hash-object", 0),
        ("hook run", 4),
        ("index-pack", 1),
        ("init", 1),
        ("ls-files", 4),
        ("maintenance run", 1),
        ("pack-objects", 1),
        ("read-tree", 1),
        ("remote", 1),
        ("reset", 1),
        ("rev-parse", 8),
        ("show", 2),
        ("symbolic-ref", 1),
        ("update-index", 1),
        ("update-ref", 2),
        ("version", 1),
        ("write-tree", 1),
    ],
    other_starts: &[("findmnt", 0), ("sh", 1), ("stat", STAT_WITHOUT_FILESYSTEM_UUID)],
    wrapper_syncs: 37,
    git_syncs: &[("index-pack", 3)],
};

/// An amending commit with no hooks.
const AMEND: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 3),
        ("commit", 1),
        ("config", 1),
        ("diff", 1),
        ("diff-index", 4),
        ("diff-tree", 1),
        ("for-each-ref", 1),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 1),
        ("init", 1),
        ("ls-files", 4),
        ("maintenance run", 1),
        ("pack-objects", 1),
        ("read-tree", 1),
        ("remote", 1),
        ("reset", 1),
        ("rev-parse", 8),
        ("show", 1),
        ("symbolic-ref", 1),
        ("update-index", 0),
        ("update-ref", 2),
        ("version", 1),
        ("write-tree", 1),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", STAT_WITHOUT_FILESYSTEM_UUID)],
    wrapper_syncs: 33,
    git_syncs: &[("index-pack", 3)],
};
//endregion

/// Which scenario to measure.
#[derive(Clone, Copy)]
enum Scenario {
    /// A new commit, no hooks.
    NoHooks,
    /// A new commit with one configuration-defined `pre-commit` hook.
    OneConfigHook,
    /// `--amend` of the commit before it, no hooks.
    Amend,
}

/// A repository whose `a.txt` is ready for the explicit-path commit of `scenario`.
///  Automatic maintenance stays in the foreground: Git 2.56.0 otherwise detaches a `repack` or
///  `multi-pack-index` after a landing, which may or may not have started when the trace ends and would make the
///  counts differ from run to run; in the foreground its children belong to the commit and are counted with it.
fn prepared_repository(fixture: &Fixture, scenario: Scenario) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("budget"));
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    match scenario {
        Scenario::NoHooks => {}
        Scenario::OneConfigHook => {
            git(fixture, repo.as_path(), &["config", "hook.early.event", "pre-commit"]);
            let log: String = format!("echo ran >> '{}'", fixture.root.join("hook.log").display());
            git(fixture, repo.as_path(), &["config", "hook.early.command", log.as_str()]);
        }
        Scenario::Amend => {
            git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base", "--", "a.txt"]);
            std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
        }
    }
    return repo;
}

/// The same command run under `strace` with the traced calls written to `trace`, in the environment and directory of
///  `inner`.
fn traced(inner: &Command, trace: &Path) -> Command {
    let mut outer: Command = Command::new("strace");
    outer
        .arg("--follow-forks")
        .arg("--quiet")
        .arg("--quiet")
        .arg("--string-limit=4096")
        .arg("--no-abbrev")
        .arg(format!("--output={}", trace.display()))
        .arg(format!("--trace={TRACED_CALLS}"))
        .arg(inner.get_program())
        .args(inner.get_args());
    outer.env_clear();
    for (name, setting) in inner.get_envs() {
        if let Some(text) = setting {
            outer.env(name, text);
        }
    }
    if let Some(directory) = inner.get_current_dir() {
        outer.current_dir(directory);
    }
    return outer;
}

/// The lines `name: count` of a section, for every name in the budget or in what was observed, in name order.
///  A name the budget lists at zero and the commit never started prints `name: 0`.
fn section(budgeted: &[(&str, usize)], observed: &BTreeMap<String, usize>) -> Vec<String> {
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    for (name, _allowed) in budgeted {
        names.insert(String::from(*name), observed.get(*name).copied().unwrap_or(0));
    }
    for (name, seen) in observed {
        names.insert(name.clone(), *seen);
    }
    return names.iter().map(|(name, seen)| { return format!("  {name}: {seen}"); }).collect();
}

/// The text of a budget as a report.
///  `other_starts` is the budget's list after the filesystem adjustment of `other_starts_expected`.
fn budget_report(budget: &Budget, other_starts: &[(&str, usize)]) -> String {
    let expected = |list: &[(&str, usize)]| -> Vec<String> {
        let mut lines: Vec<(String, usize)> = list.iter().map(|(name, allowed)| { return (String::from(*name), *allowed); }).collect();
        lines.sort();
        return lines.iter().map(|(name, allowed)| { return format!("  {name}: {allowed}"); }).collect();
    };
    return format!(
        "git starts:\n{}\nother starts:\n{}\nwrapper syncs: {}\ngit syncs:\n{}\n",
        expected(budget.git_starts).join("\n"),
        expected(other_starts).join("\n"),
        budget.wrapper_syncs,
        expected(budget.git_syncs).join("\n"),
    );
}

/// The text of what was observed, with the same names as the budget's report and zeros where the budget names
///  something the commit did not do.
fn observed_report(budget: &Budget, other_starts: &[(&str, usize)], counts: &Counts) -> String {
    return format!(
        "git starts:\n{}\nother starts:\n{}\nwrapper syncs: {}\ngit syncs:\n{}\n",
        section(budget.git_starts, &counts.git_starts).join("\n"),
        section(other_starts, &counts.other_starts).join("\n"),
        counts.wrapper_syncs,
        section(budget.git_syncs, &counts.git_syncs).join("\n"),
    );
}

/// Whether the filesystem holding `directory` has a UUID, which is what lets the wrapper identify it without `stat`.
fn filesystem_has_uuid(directory: &Path) -> bool {
    let probe: Result<Output, std::io::Error> =
        Command::new("findmnt").arg("--target").arg(directory).arg("--output=UUID").arg("--noheadings").output();
    return probe.is_ok_and(|output| { return !String::from_utf8_lossy(&output.stdout).trim().is_empty(); });
}

/// The budget's other-program starts, with `stat` at zero when the fixture's filesystem has a UUID.
fn other_starts_expected(budget: &Budget, fixture: &Fixture) -> Vec<(&'static str, usize)> {
    let uuid: bool = filesystem_has_uuid(fixture.root.as_path());
    return budget
        .other_starts
        .iter()
        .map(|(name, allowed)| { return (*name, if *name == "stat" && uuid { 0 } else { *allowed }); })
        .collect();
}

/// Sync-class calls made by programs that are neither the wrapper nor Git, which no budget allows.
fn assert_no_other_syncs(counts: &Counts, context: &str) {
    assert!(counts.other_syncs.is_empty(), "{context}: sync calls by other programs: {:?}", counts.other_syncs);
}

/// Run the scenario's commit under `strace` and require its counts to equal `budget`.
fn require_budget(label: &str, scenario: Scenario, budget: &Budget) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start or sync call was counted");
        return;
    }
    let fixture: Fixture = fixture(format!("budget-{label}").as_str());
    let repo: PathBuf = prepared_repository(&fixture, scenario);
    let trace: PathBuf = fixture.root.join("trace.txt");
    let arguments: &[&str] = match scenario {
        Scenario::Amend => &["commit", "--amend", "--quiet", "--message=x", "--", "a.txt"],
        Scenario::NoHooks | Scenario::OneConfigHook => &["commit", "--quiet", "--message=x", "--", "a.txt"],
    };
    let inner: Command = wrapped_commit(&fixture, repo.as_path(), arguments);
    let run: Output = traced(&inner, trace.as_path()).output().expect("strace starts");
    // Positive controls: a commit that stopped early would show a flatteringly small trace.
    assert_eq!(run.status.code(), Some(0), "{label}: the commit failed: {}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2", "{label}: commits");
    if matches!(scenario, Scenario::OneConfigHook) {
        let ran: String = std::fs::read_to_string(fixture.root.join("hook.log")).unwrap_or_default();
        assert_eq!(ran, "ran\n", "{label}: the configured hook must have run once");
    }
    assert_nothing_left(&fixture, repo.as_path());
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts: Counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    let version: String = String::from_utf8_lossy(&Command::new(REAL_GIT).arg("--version").output().expect("git --version").stdout)
        .trim()
        .to_string();
    assert_no_other_syncs(&counts, label);
    let other_starts: Vec<(&str, usize)> = other_starts_expected(budget, &fixture);
    let observed: String = observed_report(budget, other_starts.as_slice(), &counts);
    assert_eq!(
        observed,
        budget_report(budget, other_starts.as_slice()),
        "{label} ({version}): the counts differ from the budget (left: observed, right: budget). A saving lowers the \
         constant in binary_budget_tests.rs; a rise needs a recorded reason.\nstarted Git commands:\n{}\nstderr: {}",
        counts.commands.join("\n"),
        String::from_utf8_lossy(&run.stderr),
    );
    remove_settled(&fixture);
}

/// A commit with no hooks starts and syncs exactly what the budget names.
#[test]
fn a_commit_with_no_hooks_stays_within_its_budget() {
    require_budget("no-hooks", Scenario::NoHooks, &NO_HOOKS);
}

/// A commit with one configuration-defined hook starts and syncs exactly what the budget names.
#[test]
fn a_commit_with_one_config_hook_stays_within_its_budget() {
    require_budget("one-config-hook", Scenario::OneConfigHook, &ONE_CONFIG_HOOK);
}

/// An amending commit starts and syncs exactly what the budget names.
#[test]
fn an_amending_commit_stays_within_its_budget() {
    require_budget("amend", Scenario::Amend, &AMEND);
}
