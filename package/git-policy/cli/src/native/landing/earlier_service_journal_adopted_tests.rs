//! Controls for the adoption of the batches an earlier release's service left open: where the earlier journals are, when a
//!  service of the worktree may still be running, and that an open batch is finished in place and never twice.
//! The controls sit beside the landing controls because they land real batches with the same fixtures.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_land::Handled;
use crate::earlier_service_journal_adopted::{EarlierJournals, adopt_open_batches, earlier_commits_landed, earlier_journals_of, service_of_instance_may_run};
use crate::service_instance::instance_of;
use crate::service_index_lock::try_acquire;
use crate::service_journal::{DONE_EXTENSION, INDEXED_EXTENSION, OPEN_EXTENSION, batch_path, list_intents, mark_indexed, parse_intent, write_intent};
use crate::test_support::{git_text, remove};
use crate::wrapper_names_old_and_new::DATA_FOLDER;

/// The state folder name of an awkward test: a space, quotes and a newline.
const AWKWARD_STATE: &str = "state with 'quotes' and \"more\"\nand a break";

/// The instance of the world's worktree.
fn instance_for(world: &World) -> String {
    return instance_of(world.repo.join(".git").as_path());
}

/// The journal the unit of this release gives the service of the world's worktree, below `state`.
fn new_journal(state: &Path, instance: &str) -> PathBuf {
    return state.join(DATA_FOLDER.current).join("service").join(instance);
}

/// The context of this release's service: the world's Git and a journal of the new place.
fn new_context(world: &World, state: &Path) -> LandContext {
    let journal: PathBuf = new_journal(state, instance_for(world).as_str());
    std::fs::create_dir_all(journal.as_path()).expect("the new journal");
    return LandContext { git: world.context.git.clone(), journal, on_objects_written: None, lock_ceiling: world.context.lock_ceiling };
}

/// The world with its journal in the place the earlier unit gave it, below `state`.
fn world_with_earlier_journal(name: &str, state_name: &str) -> (World, PathBuf) {
    let mut world: World = world(name);
    let state: PathBuf = world.root.join(state_name);
    let earlier: PathBuf = state.join(DATA_FOLDER.earlier).join("service").join(instance_for(&world));
    std::fs::create_dir_all(earlier.as_path()).expect("the earlier journal");
    world.context.journal = earlier;
    return (world, state);
}

/// A proc root that lists no process.
fn empty_proc(world: &World) -> PathBuf {
    let proc_root: PathBuf = world.root.join("proc-empty");
    std::fs::create_dir_all(proc_root.as_path()).expect("proc root");
    return proc_root;
}

/// Writes a fake process with the given command line words into `proc_root`.
fn fake_process(proc_root: &Path, pid: u32, words: &[&str]) {
    let directory: PathBuf = proc_root.join(pid.to_string());
    std::fs::create_dir_all(directory.as_path()).expect("process directory");
    let mut bytes: Vec<u8> = Vec::new();
    for word in words {
        bytes.extend_from_slice(word.as_bytes());
        bytes.push(0);
    }
    std::fs::write(directory.join("cmdline"), bytes).expect("command line");
}

/// The earlier journals are named from the place the unit gives the new one, and from nothing else.
#[test]
fn the_earlier_journals_follow_the_place_of_the_new_journal() {
    let state: PathBuf = PathBuf::from("/home/u/.local/state");
    let found: Option<EarlierJournals> = earlier_journals_of(new_journal(state.as_path(), "abc").as_path(), "abc");
    assert_eq!(
        found,
        Some(EarlierJournals { per_instance: state.join("cli-git/service/abc"), shared: state.join("cli-git/service") })
    );
    for elsewhere in ["/x/journal", "/x/monochromatic-git-policy-cli/service/other", "/x/monochromatic-git-policy-cli/elsewhere/abc", "/x/cli-git/service/abc", "abc"] {
        assert_eq!(earlier_journals_of(Path::new(elsewhere), "abc"), None, "{elsewhere}");
    }
}

/// A process with the service words and the instance option of the worktree means a service may be running;
///  another instance, this process itself, an unrelated command line and a process that vanished do not;
///  a proc root that cannot be listed means it may.
#[test]
fn a_service_of_the_worktree_is_found_among_the_processes_and_nothing_else_is() {
    let world: World = world("adopt-proc");
    let proc_root: PathBuf = empty_proc(&world);
    let own: u32 = 4242;
    assert!(!service_of_instance_may_run("abc", own, proc_root.as_path()), "no process");
    fake_process(proc_root.as_path(), 10, &["/bin/sh", "-c", "service run --instance abc"]);
    assert!(!service_of_instance_may_run("abc", own, proc_root.as_path()), "one word, not separate words");
    fake_process(proc_root.as_path(), 11, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", "/j", "--instance", "other"]);
    assert!(!service_of_instance_may_run("abc", own, proc_root.as_path()), "another worktree's service");
    std::fs::create_dir_all(proc_root.join("12")).expect("a process that vanished");
    std::fs::create_dir_all(proc_root.join("self")).expect("not a number");
    fake_process(proc_root.as_path(), own, &["/p/monochromatic-git-policy-cli", "monochromatic-git-policy-cli", "service", "run", "--instance", "abc"]);
    assert!(!service_of_instance_may_run("abc", own, proc_root.as_path()), "this process is not another");
    fake_process(proc_root.as_path(), 13, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", "/j", "--instance", "abc"]);
    assert!(service_of_instance_may_run("abc", own, proc_root.as_path()), "an earlier release's service of the worktree");
    assert!(service_of_instance_may_run("abc", own, world.root.join("no-such-proc").as_path()), "an unlistable root is unproven");
    remove(world.root.as_path());
}

/// The case the owner named: the earlier helper was killed after the branch moved and before the index update, and only the new
///  program runs afterwards. Without the adoption the batch stays open for ever and the index shows the commit as reverted;
///  with it, the new service finishes the batch in the earlier journal.
#[test]
fn an_open_batch_of_the_earlier_helper_is_finished_by_the_new_service() {
    for state_name in ["state", AWKWARD_STATE] {
        let (world, state) = world_with_earlier_journal("adopt-open", state_name);
        let landed: CommitRequest = request(&world, "adopted", "adopted.txt", "adopted\n");
        land_with_failed_index(&world, 1, &[landed]);
        assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "the index shows the landed file reverted");
        let context: LandContext = new_context(&world, state.as_path());
        let reports: Vec<RecoveryReport> = adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path());
        assert_eq!(reports.iter().map(|report| { return (report.seq, report.decision); }).collect::<Vec<_>>(), vec![(1, crate::service_journal::Recovery::Finish)]);
        assert!(reports.iter().all(|report| { return report.handled == Handled::Finished; }), "{reports:?}");
        assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
        assert!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).is_empty(), "nothing is left open");
        assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "the batch is remembered as finished where it was");
        assert!(adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path()).is_empty(), "a second adoption finds nothing");
        remove(world.root.as_path());
    }
}

/// An open batch of the single service of the first releases, in the journal above every instance's own, is adopted too.
#[test]
fn an_open_batch_in_the_shared_earlier_journal_is_finished_too() {
    let (mut world, state) = world_with_earlier_journal("adopt-shared", "state");
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join("service");
    world.context.journal = shared.clone();
    let landed: CommitRequest = request(&world, "shared", "shared.txt", "shared\n");
    land_with_failed_index(&world, 1, &[landed]);
    let context: LandContext = new_context(&world, state.as_path());
    let reports: Vec<RecoveryReport> = adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path());
    assert_eq!(reports.len(), 1, "{reports:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "");
    assert!(list_intents(shared.as_path(), OPEN_EXTENSION).is_empty());
    remove(world.root.as_path());
}

/// A service of the worktree that may be running finishes its own batches, and two services never settle one batch:
///  the adoption leaves the open batch exactly as it is, and takes it once the service is gone.
#[test]
fn an_open_batch_is_left_alone_while_a_service_of_the_worktree_may_run() {
    let (world, state) = world_with_earlier_journal("adopt-running", "state");
    let landed: CommitRequest = request(&world, "running", "running.txt", "running\n");
    land_with_failed_index(&world, 1, &[landed]);
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 77, &["/p/cli-git-native", "cli-git", "service", "run", "--instance", instance.as_str()]);
    assert!(adopt_open_batches(&context, instance.as_str(), proc_root.as_path()).is_empty());
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 1, "still open");
    std::fs::remove_dir_all(proc_root.join("77")).expect("the service ended");
    assert_eq!(adopt_open_batches(&context, instance.as_str(), proc_root.as_path()).len(), 1, "taken once the service is gone");
    remove(world.root.as_path());
}

/// A worktree that has no earlier journal, and a journal of a development run in any other place, adopt nothing and create nothing.
#[test]
fn no_earlier_journal_means_nothing_is_adopted_or_made() {
    let world: World = world("adopt-none");
    let state: PathBuf = world.root.join("state");
    let context: LandContext = new_context(&world, state.as_path());
    assert!(adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path()).is_empty());
    assert!(!state.join(DATA_FOLDER.earlier).exists());
    assert!(adopt_open_batches(&world.context, instance_for(&world).as_str(), empty_proc(&world).as_path()).is_empty(), "a journal in any other place");
    remove(world.root.as_path());
}

/// A client that was not answered by the earlier helper sends the same request again to the service of this release. The
///  service remembers a landed request by its identity so it answers and lands nothing twice; the memory of the earlier
///  helper's finished batches is in its journal, and the service of this release reads it too.
#[test]
fn the_requests_the_earlier_helper_landed_are_remembered_by_the_new_service() {
    let (world, state) = world_with_earlier_journal("adopt-remembered", "state");
    let landed: CommitRequest = request(&world, "remembered", "remembered.txt", "remembered\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    let remembered = earlier_commits_landed(context.journal.as_path(), instance.as_str());
    assert_eq!(remembered.get("remembered"), Some(&git_text(world.repo.as_path(), &["rev-parse", "HEAD"])));
    assert!(earlier_commits_landed(world.context.journal.as_path(), instance.as_str()).is_empty(), "a journal in any other place has no earlier ones");
    remove(world.root.as_path());
}

/// A live landing of the earlier helper writes its intent before it takes the index lock and moves the branch, so for a while the
///  branch is where the intent says it was and the recovery's answer is "never landed". The adoption must not act on that
///  answer: removing the intent of a landing that goes on leaves a landed commit with no record, and the person's index shows it
///  reverted if the helper dies before it installs the index. Such an intent stays on disk, with its records, for the earlier
///  helper's own start-up recovery.
#[test]
fn the_intent_of_a_landing_that_has_not_moved_the_branch_yet_is_left_on_disk() {
    let (world, state) = world_with_earlier_journal("adopt-mid-landing", "state");
    let landed: CommitRequest = request(&world, "mid", "mid.txt", "mid\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    // The landing of batch 2 wrote its intent and waits for the lock: the branch is still at the commit it was based on.
    intent.seq = 2;
    intent.old = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    intent.new = "f".repeat(40);
    write_intent(world.context.journal.as_path(), &intent).expect("the intent of a landing in progress");
    mark_indexed(world.context.journal.as_path(), 2, intent.new.as_str()).expect("a record the landing wrote");
    let held = try_acquire(intent.index.as_path(), 2).expect("the landing's index lock");
    let context: LandContext = new_context(&world, state.as_path());
    let reports: Vec<RecoveryReport> = adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path());
    assert!(batch_path(world.context.journal.as_path(), 2, OPEN_EXTENSION).exists(), "the intent stays: {reports:?}");
    assert!(reports.contains(&RecoveryReport { seq: 2, decision: crate::service_journal::Recovery::Drop, handled: Handled::Left }), "and it is reported as left: {reports:?}");
    assert!(batch_path(world.context.journal.as_path(), 2, INDEXED_EXTENSION).exists(), "its record stays");
    drop(held);
    remove(world.root.as_path());
}

/// A landing that moved the branch and still holds the index lock is not finished by the adoption either: the batch stays open
///  for the helper that holds the lock, and a second adoption after the lock is gone finishes it.
#[test]
fn a_landing_that_holds_the_index_lock_is_not_finished_by_the_adoption() {
    let (world, state) = world_with_earlier_journal("adopt-lock-held", "state");
    let landed: CommitRequest = request(&world, "held", "held.txt", "held\n");
    land_with_failed_index(&world, 1, &[landed]);
    let intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    let held = try_acquire(intent.index.as_path(), 1).expect("the landing's index lock");
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    adopt_open_batches(&context, instance.as_str(), empty_proc(&world).as_path());
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 1, "still open while the lock is held");
    drop(held);
    assert_eq!(adopt_open_batches(&context, instance.as_str(), empty_proc(&world).as_path()).len(), 1);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "");
    remove(world.root.as_path());
}
