//! What:
//!  Controls of the one-shot receipt flusher on disposable lanes: `flusher.lock` contention, the recheck sweep after the lock is released,
//!  the error record that names every unforced receipt, and a launched flusher's error record.
//! Why:
//!  At most one flusher runs per lane, a client that found the lock held must still be covered by the holder, and a forcing error must
//!  not leave a receipt behind an exited flusher without a record (the model's `a_forcing_error_records_every_unforced_receipt_and_the_next_owner_retries_each`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a second flusher finds the lock held and forces nothing', ...);
//! ```

use crate::commit_helper_launch_tests::flusher_launch_code;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_flusher::{FlushError, Flushed, FlusherLock, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane, flush_lane_with, launch_flusher};
use git_policy_cli::commit_helper_launch::helpers_forked;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a test waits for a launched flusher; the work is a few file operations.
const FLUSHER_WAIT: Duration = Duration::from_secs(10);

/// A lane with a receipt directory holding `names` as empty files.
fn lane_with(label: &str, names: &[&str]) -> PathBuf {
    let lane: PathBuf = scratch(label);
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    for name in names {
        fs::write(lane.join(RECEIPT_DIR).join(name), b"receipt").expect("receipt");
    }
    return lane;
}

/// Names (last path components) in call order.
fn leaves(calls: &[PathBuf]) -> Vec<String> {
    return calls.iter().map(|path| { return path.file_name().expect("a leaf").to_string_lossy().into_owned(); }).collect();
}

/// Two opens of the lock file conflict: the second is refused while the first is held and succeeds after it is dropped.
#[test]
fn the_flusher_lock_admits_one_holder_at_a_time() {
    let lane: PathBuf = lane_with("flusher-lock", &[]);
    let first = FlusherLock::try_acquire(&lane).expect("lock").expect("the first takes it");
    assert!(FlusherLock::try_acquire(&lane).expect("lock").is_none(), "a second holder is refused without waiting");
    drop(first);
    assert!(FlusherLock::try_acquire(&lane).expect("lock").is_some(), "the lock is free after the holder drops it");
}

/// A flusher that finds the lock held forces nothing and says so; the receipt stays for the holder's recheck sweep.
#[test]
fn a_second_flusher_finds_the_lock_held_and_forces_nothing() {
    let lane: PathBuf = lane_with("flusher-held", &["r1"]);
    let _holder = FlusherLock::try_acquire(&lane).expect("lock").expect("the holder");
    let forced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let outcome: Flushed = flush_lane_with(&lane, &mut |path: &Path| { forced.borrow_mut().push(path.to_path_buf()); return Ok(()); }, &mut || {}).expect("flush");
    assert_eq!(outcome, Flushed::Held);
    assert!(forced.borrow().is_empty(), "{:?}", forced);
}

/// The release window: a receipt created after the last empty sweep and before the lock is released is found by the sweep without the
/// lock, which takes the lock again and forces it.
#[test]
fn a_receipt_created_in_the_release_window_is_forced_by_the_recheck_sweep() {
    let lane: PathBuf = lane_with("flusher-recheck", &["r1"]);
    let forced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let window: PathBuf = lane.join(RECEIPT_DIR).join("r2");
    let outcome: Flushed = flush_lane_with(
        &lane,
        &mut |path: &Path| { forced.borrow_mut().push(path.to_path_buf()); return Ok(()); },
        &mut || { fs::write(&window, b"receipt").expect("a client's receipt"); },
    )
    .expect("flush");
    assert_eq!(outcome, Flushed::Finished { forced: 2 });
    let names: Vec<String> = leaves(&forced.borrow());
    assert!(names.iter().any(|name| { return name == "r2"; }), "the window's receipt was forced: {:?}", names);
}

/// The release window with a second flusher: another flusher took the lock between the release and the recheck, so this one hands over
/// instead of forcing (the other one carries the receipt).
#[test]
fn a_recheck_that_finds_work_and_the_lock_taken_hands_over() {
    let lane: PathBuf = lane_with("flusher-handover", &["r1"]);
    let taker: RefCell<Option<FlusherLock>> = RefCell::new(None);
    let window: PathBuf = lane.join(RECEIPT_DIR).join("r2");
    let outcome: Flushed = flush_lane_with(
        &lane,
        &mut |_: &Path| { return Ok(()); },
        &mut || {
            fs::write(&window, b"receipt").expect("a client's receipt");
            *taker.borrow_mut() = FlusherLock::try_acquire(&lane).expect("lock");
        },
    )
    .expect("flush");
    assert!(taker.borrow().is_some(), "the other flusher took the free lock in the window");
    assert_eq!(outcome, Flushed::Handed { forced: 1 });
}

/// A forcing error: the snapshot's first receipt's file is forced, the second fails, and a receipt a client created meanwhile joined the
/// lock holder. The record names all three: the first receipt's name is not forced either, because the receipt directory was never forced
/// (review 9a section 3E), so a power loss could still lose it. The flusher ends with the error.
#[test]
fn a_forcing_error_records_every_receipt_whose_name_is_not_forced_the_joiner_included() {
    let lane: PathBuf = lane_with("flusher-error", &["a", "b"]);
    let joiner: PathBuf = lane.join(RECEIPT_DIR).join("c");
    let result = flush_lane_with(
        &lane,
        &mut |path: &Path| {
            if path.file_name().is_some_and(|leaf| { return leaf == "b"; }) {
                fs::write(&joiner, b"receipt").expect("the joiner's receipt");
                return Err(io::Error::other("the disk refused"));
            }
            return Ok(());
        },
        &mut || {},
    );
    assert!(matches!(result, Err(FlushError::Forcing(_))), "{:?}", result);
    let record: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the error record");
    let named: Vec<&str> = record.lines().map(|line| { return line.split('\t').next().expect("a name"); }).collect();
    assert_eq!(named, ["a", "b", "c"], "{}", record);
}

/// The receipts in the lane of the launched-flusher control: `a`, the dangling `b` and `c`; the record is complete when it names all.
const RECEIPTS_OF_THE_LAUNCHED_LANE: usize = 3;

/// The launched flusher end to end: a receipt that cannot be forced (a dangling link) makes the helper record every unforced receipt, and
/// the helper releases the lock when it ends.
#[test]
fn a_launched_flusher_records_a_forcing_error_and_releases_the_lock() {
    let lane: PathBuf = lane_with("flusher-launched", &["a", "c"]);
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("b")).expect("a dangling receipt");
    let launch_lane: PathBuf = lane.clone();
    assert_eq!(in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&launch_lane)); }), Some(0), "the launch from a single-threaded process");
    let deadline: Instant = Instant::now() + FLUSHER_WAIT;
    let record: String = loop {
        if let Ok(text) = fs::read_to_string(lane.join(RECEIPT_ERRORS))
            && text.lines().count() >= RECEIPTS_OF_THE_LAUNCHED_LANE
        {
            break text;
        }
        assert!(Instant::now() < deadline, "no error record within {:?}", FLUSHER_WAIT);
        std::thread::sleep(Duration::from_millis(5));
    };
    let named: Vec<&str> = record.lines().map(|line| { return line.split('\t').next().expect("a name"); }).collect();
    assert_eq!(named, ["a", "b", "c"], "{}", record);
    let freed: Instant = Instant::now() + FLUSHER_WAIT;
    loop {
        if FlusherLock::try_acquire(&lane).expect("lock").is_some() {
            break;
        }
        assert!(Instant::now() < freed, "the helper still holds the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The real forcing on a healthy lane finishes with nothing recorded.
#[test]
fn a_healthy_lane_is_flushed_and_nothing_is_recorded() {
    let lane: PathBuf = lane_with("flusher-healthy", &["a", "b"]);
    assert_eq!(flush_lane(&lane).expect("flush"), Flushed::Finished { forced: 2 });
    assert!(!lane.join(RECEIPT_ERRORS).exists());
}

/// How many clients launch a flusher in the many-receipts controls.
const CLIENTS: usize = 4;

/// A receipt that is a FIFO: forcing it blocks in `open` until a writer exists, which holds a flusher (and its lock) alive on demand.
fn fifo_receipt(lane: &Path, name: &str) {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(lane.join(RECEIPT_DIR).join(name).as_os_str().as_bytes()).expect("a path without NUL");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0, "mkfifo");
}

/// Waits until the lane's flusher lock is free (the helper ended).
fn wait_for_lock_free(lane: &Path) {
    let freed: Instant = Instant::now() + FLUSHER_WAIT;
    while FlusherLock::try_acquire(lane).expect("lock").is_none() {
        assert!(Instant::now() < freed, "the helper still holds the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Review 9a section 3C: the lock bounded the lock holders, not the helpers: every client forked a helper before trying the lock, so many
/// receipts meant many live helpers. With the lock held by a flusher, clients that find it held fork nothing.
#[test]
fn clients_that_find_the_flusher_lock_held_start_no_helper() {
    let lane: PathBuf = lane_with("flusher-held-launch", &["r1"]);
    let holder = FlusherLock::try_acquire(&lane).expect("lock").expect("the holder");
    let launch_lane: PathBuf = lane.clone();
    let forked: Option<i32> = in_single_threaded_process(move || {
        for _ in 0..CLIENTS {
            if launch_flusher(&launch_lane).is_err() {
                return 99;
            }
        }
        return helpers_forked() as i32;
    });
    drop(holder);
    assert_eq!(forked, Some(0), "no client forked a helper while the lock was held");
}

/// Many clients with a free lock and a flusher that is busy: one helper is forked in all, and it is the one that holds the lock.
#[test]
fn many_clients_start_one_helper_and_it_holds_the_lock() {
    let lane: PathBuf = lane_with("flusher-many-launch", &[]);
    fifo_receipt(&lane, "r1");
    let launch_lane: PathBuf = lane.clone();
    let forked: Option<i32> = in_single_threaded_process(move || {
        for _ in 0..CLIENTS {
            if launch_flusher(&launch_lane).is_err() {
                return 99;
            }
        }
        return helpers_forked() as i32;
    });
    let holds: bool = FlusherLock::try_acquire(&lane).expect("lock").is_none();
    // A writer lets the helper's `open` of the FIFO return; forcing a FIFO then fails and the helper ends.
    let opened = fs::OpenOptions::new().read(true).write(true).open(lane.join(RECEIPT_DIR).join("r1"));
    wait_for_lock_free(&lane);
    drop(opened);
    assert_eq!(forked, Some(1), "one helper for all the clients");
    assert!(holds, "the helper holds the lock while it works");
}

/// Review 10a section 3C: a launch from a process whose descriptor 0 is closed. The lock opens as descriptor 0, the helper redirects its
/// standard streams over it, and its lock description is gone while it believes it holds the lock. The lock is moved above the standard
/// streams when it is taken, so the helper holds it for as long as it works.
#[test]
fn a_launch_with_descriptor_zero_closed_leaves_the_helper_holding_the_lock() {
    let lane: PathBuf = lane_with("flusher-closed-zero", &[]);
    fifo_receipt(&lane, "r1");
    let launch_lane: PathBuf = lane.clone();
    let launched: Option<i32> = in_single_threaded_process(move || {
        // SAFETY: closing descriptor 0 of this one-thread copy of the test process; nothing else reads it.
        unsafe { libc::close(0) };
        return flusher_launch_code(launch_flusher(&launch_lane));
    });
    let holds: bool = FlusherLock::try_acquire(&lane).expect("lock").is_none();
    let opened = fs::OpenOptions::new().read(true).write(true).open(lane.join(RECEIPT_DIR).join("r1"));
    wait_for_lock_free(&lane);
    drop(opened);
    assert_eq!(launched, Some(0));
    assert!(holds, "the helper's lock description was overwritten by its standard-stream setup");
}

/// Review 11 section 3B: `FlusherLock::try_acquire` opened the lock, took its raw descriptor and then failed to move it above the standard
/// streams (`EMFILE`), and the descriptor had no owner. A failed acquisition leaves no descriptor open.
#[test]
fn a_lock_acquisition_whose_descriptor_move_fails_leaks_no_descriptor() {
    let lane: PathBuf = lane_with("flusher-lock-emfile", &[]);
    let lane_for_child: PathBuf = lane.clone();
    let code: Option<i32> = in_single_threaded_process(move || {
        let mut original = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `getrlimit` and `setrlimit` take a pointer to a live local `rlimit`; `close` takes plain integers.
        let prepared: bool = unsafe {
            libc::getrlimit(libc::RLIMIT_NOFILE, &raw mut original) == 0
                && libc::close(0) == 0
                && libc::close(1) == 0
                && libc::close(2) == 0
                && libc::setrlimit(libc::RLIMIT_NOFILE, &libc::rlimit { rlim_cur: 3, rlim_max: original.rlim_max }) == 0
        };
        if !prepared {
            return 99;
        }
        let acquired = FlusherLock::try_acquire(&lane_for_child);
        // SAFETY: restores the limit read above.
        unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &original) };
        if acquired.is_ok() {
            return 40;
        }
        let open = std::fs::read_dir("/proc/self/fd").map_or(usize::MAX, |entries| { return entries.count(); });
        // The listing holds one descriptor of its own.
        return if open == 1 { 0 } else { 41 };
    });
    assert_eq!(code, Some(0), "the failed acquisition did not fail (40) or left a descriptor open (41)");
}
