//! What:
//!  Controls of the records an interrupted copy leaves that cannot be read:
//!  each is moved aside,
//!  never deleted,
//!  never over another record,
//!  and the writer is judged before anything moves.
//! Why:
//!  The harm the pass prevents is one record that stops every command of the repository,
//!  so the controls pin what becomes of the record and of what it named,
//!  and that a record whose writer is gone is the only one acted on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const notices = await settleUnreadableRecords({ commonDir, verdicts }); expect(moved).toBe(present);
//! ```

use crate::worktree_copy_journal::{journal_root, ensure_journal_root};
use crate::worktree_copy_journal_encoding_and_validation::{Journal, PendingJournal, Phase};
use crate::worktree_copy_owner::{CopyOwner, copy_owner_of_process};
use crate::worktree_copy_stage_claim::{claim_stage_for_add_as, claim_names};
use crate::worktree_copy_stage_claim_reaping::StageVerdicts;
use crate::worktree_copy_unreadable_records::{UNREADABLE_FOLDER, move_aside, moved_record_path, settle_unreadable_records, unreadable_record_names, unreadable_records_stand};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A directory under the test's own scratch space,
/// private to this account like every item of a copy.
fn scratch(name: &str) -> PathBuf {
    let base: PathBuf = crate::test_support::fixture_with_prefix("unreadable", name);
    std::fs::remove_dir_all(base.join("common").as_path()).ok();
    std::fs::create_dir_all(base.join("common").as_path()).expect("common directory");
    std::fs::create_dir_all(base.join("worktrees").as_path()).expect("worktree parents");
    std::fs::set_permissions(base.join("common").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private common directory");
    std::fs::set_permissions(base.join("worktrees").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private worktree parent");
    return std::fs::canonicalize(base).expect("canonical scratch");
}

/// The owner of a process that really started,
/// had its birth identity read while it ran,
/// and was then killed and reaped:
/// what the records of an interrupted copy carry.
fn dead_owner() -> CopyOwner {
    let mut child: std::process::Child = std::process::Command::new("sleep").arg("60").spawn().expect("a child that runs");
    let pid: i64 = i64::from(child.id());
    let birth: String = crate::process_identity::process_birth_identity(pid).expect("identity readable").expect("the child runs");
    child.kill().expect("the child is killed");
    child.wait().expect("the child is reaped");
    return CopyOwner { pid, birth_identity: birth, pid_namespace: crate::pid_namespace::current_pid_namespace() };
}

/// One journal record of a copy into `destination`,
/// written by `owner`.
fn write_journal(common: &Path, destination: &Path, owner: &CopyOwner, body: &[u8]) {
    let root: PathBuf = ensure_journal_root(common).expect("journal root");
    let container: PathBuf = destination.parent().expect("a destination parent").join(format!(".cli-git-worktree-copy-{}", owner.token()));
    std::fs::create_dir_all(container.join("payload").as_path()).expect("stage container");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    let journal: Journal = Journal {
        created_entries: Vec::new(),
        intended_entries: Vec::new(),
        destination_root: destination.to_path_buf(),
        phase: Phase::Staged,
        selected_roots: Vec::new(),
        source_root: destination.parent().expect("a destination parent").join("source"),
        stage_container: container.clone(),
        stage_root: container.join("payload"),
    };
    let _pending: PendingJournal = PendingJournal { path: root.join(format!("{}.{}.json", "8f14e45f", owner.token())), journal };
    std::fs::write(_pending.path.as_path(), body).expect("the journal's bytes");
    std::fs::set_permissions(_pending.path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private journal");
}

/// Every journal file standing under the root,
/// so a control can say what became of one.
fn journals(common: &Path) -> Vec<String> {
    let root: PathBuf = journal_root(common);
    let mut names: Vec<String> = Vec::new();
    for listed in std::fs::read_dir(root.as_path()).expect("journal root") {
        let name: String = listed.expect("an entry").file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    names.sort();
    return names;
}

/// An empty journal of a writer that ended is moved aside rather than deleted,
/// and the folder beside the records keeps it under its own name.
#[test]
fn an_empty_journal_of_a_dead_writer_is_moved_aside_and_kept() {
    let base: PathBuf = scratch("journal-empty");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    write_journal(common.as_path(), destination.as_path(), &dead_owner(), b"");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    assert_eq!(journals(common.as_path()), Vec::<String>::new(), "no unreadable journal stands where every command reads");
    assert!(unreadable_records_stand(common.as_path()), "the record is kept");
    let kept: Vec<String> = unreadable_record_names(common.as_path()).expect("names");
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert!(kept[0].ends_with(".json"), "{kept:?}");
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(notices[0].contains("could not be read"), "{}", notices[0]);
    assert!(notices[0].contains(UNREADABLE_FOLDER), "the line says where it went: {}", notices[0]);
    assert!(
        notices[0].contains("stage folder"),
        "the line says what may be left on disk: {}",
        notices[0]
    );
    assert_eq!(std::fs::read(moved_record_path(common.as_path(), OsStr::new(kept[0].as_str()))).expect("the kept record"), b"");
}

/// A record whose writer still runs is that writer's:
/// it is left whole,
/// and the command that found it changes nothing.
#[test]
fn a_record_of_a_live_writer_is_left_alone() {
    let base: PathBuf = scratch("journal-live");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    write_journal(common.as_path(), destination.as_path(), &copy_owner_of_process(std::process::id()).expect("this process's owner"), b"");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    assert_eq!(notices, Vec::<String>::new(), "a running copy's record is not settled");
    assert_eq!(journals(common.as_path()).len(), 1, "the record stands where its writer left it");
    assert!(!unreadable_records_stand(common.as_path()), "nothing was moved");
}

/// A moved record never replaces one an earlier command kept:
/// the second one gets a name of its own and both keep their bytes.
#[test]
fn a_second_record_of_one_name_never_replaces_the_first() {
    let base: PathBuf = scratch("two-records");
    let common: PathBuf = base.join("common");
    let first: PathBuf = base.join("first.json");
    let second: PathBuf = base.join("second.json");
    std::fs::write(first.as_path(), b"first\n").expect("first");
    std::fs::write(second.as_path(), b"second\n").expect("second");

    let moved_first: PathBuf = move_aside(common.as_path(), first.as_path(), OsStr::new("one.json")).expect("moved").expect("named");
    let moved_second: PathBuf = move_aside(common.as_path(), second.as_path(), OsStr::new("one.json")).expect("moved").expect("named");

    assert_ne!(moved_first, moved_second, "the second record is kept under a name of its own");
    assert_eq!(std::fs::read(moved_first).expect("the first record"), b"first\n");
    assert_eq!(std::fs::read(moved_second).expect("the second record"), b"second\n");
    assert!(!first.exists() && !second.exists(), "no record is left where the readers look");
}

/// A record another command settled first is nobody's failure.
#[test]
fn a_record_that_vanished_is_not_an_error() {
    let base: PathBuf = scratch("vanished");
    let common: PathBuf = base.join("common");

    let moved: Option<PathBuf> = move_aside(common.as_path(), base.join("gone.json").as_path(), OsStr::new("gone.json")).expect("settled");

    assert_eq!(moved, None, "nothing was there to move");
}

/// An unreadable claim whose stage folder the record beside it names is written again,
/// so the sweep that makes the copy an add still owes has a claim to read.
#[test]
fn a_claim_whose_folder_is_named_beside_it_is_written_again() {
    let base: PathBuf = scratch("claim-rebuilt");
    let common: PathBuf = base.join("common");
    let destination: PathBuf = base.join("worktrees/two");
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let claim: crate::worktree_copy_stage_claim::StageClaim =
        claim_stage_for_add_as(common.as_path(), destination.as_path(), &dead_owner()).expect("an add's claim");
    let claim_path: PathBuf = journal_root(common.as_path()).join(format!("{}.stage", claim.identifier()));
    std::fs::write(claim_path.as_path(), b"").expect("the claim loses what it said");
    std::fs::set_permissions(claim_path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("private claim");
    let mut verdicts: StageVerdicts = StageVerdicts::default();

    let notices: Vec<String> = settle_unreadable_records(common.as_path(), &mut verdicts).expect("settled");

    let names: Vec<String> = claim_names(journal_root(common.as_path()).as_path()).expect("claims");
    assert_eq!(names.len(), 1, "a claim stands again under its own name: {names:?}");
    let recovered: Option<(PathBuf, Option<String>)> = crate::worktree_copy_stage_claim_reaping::read_claim(claim_path.as_path()).expect("read");
    let (container, owner) = recovered.expect("the claim reads again");
    assert_eq!(container, claim.container().to_path_buf(), "it names the folder the add claimed");
    assert_eq!(owner, None, "a claim written again names no owner, so nothing waits for one");
    assert!(verdicts.names_dead_claim(names[0].as_str()), "this pass settles it as a dead owner's claim");
    assert!(notices[0].contains("written again"), "{}", notices[0]);
}
