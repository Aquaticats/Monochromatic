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

/// A loose ref appears where only `packed-refs` held the branch.
fn the_loose_ref_appears_over_the_packed_one(repo: &Path) {
    git(repo, &["pack-refs", "--all"]);
    commit_of_the_same_tree(repo);
}

/// A detached `HEAD` moves to a commit of the same tree.
fn the_detached_head_moves(repo: &Path) {
    git(repo, &["switch", "--quiet", "--detach"]);
    commit_of_the_same_tree(repo);
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
fn the_stale_read_is_dropped(repo: &Path, move_head: MoveHead) {
    let context: GitContext = git_context();
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &main_identity(repo)).expect("a read starts");
    move_head(repo);
    begin_timing(repo.join(".git"), true);
    let mut held: Option<SizeReadBesideTheCheck> = Some(started);
    let fence: Fence = the_fence(repo, &context, &mut held).expect("the fence decides the proved commit");
    finish_timing(0);
    assert!(held.is_some(), "the fence left the read of the earlier commit unused");
    let record: String = the_record(repo);
    assert!(the_record_states(record.as_str(), "git ls-tree"), "the fence read the proved commit itself: {record}");
    let source: SnapshotSource = source_refusal(&context, &main_identity(repo)).expect("the source is clean");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried), "the proved commit's own map decides the paths");
    held.expect("the add holds the read it never used").settle();
}

/// The symbolic ref moving to another commit of the same tree drops the read.
#[test]
fn a_moved_symbolic_ref_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-symbolic");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), the_symbolic_ref_moves);
    remove(root.as_path());
}

/// A loose ref appearing over the branch `packed-refs` held drops the read.
#[test]
fn a_packed_ref_overlaid_by_a_loose_one_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-packed");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), the_loose_ref_appears_over_the_packed_one);
    remove(root.as_path());
}

/// A moving detached `HEAD` drops the read.
#[test]
fn a_detached_head_moving_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-detached");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), the_detached_head_moves);
    remove(root.as_path());
}

/// `HEAD` moving to a commit of another tree drops the read,
/// and the tree the check proved decides the paths.
#[test]
fn a_head_moved_to_another_tree_drops_the_read_beside_the_check() {
    let root: PathBuf = fixture("moved-tree");
    let repo: PathBuf = a_clean_source(root.as_path(), "moved");
    the_stale_read_is_dropped(repo.as_path(), the_head_moves_to_another_tree);
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
    assert!(held.is_some(), "the read of the commit before the move is unused");
    let serial: Fence = fence_the_clone(&source, &context).expect("the serial fence decides");
    assert_eq!((fence.git_writes, fence.carried), (serial.git_writes, serial.carried));
    held.expect("the read is settled by the add's end").settle();
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

/// The early read's request is the only thing that differs from the serial one.
#[test]
fn the_early_read_runs_without_lazy_fetch_and_outside_the_note() {
    let request: GitRequest = the_early_read_request(Path::new("/nowhere"), "HEAD");
    assert!(request.beside_another_step, "the read leaves the failed-step note alone");
    assert!(request.environment.contains(&(OsString::from("GIT_NO_LAZY_FETCH"), OsString::from("1"))), "{:?}", request.environment);
    assert_eq!(request.arguments, vec![OsString::from("ls-tree"), OsString::from("-r"), OsString::from("-l"), OsString::from("-z"), OsString::from("HEAD")]);
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

/// An add that never reaches the fence states the read's cost and writes nothing.
#[test]
fn an_add_that_never_reaches_the_fence_states_the_reads_cost() {
    let root: PathBuf = fixture("settled-only");
    let repo: PathBuf = a_clean_source(root.as_path(), "settled-only");
    let destination: PathBuf = root.join("topic");
    let started: SizeReadBesideTheCheck = start_beside_the_check(&git_context(), &main_identity(repo.as_path())).expect("a read starts");
    begin_timing(repo.join(".git"), true);
    let ladder: Ladder = Ladder { plan: None, rung: Rung::Git(None), slow: None, sizes_beside: Some(started) };
    let mut runner = no_runner();
    let result = ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    finish_timing(0);
    assert_eq!(result.git, GitStatus { code: Some(0) });
    let record: String = the_record(repo.as_path());
    assert!(the_record_states(record.as_str(), "fenceSizesBesideTheCheck"), "the cost the add paid is stated: {record}");
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

/// A sparse partial clone the check refuses fetches nothing,
/// and the read without the variable is the positive control that its count can move.
#[test]
fn a_read_beside_the_check_writes_nothing_into_a_sparse_partial_clone_the_check_refuses() {
    let root: PathBuf = fixture("partial");
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
    let context: GitContext = git_context();
    let refused: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "refused");
    let before: (usize, usize) = missing_and_packs(refused.as_path());
    assert!(before.0 > 0, "the clone is missing the blobs the cone leaves out: {before:?}");
    let identity: WorktreeIdentity = main_identity(refused.as_path());
    let started: SizeReadBesideTheCheck = start_beside_the_check(&context, &identity).expect("a read starts before the check");
    let refusal = source_refusal(&context, &identity).expect_err("the check refuses the sparse checkout");
    assert!(refusal.reason.contains("sparse checkout"), "{}", refusal.reason);
    started.settle();
    assert_eq!(missing_and_packs(refused.as_path()), before, "the read fetched nothing into the source the check refused");
    // The positive control: the same read without GIT_NO_LAZY_FETCH fetches, so the count above is one that can move.
    let fetched: PathBuf = a_sparse_partial_clone(origin.as_path(), root.as_path(), "fetched");
    let before_the_serial_read: (usize, usize) = missing_and_packs(fetched.as_path());
    let _serial: BlobSizes = blob_sizes(&context, fetched.as_path(), the_head_commit(fetched.as_path()).as_str()).expect("the serial read");
    assert_ne!(missing_and_packs(fetched.as_path()), before_the_serial_read, "the serial read fetches, so the pair can move");
    remove(root.as_path());
}
