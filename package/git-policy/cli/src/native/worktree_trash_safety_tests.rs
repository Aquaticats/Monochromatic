//! Controls for the findings of the cross-vendor review of the leave-out and its disposal: what pruning may move, what a
//!  disposal may delete, and what a failed cleanup must report.

use super::*;
use crate::fs_btrfs_ioctl::snapshot_create_at;
use std::os::unix::ffi::OsStrExt;

/// A snapshot of the main checkout at a stage name beside the repository, taken without the leave-out, for the controls that
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

/// The item the service would be handed for a trash directory named `name`.
fn trash_item(worktree: &Path, git_dir: &Path, name: &str) -> WorkItem {
    return WorkItem { kind: WorkKind::Trash, repository: worktree.display().to_string(), git_dir: git_dir.display().to_string(), branch_ref: String::from(TRASH_REFERENCE), oid: String::from(name), parent: String::from(name), created: now_iso(), tries: 0 };
}

/// Finding 9: a registered nested worktree that holds a file the main index tracks cannot be left out of the snapshot, so the
///  leave-out refuses before it unlinks a pointer or moves a directory, and says which path holds the tracked file.
#[test]
fn btrfs_a_registered_nested_worktree_that_holds_a_tracked_file_is_refused() {
    let Some(root) = btrfs_fixture("trash-nested-tracked") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", "ignored/wt"]);
    tracked(repo.as_path(), "ignored/wt/keep", b"tracked inside a registered worktree\n");
    let blob: String = String::from_utf8(git(repo.as_path(), &["hash-object", "-w", "--no-filters", "ignored/wt/keep"]).stdout).expect("utf8").trim().to_string();
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("100644,{blob},ignored/wt/keep").as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=track a file inside the nested worktree"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\n").expect("exclude");
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    assert!(source.nested.iter().any(|path: &PathBuf| { return path.ends_with("ignored/wt"); }), "the nested worktree is registered: {:?}", source.nested);
    let outcome: Result<usize, String> = prune_snapshot(stage.as_path(), &source);
    let reason: String = outcome.clone().expect_err(format!("pruning must refuse, but it set aside {outcome:?} and ignored/wt/keep is {}", if stage.join("ignored/wt/keep").exists() { "still there" } else { "gone" }).as_str());
    assert!(reason.contains("ignored/wt"), "the reason names the path and the tracked file: {reason}");
    assert!(stage.join("ignored/wt/keep").exists(), "the tracked file is still in the snapshot");
    remove(root.as_path());
}

/// Finding 9, at the level of the whole snapshot: the reason the rung gives names the nested worktree and the tracked file
///  inside it, not a later difference between the snapshot and the index.
#[test]
fn btrfs_a_snapshot_over_a_nested_worktree_with_a_tracked_file_says_why() {
    let Some(root) = btrfs_fixture("trash-nested-tracked-snapshot") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", "ignored/wt"]);
    tracked(repo.as_path(), "ignored/wt/keep", b"tracked inside a registered worktree\n");
    let blob: String = String::from_utf8(git(repo.as_path(), &["hash-object", "-w", "--no-filters", "ignored/wt/keep"]).stdout).expect("utf8").trim().to_string();
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("100644,{blob},ignored/wt/keep").as_str()]);
    git(repo.as_path(), &["commit", "--quiet", "--message=track a file inside the nested worktree"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("../topic")).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: std::fs::File = crate::fs_beneath::open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let failure = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice()).expect_err("the rung declines");
    assert!(failure.reason.contains("ignored/wt"), "the reason names the nested worktree: {}", failure.reason);
    remove(root.as_path());
}

/// Finding 10: a directory the index holds as a gitlink is protected at its own path, so it is not set aside as a copied
///  checkout. (A source whose index holds a gitlink is refused before a snapshot is taken; the leave-out still must not rely on that.)
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

/// Finding 11: a directory in the trash parent that the leave-out did not make (the main checkout's own, carried by the
///  snapshot) is not the snapshot's trash and is never emptied, whatever its name looks like.
#[test]
fn btrfs_a_hex_named_directory_the_snapshot_carried_is_never_emptied() {
    let Some(root) = btrfs_fixture("trash-carried") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let carried: String = "a".repeat(40);
    put(repo.as_path(), format!("{TRASH_PARENT}/{carried}/d0/precious").as_str(), b"main's own ignored file\n");
    let (stage, _source) = snapshot_of(root.as_path(), repo.as_path());
    let parent: PathBuf = stage.join(TRASH_PARENT);
    let made: Vec<String> = std::fs::read_dir(&parent).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name: &String| { return *name != carried; }).collect();
    assert_eq!(made.len(), 1, "one trash made by the leave-out beside the carried one: {made:?}");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), stage.as_path(), git_dir.as_path(), &mut notices, &|_git: &Path, _dir: &Path| { return None; }, START_WAIT);
    assert!(parent.join(&carried).join("d0/precious").exists(), "the directory the main checkout carried is untouched");
    assert!(!parent.join(&made[0]).exists(), "the trash the leave-out made is emptied");
    remove(root.as_path());
}

/// A worktree with a `.git` file, and an outside directory holding `cli-git-trash/<name>/d0/precious`.
fn worktree_and_outside(label: &str, name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(label);
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    let outside: PathBuf = root.join("outside");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    put(outside.as_path(), format!("cli-git-trash/{name}/d0/precious").as_str(), b"outside the worktree\n");
    return (root, worktree, git_dir, outside);
}

/// Finding 12: the service's handler does not follow a link at `.claude`, so a trash that is only reached through a link is
///  refused and what the link points at is untouched.
#[test]
fn the_handler_does_not_follow_a_link_at_the_claude_directory() {
    let name: String = "d".repeat(40);
    let (root, worktree, git_dir, outside) = worktree_and_outside("trash-link-claude", name.as_str());
    std::os::unix::fs::symlink(&outside, worktree.join(".claude")).expect("link");
    let verdict: Verdict = empty_trash(&trash_item(worktree.as_path(), git_dir.as_path(), name.as_str()));
    assert!(matches!(verdict, Verdict::Failed(_)), "{verdict:?}");
    assert!(outside.join("cli-git-trash").join(&name).join("d0/precious").exists(), "what the link points at is untouched");
    remove(root.as_path());
}

/// Finding 12: the foreground fallback does not follow a link at the trash parent either.
#[test]
fn the_foreground_fallback_does_not_follow_a_link_at_the_trash_parent() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, outside) = worktree_and_outside("trash-link-parent", name.as_str());
    std::fs::create_dir_all(worktree.join(".claude")).expect(".claude");
    std::os::unix::fs::symlink(outside.join("cli-git-trash"), worktree.join(TRASH_PARENT)).expect("link");
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &mut notices, &|_git: &Path, _dir: &Path| { return None; }, START_WAIT);
    assert!(outside.join("cli-git-trash").join(&name).join("d0/precious").exists(), "what the link points at is untouched");
    remove(root.as_path());
}

/// Finding 12: the leave-out does not make its trash through a link at `.claude` in the snapshot, which would put the
///  left-out directories somewhere outside the new worktree.
#[test]
fn btrfs_the_leave_out_does_not_make_its_trash_through_a_link_at_the_claude_directory() {
    let Some(root) = btrfs_fixture("trash-link-stage") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let outside: PathBuf = root.join("outside");
    std::fs::create_dir_all(&outside).expect("outside");
    std::fs::remove_dir_all(stage.join(".claude")).expect("the snapshot's .claude");
    std::os::unix::fs::symlink(&outside, stage.join(".claude")).expect("link");
    let _outcome: Result<usize, String> = prune_snapshot(stage.as_path(), &source);
    let found: Vec<String> = std::fs::read_dir(&outside).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert!(found.is_empty(), "nothing is made or moved through the link: {found:?}");
    remove(root.as_path());
}

/// Finding 13: a directory whose name is not UTF-8 and a link named with the replacement character are two entries, and the
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

/// Finding 14: the work item names the worktree by the bytes of its path, or is not written for a path it cannot spell.
#[cfg(target_os = "linux")]
#[test]
fn a_work_item_never_names_a_non_utf8_worktree_by_a_different_path() {
    let name: String = "9".repeat(40);
    let root: PathBuf = fixture("trash-hand-over-bytes");
    let worktree: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"topic-\xff"));
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/file").as_str(), b"left out\n");
    let address: String = format!("trash-bytes-{}", std::process::id());
    let service: FixtureService = serve_earlier_release(address.as_str(), true);
    let mut notices: Vec<u8> = Vec::new();
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let _handed: bool = hand_over(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), name.as_str(), &mut notices, &|_git: &Path, _dir: &Path| { return Some(route.clone()); }, START_WAIT);
    service.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let _heard: Vec<String> = service.thread.join().expect("service thread");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    if let Ok(text) = std::fs::read_to_string(&record) {
        let item: WorkItem = WorkItem::decode(text.as_str()).expect("a decodable record");
        assert_eq!(item.repository.as_bytes(), worktree.as_os_str().as_bytes(), "the record names exactly the worktree it was made for");
    }
    remove(root.as_path());
}

/// Restores the mode of a directory when dropped, so a failed control still leaves a fixture that can be removed.
struct RestoreMode {
    /// The directory.
    path: PathBuf,
}

impl Drop for RestoreMode {
    fn drop(&mut self) {
        let _restored: std::io::Result<()> = std::fs::set_permissions(&self.path, std::os::unix::fs::PermissionsExt::from_mode(0o755));
    }
}

/// Finding 15: a trash that cannot be emptied here is not forgotten: the record that names it stays for a later service.
#[test]
fn a_trash_that_cannot_be_emptied_here_keeps_its_work_item() {
    let name: String = "7".repeat(40);
    let root: PathBuf = fixture("trash-unremovable");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/file").as_str(), b"left out\n");
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    // A parent nobody may write to: the contents can go, the trash directory itself cannot.
    std::fs::set_permissions(&parent, std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only parent");
    let restore: RestoreMode = RestoreMode { path: parent.clone() };
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &mut notices, &|_git: &Path, _dir: &Path| { return None; }, START_WAIT);
    assert!(parent.join(&name).exists(), "the trash directory could not be removed from a read-only parent");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(record.exists(), "the work item stays while the trash is still there");
    drop(restore);
    remove(root.as_path());
}
