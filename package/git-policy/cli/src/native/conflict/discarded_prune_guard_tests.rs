//! What:
//!  Controls of what a pass of the bound on the folder of saved copies may touch and what it must say:
//! a link where the folder
//!  should be,
//! a directory replaced between the listing and the removal,
//! a second pass,
//! names that are no date,
//! the edge of
//!  the one-day floor,
//! sums and differences that overflow,
//! and every failure that used to read as a success.
//! Why:
//!  The copies are the only remaining content of replaced files,
//! so a pass must never reach a file outside the folder or a
//!  directory it did not judge,
//! and must never hide that it kept or half-removed something.
//! Each race is made by a step run at
//!  the moment a pass is most exposed (`prune_with`),
//! never by load,
//! so every control fails or passes the same way each run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pruneWith(common, now, limits, undefined, () => { rename(chosen, aside); mkdir(chosen); });
//! expect(await exists(join(chosen, 'young.txt'))).toBe(true);
//! ```

use super::{Limits, ProblemKind, Reason, Stamped, YOUNG_FLOOR_MILLISECONDS, doctor_section, list_stamped, parse_stamp, plan, prune, prune_with};
use crate::command_test_support::{audit_records, fixture, remove};
use crate::conflict_sides::SAVED_DIRECTORY;
use crate::discard_audit::AUDIT_FILE;
use crate::iso_time::format_iso_milliseconds;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A day in milliseconds.
const DAY: i64 = 24 * 60 * 60 * 1000;

/// A second in milliseconds:
/// the resolution of a directory name.
const SECOND: i64 = 1000;

/// A time well after 2026-11-02,
/// the clock of the controls.
const NOW: i64 = 1_793_660_000_000;

/// A directory name for a save made `age` before `NOW`,
/// the way the wrapper names it.
fn name_at(age: i64) -> String {
    return format_iso_milliseconds(NOW - age).chars().take(19).map(|each| { return if each == ':' { '-' } else { each }; }).collect();
}

/// The limits of the controls:
/// thirty days and no size limit that matters.
fn age_only() -> Limits {
    return Limits { max_age_days: 30, max_bytes: u64::MAX };
}

/// Makes the directory of a save made `age` before `NOW` under `root`,
/// holding one file,
/// and gives its path.
fn save_under(root: &Path, age: i64, file: &str, text: &str) -> PathBuf {
    let directory: PathBuf = root.join(name_at(age));
    std::fs::create_dir_all(&directory).expect("save directory");
    std::fs::write(directory.join(file), text).expect("copy");
    return directory;
}

/// How many passes empty one directory together,
/// and how many copies each of its two levels holds.
const SHARED_PASSES: usize = 4;
const SHARED_COPIES: usize = 400;

/// The folder of a repository's saved copies,
/// made.
fn saved_root_of(common: &Path) -> PathBuf {
    let saved_root: PathBuf = common.join(SAVED_DIRECTORY);
    std::fs::create_dir_all(&saved_root).expect("the folder");
    return saved_root;
}

/// What:
///  The audit file as a writer in the middle of a race can leave it:
/// a whole record,
/// an empty line,
/// a whole record,
/// and a last
///  line that ends without its newline.
/// Reading it gives the two whole records.
/// Why:
///  A writer that looks at the last byte while another writer is half-way through a page-crossing write sees no newline and
///  starts its own record on a fresh line,
/// which leaves an empty line between records (`append_line` states this and says that
///  readers skip empty lines).
/// A record is complete only at its newline,
/// so a last line without one is not a record yet.
/// A
///  reader that parses either fails with "EOF while parsing" on an input that is not an error,
/// which is how
///  `passes_emptying_one_directory_together_audit_every_copy_once` failed once in a full host run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, '{"copiesTotal":1}\n\n{"copiesTotal":2}\n{"copiesTotal":');
/// expect(auditRecords(common).map((record) => record.copiesTotal)).toEqual([1, 2]);
/// ```
#[test]
fn the_audit_reader_skips_an_empty_line_and_a_last_line_without_its_newline() {
    let root: PathBuf = fixture("prune-audit-reader");
    let common: PathBuf = root.join("repo.git");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::write(common.join(AUDIT_FILE), "{\"copiesTotal\":1}\n\n{\"copiesTotal\":2}\n{\"copiesTotal\":").expect("the audit file");
    let totals: Vec<Option<u64>> = audit_records(common.as_path()).iter().map(|record| { return record["copiesTotal"].as_u64(); }).collect();
    assert_eq!(totals, vec![Some(1), Some(2)]);
    remove(root.as_path());
}

/// A folder of saved copies that is a link to a directory elsewhere is not the wrapper's store:
/// what the link reaches is
/// not touched,
/// and the pass says why it did nothing.
#[test]
fn a_folder_that_is_a_link_is_never_followed() {
    let root: PathBuf = fixture("prune-link-folder");
    let common: PathBuf = root.join("repo.git");
    let outside: PathBuf = root.join("elsewhere");
    let victim: PathBuf = save_under(outside.as_path(), 90 * DAY, "precious.txt", "not the wrapper's");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli")).expect("the wrapper's directory");
    std::os::unix::fs::symlink(&outside, common.join(SAVED_DIRECTORY)).expect("the link");
    let outcome = prune(common.as_path(), NOW, age_only(), None);
    assert!(victim.join("precious.txt").exists(), "a directory the link reaches was removed");
    assert!(outcome.removed.is_empty());
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::StoreUnusable]);
    assert!(audit_records(common.as_path()).is_empty(), "nothing was removed, so nothing is audited");
    remove(root.as_path());
}

/// The same holds when the link stands higher up,
/// at the wrapper's own directory.
#[test]
fn a_wrapper_directory_that_is_a_link_is_never_followed() {
    let root: PathBuf = fixture("prune-link-parent");
    let common: PathBuf = root.join("repo.git");
    let outside: PathBuf = root.join("elsewhere");
    let victim: PathBuf = save_under(outside.join("discarded").as_path(), 90 * DAY, "precious.txt", "not the wrapper's");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::os::unix::fs::symlink(&outside, common.join("monochromatic-git-policy-cli")).expect("the link");
    let outcome = prune(common.as_path(), NOW, age_only(), None);
    assert!(victim.join("precious.txt").exists(), "a directory the link reaches was removed");
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::StoreUnusable]);
    remove(root.as_path());
}

/// The folder swapped for a link after the listing and before the removal sends nothing outside:
/// the pass goes on in the
/// folder it listed.
#[test]
fn a_folder_swapped_for_a_link_after_the_listing_reaches_nothing_outside() {
    let root: PathBuf = fixture("prune-link-swap");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let outside: PathBuf = root.join("elsewhere");
    let victim: PathBuf = save_under(outside.as_path(), 90 * DAY, "precious.txt", "not the wrapper's");
    let mut swapped: bool = false;
    let _outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |_candidate: &Path| {
        std::fs::rename(&saved_root, common.join("monochromatic-git-policy-cli/discarded.moved")).expect("move the folder aside");
        std::os::unix::fs::symlink(&outside, &saved_root).expect("the link");
        swapped = true;
    });
    assert!(swapped, "the step ran");
    assert!(victim.join("precious.txt").exists(), "a directory the link reaches was removed");
    remove(root.as_path());
}

/// A directory renamed away and replaced by a new one under the same name between the listing and the removal is not the
/// directory that was judged:
/// the replacement stays,
/// with everything in it,
/// and the pass says it kept it.
#[test]
fn a_replacement_under_the_chosen_name_is_kept() {
    let root: PathBuf = fixture("prune-replaced");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let mut swapped: bool = false;
    let outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |candidate: &Path| {
        assert_eq!(candidate, chosen.as_path());
        std::fs::rename(&chosen, saved_root.join("aside")).expect("the judged directory goes aside");
        std::fs::create_dir(&chosen).expect("a new directory under the name");
        std::fs::write(chosen.join("young.txt"), "written a moment ago").expect("its file");
        swapped = true;
    });
    assert!(swapped, "the step ran");
    assert_eq!(std::fs::read_to_string(chosen.join("young.txt")).expect("the replacement survived"), "written a moment ago");
    assert!(outcome.removed.is_empty(), "{:?}", outcome.removed);
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::EntryChanged]);
    assert!(audit_records(common.as_path()).is_empty());
    remove(root.as_path());
}

/// A chosen directory replaced by a link to a directory elsewhere is neither entered nor removed through the link.
#[test]
fn a_chosen_directory_replaced_by_a_link_reaches_nothing_outside() {
    let root: PathBuf = fixture("prune-entry-link");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let outside: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&outside).expect("elsewhere");
    std::fs::write(outside.join("precious.txt"), "not the wrapper's").expect("a file elsewhere");
    let outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |_candidate: &Path| {
        std::fs::rename(&chosen, saved_root.join("aside")).expect("the judged directory goes aside");
        std::os::unix::fs::symlink(&outside, &chosen).expect("the link");
    });
    assert_eq!(std::fs::read_to_string(outside.join("precious.txt")).expect("untouched"), "not the wrapper's");
    assert!(outcome.removed.is_empty(), "{:?}", outcome.removed);
    remove(root.as_path());
}

/// A subdirectory of a chosen directory replaced by a link,
/// after the listing,
/// is removed as a link:
/// what it points at stays.
#[test]
fn a_subdirectory_replaced_by_a_link_is_removed_as_a_link() {
    let root: PathBuf = fixture("prune-inner-link");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    std::fs::create_dir(chosen.join("inner")).expect("a subdirectory");
    std::fs::write(chosen.join("inner/deep.txt"), "deep").expect("its file");
    let outside: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&outside).expect("elsewhere");
    std::fs::write(outside.join("precious.txt"), "not the wrapper's").expect("a file elsewhere");
    let _outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |_candidate: &Path| {
        std::fs::remove_dir_all(chosen.join("inner")).expect("the subdirectory goes");
        std::os::unix::fs::symlink(&outside, chosen.join("inner")).expect("the link");
    });
    assert_eq!(std::fs::read_to_string(outside.join("precious.txt")).expect("untouched"), "not the wrapper's");
    remove(root.as_path());
}

/// A second pass that starts while one is running does not wait and does not give up:
/// it removes what it judged,
/// the running
/// pass then finds the directory gone and says nothing,
/// and the directory is audited once (NLK:
/// no lock covers the folder).
#[test]
fn a_second_pass_while_one_runs_does_its_own_work_and_neither_loses_a_record() {
    let root: PathBuf = fixture("prune-two-passes");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let mut nested = None;
    let outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |_candidate: &Path| {
        let second = prune(common.as_path(), NOW, age_only(), None);
        assert!(!chosen.exists(), "the second pass left the directory it judged to a pass that holds the folder: {second:?}");
        nested = Some(second);
    });
    let second = nested.expect("the step ran");
    assert_eq!(second.removed.len(), 1, "{second:?}");
    assert!(second.problems.is_empty(), "{second:?}");
    assert!(outcome.removed.is_empty() && outcome.problems.is_empty(), "the first pass found the directory gone and had nothing to say: {outcome:?}");
    assert_eq!(audit_records(common.as_path()).len(), 1, "one directory removed, one line");
    remove(root.as_path());
}

/// A file added to a judged directory after the listing makes it a directory the pass did not judge:
/// it stays,
/// with the new
/// file,
/// and the pass says why.
/// (Its change time moves when a file is added,
/// as when another pass removes files,
/// so the
/// pass compares what the directory holds now with what it held when listed.)
#[test]
fn a_file_added_after_the_listing_keeps_the_directory() {
    let root: PathBuf = fixture("prune-file-added");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let outcome = prune_with(common.as_path(), NOW, age_only(), None, &mut |_candidate: &Path| {
        std::fs::write(chosen.join("added.txt"), "written after the listing").expect("a new file");
    });
    assert_eq!(std::fs::read_to_string(chosen.join("added.txt")).expect("kept"), "written after the listing");
    assert!(chosen.join("old.txt").exists(), "nothing of the directory was removed");
    assert!(outcome.removed.is_empty(), "{:?}", outcome.removed);
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::EntryChanged]);
    assert!(audit_records(common.as_path()).is_empty());
    remove(root.as_path());
}

/// Passes that empty one directory together share its files without a lock:
/// a file is audited by the pass whose removal of it
/// succeeded,
/// so every copy is counted once across the passes and none of them reports a failure.
#[test]
fn passes_emptying_one_directory_together_audit_every_copy_once() {
    let root: PathBuf = fixture("prune-shared-directory");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "first.txt", "first");
    let copies: usize = SHARED_COPIES;
    for number in 0..copies {
        std::fs::write(chosen.join(format!("copy-{number}.txt")), "x").expect("a copy");
    }
    std::fs::create_dir(chosen.join("inner")).expect("a subdirectory");
    for number in 0..copies {
        std::fs::write(chosen.join("inner").join(format!("deep-{number}.txt")), "y").expect("a deeper copy");
    }
    let total: usize = copies * 2 + 1;
    let starting: std::sync::Barrier = std::sync::Barrier::new(SHARED_PASSES);
    let outcomes: Vec<super::PruneOutcome> = std::thread::scope(|scope| {
        let passes: Vec<_> = (0..SHARED_PASSES)
            .map(|_| { return scope.spawn(|| { starting.wait(); return prune(common.as_path(), NOW, age_only(), None); }); })
            .collect();
        return passes.into_iter().map(|pass| { return pass.join().expect("a pass finished"); }).collect();
    });
    for outcome in &outcomes {
        assert!(outcome.problems.is_empty(), "{outcome:?}");
    }
    assert!(!chosen.exists(), "the directory is gone");
    let audited: u64 = audit_records(common.as_path()).iter().map(|line| { return line["copiesTotal"].as_u64().unwrap_or(0); }).sum();
    assert_eq!(audited, u64::try_from(total).expect("a count"), "every copy is audited once across the passes: {outcomes:?}");
    remove(root.as_path());
}

/// Names that no calendar has are not the wrapper's:
/// a date that does not exist is nothing,
/// however old it would be.
#[test]
fn a_name_that_is_no_calendar_date_is_not_a_stamp() {
    for impossible in [
        "2026-02-31T00-00-00",
        "2026-02-29T00-00-00",
        "2100-02-29T00-00-00",
        "2026-04-31T00-00-00",
        "2026-06-31T00-00-00",
        "2026-09-31T00-00-00",
        "2026-11-31T00-00-00",
        "2026-01-32T00-00-00",
        "2026-00-10T00-00-00",
        "2026-11-00T00-00-00",
        "2026-11-02T14-01-60",
        "2026-11-02T24-00-00",
        "2026-11-02T14-60-00",
    ] {
        assert_eq!(parse_stamp(impossible), None, "{impossible:?}");
    }
    for possible in ["2024-02-29T00-00-00", "2000-02-29T00-00-00", "2026-12-31T23-59-59", "2026-04-30T23-59-59-12"] {
        assert!(parse_stamp(possible).is_some(), "{possible:?}");
    }
}

/// A save made at the last millisecond of its named second is a whole second younger than the name says:
/// the floor and the
/// age limit count from the end of that second.
#[test]
fn the_floor_and_the_age_limit_count_from_the_end_of_the_named_second() {
    let stamped: i64 = parse_stamp("2026-11-02T14-01-07").expect("a stamp");
    let entry: Stamped = Stamped { name: String::from("2026-11-02T14-01-07"), millis: stamped, bytes: 1 << 30, identity: super::Identity::default() };
    let tight: Limits = Limits { max_age_days: 1, max_bytes: 1 };
    let at = |now: i64| -> Vec<(usize, Reason)> { return plan(std::slice::from_ref(&entry), now, tight, None); };
    assert!(at(stamped + YOUNG_FLOOR_MILLISECONDS).is_empty(), "the save may be 999 milliseconds short of a day");
    assert!(at(stamped + YOUNG_FLOOR_MILLISECONDS + SECOND - 1).is_empty(), "a millisecond short of a day from the end of the second");
    assert_eq!(at(stamped + YOUNG_FLOOR_MILLISECONDS + SECOND), vec![(0, Reason::Size)], "a whole day from the end of the second");
    let loose: Limits = Limits { max_age_days: 30, max_bytes: u64::MAX };
    let thirty: i64 = 30 * DAY;
    assert!(plan(std::slice::from_ref(&entry), stamped + thirty + SECOND, loose, None).is_empty(), "exactly the age limit from the end of the second");
    assert_eq!(plan(std::slice::from_ref(&entry), stamped + thirty + SECOND + 1, loose, None), vec![(0, Reason::Age)]);
}

/// A clock that cannot be subtracted from an entry's time without overflow chooses nothing and does not panic.
#[test]
fn an_age_that_overflows_chooses_nothing() {
    let entry: Stamped = Stamped { name: name_at(0), millis: NOW, bytes: 1, identity: super::Identity::default() };
    let limits: Limits = Limits { max_age_days: u64::MAX, max_bytes: 1 };
    assert!(plan(std::slice::from_ref(&entry), i64::MIN, limits, None).is_empty());
    assert!(plan(std::slice::from_ref(&entry), i64::MAX, limits, None).is_empty(), "an age limit past any clock removes nothing by age, and one entry is not over a size by itself");
    let old: Stamped = Stamped { name: name_at(0), millis: i64::MIN, bytes: 1, identity: super::Identity::default() };
    assert_eq!(plan(std::slice::from_ref(&old), i64::MAX, Limits { max_age_days: 1, max_bytes: u64::MAX }, None), vec![(0, Reason::Age)], "a very old entry is old");
}

/// Sizes that together pass what a 64-bit count holds still add up,
/// so the size cut sees the folder as it is.
#[test]
fn sizes_that_pass_a_sixty_four_bit_count_still_add_up() {
    let huge: u128 = u128::from(u64::MAX);
    let entries: Vec<Stamped> = vec![
        Stamped { name: name_at(5 * DAY), millis: NOW - 5 * DAY, bytes: huge, identity: super::Identity::default() },
        Stamped { name: name_at(4 * DAY), millis: NOW - 4 * DAY, bytes: huge, identity: super::Identity::default() },
        Stamped { name: name_at(3 * DAY), millis: NOW - 3 * DAY, bytes: huge, identity: super::Identity::default() },
    ];
    let limits: Limits = Limits { max_age_days: 30, max_bytes: u64::MAX };
    assert_eq!(plan(entries.as_slice(), NOW, limits, None), vec![(0, Reason::Size), (1, Reason::Size)], "three counts of the limit leave one");
}

/// A pass cannot append to the audit file,
/// so it removes nothing:
/// a directory never goes without its line.
#[test]
fn an_audit_file_that_cannot_be_appended_stops_the_pass_before_any_removal() {
    let root: PathBuf = fixture("prune-no-audit");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    std::fs::create_dir(common.join(AUDIT_FILE)).expect("a directory where the audit file belongs");
    let outcome = prune(common.as_path(), NOW, age_only(), None);
    assert!(chosen.join("old.txt").exists(), "a directory went without its audit line");
    assert!(outcome.removed.is_empty());
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::AuditUnwritable]);
    remove(root.as_path());
}

/// A directory that cannot be removed in full is said to be half-removed,
/// and the audit line names what did go.
#[test]
fn a_removal_that_fails_part_way_is_reported_and_audited_as_partial() {
    let root: PathBuf = fixture("prune-partial");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "first.txt", "one");
    std::fs::write(chosen.join("second.txt"), "two").expect("a second copy");
    std::fs::create_dir(chosen.join("locked")).expect("a subdirectory");
    std::fs::write(chosen.join("locked/inside.txt"), "three").expect("its file");
    std::fs::set_permissions(chosen.join("locked"), std::fs::Permissions::from_mode(0o555)).expect("read-only directory");
    let before: Vec<&str> = vec!["first.txt", "locked/inside.txt", "second.txt"];
    let outcome = prune(common.as_path(), NOW, age_only(), None);
    let remaining: Vec<&str> = before.iter().copied().filter(|name| { return chosen.join(name).exists(); }).collect();
    std::fs::set_permissions(chosen.join("locked"), std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(outcome.removed.is_empty(), "a half-removed directory is not a removed one: {:?}", outcome.removed);
    assert_eq!(outcome.problems.iter().map(|problem| { return problem.kind; }).collect::<Vec<_>>(), vec![ProblemKind::PartiallyRemoved], "{outcome:?}");
    let lines = audit_records(common.as_path());
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["outcome"], "partial");
    let mut gone: Vec<String> = before.iter().filter(|name| { return !remaining.contains(name); }).map(|name| { return (*name).to_string(); }).collect();
    gone.sort();
    let listed: Vec<String> = lines[0]["copies"].as_array().expect("copies").iter().map(|each| { return each.as_str().expect("text").to_string(); }).collect();
    assert_eq!(listed, gone, "the line names the files that went and no other");
    remove(root.as_path());
}

/// A directory that cannot be read has an unknown size:
/// it is kept,
/// not counted as empty,
/// and the listing says so.
#[test]
fn an_unreadable_directory_is_kept_and_reported_not_counted_as_empty() {
    let root: PathBuf = fixture("prune-unreadable");
    let common: PathBuf = root.join("repo.git");
    let saved_root: PathBuf = saved_root_of(common.as_path());
    let chosen: PathBuf = save_under(saved_root.as_path(), 90 * DAY, "old.txt", "old");
    let readable: PathBuf = save_under(saved_root.as_path(), 80 * DAY, "fine.txt", "fine");
    std::fs::create_dir(chosen.join("closed")).expect("a subdirectory");
    std::fs::write(chosen.join("closed/inside.txt"), "hidden").expect("its file");
    std::fs::set_permissions(chosen.join("closed"), std::fs::Permissions::from_mode(0o000)).expect("closed directory");
    let listing = list_stamped(common.as_path());
    let outcome = prune(common.as_path(), NOW, age_only(), None);
    std::fs::set_permissions(chosen.join("closed"), std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert_eq!(listing.entries.iter().map(|entry| { return entry.name.clone(); }).collect::<Vec<_>>(), vec![readable.file_name().expect("name").to_string_lossy().into_owned()]);
    assert_eq!(listing.problems.iter().map(|problem| { return (problem.kind, problem.path.clone()); }).collect::<Vec<_>>(), vec![(ProblemKind::EntryUnreadable, chosen.clone())]);
    assert!(chosen.join("closed/inside.txt").exists(), "an entry that could not be read was removed");
    assert!(outcome.problems.iter().any(|problem| { return problem.kind == ProblemKind::EntryUnreadable; }), "{outcome:?}");
    remove(root.as_path());
}

/// The doctor says when the folder cannot be used,
/// naming it,
/// instead of reporting an empty folder.
#[test]
fn the_doctor_says_when_the_folder_is_a_link() {
    let root: PathBuf = fixture("prune-doctor-link");
    let common: PathBuf = root.join("repo.git");
    let outside: PathBuf = root.join("elsewhere");
    save_under(outside.as_path(), 9 * DAY, "one.txt", "one");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli")).expect("the wrapper's directory");
    std::os::unix::fs::symlink(&outside, common.join(SAVED_DIRECTORY)).expect("the link");
    let text: String = doctor_section(common.as_path(), Limits::default());
    assert!(text.contains(&common.join(SAVED_DIRECTORY).display().to_string()), "{text}");
    assert!(text.contains("symbolic link"), "{text}");
    remove(root.as_path());
}
