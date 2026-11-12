//! Controls for the awaited walk over a replay's landed history:
//! it decides every shared path the way the synchronous walk decides it,
//! and it
//!  decides them inside the runtime,
//! where the synchronous walk cannot start Git at all.

use super::tests::{History, captured_of, history, landed, path_of, shared};
use super::*;
use crate::commit_transaction::TransactionStop;
use crate::replay_capture_order::CaptureOrderDecision;
use crate::replay_shared_paths::SharedPath;
use crate::service_replay::{Captured, MergeSite};
use crate::test_support::{git_context, git_text, remove};
use std::path::PathBuf;

/// What:
///  The decisions one history's walk gives through both faces.
/// Why:
///  The two faces decide the paths of one replay,
/// so a difference here is a conflict whose bytes land on one
///  face and stay on the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothWalks(site, captured, tip, paths);
/// ```
fn both_walks(
    site: &MergeSite<'_>,
    captured: &Captured,
    tip: &str,
    paths: &[SharedPath],
) -> (Result<BTreeMap<Vec<u8>, CaptureOrderDecision>, TransactionStop>, Result<BTreeMap<Vec<u8>, CaptureOrderDecision>, TransactionStop>) {
    let synchronous: Result<BTreeMap<Vec<u8>, CaptureOrderDecision>, TransactionStop> = capture_order_of_each(site, captured, tip, paths);
    let awaited: Result<BTreeMap<Vec<u8>, CaptureOrderDecision>, TransactionStop> =
        crate::lazy_runtime::run_on_runtime(capture_order_of_each_awaiting(site, captured, tip, paths)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  Both faces decide every shared path the same way,
///  and the awaited face decides them inside the runtime,
/// where the
///  synchronous face cannot start its Git at all.
/// Why:
///  The four answers are what the replay acts on:
/// `Prepared` lands the captured entry,
/// `Landed` keeps the
///  branch's,
/// and `Unordered` hands the path to subsumption and the merge;
/// a face that decided any of them
///  differently would land one conflict's bytes and keep another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...bothWalks(site, captured, tip, paths)); assert.equal(awaited.get(early), 'prepared');
/// ```
#[test]
fn the_awaited_walk_decides_what_the_synchronous_one_decides() {
    let history: History = history("capture-order-awaiting-agrees", 3);
    let scratch: PathBuf = history.root.join("scratch");
    std::fs::create_dir_all(scratch.as_path()).expect("scratch");
    let context: crate::transaction_git::GitContext = git_context();
    let common: PathBuf = history.repo.join(".git");
    let objects: PathBuf = common.join("objects");
    let site: MergeSite<'_> = MergeSite { context: &context, cwd: history.repo.as_path(), common_dir: common.as_path(), object_directory: objects.as_path(), scratch: scratch.as_path() };
    let tip: String = git_text(history.repo.as_path(), &["rev-parse", "HEAD"]);
    let early: &str = history.commits[0].as_str();
    let late: &str = history.commits[1].as_str();
    let captured: Captured = captured_of(&history, 100, vec![landed(early, 10), landed(late, 200)]);
    let paths: Vec<SharedPath> = vec![shared(path_of(0).as_slice()), shared(path_of(1).as_slice()), shared(path_of(2).as_slice()), shared(b"never-changed.txt")];
    let (synchronous, awaited) = both_walks(&site, &captured, tip.as_str(), paths.as_slice());
    let decided: &BTreeMap<Vec<u8>, CaptureOrderDecision> = synchronous.as_ref().expect("the synchronous walk decides");
    assert_eq!(decided.get(path_of(0).as_slice()), Some(&CaptureOrderDecision::Prepared), "a commit captured before this one keeps the captured bytes");
    assert_eq!(decided.get(path_of(1).as_slice()), Some(&CaptureOrderDecision::Landed), "a commit captured after this one keeps the landed bytes");
    assert_eq!(
        awaited.as_ref().expect("the awaited walk decides"),
        decided,
        "the two faces decide every shared path the same way"
    );
    remove(history.root.as_path());
}

/// What:
///  A capture with no stamp of its own is decided by neither face through Git,
///  and both answer the same set.
/// Why:
///  The order decides nothing there,
/// so a face that asked anyway would spend a walk of the whole history on no
///  decision;
/// the answer the caller reads is the same undecided set on both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...bothWalks(site, capturedWithStamp(0), tip, paths));
/// ```
#[test]
fn a_capture_with_no_stamp_is_decided_the_same_way_by_both_faces() {
    let history: History = history("capture-order-awaiting-unsampled", 2);
    let scratch: PathBuf = history.root.join("scratch");
    std::fs::create_dir_all(scratch.as_path()).expect("scratch");
    let context: crate::transaction_git::GitContext = git_context();
    let common: PathBuf = history.repo.join(".git");
    let objects: PathBuf = common.join("objects");
    let site: MergeSite<'_> = MergeSite { context: &context, cwd: history.repo.as_path(), common_dir: common.as_path(), object_directory: objects.as_path(), scratch: scratch.as_path() };
    let tip: String = git_text(history.repo.as_path(), &["rev-parse", "HEAD"]);
    let unsampled: Captured = captured_of(&history, 0, Vec::new());
    let paths: Vec<SharedPath> = vec![shared(path_of(0).as_slice())];
    let (synchronous, awaited) = both_walks(&site, &unsampled, tip.as_str(), paths.as_slice());
    let decided: &BTreeMap<Vec<u8>, CaptureOrderDecision> = synchronous.as_ref().expect("the synchronous walk answers");
    assert_eq!(
        awaited.as_ref().expect("the awaited walk answers"),
        decided,
        "a capture with no stamp of its own is decided the same way by both faces"
    );
    assert_eq!(
        decided.get(path_of(0).as_slice()),
        Some(&CaptureOrderDecision::Unordered),
        "a capture with no stamp of its own is not decided by order"
    );
    remove(history.root.as_path());
}
