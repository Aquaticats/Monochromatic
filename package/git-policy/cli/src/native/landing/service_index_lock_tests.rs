//! Controls for the service's hold on the index lock:
//! exclusive creation with the marker in place,
//! a wait that ends when the
//!  holder lets go and is not woken by anything else,
//! the wait for a holder proven alive or one nothing proves alive,
//!  which ends at its ceiling,
//! release by rename,
//! and
//!  the removal of a stale lock only when it is the service's own and its holder is gone.

use super::*;

/// A function-like value with the right shape is an announcer in these controls:
/// the shipped callers write a small struct that holds the state.
impl<Say: Fn(Announce)> Announcer for Say {
    fn announce(&self, note: Announce) {
        self(note);
    }
}
use crate::process_identity::{ProcessIdentityError, process_birth_identity};
use crate::service_index_marker::own_identity;
use crate::service_index_placement::{place_marked, rename_noreplace};
use crate::service_index_staging::stage_marker;
use crate::service_index_marker::{HolderState, ReaderView, check_procfs_view, holder_is_gone, holder_is_running, holder_state_in, identity_in_view, marker_of, pid_of_self_stat};
use crate::test_support::{fixture, remove, wait_until};
use std::time::Instant;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// The marker this process writes for batch `seq`.
fn own_marker(seq: u64) -> String {
    return marker_text(seq, &own_identity).expect("this process has an identity");
}

/// What:
///  Lets a holder the control made go,
/// however the control ends.
/// Why:
///   Each control below spawns the holder and then joins it;
/// a wait that says nothing never signals it,
/// so the join would wait out the holder's own sixty-second bound and a failing control would take a minute to
///  report.
struct LetsGoOnEveryPath<'signal>(&'signal std::sync::mpsc::Sender<()>);

impl Drop for LetsGoOnEveryPath<'_> {
    /// Tell the holder the wait is over.
    fn drop(&mut self) {
        let _sent = self.0.send(());
    }
}

/// The number of a process that has exited and been reaped,
/// so nothing runs under it.
fn dead_pid() -> i64 {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return pid;
}

/// The marker text of a service holder `pid` born as `birth`.
fn marker_of_holder(seq: u64, pid: i64, birth: &str) -> String {
    return format!("{LOCK_MARKER}{}{seq} {pid} {birth}\n", crate::service_index_marker::MARKER_VERSION);
}

/// The names in the index's directory other than the index and its lock.
fn leftovers(index: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(index.parent().expect("directory"))
        .expect("listing")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != "index" && name != "index.lock"; })
        .collect();
    names.sort();
    return names;
}

/// The lock is created exclusively with this process's marker,
/// a second attempt fails while it is held (its holder runs),
///  and the release moves the new index over the real one and leaves no lock and no staging file.
#[test]
fn the_lock_is_exclusive_and_its_release_installs_the_new_index() {
    let (root, index) = index_in("index-lock-release");
    let lock = acquire(&index, 7, &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(7));
    assert!(parse_marker(own_marker(7).as_str()).is_some_and(|marked| { return marked.holder.is_some(); }), "the marker names its holder");
    assert!(try_acquire(&index, 70).is_none(), "a live holder's lock is not taken over");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    lock.release_with(&built).expect("released");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists() && !built.exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// The release installs the built file itself by one rename (the index is the built file's inode afterwards),
/// so the marked
///  lock stands until the new index is in place;
/// copying the index over the lock first is what left a lock without its marker.
#[test]
fn the_release_renames_the_built_index_into_place_before_the_lock_goes() {
    use std::os::unix::fs::MetadataExt;
    let (root, index) = index_in("index-lock-rename");
    let lock = acquire(&index, 8, &|_| {}).expect("taken");
    let built: PathBuf = index.with_file_name("index.cli-git-8");
    std::fs::write(&built, "new index").expect("built");
    let built_inode: u64 = std::fs::metadata(&built).expect("built").ino();
    lock.release_with(&built).expect("released");
    assert_eq!(std::fs::metadata(&index).expect("index").ino(), built_inode, "installed by rename, not by a copy");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists() && !built.exists());
    remove(root.as_path());
}

/// Staging files a killed attempt left are removed once the lock is taken:
/// the fixed name an earlier release used and one
///  whose writer is gone;
/// a live writer's staging file stays.
/// The new lock carries its own marker.
#[test]
fn staging_files_of_killed_attempts_are_removed_and_a_live_writers_stays() {
    let (root, index) = index_in("index-lock-staged");
    let lock_path: PathBuf = lock_path_of(&index);
    let fixed: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}", lock_path.display()));
    std::fs::write(&fixed, "cli-git-service 41\n").expect("an earlier release's staging file");
    let dead: i64 = dead_pid();
    let dead_staged: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}-41", lock_path.display()));
    std::fs::write(&dead_staged, marker_of_holder(41, dead, "gone")).expect("a killed writer's staging file");
    let live_staged: PathBuf = staged_path_for(&lock_path, 40, "token");
    std::fs::write(&live_staged, own_marker(40)).expect("a live writer's staging file");
    let lock = acquire(&index, 42, &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(42));
    assert!(!fixed.exists() && !dead_staged.exists(), "{:?}", leftovers(&index));
    assert!(live_staged.exists(), "a live writer's staging file stays");
    std::fs::remove_file(&live_staged).expect("cleanup");
    drop(lock);
    let held = try_acquire(&index, 43).expect("an attempt without a wait takes it too");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(43));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(held);
    remove(root.as_path());
}

/// A dead holder's lock with a hard-linked alias (a kill after the link,
/// before the alias was removed) is taken over without
///  opening the alias,
/// so the alias never empties the lock;
/// the alias goes with the dead holder.
#[test]
fn a_dead_holders_lock_with_a_linked_alias_is_taken_over_without_emptying_it() {
    let (root, index) = index_in("index-lock-alias");
    let lock_path: PathBuf = lock_path_of(&index);
    let dead: i64 = dead_pid();
    std::fs::write(&lock_path, marker_of_holder(50, dead, "gone")).expect("a killed holder's lock");
    let alias: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}-50", lock_path.display()));
    std::fs::hard_link(&lock_path, &alias).expect("the alias a kill left");
    let lock = acquire(&index, 51, &|_| {}).expect("taken at once");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(51));
    assert!(!alias.exists(), "{:?}", leftovers(&index));
    drop(lock);
    remove(root.as_path());
}

/// A lock that goes out of scope without a release is removed,
/// and the index is as it was.
#[test]
fn a_lock_that_is_not_released_leaves_the_index_as_it_was() {
    let (root, index) = index_in("index-lock-drop");
    drop(acquire(&index, 1, &|_| {}).expect("taken"));
    assert!(!lock_path_of(&index).exists());
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    remove(root.as_path());
}

/// A foreign holder that lets go while the service waits is followed at once by the service's own lock,
/// and the wait is woken by that removal and by nothing else.
/// What that is read from is the order of the placements,
/// not how fast the host runs:
/// no placement
///  after the first met the holder,
/// so nothing but the removal that ended the hold woke the wait,
/// and a wait woken by a file of its own would meet the holder over and over.
/// Whether the wake was the removal and not the reminder's timer is the watch's own count of the waits it ended
/// at their timer:
/// a wait the timer ended adds one,
/// and the placement that follows such a wake either meets the holder (caught above) or places the lock after the
///  removal and leaves no trace of why it woke,
/// so the count is what tells the two apart.
/// No clock is read for it.
#[test]
fn a_wait_ends_when_the_foreign_holder_lets_go_and_does_not_wake_itself() {
    /// How long the foreign holder keeps the lock after the wait's first look.
    ///
    /// It is longer than the look behind it,
    /// so the wait cannot find the lock gone before it waits at all,
    /// and it is the hold the wake is measured against below.
    const HOLD: Duration = Duration::from_millis(400);
    let (root, index) = index_in("index-lock-wait");
    let foreign: PathBuf = lock_path_of(&index);
    std::fs::write(&foreign, "foreign").expect("foreign lock");
    let (looked, looking) = std::sync::mpsc::channel::<()>();
    let let_go: PathBuf = foreign.clone();
    let release = std::thread::spawn(move || {
        looking.recv_timeout(Duration::from_secs(60)).expect("the wait looked at the lock");
        std::thread::sleep(HOLD);
        std::fs::remove_file(let_go).expect("the foreign holder lets go");
    });
    let letting_go: LetsGoOnEveryPath<'_> = LetsGoOnEveryPath(&looked);
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    // How many waits had ended at their timer when this control's wait began.
    let timed_out_before: usize = crate::service_index_watch::WAKES_BY_THE_TIMER.load(std::sync::atomic::Ordering::SeqCst);
    // Whether the foreign lock stood at each placement, in the order the placements were made.
    let met: Mutex<Vec<bool>> = Mutex::new(Vec::new());
    let noting = |from: &Path, to: &Path| -> std::io::Result<()> {
        met.lock().expect("lock").push(foreign.exists());
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { rename: &noting, ..SYSTEM };
    // When the wait said what it waits for: the interval its own reminder is counted in begins there, after the staging.
    let said_at: Mutex<Option<Instant>> = Mutex::new(None);
    let lock = acquire_on(&platform, &index, 2, &|note| {
        {
            let mut when = said_at.lock().expect("lock");
            if when.is_none() {
                *when = Some(Instant::now());
                let _sent = looked.send(());
            }
        }
        heard.lock().expect("lock").push(note);
    }).expect("taken after the wait");
    let said: Option<Instant> = *said_at.lock().expect("lock");
    drop(letting_go);
    release.join().expect("thread");
    let placements: Vec<bool> = met.lock().expect("lock").clone();
    assert_eq!(placements.first(), Some(&true), "the first placement meets the foreign holder: {placements:?}");
    assert!(placements[1..].iter().all(|stood| return !stood), "every placement after the first follows the holder's removal, so the wait woke for nothing else: {placements:?}");
    assert!(!heard.lock().expect("lock").is_empty(), "the wait said nothing at all, so the lock it was to wait for went away first");
    // The wait says which of the two ended it, so no reading of the clock is asked whether the removal was the faster one.
    assert_eq!(
        crate::service_index_watch::WAKES_BY_THE_TIMER.load(std::sync::atomic::Ordering::SeqCst),
        timed_out_before,
        "the wait ended at its timer and not at the removal; placements {placements:?}"
    );
    // The holder let go a hold after the wait's line, so a wait that returned before the foreign lock was gone is caught by the placements and here.
    let since_said: Duration = said.expect("the wait said what it waits for").elapsed();
    assert!(since_said >= HOLD, "the wait returned before the hold it was told to keep: {since_said:?} against {HOLD:?}, placements {placements:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(2));
    drop(lock);
    remove(root.as_path());
}

/// An owner nothing proves alive is waited for like one proven alive:
/// the clients are told what is known and how to remove the lock if no program uses the repository,
/// the wait ends only when the holder lets go,
/// and the lock is taken then.
/// The hold begins when the wait has spoken,
/// so it is the wait's own line that ends the hold.
#[test]
fn an_owner_nothing_proves_alive_is_waited_for_like_a_proven_one() {
    let (root, index) = index_in("index-lock-unproven-unbounded");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let holding: Duration = LONG_HOLD;
    let foreign: PathBuf = lock_path_of(&index);
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let (said_once, speaking) = std::sync::mpsc::channel::<()>();
    // The holder lets go a hold after the wait said what it waits for; a hold counted from before the acquisition's staging could
    //  end before the wait's first look on a host whose flush is slow.
    let release = std::thread::spawn(move || {
        speaking.recv_timeout(Duration::from_secs(60)).expect("the wait said what it waits for");
        std::thread::sleep(holding);
        std::fs::remove_file(foreign).expect("the foreign lock goes");
    });
    let letting_go: LetsGoOnEveryPath<'_> = LetsGoOnEveryPath(&said_once);
    let began: Instant = Instant::now();
    let lock = acquire(&index, 4, &|note| {
        let _sent = said_once.send(());
        heard.lock().expect("lock").push(note);
    }).expect("taken once the lock went");
    assert!(began.elapsed() >= holding, "the wait returned before the hold it was told to keep: {:?}", began.elapsed());
    drop(letting_go);
    release.join().expect("thread");
    let said: Vec<Announce> = heard.lock().expect("lock").clone();
    // A wait that said nothing never looked at the held lock: the lock went away before its first look, which no order of this
    //  control's own steps can make it do.
    assert!(!said.is_empty(), "the wait said nothing at all, so the lock it was to wait for went away first");
    assert!(said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("cannot be proven alive") && text.contains("removing the file") && text.contains("has the lock open")); }), "{said:?}");
    assert!(!said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("not made")); }), "{said:?}");
    drop(lock);
    remove(root.as_path());
}

/// Where no watch can be set (the watch limit is reached,
/// or the file system gives none),
/// the wait polls:
/// a foreign lock that goes
///  after a while is followed by the service's own lock,
/// where the wait used to fail the commit at once.
#[test]
fn where_no_watch_can_be_set_the_wait_polls_until_the_lock_goes() {
    let (root, index) = index_in("index-lock-no-watch-poll");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let foreign: PathBuf = lock_path_of(&index);
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_file(foreign).expect("the foreign lock goes");
    });
    let unwaitable: Platform = Platform { watch: &|_| { return None; }, ..SYSTEM };
    let lock = acquire_on(&unwaitable, &index, 12, &|_| {}).expect("taken once the lock went, without a watch");
    release.join().expect("thread");
    drop(lock);
    remove(root.as_path());
}

/// How long the controls keep a foreign lock after the wait has spoken:
/// more than the one second an owner nothing proves alive was once backed off for,
/// so a wait that ends by itself in that time is caught,
/// and no control reads the host's speed for it.
const LONG_HOLD: Duration = Duration::from_millis(2500);

/// The shortest ceiling a wait can be given:
/// the wait's clock has already run out when it begins,
/// so an ending that comes before the ceiling is proven to take no time over it,
/// however long the host stalls.
const NO_CEILING: Duration = Duration::ZERO;

/// A PID file that names a number this process table does not have is no proof that its owner is gone,
/// so the commit waits within its ceiling as it does for a holder proven alive,
/// nothing is removed,
/// and the words say which number was read and that its writer may be another PID namespace's.
/// Why:
///  A live Git in another PID namespace shares the repository,
///  writes a number this table has never had and holds the lock through its editor with no descriptor open,
///  and a verdict of dead ended the queued commit at once (cross-vendor review of 2026-11-09,
/// finding 2).
/// The ceiling is already out when the wait begins,
/// so the wait ends in the ceiling's words,
/// at once and without a stopwatch to read.
#[test]
fn a_number_this_table_does_not_have_is_waited_for_within_the_ceiling() {
    let (root, index) = index_in("index-lock-dead-owner");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let gone: i64 = dead_pid();
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {gone}\n")).expect("PID file");
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let failure = acquire_within(&SYSTEM, &index, 5, NO_CEILING, &|note| { heard.lock().expect("lock").push(note); }).expect_err("refused");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    let said: Vec<Announce> = heard.lock().expect("lock").clone();
    assert!(said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains(format!("PID {gone}").as_str()) && text.contains("another PID namespace")); }), "{said:?}");
    assert!(said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("still held after")); }), "the ceiling's words end the wait: {said:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("foreign lock stands"), "foreign");
    remove(root.as_path());
}

/// A holder proven alive is waited for until it lets go:
/// the wait names the holder from its Git lock PID file and leaves the lock in place,
/// the wait goes on through the whole hold with no line that gives up,
/// and the service's lock is taken once the holder lets go.
/// The hold begins when the wait has spoken,
/// so it is the wait's own line that ends it.
/// A hold counted from before the acquisition's staging ended before the wait's first look on a slow host,
/// and the control then read no line at all
/// (measured 2026-11-08:
/// a flush of 10.2 s against a hold of 2.5 s).
#[test]
fn a_holder_proven_alive_is_waited_for_until_the_holder_lets_go() {
    let (root, index) = index_in("index-lock-proven-alive");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    // The PID file names this process, which runs and started before the lock was made.
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {}\n", std::process::id())).expect("PID file");
    let holding: Duration = LONG_HOLD;
    let foreign: PathBuf = lock_path_of(&index);
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let (said_once, speaking) = std::sync::mpsc::channel::<()>();
    // The holder lets go a hold after the wait said what it waits for; a hold counted from before the acquisition's staging could
    //  end before the wait's first look on a host whose flush is slow.
    let release = std::thread::spawn(move || {
        speaking.recv_timeout(Duration::from_secs(60)).expect("the wait said what it waits for");
        std::thread::sleep(holding);
        std::fs::remove_file(foreign).expect("the foreign holder lets go");
    });
    let letting_go: LetsGoOnEveryPath<'_> = LetsGoOnEveryPath(&said_once);
    let began: Instant = Instant::now();
    let lock = acquire(&index, 6, &|note| {
        let _sent = said_once.send(());
        heard.lock().expect("lock").push(note);
    }).expect("taken once the holder let go");
    assert!(began.elapsed() >= holding, "the wait returned before the hold it was told to keep: {:?}", began.elapsed());
    drop(letting_go);
    release.join().expect("thread");
    let said: Vec<Announce> = heard.lock().expect("lock").clone();
    // As in the control above: nothing said means the lock went away before the wait looked at it.
    assert!(!said.is_empty(), "the wait said nothing at all, so the lock it was to wait for went away first");
    assert!(said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("proven by its Git lock PID file")); }), "{said:?}");
    assert!(!said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("giving up") || text.contains("not made")); }), "{said:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(6));
    drop(lock);
    remove(root.as_path());
}

/// A lock with the service's marker whose holder is gone is a dead instance's:
/// the next landing removes it and takes its own at
///  once,
/// whether the marker names a dead holder or is one an earlier release wrote without a holder.
#[test]
fn a_dead_service_instances_lock_does_not_block_the_next_landing() {
    let (root, index) = index_in("index-lock-dead-instance");
    std::fs::write(lock_path_of(&index), marker_of_holder(99, dead_pid(), "gone")).expect("a dead instance's lock");
    // No watch can be set,
    // so nothing can be waited for:
    // the lock has to be taken without a wait or the attempt fails at once,
    // whatever the host's load,
    // and a landing that deferred the removal to the end of a wait would be refused here.
    let unwaitable: Platform = Platform { watch: &|_| { return None; }, ..SYSTEM };
    let lock = acquire_on(&unwaitable, &index, 100, &|_| {}).expect("taken at once");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(100));
    drop(lock);
    std::fs::write(lock_path_of(&index), marker_of_holder(7, dead_pid(), "gone")).expect("a killed holder's lock");
    assert!(try_acquire(&index, 101).is_some(), "an attempt without a wait removes it too");
    remove(root.as_path());
}

/// A child process that is killed and reaped when the control ends,
/// failed or not,
/// so a failing control leaks nothing.
struct Holder(std::process::Child);

impl Drop for Holder {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _reaped = self.0.wait();
    }
}

/// What:
///  A service lock whose holder still runs is waited for like a foreign one and never taken over;
///  once that holder is gone,
///  the waiting acquisition takes the lock and holds its own.
/// Why:
///   The wait's own notice is what shows it reached the held lock,
/// the holder's marker still in the lock is what shows it took nothing over,
/// and the marker the waiter then holds is what shows it went on;
/// no clock decides the verdict,
/// and nothing here races the waiter for the lock.
/// The control used to ask for the lock itself,
/// and when it won that race the waiter could only retry at its notice interval:
/// on 2026-11-09 alone this control took 0.4 s,
/// 2.7 s,
/// 11.1 s
/// and 21.4 s over eight runs,
/// and another agent reports it failing in a suite at 34 s against the 30 s bound of that race.
#[test]
fn a_live_service_holders_lock_is_never_taken_over() {
    let (root, index) = index_in("index-lock-live-holder");
    let holder: Holder = Holder(std::process::Command::new("sleep").arg("30").spawn().expect("a live holder"));
    let pid: i64 = i64::from(holder.0.id());
    let birth: String = process_birth_identity(pid).expect("identity").expect("running");
    let held: String = marker_of_holder(9, pid, birth.as_str());
    std::fs::write(lock_path_of(&index), held.as_str()).expect("a live holder's lock");
    // The wait blocks until the holder lets go, so it runs on a thread; the notice it says here is the look that found the
    //  live holder, and the lock is read right after it.
    let said: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let taken: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let (counting, recording) = (Arc::clone(&said), Arc::clone(&taken));
    let waiter_index: PathBuf = index.clone();
    let waiter = std::thread::spawn(move || {
        let lock = acquire(&waiter_index, 10, &|_note| {
            let mut heard = counting.lock().expect("count");
            *heard += 1;
        })
        .expect("taken once the holder is gone");
        let marker: String = std::fs::read_to_string(lock_path_of(&waiter_index)).expect("the lock this acquisition holds");
        *recording.lock().expect("record") = Some(marker);
        drop(lock);
    });
    assert!(wait_until(&|| { return *said.lock().expect("count") > 0; }), "the waiting acquisition said what it waits for, so it waited for the live holder instead of taking its lock");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("stands"), held, "a live holder's lock was taken over");
    assert!(try_acquire(&index, 10).is_none(), "an attempt without a wait does not take a live holder's lock");
    remove_stale(&index, 9);
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("stands"), held, "the reclaim of the service leaves a live holder's lock");
    drop(holder);
    // The holder is gone, so the lock is the service's to remove, and its removal is the event the wait is woken by:
    // no timer and no race for the lock stand between the two.
    remove_stale(&index, 9);
    assert!(wait_until(&|| { return taken.lock().expect("record").is_some(); }), "the waiting acquisition took the lock once the holder was gone");
    assert_eq!(taken.lock().expect("record").clone(), Some(own_marker(10)), "the acquisition that waited holds its own lock");
    waiter.join().expect("waiter");
    remove(root.as_path());
}

/// A stale lock is removed only when it carries the service's marker for that batch and its holder is gone.
#[test]
fn a_stale_lock_is_removed_only_when_it_is_the_services_own() {
    let (root, index) = index_in("index-lock-stale");
    std::fs::write(lock_path_of(&index), marker_of_holder(5, dead_pid(), "gone")).expect("ours");
    remove_stale(&index, 6);
    assert!(lock_path_of(&index).exists(), "another batch's marker is left");
    remove_stale(&index, 5);
    assert!(!lock_path_of(&index).exists(), "its own marker is removed");
    std::fs::write(lock_path_of(&index), own_marker(5)).expect("a live holder's");
    remove_stale(&index, 5);
    assert!(lock_path_of(&index).exists(), "a live holder's lock is never touched");
    std::fs::write(lock_path_of(&index), "DIRCbinary").expect("foreign");
    remove_stale(&index, 5);
    assert!(lock_path_of(&index).exists(), "a foreign lock is never touched");
    remove(root.as_path());
}

/// The marker reads back as written,
/// an earlier release's marker reads as one without a holder,
/// and anything else is not a
///  service marker.
#[test]
fn markers_read_back_and_others_are_not_the_services() {
    assert_eq!(parse_marker("cli-git-service 12 345 linux:boot:678\n"), Some(Marked { seq: 12, holder: Some((345, String::from("linux:boot:678"))), current: false }));
    assert_eq!(parse_marker("cli-git-service v2 12 345 linux:boot:678\n"), Some(Marked { seq: 12, holder: Some((345, String::from("linux:boot:678"))), current: true }));
    assert_eq!(parse_marker("cli-git-service 12\n"), Some(Marked { seq: 12, holder: None, current: false }));
    assert_eq!(parse_marker("cli-git-service 12 345 a birth with spaces\n").and_then(|marked| { return marked.holder; }), Some((345, String::from("a birth with spaces"))));
    for other in ["", "DIRC", "cli-git-service \n", "cli-git-service x\n", "cli-git-service 12 345\n", "cli-git-service 12 pid birth\n", "Cli-git-service 12\n"] {
        assert_eq!(parse_marker(other), None, "{other:?}");
    }
}

/// The wait's event filter wakes for the lock's own removal and for an overflow,
/// never for another file in the directory.
#[test]
fn the_wait_wakes_only_for_the_lock_itself() {
    let event = |mask: u32, name: &str| -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1_i32.to_ne_bytes());
        bytes.extend_from_slice(&mask.to_ne_bytes());
        bytes.extend_from_slice(&0_u32.to_ne_bytes());
        let mut padded: Vec<u8> = name.as_bytes().to_vec();
        padded.resize(name.len().div_ceil(16).max(1) * 16, 0);
        bytes.extend_from_slice(&u32::try_from(padded.len()).expect("small").to_ne_bytes());
        bytes.extend_from_slice(padded.as_slice());
        return bytes;
    };
    let staging: Vec<u8> = event(libc::IN_DELETE, "index.lock.cli-git-marker-1-2");
    let lock: Vec<u8> = event(libc::IN_DELETE, "index.lock");
    let both: Vec<u8> = [staging.clone(), lock.clone()].concat();
    assert!(!events_name(staging.as_slice(), b"index.lock"), "a staging file does not wake the wait");
    assert!(events_name(lock.as_slice(), b"index.lock"));
    assert!(events_name(both.as_slice(), b"index.lock"), "the lock's event is found after another");
    assert!(!events_name(event(libc::IN_DELETE, "index.lock.other").as_slice(), b"index.lock"), "a name that only starts like it");
    assert!(events_name(event(libc::IN_Q_OVERFLOW, "").as_slice(), b"index.lock"), "an overflow may have hidden it");
    assert!(!events_name(&[], b"index.lock"));
}

//region Fixtures that stand in for environments no fixture can be made to have.

/// A refusal with `code`,
/// as the kernel gives it.
fn refusal(code: i32) -> std::io::Result<()> {
    return Err(std::io::Error::from_raw_os_error(code));
}

/// The calls of a filesystem that has neither a rename without replacing nor hard links (vfat before Linux 4.x).
fn bare_filesystem<'a>(rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>, link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>) -> Platform<'a> {
    return Platform { rename, link, ..SYSTEM };
}

/// The staging name the release before this one used:
/// the process and the batch,
/// with nothing that makes it new.
fn staged_by_the_last_release(lock: &Path, pid: u32, seq: u64) -> PathBuf {
    return PathBuf::from(format!("{}{STAGED_INFIX}-{pid}-{seq}", lock.display()));
}

//endregion

/// Where neither a rename without replacing nor a hard link works,
/// the lock is taken as Git takes it:
/// created by exclusive creation and then given its marker,
/// so a commit lands there as it does with plain Git.
/// The hold is the lock's own file,
/// so its release installs the new index and removes the lock,
/// and nothing stays in the directory.
#[test]
fn a_filesystem_that_can_place_no_lock_atomically_gets_one_by_exclusive_creation() {
    let (root, index) = index_in("index-lock-bare");
    let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EINVAL); };
    let link = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EPERM); };
    let platform: Platform = bare_filesystem(&rename, &link);
    let lock = acquire_on(&platform, &index, 20, &|_| {}).expect("taken as Git takes it");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(20), "the marker is in the lock once the hold begins");
    assert!(try_acquire_on(&platform, &index, 21).is_none(), "a live holder's lock is not taken over");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    lock.release_with(&built).expect("the hold names the created lock, so the release finds it the holder's own");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index");
    assert!(!lock_path_of(&index).exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// Where only the rename is refused as unsupported,
/// the hard link places the lock,
/// marked,
/// and its staging name goes.
#[test]
fn a_filesystem_with_links_but_no_exclusive_rename_gets_the_lock_by_link() {
    let (root, index) = index_in("index-lock-link");
    let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EINVAL); };
    let platform: Platform = Platform { rename: &rename, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 22, &|_| {}).expect("taken by link");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(22));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(lock);
    assert!(!lock_path_of(&index).exists());
    remove(root.as_path());
}

/// A process that cannot read its own birth identity writes no marker and takes no lock:
/// a marker without an identity reads as a dead instance's,
/// so any other process could take the lock from a holder that runs.
#[test]
fn a_process_without_an_identity_takes_no_lock() {
    let (root, index) = index_in("index-lock-no-identity");
    let missing = || -> Result<String, ProcessIdentityError> { return Err(ProcessIdentityError::CurrentUnavailable); };
    let platform: Platform = Platform { identity: &missing, ..SYSTEM };
    assert!(marker_text(23, &missing).is_err(), "no marker is written without an identity");
    let failure = acquire_on(&platform, &index, 23, &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(try_acquire_on(&platform, &index, 24).is_none());
    assert!(!lock_path_of(&index).exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// A dead holder's lock,
/// with the hard-link alias it left under the name this process would use for the same batch in the last release (the process number came round again),
/// is taken over:
/// the alias is never opened,
/// so it cannot empty the lock,
/// and the batch is not refused or left holding an empty lock.
#[test]
fn a_recycled_process_number_does_not_reopen_a_dead_holders_alias() {
    let (root, index) = index_in("index-lock-recycled");
    let lock_path: PathBuf = lock_path_of(&index);
    let dead: i64 = dead_pid();
    std::fs::write(&lock_path, marker_of_holder(60, dead, "gone")).expect("a killed holder's lock");
    let alias: PathBuf = staged_by_the_last_release(&lock_path, std::process::id(), 60);
    std::fs::hard_link(&lock_path, &alias).expect("the alias a kill left");
    let lock = acquire(&index, 60, &|_| {}).expect("taken over, not refused");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(60));
    // The alias is never opened for writing: it is unchanged, or removed with the dead holder (a whole marker that names a gone
    //  holder under a process number that runs now is what the cleanup takes for a killed service's file).
    assert!(std::fs::read_to_string(&alias).map_or(true, |text| { return text == marker_of_holder(60, dead, "gone"); }), "the alias is not ours to empty");
    drop(lock);
    // The dead holder's lock is back, with an alias naming it, and the attempt without a wait takes it over.
    std::fs::write(&lock_path, marker_of_holder(60, dead, "gone")).expect("the dead holder's lock, as the kill left it");
    let again: PathBuf = staged_by_the_last_release(&lock_path, std::process::id(), 61);
    std::fs::hard_link(&lock_path, &again).expect("the alias a kill left");
    let held = try_acquire(&index, 60).expect("an attempt without a wait takes over as well");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("marker"), own_marker(60));
    assert!(std::fs::read_to_string(&again).map_or(true, |text| { return text == marker_of_holder(60, dead, "gone"); }), "the alias is not ours to empty");
    drop(held);
    remove(root.as_path());
}

/// Two attempts of one process for one batch use two names,
/// and a name that exists is an error,
/// not a file to empty.
#[test]
fn staging_files_are_created_new_under_names_no_attempt_repeats() {
    let (root, index) = index_in("index-lock-new-names");
    let lock_path: PathBuf = lock_path_of(&index);
    let (first, held_first) = stage_marker(&lock_path, 61, "one").expect("first");
    let (second, held_second) = stage_marker(&lock_path, 61, "two").expect("second");
    assert_ne!(first, second);
    assert_eq!(std::fs::read_to_string(&first).expect("first"), "one");
    let taken = std::fs::OpenOptions::new().write(true).create_new(true).open(&first);
    assert!(taken.is_err_and(|error| { return error.kind() == std::io::ErrorKind::AlreadyExists; }), "creation never truncates");
    std::fs::remove_file(&first).expect("cleanup");
    std::fs::remove_file(&second).expect("cleanup");
    drop((held_first, held_second));
    remove(root.as_path());
}

/// A staging file is judged by the process its name carries,
/// in the PID namespace its name carries,
/// and not by its content:
/// one that is still empty or half written belongs to a writer that runs and stays;
/// an empty one of a process of this namespace that is gone goes;
/// one of another namespace,
/// one an earlier release named without a namespace,
/// and one that names no process stay.
#[test]
fn staging_files_are_judged_by_the_writer_in_their_name_and_never_by_their_content() {
    let (root, index) = index_in("index-lock-staging-by-name");
    let lock_path: PathBuf = lock_path_of(&index);
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let live_empty: PathBuf = staged_path_for(&lock_path, 62, "writing");
    std::fs::write(&live_empty, "").expect("a live writer between creating its file and writing it");
    let live_half: PathBuf = staged_by_the_last_release(&lock_path, std::process::id(), 63);
    std::fs::write(&live_half, "cli-git-ser").expect("a live writer part way through");
    let dead: u32 = u32::try_from(dead_pid()).expect("pid");
    let dead_empty: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}.{scope}-64-token", lock_path.display()));
    std::fs::write(&dead_empty, "").expect("a dead writer's empty file");
    let other_namespace: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-{dead}.1-65-token", lock_path.display()));
    std::fs::write(&other_namespace, "").expect("an empty file of a process number of another namespace");
    let legacy: PathBuf = staged_by_the_last_release(&lock_path, dead, 66);
    std::fs::write(&legacy, "").expect("an empty file an earlier release named without a namespace");
    let unnamed: PathBuf = PathBuf::from(format!("{}{STAGED_INFIX}-not-a-process", lock_path.display()));
    std::fs::write(&unnamed, "").expect("a file that names no process");
    let lock = acquire(&index, 67, &|_| {}).expect("taken");
    assert!(live_empty.exists() && live_half.exists(), "a running writer's file stays whatever it holds: {:?}", leftovers(&index));
    assert!(!dead_empty.exists(), "a dead writer's file of this namespace goes: {:?}", leftovers(&index));
    assert!(other_namespace.exists(), "a number of another namespace is not looked up here");
    assert!(legacy.exists(), "a name without a namespace is ambiguous and stays");
    assert!(unnamed.exists(), "a name that names no process is left alone");
    drop(lock);
    remove(root.as_path());
}

/// A whole marker decides alone,
/// and its verdict vetoes the name:
/// a writer that runs keeps its file though the name carries a number nothing here runs under,
/// a writer this reader cannot compare with (another namespace) keeps it,
/// and a holder that is gone loses it whatever the name says.
#[test]
fn a_whole_marker_decides_a_staging_files_fate_over_its_name() {
    let (root, index) = index_in("index-lock-staging-by-marker");
    let lock_path: PathBuf = lock_path_of(&index);
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let dead: u32 = u32::try_from(dead_pid()).expect("pid");
    let named = |tag: &str| -> PathBuf { return PathBuf::from(format!("{}{STAGED_INFIX}-{dead}.{scope}-{tag}", lock_path.display())); };
    let running: PathBuf = named("70-running");
    std::fs::write(&running, own_marker(70)).expect("a running writer's whole marker");
    let own: String = own_identity().expect("identity");
    let parts: Vec<&str> = own.split('#').collect();
    let elsewhere: PathBuf = named("71-elsewhere");
    std::fs::write(&elsewhere, format!("{LOCK_MARKER}71 {} {}#{}#1\n", std::process::id(), parts[0], parts[1])).expect("a marker of another namespace");
    let gone: PathBuf = named("72-gone");
    std::fs::write(&gone, marker_of_holder(72, std::process::id().into(), "linux:1")).expect("a marker of a holder that is gone");
    let lock = acquire(&index, 73, &|_| {}).expect("taken");
    assert!(running.exists(), "a writer that runs keeps its file: {:?}", leftovers(&index));
    assert!(elsewhere.exists(), "a writer of another namespace keeps its file");
    assert!(!gone.exists(), "a holder that is gone loses it");
    drop(lock);
    remove(root.as_path());
}

/// A reader whose `/proc` does not number it as the system does has no evidence of death:
/// a file whose name says a process of its namespace is gone stays,
/// though the lookup finds nothing there.
/// With a `/proc` that agrees,
/// the same file goes.
#[test]
fn an_unreadable_proc_is_no_evidence_that_a_writer_died() {
    use crate::service_index_staging::staging_is_dead_in;
    let (root, index) = index_in("index-lock-staging-proc");
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let path: PathBuf = root.join("empty-staging-file");
    std::fs::write(&path, "").expect("an empty file");
    let suffix: String = format!("-4242.{scope}-1-token");
    let nothing_runs = |_: i64| -> Result<Option<String>, ProcessIdentityError> { return Ok(None) };
    let agreeing = ReaderView { boot: "0123456789abcdef0123456789abcdef", scope: Some(scope), machine: None, procfs_agrees: true };
    let disagreeing = ReaderView { procfs_agrees: false, ..agreeing };
    assert!(staging_is_dead_in(&path, suffix.as_bytes(), &agreeing, &nothing_runs));
    assert!(!staging_is_dead_in(&path, suffix.as_bytes(), &disagreeing, &nothing_runs), "a /proc that disagrees proves nothing");
    let elsewhere = ReaderView { scope: Some("1"), ..agreeing };
    assert!(!staging_is_dead_in(&path, suffix.as_bytes(), &elsewhere, &nothing_runs), "another namespace's number is not looked up");
    let _unused = index;
    remove(root.as_path());
}

/// A marker cut anywhere before its newline is a marker that was not finished:
/// it reads as nothing,
/// from the text and from a file,
/// whether the cut falls in the batch,
/// the process number,
/// the start tick,
/// the boot ID or the PID namespace.
/// The whole marker reads back,
/// and names a holder that runs.
#[test]
fn a_marker_cut_at_any_point_reads_as_no_marker() {
    let (root, index) = index_in("index-lock-cut-markers");
    let lock_path: PathBuf = lock_path_of(&index);
    let whole: String = own_marker(75);
    assert!(parse_marker(whole.as_str()).is_some_and(|marked| { return holder_is_running(marked.holder.as_ref()); }));
    for cut in 0..whole.len() {
        let part: &str = &whole[..cut];
        assert_eq!(parse_marker(part), None, "cut at {cut}: {part:?}");
        std::fs::write(&lock_path, part).expect("a cut marker");
        assert!(crate::service_index_marker::marker_of(&lock_path).is_none(), "cut at {cut}");
        assert!(!crate::service_index_marker::is_reclaimable(&lock_path), "cut at {cut}: a cut marker is not a dead instance's");
    }
    assert!(parse_marker(whole.as_str()).is_some());
    remove(root.as_path());
}

/// The scan for the processes that have the lock open stops at its deadline and says so,
/// and keeps what it found before:
/// a scan with no deadline finds the holder.
/// The commit service scans with a bound,
/// so a host with many processes does not hold its queue.
#[test]
fn the_scan_for_open_holders_stops_at_its_deadline_and_says_so() {
    use crate::index_lock_holders::scan_proc_fd_holders_until;
    use std::os::unix::fs::MetadataExt;
    let (root, index) = index_in("index-lock-scan-bound");
    let lock_path: PathBuf = lock_path_of(&index);
    std::fs::write(&lock_path, "").expect("a lock");
    let holder: Holder = Holder(std::process::Command::new("sh").arg("-c").arg("exec 3<\"$0\"; exec sleep 30").arg(&lock_path).spawn().expect("a holder"));
    let pid: u32 = holder.0.id();
    let began: Instant = Instant::now();
    // The holder is ready when it has the lock open and has become `sleep`: the shell opens the lock first and executes `sleep` after,
    //  so a look in between names the shell (measured in the container gate, 2026-11-07: `open_by=[14748 (sh)]`).
    while !(std::fs::read_dir(format!("/proc/{pid}/fd")).is_ok_and(|entries| {
        return entries.flatten().any(|entry| { return std::fs::read_link(entry.path()).is_ok_and(|target| { return target == lock_path; }); });
    }) && std::fs::read_to_string(format!("/proc/{pid}/comm")).is_ok_and(|name| { return name.trim() == "sleep"; }))
    {
        assert!(began.elapsed() < Duration::from_secs(10), "the holder never opened the lock and became sleep");
        std::thread::sleep(Duration::from_millis(5));
    }
    let metadata = std::fs::metadata(&lock_path).expect("lock");
    // The scan matches a descriptor to the lock by the device number and inode the lock has, as the service's own scan does.
    let unbounded = scan_proc_fd_holders_until(Path::new("/proc"), metadata.dev(), metadata.ino(), std::process::id(), None);
    assert!(unbounded.holders.iter().any(|found| { return found.pid == i64::from(pid); }), "{unbounded:?}");
    let bounded = scan_proc_fd_holders_until(Path::new("/proc"), metadata.dev(), metadata.ino(), std::process::id(), Some(Instant::now()));
    assert!(bounded.holders.is_empty(), "a scan past its deadline looks at nothing: {bounded:?}");
    assert!(bounded.partial.iter().any(|reason| { return reason.contains("time bound"); }), "{bounded:?}");
    drop(holder);
    remove(root.as_path());
}

/// A stale lock that cannot be removed ends the attempt at once with the plain failure,
/// naming nothing to wait for:
/// waiting longer would not remove it.
/// Before,
/// the failed removal skipped the deadline and the attempt repeated without end.
#[test]
fn a_stale_lock_that_cannot_be_removed_ends_the_wait_instead_of_repeating_it() {
    let (root, index) = index_in("index-lock-unremovable");
    let dead: i64 = dead_pid();
    std::fs::write(lock_path_of(&index), marker_of_holder(66, dead, "gone")).expect("a killed holder's lock");
    let calls: Cell<usize> = Cell::new(0);
    let unlink = |_: &Path| -> std::io::Result<()> {
        calls.set(calls.get() + 1);
        assert!(calls.get() < 50, "the attempt repeats without end");
        return refusal(libc::EPERM);
    };
    let platform: Platform = Platform { unlink: &unlink, ..SYSTEM };
    let failure = acquire_on(&platform, &index, 67, &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert_eq!(calls.get(), 1, "removal is tried once");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("stands"), marker_of_holder(66, dead, "gone"));
    calls.set(0);
    assert!(try_acquire_on(&platform, &index, 68).is_none());
    assert_eq!(calls.get(), 1);
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// A lock that is gone by the time it is removed is not a failure:
/// the attempt goes on and takes it.
#[test]
fn a_stale_lock_that_vanishes_before_its_removal_is_taken_at_once() {
    let (root, index) = index_in("index-lock-vanished");
    std::fs::write(lock_path_of(&index), marker_of_holder(69, dead_pid(), "gone")).expect("a killed holder's lock");
    let vanished = |path: &Path| -> std::io::Result<()> {
        std::fs::remove_file(path)?;
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    };
    let platform: Platform = Platform { unlink: &vanished, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 70, &|_| {}).expect("taken");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(70));
    drop(lock);
    remove(root.as_path());
}

/// How many acquisitions the controls of the count below make:
/// two would already show a count shared between acquisitions,
/// and the third leaves one spare for a scheduling race.
/// Why not forty:
/// a lock is placed and its marker flushed for each one,
/// so one acquisition cost the contended disk 1.7 s on 2026-11-09,
/// and forty
///  of them on two indexes took this control 140.7 s alone,
/// which a suite run met as the runner's five-minute limit.
const ACQUISITIONS: usize = 3;

/// One acquisition that met no holder makes one attempt:
/// the number is the acquisition's own.
#[test]
fn one_acquisition_counts_one_attempt() {
    let (root, index) = index_in("index-lock-attempts-one");
    let attempts_made: Cell<usize> = Cell::new(0);
    let counting = |from: &Path, to: &Path| -> std::io::Result<()> {
        attempts_made.set(attempts_made.get() + 1);
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    drop(acquire_on(&platform, &index, 0, &|_| {}).expect("taken"));
    assert_eq!(attempts_made.get(), 1, "one attempt for an acquisition that met no holder");
    remove(root.as_path());
}

/// The number of attempts is that of one acquisition:
/// another acquisition running at the same time,
/// on its own thread and its own index,
/// adds nothing to it.
#[test]
fn another_acquisitions_attempts_are_not_counted() {
    let (root, index) = index_in("index-lock-attempts");
    let (other_root, other_index) = index_in("index-lock-attempts-other");
    let busy = std::thread::spawn(move || {
        for seq in 0..ACQUISITIONS {
            drop(acquire(&other_index, 1000 + u64::try_from(seq).expect("a small count"), &|_| {}).expect("taken"));
        }
    });
    let attempts_made: Cell<usize> = Cell::new(0);
    let counting = |from: &Path, to: &Path| -> std::io::Result<()> {
        attempts_made.set(attempts_made.get() + 1);
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    for seq in 0..ACQUISITIONS {
        drop(acquire_on(&platform, &index, u64::try_from(seq).expect("a small count"), &|_| {}).expect("taken"));
    }
    busy.join().expect("thread");
    assert_eq!(attempts_made.get(), ACQUISITIONS, "one attempt for each of the acquisitions that met no holder");
    remove(root.as_path());
    remove(other_root.as_path());
}

/// A lock that was removed and replaced by another program's is not released or removed by the holder that placed the first:
/// the index is not installed,
/// the other lock stands,
/// and the failure names the lock.
/// Two takers of one lock thus never both finish.
#[test]
fn a_lock_another_program_replaced_is_neither_released_nor_removed() {
    let (root, index) = index_in("index-lock-replaced");
    let lock_path: PathBuf = lock_path_of(&index);
    let lock = acquire(&index, 71, &|_| {}).expect("taken");
    std::fs::remove_file(&lock_path).expect("a person deletes the lock");
    std::fs::write(&lock_path, marker_of_holder(72, 1, "another service")).expect("another program takes it");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    let error = lock.release_with(&built).expect_err("refused");
    assert!(error.to_string().contains(lock_path.to_string_lossy().as_ref()), "{error}");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index", "the index was not installed");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("the other lock stands"), marker_of_holder(72, 1, "another service"));
    assert!(built.exists());
    remove(root.as_path());
}

/// A holder that goes out of scope without a release leaves a lock that is no longer its own where it is,
/// and removes its own.
#[test]
fn a_lock_that_is_not_the_holders_own_is_left_when_the_holder_goes_out_of_scope() {
    let (root, index) = index_in("index-lock-replaced-drop");
    let lock_path: PathBuf = lock_path_of(&index);
    let lock = acquire(&index, 73, &|_| {}).expect("taken");
    std::fs::write(&lock_path, "foreign").expect("replaced");
    drop(lock);
    assert_eq!(std::fs::read_to_string(&lock_path).expect("stands"), "foreign");
    std::fs::remove_file(&lock_path).expect("cleanup");
    let gone = acquire(&index, 74, &|_| {}).expect("taken");
    std::fs::remove_file(&lock_path).expect("removed by someone");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    gone.release_with(&built).expect_err("a lock that is gone is not released");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    remove(root.as_path());
}

/// The events that end a watch (an unmount,
/// the kernel dropping the watch) wake the wait and say the watch is over,
/// though they name no file;
/// a removal of another file does neither.
#[test]
fn events_that_end_a_watch_wake_the_wait_and_are_told_apart() {
    let event = |mask: u32, name: &str| -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1_i32.to_ne_bytes());
        bytes.extend_from_slice(&mask.to_ne_bytes());
        bytes.extend_from_slice(&0_u32.to_ne_bytes());
        let mut padded: Vec<u8> = name.as_bytes().to_vec();
        padded.resize(name.len().div_ceil(16) * 16, 0);
        bytes.extend_from_slice(&u32::try_from(padded.len()).expect("small").to_ne_bytes());
        bytes.extend_from_slice(padded.as_slice());
        return bytes;
    };
    for mask in [libc::IN_UNMOUNT, libc::IN_IGNORED, libc::IN_MOVE_SELF] {
        let ended: Vec<u8> = event(mask, "");
        assert!(events_name(ended.as_slice(), b"index.lock"), "{mask:#x} wakes the wait");
        assert!(events_end_watch(ended.as_slice()), "{mask:#x} ends the watch");
    }
    let other: Vec<u8> = event(libc::IN_DELETE, "index.lock.other");
    assert!(!events_end_watch(other.as_slice()));
    assert!(!events_end_watch(event(libc::IN_Q_OVERFLOW, "").as_slice()), "an overflow wakes the wait but leaves the watch");
    assert!(events_name(event(libc::IN_Q_OVERFLOW, "").as_slice(), b"index.lock"));
    let lock_then_end: Vec<u8> = [event(libc::IN_DELETE, "index.lock"), event(libc::IN_IGNORED, "")].concat();
    assert!(events_end_watch(lock_then_end.as_slice()), "the end is found after another event");
}

/// A watch whose directory is removed is over:
/// the wait returns at once,
/// not at its timer,
/// and says so.
#[test]
fn a_wait_on_a_watch_whose_directory_is_removed_returns_at_once_and_reports_it() {
    let root: PathBuf = fixture("index-lock-watch-ended");
    let directory: PathBuf = root.join("watched");
    std::fs::create_dir(&directory).expect("directory");
    let watch: Watch = Watch::on(&directory).expect("watch");
    assert!(watch.wait(Duration::from_millis(50), std::ffi::OsStr::new("index.lock")), "a quiet directory leaves the watch standing");
    std::fs::remove_dir(&directory).expect("removed");
    let standing: bool = watch.wait(Duration::from_secs(30), std::ffi::OsStr::new("index.lock"));
    assert!(!standing, "the watch is over");
    remove(root.as_path());
}

//region Ownership: owned, replaced and a check that could not be made are three answers.

/// The check of a holder that cannot be made,
/// as under descriptor exhaustion.
fn unreadable(_: &Path, _: &std::fs::File, _: &str) -> Ownership {
    return Ownership::Unreadable(std::io::Error::from_raw_os_error(libc::EMFILE));
}

/// The path names the file the holder placed with its marker:
/// owned;
/// another file with the same text (the same batch taken again),
/// a missing file and the same file with other text:
/// replaced.
#[test]
fn the_ownership_check_tells_the_holders_own_file_from_a_replacement_with_the_same_text() {
    let (root, index) = index_in("index-lock-ownership");
    let lock_path: PathBuf = lock_path_of(&index);
    let marker: String = own_marker(80);
    let (staged, held) = stage_marker(&lock_path, 80, marker.as_str()).expect("staged");
    std::fs::rename(&staged, &lock_path).expect("placed");
    assert!(matches!(inspect_ownership(&lock_path, &held, marker.as_str()), Ownership::Owned));
    std::fs::remove_file(&lock_path).expect("removed");
    assert!(matches!(inspect_ownership(&lock_path, &held, marker.as_str()), Ownership::Replaced), "nothing at the path");
    std::fs::write(&lock_path, marker.as_bytes()).expect("the same batch's text in another file");
    assert!(matches!(inspect_ownership(&lock_path, &held, marker.as_str()), Ownership::Replaced), "same text, another inode");
    std::fs::remove_file(&lock_path).expect("removed");
    std::fs::hard_link(std::fs::canonicalize(&root).expect("root").join("git-dir/index"), &lock_path).expect("link to another file");
    assert!(matches!(inspect_ownership(&lock_path, &held, marker.as_str()), Ownership::Replaced), "another file");
    remove(root.as_path());
}

/// A holder whose check cannot be made keeps its lock when it goes out of scope,
/// says so,
/// and counts it as unsettled,
/// so the service ends after the batch and the restart reclaims the lock.
/// Before,
/// the failed check read as "another program's lock" and the lock was abandoned with this process's name in it.
#[test]
fn a_check_that_cannot_be_made_leaves_the_lock_and_marks_it_unsettled() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, index) = index_in("index-lock-unreadable-drop");
        let platform: Platform = Platform { inspect: unreadable, ..SYSTEM };
        let before: usize = unsettled_locks();
        let lock = acquire_on(&platform, &index, 81, &|_| {}).expect("taken");
        drop(lock);
        assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock stays"), own_marker(81));
        assert_eq!(unsettled_locks(), before + 1);
        remove(root.as_path());
    });
}

/// The same under a release:
/// the index is not installed,
/// the lock stays with the holder's marker,
/// the error names the lock and the cause,
/// and the lock counts as unsettled only when it is dropped (it is,
/// with the failed release).
#[test]
fn a_release_whose_check_cannot_be_made_keeps_the_lock_and_the_old_index() {
    let (root, index) = index_in("index-lock-unreadable-release");
    let platform: Platform = Platform { inspect: unreadable, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 82, &|_| {}).expect("taken");
    let built: PathBuf = index.with_file_name("index.built");
    std::fs::write(&built, "new index").expect("built");
    let error = lock.release_with(&built).expect_err("refused");
    let text: String = error.to_string();
    assert!(text.contains(lock_path_of(&index).to_string_lossy().as_ref()) && text.contains("could not be checked"), "{text}");
    assert_eq!(std::fs::read_to_string(&index).expect("index"), "old index");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock stays"), own_marker(82));
    remove(root.as_path());
}

//endregion

//region The watch: dropped before the next is set, and ended by a rename of its directory.

/// A wait whose watch ended sets the next one only after dropping the first:
/// with the quota of inotify instances used up by that one,
/// the second is created at all.
/// The closure plays a quota of one by refusing a watch while the first is open.
/// The foreign holder lets go inside the second call,
/// once the quota has been checked,
/// so no clock decides whether the lock is still held when the wait looks at it (a slow staging flush,
/// as under load,
/// changes nothing).
#[test]
fn an_ended_watch_is_dropped_before_the_next_is_set() {
    let (root, index) = index_in("index-lock-watch-quota");
    let foreign: PathBuf = lock_path_of(&index);
    std::fs::write(&foreign, "foreign").expect("foreign lock");
    let decoy: PathBuf = root.join("decoy");
    std::fs::create_dir(&decoy).expect("decoy");
    let calls: Cell<u32> = Cell::new(0);
    let quota_of_one = |directory: &Path| -> Option<Watch> {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            let ending: Watch = Watch::on(&decoy).expect("watch");
            std::fs::remove_dir(&decoy).expect("the watched directory goes");
            return Some(ending);
        }
        if crate::service_index_watch::live_watches_on_this_thread() > 0 {
            return None;
        }
        std::fs::remove_file(&foreign).expect("the foreign holder lets go");
        return Watch::on(directory);
    };
    let slow_flush = |_: &std::fs::File| -> std::io::Result<()> { std::thread::sleep(Duration::from_millis(600)); return Ok(()); };
    let platform: Platform = Platform { watch: &quota_of_one, sync: &slow_flush, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 90, &|_| {}).expect("taken once the foreign holder let go");
    assert_eq!(calls.get(), 2, "a second watch was set");
    drop(lock);
    remove(root.as_path());
}

/// A lock that is free at the first attempt needs no second watch:
/// the wait is for a held lock,
/// and the watch set before the attempt is the only one.
#[test]
fn a_lock_free_at_the_first_attempt_sets_one_watch() {
    let (root, index) = index_in("index-lock-watch-free");
    let calls: Cell<u32> = Cell::new(0);
    let counting = |directory: &Path| -> Option<Watch> {
        calls.set(calls.get() + 1);
        return Watch::on(directory);
    };
    let platform: Platform = Platform { watch: &counting, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 92, &|_| {}).expect("taken");
    assert_eq!(calls.get(), 1, "one watch, set before the attempt");
    drop(lock);
    remove(root.as_path());
}

/// A directory renamed aside ends the watch on it:
/// the wait returns at once and says so,
/// though no file in it changed.
#[test]
fn a_watch_whose_directory_is_renamed_aside_ends() {
    let root: PathBuf = fixture("index-lock-watch-moved");
    let directory: PathBuf = root.join("watched");
    std::fs::create_dir(&directory).expect("directory");
    let watch: Watch = Watch::on(&directory).expect("watch");
    assert!(watch.wait(Duration::from_millis(50), std::ffi::OsStr::new("index.lock")), "a quiet directory leaves the watch standing");
    std::fs::rename(&directory, root.join("moved")).expect("renamed aside");
    assert!(!watch.wait(Duration::from_secs(30), std::ffi::OsStr::new("index.lock")), "the watch is over");
    remove(root.as_path());
}

//endregion

//region The hold's age, the identity of a process, the boot, and the staged marker's flush.

/// A lock that waited long between staging and placement is as new as its placement:
/// the file the wrapped commands judge by age carries the time of the placement.
/// The flush stands in for the long wait by making the staged file old.
#[test]
fn a_hold_is_as_old_as_its_placement_not_its_staging() {
    let (root, index) = index_in("index-lock-age");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - Duration::from_secs(3600);
    let aging = |file: &std::fs::File| -> std::io::Result<()> { return file.set_modified(long_ago); };
    let platform: Platform = Platform { sync: &aging, ..SYSTEM };
    let lock = acquire_on(&platform, &index, 91, &|_| {}).expect("taken");
    let age: Duration = std::fs::metadata(lock_path_of(&index)).and_then(|metadata| { return metadata.modified(); }).expect("modified")
        .elapsed().expect("not in the future");
    assert!(age < Duration::from_secs(60), "the lock is {age:?} old");
    drop(lock);
    remove(root.as_path());
}

/// A `/proc` that numbers this process otherwise is another PID namespace's:
/// no lock is taken on its word.
/// The same record with the process's own number passes,
/// whatever bytes the process's name holds.
#[test]
fn a_proc_of_another_pid_namespace_is_refused() {
    let own: u32 = std::process::id();
    assert!(check_procfs_view(format!("{own} (cli) S 1 2 3").as_bytes(), own).is_ok());
    let error = check_procfs_view(b"1 (init) S 0 1 1", 4242).expect_err("another namespace");
    assert!(matches!(error, ProcessIdentityError::ProcMismatch { procfs: 1, own: 4242 }), "{error}");
    assert!(error.to_string().contains("PID 1") && error.to_string().contains("4242"), "{error}");
    assert!(matches!(check_procfs_view(b"", own), Err(ProcessIdentityError::MalformedStat { .. })));
    // The identity is not given on a mismatch, and is on the process's own view; the real one agrees here.
    let boot: &str = "0123456789abcdef0123456789abcdef";
    assert!(matches!(identity_in_view(b"1 (init) S 0 1 1", own, boot, Some("1")), Err(ProcessIdentityError::ProcMismatch { .. })));
    assert!(identity_in_view(format!("{own} (cli) S 1 2 3").as_bytes(), own, boot, Some("1")).is_ok());
    assert!(own_identity().is_ok());
}

/// A process whose name holds bytes that are not UTF-8 (Linux allows any) still has a record that numbers it:
/// only the leading digits are read.
/// Before,
/// the record was read as text and such a process could take no lock at all.
#[test]
fn a_process_name_that_is_not_utf8_does_not_stop_the_identity() {
    let own: u32 = std::process::id();
    let mut record: Vec<u8> = format!("{own} (").into_bytes();
    record.extend_from_slice(&[0xff, 0xfe, b'x']);
    record.extend_from_slice(b") S 1 2 3");
    assert!(std::str::from_utf8(record.as_slice()).is_err(), "the premise: not text");
    assert_eq!(pid_of_self_stat(record.as_slice()), Some(i64::from(own)));
    assert!(identity_in_view(record.as_slice(), own, "0123456789abcdef0123456789abcdef", Some("4026531836")).is_ok());
}

/// On Linux a process that cannot name its boot or its PID namespace takes no lock:
/// a marker without them cannot be told from a reboot's collision or from a number of another namespace.
/// With both,
/// the identity ends in them,
/// in that order.
#[test]
fn an_identity_needs_a_boot_and_a_namespace() {
    let own: u32 = std::process::id();
    let record: String = format!("{own} (cli) S 1 2 3");
    let boot: &str = "0123456789abcdef0123456789abcdef";
    assert!(matches!(identity_in_view(record.as_bytes(), own, "", Some("4026531836")), Err(ProcessIdentityError::BootUnavailable)));
    assert!(matches!(identity_in_view(record.as_bytes(), own, boot, None), Err(ProcessIdentityError::ScopeUnavailable)));
    let identity: String = identity_in_view(record.as_bytes(), own, boot, Some("4026531836")).expect("identity");
    assert!(identity.ends_with(format!("#{boot}#4026531836").as_str()), "{identity}");
    assert!(ProcessIdentityError::BootUnavailable.to_string().contains("boot ID"));
    assert!(ProcessIdentityError::ScopeUnavailable.to_string().contains("PID namespace"));
}

/// The identity a marker carries names the boot and the PID namespace:
/// it ends in both,
/// and a process identity read back from it is the process's own.
#[test]
fn the_identity_in_a_marker_names_the_boot_and_the_namespace() {
    let identity: String = own_identity().expect("identity");
    let parts: Vec<&str> = identity.split('#').collect();
    // The machine ID of the host comes fourth when the host has a valid one.
    let machine: &str = crate::claim_time::current_machine_id();
    assert_eq!(parts.len(), if machine.is_empty() { 3 } else { 4 }, "{identity}");
    assert_eq!(parts.get(3).copied().unwrap_or(""), machine, "{identity}");
    assert_eq!(parts[1].len(), 32, "{identity}");
    assert!(!parts[2].is_empty() && parts[2].bytes().all(|byte| { return byte.is_ascii_digit(); }), "{identity}");
    assert_eq!(process_birth_identity(i64::from(std::process::id())).expect("identity").as_deref(), Some(parts[0]));
    let marked: Marked = parse_marker(own_marker(92).as_str()).expect("a marker");
    assert_eq!(marked.holder.map(|(_, text)| { return text; }), Some(identity));
}

/// A marker of another boot is gone though a process of that number and start tick runs now,
/// and is not proven running;
/// the same marker with this boot's ID,
/// and one an earlier release wrote without a boot,
/// are judged by the process alone.
#[test]
fn a_marker_of_another_boot_is_gone_though_the_same_number_and_tick_run() {
    let pid: i64 = i64::from(std::process::id());
    let own: String = own_identity().expect("identity");
    let parts: Vec<&str> = own.split('#').collect();
    // The marker of a process of this machine in another boot: the machine ID is the host's, if it has one.
    let other_boot: (i64, String) = (pid, format!("{}#{}#{}{}", parts[0], "0".repeat(32), parts[2], parts.get(3).map_or_else(String::new, |machine| { return format!("#{machine}"); })));
    assert!(holder_is_gone(Some(&other_boot)));
    assert!(!holder_is_running(Some(&other_boot)));
    let this_boot: (i64, String) = (pid, own.clone());
    assert!(!holder_is_gone(Some(&this_boot)));
    assert!(holder_is_running(Some(&this_boot)));
    let legacy: (i64, String) = (pid, parts[0].to_string());
    assert!(!holder_is_gone(Some(&legacy)), "an earlier release's marker names no boot");
    assert!(holder_is_running(Some(&legacy)));
    let without_namespace: (i64, String) = (pid, format!("{}#{}", parts[0], parts[1]));
    assert!(holder_is_running(Some(&without_namespace)), "a marker of the release before namespaces names none");
    assert!(!holder_is_gone(None) && !holder_is_running(None), "a marker that names no process proves neither");
}

/// What a reader makes of a holder depends on whether it can compare like with like:
/// a boot that ended ends the process whatever the reader's numbering;
/// another PID namespace,
/// a `/proc` that does not agree with the reader's PID,
/// or a reader with no boot ID to compare is
/// unknown,
/// which keeps the lock and proves nothing;
/// only the same boot and namespace let the process be asked.
#[test]
fn a_reader_that_cannot_compare_like_with_like_says_unknown_not_gone() {
    let boot: &str = "0123456789abcdef0123456789abcdef";
    let holder: (i64, String) = (100, format!("linux:777#{boot}#4026532000"));
    let view = |at_boot: &'static str, scope: Option<&'static str>, procfs_agrees: bool| -> ReaderView<'static> { return ReaderView { boot: at_boot, scope, machine: None, procfs_agrees }; };
    let boot_now: &'static str = "0123456789abcdef0123456789abcdef";
    let asked: Cell<usize> = Cell::new(0);
    let missing = |_: i64| -> Result<Option<String>, ProcessIdentityError> {
        asked.set(asked.get() + 1);
        return Ok(None);
    };
    let same = |_: i64| -> Result<Option<String>, ProcessIdentityError> { return Ok(Some(String::from("linux:777"))) };
    let born_since = |_: i64| -> Result<Option<String>, ProcessIdentityError> { return Ok(Some(String::from("linux:778"))) };
    let unreadable = |_: i64| -> Result<Option<String>, ProcessIdentityError> { return Err(ProcessIdentityError::CurrentUnavailable) };
    let home = view(boot_now, Some("4026532000"), true);
    assert_eq!(holder_state_in(Some(&holder), &home, &same), HolderState::Running);
    assert_eq!(holder_state_in(Some(&holder), &home, &born_since), HolderState::Gone);
    assert_eq!(holder_state_in(Some(&holder), &home, &missing), HolderState::Gone);
    assert_eq!(holder_state_in(Some(&holder), &home, &unreadable), HolderState::Unknown);
    assert_eq!(asked.get(), 1);
    asked.set(0);
    let host = view(boot_now, Some("4026531836"), true);
    assert_eq!(holder_state_in(Some(&holder), &host, &missing), HolderState::Unknown, "a number of another namespace is not a local process");
    assert_eq!(asked.get(), 0, "the process is not asked about in the wrong namespace");
    assert_eq!(holder_state_in(Some(&holder), &view(boot_now, None, true), &missing), HolderState::Unknown, "no namespace to compare");
    assert_eq!(holder_state_in(Some(&holder), &view(boot_now, Some("4026532000"), false), &missing), HolderState::Unknown, "a /proc that disagrees");
    assert_eq!(holder_state_in(Some(&holder), &view("", Some("4026532000"), true), &missing), HolderState::Unknown, "no boot ID to compare");
    let other_boot = view("fedcba9876543210fedcba9876543210", Some("4026531836"), false);
    assert_eq!(holder_state_in(Some(&holder), &other_boot, &same), HolderState::Gone, "another boot ends every process, whatever the numbering");
    assert_eq!(holder_state_in(None, &host, &same), HolderState::Unknown, "a marker without a holder says nothing about any process");
    let legacy: (i64, String) = (100, String::from("linux:777"));
    assert_eq!(holder_state_in(Some(&legacy), &host, &same), HolderState::Running, "a marker without boot and namespace is judged by the process");
}

/// The staged marker is flushed before it becomes the lock,
/// and a flush that fails ends the attempt with no lock and no staging file left.
#[test]
fn the_staged_marker_is_flushed_before_it_becomes_the_lock() {
    let (root, index) = index_in("index-lock-flush");
    let order: std::cell::RefCell<Vec<&str>> = std::cell::RefCell::new(Vec::new());
    let sync = |file: &std::fs::File| -> std::io::Result<()> {
        order.borrow_mut().push("sync");
        return file.sync_all();
    };
    let rename = |from: &Path, to: &Path| -> std::io::Result<()> {
        order.borrow_mut().push("rename");
        return rename_noreplace(from, to);
    };
    let platform: Platform = Platform { sync: &sync, rename: &rename, ..SYSTEM };
    drop(acquire_on(&platform, &index, 93, &|_| {}).expect("taken"));
    assert_eq!(order.borrow().as_slice(), ["sync", "rename"]);
    let failing = |_: &std::fs::File| -> std::io::Result<()> { return refusal(libc::EIO); };
    let broken: Platform = Platform { sync: &failing, ..SYSTEM };
    let failure = acquire_on(&broken, &index, 94, &|_| {}).expect_err("refused");
    assert_eq!(failure.code, FailureCode::Unavailable);
    assert!(try_acquire_on(&broken, &index, 95).is_none());
    assert!(!lock_path_of(&index).exists());
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

//endregion

/// The setting of the repetition of the holder's line:
/// a positive number of seconds is used (blanks around it are ignored);
///  zero,
/// a negative number,
/// text and no setting give ten seconds,
/// because a zero would repeat the line without a pause.
#[test]
fn the_notice_interval_is_a_positive_number_of_seconds_or_the_default() {
    assert_eq!(notice_interval(Some("5")), Duration::from_secs(5));
    assert_eq!(notice_interval(Some(" 7 ")), Duration::from_secs(7));
    assert_eq!(notice_interval(Some("1")), Duration::from_secs(1));
    for unusable in [None, Some("0"), Some("-3"), Some("soon"), Some("")] {
        assert_eq!(notice_interval(unusable), NOTICE_EVERY, "{unusable:?}");
    }
}

/// A failure to place says what failed,
/// a lock that exists is no failure,
/// and a filesystem that answers that it cannot place a file is named with what to do:
/// the log is the only place the person who reads it finds these.
#[test]
fn a_placement_failure_is_worded_by_what_failed() {
    let (root, index) = index_in("index-lock-failure-text");
    let lock_path: PathBuf = lock_path_of(&index);
    let held = std::io::Error::from(std::io::ErrorKind::AlreadyExists);
    assert_eq!(placement_failure_text(&lock_path, &held), None, "a held lock is not a failure to place");
    let storage = std::io::Error::from_raw_os_error(libc::EIO);
    let storage_said: String = placement_failure_text(&lock_path, &storage).expect("a failure");
    assert!(storage_said.contains("cannot take the index lock") && storage_said.contains(lock_path.to_string_lossy().as_ref()), "{storage_said}");
    assert!(storage_said.contains(storage.to_string().as_str()) && !storage_said.contains("can neither rename"), "{storage_said}");
    let unsupported = std::io::Error::new(std::io::ErrorKind::Unsupported, "no way");
    let unsupported_said: String = placement_failure_text(&lock_path, &unsupported).expect("a failure");
    assert!(unsupported_said.contains("cannot place a file") && unsupported_said.contains("filesystem") && unsupported_said.contains("no way"), "{unsupported_said}");
    assert!(!unsupported_said.contains("can neither rename"), "the refusal of both atomic ways is no longer a failure: {unsupported_said}");
    remove(root.as_path());
}

//region A lock the holder owns that cannot be removed is unsettled, never forgotten.

/// The removal of a lock that the directory's permissions refuse.
fn refused_removal(_: &Path) -> std::io::Result<()> {
    return refusal(libc::EACCES);
}

/// The removal of a lock that is already gone.
fn vanished_removal(path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)?;
    return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
}

/// An owned lock whose removal fails when the holder goes out of scope stays,
/// is said in the log,
/// and counts as unsettled,
/// so the service ends after the batch and the restart reclaims it.
/// Before,
/// the failure was dropped and a lock naming this live process stayed for the whole life of the process.
#[test]
fn an_owned_lock_that_cannot_be_removed_is_unsettled() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, index) = index_in("index-lock-unremovable-owned");
        let platform: Platform = Platform { release: refused_removal, ..SYSTEM };
        let before: usize = unsettled_locks();
        let lock = acquire_on(&platform, &index, 100, &|_| {}).expect("taken");
        drop(lock);
        assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock stays"), own_marker(100));
        assert_eq!(unsettled_locks(), before + 1);
        remove(root.as_path());
    });
}

/// The same under a release:
/// the index is installed by the rename,
/// the failure to remove the lock is returned,
/// and the lock is counted as unsettled.
#[test]
fn a_release_whose_lock_cannot_be_removed_is_unsettled() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, index) = index_in("index-lock-unremovable-release");
        let platform: Platform = Platform { release: refused_removal, ..SYSTEM };
        let before: usize = unsettled_locks();
        let lock = acquire_on(&platform, &index, 101, &|_| {}).expect("taken");
        let built: PathBuf = index.with_file_name("index.built");
        std::fs::write(&built, "new index").expect("built");
        let error = lock.release_with(&built).expect_err("the removal failed");
        assert_eq!(error.raw_os_error(), Some(libc::EACCES));
        assert_eq!(std::fs::read_to_string(&index).expect("index"), "new index", "the index was installed");
        assert!(unsettled_locks() > before);
        assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock stays"), own_marker(101));
        remove(root.as_path());
    });
}

/// A lock that is already gone when the holder removes it is removed:
/// nothing is unsettled.
#[test]
fn an_owned_lock_that_is_already_gone_is_not_unsettled() {
    let (root, index) = index_in("index-lock-vanished-owned");
    let platform: Platform = Platform { release: vanished_removal, ..SYSTEM };
    let before: usize = unsettled_locks();
    let lock = acquire_on(&platform, &index, 102, &|_| {}).expect("taken");
    drop(lock);
    assert!(!lock_path_of(&index).exists());
    assert_eq!(unsettled_locks(), before);
    remove(root.as_path());
}

//endregion

/// A lock that a power loss brought back after its release (the unlink was not yet on storage) carries the marker of the boot that ended:
/// the next acquisition reclaims it and takes the lock without waiting at all,
/// though a process of that number and tick runs now.
/// The ceiling the acquisition is given is already past when it begins,
/// so a lock waited for like a live holder's would end in the ceiling's words at once;
/// a reclaim announces no wait,
/// and that is the claim,
/// where a stopwatch measured the flush of the marker instead
/// (measured 2026-11-08:
/// 14.3 s under a capped flush).
/// This is why the release does not sync the directory.
#[test]
fn a_lock_that_came_back_from_an_earlier_boot_is_reclaimed_by_the_next_acquisition() {
    let (root, index) = index_in("index-lock-came-back");
    let own: String = own_identity().expect("identity");
    let parts: Vec<&str> = own.split('#').collect();
    let pid: u32 = std::process::id();
    std::fs::write(lock_path_of(&index), format!("{LOCK_MARKER}{}103 {pid} {}#{}#{}{}\n", crate::service_index_marker::MARKER_VERSION, parts[0], "0".repeat(32), parts[2], parts.get(3).map_or_else(String::new, |machine| { return format!("#{machine}"); }))).expect("the lock that came back");
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let lock = acquire_within(&SYSTEM, &index, 104, NO_CEILING, &|note| { heard.lock().expect("lock").push(note); }).expect("taken with no wait");
    let said: Vec<Announce> = heard.lock().expect("lock").clone();
    assert!(said.is_empty(), "the lock that came back from an earlier boot was waited for: {said:?}");
    assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("marker"), own_marker(104));
    drop(lock);
    remove(root.as_path());
}

//region What a wait for the lock found is written down for the service's log.

/// A lock that is free at the first placement leaves no description and no looks:
/// the fast path pays nothing for the notes.
/// The flush of the marker is timed all the same.
#[test]
fn a_free_lock_leaves_no_description() {
    let (root, index) = index_in("index-lock-notes-free");
    let mut notes: WaitNotes = WaitNotes::default();
    let lock = acquire_noting_wait(&index, 110, &|_| {}, &mut notes).expect("taken");
    assert_eq!((notes.looks, notes.found.as_ref(), notes.last.as_ref()), (0, None, None));
    assert!(notes.staging > Duration::ZERO, "the flush of the marker is timed");
    drop(lock);
    remove(root.as_path());
}

/// A foreign lock that is waited for is described at the first look and at the last:
/// it has no marker,
/// and a process that has it open is named.
/// The wait is timed apart from the flush.
#[test]
fn a_foreign_lock_is_described_with_the_process_that_has_it_open() {
    let (root, index) = index_in("index-lock-notes-foreign");
    let lock_path: PathBuf = lock_path_of(&index);
    std::fs::write(&lock_path, "").expect("foreign lock");
    // A process that has the lock open, as a Git that writes the index has.
    let holder: Holder = Holder(
        std::process::Command::new("sh").arg("-c").arg("exec 3<\"$0\"; exec sleep 30").arg(&lock_path).spawn().expect("a holder"),
    );
    let pid: u32 = holder.0.id();
    let began: Instant = Instant::now();
    // The holder is ready when it has the lock open and has become `sleep`: the shell opens the lock first and executes `sleep` after,
    //  so a look in between names the shell (measured in the container gate, 2026-11-07: `open_by=[14748 (sh)]`).
    while !(std::fs::read_dir(format!("/proc/{pid}/fd")).is_ok_and(|entries| {
        return entries.flatten().any(|entry| { return std::fs::read_link(entry.path()).is_ok_and(|target| { return target == lock_path; }); });
    }) && std::fs::read_to_string(format!("/proc/{pid}/comm")).is_ok_and(|name| { return name.trim() == "sleep"; }))
    {
        assert!(began.elapsed() < Duration::from_secs(10), "the holder never opened the lock and became sleep");
        std::thread::sleep(Duration::from_millis(5));
    }
    // The foreign holder lets go a while after the wait's first look, however long the marker's flush took before it.
    let (looked, looking) = std::sync::mpsc::channel::<()>();
    let release = std::thread::spawn({
        let path: PathBuf = lock_path.clone();
        move || {
            looking.recv_timeout(Duration::from_secs(60)).expect("the wait looked at the lock");
            std::thread::sleep(Duration::from_millis(400));
            std::fs::remove_file(path).expect("the foreign holder lets go");
        }
    });
    let letting_go: LetsGoOnEveryPath<'_> = LetsGoOnEveryPath(&looked);
    let mut notes: WaitNotes = WaitNotes::default();
    let lock = acquire_noting_wait(&index, 111, &|_| { let _sent = looked.send(()); }, &mut notes).expect("taken after the wait");
    drop(letting_go);
    release.join().expect("thread");
    assert!(notes.looks >= 1, "{notes:?}");
    let found: &str = notes.found.as_deref().expect("a description of the first look");
    assert!(found.starts_with("foreign no-marker size=0 age_ms="), "{found}");
    assert!(found.contains(format!("open_by=[{pid} (sleep)]").as_str()), "{found}");
    // The name is the program's `comm`; the arguments the holder was started with (`30`) are not in the line.
    assert!(!found.contains("sleep 30") && !found.contains(" 30"), "{found}");
    assert!(notes.last.is_some());
    assert!(notes.placing >= Duration::from_millis(300), "the wait for the holder is timed: {notes:?}");
    drop(lock);
    drop(holder);
    remove(root.as_path());
}

/// A lock with the service's marker is described by its marker:
/// the batch,
/// the process,
/// and what a reader makes of that process,
/// here a service that runs.
#[test]
fn a_service_lock_is_described_by_its_marker_and_the_state_of_its_holder() {
    let (root, index) = index_in("index-lock-notes-service");
    let lock_path: PathBuf = lock_path_of(&index);
    std::fs::write(&lock_path, own_marker(112)).expect("a running service's lock");
    let said: String = describe_lock(&lock_path);
    assert!(said.starts_with("service seq=112 pid="), "{said}");
    assert!(said.contains(format!("pid={} state=running age_ms=", std::process::id()).as_str()), "{said}");
    // The marker's birth identity, boot ID and PID namespace are not in the line: the state says what they were used for.
    let identity: String = own_identity().expect("identity");
    assert!(!said.contains("birth") && !said.contains(identity.split('#').nth(1).expect("boot")), "{said}");
    std::fs::write(&lock_path, marker_of_holder(113, dead_pid(), "linux:1")).expect("a dead service's lock");
    assert!(describe_lock(&lock_path).contains("state=gone"));
    std::fs::write(&lock_path, "cli-git-service 114\n").expect("an earlier release's lock");
    assert!(describe_lock(&lock_path).contains("holder=none (an earlier release's marker)"));
    std::fs::remove_file(&lock_path).expect("removed");
    assert_eq!(describe_lock(&lock_path), "vanished");
    remove(root.as_path());
}

//endregion

//region Survivors of the mutation campaign over round 4: each control checks the behaviour one mutant changes.

/// A marker whose holder fields are present but whose identity is empty is no marker:
/// a trailing space after the process number is a cut,
/// not a holder.
#[test]
fn a_marker_with_an_empty_identity_is_no_marker() {
    assert_eq!(parse_marker("cli-git-service 7 123 \n"), None);
    assert_eq!(parse_marker("cli-git-service 7  \n"), None);
    assert_eq!(parse_marker("cli-git-service 7 123\n"), None, "a process number alone");
    assert!(parse_marker("cli-git-service 7 123 linux:1\n").is_some());
}

/// The PID namespace is the number of a `pid:[number]` link and nothing else:
/// an empty number,
/// letters in it,
/// another kind of namespace and a missing bracket are no namespace.
#[test]
fn only_a_pid_link_with_a_number_names_a_scope() {
    use crate::service_index_identity::scope_of_link;
    assert_eq!(scope_of_link("pid:[4026531836]").as_deref(), Some("4026531836"));
    for other in ["pid:[]", "pid:[12ab]", "pid:[ab]", "pid:[1 2]", "net:[4026531836]", "pid:[4026531836", "pid:4026531836]", "", "pid:[-1]"] {
        assert_eq!(scope_of_link(other), None, "{other:?}");
    }
}

/// A process's `/proc` numbers it as the system does only when its own record is readable and names its own number:
/// an unreadable record,
/// an empty one and one of another process do not.
#[test]
fn a_proc_agrees_only_when_its_record_names_the_process() {
    use crate::service_index_identity::procfs_agrees_with;
    let own: u32 = std::process::id();
    assert!(procfs_agrees_with(Ok(format!("{own} (cli) S 1").into_bytes()), own));
    assert!(!procfs_agrees_with(Err(std::io::Error::from_raw_os_error(libc::EACCES)), own), "an unreadable record proves nothing");
    assert!(!procfs_agrees_with(Ok(Vec::new()), own));
    assert!(!procfs_agrees_with(Ok(b"1 (init) S 0".to_vec()), own));
    assert!(crate::service_index_identity::ReaderView::current().procfs_agrees, "the real /proc of this process agrees");
}

thread_local! {
    /// How many times the check of the controls below was made on this thread.
    static CHECKS: Cell<usize> = const { Cell::new(0) };
}

/// A check that answers `Owned` and counts its calls.
fn counted_owned(_: &Path, _: &std::fs::File, _: &str) -> Ownership {
    CHECKS.with(|checks| { checks.set(checks.get() + 1); });
    return Ownership::Owned;
}

/// A check that cannot be made,
/// counting its calls.
fn counted_unreadable(_: &Path, _: &std::fs::File, _: &str) -> Ownership {
    CHECKS.with(|checks| { checks.set(checks.get() + 1); });
    return Ownership::Unreadable(std::io::Error::from_raw_os_error(libc::EMFILE));
}

/// A check that cannot be made the first time and answers the second.
fn unreadable_once(_: &Path, _: &std::fs::File, _: &str) -> Ownership {
    let calls: usize = CHECKS.with(|checks| { checks.set(checks.get() + 1); return checks.get(); });
    return if calls == 1 { Ownership::Unreadable(std::io::Error::from_raw_os_error(libc::EMFILE)) } else { Ownership::Owned };
}

/// The holder asks about its lock once when the answer is final,
/// and again,
/// up to a bound,
/// only when the check could not be
/// made:
/// an answer is not asked for twice,
/// and a failure that passes is not taken for the final one.
#[test]
fn a_check_is_repeated_only_while_it_cannot_be_made() {
    let (root, index) = index_in("index-lock-check-count");
    let measure = |check: fn(&Path, &std::fs::File, &str) -> Ownership| -> usize {
        CHECKS.with(|checks| { checks.set(0); });
        let platform: Platform = Platform { inspect: check, ..SYSTEM };
        let lock = acquire_on(&platform, &index, 120, &|_| {}).expect("taken");
        let built: PathBuf = index.with_file_name("index.built");
        std::fs::write(&built, "new index").expect("built");
        let _released = lock.release_with(&built);
        // A check that cannot be made leaves the lock for the restart; the next measurement starts without it.
        let _removed = std::fs::remove_file(lock_path_of(&index));
        return CHECKS.with(Cell::get);
    };
    assert_eq!(measure(counted_owned), 1, "a final answer is asked for once");
    assert!(measure(counted_unreadable) > 1, "a check that cannot be made is asked again");
    assert_eq!(measure(unreadable_once), 2, "the second look answers, so there is no third");
    remove(root.as_path());
}

/// What:
///  The flush the placement asks of the index lock's marker is the service's background task's:
///  `sync_file` queues a handle on the marker and answers the queueing,
///  so a file the operating system refuses to flush (a pipe has no data to make durable) is not an error there,
///  and a file it can flush is not one either.
/// Why:
///  The marker names the process that holds the lock and nothing else,
///  and its flush sat on every commit's waiting path,
///  8 to 12.5 s of it under load,
///  so 322079b2f0 handed it to the background task
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
///  A marker a power cut tears names no holder,
///  and the step that runs before a landing removes exactly that file at once
///  (`service_index_unheld_removal.rs`).
///  Durability itself is not observable without a crash,
///  and neither is the background task's answer:
///  what is observable here is that no refusal of a marker's flush reaches the commit,
///  and `service_durable_tests.rs` shows the flush being made there
///  (`the_lock_markers_flush_is_queued_and_the_background_task_makes_it`).
#[test]
fn the_flush_of_the_marker_is_the_background_tasks_and_a_refusal_fails_nothing() {
    use crate::service_index_placement::sync_file;
    use std::os::fd::FromRawFd;
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `pipe` fills the two integers of the array it is given.
    assert_eq!(unsafe { libc::pipe(ends.as_mut_ptr()) }, 0);
    // SAFETY: each descriptor is a fresh end of the pipe, owned by exactly one `File`.
    let (reader, writer) = unsafe { (std::fs::File::from_raw_fd(ends[0]), std::fs::File::from_raw_fd(ends[1])) };
    // Positive control: the operating system does refuse this flush, so it is the queueing and not a lenient answer that keeps
    // the refusal off the commit.
    let refused = writer.sync_all().expect_err("a pipe cannot be flushed");
    assert_eq!(refused.raw_os_error(), Some(libc::EINVAL));
    assert!(sync_file(&writer).is_ok(), "a flush the system refuses is never the placement's to fail");
    drop((reader, writer));
    let (root, index) = index_in("index-lock-sync-file");
    let file = std::fs::File::create(index.with_file_name("a-file")).expect("a file");
    assert!(sync_file(&file).is_ok());
    crate::service_durable::drain();
    remove(root.as_path());
}

/// A rename that fails for any reason but "this filesystem cannot do it" ends the placement with that error and tries no hard
/// link:
/// a lock that exists,
/// storage that failed and a refusal are not reasons to place the lock another way.
/// The reasons that do say the filesystem cannot are the ones the system names for it.
#[test]
fn only_an_unsupported_rename_falls_back_to_a_link() {
    let (root, index) = index_in("index-lock-fallback");
    let lock_path: PathBuf = lock_path_of(&index);
    let staged: PathBuf = root.join("staged");
    std::fs::write(&staged, "marker").expect("staged");
    let tried_link: Cell<bool> = Cell::new(false);
    let link = |_: &Path, _: &Path| -> std::io::Result<()> {
        tried_link.set(true);
        return refusal(libc::EPERM);
    };
    for code in [libc::EEXIST, libc::EIO, libc::EACCES, libc::ENOSPC, libc::EXDEV] {
        tried_link.set(false);
        let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(code); };
        let platform: Platform = Platform { rename: &rename, link: &link, ..SYSTEM };
        let error = place_marked(&platform, &staged, &lock_path).expect_err("refused");
        assert_eq!(error.raw_os_error(), Some(code), "the rename's own error comes back");
        assert!(!tried_link.get(), "errno {code} is no reason to link");
    }
    for code in [libc::EINVAL, libc::ENOSYS, libc::EOPNOTSUPP] {
        tried_link.set(false);
        let rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(code); };
        let platform: Platform = Platform { rename: &rename, link: &link, ..SYSTEM };
        let error = place_marked(&platform, &staged, &lock_path).expect_err("neither way works");
        assert!(tried_link.get(), "errno {code} says the filesystem cannot rename this way, so a link is tried");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    }
    remove(root.as_path());
}

/// A hard link that fails for any reason but "this filesystem makes none" ends the placement with that error;
/// the two reasons the system names for it (`EPERM`,
/// `EOPNOTSUPP`) say neither way works.
#[test]
fn only_an_unsupported_link_ends_as_unsupported() {
    let (root, index) = index_in("index-lock-link-reasons");
    let lock_path: PathBuf = lock_path_of(&index);
    let staged: PathBuf = root.join("staged");
    std::fs::write(&staged, "marker").expect("staged");
    let no_rename = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(libc::EINVAL); };
    for code in [libc::EEXIST, libc::EIO, libc::EACCES, libc::ENOSPC] {
        let link = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(code); };
        let platform: Platform = Platform { rename: &no_rename, link: &link, ..SYSTEM };
        let error = place_marked(&platform, &staged, &lock_path).expect_err("refused");
        assert_eq!(error.raw_os_error(), Some(code), "the link's own error comes back");
    }
    for code in [libc::EPERM, libc::EOPNOTSUPP] {
        let link = |_: &Path, _: &Path| -> std::io::Result<()> { return refusal(code); };
        let platform: Platform = Platform { rename: &no_rename, link: &link, ..SYSTEM };
        let error = place_marked(&platform, &staged, &lock_path).expect_err("neither way works");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported, "errno {code}");
    }
    remove(root.as_path());
}

/// The rename that does not replace really places the file:
/// the staged name is gone,
/// the lock holds its text,
/// and a second rename onto the lock fails with `AlreadyExists`
/// and leaves both files as they were.
#[test]
#[cfg(target_os = "linux")]
fn the_rename_that_does_not_replace_places_the_file_and_refuses_a_held_lock() {
    let (root, index) = index_in("index-lock-rename-real");
    let lock_path: PathBuf = lock_path_of(&index);
    let first: PathBuf = root.join("first");
    let second: PathBuf = root.join("second");
    std::fs::write(&first, "first").expect("first");
    std::fs::write(&second, "second").expect("second");
    rename_noreplace(&first, &lock_path).expect("placed");
    assert!(!first.exists(), "the staged name is gone");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("lock"), "first");
    let refused = rename_noreplace(&second, &lock_path).expect_err("a lock that exists is not replaced");
    assert_eq!(refused.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read_to_string(&second).expect("second stays"), "second");
    assert_eq!(std::fs::read_to_string(&lock_path).expect("lock stays"), "first");
    remove(root.as_path());
}

/// Environment variable that makes the test below run its inner half:
/// the number of the PID namespace the outer half left.
#[cfg(target_os = "linux")]
const OUTSIDE_SCOPE: &str = "INDEX_LOCK_TEST_OUTSIDE_PID_SCOPE";

/// A process in a PID namespace of its own,
/// still reading the `/proc` of the namespace it left,
/// is told by the view of the
/// reader:
/// its `/proc` does not number it as the system does,
/// and its scope is the new namespace's.
/// The outer half starts the inner half through `unshare`;
/// the inner half only runs when started that way.
#[test]
#[cfg(target_os = "linux")]
#[allow(
    clippy::disallowed_methods,
    reason = "starts this test binary, not the wrapper, under `unshare`; the ban on `current_exe` guards the wrapper's own start"
)]
fn a_new_pid_namespace_over_the_old_proc_is_told_from_the_view() {
    let Ok(outside) = std::env::var(OUTSIDE_SCOPE) else {
        let probe = std::process::Command::new("unshare").args(["--user", "--map-root-user", "--pid", "--fork", "true"]).output();
        if !probe.is_ok_and(|done| { return done.status.success(); }) {
            eprintln!("skipped: this host does not let an unprivileged process make a PID namespace");
            return;
        }
        let own: String = std::fs::read_link("/proc/self/ns/pid").expect("own namespace").to_string_lossy().to_string();
        let own_number: &str = own.strip_prefix("pid:[").and_then(|rest| { return rest.strip_suffix(']'); }).expect("a pid link");
        let done = std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "--pid", "--fork"])
            .arg(std::env::current_exe().expect("this test binary"))
            .args(["--exact", "landing::service_index_lock::tests::a_new_pid_namespace_over_the_old_proc_is_told_from_the_view", "--nocapture"])
            .env(OUTSIDE_SCOPE, own_number)
            .output()
            .expect("unshare runs");
        let text: String = String::from_utf8_lossy(done.stdout.as_slice()).to_string() + String::from_utf8_lossy(done.stderr.as_slice()).as_ref();
        assert!(done.status.success(), "the inner half failed:\n{text}");
        assert!(text.contains("1 passed"), "the inner half did not run:\n{text}");
        return;
    };
    let view = crate::service_index_identity::ReaderView::current();
    assert!(!view.procfs_agrees, "the old /proc numbers this process by the old namespace");
    let scope: &str = view.scope.expect("a pid link names a scope");
    assert!(scope.bytes().all(|byte| { return byte.is_ascii_digit(); }) && !scope.is_empty());
    assert_ne!(scope, outside, "the scope is the new namespace's");
}

//endregion


/// What the person is told about a holder proven alive:
/// its number,
/// its program,
/// when it started,
/// and the way out.
/// Why:
///  On 2026-11-07 a commit waited for a lock held by a live `git commit` and was told only that the owner "cannot be proven alive".
///  A holder the wrapper can see is named with what a person needs to end it if it is stuck:
/// the PID,
/// the program,
/// the start time and the command.
#[test]
fn a_holder_proven_alive_is_named_with_its_program_start_time_and_the_way_out() {
    let (root, index) = index_in("index-lock-named-holder");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    // The PID file names this process, which runs and started before the lock was made.
    let pid: u32 = std::process::id();
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {pid}\n")).expect("PID file");
    let program: String = std::fs::read_to_string("/proc/self/comm").expect("own name").trim().to_string();
    let started_at_ms: f64 = match crate::process_start::resolve_process_start(i64::from(pid)) {
        crate::process_start::ProcessStart::Running { started_at_ms, .. } => started_at_ms,
        other => panic!("this process runs: {other:?}"),
    };
    let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
    let refused = acquire_within(&SYSTEM, &index, 20, Duration::from_millis(200), &|note| { heard.lock().expect("lock").push(note); });
    assert!(refused.is_err(), "the holder never let go");
    let said: Vec<String> = heard.lock().expect("lock").iter().filter_map(|note| {
        return match note { Announce::Waiting(text) => Some(text.clone()), Announce::Holder(_) => None };
    }).collect();
    let first: &String = said.first().expect("the wait said what it waits for");
    assert!(first.contains(format!("PID {pid} ({program})").as_str()), "{first}");
    // The start time is read again from the system when the line is made, and the system derives it from the uptime, so two readings
    //  differ by a few milliseconds: the line's time is compared with this one within a second.
    let named: &str = first.split("started ").nth(1).and_then(|rest| { return rest.split(';').next(); }).expect("the line names a start time");
    let named_ms: i64 = named.parse::<jiff::Timestamp>().expect("an ISO time").as_millisecond();
    assert!((named_ms - started_at_ms as i64).abs() < 1000, "{first}");
    assert!(first.contains(format!("kill {pid}").as_str()), "{first}");
    remove(root.as_path());
}

/// A lock that already outlasted one commit's wait is refused at once to the commits queued behind it.
/// Why:
///  Four commits of one worktree each waited a full ceiling for the same lock one after another,
/// 678 to 1,229 s each,
/// because
///  every queued commit began its own wait.
/// The lock is the same file;
/// a second wait for it tells nothing the first did not.
/// What is claimed is the order:
/// the second commit's ending names the first commit's wait,
/// in the seconds that commit was told,
/// and the second is given no ceiling of its own,
/// so no clock of its own can be what its verdict came from.
#[test]
fn a_lock_that_outlasted_one_wait_is_refused_at_once_to_the_next_commit() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, index) = index_in("index-lock-refused-once");
        std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
        // Over a whole second, so the first wait's seconds cannot be the zero a fresh wait of the second commit would report.
        let first_ceiling: Duration = Duration::from_millis(1200);
        let first_heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
        assert!(
            acquire_within(&SYSTEM, &index, 21, first_ceiling, &|note| { first_heard.lock().expect("lock").push(note); }).is_err(),
            "the holder never let go"
        );
        let first_said: Vec<Announce> = first_heard.lock().expect("lock").clone();
        let ceiling_words: &String = first_said
            .iter()
            .find_map(|note| { return match note { Announce::Waiting(text) if text.contains("still held after") => Some(text), _ => None }; })
            .expect("the first wait ended in the ceiling's words");
        let first_seconds: &str = ceiling_words.split("still held after ").nth(1).and_then(|rest| { return rest.split(' ').next(); }).expect("the ceiling's line says how long the lock was held");
        assert!(first_seconds.parse::<u64>().expect("whole seconds") >= 1, "the first wait measured no whole second: {ceiling_words}");
        let heard: Mutex<Vec<Announce>> = Mutex::new(Vec::new());
        let next = acquire_within(&SYSTEM, &index, 22, NO_CEILING, &|note| { heard.lock().expect("lock").push(note); });
        assert_eq!(next.expect_err("a lock that outlasted a wait is refused").code, FailureCode::IndexInUse);
        let said: Vec<Announce> = heard.lock().expect("lock").clone();
        assert!(said.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains("still held")); }), "{said:?}");
        assert!(
            said.iter().any(|note| {
                return matches!(note, Announce::Waiting(text) if text.contains(format!("stopped an earlier commit after {first_seconds} seconds").as_str()));
            }),
            "the refusal names the earlier commit's wait of {first_seconds} seconds: {said:?}"
        );
        remove(root.as_path());
    });
}

/// A lock that is another file than the one that outlasted a wait is waited for in full.
#[test]
fn a_new_lock_is_waited_for_after_the_one_that_outlasted_a_wait_is_gone() {
    let (root, index) = index_in("index-lock-refused-then-replaced");
    std::fs::write(lock_path_of(&index), "foreign").expect("foreign lock");
    let ceiling: Duration = Duration::from_millis(400);
    assert!(acquire_within(&SYSTEM, &index, 23, ceiling, &|_| {}).is_err());
    std::fs::remove_file(lock_path_of(&index)).expect("the holder lets go");
    std::fs::write(lock_path_of(&index), "another foreign").expect("another program's lock");
    let began: Instant = Instant::now();
    assert!(acquire_within(&SYSTEM, &index, 24, ceiling, &|_| {}).is_err());
    assert!(began.elapsed() >= ceiling, "a different lock was refused at once: {:?}", began.elapsed());
    remove(root.as_path());
}

/// The landing's wait for a foreign lock is the longest self-released wait with a margin of five,
/// in whole minutes,
/// the way every
///  other ceiling of `wait_ceilings.rs` is derived.
#[test]
fn the_landing_ceiling_follows_the_longest_measured_foreign_wait() {
    use crate::wait_ceilings::{LANDING_LOCK_CEILING, LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT};
    const MARGIN: u32 = 5;
    const MINUTE: Duration = Duration::from_secs(60);
    let wanted: Duration = LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT * MARGIN;
    assert!(LANDING_LOCK_CEILING >= wanted, "{LANDING_LOCK_CEILING:?}");
    assert!(LANDING_LOCK_CEILING < wanted + MINUTE, "{LANDING_LOCK_CEILING:?}");
}

/// What:
///  A lock that is a named pipe names no marker at once,
/// and is described as a foreign holder.
/// Why:
///  The service reads the lock's marker before it places its own,
/// and opening a named pipe for
/// reading waits for a writer that may never come:
/// a landing stopped there,
/// so every commit behind it queued
///  behind a wait with no end
/// (`doc/troubleshooting/a-fifo-where-a-file-is-expected-ends-no-command.md`).
#[test]
fn a_lock_that_is_a_named_pipe_names_no_marker_at_once() {
    let (root, index) = index_in("index-lock-pipe");
    let lock_path: PathBuf = lock_path_of(&index);
    crate::subvolume_sync_attrs::make_fifo(lock_path.as_path(), 0o600).expect("named pipe");
    let named_pipe: PathBuf = lock_path.clone();
    let read: Option<Option<crate::service_index_marker::Marked>> = crate::test_support::finished_within(move || { return marker_of(named_pipe.as_path()); });
    let Some(marker) = read else { panic!("reading a marker out of a named pipe did not finish within five seconds") };
    assert_eq!(marker, None);
    let described_path: PathBuf = lock_path.clone();
    let described: Option<String> = crate::test_support::finished_within(move || { return describe_lock(described_path.as_path()); });
    let Some(description) = described else { panic!("describing a named pipe lock did not finish within five seconds") };
    assert!(description.starts_with("foreign no-marker"), "{description}");
    remove(root.as_path());
}
