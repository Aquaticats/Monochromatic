//! Controls for the plan on disk:
//! a plan is read back whole,
//! a newer plan of a branch replaces the older,
//! a removed plan is gone,
//! and a
//! file that holds no plan is removed instead of being said at every start.

use super::*;
use crate::test_support::{fixture, remove as remove_fixture};
use crate::work_item::WorkKind;

/// The oid of the first commit of the controls.
const FIRST_OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// The oid of the second commit of the controls.
const SECOND_OID: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn plan(branch: &str, oid: &str, attempt: u32) -> StoredPlan {
    return StoredPlan {
        git_dir: PathBuf::from("/srv/repo/.git"),
        item: WorkItem {
            kind: WorkKind::Push,
            repository: PathBuf::from("/srv/repo"),
            git_dir: PathBuf::from("/srv/repo/.git"),
            branch_ref: branch.to_owned(),
            oid: oid.to_owned(),
            parent: String::new(),
            created: String::from("2026-11-03T05:40:00.000Z"),
            tries: 0,
        },
        attempt,
    };
}

/// A plan comes back as it went in,
/// one per branch.
#[test]
fn a_saved_plan_is_read_back_whole() {
    let root: PathBuf = fixture("retry-store-roundtrip");
    let main: StoredPlan = plan("refs/heads/main", FIRST_OID, 3);
    let topic: StoredPlan = plan("refs/heads/topic", SECOND_OID, 1);
    save(root.as_path(), &main).expect("saved");
    save(root.as_path(), &topic).expect("saved");
    let mut loaded: Vec<StoredPlan> = load(root.as_path());
    loaded.sort_by(|left, right| { return left.item.branch_ref.cmp(&right.item.branch_ref); });
    assert_eq!(loaded, vec![main, topic]);
    remove_fixture(root.as_path());
}

/// What:
///  A plan whose Git directory is not text is read back whole.
/// Why:
///   The plan names the directory its push belongs to,
/// and a restart finds the push by that name,
/// so a name that lost the path's bytes would leave the push untried and its file unremoved.
#[test]
fn a_plan_of_a_directory_that_is_not_text_is_read_back_whole() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("retry-store-non-utf8");
    let git_dir: PathBuf = root.join(OsStr::from_bytes(b"c\xffmmon"));
    let worktree: PathBuf = root.join(OsStr::from_bytes(b"w\xffrktree"));
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let saved: StoredPlan = StoredPlan {
        git_dir: git_dir.clone(),
        item: WorkItem { repository: worktree, git_dir: git_dir.clone(), ..plan("refs/heads/main", FIRST_OID, 2).item },
        attempt: 2,
    };
    save(root.as_path(), &saved).expect("saved");
    assert_eq!(load(root.as_path()), vec![saved.clone()], "the plan comes back as it went in");
    remove_fixture(root.as_path());
}

/// What:
///  A service that starts again finds the plan of a directory that is not text,
/// and tries it when it is due.
/// Why:
///   This is the plan's whole purpose:
/// a push that waits across a restart is pushed again.
/// The list is keyed by the directory the drain names,
/// which is the real one.
#[test]
fn a_restored_list_keeps_the_plan_of_a_directory_that_is_not_text() {
    use crate::push_retry::{Due, PushRetries};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::time::{Duration, Instant};
    let root: PathBuf = fixture("retry-store-non-utf8-restored");
    let git_dir: PathBuf = root.join(OsStr::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let saved: StoredPlan = StoredPlan { git_dir: git_dir.clone(), ..plan("refs/heads/main", FIRST_OID, 1) };
    save(root.as_path(), &saved).expect("saved");
    let mut retries: PushRetries = PushRetries::restored(Duration::from_secs(1), root.as_path());
    assert_eq!(retries.directories(), vec![git_dir.clone()], "the plan is held under the directory it was saved for");
    let later: Instant = Instant::now() + Duration::from_secs(600);
    let due: Vec<Due> = retries.take_due(git_dir.as_path(), later);
    assert_eq!(due.len(), 1, "the plan of this directory is due");
    assert_eq!(due[0].git_dir, git_dir);
    remove_fixture(root.as_path());
}

/// What:
///  Two Git directories that differ only in bytes that are not text keep their own plans.
/// Why:
///   A plan file is found by the key of its directory,
/// so a key that loses those bytes gives two directories one key,
/// and saving the second plan deletes the first,
/// leaving one push untried after a restart.
#[test]
fn plans_of_directories_that_differ_only_in_bytes_are_kept_apart() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("retry-store-distinct-bytes");
    let higher: StoredPlan = StoredPlan { git_dir: root.join(OsStr::from_bytes(b"r\xff")).join(".git"), ..plan("refs/heads/main", FIRST_OID, 1) };
    let lower: StoredPlan = StoredPlan { git_dir: root.join(OsStr::from_bytes(b"r\xfe")).join(".git"), ..plan("refs/heads/main", SECOND_OID, 2) };
    save(root.as_path(), &higher).expect("saved");
    save(root.as_path(), &lower).expect("saved");
    let mut loaded: Vec<StoredPlan> = load(root.as_path());
    loaded.sort_by(|left, right| { return left.git_dir.cmp(&right.git_dir); });
    assert_eq!(loaded, vec![lower, higher], "both plans survive the second save");
    remove_fixture(root.as_path());
}

/// What:
///  A plan file an earlier release wrote is removed with the plan of its branch.
/// Why:
///   That file's name is keyed by the text of the directory,
/// so a removal that knew only the key of its bytes would leave it to be read again at every start.
#[test]
fn a_plan_file_of_an_earlier_release_is_removed_with_its_branch() {
    let root: PathBuf = fixture("retry-store-earlier-key");
    let first: StoredPlan = plan("refs/heads/main", FIRST_OID, 1);
    let earlier: PathBuf = root.join(format!(
        "{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}",
        earlier_branch_key(first.git_dir.as_path(), first.item.branch_ref.as_str()),
        1,
        crate::push_failure_notice::fnv_hex(FIRST_OID)
    ));
    std::fs::write(earlier.as_path(), encode(&first)).expect("a plan an earlier release wrote");
    assert_eq!(load(root.as_path()), vec![first], "the earlier plan is read before the upgrade writes one");
    save(root.as_path(), &plan("refs/heads/main", SECOND_OID, 1)).expect("saved");
    assert!(!earlier.exists(), "the earlier file goes with the branch");
    remove_fixture(root.as_path());
}

/// What:
///  A plan of another directory survives the cleanup of the key an earlier release wrote.
/// Why:
///   That key was built from the text of the path,
/// and the text of a name that holds a byte which is not text shows the replacement character,
/// so a directory named with that character itself and one named with the byte get one key;
/// saving a plan for one must not delete the other's,
/// whose push is neither done nor cancelled.
#[test]
fn a_legacy_key_does_not_remove_another_directorys_plan() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("retry-store-legacy-collision");
    let replacement: StoredPlan = StoredPlan { git_dir: root.join("r\u{fffd}").join(".git"), ..plan("refs/heads/main", FIRST_OID, 1) };
    let other: StoredPlan = StoredPlan { git_dir: root.join(OsStr::from_bytes(b"r\xff")).join(".git"), ..plan("refs/heads/main", SECOND_OID, 2) };
    assert_eq!(replacement.git_dir.display().to_string(), other.git_dir.display().to_string(), "the two names have one text, which is what the earlier key was made of");
    let legacy: PathBuf = root.join(format!(
        "{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}",
        earlier_branch_key(replacement.git_dir.as_path(), replacement.item.branch_ref.as_str()),
        1,
        crate::push_failure_notice::fnv_hex(FIRST_OID)
    ));
    std::fs::write(legacy.as_path(), encode(&replacement)).expect("a plan an earlier release wrote");
    save(root.as_path(), &other).expect("saved");
    let mut loaded: Vec<StoredPlan> = load(root.as_path());
    loaded.sort_by(|left, right| { return left.git_dir.cmp(&right.git_dir); });
    assert_eq!(loaded, vec![replacement, other], "the other directory's plan is still there");
    remove_fixture(root.as_path());
}

/// What:
///  A plan file that cannot be read is not removed by the cleanup of a legacy key.
/// Why:
///   "never discard user data."
/// The cleanup removes a file only when the record inside it was read and names this directory and this branch;
/// a file that cannot be read says nothing about the push it schedules,
/// which may be neither done nor cancelled,
/// and deleting it needs permission on its directory alone,
/// so it would go.
#[test]
fn a_plan_that_cannot_be_read_is_not_removed() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("retry-store-unreadable-plan");
    let kept: StoredPlan = plan("refs/heads/main", FIRST_OID, 1);
    let name: String = format!(
        "{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}",
        earlier_branch_key(kept.git_dir.as_path(), kept.item.branch_ref.as_str()),
        1,
        crate::push_failure_notice::fnv_hex(FIRST_OID)
    );
    let path: PathBuf = root.join(name);
    std::fs::write(path.as_path(), encode(&kept)).expect("a plan an earlier release wrote");
    // Nobody may read the record, and the directory it sits in is still writable, which is what deleting it needs.
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o000)).expect("unreadable");
    remove(root.as_path(), kept.git_dir.as_path(), kept.item.branch_ref.as_str());
    assert!(path.exists(), "a file that could not be read is kept, so its push is not forgotten");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("readable again");
    remove_fixture(root.as_path());
}

/// A plan saved for a branch replaces the earlier plan of that branch,
/// with another try number or another commit.
#[test]
fn a_newer_plan_of_a_branch_replaces_the_older() {
    let root: PathBuf = fixture("retry-store-replace");
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 1)).expect("saved");
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 2)).expect("saved");
    let newest: StoredPlan = plan("refs/heads/main", SECOND_OID, 1);
    save(root.as_path(), &newest).expect("saved");
    assert_eq!(load(root.as_path()), vec![newest]);
    assert_eq!(std::fs::read_dir(root.as_path()).expect("directory").count(), 1, "the earlier files went");
    remove_fixture(root.as_path());
}

/// A removed plan is gone,
/// and the plan of another branch is not touched.
#[test]
fn a_removed_plan_is_gone_and_another_branch_stays() {
    let root: PathBuf = fixture("retry-store-remove");
    let kept: StoredPlan = plan("refs/heads/topic", SECOND_OID, 1);
    save(root.as_path(), &plan("refs/heads/main", FIRST_OID, 1)).expect("saved");
    save(root.as_path(), &kept).expect("saved");
    remove(root.as_path(), Path::new("/srv/repo/.git"), "refs/heads/main");
    assert_eq!(load(root.as_path()), vec![kept]);
    remove_fixture(root.as_path());
}

/// A file that holds no plan is not a plan,
/// and is removed so that it is not said again.
#[test]
fn a_file_that_holds_no_plan_is_removed() {
    let root: PathBuf = fixture("retry-store-garbage");
    let garbage: PathBuf = root.join(format!("nothing.{PLAN_EXTENSION}"));
    std::fs::write(garbage.as_path(), "not a plan").expect("file");
    assert!(load(root.as_path()).is_empty());
    assert!(!garbage.exists());
    remove_fixture(root.as_path());
}

/// How long a control's reading of one plan's file is given before it counts as waiting for a writer.
/// Why:
///  A read of a pipe nobody writes to waits for ever in a service,
///  so the control puts a deadline on the read instead of holding the suite:
///  an answer inside it is the proof that no writer was waited for.
const A_READ_THAT_MUST_NOT_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// What:
///  Make `path` a pipe nobody writes to,
///  which is a name a plan's file can hold without being a plan.
/// Why:
///  Opening such a name for reading waits inside `open(2)` for a writer that may never come,
///  which is the wait a record's reader must never make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert(mkfifo(path, 0o600) === 0);
/// ```
fn a_pipe_at(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe stands at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends a read of it that waited for a writer.
/// Why:
///  A read that waited is left behind by the control's deadline,
///  and the thread it holds is released here rather than with the process.
fn end_the_read_of(path: &Path) {
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < A_READ_THAT_MUST_NOT_WAIT {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(writer) => {
                drop(writer);
                return;
            }
            // No reader waits in the pipe, so nothing is left to release.
            Err(_no_reader) => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
}

/// What:
///  A plan's file that is a pipe is never waited for,
///  read by the removal, by a save's cleanup or by a start,
///  and it is never discarded either.
/// Why:
///  Reading a record waits inside `open(2)` for a writer when the name holds a pipe,
///  and the drain does that read on a worker of the runtime,
///  so a pipe at a plan's name would hold the drain for ever
/// (finding 2 of the review of `96c4677d30`).
///  The record is read the way every record of this wrapper is read,
///  so the pipe is answered at once and stays:
///  it is not a plan,
///  and the store discards what it cannot read as one.
///  A read that was answered without a writer is the proof, and the deadline names the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(planPath); expect(within(2, () => remove(dir, gitDir, branch))).toBeDefined(); expect(exists(planPath)).toBe(true);
/// ```
#[test]
fn a_plan_file_that_is_a_pipe_is_never_waited_for() {
    use crate::test_support::finished_within_at;
    let root: PathBuf = fixture("retry-store-plan-pipe");
    let waiting: StoredPlan = plan("refs/heads/main", FIRST_OID, 1);
    save(root.as_path(), &waiting).expect("saved");
    let only: PathBuf = std::fs::read_dir(root.as_path()).expect("listing").flatten().next().expect("the plan's file").path();
    std::fs::remove_file(only.as_path()).expect("the plan's file makes way for the pipe");
    a_pipe_at(only.as_path());
    let removed: Option<()> = finished_within_at(A_READ_THAT_MUST_NOT_WAIT, {
        let (directory, git_dir, branch): (PathBuf, PathBuf, String) = (root.clone(), waiting.git_dir.clone(), waiting.item.branch_ref.clone());
        move || { remove(directory.as_path(), git_dir.as_path(), branch.as_str()); }
    });
    if removed.is_none() {
        end_the_read_of(only.as_path());
    }
    assert!(removed.is_some(), "the removal of a branch's plans waited for a writer of a pipe that holds no plan");
    assert!(only.exists(), "a name the removal could not read as a plan stays, so nothing of the person's is discarded");
    let kept: Option<()> = finished_within_at(A_READ_THAT_MUST_NOT_WAIT, {
        let (directory, next): (PathBuf, StoredPlan) = (root.clone(), plan("refs/heads/main", SECOND_OID, 2));
        move || {
            save(directory.as_path(), &next).expect("the newer plan of the branch");
        }
    });
    if kept.is_none() {
        end_the_read_of(only.as_path());
    }
    assert!(kept.is_some(), "the save of a plan waited for a writer of a pipe that holds no plan");
    assert!(only.exists(), "the save's cleanup of the branch's earlier files keeps what it cannot read as a plan");
    let read: Option<LoadedPlans> = finished_within_at(A_READ_THAT_MUST_NOT_WAIT, {
        let directory: PathBuf = root.clone();
        move || { return load_reporting(directory.as_path()); }
    });
    if read.is_none() {
        end_the_read_of(only.as_path());
    }
    let journal: LoadedPlans = read.expect("the read of the journal waited for a writer of a pipe that holds no plan");
    assert_eq!(journal.plans, vec![plan("refs/heads/main", SECOND_OID, 2)], "the plan beside the pipe is read, so the pipe costs no push its retry");
    assert!(!journal.unread, "a name that is no plan is not read again after a pause, because a second read of it answers the same");
    assert!(only.exists(), "the read keeps what is not a plan it can remove, so no file of another program is discarded");
    remove_fixture(root.as_path());
}

/// What:
///  A listing that failed part way through is not read as a complete journal.
/// Why:
///  `read_dir` opens a directory and then yields its entries,
///  and the iterator can fail before it named them all,
///  which is what a directory read on a mount that stopped answering does.
///  A plan the listing never named is absent from the plans this process holds,
///  and an item already in `failed` produces no wake of its own,
///  so a read that treated the failure as the end of the listing
///  leaves that push waiting for the life of the service
/// (finding 3 of the review of `96c4677d30`).
///  The control hands the reader one plan and then the error,
///  which is the order a directory read fails in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = readThePlans(dir, [ok(plan), err('the directory stopped answering')]); expect(read.unread).toBe(true);
/// ```
#[test]
fn a_listing_that_failed_part_way_through_is_read_again() {
    let root: PathBuf = fixture("retry-store-listing-failed");
    let waiting: StoredPlan = plan("refs/heads/main", FIRST_OID, 1);
    save(root.as_path(), &waiting).expect("saved");
    let only: PathBuf = std::fs::read_dir(root.as_path()).expect("listing").flatten().next().expect("the plan's file").path();
    let written: SystemTime = std::fs::metadata(only.as_path()).and_then(|meta| { return meta.modified(); }).expect("the plan's own time");
    // The listing names the plan first and fails after it, so the error is the entry the read below meets.
    let listed: Vec<std::io::Result<ListedPlan>> = vec![
        Ok(ListedPlan { path: only.clone(), written }),
        Err(std::io::Error::other("the directory stopped answering")),
    ];
    let read: LoadedPlans = read_the_plans(root.as_path(), listed.into_iter());
    assert_eq!(read.plans, vec![waiting], "the plan the listing named before the failure is read");
    assert!(
        read.unread,
        "a listing that failed part way through is read again, so a plan it never named is not left for the life of the service"
    );
    remove_fixture(root.as_path());
}

/// What:
///  A plan whose record could not be read is read again at a later start,
/// not removed.
/// Why:
///  "never discard user data":
/// the cleanup (`remove_files_of`) keeps a file whose record it could not read,
/// because an unreadable plan may schedule a push that is neither done nor cancelled,
/// so `load` must not take it either:
/// a read that failed says nothing about what the file holds,
/// and the next start may read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(path, 0o000); expect(load(root)).toEqual([]); expect(existsSync(path)).toBe(true); expect(loadReporting(root).unread).toBe(true);
/// ```
#[test]
fn a_plan_whose_record_could_not_be_read_is_kept_and_read_again() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("retry-store-unreadable-load");
    let waiting: StoredPlan = plan("refs/heads/main", FIRST_OID, 1);
    save(root.as_path(), &waiting).expect("saved");
    let only: PathBuf = std::fs::read_dir(root.as_path()).expect("listing").flatten().next().expect("the plan's file").path();
    std::fs::set_permissions(only.as_path(), std::fs::Permissions::from_mode(0o000)).expect("unreadable");
    assert!(load(root.as_path()).is_empty(), "a plan that could not be read is not read as a push");
    assert!(only.exists(), "a plan that could not be read is kept, so its push is not forgotten");
    assert!(load_reporting(root.as_path()).unread, "a start that met a file it could not read says so, so it reads the plan again");
    std::fs::set_permissions(only.as_path(), std::fs::Permissions::from_mode(0o600)).expect("readable again");
    assert_eq!(load(root.as_path()), vec![waiting], "the next read finds the plan the failed one kept");
    remove_fixture(root.as_path());
}
