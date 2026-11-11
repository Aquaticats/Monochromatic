//! The tests of mechanism C:
//! a tree whose workspace is gone and that has been idle is removed,
//! and a tree in use,
//! one whose workspace exists,
//! one within its idle period,
//! one whose lock is held,
//! and one whose toolchain is still current are all kept.

use std::{collections::HashSet, fs, fs::File, os::unix::fs::PermissionsExt, path::{Path, PathBuf}, process::{Child, Command}, time::Duration};

use rustix::fs::{FlockOperation, flock};

use crate::lean_build_trees::{Tree, TreeKind, Workspace, list_trees, profile_dirs_of, workspace_of};
use crate::lean_identity::Deadline;
use crate::lean_idle_trees::{IdlePolicy, TreeCollection, TreeOutcome, TreeVerdict, collect_idle_trees};
use crate::lean_test_support::{DAY, NOW, age_everything, make_tree, real_environment};
use crate::test_scratch::scratch;

/// The idle period of these tests for a tree whose workspace is gone.
const IDLE: i64 = 3 * DAY;

/// The idle period for a tree nothing identifies.
const UNIDENTIFIED_IDLE: i64 = 10 * DAY;

/// The policy of these tests.
const POLICY: IdlePolicy = IdlePolicy { idle_seconds: IDLE, unidentified_idle_seconds: UNIDENTIFIED_IDLE };

/// A child that is stopped when the guard drops.
struct Sleeper {
    /// The child.
    child: Child,
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Runs the collection over the base's trees.
fn collect(base: &Path, retired: &[&str], dry_run: bool) -> Vec<TreeOutcome> {
    return collect_reporting(base, retired, dry_run).0.outcomes;
}

/// Runs the collection and keeps the log lines it wrote.
fn collect_reporting(base: &Path, retired: &[&str], dry_run: bool) -> (TreeCollection, Vec<String>) {
    let trees: Vec<Tree> = list_trees(base);
    let retired_set: HashSet<String> = retired.iter().map(|name| { return (*name).to_string(); }).collect();
    let mut log: Vec<String> = Vec::new();
    let collection: TreeCollection = collect_idle_trees(&trees, &[], base, &retired_set, &POLICY, &real_environment(), NOW, dry_run, &Deadline::after(Duration::from_secs(60)), &mut log);
    return (collection, log);
}

/// The verdict for one tree.
fn verdict_of(outcomes: &[TreeOutcome], tree: &Path) -> TreeVerdict {
    for outcome in outcomes {
        if outcome.tree == tree {
            return outcome.verdict.clone();
        }
    }
    panic!("no outcome for {}", tree.display());
}

/// A tree whose workspace is the given path,
/// aged by the given seconds.
fn aged_tree(base: &Path, relative: &str, workspace: &Path, idle: i64) -> PathBuf {
    let tree: PathBuf = make_tree(base, relative, Some(workspace));
    age_everything(&tree, NOW - idle);
    return tree;
}

#[test]
fn a_tree_whose_workspace_is_gone_and_that_is_idle_is_removed() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let gone: PathBuf = root.path().join("no-longer-here");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000001", &gone, 5 * DAY);
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Removed { bytes, .. } if bytes > 0), "{outcomes:?}");
    assert!(!tree.exists());
    assert!(!base.join(".cargo-shadow-trash").exists());
}

#[test]
fn a_tree_whose_workspace_exists_is_kept_however_idle() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let alive: PathBuf = root.path().join("alive");
    fs::create_dir_all(&alive).expect("workspace");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000002", &alive, 90 * DAY);
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Keep(_)));
    assert!(tree.exists());
}

#[test]
fn a_tree_within_its_idle_period_is_kept() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000003", &root.path().join("gone"), DAY);
    assert!(matches!(verdict_of(&collect(&base, &[], false), &tree), TreeVerdict::Keep(_)));
    assert!(tree.exists());
}

#[test]
fn a_tree_whose_build_lock_is_held_is_skipped() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000004", &root.path().join("gone"), 5 * DAY);
    let lock: File = File::open(tree.join("debug").join(".cargo-build-lock")).expect("lock");
    flock(&lock, FlockOperation::NonBlockingLockExclusive).expect("take like a build");
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Skip(ref why) if why.contains("busy")), "{outcomes:?}");
    assert!(tree.exists());
}

#[test]
fn a_tree_a_process_uses_is_kept() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000005", &root.path().join("gone"), 5 * DAY);
    let child: Child = Command::new("sleep").arg("120").current_dir(tree.join("debug").join("build")).spawn().expect("start sleep");
    let _holder: Sleeper = Sleeper { child };
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Skip(ref why) if why.contains("uses")), "{outcomes:?}");
    assert!(tree.exists());
}

#[test]
fn a_tree_nothing_identifies_waits_for_the_longer_idle_period() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let young: PathBuf = make_tree(&base, "ab/00000000000006", None);
    age_everything(&young, NOW - 5 * DAY);
    let old: PathBuf = make_tree(&base, "ab/00000000000007", None);
    age_everything(&old, NOW - 20 * DAY);
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &young), TreeVerdict::Keep(_)));
    assert!(matches!(verdict_of(&outcomes, &old), TreeVerdict::Removed { .. }));
    assert!(young.exists() && !old.exists());
}

#[test]
fn a_retired_toolchains_tree_goes_even_when_its_workspace_exists_and_a_current_ones_stays() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let alive: PathBuf = root.path().join("alive");
    fs::create_dir_all(&alive).expect("workspace");
    let retired_tree: PathBuf = aged_tree(&base, "toolchain/nightly-old/ab/00000000000008", &alive, 5 * DAY);
    let current_tree: PathBuf = aged_tree(&base, "toolchain/nightly-new/ab/00000000000008", &alive, 5 * DAY);
    let outcomes = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&outcomes, &retired_tree), TreeVerdict::Removed { .. }), "{outcomes:?}");
    assert!(matches!(verdict_of(&outcomes, &current_tree), TreeVerdict::Keep(_)));
}

#[test]
fn a_tree_that_holds_another_trees_workspace_is_kept_until_that_one_goes() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let outer: PathBuf = aged_tree(&base, "ab/00000000000009", &root.path().join("gone"), 5 * DAY);
    let inner_workspace: PathBuf = outer.join("package").join("crate-0.1.0");
    let inner: PathBuf = aged_tree(&base, "ab/0000000000000a", &inner_workspace, 5 * DAY);
    let first = collect(&base, &[], false);
    assert!(matches!(verdict_of(&first, &outer), TreeVerdict::Keep(ref why) if why.contains("lies inside")), "{first:?}");
    assert!(matches!(verdict_of(&first, &inner), TreeVerdict::Removed { .. }));
    let second = collect(&base, &[], false);
    assert!(matches!(verdict_of(&second, &outer), TreeVerdict::Removed { .. }), "{second:?}");
}

#[test]
fn a_dry_run_removes_nothing_and_reports_the_bytes() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let tree: PathBuf = aged_tree(&base, "ab/0000000000000b", &root.path().join("gone"), 5 * DAY);
    let outcomes = collect(&base, &[], true);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Removed { bytes, .. } if bytes > 0));
    assert!(tree.exists());
}

#[test]
fn a_lead_from_dependency_information_is_judged_by_whether_its_source_exists() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let existing_source: PathBuf = root.path().join("src").join("lib.rs");
    fs::create_dir_all(existing_source.parent().expect("parent")).expect("source directory");
    fs::write(&existing_source, b"").expect("source");
    let kept: PathBuf = make_tree(&base, "ab/0000000000000c", None);
    let removed: PathBuf = make_tree(&base, "ab/0000000000000d", None);
    for (tree, source) in [(&kept, existing_source.clone()), (&removed, root.path().join("deleted").join("lib.rs"))] {
        let out: PathBuf = tree.join("debug").join("build").join("alpha").join("00000000000000aa").join("out");
        fs::create_dir_all(&out).expect("out directory");
        fs::write(out.join("alpha.d"), format!("{}/alpha.d: {}\n", tree.display(), source.display())).expect("dependency information");
        age_everything(tree, NOW - 60 * DAY);
    }
    assert!(matches!(workspace_of(&kept), Workspace::Lead(_)));
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &kept), TreeVerdict::Keep(_)), "{outcomes:?}");
    assert!(matches!(verdict_of(&outcomes, &removed), TreeVerdict::Removed { .. }));
}

#[test]
fn trees_are_listed_in_both_layouts_and_the_trash_is_not_a_tree() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let legacy: PathBuf = make_tree(&base, "ab/00000000000001", None);
    let keyed: PathBuf = make_tree(&base, "toolchain/nightly-x/cd/00000000000002", None);
    fs::create_dir_all(base.join(".cargo-shadow-trash").join("x")).expect("trash");
    let trees: Vec<Tree> = list_trees(&base);
    assert_eq!(trees.len(), 2);
    assert!(trees.contains(&Tree { path: legacy.clone(), kind: TreeKind::Legacy }));
    assert!(trees.contains(&Tree { path: keyed, kind: TreeKind::Keyed(String::from("nightly-x")) }));
    assert_eq!(profile_dirs_of(&legacy), vec![legacy.join("debug")]);
}

#[test]
fn a_dependency_token_the_read_cut_in_two_is_no_lead_and_names_no_gone_source() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let tree: PathBuf = make_tree(&base, "ab/0000000000000e", None);
    let out: PathBuf = tree.join("debug").join("build").join("alpha").join("00000000000000aa").join("out");
    fs::create_dir_all(&out).expect("out directory");
    // One path longer than the read: whatever the read keeps of it is a prefix of a path, not a path.
    fs::write(out.join("alpha.d"), format!("{}/alpha.d: /gone/{}\n", tree.display(), "z".repeat(6_000))).expect("dependency information");
    age_everything(&tree, NOW - 2 * DAY);
    assert_eq!(workspace_of(&tree), Workspace::Unidentified);
    let outcomes = collect(&base, &[], false);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Keep(_)), "{outcomes:?}");
    assert!(tree.exists());
}

/// What:
/// `fn close(path: &Path, mode: u32)` makes a directory closed to its owner,
/// after everything in it is written.
/// Why:
/// A directory this user may not write is what stopped the removal in issue 6282,
/// and it has to be the last step because the files go in first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const close = (path: string, mode: number): void => chmodSync(path, mode);
/// ```
fn close(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("mode");
}

#[test]
fn a_tree_that_holds_directories_this_user_may_not_write_is_removed_and_the_line_says_their_mode_changed() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let gone: PathBuf = root.path().join("no-longer-here");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000002", &gone, 5 * DAY);
    let closed: PathBuf = tree.join("debug").join("build").join("closed");
    fs::create_dir_all(closed.join("inner")).expect("closed directories");
    fs::write(closed.join("inner").join("file"), b"x").expect("file");
    fs::write(closed.join("file"), b"x").expect("file");
    close(&closed.join("inner"), 0o500);
    close(&closed, 0o555);
    let (collection, log): (TreeCollection, Vec<String>) = collect_reporting(&base, &[], false);
    assert!(matches!(verdict_of(&collection.outcomes, &tree), TreeVerdict::Removed { .. }), "{:?}", collection.outcomes);
    assert!(!tree.exists(), "the whole tree is gone");
    assert_eq!(collection.judged, 1);
    let line: &String = log.iter().find(|line| { return line.starts_with("removed tree "); }).expect("the removal line");
    assert!(line.contains("mode changed on 2 directories this user could not write"), "{line}");
}

/// A tree whose record is the only thing keeping another tree from being removed is not removed itself,
/// even when its own toolchain is retired:
/// the record it holds is where the live workspace inside that other tree is known from,
/// and a later pass has nothing else to guard with.
#[test]
fn a_retired_tree_is_kept_while_its_record_guards_a_live_workspace_inside_another_tree() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let outer: PathBuf = aged_tree(&base, "ab/00000000000010", &root.path().join("gone"), 5 * DAY);
    let live: PathBuf = outer.join("live-workspace");
    fs::create_dir_all(&live).expect("the live workspace");
    let important: PathBuf = live.join("important.txt");
    fs::write(&important, b"live user source\n").expect("the live workspace's file");
    let retired: PathBuf = aged_tree(&base, "toolchain/nightly-old/ab/00000000000011", &live, 5 * DAY);
    let first: Vec<TreeOutcome> = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&first, &outer), TreeVerdict::Keep(ref why) if why.contains("lies inside")), "{first:?}");
    assert!(matches!(verdict_of(&first, &retired), TreeVerdict::Keep(_)), "the record that guards {} is not thrown away with its tree: {first:?}", outer.display());
    assert!(retired.exists() && important.exists(), "{first:?}");
    // The second pass has the same records to work from, so it makes the same decision.
    let second: Vec<TreeOutcome> = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&second, &outer), TreeVerdict::Keep(_)), "a later pass keeps the tree the live workspace lies inside: {second:?}");
    assert!(important.exists(), "the live workspace's file is user data and must never be removed by a pass over idle trees: {second:?}");
}

/// A workspace behind a directory this user may not search has not stopped existing,
/// so the ledger that names it keeps the tree holding that workspace until the workspace is confirmed absent.
#[test]
fn a_workspace_behind_a_directory_this_user_may_not_search_is_not_a_gone_one() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let outer: PathBuf = make_tree(&base, "aa/00000000000001", Some(&root.path().join("gone")));
    let closed: PathBuf = outer.join("closed");
    let live: PathBuf = closed.join("live-workspace");
    fs::create_dir_all(&live).expect("the live workspace");
    let important: PathBuf = live.join("important.txt");
    fs::write(&important, b"live user source\n").expect("the live workspace's file");
    age_everything(&outer, NOW - 5 * DAY);
    let retired: PathBuf = aged_tree(&base, "toolchain/nightly-old/ab/00000000000002", &live, 5 * DAY);
    close(&closed, 0o000);

    let first: Vec<TreeOutcome> = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&first, &outer), TreeVerdict::Keep(_)), "{first:?}");
    assert!(matches!(verdict_of(&first, &retired), TreeVerdict::Keep(_)), "the ledger that names a workspace this user may not look for is not thrown away with its tree: {first:?}");

    close(&closed, 0o700);
    assert_eq!(fs::read_to_string(&important).expect("the live workspace's file survived the first pass"), "live user source\n");
    close(&closed, 0o000);

    let second: Vec<TreeOutcome> = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&second, &outer), TreeVerdict::Keep(_)), "{second:?}");
    close(&closed, 0o700);
    assert!(important.exists(), "a pass over idle trees never removes a live workspace: {second:?}");
}

/// A tree is kept whenever its workspace cannot be shown to be gone,
/// which is every answer other than a confirmed absence.
#[test]
fn a_tree_whose_workspace_cannot_be_shown_to_be_gone_is_kept() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let closed: PathBuf = root.path().join("closed");
    let workspace: PathBuf = closed.join("workspace");
    fs::create_dir_all(&workspace).expect("the workspace behind a closed directory");
    let tree: PathBuf = aged_tree(&base, "ab/00000000000013", &workspace, 5 * DAY);
    close(&closed, 0o000);
    let outcomes: Vec<TreeOutcome> = collect(&base, &[], false);
    close(&closed, 0o700);
    assert!(matches!(verdict_of(&outcomes, &tree), TreeVerdict::Keep(_)), "{outcomes:?}");
    assert!(tree.exists());
}

/// A retired tree whose record names a workspace outside the base still goes,
/// so keeping the record of a nested workspace does not keep every retired tree.
#[test]
fn a_retired_tree_whose_workspace_is_outside_the_base_still_goes() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let alive: PathBuf = root.path().join("a-checkout-that-still-exists");
    fs::create_dir_all(&alive).expect("the workspace");
    let retired: PathBuf = aged_tree(&base, "toolchain/nightly-old/ab/00000000000012", &alive, 5 * DAY);
    let outcomes: Vec<TreeOutcome> = collect(&base, &["nightly-old"], false);
    assert!(matches!(verdict_of(&outcomes, &retired), TreeVerdict::Removed { .. }), "{outcomes:?}");
    assert!(alive.exists(), "the workspace is outside the base and is not removed with the tree");
}
