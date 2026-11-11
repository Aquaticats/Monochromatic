//! Controls for the snapshots the wrapper makes ahead of an add:
//! an add claims the newest one that proves equal to the source's index,
//! refuses and hands back the one that fell behind it,
//! waits no longer than its ceiling,
//! and never claims one for a source with changes;
//! with the service's own making of one,
//! and the two names it writes.

use super::*;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_snapshot_ahead_claim::{identity_of_name, look_for_a_snapshot_ahead};
use crate::fs_btrfs_ioctl::{snapshot_create_at, subvolume_create_at};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, git_context, remove};
use crate::work_item::{FAILED, PENDING, state_directory};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};
use crate::worktree_snapshot_source::choose_snapshot_source;
use std::cell::RefCell;
use std::time::Instant;

/// What:
///  The fixture of these controls:
/// a btrfs subvolume repository with tracked files and ignored state,
/// the plan for a
///  destination inside it,
/// and the snapshot source of that plan.
/// Why:
///   A claim needs a real subvolume,
/// an index proven equal to the files,
/// and a folder a snapshot can be made in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, plan, source, pool } = fixtureOf('ahead-claim');
/// ```
struct Fixture {
    /// The folder the whole control is made in;
    /// it is removed when the control ends.
    root: PathBuf,
    /// The repository whose root is the source of every snapshot here.
    repo: PathBuf,
    /// The destination the plan names.
    plan: AddPlan,
    /// The source as a clean check proved it.
    source: SnapshotSource,
    /// The folder the wrapper keeps the snapshots ahead of this repository in.
    pool: PathBuf,
}

/// Make the fixture;
/// nothing is made when this host has no btrfs fixture root.
fn fixture_of(name: &str) -> Option<Fixture> {
    let root: PathBuf = btrfs_fixture(name)?;
    let repo: PathBuf = subvolume_repository(root.as_path());
    let destination: PathBuf = repo.join("destination");
    let plan: AddPlan = plan_to(destination);
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a clean subvolume source");
    let pool: PathBuf = pool_folder(repo.join(".git").as_path());
    return Some(Fixture { root: root.clone(), repo: repo.clone(), plan, source, pool });
}

/// The stage name of one control,
/// which is a private name in the pool folder.
fn stage_of(fixture: &Fixture, name: &str) -> PathBuf {
    return fixture.repo.join(format!("{}{name}", crate::worktree_copy_selection::STAGE_PREFIX));
}

/// Make one snapshot ahead the way the service does,
/// in the pool folder.
fn make_one_ahead(fixture: &Fixture) -> OsString {
    let item: WorkItem = request_item(fixture.repo.join(".git").as_path(), fixture.repo.as_path(), fixture.pool.as_path()).expect("a request");
    let verdict = crate::worktree_snapshot_ahead_job::snapshot_ahead(&item);
    assert_eq!(verdict, crate::service_work::Verdict::Done, "the service makes one snapshot ahead");
    let made: Vec<OsString> = kept_in(fixture.pool.as_path(), fixture.repo.as_path());
    assert_eq!(made.len(), 1, "one snapshot ahead stands in the pool folder");
    return made[0].clone();
}

/// An add claims the snapshot the service made ahead,
/// with the proof that its files equal the source's index,
/// and the
///  pool folder no longer holds it.
#[test]
fn btrfs_an_add_claims_the_snapshot_the_wrapper_made_ahead() {
    let Some(fixture) = fixture_of("ahead-claim") else { return };
    let made: OsString = make_one_ahead(&fixture);
    let stage: PathBuf = stage_of(&fixture, "claim");
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    // The seam of a control: it records every name it is handed and makes nothing, so no control reaches a service, a
    // session bus or a socket.
    let service: AheadService<'_> = AheadService {
        ask: &|| { return Ok(None); },
        hand_over: &|stale: &[OsString]| { handed.borrow_mut().extend_from_slice(stale); return Ok(()); },
        ceiling: Duration::from_secs(1),
    };
    let look: AheadLook = look_for_a_snapshot_ahead(&fixture.source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    let AheadLook::Claimed { made: claimed, proof } = look else { panic!("the add claims the snapshot ahead, and it answered {look:?}") };
    assert_eq!(proof.entries.len(), fixture.source.clean.tracked, "the proof covers every tracked file");
    assert_eq!(identity_of_name(stage.as_path()), Some(claimed), "the claimed object is the one recorded");
    assert_eq!(std::fs::read(stage.join("shared.txt")).expect("the claimed copy's file"), b"second, longer\n");
    assert_eq!(std::fs::read(stage.join("ignored/pkg/index.js")).expect("the claimed copy's ignored state"), b"module.exports = 1;\n");
    assert!(kept_in(fixture.pool.as_path(), fixture.repo.as_path()).is_empty(), "the entry moved out of the pool folder");
    assert!(!fixture.pool.join(&made).exists(), "the name the service made is gone");
    assert!(handed.borrow().is_empty(), "a claimed snapshot leaves nothing to hand over");
    remove(fixture.root.as_path());
}

/// A copy no add claimed for longer than an add keeps one is handed to the service by the next add,
/// whether that add
///  claims a copy or not.
/// Why:
///   The sweep is what keeps a repository whose adds stopped from pinning the source's blocks for ever,
/// and an add
///  that claims one leaves the pool through the claim's own return,
/// so a sweep that runs only on the waiting
///  path never runs in a repository whose pool answers every add.
#[test]
fn btrfs_a_claim_hands_over_the_copies_an_add_keeps_no_longer() {
    let Some(fixture) = fixture_of("ahead-claim-sweep") else { return };
    make_one_ahead(&fixture);
    let aged: OsString = ahead_name(fixture.repo.as_path(), SystemTime::UNIX_EPOCH).expect("a name");
    snapshot_create_at(fixture.repo.as_path(), fixture.pool.join(&aged).as_path()).expect("a copy older than an add keeps");
    let stage: PathBuf = stage_of(&fixture, "sweep");
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    let service: AheadService<'_> = AheadService {
        ask: &|| { return Ok(None); },
        hand_over: &|stale: &[OsString]| { handed.borrow_mut().extend_from_slice(stale); return Ok(()); },
        ceiling: Duration::from_secs(1),
    };
    let look: AheadLook = look_for_a_snapshot_ahead(&fixture.source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    assert!(matches!(look, AheadLook::Claimed { .. }), "the newest copy proves equal to the source's index: {look:?}");
    assert_eq!(handed.borrow().as_slice(), std::slice::from_ref(&aged), "the copy no add keeps is handed to the service");
    remove(fixture.root.as_path());
}

/// An add whose own request the service gave up on stops waiting at once:
/// it takes its own snapshot instead of paying the ceiling for a copy nothing will make.
/// Why:
///  A request the service refused for a condition nothing of the add's changes ends in `failed`,
/// which is the
///  state nothing scans,
/// so a wait that could only end at its ceiling costs the add that ceiling for work that
///  cannot come.
#[test]
fn btrfs_the_add_stops_waiting_when_the_service_gave_its_request_up() {
    let Some(fixture) = fixture_of("ahead-given-up") else { return };
    let stage: PathBuf = stage_of(&fixture, "given-up");
    let git_dir: PathBuf = fixture.repo.join(".git");
    let work: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    let service: AheadService<'_> = AheadService {
        ask: &|| {
            // The service answers this request once and gives it up, as one refused for good does.
            let asked: WorkItem = crate::worktree_snapshot_ahead::ask_with(git_dir.as_path(), fixture.repo.as_path(), fixture.pool.as_path())?;
            let id: String = asked.id();
            std::fs::rename(
                crate::work_item::state_directory(work.as_path(), crate::work_item::PENDING).join(id.as_str()),
                crate::work_item::state_directory(work.as_path(), crate::work_item::FAILED).join(id.as_str()),
            )
            .map_err(|error| { return format!("the record could not be given up: {error}"); })?;
            return Ok(Some(id));
        },
        hand_over: &|stale: &[OsString]| { handed.borrow_mut().extend_from_slice(stale); return Ok(()); },
        ceiling: Duration::from_secs(20),
    };
    let began: Instant = Instant::now();
    let look: AheadLook = look_for_a_snapshot_ahead(&fixture.source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    let spent: Duration = began.elapsed();
    assert!(matches!(look, AheadLook::None), "the add takes its own snapshot: {look:?}");
    assert!(spent < Duration::from_secs(10), "the add pays no ceiling for a request the service gave up: {spent:?}");
    remove(fixture.root.as_path());
}

/// A snapshot made ahead of another source is never claimed,
/// and a snapshot that fell behind the source's index is refused and handed to the service.
#[test]
fn btrfs_a_snapshot_ahead_that_fell_behind_the_index_is_refused_and_handed_over() {
    let Some(fixture) = fixture_of("ahead-stale") else { return };
    let fell_behind: OsString = make_one_ahead(&fixture);
    // A commit after the snapshot moves the index on, so the copy no longer names what the source's index does.
    tracked(fixture.repo.as_path(), "later.txt", b"added after the snapshot\n");
    commit_all(fixture.repo.as_path(), "later");
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(fixture.repo.as_path()), &fixture.plan).expect("the source again");
    let stage: PathBuf = stage_of(&fixture, "stale");
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    let service: AheadService<'_> = AheadService {
        ask: &|| { return Ok(None); },
        hand_over: &|stale: &[OsString]| { handed.borrow_mut().extend_from_slice(stale); return Ok(()); },
        ceiling: Duration::from_millis(1),
    };
    let look: AheadLook = look_for_a_snapshot_ahead(&source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    assert!(matches!(look, AheadLook::TimedOut(_)), "nothing proved equal to the source's index, so the add waits out its ceiling and answers why: {look:?}");
    assert_eq!(handed.borrow().as_slice(), std::slice::from_ref(&fell_behind), "the entry that fell behind is handed to the service");
    assert!(!stage.exists(), "nothing was renamed into the stage name");
    remove(fixture.root.as_path());
}

/// The wait for a snapshot ahead ends at its ceiling and names that ceiling,
/// so the add never waits on a btrfs call with no end.
#[test]
fn btrfs_the_wait_for_a_snapshot_ahead_ends_at_its_ceiling_and_says_so() {
    let Some(fixture) = fixture_of("ahead-ceiling") else { return };
    let stage: PathBuf = stage_of(&fixture, "ceiling");
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    let service: AheadService<'_> = AheadService {
        ask: &|| { return Ok(None); },
        hand_over: &|_stale: &[OsString]| { return Ok(()); },
        ceiling: Duration::from_millis(150),
    };
    let began: Instant = Instant::now();
    let look: AheadLook = look_for_a_snapshot_ahead(&fixture.source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    let spent: Duration = began.elapsed();
    let AheadLook::TimedOut(reason) = look else { panic!("an empty pool and a service that makes nothing ends at the ceiling, and it answered {look:?}") };
    assert!(reason.contains("150 ms"), "the reason names the ceiling it reached: {reason}");
    assert!(spent >= Duration::from_millis(150), "the wait is the ceiling and not a look: {spent:?}");
    assert!(spent < Duration::from_secs(20), "the wait ends when the ceiling passes: {spent:?}");
    assert!(handed.borrow().is_empty(), "nothing stood in the pool to hand anything over");
    remove(fixture.root.as_path());
}

/// A source with uncommitted changes is never served from the pool:
/// its snapshot has to be taken on the add's own path,
///  and the ask for one is not made.
#[test]
fn btrfs_a_source_with_changes_is_not_served_a_snapshot_ahead() {
    let Some(fixture) = fixture_of("ahead-dirty") else { return };
    make_one_ahead(&fixture);
    std::fs::write(fixture.repo.join("shared.txt"), b"an uncommitted change\n").expect("a change");
    let source: SnapshotSource = crate::worktree_snapshot_source::source_or_dirty(&git_context(), &main_identity(fixture.repo.as_path())).expect("a source with changes");
    assert!(source.dirty.is_some(), "the source's change is what this control is about");
    let stage: PathBuf = stage_of(&fixture, "dirty");
    let handed: RefCell<Vec<OsString>> = RefCell::new(Vec::new());
    let service: AheadService<'_> = AheadService {
        ask: &|| { return Ok(None); },
        hand_over: &|stale: &[OsString]| { handed.borrow_mut().extend_from_slice(stale); return Ok(()); },
        ceiling: Duration::from_secs(30),
    };
    let look: AheadLook = look_for_a_snapshot_ahead(&source, &fixture.plan, stage.as_path(), &crate::fixture_content_decision::content_environment(), &service);
    assert!(matches!(look, AheadLook::None), "the add takes its own snapshot instead: {look:?}");
    assert!(handed.borrow().is_empty(), "a source with changes asks for nothing and hands nothing over");
    assert!(!stage.exists(), "no pool entry was renamed into the stage name");
    remove(fixture.root.as_path());
}

/// The service makes one snapshot ahead of a worktree and stops at what an add keeps.
#[test]
fn btrfs_the_service_makes_snapshots_ahead_up_to_what_an_add_keeps() {
    let Some(fixture) = fixture_of("ahead-service") else { return };
    for made in 0..AHEAD_KEPT {
        let item: WorkItem = request_item(fixture.repo.join(".git").as_path(), fixture.repo.as_path(), fixture.pool.as_path()).expect("a request");
        assert_eq!(crate::worktree_snapshot_ahead_job::snapshot_ahead(&item), crate::service_work::Verdict::Done, "snapshot {made} is made");
        assert_eq!(kept_in(fixture.pool.as_path(), fixture.repo.as_path()).len(), made + 1);
    }
    let beyond: WorkItem = request_item(fixture.repo.join(".git").as_path(), fixture.repo.as_path(), fixture.pool.as_path()).expect("a request");
    let verdict = crate::worktree_snapshot_ahead_job::snapshot_ahead(&beyond);
    assert!(matches!(verdict, crate::service_work::Verdict::Skipped(_)), "the pool is full, so nothing more is made: {verdict:?}");
    assert_eq!(kept_in(fixture.pool.as_path(), fixture.repo.as_path()).len(), AHEAD_KEPT);
    remove(fixture.root.as_path());
}

/// A record that names a source its own Git directory does not belong to makes no snapshot.
#[test]
fn btrfs_a_record_that_names_another_tree_makes_no_snapshot() {
    let Some(fixture) = fixture_of("ahead-foreign") else { return };
    let other: PathBuf = fixture.root.join("other");
    subvolume_create_at(other.as_path()).expect("another subvolume");
    let item: WorkItem = request_item(fixture.repo.join(".git").as_path(), other.as_path(), fixture.pool.as_path()).expect("a request");
    let verdict = crate::worktree_snapshot_ahead_job::snapshot_ahead(&item);
    assert!(matches!(verdict, crate::service_work::Verdict::Failed(_)), "the record names a tree its Git directory does not own: {verdict:?}");
    assert!(kept_in(fixture.pool.as_path(), other.as_path()).is_empty());
    remove(fixture.root.as_path());
}

/// The names of one source's snapshots ahead sort newest first,
/// and another source's names are none of them.
#[test]
fn a_snapshots_name_carries_its_source_and_the_millisecond_it_was_made_in() {
    let first: PathBuf = PathBuf::from("/a/repo");
    let older: OsString = ahead_name(first.as_path(), SystemTime::UNIX_EPOCH + Duration::from_millis(1_000)).expect("a name");
    let newer: OsString = ahead_name(first.as_path(), SystemTime::UNIX_EPOCH + Duration::from_millis(2_000)).expect("a name");
    assert!(older.to_string_lossy().starts_with(AHEAD_PREFIX), "{older:?}");
    assert_eq!(made_at(older.as_os_str()), Some(1_000));
    assert_eq!(made_at(newer.as_os_str()), Some(2_000));
    assert!(is_of_this_source(older.as_os_str(), ahead_key(first.as_path()).as_str()));
    assert!(!is_of_this_source(older.as_os_str(), "0000000000000000"), "another source's key is no candidate");
    let second: PathBuf = first.join("another");
    assert!(!is_of_this_source(older.as_os_str(), ahead_key(second.as_path()).as_str()));
    assert_eq!(made_at(OsStr::new(".cli-git-worktree-copy-1234")), None, "a stage name is no snapshot ahead");
    assert_eq!(made_at(OsStr::new(".cli-git-worktree-ahead-")), None);
}

/// The pool keeps what an add keeps and turns everything older than its age into names the service deletes.
#[test]
fn the_snapshots_ahead_kept_are_the_newest_few_and_the_young() {
    let root: PathBuf = PathBuf::from("/a/repo");
    let name_at = |milliseconds: u64| { return ahead_name(root.as_path(), SystemTime::UNIX_EPOCH + Duration::from_millis(milliseconds)).expect("a name"); };
    let now: u128 = AHEAD_OLDEST.as_millis() * 2;
    let young: Vec<OsString> = (0..AHEAD_KEPT as u64).map(|index| { return name_at(now as u64 - index); }).collect();
    assert!(over_kept(young.as_slice(), now).is_empty(), "what an add keeps and what is young stays");
    let old: OsString = name_at(1);
    let mut with_an_old_one: Vec<OsString> = young.clone();
    with_an_old_one.push(old.clone());
    assert_eq!(over_kept(with_an_old_one.as_slice(), now), vec![old], "an entry no add claimed for longer than the age goes");
    let mut too_many: Vec<OsString> = young.clone();
    for index in 0..2 {
        too_many.push(name_at(now as u64 - AHEAD_KEPT as u64 - index));
    }
    assert_eq!(over_kept(too_many.as_slice(), now).len(), 2, "everything after the newest few goes");
}

/// What the service is asked for names the source,
/// its own Git directory,
/// the folder and a kind of its own,
/// so no other
///  kind of work reads the record and no push is made for it.
#[test]
fn a_request_names_the_source_and_the_folder_and_no_branch() {
    let root: PathBuf = PathBuf::from("/a/repo");
    let item: WorkItem = request_item(root.join(".git").as_path(), root.as_path(), root.as_path()).expect("a request");
    assert_eq!(item.kind, WorkKind::SnapshotAhead);
    assert_eq!(item.kind.name(), "snapshot-ahead");
    assert_eq!(WorkKind::from_name("snapshot-ahead"), Some(WorkKind::SnapshotAhead));
    assert_eq!(item.repository, root);
    assert_eq!(item.git_dir, root.join(".git"));
    assert_eq!(item.branch_ref, AHEAD_REFERENCE);
    assert_eq!(item.id(), format!("snapshot-ahead-{}", item.oid));
    assert_eq!(item.oid.len(), 64, "the identity is hexadecimal and long enough for a record of a landed object");
    assert!(item.oid.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }));
    assert_eq!(item.parent, root.display().to_string());
}

/// A stale snapshot is deleted by the same kind a removed worktree's subvolume is,
/// by its own path.
#[test]
fn a_stale_snapshot_is_deleted_by_the_path_the_service_is_given() {
    let root: PathBuf = PathBuf::from("/a/repo");
    let name: OsString = ahead_name(root.as_path(), SystemTime::UNIX_EPOCH).expect("a name");
    let item: WorkItem = deletion_item(root.join(".git").as_path(), root.as_path(), name.as_os_str());
    assert_eq!(item.kind, WorkKind::SubvolumeDelete);
    assert_eq!(item.parent, root.join(&name).display().to_string());
    assert_eq!(item.branch_ref, crate::worktree_remove_hand_over::DELETE_REFERENCE);
    assert_eq!(item.oid.len(), 64);
    let again: WorkItem = deletion_item(root.join(".git").as_path(), root.as_path(), name.as_os_str());
    assert_eq!(item.id(), again.id(), "two adds that prove one entry stale write one record");
}

/// A delete the service gave up on is queued again by the next add that proves the copy stale.
/// Why:
///  The id of the delete of one path is that path,
/// so every later add names the same record;
/// a queue that only
///  answers "already there" would never delete that copy,
/// and its blocks would stay pinned for good.
#[test]
fn a_delete_the_service_gave_up_on_is_queued_again() {
    let root: PathBuf = fixture("ahead-delete-given-up");
    let repo: PathBuf = root.join("repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("a Git directory");
    let pool: PathBuf = pool_folder(git_dir.as_path());
    let work: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let name: OsString = ahead_name(repo.as_path(), SystemTime::UNIX_EPOCH).expect("a name");
    let id: String = deletion_item(git_dir.as_path(), pool.as_path(), name.as_os_str()).id();
    hand_over_with(git_dir.as_path(), pool.as_path(), std::slice::from_ref(&name)).expect("the delete is recorded");
    assert!(state_directory(work.as_path(), PENDING).join(&id).exists(), "the delete waits for a round");
    // The service gave up on it, and nothing scans `failed`.
    std::fs::rename(state_directory(work.as_path(), PENDING).join(&id), state_directory(work.as_path(), FAILED).join(&id)).expect("the record is given up");
    hand_over_with(git_dir.as_path(), pool.as_path(), std::slice::from_ref(&name)).expect("the delete is recorded again");
    assert!(state_directory(work.as_path(), PENDING).join(&id).exists(), "the delete is queued again instead of staying given up");
    remove(root.as_path());
}

/// The snapshots ahead of a repository stand inside its own Git directory,
/// where no name Git lists as untracked can be,
///  and beside the work queue the service already keeps there.
#[test]
fn the_pool_folder_is_inside_the_git_directory() {
    assert_eq!(pool_folder(Path::new("/a/repo/.git")), PathBuf::from("/a/repo/.git/monochromatic-git-policy-cli/ahead"));
    assert_ne!(pool_folder(Path::new("/a/repo/.git")), PathBuf::from("/a/worktrees"), "no person's worktrees folder holds one");
}
