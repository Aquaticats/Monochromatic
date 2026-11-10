//! Controls for the adoption of the batches an earlier release's service left open:
//! where the earlier journals are,
//! when a
//!  service of the worktree may still be running,
//! and that an open batch is finished in place and never twice.
//! The controls sit beside the landing controls because they land real batches with the same fixtures.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_land::Handled;
use std::time::Duration;
use crate::earlier_service_journal_adopted::{EarlierJournals, RequestedSettlement, earlier_commits_landed, earlier_journals_of, settle_on_request};
use crate::earlier_service_process_look::{ServiceLook, effective_user, service_of_instance_may_run};
use crate::service_instance::instance_of;
use crate::service_index_lock::try_acquire;
use crate::service_journal::{DONE_EXTENSION, INDEXED_EXTENSION, OPEN_EXTENSION, batch_path, list_intents, mark_indexed, parse_intent, write_intent};
use crate::test_support::{git_text, remove};
use crate::wrapper_names_old_and_new::DATA_FOLDER;

/// The settling the person asks for,
/// for the journals the earlier unit gives the instance of the new service's journal:
/// the
///  reports of the batches it dealt with,
/// none when it found nothing open or an earlier helper may run.
fn adopt_open_batches(context: &LandContext, instance: &str, proc_root: &Path) -> Vec<RecoveryReport> {
    let Some(earlier) = earlier_journals_of(context.journal.as_path(), instance) else { return Vec::new() };
    return match settle_on_request(&context.git, &earlier, instance, proc_root, context.lock_ceiling) {
        RequestedSettlement::Settled { batches, .. } => batches.into_iter().map(|batch| { return batch.report; }).collect(),
        RequestedSettlement::NothingOpen | RequestedSettlement::Held { .. } => Vec::new(),
    };
}

/// The state folder name of an awkward test:
/// a space,
/// quotes and a newline.
const AWKWARD_STATE: &str = "state with 'quotes' and \"more\"\nand a break";

/// Writes what a landing of this release has written at a moment these controls look at the earlier journals:
/// the intent of a batch,
/// and its record that the index was installed when `indexed` names the commit.
/// The writes are made inside a landing of its own that publishes,
/// as every landing of the service makes them,
/// so each flush is queued for the background thread instead of being waited for on the disk this host shares.
fn write_as_a_landing_in_progress(journal: &Path, intent: &crate::service_journal::Intent, indexed: Option<&str>) {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let deferral = crate::service_durable::begin(false);
        write_intent(journal, intent).expect("the intent of a landing in progress");
        if let Some(new) = indexed {
            mark_indexed(journal, intent.seq, new).expect("a record the landing wrote");
        }
        deferral.settle(true);
    });
}

/// The instance of the world's worktree.
pub(super) fn instance_for(world: &World) -> String {
    return instance_of(world.repo.join(".git").as_path());
}

/// The journal the unit of this release gives the service of the world's worktree,
/// below `state`.
fn new_journal(state: &Path, instance: &str) -> PathBuf {
    return state.join(DATA_FOLDER.current).join("service").join(instance);
}

/// The context of this release's service:
/// the world's Git and a journal of the new place.
fn new_context(world: &World, state: &Path) -> LandContext {
    let journal: PathBuf = new_journal(state, instance_for(world).as_str());
    std::fs::create_dir_all(journal.as_path()).expect("the new journal");
    return LandContext { git: world.context.git.clone(), journal, on_objects_written: None, lock_ceiling: world.context.lock_ceiling };
}

/// The world with its journal in the place the earlier unit gave it,
/// below `state`.
pub(super) fn world_with_earlier_journal(name: &str, state_name: &str) -> (World, PathBuf) {
    let mut world: World = world(name);
    let state: PathBuf = world.root.join(state_name);
    let earlier: PathBuf = state.join(DATA_FOLDER.earlier).join("service").join(instance_for(&world));
    std::fs::create_dir_all(earlier.as_path()).expect("the earlier journal");
    world.context.journal = earlier;
    return (world, state);
}

/// A proc root that lists no process and no socket.
pub(super) fn empty_proc(world: &World) -> PathBuf {
    let proc_root: PathBuf = world.root.join("proc-empty");
    std::fs::create_dir_all(proc_root.as_path()).expect("proc root");
    write_unix_table(proc_root.as_path(), &[]);
    return proc_root;
}

/// Writes a fake process with the given command line words into `proc_root`.
pub(super) fn fake_process(proc_root: &Path, pid: u32, words: &[&str]) {
    let directory: PathBuf = proc_root.join(pid.to_string());
    std::fs::create_dir_all(directory.as_path()).expect("process directory");
    let mut bytes: Vec<u8> = Vec::new();
    for word in words {
        bytes.extend_from_slice(word.as_bytes());
        bytes.push(0);
    }
    std::fs::write(directory.join("cmdline"), bytes).expect("command line");
}

/// The earlier journals are named from the place the unit gives the new one,
/// and from nothing else.
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

/// The look of the instance `abc` whose earlier journals are `/s/abc` and `/s`,
/// run by `own`.
fn look_for(own_pid: u32) -> ServiceLook {
    return ServiceLook {
        instance: String::from("abc"),
        journals: vec![PathBuf::from("/s/abc"), PathBuf::from("/s")],
        own_pid,
        own_user: effective_user(),
        ceiling: Duration::from_secs(30),
    };
}

/// A process with the service words and the instance option of the worktree,
/// or the journal of the worktree's earlier service
///  as its journal,
/// means a service may be running;
/// another instance or journal,
/// this process itself,
/// an unrelated command
///  line and a process that vanished do not;
/// a proc root that cannot be listed means it may.
#[test]
fn a_service_of_the_worktree_is_found_among_the_processes_and_nothing_else_is() {
    let world: World = world("adopt-proc");
    let proc_root: PathBuf = empty_proc(&world);
    let own: u32 = 4242;
    // Journals other than the worktree's earlier ones exist as folders: a journal that cannot be resolved counts as "may be running".
    let other_journal_folder: PathBuf = world.root.join("other-journal");
    let other_instance_journal_folder: PathBuf = world.root.join("shared-of-other/other");
    std::fs::create_dir_all(other_journal_folder.as_path()).expect("another journal");
    std::fs::create_dir_all(other_instance_journal_folder.as_path()).expect("another instance's journal");
    let other_journal: String = other_journal_folder.to_string_lossy().into_owned();
    let other_instance_journal: String = other_instance_journal_folder.to_string_lossy().into_owned();
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "no process");
    fake_process(proc_root.as_path(), 10, &["/bin/sh", "-c", "service run --instance abc"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "one word, not separate words");
    fake_process(proc_root.as_path(), 11, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", other_journal.as_str(), "--instance", "other"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "another worktree's service");
    fake_process(proc_root.as_path(), 14, &["/p/cli-git-native", "--journal", other_instance_journal.as_str(), "--git", "/usr/bin/git", "--instance", "other"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "another worktree's service of the earlier unit shape");
    fake_process(proc_root.as_path(), 15, &["/p/cli-git-native", "/s/abc", "--journal"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "the journal is the word after the option");
    std::fs::create_dir_all(proc_root.join("12")).expect("a process that vanished");
    std::fs::create_dir_all(proc_root.join("self")).expect("not a number");
    fake_process(proc_root.as_path(), own, &["/p/monochromatic-git-policy-cli", "monochromatic-git-policy-cli", "service", "run", "--instance", "abc"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "this process is not another");
    fake_process(proc_root.as_path(), 13, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", "/j", "--instance", "abc"]);
    assert!(service_of_instance_may_run(look_for(own), proc_root.as_path()), "an earlier release's service of the worktree");
    std::fs::remove_dir_all(proc_root.join("13")).expect("ended");
    fake_process(proc_root.as_path(), 16, &["/p/cli-git-native", "--journal", "/s/abc", "--git", "/usr/bin/git", "--instance", "abc"]);
    assert!(service_of_instance_may_run(look_for(own), proc_root.as_path()), "the unit shape before the service mode");
    std::fs::remove_dir_all(proc_root.join("16")).expect("ended");
    fake_process(proc_root.as_path(), 17, &["/p/cli-git-native", "--journal", "/s", "--git", "/usr/bin/git"]);
    assert!(service_of_instance_may_run(look_for(own), proc_root.as_path()), "the shared service of the first releases");
    assert!(service_of_instance_may_run(look_for(own), world.root.join("no-such-proc").as_path()), "an unlistable root is unproven");
    remove(world.root.as_path());
}

/// Only the processes of the user count:
/// a service another user runs for the same worktree is not this user's to wait for.
#[test]
fn a_service_of_another_user_does_not_hold_the_adoption() {
    let world: World = world("adopt-proc-user");
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 41, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", "/j", "--instance", "abc"]);
    let theirs: ServiceLook = ServiceLook { own_user: effective_user() + 1, ..look_for(4242) };
    assert!(!service_of_instance_may_run(theirs, proc_root.as_path()), "the folder of the process belongs to someone else");
    assert!(service_of_instance_may_run(look_for(4242), proc_root.as_path()), "control: the same table, the user's own");
    remove(world.root.as_path());
}

/// Reading a command line can wait for the process for ever;
/// the look ends at its ceiling and answers "may be running".
///  A named pipe in place of the file stands for it:
/// opening succeeds,
/// reading waits.
#[test]
fn a_command_line_that_cannot_be_read_in_time_counts_as_may_be_running() {
    use std::os::unix::ffi::OsStrExt;
    let world: World = world("adopt-proc-stuck");
    let proc_root: PathBuf = empty_proc(&world);
    let process: PathBuf = proc_root.join("51");
    std::fs::create_dir_all(process.as_path()).expect("process directory");
    let pipe: std::ffi::CString = std::ffi::CString::new(process.join("cmdline").as_os_str().as_bytes()).expect("path");
    // SAFETY: `pipe` is a NUL-terminated path that lives through the call.
    assert_eq!(unsafe { libc::mkfifo(pipe.as_ptr(), 0o600) }, 0, "make the pipe");
    // Holding the pipe open for reading and writing lets the look open it without a writer and then wait for bytes.
    let holder = std::fs::OpenOptions::new().read(true).write(true).open(process.join("cmdline")).expect("hold the pipe");
    let short: ServiceLook = ServiceLook { ceiling: Duration::from_millis(300), ..look_for(4242) };
    let began: std::time::Instant = std::time::Instant::now();
    assert!(service_of_instance_may_run(short, proc_root.as_path()));
    assert!(began.elapsed() < Duration::from_secs(20), "it came back at its ceiling, not when the read ended");
    drop(holder);
    remove(world.root.as_path());
}

/// The batch these controls leave open is landed the way the service lands one:
/// it waits for no flush of a record it writes,
/// because each is queued for the background thread.
/// Why:
///  One flush of this repository on a host other sessions fill takes seconds to tens of seconds
/// (measured 2026-11-09,
/// 8 flushes of one setup landing took 65 s of a 79.6 s control),
///  so a control that waits for them passes alone and reaches the runner's bound inside a larger run.
/// The setup landing is the subject here;
/// the adoption that finishes the batch is the product's own path and is checked by the controls above.
#[test]
fn the_batch_these_controls_leave_open_is_landed_without_waiting_for_a_flush() {
    let (world, _state) = world_with_earlier_journal("adopt-quiet-landing", "state");
    crate::service_fault::hooks::take_events();
    let landed: CommitRequest = request(&world, "quiet", "quiet.txt", "quiet\n");
    land_with_failed_index(&world, 1, &[landed]);
    let events: Vec<String> = crate::service_fault::hooks::take_events();
    let waited: Vec<&String> = events.iter().filter(|event| { return event.starts_with("sync-waited:"); }).collect();
    assert!(waited.is_empty(), "the setup landing waited for a flush: {waited:?}");
    assert!(events.iter().any(|event| { return event.starts_with("sync-queued:"); }), "positive control: the landing queues its records: {events:?}");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// The case the owner named:
/// the earlier helper was killed after the branch moved and before the index update,
/// and only the new
///  program runs afterwards.
/// Without the adoption the batch stays open for ever and the index shows the commit as reverted;
///  with it,
/// the new service finishes the batch in the earlier journal.
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

/// An open batch of the single service of the first releases,
/// in the journal above every instance's own,
/// is adopted too.
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

/// A service of the worktree that may be running finishes its own batches,
/// and two services never settle one batch:
///  the adoption leaves the open batch exactly as it is,
/// and takes it once the service is gone.
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

/// A worktree that has no earlier journal,
/// and a journal of a development run in any other place,
/// adopt nothing and create nothing.
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

/// A client that was not answered by the earlier helper sends the same request again to the service of this release.
/// The
///  service remembers a landed request by its identity so it answers and lands nothing twice;
/// the memory of the earlier
///  helper's finished batches is in its journal,
/// and the service of this release reads it too.
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

/// A live landing of the earlier helper writes its intent before it takes the index lock and moves the branch,
/// so for a while the
///  branch is where the intent says it was and the recovery's answer is "never landed".
/// The adoption must not act on that
///  answer:
/// removing the intent of a landing that goes on leaves a landed commit with no record,
/// and the person's index shows it
///  reverted if the helper dies before it installs the index.
/// Such an intent stays on disk,
/// with its records,
/// for the earlier
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
    write_as_a_landing_in_progress(world.context.journal.as_path(), &intent, Some(intent.new.as_str()));
    let held = try_acquire(intent.index.as_path(), 2).expect("the landing's index lock");
    let context: LandContext = new_context(&world, state.as_path());
    let reports: Vec<RecoveryReport> = adopt_open_batches(&context, instance_for(&world).as_str(), empty_proc(&world).as_path());
    assert!(batch_path(world.context.journal.as_path(), 2, OPEN_EXTENSION).exists(), "the intent stays: {reports:?}");
    assert!(reports.contains(&RecoveryReport { seq: 2, decision: crate::service_journal::Recovery::Drop, handled: Handled::Left }), "and it is reported as left: {reports:?}");
    assert!(batch_path(world.context.journal.as_path(), 2, INDEXED_EXTENSION).exists(), "its record stays");
    drop(held);
    remove(world.root.as_path());
}

/// A landing that moved the branch and still holds the index lock is not finished by the adoption either:
/// the batch stays open
///  for the helper that holds the lock,
/// and a second adoption after the lock is gone finishes it.
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

/// Sets up an open batch in the earlier per-instance journal and a fake process table that `with_processes` fills,
/// then says
///  whether the adoption took the batch.
/// The batch is taken once the table is emptied.
fn adoption_takes_the_batch(name: &str, with_processes: impl FnOnce(&Path, &World, &str)) -> bool {
    let (world, state) = world_with_earlier_journal(name, "state");
    let landed: CommitRequest = request(&world, name, "taken.txt", "taken\n");
    land_with_failed_index(&world, 1, &[landed]);
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    let proc_root: PathBuf = empty_proc(&world);
    with_processes(proc_root.as_path(), &world, instance.as_str());
    let taken: bool = !adopt_open_batches(&context, instance.as_str(), proc_root.as_path()).is_empty();
    remove(world.root.as_path());
    return taken;
}

/// The unit written before commit 9b360150c0 (2026-11-04T16:09:19-05:00) started the helper as
///  `<binary> --journal %S/cli-git/service/%i --git <git> --instance %i`:
/// no `service run` words.
/// A helper started that way
///  is still alive after the unit was rewritten (a process keeps its start arguments;
/// the idle exit is three hours).
#[test]
fn a_helper_started_by_the_unit_before_the_service_mode_is_found_by_its_journal() {
    let taken: bool = adoption_takes_the_batch("adopt-unit-before-mode", |proc_root, world, instance| {
        let journal: String = world.context.journal.to_string_lossy().into_owned();
        fake_process(proc_root, 21, &["/p/cli-git-native", "--journal", journal.as_str(), "--git", "/usr/bin/git", "--instance", instance]);
    });
    assert!(!taken, "a running helper of the earlier unit shape finishes its own batches");
}

/// The first releases ran one service for every worktree:
/// `<binary> --journal <state>/cli-git/service --git <git>`,
/// with no
///  instance.
/// It serves this worktree's batches too.
#[test]
fn the_single_service_of_the_first_releases_is_found_by_the_shared_journal() {
    let taken: bool = adoption_takes_the_batch("adopt-shared-service", |proc_root, world, _instance| {
        let shared: String = world.context.journal.parent().expect("shared journal").to_string_lossy().into_owned();
        fake_process(proc_root, 22, &["/p/cli-git-native", "--journal", shared.as_str(), "--git", "/usr/bin/git"]);
    });
    assert!(!taken, "a running shared service finishes the batches of every worktree");
}

/// Control:
/// a process with a journal that is neither of the earlier ones,
/// and another instance,
/// is no reason to wait.
#[test]
fn a_process_with_another_journal_and_another_instance_does_not_hold_the_adoption() {
    let taken: bool = adoption_takes_the_batch("adopt-other-journal", |proc_root, world, _instance| {
        std::fs::create_dir_all(world.root.join("elsewhere")).expect("another journal");
        let other: String = world.root.join("elsewhere").to_string_lossy().into_owned();
        fake_process(proc_root, 23, &["/p/cli-git-native", "--journal", other.as_str(), "--git", "/usr/bin/git", "--instance", "other"]);
    });
    assert!(taken);
}

/// A process whose command line cannot be read says nothing,
/// so it may be the service:
/// a failure other than "the process is
///  gone" counts as "may be running".
#[test]
fn a_command_line_that_cannot_be_read_counts_as_may_be_running() {
    let unreadable_by_kind: [(&str, fn(&Path)); 2] = [
        ("adopt-cmdline-is-directory", |process| { std::fs::create_dir_all(process.join("cmdline")).expect("a directory read fails with EISDIR"); }),
        ("adopt-cmdline-no-access", |process| {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(process.join("cmdline"), b"x").expect("file");
            std::fs::set_permissions(process.join("cmdline"), std::fs::Permissions::from_mode(0o000)).expect("no access");
        }),
    ];
    for (name, make) in unreadable_by_kind {
        let taken: bool = adoption_takes_the_batch(name, |proc_root, _world, _instance| {
            let process: PathBuf = proc_root.join("31");
            std::fs::create_dir_all(process.as_path()).expect("process directory");
            make(process.as_path());
        });
        assert!(!taken, "{name}");
    }
}

/// The earlier helper may be alive and writing:
/// a `.tmp` or `.claim` file it owns can be older than the grace of a listing while
///  it waits on a slow disk,
/// and removing it would fail its rename or hand its batch number to a second landing.
/// Reading the
///  earlier journals,
/// for the adoption and for the memory of what they landed,
/// removes nothing the adoption does not settle.
#[test]
fn reading_the_earlier_journals_removes_no_file_of_the_earlier_helper() {
    let (world, state) = world_with_earlier_journal("adopt-read-only", "state");
    let landed: CommitRequest = request(&world, "readonly", "readonly.txt", "readonly\n");
    land_with_failed_index(&world, 1, &[landed]);
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - crate::service_journal::TEMPORARY_GRACE - Duration::from_secs(60);
    let shared: PathBuf = world.context.journal.parent().expect("shared journal").to_path_buf();
    let mut stale: Vec<PathBuf> = Vec::new();
    for folder in [world.context.journal.as_path(), shared.as_path()] {
        for extension in ["tmp", "claim"] {
            let file: PathBuf = batch_path(folder, 90, extension);
            std::fs::write(&file, "a write in progress").expect("file of the earlier helper");
            std::fs::File::options().append(true).open(&file).and_then(|opened| { return opened.set_modified(long_ago); }).expect("aged");
            stale.push(file);
        }
    }
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    assert_eq!(adopt_open_batches(&context, instance.as_str(), empty_proc(&world).as_path()).len(), 1, "the batch is still finished");
    let _remembered = earlier_commits_landed(context.journal.as_path(), instance.as_str());
    for file in stale {
        assert!(file.exists(), "{} was removed from the earlier journal", file.display());
    }
    remove(world.root.as_path());
}

/// The adoption says what it did with each batch,
/// as the settling of the service's own journal does:
/// a batch finished,
/// a batch
///  left open,
/// a batch whose intent was left on disk,
/// and the skip of the whole adoption with the folder it protects.
#[test]
fn the_adoption_says_what_it_did_with_each_batch() {
    let (world, state) = world_with_earlier_journal("adopt-says", "state");
    let landed: CommitRequest = request(&world, "says", "says.txt", "says\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    intent.seq = 2;
    intent.old = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    intent.new = "f".repeat(40);
    write_as_a_landing_in_progress(world.context.journal.as_path(), &intent, None);
    let context: LandContext = new_context(&world, state.as_path());
    let instance: String = instance_for(&world);
    let held = try_acquire(intent.index.as_path(), 1).expect("a lock that keeps batch 1 open");
    let entries = crate::service_timing::capture::entries_of(|| { let _reports = adopt_open_batches(&context, instance.as_str(), empty_proc(&world).as_path()); });
    let said: Vec<String> = entries.iter().map(|entry| { return entry.message.clone(); }).collect();
    assert!(said.iter().any(|line| { return line.contains("batch=1") && line.contains("left open"); }), "{said:?}");
    assert!(said.iter().any(|line| { return line.contains("batch=2") && line.contains("left on disk"); }), "{said:?}");
    assert!(entries.iter().all(|entry| { return entry.fields.iter().any(|(name, _)| { return name == "CLI_GIT_BATCH"; }) || !entry.message.contains("batch="); }), "{entries:?}");
    drop(held);
    let finished = crate::service_timing::capture::entries_of(|| { let _reports = adopt_open_batches(&context, instance.as_str(), empty_proc(&world).as_path()); });
    assert!(finished.iter().any(|entry| { return entry.message.contains("batch=1") && entry.message.contains("finish") && entry.message.contains(world.context.journal.to_string_lossy().as_ref()); }), "{finished:?}");
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 61, &["/p/cli-git-native", "cli-git", "service", "run", "--instance", instance.as_str()]);
    let skipped = crate::service_timing::capture::entries_of(|| { let _reports = adopt_open_batches(&context, instance.as_str(), proc_root.as_path()); });
    assert!(skipped.iter().any(|entry| { return entry.message.contains(world.context.journal.to_string_lossy().as_ref()); }), "the skip names the per-instance journal: {skipped:?}");
    remove(world.root.as_path());
}

/// The flags column of `/proc/net/unix` for a socket that listens (`SO_ACCEPTCON`),
/// as the kernel prints it.
pub(super) const LISTENING_FLAGS: &str = "00010000";

/// The flags column of `/proc/net/unix` for a socket that does not listen.
const CONNECTED_FLAGS: &str = "00000000";

/// The first line of `/proc/net/unix`.
const UNIX_TABLE_HEADER: &str = "Num       RefCount Protocol Flags    Type St Inode Path\n";

/// A user's runtime directory,
/// as the sockets of the units name it.
pub(super) const RUNTIME_DIRECTORY: &str = "/run/user/1000";

/// The name of the socket the single service of the first releases listened on,
/// below the earlier program folder.
const SHARED_SOCKET_NAME: &str = "service.sock";

/// One line of `/proc/net/unix` for a socket bound at `<runtime>/<folder>/<file>`,
/// in the columns the kernel prints.
pub(super) fn unix_table_line(flags: &str, folder: &str, file: &str) -> String {
    return format!("000000002982a48d: 00000002 00000000 {flags} 0001 01 2269401 {RUNTIME_DIRECTORY}/{folder}/{file}\n");
}

/// Writes the unix socket table of a fake proc root.
pub(super) fn write_unix_table(proc_root: &Path, lines: &[String]) {
    let table: PathBuf = proc_root.join("net/unix");
    std::fs::create_dir_all(table.parent().expect("net folder")).expect("net folder");
    std::fs::write(table, format!("{UNIX_TABLE_HEADER}{}", lines.concat())).expect("unix table");
}

/// Whether the adoption takes an open batch when the fake unix socket table holds `lines` (`None`:
/// no table at all).
fn adoption_takes_the_batch_with_sockets(name: &str, lines: impl FnOnce(&str) -> Option<Vec<String>>) -> bool {
    return adoption_takes_the_batch(name, |proc_root, _world, instance| {
        match lines(instance) {
            Some(listed) => write_unix_table(proc_root, listed.as_slice()),
            None => {
                let _absent = std::fs::remove_file(proc_root.join("net/unix"));
            }
        }
    });
}

/// The earlier socket unit of a worktree listens on `<runtime>/cli-git/<instance>.sock`,
/// and a connection to it starts the
///  earlier helper at any moment,
/// with no process to be seen before.
/// The old helper holds no lock on its journal that this
///  program could also take,
/// so the adoption waits while that socket listens,
/// whoever holds it.
/// The first releases' single
///  service listened on `<runtime>/cli-git/service.sock`;
/// it serves the worktree's batches in the shared journal.
#[test]
fn an_earlier_socket_that_listens_keeps_the_adoption_away() {
    assert!(!adoption_takes_the_batch_with_sockets("adopt-socket-instance", |instance| {
        return Some(vec![unix_table_line(LISTENING_FLAGS, DATA_FOLDER.earlier, format!("{instance}.sock").as_str())]);
    }), "the socket of the worktree's earlier unit");
    assert!(!adoption_takes_the_batch_with_sockets("adopt-socket-shared", |_instance| {
        return Some(vec![unix_table_line(LISTENING_FLAGS, DATA_FOLDER.earlier, SHARED_SOCKET_NAME)]);
    }), "the socket of the first releases' single service");
}

/// Control:
/// what is not a way for the earlier helper to start does not hold the adoption:
/// the socket of another worktree,
/// a
///  socket that does not listen,
/// a socket of another program,
/// and a table with no socket at all.
#[test]
fn a_socket_that_cannot_start_the_earlier_helper_does_not_hold_the_adoption() {
    let other_instance: String = "f".repeat(crate::service_instance::INSTANCE_LENGTH);
    assert!(adoption_takes_the_batch_with_sockets("adopt-socket-other-instance", |_instance| {
        return Some(vec![unix_table_line(LISTENING_FLAGS, DATA_FOLDER.earlier, format!("{other_instance}.sock").as_str())]);
    }), "another worktree's socket");
    assert!(adoption_takes_the_batch_with_sockets("adopt-socket-not-listening", |instance| {
        return Some(vec![unix_table_line(CONNECTED_FLAGS, DATA_FOLDER.earlier, format!("{instance}.sock").as_str())]);
    }), "a socket that does not listen");
    assert!(adoption_takes_the_batch_with_sockets("adopt-socket-other-program", |instance| {
        return Some(vec![unix_table_line(LISTENING_FLAGS, DATA_FOLDER.current, format!("{instance}.sock").as_str())]);
    }), "the socket of this release's unit");
    assert!(adoption_takes_the_batch_with_sockets("adopt-socket-none", |_instance| { return Some(Vec::new()); }), "no socket");
}

/// A table of sockets that cannot be read proves nothing about the earlier unit,
/// so the adoption waits.
#[test]
fn an_unreadable_table_of_sockets_counts_as_may_be_running() {
    assert!(!adoption_takes_the_batch_with_sockets("adopt-socket-table-missing", |_instance| { return None; }));
}

/// A journal the earlier helper was started with,
/// spelled through a symbolic link (on this machine `/home` is a link to
///  `/var/home`),
/// is the same folder as the one the service derives,
/// and a relative spelling is the folder the process's own
///  working directory gives it.
/// A spelling that names another existing folder is not the journal;
/// one that cannot be resolved
///  proves nothing,
/// so it counts as "may be running".
#[test]
fn a_journal_is_compared_by_the_folder_it_names_not_by_its_spelling() {
    use std::os::unix::fs::symlink;
    let world: World = world("adopt-proc-identity");
    let proc_root: PathBuf = empty_proc(&world);
    let own: u32 = 4242;
    let real: PathBuf = world.root.join("real");
    std::fs::create_dir_all(real.join("abc")).expect("the journal");
    std::fs::create_dir_all(world.root.join("other")).expect("another folder");
    symlink(real.as_path(), world.root.join("alias")).expect("a link to the folder above the journal");
    let look: ServiceLook = ServiceLook { journals: vec![real.join("abc"), real.clone()], ..look_for(own) };
    let older_shape = |journal: &str| -> Vec<String> { return vec![String::from("/p/cli-git-native"), String::from("--journal"), journal.to_string(), String::from("--git"), String::from("/usr/bin/git")]; };
    let say = |pid: u32, journal: &str| {
        let words: Vec<String> = older_shape(journal);
        fake_process(proc_root.as_path(), pid, words.iter().map(String::as_str).collect::<Vec<_>>().as_slice());
    };
    say(70, world.root.join("other").to_string_lossy().as_ref());
    assert!(!service_of_instance_may_run(look.clone(), proc_root.as_path()), "another existing folder");
    say(71, world.root.join("alias/abc").to_string_lossy().as_ref());
    assert!(service_of_instance_may_run(look.clone(), proc_root.as_path()), "the journal through a link");
    std::fs::remove_dir_all(proc_root.join("71")).expect("ended");
    say(72, "real/abc");
    symlink(world.root.as_path(), proc_root.join("72/cwd")).expect("the working directory of the process");
    assert!(service_of_instance_may_run(look.clone(), proc_root.as_path()), "the journal relative to the working directory of the process");
    std::fs::remove_dir_all(proc_root.join("72")).expect("ended");
    say(73, "other");
    symlink(world.root.as_path(), proc_root.join("73/cwd")).expect("the working directory of the process");
    assert!(!service_of_instance_may_run(look.clone(), proc_root.as_path()), "a relative spelling of another existing folder");
    std::fs::remove_dir_all(proc_root.join("73")).expect("ended");
    say(74, world.root.join("no-such-folder").to_string_lossy().as_ref());
    assert!(service_of_instance_may_run(look.clone(), proc_root.as_path()), "a journal that cannot be resolved");
    std::fs::remove_dir_all(proc_root.join("74")).expect("ended");
    say(75, "real/abc");
    assert!(service_of_instance_may_run(look, proc_root.as_path()), "a relative spelling of a process whose working directory cannot be read");
    remove(world.root.as_path());
}

/// Only the first bytes of a command line are read,
/// and a line that fills that limit has more words that were not looked at:
/// an
///  earlier helper started with a very long program name has its service words past the limit.
/// Such a line counts as "may be
///  running",
/// and a line that ends exactly at the limit was read whole.
#[test]
fn a_command_line_longer_than_the_bytes_read_counts_as_may_be_running() {
    let limit: usize = usize::try_from(crate::earlier_service_process_look::COMMAND_LINE_BYTES).expect("the limit fits");
    let world: World = world("adopt-proc-long");
    let proc_root: PathBuf = empty_proc(&world);
    let long_program: String = "p".repeat(limit + limit / 2);
    fake_process(proc_root.as_path(), 80, &[long_program.as_str(), "cli-git", "service", "run", "--journal", "/j", "--instance", "abc"]);
    assert!(service_of_instance_may_run(look_for(4242), proc_root.as_path()), "the service words are past the limit");
    std::fs::remove_dir_all(proc_root.join("80")).expect("ended");
    // The program name and its NUL fill the limit exactly: nothing was left unread.
    let exact_program: String = "p".repeat(limit - 1);
    fake_process(proc_root.as_path(), 81, &[exact_program.as_str()]);
    assert!(!service_of_instance_may_run(look_for(4242), proc_root.as_path()), "a command line that ends at the limit was read whole");
    remove(world.root.as_path());
}
