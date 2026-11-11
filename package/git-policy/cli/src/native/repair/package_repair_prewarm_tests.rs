//! Controls of the prewarm record:
//! what it carries,
//! what it refuses,
//! and what a hand-over leaves behind when no service can be reached.

use super::{PREWARM_REFERENCE, PrewarmEntry, Reach, common_directory_awaiting, decode, encode, hand_over_with, item_of, object_name_of};
use crate::service_route::Route;
use crate::service_work::work_directory;
use crate::test_support::{fixture, git, git_context, remove};
use crate::work_item::{DONE, FAILED, PENDING, WorkItem, WorkKind, state_directory};
use std::path::{Path, PathBuf};

/// One workspace whose build directory is to be filled,
/// with two members so the payload holds more than one name.
fn entry(root: &str, from: &str, to: &str) -> PrewarmEntry {
    return PrewarmEntry {
        root: PathBuf::from(root),
        from: PathBuf::from(from),
        to: PathBuf::from(to),
        members: vec![String::from("monochromatic-lint"), String::from("monochromatic-lint-closure")],
    };
}

/// A route finder that names no service,
/// as a session with no unit for this repository does.
fn no_service(_git: &Path, _git_dir: &Path) -> Option<Route> {
    return None;
}

/// A provisioner that has nothing to set up.
fn nothing_to_set_up(_host: &crate::service_provision::ProvisionHost) -> Result<(), crate::service_messages::ClientFailure> {
    return Ok(());
}

/// The reach of a control:
/// a route finder that finds nothing and a provisioner that sets nothing up,
/// so no service and no session bus is reached.
fn no_reach() -> Reach<'static> {
    return Reach { router: &no_service, provisioner: &nothing_to_set_up, start_wait: std::time::Duration::from_millis(200) };
}

/// What:
///  A payload with two workspaces round-trips,
/// and the entries come back in the order the add made them.
/// Why:
///   The record is the only thing the service reads,
/// so what it carries must survive being written and read back.
#[test]
fn two_entries_round_trip_through_the_payload() {
    let entries: Vec<PrewarmEntry> = vec![
        entry("/w/package/linter/rust", "/var/mnt/encrypted/cargo-build/ae/b0b48f8399eb6a", "/var/mnt/encrypted/cargo-build/6f/0123456789abcd"),
        entry("/w/package/git-policy/cli", "/var/mnt/encrypted/cargo-build/2b/89abcdef012345", "/var/mnt/encrypted/cargo-build/7c/ba9876543210fe"),
    ];
    let text: String = encode(entries.as_slice()).expect("the payload");
    assert_eq!(decode(text.as_str()).expect("the entries"), entries);
}

/// What:
///  An entry with no member is refused.
/// Why:
///   A clean with nothing to clean would publish the source's own artifacts under the worktree's
/// name,
/// which is exactly what the repair exists to prevent.
#[test]
fn a_payload_that_names_no_member_is_refused() {
    let text: &str = "[{\"root\":\"/w\",\"from\":\"/a\",\"to\":\"/b\",\"members\":[]}]";
    let refused: Result<Vec<PrewarmEntry>, String> = decode(text);
    assert!(refused.is_err(), "{refused:?}");
}

/// What:
///  A payload of no workspace,
/// or of text that is not the array,
/// is refused.
/// Why:
///   A record a person or another release wrote must be reported rather than acted on.
#[test]
fn a_payload_of_nothing_is_refused() {
    for text in ["[]", "not json", "{}", "[{\"root\":\"/w\",\"from\":\"/a\",\"to\":\"/b\",\"members\":[\"x\"]", "{\"root\":\"/w\"}"] {
        let refused: Result<Vec<PrewarmEntry>, String> = decode(text);
        assert!(refused.is_err(), "{text} was accepted as {refused:?}");
    }
}

/// What:
///  Two worktrees of the same commit,
/// and two hand-overs of one worktree at different moments,
/// are told apart by their object names.
/// Why:
///   A record's id is its idempotency key,
/// so a shared object name would drop a second worktree's filling as already known,
/// and a worktree removed and added again at the same path must not be answered by the record the
///  earlier one settled.
/// A work item's object name must be forty hexadecimal digits to be acted on.
#[test]
fn two_worktrees_get_their_own_object_name() {
    let moment: &str = "2026-11-10T18:00:00.000Z";
    let later: &str = "2026-11-10T18:05:00.000Z";
    let one: &Path = Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/one");
    let two: &Path = Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/two");
    let first: String = object_name_of(one, moment);
    assert_eq!(first.len(), 40, "{first}");
    assert!(first.as_bytes().iter().all(u8::is_ascii_hexdigit), "{first}");
    assert_ne!(first, object_name_of(two, moment), "two worktrees of one commit are told apart by their paths");
    assert_ne!(first, object_name_of(one, later), "two hand-overs of one path are told apart by when they were asked for");
}

/// What:
///  The record names the worktree it fills,
/// the common directory whose service drains it,
/// and the payload the add read.
/// Why:
///   `is_acceptable` refuses a record whose Git directory is not the one it was read from,
/// and the service finds the work to do from these fields alone.
#[test]
fn the_record_names_the_worktree_the_common_directory_and_the_payload() {
    let root: PathBuf = fixture("prewarm-item");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let entries: Vec<PrewarmEntry> = vec![entry("/w/package/linter/rust", "/cache/old", "/cache/new")];
    let item: WorkItem = item_of(common.as_path(), destination.as_path(), entries.as_slice()).expect("the record");
    assert_eq!(item.kind, WorkKind::Prewarm);
    assert_eq!(item.repository, destination);
    assert_eq!(item.git_dir, common);
    assert_eq!(item.branch_ref, PREWARM_REFERENCE);
    assert_eq!(item.oid, object_name_of(destination.as_path(), item.created.as_str()), "the object name is the worktree's path and the moment the record was written");
    assert_eq!(decode(item.parent.as_str()).expect("the entries"), entries);
    remove(root.as_path());
}

/// What:
///  A hand-over that finds no service still leaves the record under the repository's common
/// directory,
/// and says so.
/// Why:
///   The record is the whole difference between a filling that happens later and one nobody knows
/// about,
/// and the add must never fail for it:
/// the worktree is correct without the filling.
#[test]
fn a_hand_over_with_no_service_leaves_the_record_and_says_why() {
    let root: PathBuf = fixture("prewarm-no-service");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let entries: Vec<PrewarmEntry> = vec![entry("/w/package/linter/rust", "/cache/old", "/cache/new")];
    let told: Result<(), String> = hand_over_with(&git_context(), common.as_path(), destination.as_path(), entries.as_slice(), &no_reach());
    let reason: String = told.expect_err("a session with no service is reported");
    assert!(reason.contains("no commit service can be started from this session"), "{reason}");
    let queued: Vec<String> = records_in(work_directory(common.as_path()).as_path(), PENDING);
    assert_eq!(queued.len(), 1, "one record is queued: {queued:?}");
    assert!(queued[0].starts_with("prewarm-"), "the record is a prewarm's: {queued:?}");
    let record: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING).join(&queued[0]);
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record the hand-over wrote");
    let item: WorkItem = WorkItem::decode(text.as_str()).expect("the record is a work item");
    assert_eq!(item.kind, WorkKind::Prewarm);
    assert_eq!(decode(item.parent.as_str()).expect("the entries"), entries);
    remove(root.as_path());
}

/// What:
///  A hand-over of no workspace writes nothing and says nothing.
/// Why:
///   A worktree whose build directories need no filling must leave no record for the service to pick
/// up.
#[test]
fn a_hand_over_of_nothing_writes_no_record() {
    let root: PathBuf = fixture("prewarm-nothing");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    hand_over_with(&git_context(), common.as_path(), destination.as_path(), &[], &no_reach()).expect("nothing to hand over never fails");
    assert!(!work_directory(common.as_path()).exists(), "no work directory is made for nothing");
    remove(root.as_path());
}

/// What:
///  The common directory of a repository whose directory name is not UTF-8 is the directory Git named.
/// Why:
///   The answer was read as text,
///  so the path the record was keyed by held U+FFFD where the name held a byte,
/// and the record's own acceptability check refused every record of such a repository.
#[test]
fn the_common_directory_of_a_repository_whose_name_is_not_text_is_gits() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("prewarm-common-not-utf8");
    let repo: PathBuf = root.join(OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let context = git_context();
    let named: PathBuf = crate::lazy_runtime::run_on_runtime(async {
        return common_directory_awaiting(&context, repo.as_path()).await;
    })
    .expect("the runtime")
    .expect("Git names the common directory");
    assert_eq!(named, std::fs::canonicalize(repo.join(".git")).expect("the common directory"));
    remove(root.as_path());
}

/// What:
///  The names of the records a state directory holds,
/// in the order they sort,
/// so a control reads what the queue took without knowing an id.
/// Why:
///   A record's name is its id,
/// which the hand-over derives rather than states,
/// so a control asks the directory what is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordsIn = (work, state) => readdirSync(join(work, state)).sort();
/// ```
fn records_in(work: &Path, state: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(state_directory(work, state))
        .map(|listing| return listing.flatten().filter_map(|entry| return entry.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    names.sort();
    return names;
}

/// What:
///  A worktree removed from a path and added again there gets a filling of its own,
/// whether the record of the earlier worktree finished or failed.
/// Why:
///   A record's id is its idempotency key,
/// and the queue answers an id it already knows as known in every state.
/// With the worktree's path as the whole id,
/// the second worktree's filling would be dropped as already known,
/// and a filling that failed could never be asked for again:
/// the service settles a failure into `failed` and reads no record back.
#[test]
fn a_worktree_added_again_at_the_same_path_gets_its_own_filling() {
    for settled in [DONE, FAILED] {
        let root: PathBuf = fixture(format!("prewarm-added-again-{settled}").as_str());
        let common: PathBuf = root.join("common");
        let destination: PathBuf = root.join("topic");
        let entries: Vec<PrewarmEntry> = vec![entry("/w/package/linter/rust", "/cache/old", "/cache/new")];
        let work: PathBuf = work_directory(common.as_path());
        std::fs::create_dir_all(common.as_path()).expect("the common directory");
        std::fs::create_dir_all(destination.as_path()).expect("the first worktree");
        let _told: Result<(), String> = hand_over_with(&git_context(), common.as_path(), destination.as_path(), entries.as_slice(), &no_reach());
        let first: Vec<String> = records_in(work.as_path(), PENDING);
        assert_eq!(first.len(), 1, "the first worktree's record is queued: {first:?}");
        // The service runs the record and settles it into the state its verdict names.
        let settled_at: PathBuf = state_directory(work.as_path(), settled).join(&first[0]);
        std::fs::rename(state_directory(work.as_path(), PENDING).join(&first[0]), settled_at.as_path()).expect("the record is settled");
        // The worktree is removed, and another is added at the same path.
        remove(destination.as_path());
        std::fs::create_dir_all(destination.as_path()).expect("a worktree added again at the same path");
        let _told_again: Result<(), String> = hand_over_with(&git_context(), common.as_path(), destination.as_path(), entries.as_slice(), &no_reach());
        let second: Vec<String> = records_in(work.as_path(), PENDING);
        assert_eq!(second.len(), 1, "the worktree added again is queued for a filling of its own, though its predecessor settled into {settled}: {second:?}");
        assert_ne!(second[0], first[0], "the second record is not the first one ({settled})");
        remove(root.as_path());
    }
}
