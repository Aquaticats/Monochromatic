//! Controls for the adoption of the batches an earlier release's service left open: where the earlier journals are, when a
//!  service of the worktree may still be running, and that an open batch is finished in place and never twice.
//! The controls sit beside the landing controls because they land real batches with the same fixtures.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_land::Handled;
use std::time::Duration;
use crate::earlier_service_journal_adopted::{EarlierJournals, adopt_open_batches, earlier_commits_landed, earlier_journals_of};
use crate::earlier_service_process_look::{ServiceLook, effective_user, service_of_instance_may_run};
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

/// The look of the instance `abc` whose earlier journals are `/s/abc` and `/s`, run by `own`.
fn look_for(own_pid: u32) -> ServiceLook {
    return ServiceLook {
        instance: String::from("abc"),
        journals: vec![PathBuf::from("/s/abc"), PathBuf::from("/s")],
        own_pid,
        own_user: effective_user(),
        ceiling: Duration::from_secs(30),
    };
}

/// A process with the service words and the instance option of the worktree, or the journal of the worktree's earlier service
///  as its journal, means a service may be running; another instance or journal, this process itself, an unrelated command
///  line and a process that vanished do not; a proc root that cannot be listed means it may.
#[test]
fn a_service_of_the_worktree_is_found_among_the_processes_and_nothing_else_is() {
    let world: World = world("adopt-proc");
    let proc_root: PathBuf = empty_proc(&world);
    let own: u32 = 4242;
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "no process");
    fake_process(proc_root.as_path(), 10, &["/bin/sh", "-c", "service run --instance abc"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "one word, not separate words");
    fake_process(proc_root.as_path(), 11, &["/p/cli-git-native", "cli-git", "service", "run", "--journal", "/j", "--instance", "other"]);
    assert!(!service_of_instance_may_run(look_for(own), proc_root.as_path()), "another worktree's service");
    fake_process(proc_root.as_path(), 14, &["/p/cli-git-native", "--journal", "/s/other", "--git", "/usr/bin/git", "--instance", "other"]);
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

/// Only the processes of the user count: a service another user runs for the same worktree is not this user's to wait for.
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

/// Reading a command line can wait for the process for ever; the look ends at its ceiling and answers "may be running".
///  A named pipe in place of the file stands for it: opening succeeds, reading waits.
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

/// Sets up an open batch in the earlier per-instance journal and a fake process table that `with_processes` fills, then says
///  whether the adoption took the batch. The batch is taken once the table is emptied.
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
///  `<binary> --journal %S/cli-git/service/%i --git <git> --instance %i`: no `service run` words. A helper started that way
///  is still alive after the unit was rewritten (a process keeps its start arguments; the idle exit is three hours).
#[test]
fn a_helper_started_by_the_unit_before_the_service_mode_is_found_by_its_journal() {
    let taken: bool = adoption_takes_the_batch("adopt-unit-before-mode", |proc_root, world, instance| {
        let journal: String = world.context.journal.to_string_lossy().into_owned();
        fake_process(proc_root, 21, &["/p/cli-git-native", "--journal", journal.as_str(), "--git", "/usr/bin/git", "--instance", instance]);
    });
    assert!(!taken, "a running helper of the earlier unit shape finishes its own batches");
}

/// The first releases ran one service for every worktree: `<binary> --journal <state>/cli-git/service --git <git>`, with no
///  instance. It serves this worktree's batches too.
#[test]
fn the_single_service_of_the_first_releases_is_found_by_the_shared_journal() {
    let taken: bool = adoption_takes_the_batch("adopt-shared-service", |proc_root, world, _instance| {
        let shared: String = world.context.journal.parent().expect("shared journal").to_string_lossy().into_owned();
        fake_process(proc_root, 22, &["/p/cli-git-native", "--journal", shared.as_str(), "--git", "/usr/bin/git"]);
    });
    assert!(!taken, "a running shared service finishes the batches of every worktree");
}

/// Control: a process with a journal that is neither of the earlier ones, and another instance, is no reason to wait.
#[test]
fn a_process_with_another_journal_and_another_instance_does_not_hold_the_adoption() {
    let taken: bool = adoption_takes_the_batch("adopt-other-journal", |proc_root, world, _instance| {
        let other: String = world.root.join("elsewhere").to_string_lossy().into_owned();
        fake_process(proc_root, 23, &["/p/cli-git-native", "--journal", other.as_str(), "--git", "/usr/bin/git", "--instance", "other"]);
    });
    assert!(taken);
}

/// A process whose command line cannot be read says nothing, so it may be the service: a failure other than "the process is
///  gone" counts as "may be running".
#[test]
fn a_command_line_that_cannot_be_read_counts_as_may_be_running() {
    let unreadable_by_kind: [(&str, fn(&Path)); 2] = [
        ("adopt-cmdline-is-directory", |process| { std::fs::create_dir_all(process.join("cmdline")).expect("a directory read fails with EISDIR"); }),
        ("adopt-cmdline-no-access", |process| {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(process.join("cmdline"), b"x").expect("file");
            std::fs::set_permissions(process.join("cmdline"), std::fs::Permissions::from_mode(0)).expect("no access");
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
