//! Unit tests of the old copies of this repository's own tools:
//! which files in the shared cargo bin directory count as old copies,
//! which links move and where,
//! when a copy is kept (and why),
//! and the git wrapper case from `install` to `list`.
//! Every fixture sits in a disposable home;
//! cargo is a recording fake,
//! so nothing reaches the real `~/.cargo`,
//! `~/.local` or a real toolchain.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::cargo_installs::{current_link, point_current_at_newest, version_directory};
use super::context::Context;
use super::fakes::{Rig, write_executable};
use super::fakes_cargo::{fake_binstall, fake_uninstall, finish_install};
use super::runner::RunRequest;
use super::lock::{Lock, render_lock};
use super::model::LockedTool;
use super::old_copies::plan_old_copies;
use super::old_copy_apply::{apply_old_copies, preview_lines, retire_old_copies};
use super::plan::parse_plan;
use super::published_archive::{current_link_of, point_current_at_newest_archive};
use super::published_archive_tests::{BIN as ARCHIVE_BIN, ID as ARCHIVE_ID, finish_archive, release_tool};
use super::scope::{ScopeKind, global_scope, toolchain_roots};
use super::status_cmd::run_status;
use super::sync::{FrozenReason, LockPolicy, format_sync_report, run_lockfile_only, run_sync_with};

/// The plan entry of the git wrapper.
const GIT_TOOL: &str = "cargo:monochromatic-git-policy-cli";

/// Its crate.
const GIT_CRATE: &str = "monochromatic-git-policy-cli";

/// Its bin.
const GIT_BIN: &str = "cli-git-native";

/// The plan the scene writes:
/// the git wrapper with the postinstall link the repository's plan gives it.
const GIT_PLAN: &str = r#"{
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

/// A disposable home and worktree holding the git wrapper's plan and lock (locked at 0.4.5),
/// with fakes for cargo and the network.
struct Scene {
    /// The disposable scope,
    /// environment and fakes.
    rig: Rig,
    /// The lock of the scene (only the git wrapper).
    lock: Lock,
}

/// The methods of `Scene`.
impl Scene {
    /// A scene with the plan and lock on disk and `cargo binstall` faked.
    fn new() -> Scene {
        let rig = Rig::new();
        let plan = parse_plan(GIT_PLAN, "git plan").expect("the plan parses");
        let mut tools: Vec<LockedTool> = Vec::new();
        for declaration in plan.tools {
            tools.push(LockedTool { declaration, version: "0.4.5".to_string(), artifacts: Vec::new() });
        }
        let lock = Lock { tools };
        fs::write(&rig.scope.plan_file, GIT_PLAN).expect("write the plan");
        fs::write(&rig.scope.lock_file, render_lock(&lock)).expect("write the lock");
        rig.runner.on("cargo binstall", fake_binstall(&[GIT_BIN]));
        return Scene { rig, lock };
    }

    /// Makes `cargo uninstall` work the way cargo does (the bin and its record go).
    fn cargo_uninstalls(&self) {
        self.rig.runner.on("cargo uninstall", fake_uninstall());
    }

    /// A context for the scene's repository scope.
    fn context(&self) -> Context<'_> {
        return Context {
            scope: &self.rig.scope,
            env: &self.rig.env,
            runner: &self.rig.runner,
            http: &self.rig.http,
            host: &self.rig.host,
            tools: &self.lock.tools,
        };
    }

    /// The disposable home.
    fn home(&self) -> &Path {
        return &self.rig.scope.home;
    }

    /// `<home>/.cargo`.
    fn cargo_home(&self) -> PathBuf {
        return self.home().join(".cargo");
    }

    /// The old copy's place in the shared cargo bin directory.
    fn old_copy(&self) -> PathBuf {
        return self.cargo_home().join("bin").join(GIT_BIN);
    }

    /// Puts an old copy in the shared cargo bin directory.
    fn write_old_copy(&self) {
        write_executable(&self.old_copy());
    }

    /// Writes cargo's record naming the git wrapper crate at `version`.
    fn write_record(&self, version: &str) {
        fs::create_dir_all(self.cargo_home()).expect("make the cargo home");
        fs::write(
            self.cargo_home().join(".crates.toml"),
            format!("[v1]\n\"{GIT_CRATE} {version} (registry+https://github.com/rust-lang/crates.io-index)\" = [\"{GIT_BIN}\"]\n"),
        )
        .expect("write the record");
    }

    /// Finishes a managed install at `version` under `root` and points `current` at the newest.
    fn install_managed_under(&self, root: &Path, version: &str) -> PathBuf {
        finish_install(&version_directory(root, GIT_CRATE, version), GIT_CRATE, version, &[GIT_BIN]);
        point_current_at_newest(root, GIT_CRATE).expect("point current at the newest");
        return current_link(root, GIT_CRATE).join("bin").join(GIT_BIN);
    }

    /// Finishes a managed install in the scene's own (repository) toolchain root;
    /// returns the managed copy as links reach it,
    /// through `current`.
    fn install_managed(&self, version: &str) -> PathBuf {
        return self.install_managed_under(&self.rig.scope.toolchain_root, version);
    }

    /// The toolchain root of the other scope (the global one).
    fn global_root(&self) -> PathBuf {
        return toolchain_roots(&self.rig.env).remove(1);
    }

    /// `~/.local/bin/git`.
    fn local_git(&self) -> PathBuf {
        return self.home().join(".local").join("bin").join("git");
    }

    /// A second checkout of the repository with a `.cache/cli-git-bin/git` link to the old copy.
    fn second_checkout_git(&self) -> (PathBuf, PathBuf) {
        let checkout = self.rig._directory.path().join("work-two");
        let link = checkout.join(".cache").join("cli-git-bin").join("git");
        fs::create_dir_all(link.parent().expect("a parent")).expect("make the link directory");
        symlink(self.old_copy(), &link).expect("make the link");
        return (checkout, link);
    }

    /// Makes git list these worktrees.
    fn list_worktrees(&self, directories: &[&Path]) {
        let mut text: String = String::new();
        for directory in directories {
            text.push_str(&format!("worktree {}\nHEAD 0000\n\n", directory.display()));
        }
        self.rig.runner.answer("git -C", &text, 0);
    }

    /// How many times cargo was asked to uninstall.
    fn uninstalls(&self) -> usize {
        return self.rig.runner.lines().iter().filter(|line| return line.starts_with("cargo uninstall")).count();
    }
}

/// Points `link` at `target`,
/// making the link's directory.
fn make_link(link: &Path, target: &Path) {
    fs::create_dir_all(link.parent().expect("a parent")).expect("make the directory");
    symlink(target, link).expect("make the link");
}

//region what counts as an old copy

#[test]
fn a_file_named_like_a_declared_bin_is_an_old_copy_and_no_other_file_in_the_directory_is() {
    let scene = Scene::new();
    scene.write_old_copy();
    let managed = scene.install_managed("0.4.5");
    // Rejected fixtures: a file no lock entry declares.
    write_executable(&scene.cargo_home().join("bin").join("cargo-nextest"));
    write_executable(&scene.cargo_home().join("bin").join("monochromatic-git-policy-cli"));
    scene.list_worktrees(&[]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans.len(), 1, "{plans:?}");
    assert_eq!(plans.plans[0].path, scene.old_copy());
    assert_eq!(plans.plans[0].managed, Some(managed));
    assert_eq!(plans.plans[0].kept, None, "{plans:?}");
}

#[test]
fn a_third_party_tool_and_a_repository_tool_with_no_copy_there_are_not_judged() {
    let scene = Scene::new();
    let mut tools: Vec<LockedTool> = scene.rig.lock.tools.clone();
    tools.extend(scene.lock.tools.clone());
    write_executable(&scene.cargo_home().join("bin").join("fd"));
    write_executable(&scene.cargo_home().join("bin").join("mpm-shadow-but-not-declared"));
    let context = Context { tools: &tools, ..scene.context() };
    let plans = plan_old_copies(&context, &[]);
    assert!(plans.plans.is_empty(), "fd is another repository's tool and nothing is installed for the others: {plans:?}");
    assert!(scene.rig.runner.lines().is_empty(), "with no old copy nothing asked git or cargo");
}

#[test]
fn a_copy_cargo_attributes_to_another_crate_is_another_program_and_is_not_judged() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    fs::write(
        scene.cargo_home().join(".crates.toml"),
        "[v1]\n\"someone-elses-crate 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"cli-git-native\"]\n",
    )
    .expect("write the record");
    assert!(plan_old_copies(&scene.context(), &[]).plans.is_empty());
}

#[test]
fn a_symbolic_link_in_the_cargo_bin_directory_is_kept_as_it_is() {
    let scene = Scene::new();
    scene.install_managed("0.4.5");
    let dev_build = scene.home().join("dev").join(GIT_BIN);
    write_executable(&dev_build);
    make_link(&scene.old_copy(), &dev_build);
    scene.list_worktrees(&[]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans.len(), 1, "{plans:?}");
    assert!(plans.plans[0].kept.as_deref().unwrap_or("").contains("symbolic link"), "{plans:?}");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert_eq!(outcome.warnings.len(), 1);
    assert!(fs::symlink_metadata(scene.old_copy()).is_ok(), "the link is untouched");
}

#[test]
fn a_cargo_bin_directory_that_leads_into_a_toolchain_root_holds_no_old_copy() {
    let scene = Scene::new();
    scene.install_managed("0.4.5");
    let managed_bin = version_directory(&scene.rig.scope.toolchain_root, GIT_CRATE, "0.4.5").join("bin");
    fs::create_dir_all(scene.cargo_home()).expect("make the cargo home");
    symlink(&managed_bin, scene.cargo_home().join("bin")).expect("make the bin directory a link");
    assert!(scene.old_copy().is_file(), "the file the link reaches is the managed copy itself");
    assert!(plan_old_copies(&scene.context(), &[]).plans.is_empty());
}

//endregion

//region when a copy is kept

#[test]
fn with_no_finished_managed_copy_the_old_one_and_every_link_to_it_stay_as_they_are() {
    let scene = Scene::new();
    scene.write_old_copy();
    make_link(&scene.local_git(), &scene.old_copy());
    scene.list_worktrees(&[]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans.len(), 1);
    assert!(plans.plans[0].kept.as_deref().unwrap_or("").contains("only working copy"), "{plans:?}");
    assert!(plans.plans[0].repoints.is_empty(), "{plans:?}");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert!(outcome.lines.is_empty() && outcome.failed.is_empty(), "{outcome:?}");
    assert_eq!(outcome.warnings.len(), 1);
    assert!(scene.old_copy().exists());
    assert_eq!(fs::read_link(scene.local_git()).expect("still a link"), scene.old_copy());
}

#[test]
fn an_unfinished_managed_install_behind_current_is_not_a_managed_copy() {
    let scene = Scene::new();
    scene.write_old_copy();
    make_link(&scene.local_git(), &scene.old_copy());
    // An executable at the right path, but cargo never recorded the install: possibly a truncated one.
    let unfinished = version_directory(&scene.rig.scope.toolchain_root, GIT_CRATE, "0.4.5");
    write_executable(&unfinished.join("bin").join(GIT_BIN));
    symlink("0.4.5", current_link(&scene.rig.scope.toolchain_root, GIT_CRATE)).expect("point current at it");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert!(plans.plans[0].kept.as_deref().unwrap_or("").contains("only working copy"), "{plans:?}");
    assert!(plans.plans[0].repoints.is_empty(), "{plans:?}");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), scene.old_copy());
    assert!(scene.old_copy().exists() && outcome.failed.is_empty(), "{outcome:?}");
}

#[test]
fn a_tool_whose_install_failed_in_this_run_keeps_its_old_copy_even_when_an_older_managed_one_exists() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.1");
    scene.list_worktrees(&[]);
    let failed = vec![(GIT_TOOL.to_string(), "cargo binstall failed".to_string())];
    let plans = plan_old_copies(&scene.context(), &failed);
    assert!(plans.plans[0].kept.as_deref().unwrap_or("").contains("failed in this run"), "{plans:?}");
    assert!(plans.plans[0].repoints.is_empty());
    let allowed = plan_old_copies(&scene.context(), &[]);
    assert_eq!(allowed.plans[0].kept, None, "without the failure the same fixture is removable: {allowed:?}");
}

#[test]
fn a_record_that_says_the_old_copy_is_newer_than_the_managed_one_keeps_it() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[]);
    scene.write_record("0.4.6");
    let newer = plan_old_copies(&scene.context(), &[]);
    let reason = newer.plans[0].kept.clone().unwrap_or_default();
    assert!(reason.contains("0.4.6") && reason.contains(&format!("`mpm update {GIT_TOOL}`")), "{reason}");
    for version in ["0.4.5", "0.4.4", "0.2.13"] {
        scene.write_record(version);
        let plans = plan_old_copies(&scene.context(), &[]);
        assert_eq!(plans.plans[0].kept, None, "a record at {version} does not outrank the managed 0.4.5: {plans:?}");
    }
}

#[test]
fn a_git_listing_that_fails_keeps_the_copy_but_still_moves_the_links_it_can_see() {
    let scene = Scene::new();
    scene.write_old_copy();
    let managed = scene.install_managed("0.4.5");
    make_link(&scene.local_git(), &scene.old_copy());
    scene.rig.runner.fail("git -C", "fatal: not a git repository");
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans[0].repoints.len(), 1, "{plans:?}");
    assert!(plans.plans[0].kept.as_deref().unwrap_or("").contains("could not be searched"), "{plans:?}");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), managed);
    assert!(scene.old_copy().exists(), "a checkout's link could be hiding in a place git would not tell");
    assert_eq!(outcome.warnings.len(), 1);
}

#[test]
fn with_both_roots_holding_a_copy_every_link_goes_to_the_newest_one_wherever_it_is() {
    for (repository_version, global_version) in [("0.4.5", "0.4.4"), ("0.4.5", "0.4.6")] {
        let scene = Scene::new();
        scene.write_old_copy();
        let repository_copy = scene.install_managed(repository_version);
        let global_copy = scene.install_managed_under(&scene.global_root(), global_version);
        make_link(&scene.local_git(), &scene.old_copy());
        let (checkout, checkout_link) = scene.second_checkout_git();
        scene.list_worktrees(&[&scene.rig.scope.root, &checkout]);
        let plans = plan_old_copies(&scene.context(), &[]);
        let plan = &plans.plans[0];
        assert_eq!(plan.kept, None, "no link is left to anyone's decision: {plan:?}");
        let newest = if global_version > repository_version { &global_copy } else { &repository_copy };
        assert_eq!(plan.repoints.len(), 2, "{plan:?}");
        for repoint in &plan.repoints {
            assert_eq!(&repoint.target, newest, "{} runs the newest copy ({global_version} against {repository_version})", repoint.link.display());
        }
        apply_old_copies(&scene.context(), &plans);
        assert_eq!(&fs::read_link(scene.local_git()).expect("a link"), newest);
        assert_eq!(&fs::read_link(&checkout_link).expect("a link"), newest);
        assert!(!scene.old_copy().exists());
    }
}

#[test]
fn a_global_run_moves_the_links_outside_checkouts_to_its_own_copy_and_keeps_the_old_one() {
    let scene = Scene::new();
    scene.write_old_copy();
    let global_copy = scene.install_managed_under(&scene.global_root(), "0.4.4");
    make_link(&scene.local_git(), &scene.old_copy());
    let mut global = global_scope(&scene.rig.env).expect("a global scope");
    global.home = scene.home().to_path_buf();
    assert_eq!(global.kind, ScopeKind::Global);
    let context = Context { scope: &global, ..scene.context() };
    let plans = plan_old_copies(&context, &[]);
    let plan = &plans.plans[0];
    assert_eq!(plan.repoints.len(), 1, "{plan:?}");
    assert_eq!(plan.repoints[0].target, global_copy);
    assert!(plan.kept.as_deref().unwrap_or("").contains("inside a checkout"), "{plan:?}");
    apply_old_copies(&context, &plans);
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), global_copy);
    assert!(scene.old_copy().exists(), "the checkouts' links are unseen, so the copy stays");
}

//endregion

//region carrying out the plan

#[test]
fn the_git_wrapper_is_replaced_end_to_end_links_first_and_the_copy_and_its_record_after() {
    let scene = Scene::new();
    // The machine the owner has: an old copy cargo's record names, a stale second record, links into it from three
    // kinds of places, and links and files that are not about it.
    scene.cargo_uninstalls();
    scene.write_old_copy();
    scene.write_record("0.4.5");
    fs::write(scene.cargo_home().join(".crates2.json"), r#"{"installs":{"monochromatic-git-policy-cli 0.2.13 (registry+x)":{"bins":["cli-git-native"]}}}"#)
        .expect("write the stale second record");
    make_link(&scene.local_git(), &scene.old_copy());
    let (checkout, other_checkout_link) = scene.second_checkout_git();
    let global = global_scope(&scene.rig.env).expect("a global scope");
    let farm_link = global.exposure_dir.join(GIT_BIN);
    make_link(&farm_link, &scene.old_copy());
    let foreign_target = scene.home().join(".local").join("share").join("uv").join("hf");
    write_executable(&foreign_target);
    make_link(&scene.home().join(".local").join("bin").join("hf"), &foreign_target);
    write_executable(&scene.home().join(".local").join("bin").join("not-a-link"));
    scene.list_worktrees(&[&scene.rig.scope.root, &checkout]);

    let report = run_sync_with(
        &scene.rig.scope,
        &scene.rig.env,
        &scene.rig.runner,
        &scene.rig.http,
        &scene.rig.host,
        LockPolicy::Frozen(FrozenReason::Flag),
    )
    .expect("synced");
    assert_eq!(report.installed, vec![GIT_TOOL.to_string()], "{report:?}");
    assert!(report.failed.is_empty(), "{report:?}");

    let managed = current_link(&scene.rig.scope.toolchain_root, GIT_CRATE).join("bin").join(GIT_BIN);
    for link in [&scene.local_git(), &other_checkout_link, &farm_link] {
        assert_eq!(fs::read_link(link).expect("still a link"), managed, "{}", link.display());
    }
    assert!(managed.is_file(), "the managed copy runs");
    assert!(fs::symlink_metadata(scene.old_copy()).is_err(), "the old copy is gone");
    let record = fs::read_to_string(scene.cargo_home().join(".crates.toml")).expect("read the record");
    assert!(!record.contains(GIT_CRATE), "cargo's record no longer names it: {record}");
    assert_eq!(fs::read_link(scene.home().join(".local").join("bin").join("hf")).expect("a link"), foreign_target, "a foreign link is untouched");
    assert!(scene.home().join(".local").join("bin").join("not-a-link").is_file(), "a regular file is untouched");

    // The report names every action, in the owner's order.
    assert_eq!(report.old_copies.len(), 4, "{:?}", report.old_copies);
    assert!(report.old_copies[..3].iter().all(|line| return line.starts_with("re-pointed ")), "{:?}", report.old_copies);
    assert!(report.old_copies[3].starts_with(&format!("removed {}", scene.old_copy().display())), "{:?}", report.old_copies);
    let text = format_sync_report(&report);
    assert!(text.contains(&format!("toolchain install: re-pointed {} -> {}", scene.local_git().display(), managed.display())), "{text}");
    assert_eq!(scene.uninstalls(), 1);
    assert!(
        scene.rig.runner.lines().contains(&format!("cargo uninstall --root {} --bin {GIT_BIN} -- {GIT_CRATE}", scene.cargo_home().display())),
        "{:?}",
        scene.rig.runner.lines()
    );

    // A second run finds nothing to do and says nothing.
    let again = run_sync_with(
        &scene.rig.scope,
        &scene.rig.env,
        &scene.rig.runner,
        &scene.rig.http,
        &scene.rig.host,
        LockPolicy::Resolve,
    )
    .expect("synced again");
    assert!(again.old_copies.is_empty() && again.warnings.is_empty() && again.failed.is_empty(), "{again:?}");
    assert_eq!(scene.uninstalls(), 1, "no second uninstall");
}

#[test]
fn a_failed_install_of_a_newer_version_leaves_the_old_copy_and_its_links_even_beside_an_older_managed_copy() {
    let scene = Scene::new();
    scene.cargo_uninstalls();
    scene.write_old_copy();
    scene.write_record("0.4.5");
    scene.install_managed("0.4.1");
    make_link(&scene.local_git(), &scene.old_copy());
    scene.list_worktrees(&[&scene.rig.scope.root]);
    scene.rig.runner.fail("cargo binstall", "error: could not fetch the crate");
    let report = run_sync_with(
        &scene.rig.scope,
        &scene.rig.env,
        &scene.rig.runner,
        &scene.rig.http,
        &scene.rig.host,
        LockPolicy::Frozen(FrozenReason::Flag),
    )
    .expect("synced");
    assert_eq!(report.failed.len(), 1, "the install failed and nothing else did: {report:?}");
    assert!(report.old_copies.is_empty(), "no link moved and no copy went: {report:?}");
    assert!(scene.old_copy().exists(), "this may be the only copy newer than the managed 0.4.1");
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), scene.old_copy());
    assert!(report.warnings.iter().any(|warning| return warning.contains("failed in this run")), "{report:?}");
    assert_eq!(scene.uninstalls(), 0);
}

#[test]
fn the_old_copy_of_a_release_archive_tool_is_retired_the_same_way_with_or_without_a_cargo_record() {
    for recorded in [false, true] {
        let rig = Rig::new();
        rig.runner.on("cargo uninstall", fake_uninstall());
        let lock = Lock { tools: vec![release_tool(&rig, "0.2.2")] };
        let root = rig.scope.toolchain_root.clone();
        finish_archive(&root, "0.2.2");
        point_current_at_newest_archive(&root, ARCHIVE_ID, ARCHIVE_BIN).expect("point current at the newest");
        let old_copy = rig.scope.home.join(".cargo").join("bin").join(ARCHIVE_BIN);
        write_executable(&old_copy);
        if recorded {
            fs::write(
                rig.scope.home.join(".cargo").join(".crates.toml"),
                format!("[v1]\n\"monochromatic-mpm-shadow 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"{ARCHIVE_BIN}\"]\n"),
            )
            .expect("write the record");
        }
        let link = rig.scope.home.join(".local").join("bin").join("mpm");
        make_link(&link, &old_copy);
        rig.runner.answer("git -C", &format!("worktree {}\nHEAD 0000\n\n", rig.scope.root.display()), 0);
        let context = Context { scope: &rig.scope, env: &rig.env, runner: &rig.runner, http: &rig.http, host: &rig.host, tools: &lock.tools };
        let outcome = retire_old_copies(&context, &[]);
        assert!(outcome.failed.is_empty() && outcome.warnings.is_empty(), "{outcome:?}");
        let managed = current_link_of(&root, ARCHIVE_ID).expect("a plain id").join("release-folder").join(ARCHIVE_BIN);
        assert_eq!(fs::read_link(&link).expect("a link"), managed, "recorded: {recorded}");
        assert!(fs::symlink_metadata(&old_copy).is_err(), "recorded: {recorded}");
        let uninstalls = rig.runner.lines().iter().filter(|line| return line.starts_with("cargo uninstall")).count();
        assert_eq!(uninstalls, usize::from(recorded), "cargo is asked only when its record names the bin");
    }
}

#[test]
fn a_sibling_link_in_the_cargo_bin_directory_is_re_pointed_so_nothing_dangles() {
    let scene = Scene::new();
    scene.write_old_copy();
    let managed = scene.install_managed("0.4.5");
    let bin = scene.cargo_home().join("bin");
    // The probe the review ran: `~/.cargo/bin/git` is a relative link to its sibling, the copy.
    symlink(GIT_BIN, bin.join("git")).expect("make the sibling link");
    make_link(&scene.local_git(), &scene.old_copy());
    // Rejected fixtures: rustup's proxies and a link to another program in the same directory.
    write_executable(&bin.join("rustup"));
    symlink("rustup", bin.join("cargo")).expect("make a proxy link");
    write_executable(&scene.home().join("elsewhere").join("tool"));
    make_link(&bin.join("other"), &scene.home().join("elsewhere").join("tool"));
    scene.list_worktrees(&[&scene.rig.scope.root]);

    let outcome = retire_old_copies(&scene.context(), &[]);
    assert!(outcome.failed.is_empty() && outcome.warnings.is_empty(), "{outcome:?}");
    assert!(fs::symlink_metadata(scene.old_copy()).is_err(), "the copy is gone");
    assert_eq!(fs::read_link(bin.join("git")).expect("still a link"), managed, "the sibling link follows the copy to the managed one");
    assert!(fs::metadata(bin.join("git")).is_ok(), "and does not dangle");
    assert_eq!(fs::read_link(bin.join("cargo")).expect("a link"), PathBuf::from("rustup"), "rustup's proxy is untouched");
    assert_eq!(fs::read_link(bin.join("other")).expect("a link"), scene.home().join("elsewhere").join("tool"), "a link to another program is untouched");
}

#[test]
fn a_record_rewritten_to_a_hostile_crate_name_after_the_plan_never_reaches_cargo() {
    let scene = Scene::new();
    scene.cargo_uninstalls();
    scene.write_old_copy();
    scene.write_record("0.4.5");
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans[0].kept, None, "{plans:?}");
    // The file is not ours: between the plan and the removal it names the bin under a crate that looks like an option.
    fs::write(
        scene.cargo_home().join(".crates.toml"),
        "[v1]\n\"--root=/somewhere 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"cli-git-native\"]\n",
    )
    .expect("rewrite the record");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert_eq!(scene.uninstalls(), 0, "cargo was never asked: {:?}", scene.rig.runner.lines());
    assert!(scene.old_copy().exists(), "the copy stays");
    assert_eq!(outcome.failed.len(), 1, "{outcome:?}");
    assert!(outcome.failed[0].1.contains("does not publish"), "{outcome:?}");
}

#[test]
fn a_copy_another_run_removed_first_is_done_and_not_a_failure() {
    // Without a record: the file is gone between the plan and the removal.
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let plans = plan_old_copies(&scene.context(), &[]);
    fs::remove_file(scene.old_copy()).expect("the other run removes it");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert!(outcome.failed.is_empty() && outcome.warnings.is_empty(), "{outcome:?}");
    assert!(outcome.lines[0].contains("another run had already removed the file"), "{outcome:?}");

    // With a record: the other run finishes while this run's `cargo uninstall` is failing on the lock.
    let recorded = Scene::new();
    recorded.write_old_copy();
    recorded.write_record("0.4.5");
    recorded.install_managed("0.4.5");
    recorded.list_worktrees(&[&recorded.rig.scope.root]);
    let (copy, record) = (recorded.old_copy(), recorded.cargo_home().join(".crates.toml"));
    recorded.rig.runner.on(
        "cargo uninstall",
        Box::new(move |_: &RunRequest| {
            fs::remove_file(&copy).expect("the other run removes the copy");
            fs::write(&record, "[v1]\n").expect("and its record");
        }),
    );
    recorded.rig.runner.fail("cargo uninstall", "error: could not acquire package cache lock");
    let finished = retire_old_copies(&recorded.context(), &[]);
    assert!(finished.failed.is_empty(), "{finished:?}");
    assert!(finished.lines.iter().any(|line| return line.contains("another run had already removed it and its record")), "{finished:?}");
}

#[test]
fn list_names_what_install_would_do_and_stops_once_it_did() {
    let scene = Scene::new();
    scene.cargo_uninstalls();
    scene.write_old_copy();
    scene.write_record("0.4.5");
    scene.install_managed("0.4.5");
    make_link(&scene.local_git(), &scene.old_copy());
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let before = run_status(&scene.rig.scope, &scene.rig.env, &scene.rig.runner, &scene.rig.host).expect("listed");
    assert!(!before.drifted, "an old copy is not drift: {}", before.text);
    assert!(before.text.contains("toolchain list: warning: old copy "), "{}", before.text);
    assert!(before.text.contains("would be removed by `install` after it did: re-point 1 link(s)"), "{}", before.text);
    assert!(scene.old_copy().exists() && fs::read_link(scene.local_git()).is_ok_and(|target| return target == scene.old_copy()), "list changes nothing");

    run_sync_with(&scene.rig.scope, &scene.rig.env, &scene.rig.runner, &scene.rig.http, &scene.rig.host, LockPolicy::Resolve).expect("synced");
    let after = run_status(&scene.rig.scope, &scene.rig.env, &scene.rig.runner, &scene.rig.host).expect("listed");
    assert!(!after.text.contains("old copy"), "{}", after.text);
}

#[test]
fn install_lockfile_only_leaves_every_old_copy_and_link_alone() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    make_link(&scene.local_git(), &scene.old_copy());
    let calls_before = scene.rig.runner.lines().len();
    run_lockfile_only(&scene.rig.scope, &scene.rig.env, &scene.rig.runner, &scene.rig.http, &scene.rig.host, LockPolicy::Resolve)
        .expect("locked");
    assert!(scene.old_copy().exists());
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), scene.old_copy());
    assert_eq!(scene.rig.runner.lines().len(), calls_before, "nothing ran");
}

#[test]
fn with_no_record_of_the_copy_it_is_removed_directly_and_cargo_is_not_asked() {
    for record in [None, Some("[v1]\n"), Some("[v1]\n\"fd-find 10.5.0 (registry+x)\" = [\"fd\"]\n")] {
        let scene = Scene::new();
        scene.cargo_uninstalls();
        scene.write_old_copy();
        scene.install_managed("0.4.5");
        if let Some(text) = record {
            fs::write(scene.cargo_home().join(".crates.toml"), text).expect("write the record");
        }
        // Cargo's other file names the crate alone: cargo itself ignores it (measured with cargo 1.101), so the copy goes directly.
        fs::write(scene.cargo_home().join(".crates2.json"), r#"{"installs":{"monochromatic-git-policy-cli 0.2.13 (registry+x)":{"bins":["cli-git-native"]}}}"#)
            .expect("write the stale second record");
        scene.list_worktrees(&[&scene.rig.scope.root]);
        let outcome = retire_old_copies(&scene.context(), &[]);
        assert!(outcome.failed.is_empty(), "{outcome:?}");
        assert!(fs::symlink_metadata(scene.old_copy()).is_err(), "{record:?}");
        assert_eq!(scene.uninstalls(), 0, "{record:?}");
        assert!(outcome.lines[0].contains("cargo has no record of it"), "{outcome:?}");
    }
}

#[test]
fn a_record_that_names_an_older_version_still_goes_through_cargo_uninstall() {
    let scene = Scene::new();
    scene.cargo_uninstalls();
    scene.write_old_copy();
    scene.write_record("0.2.13");
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let outcome = retire_old_copies(&scene.context(), &[]);
    assert!(outcome.failed.is_empty(), "{outcome:?}");
    assert_eq!(scene.uninstalls(), 1);
    assert!(fs::symlink_metadata(scene.old_copy()).is_err());
    assert!(outcome.lines[0].contains("`cargo uninstall` removed it from cargo's record of monochromatic-git-policy-cli 0.2.13"), "{outcome:?}");
}

#[test]
fn a_cargo_uninstall_that_fails_keeps_the_copy_and_is_reported_as_a_failure() {
    let failing = Scene::new();
    failing.write_old_copy();
    failing.write_record("0.4.5");
    failing.install_managed("0.4.5");
    failing.list_worktrees(&[&failing.rig.scope.root]);
    failing.rig.runner.fail("cargo uninstall", "error: could not acquire package cache lock");
    let outcome = retire_old_copies(&failing.context(), &[]);
    assert_eq!(outcome.failed.len(), 1, "{outcome:?}");
    assert!(outcome.failed[0].1.contains("could not acquire package cache lock"), "{outcome:?}");
    assert!(outcome.lines.is_empty(), "{outcome:?}");
    assert!(failing.old_copy().exists(), "a failed uninstall removes nothing");
}

#[test]
fn a_link_made_after_the_plan_keeps_the_copy_because_removing_it_would_leave_that_link_dangling() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans[0].kept, None);
    make_link(&scene.local_git(), &scene.old_copy());
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert!(scene.old_copy().exists());
    assert_eq!(fs::read_link(scene.local_git()).expect("a link"), scene.old_copy());
    assert_eq!(outcome.warnings.len(), 1, "{outcome:?}");
    assert!(outcome.warnings[0].contains("would leave that link dangling"), "{outcome:?}");
}

#[test]
fn a_link_that_cannot_be_re_pointed_keeps_the_copy_and_is_a_failure() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    make_link(&scene.local_git(), &scene.old_copy());
    let directory = scene.local_git().parent().expect("a parent").to_path_buf();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).expect("make the directory read-only");
    let probe = directory.join("probe");
    assert!(fs::write(&probe, "x").is_err(), "the fixture needs a directory this user cannot write (not run as root)");
    let outcome = retire_old_copies(&scene.context(), &[]);
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).expect("restore the directory");
    assert_eq!(outcome.failed.len(), 1, "{outcome:?}");
    assert!(outcome.failed[0].1.contains("cannot re-point"), "{outcome:?}");
    assert!(scene.old_copy().exists(), "the link still reaches it");
    assert_eq!(outcome.warnings.len(), 1, "{outcome:?}");
}

#[test]
fn a_managed_copy_that_stops_being_executable_before_the_removal_keeps_the_old_copy() {
    let scene = Scene::new();
    scene.write_old_copy();
    let managed = scene.install_managed("0.4.5");
    scene.list_worktrees(&[&scene.rig.scope.root]);
    let plans = plan_old_copies(&scene.context(), &[]);
    assert_eq!(plans.plans[0].kept, None);
    fs::set_permissions(fs::canonicalize(&managed).expect("resolve the managed copy"), fs::Permissions::from_mode(0o644)).expect("drop the execute bit");
    let outcome = apply_old_copies(&scene.context(), &plans);
    assert!(scene.old_copy().exists());
    assert!(outcome.warnings[0].contains("no longer executable"), "{outcome:?}");
}

#[test]
fn the_preview_for_a_kept_copy_gives_the_reason_and_the_moves_install_would_still_make() {
    let scene = Scene::new();
    scene.write_old_copy();
    scene.install_managed("0.4.5");
    make_link(&scene.local_git(), &scene.old_copy());
    scene.rig.runner.fail("git -C", "fatal: not a git repository");
    let lines = preview_lines(&plan_old_copies(&scene.context(), &[]));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("stays: a place could not be searched for links"), "{lines:?}");
    assert!(lines[0].contains("`install` would re-point 1 link(s)"), "{lines:?}");
}

//endregion
