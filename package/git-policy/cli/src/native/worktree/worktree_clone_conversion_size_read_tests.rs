//! Controls for the fence's size read beside the source check:
//! the guard that leaves a read of another commit unused,
//! the deadline and the dead task that refuse the rung in a failed read's own sentence,
//! the partial clone the read may not fetch from,
//! and the cost an add states for a read it never used.

use super::*;
use crate::add_timing::{ADD_TIMING_FILE, begin as begin_timing, finish as finish_timing};
use crate::test_support::{git, git_context, git_output, remove, repository};
use crate::worktree_clone_conversion_fence::fence_the_clone;
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_ladder_run::{Ladder, Rung};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use crate::worktree_snapshot_source::source_refusal;
use std::path::{Path, PathBuf};

/// The fixture root of one control,
/// under the package's own test directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = mkdtemp(join(fixtureRoot(), 'size-read-settled'));
/// ```
fn fixture(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture_root().join(format!("size-read-{name}"));
    let _gone = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("the fixture root");
    return root;
}

/// What:
///  A clean source with two tracked files and one commit,
///  older than its index.
/// Why:
///   The source check must accept it,
/// and every path of it needs a blob size,
///  so the fence's decision reaches the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(root, name); tracked(repo, 'one.txt', 'one\n'); commitAll(repo, 'a tree');
/// ```
fn a_clean_source(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    tracked(repo.as_path(), "one.txt", b"one\n");
    tracked(repo.as_path(), "dir/two.txt", b"two, two\n");
    commit_all(repo.as_path(), "a tree");
    return std::fs::canonicalize(repo).expect("canonical");
}

/// A runner that must not be called:
/// the add this control settles never reaches Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noRunner = () => { throw new Error('no Git run is expected'); };
/// ```
fn no_runner() -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
    return |_step: &GitStep<'_>| { panic!("this add never runs Git here"); };
}

/// The record the last add appended,
/// as the one line of JSON it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readFileSync(join(common, ADD_TIMING_FILE), 'utf8');
/// ```
fn the_record(repo: &Path) -> String {
    return std::fs::read_to_string(repo.join(".git").join(ADD_TIMING_FILE)).expect("the add's record");
}

/// What:
///  Whether the record names `name` as a step or as a work entry.
/// Why:
///   The quiet window of every Git run is noted under the run's own name as a plain number,
/// so a search for the name alone would find `git ls-tree` in `quiet`
///  and could not tell a step of the add from a gap in one;
/// a step and a work entry are the only members spelled `{"ms":...}`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const states = (record, name) => record.includes(`"${name}":{"ms":`);
/// ```
fn the_record_states(record: &str, name: &str) -> bool {
    return record.contains(format!("\"{name}\":{{\"ms\":").as_str());
}

/// Both halves of the snapshot rung's fence for a source the check proved,
/// with the read this add is holding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fence = fenceTheCloneWithTheReadBesideTheCheck(source, context, started);
/// ```
fn the_fence(repo: &Path, context: &GitContext, started: &mut Option<SizeReadBesideTheCheck>) -> Result<Fence, String> {
    let identity: WorktreeIdentity = main_identity(repo);
    let source: SnapshotSource = source_refusal(context, &identity).expect("the source is clean");
    return fence_the_clone_with_the_read_beside_the_check(&source, context, started);
}

/// A settled `HEAD` is the positive control:
/// the fence takes the read,
///  pays no second `git ls-tree`,
///  and the record states the read's own cost.
#[test]
fn a_settled_head_uses_the_read_beside_the_check() {
    let root: PathBuf = fixture("settled");
    let repo: PathBuf = a_clean_source(root.as_path(), "settled");
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo.as_path())).expect("a read starts");
    begin_timing(repo.join(".git"), true);
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = the_fence(repo.as_path(), &context, &mut held).expect("the fence decides");
    finish_timing(0);
    assert!(held.is_none(), "the fence took the read the check proved");
    let record: String = the_record(repo.as_path());
    assert!(!the_record_states(record.as_str(), "git ls-tree"), "the fence paid no second read: {record}");
    assert!(the_record_states(record.as_str(), "fenceSizesBesideTheCheck"), "the read's own cost is the record's work: {record}");
    assert!(the_record_states(record.as_str(), "fenceSizesWaitingForTheRead"), "the join's wait is a step: {record}");
    let source: SnapshotSource = source_refusal(&context, &main_identity(repo.as_path())).expect("the source is clean");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried), "the same map decides the same paths");
    remove(root.as_path());
}

/// What:
///  What one `HEAD` shape does between the read's start and the source check.
/// Why:
///   The window the guard covers is exactly the moves here,
/// and each shape writes the reference differently.
type MoveHead = fn(&Path);

/// A commit with a different tree,
/// so the commit the check proves is not a name for the read's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tracked(repo, 'dir/three.txt', 'three\n'); commitAll(repo, 'another tree');
/// ```
fn commit_of_another_tree(repo: &Path) {
    tracked(repo, "dir/three.txt", b"three\n");
    commit_all(repo, "another tree");
}

/// A commit of the same tree,
/// so only the commit id differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(repo, ['commit', '--allow-empty', '--quiet', '--message=same tree']);
/// ```
fn commit_of_the_same_tree(repo: &Path) {
    git(repo, &["commit", "--allow-empty", "--quiet", "--message=same tree"]);
}

/// The symbolic ref moves to a commit of the same tree.
fn the_symbolic_ref_moves(repo: &Path) {
    commit_of_the_same_tree(repo);
}

/// The branch lives only in `packed-refs`,
/// so the read has to resolve `HEAD` from there before a loose ref appears over it.
fn the_branch_lives_only_in_packed_refs(repo: &Path) {
    git(repo, &["pack-refs", "--all"]);
}

/// `HEAD` is detached,
/// so the read resolves a direct `HEAD` before it moves.
fn the_head_is_detached(repo: &Path) {
    git(repo, &["switch", "--quiet", "--detach"]);
}

/// `HEAD` moves to a commit whose tree is another one.
fn the_head_moves_to_another_tree(repo: &Path) {
    commit_of_another_tree(repo);
}

/// What:
///  The one control of a stale read:
/// the read lists the commit before the move,
///  the check proves the one after,
///  and the fence must read the proved commit itself.
/// Why:
///   The guard compares commits,
/// and the map is a function of the commit and of nothing else,
///  so the destination is the one the serial order decides for the proved commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = startBesideTheCheck(context, identity); moveHead(repo); const fence = theFence(repo, context, started);
/// expect(fence.gitWrites).toEqual(serial.gitWrites);
/// ```
fn the_stale_read_is_dropped(repo: &Path, prepare: Option<MoveHead>, move_head: MoveHead) {
    if let Some(before) = prepare {
        before(repo);
    }
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo)).expect("a read starts");
    move_head(repo);
    begin_timing(repo.join(".git"), true);
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = the_fence(repo, &context, &mut held).expect("the fence decides the proved commit");
    finish_timing(0);
    assert!(held.is_none(), "the fence ended the read of the earlier commit where it decided not to use it");
    let record: String = the_record(repo);
    assert!(the_record_states(record.as_str(), "git ls-tree"), "the fence read the proved commit itself: {record}");
    let source: SnapshotSource = source_refusal(&context, &main_identity(repo)).expect("the source is clean");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried), "the proved commit's own map decides the paths");
}

/// The symbolic ref moving to another commit of the same tree drops the read.
#[test]
fn a_moved_symbolic_ref_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-symbolic");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), None, the_symbolic_ref_moves);
    remove(root.as_path());
}

/// A loose ref appearing over the branch `packed-refs` held drops the read.
#[test]
fn a_packed_ref_overlaid_by_a_loose_one_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-packed");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), Some(the_branch_lives_only_in_packed_refs), commit_of_the_same_tree);
    remove(root.as_path());
}

/// A moving detached `HEAD` drops the read.
#[test]
fn a_detached_head_moving_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-detached");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), Some(the_head_is_detached), commit_of_the_same_tree);
    remove(root.as_path());
}

/// `HEAD` moving to a commit of another tree drops the read,
/// and the tree the check proved decides the paths.
#[test]
fn a_head_moved_to_another_tree_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-tree");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), None, the_head_moves_to_another_tree);
    remove(root.as_path());
}

/// A linked worktree's own `HEAD` moving drops the read.
#[test]
fn a_linked_worktrees_head_moving_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-linked");
    let repo: PathBuf = a_clean_source(root.as_path(), "linked-main");
    let made: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", made.to_str().expect("utf8")]);
    let linked: PathBuf = std::fs::canonicalize(made).expect("canonical");
    // A checkout made a moment ago is racy against its own index, and the clean check refuses that until the files are older.
    age_every_file(linked.as_path());
    let common: PathBuf = repo.join(".git");
    let identity = WorktreeIdentity::LinkedWorktree { worktree_root: linked.clone(), git_dir: common.join("worktrees").join("linked"), common_dir: common };
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &identity).expect("a read starts");
    commit_of_the_same_tree(linked.as_path());
    let source: SnapshotSource = source_refusal(&context, &identity).expect("the linked source is clean");
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = fence_the_clone_with_the_read_beside_the_check(&source, &context, &mut held).expect("the fence decides");
    assert!(held.is_none(), "the fence ended the read of the commit before the move");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried));
    remove(root.as_path());
}

/// A linked worktree's settled own `HEAD` is the positive control for its own resolution:
/// the read lists that `HEAD`,
/// not the main worktree's.
#[test]
fn a_linked_worktrees_settled_head_uses_the_read_beside_the_check() {
    let root: PathBuf = fixture("linked-settled");
    let repo: PathBuf = a_clean_source(root.as_path(), "linked-main");
    let first: String = the_head_commit(repo.as_path());
    // The main checkout moves on, so its `HEAD` names a commit the linked worktree's does not.
    commit_of_another_tree(repo.as_path());
    let made: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", made.to_str().expect("utf8"), first.as_str()]);
    let linked: PathBuf = std::fs::canonicalize(made).expect("canonical");
    age_every_file(linked.as_path());
    // The checkout's index records the times the files had when it was written; refreshing them records the aged ones,
    // so every entry is older than the index that holds it.
    git(linked.as_path(), &["update-index", "--really-refresh"]);
    let common: PathBuf = repo.join(".git");
    let identity = WorktreeIdentity::LinkedWorktree { worktree_root: linked.clone(), git_dir: common.join("worktrees").join("linked"), common_dir: common };
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &identity).expect("a read starts");
    let source: SnapshotSource = source_refusal(&context, &identity).expect("the linked source is clean");
    begin_timing(repo.join(".git"), true);
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = fence_the_clone_with_the_read_beside_the_check(&source, &context, &mut held).expect("the fence decides");
    finish_timing(0);
    assert!(held.is_none(), "the read listed the linked worktree's own HEAD, which is the commit the check proved");
    let record: String = the_record(repo.as_path());
    assert!(!the_record_states(record.as_str(), "git ls-tree"), "a read of the main checkout's HEAD would have been dropped and read here: {record}");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried));
    remove(root.as_path());
}

/// A read that outlives its deadline refuses the rung in the sentence a failed read gives.
#[test]
fn a_read_that_outlives_its_deadline_refuses_the_rung() {
    let root: PathBuf = fixture("deadline");
    let repo: PathBuf = a_clean_source(root.as_path(), "deadline");
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo.as_path())).expect("a read starts");
    let hold = a_control_holds_the_read();
    let shortened = a_control_shortens_the_deadline_to(Duration::from_millis(50));
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let outcome: Result<Fence, String> = the_fence(repo.as_path(), &context, &mut held);
    let Err(reason) = outcome else { panic!("the rung is refused rather than held") };
    assert!(reason.starts_with("a Git run of the fence could not start ("), "{reason}");
    assert!(reason.contains("did not answer within 50ms"), "{reason}");
    drop(hold);
    drop(shortened);
    remove(root.as_path());
}

/// A read whose task dies refuses the rung in the sentence a failed read gives.
#[test]
fn a_read_whose_task_dies_refuses_the_rung() {
    let root: PathBuf = fixture("dies");
    let repo: PathBuf = a_clean_source(root.as_path(), "dies");
    let dies = a_control_makes_the_read_die();
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo.as_path())).expect("a read starts");
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let outcome: Result<Fence, String> = the_fence(repo.as_path(), &context, &mut held);
    let Err(reason) = outcome else { panic!("the rung is refused rather than held") };
    assert!(reason.starts_with("a Git run of the fence could not start ("), "{reason}");
    drop(dies);
    remove(root.as_path());
}

/// A read that cannot start gives exactly the sentence the serial read gives.
#[test]
fn a_read_that_cannot_start_gives_the_serial_reads_sentence() {
    let root: PathBuf = fixture("cannot-start");
    let repo: PathBuf = a_clean_source(root.as_path(), "cannot-start");
    let mut broken: GitContext = git_context();
    broken.real_git = root.join("no-git-here");
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let early: String = start_beside_the_check(&broken, &identity).expect("a read starts").join().expect_err("the read cannot start");
    let commit: String = String::from_utf8(git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("utf8").trim().to_string();
    let serial: String = blob_sizes(&broken, repo.as_path(), commit.as_str()).expect_err("the serial read cannot start");
    assert_eq!(early, serial, "one sentence names both runs");
    remove(root.as_path());
}

/// Give every file below `root` the fixture's own past time,
/// so the index that was written a moment ago is newer than all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdirSync(root, { withFileTypes: true })) { entry.isDirectory() ? ageEveryFile(join(root, entry.name)) : utimesSync(join(root, entry.name), past, past); }
/// ```
fn age_every_file(root: &Path) {
    for entry in std::fs::read_dir(root).expect("the checkout").flatten() {
        let path: PathBuf = entry.path();
        if entry.file_type().expect("a file type").is_dir() {
            if path.file_name() != Some(std::ffi::OsStr::new(".git")) {
                age_every_file(path.as_path());
            }
            continue;
        }
        crate::test_support::age(path.as_path());
    }
}

/// The commit `HEAD` names,
/// as a fixture's own Git spells it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = execFileSync(git, ['rev-parse', 'HEAD'], { cwd: repo }).toString().trim();
/// ```
fn the_head_commit(repo: &Path) -> String {
    return String::from_utf8(git(repo, &["rev-parse", "HEAD"]).stdout).expect("utf8").trim().to_string();
}

/// The read leaves another step's failure note alone,
/// and asks Git for the sizes it decides with.
#[test]
fn the_read_beside_the_check_leaves_another_steps_note_alone() {
    let root: PathBuf = fixture("note");
    let repo: PathBuf = a_clean_source(root.as_path(), "note");
    crate::git_step_failure::note_failed_step("git add", Some(1));
    let sizes: BlobSizes = start_beside_the_check(&git_context(), &main_identity(repo.as_path())).expect("a read starts").join().expect("the sizes");
    assert_eq!(sizes.len(), 2, "both tracked paths have a size");
    assert_eq!(crate::git_step_failure::drain_failed_step(), Some(crate::git_step_failure::FailedStep { name: String::from("git add"), status: Some(1) }), "the read took no note of its own");
    remove(root.as_path());
}

/// The early read's request is the only thing that differs from the serial one,
/// and it carries the no-fetch variable on every source and no option.
#[test]
fn the_early_read_runs_without_lazy_fetch_and_outside_the_note() {
    let read: GitRequest = the_early_read_request(Path::new("/nowhere"), "HEAD");
    assert!(read.beside_another_step, "the read leaves the failed-step note alone");
    assert!(read.end_with_the_starting_thread, "the read's child ends with the thread that starts it");
    assert_eq!(
        read.environment,
        vec![(OsString::from("GIT_NO_LAZY_FETCH"), OsString::from("1"))],
        "every early read carries the variable, with no test of the source"
    );
    assert_eq!(
        read.arguments,
        vec![OsString::from("ls-tree"), OsString::from("-r"), OsString::from("-l"), OsString::from("-z"), OsString::from("HEAD")],
        "the option is not used: an older Git refuses it and a plain source would fall to Git's checkout"
    );
}

/// The sizes both reads answer are the same map for the same commit.
#[test]
fn the_early_read_answers_the_map_the_serial_read_answers() {
    let root: PathBuf = fixture("same-map");
    let repo: PathBuf = a_clean_source(root.as_path(), "same-map");
    let context: GitContext = git_context();
    let early: BlobSizes = start_beside_the_check(&context, &main_identity(repo.as_path())).expect("a read starts").join().expect("the sizes");
    let serial: BlobSizes = blob_sizes(&context, repo.as_path(), the_head_commit(repo.as_path()).as_str()).expect("the serial sizes");
    assert_eq!(early, serial);
    remove(root.as_path());
}

/// The read travels as the add's own value:
/// no process-wide slot holds it,
///  and the ladder carries it from the start to its own end.
#[test]
fn no_process_wide_slot_holds_the_read_beside_the_check() {
    let here: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/native/worktree/worktree_clone_conversion_size_read.rs")).expect("this module's source");
    for line in here.lines() {
        if line.trim_start().starts_with("static ") {
            assert!(!line.contains("SizeReadBesideTheCheck"), "a process-wide slot would carry one command's read into another: {line}");
        }
    }
    let ladder: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/native/worktree/worktree_ladder_run.rs")).expect("the ladder's source");
    assert!(ladder.contains("pub(crate) sizes_beside: Option<crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck>"), "the add owns its read");
}

/// What:
///  The record a settle writes for a read with a chosen answer.
/// Why:
///   The two answers a settle can get are facts about the add,
///  and a hand-made receiver states each one without a running Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settledRecord = (answer?: [number, Result]) => { const started = makeRead(answer); beginTiming(); started.settle(); finishTiming(0); return record(); };
/// ```
fn settle_with(answer: Option<ReadAnswer>) -> String {
    let root: PathBuf = fixture(if answer.is_some() { "settle-answered" } else { "settle-running" });
    let repo: PathBuf = a_clean_source(root.as_path(), "settle");
    let (sender, receiver) = std::sync::mpsc::channel();
    if let Some((spent, outcome)) = answer {
        sender.send((spent, outcome)).expect("the receiver is held");
    }
    let read: SizeReadBesideTheCheck = SizeReadBesideTheCheck { commit: String::from("the commit"), answer: receiver, child: std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0)) };
    begin_timing(repo.join(".git"), true);
    read.settle();
    finish_timing(0);
    let record: String = the_record(repo.as_path());
    drop(sender);
    remove(root.as_path());
    return record;
}

/// A read that has already answered is stated as the work it cost,
/// with no wait of the adding thread.
#[test]
fn a_read_that_already_answered_is_stated_as_work() {
    let record: String = settle_with(Some((Duration::from_millis(7), Ok(BlobSizes::default()))));
    assert!(the_record_states(record.as_str(), WORK), "the read's own cost is the record's work: {record}");
    assert!(record.contains("\"calls\":1"), "the work is one entry: {record}");
    assert!(!the_record_states(record.as_str(), "fenceSizesWaitingForTheRead"), "the settle entered no wait: {record}");
}

/// A read that has not answered is stated as started and not joined,
/// and the add ends without waiting for it.
#[test]
fn a_read_that_has_not_answered_is_started_and_not_joined() {
    let began = std::time::Instant::now();
    let record: String = settle_with(None);
    let elapsed: Duration = began.elapsed();
    assert!(elapsed < Duration::from_secs(2), "the settle never waits for the read: {elapsed:?}");
    assert!(record.contains("\"fenceSizesBesideTheCheck\":\"started and not joined\""), "the record states the read was started and not joined: {record}");
    assert!(!the_record_states(record.as_str(), WORK), "a read still running has no cost to state: {record}");
    assert!(!the_record_states(record.as_str(), "fenceSizesWaitingForTheRead"), "the settle entered no wait: {record}");
}

/// An add that never reaches the fence states the read it holds as started and not joined,
/// and ends without waiting for it.
#[test]
fn an_add_that_never_reaches_the_fence_does_not_wait_for_the_read() {
    let root: PathBuf = fixture("settled-only");
    let repo: PathBuf = a_clean_source(root.as_path(), "settled-only");
    let destination: PathBuf = root.join("topic");
    let started: SizeReadBesideTheCheck = start_beside_the_check(&git_context(), &main_identity(repo.as_path())).expect("a read starts");
    let hold = a_control_holds_the_read();
    begin_timing(repo.join(".git"), true);
    let ladder: Ladder = Ladder { plan: None, rung: Rung::Git(None), slow: None, sizes_beside: Some(started) };
    let mut runner = no_runner();
    let began = std::time::Instant::now();
    let result = ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    let elapsed: Duration = began.elapsed();
    finish_timing(0);
    drop(hold);
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert!(elapsed < Duration::from_secs(2), "a held read does not hold the add: {elapsed:?}");
    let record: String = the_record(repo.as_path());
    assert!(record.contains("\"fenceSizesBesideTheCheck\":\"started and not joined\""), "the add states the read it did not join: {record}");
    assert!(!the_record_states(record.as_str(), "fenceSizesWaitingForTheRead"), "the add entered no wait: {record}");
    assert!(!destination.exists(), "the read writes nothing where the worktree goes");
    remove(root.as_path());
}

/// A sparse partial clone of `origin` with the cone on `dir0`,
/// so every blob of `dir1` is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(root, ['clone', '--sparse', '--filter=blob:none', `file://${origin}`, clone]); git(clone, ['sparse-checkout', 'set', 'dir0']);
/// ```
fn a_sparse_partial_clone(origin: &Path, root: &Path, name: &str) -> PathBuf {
    let clone: PathBuf = root.join(name);
    let url: String = format!("file://{}", origin.display());
    git(root, &["clone", "--quiet", "--sparse", "--filter=blob:none", url.as_str(), clone.to_str().expect("utf8")]);
    git(clone.as_path(), &["sparse-checkout", "set", "dir0"]);
    return std::fs::canonicalize(clone).expect("canonical");
}

/// What:
///  An origin with three blobs in each of `dir0` and `dir1`,
///  whose upload wants the filter a partial clone asks for.
/// Why:
///   Every shape below starts from a sparse partial clone of it,
/// so `dir1`'s six blobs are the ones a lazy fetch downloads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const d of ['dir0', 'dir1']) for (let i = 0; i < 3; i += 1) writeFile(join(origin, d, `f${i}.txt`), `${d}-${i}\n`); git(origin, ['add', '--all']); git(origin, ['commit', '-m', 'blobs']); git(origin, ['config', 'uploadpack.allowfilter', 'true']);
/// ```
fn a_partial_clone_origin(root: &Path) -> PathBuf {
    let origin: PathBuf = root.join("origin");
    for directory in ["dir0", "dir1"] {
        std::fs::create_dir_all(origin.join(directory)).expect("a directory");
        for index in 0..3 {
            std::fs::write(origin.join(directory).join(format!("f{index}.txt")), format!("{directory}-{index}\n")).expect("a blob");
        }
    }
    git(origin.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(origin.as_path(), &["add", "--all"]);
    git(origin.as_path(), &["commit", "--quiet", "--message=blobs"]);
    git(origin.as_path(), &["config", "uploadpack.allowfilter", "true"]);
    return std::fs::canonicalize(origin).expect("canonical");
}

/// What:
///  How many objects the clone does not hold,
///  and how many packs it holds.
/// Why:
///   A lazy fetch writes a pack for what it downloads and makes the missing objects present,
/// so the pair answers whether a read fetched anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const missing = git(clone, ['rev-list', '--objects', '--all', '--missing=print']).split('\n').filter((line) => line.startsWith('?')).length;
/// const packs = readdirSync(join(clone, '.git/objects/pack')).filter((name) => name.endsWith('.pack')).length;
/// ```
fn missing_and_packs(clone: &Path) -> (usize, usize) {
    let listed: std::process::Output = git_output(clone, &["rev-list", "--objects", "--all", "--missing=print"]);
    let missing: usize = listed.stdout.split(|byte| return *byte == b'\n').filter(|line| return line.first() == Some(&b'?')).count();
    let packs: usize = std::fs::read_dir(clone.join(".git/objects/pack"))
        .map(|entries| { return entries.flatten().filter(|entry| return entry.path().extension() == Some(std::ffi::OsStr::new("pack"))).count(); })
        .unwrap_or(0);
    return (missing, packs);
}

/// What:
///  Remove every `.promisor` marker from the clone's pack directory.
/// Why:
///   The integrated recheck's misses were promisor remotes with no marker in the pack directory
///  (the integration recheck's High 1 of 2026-11-22);
/// a control has to take the markers away to reproduce them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdirSync(join(clone, '.git/objects/pack'))) if (name.endsWith('.promisor')) rmSync(join(clone, '.git/objects/pack', name));
/// ```
fn remove_every_promisor_marker(clone: &Path) {
    let packs: PathBuf = clone.join(".git/objects/pack");
    for entry in std::fs::read_dir(packs).expect("the pack directory").flatten() {
        if entry.path().extension() == Some(std::ffi::OsStr::new("promisor")) {
            std::fs::remove_file(entry.path()).expect("the marker goes");
        }
    }
}

/// What:
///  Move the remote's promisor flags out of the common config and into `remote-flags.cfg`,
///  which the common config includes,
///  and take the pack markers away.
/// Why:
///   This is the shape the recheck made fetch:
/// a promisor remote whose flags arrive through an included file,
///  with no word `promisor` in the common config and no `.promisor` marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(clone, ['config', '--unset', 'remote.origin.promisor']); git(clone, ['config', '--unset', 'remote.origin.partialclonefilter']); git(clone, ['config', 'include.path', 'remote-flags.cfg']); git(clone, ['config', '--file', '.git/remote-flags.cfg', 'remote.origin.promisor', 'true']); git(clone, ['config', '--file', '.git/remote-flags.cfg', 'remote.origin.partialclonefilter', 'blob:none']); removeEveryPromisorMarker(clone);
/// ```
fn the_promisor_flags_in_an_included_file(clone: &Path) {
    let _unset_promisor: std::process::Output = git_output(clone, &["config", "--unset", "remote.origin.promisor"]);
    let _unset_filter: std::process::Output = git_output(clone, &["config", "--unset", "remote.origin.partialclonefilter"]);
    git(clone, &["config", "include.path", "remote-flags.cfg"]);
    let flags: PathBuf = clone.join(".git/remote-flags.cfg");
    git(clone, &["config", "--file", flags.to_str().expect("utf8"), "remote.origin.promisor", "true"]);
    git(clone, &["config", "--file", flags.to_str().expect("utf8"), "remote.origin.partialclonefilter", "blob:none"]);
    remove_every_promisor_marker(clone);
}

/// What:
///  A read started beside the check,
///  joined and counted:
///  the clone's missing objects and packs before and after,
///  which are equal when the read fetched nothing.
/// Why:
///   The join is what lets a late fetch land before the count;
/// the old control settled without waiting,
///  so a fetch could arrive after it looked
///  (the integration recheck's High 1 of 2026-11-22).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = missingAndPacks(clone); startBesideTheCheck(context, identity).join(); return [before, missingAndPacks(clone)];
/// ```
fn an_early_read_joined_leaves_the_clone_alone(clone: &Path, context: &GitContext) -> ((usize, usize), (usize, usize)) {
    let before: (usize, usize) = missing_and_packs(clone);
    let _answer: Result<BlobSizes, String> = start_beside_the_check(context, &main_identity(clone)).expect("a read starts before the check").join();
    return (before, missing_and_packs(clone));
}

/// A sparse partial clone the check refuses fetches nothing,
/// and the read without the variable is the positive control that its count can move.
#[test]
fn a_read_beside_the_check_writes_nothing_into_a_sparse_partial_clone_the_check_refuses() {
    let root: PathBuf = fixture("partial");
    let origin: PathBuf = a_partial_clone_origin(root.as_path());
    let context: GitContext = git_context();
    let refused: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "refused");
    let before: (usize, usize) = missing_and_packs(refused.as_path());
    assert!(before.0 > 0, "the clone is missing the blobs the cone leaves out: {before:?}");
    let identity: WorktreeIdentity = main_identity(refused.as_path());
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &identity).expect("a read starts before the check");
    let refusal = source_refusal(&context, &identity).expect_err("the check refuses the sparse checkout");
    assert!(refusal.reason.contains("sparse checkout"), "{}", refusal.reason);
    // The read is joined, not settled: a fetch that lands late must still be seen by the count.
    let _answer: Result<BlobSizes, String> = started.join();
    assert_eq!(missing_and_packs(refused.as_path()), before, "the read fetched nothing into the source the check refused");
    // The positive control: the same read without GIT_NO_LAZY_FETCH fetches, so the count above is one that can move.
    let fetched: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "fetched");
    let before_the_serial_read: (usize, usize) = missing_and_packs(fetched.as_path());
    let _serial: BlobSizes = blob_sizes(&context, fetched.as_path(), the_head_commit(fetched.as_path()).as_str()).expect("the serial read");
    assert_ne!(missing_and_packs(fetched.as_path()), before_the_serial_read, "the serial read fetches, so the pair can move");
    remove(root.as_path());
}

/// The promisor remote whose flags arrive through an included file fetches nothing.
#[test]
fn a_read_with_the_promisor_flags_in_an_included_file_fetches_nothing() {
    let root: PathBuf = fixture("promisor-include");
    let origin: PathBuf = a_partial_clone_origin(root.as_path());
    let clone: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "included");
    the_promisor_flags_in_an_included_file(clone.as_path());
    let (before, after) = an_early_read_joined_leaves_the_clone_alone(clone.as_path(), &git_context());
    assert!(before.0 > 0, "the clone is missing the blobs the cone leaves out: {before:?}");
    assert_eq!(after, before, "the flags arriving through an include fetch nothing");
    remove(root.as_path());
}

/// The promisor remote set only through `GIT_CONFIG_COUNT` and its keys fetches nothing.
#[test]
fn a_read_with_the_promisor_remote_from_the_environment_fetches_nothing() {
    let root: PathBuf = fixture("promisor-environment");
    let origin: PathBuf = a_partial_clone_origin(root.as_path());
    let clone: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "environment");
    let _unset_promisor: std::process::Output = git_output(clone.as_path(), &["config", "--unset", "remote.origin.promisor"]);
    let _unset_filter: std::process::Output = git_output(clone.as_path(), &["config", "--unset", "remote.origin.partialclonefilter"]);
    let _unset_default: std::process::Output = git_output(clone.as_path(), &["config", "--unset", "extensions.partialclone"]);
    remove_every_promisor_marker(clone.as_path());
    // SAFETY: nextest gives this control its own process, and no thread reads the environment before these are set.
    unsafe {
        std::env::set_var("GIT_CONFIG_COUNT", "2");
        std::env::set_var("GIT_CONFIG_KEY_0", "remote.origin.promisor");
        std::env::set_var("GIT_CONFIG_VALUE_0", "true");
        std::env::set_var("GIT_CONFIG_KEY_1", "remote.origin.partialclonefilter");
        std::env::set_var("GIT_CONFIG_VALUE_1", "blob:none");
    }
    let (before, after) = an_early_read_joined_leaves_the_clone_alone(clone.as_path(), &git_context());
    // SAFETY: this control's own process, and the read has ended.
    unsafe {
        std::env::remove_var("GIT_CONFIG_COUNT");
        std::env::remove_var("GIT_CONFIG_KEY_0");
        std::env::remove_var("GIT_CONFIG_VALUE_0");
        std::env::remove_var("GIT_CONFIG_KEY_1");
        std::env::remove_var("GIT_CONFIG_VALUE_1");
    }
    assert!(before.0 > 0, "the clone is missing the blobs the cone leaves out: {before:?}");
    assert_eq!(after, before, "a remote from the environment fetches nothing");
    remove(root.as_path());
}

/// A `--reference` clone of a partial clone,
/// with its promisor flags in an include,
/// fetches nothing.
#[test]
fn a_read_beside_a_reference_clone_of_a_partial_clone_fetches_nothing() {
    let root: PathBuf = fixture("promisor-reference");
    let origin: PathBuf = a_partial_clone_origin(root.as_path());
    let partial: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "reference-source");
    let reference: PathBuf = root.join("reference");
    let url: String = format!("file://{}", origin.display());
    git(root.as_path(), &["clone", "--quiet", "--sparse", "--reference", partial.to_str().expect("utf8"), "--filter=blob:none", url.as_str(), reference.to_str().expect("utf8")]);
    git(reference.as_path(), &["sparse-checkout", "set", "dir0"]);
    let reference: PathBuf = std::fs::canonicalize(reference).expect("canonical");
    the_promisor_flags_in_an_included_file(reference.as_path());
    let (before, after) = an_early_read_joined_leaves_the_clone_alone(reference.as_path(), &git_context());
    assert!(before.0 > 0, "the reference clone is missing the blobs its alternate lacks: {before:?}");
    assert_eq!(after, before, "the flags arriving through an include on a reference clone fetch nothing");
    remove(root.as_path());
}

/// What:
///  Whether the process `pid` still exists.
/// Why:
///   A child that ended is a fact the kernel answers without a signal reaching anyone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alive = (pid) => { try { process.kill(pid, 0); return true; } catch { return false; } };
/// ```
fn alive(pid: i32) -> bool {
    return unsafe { libc::kill(pid, 0) } == 0;
}

/// The fake Git program of the child control:
/// an `ls-tree` run writes its own process id and then sleeps out any add,
/// and every other command is the fixture's real Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const script = `#!/bin/sh\nfor arg in "$@"; do if [ "$arg" = ls-tree ]; then printf '%s' "$$" > '${marker}'; exec sleep 30; fi; done\nexec /usr/bin/git "$@"\n`;
/// ```
fn a_git_whose_size_read_holds(wrote: &Path) -> String {
    // The id goes to a temporary name and is renamed into place, so a reader never sees the file before it holds
    //  a whole process id (the integration recheck's Medium 2 of 2026-11-22).
    return format!(
        "#!/bin/sh\nfor arg in \"$@\"; do\n  if [ \"$arg\" = \"ls-tree\" ]; then\n    printf '%s' \"$$\" > '{wrote}.tmp'\n    mv '{wrote}.tmp' '{wrote}'\n    exec sleep 30\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        wrote = wrote.display()
    );
}


/// Wait,
/// bounded,
/// until `path` holds a process id,
/// then answer it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pidFrom = async (path) => { while (!exists(path) && Date.now() < deadline) await sleep(5); return Number(readFile(path)); };
/// ```
fn a_process_id_written_within(path: &Path) -> i32 {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Some(pid) = a_process_id_in(path) {
            return pid;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the fake Git wrote a process id within 5 s");
}

/// What:
///  The process id `path` holds,
///  or nothing while the file is absent or holds no whole id.
/// Why:
///   The marker is written by another process,
///  and a reader that takes the file's existence for its content can read an empty file;
///  a whole,
/// parseable id is the only thing either side of the marker may act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const processIdIn = (path) => { try { const text = readFileSync(path, 'utf8').trim(); return /^\d+$/.test(text) ? Number(text) : undefined; } catch { return undefined; } };
/// ```
fn a_process_id_in(path: &Path) -> Option<i32> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return text.trim().parse().ok();
}

/// Wait,
/// bounded,
/// for `pid` to be gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const goneWithin = async (pid, ms) => { const deadline = Date.now() + ms; while (alive(pid) && Date.now() < deadline) await sleep(10); return !alive(pid); };
/// ```
fn gone_within(pid: i32, within: Duration) -> bool {
    let deadline = std::time::Instant::now() + within;
    while alive(pid) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    return !alive(pid);
}

/// What:
///  A context whose real Git is the fake that holds an `ls-tree` for 30 s.
/// Why:
///   The read's child is the thing an abandon must end,
/// and no fixture Git holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdingContext = (fake) => ({ ...gitContext(), realGit: fake });
/// ```
fn a_context_whose_git_holds_the_read(fake: &Path) -> GitContext {
    let mut context: GitContext = git_context();
    context.real_git = fake.to_path_buf();
    return context;
}

/// A read the add abandons ends its child while the adding process still lives,
/// and the line its child start left in the repository's record is swept once the child is gone.
#[test]
fn an_abandoned_read_ends_its_child_and_its_record_is_swept() {
    let root: PathBuf = fixture("abandon");
    let repo: PathBuf = a_clean_source(root.as_path(), "abandon");
    // The repository's per-process record names every Git this process starts, so arm it before the read.
    crate::lock_holder_records::remember_repository(repo.join(".git").as_path());
    let fake: PathBuf = root.join("git-that-holds-the-size-read");
    let wrote: PathBuf = root.join("size-read-started");
    std::fs::write(fake.as_path(), a_git_whose_size_read_holds(wrote.as_path())).expect("the fake Git");
    std::fs::set_permissions(fake.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("an executable");
    // The positive control: this test starts the fake itself, and the probe finds it alive.
    let mut held = std::process::Command::new(fake.as_path()).args(["ls-tree", "HEAD"]).spawn().expect("the fake starts");
    let positive: i32 = a_process_id_written_within(wrote.as_path());
    assert!(alive(positive), "the probe finds a running fake Git, so the search can see one");
    let _killed = held.kill();
    let _reaped = held.wait();
    std::fs::remove_file(wrote.as_path()).expect("the positive control's marker is gone");
    let started: SizeReadBesideTheCheck = start_beside_the_check(&a_context_whose_git_holds_the_read(fake.as_path()), &main_identity(repo.as_path())).expect("a read starts");
    let child: i32 = a_process_id_written_within(wrote.as_path());
    assert!(alive(child), "the read's child runs before the abandon");
    started.abandon();
    assert!(gone_within(child, Duration::from_secs(5)), "the abandon ended the read's child while the adding process still lives");
    let common: PathBuf = repo.join(".git");
    let record: String = the_records(&common);
    assert!(record.contains(format!("child {child} ").as_str()), "the child start is named in the repository's record: {record}");
    // The sweep is the pass every command that may meet a lock runs; a record it can no longer explain goes.
    crate::lock_holder_record_reads::sweep(common.as_path(), &[]);
    assert!(!the_record_folder_exists(&common), "the line a killed read's child left is swept once every holder it names is gone");
    remove(root.as_path());
}

/// What:
///  Every record the repository's holder folder holds,
///  as one text.
/// Why:
///  A record names the process and every Git it started;
///  a control reads them to see the line a child start wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = (common) => readdirSync(folderOf(common)).map((name) => readFileSync(join(folderOf(common), name), 'utf8')).join('');
/// ```
fn the_records(common: &Path) -> String {
    let folder: PathBuf = crate::lock_holder_records::folder_of(common);
    let entries = std::fs::read_dir(folder.as_path()).expect("the holder folder");
    let mut text: String = String::new();
    for entry in entries.flatten() {
        text.push_str(std::fs::read_to_string(entry.path()).expect("a record").as_str());
    }
    return text;
}

/// Whether the repository's holder folder still holds any record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = (common) => existsSync(folderOf(common)) && readdirSync(folderOf(common)).length > 0;
/// ```
fn the_record_folder_exists(common: &Path) -> bool {
    let folder: PathBuf = crate::lock_holder_records::folder_of(common);
    return std::fs::read_dir(folder.as_path()).is_ok_and(|entries| return entries.flatten().next().is_some());
}

/// What:
///  A process that starts the read and leaves while its child still runs.
/// Why:
///   A forked process is one process exit without starting this binary again,
///  which the wrapper's no-self-invocation rule forbids.
///  The parent starts no thread before the fork,
/// so the child has one thread when it forks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = fork(); if (pid === 0) { startRead(); waitMarker(); _exit(0); }
/// ```
fn fork_a_process_that_ends_with_the_read(fake: &Path, repo: &Path, marker: &Path) -> std::process::ExitStatus {
    // SAFETY: the parent has one thread here (nextest runs one test per process, and no thread is started
    // before this call), so the child's inherited state is consistent; it starts a read and exits with `_exit`,
    // which runs no destructor and leaves the read's thread to the process exit.
    let pid: libc::pid_t = unsafe { libc::fork() };
    assert!(pid >= 0, "the fork starts a process");
    if pid == 0 {
        let context: GitContext = a_context_whose_git_holds_the_read(fake);
        let started = start_beside_the_check(&context, &main_identity(repo));
        if started.is_none() {
            unsafe { libc::_exit(1) };
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if a_process_id_in(marker).is_some() {
                unsafe { libc::_exit(0) };
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        unsafe { libc::_exit(2) };
    }
    let mut status: libc::c_int = 0;
    // SAFETY: `pid` is the child this call just forked, and `status` is one `c_int` it writes.
    let waited: libc::pid_t = unsafe { libc::waitpid(pid, &mut status, 0) };
    assert_eq!(waited, pid, "the child is the one just forked");
    return std::os::unix::process::ExitStatusExt::from_raw(status);
}

/// A process that ends while its early read still runs leaves no child behind,
/// does not wait for one,
/// and its exit is not delayed by one.
#[test]
fn a_process_that_ends_with_a_running_read_leaves_no_child_behind() {
    let root: PathBuf = fixture("process-exit");
    let repo: PathBuf = a_clean_source(root.as_path(), "process-exit");
    let fake: PathBuf = root.join("git-that-holds-the-size-read");
    let wrote: PathBuf = root.join("size-read-started");
    std::fs::write(fake.as_path(), a_git_whose_size_read_holds(wrote.as_path())).expect("the fake Git");
    std::fs::set_permissions(fake.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("an executable");
    let began = std::time::Instant::now();
    let status = fork_a_process_that_ends_with_the_read(fake.as_path(), repo.as_path(), wrote.as_path());
    let elapsed: Duration = began.elapsed();
    assert!(status.success(), "the add's own process ended after its read started");
    assert!(elapsed < Duration::from_secs(20), "the process does not wait for a read that sleeps for 30 s: {elapsed:?}");
    let child: i32 = a_process_id_written_within(wrote.as_path());
    assert!(gone_within(child, Duration::from_secs(5)), "the read's child ended with the process that started it, not after its 30 s sleep");
    remove(root.as_path());
}

/// The fake Git program of the no-reused-id control:
/// an `ls-tree` run writes its own process id and exits at once,
/// and every other command is the fixture's real Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const script = `#!/bin/sh\nfor arg in "$@"; do if [ "$arg" = ls-tree ]; then printf '%s' "$$" > '${wrote}.tmp'; mv '${wrote}.tmp' '${wrote}'; exit 0; fi; done\nexec /usr/bin/git "$@"\n`;
/// ```
fn a_git_whose_size_read_ends_at_once(wrote: &Path) -> String {
    return format!(
        "#!/bin/sh\nfor arg in \"$@\"; do\n  if [ \"$arg\" = \"ls-tree\" ]; then\n    printf '%s' \"$$\" > '{wrote}.tmp'\n    mv '{wrote}.tmp' '{wrote}'\n    exit 0\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
        wrote = wrote.display()
    );
}

/// An abandon that comes before the read's thread reached its spawn does not leave a child running its whole length.
#[test]
fn an_abandon_before_the_child_is_published_leaves_no_child_running() {
    let root: PathBuf = fixture("abandon-before-child");
    let repo: PathBuf = a_clean_source(root.as_path(), "abandon-before-child");
    let fake: PathBuf = root.join("git-that-holds-the-size-read");
    let wrote: PathBuf = root.join("size-read-started");
    std::fs::write(fake.as_path(), a_git_whose_size_read_holds(wrote.as_path())).expect("the fake Git");
    std::fs::set_permissions(fake.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("an executable");
    let hold = a_control_holds_the_read();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&a_context_whose_git_holds_the_read(fake.as_path()), &main_identity(repo.as_path())).expect("a read starts");
    // The read is held before its Git run, so no child exists yet: this is exactly the abandon a check that refuses at once makes.
    started.abandon();
    drop(hold);
    // The read may start its child after the abandon; that child must be ended the moment it is published.
    // The marker is waited for, bounded, because the read's thread starts the fake after this thread released it:
    //  a check that looked once could miss a child that the old code then left for its whole 30 s sleep.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut child: Option<i32> = None;
    while child.is_none() && std::time::Instant::now() < deadline {
        child = a_process_id_in(wrote.as_path());
        if child.is_none() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    if let Some(child) = child {
        assert!(gone_within(child, Duration::from_secs(5)), "the child the abandon came before is ended at publication, not left for its 30 s sleep");
    }
    // Nothing of the fake may still run, whether its marker was seen or the kill landed before it wrote one.
    let still = a_process_id_in(wrote.as_path());
    if let Some(still) = still {
        assert!(gone_within(still, Duration::from_secs(1)), "nothing of the abandoned read is alive afterwards");
    }
    remove(root.as_path());
}

/// A child reaped before an abandon is not named by the read's cell,
/// so the abandon signals no reused process id.
#[test]
fn an_abandon_after_the_child_ended_names_no_child() {
    let root: PathBuf = fixture("no-reused-id");
    let repo: PathBuf = a_clean_source(root.as_path(), "no-reused-id");
    let fake: PathBuf = root.join("git-whose-size-read-ends");
    let wrote: PathBuf = root.join("size-read-started");
    std::fs::write(fake.as_path(), a_git_whose_size_read_ends_at_once(wrote.as_path())).expect("the fake Git");
    std::fs::set_permissions(fake.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("an executable");
    let hold = a_control_holds_the_sizes_after_the_wait();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&a_context_whose_git_holds_the_read(fake.as_path()), &main_identity(repo.as_path())).expect("a read starts");
    let child: i32 = a_process_id_written_within(wrote.as_path());
    assert!(gone_within(child, Duration::from_secs(5)), "the fake Git ends by itself");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !the_sizes_hold_was_reached() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(the_sizes_hold_was_reached(), "the read reached the seam after its wait");
    assert_eq!(started.published_child_for_a_control(), 0, "the cell names no reaped child, so an abandon here signals nothing");
    drop(hold);
    drop(started);
    remove(root.as_path());
}

/// A fence with no path to size ends the read it started rather than leaving it to run to the process's end.
#[test]
fn the_fence_ends_a_read_no_path_needs() {
    let root: PathBuf = fixture("no-sizes");
    let repo: PathBuf = a_clean_source(root.as_path(), "no-sizes");
    // Every tracked path converts to CRLF on checkout, so no path needs a blob size at all.
    git(repo.as_path(), &["config", "core.autocrlf", "true"]);
    let fake: PathBuf = root.join("git-that-holds-the-size-read");
    let wrote: PathBuf = root.join("size-read-started");
    std::fs::write(fake.as_path(), a_git_whose_size_read_holds(wrote.as_path())).expect("the fake Git");
    std::fs::set_permissions(fake.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("an executable");
    let context: GitContext = a_context_whose_git_holds_the_read(fake.as_path());
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo.as_path())).expect("a read starts");
    let child: i32 = a_process_id_written_within(wrote.as_path());
    assert!(alive(child), "the read's child runs before the fence");
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    begin_timing(repo.join(".git"), true);
    let fence: Fence = the_fence(repo.as_path(), &context, &mut held).expect("the fence decides without sizes");
    finish_timing(0);
    assert!(held.is_none(), "the fence ended the read no path needs");
    assert!(gone_within(child, Duration::from_secs(5)), "the child is ended where the fence decides it has no use for it");
    let record: String = the_record(repo.as_path());
    assert!(!the_record_states(record.as_str(), "git ls-tree"), "the fence read no sizes: {record}");
    // Nothing needed a size: every path converts to CRLF, so the fence hands each one to Git's writing.
    assert!(!fence.git_writes.is_empty(), "a path that converts to CRLF is written by Git");
    remove(root.as_path());
}

/// A refused thread start loses only the overlap:
/// the fence reads where it stands.
#[test]
fn a_refused_thread_start_reads_where_it_stands() {
    let root: PathBuf = fixture("refused-start");
    let repo: PathBuf = a_clean_source(root.as_path(), "refused-start");
    let context: GitContext = git_context();
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let refused = crate::scoped_start::RefusedStarts::here();
    let started: Option<SizeReadBesideTheCheck> = start_beside_the_check(&context, &identity);
    assert!(started.is_none(), "a start the system refuses answers no read");
    let mut held: Option<SizeReadBesideTheCheck> = started;
    begin_timing(repo.join(".git"), true);
    let fence: Fence = the_fence(repo.as_path(), &context, &mut held).expect("the fence reads where it stands");
    finish_timing(0);
    drop(refused);
    assert!(held.is_none(), "no read is left for the add to settle");
    let record: String = the_record(repo.as_path());
    assert!(the_record_states(record.as_str(), "git ls-tree"), "the fence read the sizes itself: {record}");
    assert!(!the_record_states(record.as_str(), "fenceSizesWaitingForTheRead"), "no join was entered: {record}");
    let source: SnapshotSource = source_refusal(&context, &main_identity(repo.as_path())).expect("the source is clean");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried), "the refused start decides the same paths");
    remove(root.as_path());
}

/// What:
///  The variable that makes the multi-thread runtime's build miss its deadline.
/// Why:
///   It is the shipped seam for the current-thread stand-in,
/// and a control sets it before the first need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS = '1500';
/// ```
const THE_SLOW_RUNTIME_BUILD_OF_THE_CONTROL: &str = "CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS";

/// The early read and the fence complete on the current-thread runtime stand-in.
#[test]
fn a_read_and_the_fence_complete_on_the_current_thread_stand_in() {
    let root: PathBuf = fixture("stand-in");
    let repo: PathBuf = a_clean_source(root.as_path(), "stand-in");
    // SAFETY: nextest gives this control its own process, and this runs before the runtime is built or any
    // thread of the control starts, so no other reader of the environment exists yet.
    unsafe { std::env::set_var(THE_SLOW_RUNTIME_BUILD_OF_THE_CONTROL, "1500") };
    let context: GitContext = git_context();
    let identity: WorktreeIdentity = main_identity(repo.as_path());
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &identity).expect("a read starts");
    let source: SnapshotSource = source_refusal(&context, &identity).expect("the source is clean");
    // The read is handed to the fence, which joins it here: the read's Git run and the fence's own run are then
    //  concurrent on the stand-in, which is the shape the earlier control never reached by joining first.
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = fence_the_clone_with_the_read_beside_the_check(&source, &context, &mut held).expect("the fence decides under the stand-in");
    assert!(held.is_none(), "the fence took the read and joined it under the stand-in");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried), "the stand-in decides the same paths");
    assert_eq!(crate::lazy_runtime::built_flavor(), Some(crate::lazy_runtime::BuiltFlavor::CurrentThreadAfterDeadline), "the process ran on the stand-in");
    remove(root.as_path());
}
