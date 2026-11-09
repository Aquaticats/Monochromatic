//! What:
//!  Controls of the bound on the folder of saved copies:
//! the age cut,
//! the size cut,
//! the one-day floor under a burst,
//! the
//!  settings,
//! the audit line,
//! the copy just written,
//! and a writer that saves while a pass runs.
//! Why:
//!  The copies are the only remaining content of replaced files,
//! so every way a pass could remove one that is still wanted
//!  (a young one,
//! the one just written,
//! a directory that is not the wrapper's,
//! a limit read as zero) is checked,
//! each with
//!  a real folder in a fixture and none against this repository's own `.git/monochromatic-git-policy-cli`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(plan([old, young], now, { maxAgeDays: 30, maxBytes: 1 }, undefined)).toEqual([[0, 'age']]);
//! ```

use super::{
    DEFAULT_MAX_AGE_DAYS, DEFAULT_MAX_BYTES, Identity, Limits, Reason, STAMP_RESOLUTION_MILLISECONDS, Stamped, YOUNG_FLOOR_MILLISECONDS, directory_name_of, doctor_section,
    list_stamped, parse_size, parse_stamp, plan, prune, prune_line, report,
};
use crate::command_test_support::{audit_records, fixture, remove};
use crate::conflict_sides::{SAVED_DIRECTORY, save_copies};
use crate::iso_time::format_iso_milliseconds;
use std::path::{Path, PathBuf};

/// A day in milliseconds.
const DAY: i64 = 24 * 60 * 60 * 1000;

/// A time well after 2026-11-02,
/// the clock of the controls.
const NOW: i64 = 1_793_660_000_000;

/// A directory name for a save made `age` before `NOW`,
/// the way the wrapper names it.
fn name_at(age: i64) -> String {
    return format_iso_milliseconds(NOW - age).chars().take(19).map(|each| { return if each == ':' { '-' } else { each }; }).collect();
}

/// A listed directory made `age` before `NOW` holding `bytes`.
fn listed(age: i64, bytes: u128) -> Stamped {
    return Stamped { name: name_at(age), millis: NOW - age, bytes, identity: Identity::default() };
}

/// The limits of the controls:
/// thirty days,
/// and a size each control chooses.
fn limits(max_bytes: u64) -> Limits {
    return Limits { max_age_days: 30, max_bytes };
}

/// Makes the directory of a save made `age` before `NOW`,
/// holding one file of `bytes` bytes,
/// and gives its path.
fn make_save(common: &Path, age: i64, file: &str, bytes: usize) -> PathBuf {
    let directory: PathBuf = common.join(SAVED_DIRECTORY).join(name_at(age));
    std::fs::create_dir_all(&directory).expect("save directory");
    std::fs::write(directory.join(file), vec![b'c'; bytes]).expect("copy");
    return directory;
}

/// The real clock in whole milliseconds,
/// which the concurrent control uses because its saves are made now.
fn now_offset() -> i64 {
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| { return i64::try_from(elapsed.as_millis()).unwrap_or(0); }) - NOW;
}

/// A copy older than the age limit goes,
/// one inside it stays,
/// and the limit is the day count the settings give.
#[test]
fn a_copy_past_the_age_limit_is_chosen_and_a_younger_one_is_not() {
    let entries: Vec<Stamped> = vec![listed(31 * DAY, 1), listed(29 * DAY, 1), listed(2 * DAY, 1)];
    assert_eq!(plan(entries.as_slice(), NOW, limits(u64::MAX), None), vec![(0, Reason::Age)]);
    let tighter: Limits = Limits { max_age_days: 1, max_bytes: u64::MAX };
    assert_eq!(plan(entries.as_slice(), NOW, tighter, None), vec![(0, Reason::Age), (1, Reason::Age), (2, Reason::Age)]);
}

/// Past the size limit the oldest go first,
/// and only as many as bring the folder under it.
#[test]
fn past_the_size_limit_the_oldest_go_first_and_no_more_than_needed() {
    let entries: Vec<Stamped> = vec![listed(5 * DAY, 400), listed(9 * DAY, 400), listed(3 * DAY, 400), listed(2 * DAY, 400)];
    // 1600 bytes against 1000: the two oldest (9 and 5 days) bring it to 800.
    let doomed = plan(entries.as_slice(), NOW, limits(1000), None);
    assert_eq!(doomed, vec![(1, Reason::Size), (0, Reason::Size)]);
    assert!(plan(entries.as_slice(), NOW, limits(1600), None).is_empty(), "at the limit nothing goes");
}

/// A copy past the age limit is counted out of the size before the size cut is made,
/// so it is not removed twice for two
/// reasons and the size cut takes only what the age cut left over.
#[test]
fn the_age_cut_comes_first_and_the_size_cut_counts_what_is_left() {
    let entries: Vec<Stamped> = vec![listed(40 * DAY, 900), listed(5 * DAY, 300), listed(4 * DAY, 300)];
    // The 40-day copy goes by age; the remaining 600 bytes are under 700, so nothing goes by size.
    assert_eq!(plan(entries.as_slice(), NOW, limits(700), None), vec![(0, Reason::Age)]);
}

/// A burst of saves inside one day is out of reach of the size cut however far over the limit it puts the folder:
/// the
/// floor is the protection against deleting what was just made.
#[test]
fn a_burst_of_young_copies_is_never_cut_by_size() {
    let burst: Vec<Stamped> = (0..20).map(|number| { return listed(number * 60_000, 1 << 30); }).collect();
    assert!(plan(burst.as_slice(), NOW, limits(1), None).is_empty());
    let almost: Stamped = listed(YOUNG_FLOOR_MILLISECONDS - 1, 1 << 30);
    // A name says only the second of the save, so a day is counted from the end of that second.
    let exactly: Stamped = listed(YOUNG_FLOOR_MILLISECONDS + STAMP_RESOLUTION_MILLISECONDS, 1 << 30);
    assert!(plan(&[almost], NOW, limits(1), None).is_empty(), "a millisecond short of a day stays");
    assert_eq!(plan(&[exactly], NOW, limits(1), None), vec![(0, Reason::Size)], "a day old is within reach of the size cut");
}

/// The directory just written is never chosen,
/// even when its name says it is old (a save made with an old clock).
#[test]
fn the_copy_just_written_is_never_chosen() {
    let entries: Vec<Stamped> = vec![listed(90 * DAY, 10), listed(80 * DAY, 10)];
    let kept = plan(entries.as_slice(), NOW, limits(1), Some(entries[1].name.as_str()));
    assert_eq!(kept, vec![(0, Reason::Age)]);
}

/// The defaults are the owner's:
/// thirty days and one gibibyte.
#[test]
fn the_defaults_are_thirty_days_and_one_gibibyte() {
    assert_eq!(Limits::default(), Limits { max_age_days: DEFAULT_MAX_AGE_DAYS, max_bytes: DEFAULT_MAX_BYTES });
    assert_eq!((DEFAULT_MAX_AGE_DAYS, DEFAULT_MAX_BYTES), (30, 1 << 30));
}

/// Sizes are read as Git's integer settings read them.
#[test]
fn sizes_take_git_suffixes() {
    assert_eq!(parse_size("512"), Some(512));
    assert_eq!(parse_size("1k"), Some(1024));
    assert_eq!(parse_size("3M"), Some(3 * 1024 * 1024));
    assert_eq!(parse_size("2g"), Some(2 * 1024 * 1024 * 1024));
    for bad in ["", "0", "0k", "-1", "k", "1x", "1.5g", "99999999999999999999g"] {
        assert_eq!(parse_size(bad), None, "{bad:?}");
    }
}

/// Names read back to the second the wrapper wrote them,
/// with the number a second save in one second gets;
/// a name of
/// another shape is nothing.
#[test]
fn a_directory_name_reads_back_to_the_time_it_was_written() {
    let written: i64 = 1_793_628_067_000;
    assert_eq!(parse_stamp("2026-11-02T14-01-07").map(|millis| { return format_iso_milliseconds(millis); }), Some(String::from("2026-11-02T14:01:07.000Z")));
    assert_eq!(parse_stamp(&name_at(NOW - written)), Some(written / 1000 * 1000));
    assert_eq!(parse_stamp("2026-11-02T14-01-07-3"), parse_stamp("2026-11-02T14-01-07"));
    for other in ["", "notes", "2026-11-02", "2026-11-02T14-01-07x", "2026-11-02T14-01-07-", "2026-13-02T14-01-07", "2026-11-02T25-01-07", ".git"] {
        assert_eq!(parse_stamp(other), None, "{other:?}");
    }
}

/// A pass removes what the plan chooses from a real folder,
/// leaves directories that are not the wrapper's untouched,
/// and
/// leaves one audit line for each removed directory with its time,
/// path,
/// bytes and reason.
#[test]
fn a_pass_removes_the_chosen_directories_and_audits_each() {
    let common: PathBuf = fixture("prune-pass");
    let old: PathBuf = make_save(common.as_path(), 40 * DAY, "old.txt", 700);
    let middle: PathBuf = make_save(common.as_path(), 10 * DAY, "middle.txt", 600);
    let young: PathBuf = make_save(common.as_path(), 2 * DAY, "young.txt", 500);
    let foreign: PathBuf = common.join(SAVED_DIRECTORY).join("somebody-elses-notes");
    std::fs::create_dir_all(&foreign).expect("foreign directory");
    std::fs::write(foreign.join("keep.txt"), vec![b'k'; 5000]).expect("foreign file");
    let removed = prune(common.as_path(), NOW, limits(1000), None).removed;
    assert_eq!(removed.iter().map(|each| { return (each.path.clone(), each.bytes, each.reason); }).collect::<Vec<_>>(), vec![(old.clone(), 700, Reason::Age), (middle.clone(), 600, Reason::Size)]);
    assert!(!old.exists() && !middle.exists() && young.exists() && foreign.join("keep.txt").exists());
    let lines = audit_records(common.as_path());
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(lines[0]["outcome"], "pruned");
    assert_eq!(lines[0]["form"], "PruneDiscarded");
    assert_eq!(lines[0]["reason"], "age");
    assert_eq!(lines[0]["path"], old.to_string_lossy().as_ref());
    assert_eq!(lines[0]["bytes"], 700);
    assert_eq!(lines[0]["copies"], serde_json::Value::from(vec!["old.txt"]));
    assert_eq!(lines[0]["limit"], 30);
    assert_eq!(lines[0]["time"], format_iso_milliseconds(NOW));
    assert_eq!(lines[1]["reason"], "size");
    assert_eq!(lines[1]["limit"], 1000);
    assert_eq!(lines[1]["folderBytes"], 1800);
    remove(common.as_path());
}

/// A line names at most a hundred copies and states how many there were.
#[test]
fn a_line_for_a_directory_of_many_copies_lists_a_hundred_and_counts_all() {
    let removed = super::Removed { path: PathBuf::from("/c/x"), bytes: 1, reason: Reason::Age };
    let names: Vec<String> = (0..250).map(|number| { return format!("f{number}.txt"); }).collect();
    let value: serde_json::Value = serde_json::from_str(prune_line(&removed, names.as_slice(), Limits::default(), 1, NOW).trim_end()).expect("JSON");
    assert_eq!(value["copiesTotal"], 250);
    assert_eq!(value["copies"].as_array().map(Vec::len), Some(100));
}

/// A save with an old clock is complete and durable when the pass runs,
/// so it is the one thing the pass cannot remove,
/// while
/// the same directory with no `keep` is within reach of the age cut.
/// The older copies of the folder are the pass's to judge.
#[test]
fn the_copy_just_saved_survives_a_pass_that_would_otherwise_remove_it() {
    let root: PathBuf = fixture("prune-just-saved");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(&common).expect("common directory");
    std::fs::write(root.join("a.rs"), "hand edited\n").expect("worktree file");
    let long_ago: i64 = NOW - 60 * DAY;
    let copies = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], long_ago).expect("saved");
    let saved_root: PathBuf = common.join(SAVED_DIRECTORY);
    let place: &Path = copies[0].saved_to.as_deref().expect("copied");
    let name: String = directory_name_of(saved_root.as_path(), place).expect("under the folder");
    assert!(prune(common.as_path(), NOW, limits(u64::MAX), Some(name.as_str())).removed.is_empty());
    assert_eq!(std::fs::read(place).expect("still there"), b"hand edited\n");
    assert_eq!(prune(common.as_path(), NOW, limits(u64::MAX), None).removed.len(), 1, "without the name it is past the age limit");
    assert!(!place.exists());
    remove(root.as_path());
}

/// How many saves the writer of the control below makes:
/// one for each of the phases a pass can meet it in,
/// and no more.
/// Why:
///   A save is a durable copy: the file is flushed and the two directories that hold it,
/// measured on 2026-11-09 at 30
///  to 82 ms a save against 0.4 ms a pass,
/// so forty saves cost the disk a hundred and twenty flushes and took one suite run
///  to the five-minute limit of the runner (305 s),
/// for work four overlapping saves already demonstrate:
/// the pass ran 4,936 times
///  in the 2.0 s one run's forty saves took,
/// so every save of four is met by hundreds of passes.
const SAVES: usize = 4;

/// A writer that saves copies while a pass runs loses nothing:
/// every save is a young directory,
/// which neither cut reaches,
/// so a pass never meets a half-written copy it could remove.
#[test]
fn a_writer_saving_during_a_pass_loses_no_copy() {
    let root: PathBuf = fixture("prune-concurrent");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(&common).expect("common directory");
    let ancient: PathBuf = make_save(common.as_path(), 90 * DAY - now_offset(), "ancient.txt", 10);
    let content: Vec<u8> = vec![b'w'; 20_000];
    std::fs::write(root.join("a.rs"), &content).expect("worktree file");
    let now: i64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| { return i64::try_from(elapsed.as_millis()).unwrap_or(0); });
    let saves: usize = SAVES;
    let mut places: Vec<PathBuf> = Vec::new();
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| {
            let mut made: Vec<PathBuf> = Vec::new();
            for number in 0..saves {
                let copies = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], now + i64::try_from(number).unwrap_or(0)).expect("saved");
                made.push(copies[0].saved_to.clone().expect("copied"));
            }
            return made;
        });
        // The pass runs over and over until the writer is done, with the tightest limits there are.
        while !writer.is_finished() {
            let _removed = prune(common.as_path(), now, Limits { max_age_days: 1, max_bytes: 1 }, None);
        }
        places = writer.join().expect("the writer finished");
    });
    assert_eq!(places.len(), saves);
    for place in &places {
        assert_eq!(std::fs::read(place).expect("a save survived the passes"), content, "{}", place.display());
    }
    // The passes did run and did remove what is within reach, so the saves above survived them and not an idle pass.
    assert!(!ancient.exists(), "the positive control: the old directory went");
    remove(root.as_path());
}

/// The doctor's block states the size,
/// the count and the oldest save,
/// and is empty when the folder holds nothing.
#[test]
fn the_doctor_reports_size_count_and_oldest() {
    let common: PathBuf = fixture("prune-doctor");
    assert_eq!(doctor_section(common.as_path(), Limits::default()), "");
    assert_eq!(report(common.as_path()), None);
    make_save(common.as_path(), 9 * DAY, "one.txt", 100);
    let newer: PathBuf = make_save(common.as_path(), 3 * DAY, "two.txt", 250);
    std::fs::write(newer.join("three.txt"), vec![b'z'; 50]).expect("second copy");
    let found = report(common.as_path()).expect("a report");
    assert_eq!((found.bytes, found.copies, found.saves, found.oldest.as_str()), (400, 3, 2, name_at(9 * DAY).as_str()));
    let text: String = doctor_section(common.as_path(), Limits { max_age_days: 7, max_bytes: 2048 });
    for part in ["400 bytes", "3 copies", "2 saves", name_at(9 * DAY).as_str(), "older than 7 days", "past 2048 bytes", "discardedCopies.maxAgeDays", "discardedCopies.maxSize"] {
        assert!(text.contains(part), "{part} in {text}");
    }
    assert_eq!(list_stamped(common.as_path()).entries.len(), 2);
    remove(common.as_path());
}
