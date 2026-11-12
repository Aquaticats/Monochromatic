//! Controls for settling,
//! when the person asks,
//! the batches an earlier release's service left open:
//! which batches count as
//!  the worktree's,
//! that nothing is changed while an earlier helper may run or start,
//! and that what is settled is the
//!  finish-only recovery of the adoption.

use super::adoption_tests::{LISTENING_FLAGS, empty_proc, fake_process, instance_for, unix_table_line, world_with_earlier_journal, write_unix_table};
use super::tests::{World, land_with_failed_index, request};
use super::*;
use crate::earlier_service_journal_adopted::{RequestedSettlement, UnansweredBatch, earlier_journals_in_state, open_batch_count, settle_on_request};
use crate::earlier_service_process_look::Hold;
use crate::held_commit_record::{Holding, hold};
use crate::service_journal::{OPEN_EXTENSION, batch_path, list_intents, parse_intent, write_intent};
use crate::test_support::{git_text, remove};
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::time::Duration;

/// The time the recovery of a control may wait for an index lock.
const LOCK_WAIT: Duration = Duration::from_secs(5);

/// The journals the earlier release kept for the world's worktree,
/// below `state`.
fn journals_of(world: &World, state: &Path) -> crate::earlier_service_journal_adopted::EarlierJournals {
    return earlier_journals_in_state(state, instance_for(world).as_str());
}

/// Settles with the world's Git and the process table `proc_root`.
fn settle(world: &World, state: &Path, proc_root: &Path) -> RequestedSettlement {
    return settle_on_request(&world.context.git, &journals_of(world, state), instance_for(world).as_str(), proc_root, LOCK_WAIT);
}

/// The earlier journals of an instance are named from the state folder alone.
#[test]
fn the_earlier_journals_are_named_from_the_state_folder() {
    let found = earlier_journals_in_state(Path::new("/home/u/.local/state"), "abc");
    assert_eq!(found.per_instance, PathBuf::from("/home/u/.local/state/cli-git/service/abc"));
    assert_eq!(found.shared, PathBuf::from("/home/u/.local/state/cli-git/service"));
}

/// The count is the open batches of this worktree in the per-instance journal and in the shared one;
/// a batch of another
///  worktree in the shared journal,
/// a finished batch and an unreadable file are not counted,
/// and counting changes no file.
#[test]
fn the_count_is_the_open_batches_of_this_worktree_and_changes_nothing() {
    let (world, state) = world_with_earlier_journal("settle-count", "state");
    let per_instance: PathBuf = world.context.journal.clone();
    let landed: CommitRequest = request(&world, "counted", "counted.txt", "counted\n");
    land_with_failed_index(&world, 1, &[landed]);
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join("service");
    let own_text: String = std::fs::read_to_string(batch_path(per_instance.as_path(), 1, OPEN_EXTENSION)).expect("read");
    let mut shared_own = parse_intent(own_text.as_str()).expect("parse");
    shared_own.seq = 7;
    write_intent(shared.as_path(), &shared_own).expect("an open batch of this worktree in the shared journal");
    let mut other = shared_own.clone();
    other.seq = 8;
    other.git_dir = world.root.join("another-worktree/.git");
    write_intent(shared.as_path(), &other).expect("an open batch of another worktree in the shared journal");
    std::fs::write(shared.join("9.intent"), "not an intent").expect("an unreadable file");
    let before: Vec<String> = names_in(&[per_instance.as_path(), shared.as_path()]);
    assert_eq!(open_batch_count(&journals_of(&world, state.as_path()), instance_for(&world).as_str()), 2);
    assert_eq!(names_in(&[per_instance.as_path(), shared.as_path()]), before, "counting changed no file");
    remove(world.root.as_path());
}

/// The names of the files in some folders,
/// sorted.
fn names_in(folders: &[&Path]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for folder in folders {
        for entry in std::fs::read_dir(folder).expect("folder").flatten() {
            names.push(entry.path().display().to_string());
        }
    }
    names.sort();
    return names;
}

/// With no batch open nothing is looked at:
/// not the sockets,
/// not the processes.
#[test]
fn nothing_open_means_nothing_to_look_at() {
    let (world, state) = world_with_earlier_journal("settle-nothing", "state");
    let proc_root: PathBuf = world.root.join("proc-unreadable");
    assert_eq!(settle(&world, state.as_path(), proc_root.as_path()), RequestedSettlement::NothingOpen);
    remove(world.root.as_path());
}

/// With an open batch and nothing that runs or listens,
/// the batch is finished as the adoption finishes it,
/// each batch is
///  logged with its number,
/// and a second request finds nothing.
#[test]
fn an_open_batch_is_finished_when_asked_and_each_batch_is_logged() {
    let (world, state) = world_with_earlier_journal("settle-finish", "state");
    let landed: CommitRequest = request(&world, "finish", "finish.txt", "finish\n");
    land_with_failed_index(&world, 1, &[landed]);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "the index shows the landed file reverted");
    let proc_root: PathBuf = empty_proc(&world);
    let mut settled: Option<RequestedSettlement> = None;
    let entries = crate::service_timing::capture::entries_of(|| { settled = Some(settle(&world, state.as_path(), proc_root.as_path())); });
    let Some(RequestedSettlement::Settled { batches, .. }) = settled else { panic!("{settled:?}") };
    assert_eq!(batches.iter().map(|batch| { return (batch.report.seq, batch.report.handled, batch.journal.clone()); }).collect::<Vec<_>>(), vec![(1, Handled::Finished, world.context.journal.clone())]);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    assert!(entries.iter().any(|entry| { return entry.message.contains("batch=1") && entry.message.contains("finish"); }), "{entries:?}");
    assert_eq!(settle(&world, state.as_path(), proc_root.as_path()), RequestedSettlement::NothingOpen);
    remove(world.root.as_path());
}

/// Each reason the look gives for holding back is passed on,
/// and nothing is settled while it stands:
/// the batch stays open.
#[test]
fn nothing_is_settled_while_an_earlier_helper_may_run_or_start_and_the_reason_is_given() {
    let (world, state) = world_with_earlier_journal("settle-held", "state");
    let landed: CommitRequest = request(&world, "held", "held.txt", "held\n");
    land_with_failed_index(&world, 1, &[landed]);
    let instance: String = instance_for(&world);
    let listening: PathBuf = empty_proc(&world);
    write_unix_table(listening.as_path(), &[unix_table_line(LISTENING_FLAGS, "cli-git", format!("{instance}.sock").as_str())]);
    assert_eq!(settle(&world, state.as_path(), listening.as_path()), RequestedSettlement::Held { open: 1, because: Hold::InstanceSocketListens.into() });
    let running: PathBuf = empty_proc(&world);
    fake_process(running.as_path(), 77, &["/p/cli-git-native", "cli-git", "service", "run", "--instance", instance.as_str()]);
    assert_eq!(settle(&world, state.as_path(), running.as_path()), RequestedSettlement::Held { open: 1, because: Hold::InstanceHelperRuns.into() });
    let unreadable: PathBuf = empty_proc(&world);
    std::fs::remove_file(unreadable.join("net/unix")).expect("the table is gone");
    assert_eq!(settle(&world, state.as_path(), unreadable.as_path()), RequestedSettlement::Held { open: 1, because: Hold::CannotLook.into() });
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 1, "still open");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "the index is as it was");
    remove(world.root.as_path());
}

/// A batch of another worktree in the shared journal is that worktree's:
/// it is not counted,
/// looked at or settled.
#[test]
fn the_batch_of_another_worktree_in_the_shared_journal_is_not_settled() {
    let (world, state) = world_with_earlier_journal("settle-other", "state");
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join("service");
    let landed: CommitRequest = request(&world, "other", "other.txt", "other\n");
    land_with_failed_index(&world, 1, &[landed]);
    let text: String = std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read");
    std::fs::remove_file(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("moved away");
    let mut intent = parse_intent(text.as_str()).expect("parse");
    intent.git_dir = world.root.join("another-worktree/.git");
    write_intent(shared.as_path(), &intent).expect("the other worktree's batch");
    assert_eq!(settle(&world, state.as_path(), empty_proc(&world).as_path()), RequestedSettlement::NothingOpen);
    assert_eq!(list_intents(shared.as_path(), OPEN_EXTENSION).len(), 1);
    remove(world.root.as_path());
}

/// The recovery is finish-only:
/// a batch that looks as if it never landed keeps its intent on disk,
/// because a landing in
///  progress writes its intent first,
/// and the request says so.
#[test]
fn a_batch_that_looks_unlanded_is_left_on_disk() {
    let (world, state) = world_with_earlier_journal("settle-left", "state");
    let landed: CommitRequest = request(&world, "left", "left.txt", "left\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    intent.seq = 2;
    intent.old = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    intent.new = "f".repeat(40);
    write_intent(world.context.journal.as_path(), &intent).expect("the intent of a landing in progress");
    let RequestedSettlement::Settled { batches, .. } = settle(&world, state.as_path(), empty_proc(&world).as_path()) else { panic!("settled") };
    let handled: Vec<(u64, Handled)> = batches.iter().map(|batch| { return (batch.report.seq, batch.report.handled); }).collect();
    assert!(handled.contains(&(1, Handled::Finished)) && handled.contains(&(2, Handled::Left)), "{handled:?}");
    assert!(batch_path(world.context.journal.as_path(), 2, OPEN_EXTENSION).exists(), "the intent stays");
    remove(world.root.as_path());
}

/// Found when the settle command first ran against a real wrapper:
/// the service of this release runs the same words
///  (`service run --instance <instance>`) behind the current command word,
/// and it is no earlier helper,
/// so it does not hold the
///  command back;
/// the same words behind the earlier command word do.
#[test]
fn the_service_of_this_release_is_no_earlier_helper() {
    let (world, state) = world_with_earlier_journal("settle-own-service", "state");
    let landed: CommitRequest = request(&world, "own", "own.txt", "own\n");
    land_with_failed_index(&world, 1, &[landed]);
    let instance: String = instance_for(&world);
    let proc_root: PathBuf = empty_proc(&world);
    let new_journal: String = state.join("monochromatic-git-policy-cli/service").join(instance.as_str()).display().to_string();
    std::fs::create_dir_all(new_journal.as_str()).expect("the new service's journal exists, as it does for a running service");
    fake_process(proc_root.as_path(), 41, &["/p/monochromatic-git-policy-cli", "monochromatic-git-policy-cli", "service", "run", "--journal", new_journal.as_str(), "--instance", instance.as_str()]);
    let RequestedSettlement::Settled { batches, .. } = settle(&world, state.as_path(), proc_root.as_path()) else { panic!("the new service held the settling back") };
    assert_eq!(batches.len(), 1);
    remove(world.root.as_path());
}

/// The listener of the single service of the first releases,
/// and a process started with the shared journal,
/// serve every
///  worktree,
/// so the hold names them as such and not as this worktree's;
/// a process whose journal folder cannot be resolved
///  (here a relative word,
/// whose working directory is not there) may be either and is told apart too.
#[test]
fn the_hold_keeps_which_socket_or_helper_caused_it() {
    let (world, state) = world_with_earlier_journal("settle-which", "state");
    let landed: CommitRequest = request(&world, "which", "which.txt", "which\n");
    land_with_failed_index(&world, 1, &[landed]);
    let instance: String = instance_for(&world);
    let shared: String = state.join(DATA_FOLDER.earlier).join("service").display().to_string();
    let own: String = world.context.journal.display().to_string();
    let held = |proc_root: &Path| -> RequestedSettlement { return settle(&world, state.as_path(), proc_root); };
    let shared_socket: PathBuf = empty_proc(&world);
    write_unix_table(shared_socket.as_path(), &[unix_table_line(LISTENING_FLAGS, "cli-git", "service.sock")]);
    assert_eq!(held(shared_socket.as_path()), RequestedSettlement::Held { open: 1, because: Hold::SharedSocketListens.into() });
    let own_socket: PathBuf = empty_proc(&world);
    write_unix_table(own_socket.as_path(), &[unix_table_line(LISTENING_FLAGS, "cli-git", "service.sock"), unix_table_line(LISTENING_FLAGS, "cli-git", format!("{instance}.sock").as_str())]);
    assert_eq!(held(own_socket.as_path()), RequestedSettlement::Held { open: 1, because: Hold::InstanceSocketListens.into() }, "this worktree's own socket is named first");
    for (label, words, expected) in [
        ("shared journal", vec!["/p/cli-git-native", "--journal", shared.as_str(), "--git", "/usr/bin/git"], Hold::SharedHelperRuns),
        ("own journal", vec!["/p/cli-git-native", "--journal", own.as_str(), "--git", "/usr/bin/git", "--instance", instance.as_str()], Hold::InstanceHelperRuns),
        ("mode words", vec!["/p/cli-git-native", "cli-git", "service", "run", "--instance", instance.as_str()], Hold::InstanceHelperRuns),
        ("unresolvable", vec!["/p/cli-git-native", "--journal", "not/yet/there"], Hold::HelperUnsure),
    ] {
        let proc_root: PathBuf = empty_proc(&world);
        fake_process(proc_root.as_path(), 52, &words);
        let RequestedSettlement::Held { open, because } = held(proc_root.as_path()) else { panic!("{label}: held back") };
        assert_eq!((open, because.because), (1, expected), "{label}");
        let named: Option<(u32, String, String)> = because.program.map(|program| { return (program.pid, program.journal_word, program.reason); });
        match expected {
            Hold::HelperUnsure => {
                let (pid, word, reason) = named.expect("the program that cannot be ruled out is named");
                assert_eq!((pid, word.as_str()), (52, "not/yet/there"), "{label}");
                assert!(reason.contains("No such file or directory"), "{label}: {reason}");
            }
            _ => assert_eq!(named, None, "{label}: only a program that cannot be ruled out is named"),
        }
    }
    remove(world.root.as_path());
}

/// The service log says which program held a settling back (its process ID,
/// the program and the folder that did not resolve)
///  and shows no other word of its command line,
/// because any of them can be a secret that no masker knows.
#[test]
fn the_log_line_of_a_held_settling_names_the_program_and_folder_and_no_other_argument() {
    let (world, state) = world_with_earlier_journal("settle-log-program", "state");
    let landed: CommitRequest = request(&world, "logged", "logged.txt", "logged\n");
    land_with_failed_index(&world, 1, &[landed]);
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 52, &["/usr/bin/unrelated-tool", "--journal", "not/yet/there", "--remote", "https://user:s3cret@example.invalid/x", "--token", "s3cret-token"]);
    let mut settled: Option<RequestedSettlement> = None;
    let entries = crate::service_timing::capture::entries_of(|| { settled = Some(settle(&world, state.as_path(), proc_root.as_path())); });
    assert!(matches!(settled, Some(RequestedSettlement::Held { .. })), "{settled:?}");
    let line: &str = entries.iter().map(|entry| { return entry.message.as_str(); }).find(|message| { return message.contains("were not settled"); }).expect("the held settling is logged");
    for wanted in ["52", "/usr/bin/unrelated-tool", "not/yet/there"] {
        assert!(line.contains(wanted), "{wanted}: {line}");
    }
    for secret in ["s3cret", "--token", "--remote", "example.invalid"] {
        assert!(!entries.iter().any(|entry| { return entry.message.contains(secret); }), "{secret} must not be logged: {entries:?}");
    }
    remove(world.root.as_path());
}

/// An unrelated program that carries a `--journal` option naming a folder that is not there holds nothing back:
/// a folder that is not there holds no batch of this worktree,
/// so the program cannot be settling one.
/// The
///  release gate of 2026-11-10 failed only on this host,
/// where another session ran such a program,
/// and every settle control
///  waited for it until its patience ran out.
#[test]
fn an_unrelated_program_whose_journal_folder_is_not_there_holds_nothing_back() {
    let (world, state) = world_with_earlier_journal("settle-absent-journal", "state");
    let landed: CommitRequest = request(&world, "absent", "absent.txt", "absent\n");
    land_with_failed_index(&world, 1, &[landed]);
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 61, &["/usr/bin/unrelated-tool", "--journal", "/not/a/folder/of/a/worktree", "--token", "s3cret-token"]);
    let RequestedSettlement::Settled { batches, .. } = settle(&world, state.as_path(), proc_root.as_path()) else {
        panic!("a folder that is not there holds no batch, so the settling goes on")
    };
    assert_eq!(batches.len(), 1);
    remove(world.root.as_path());
}

/// What:
///  A program that named this worktree's earlier journal through a link which another command removed after the program
///  started still holds the settling back.
/// Why:
///   The link resolved when the program started and is gone now,
/// so the word the program carries no longer shows the folder it
///  names,
/// while the folder itself is there with the batch the program may be finishing.
/// The word cannot be compared by
///  the folder it named,
/// and the same command line names this worktree's instance,
/// which is what an earlier helper of
///  one worktree was started with,
/// so it may be that helper and every batch is left.
/// Kimi K3.1 Pro found
///  this by reading the settle that finished a live helper's batch
/// (`package/git-policy/cli/src/native/landing/earlier_service_process_look.rs`,
/// the branch that compares a folder that is
///  not there by the path the word spells).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkSync(sharedJournal, link); fakeProcess(['--journal', `${link}/${instance}`, '--git', git, '--instance', instance]); unlinkSync(link); expect(settleOnRequest(...)).toMatchObject({ held: true })
/// ```
#[cfg(unix)]
#[test]
fn a_live_helpers_journal_word_a_removed_link_no_longer_resolves_still_holds_the_settling() {
    let (world, state) = world_with_earlier_journal("settle-removed-link", "state");
    let landed: CommitRequest = request(&world, "removed-link", "removed-link.txt", "removed-link\n");
    land_with_failed_index(&world, 1, &[landed]);
    let instance: String = instance_for(&world);
    let per_instance: PathBuf = world.context.journal.clone();
    let shared: PathBuf = per_instance.parent().expect("the shared journal").to_path_buf();
    let link: PathBuf = world.root.join("a-link-to-the-state-folder");
    std::os::unix::fs::symlink(shared.as_path(), link.as_path()).expect("the link the helper starts with");
    let spelled: String = link.join(instance.as_str()).display().to_string();
    assert_eq!(
        std::fs::canonicalize(spelled.as_str()).expect("the link resolves while the helper starts"),
        std::fs::canonicalize(per_instance.as_path()).expect("the journal the helper writes"),
    );
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(
        proc_root.as_path(),
        71,
        &["/p/cli-git-native", "--journal", spelled.as_str(), "--git", "/usr/bin/git", "--instance", instance.as_str()],
    );
    std::fs::remove_file(link.as_path()).expect("another command removes the link after the helper started");
    assert!(per_instance.is_dir(), "the helper's journal and its open batch remain");
    assert!(!Path::new(spelled.as_str()).exists(), "the spelling the helper carries no longer resolves");
    let before: Vec<String> = names_in(&[per_instance.as_path()]);
    assert_eq!(
        settle(&world, state.as_path(), proc_root.as_path()),
        RequestedSettlement::Held { open: 1, because: Hold::InstanceHelperRuns.into() },
        "the live helper was missed, so its batch may be settled twice"
    );
    assert_eq!(names_in(&[per_instance.as_path()]), before, "nothing of the batch changed");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "the index is as it was");
    remove(world.root.as_path());
}

/// What:
///  This release's own service holds nothing back,
/// even when the journal folder its command line names is not there.
/// Why:
///   The service of this release serves the worktree's new journal and settles its own batches,
/// never an earlier release's,
/// so its command line is no reason to leave any batch of the earlier journals alone.
/// The look tells it apart by
///  the mode words behind this release's command word,
/// and a word it cannot resolve must not turn it into an earlier
///  helper:
/// the hold would name the wrong release and stop the settling for a reason that never clears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fakeProcess(['/p/monochromatic-git-policy-cli', 'monochromatic-git-policy-cli', 'service', 'run', '--journal', notMadeYet, '--instance', instance]); expect(settle(...)).toMatchObject({ settled: true })
/// ```
#[test]
fn the_service_of_this_release_holds_nothing_back_when_its_folder_is_not_there() {
    let (world, state) = world_with_earlier_journal("settle-own-service-absent", "state");
    let landed: CommitRequest = request(&world, "own-absent", "own-absent.txt", "own-absent\n");
    land_with_failed_index(&world, 1, &[landed]);
    let instance: String = instance_for(&world);
    let not_made_yet: String = state.join("monochromatic-git-policy-cli/service").join(instance.as_str()).display().to_string();
    assert!(!Path::new(not_made_yet.as_str()).exists(), "the folder the service names is not made yet");
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(
        proc_root.as_path(),
        42,
        &[
            "/p/monochromatic-git-policy-cli",
            "monochromatic-git-policy-cli",
            "service",
            "run",
            "--journal",
            not_made_yet.as_str(),
            "--instance",
            instance.as_str(),
        ],
    );
    let RequestedSettlement::Settled { batches, .. } = settle(&world, state.as_path(), proc_root.as_path()) else {
        panic!("this release's own service held the settling back")
    };
    assert_eq!(batches.len(), 1);
    remove(world.root.as_path());
}

/// What the look still keeps:
/// an earlier helper shows its `--journal` word before it makes that folder,
/// so a word that names one of
///  this worktree's earlier journals,
/// under another spelling of the same folder,
/// holds the settling back even
///  while the folder is not made yet.
/// The helper settles the shared journal's batches of this worktree at its own
///  start,
/// and two services must never settle one batch.
#[cfg(unix)]
#[test]
fn a_journal_not_made_yet_is_recognised_through_another_spelling_of_the_same_folder() {
    let (world, state) = world_with_earlier_journal("settle-not-made-yet", "state");
    let landed: CommitRequest = request(&world, "not-made-yet", "not-made-yet.txt", "not-made-yet\n");
    land_with_failed_index(&world, 1, &[landed]);
    let per_instance: PathBuf = world.context.journal.clone();
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join("service");
    // The open batch waits in the shared journal, which a starting helper settles at its own start.
    let text: String = std::fs::read_to_string(batch_path(per_instance.as_path(), 1, OPEN_EXTENSION)).expect("the open batch");
    let mut moved = parse_intent(text.as_str()).expect("parse");
    moved.seq = 4;
    write_intent(shared.as_path(), &moved).expect("an open batch in the shared journal");
    remove(per_instance.as_path());
    // The helper names its own journal through the link and has not made that folder yet.
    let another_name: PathBuf = world.root.join("another-name-for-the-fixture-root");
    std::os::unix::fs::symlink(world.root.as_path(), another_name.as_path()).expect("a link to the fixture root");
    let spelled: String = another_name
        .join(state.strip_prefix(world.root.as_path()).expect("the state folder is inside the fixture"))
        .join(DATA_FOLDER.earlier)
        .join("service")
        .join(instance_for(&world))
        .display()
        .to_string();
    assert!(!Path::new(spelled.as_str()).exists(), "the folder the helper names is not made yet");
    let proc_root: PathBuf = empty_proc(&world);
    fake_process(proc_root.as_path(), 62, &["/p/cli-git-native", "--journal", spelled.as_str(), "--git", "/usr/bin/git"]);
    assert_eq!(
        settle(&world, state.as_path(), proc_root.as_path()),
        RequestedSettlement::Held { open: 1, because: Hold::InstanceHelperRuns.into() },
        "{spelled} names the same folder as {}",
        per_instance.display()
    );
    remove(world.root.as_path());
}

/// A batch Git cannot answer for is named,
/// not left out:
/// here one ordinary landed batch and another valid intent of the same
///  worktree whose repository is unavailable.
/// The first is finished,
/// the second is reported as unanswered with its journal,
///  and its intent stays on disk (nothing is discarded).
#[test]
fn a_batch_git_cannot_answer_for_is_named_and_stays_on_disk() {
    let (world, state) = world_with_earlier_journal("settle-unanswered", "state");
    let landed: CommitRequest = request(&world, "unanswered", "unanswered.txt", "unanswered\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    intent.seq = 2;
    intent.repository = world.root.join("repository-that-is-not-there");
    write_intent(world.context.journal.as_path(), &intent).expect("an intent whose repository is unavailable");
    let RequestedSettlement::Settled { batches, unanswered } = settle(&world, state.as_path(), empty_proc(&world).as_path()) else { panic!("settled") };
    assert_eq!(batches.iter().map(|batch| { return (batch.report.seq, batch.report.handled); }).collect::<Vec<_>>(), vec![(1, Handled::Finished)]);
    assert_eq!(unanswered, vec![UnansweredBatch { journal: world.context.journal.clone(), seq: 2 }]);
    assert!(batch_path(world.context.journal.as_path(), 2, OPEN_EXTENSION).exists(), "its intent stays");
    remove(world.root.as_path());
}

/// What:
///  A held batch the request settles completely is in the settled list,
///  with its branch moved and nothing left owed,
///  and not in the unanswered one.
/// Why:
///  `settle-earlier-batches` prints one line per settled batch and one per unanswered batch,
///  so a batch in neither list is printed nowhere:
///  the command would end with empty standard output and exit 0 after its recovery landed the batch and updated the index,
///  telling the person nothing about work that was done.
///  The settlement removes the held record when nothing is owed and marks the batch done by renaming its intent,
///  which is why the recovery cannot decide the batch from a record any more
///  (a Kimi K3.1 Pro review of the jam-fix integration,
/// its finding that a settled held batch is left out of the report).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = settleOnRequest(...); expect(answer.batches.map(b => [b.report.seq, b.report.handled])).toEqual([[1, 'finished']]);
/// ```
#[test]
fn a_held_batch_the_request_finishes_is_in_the_settled_list() {
    let (world, state) = world_with_earlier_journal("settle-held-finished", "state");
    let landed: CommitRequest = request(&world, "held-finished", "held-finished.txt", "held\n");
    // The branch already holds the batch's commit and its index update is owed: what a settlement killed between its branch move
    //  and the removal of its record leaves, and the record is what says a reference lock kept the batch off the branch before.
    land_with_failed_index(&world, 1, &[landed]);
    let intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("the open batch");
    let git_dir: PathBuf = world.repo.join(".git");
    let _held = hold(Holding {
        journal: world.context.journal.as_path(),
        repository: world.repo.as_path(),
        common_dir: git_dir.as_path(),
        git_dir: git_dir.as_path(),
        index: intent.index.as_path(),
        reference: &intent.reference,
        seq: 1,
        intent: &intent,
        locks: vec![git_dir.join("refs/heads/main.lock")],
        reflog: b"commit: the held edit".as_slice(),
    })
    .expect("the held record");
    let RequestedSettlement::Settled { batches, unanswered } = settle(&world, state.as_path(), empty_proc(&world).as_path()) else { panic!("settled") };
    assert_eq!(
        batches.iter().map(|batch| { return (batch.report.seq, batch.report.handled, batch.journal.clone()); }).collect::<Vec<_>>(),
        vec![(1, Handled::Finished, world.context.journal.clone())],
        "the batch the request finished is named, so the command prints it"
    );
    assert_eq!(unanswered, vec![], "nothing is left unexplained");
    assert!(!batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch is finished");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}
