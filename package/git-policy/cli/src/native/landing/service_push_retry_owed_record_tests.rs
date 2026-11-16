//! What:
//!  Controls for a push whose end the person's storage refused to record:
//! the plan that owes the record keeps it with its words,
//! a look writes the record and ends the plan without pushing again,
//! and a storage that keeps refusing it is given up on after `OWED_RECORD_LOOKS` looks.
//! Why:
//!  The plan is the only durable record that the push waits for the person,
//! and the item is in `failed`,
//! which nothing scans:
//! a plan dropped with the refused record is a backup silently given up on
//! (`never discard user data`) and a refusal nobody reads,
//! which is what the review of 2026-11-14 reproduced as its finding 1.
//! The words travel in the plan's own file,
//! so each look below starts from a plan read back from disk,
//! as a service that restarted does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await round(gitDir, retries); expect(load(plans)).toHaveLength(1); mend(); await round(gitDir, retries); expect(shown).toContain(words);
//! ```

#![cfg(unix)]

use super::*;
use crate::parallel_budget::HostResources;
use crate::push_failure_notice::{notice_directory, push_failures};
use crate::push_retry::OWED_RECORD_LOOKS;
use crate::push_retry::PLAN_DIRECTORY;
use crate::push_retry::PushRetries;
use crate::push_retry_store::{OwedRecord, OwedRecordKind, StoredPlan, load, save};
use crate::service_work_run_round::run_planned_round;
use crate::test_support::{fixture, remove, shown_notices};
use crate::work_item::WorkItem;
use crate::work_item::WorkKind;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Handlers whose pushes answer from a script,
/// one verdict per push,
/// and how many pushes ran.
struct Scripted {
    /// The verdicts still to give,
    /// the next last.
    verdicts: Mutex<Vec<Verdict>>,
    /// How many pushes ran.
    pushes: AtomicUsize,
}

impl WorkHandlers for Scripted {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        self.pushes.fetch_add(1, Ordering::SeqCst);
        return self.verdicts.lock().expect("the script").pop().unwrap_or(Verdict::Done);
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// One round over the repository,
/// the way the service's loop runs it:
/// what is due comes out of `retries` and goes back in.
fn a_round(git_dir: &Path, handlers: &Arc<Scripted>, retries: &mut PushRetries) -> Option<Duration> {
    let asked: Vec<(PathBuf, Duration)> = run_planned_round(&HostResources::read(), &[git_dir.to_path_buf()], handlers, retries, &ClaimTime::read());
    return asked.into_iter().find(|(directory, _)| { return directory == git_dir; }).map(|(_, wait)| { return wait; });
}

/// What:
///  A repository whose plan is due at once,
///  and handlers that answer its pushes from a script.
/// Why:
///   Nothing is enqueued,
/// so the only push a round can run is the plan's,
///  and every look below starts from `PushRetries::restored`,
/// which reads the plan back from its file as a service that restarted does.
/// The first pause is zero,
/// so each look is due the moment the round takes it,
/// and a control never waits on a clock.
fn world(name: &str, verdicts: Vec<Verdict>) -> (PathBuf, PathBuf, PathBuf, Arc<Scripted>) {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("worktree");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let plans: PathBuf = root.join("journal").join(PLAN_DIRECTORY);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository,
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-14T00:00:00.000Z"),
        tries: 1,
    };
    save(plans.as_path(), &StoredPlan { git_dir: git_dir.clone(), item, attempt: 1, owes: None }).expect("the plan of the try that failed");
    let handlers: Arc<Scripted> = Arc::new(Scripted { verdicts: Mutex::new(verdicts), pushes: AtomicUsize::new(0) });
    return (root, git_dir, plans, handlers);
}

/// What:
///  A file where the notices folder must be,
///  so every write of a record for the person is refused by the storage.
/// Why:
///   A full or read-only disk refuses the write the same way,
/// and this fault is one a control can mend between two rounds.
fn block_the_notices(git_dir: &Path) -> PathBuf {
    let directory: PathBuf = notice_directory(git_dir);
    std::fs::create_dir_all(directory.parent().expect("the notices folder has a parent")).expect("the folder the notices go in");
    std::fs::write(directory.as_path(), b"not a directory\n").expect("the blocker");
    return directory;
}

/// What:
///  The record the plan of the branch owes the person,
///  read back from the plan's file.
/// Why:
///   A control that finds nothing would otherwise only learn that the plan is there,
/// and the words and the looks are what the next look needs.
fn owed_by_its_plan(plans: &Path, owed_kind: OwedRecordKind) -> OwedRecord {
    let kept: Vec<StoredPlan> = load(plans);
    assert_eq!(kept.len(), 1, "the plan that owes the person the record is gone although no notice was written: {kept:?}");
    let owed: OwedRecord = kept[0].owes.clone().expect("the plan that owes the person a record names none");
    assert_eq!(owed.kind, owed_kind, "the plan owes the wrong kind of record");
    return owed;
}

/// What:
///  A refusal no retry mends,
/// whose record the storage refused,
/// keeps its plan and its words,
///  and the plan's next look writes the record and ends the plan without pushing again.
/// Why:
///  The defect this control holds shut:
/// the arm wrote the record,
/// logged the refusal,
/// and dropped the plan anyway,
/// so the plan file,
/// the notice and every later automatic try were gone together,
///  and the commit waited for a push by hand nobody was told to make.
#[test]
fn a_refusal_no_retry_mends_keeps_the_plan_and_its_words_until_a_look_writes_them() {
    const REFUSAL: &str = "error: the server refused this push: a hook on the server declined it";
    let (root, git_dir, plans, handlers) = world("push-owed-refusal", vec![Verdict::Failed(String::from(REFUSAL))]);
    let notices: PathBuf = block_the_notices(git_dir.as_path());
    let mut retries: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _ran: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut retries);
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the plan's try did not run");
    let owed: OwedRecord = owed_by_its_plan(plans.as_path(), OwedRecordKind::Refused);
    assert!(owed.words.contains(REFUSAL), "the plan does not hold the words the person must read: {:?}", owed.words);
    assert_eq!(owed.looks_left, OWED_RECORD_LOOKS, "the plan does not owe the looks a record gets: {owed:?}");
    assert!(shown_notices(push_failures(git_dir.as_path())).is_empty(), "a notice exists although the storage refused one");
    // The fault is mended, and the plan read back from its file writes the record without asking the server again.
    std::fs::remove_file(notices.as_path()).expect("the fault is mended");
    let mut after: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _again: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut after);
    let shown: String = shown_notices(push_failures(git_dir.as_path())).join("\n");
    assert!(shown.contains(REFUSAL), "the refusal the plan kept reached nobody on its next look: {shown:?}");
    assert!(load(plans.as_path()).is_empty(), "the plan is still there although its record was written");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the look that wrote the record pushed the commit again");
    remove(root.as_path());
}

/// What:
///  A skip whose record the storage refused keeps its plan and its words,
///  and the plan's next look writes the skip and ends the plan without pushing again.
/// Why:
///  The skipped arm has the same shape as the refusal's:
/// its words were logged and the plan dropped,
/// and a skip is the service's own reason for a backup that does not exist,
/// so nothing else would ever say it.
#[test]
fn a_skip_the_storage_refused_keeps_the_plan_and_its_words_until_a_look_writes_them() {
    const SKIP: &str = "monochromatic-git-policy-cli: the branch main no longer exists in this repository, so this push was not made";
    let (root, git_dir, plans, handlers) = world("push-owed-skip", vec![Verdict::Skipped(String::from(SKIP))]);
    let notices: PathBuf = block_the_notices(git_dir.as_path());
    let mut retries: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _ran: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut retries);
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the plan's try did not run");
    let owed: OwedRecord = owed_by_its_plan(plans.as_path(), OwedRecordKind::Skipped);
    assert!(owed.words.contains(SKIP), "the plan does not hold the words the person must read: {:?}", owed.words);
    assert!(shown_notices(push_failures(git_dir.as_path())).is_empty(), "a notice exists although the storage refused one");
    std::fs::remove_file(notices.as_path()).expect("the fault is mended");
    let mut after: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _again: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut after);
    let shown: String = shown_notices(push_failures(git_dir.as_path())).join("\n");
    assert!(shown.contains(SKIP), "the skip the plan kept reached nobody on its next look: {shown:?}");
    assert!(load(plans.as_path()).is_empty(), "the plan is still there although its skip was written");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the look that wrote the skip pushed the commit again");
    remove(root.as_path());
}

/// What:
///  A refusal a writable storage takes ends the plan in the same round,
///  as it did before the plan could keep a record.
/// Why:
///  Nothing may get slower or wait longer where the storage works:
/// the record goes first,
/// the plan ends with it,
/// and no look is kept for a record that is already written.
#[test]
fn a_refusal_a_writable_storage_takes_ends_the_plan_in_the_same_look() {
    const REFUSAL: &str = "error: the remote wants a login the service does not have";
    let (root, git_dir, plans, handlers) = world("push-owed-refusal-written", vec![Verdict::Failed(String::from(REFUSAL))]);
    let mut retries: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _ran: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut retries);
    let shown: String = shown_notices(push_failures(git_dir.as_path())).join("\n");
    assert!(shown.contains(REFUSAL), "the refusal reached nobody although the storage took it: {shown:?}");
    assert!(load(plans.as_path()).is_empty(), "the plan outlived the record it owed");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "the round pushed the commit more than once");
    remove(root.as_path());
}

/// What:
///  A storage that keeps refusing the record is asked for it `OWED_RECORD_LOOKS` looks and no more,
///  and the plan ends without the push being tried again.
/// Why:
///   Nothing may wait or repeat without an end:
/// a record the storage never takes is given up on,
/// its words stay in the journal's own line of that push's end,
///  and the branch is not pushed again for a failure no retry mends.
/// The control counts the looks rather than trusting the constant:
/// a plan still standing after the bound,
/// or one given up before it,
/// fails here.
#[test]
fn a_storage_that_keeps_refusing_the_record_is_asked_for_it_the_owed_looks_and_no_more() {
    const REFUSAL: &str = "error: a hook on the server declined the push the plan tried again";
    let (root, git_dir, plans, handlers) = world("push-owed-refusal-refused-forever", vec![Verdict::Failed(String::from(REFUSAL))]);
    let _blocked: PathBuf = block_the_notices(git_dir.as_path());
    let mut retries: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _ran: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut retries);
    // The failure kept the plan with every look it owes; each look after it is refused and leaves one fewer.
    for look in 1..OWED_RECORD_LOOKS {
        let mut again: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
        let _look: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut again);
        let owed: OwedRecord = owed_by_its_plan(plans.as_path(), OwedRecordKind::Refused);
        assert_eq!(
            owed.looks_left,
            OWED_RECORD_LOOKS - look,
            "look {look} of {OWED_RECORD_LOOKS} did not leave the looks it should have: {owed:?}"
        );
    }
    // The last look it owes is refused too, and the plan ends there.
    let mut last: PushRetries = PushRetries::restored(Duration::ZERO, plans.as_path());
    let _last_look: Option<Duration> = a_round(git_dir.as_path(), &handlers, &mut last);
    assert!(load(plans.as_path()).is_empty(), "the plan outlived every look it owed the record");
    assert_eq!(handlers.pushes.load(Ordering::SeqCst), 1, "a look pushed the commit again for a failure no retry mends");
    remove(root.as_path());
}
