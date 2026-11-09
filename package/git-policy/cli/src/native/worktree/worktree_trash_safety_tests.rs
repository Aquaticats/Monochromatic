//! Controls for the findings of the cross-vendor review of the leave-out and its disposal:
//! what pruning may move,
//! what a
//!  disposal may delete,
//! and what a failed cleanup must report.

use super::*;
use crate::fs_btrfs_ioctl::snapshot_create_at;
use std::os::unix::ffi::OsStrExt;

/// A snapshot of the main checkout at a stage name beside the repository,
/// taken without the leave-out,
/// for the controls that
///  run the leave-out themselves.
fn unpruned_snapshot(root: &Path, repo: &Path) -> (PathBuf, crate::worktree_snapshot_source::SnapshotSource) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("../topic")).expect("destination");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect("a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    snapshot_create_at(source.root.as_path(), stage.as_path()).expect("snapshot");
    std::fs::set_permissions(&stage, std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    return (stage, source);
}

/// A main checkout with a registered worktree at `ignored/wt` (an ignored path) in which the index tracks `ignored/wt/keep`.
fn main_with_tracked_file_in_nested_worktree(root: &Path) -> PathBuf {
    let repo: PathBuf = subvolume_repository(root);
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", "ignored/wt"]);
    tracked(repo.as_path(), "ignored/wt/keep", b"tracked inside a registered worktree\n");
    let blob: String = String::from_utf8(git(repo.as_path(), &["hash-object", "-w", "--no-filters", "ignored/wt/keep"]).stdout).expect("utf8").trim().to_string();
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("100644,{blob},ignored/wt/keep").as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=track a file inside the nested worktree"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\n").expect("exclude");
    return repo;
}

/// Finding 9:
/// a registered nested worktree that holds a file the main index tracks cannot be left out of the snapshot,
/// so the
///  leave-out refuses before it unlinks a pointer or moves a directory,
/// and says which path holds the tracked file.
#[test]
fn btrfs_a_registered_nested_worktree_that_holds_a_tracked_file_is_refused() {
    let Some(root) = btrfs_fixture("trash-nested-tracked") else { return };
    let repo: PathBuf = main_with_tracked_file_in_nested_worktree(root.as_path());
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    assert!(source.nested.iter().any(|path: &PathBuf| { return path.ends_with("ignored/wt"); }), "the nested worktree is registered: {:?}", source.nested);
    let outcome: Result<Pruned, String> = prune_snapshot(stage.as_path(), &source);
    let reason: String = outcome.clone().expect_err(format!("pruning must refuse, but it returned {outcome:?} and ignored/wt/keep is {}", if stage.join("ignored/wt/keep").exists() { "still there" } else { "gone" }).as_str());
    assert!(reason.contains("ignored/wt") && reason.contains("ignored/wt/keep"), "the reason names the worktree and the tracked file: {reason}");
    assert!(stage.join("ignored/wt/keep").exists(), "the tracked file is still in the snapshot");
    assert!(stage.join("ignored/wt/.git").exists(), "no pointer was unlinked before the refusal");
    assert!(stage.join("node_modules").exists() == repo.join("node_modules").exists(), "nothing else was touched either");
    remove(root.as_path());
}

/// Finding 9,
/// at the level of the whole snapshot:
/// the reason the rung gives names the nested worktree and the tracked file
///  inside it.
#[test]
fn btrfs_a_snapshot_over_a_nested_worktree_with_a_tracked_file_says_why() {
    let Some(root) = btrfs_fixture("trash-nested-tracked-snapshot") else { return };
    let repo: PathBuf = main_with_tracked_file_in_nested_worktree(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("../topic")).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: std::fs::File = crate::fs_beneath::open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let failure = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice(), &crate::fixture_content_decision::content_environment()).expect_err("the rung declines");
    assert!(failure.reason.contains("ignored/wt/keep") && failure.reason.contains("registered worktree"), "the reason says why: {}", failure.reason);
    remove(root.as_path());
}

/// Finding 10:
/// a directory the index holds as a gitlink is protected at its own path,
/// so it is not set aside as a copied
///  checkout.
/// (A source whose index holds a gitlink is refused before a snapshot is taken;
/// the leave-out still must not rely on that.)
#[test]
fn btrfs_a_directory_the_index_holds_as_a_gitlink_is_not_set_aside() {
    let Some(root) = btrfs_fixture("trash-gitlink") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\n").expect("exclude");
    let (stage, mut source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let head: String = String::from_utf8(git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("utf8").trim().to_string();
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{head},vendor/lib").as_str()]);
    source.clean.index_bytes = std::fs::read(repo.join(".git/index")).expect("index with a gitlink");
    put(stage.as_path(), "vendor/lib/.git", b"gitdir: ../../.git/modules/lib\n");
    put(stage.as_path(), "vendor/lib/file", b"submodule content\n");
    prune_snapshot(stage.as_path(), &source).expect("pruned");
    assert!(stage.join("vendor/lib/file").exists(), "the gitlink directory stays in the new worktree");
    remove(root.as_path());
}

/// Finding 11:
/// a directory in the trash parent that the leave-out did not make (the main checkout's own,
/// carried by the
///  snapshot) is not the snapshot's trash and is never emptied,
/// whatever its name looks like.
#[test]
fn btrfs_a_hex_named_directory_the_snapshot_carried_is_never_emptied() {
    let Some(root) = btrfs_fixture("trash-carried") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let carried: String = "a".repeat(40);
    put(repo.as_path(), format!("{TRASH_PARENT}/{carried}/d0/precious").as_str(), b"main's own ignored file\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    let parent: PathBuf = stage.join(TRASH_PARENT);
    assert_ne!(manifest.name, carried, "the trash is not the carried directory");
    assert!(parent.join(&manifest.name).exists() && parent.join(&carried).exists(), "both are in the trash parent");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::write(stage.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("pointer");
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), stage.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return None; }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT }).expect("emptied here");
    assert!(parent.join(&carried).join("d0/precious").exists(), "the directory the main checkout carried is untouched");
    assert!(!parent.join(&manifest.name).exists(), "the trash the leave-out made is emptied");
    remove(root.as_path());
}

/// A worktree with a `.git` file and a planted trash of `count` entries,
/// with the admin directory and the manifest.
fn worktree_with_planted_trash(label: &str, name: &str, count: usize) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(label);
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name, count);
    return (root, worktree, git_dir, manifest);
}

/// A worktree with a planted trash that was then moved outside:
/// the worktree holds only a `.git` file and an empty `.claude`,
///  and the outside directory holds `cli-git-trash/<name>`;
/// the manifest still describes it.
fn worktree_with_trash_moved_out(label: &str, name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf, TrashManifest) {
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash(label, name, 1);
    let outside: PathBuf = root.join("outside");
    std::fs::create_dir_all(&outside).expect("outside");
    std::fs::rename(worktree.join(TRASH_PARENT), outside.join("cli-git-trash")).expect("move the trash parent out");
    return (root, worktree, git_dir, outside, manifest);
}

/// Finding 11:
/// the trash directory named in a record is emptied only while it is the object the leave-out made.
#[test]
fn a_trash_directory_that_was_replaced_is_not_emptied() {
    let name: String = "1".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-replaced", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(&trash, worktree.join(TRASH_PARENT).join("moved-away")).expect("move the real one away");
    // An empty directory under the same name: nothing inside it could stop the removal, only its identity can.
    std::fs::create_dir(&trash).expect("somebody else's directory under the same name");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains(&name)), "{verdict:?}");
    assert!(trash.is_dir(), "the replacement is untouched");
    remove(root.as_path());
}

/// Finding 11:
/// an entry of the trash that was replaced by a link is left alone,
/// what it points at is untouched,
/// and the other
///  entries are still emptied.
#[test]
fn a_trash_entry_that_was_replaced_by_a_link_is_not_followed() {
    let name: String = "2".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-entry-link", name.as_str(), 2);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let outside: PathBuf = root.join("outside");
    put(outside.as_path(), "precious", b"keep");
    std::fs::remove_dir_all(trash.join("d0")).expect("remove the first entry");
    std::os::unix::fs::symlink(&outside, trash.join("d0")).expect("link in its place");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains(format!("{name}/d0").as_str())), "{verdict:?}");
    assert!(outside.join("precious").exists(), "what the link points at is untouched");
    assert!(!trash.join("d1").exists(), "the entry that was still the recorded object is emptied");
    remove(root.as_path());
}

/// Finding 11:
/// an entry of the trash that was replaced by another real directory under the same name is not the object the
///  leave-out moved there,
/// so it is left alone and reported.
#[test]
fn a_trash_entry_that_was_replaced_by_another_directory_is_not_emptied() {
    let name: String = "4".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-entry-replaced", name.as_str(), 2);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(trash.join("d0"), worktree.join("moved-away")).expect("move the real entry away");
    put(trash.as_path(), "d0/precious", b"somebody else's directory under the same name\n");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains(format!("{name}/d0").as_str())), "{verdict:?}");
    assert!(trash.join("d0/precious").exists(), "the replacement is untouched");
    assert!(!trash.join("d1").exists(), "the entry that was still the recorded object is emptied");
    remove(root.as_path());
}

/// Finding 11:
/// something in the trash that the leave-out did not put there stays,
/// and so does the trash directory.
#[test]
fn something_in_the_trash_that_was_not_moved_there_stays() {
    let name: String = "3".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-extra", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    put(trash.as_path(), "unrecorded/file", b"not ours\n");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains("unrecorded") && reason.contains("did not put there")), "{verdict:?}");
    assert!(trash.join("unrecorded/file").exists(), "the unrecorded entry is untouched");
    assert!(!trash.join("d0").exists(), "the recorded entry is emptied");
    remove(root.as_path());
}

/// What:
///  The trash of a disposal that failed is tried again by the next add,
///  and named to the person each time it still cannot be emptied,
///  instead of being dropped when the service gives up on it.
/// Why:
///   The service settles a failed trash item as failed,
/// so it is never tried again,
/// and the inline path's promise that the
///  service "empties it when it next runs" holds only for its first try.
/// Nothing else swept the trash parent,
/// so a full copy of
///  left-out directories could sit in a worktree for ever with no command naming it.
#[test]
fn a_trash_that_could_not_be_emptied_is_tried_again_and_named() {
    let name: String = "9".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-retry", name.as_str(), 2);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    // One entry is a directory somebody else put under the name the leave-out used: the disposal refuses for ever.
    std::fs::rename(trash.join("d0"), worktree.join("moved-away")).expect("move the real entry away");
    put(trash.as_path(), "d0/precious", b"somebody else's directory under the same name
");
    // The record of the failed emptying, in the state the service leaves it in.
    let mut say: Vec<u8> = Vec::new();
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), crate::work_item::PENDING).join(item.id());
    let failed: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), crate::work_item::FAILED).join(item.id());
    std::fs::rename(pending.as_path(), failed.as_path()).expect("the service gave up on it");
    retry_given_up_trash(git_dir.as_path(), worktree.as_path(), &mut say);
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(trash.to_str().expect("UTF-8")), "the person is told which trash stays: {said}");
    assert!(said.contains("precious") || said.contains("d0"), "and why: {said}");
    assert!(failed.is_file() && trash.join("d0/precious").exists(), "the record and the replacement stay");
    // The cause is dealt with, so the next add empties what the record names.
    std::fs::remove_dir_all(trash.join("d0")).expect("the foreign directory goes");
    let mut second: Vec<u8> = Vec::new();
    retry_given_up_trash(git_dir.as_path(), worktree.as_path(), &mut second);
    assert!(!trash.exists() && !failed.exists(), "the trash and its record are gone");
    assert!(String::from_utf8_lossy(second.as_slice()).contains(&name[..8]), "the emptying is said: {}", String::from_utf8_lossy(second.as_slice()));
    remove(root.as_path());
}

/// What:
///  A directory in the trash parent that no record names is named to the person and never deleted.
/// Why:
///   Removing trash deletes a person's folders,
/// and a directory nobody recorded cannot be proven to be one the leave-out moved,
///  so the sweep only says it is there.
#[test]
fn a_trash_no_record_names_is_named_and_left_alone() {
    let (root, worktree, git_dir, _manifest) = worktree_with_planted_trash("trash-unrecorded", "7".repeat(40).as_str(), 1);
    let unrecorded: PathBuf = worktree.join(TRASH_PARENT).join("6".repeat(40));
    put(unrecorded.as_path(), "d0/precious", b"copied by hand\n");
    let mut say: Vec<u8> = Vec::new();
    retry_given_up_trash(git_dir.as_path(), worktree.as_path(), &mut say);
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(unrecorded.to_str().expect("UTF-8")), "the person is told what sits there: {said}");
    assert!(unrecorded.join("d0/precious").exists(), "nothing is deleted");
    remove(root.as_path());
}

/// Finding 12:
/// the service's handler does not follow a link at `.claude`,
/// so a trash that is only reached through a link is
///  refused and what the link points at is untouched.
#[test]
fn the_handler_does_not_follow_a_link_at_the_claude_directory() {
    let name: String = "d".repeat(40);
    let (root, worktree, git_dir, outside, manifest) = worktree_with_trash_moved_out("trash-link-claude", name.as_str());
    std::fs::remove_dir_all(worktree.join(".claude")).expect("the empty .claude");
    std::os::unix::fs::symlink(&outside, worktree.join(".claude")).expect("link");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(verdict, Verdict::Failed(_)), "{verdict:?}");
    assert!(outside.join("cli-git-trash").join(&name).join("d0/file").exists(), "what the link points at is untouched");
    remove(root.as_path());
}

/// Finding 12:
/// the foreground fallback does not follow a link at the trash parent either,
/// and says so.
#[test]
fn the_foreground_fallback_does_not_follow_a_link_at_the_trash_parent() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, outside, manifest) = worktree_with_trash_moved_out("trash-link-parent", name.as_str());
    std::os::unix::fs::symlink(outside.join("cli-git-trash"), worktree.join(TRASH_PARENT)).expect("link");
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return None; }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT });
    let reason: String = outcome.expect_err("a trash reached through a link is not emptied");
    assert!(reason.contains("cli-git-trash"), "the reason names the path: {reason}");
    assert!(outside.join("cli-git-trash").join(&name).join("d0/file").exists(), "what the link points at is untouched");
    remove(root.as_path());
}

/// Finding 12:
/// a worktree whose `.git` file points at another administrative directory is not the worktree of the record.
#[test]
fn the_handler_refuses_a_worktree_that_points_at_another_administrative_directory() {
    let name: String = "f".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-other-admin", name.as_str(), 1);
    std::fs::write(worktree.join(".git"), b"gitdir: /nowhere/else\n").expect("another pointer");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains("/nowhere/else")), "{verdict:?}");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0/file").exists());
    remove(root.as_path());
}

/// Finding 12:
/// the leave-out does not make its trash through a link at `.claude` in the snapshot,
/// which would put the
///  left-out directories somewhere outside the new worktree;
/// it removes them in place instead.
#[test]
fn btrfs_the_leave_out_does_not_make_its_trash_through_a_link_at_the_claude_directory() {
    let Some(root) = btrfs_fixture("trash-link-stage") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let outside: PathBuf = root.join("outside");
    std::fs::create_dir_all(&outside).expect("outside");
    std::fs::remove_dir_all(stage.join(".claude")).expect("the snapshot's .claude");
    std::os::unix::fs::symlink(&outside, stage.join(".claude")).expect("link");
    let pruned: Pruned = prune_snapshot(stage.as_path(), &source).expect("pruned");
    let found: Vec<String> = std::fs::read_dir(&outside).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert!(found.is_empty(), "nothing is made or moved through the link: {found:?}");
    assert_eq!(pruned.trash, None, "there is no trash to hand over");
    assert!(pruned.count > 0, "the caches and the other checkouts are still left out");
    assert!(std::fs::symlink_metadata(stage.join("node_modules/.cache")).is_err(), "removed in place");
    remove(root.as_path());
}

/// Finding 13:
/// a directory whose name is not UTF-8 and a link named with the replacement character are two entries,
/// and the
///  walk never acts on the link's target for the directory.
#[test]
fn btrfs_a_directory_with_a_non_utf8_name_is_not_taken_for_a_link_named_with_the_replacement_character() {
    let Some(root) = btrfs_fixture("trash-non-utf8") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let outside: PathBuf = root.join("outside");
    put(outside.as_path(), ".git", b"gitdir: /elsewhere\n");
    put(outside.as_path(), "data", b"precious\n");
    std::fs::create_dir_all(stage.join(std::ffi::OsStr::from_bytes(b"\xff"))).expect("the non-UTF-8 directory");
    std::os::unix::fs::symlink(&outside, stage.join("\u{fffd}")).expect("link");
    prune_snapshot(stage.as_path(), &source).expect("pruned");
    assert!(outside.join(".git").exists(), "the pointer outside the snapshot is not unlinked");
    assert!(outside.join("data").exists(), "the directory outside the snapshot is not moved");
    assert!(std::fs::symlink_metadata(stage.join("\u{fffd}")).is_ok_and(|found: std::fs::Metadata| { return found.is_symlink(); }), "the link stays");
    assert!(stage.join(std::ffi::OsStr::from_bytes(b"\xff")).is_dir(), "the directory with the non-UTF-8 name stays");
    remove(root.as_path());
}

/// Finding 13:
/// a cache below a directory whose name is not UTF-8 is still found,
/// and a name that only looks like a cache once
///  its bytes are replaced is not one.
#[test]
fn btrfs_non_utf8_names_are_judged_by_their_bytes() {
    let Some(root) = btrfs_fixture("trash-non-utf8-rules") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let lookalike: &std::ffi::OsStr = std::ffi::OsStr::from_bytes(b".cache\xff");
    let odd: &std::ffi::OsStr = std::ffi::OsStr::from_bytes(b"\xfe");
    std::fs::create_dir_all(stage.join("package/b").join(lookalike)).expect("a directory that is not named .cache");
    std::fs::create_dir_all(stage.join("package/b/.cache")).expect("a cache");
    std::fs::create_dir_all(stage.join("node_modules").join(odd).join(".cache")).expect("a cache below a non-UTF-8 directory");
    prune_snapshot(stage.as_path(), &source).expect("pruned");
    assert!(stage.join("package/b").join(lookalike).is_dir(), "a name that is not .cache stays");
    assert!(std::fs::symlink_metadata(stage.join("package/b/.cache")).is_err(), "a cache is left out");
    assert!(std::fs::symlink_metadata(stage.join("node_modules").join(odd).join(".cache")).is_err(), "a cache below a directory whose name is not UTF-8 is left out");
    remove(root.as_path());
}

/// Finding 14:
/// a worktree whose path is not UTF-8 is never named by a work item that spells another path;
/// its trash is
///  emptied here,
/// by the bytes of the path.
#[cfg(target_os = "linux")]
#[test]
fn a_non_utf8_worktree_is_emptied_here_and_no_work_item_names_another_path() {
    let name: String = "9".repeat(40);
    let root: PathBuf = fixture("trash-hand-over-bytes");
    let worktree: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"topic-\xff"));
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    let address: String = format!("trash-bytes-{}", std::process::id());
    let service: FixtureService = serve_earlier_release(address.as_str(), true);
    let mut notices: Vec<u8> = Vec::new();
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let outcome: Result<(), String> = dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return Some(route.clone()); }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT });
    service.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let heard: Vec<String> = service.thread.join().expect("service thread");
    outcome.expect("emptied here");
    assert!(heard.is_empty(), "the service was never asked about a path it cannot be told: {heard:?}");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(!record.exists(), "no work item names a lossy spelling of the path");
    assert!(!worktree.join(TRASH_PARENT).exists(), "the trash was emptied by the path's bytes");
    remove(root.as_path());
}

/// Restores the mode of a directory when dropped,
/// so a failed control still leaves a fixture that can be removed.
struct RestoreMode {
    /// The directory.
    path: PathBuf,
}

impl Drop for RestoreMode {
    fn drop(&mut self) {
        let _restored: std::io::Result<()> = std::fs::set_permissions(&self.path, std::os::unix::fs::PermissionsExt::from_mode(0o755));
    }
}

/// Finding 15:
/// a trash that cannot be emptied here is reported with its path and the reason,
/// and the record that names it
///  stays for a later service.
#[test]
fn a_trash_that_cannot_be_emptied_here_is_reported_and_keeps_its_work_item() {
    let name: String = "7".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-unremovable", name.as_str(), 1);
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    // A parent nobody may write to: the contents can go, the trash directory itself cannot.
    std::fs::set_permissions(&parent, std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only parent");
    let restore: RestoreMode = RestoreMode { path: parent.clone() };
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return None; }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT });
    let reason: String = outcome.expect_err("the failure is returned, not swallowed");
    assert!(reason.contains(name.as_str()), "the reason names the trash: {reason}");
    assert!(reason.contains("stays"), "the reason says what happens to it: {reason}");
    assert!(parent.join(&name).exists(), "the trash directory could not be removed from a read-only parent");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(record.exists(), "the work item stays while the trash is still there");
    drop(restore);
    remove(root.as_path());
}

/// Finding 15:
/// the handler reports a trash it cannot remove as a failure,
/// never as done.
#[test]
fn the_handler_reports_a_trash_it_cannot_remove() {
    let name: String = "6".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-handler-unremovable", name.as_str(), 1);
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    std::fs::set_permissions(&parent, std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only parent");
    let restore: RestoreMode = RestoreMode { path: parent.clone() };
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(matches!(&verdict, Verdict::Failed(reason) if reason.contains(&name)), "{verdict:?}");
    drop(restore);
    remove(root.as_path());
}

/// A provisioning host that no control reaches:
/// the provisioner of a control is a closure.
fn unreachable_host() -> crate::service_provision::ProvisionHost {
    return crate::service_provision::ProvisionHost {
        runtime_dir: PathBuf::from("/nonexistent-runtime"),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent-bus"),
        service_binary: PathBuf::from("/nonexistent-service"),
        git: PathBuf::from("/nonexistent-git"),
        instance: String::from("fixture-instance"),
        git_dir: std::path::PathBuf::new(),
        retirement_wait: std::time::Duration::from_millis(10),
    };
}

/// A new worktree has a commit service instance of its own and no socket until something sets it up:
/// the hand-over sets it
///  up first,
/// as a commit does,
/// and the service then takes the work item.
/// (Without that step the wake waited out the whole
///  start for a socket that nothing would create,
/// and the trash was then emptied in the foreground for minutes.)
#[cfg(target_os = "linux")]
#[test]
fn the_sockets_of_a_new_worktree_are_set_up_before_the_service_is_woken() {
    let name: String = "5".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-provision", name.as_str(), 1);
    let address: String = format!("trash-provision-{}", std::process::id());
    let started: std::cell::RefCell<Option<FixtureService>> = std::cell::RefCell::new(None);
    let provisioned: std::cell::Cell<usize> = std::cell::Cell::new(0);
    let route: Route = Route { address: Bind::Abstract(address.clone()), provision: Some(unreachable_host()) };
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash_with(
        Path::new("/usr/bin/git"),
        worktree.as_path(),
        git_dir.as_path(),
        &manifest,
        &mut notices,
        &Reach {
            router: &|_git: &Path, _dir: &Path| { return Some(route.clone()); },
            // The socket exists only once the worktree is provisioned.
            provisioner: &|host: &ProvisionHost| {
                provisioned.set(provisioned.get() + 1);
                assert_eq!(host.instance, "fixture-instance");
                *started.borrow_mut() = Some(serve_earlier_release(address.as_str(), true));
                return Ok(());
            },
            start_wait: std::time::Duration::from_millis(400),
        },
    );
    outcome.expect("handed over");
    let service: FixtureService = started.borrow_mut().take().expect("the provisioner ran");
    service.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let heard: Vec<String> = service.thread.join().expect("service thread");
    assert_eq!(provisioned.get(), 1, "provisioned once");
    let work: String = crate::service_protocol::encode_client(&crate::service_protocol::ClientMessage::Work { git_dir: git_dir.clone(), review_key: None });
    assert!(heard.contains(&work), "the service was woken for the worktree: {heard:?}");
    assert!(worktree.join(TRASH_PARENT).join(&name).exists(), "the service empties it, not the wrapper");
    assert!(!String::from_utf8_lossy(notices.as_slice()).contains("emptying the directories"), "no foreground line: {}", String::from_utf8_lossy(notices.as_slice()));
    remove(root.as_path());
}

/// When the worktree cannot be provisioned,
/// the trash is emptied here after one plain line that says what is done and why,
///  and the reason is the manager's own,
/// not an internal term.
#[test]
fn a_provisioning_failure_is_said_once_before_the_foreground_removal() {
    let name: String = "8".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-provision-fails", name.as_str(), 1);
    let route: Route = Route { address: Bind::Abstract(String::from("trash-provision-fails-never-bound")), provision: Some(unreachable_host()) };
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash_with(
        Path::new("/usr/bin/git"),
        worktree.as_path(),
        git_dir.as_path(),
        &manifest,
        &mut notices,
        &Reach {
            router: &|_git: &Path, _dir: &Path| { return Some(route.clone()); },
            provisioner: &|_host: &ProvisionHost| { return Err(crate::service_messages::ClientFailure::CouldNotStart(String::from("the session bus refused the call"))); },
            start_wait: START_WAIT,
        },
    );
    outcome.expect("emptied here");
    let said: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert_eq!(said.lines().count(), 1, "one line: {said}");
    assert!(said.contains(worktree.display().to_string().as_str()) && said.contains("the session bus refused the call"), "the line names the worktree and the reason: {said}");
    assert!(!worktree.join(TRASH_PARENT).exists(), "emptied in the foreground");
    remove(root.as_path());
}

/// When no service can be started from the session,
/// the trash is emptied here after one plain line.
#[test]
fn no_route_is_said_once_before_the_foreground_removal() {
    let name: String = "4".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-no-route", name.as_str(), 1);
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return None; }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT }).expect("emptied here");
    let said: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert_eq!(said.lines().count(), 1, "one line: {said}");
    assert!(said.contains("no commit service can be started"), "{said}");
    remove(root.as_path());
}
