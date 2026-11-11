//! Controls of the settlement of an add whose own snapshot was swapped into place before its command was
//! killed,
//! and of the copy such an add never owes.

use super::*;
use crate::test_support::{dead_copy_owner, fixture, git, git_context, remove, repository};
use crate::worktree_add_binding::{OwedCopy, OwedSelection, bind_registration};
use crate::worktree_copy_recovery::{RecoveryDestinations, recover_pending_worktree_copies};
use crate::worktree_copy_stage_claim::claim_stage_for_add_as_owing;
use crate::worktree_copy_stage_made_object::{made_record_beside_claim, read_made, record_made};

/// One add whose exchange completed,
/// as a command killed between the exchange and its settlement leaves it.
struct Exchanged {
    /// The fixture root,
    /// removed when the control ends.
    root: PathBuf,
    /// The repository the add was made in.
    repo: PathBuf,
    /// The repository's common directory.
    common: PathBuf,
    /// The worktree the add made,
    /// now holding the snapshot.
    destination: PathBuf,
    /// The claim the add wrote before real Git ran.
    claim: PathBuf,
    /// The stage name,
    /// now holding the directory Git made.
    container: PathBuf,
    /// A path no command of these controls runs in,
    /// standing for another worktree.
    observer: PathBuf,
}

/// What:
///  One add killed right after its own snapshot was swapped into the destination.
/// Why:
///   That is the state the exchange leaves.
/// The claim,
/// the record of the destination Git made and the record of the snapshot are all written before
///  the swap,
/// with the two objects at each other's names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('name'); // claim + .add + .made + exchange
/// ```
fn exchanged_add(name: &str) -> Exchanged {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked content\n").expect("a tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common directory");
    let canonical_root: PathBuf = std::fs::canonicalize(root.as_path()).expect("the fixture root");
    let planned: PathBuf = canonical_root.join("topic");
    let owed: OwedCopy = OwedCopy { source: repo.clone(), selection: OwedSelection::MainCheckout };
    let claim = claim_stage_for_add_as_owing(common.as_path(), planned.as_path(), &dead_copy_owner(), Some(&owed)).expect("claim");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("the destination Git made");
    // The wrapper's own snapshot carries the source's index, so every tracked file is in it before the swap;
    // Git's own checkout is the step the snapshot replaced.
    git(destination.as_path(), &["read-tree", "HEAD"]);
    let bound = bind_registration(common.as_path(), destination.as_path()).expect("the binding Git's registration proves");
    claim.record_binding(&bound.binding).expect("recorded");
    let container: PathBuf = claim.container().to_path_buf();
    std::fs::create_dir(container.as_path()).expect("the snapshot at the stage name");
    std::fs::write(container.join("tracked.txt"), b"tracked content\n").expect("the tracked file the snapshot carried");
    std::fs::write(container.join("ignored.txt"), b"ignored content\n").expect("the source's ignored state");
    std::fs::write(container.join(".git"), bound.binding.pointer.as_slice()).expect("the new worktree's pointer");
    let snapshot: crate::fs_beneath::ObjectId = path_identity(container.as_path()).expect("the snapshot's identity");
    record_made(common.as_path(), container.as_path(), snapshot).expect("the record of the stage's creation");
    let parent = open_root(container.parent().expect("the stage's parent")).expect("the parent");
    crate::fs_beneath::exchange_at(&parent, container.file_name().expect("the stage's name"), destination.file_name().expect("the destination's name")).expect("the exchange");
    let claim_path: PathBuf = claim_file_beside(common.as_path());
    return Exchanged { root, repo, common, destination, claim: claim_path, container, observer: canonical_root.join("observer") };
}

/// What:
///  The one claim an add wrote under a journal root.
/// Why:
///   The claim's own field is private to its module,
/// and the file's name is what every recovery pass reads,
/// so a
///  control names it the way those passes do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimPath = readdirSync(journalRoot(common)).filter((name) => name.endsWith('.stage'))[0];
/// ```
fn claim_file_beside(common: &Path) -> PathBuf {
    let root: PathBuf = crate::worktree_copy_journal::journal_root(common);
    let mut found: Vec<PathBuf> = std::fs::read_dir(root.as_path())
        .expect("the journal root")
        .flatten()
        .map(|entry| { return entry.path() })
        .filter(|path| { return path.extension().is_some_and(|extension| { return extension == "stage" }) })
        .collect();
    found.sort();
    return found.first().expect("a claim").clone();
}

/// What:
///  Every record the add wrote under the journal root.
/// Why:
///   A settled add keeps none of them,
/// and a claim that stays keeps all of its own,
/// so the listing is the whole
///  answer the next command reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = readdirSync(join(common, 'monochromatic-git-policy-cli-worktree-copy/v1')).sort();
/// ```
fn records(scene: &Exchanged) -> Vec<String> {
    let root: PathBuf = crate::worktree_copy_journal::journal_root(scene.common.as_path());
    let Ok(listing) = std::fs::read_dir(root.as_path()) else { return Vec::new() };
    let mut names: Vec<String> = listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned() }).collect();
    names.sort();
    return names;
}

/// The recovery pass a command in `running_in` makes.
fn recovery_words(scene: &Exchanged, running_in: &Path) -> String {
    let mut said: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), RecoveryDestinations::OnlyThisWorktree(running_in), &git_context(), &mut said).expect("the recovery pass");
    return String::from_utf8(said).expect("the words are text");
}

/// What:
///  The exchange an add's own snapshot made is settled by a command in another worktree,
/// which owes that
///  worktree no copy.
/// Why:
///   The snapshot carried the source's whole ignored state,
/// so the worktree holds everything and no command may
///  say a copy is still owed;
/// the stage name keeps only Git's empty directory,
/// which the settlement removes
///  and nothing else does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-elsewhere'); const said = recoveryWords(scene, scene.observer);
/// expect(said).not.toMatch(/still owed|never finished/); expect(said).toMatch(/swapped into place/);
/// expect(existsSync(scene.container)).toBe(false); expect(records(scene)).toEqual([]);
/// ```
#[test]
fn an_exchange_another_worktree_meets_is_settled_without_a_copy() {
    let scene: Exchanged = exchanged_add("swap-settled-elsewhere");
    let said: String = recovery_words(&scene, scene.observer.as_path());
    assert!(!said.contains("still owed"), "no copy is owed by an add whose snapshot carried the state: {said}");
    assert!(!said.contains("never finished"), "and none may be reported as unfinished: {said}");
    assert!(said.contains("was interrupted after its snapshot was swapped into place"), "the settlement names what the kill left: {said}");
    assert!(!scene.container.exists(), "the directory Git made is removed from the stage name");
    assert!(records(&scene).is_empty(), "the add's records are settled: {:?}", records(&scene));
    assert_eq!(std::fs::read(scene.destination.join("ignored.txt")).ok(), Some(b"ignored content\n".to_vec()), "the snapshot's ignored state stays");
    assert!(scene.destination.join(".git").exists(), "the worktree keeps its pointer");
    assert!(scene.destination.join("tracked.txt").exists(), "and the files the snapshot carried");
    remove(scene.root.as_path());
}

/// What:
///  The same exchange is settled by the command that runs in the worktree it filled.
/// Why:
///   That command is the one that would make a copy,
/// and it must not stage the source again for state the snapshot already
///  carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-in-destination'); const said = recoveryWords(scene, scene.destination);
/// expect(said).not.toMatch(/still owed|did not make it/); expect(records(scene)).toEqual([]);
/// ```
#[test]
fn an_exchange_the_worktrees_own_command_meets_is_settled_without_a_copy() {
    let scene: Exchanged = exchanged_add("swap-settled-in-destination");
    let said: String = recovery_words(&scene, scene.destination.as_path());
    assert!(!said.contains("still owed"), "the worktree holds the state: {said}");
    assert!(!said.contains("did not make it"), "and no reason for a copy that was never owed: {said}");
    assert!(said.contains("was interrupted after its snapshot was swapped into place"), "the settlement names the exchange: {said}");
    assert!(records(&scene).is_empty(), "the add's records are settled: {:?}", records(&scene));
    assert!(!scene.container.exists(), "the stage name is clear");
    remove(scene.root.as_path());
}

/// What:
///  A command killed after it removed the directory Git made and before it settled the claim leaves the
///  claim and its records,
/// and the next command settles them without a copy.
/// Why:
///   The snapshot stands at the destination,
/// so the exchange is proven without the folder that proved it,
///  and
///  the worktree is whole;
/// nothing may keep the claim alive for a copy the snapshot carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-settled-claim-only'); rmSync(scene.container, {recursive:true});
/// const said = recoveryWords(scene, scene.observer); expect(records(scene)).toEqual([]);
/// ```
#[test]
fn an_exchange_whose_stage_was_removed_before_the_claim_settled_still_settles() {
    let scene: Exchanged = exchanged_add("swap-settled-claim-only");
    std::fs::remove_dir_all(scene.container.as_path()).expect("a command removed the directory Git made");
    let said: String = recovery_words(&scene, scene.observer.as_path());
    assert!(said.contains("was interrupted after its snapshot was swapped into place"), "the settlement names the exchange: {said}");
    assert!(!said.contains("still owed"), "nothing is owed: {said}");
    assert!(records(&scene).is_empty(), "the claim the killed command left is settled: {:?}", records(&scene));
    remove(scene.root.as_path());
}

/// What:
///  A worktree the person removed after the interrupted add is not filled,
/// and the directory Git made does
///  not stay at the stage name forever.
/// Why:
///   The remedy every notice names is removing the worktree and adding it again;
/// a stage left by the exchange must not
///  outlive that remedy,
/// and nothing in the directory but the pointer Git wrote is removed from
///  it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-after-removal'); git(scene.repo, ['worktree','remove','--force', scene.destination]);
/// const said = recoveryWords(scene, scene.observer); expect(existsSync(scene.container)).toBe(false); expect(records(scene)).toEqual([]);
/// ```
#[test]
fn an_exchange_whose_destination_was_removed_settles_the_stage_name() {
    let scene: Exchanged = exchanged_add("swap-after-removal");
    git(scene.repo.as_path(), &["worktree", "remove", "--force", scene.destination.to_str().expect("a text path")]);
    let said: String = recovery_words(&scene, scene.observer.as_path());
    assert!(said.contains("no longer the one the add made"), "the settlement says the worktree moved on: {said}");
    assert!(!said.contains("still owed"), "nothing is owed to a worktree that is gone: {said}");
    assert!(!scene.container.exists(), "the directory Git made is removed: {said}");
    assert!(records(&scene).is_empty(), "the add's records are settled: {:?}", records(&scene));
    remove(scene.root.as_path());
}

/// What:
///  An entry another process added to the directory Git made before the kill moves into the worktree,
/// as
///  the creating command settles it,
/// rather than being removed with the directory.
/// Why:
///   The destination is the worktree the entry belongs to,
/// and the settle never replaces an entry that is already
///  there,
/// so nothing a person put in the folder is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-stray'); writeFileSync(join(scene.container,'mine.txt'),'mine');
/// recoveryWords(scene, scene.observer); expect(readFileSync(join(scene.destination,'mine.txt'))).toBe('mine');
/// ```
#[test]
fn an_entry_added_to_the_directory_git_made_moves_into_the_worktree() {
    let scene: Exchanged = exchanged_add("swap-stray-entry");
    std::fs::write(scene.container.join("mine.txt"), b"mine\n").expect("an entry another process added");
    let said: String = recovery_words(&scene, scene.observer.as_path());
    assert!(!said.contains("still owed"), "no copy is owed: {said}");
    assert_eq!(std::fs::read(scene.destination.join("mine.txt")).ok(), Some(b"mine\n".to_vec()), "the entry moves into the worktree: {said}");
    assert!(!scene.container.exists(), "and the directory Git made goes: {said}");
    assert!(records(&scene).is_empty(), "the add's records are settled: {:?}", records(&scene));
    remove(scene.root.as_path());
}

/// What:
///  A folder at the stage name that still holds something besides the pointer Git wrote,
/// once the worktree it
///  belonged to is gone,
/// is named and kept with its claim and records.
/// Why:
///   Only the pointer bytes are proven to be the add's there,
/// so anything else may be the person's and stays;
///  the folder is named with how to remove it rather than removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scene = exchangedAdd('swap-kept'); git(scene.repo, ['worktree','remove','--force', scene.destination]);
/// writeFileSync(join(scene.container,'mine.txt'),'mine'); const said = recoveryWords(scene, scene.observer);
/// expect(said).toMatch(/was not changed/); expect(readFileSync(join(scene.container,'mine.txt'))).toBe('mine');
/// ```
#[test]
fn a_folder_holding_more_than_the_pointer_is_kept_and_named() {
    let scene: Exchanged = exchanged_add("swap-kept-by-a-persons-file");
    git(scene.repo.as_path(), &["worktree", "remove", "--force", scene.destination.to_str().expect("a text path")]);
    std::fs::write(scene.container.join("mine.txt"), b"mine\n").expect("a person's file in the folder");
    let said: String = recovery_words(&scene, scene.observer.as_path());
    assert!(said.contains("was not changed"), "the folder is named as kept: {said}");
    assert!(said.contains("mine.txt"), "the entry that keeps it is named: {said}");
    assert!(!said.contains("still owed"), "no copy is owed: {said}");
    assert_eq!(std::fs::read(scene.container.join("mine.txt")).ok(), Some(b"mine\n".to_vec()), "the person's file stays");
    assert!(!records(&scene).is_empty(), "the claim stays with the folder it names: {:?}", records(&scene));
    remove(scene.root.as_path());
}

/// The record of the stage's creation is what proves the snapshot,
/// so a control reads it back to be sure the scene wrote it.
#[test]
fn the_scene_records_the_object_it_made() {
    let scene: Exchanged = exchanged_add("swap-scene-recorded");
    let made = read_made(scene.claim.as_path()).expect("the record of the stage's creation");
    assert_eq!(made.container, scene.container, "the record names the stage container");
    assert!(made_record_beside_claim(scene.claim.as_path()).exists(), "the record stands beside the claim");
    assert!(made.object.is_some(), "the scene's record names the object it made");
    assert_eq!(path_identity(scene.destination.as_path()), made.object, "the snapshot is at the destination after the exchange");
    remove(scene.root.as_path());
}
