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

/// What:
///  A directory under the trash parent that the current leave-out did not make is an earlier add's trash,
/// so it
///  is emptied with this add's,
/// while a directory of the same name outside the trash parent is never touched.
/// Why:
///   The trash parent is this wrapper's own namespace,
/// and a kill before the emptying leaves its content there;
///  a snapshot that carried it forward would keep another command's stage container with its credentials,
/// and
///  nothing would ever dispose of it,
/// because the record names only the add that runs.
/// The identity the manifest
///  recorded is what the emptying checks,
/// so a name alone still never removes a directory the leave-out did not move here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(repo, `${TRASH_PARENT}/${earlier}/d0/payload`, 'token'); put(repo, `.claude/${earlier}/keep`, 'mine');
/// const { manifest } = await snapshotOf(repo); await disposePendingTrash(...);
/// expect(existsSync(join(stage, TRASH_PARENT, earlier))).toBe(false); expect(existsSync(join(stage, '.claude', earlier, 'keep'))).toBe(true);
/// ```
#[test]
fn btrfs_an_earlier_adds_trash_is_emptied_with_this_adds() {
    let Some(root) = btrfs_fixture("trash-carried") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let earlier: String = "a".repeat(40);
    put(repo.as_path(), format!("{TRASH_PARENT}/{earlier}/d0/payload/credentials").as_str(), b"another worktree's token\n");
    put(repo.as_path(), format!(".claude/{earlier}/keep").as_str(), b"a directory the leave-out never made\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    let parent: PathBuf = stage.join(TRASH_PARENT);
    assert_ne!(manifest.name, earlier, "the trash is not the directory the earlier add left");
    assert!(!parent.join(&earlier).exists(), "the earlier trash is not carried as itself");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::write(stage.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("pointer");
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash(&git_context(), stage.as_path(), git_dir.as_path(), Some(&manifest), &mut notices).expect("emptied here");
    assert!(!parent.exists(), "both trashes are emptied");
    assert!(stage.join(".claude").join(&earlier).join("keep").exists(), "a directory with that name outside the trash parent is untouched");
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
    let outcome: Result<(), String> = dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices);
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

/// Review round 5,
/// finding 1:
/// an ordinary nested checkout is set aside whole,
/// and the sweep still looks inside it,
/// so its
///  own `.git`
/// directory does not hide another command's stage container.
/// Why:
///   A nested checkout that is a repository of its own has a real `.git` directory,
/// while a linked worktree
///  has a pointer file the leave-out unlinks before the sweep runs;
/// the sweep skipped every directory that
///  held `.git`,
/// which is the directory it was handed,
/// so the container beside that marker travelled into
///  the trash with its credentials and a target commit that stops ignoring the path makes it stageable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(repo, `clone/.claude/worktrees/.cli-git-worktree-copy-<token>/payload/credentials`, 'token');
/// const { stage } = await snapshotOf(repo);
/// expect(holdsANamedFile(join(stage, TRASH_PARENT), 'credentials')).toBe(false);
/// ```
#[test]
fn btrfs_a_container_inside_an_ordinary_nested_checkout_is_removed_with_it() {
    let Some(root) = btrfs_fixture("trash-container-in-other-checkout") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    assert!(repo.join("clone/.git").is_dir(), "the other checkout holds a real administrative directory, not a pointer");
    let inside: String = format!("clone/.claude/worktrees/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    put(repo.as_path(), format!("{inside}/payload/credentials").as_str(), b"another worktree's token\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    assert!(std::fs::symlink_metadata(stage.join("clone")).is_err(), "the other checkout itself is still set aside whole");
    assert!(!holds_a_named_file(stage.join(TRASH_PARENT).as_path(), "credentials"), "the container beside its .git marker is not in the trash");
    assert!(trash.is_some(), "the leave-out made a trash for the rest");
    remove(root.as_path());
}

/// Review round 5,
/// finding 2:
/// a subtree the sweep cannot examine is not evidence that it holds nothing to sweep,
/// so the leave-out
///  refuses and names it.
/// Why:
///   The sweep asked whether an entry is a directory and turned any failure into `false`,
/// which skipped
///  the subtree and reported success;
/// an inherited trash directory the person can list and that can be renamed but not
///  looked inside then travelled into this add's trash with another command's stage container and
///  credentials.
/// Mode `0600` is that directory:
/// read and write let the sweep list it and the leave-out rename
///  it,
/// while the missing search bit makes every lookup inside it fail (measured on this host:
/// a
///  directory at `0400` cannot be renamed at all,
/// `rename` answers `EACCES`,
/// so the leave-out already fails before it
///  moves anything).
/// A subtree nobody examined is not a subtree with nothing in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(repo, `${TRASH_PARENT}/${earlier}/wrapper/.cli-git-worktree-copy-<token>/payload/credentials`, 'token');
/// chmod(join(repo, TRASH_PARENT, earlier), 0o600);
/// await expect(pruneSnapshot(stage, source)).rejects.toContain(join(TRASH_PARENT, earlier, 'wrapper'));
/// ```
#[test]
fn btrfs_a_subtree_the_sweep_cannot_examine_is_left_where_it_is_and_named() {
    let Some(root) = btrfs_fixture("trash-unexamined-subtree") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let earlier: String = "b".repeat(40);
    let wrapper: String = format!("{TRASH_PARENT}/{earlier}/wrapper/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    let token: PathBuf = Path::new(wrapper.as_str()).join("payload/credentials");
    put(repo.as_path(), token.to_str().expect("a path of text"), b"another worktree's token\n");
    let unsearchable: PathBuf = repo.join(TRASH_PARENT).join(&earlier);
    std::fs::set_permissions(&unsearchable, std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("a directory that can be listed and renamed but not looked inside");
    let restore: RestoreMode = RestoreMode { path: unsearchable.clone() };
    let (stage, source) = unpruned_snapshot(root.as_path(), repo.as_path());
    let restore_stage: RestoreMode = RestoreMode { path: stage.join(TRASH_PARENT).join(&earlier) };
    let outcome: Result<Pruned, String> = prune_snapshot(stage.as_path(), &source);
    let reason: String = outcome.clone().expect_err(format!("pruning must refuse a subtree it cannot examine, but it returned {outcome:?} and the credentials are {}", if stage.join(&token).exists() { "still there" } else { "gone" }).as_str());
    assert!(reason.contains(format!("{TRASH_PARENT}/{earlier}/wrapper").as_str()), "the reason names the subtree it could not examine: {reason}");
    // The mode is put back before anything is looked at inside the subtree: without its search bit no lookup below it succeeds.
    drop(restore_stage);
    drop(restore);
    assert!(stage.join(&token).exists(), "the subtree keeps its files where it is");
    assert!(repo.join(&token).exists(), "and the source keeps them too");
    remove(root.as_path());
}

/// Finding 14:
/// a worktree whose path is not UTF-8 has its trash emptied here,
/// by the bytes of the path,
/// and nothing
///  names another path.
#[cfg(target_os = "linux")]
#[test]
fn a_non_utf8_worktree_is_emptied_by_its_path_bytes() {
    let name: String = "9".repeat(40);
    let root: PathBuf = fixture("trash-hand-over-bytes");
    let worktree: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"topic-\xff"));
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices).expect("emptied here");
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

/// What:
///  A trash that cannot be emptied is reported with its path and the reason,
/// and the line says what is
///  true of it:
/// it stays,
/// it is the person's to remove,
/// and nothing else will empty it.
/// Why:
///   This path writes no work item and keeps no durable record of the manifest,
/// so a line promising that
///  the commit service names it later would promise work that never runs;
/// the person is the only one who will
///  empty it,
/// and the line gives the command that does,
/// quoted for their shell.
/// It also says that no work item exists,
/// which is the fact the old sentence got wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = await disposePendingTrash(...); expect(reason).toContain(`rm --recursive --force -- '${join(worktree, TRASH_PARENT, name)}'`); expect(reason).not.toContain('commit service');
/// ```
#[test]
fn a_trash_that_cannot_be_emptied_here_is_reported_as_the_persons_to_remove() {
    let name: String = "7".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-unremovable", name.as_str(), 1);
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    // A parent nobody may write to: the contents can go, the trash directory itself cannot.
    std::fs::set_permissions(&parent, std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only parent");
    let restore: RestoreMode = RestoreMode { path: parent.clone() };
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices);
    let reason: String = outcome.expect_err("the failure is returned, not swallowed");
    let trash_path: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let quoted: String = crate::shell_quote::shell_quote(trash_path.as_os_str().as_bytes());
    assert!(reason.contains(quoted.as_str()), "the reason names the trash the way the person's shell reads it: {reason}");
    assert!(reason.contains("stays inside the worktree") && reason.contains("only you will remove it"), "the reason says what is true of it: {reason}");
    assert!(reason.contains(format!("rm --recursive --force -- {quoted}").as_str()), "the reason gives the command that removes it: {reason}");
    assert!(!reason.contains("commit service"), "and promises no service that would never see it: {reason}");
    assert!(parent.join(&name).exists(), "the trash directory could not be removed from a read-only parent");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(!record.exists(), "no work item is written, so a later command does not report a trash it never made");
    drop(restore);
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash that was replaced under the recorded name is not handed to the person with the command that removes
///  a recursive tree,
/// because that command would delete the directory standing there now.
/// Why:
///   The emptying refused to delete it precisely because it is not the object the snapshot made;
/// a line that still
///  ends in `rm --recursive --force` overrides that refusal and tells the person to delete somebody else's
///  data.
/// The line names the path,
/// says what stands there instead,
/// and gives no removal command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename(join(worktree, TRASH_PARENT, name), movedAway); put(join(worktree, TRASH_PARENT, name), 'theirs/file', 'x');
/// const reason = await disposePendingTrash(...); expect(reason).toContain('not the directory the snapshot made'); expect(reason).not.toContain('rm --recursive --force');
/// ```
#[test]
fn a_trash_that_is_not_the_snapshots_is_not_given_a_removal_command() {
    let name: String = "8".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-replaced-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(&trash, worktree.join(TRASH_PARENT).join("moved-away")).expect("move the real trash away");
    put(trash.as_path(), "theirs/file", b"somebody else's data\n");
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices);
    let reason: String = outcome.expect_err("a trash that is not the snapshot's is left alone");
    let quoted: String = crate::shell_quote::shell_quote(trash.as_os_str().as_bytes());
    assert!(reason.contains(quoted.as_str()), "the reason names the path that stays: {reason}");
    assert!(reason.contains("not the directory the snapshot made"), "the reason says why it was left: {reason}");
    assert!(!reason.contains("rm --recursive --force"), "no removal command is given for a directory the snapshot did not make: {reason}");
    assert!(trash.join("theirs/file").exists(), "what stands there is untouched");
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash that holds a file the snapshot did not put there is not handed to the person with the command that removes
///  a recursive tree either.
/// Why:
///   The emptying emptied the recorded entry and then left the whole trash directory because of that file;
/// the file
///  is deliberately kept,
/// so a command that removes the trash recursively deletes exactly what the refusal
///  preserved.
/// The line names the path and the file and gives no removal command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(join(worktree, TRASH_PARENT, name, 'unrecorded/file'), 'not ours');
/// const reason = await disposePendingTrash(...); expect(reason).toContain('unrecorded'); expect(reason).not.toContain('rm --recursive --force');
/// ```
#[test]
fn a_trash_holding_what_was_not_moved_there_is_not_given_a_removal_command() {
    let name: String = "c".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-extra-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    put(trash.as_path(), "unrecorded/file", b"not ours\n");
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices);
    let reason: String = outcome.expect_err("a trash holding what the snapshot did not put there is left alone");
    let quoted: String = crate::shell_quote::shell_quote(trash.as_os_str().as_bytes());
    assert!(reason.contains(quoted.as_str()) && reason.contains("unrecorded"), "the reason names the path and what it kept: {reason}");
    assert!(!reason.contains("rm --recursive --force"), "no removal command is given for a directory holding what the emptying kept: {reason}");
    assert!(trash.join("unrecorded/file").exists(), "the file the emptying kept is untouched");
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash entry that is no longer the object the snapshot moved there is deliberately left,
/// so the line that
///  reports it gives no removal command for the trash either.
/// Why:
///   The emptying compares each entry's identity and skips the ones that changed;
/// those are the entries the
///  person would lose by following a recursive removal of the trash,
/// and the line must not suggest one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename(join(trash, 'd0'), join(worktree, 'moved-away')); put(join(trash, 'd0/precious'), 'theirs');
/// const reason = await disposePendingTrash(...); expect(reason).toContain('d0'); expect(reason).not.toContain('rm --recursive --force');
/// ```
#[test]
fn a_trash_entry_that_was_replaced_is_not_given_a_removal_command() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-entry-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(trash.join(entry_name(0).as_str()), worktree.join("moved-away")).expect("move the recorded entry away");
    put(trash.as_path(), format!("{}/precious", entry_name(0)).as_str(), b"somebody else's directory\n");
    let mut notices: Vec<u8> = Vec::new();
    let outcome: Result<(), String> = dispose_pending_trash(&git_context(), worktree.as_path(), git_dir.as_path(), Some(&manifest), &mut notices);
    let reason: String = outcome.expect_err("an entry that changed is left where it is");
    assert!(reason.contains(entry_name(0).as_str()), "the reason names the entry it kept: {reason}");
    assert!(!reason.contains("rm --recursive --force"), "no removal command is given for a directory whose entry stayed: {reason}");
    assert!(trash.join(entry_name(0).as_str()).join("precious").exists(), "the entry that changed is untouched");
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
