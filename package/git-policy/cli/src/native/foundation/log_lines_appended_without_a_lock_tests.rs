//! What:
//!  The appender:
//! one line per record,
//! one write of it,
//! the move of a full file to a new generation,
//! and the guards that keep a writer
//!  that lost a race from replacing another writer's history.
//! Why:
//!  Three ways a lock-free appender goes wrong each cost lines a person needs:
//! a second rotation over the
//!  first one's new file,
//! a torn line that swallows the next record,
//! and an error that drops a record.
//! The tests put files in
//!  a disposable directory and never touch a real Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // appendLine(full, 'next\n'); expect(exists(`${log}.1`)).toBe(true);
//! ```

use super::{ROTATE_AT_BYTES, RotationStep, append_entry, append_line, move_aside_between, move_aside_if_still_named};
use crate::command_test_support::{fixture, remove};
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use std::path::{Path, PathBuf};

/// A log name that no shipped log uses,
/// so a control never reads a real log's path.
const LOG_FILE: &str = "monochromatic-git-policy-cli-log.jsonl";

/// A file of exactly `ROTATE_AT_BYTES` bytes,
/// every line ended,
/// so that the next append sees a full file.
fn full_file() -> Vec<u8> {
    let mut filler: Vec<u8> = vec![b'x'; ROTATE_AT_BYTES as usize];
    filler[ROTATE_AT_BYTES as usize - 1] = b'\n';
    return filler;
}

/// The generation numbers `<target>.<number>` that exist,
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

/// The regular files of `root` whose name holds a private rotation name.
fn private_names(root: &Path) -> Vec<String> {
    return std::fs::read_dir(root)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.contains(".moving-"); })
        .collect();
}

/// A file at the size limit moves aside to a generation and a new one starts;
/// below the limit nothing moves.
#[test]
fn a_full_file_moves_aside() {
    let root: PathBuf = fixture("appender-rotate");
    let target: PathBuf = root.join(LOG_FILE);
    let mut almost_full: Vec<u8> = vec![b'x'; ROTATE_AT_BYTES as usize - 1];
    almost_full[ROTATE_AT_BYTES as usize - 2] = b'\n';
    std::fs::write(&target, almost_full).expect("nearly full");
    append_line(target.as_path(), b"next\n", &UNPINNED_LOG).expect("append");
    assert!(!root.join(format!("{LOG_FILE}.1")).exists(), "a file under the limit takes the line alone");
    assert_eq!(std::fs::metadata(&target).expect("metadata").len(), ROTATE_AT_BYTES + 4);
    append_line(target.as_path(), b"after\n", &UNPINNED_LOG).expect("append past the limit");
    assert_eq!(std::fs::read(&target).expect("new file"), b"after\n");
    assert_eq!(std::fs::metadata(root.join(format!("{LOG_FILE}.1"))).expect("aside").len(), ROTATE_AT_BYTES + 4);
    remove(root.as_path());
}

/// A zero size limit would move the log aside before every write,
/// so the limit has to be positive.
#[test]
fn the_size_limit_is_positive() {
    const { assert!(ROTATE_AT_BYTES > 0) };
}

/// A write a full disk or a signal cut short leaves a last line without its newline;
/// the next record starts on a line of its
/// own,
/// so the torn line is lost alone and the record after it still parses.
#[test]
fn a_torn_last_line_does_not_swallow_the_next_record() {
    let root: PathBuf = fixture("appender-torn");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, b"{\"ok\":1}\n{\"torn\":").expect("a torn tail");
    append_line(target.as_path(), b"{\"next\":2}\n", &UNPINNED_LOG).expect("append");
    append_line(target.as_path(), b"{\"last\":3}\n", &UNPINNED_LOG).expect("append");
    let text: String = std::fs::read_to_string(&target).expect("file");
    let parsed: Vec<bool> = text.lines().map(|line| { return serde_json::from_str::<serde_json::Value>(line).is_ok(); }).collect();
    assert_eq!(parsed, vec![true, false, true, true], "{text}");
    assert_eq!(text.lines().last(), Some("{\"last\":3}"), "a whole file adds nothing extra");
    remove(root.as_path());
}

/// A writer that saw a full file but lost the race to another writer's rotation must not take the new,
/// small file in place of
/// the history that writer already kept.
#[test]
fn a_second_rotation_does_not_replace_the_history_with_the_new_file() {
    let root: PathBuf = fixture("appender-rotate-race");
    let target: PathBuf = root.join(LOG_FILE);
    let aside: PathBuf = root.join(format!("{LOG_FILE}.1"));
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    // Another writer rotates first and starts the new file.
    std::fs::rename(&target, &aside).expect("the other writer's rotation");
    std::fs::write(&target, b"{\"new\":1}\n").expect("the new file");
    move_aside_if_still_named(target.as_path(), &seen_full).expect("no error");
    assert_eq!(std::fs::read(&aside).expect("aside"), b"the full history\n", "the history is still there");
    assert_eq!(std::fs::read(&target).expect("target"), b"{\"new\":1}\n", "the new file stays");
    // The same writer, when the path still names its file, does move it.
    let current: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    move_aside_if_still_named(target.as_path(), &current).expect("moved");
    assert!(!target.exists(), "the file at the name was taken off it");
    assert_eq!(std::fs::read(&aside).expect("aside"), b"the full history\n", "and the history is not replaced");
    assert_eq!(generations(target.as_path()), vec![1, 2], "the small file is a generation of its own");
    assert_eq!(std::fs::read(root.join(format!("{LOG_FILE}.2"))).expect("second generation"), b"{\"new\":1}\n");
    remove(root.as_path());
}

/// A writer that saw a full file and passed its check,
/// but whose file another writer rotated before it moved anything,
/// must
/// not take the other writer's new file:
/// the history stays in its generation and the new file stays where it is.
#[test]
fn a_rotation_that_loses_the_race_after_its_check_leaves_the_history_and_the_new_file() {
    let root: PathBuf = fixture("appender-rotate-check-race");
    let target: PathBuf = root.join(LOG_FILE);
    let aside: PathBuf = root.join(format!("{LOG_FILE}.1"));
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    let mut steps: Vec<RotationStep> = Vec::new();
    move_aside_between(target.as_path(), &seen_full, &mut |step| {
        steps.push(step);
        // The other writer rotates and starts the new file after this writer's check and before its move.
        if step == RotationStep::BeforeMove {
            std::fs::rename(&target, &aside).expect("the other writer's rotation");
            std::fs::write(&target, b"{\"new\":1}\n").expect("the new file");
        }
    })
    .expect("no error");
    assert_eq!(steps.first(), Some(&RotationStep::BeforeMove), "the check passed, so the move was reached");
    assert_eq!(std::fs::read(&aside).expect("aside"), b"the full history\n", "the history is still in its generation");
    assert_eq!(std::fs::read(&target).expect("target"), b"{\"new\":1}\n", "the new file is back at its name");
    remove(root.as_path());
}

/// A file another writer rotated away after this writer's check leaves nothing to move,
/// which is not an error:
/// an error
/// would drop the record this writer is about to append.
#[test]
fn a_file_that_vanished_after_the_check_is_not_an_error() {
    let root: PathBuf = fixture("appender-rotate-vanished");
    let target: PathBuf = root.join(LOG_FILE);
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    let outcome: std::io::Result<()> = move_aside_between(target.as_path(), &seen_full, &mut |step| {
        if step == RotationStep::BeforeMove {
            std::fs::rename(&target, &elsewhere).expect("the other writer's rotation");
        }
    });
    assert!(outcome.is_ok(), "{outcome:?}");
    assert_eq!(std::fs::read(&elsewhere).expect("history"), b"the full history\n");
    remove(root.as_path());
}

/// A rotation that cannot finish,
/// here because a directory stands where the generation belongs,
/// never costs the record:
/// it lands in the file.
#[test]
fn a_rotation_that_fails_does_not_drop_the_record() {
    let root: PathBuf = fixture("appender-rotate-fails");
    let target: PathBuf = root.join(LOG_FILE);
    let filler: Vec<u8> = full_file();
    std::fs::write(&target, &filler).expect("a full file");
    std::fs::create_dir(root.join(format!("{LOG_FILE}.1"))).expect("an obstacle where a generation belongs");
    append_line(target.as_path(), b"{\"kept\":1}\n", &UNPINNED_LOG).expect("the record is appended whatever the rotation did");
    let text: Vec<u8> = std::fs::read(&target).expect("target");
    assert!(text.ends_with(b"{\"kept\":1}\n"), "the record is the last line of the file at the name");
    assert!(text.starts_with(&filler), "the full history is still in the file at the name");
    remove(root.as_path());
}

/// A rotation that took a newer file by mistake gives it back even when a writer has started yet another file at the name in the
/// meantime:
/// nothing is replaced,
/// and the newer file stays on disk beside it with every line.
#[test]
fn a_newer_file_taken_by_mistake_is_kept_when_the_name_is_already_used_again() {
    let root: PathBuf = fixture("appender-rotate-double-race");
    let target: PathBuf = root.join(LOG_FILE);
    let aside: PathBuf = root.join(format!("{LOG_FILE}.1"));
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    move_aside_between(target.as_path(), &seen_full, &mut |step| match step {
        RotationStep::BeforeMove => {
            std::fs::rename(&target, &aside).expect("the other writer's rotation");
            std::fs::write(&target, b"{\"second\":1}\n").expect("the newer file");
        }
        RotationStep::AfterMove => {
            std::fs::write(&target, b"{\"third\":1}\n").expect("a third writer starts a file while the name is free");
        }
    })
    .expect("no error");
    assert_eq!(std::fs::read(&aside).expect("aside"), b"the full history\n");
    assert_eq!(std::fs::read(&target).expect("target"), b"{\"third\":1}\n", "the file at the name is not replaced");
    let kept: Vec<Vec<u8>> = std::fs::read_dir(&root)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().contains(".moving-"); }); })
        .map(|path| { return std::fs::read(path).expect("kept file"); })
        .collect();
    assert_eq!(kept, vec![b"{\"second\":1}\n".to_vec()], "the newer file keeps every line");
    remove(root.as_path());
}

/// A rotation that succeeds leaves the history in its generation and no private name behind.
#[test]
fn a_rotation_leaves_only_the_history_in_a_generation() {
    let root: PathBuf = fixture("appender-rotate-clean");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    move_aside_if_still_named(target.as_path(), &seen_full).expect("moved");
    let mut names: Vec<String> = std::fs::read_dir(&root)
        .expect("directory")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    assert_eq!(names, vec![format!("{LOG_FILE}.1")]);
    assert_eq!(std::fs::read(root.join(format!("{LOG_FILE}.1"))).expect("generation"), b"the full history\n");
    remove(root.as_path());
}

/// Writers in several threads each append whole lines:
/// none is lost,
/// torn or joined to another.
#[test]
fn concurrent_appenders_lose_no_line_and_tear_none() {
    let root: PathBuf = fixture("appender-concurrent");
    let target: PathBuf = root.join(LOG_FILE);
    let writers: usize = 6;
    let each: usize = 60;
    std::thread::scope(|scope| {
        for writer in 0..writers {
            let path: &Path = target.as_path();
            scope.spawn(move || {
                for number in 0..each {
                    let line: String = format!("{{\"writer\":{writer},\"number\":{number},\"pad\":\"{}\"}}\n", "p".repeat(200));
                    append_line(path, line.as_bytes(), &UNPINNED_LOG).expect("append");
                }
            });
        }
    });
    let text: String = std::fs::read_to_string(&target).expect("file");
    let mut seen: std::collections::BTreeSet<(u64, u64)> = std::collections::BTreeSet::new();
    // A writer that reads the last byte while another one is half-way through a page-crossing write sees no newline and
    //  starts its line on a fresh one: the result is an empty line between records, which every reader of the file skips.
    for line in text.lines().filter(|line| { return !line.is_empty(); }) {
        let value: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|error| { panic!("a torn line {line:?}: {error}") });
        seen.insert((value["writer"].as_u64().expect("writer"), value["number"].as_u64().expect("number")));
    }
    assert_eq!(seen.len(), writers * each);
    remove(root.as_path());
}

/// A file the user may write but not read still gets its records:
/// the check of the last byte is skipped,
/// not the record.
#[test]
fn a_file_that_cannot_be_read_is_still_appended_to() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("appender-write-only");
    let target: PathBuf = root.join(LOG_FILE);
    std::fs::write(&target, b"{\"first\":1}\n").expect("file");
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o200)).expect("write only");
    append_line(target.as_path(), b"{\"second\":2}\n", &UNPINNED_LOG).expect("appended without read permission");
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).expect("readable again");
    assert_eq!(std::fs::read_to_string(&target).expect("file"), "{\"first\":1}\n{\"second\":2}\n");
    remove(root.as_path());
}

/// A disk that is full is an error the caller can see,
/// in one call,
/// with nothing retried.
#[test]
fn a_full_disk_is_an_error_not_a_second_write() {
    let error = append_line(Path::new("/dev/full"), b"{\"x\":1}\n", &UNPINNED_LOG).expect_err("the device refuses every write");
    assert_eq!(error.raw_os_error(), Some(libc::ENOSPC), "{error:?}");
}

/// An entry is written as the caller composed it:
/// the bytes land whole,
/// nothing is added before them,
/// and the log's folder is created private.
#[test]
fn an_entry_lands_as_composed_in_a_private_folder() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("appender-entry");
    let log: PathBuf = root.join("monochromatic-git-policy-cli").join("entry.md");
    append_entry(log.as_path(), b"\nfirst entry", &UNPINNED_LOG).expect("the first entry");
    append_entry(log.as_path(), b"\nsecond entry", &UNPINNED_LOG).expect("the second entry");
    assert_eq!(std::fs::read_to_string(&log).expect("log"), "\nfirst entry\nsecond entry", "each entry lands as it was composed");
    let folder_mode: u32 = std::fs::metadata(log.parent().expect("folder")).expect("folder metadata").permissions().mode();
    assert_eq!(folder_mode & 0o077, 0, "{folder_mode:o}");
    let file_mode: u32 = std::fs::metadata(&log).expect("log metadata").permissions().mode();
    assert_eq!(file_mode & 0o077, 0, "{file_mode:o}");
    remove(root.as_path());
}

/// A final link where an entry's log belongs is refused:
/// an entry names a commit's subject,
/// and a link could carry it somewhere else.
#[test]
fn an_entrys_log_is_never_a_final_link() {
    let root: PathBuf = fixture("appender-entry-link");
    let elsewhere: PathBuf = root.join("elsewhere.md");
    std::fs::write(&elsewhere, b"the place the link points at").expect("the place");
    let log: PathBuf = root.join("monochromatic-git-policy-cli").join("entry.md");
    std::fs::create_dir_all(log.parent().expect("folder")).expect("the folder");
    std::os::unix::fs::symlink(&elsewhere, &log).expect("a link where the log belongs");
    let written = append_entry(log.as_path(), b"\nentry", &UNPINNED_LOG);
    assert!(written.is_err(), "the link is refused: {written:?}");
    assert_eq!(std::fs::read(&elsewhere).expect("the place"), b"the place the link points at", "nothing was written through it");
    remove(root.as_path());
}

/// The private names a killed rotation leaves and the second names of one generation are settled by the pass beside
/// `log_generations_kept_to_a_bound_tests.rs`;
/// here the appender's own opening modes and its rotation are the subject.
#[test]
fn an_entry_past_the_size_limit_moves_aside() {
    let root: PathBuf = fixture("appender-entry-rotate");
    let log: PathBuf = root.join("monochromatic-git-policy-cli").join("entry.md");
    append_entry(log.as_path(), b"\nfirst entry", &UNPINNED_LOG).expect("the first entry");
    std::fs::write(&log, full_file()).expect("a full log");
    append_entry(log.as_path(), b"\nsecond entry", &UNPINNED_LOG).expect("the second entry");
    assert_eq!(std::fs::read(root.join("monochromatic-git-policy-cli").join("entry.md.1")).expect("generation"), full_file());
    assert_eq!(std::fs::read_to_string(&log).expect("live log"), "\nsecond entry");
    assert_eq!(private_names(root.join("monochromatic-git-policy-cli").as_path()), Vec::<String>::new());
    remove(root.as_path());
}
