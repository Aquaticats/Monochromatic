//! What:
//!  The history of the audit file:
//! every rotation keeps its own generation,
//! generations that name a saved copy still on disk
//!  are never removed,
//! and the rest are kept up to a size;
//! the timing logs keep one generation.
//! Why:
//!  The audit file attributes a loss of uncommitted work,
//! and each `saved-conflict` line names the saved copies it made.
//! With
//!  one generation the second rotation dropped the lines that named copies still on disk (the delegated pick 11,
//! option C,
//!  in `doc/planning/cli-git-shipped-limits-audit.md`).
//! The tests put files in a disposable directory and never touch a real
//!  Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await rotate(auditFile); expect(await generations(dir)).toEqual([1, 2]);
//! ```

/// The writer,
/// the file names,
/// the rotation steps and the bound under test.
use super::{
    AUDIT_FILE, AUDIT_HISTORY_BYTES, AuditOutcome, AuditRecord, DecidedForm, ROTATE_AT_BYTES, RotationStep, append_line, audit_line,
    move_aside_between,
};
use crate::command_test_support::{fixture, remove};
use crate::commit_timing::TIMING_FILE;
use crate::rule_linked_worktree::GuardedCommand;
use std::path::{Path, PathBuf};

/// How many whole generations the bound holds.
fn kept_generations() -> u64 {
    return AUDIT_HISTORY_BYTES / ROTATE_AT_BYTES;
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

/// The real audit line of a `saved-conflict` decision that saved `copies` (each `path => place`).
fn saved_conflict_line(root: &Path, copies: &[String]) -> String {
    let place: (PathBuf, &'static str) = (root.join(".git"), "main");
    let arguments = crate::command_test_support::os_arguments(&["checkout", "--theirs", "--", "a.rs"]);
    let line: String = audit_line(&AuditRecord {
        time_millis: 1_700_000_000_000,
        process: 1,
        parent: 1,
        form: DecidedForm::Guarded(GuardedCommand::CheckoutPaths),
        outcome: AuditOutcome::SavedConflict,
        place: &place,
        git_dir: place.0.as_path(),
        directory: root,
        arguments: arguments.as_slice(),
        dirty: None,
        copies,
        facts: &[],
    });
    return line.trim_end().to_string();
}

/// A saved copy that exists on disk,
/// and the `path => place` text an audit line names it by.
fn existing_copy(root: &Path, name: &str) -> String {
    let place: PathBuf = root.join(name);
    std::fs::write(&place, b"the only content of a replaced file\n").expect("a saved copy");
    return format!("a.rs => {}", place.display());
}

/// Fills the live file so that the next append rotates it,
/// then appends one line.
fn rotate_once(target: &Path, name: &str) {
    std::fs::write(target, full_file(&[marker(name)])).expect("a full live file");
    append_line(target, b"{\"live\":1}\n").expect("the append that rotates");
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

/// The names in the directory that hold a file taken off the audit name and not yet put anywhere.
fn private_names(root: &Path) -> Vec<String> {
    return std::fs::read_dir(root)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.contains(".moving-"); })
        .collect();
}

/// Each rotation keeps the file it moved,
/// under a number nobody used,
/// so the second rotation does not replace the first.
#[test]
fn each_rotation_of_the_audit_file_keeps_its_own_generation() {
    let root: PathBuf = fixture("generations-each");
    let target: PathBuf = root.join(AUDIT_FILE);
    for name in ["first", "second", "third"] {
        rotate_once(target.as_path(), name);
    }
    assert_eq!(generations(target.as_path()), vec![1, 2, 3]);
    assert_eq!(first_line_of(target.as_path(), 1), marker("first"));
    assert_eq!(first_line_of(target.as_path(), 3), marker("third"));
    assert_eq!(std::fs::read(&target).expect("live file"), b"{\"live\":1}\n", "the live file holds only what came after");
    remove(root.as_path());
}

/// Over the bound,
/// the oldest generation that names no saved copy goes first,
/// and the newest stay.
#[test]
fn over_the_bound_the_oldest_generation_that_names_no_copy_goes() {
    let root: PathBuf = fixture("generations-bound");
    let target: PathBuf = root.join(AUDIT_FILE);
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

/// A generation that names a saved copy still on disk stays when the bound is hit,
/// and the next oldest goes in its place.
#[test]
fn a_generation_that_names_a_copy_on_disk_is_never_removed() {
    let root: PathBuf = fixture("generations-pinned");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let copy: String = existing_copy(root.as_path(), "copy-a");
    // The copy is named on the last line of an otherwise full generation, so only a read to its end finds it.
    write_generation(target.as_path(), 1, &[marker("g1"), saved_conflict_line(root.as_path(), &[copy])]);
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    let mut expected: Vec<u64> = vec![1];
    expected.extend(extra + 1..=newest);
    assert_eq!(generations(target.as_path()), expected, "generation 1 stays; the bound is spent on the others");
    remove(root.as_path());
}

/// A generation whose copy is gone no longer explains anything on disk,
/// so the bound removes it like any other.
#[test]
fn a_generation_that_names_a_deleted_copy_goes_like_any_other() {
    let root: PathBuf = fixture("generations-deleted");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let copy: String = existing_copy(root.as_path(), "copy-b");
    write_generation(target.as_path(), 1, &[marker("g1"), saved_conflict_line(root.as_path(), &[copy])]);
    std::fs::remove_file(root.join("copy-b")).expect("the copy is deleted");
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>());
    remove(root.as_path());
}

/// When every older generation names a copy on disk,
/// the bound removes none of them,
/// and the newest history that names
/// nothing is not squeezed out by them.
#[test]
fn when_only_pinned_generations_exceed_the_bound_none_is_removed() {
    let root: PathBuf = fixture("generations-all-pinned");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    for number in 1..=kept_generations() + extra - 1 {
        let copy: String = existing_copy(root.as_path(), &format!("copy-{number}"));
        write_generation(target.as_path(), number, &[marker(&format!("g{number}")), saved_conflict_line(root.as_path(), &[copy])]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (1..=newest).collect::<Vec<u64>>(), "all stay, over the bound");
    remove(root.as_path());
}

/// Only a `saved-conflict` line that names a place on disk pins a generation:
/// a copy list with nothing to copy,
/// and a pruned
/// directory's line (which records a removal),
/// explain nothing that still exists.
#[test]
fn lines_that_name_nothing_on_disk_do_not_pin_a_generation() {
    let root: PathBuf = fixture("generations-unpinned-lines");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let pruned: String = format!(
        "{{\"schemaVersion\":1,\"form\":\"PruneDiscarded\",\"outcome\":\"pruned\",\"path\":{},\"copies\":[\"a.rs\"]}}",
        crate::json_record::quote(&root.to_string_lossy())
    );
    write_generation(
        target.as_path(),
        1,
        &[marker("g1"), saved_conflict_line(root.as_path(), &[String::from("a.rs => none")]), pruned],
    );
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>());
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
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 2;
    std::fs::write(&target, full_file(&[marker("killed")])).expect("a full live file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
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

/// The lines of every generation of `target`,
/// each file's first line,
/// in number order.
fn first_lines(target: &Path) -> Vec<String> {
    return generations(target).into_iter().map(|number| { return first_line_of(target, number); }).collect();
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
    let target: PathBuf = root.join(AUDIT_FILE);
    std::fs::write(&target, full_file(&[marker("mid-flight")])).expect("a full live file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    // Whether the install then succeeds or reports that its private name is gone, the file is whole in one generation.
    let _moved = move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::AfterMove {
            super::trim_generations(target.as_path(), AUDIT_HISTORY_BYTES).expect("the other writer's pass");
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
    let target: PathBuf = root.join(AUDIT_FILE);
    write_generation(target.as_path(), 1, &[marker("once")]);
    std::fs::hard_link(root.join(format!("{AUDIT_FILE}.1")), root.join(format!("{AUDIT_FILE}.2"))).expect("a second number");
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
    let target: PathBuf = root.join(AUDIT_FILE);
    write_generation(target.as_path(), 1, &[marker("g1")]);
    std::fs::hard_link(root.join(format!("{AUDIT_FILE}.1")), root.join(format!("{AUDIT_FILE}.moving-4242-0"))).expect("second name");
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
    let target: PathBuf = root.join(AUDIT_FILE);
    std::fs::write(&target, full_file(&[marker("ours")])).expect("a full live file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
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
fn an_audit_rotation_that_fails_does_not_drop_the_record() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("generations-fails");
    let target: PathBuf = root.join(AUDIT_FILE);
    std::fs::write(&target, full_file(&[marker("full")])).expect("a full live file");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o500)).expect("a directory that takes no new name");
    let appended = append_line(target.as_path(), b"{\"kept\":1}\n");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).expect("writable again");
    appended.expect("the record is appended whatever the rotation did");
    let text: Vec<u8> = std::fs::read(&target).expect("target");
    assert!(text.ends_with(b"{\"kept\":1}\n"));
    assert!(text.starts_with(&full_file(&[marker("full")])), "the history is still in the file at the name");
    remove(root.as_path());
}

/// The timing logs keep one generation:
/// a third rotation replaces the file that the second left in `.1`,
/// and nothing numbered
/// beyond it is made,
/// however many audit generations stay.
#[test]
fn a_timing_log_keeps_one_generation() {
    for file in [TIMING_FILE, crate::add_timing::ADD_TIMING_FILE, crate::push_trigger_log::TRIGGER_FILE] {
        let root: PathBuf = fixture(&format!("generations-timing-{file}"));
        let target: PathBuf = root.join(file);
        for name in ["first", "second", "third"] {
            rotate_once(target.as_path(), name);
        }
        assert_eq!(generations(target.as_path()), vec![1], "{file} keeps one generation");
        assert_eq!(first_line_of(target.as_path(), 1), marker("third"), "{file}: the newest rotation is the one kept");
        remove(root.as_path());
    }
}

/// Every log of the wrapper keeps its rotations as numbered generations,
/// not one `.1` that the next rotation replaces:
/// a timing log's third rotation still reads its first.
#[test]
fn a_timing_log_keeps_every_rotation_as_a_generation() {
    for file in [TIMING_FILE, crate::add_timing::ADD_TIMING_FILE, crate::push_trigger_log::TRIGGER_FILE] {
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

/// A writer whose rotation loses the race to another writer's whole rotation,
/// while several generations already stand beside the log,
/// keeps every generation and gives the other writer's new file back:
/// the check of the file it opened is what saves them.
#[test]
fn a_rotation_that_loses_the_race_with_more_than_one_generation_keeps_them_all() {
    for file in [AUDIT_FILE, TIMING_FILE] {
        let root: PathBuf = fixture(&format!("generations-race-many-{file}"));
        let target: PathBuf = root.join(file);
        for number in 1..=3 {
            write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
        }
        std::fs::write(&target, full_file(&[marker("full")])).expect("a full live file");
        let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
        move_aside_between(target.as_path(), &seen_full, &mut |step| {
            if step == RotationStep::BeforeMove {
                // The other writer's whole rotation, and its own line after it.
                move_aside_between(target.as_path(), &seen_full, &mut |_inner: RotationStep| {}).expect("the other writer's rotation");
                std::fs::write(&target, b"{\"theirs\":1}\n").expect("the other writer's new file");
            }
        })
        .expect("no error");
        assert_eq!(generations(target.as_path()), vec![1, 2, 3, 4], "{file}: no generation was replaced or lost");
        assert_eq!(first_line_of(target.as_path(), 1), marker("g1"), "{file}: the oldest generation keeps its line");
        assert_eq!(first_line_of(target.as_path(), 4), marker("full"), "{file}: the other writer's rotation kept the full file");
        assert_eq!(std::fs::read(&target).expect("live file"), b"{\"theirs\":1}\n", "{file}: the new file is back at its name");
        assert_eq!(private_names(root.as_path()), Vec::<String>::new(), "{file}: nothing stays at a private name");
        remove(root.as_path());
    }
}

/// Another writer that takes the number this writer was about to give its own file,
/// while three generations already stand beside the log,
/// loses neither file: each gets the next free generation.
#[test]
fn a_number_taken_by_another_writer_beside_three_generations_keeps_both() {
    let root: PathBuf = fixture("generations-number-race-many");
    let target: PathBuf = root.join(AUDIT_FILE);
    for number in 1..=3 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    std::fs::write(&target, full_file(&[marker("ours")])).expect("a full live file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
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

/// A `saved-conflict` line that cannot be parsed (a write cut short) or that names a place the writer could not show as text
/// might name a copy that exists,
/// so its generation stays.
#[test]
fn a_line_that_may_name_a_copy_keeps_its_generation() {
    let root: PathBuf = fixture("generations-doubtful");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let torn: String = String::from("{\"schemaVersion\":1,\"outcome\":\"saved-conflict\",\"copies\":[\"a.rs => /where/it/we");
    let unshowable: String = saved_conflict_line(root.as_path(), &[String::from("a.rs => /bytes/that/are/\u{FFFD}/in/the/path")]);
    write_generation(target.as_path(), 1, &[marker("g1"), torn]);
    write_generation(target.as_path(), 2, &[marker("g2"), unshowable]);
    for number in 3..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    let numbered: Vec<u64> = generations(target.as_path());
    assert!(numbered.starts_with(&[1, 2]), "both doubtful generations stay: {numbered:?}");
    assert_eq!(numbered.last(), Some(&newest));
    remove(root.as_path());
}
