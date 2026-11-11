//! Ratchet budgets for the cost of the wrapper's side of one commit:
//! how many processes it starts and how many fsync-class
//!  calls it makes,
//! so a change that makes a commit dearer fails a test that names the cause.
//! The service lands the commit
//!  and runs the work after it in a process of its own,
//! outside this trace.
//! Each control runs the real wrapped binary under `strace` on a disposable repository (an explicit-path commit,
//!  `git commit --quiet --message=x -- a.txt`),
//! reads every `execve` and sync call of the whole process tree,
//! and
//!  compares exact per-subcommand counts with the budget constants below.
//! The counts are exact,
//! not upper bounds:
//! a commit that got cheaper fails too,
//! so the constant is lowered in the
//!  same change and the saving cannot be given back unnoticed.
//! The controls need `strace`;
//! a host without it reports that and checks nothing,
//! so the host suite is where they run.

use super::budget_trace::{Counts, TRACED_CALLS, count};
use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::strace::{strace_available, strace_program};
use super::support::{Fixture, REAL_GIT, direct, fixture, git, repository};
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
// `check-ignore` is one of those zeros, and its reason is a rule: the ignored-path question of an explicit-path
//  commit (`commit_named_ignored_files.rs`) is decided from the real index the wrapper reads in process, so a commit
//  whose named paths the index holds starts no such child, and a named path no entry holds starts exactly one. Every
//  scenario here commits a tracked `a.txt`, so a start of one is the failure the release gate reported at
//  `483bbc6587` and the reading of the rule it broke.
// The stat-data refresh that keeps the scratch commit inside its ceiling (`commit_index_stat_staleness.rs`) asks for Git
//  only when the real index no longer describes the worktree, and every repository here has an index Git has just
//  written, so the `update-index` counts are the ones measured before that refresh existed.
// `wrapper_syncs` is 0, and must stay 0: the verdict a commit's own post-commit gate reaches is written into the held
//  push record through `replace_private_file_unsynced` (`work_item_holding.rs`, `record_verdict`), one rename that
//  flushes neither the record nor its directory, because native Git's commit makes no sync
//  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`, "A commit"). A refusal is the one verdict that waits for
//  the disk (`record_refusal`), and no scenario here refuses.
// Measured with host Git 2.56.0 on Linux, maintenance kept in the foreground (see `prepared_repository`), over five
//  runs of every scenario; the counts were identical in every run.
// Variance sources: another Git than 2.56.0 may start other children (`gc`, `repack`) in the foreground maintenance
//  of a commit, and the gate's Git is the authority for the budget. The filesystem under the temporary directory is
//  no source: the degraded `f_fsid` identity of a filesystem without a UUID (tmpfs, overlayfs) is read in-process,
//  so `stat` starts on none.

/// What one scenario may cost.
struct Budget {
    /// Starts of Git by subcommand.
    git_starts: &'static [(&'static str, usize)],
    /// Starts of other programs by file name (the test's own `timeout` and the wrapper are not counted).
    other_starts: &'static [(&'static str, usize)],
    /// Sync-class calls made by the wrapper process and its threads.
    wrapper_syncs: usize,
    /// Sync-class calls made by Git processes,
    /// by the subcommand that made them.
    git_syncs: &'static [(&'static str, usize)],
}

/// A commit with no hooks.
const NO_HOOKS: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 2),
        ("check-ignore", 0),
        ("commit", 0),
        ("config", 0),
        ("diff", 0),
        ("diff-index", 1),
        ("diff-tree", 0),
        ("for-each-ref", 0),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 0),
        ("init", 0),
        ("ls-files", 0),
        ("maintenance run", 0),
        ("pack-objects", 0),
        ("read-tree", 1),
        ("remote", 0),
        ("reset", 0),
        ("rev-parse", 1),
        ("show", 0),
        ("symbolic-ref", 0),
        ("update-index", 1),
        ("update-ref", 0),
        ("var", 2),
        ("version", 1),
        ("write-tree", 0),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", 0)],
    wrapper_syncs: 0,
    git_syncs: &[("index-pack", 0)],
};

/// A commit in a repository that configures a `post-commit` hook:
/// the hook is switched off and the loud notice costs
///  no process,
/// so every count equals the commit with no hooks.
const CONFIGURED_HOOK: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 2),
        ("check-ignore", 0),
        ("commit", 0),
        ("config", 0),
        ("diff", 0),
        ("diff-index", 1),
        ("diff-tree", 0),
        ("for-each-ref", 0),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 0),
        ("init", 0),
        ("ls-files", 0),
        ("maintenance run", 0),
        ("pack-objects", 0),
        ("read-tree", 1),
        ("remote", 0),
        ("reset", 0),
        ("rev-parse", 1),
        ("show", 0),
        ("symbolic-ref", 0),
        ("update-index", 1),
        ("update-ref", 0),
        ("var", 2),
        ("version", 1),
        ("write-tree", 0),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", 0)],
    wrapper_syncs: 0,
    git_syncs: &[("index-pack", 0)],
};

/// An amending commit with no hooks.
const AMEND: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 3),
        ("check-ignore", 0),
        ("commit", 0),
        ("config", 0),
        ("diff", 0),
        ("diff-index", 1),
        ("diff-tree", 0),
        ("for-each-ref", 0),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 0),
        ("init", 0),
        ("ls-files", 0),
        ("maintenance run", 0),
        ("pack-objects", 0),
        ("read-tree", 0),
        ("remote", 0),
        ("reset", 0),
        ("rev-parse", 3),
        ("show", 0),
        ("symbolic-ref", 0),
        ("update-index", 0),
        ("update-ref", 0),
        ("var", 1),
        ("version", 1),
        ("write-tree", 1),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", 0)],
    wrapper_syncs: 0,
    git_syncs: &[("index-pack", 0)],
};

/// A second wrapped commit in the repository,
/// no hooks:
/// what a first commit creates exists already,
/// and the work after
///  the landing (the push,
/// the upkeep) is the service's,
/// not the wrapper's.
const WARM: Budget = Budget {
    git_starts: &[
        ("add", 1),
        ("cat-file", 2),
        ("check-ignore", 0),
        ("commit", 0),
        ("config", 0),
        ("diff", 0),
        ("diff-index", 1),
        ("diff-tree", 0),
        ("for-each-ref", 0),
        ("hash-object", 0),
        ("hook run", 0),
        ("index-pack", 0),
        ("init", 0),
        ("ls-files", 0),
        ("maintenance run", 0),
        ("multi-pack-index write", 0),
        ("pack-objects", 0),
        ("read-tree", 0),
        ("remote", 0),
        ("repack", 0),
        ("reset", 0),
        ("rev-parse", 1),
        ("show", 0),
        ("symbolic-ref", 0),
        ("update-index", 0),
        ("update-ref", 0),
        ("var", 2),
        ("version", 0),
        ("write-tree", 0),
    ],
    other_starts: &[("findmnt", 0), ("sh", 0), ("stat", 0)],
    wrapper_syncs: 0,
    git_syncs: &[("index-pack", 0), ("multi-pack-index write", 0), ("pack-objects", 0)],
};
//endregion

/// Which scenario to measure.
#[derive(Clone, Copy)]
enum Scenario {
    /// A new commit,
    /// no hooks.
    NoHooks,
    /// A new commit,
    /// no hooks,
    /// in a repository with `extensions.worktreeConfig` on and a `config.worktree`.
    WorktreeConfig,
    /// A new commit,
    /// no hooks,
    /// in a repository whose configuration has `core.fsmonitor = true`.
    MonitorOn,
    /// A new commit in a repository that configures a `post-commit` hook,
    /// which never runs.
    ConfiguredHook,
    /// The second wrapped commit of the repository,
    /// no hooks.
    Warm,
    /// `--amend` of the commit before it,
    /// no hooks.
    Amend,
}

/// A repository whose `a.txt` is ready for the explicit-path commit of `scenario`.
///  Automatic maintenance stays in the foreground:
/// Git 2.56.0 otherwise detaches a `repack` or
///  `multi-pack-index` after a landing,
/// which may or may not have started when the trace ends and would make the
///  counts differ from run to run;
/// in the foreground its children belong to the commit and are counted with it.
fn prepared_repository(fixture: &Fixture, scenario: Scenario) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("budget"));
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    match scenario {
        Scenario::NoHooks => {}
        Scenario::WorktreeConfig => {
            git(fixture, repo.as_path(), &["config", "extensions.worktreeConfig", "true"]);
            std::fs::write(repo.join(".git/config.worktree"), b"[user]\n\tname = Only Here\n").expect("config.worktree");
        }
        Scenario::MonitorOn => {
            git(fixture, repo.as_path(), &["config", "core.fsmonitor", "true"]);
            // As for the warm scenario:
            // the first commit is not traced,
            // and it is the one that starts the monitor daemon,
            // which
            //  would otherwise stay in the trace as a child that never ends.
            let first: Output = wrapped_commit(fixture, repo.as_path(), &["commit", "--quiet", "--message=first", "--", "a.txt"])
                .output()
                .expect("the first wrapped commit runs");
            assert_eq!(first.status.code(), Some(0), "the first wrapped commit: {}", String::from_utf8_lossy(&first.stderr));
            std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
        }
        Scenario::ConfiguredHook => {
            git(fixture, repo.as_path(), &["config", "hook.early.event", "post-commit"]);
            let log: String = format!("echo ran >> '{}'", fixture.root.join("hook.log").display());
            git(fixture, repo.as_path(), &["config", "hook.early.command", log.as_str()]);
        }
        Scenario::Warm => {
            // The first wrapped commit is not traced; it creates what a first commit creates.
            let first: Output = wrapped_commit(fixture, repo.as_path(), &["commit", "--quiet", "--message=first", "--", "a.txt"])
                .output()
                .expect("the first wrapped commit runs");
            assert_eq!(first.status.code(), Some(0), "the first wrapped commit: {}", String::from_utf8_lossy(&first.stderr));
            std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
        }
        Scenario::Amend => {
            git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base", "--", "a.txt"]);
            std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
        }
    }
    return repo;
}

/// The same command run under `strace` with the traced calls written to `trace`,
/// in the environment and directory of
///  `inner`.
/// The trace keeps the `+++ exited with 0 +++` lines:
/// one `--quiet` suppresses only the attach and
///  personality messages,
/// and `binary_budget_trace` reads a line only when it begins with a process ID and a traced call.
fn traced(inner: &Command, trace: &Path) -> Command {
    let mut outer: Command = Command::new(strace_program().expect("strace is installed"));
    outer
        .arg("--follow-forks")
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

/// Every figure of a budget or of a trace,
/// keyed by what it counts.
type Figures = BTreeMap<String, usize>;

/// The figures a trace shows.
fn observed_figures(counts: &Counts) -> Figures {
    let mut figures: Figures = Figures::new();
    for (name, seen) in &counts.git_starts {
        figures.insert(format!("starts of git {name}"), *seen);
    }
    for (name, seen) in &counts.other_starts {
        figures.insert(format!("starts of {name}"), *seen);
    }
    figures.insert(String::from("sync calls by the wrapper"), counts.wrapper_syncs);
    for (name, seen) in &counts.git_syncs {
        figures.insert(format!("sync calls by git {name}"), *seen);
    }
    for (name, seen) in &counts.other_syncs {
        figures.insert(format!("sync calls by {name}"), *seen);
    }
    return figures;
}

/// The figures a budget allows;
/// `other_starts` is its list after the filesystem adjustment.
fn budget_figures(budget: &Budget, other_starts: &[(&str, usize)]) -> Figures {
    let mut figures: Figures = Figures::new();
    for (name, allowed) in budget.git_starts {
        figures.insert(format!("starts of git {name}"), *allowed);
    }
    for (name, allowed) in other_starts {
        figures.insert(format!("starts of {name}"), *allowed);
    }
    figures.insert(String::from("sync calls by the wrapper"), budget.wrapper_syncs);
    for (name, allowed) in budget.git_syncs {
        figures.insert(format!("sync calls by git {name}"), *allowed);
    }
    return figures;
}

/// One line for each figure that differs,
/// naming what it counts;
/// a figure missing on one side counts as zero there.
fn differences(budgeted: &Figures, observed: &Figures) -> Vec<String> {
    let mut keys: Vec<&String> = budgeted.keys().chain(observed.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut lines: Vec<String> = Vec::new();
    for key in keys {
        let allowed: usize = budgeted.get(key).copied().unwrap_or(0);
        let seen: usize = observed.get(key).copied().unwrap_or(0);
        if allowed != seen {
            let unlisted: &str = if budgeted.contains_key(key) { "" } else { " (not in the budget)" };
            lines.push(format!("  {key}: observed {seen}, budget {allowed}{unlisted}"));
        }
    }
    return lines;
}

/// Stops the monitor daemon a scenario may have started in `repo`,
/// whether the control passed or not.
struct StopsMonitor<'a> {
    fixture: &'a Fixture,
    repo: PathBuf,
    started: bool,
}

impl Drop for StopsMonitor<'_> {
    fn drop(&mut self) {
        if self.started {
            // No daemon running is not a failure of the control,
            // and stopping it is the whole job.
            let _stopped = direct(self.fixture).current_dir(self.repo.as_path()).args(["fsmonitor--daemon", "stop"]).output();
        }
    }
}

/// Run the scenario's commit under `strace` and require its counts to equal `budget`.
fn require_budget(label: &str, scenario: Scenario, budget: &Budget) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start or sync call was counted");
        return;
    }
    let fixture: Fixture = fixture(format!("budget-{label}").as_str());
    let repo: PathBuf = prepared_repository(&fixture, scenario);
    let stops: StopsMonitor<'_> = StopsMonitor { fixture: &fixture, repo: repo.clone(), started: matches!(scenario, Scenario::MonitorOn) };
    let trace: PathBuf = fixture.root.join("trace.txt");
    let arguments: &[&str] = match scenario {
        Scenario::Amend => &["commit", "--amend", "--quiet", "--message=x", "--", "a.txt"],
        Scenario::NoHooks | Scenario::WorktreeConfig | Scenario::MonitorOn | Scenario::ConfiguredHook | Scenario::Warm => &["commit", "--quiet", "--message=x", "--", "a.txt"],
    };
    let inner: Command = wrapped_commit(&fixture, repo.as_path(), arguments);
    let run: Output = traced(&inner, trace.as_path()).output().expect("strace starts");
    // The repository starts with one commit; the warm scenario adds an untraced one before the traced commit.
    let commits: &str = if matches!(scenario, Scenario::Warm | Scenario::MonitorOn) { "3" } else { "2" };
    // Positive controls: a commit that stopped early would show a flatteringly small trace.
    assert_eq!(run.status.code(), Some(0), "{label}: the commit failed: {}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), commits, "{label}: commits");
    if matches!(scenario, Scenario::ConfiguredHook) {
        assert!(!fixture.root.join("hook.log").exists(), "{label}: the configured hook must not have run");
    }
    assert_nothing_left(&fixture, repo.as_path());
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts: Counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    let version: String = String::from_utf8_lossy(&Command::new(REAL_GIT).arg("--version").output().expect("git --version").stdout)
        .trim()
        .to_string();
    let differing: Vec<String> =
        differences(&budget_figures(budget, budget.other_starts), &observed_figures(&counts));
    assert!(
        differing.is_empty(),
        "{label} ({version}): the counts differ from the budget in binary_budget_tests.rs. A saving lowers the constant; \
         a rise needs a recorded reason.\n{}\nstarted Git commands:\n{}\nstderr: {}",
        differing.join("\n"),
        counts.commands.join("\n"),
        String::from_utf8_lossy(&run.stderr),
    );
    // The daemon watches the repository,
    // so it stops before the fixture goes.
    drop(stops);
    remove_settled(&fixture);
}

/// A commit with no hooks starts and syncs exactly what the budget names.
#[test]
fn a_commit_with_no_hooks_stays_within_its_budget() {
    require_budget("no-hooks", Scenario::NoHooks, &NO_HOOKS);
}

/// A commit in a repository with a per-worktree configuration costs exactly what a commit without one costs:
/// the
///  layout,
/// the configuration listing and the upstream are read in process,
/// so no extra `rev-parse` or `config` starts.
#[test]
fn a_commit_with_a_worktree_config_costs_what_one_without_costs() {
    require_budget("worktree-config", Scenario::WorktreeConfig, &NO_HOOKS);
}

/// The second wrapped commit of a repository whose configuration has `core.fsmonitor = true` costs exactly what one without it
///  costs:
/// the
///  reader used to decline the value,
/// and thirteen more Git processes (`rev-parse`,
/// `diff-index`,
/// `ls-files`,
/// `config`,
/// `diff` and
///  `diff-tree` starts) answered the questions it would have answered from the files.
/// The wrapper's own Git passes `-c core.fsmonitor=false` (`hooks_off.rs`),
/// so the commit starts no daemon either;
/// the setting reaches only the person's own commands.
#[test]
fn a_commit_with_core_fsmonitor_true_costs_what_one_without_costs() {
    require_budget("monitor-on", Scenario::MonitorOn, &WARM);
}

/// A commit in a repository with a configured hook starts and syncs exactly what the budget names.
#[test]
fn a_commit_with_a_configured_hook_stays_within_its_budget() {
    require_budget("configured-hook", Scenario::ConfiguredHook, &CONFIGURED_HOOK);
}

/// The second wrapped commit of a repository starts and syncs exactly what the budget names.
#[test]
fn a_warm_commit_stays_within_its_budget() {
    require_budget("warm", Scenario::Warm, &WARM);
}

/// An amending commit starts and syncs exactly what the budget names.
#[test]
fn an_amending_commit_stays_within_its_budget() {
    require_budget("amend", Scenario::Amend, &AMEND);
}

/// The wrapped commit with the system configuration file left in play (no `GIT_CONFIG_NOSYSTEM`),
/// run under `strace`.
///  The wrapper process itself reads the repository in process (the transaction,
/// the landing,
/// the repository facts) and
///  must know which system file the real Git would read;
/// it learns that from the real Git named by the forward-target
///  variable in the environment `entry.rs` builds,
/// once per executable,
/// and keeps it in the user's cache directory.
fn traced_commit_with_the_system_file(label: &str, learned_before: bool) -> Counts {
    let fixture: Fixture = fixture(format!("budget-{label}").as_str());
    let repo: PathBuf = prepared_repository(&fixture, Scenario::NoHooks);
    let system_file_in_play = |command: &mut Command| {
        command.env_remove("GIT_CONFIG_NOSYSTEM");
    };
    if learned_before {
        let mut first: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=first", "--", "a.txt"]);
        system_file_in_play(&mut first);
        let done: Output = first.output().expect("the first wrapped commit runs");
        assert_eq!(done.status.code(), Some(0), "{label}: {}", String::from_utf8_lossy(&done.stderr));
        std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    }
    let trace: PathBuf = fixture.root.join("trace.txt");
    let mut inner: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    system_file_in_play(&mut inner);
    let run: Output = traced(&inner, trace.as_path()).output().expect("strace starts");
    assert_eq!(run.status.code(), Some(0), "{label}: the commit failed: {}", String::from_utf8_lossy(&run.stderr));
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts: Counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    remove_settled(&fixture);
    return counts;
}

/// With the system file in play the in-process reads still answer:
/// the read-only starts equal the budget of a commit
///  whose system file is switched off,
/// and the one extra start is the question for the compiled-in path,
/// asked once.
#[test]
fn the_wrapper_learns_the_system_config_path_once_and_its_reads_still_answer() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start was counted");
        return;
    }
    let read_only: [&str; 9] = ["config", "diff", "diff-index", "diff-tree", "for-each-ref", "ls-files", "remote", "show", "symbolic-ref"];
    for (label, learned_before) in [("system-file-cold", false), ("system-file-warm", true)] {
        let counts: Counts = traced_commit_with_the_system_file(label, learned_before);
        for name in read_only {
            let allowed: usize = NO_HOOKS.git_starts.iter().find(|(known, _)| { return *known == name; }).map_or(0, |(_, allowed)| { return *allowed; });
            assert_eq!(counts.git_starts.get(name).copied().unwrap_or(0), allowed, "{label}: starts of git {name}\n{}", counts.commands.join("\n"));
        }
        // The wrapper also asks Git for the author and committer identities (`git var GIT_AUTHOR_IDENT`); only the question about
        //  the system file counts here.
        let asked: usize = counts.commands.iter().filter(|command| { return command.contains("var GIT_CONFIG_SYSTEM"); }).count();
        assert_eq!(asked, usize::from(!learned_before), "{label}: starts of git var GIT_CONFIG_SYSTEM\n{}", counts.commands.join("\n"));
    }
}

/// How many times Git was started for `var`,
/// whichever variable it was asked for.
fn var_starts(counts: &Counts) -> usize {
    return counts.git_starts.iter().filter(|(name, _)| { return name.starts_with("var"); }).map(|(_, seen)| { return *seen; }).sum();
}

/// Runs the wrapped commit under `strace`,
/// with a service listening for it,
/// and returns what the trace shows.
fn traced_through_the_service(label: &str, detached: bool) -> (Counts, Output) {
    let fixture: Fixture = fixture(format!("budget-service-{label}").as_str());
    let repo: PathBuf = prepared_repository(&fixture, Scenario::NoHooks);
    if detached {
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "--detach"]);
    }
    let name: String = format!("budget-{label}-{}", std::process::id());
    let mut service = super::service_commit::start(&fixture, name.as_str());
    let mut inner: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    inner.env("XDG_RUNTIME_DIR", super::service_commit::runtime_for(name.as_str()));
    let trace: PathBuf = fixture.root.join("trace.txt");
    let run: Output = traced(&inner, trace.as_path()).output().expect("strace starts");
    assert_eq!(run.status.code(), Some(0), "{label}: the commit failed: {}", String::from_utf8_lossy(&run.stderr));
    let text: String = std::fs::read_to_string(&trace).expect("trace");
    let counts: Counts = count(text.as_str(), fixture.root.to_str().expect("UTF-8 root"), REAL_GIT);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
    return (counts, run);
}

/// A commit the service builds from the command line asks Git for the two identities,
/// and a commit it declines (here a
///  detached head,
/// which the transaction commits) asks for none:
/// the cheap declines come before anything starts Git for a
///  message or an identity.
#[test]
fn a_commit_the_service_declines_starts_no_git_for_an_identity() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start was counted");
        return;
    }
    let (taken, _) = traced_through_the_service("taken", false);
    assert_eq!(var_starts(&taken), 2, "the committer and the author: {:?}", taken.commands);
    let (declined, _) = traced_through_the_service("declined", true);
    assert_eq!(var_starts(&declined), 0, "{:?}", declined.commands);
}
