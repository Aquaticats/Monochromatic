//! Controls of the relink record:
//! what it carries,
//! what it refuses,
//! and what a hand-over leaves behind when no service can be reached.

use super::{RELINK_REFERENCE, RelinkEntry, Reach, RelinkTool, decode, encode, hand_over_with, item_of, object_name_of};
use crate::service_route::Route;
use crate::service_work::work_directory;
use crate::test_support::{fixture, git_context, remove};
use crate::work_item::{PENDING, WorkItem, WorkKind, state_directory};
use std::path::{Path, PathBuf};

/// One worktree that needs both tools,
/// so the payload holds more than one name.
fn entry(root: &Path) -> RelinkEntry {
    return RelinkEntry { root: root.to_path_buf(), tools: vec![RelinkTool::Pnpm, RelinkTool::Toolchain] };
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
///  A worktree that needs both tools round-trips through the payload,
/// with the order the report has always named them in.
/// Why:
///   The record is the only thing the service reads,
/// so what it carries must survive being written and read back.
#[test]
fn both_tools_round_trip_through_the_payload() {
    let written: RelinkEntry = entry(Path::new("/w/topic"));
    let text: String = encode(&written).expect("the payload");
    assert_eq!(decode(text.as_str()).expect("the entry"), written);
}

/// What:
///  A payload of no tool,
/// of a tool this release does not know,
/// or of text that is not the object,
/// is refused.
/// Why:
///   A record a person or another release wrote must be reported rather than acted on,
/// and a record of no tool would make the service start nothing.
#[test]
fn a_payload_that_names_no_known_tool_is_refused() {
    for text in [
        "{\"root\":\"/w\",\"tools\":[]}",
        "{\"root\":\"/w\",\"tools\":[\"npm\"]}",
        "{\"root\":\"/w\",\"tools\":[7]}",
        "{\"tools\":[\"pnpm\"]}",
        "{\"root\":\"/w\"}",
        "[]",
        "{}",
        "not json",
    ] {
        let refused: Result<RelinkEntry, String> = decode(text);
        assert!(refused.is_err(), "{text} was accepted as {refused:?}");
    }
    assert!(encode(&RelinkEntry { root: PathBuf::from("/w"), tools: Vec::new() }).is_err(), "a relink of no tool is no work");
}

/// What:
///  Two worktrees of the same commit are told apart by their object names.
/// Why:
///   A record's id is its idempotency key,
/// so a shared object name would drop the second worktree's relink as already known,
/// and a work item's object name must be forty hexadecimal digits to be acted on.
#[test]
fn two_worktrees_get_their_own_object_name() {
    let first: String = object_name_of(Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/one"));
    let second: String = object_name_of(Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/two"));
    assert_eq!(first.len(), 40, "{first}");
    assert!(first.as_bytes().iter().all(u8::is_ascii_hexdigit), "{first}");
    assert_ne!(first, second);
}

/// What:
///  The record names the worktree it relinks,
/// the common directory whose service drains it,
/// and the tools the add decided on.
/// Why:
///   `is_acceptable` refuses a record whose Git directory is not the one it was read from,
/// and the service finds the work to do from these fields alone.
#[test]
fn the_record_names_the_worktree_the_common_directory_and_the_tools() {
    let root: PathBuf = fixture("relink-item");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let written: RelinkEntry = entry(destination.as_path());
    let item: WorkItem = item_of(common.as_path(), destination.as_path(), &written).expect("the record");
    assert_eq!(item.kind, WorkKind::Relink);
    assert_eq!(item.repository, destination);
    assert_eq!(item.git_dir, common);
    assert_eq!(item.branch_ref, RELINK_REFERENCE);
    assert_eq!(item.oid, object_name_of(destination.as_path()));
    assert_eq!(decode(item.parent.as_str()).expect("the entry"), written);
    remove(root.as_path());
}

/// What:
///  A hand-over that finds no service still leaves the record under the repository's common
/// directory,
/// and says so.
/// Why:
///   The record is the whole difference between a relink that happens in the service and one nobody knows
/// about,
/// and the add must never fail for it:
/// a worktree whose relink no service took is relinked by the add itself.
#[test]
fn a_hand_over_with_no_service_leaves_the_record_and_says_why() {
    let root: PathBuf = fixture("relink-no-service");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let written: RelinkEntry = entry(destination.as_path());
    let told: Result<(), String> = hand_over_with(&git_context(), common.as_path(), destination.as_path(), &written, &no_reach());
    let reason: String = told.expect_err("a session with no service is reported");
    assert!(reason.contains("no commit service can be started from this session"), "{reason}");
    let record: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING).join(format!("relink-{}", object_name_of(destination.as_path())));
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record the hand-over wrote");
    let item: WorkItem = WorkItem::decode(text.as_str()).expect("the record is a work item");
    assert_eq!(item.kind, WorkKind::Relink);
    assert_eq!(decode(item.parent.as_str()).expect("the entry"), written);
    remove(root.as_path());
}
