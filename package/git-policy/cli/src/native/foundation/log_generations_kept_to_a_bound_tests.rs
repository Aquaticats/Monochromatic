//! What:
//!  The history of a log:
//! every rotation keeps its own generation,
//! the oldest go first when the bound is passed,
//! and a killed rotation's file is
//!  given a number a reader finds;
//! the audit file's pinning of a generation that names a saved copy is
//!  controlled in `discard_audit_generation_tests.rs`.
//! Why:
//!  A log a person needs after two rotations must still hold the lines that the third rotation did not replace,
//! and the bound
//!  the owner chose (32 MiB,
//! cited beside `LOG_HISTORY_BYTES`) is what keeps the common Git directory from growing without one.
//! The tests put files in a disposable
//!  directory and never touch a real Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await rotate(auditFile); expect(await generations(dir)).toEqual([1, 2]);
//! ```

use super::{ROTATE_AT_BYTES, RotationStep, append_line, move_aside_between};
use crate::add_timing::ADD_TIMING_FILE;
use crate::command_test_support::{fixture, remove};
use crate::commit_timing::TIMING_FILE;
use crate::log_generations_kept_to_a_bound::{LOG_HISTORY_BYTES, UNPINNED_LOG, a_pass_is_running, claim_a_pass, pass_ended, trim_generations};
use crate::push_trigger_log::TRIGGER_FILE;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A log name that no shipped log uses,
/// so a control never reads a real log's path.
const LOG_FILE: &str = "monochromatic-git-policy-cli-log.jsonl";

/// How many whole generations the bound holds.
fn kept_generations() -> u64 {
    return LOG_HISTORY_BYTES / ROTATE_AT_BYTES;
}

/// A file of exactly `ROTATE_AT_BYTES` bytes whose lines are `lines` and then filler,
/// every line ended,
/// so that the next
/// append sees a full file.
fn full_file(lines: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    let room: usize = ROTATE_AT_BYTES as usize - bytes.len();
    bytes.extend(std::iter::repeat_n(b'x', room - 1));
    bytes.push(b'\n');
    return bytes;
}

/// A line that names its generation and nothing else.
fn marker(name: &str) -> String {
    return format!("{{\"marker\":\"{name}\"}}");
}

/// Writes the file `target.<number>` as a generation made of the lines and filler.
fn write_generation(target: &Path, number: u64, lines: &[String]) {
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".{number}"));
    std::fs::write(PathBuf::from(name), full_file(lines)).expect("a generation");
}

/// Fills the live file so that the next append rotates it,
/// then appends one line,
/// and waits for the pass the rotation schedules,
/// so what the control reads afterwards is the state the shipped code settles at.
fn rotate_once(target: &Path, name: &str) {
    std::fs::write(target, full_file(&[marker(name)])).expect("a full live file");
    append_line(target, b"{\"live\":1}\n", &UNPINNED_LOG).expect("the append that rotates");
    wait_for_the_pass(target);
}

/// Waits,
/// within a deadline,
/// for the pass a rotation scheduled to end.
/// Why:
///   The pass runs on a thread of its own so that a command never waits for it;
/// a control that reads the generations must wait for
///  it instead of racing it,
/// and a pass that never ends fails the control here rather than making its assertion flake.
fn wait_for_the_pass(target: &Path) {
    let deadline: Instant = Instant::now() + Duration::from_secs(120);
    while a_pass_is_running(target) {
        assert!(Instant::now() < deadline, "the pass for {} did not end within its deadline", target.display());
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The numbers of the generations `target.<number>` that exist,
/// ascending.
fn generations(target: &Path) -> Vec<u64> {
    let directory: &Path = target.parent().expect("a directory");
    let prefix: String = format!("{}.", target.file_name().expect("a name").to_string_lossy());
    let mut numbers: Vec<u64> = std::fs::read_dir(directory)
        .expect("directory")
        .filter_map(|entry| {
            let name: String = entry.expect("entry").file_name().to_string_lossy().into_owned();
            return name.strip_prefix(prefix.as_str()).and_then(|rest| { return rest.parse::<u64>().ok(); });
        })
        .collect();
    numbers.sort_unstable();
    return numbers;
}

/// The first line of generation `number`.
fn first_line_of(target: &Path, number: u64) -> String {
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".{number}"));
    let text: String = String::from_utf8_lossy(&std::fs::read(PathBuf::from(name)).expect("a generation")).into_owned();
    return text.lines().next().unwrap_or_default().to_string();
}

/// The lines of every generation of `target`,
/// each file's first line,
/// in number order.
fn first_lines(target: &Path) -> Vec<String> {
    return generations(target).into_iter().map(|number| { return first_line_of(target, number); }).collect();
}

/// The names in the directory that hold a file taken off the log's name and not yet put anywhere.
fn private_names(root: &Path) -> Vec<String> {
    return std::fs::read_dir(root)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.contains(".moving-"); })
        .collect();
}

/// A writer that opens the full file at `target`,
/// ready to rotate it.
fn opened_full_file(target: &Path) -> std::fs::File {
    return std::fs::OpenOptions::new().read(true).append(true).open(target).expect("open");
}

/// Each rotation keeps the file it moved,
/// under a number nobody used,
/// so the second rotation does not replace the first.
#[test]
fn each_rotation_keeps_its_own_generation() {
    let root: PathBuf = fixture("generations-each");
    let target: PathBuf = root.join(LOG_FILE);
    for name in ["first", "second", "third"] {
        rotate_once(target.as_path(), name);
    }
    assert_eq!(generations(target.as_path()), vec![1, 2, 3]);
    assert_eq!(first_line_of(target.as_path(), 1), marker("first"));
    assert_eq!(first_line_of(target.as_path(), 3), marker("third"));
    assert_eq!(std::fs::read(&target).expect("live file"), b"{\"live\":1}\n", "the live file holds only what came after");
    remove(root.as_path());
}

/// The generations may take the bound and no more:
/// the oldest goes first,
/// and the newest stay.
#[test]
fn over_the_bound_the_oldest_generation_goes() {
    let root: PathBuf = fixture("generations-bound");
    let target: PathBuf = root.join(LOG_FILE);
    let extra: u64 = 3;
    for number in 1..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    let expected: Vec<u64> = (extra + 1..=newest).collect();
    assert_eq!(generations(target.as_path()), expected, "the oldest {extra} went and {} stay", kept_generations());
    assert_eq!(first_line_of(target.as_path(), newest), marker("newest"));
    remove(root.as_path());
}

/// A writer killed after it took the file off its name leaves the file at a private name with every line,
/// and readers go by
/// number:
/// the next retention pass gives the file a generation number,
/// so its lines are read,
/// and its bytes count against the
/// bound like any generation's.
#[test]
fn a_file_left_by_a_killed_rotation_becomes_a_generation_that_readers_find() {
    let root: PathBuf = fixture("generations-killed");
    let target: PathBuf = root.join(LOG_FILE);
    let extra: u64 = 2;
    std::fs::write(&target, full_file(&[marker("killed")])).expect("a full live file");
    let seen_full: std::fs::File = opened_full_file(target.as_path());
    // The writer is killed at the instant after the move: a panic stands in for the kill, and nothing after it runs.
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = move_aside_between(target.as_path(), &seen_full, &mut |step| {
            if step == RotationStep::AfterMove {
                panic!("killed after the move");
            }
        });
    }));
    assert!(killed.is_err(), "the stand-in kill happened");
    assert_eq!(private_names(root.as_path()).len(), 1, "the file waits at its private name");
    for number in 1..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    assert_eq!(private_names(root.as_path()), Vec::<String>::new(), "no file stays at a name that no reader looks at");
    let numbered: Vec<u64> = generations(target.as_path());
    let adopted: u64 = *numbered.last().expect("a generation");
    assert_eq!(first_line_of(target.as_path(), adopted), marker("killed"), "the killed writer's lines are in a numbered generation");
    assert_eq!(first_line_of(target.as_path(), adopted - 1), marker("newest"), "the rotation that found it kept its own file first");
    assert!(numbered.len() < usize::try_from(adopted).expect("small"), "its bytes took part of the bound, so older generations went: {numbered:?}");
    remove(root.as_path());
}

/// Another writer's retention pass runs while this writer holds a file at its private name,
/// between its take and its install:
/// the pass gives the file a number,
/// this writer's install finds the name gone,
/// and the file is in one generation,
/// once;
/// no
/// record is lost and none is doubled.
#[test]
fn a_pass_that_finds_a_live_rotation_between_its_take_and_its_install_neither_loses_nor_doubles_the_file() {
    let root: PathBuf = fixture("generations-live");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, full_file(&[marker("mid-flight")])).expect("a full live file");
    let seen_full: std::fs::File = opened_full_file(target.as_path());
    // Whether the install then succeeds or reports that its private name is gone, the file is whole in one generation.
    let _moved = move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::AfterMove {
            trim_generations(target.as_path(), &UNPINNED_LOG).expect("the other writer's pass");
        }
    });
    assert_eq!(private_names(root.as_path()), Vec::<String>::new());
    assert_eq!(first_lines(target.as_path()), vec![marker("mid-flight")], "one generation holds the file");
    remove(root.as_path());
}

/// Two numbers for one file (two passes each gave a private name its number) hold every line twice:
/// the pass keeps the lowest
/// number and removes the other name,
/// which loses nothing.
#[test]
fn two_generation_numbers_for_one_file_are_reduced_to_the_lowest() {
    let root: PathBuf = fixture("generations-twice");
    let target: PathBuf = root.join(LOG_FILE);
    write_generation(target.as_path(), 1, &[marker("once")]);
    std::fs::hard_link(root.join(format!("{LOG_FILE}.1")), root.join(format!("{LOG_FILE}.2"))).expect("a second number");
    rotate_once(target.as_path(), "newest");
    assert_eq!(generations(target.as_path()), vec![1, 3]);
    assert_eq!(first_line_of(target.as_path(), 1), marker("once"));
    remove(root.as_path());
}

/// A writer killed after it gave the file its number but before it removed the private name leaves two names for one file;
/// the next retention pass removes the private name and keeps the numbered one.
#[test]
fn a_second_name_left_by_a_killed_rotation_is_removed_and_the_generation_kept() {
    let root: PathBuf = fixture("generations-two-names");
    let target: PathBuf = root.join(LOG_FILE);
    write_generation(target.as_path(), 1, &[marker("g1")]);
    std::fs::hard_link(root.join(format!("{LOG_FILE}.1")), root.join(format!("{LOG_FILE}.moving-4242-0"))).expect("second name");
    rotate_once(target.as_path(), "newest");
    assert_eq!(private_names(root.as_path()), Vec::<String>::new(), "the extra name is gone");
    assert_eq!(generations(target.as_path()), vec![1, 2]);
    assert_eq!(first_line_of(target.as_path(), 1), marker("g1"), "the generation keeps every line");
    remove(root.as_path());
}

/// Another writer that took the next number between this writer's look and its install does not lose its generation:
/// this writer takes the number after it.
#[test]
fn a_number_taken_by_another_writer_is_never_replaced() {
    let root: PathBuf = fixture("generations-number-race");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, full_file(&[marker("ours")])).expect("a full live file");
    let seen_full: std::fs::File = opened_full_file(target.as_path());
    move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::AfterMove {
            write_generation(target.as_path(), 1, &[marker("theirs")]);
        }
    })
    .expect("moved");
    assert_eq!(generations(target.as_path()), vec![1, 2]);
    assert_eq!(first_line_of(target.as_path(), 1), marker("theirs"));
    assert_eq!(first_line_of(target.as_path(), 2), marker("ours"));
    remove(root.as_path());
}

/// A rotation that cannot finish,
/// here because the directory refuses a new name,
/// never costs the record.
#[test]
fn a_rotation_that_fails_does_not_drop_the_record() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("generations-fails");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, full_file(&[marker("full")])).expect("a full live file");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o500)).expect("a directory that takes no new name");
    let appended = append_line(target.as_path(), b"{\"kept\":1}\n", &UNPINNED_LOG);
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).expect("writable again");
    appended.expect("the record is appended whatever the rotation did");
    let text: Vec<u8> = std::fs::read(&target).expect("target");
    assert!(text.ends_with(b"{\"kept\":1}\n"));
    assert!(text.starts_with(&full_file(&[marker("full")])), "the history is still in the file at the name");
    remove(root.as_path());
}

/// Every log of the wrapper keeps its rotations as numbered generations,
/// not one `.1` that the next rotation replaces:
/// the timing logs and the push trigger log keep the first rotation after the third.
#[test]
fn every_log_keeps_every_rotation_as_a_generation() {
    for file in [TIMING_FILE, ADD_TIMING_FILE, TRIGGER_FILE] {
        let root: PathBuf = fixture(&format!("generations-every-timing-{file}"));
        let target: PathBuf = root.join(file);
        for name in ["first", "second", "third"] {
            rotate_once(target.as_path(), name);
        }
        assert_eq!(generations(target.as_path()), vec![1, 2, 3], "{file} keeps every rotation");
        assert_eq!(first_line_of(target.as_path(), 1), marker("first"), "{file}: the oldest rotation still reads");
        assert_eq!(first_line_of(target.as_path(), 3), marker("third"), "{file}: and the newest");
        remove(root.as_path());
    }
}

/// A timing log is bounded like every other log:
/// its oldest generation goes once the generations pass 32 MiB.
#[test]
fn a_timing_log_keeps_generations_up_to_the_bound() {
    let root: PathBuf = fixture("generations-timing-bound");
    let target: PathBuf = root.join(TIMING_FILE);
    let extra: u64 = 2;
    for number in 1..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>(), "the oldest {extra} went");
    remove(root.as_path());
}

/// A writer whose rotation loses the race to another writer's whole rotation,
/// while several generations already stand beside the log,
/// keeps every generation and gives the other writer's new file back:
/// the check of the file it opened is what saves them.
#[test]
fn a_rotation_that_loses_the_race_with_more_than_one_generation_keeps_them_all() {
    let root: PathBuf = fixture("generations-race-many");
    let target: PathBuf = root.join(LOG_FILE);
    for number in 1..=3 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    std::fs::write(&target, full_file(&[marker("full")])).expect("a full live file");
    let seen_full: std::fs::File = opened_full_file(target.as_path());
    move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::BeforeMove {
            // The other writer's whole rotation, and its own line after it.
            move_aside_between(target.as_path(), &seen_full, &mut |_inner: RotationStep| {}).expect("the other writer's rotation");
            std::fs::write(&target, b"{\"theirs\":1}\n").expect("the other writer's new file");
        }
    })
    .expect("no error");
    assert_eq!(generations(target.as_path()), vec![1, 2, 3, 4], "no generation was replaced or lost");
    assert_eq!(first_line_of(target.as_path(), 1), marker("g1"), "the oldest generation keeps its line");
    assert_eq!(first_line_of(target.as_path(), 4), marker("full"), "the other writer's rotation kept the full file");
    assert_eq!(std::fs::read(&target).expect("live file"), b"{\"theirs\":1}\n", "the new file is back at its name");
    assert_eq!(private_names(root.as_path()), Vec::<String>::new(), "nothing stays at a private name");
    remove(root.as_path());
}

/// Another writer that takes the number this writer was about to give its own file,
/// while three generations already stand beside the log,
/// loses neither file:
/// each gets the next free generation.
#[test]
fn a_number_taken_by_another_writer_beside_three_generations_keeps_both() {
    let root: PathBuf = fixture("generations-number-race-many");
    let target: PathBuf = root.join(LOG_FILE);
    for number in 1..=3 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    std::fs::write(&target, full_file(&[marker("ours")])).expect("a full live file");
    let seen_full: std::fs::File = opened_full_file(target.as_path());
    move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::AfterMove {
            write_generation(target.as_path(), 4, &[marker("theirs")]);
        }
    })
    .expect("moved");
    assert_eq!(generations(target.as_path()), vec![1, 2, 3, 4, 5]);
    assert_eq!(first_line_of(target.as_path(), 4), marker("theirs"), "the other writer keeps the number it took");
    assert_eq!(first_line_of(target.as_path(), 5), marker("ours"), "and this writer the one after");
    remove(root.as_path());
}

/// A directory standing where a generation belongs does not make the numbering run on for ever:
/// the rotation reports it and the line goes to the file at the name,
/// as a rotation that failed always does.
#[test]
fn a_directory_at_a_generation_name_ends_the_numbering() {
    let root: PathBuf = fixture("generations-directory");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::create_dir(root.join(format!("{LOG_FILE}.1"))).expect("a directory where a generation belongs");
    std::fs::write(&target, full_file(&[marker("full")])).expect("a full live file");
    let opened: std::fs::File = opened_full_file(target.as_path());
    let rotated = move_aside_between(target.as_path(), &opened, &mut |_step: RotationStep| {});
    assert!(rotated.is_err(), "the rotation gives up instead of trying every number: {rotated:?}");
    remove(root.as_path());
}

/// One log has one pass at a time in a process:
/// the rotation that claims it starts the thread,
/// and every rotation until the pass ends is told to leave it alone,
/// so a burst of rotations cannot start a thread for each.
#[test]
fn one_log_has_one_pass_at_a_time() {
    let root: PathBuf = fixture("generations-one-pass");
    let target: PathBuf = root.join(LOG_FILE);
    assert!(claim_a_pass(target.as_path()), "the first rotation claims the pass");
    assert!(!claim_a_pass(target.as_path()), "a second rotation leaves the running pass alone");
    pass_ended(target.as_path());
    assert!(claim_a_pass(target.as_path()), "once the pass ends, the next rotation claims another");
    pass_ended(target.as_path());
    remove(root.as_path());
}

/// A rotation over the bound appends its line at once and leaves the pass to a thread:
/// the generations settle to the bound,
/// and the record is in the live file before the pass has done anything.
#[test]
fn a_rotation_over_the_bound_appends_at_once_and_lets_a_pass_settle_the_generations() {
    let root: PathBuf = fixture("generations-scheduled");
    let target: PathBuf = root.join(LOG_FILE);
    let extra: u64 = 3;
    for number in 1..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    std::fs::write(&target, full_file(&[marker("newest")])).expect("a full live file");
    append_line(target.as_path(), b"{\"live\":1}\n", &UNPINNED_LOG).expect("the append that rotates");
    assert_eq!(std::fs::read(&target).expect("live file"), b"{\"live\":1}\n", "the line is in a file of its own at the name, before any pass");
    wait_for_the_pass(target.as_path());
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>(), "and the pass brings the generations back to the bound");
    remove(root.as_path());
}
