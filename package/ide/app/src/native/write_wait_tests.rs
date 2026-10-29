//! A save in progress is not shown:
//!  an in-place save and a delete-then-rewrite reach the window only
//! once finished,
//!  as long as the writer pauses for less than the write-quiet period.
//! An ignored measurement reports which pause lengths let a truncated file through.

/// Shipped window setup and the bounded wait shared with the watch tests.
use super::watch_tests::open;
/// The production window and the bounded wait for startup state.
use super::{AppWindow, render};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Reading positions,
///  and the shipped write-wait intervals named in the measurement output.
use ide_app::{
    document::ReadingPosition,
    refresh_policy::{REREAD_GAP, WRITE_QUIET, WRITE_WAIT_LIMIT},
};
/// Real headless timers drive the same refresh timers as the shipped event loop;
/// `ComponentHandle` provides `hide` on the generated window.
use slint::{ComponentHandle, platform::update_timers_and_animations};
/// What:
///  `OpenOptions` opens a file with chosen flags;
///  `Write` provides `write_all`;
///  `Path` borrows a path.
/// Why:
///  The test plays a slow writer:
///  truncate,
///  wait,
///  write,
///  close,
///  with the window running in between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = openSync(path, 'w'); /* wait */ writeSync(handle, text); closeSync(handle);
/// ```
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

/// Text on disk before each save,
///  with `am a` selected.
pub(super) const OLD: &str = "I am a big cat\n";

/// Text the save writes;
///  the selection corresponds to `was a`.
pub(super) const NEW: &str = "I was a big cat, but now I am a human!\n";

/// How long the writer leaves the file unfinished in the asserting tests:
///  below the 50 ms quiet period,
/// and long enough for two 20 ms native ticks,
///  so a read that did not wait happens during the pause.
/// Timers run only inside the pause,
///  so a stalled test thread cannot stretch the wait they observe.
pub(super) const PAUSE: Duration = Duration::from_millis(40);

/// Record the window's source text when it differs from every text recorded so far.
fn note(window: &AppWindow, seen: &mut Vec<String>) {
    // What: `to_string` copies the toolkit string into an owned Rust `String`.
    // Why: The record outlives the next model update.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = window.sourceText; if (!seen.includes(text)) seen.push(text);
    // ```
    let text = window.get_source_text().to_string();
    if !seen.contains(&text) {
        seen.push(text);
    }
}

/// Run native timers for `span`,
///  recording every distinct source text the window shows.
pub(super) fn watch_texts(window: &AppWindow, span: Duration, seen: &mut Vec<String>) {
    let start = Instant::now();
    // The time check comes first, so a stalled thread leaves the loop without running timers late.
    while start.elapsed() < span {
        update_timers_and_animations();
        note(window, seen);
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Run native timers until the window shows `target`,
///  recording every text shown on the way.
pub(super) fn until_text(window: &AppWindow, target: &str, seen: &mut Vec<String>) {
    let start = Instant::now();
    loop {
        update_timers_and_animations();
        note(window, seen);
        if window.get_source_text() == target {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "the window did not show the expected text within 2 s; texts shown: {seen:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Run native timers until the gap between notified rereads has passed since the last read.
/// After it,
///  only the write wait can hold back the read of a notification that follows.
fn rest(window: &AppWindow) {
    let mut ignored = Vec::new();
    watch_texts(window, REREAD_GAP + Duration::from_millis(50), &mut ignored);
}

/// Put `OLD` on disk by atomic replace,
///  after a marker text,
///  waiting until the window shows each.
/// The read that shows `OLD` restarts the 1 s sweep clock,
///  so the save that follows within a few
/// hundred milliseconds cannot coincide with a sweep read.
/// This is setup,
///  so it waits with the 2 s bound;
///  the watch tests assert that a replace shows promptly.
pub(super) fn baseline(window: &AppWindow, displayed: &Path) {
    for text in ["marker\n", OLD] {
        let staged = displayed.with_extension("tmp");
        fs::write(&staged, text).expect("staged baseline");
        fs::rename(&staged, displayed).expect("atomic baseline");
        let mut ignored = Vec::new();
        until_text(window, text, &mut ignored);
    }
    rest(window);
}

/// An in-place save (truncate,
///  part of the text,
///  a pause,
///  the rest,
///  close) is shown only when finished,
/// and the selection follows the replaced region.
#[test]
fn native_source_does_not_show_an_in_place_save_before_it_finishes() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let folder = fixture.path().join("nested");
    fs::create_dir(&folder).expect("collapsed folder");
    let displayed = folder.join("view.txt");
    fs::write(&displayed, OLD).expect("displayed source");
    let (window, state, _timers) = open(fixture.path(), &displayed);
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().refresh.is_watched()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    baseline(&window, &displayed);
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    render(&window, &state);
    let mut seen = Vec::new();
    // Opening with truncation empties the file; the first write leaves it half written.
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&displayed)
        .expect("open for an in-place save");
    file.write_all(b"I was a big cat, ").expect("first half");
    watch_texts(&window, PAUSE, &mut seen);
    file.write_all(b"but now I am a human!\n")
        .expect("second half");
    drop(file);
    until_text(&window, NEW, &mut seen);
    assert_eq!(seen, [OLD, NEW], "a save in progress was shown");
    assert_eq!(
        window.get_selected_text(),
        "was a",
        "the selection did not follow the saved replacement"
    );
    window.hide().expect("close save window");
}

/// A save that deletes the file and writes it again is shown only when the new file is finished:
/// neither the missing file nor the empty new file replaces the displayed text.
/// The new file is created after the missing file was read and the reread gap has passed,
///  so only
/// the write wait keeps the empty file from being read.
#[test]
fn native_source_does_not_show_a_deleted_file_before_its_rewrite_finishes() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let folder = fixture.path().join("nested");
    fs::create_dir(&folder).expect("collapsed folder");
    let displayed = folder.join("view.txt");
    fs::write(&displayed, OLD).expect("displayed source");
    let (window, state, _timers) = open(fixture.path(), &displayed);
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().refresh.is_watched()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    baseline(&window, &displayed);
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    render(&window, &state);
    let mut seen = Vec::new();
    fs::remove_file(&displayed).expect("delete before rewriting");
    watch_texts(&window, REREAD_GAP + Duration::from_millis(50), &mut seen);
    let mut file = File::create(&displayed).expect("create the new file");
    watch_texts(&window, PAUSE, &mut seen);
    file.write_all(NEW.as_bytes()).expect("write the new file");
    drop(file);
    until_text(&window, NEW, &mut seen);
    assert_eq!(seen, [OLD, NEW], "a rewrite in progress was shown");
    assert_eq!(
        window.get_selected_text(),
        "was a",
        "the selection did not follow the rewritten file"
    );
    {
        let mut waiting = Waiting::prompt();
        while !(state.borrow().file_error.is_none()) {
            waiting.step_missing("the cleared missing-file diagnostic");
        }
    }
    window.hide().expect("close rewrite window");
}

/// For each pause between truncating the displayed file and writing it,
///  count how often the window
/// showed the truncated (empty) file and how often the selection still followed the replacement.
/// Pauses longer than the quiet period are expected to show the truncated file.
#[test]
#[ignore = "measurement; run through inspect:refresh-latency with the filter write_wait_pause"]
fn write_wait_pause_sweep() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let folder = fixture.path().join("nested");
    fs::create_dir(&folder).expect("collapsed folder");
    let displayed = folder.join("view.txt");
    fs::write(&displayed, OLD).expect("displayed source");
    let (window, state, _timers) = open(fixture.path(), &displayed);
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().refresh.is_watched()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    let trials = 8;
    for pause_ms in [10_u64, 20, 30, 40, 50, 60, 70, 80, 100, 150, 200] {
        let mut truncated_shown = 0;
        let mut selection_kept = 0;
        // What: `Vec<u128>` collects, per trial that showed the truncated file, the milliseconds from the
        //       start of the trial to the truncation (`as_millis` returns `u128`, an unsigned integer).
        // Why: A read that the save's own notification did not ask for can still meet the unfinished file:
        //      about 1000 ms points at the safety sweep, a few hundred at a read requested before the save.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const shownSetupMs: number[] = [];
        // ```
        let mut shown_setup_ms = Vec::new();
        // The slowest setup of this pause length; about 400 ms is usual. Much more means the host or the
        // fixture's disk stalled, or a baseline text arrived only with the sweep.
        let mut slowest_setup = Duration::ZERO;
        for _ in 0..trials {
            let begun = Instant::now();
            baseline(&window, &displayed);
            state.borrow_mut().document.select(ReadingPosition {
                anchor: 2,
                head: 6,
                viewport: 0,
                affinity: ide_app::document::ReadingAffinity::Downstream,
            });
            render(&window, &state);
            let mut seen = Vec::new();
            let setup = begun.elapsed();
            slowest_setup = slowest_setup.max(setup);
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&displayed)
                .expect("open for an in-place save");
            watch_texts(&window, Duration::from_millis(pause_ms), &mut seen);
            file.write_all(NEW.as_bytes())
                .expect("write after the pause");
            drop(file);
            until_text(&window, NEW, &mut seen);
            // What: `String::is_empty` is true for the empty string; `any` asks whether one recorded text is.
            // Why: The truncated file is empty, so showing it means the window read mid-save.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (seen.some(text => text === '')) truncatedShown += 1;
            // ```
            if seen.iter().any(String::is_empty) {
                truncated_shown += 1;
                shown_setup_ms.push(setup.as_millis());
            }
            if window.get_selected_text() == "was a" {
                selection_kept += 1;
            }
        }
        println!(
            "{{\"case\":\"write-wait-pause\",\"quiet_ms\":{},\"limit_ms\":{},\"pause_ms\":{pause_ms},\"trials\":{trials},\"truncated_shown\":{truncated_shown},\"selection_kept\":{selection_kept},\"shown_setup_ms\":{shown_setup_ms:?},\"slowest_setup_ms\":{}}}",
            WRITE_QUIET.as_millis(),
            WRITE_WAIT_LIMIT.as_millis(),
            slowest_setup.as_millis()
        );
    }
    window.hide().expect("close measurement window");
}
