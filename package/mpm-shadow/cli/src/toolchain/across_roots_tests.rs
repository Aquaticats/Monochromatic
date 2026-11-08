//! Unit tests of the rule for this repository's own tools across both toolchain roots:
//! the newest finished copy wins wherever it is,
//! a scope whose lock pins an older version installs nothing,
//! runs the newer copy and warns with its own upgrade command,
//! and the older managed copy is removed after its links moved,
//! never when it could be the only working copy.
//! Every fixture sits in a disposable home with cargo faked.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::across_roots::{winning_copy, winning_root, winning_version};
use super::cargo_installs::{current_link, older_lock_message_for, point_current_at_newest, version_directory};
use super::context::Context;
use super::fakes::{Rig, write_executable};
use super::fakes_cargo::{fake_binstall, finish_install};
use super::launcher::{LaunchRecord, parse_record, prepare_launch, record_path, warning_naming_program};
use super::layout::find_tool_bin;
use super::lock::{Lock, render_lock};
use super::model::LockedTool;
use super::observe::observe_tool;
use super::old_copy_apply::{apply_old_copies, preview_lines, retire_old_copies};
use super::old_copies::plan_old_copies;
use super::plan::parse_plan;
use super::prune_cargo::prune_toolchain;
use super::published_archive::{current_link_of, point_current_at_newest_archive};
use super::published_archive_tests::{BIN as ARCHIVE_BIN, ID as ARCHIVE_ID, finish_archive, release_tool};
use super::scope::{Scope, ScopeKind, global_scope};
use super::status_cmd::run_status;
use super::sync::{FrozenReason, LockPolicy, run_sync_with};

/// The plan entry of the git wrapper.
const GIT_TOOL: &str = "cargo:monochromatic-git-policy-cli";

/// Its crate.
const GIT_CRATE: &str = "monochromatic-git-policy-cli";

/// Its bin.
const GIT_BIN: &str = "cli-git-native";

/// The repository plan:
/// the git wrapper with the postinstall link of the repository's plan.
const REPOSITORY_PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:monochromatic-git-policy-cli": {
      "request": "cargo:monochromatic-git-policy-cli@latest",
      "route": "cargo-binstall",
      "bins": ["cli-git-native"],
      "os": ["linux"],
      "options": {
        "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git", "linkDir": ".cache/cli-git-bin" } },
      },
    },
  },
}
"#;

/// The machine-global plan:
/// the same tool,
/// with the postinstall link in the global scope's default link directory (`~/.local/bin`).
const GLOBAL_PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:monochromatic-git-policy-cli": {
      "request": "cargo:monochromatic-git-policy-cli@latest",
      "route": "cargo-binstall",
      "bins": ["cli-git-native"],
      "os": ["linux"],
      "options": {
        "postinstall": { "when": "always", "link": { "sourceBin": "cli-git-native", "linkName": "git" } },
      },
    },
  },
}
"#;

/// A disposable home with a repository scope and a global scope that share it,
/// and so share the two toolchain roots.
struct Duo {
    /// The repository scope's rig (fakes,
    /// environment,
    /// disposable directory).
    rig: Rig,
    /// The global scope of the same home.
    global: Scope,
}

/// The methods of `Duo`.
impl Duo {
    /// A duo with `cargo binstall` faked.
    fn new() -> Duo {
        let rig = Rig::new();
        rig.runner.on("cargo binstall", fake_binstall(&[GIT_BIN]));
        let global = global_scope(&rig.env).expect("a global scope");
        return Duo { rig, global };
    }

    /// The repository scope's toolchain root.
    fn repository_root(&self) -> PathBuf {
        return self.rig.scope.toolchain_root.clone();
    }

    /// The global scope's toolchain root.
    fn global_root(&self) -> PathBuf {
        return self.global.toolchain_root.clone();
    }

    /// The lock of the plan with the tool at `version`.
    fn lock_of(&self, plan: &str, version: &str) -> Lock {
        let mut tools: Vec<LockedTool> = Vec::new();
        for declaration in parse_plan(plan, "plan").expect("the plan parses").tools {
            tools.push(LockedTool { declaration, version: version.to_string(), artifacts: Vec::new() });
        }
        return Lock { tools };
    }

    /// Writes the repository scope's plan and a lock pinning `version`.
    fn lock_repository(&self, version: &str) -> Lock {
        let lock = self.lock_of(REPOSITORY_PLAN, version);
        fs::write(&self.rig.scope.plan_file, REPOSITORY_PLAN).expect("write the plan");
        fs::write(&self.rig.scope.lock_file, render_lock(&lock)).expect("write the lock");
        return lock;
    }

    /// Writes the global scope's plan and a lock pinning `version`.
    fn lock_global(&self, version: &str) -> Lock {
        let lock = self.lock_of(GLOBAL_PLAN, version);
        fs::create_dir_all(&self.global.root).expect("make the config directory");
        fs::write(&self.global.plan_file, GLOBAL_PLAN).expect("write the plan");
        fs::write(&self.global.lock_file, render_lock(&lock)).expect("write the lock");
        return lock;
    }

    /// Finishes an install of `version` under `root` and points that root's `current` at its newest.
    fn install_under(&self, root: &Path, version: &str) {
        finish_install(&version_directory(root, GIT_CRATE, version), GIT_CRATE, version, &[GIT_BIN]);
        point_current_at_newest(root, GIT_CRATE).expect("point current at the newest");
    }

    /// The managed copy as links reach it,
    /// through `current` of `root`.
    fn copy_in(&self, root: &Path) -> PathBuf {
        return current_link(root, GIT_CRATE).join("bin").join(GIT_BIN);
    }

    /// A context for `scope` and `lock`.
    fn context<'a>(&'a self, scope: &'a Scope, lock: &'a Lock) -> Context<'a> {
        return Context { scope, env: &self.rig.env, runner: &self.rig.runner, http: &self.rig.http, host: &self.rig.host, tools: &lock.tools };
    }

    /// Runs the repository scope's `install`.
    fn install_repository(&self) -> super::sync::SyncReport {
        return run_sync_with(&self.rig.scope, &self.rig.env, &self.rig.runner, &self.rig.http, &self.rig.host, LockPolicy::Frozen(FrozenReason::Flag))
            .expect("synced");
    }

    /// Runs the global scope's `install`.
    fn install_global(&self) -> super::sync::SyncReport {
        return run_sync_with(&self.global, &self.rig.env, &self.rig.runner, &self.rig.http, &self.rig.host, LockPolicy::Frozen(FrozenReason::Flag))
            .expect("synced");
    }

    /// How many times cargo was asked to install.
    fn installs(&self) -> usize {
        return self.rig.runner.lines().iter().filter(|line| return line.starts_with("cargo binstall")).count();
    }

    /// Makes git list the scope's own worktree only.
    fn list_own_worktree(&self) {
        self.rig.runner.answer("git -C", &format!("worktree {}\nHEAD 0000\n\n", self.rig.scope.root.display()), 0);
    }
}

/// The record beside a launcher.
fn launcher_record(launcher: &Path) -> LaunchRecord {
    let text: String = fs::read_to_string(record_path(launcher)).expect("read the record");
    return parse_record(&text).expect("a record");
}

/// Points `link` at `target`,
/// making the link's directory.
fn make_link(link: &Path, target: &Path) {
    fs::create_dir_all(link.parent().expect("a parent")).expect("make the directory");
    symlink(target, link).expect("make the link");
}

//region which copy wins

#[test]
fn the_newest_finished_copy_across_both_roots_wins_and_a_tie_goes_to_the_scope_that_asks() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    let tool = &lock.tools[0];
    assert_eq!(winning_copy(&duo.rig.scope, tool), None, "nothing installed anywhere");
    assert_eq!(winning_root(&duo.rig.scope, tool), duo.repository_root(), "nothing installed: the scope's own root");

    duo.install_under(&duo.repository_root(), "0.4.4");
    assert_eq!(winning_version(&duo.rig.scope, tool).as_deref(), Some("0.4.4"));
    assert_eq!(winning_root(&duo.rig.scope, tool), duo.repository_root());

    duo.install_under(&duo.global_root(), "0.4.5");
    let winner = winning_copy(&duo.rig.scope, tool).expect("a winner");
    assert_eq!((winner.version.as_str(), winner.root.clone()), ("0.4.5", duo.global_root()), "the newer one wins in the other root");
    assert_eq!(winning_copy(&duo.global, tool).expect("a winner"), winner, "from the global scope too");

    duo.install_under(&duo.repository_root(), "0.4.5");
    assert_eq!(winning_root(&duo.rig.scope, tool), duo.repository_root(), "a tie goes to the asking repository scope");
    assert_eq!(winning_root(&duo.global, tool), duo.global_root(), "and to the asking global scope");
}

#[test]
fn an_unfinished_newer_version_in_the_other_root_is_not_a_copy_that_wins() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.4");
    duo.install_under(&duo.repository_root(), "0.4.4");
    write_executable(&version_directory(&duo.global_root(), GIT_CRATE, "0.4.9").join("bin").join(GIT_BIN));
    assert_eq!(winning_version(&duo.rig.scope, &lock.tools[0]).as_deref(), Some("0.4.4"), "cargo never recorded the 0.4.9");
}

#[test]
fn the_older_lock_sentence_names_the_upgrade_command_of_the_scope_it_is_for() {
    let repository = older_lock_message_for(ScopeKind::Repo, GIT_TOOL, "0.4.4", "0.4.5");
    assert!(repository.contains("in this tree") && repository.contains(&format!("`mpm update {GIT_TOOL}` or merge main")), "{repository}");
    assert!(!repository.contains("update -g"), "{repository}");
    let global = older_lock_message_for(ScopeKind::Global, GIT_TOOL, "0.4.4", "0.4.5");
    assert!(global.contains("in this machine's global lock") && global.contains(&format!("`mpm update -g {GIT_TOOL}`")), "{global}");
    assert!(!global.contains("merge main"), "{global}");
}

//endregion

//region a scope whose lock pins an older version

#[test]
fn a_repository_scope_pinning_an_older_version_installs_nothing_runs_the_newer_global_copy_and_warns() {
    let duo = Duo::new();
    duo.lock_repository("0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert_eq!(duo.installs(), 0, "{:?}", duo.rig.runner.lines());
    assert!(report.installed.is_empty() && report.failed.is_empty(), "{report:?}");
    assert!(
        !version_directory(&duo.repository_root(), GIT_CRATE, "0.4.4").exists(),
        "the older version is neither installed nor kept"
    );
    assert!(report.warnings.iter().any(|warning| return warning.contains("locked at 0.4.4") && warning.contains("0.4.5 runs") && warning.contains(&format!("`mpm update {GIT_TOOL}`"))), "{report:?}");
    // The tree's exposed link and its postinstall link go through its launcher, whose record names the winner.
    let launcher = duo.rig.scope.launcher_dir.join(GIT_BIN);
    let record = launcher_record(&launcher);
    assert_eq!(record.real, duo.copy_in(&duo.global_root()));
    assert_eq!(record.scope, ScopeKind::Repo);
    assert_eq!(fs::read_link(duo.rig.scope.exposure_dir.join(GIT_BIN)).expect("a link"), launcher);
    assert_eq!(fs::read_link(duo.rig.scope.root.join(".cache/cli-git-bin/git")).expect("a link"), launcher);
}

#[test]
fn the_global_scope_does_the_same_with_the_global_upgrade_command_and_its_postinstall_link_follows_the_newest() {
    let duo = Duo::new();
    let lock = duo.lock_global("0.4.4");
    duo.install_under(&duo.repository_root(), "0.4.5");
    let report = duo.install_global();
    assert_eq!(duo.installs(), 0, "the global lock's 0.4.4 is not installed: {:?}", duo.rig.runner.lines());
    assert!(!version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists());
    assert!(report.failed.is_empty(), "{report:?}");
    let expected = older_lock_message_for(ScopeKind::Global, GIT_TOOL, "0.4.4", "0.4.5");
    assert!(report.warnings.contains(&expected), "{report:?}");

    let local_git = duo.rig.scope.home.join(".local/bin/git");
    let launcher = duo.global.launcher_dir.join(GIT_BIN);
    assert_eq!(fs::read_link(&local_git).expect("a link"), launcher, "~/.local/bin/git goes through the global launcher");
    let record = launcher_record(&launcher);
    assert_eq!(record.real, duo.copy_in(&duo.repository_root()), "the launcher runs the repository root's newest copy");
    assert_eq!(record.scope, ScopeKind::Global);
    let launch = prepare_launch(&record, Some(&fs::read_to_string(&duo.global.lock_file).expect("read the lock")));
    assert_eq!(
        launch.warning,
        Some(warning_naming_program(&expected, &launch.program)),
        "the launcher warns on every run with the global command, naming the program it starts"
    );
    assert!(launch.program.is_file());

    let listed = run_status(&duo.global, &duo.rig.env, &duo.rig.runner, &duo.rig.host).expect("listed");
    assert!(!listed.drifted, "{}", listed.text);
    assert!(listed.text.contains(&format!("`mpm update -g {GIT_TOOL}`")), "{}", listed.text);
    drop(lock);
}

#[test]
fn an_older_copy_in_the_other_root_does_not_satisfy_a_newer_lock_so_the_scope_installs_its_own() {
    let duo = Duo::new();
    duo.lock_repository("0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert_eq!(duo.installs(), 1, "the 0.4.4 elsewhere is older than the lock: {:?}", duo.rig.runner.lines());
    assert_eq!(report.installed, vec![GIT_TOOL.to_string()], "{report:?}");
    assert!(report.warnings.is_empty(), "this lock is the newest: {report:?}");
}

//endregion

//region removing the older managed copy

#[test]
fn a_newer_lock_removes_the_other_roots_older_copy_after_moving_its_links_and_the_emptied_root_forwards() {
    let duo = Duo::new();
    duo.lock_repository("0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    let older_file = version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").join("bin").join(GIT_BIN);
    let direct_link = duo.rig.scope.home.join(".local/bin/git");
    make_link(&direct_link, &older_file);
    let farm_link = duo.global.exposure_dir.join(GIT_BIN);
    make_link(&farm_link, &duo.copy_in(&duo.global_root()));
    let foreign = duo.rig.scope.home.join(".local/bin/other");
    write_executable(&duo.rig.scope.home.join(".local/share/other/tool"));
    make_link(&foreign, &duo.rig.scope.home.join(".local/share/other/tool"));
    // A launcher the global scope made earlier, and the record that names the global root's `current`.
    let launcher = duo.global.launcher_dir.join(GIT_BIN);
    fs::create_dir_all(&duo.global.launcher_dir).expect("make the launcher directory");
    fs::write(
        record_path(&launcher),
        format!("{{\"toolId\":\"{GIT_TOOL}\",\"lockFile\":\"{}\",\"real\":\"{}\",\"scope\":\"global\"}}", duo.global.lock_file.display(), duo.copy_in(&duo.global_root()).display()),
    )
    .expect("write the record");
    duo.list_own_worktree();

    let report = duo.install_repository();
    assert_eq!(report.installed, vec![GIT_TOOL.to_string()], "{report:?}");
    assert!(report.failed.is_empty(), "{report:?}");
    let winner = duo.copy_in(&duo.repository_root());
    assert_eq!(fs::read_link(&direct_link).expect("a link"), winner, "the direct link moved to the newest");
    assert_eq!(fs::read_link(&farm_link).expect("a link"), winner, "so did the global farm's link, which went through the other root's current");
    assert_eq!(fs::read_link(&foreign).expect("a link"), duo.rig.scope.home.join(".local/share/other/tool"), "a foreign link is untouched");
    assert!(!version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "the older managed copy is removed");
    assert_eq!(
        fs::read_link(current_link(&duo.global_root(), GIT_CRATE)).expect("a link"),
        current_link(&duo.repository_root(), GIT_CRATE),
        "the emptied root's current forwards to the winner's"
    );
    let record = launcher_record(&launcher);
    assert!(record.real.is_file(), "a launcher record that names the emptied root's current still runs the winner");
    assert!(report.old_copies.iter().any(|line| return line.starts_with("removed ") && line.contains("older managed copy")), "{:?}", report.old_copies);
    assert!(report.old_copies.iter().any(|line| return line.contains("now forwards to")), "{:?}", report.old_copies);
    let forwarded_at = report.old_copies.iter().position(|line| return line.contains("now forwards to")).expect("a forward line");
    let removed_at = report.old_copies.iter().position(|line| return line.starts_with("removed ")).expect("a removal line");
    assert!(forwarded_at < removed_at, "the forward comes before the removal, so no launcher sees a dangling window: {:?}", report.old_copies);

    let again = duo.install_repository();
    assert!(again.old_copies.is_empty() && again.warnings.is_empty() && again.failed.is_empty(), "a second run finds nothing: {again:?}");
}

#[test]
fn the_scopes_own_older_copy_goes_when_the_other_root_holds_a_newer_one() {
    let duo = Duo::new();
    duo.lock_repository("0.4.4");
    duo.install_under(&duo.repository_root(), "0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert_eq!(duo.installs(), 0);
    assert!(!version_directory(&duo.repository_root(), GIT_CRATE, "0.4.4").exists(), "this scope does not keep the older copy");
    assert_eq!(fs::read_link(current_link(&duo.repository_root(), GIT_CRATE)).expect("a link"), current_link(&duo.global_root(), GIT_CRATE));
    assert!(report.warnings.iter().any(|warning| return warning.contains("locked at 0.4.4")), "{report:?}");
    let launcher = fs::read_link(duo.rig.scope.exposure_dir.join(GIT_BIN)).expect("a link");
    assert_eq!(launcher_record(&launcher).real, duo.copy_in(&duo.global_root()), "the launcher runs the newer copy");
}

#[test]
fn a_forwarding_current_gives_way_when_the_scope_installs_a_newer_version_and_the_other_root_then_forwards() {
    let duo = Duo::new();
    duo.lock_repository("0.4.4");
    duo.install_under(&duo.repository_root(), "0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    duo.install_repository();
    assert_eq!(fs::read_link(current_link(&duo.repository_root(), GIT_CRATE)).expect("a link"), current_link(&duo.global_root(), GIT_CRATE));

    duo.lock_repository("0.4.6");
    let report = duo.install_repository();
    assert_eq!(report.installed, vec![GIT_TOOL.to_string()], "{report:?}");
    assert_eq!(fs::read_link(current_link(&duo.repository_root(), GIT_CRATE)).expect("a link"), PathBuf::from("0.4.6"), "a real current again");
    assert!(!version_directory(&duo.global_root(), GIT_CRATE, "0.4.5").exists(), "the global root's 0.4.5 is older now");
    assert_eq!(fs::read_link(current_link(&duo.global_root(), GIT_CRATE)).expect("a link"), current_link(&duo.repository_root(), GIT_CRATE));
    assert!(report.warnings.is_empty(), "this lock is the newest again: {report:?}");
}

#[test]
fn prune_leaves_a_crate_directory_that_only_forwards_to_the_other_root() {
    let duo = Duo::new();
    duo.lock_repository("0.4.4");
    duo.install_under(&duo.repository_root(), "0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    duo.install_repository();
    let forward = current_link(&duo.repository_root(), GIT_CRATE);
    assert!(fs::symlink_metadata(&forward).is_ok(), "the repository root now only forwards");
    duo.rig.runner.answer("git -C", &format!("worktree {}\nHEAD 0000\n\n", duo.rig.scope.root.display()), 0);
    prune_toolchain(&duo.rig.scope, &duo.rig.runner, &[], false).expect("pruned");
    assert!(fs::symlink_metadata(&forward).is_ok(), "nothing was pruned from the crate directory, so the forwarding link stays");
}

#[test]
fn a_newer_version_another_tree_installed_into_the_emptied_root_after_the_plan_is_not_overwritten_by_the_forward() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let plans = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    assert_eq!(plans.losers[0].kept, None, "{plans:?}");
    // Between the plan and its application the global scope installs a version newer than the winner.
    duo.install_under(&duo.global_root(), "0.4.6");
    let outcome = apply_old_copies(&duo.context(&duo.rig.scope, &lock), &plans);
    assert!(outcome.failed.is_empty(), "{outcome:?}");
    assert!(!version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "the older version still goes");
    assert_eq!(
        fs::read_link(current_link(&duo.global_root(), GIT_CRATE)).expect("a link"),
        PathBuf::from("0.4.6"),
        "the root's own current is not overwritten by a forward to the old winner"
    );
    assert!(!outcome.lines.iter().any(|line| return line.contains("now forwards to")), "{outcome:?}");
}

#[test]
fn a_version_directory_another_run_removed_first_is_done_and_not_a_failure() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let plans = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    fs::remove_dir_all(version_directory(&duo.global_root(), GIT_CRATE, "0.4.4")).expect("the other run removes it");
    let outcome = apply_old_copies(&duo.context(&duo.rig.scope, &lock), &plans);
    assert!(outcome.failed.is_empty(), "{outcome:?}");
}

#[test]
fn a_stale_current_in_the_winning_root_is_repaired_before_another_scope_links_the_copy() {
    let duo = Duo::new();
    duo.lock_repository("0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    // A run killed between the completion marker and the link: 0.4.5 is finished but `current` still names 0.4.4.
    finish_install(&version_directory(&duo.global_root(), GIT_CRATE, "0.4.4"), GIT_CRATE, "0.4.4", &[GIT_BIN]);
    fs::remove_file(current_link(&duo.global_root(), GIT_CRATE)).expect("drop the right link");
    symlink("0.4.4", current_link(&duo.global_root(), GIT_CRATE)).expect("leave the stale link");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(
        fs::read_link(current_link(&duo.global_root(), GIT_CRATE)).expect("a link"),
        PathBuf::from("0.4.5"),
        "the repository scope's install repaired the winning root's current"
    );
    let launcher = fs::read_link(duo.rig.scope.exposure_dir.join(GIT_BIN)).expect("a link");
    let resolved = fs::canonicalize(launcher_record(&launcher).real).expect("the launcher's program resolves");
    assert!(resolved.ends_with(Path::new("0.4.5").join("bin").join(GIT_BIN)), "the newest copy runs: {}", resolved.display());
}

#[test]
fn prune_never_removes_a_version_the_other_root_forwards_to() {
    let forwarded = Duo::new();
    fs::write(&forwarded.rig.scope.lock_file, "{\"schemaVersion\":1,\"tools\":{}}").expect("a lock that names nothing");
    forwarded.install_under(&forwarded.repository_root(), "0.4.5");
    fs::create_dir_all(forwarded.global_root().join("cargo").join(GIT_CRATE)).expect("make the global crate directory");
    symlink(current_link(&forwarded.repository_root(), GIT_CRATE), current_link(&forwarded.global_root(), GIT_CRATE)).expect("forward");
    forwarded.list_own_worktree();
    prune_toolchain(&forwarded.rig.scope, &forwarded.rig.runner, &[], false).expect("pruned");
    assert!(version_directory(&forwarded.repository_root(), GIT_CRATE, "0.4.5").exists(), "the global root runs it through its forward");
    assert!(fs::metadata(forwarded.copy_in(&forwarded.global_root())).is_ok(), "and still reaches it");

    // Rejected fixture: with no forward the same prune removes the version no lock names.
    let alone = Duo::new();
    fs::write(&alone.rig.scope.lock_file, "{\"schemaVersion\":1,\"tools\":{}}").expect("a lock that names nothing");
    alone.install_under(&alone.repository_root(), "0.4.5");
    alone.list_own_worktree();
    prune_toolchain(&alone.rig.scope, &alone.rig.runner, &[], false).expect("pruned");
    assert!(!version_directory(&alone.repository_root(), GIT_CRATE, "0.4.5").exists());
}

#[test]
fn equal_versions_in_both_roots_are_not_older_and_both_stay() {
    let duo = Duo::new();
    duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert!(report.old_copies.is_empty() && report.warnings.is_empty(), "{report:?}");
    assert!(version_directory(&duo.global_root(), GIT_CRATE, "0.4.5").exists());
    assert!(version_directory(&duo.repository_root(), GIT_CRATE, "0.4.5").exists());
}

#[test]
fn a_root_that_keeps_an_equal_version_loses_only_its_older_ones_and_is_not_made_to_forward() {
    let duo = Duo::new();
    duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.install_under(&duo.global_root(), "0.4.5");
    duo.list_own_worktree();
    let report = duo.install_repository();
    assert!(report.failed.is_empty(), "{report:?}");
    assert!(!version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "the older one goes");
    assert!(version_directory(&duo.global_root(), GIT_CRATE, "0.4.5").exists(), "the equal one stays");
    assert_eq!(
        fs::read_link(current_link(&duo.global_root(), GIT_CRATE)).expect("a link"),
        PathBuf::from("0.4.5"),
        "the global root still runs its own copy: nothing emptied it"
    );
}

#[test]
fn a_release_archive_tool_installed_only_in_the_other_root_is_found_and_observed_there() {
    let rig = Rig::new();
    let global = global_scope(&rig.env).expect("a global scope");
    let tool = release_tool(&rig, "0.2.1");
    finish_archive(&global.toolchain_root, "0.2.1");
    point_current_at_newest_archive(&global.toolchain_root, ARCHIVE_ID, ARCHIVE_BIN).expect("point current");
    let found = find_tool_bin(&tool, ARCHIVE_BIN, &rig.scope, &rig.env, &[]).expect("found in the other root");
    assert_eq!(found, current_link_of(&global.toolchain_root, ARCHIVE_ID).expect("a plain id").join("release-folder").join(ARCHIVE_BIN));
    let lock = Lock { tools: vec![tool] };
    let context = Context { scope: &rig.scope, env: &rig.env, runner: &rig.runner, http: &rig.http, host: &rig.host, tools: &lock.tools };
    assert_eq!(observe_tool(&context, &lock.tools[0]).as_deref(), Some("0.2.1"));
}

#[test]
fn an_older_copy_in_the_other_root_stays_while_a_place_could_not_be_searched_but_the_scopes_own_goes() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.rig.runner.fail("git -C", "fatal: not a git repository");
    let plans = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    assert_eq!(plans.losers.len(), 1, "{plans:?}");
    assert!(plans.losers[0].kept.as_deref().unwrap_or("").contains("another scope's links may point at it"), "{plans:?}");
    apply_old_copies(&duo.context(&duo.rig.scope, &lock), &plans);
    assert!(version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "kept: a checkout's launcher could be hiding where git would not tell");

    // The same search failure does not keep the scope's own older copy: nothing else points into it.
    let own = Duo::new();
    let own_lock = own.lock_repository("0.4.4");
    own.install_under(&own.repository_root(), "0.4.4");
    own.install_under(&own.global_root(), "0.4.5");
    own.rig.runner.fail("git -C", "fatal: not a git repository");
    let own_plans = plan_old_copies(&own.context(&own.rig.scope, &own_lock), &[]);
    assert_eq!(own_plans.losers[0].kept, None, "{own_plans:?}");
}

#[test]
fn the_older_copy_is_never_removed_when_it_could_be_the_only_working_one() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let failed = vec![(GIT_TOOL.to_string(), "cargo binstall failed".to_string())];
    let after_failure = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &failed);
    assert!(after_failure.losers[0].kept.as_deref().unwrap_or("").contains("failed in this run"), "{after_failure:?}");

    // A winner that cargo recorded but whose program cannot run.
    let winner_file = fs::canonicalize(duo.copy_in(&duo.repository_root())).expect("resolve the winner");
    fs::set_permissions(&winner_file, fs::Permissions::from_mode(0o644)).expect("drop the execute bit");
    let broken = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    assert!(broken.losers[0].kept.as_deref().unwrap_or("").contains("no working"), "{broken:?}");
    let outcome = apply_old_copies(&duo.context(&duo.rig.scope, &lock), &broken);
    assert!(version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "{outcome:?}");
}

#[test]
fn a_link_that_appears_after_the_plan_keeps_the_older_copy() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let plans = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    assert_eq!(plans.losers[0].kept, None, "{plans:?}");
    let late = duo.rig.scope.home.join(".local/bin/late");
    make_link(&late, &version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").join("bin").join(GIT_BIN));
    let outcome = apply_old_copies(&duo.context(&duo.rig.scope, &lock), &plans);
    assert!(version_directory(&duo.global_root(), GIT_CRATE, "0.4.4").exists(), "{outcome:?}");
    assert!(outcome.warnings.iter().any(|warning| return warning.contains("would leave that link dangling")), "{outcome:?}");
}

#[test]
fn list_names_the_older_managed_copy_install_would_remove_and_install_then_leaves_nothing_to_name() {
    let duo = Duo::new();
    duo.lock_repository("0.4.5");
    duo.install_under(&duo.repository_root(), "0.4.5");
    duo.install_under(&duo.global_root(), "0.4.4");
    duo.list_own_worktree();
    let before = run_status(&duo.rig.scope, &duo.rig.env, &duo.rig.runner, &duo.rig.host).expect("listed");
    assert!(!before.drifted, "{}", before.text);
    assert!(before.text.contains("older managed copy of cargo:monochromatic-git-policy-cli in"), "{}", before.text);
    assert!(before.text.contains("would be removed by `install` after it did"), "{}", before.text);
    duo.install_repository();
    let after = run_status(&duo.rig.scope, &duo.rig.env, &duo.rig.runner, &duo.rig.host).expect("listed");
    assert!(!after.text.contains("older managed copy"), "{}", after.text);
    assert!(preview_lines(&plan_old_copies(&duo.context(&duo.rig.scope, &duo.lock_repository("0.4.5")), &[])).is_empty());
}

#[test]
fn a_release_archive_tools_older_copy_in_the_other_root_goes_with_its_downloads_and_the_root_forwards() {
    let rig = Rig::new();
    let global = global_scope(&rig.env).expect("a global scope");
    let lock = Lock { tools: vec![release_tool(&rig, "0.2.2")] };
    let repository_root = rig.scope.toolchain_root.clone();
    let global_root = global.toolchain_root.clone();
    finish_archive(&repository_root, "0.2.2");
    point_current_at_newest_archive(&repository_root, ARCHIVE_ID, ARCHIVE_BIN).expect("point current");
    finish_archive(&global_root, "0.2.1");
    point_current_at_newest_archive(&global_root, ARCHIVE_ID, ARCHIVE_BIN).expect("point current");
    let downloads = global_root.join("downloads").join(ARCHIVE_ID).join("0.2.1");
    fs::create_dir_all(&downloads).expect("make the downloads");
    fs::write(downloads.join("mpm-shadow-0.2.1.tar.gz"), "bytes").expect("write a download");
    let older_file = global_root.join(ARCHIVE_ID).join("0.2.1").join("release-folder").join(ARCHIVE_BIN);
    let link = rig.scope.home.join(".local/bin/mpm");
    make_link(&link, &older_file);
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD 0000\n\n", rig.scope.root.display()), 0);
    let context = Context { scope: &rig.scope, env: &rig.env, runner: &rig.runner, http: &rig.http, host: &rig.host, tools: &lock.tools };
    let outcome = retire_old_copies(&context, &[]);
    assert!(outcome.failed.is_empty() && outcome.warnings.is_empty(), "{outcome:?}");
    let winner = current_link_of(&repository_root, ARCHIVE_ID).expect("a plain id").join("release-folder").join(ARCHIVE_BIN);
    assert_eq!(fs::read_link(&link).expect("a link"), winner);
    assert!(!global_root.join(ARCHIVE_ID).join("0.2.1").exists() && !downloads.exists(), "the version and its downloads are gone");
    assert_eq!(
        fs::read_link(current_link_of(&global_root, ARCHIVE_ID).expect("a plain id")).expect("a link"),
        current_link_of(&repository_root, ARCHIVE_ID).expect("a plain id")
    );
}

//endregion

/// What:
///  A tool id that a shell would split or read specially is printed inside both hints as one single-quoted word,
///  a plain id stays bare,
///  and neither hint names a mise task.
/// Why:
///  The hint is a command the person pastes into a shell,
///  so it must name exactly the tool whatever its id holds,
///  and mise is being retired,
/// so the hint names mpm's own pnpm-style verb.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(olderLockMessageFor('global', "odd id;'x", '0.4.4', '0.4.5')).toContain("`mpm update -g 'odd id;'\\''x'`");
/// ```
#[test]
fn the_hint_names_the_tool_as_one_shell_word_and_names_no_mise_task() {
    // Each pair is an id and the single shell word that reads back as it:
    // a space, a semicolon and a single quote (closed, escaped and reopened),
    // then a dollar sign, a backtick and a line break, which single quotes keep literal.
    let cases: [(&str, &str); 2] = [("odd id;'x", "'odd id;'\\''x'"), ("a$b`c\nd", "'a$b`c\nd'")];
    for (odd, quoted) in cases {
        let repository: String = older_lock_message_for(ScopeKind::Repo, odd, "0.4.4", "0.4.5");
        let global: String = older_lock_message_for(ScopeKind::Global, odd, "0.4.4", "0.4.5");
        assert!(repository.contains(&format!("`mpm update {quoted}` or merge main")), "{repository}");
        assert!(global.contains(&format!("`mpm update -g {quoted}`")), "{global}");
        assert!(!repository.contains("mise") && !global.contains("mise"), "{repository}\n{global}");
    }
    let plain: String = older_lock_message_for(ScopeKind::Global, GIT_TOOL, "0.4.4", "0.4.5");
    assert!(plain.contains(&format!("`mpm update -g {GIT_TOOL}`")), "{plain}");
}

#[test]
fn the_winning_copy_across_both_roots_is_the_longer_alpha_number() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5-alpha.10");
    duo.install_under(&duo.repository_root(), "0.4.5-alpha.10");
    duo.install_under(&duo.global_root(), "0.4.5-alpha.9");
    let winner = winning_copy(&duo.rig.scope, &lock.tools[0]).expect("a winner");
    assert_eq!(winner.version, "0.4.5-alpha.10");
    assert_eq!(winner.root, duo.repository_root());
}

#[test]
fn the_older_copy_in_the_other_root_is_planned_for_removal_by_the_alpha_number_read_as_a_number() {
    let duo = Duo::new();
    let lock = duo.lock_repository("0.4.5-alpha.10");
    duo.install_under(&duo.repository_root(), "0.4.5-alpha.10");
    duo.install_under(&duo.global_root(), "0.4.5-alpha.9");
    duo.list_own_worktree();
    let plans = plan_old_copies(&duo.context(&duo.rig.scope, &lock), &[]);
    assert_eq!(plans.losers.len(), 1, "{plans:?}");
    assert_eq!(plans.losers[0].kept, None, "{plans:?}");
    assert_eq!(plans.losers[0].winner_version, "0.4.5-alpha.10", "{plans:?}");
    assert_eq!(plans.losers[0].root, duo.global_root(), "the copy to remove is in the other root: {plans:?}");
    assert_eq!(plans.losers[0].copies.len(), 1, "{plans:?}");
    assert_eq!(plans.losers[0].copies[0].version, "0.4.5-alpha.9", "{plans:?}");
}
