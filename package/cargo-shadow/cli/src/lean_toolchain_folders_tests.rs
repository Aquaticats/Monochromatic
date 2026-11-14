//! The controls of mechanism G:
//! the toolchain folders the volume's free space asks for,
//! the ones a build keeps,
//! and the removal a killed pass leaves for the next one.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use crate::lean_guard::GuardEnvironment;
use crate::lean_identity::{Deadline, now_seconds};
use crate::lean_test_support::{DAY, age_everything, set_time};
use crate::lean_toolchain_folder_pieces::{Volume, arm_stop_after_pieces, children_of, disarm_stop_after_pieces, folder_name_of, retiring_directory};
use crate::lean_toolchain_folders::{
    Candidate,
    ToolchainFolderLimits,
    candidates,
    changed_since,
    identity_of_candidate,
    pieces_of,
    retire_toolchain_folders,
};
use crate::test_scratch::scratch;

/// A drive a test decides about.
struct FixtureVolume {
    /// The free bytes this volume answers with,
    /// which a test sets below or above the bound.
    free: Cell<u64>,
    /// The readings it answers with,
    /// in the order the pass asks for them.
    readings: RefCell<VecDeque<Duration>>,
}

/// A volume that has this much free space and answers every reading at once.
fn volume(free: u64) -> FixtureVolume {
    return FixtureVolume { free: Cell::new(free), readings: RefCell::new(VecDeque::new()) };
}

impl Volume for FixtureVolume {
    fn free_bytes(&self, _base: &Path) -> io::Result<u64> {
        return Ok(self.free.get());
    }

    fn reading(&self, _probe: &Path) -> io::Result<Duration> {
        return Ok(self.readings.borrow_mut().pop_front().unwrap_or(Duration::from_millis(1)));
    }
}

/// The bounds of these tests:
/// a bound the fixture volume can be above or below,
/// an age of one day,
/// half an hour of pass,
/// and a slow reading of a second.
fn limits() -> ToolchainFolderLimits {
    return ToolchainFolderLimits {
        free_space_bound_bytes: 100 * (1_u64 << 30),
        age_seconds: DAY,
        pass_limit: Duration::from_secs(1_800),
        slow_reading: Duration::from_secs(1),
        slow_readings_in_a_row: 2,
    };
}

/// A base with one toolchain folder of the pieces named,
/// each piece a tree with one built file in it.
fn make_folder(base: &Path, name: &str, shards: &[&str]) {
    for shard in shards {
        let tree: PathBuf = base.join("toolchain").join(name).join(shard).join("00000000000001");
        fs::create_dir_all(tree.join("debug").join("deps")).expect("a tree's deps directory");
        fs::write(tree.join("debug").join("deps").join("libalpha.rlib"), vec![3_u8; 4096]).expect("a built file");
    }
}

/// The machine the pass reads:
/// the real `/proc`,
/// so a test's own child process is found the way any process is.
fn environment() -> GuardEnvironment {
    return GuardEnvironment::real();
}

/// The deadline of these tests,
/// far enough away that it never decides anything.
fn deadline() -> Deadline {
    return Deadline::after(Duration::from_secs(600));
}

/// A child process whose working directory is inside a folder,
/// killed and reaped when it drops.
struct ChildInFolder {
    /// The process.
    child: Child,
}

/// Killing and reaping at the end of a test.
impl Drop for ChildInFolder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Starts a program that stays alive with its working directory in a folder.
fn process_in(folder: &Path) -> ChildInFolder {
    let child: Child = Command::new("sleep").arg("60").current_dir(folder).spawn().expect("start a process whose working directory is the folder");
    return ChildInFolder { child };
}

#[test]
fn a_folder_over_the_bound_goes_oldest_first_and_the_ones_a_build_uses_stay() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01"]);
    make_folder(&base, "nightly-2026-11-07", &["00"]);
    make_folder(&base, "nightly-2026-11-13", &["00"]);
    let now: i64 = now_seconds();
    // The folder of the toolchain nobody builds with has gone longest without a build;
    // the newer stale one a week less; the calling build's two days, which is still over the age.
    age_everything(&base.join("toolchain").join("nightly-2026-11-07"), now - 3 * DAY);
    age_everything(&base.join("toolchain").join("nightly-2026-11-10"), now - 30 * DAY);
    age_everything(&base.join("toolchain").join("nightly-2026-11-13"), now - 2 * DAY);
    let drive: FixtureVolume = volume(1);
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, Some("nightly-2026-11-13"), &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(!base.join("toolchain").join("nightly-2026-11-10").exists(), "the folder modified longest ago went: {log:?}");
    assert!(!base.join("toolchain").join("nightly-2026-11-07").exists(), "the volume was still short, so the next oldest went too: {log:?}");
    assert!(base.join("toolchain").join("nightly-2026-11-13").exists(), "the calling build's folder stays: {log:?}");
    assert!(outcome.lines.iter().any(|line| { return line.contains("kept toolchain folder") && line.contains("nightly-2026-11-13") && line.contains("the build this pass runs under"); }), "{:?}", outcome.lines);
    let oldest: usize = log.iter().position(|line| { return line.contains("removed toolchain folder") && line.contains("nightly-2026-11-10"); }).expect("a line about the oldest folder");
    let newer: usize = log.iter().position(|line| { return line.contains("removed toolchain folder") && line.contains("nightly-2026-11-07"); }).expect("a line about the second folder");
    assert!(oldest < newer, "the folders go in the order of their own modification time: {log:?}");
    assert_eq!(outcome.removed, 2, "{outcome:?}");
    assert_eq!(outcome.kept, 1, "{outcome:?}");
    assert!(outcome.bytes >= 3 * 4096, "the removal counts what it freed: {outcome:?}");
    assert_eq!(pieces_of(&retiring_directory(&base)).len(), 0, "a pass that finished leaves nothing claimed: {outcome:?}");
}

#[test]
fn a_volume_with_its_free_space_back_removes_nothing_and_says_nothing() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01"]);
    let now: i64 = now_seconds();
    age_everything(&base.join("toolchain").join("nightly-2026-11-10"), now - 30 * DAY);
    let drive: FixtureVolume = volume(200 * (1_u64 << 30));
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(base.join("toolchain").join("nightly-2026-11-10").exists(), "under the bound nothing goes");
    assert_eq!(outcome.lines.len(), 0, "a run that removed nothing and kept nothing says nothing: {outcome:?}");
    assert_eq!(log.len(), 0, "and it writes nothing to the log: {log:?}");
}

#[test]
fn the_folder_a_running_process_works_in_stays() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00"]);
    let now: i64 = now_seconds();
    let folder: PathBuf = base.join("toolchain").join("nightly-2026-11-10");
    age_everything(&folder, now - 30 * DAY);
    let running: ChildInFolder = process_in(&folder.join("00"));
    let pid: u32 = running.child.id();
    let drive: FixtureVolume = volume(1);
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(folder.exists(), "a folder a process works in stays: {log:?}");
    let kept: Option<&String> = outcome.lines.iter().find(|line| { return line.contains("kept toolchain folder"); });
    let line: &String = match kept {
        Some(found) => found,
        None => {
            panic!("a kept line for the folder: {:?}", outcome.lines);
        }
    };
    assert!(line.contains(&pid.to_string()), "the line names the process: {line}");
    assert!(line.contains("cwd"), "the line names how the process uses it: {line}");
    assert_eq!(outcome.removed, 0, "{outcome:?}");
    assert_eq!(outcome.kept, 1, "{outcome:?}");
}

#[test]
fn the_folder_a_build_changed_while_the_pass_judged_it_is_kept() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00"]);
    let now: i64 = now_seconds();
    let folder: PathBuf = base.join("toolchain").join("nightly-2026-11-10");
    age_everything(&folder, now - 30 * DAY);
    let judged: Candidate = identity_of_candidate(&folder).expect("the folder as it was judged");
    assert_eq!(changed_since(&judged), None, "a folder nothing touched is the one judged");
    // A build writes a directory entry in the folder, and the folder's time is set to the second after the judged one.
    fs::create_dir_all(folder.join("a-new-directory-entry")).expect("a write in the folder");
    set_time(&folder, judged.modified + 60);
    assert_eq!(changed_since(&judged), Some(String::from("a build wrote in it while the pass judged it")), "a folder whose own time moved is not the one judged any more");
    // A folder swapped for another entry is not the one judged either, whatever its time says.
    let again: Candidate = identity_of_candidate(&folder).expect("the folder as it is now");
    fs::rename(&folder, base.join("moved-away")).expect("the folder away");
    fs::create_dir_all(&folder).expect("another entry at its place");
    assert_eq!(changed_since(&again), Some(String::from("another entry is at its place now")));
}

#[test]
fn a_removal_stopped_between_two_pieces_is_finished_by_the_next_pass_and_never_built_into_between() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01", "02", "03"]);
    let now: i64 = now_seconds();
    let folder: PathBuf = base.join("toolchain").join("nightly-2026-11-10");
    age_everything(&folder, now - 30 * DAY);
    let workspace: PathBuf = root.path().join("some-worktree");
    fs::create_dir_all(&workspace).expect("a workspace the build is for");
    let fresh: PathBuf = crate::lean_keyed_folder::keyed_build_dir(&base, "nightly-2026-11-10", &workspace);

    // The first pass is stopped after one piece, the way a killed one is.
    arm_stop_after_pieces(1);
    let drive: FixtureVolume = volume(1);
    let mut first_log: Vec<String> = Vec::new();
    let first = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut first_log);
    disarm_stop_after_pieces();
    assert!(!folder.exists(), "the folder left the place a build is given, so nothing is built into what is being removed: {first_log:?}");
    let claimed: Vec<PathBuf> = children_of(&retiring_directory(&base));
    assert_eq!(claimed.len(), 1, "the claim waits in the retiring directory: {first_log:?}");
    assert_eq!(claimed[0].as_path(), first.stopped_in.as_deref().expect("the pass says where it stopped"));
    assert_eq!(folder_name_of(&claimed[0]), "nightly-2026-11-10", "the claim still names the toolchain it was made from");
    assert_eq!(pieces_of(&claimed[0]), vec![String::from("01"), String::from("02"), String::from("03")], "one piece of the four went");
    assert!(first_log.iter().any(|line| { return line.contains("1 of 4 piece(s) gone"); }), "{first_log:?}");
    assert!(first_log.iter().any(|line| { return line.contains("the next pass finishes it"); }), "{first_log:?}");

    // A build that meets the removal is given a fresh folder at the place the shadow hands out, never inside the claim.
    crate::lean_keyed_folder::leave_ledger(&fresh, &workspace);
    assert!(fresh.exists(), "the shadow's own folder is made again at its place");
    assert!(fresh.starts_with(base.join("toolchain")), "and not inside what is being removed: {}", fresh.display());
    assert_eq!(pieces_of(&claimed[0]), vec![String::from("01"), String::from("02"), String::from("03")], "the fresh folder is none of the claim's pieces");

    // The next pass finishes the claim whatever the free space is now, and leaves the fresh folder alone for its age.
    let drive_again: FixtureVolume = volume(200 * (1_u64 << 30));
    let mut second_log: Vec<String> = Vec::new();
    let second = retire_toolchain_folders(&base, None, &limits(), &drive_again, &environment(), now_seconds(), false, &deadline(), &mut second_log);
    assert_eq!(children_of(&retiring_directory(&base)).len(), 0, "the next pass finished the claim: {second_log:?}");
    assert!(second_log.iter().any(|line| { return line.contains("finished the removal of toolchain folder nightly-2026-11-10 that an earlier pass started"); }), "{second_log:?}");
    assert!(fresh.exists(), "the fresh folder of the build that met the removal is not part of the claim: {second_log:?}");
    assert_eq!(second.removed, 1, "the claim is the one folder this pass finished: {second:?}");
    assert!(second_log.iter().any(|line| { return line.contains("3 of 3 piece(s) gone"); }), "the three pieces the claim held: {second_log:?}");
}

#[test]
fn a_dry_run_names_the_folders_it_would_remove_and_removes_nothing() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01"]);
    let now: i64 = now_seconds();
    let folder: PathBuf = base.join("toolchain").join("nightly-2026-11-10");
    age_everything(&folder, now - 30 * DAY);
    let drive: FixtureVolume = volume(1);
    let mut log: Vec<String> = Vec::new();
    retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, true, &deadline(), &mut log);
    assert!(folder.exists(), "a dry run removes nothing: {log:?}");
    assert!(log.iter().any(|line| { return line.contains("would remove toolchain folder") && line.contains("2 piece(s)"); }), "{log:?}");
    assert_eq!(pieces_of(&folder), vec![String::from("00"), String::from("01")], "every piece is where it was");
    assert_eq!(children_of(&retiring_directory(&base)).len(), 0, "a dry run claims nothing");
}

#[test]
fn a_slow_drive_stops_the_pass_between_two_pieces_and_the_next_pass_continues() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01", "02", "03"]);
    let now: i64 = now_seconds();
    age_everything(&base.join("toolchain").join("nightly-2026-11-10"), now - 30 * DAY);
    let drive: FixtureVolume = volume(1);
    // The reading before the first piece is quick; the ones after the first and the second piece are slow.
    drive.readings.borrow_mut().extend([Duration::from_millis(10), Duration::from_secs(6), Duration::from_secs(7)]);
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(log.iter().any(|line| { return line.contains("the drive answered over 1000 ms 2 times in a row"); }), "{log:?}");
    let claimed: Vec<PathBuf> = children_of(&retiring_directory(&base));
    assert_eq!(claimed.len(), 1, "the folder is claimed and waits: {log:?}");
    assert_eq!(pieces_of(&claimed[0]), vec![String::from("02"), String::from("03")], "two of the four pieces went");
    assert!(outcome.stopped_in.is_some(), "{outcome:?}");
    assert_eq!(outcome.removed, 0, "a folder that is not whole is not a folder removed: {outcome:?}");
}

#[test]
fn a_volume_that_has_its_space_back_after_one_piece_stops_the_removal_under_the_bound() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-10", &["00", "01", "02"]);
    let now: i64 = now_seconds();
    age_everything(&base.join("toolchain").join("nightly-2026-11-10"), now - 30 * DAY);
    /// A volume that is short of space until one piece has gone.
    struct SpaceBack {
        /// How many readings have been taken.
        readings: Cell<usize>,
    }
    impl Volume for SpaceBack {
        fn free_bytes(&self, _base: &Path) -> io::Result<u64> {
            return if self.readings.get() > 0 { Ok(200 * (1_u64 << 30)) } else { Ok(1) };
        }
        fn reading(&self, _probe: &Path) -> io::Result<Duration> {
            self.readings.set(self.readings.get() + 1);
            return Ok(Duration::from_millis(1));
        }
    }
    let drive: SpaceBack = SpaceBack { readings: Cell::new(0) };
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(outcome.bytes >= 4096, "one piece went: {outcome:?}\n{log:?}");
    assert!(log.iter().any(|line| { return line.contains("1 of 3 piece(s) gone"); }), "{log:?}");
    assert!(log.iter().any(|line| { return line.contains("the next pass finishes it"); }), "the pieces that stay are the next pass's work: {log:?}");
    assert_eq!(outcome.removed, 0, "a folder whose removal stopped under the bound is not a folder removed: {outcome:?}");
}

#[test]
fn the_folders_are_listed_oldest_first_by_their_own_time() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-07", &["00"]);
    make_folder(&base, "nightly-2026-11-10", &["00"]);
    make_folder(&base, "nightly-2026-11-09", &["00"]);
    let now: i64 = now_seconds();
    age_everything(&base.join("toolchain").join("nightly-2026-11-07"), now - 3 * DAY);
    age_everything(&base.join("toolchain").join("nightly-2026-11-10"), now - 30 * DAY);
    age_everything(&base.join("toolchain").join("nightly-2026-11-09"), now - 9 * DAY);
    // A file where a folder would be is not a toolchain's folder.
    fs::write(base.join("toolchain").join("not-a-folder"), b"").expect("a file beside the folders");
    let (listed, notes): (Vec<Candidate>, Vec<String>) = candidates(&base);
    let names: Vec<String> = listed.iter().map(|candidate| { return candidate.name.clone(); }).collect();
    assert_eq!(names, vec![String::from("nightly-2026-11-10"), String::from("nightly-2026-11-09"), String::from("nightly-2026-11-07")], "the folder's own time decides the order");
    assert_eq!(notes.len(), 0, "every folder was readable: {notes:?}");
}

#[test]
fn a_folder_younger_than_the_age_stays_even_when_the_volume_is_short() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    make_folder(&base, "nightly-2026-11-13", &["00"]);
    let now: i64 = now_seconds();
    let folder: PathBuf = base.join("toolchain").join("nightly-2026-11-13");
    // The age of these tests is one day; a build an hour ago is inside it.
    age_everything(&folder, now - 3_600);
    let drive: FixtureVolume = volume(1);
    let mut log: Vec<String> = Vec::new();
    let outcome = retire_toolchain_folders(&base, None, &limits(), &drive, &environment(), now, false, &deadline(), &mut log);
    assert!(folder.exists(), "a folder a build wrote in an hour ago stays: {:?}", outcome.lines);
    assert!(outcome.lines.iter().any(|line| { return line.contains("kept toolchain folder") && line.contains("within the 86400 s age"); }), "{:?}", outcome.lines);
    assert_eq!(outcome.removed, 0);
    assert_eq!(log.len(), 0, "nothing was removed, so the log says nothing: {log:?}");
}
