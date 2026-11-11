//! Controls for the findings of the cross-vendor review of the leave-out and its emptying:
//! what pruning may move,
//! what the
//!  emptying may delete,
//! and what a failed emptying must report.

use super::*;
use crate::worktree_snapshot_ahead::AheadService;
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
    let failure = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice(), &crate::fixture_content_decision::content_environment(), &AheadService::nothing()).expect_err("the rung declines");
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
/// plantTrash(repo, earlier); writeTrashManifest(repo, earlier); put(repo, `.claude/${earlier}/keep`, 'mine');
/// const { manifest } = await snapshotOf(repo); await emptyTrash(itemOf(stage, commonDir, manifest));
/// expect(existsSync(join(stage, TRASH_PARENT, earlier))).toBe(false); expect(existsSync(join(stage, '.claude', earlier, 'keep'))).toBe(true);
/// ```
#[test]
fn btrfs_an_earlier_adds_trash_is_emptied_with_this_adds() {
    let Some(root) = btrfs_fixture("trash-carried") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let earlier: String = "a".repeat(40);
    let earlier_manifest: TrashManifest = plant_trash(repo.as_path(), earlier.as_str(), 1);
    crate::worktree_trash_own_manifest::write(repo.as_path(), &earlier_manifest).expect("the earlier trash says what it is");
    put(repo.as_path(), format!(".claude/{earlier}/keep").as_str(), b"a directory the leave-out never made\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    let parent: PathBuf = stage.join(TRASH_PARENT);
    assert_ne!(manifest.name, earlier, "the trash is not the directory the earlier add left");
    assert!(!parent.join(&earlier).exists(), "the earlier trash is not carried as itself");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::write(stage.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("pointer");
    assert_eq!(empty_trash(&item_of(stage.as_path(), git_dir.as_path(), &manifest)), Verdict::Done, "the handler empties what the leave-out recorded");
    assert!(!parent.exists(), "this add's trash and the one it took over go together");
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

/// What:
///  A trash that a command could not empty is named to the person by the next command in its worktree,
///  with its path,
///  and nothing deletes it.
/// Why:
///   A command empties the trash of its own snapshot before it returns (`empty_before_returning`),
/// so a disposal that failed leaves the directory in the worktree with no record naming it,
/// and nothing else would ever mention it again.
/// The trash is a full copy of the directories the leave-out moved,
/// so a person who is not told cannot find it,
/// and a scan that deleted what it cannot prove is a trash would discard another command's state.
#[test]
fn a_trash_that_could_not_be_emptied_is_named_and_left_alone() {
    let name: String = "9".repeat(40);
    let (root, worktree, git_dir, _manifest) = worktree_with_planted_trash("trash-named", name.as_str(), 2);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(trash.to_str().expect("UTF-8")), "the person is told which trash stays: {said}");
    assert!(trash.join("d0").is_dir() && trash.join("d1").is_dir(), "nothing in it is deleted");
    // Said once for the command it is said in, and again by the next one, so a long-lived worktree does not lose it.
    let mut second: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut second, &nothing_woken());
    assert!(String::from_utf8_lossy(second.as_slice()).contains(trash.to_str().expect("UTF-8")), "the next command says it again: {}", String::from_utf8_lossy(second.as_slice()));
    remove(root.as_path());
}

/// What:
///  A trash that a record of this worktree names is not named as unaccounted for,
///  and the record and the trash are left as they are.
/// Why:
///   The record is work of the service,
/// whose own handler empties the trash it names (`empty_trash`),
/// and a command that emptied a trash the service is working on would be a second writer,
/// while naming it would send the person to look at work that is not theirs.
#[test]
fn a_trash_a_record_accounts_for_is_not_named() {
    let name: String = "a".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-recorded", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("record");
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(!said.contains(trash.to_str().expect("UTF-8")), "a trash its record accounts for is not named: {said}");
    assert!(trash.join("d0/file").is_file(), "the trash the record names is not touched");
    remove(root.as_path());
}

/// What:
///  A trash whose record nothing ever claimed is named to the person once one claim lease has passed,
/// and left alone.
/// Why:
///   The add writes the record and wakes the service,
/// but a wake that failed,
/// or an add killed between the two,
/// leaves a
///  record no service ever read;
/// a service claims a record within the wake that asked it to,
/// so a pending record older than
///  the lease the drain keeps is one nobody will look at,
/// and naming it is the only way the person learns of it.
/// The file's age is what says so,
/// so the control moves the record's modification time back instead of waiting it out.
#[test]
fn a_pending_record_no_service_took_is_named_after_one_claim_lease() {
    let name: String = "f".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-pending-stale", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work.as_path(), crate::work_item::PENDING).join(item.id());
    age_by(&pending, std::time::Duration::from_secs(600));
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(trash.to_str().expect("UTF-8")), "a record nothing claimed is named with its trash: {said}");
    assert!(pending.is_file() && trash.join("d0/file").is_file(), "the record and the trash stay");
    remove(root.as_path());
}

/// What:
///  A trash that carries valid manifest bytes naming another directory is not the wrapper's,
///  so the file at that name and the trash directory itself both stay,
///  and the emptying says why.
/// Why:
///   The proof is the identity the manifest names,
/// compared with the directory that holds it;
/// a manifest that decodes but names another directory is somebody's file,
/// and the emptying removes only what it can prove the leave-out moved there.
/// The entries are still the recorded objects,
/// so those go,
/// and the trash directory stays only because the file in it is not the wrapper's.
#[test]
fn a_manifest_naming_another_trash_keeps_its_file_and_the_trash() {
    let name: String = "9".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-foreign-manifest", name.as_str(), 1);
    crate::worktree_trash_own_manifest::write(worktree.as_path(), &manifest).expect("the trash says what it is");
    let foreign: TrashManifest = TrashManifest {
        name: name.clone(),
        trash: ObjectId { device: manifest.trash.device, inode: manifest.trash.inode.wrapping_add(1), birth: manifest.trash.birth },
        entries: Vec::new(),
    };
    let file: PathBuf = worktree.join(TRASH_PARENT).join(&name).join(crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME);
    std::fs::write(file.as_path(), foreign.encode()).expect("a manifest that names another directory");
    let verdict: Verdict = empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest));
    assert!(file.is_file(), "the file that is not the wrapper's stays; the emptying answered {verdict:?}");
    assert!(matches!(verdict, Verdict::Failed(_)), "and the trash directory stays with it, got {verdict:?}");
    remove(root.as_path());
}

/// What:
///  A trash whose record the service already settled is named as one nothing will empty,
///  even though the trash says what it is,
///  and no second record is invented for it.
/// Why:
///   The queue answers `Known` for an id that is already in `failed` or `done`,
/// and no round claims a settled record,
///  so a scan that said the trash "is recorded now and the commit service empties it"
/// would promise a run that never comes.
/// A failed record is where the promise of a later attempt stops
/// (`SPEC.md`,
/// "A trash that cannot be emptied ...").
#[test]
fn a_settled_record_makes_the_notice_name_the_trash_instead_of_promising_a_run() {
    let name: String = "a".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-settled-manifest", name.as_str(), 1);
    crate::worktree_trash_own_manifest::write(worktree.as_path(), &manifest).expect("the trash says what it is");
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work.as_path(), crate::work_item::PENDING).join(item.id());
    let failed: PathBuf = state_directory(work.as_path(), crate::work_item::FAILED).join(item.id());
    std::fs::rename(pending.as_path(), failed.as_path()).expect("the service gave up on it");
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    assert!(said.contains(trash.to_str().expect("UTF-8")), "the trash is named: {said}");
    assert!(!said.contains("recorded now"), "and nothing promises a record was written: {said}");
    assert!(failed.is_file() && !pending.exists(), "no pending record is invented from a settled one");
    remove(root.as_path());
}

/// A scan that asks no service,
/// which is what a control wants:
/// no socket,
/// no service manager and no session bus.
fn nothing_woken() -> crate::held_push_release_wakes_the_service::NothingWoken {
    return crate::held_push_release_wakes_the_service::NothingWoken;
}

/// What:
///  A trash recovered from the manifest it carries tells the repository's service,
///  because a record alone is not a run.
/// Why:
///  The service drains what it is woken for,
/// and it is woken by a hand-over,
/// by a commit,
/// or by an add;
/// a scan that only wrote the record
///  left the trash pending until some unrelated wake,
/// which need never come,
/// while the line it printed said the
///  commit service empties it.
#[test]
fn a_recovered_trash_tells_the_service() {
    let name: String = "6".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-recovered-wake", name.as_str(), 1);
    crate::worktree_trash_own_manifest::write(worktree.as_path(), &manifest).expect("the trash says what it is");
    let woken = Woken::default();
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &woken);
    assert_eq!(woken.named(), vec![git_dir.clone()], "the repository's service is told: {}", String::from_utf8_lossy(say.as_slice()));
    let pending: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), crate::work_item::PENDING).join(item_of(worktree.as_path(), git_dir.as_path(), &manifest).id());
    assert!(pending.is_file(), "and the record it empties stands");
    remove(root.as_path());
}

/// A wake a control can read,
/// which reaches no socket and no service manager.
#[derive(Clone, Default)]
struct Woken(std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>);

impl Woken {
    /// The Git directories the scan has told so far,
    /// in the order it told them.
    fn named(&self) -> Vec<PathBuf> {
        return self.0.lock().map(|named: std::sync::MutexGuard<'_, Vec<PathBuf>>| return named.clone()).unwrap_or_default();
    }
}

impl crate::held_push_release_wakes_the_service::WakeTheService for Woken {
    fn wake(&self, git_dir: &Path) {
        if let Ok(mut named) = self.0.lock() {
            named.push(git_dir.to_path_buf());
        }
    }
}

/// Move a file's modification time into the past,
/// so a control can read a record as one that waited.
fn age_by(path: &Path, back: std::time::Duration) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let past: libc::time_t = (std::time::SystemTime::now() - back).duration_since(std::time::SystemTime::UNIX_EPOCH).expect("after the epoch").as_secs() as libc::time_t;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: past, tv_nsec: 0 }, libc::timespec { tv_sec: past, tv_nsec: 0 }];
    // SAFETY: `name` is a NUL-terminated string that outlives the call, and `times` holds the two timestamps it reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), 0) };
    assert_eq!(result, 0, "age {}", path.display());
}

/// What:
///  A trash whose record the commit service gave up on is named to the person and left alone.
/// Why:
///   The service settles a trash item it cannot empty as failed and never claims it again,
/// so that record accounts for nothing,
/// and without this scan the trash,
/// a full copy of what a snapshot left out,
/// would sit in the worktree unmentioned for ever.
#[test]
fn a_trash_the_service_gave_up_on_is_named_and_left_alone() {
    let name: String = "b".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-given-up", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work.as_path(), crate::work_item::PENDING).join(item.id());
    let failed: PathBuf = state_directory(work.as_path(), crate::work_item::FAILED).join(item.id());
    std::fs::rename(pending.as_path(), failed.as_path()).expect("the service gave up on it");
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(trash.to_str().expect("UTF-8")), "the trash of a given-up record is named: {said}");
    assert!(failed.is_file() && trash.join("d0/file").is_file(), "the record and the trash stay");
    remove(root.as_path());
}

/// What:
///  A trash that says what it is and whose record the service gave up on is named as left where it is,
///  and no command claims a new record for it.
/// Why:
///   The scan re-records a trash whose manifest proves the wrapper made it,
/// but the queue's id is the trash's name,
/// so a
///  terminal failed record answers that the trash is known and writes nothing;
/// a line that then said it was
///  recorded now told the person the commit service would empty it,
/// which never happens again,
/// and the
///  command would repeat that promise for ever.
#[test]
fn a_trash_the_service_gave_up_on_is_not_promised_a_new_record() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-given-up-again", name.as_str(), 1);
    crate::worktree_trash_own_manifest::write(worktree.as_path(), &manifest).expect("the trash says what it is");
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work.as_path(), crate::work_item::PENDING).join(item.id());
    let failed: PathBuf = state_directory(work.as_path(), crate::work_item::FAILED).join(item.id());
    std::fs::rename(pending.as_path(), failed.as_path()).expect("the service gave up on it");
    for attempt in ["the first command", "the next command"] {
        let mut say: Vec<u8> = Vec::new();
        name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
        let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
        assert!(!said.contains("recorded now"), "{attempt} must not promise a record that was not written: {said}");
        assert!(said.contains(trash.to_str().expect("UTF-8")), "{attempt} names the trash that stays: {said}");
        assert!(!pending.exists(), "nothing wrote a pending record: {said}");
        assert!(failed.is_file() && trash.join("d0/file").is_file(), "the terminal record and the trash stay");
    }
    remove(root.as_path());
}

/// What:
///  A trash that says what it is and whose record nothing ever claimed is reported as already in the queue,
///  not as recorded now,
/// and the record is left where it is for the service.
/// Why:
///   The id of a trash's record is the trash's name,
/// so a record that waited without a service looking at it is this
///  trash's record;
/// claiming a new one would be a second write of what is there already,
/// and only
///  the write a command makes says the service was told of this trash now.
#[test]
fn a_trash_whose_record_already_waits_is_not_reported_as_recorded_now() {
    let name: String = "c".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-record-there", name.as_str(), 1);
    crate::worktree_trash_own_manifest::write(worktree.as_path(), &manifest).expect("the trash says what it is");
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    let work: PathBuf = work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("record");
    let pending: PathBuf = state_directory(work.as_path(), crate::work_item::PENDING).join(item.id());
    age_by(&pending, std::time::Duration::from_secs(600));
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(!said.contains("recorded now"), "a record the command did not write is not reported as written: {said}");
    assert!(said.contains("already in the queue"), "the record that waits is named as there: {said}");
    assert!(said.contains(trash.to_str().expect("UTF-8")), "the line names the trash: {said}");
    assert!(pending.is_file() && trash.join("d0/file").is_file(), "the record and the trash stay");
    remove(root.as_path());
}

/// What:
///  A named pipe among the trash records does not hold the command that reads them,
///  and the trash they account for is left alone.
/// Why:
///   This scan runs on every command in a worktree that has a trash,
/// so an entry that is not a plain file would make every
///  command wait for a writer that never comes.
#[test]
fn a_named_pipe_among_the_trash_records_does_not_hold_the_command() {
    let (root, worktree, git_dir, _manifest) = worktree_with_planted_trash("trash-pipe", "8".repeat(40).as_str(), 1);
    let failed: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), crate::work_item::FAILED);
    std::fs::create_dir_all(failed.as_path()).expect("the failed records' directory");
    let pipe: PathBuf = failed.join("pipe-not-a-record");
    let name = std::ffi::CString::new(pipe.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and outlives the call; mode 0600 is this test's own.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "the named pipe");
    let (sender, receiver) = std::sync::mpsc::channel();
    let (git_dir_for_scan, worktree_for_scan) = (git_dir.clone(), worktree.clone());
    let runner = std::thread::spawn(move || {
        let mut said: Vec<u8> = Vec::new();
        name_unaccounted_trash(git_dir_for_scan.as_path(), worktree_for_scan.as_path(), &mut said, &nothing_woken());
        let _sent = sender.send(said);
    });
    let said: Vec<u8> = match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(said) => said,
        Err(_still_waiting) => panic!("the trash retry waited on a named pipe instead of skipping it"),
    };
    runner.join().expect("the scan ends");
    assert!(pipe.exists(), "the named pipe itself is left alone");
    assert!(String::from_utf8_lossy(said.as_slice()).contains("holds directories a snapshot left out"), "the trash is still named: {}", String::from_utf8_lossy(said.as_slice()));
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
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
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
/// the emptying does not follow a link at the trash parent either,
/// and says so.
#[test]
fn the_emptying_does_not_follow_a_link_at_the_trash_parent() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, outside, manifest) = worktree_with_trash_moved_out("trash-link-parent", name.as_str());
    std::os::unix::fs::symlink(outside.join("cli-git-trash"), worktree.join(TRASH_PARENT)).expect("link");
    let left: TrashLeft = delete_trash(worktree.as_path(), git_dir.as_path(), &manifest).expect_err("a trash reached through a link is not emptied");
    assert!(left.reason.contains("cli-git-trash"), "the reason names the path: {}", left.reason);
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

/// Finding 5 of the trash hand-over review:
/// a worktree whose path is not UTF-8 is named by a record like any other,
/// and the emptying then works by the bytes
///  of the path.
/// Why:
///   The record and the wake hold a path as its bytes beside its text (`path_bytes_in_records.rs`),
/// so a byte that is
///  not text is no reason to leave a trash no record names:
/// such a trash is one the service never empties,
/// and every later
///  command only warns about it.
#[cfg(target_os = "linux")]
#[test]
fn a_non_utf8_worktree_is_named_by_a_record_and_emptied_by_its_path_bytes() {
    let name: String = "9".repeat(40);
    let root: PathBuf = fixture("trash-hand-over-bytes");
    let worktree: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"topic-\xff"));
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    let pending = PendingTrash { destination: worktree.clone(), common_dir: git_dir.clone(), manifest: manifest.clone(), taken_over: 0, quiet: false };
    let mut notices: Vec<u8> = Vec::new();
    let reason: String = hand_over_with(&git_context(), &pending, &mut notices, &no_service_reach()).expect_err("no service is in reach");
    assert!(reason.contains("no commit service can be started from this session") && reason.contains("when it next runs for this repository"), "the reason is the missing service, and the record still empties the trash later: {reason}");
    let record: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    let item: WorkItem = WorkItem::decode(std::fs::read_to_string(record.as_path()).expect("the record the hand-over wrote for a path that is not text").as_str()).expect("a work item");
    assert_eq!(item.repository, worktree, "the record names the worktree by the bytes of its path");
    assert_eq!(empty_trash(&item), Verdict::Done, "so the emptying works by the path's bytes");
    assert!(!worktree.join(TRASH_PARENT).exists(), "and the trash is emptied all the same");
    remove(root.as_path());
}

/// A reach with no service and a provisioner that sets nothing up,
/// for the controls that never want a service touched.
fn no_service_reach<'a>() -> Reach<'a> {
    return Reach { router: &|_git: &Path, _directory: &Path| return None, provisioner: &|_host: &crate::service_provision::ProvisionHost| return Ok(()), start_wait: std::time::Duration::from_millis(400) };
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
///  A trash that cannot be emptied is reported with its path and the reason it stays,
/// so the service's line and
///  every later command name the same directory.
/// Why:
///   The add no longer empties anything,
/// so the only report of a trash that stays is the one the emptying gives
///  when the service runs it;
/// the reason must name what is there and never promise the person a command that
///  removes more than the snapshot made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await deleteTrash({ repository, gitDir, manifest }); expect(left.reason).toContain(name);
/// ```
#[test]
fn a_trash_that_cannot_be_emptied_is_reported_with_its_path_and_reason() {
    let name: String = "7".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-unremovable", name.as_str(), 1);
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    // A parent nobody may write to: the contents can go, the trash directory itself cannot.
    std::fs::set_permissions(&parent, std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only parent");
    let restore: RestoreMode = RestoreMode { path: parent.clone() };
    assert!(matches!(&empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &manifest)), Verdict::Failed(reason) if reason.contains(&name)), "the handler reports the trash it could not empty");
    assert!(parent.join(&name).exists(), "the trash directory could not be removed from a read-only parent");
    drop(restore);
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash that was replaced under the recorded name is left where it is,
/// with the reason it gives.
/// Why:
///   The emptying refuses to delete it precisely because it is not the object the snapshot made;
/// what stands
///  there now is somebody else's,
/// and the reason names the path and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename(join(worktree, TRASH_PARENT, name), movedAway); put(join(worktree, TRASH_PARENT, name), 'theirs/file', 'x');
/// const left = await deleteTrash({ repository, gitDir, manifest }); expect(left.reason).toContain('not the directory the snapshot made');
/// ```
#[test]
fn a_trash_that_is_not_the_snapshots_is_left_alone_with_its_reason() {
    let name: String = "8".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-replaced-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(&trash, worktree.join(TRASH_PARENT).join("moved-away")).expect("move the real trash away");
    put(trash.as_path(), "theirs/file", b"somebody else's data\n");
    let left: TrashLeft = delete_trash(worktree.as_path(), git_dir.as_path(), &manifest).expect_err("a trash that is not the snapshot's is left alone");
    assert!(left.reason.contains(trash.display().to_string().as_str()), "the reason names the path that stays: {}", left.reason);
    assert!(left.reason.contains("not the directory the snapshot made"), "the reason says why it was left: {}", left.reason);
    assert!(trash.join("theirs/file").exists(), "what stands there is untouched");
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash that holds a file the snapshot did not put there keeps the whole trash directory,
/// and the reason names
///  the file.
/// Why:
///   The emptying empties the recorded entry and then leaves the trash directory because of that file;
/// the file is
///  kept deliberately,
/// so the reason says what stayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(join(worktree, TRASH_PARENT, name, 'unrecorded/file'), 'not ours');
/// const left = await deleteTrash({ repository, gitDir, manifest }); expect(left.reason).toContain('unrecorded');
/// ```
#[test]
fn a_trash_holding_what_was_not_moved_there_keeps_its_directory() {
    let name: String = "c".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-extra-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    put(trash.as_path(), "unrecorded/file", b"not ours\n");
    let left: TrashLeft = delete_trash(worktree.as_path(), git_dir.as_path(), &manifest).expect_err("a trash holding what the snapshot did not put there is left alone");
    assert!(left.reason.contains(trash.display().to_string().as_str()) && left.reason.contains("unrecorded"), "the reason names the path and what it kept: {}", left.reason);
    assert!(trash.join("unrecorded/file").exists(), "the file the emptying kept is untouched");
    remove(root.as_path());
}

/// Review round 5,
/// finding 3:
/// a trash entry that is no longer the object the snapshot moved there is deliberately left,
/// and the reason names
///  it.
/// Why:
///   The emptying compares each entry's identity and skips the ones that changed;
/// what stands under that name
///  now is somebody else's,
/// and the reason says which entry it could not empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename(join(trash, 'd0'), join(worktree, 'moved-away')); put(join(trash, 'd0/precious'), 'theirs');
/// const left = await deleteTrash({ repository, gitDir, manifest }); expect(left.reason).toContain('d0');
/// ```
#[test]
fn a_trash_entry_that_was_replaced_is_left_alone_with_its_reason() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_planted_trash("trash-entry-message", name.as_str(), 1);
    let trash: PathBuf = worktree.join(TRASH_PARENT).join(&name);
    std::fs::rename(trash.join(entry_name(0).as_str()), worktree.join("moved-away")).expect("move the recorded entry away");
    put(trash.as_path(), format!("{}/precious", entry_name(0)).as_str(), b"somebody else's directory\n");
    let left: TrashLeft = delete_trash(worktree.as_path(), git_dir.as_path(), &manifest).expect_err("an entry that changed is left where it is");
    assert!(left.reason.contains(entry_name(0).as_str()), "the reason names the entry it kept: {}", left.reason);
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

/// What:
///  A directory below the trash parent whose name is not one the wrapper gives a trash is the person's own state,
/// so the
///  snapshot carries it into the new worktree and never moves it into this add's trash.
/// Why:
///  `info/exclude` keeps the trash parent out of every commit,
/// and a person may keep files the repository ignores
///  there.
/// A snapshot that took every directory below the parent for an earlier add's trash moved those files into
///  the trash of this add,
/// the hand-over queued them,
/// and the service deleted them,
/// though no wrapper command had ever
///  recorded them.
/// An earlier add's trash is named with forty lowercase hexadecimal digits,
/// which is the one thing this
///  selection may go by,
/// and a name of another shape is state of the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(repo, `${TRASH_PARENT}/personal/important-notes.txt`, 'mine');
/// const { stage, trash } = await snapshotOf(repo);
/// expect(existsSync(join(stage, TRASH_PARENT, 'personal/important-notes.txt'))).toBe(true);
/// expect(findIn(join(stage, TRASH_PARENT, trash.name), 'important-notes.txt')).toEqual([]);
/// ```
#[test]
fn btrfs_a_personal_directory_below_the_trash_parent_is_carried_into_the_new_worktree() {
    let Some(root) = btrfs_fixture("trash-personal-below-parent") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    put(repo.as_path(), format!("{TRASH_PARENT}/personal/important-notes.txt").as_str(), b"the person's own notes\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    assert!(stage.join(TRASH_PARENT).join("personal/important-notes.txt").is_file(), "the person's directory is carried into the new worktree");
    for index in 0..manifest.entries.len() {
        assert!(!stage.join(TRASH_PARENT).join(&manifest.name).join(entry_name(index)).join("personal").exists(), "and it is no entry of this add's trash, so nothing will empty it");
    }
    assert!(repo.join(TRASH_PARENT).join("personal/important-notes.txt").is_file(), "the source keeps it too");
    remove(root.as_path());
}

/// What:
///  A directory below the trash parent that is not named like a trash is not named to the person as a trash nothing will
///  empty,
/// and nothing in it is deleted.
/// Why:
///   Every later command in the worktree names what it cannot account for,
/// and the line tells the person to
///  remove it themselves.
/// A line about a directory the wrapper never made sends a person to delete their own files,
/// so the
///  scan names only names the wrapper gives a trash,
/// which is the same shape the leave-out goes by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(worktree, `${TRASH_PARENT}/personal/important-notes.txt`, 'mine');
/// const said = nameUnaccountedTrash(gitDir, worktree); expect(said).toContain(`${TRASH_PARENT}/${name}`); expect(said).not.toContain('personal');
/// ```
#[test]
fn a_personal_directory_below_the_trash_parent_is_not_named_as_a_trash() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, _manifest) = worktree_with_planted_trash("trash-personal-named", name.as_str(), 1);
    put(worktree.join(TRASH_PARENT).as_path(), "personal/important-notes.txt", b"the person's own notes\n");
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(git_dir.as_path(), worktree.as_path(), &mut say, &nothing_woken());
    let said: String = String::from_utf8_lossy(say.as_slice()).into_owned();
    assert!(said.contains(worktree.join(TRASH_PARENT).join(&name).display().to_string().as_str()), "the trash the wrapper made is named: {said}");
    assert!(!said.contains(format!("{TRASH_PARENT}/personal").as_str()), "the person's own directory is not called a trash: {said}");
    assert!(worktree.join(TRASH_PARENT).join("personal/important-notes.txt").is_file(), "and nothing touches it");
    remove(root.as_path());
}

/// What:
///  A trash whose record a kill prevented is still recorded and emptied by the next pass,
/// because the trash carries its
///  own manifest.
/// Why:
///   The work item is written after the snapshot is published,
/// so a `SIGKILL` in between leaves a trash no record names,
///  and nothing would ever empty it.
/// The manifest inside the trash travels with it through the swap,
/// so the next command in that worktree
///  records the trash and the service empties it,
/// and the trash is never deleted on a guess:
/// only a manifest
///  whose identities are the directory's own proves the wrapper made it.
/// This control plants the state that kill leaves,
/// a real snapshot whose record was never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stage, trash } = await snapshotOf(repo); deletePendingRecord(common, trash.name);
/// nameUnaccountedTrash(common, stage); expect(pendingRecord(common, trash.name)).toBeDefined();
/// expect(await emptyTrash(itemOf(stage, common, trash))).toBe('done'); expect(existsSync(trashPath)).toBe(false);
/// ```
#[test]
fn btrfs_a_trash_whose_record_a_kill_prevented_is_recorded_and_emptied_by_the_next_pass() {
    let Some(root) = btrfs_fixture("trash-killed-before-record") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.join("worktrees/topic")).expect("admin");
    std::fs::write(stage.join(".git"), format!("gitdir: {}\n", common.join("worktrees/topic").display())).expect("pointer");
    let work: PathBuf = work_directory(common.as_path());
    assert!(std::fs::read_dir(state_directory(work.as_path(), "pending")).is_err(), "the kill left no record");
    let mut say: Vec<u8> = Vec::new();
    name_unaccounted_trash(common.as_path(), stage.as_path(), &mut say, &nothing_woken());
    let record: PathBuf = state_directory(work.as_path(), "pending").join(format!("trash-{}", manifest.name));
    let item: WorkItem = WorkItem::decode(std::fs::read_to_string(record.as_path()).expect("the next pass records a trash that says what it is").as_str()).expect("a work item");
    assert_eq!(item.parent, manifest.encode(), "the record carries the identities the trash itself wrote");
    assert_eq!(item.repository, stage, "and names the worktree that holds it");
    assert_eq!(empty_trash(&item), Verdict::Done, "so the handler empties it");
    assert!(!stage.join(TRASH_PARENT).exists(), "and nothing is left behind: {}", stage.display());
    remove(root.as_path());
}
