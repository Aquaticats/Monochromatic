//! Controls for the unit-test fixtures:
//! where they live and that they go away however their test ends (issue 1809).

use super::*;

/// A test thread that panics after creating a fixture leaves nothing behind.
#[test]
fn a_fixture_is_removed_when_its_test_thread_panics() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let root: PathBuf = fixture("guard-panic");
        std::fs::write(root.join("file"), "content").expect("file");
        sender.send(root).expect("send the root");
        panic!("the test fails before it removes its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// A test thread that ends normally leaves nothing behind either,
/// with or without an explicit `remove`.
#[test]
fn a_fixture_is_removed_when_its_test_thread_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    std::thread::spawn(move || {
        let kept: PathBuf = fixture("guard-ends");
        let removed: PathBuf = fixture("guard-removed");
        remove(removed.as_path());
        sender.send(kept).expect("send the root");
        sender.send(removed).expect("send the root");
    })
    .join()
    .expect("the thread ends normally");
    for root in receiver.try_iter() {
        assert!(!root.exists(), "{} was left behind", root.display());
    }
}

/// Settling keeps a tree on request,
/// removes it otherwise,
/// and accepts a tree that is already gone.
#[test]
fn settling_keeps_removes_and_accepts_a_tree_that_is_gone() {
    let root: PathBuf = fixture("settle");
    std::fs::write(root.join("file"), "content").expect("file");
    settle_fixture(root.as_path(), true);
    assert!(root.join("file").exists(), "a kept fixture stays");
    settle_fixture(root.as_path(), false);
    assert!(!root.exists(), "a settled fixture goes");
    settle_fixture(root.as_path(), false);
}

/// How long a control gives the removal to return inside while a thread this process started is held.
/// Why:
///   A removal that waits at all cannot return before that thread is let go,
/// however slowly the host runs;
/// a removal that does not wait returns in microseconds.
/// The window is a second,
/// far above the microseconds and below a control's own patience,
/// and it is paid on every run of the control because a removal that waits uses all of it.
const WHILE_A_WRITER_IS_HELD: std::time::Duration = std::time::Duration::from_secs(1);

/// How long a control waits for its own pieces to settle once it has let them go.
const SETTLING: std::time::Duration = std::time::Duration::from_secs(30);

/// The ceiling the held note of the control below is granted:
/// short,
/// so that control ends in seconds.
const A_HELD_NOTES_CEILING: std::time::Duration = std::time::Duration::from_millis(400);

/// What:
///  A fixture is removed only after the threads this process started have ended.
/// Why:
///   A job `spawn_named` starts belongs to no caller:
///  the service starts the pool fill,
/// the repository scratch collection and the draw beside a landing and returns
///  (`service_server_land_group.rs`),
///  and the draw makes `monochromatic-git-policy-cli-shadow-drawn` under the repository's Git directory
///  after that caller returned.
///  It is what left
///  `a_landed_commit_holds_its_push_where_no_round_runs_it` panicking at
///  `remove only the fixture: Os { code: 39, kind: DirectoryNotEmpty }` in the gate of 2026-11-14.
///  The writer here is held,
/// so "while a thread this process started runs" is a state and not a duration;
/// the removal is given the window below to return inside,
/// which it cannot use while that thread is held.
#[test]
fn a_fixture_is_removed_after_the_threads_this_process_started_have_ended() {
    let root: PathBuf = fixture("guard-writer");
    std::fs::write(root.join("before"), "content").expect("a file the removal is about");
    let (reached_sent, reached): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let (let_go, held): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let (wrote_sent, wrote): (std::sync::mpsc::Sender<bool>, std::sync::mpsc::Receiver<bool>) = std::sync::mpsc::channel::<bool>();
    let written: PathBuf = root.join("after");
    let writer = crate::scoped_start::spawn_named("guard-writer", move || {
        let _said = reached_sent.send(());
        // Held until the control lets it go, so the tree stays whole until the removal has had its chance to run.
        let _held = held.recv();
        // What the writer found: a tree the removal waited for is whole, and one removed under it is gone (`ENOENT`).
        let _told = wrote_sent.send(std::fs::write(written.as_path(), b"content").is_ok());
    })
    .expect("a thread for the writer");
    reached.recv().expect("the writer has started and waits for the control");
    let (calling_sent, calling): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let (removed_sent, removed): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let removing: PathBuf = root.clone();
    let remover = std::thread::spawn(move || {
        // Said before the call, so the window below opens where the removal begins and not where its thread does.
        let _said = calling_sent.send(());
        remove(removing.as_path());
        let _told = removed_sent.send(());
    });
    calling.recv().expect("the removal is about to run");
    let returned_while_writing: bool = removed.recv_timeout(WHILE_A_WRITER_IS_HELD).is_ok();
    let _let_go = let_go.send(());
    let the_write_found_its_fixture: bool = wrote.recv_timeout(SETTLING).unwrap_or(false);
    remover.join().expect("the removal ends");
    writer.join().expect("the writer ends");
    assert!(!returned_while_writing, "the fixture was removed while a thread this process started was still writing into it");
    assert!(the_write_found_its_fixture, "the writer wrote into a tree the removal had already taken");
    assert!(!root.exists(), "the removal did not remove the fixture");
}

/// What:
///  A fixture is removed only after the note a landing handed to a thread of its own has ended.
/// Why:
///   `note_for_the_next_service_within` writes the note on a thread of its own (`work_within_reporting`)
///  and stops waiting for it at its ceiling,
///  so a store that gives no answer inside the ceiling leaves that thread writing
///  (`landing/service_claim_watch_note_within_a_ceiling.rs`).
///  Every counted job of a landing has ended by then,
///  because the note's thread is not one of them,
///  and the fixture goes out from under a writer that then makes its directories again:
///  the review of this branch reproduced exactly that
///  (`REPRO_REMOVE_RETURNED_WHILE_NOTE_WRITER_WAS_HELD: the late note writer recreated the tree: true`).
///  The note here is held at the fault point the landing reaches,
/// so "while the note writes" is a state and not a duration.
#[test]
fn a_fixture_is_removed_after_the_note_a_landing_left_behind_has_ended() {
    let root: PathBuf = fixture("guard-note-writer");
    let git_dir: PathBuf = root.join("repo.git");
    std::fs::create_dir_all(git_dir.join(crate::work_item::WORK_DIRECTORY)).expect("the work directory of the Git directory");
    let store: PathBuf = root.join("journal").join(crate::service_claim_watch::WATCH_DIRECTORY);
    let (_reached, let_go): (tokio::sync::oneshot::Receiver<()>, std::sync::mpsc::Sender<()>) = crate::service_fault::hooks::hold_the_point(crate::service_claim_watch::IN_THE_NEXT_SERVICE_NOTE);
    // Held on the thread that writes it, the note gives no answer within its ceiling and is left behind.
    let answered: crate::service_claim_watch_note_within_a_ceiling::NoteForTheNextService = crate::service_claim_watch_note_within_a_ceiling::note_for_the_next_service_within(A_HELD_NOTES_CEILING, store.as_path(), None, git_dir.as_path());
    assert_eq!(answered, crate::service_claim_watch_note_within_a_ceiling::NoteForTheNextService::NoAnswer, "a held note is left behind at its ceiling");
    let (calling_sent, calling): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let (removed_sent, removed): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let removing: PathBuf = root.clone();
    let remover = std::thread::spawn(move || {
        // Said before the call, so the window below opens where the removal begins and not where its thread does.
        let _said = calling_sent.send(());
        remove(removing.as_path());
        let _told = removed_sent.send(());
    });
    calling.recv().expect("the removal is about to run");
    let returned_while_the_note_was_held: bool = removed.recv_timeout(WHILE_A_WRITER_IS_HELD).is_ok();
    let _let_go = let_go.send(());
    remover.join().expect("the removal ends");
    assert!(!returned_while_the_note_was_held, "the fixture was removed while the note a landing left behind was still writing into it");
    assert!(!root.exists(), "the removal did not remove the fixture");
}

/// Fixtures are created beside the build output,
/// never in the system temporary directory,
/// unless the person moved the
/// root on purpose.
#[test]
fn the_fixture_root_is_not_the_system_temporary_directory() {
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let root: PathBuf = fixture_root();
    // The last component is this process's own directory, so the names under it that are not fixtures' own are the
    //  process's: the build output above `test-fixtures` is shared by every checkout that builds here.
    assert!(root.parent().is_some_and(|above| { return above.ends_with("test-fixtures"); }), "{}", root.display());
    assert!(root.ends_with(format!("process-{}", std::process::id()).as_str()), "{}", root.display());
    // The system temporary directory is `TMPDIR` when set and `/tmp` otherwise; reading the variable compares against it
    // without calling the temporary-directory function that the lint configuration bans.
    let mut system: Vec<PathBuf> = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
    system.extend(std::env::var_os("TMPDIR").map(PathBuf::from));
    for directory in system {
        assert!(!root.starts_with(&directory), "{} is under the system temporary directory {}", root.display(), directory.display());
    }
}

/// A process a test starts cannot write a core dump:
/// tests make services abort or kill them on purpose,
/// and every dump costs
///  the system's core-dump handler processor time and disk writes on a disk other sessions share (several hundred dumps of
///  `cli-git-service` from test runs on 2026-11-03).
/// The limit is the harness's,
/// set where fixtures are made;
/// the product
///  sets none.
#[test]
fn a_process_a_test_starts_may_not_write_a_core_dump() {
    let _root: PathBuf = fixture("core-limit");
    let output = std::process::Command::new("sh").args(["-c", "ulimit -c; ulimit -H -c"]).output().expect("sh runs");
    assert_eq!(String::from_utf8_lossy(&output.stdout).split_whitespace().collect::<Vec<&str>>(), vec!["0", "0"]);
}

/// A wait whose event has already happened returns at once,
/// without looking at the run.
/// How long the call took cannot say that:
/// a thread the scheduler holds just before it makes a wait
///  that looked slow,
/// and one it holds just after a sleep makes one that slept look quick,
/// so the control reads the
///  looks this thread took.
#[test]
fn a_wait_whose_event_has_happened_returns_at_once() {
    let before: u64 = crate::run_progress::looks_taken_by_this_thread();
    assert!(wait_until(&|| { return true; }));
    assert_eq!(
        crate::run_progress::looks_taken_by_this_thread(),
        before,
        "the wait looked at the run although its event had already happened"
    );
}

/// A wait whose event never happens and whose run shows nothing ends with the window,
/// and says it gave up because the run stopped moving.
#[test]
fn a_wait_gives_up_when_the_run_shows_nothing() {
    let window: std::time::Duration = std::time::Duration::from_millis(300);
    let began: std::time::Instant = std::time::Instant::now();
    let held: bool = wait_until_for(WaitingFor { window, done: &|| { return false; } });
    assert!(!held, "an event that never happened was reported as held");
    assert!(began.elapsed() >= window, "the wait ended before its window passed: {:?}", began.elapsed());
}

/// How much of its window a wait is held for here:
/// two windows' worth of looks,
/// worked out from the window and the interval between looks.
/// Why:
///   The event is delivered by the wait's own predicate after this many looks,
/// so the control holds the worker working for
///  longer than the window without depending on the clock:
/// a thread the scheduler holds only delays the looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const looksPastTheWindow = 2 * (windowMs / lookEveryMs);
/// ```
fn looks_past_the_window(window: std::time::Duration) -> usize {
    let per_window: u128 = window.as_millis() / WAIT_LOOK_EVERY.as_millis();
    return usize::try_from(per_window * 2).expect("a window that fits a count of looks");
}

/// A wait outlives many windows while the run keeps working,
/// so a slow event is waited for however long it takes.
/// The worker stops only once the wait has looked at the run for two of its windows,
/// which is what the
///  predicate counts:
/// the control cannot pass with a worker that finished before the wait began.
#[test]
fn a_wait_outlives_its_window_while_the_run_keeps_working() {
    let window: std::time::Duration = std::time::Duration::from_millis(200);
    let working: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let still_working: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&working);
    let worker = std::thread::spawn(move || {
        use std::io::Write as _;
        let mut sink: std::fs::File = std::fs::OpenOptions::new().write(true).open("/dev/null").expect("a sink");
        while still_working.load(std::sync::atomic::Ordering::SeqCst) {
            sink.write_all(&[b'x'; 4096]).expect("a write");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    let looks: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let held_for: usize = looks_past_the_window(window);
    let stop_after = std::sync::atomic::AtomicBool::new(false);
    let began: std::time::Instant = std::time::Instant::now();
    let held: bool = wait_until_for(WaitingFor {
        window,
        done: &|| {
            if looks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 >= held_for {
                stop_after.store(true, std::sync::atomic::Ordering::SeqCst);
                working.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            return stop_after.load(std::sync::atomic::Ordering::SeqCst);
        },
    });
    working.store(false, std::sync::atomic::Ordering::SeqCst);
    worker.join().expect("the worker ends");
    assert!(held, "an event the working run delivered was given up on");
    assert!(began.elapsed() >= window, "the wait ended inside its window: {:?}", began.elapsed());
}

/// How long a control waits for two of its waits to give up before it fails the control.
/// Why:
///   A wait the run keeps alive never ends on its own,
/// so the control needs a bound of its own to report the
///  finding instead of leaving its threads behind.
const GIVE_UP_PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// How many waits the control below starts at once.
const WAITS: usize = 2;

/// Two waits for events that never happen end with their own windows:
/// neither takes the other's readings of the run for the run's work.
#[test]
fn two_waits_do_not_keep_each_other_alive() {
    let window: std::time::Duration = std::time::Duration::from_millis(300);
    let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (told, heard) = std::sync::mpsc::channel::<bool>();
    let mut waits: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for _ in 0..WAITS {
        let given_up: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&stop);
        let tell = told.clone();
        waits.push(std::thread::spawn(move || {
            let held: bool = wait_until_for(WaitingFor { window, done: &|| { return given_up.load(std::sync::atomic::Ordering::SeqCst); } });
            let _sent = tell.send(held);
        }));
    }
    drop(told);
    let began: std::time::Instant = std::time::Instant::now();
    let mut answers: Vec<bool> = Vec::new();
    while answers.len() < WAITS && began.elapsed() < GIVE_UP_PATIENCE {
        if let Ok(held) = heard.recv_timeout(WAIT_LOOK_EVERY) {
            answers.push(held);
        }
    }
    // Ends a wait that the run kept alive, so the control reports its finding rather than leaving its threads behind.
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    for wait in waits {
        wait.join().expect("the waiting thread ends");
    }
    assert_eq!(answers, vec![false; WAITS], "{WAITS} waits for events that never happen kept each other alive for {:?} by counting each other's readings as the run's work", began.elapsed());
}

/// The window a control's wait uses is shorter than the watchdog's,
/// so a control reports why it gave up before the run is ended.
#[test]
fn the_controls_window_is_below_the_watchdogs() {
    assert!(STILL_WORKING_WINDOW < QUIET_WINDOW, "{STILL_WORKING_WINDOW:?} is not below {QUIET_WINDOW:?}");
}

/// What:
///  A wait whose own predicate does the work the event is made of is waited for,
///  and the watchdog still sees that work.
/// Why:
///   The registration that keeps a waiter's `/proc` reads from counting as work must not hide the work the predicate
/// itself does:
/// the controls that make a retry or a round inside the predicate do real disk work on the waiting thread,
/// and a
///  registration that covered the predicate would hide the very `D` state and bytes `run_progress` reads as work.
/// This one writes 64 KiB at every call and delivers its event on the eighth,
/// which is several windows of a 300 ms
///  window later,
/// so a wait that hid the writes gives up and a wait that sees them does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = waitUntilFor({ window: 300, done: () => { write(64 * 1024); sleep(40); return ++calls >= 8 } });
/// ```
#[test]
fn a_wait_whose_predicate_works_is_not_abandoned() {
    let window: std::time::Duration = std::time::Duration::from_millis(300);
    // The event is delivered after twice as many calls as a window holds looks, each of them doing real work on this thread.
    let calls_to_deliver: usize = 8;
    let calls: std::cell::Cell<usize> = std::cell::Cell::new(0);
    let began: std::time::Instant = std::time::Instant::now();
    let held: bool = wait_until_for(WaitingFor {
        window,
        done: &|| {
            use std::io::Write as _;
            let mut sink: std::fs::File = std::fs::OpenOptions::new().write(true).open("/dev/null").expect("a sink");
            sink.write_all(&[b'w'; 64 * 1024]).expect("a write of the predicate's own");
            std::thread::sleep(std::time::Duration::from_millis(40));
            calls.set(calls.get() + 1);
            return calls.get() >= calls_to_deliver;
        },
    });
    assert!(held, "the wait gave up although its predicate was doing the work the event is made of");
    assert!(calls.get() >= calls_to_deliver, "the event was delivered: {} calls", calls.get());
    assert!(began.elapsed() >= window, "the event came later than the window: {:?}", began.elapsed());
}

/// What:
///  A directory this user may not write inside a fixture does not outlive the test that made it,
///  whether the test returns or panics.
/// Why:
///   Issue 6282:
/// the fixtures of `rollback-read-only`,
/// `remove-old-read-only`,
/// `migrate-unreadable-dir` and `sync-unreadable-dir`
/// stayed in the build tree of this package,
/// and a build-tree sweeper could not delete the trees they sat in.
/// A teardown that gives the permission back before it removes the tree is what the fix is,
/// so this control ends both ways.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a closed fixture directory goes when its test ends', ...)
/// ```
#[cfg(unix)]
#[test]
fn a_closed_fixture_directory_goes_when_its_test_returns_or_panics() {
    use std::os::unix::fs::PermissionsExt as _;
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let root: PathBuf = fixture("closed-returning");
        let closed: PathBuf = root.join("checkout/ignored/closed");
        std::fs::create_dir_all(closed.join("inner")).expect("closed directories");
        std::fs::write(closed.join("inner").join("file"), "content").expect("file");
        std::fs::set_permissions(&closed, std::fs::Permissions::from_mode(0o555)).expect("mode");
        std::fs::set_permissions(closed.join("inner"), std::fs::Permissions::from_mode(0o500)).expect("mode");
        sender.send(root).expect("send the root");
        panic!("the test fails with a read-only directory in its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// A fixture whose directories are closed to this user is removed by the teardown,
/// at every mode and depth the wrapper's controls use.
#[cfg(unix)]
#[test]
fn the_teardown_gives_the_owner_back_the_permission_and_removes_the_tree() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("closed-modes");
    for (name, mode) in [("searchable", 0o500), ("readable", 0o555), ("closed", 0o000)] {
        let directory: PathBuf = root.join(name).join("below");
        std::fs::create_dir_all(&directory).expect("directory");
        std::fs::write(directory.join("file"), "content").expect("file");
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(mode)).expect("mode");
    }
    settle_fixture(root.as_path(), false);
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// Giving the permission back changes the owner's three bits and nothing else,
/// and never follows a symbolic link out of the tree.
#[cfg(unix)]
#[test]
fn restoring_the_owner_access_keeps_group_and_other_bits_and_leaves_a_link_s_target_alone() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("closed-bits");
    // Another fixture, so it is outside the tree whose directories are given their permission back.
    let outside: PathBuf = fixture("closed-bits-outside");
    std::fs::create_dir_all(&outside).expect("outside");
    std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(0o500)).expect("mode");
    let inside: PathBuf = root.join("inside");
    std::fs::create_dir_all(&inside).expect("inside");
    std::fs::set_permissions(&inside, std::fs::Permissions::from_mode(0o540)).expect("mode");
    std::os::unix::fs::symlink(&outside, root.join("to-outside")).expect("link");
    restore_owner_access(root.as_path());
    assert_eq!(std::fs::metadata(&inside).expect("inside").permissions().mode() & 0o7777, 0o740, "only this user's read, write and search were added");
    assert_eq!(std::fs::metadata(&outside).expect("outside").permissions().mode() & 0o7777, 0o500, "a directory outside the tree a link names keeps its mode");
}

/// What:
///  The folder a runtime directory stands in is the longest place this account offers that stays inside the room a
///  control asks for:
/// a limit of exactly its own length keeps that very place,
/// byte for byte,
/// and a limit one byte under it moves the folder to a shorter place.
/// Why:
///   A runtime directory of a fixed length leaves whatever is past its folder for its name,
/// and how much that is depends on this account and this process:
/// the gate's container leaves seven bytes under `$HOME/temp/agent` where eight were needed,
/// so the control that failed there passed here.
/// A folder chosen once and never shortened is that difference,
/// so the shortening is what this holds,
/// and the place that fits its limit to the byte is the half a `>` where `>=` belongs would refuse.
#[test]
fn the_folder_of_a_runtime_directory_is_shortened_until_the_room_a_name_needs_is_there() {
    let prefix: &str = "zc";
    let longest_place: PathBuf = runtime_directory_parent(prefix, usize::MAX);
    assert!(longest_place.is_dir(), "the longest place is made: {}", longest_place.display());
    let room_it_takes: usize = longest_place.as_os_str().len();
    let exact: PathBuf = runtime_directory_parent(prefix, room_it_takes);
    assert_eq!(
        exact.as_os_str(),
        longest_place.as_os_str(),
        "a limit of {room_it_takes} bytes keeps {}, the folder that fills it to the byte",
        longest_place.display()
    );
    let one_byte_less: usize = room_it_takes - 1;
    let shorter: PathBuf = runtime_directory_parent(prefix, one_byte_less);
    assert!(shorter.is_dir(), "the shorter place is made: {}", shorter.display());
    assert!(
        shorter.as_os_str().len() < room_it_takes,
        "one byte less of room than {} has moves the runtime directory to {}, which is not shorter",
        longest_place.display(),
        shorter.display()
    );
    // The folder it makes stays inside the room it is asked for, which is the contract both calls rest on: the first
    // fills its limit to the byte, and the second is held against its own limit here.
    assert_eq!(exact.as_os_str().len(), room_it_takes, "the folder that fits a limit of {room_it_takes} bytes is that long");
    assert!(
        shorter.as_os_str().len() <= one_byte_less,
        "a limit of {one_byte_less} bytes is met by {}, which is {} bytes",
        shorter.display(),
        shorter.as_os_str().len()
    );
    for place in [&longest_place, &shorter] {
        std::fs::remove_dir(place.as_path()).expect("the place this control made");
    }
}

/// What:
///  A runtime directory is named for its tag where the room holds the tag and for a digest of it where it does not,
///  and it stands in the folder it was given.
/// Why:
///   The name is what the fixed length is spent on,
/// and a name shorter or longer than the room would put the directory off the boundary the controls are about:
/// a control of the longest runtime directory a socket path carries would pass a byte early or fail a byte late.
/// The two sides are read by the bytes each name is made of,
/// never by a length the two can share:
/// a tag that fills the room exactly is the tag's own bytes,
/// and one byte less of room is the leading bytes of its digest.
#[test]
fn a_runtime_directory_is_named_for_its_tag_or_for_a_digest_of_it() {
    let parent: PathBuf = runtime_directory_parent("zd", usize::MAX);
    // A tag longer than a digest, so the room can be the tag's own length and, one byte under it, the digest's.
    let tag: &str = "at-the-longest";
    for (room, the_room_holds_the_tag) in [
        (tag.len(), true),
        (tag.len() - 1, false),
        (room_a_runtime_name_needs(tag), false),
    ] {
        let length: usize = parent.as_os_str().len() + 1 + room;
        let runtime: PathBuf = runtime_directory_under(parent.as_path(), length, tag);
        assert_eq!(runtime.as_os_str().len(), length, "the directory is exactly as long as the control asks");
        assert_eq!(runtime.parent(), Some(parent.as_path()), "the directory stands in the folder it was given");
        let name: &OsStr = runtime.file_name().expect("a runtime directory has a name");
        assert_eq!(name.len(), room, "the name fills the room the folder was asked for: {}", runtime.display());
        if the_room_holds_the_tag {
            assert_eq!(name, OsStr::new(tag), "a room of {room} bytes holds the tag, so the name is the tag's own bytes");
        } else {
            let digest: String = crate::service_instance::digest_of_bytes(tag.as_bytes());
            assert_eq!(
                name.to_string_lossy().into_owned(),
                String::from(digest.get(..room).expect("the digest is longer than the room")),
                "a room of {room} bytes does not hold the tag, so the name is the leading bytes of its digest"
            );
            assert_ne!(name, OsStr::new(tag), "a room that does not hold the tag does not name the directory for it");
        }
    }
    std::fs::remove_dir_all(parent.as_path()).expect("the folder this control made");
}

/// What:
///  A runtime directory whose name has less room than the name needs is refused,
/// naming the folder and the room.
/// Why:
///   The alternative is a control that builds a name the length cannot hold,
/// which is what a truncated digest or a padded tag would be:
/// a directory off the boundary proves nothing about the boundary,
/// so the refusal has to be the answer rather than a name of the wrong length.
#[test]
#[should_panic(expected = "too little to name a control")]
fn a_runtime_directory_with_no_room_for_its_name_is_refused() {
    let parent: PathBuf = runtime_directory_parent("ze", usize::MAX);
    let tag: &str = "longest";
    let room: usize = room_a_runtime_name_needs(tag) - 1;
    let _runtime: PathBuf = runtime_directory_under(parent.as_path(), parent.as_os_str().len() + 1 + room, tag);
}

/// What:
///  A room no place this account offers can give is refused,
/// naming how much room was needed and every place tried.
/// Why:
///   The refusal is what a person reads when a control cannot run where they are,
/// and it has to say what to change rather than fail as a name that does not fit its own length.
#[test]
#[should_panic(expected = "no folder this account offers leaves room for a runtime directory's name")]
fn a_folder_with_room_for_no_runtime_directory_name_is_refused_naming_every_place() {
    // The shortest path a folder could have is the root, so nothing this account offers is this short.
    let no_place_is_this_short: usize = Path::new("/").as_os_str().len();
    let _parent: PathBuf = runtime_directory_parent("zn", no_place_is_this_short);
}

/// What:
///  The folder a runtime directory was to stand in goes away when the test thread that made it ends,
///  in the longest place this account offers and in the shorter one a tighter room moves it to.
/// Why:
///   The helper picks the folder from places this machine shares with other tests and with people,
/// and the room a name needs decides which of them it picks:
/// with a home directory of two hundred bytes the shortest is the session's runtime directory.
/// Before this control,
/// nineteen passing controls left `/r/temp/agent/ze1000007`,
/// `pr1000028` and `sa1000013`
/// behind,
/// and the same under `$HOME/temp` and under `$XDG_RUNTIME_DIR`,
/// so a passing control littered a place it shares and a later run could meet what it left.
/// The folder is the owner of everything the helper puts in it,
/// which is why the thread that made it is what this control holds to.
#[test]
fn the_folder_a_runtime_directory_stands_in_goes_when_its_test_thread_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<PathBuf>>();
    std::thread::spawn(move || {
        let mut made: Vec<PathBuf> = Vec::new();
        let longest: PathBuf = runtime_directory_parent("zg", usize::MAX);
        made.push(longest.clone());
        // One byte less of room than the longest place takes moves the folder to the next place this account offers.
        made.push(runtime_directory_parent("zg", longest.as_os_str().len() - 1));
        sender.send(made).expect("the folders this thread made");
    })
    .join()
    .expect("the thread ends");
    for folder in receiver.try_iter().flatten() {
        assert!(!folder.exists(), "{} was left behind in a place this machine shares", folder.display());
    }
}
