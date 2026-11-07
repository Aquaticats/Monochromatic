//! Controls for the fold of an earlier log into the current log, each in its own disposable folder.

use super::*;
use crate::wrapper_names_old_and_new::COMMIT_TIMING_LOG;
use std::path::PathBuf;

/// One empty fixture folder for a control.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-log-fold", name);
}

/// A stop that ends the fold when it has been told of `limit` steps, as a kill at that instant would.
struct StopAfter {
    /// How many steps may pass.
    limit: usize,
    /// How many steps have been told.
    seen: usize,
}

impl FoldSteps for StopAfter {
    fn may_continue(&mut self, _step: &'static str) -> bool {
        self.seen += 1;
        return self.seen <= self.limit;
    }
}

/// A step observer that adds a line to the moved log after the first append, like a program that had the log open when it moved.
struct StragglerWrites {
    /// The moved log.
    waiting: PathBuf,
    /// Whether the line was added.
    wrote: bool,
}

impl FoldSteps for StragglerWrites {
    fn may_continue(&mut self, step: &'static str) -> bool {
        if step == "after-append" && !self.wrote {
            self.wrote = true;
            let mut log = std::fs::OpenOptions::new().append(true).open(self.waiting.as_path()).expect("moved log");
            log.write_all(b"{\"time\":\"2026-11-05T03:59:00.000Z\",\"from\":\"straggler\"}\n").expect("straggler line");
        }
        return true;
    }
}

/// The lines of the earlier log.
const EARLIER_LINES: &str = "{\"time\":\"2026-11-05T03:10:00.000Z\",\"from\":\"earlier\",\"n\":1}\n{\"time\":\"2026-11-05T03:11:00.000Z\",\"from\":\"earlier\",\"n\":2}\n";

/// The lines of the current log, one of them also in the earlier log.
const CURRENT_LINES: &str = "{\"time\":\"2026-11-03T03:10:00.000Z\",\"from\":\"current\",\"n\":1}\n{\"time\":\"2026-11-05T03:10:00.000Z\",\"from\":\"earlier\",\"n\":1}\n";

/// Plant both logs.
fn plant(root: &Path) {
    std::fs::write(root.join(COMMIT_TIMING_LOG.earlier), EARLIER_LINES).expect("earlier log");
    std::fs::write(root.join(COMMIT_TIMING_LOG.current), CURRENT_LINES).expect("current log");
}

/// The text of the current log.
fn current_text(root: &Path) -> String {
    return std::fs::read_to_string(root.join(COMMIT_TIMING_LOG.current)).expect("current log");
}

/// Nothing is left of the fold's own files.
fn assert_clean(root: &Path) {
    for leftover in [COMMIT_TIMING_LOG.earlier.to_owned(), format!("{}{WAITING_SUFFIX}", COMMIT_TIMING_LOG.current), format!("{}{JOURNAL_SUFFIX}", COMMIT_TIMING_LOG.current)] {
        assert!(!root.join(leftover.as_str()).exists(), "{leftover} is gone");
    }
}

/// The earlier log's bytes end the current log after the current log's own, nothing changes a byte,
///  a line both logs hold stays twice, and the earlier log, the moved log and the journal are gone.
#[test]
fn the_earlier_logs_bytes_end_the_current_log_and_nothing_is_left() {
    let root: PathBuf = fixture("appends");
    plant(root.as_path());
    let done: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("fold");
    assert_eq!(done, LogFold::Folded);
    assert_eq!(current_text(root.as_path()), format!("{CURRENT_LINES}{EARLIER_LINES}"));
    assert_clean(root.as_path());
    assert_eq!(fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("again"), LogFold::Nothing, "a second fold has nothing to do");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A last line without a newline gets one, in the current log and in the added bytes, so no two lines run together.
#[test]
fn a_last_line_without_a_newline_is_ended_before_the_next_begins() {
    let root: PathBuf = fixture("no-newline");
    std::fs::write(root.join(COMMIT_TIMING_LOG.earlier), "earlier-1\nearlier-2").expect("earlier log");
    std::fs::write(root.join(COMMIT_TIMING_LOG.current), "current-1").expect("current log");
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("fold");
    assert_eq!(current_text(root.as_path()), "current-1\nearlier-1\nearlier-2\n");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A kill at every step, then one more fold: each line of both logs is in the current log exactly once.
#[test]
fn a_kill_at_any_step_then_a_second_fold_loses_and_repeats_no_line() {
    let expected: String = format!("{CURRENT_LINES}{EARLIER_LINES}");
    let mut limit: usize = 0;
    loop {
        let root: PathBuf = fixture(format!("kill-{limit}").as_str());
        plant(root.as_path());
        let first: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut StopAfter { limit, seen: 0 }).expect("first");
        let finished: bool = first == LogFold::Folded;
        let second: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("second");
        assert!(matches!(second, LogFold::Folded | LogFold::Nothing), "kill after {limit} steps: {second:?}");
        assert_eq!(current_text(root.as_path()), expected, "kill after {limit} steps");
        assert_clean(root.as_path());
        std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
        if finished {
            assert!(limit > 4, "the fold has several steps to stop at: {limit}");
            break;
        }
        limit += 1;
    }
}

/// A program of this release appends to the current log between the kill and the restart: its line stays, and the restart adds
///  the earlier bytes once.
#[test]
fn a_line_appended_between_the_kill_and_the_restart_stays() {
    let root: PathBuf = fixture("append-between");
    plant(root.as_path());
    let stopped: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut StopAfter { limit: 4, seen: 0 }).expect("first");
    assert!(matches!(stopped, LogFold::Stopped(_)), "{stopped:?}");
    let mut log = std::fs::OpenOptions::new().append(true).open(root.join(COMMIT_TIMING_LOG.current)).expect("current log");
    log.write_all(b"{\"from\":\"another program\"}\n").expect("their line");
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("second");
    let text: String = current_text(root.as_path());
    assert_eq!(text.matches("another program").count(), 1, "{text}");
    assert_eq!(text.matches("\"from\":\"earlier\",\"n\":2").count(), 1, "{text}");
    assert_eq!(text.matches("\"from\":\"earlier\",\"n\":1").count(), 2, "one in each log: {text}");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A line that a program with the moved log still open adds after the move is added too.
#[test]
fn a_line_added_to_the_moved_log_after_the_move_is_folded_too() {
    let root: PathBuf = fixture("straggler");
    plant(root.as_path());
    let waiting: PathBuf = root.join(format!("{}{WAITING_SUFFIX}", COMMIT_TIMING_LOG.current));
    let done: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut StragglerWrites { waiting, wrote: false }).expect("fold");
    assert_eq!(done, LogFold::Folded);
    assert!(current_text(root.as_path()).ends_with("\"from\":\"straggler\"}\n"));
    assert_clean(root.as_path());
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// An earlier release's wrapper that opened the earlier log before the fold's rename and writes its line after the fold's last
///  size check, the instant before the fold would remove the moved log.
struct LateWriterWithTheLogOpen {
    /// The descriptor opened by name before the fold started; a rename does not close it.
    log: std::fs::File,
    /// The line written through the descriptor.
    line: &'static [u8],
    /// Whether the line was written.
    wrote: bool,
}

impl FoldSteps for LateWriterWithTheLogOpen {
    fn may_continue(&mut self, step: &'static str) -> bool {
        if step == "before-remove-moved-log" && !self.wrote {
            self.wrote = true;
            self.log.write_all(self.line).expect("late line");
        }
        return true;
    }
}

/// The line the late writer adds.
const LATE_LINE: &[u8] = b"{\"time\":\"2026-11-07T10:00:00.000Z\",\"from\":\"late writer\"}\n";

/// A line that a program writes through a descriptor it opened before the move, after the fold's last size check, ends up in the
///  current log once that program has closed the file: the fold never removes a log a writer still holds open.
#[test]
fn a_line_written_after_the_last_size_check_by_a_writer_that_opened_before_the_move_is_not_lost() {
    let root: PathBuf = fixture("late-writer");
    plant(root.as_path());
    let log = std::fs::OpenOptions::new().append(true).open(root.join(COMMIT_TIMING_LOG.earlier)).expect("earlier log opened by the writer");
    let mut writer: LateWriterWithTheLogOpen = LateWriterWithTheLogOpen { log, line: LATE_LINE, wrote: false };
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut writer).expect("first fold");
    assert!(writer.wrote, "the control reached the step after the last size check");
    drop(writer);
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("fold after the writer closed the file");
    let text: String = current_text(root.as_path());
    let late: &str = std::str::from_utf8(LATE_LINE).expect("text");
    assert_eq!(text.matches(late).count(), 1, "{text}");
    assert!(text.starts_with(CURRENT_LINES), "{text}");
    assert_clean(root.as_path());
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The next append of the current log rotated it after the kill: the added bytes are in the older copy, and the restart does not
///  add them again.
#[test]
fn bytes_already_added_and_rotated_into_an_older_copy_are_not_added_again() {
    let root: PathBuf = fixture("rotated");
    plant(root.as_path());
    let stopped: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut StopAfter { limit: 5, seen: 0 }).expect("first");
    assert!(matches!(stopped, LogFold::Stopped("before-remove-moved-log")), "{stopped:?}");
    let current: PathBuf = root.join(COMMIT_TIMING_LOG.current);
    let rotated: String = format!("{}.1", COMMIT_TIMING_LOG.current);
    std::fs::rename(current.as_path(), root.join(rotated.as_str())).expect("rotation");
    std::fs::write(current.as_path(), "fresh line\n").expect("fresh log");
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("second");
    assert_eq!(current_text(root.as_path()), "fresh line\n");
    assert_eq!(std::fs::read_to_string(root.join(rotated.as_str())).expect("older copy"), format!("{CURRENT_LINES}{EARLIER_LINES}"));
    assert_clean(root.as_path());
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Another process holds the journal's lock: this fold changes nothing.
#[cfg(unix)]
#[test]
fn a_fold_that_another_process_holds_is_left_alone() {
    let root: PathBuf = fixture("busy");
    plant(root.as_path());
    let journal: PathBuf = root.join(format!("{}{JOURNAL_SUFFIX}", COMMIT_TIMING_LOG.current));
    let holder = hold_journal(journal.as_path()).expect("lock").expect("held");
    let result: LogFold = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("fold");
    assert_eq!(result, LogFold::Busy);
    assert_eq!(current_text(root.as_path()), CURRENT_LINES);
    assert_eq!(std::fs::read_to_string(root.join(COMMIT_TIMING_LOG.earlier)).expect("earlier"), EARLIER_LINES);
    drop(holder);
    assert_eq!(fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect("after release"), LogFold::Folded);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A Git directory that cannot be written: the fold fails and the earlier log keeps every byte.
#[cfg(unix)]
#[test]
fn a_read_only_directory_fails_the_fold_and_keeps_the_earlier_log() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("read-only");
    plant(root.as_path());
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only");
    let probe = std::fs::File::create(root.join("probe"));
    let result = fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped);
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(0o700)).expect("writable again");
    if probe.is_ok() {
        // The control: this user can write there anyway (it is not subject to permissions), so the case cannot be made.
        std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
        return;
    }
    assert!(result.is_err(), "{result:?}");
    assert_eq!(std::fs::read_to_string(root.join(COMMIT_TIMING_LOG.earlier)).expect("earlier"), EARLIER_LINES);
    assert_eq!(current_text(root.as_path()), CURRENT_LINES);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A journal cut short records nothing, and text that is not a record is not one.
#[test]
fn only_a_whole_journal_line_records_a_round() {
    assert_eq!(parse_journal(journal_text(Round { offset: 3, from: 0, to: 9 }).as_str()), Some(Round { offset: 3, from: 0, to: 9 }));
    for cut in ["", "3 0", "3 0 9", "3 0 9 1\n", "a b c\n", "3 9 0\n"] {
        assert_eq!(parse_journal(cut), None, "{cut:?}");
    }
}
