//! Controls for the awaited faces of the commit's owed index repair:
//! both faces leave the same paths out and leave the index the same,
//! a record that gives no answer holds
//!  no worker and turns into a sentence rather than a guess,
//! and the repair runs inside a task where the synchronous face is refused.

use super::{exclude_with_note, exclude_with_note_awaiting};
use crate::owed_index_update::Owed;
use crate::service_land_start::{IndexWitness, witness_of};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The one path the worktree's index owes an update for.
const THE_OWED_PATH: &str = "one.txt";

/// The content the branch held before the landing.
const THE_REPLACED_CONTENT: &str = "old\n";

/// The content the landing put on the branch.
const THE_LANDED_CONTENT: &str = "new\n";

/// The shortest a wait can be and still leave a task beside it time to run on this host.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// How often a control looks at the flag a task beside the work sets.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most a control waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// How long a control waits after it saw the task beside the read run,
/// so the read's ceiling has surely passed
/// before the pipe that stalls it is let go:
/// the ceiling itself,
/// which is the only bound the read has.
const PAST_THE_CEILING: Duration = SHORT_CEILING;

/// What:
///  A worktree whose index still holds the entry its landing replaced,
/// with the record of what the landing owes.
/// Why:
///  This is the state a landing whose index update never ran leaves:
/// the branch holds the landed content,
/// the index holds the content before it,
/// and the worktree holds what landed,
/// so a commit of that
///  index would record the landed file as the old content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); commit('old'); commit('new'); updateIndexCacheinfo(oldBlob); writeOwed({ seq: 1, old, new, paths: ['one.txt'] })
/// ```
struct Owing {
    /// The fixture directory the repository and its copies are made in.
    root: PathBuf,
    /// The worktree the index belongs to.
    repo: PathBuf,
    /// The worktree's own Git directory,
    /// which holds the record of what the landing owes.
    git_dir: PathBuf,
    /// The live index,
    /// which the commit copies and never writes.
    index: PathBuf,
    /// The commit the landing started from.
    old: String,
    /// The commit the landing made.
    new: String,
}

/// The repository of one `Owing`,
/// with its two commits,
/// its index set back to the entry the landing replaced and its record written.
fn owing(name: &str) -> Owing {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(THE_OWED_PATH), THE_REPLACED_CONTENT.as_bytes()).expect("write the replaced content");
    git(repo.as_path(), &["add", "--", THE_OWED_PATH]);
    git(repo.as_path(), &["commit", "--quiet", "--message=before the landing"]);
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(THE_OWED_PATH), THE_LANDED_CONTENT.as_bytes()).expect("write the landed content");
    git(repo.as_path(), &["add", "--", THE_OWED_PATH]);
    git(repo.as_path(), &["commit", "--quiet", "--message=the landing"]);
    let new: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let git_dir: PathBuf = repo.join(".git");
    let index: PathBuf = git_dir.join("index");
    // The index the landing never updated still holds the entry of the content before it, while the worktree holds what landed.
    let replaced: String = git_text(repo.as_path(), &["rev-parse", format!("{old}:{THE_OWED_PATH}").as_str()]);
    git(repo.as_path(), &["update-index", "--cacheinfo", format!("100644,{replaced},{THE_OWED_PATH}").as_str()]);
    let owed: Owed = Owed {
        seq: 1,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        index: index.clone(),
        reference: String::from("refs/heads/main"),
        old: old.clone(),
        new: new.clone(),
        paths: vec![THE_OWED_PATH.as_bytes().to_vec()],
        derived: false,
    };
    assert!(crate::owed_index_update::write(&owed).is_some(), "the record of what the landing owes is written beside the worktree");
    return Owing { root, repo, git_dir, index, old, new };
}

/// The entries of one index file,
/// as Git lists them.
fn listing_of(repo: &Path, index: &Path) -> String {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .args(["ls-files", "-s", "--", THE_OWED_PATH])
        .output()
        .expect("Git runs");
    let text: String = String::from_utf8(output.stdout).expect("text");
    return String::from(text.trim_end());
}

/// A copy of the live index at `name`,
/// as a commit's preparation makes before it reads the index.
fn copy_of_the_index(owing: &Owing, name: &str) -> PathBuf {
    let copy: PathBuf = owing.root.join(name);
    std::fs::copy(owing.index.as_path(), copy.as_path()).expect("a copy of the index");
    return copy;
}

/// What:
///  What the repair answers and leaves,
/// through one of its two faces.
/// Why:
///   A control compares the two faces without asking which one answered,
/// so it names in its own words which of the two
///  disagreed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { left, note, index };
/// ```
#[derive(Debug, Eq, PartialEq)]
struct Repaired {
    /// The paths the commit leaves out.
    left: Vec<String>,
    /// The sentence the commit prints about the record it could not read,
    /// and nothing when every record could be read.
    note: Option<String>,
    /// The entries the commit's own index holds after the repair.
    index: String,
}

/// What the synchronous face answers and leaves about one copy of the index.
fn repaired_by_the_synchronous_face(owing: &Owing, copy: &Path) -> Repaired {
    let captured: IndexWitness = witness_of(owing.index.as_path()).expect("the live index is witnessed");
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note(
        &git_context(),
        owing.repo.as_path(),
        owing.git_dir.as_path(),
        owing.index.as_path(),
        Some(copy),
        Some(copy),
        &captured,
        &mut note,
    )
    .expect("the repair of a worktree that owes one readable record");
    let index: String = listing_of(owing.repo.as_path(), copy);
    return Repaired { left, note, index };
}

/// What the awaited face answers and leaves about one copy of the index.
async fn repaired_by_the_awaited_face(owing: &Owing, copy: &Path) -> Repaired {
    let captured: IndexWitness = witness_of(owing.index.as_path()).expect("the live index is witnessed");
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note_awaiting(
        &git_context(),
        owing.repo.as_path(),
        owing.git_dir.as_path(),
        owing.index.as_path(),
        Some(copy),
        Some(copy),
        &captured,
        &mut note,
    )
    .await
    .expect("the repair of a worktree that owes one readable record");
    let index: String = listing_of(owing.repo.as_path(), copy);
    return Repaired { left, note, index };
}

/// What:
///  Stand a pipe with no writer at `path`,
/// so a read of it waits in the kernel until a writer opens it.
/// Why:
///  This is the one way to make a read of a file hold its thread with no end,
/// which is what a mount that stopped answering does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
fn stall_a_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  fixture takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the record belongs: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends the read a control left behind.
/// Why:
///  The read of the pool is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w')); return } catch { await sleep(10) } }
/// ```
fn release_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < THE_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  Both faces leave the same paths out of the commit,
/// say the same thing about what they could not read,
/// and leave the
///  commit's own index the same.
/// Why:
///   The repair reads the records of what the index owes,
/// the tree the landing replaced,
/// the entry the commit's own index holds and the worktree's content,
/// and a task that reads another one of them leaves another commit out.
/// The live index is
///  never written by either face,
/// so the control also pins that no staging of the person's is replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(repairedByTheSynchronousFace(owing, copyA), await repairedByTheAwaitedFace(owing, copyB));
/// ```
#[test]
fn the_awaited_repair_answers_what_the_synchronous_one_answers() {
    let owing: Owing = owing("owed-repair-awaiting-agrees");
    let before: String = listing_of(owing.repo.as_path(), owing.index.as_path());
    let synchronous: Repaired = repaired_by_the_synchronous_face(&owing, copy_of_the_index(&owing, "commit-sync.index").as_path());
    let awaited_copy: PathBuf = copy_of_the_index(&owing, "commit-awaited.index");
    let awaited: Repaired = crate::lazy_runtime::run_on_runtime(repaired_by_the_awaited_face(&owing, awaited_copy.as_path())).expect("the runtime");
    assert_eq!(synchronous.left, vec![String::from(THE_OWED_PATH)], "the fixture is one path whose entry the landing replaced");
    assert_eq!(synchronous.left, awaited.left, "the two faces leave the same paths out of the commit");
    assert_eq!(synchronous.note, awaited.note, "the two faces say the same thing about the records they read");
    assert_eq!(synchronous.index, awaited.index, "the two faces leave the commit's own index holding the same entry");
    assert_eq!(listing_of(owing.repo.as_path(), owing.index.as_path()), before, "neither face writes the live index, so no staging of the person's is replaced");
    assert_eq!(git_text(owing.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main", "the fixture's branch is the one its record names");
    assert_eq!(git_text(owing.repo.as_path(), &["rev-parse", "HEAD"]), owing.new, "the fixture's branch holds the commit the record names");
    assert_ne!(owing.old, owing.new, "the fixture's two commits are two commits");
    remove(owing.root.as_path());
}

/// What:
///  Both faces answer one thing about an index that moved after the commit copied it,
/// and leave the commit's own index
///  the same.
/// Why:
///   A live index that is another file than the one the commit copied may hold entries the copy does not,
/// so the paths
///  that differ are left out:
/// what the copy would record there is not the person's current intent.
/// The step reads the tree
///  of the live index and the paths the copy holds differently from it,
/// and a task that reads either of them elsewhere
///  leaves another commit out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const captured = witnessOf(index); stageAnotherEntry(index); assert.deepEqual(repairedByTheSynchronousFace(...), await repairedByTheAwaitedFace(...));
/// ```
#[test]
fn both_faces_answer_one_thing_about_an_index_that_moved_after_the_copy() {
    let owing: Owing = owing("owed-repair-awaiting-moved");
    let copy_sync: PathBuf = copy_of_the_index(&owing, "commit-sync.index");
    let copy_awaited: PathBuf = copy_of_the_index(&owing, "commit-awaited.index");
    // The witness the commit recorded is the index as it was when it copied it; the live index is green afterwards, which is
    //  what a writer that replaced the index leaves.
    let captured: IndexWitness = witness_of(owing.index.as_path()).expect("the live index is witnessed");
    let landed: String = git_text(owing.repo.as_path(), &["rev-parse", format!("{}:{THE_OWED_PATH}", owing.new).as_str()]);
    git(owing.repo.as_path(), &["update-index", "--cacheinfo", format!("100644,{landed},{THE_OWED_PATH}").as_str()]);
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note(
        &git_context(),
        owing.repo.as_path(),
        owing.git_dir.as_path(),
        owing.index.as_path(),
        Some(copy_sync.as_path()),
        Some(copy_sync.as_path()),
        &captured,
        &mut note,
    )
    .expect("the repair of an index that moved");
    let synchronous: Repaired = Repaired { left, note, index: listing_of(owing.repo.as_path(), copy_sync.as_path()) };
    let awaited: Repaired = crate::lazy_runtime::run_on_runtime({
        let repo: PathBuf = owing.repo.clone();
        let git_dir: PathBuf = owing.git_dir.clone();
        let index: PathBuf = owing.index.clone();
        let copy: PathBuf = copy_awaited.clone();
        async move {
            let mut later_note: Option<String> = None;
            let later_left: Vec<String> = exclude_with_note_awaiting(&git_context(), repo.as_path(), git_dir.as_path(), index.as_path(), Some(copy.as_path()), Some(copy.as_path()), &captured, &mut later_note)
                .await
                .expect("the repair of an index that moved");
            let later_listing: String = listing_of(repo.as_path(), copy.as_path());
            return Repaired { left: later_left, note: later_note, index: later_listing };
        }
    })
    .expect("the runtime");
    assert_eq!(synchronous.left, vec![String::from(THE_OWED_PATH)], "an index that moved after the copy leaves its path out");
    assert_eq!(synchronous.left, awaited.left, "the two faces leave the same paths out of a commit whose index moved");
    assert_eq!(synchronous.note, awaited.note, "the two faces say the same thing about a commit whose index moved");
    assert_eq!(synchronous.index, awaited.index, "the two faces leave the commit's own index holding the same entry after a move");
    remove(owing.root.as_path());
}

/// What:
///  A record that gives no answer holds no worker,
/// and the commit stops with a sentence saying the records could not be looked
///  at,
/// instead of recording what it never read.
/// Why:
///   The look at the records lists a directory and reads one file of each,
/// which waits in the kernel on a disk or a
///  mount that stopped answering;
/// a task that did it itself would hold the one worker the runtime has,
/// and the answer it
///  gives must not be "nothing is owed".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(owedRecord); assert(await runsBeside(() => excludeWithNoteAwaiting(...)) === 'could not be looked at in time')
/// ```
#[test]
fn a_record_that_gives_no_answer_holds_no_worker_and_stops_the_commit() {
    let owing: Owing = owing("owed-repair-awaiting-stalls");
    let record: PathBuf = crate::owed_index_update::path_of(owing.git_dir.as_path(), 1);
    std::fs::remove_file(record.as_path()).expect("the record the pipe stands in place of");
    stall_a_file(record.as_path());
    let copy: PathBuf = copy_of_the_index(&owing, "commit-stalled.index");
    let before: String = listing_of(owing.repo.as_path(), owing.index.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let asked = {
        let context: GitContext = git_context();
        let repo: PathBuf = owing.repo.clone();
        let git_dir: PathBuf = owing.git_dir.clone();
        let index: PathBuf = owing.index.clone();
        let captured: IndexWitness = witness_of(index.as_path()).expect("the live index is witnessed");
        async move {
            let mut note: Option<String> = None;
            return super::exclude_with_note_within(SHORT_CEILING, &context, repo.as_path(), git_dir.as_path(), index.as_path(), Some(copy.as_path()), Some(copy.as_path()), &captured, &mut note)
                .await;
        }
    };
    let reading = runtime.spawn(asked);
    let began: std::time::Instant = std::time::Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The wait is past the ceiling before the pipe is let go, so the ceiling is what ends it and not a writer that arrived
    //  first, and the read the pool abandoned is ended by the writer so no thread of the process stays in it.
    std::thread::sleep(PAST_THE_CEILING);
    release_the_pipe(record.as_path());
    let answered: Result<Vec<String>, String> = runtime.block_on(async { return reading.await.expect("the repair task ends"); });
    assert!(ran_beside, "the task beside a record that gives no answer still runs, so no worker is held by that read");
    let refused: String = answered.expect_err("a look that gave no answer stops the commit rather than answering that nothing is owed");
    assert!(refused.contains("could not be looked at in time"), "the sentence says what happened: {refused}");
    assert_eq!(listing_of(owing.repo.as_path(), owing.index.as_path()), before, "a commit that was not made leaves the live index exactly as it was");
    assert!(began.elapsed() < THE_PATIENCE, "the look ended at its ceiling and never waited on the file without end: {} ms", began.elapsed().as_millis());
    remove(owing.root.as_path());
}

/// What:
///  The repair answers inside a task,
/// and the synchronous face of the same repair is refused there.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// so the commit path
///  needs these faces once it is a task;
/// the paths the awaited face leaves out are the ones the synchronous face leaves out,
///  which is what pins that its children ran and were awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await runtime.spawn(() => excludeWithNoteAwaiting(...)); const refused = await runtime.spawn(() => excludeWithNote(...));
/// ```
#[test]
fn the_repair_answers_inside_a_task_where_the_synchronous_one_is_refused() {
    let owing: Owing = owing("owed-repair-awaiting-in-a-task");
    let awaited_copy: PathBuf = copy_of_the_index(&owing, "commit-awaited.index");
    let synchronous_copy: PathBuf = copy_of_the_index(&owing, "commit-sync.index");
    let expected: Repaired = repaired_by_the_synchronous_face(&owing, synchronous_copy.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let answered = {
        let repo: PathBuf = owing.repo.clone();
        let git_dir: PathBuf = owing.git_dir.clone();
        let index: PathBuf = owing.index.clone();
        let copy: PathBuf = awaited_copy.clone();
        runtime.block_on(async move {
            let reading = tokio::spawn(async move { return repaired_by_the_awaited_face_of(&repo, &git_dir, &index, copy.as_path()).await; });
            return reading.await.expect("the repair task ends");
        })
    };
    assert_eq!(answered.left, expected.left, "the awaited face leaves the same paths out inside a task");
    assert_eq!(answered.index, expected.index, "the awaited face leaves the commit's own index holding the same entry inside a task");
    let refused = {
        let repo: PathBuf = owing.repo.clone();
        let git_dir: PathBuf = owing.git_dir.clone();
        let index: PathBuf = owing.index.clone();
        let copy: PathBuf = copy_of_the_index(&owing, "commit-still.index");
        runtime.block_on(async move {
            let asking = tokio::spawn(async move { return repaired_by_the_synchronous_face_of(&repo, &git_dir, &index, copy.as_path()); });
            return asking.await.expect("the task that asks ends");
        })
    };
    let said: String = refused.expect_err("the synchronous repair cannot start its Git from inside the runtime");
    assert!(said.contains("Unavailable"), "the refusal says Git could not be asked: {said}");
    remove(owing.root.as_path());
}

/// What the awaited repair answers about one copy of the index,
/// built from paths that a task may own.
/// Why:
///  A task owns everything it touches,
/// so a control of the repair inside a task names the paths it reads instead of lending them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await repairedByTheAwaitedFaceOf(repo, gitDir, index, copy);
/// ```
async fn repaired_by_the_awaited_face_of(repo: &Path, git_dir: &Path, index: &Path, copy: &Path) -> Repaired {
    let captured: IndexWitness = witness_of(index).expect("the live index is witnessed");
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note_awaiting(&git_context(), repo, git_dir, index, Some(copy), Some(copy), &captured, &mut note)
        .await
        .expect("the repair of a worktree that owes one readable record");
    let listing: String = listing_of(repo, copy);
    return Repaired { left, note, index: listing };
}

/// What the synchronous repair answers about one copy of the index,
/// as a task that may not start Git asks it.
/// Why:
///  The refusal is the answer the control pins,
/// and the index it would have written is what the task's own look must not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWithNote(git, repo, gitDir, index, copy, copy, captured)
/// ```
fn repaired_by_the_synchronous_face_of(repo: &Path, git_dir: &Path, index: &Path, copy: &Path) -> Result<Repaired, String> {
    let captured: IndexWitness = witness_of(index).expect("the live index is witnessed");
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note(&git_context(), repo, git_dir, index, Some(copy), Some(copy), &captured, &mut note)?;
    let listing: String = listing_of(repo, copy);
    return Ok(Repaired { left, note, index: listing });
}
