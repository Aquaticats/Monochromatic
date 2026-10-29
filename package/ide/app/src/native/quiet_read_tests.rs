//! Reads that no write notification asked for do not show a save in progress.
//!  A displayed file outside
//! the project has no watch at all,
//!  so no notification of the save ever arrives;
//!  the test asks for the
//! reread a new watch,
//!  a newly displayed file,
//!  or a full reread after lost events asks for,
//!  mid-save.

/// Shipped window setup shared with the watch tests.
use super::watch_tests::open;
/// The slow-writer helpers and texts of the write-wait tests.
use super::write_wait_tests::{NEW, OLD, PAUSE, until_text, watch_texts};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// The kind of change the navigation records for a reread no write was seen behind.
use ide_app::change_watch::SourceChange;
/// The quiet period a save must stay under for the assertion to apply.
use ide_app::refresh_policy::WRITE_QUIET;
/// `ComponentHandle` provides `hide` on the generated window.
use slint::ComponentHandle;
/// What:
///  `OpenOptions` opens a file with chosen flags;
///  `Write` provides `write_all`;
///  `Instant` is a monotonic time.
/// Why:
///  The test plays a save that truncates the file and finishes 40 ms later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = openSync(path, 'w'); /* wait */ writeSync(handle, text); closeSync(handle);
/// ```
use std::{
    fs::{self, OpenOptions},
    io::Write,
    time::{Duration, Instant},
};

/// A save in progress when a reread with no write behind it is asked for is shown only when finished.
/// A trial in which the test thread itself stalled until the save outlasted the quiet period proves
/// nothing,
///  so up to five trials run until one did not stall.
#[test]
fn reads_no_notification_asked_for_do_not_show_a_save_in_progress() {
    let project = tempfile::tempdir().expect("disposable project");
    let elsewhere = tempfile::tempdir().expect("disposable outside folder");
    let displayed = elsewhere
        .path()
        .canonicalize()
        .expect("canonical outside folder")
        .join("view.txt");
    let mut checked = 0;
    for _ in 0..5 {
        fs::write(&displayed, OLD).expect("displayed source");
        let (window, state, _timers) = open(project.path(), &displayed);
        // Opened from a language target: outside the project, so nothing watches it.
        state.borrow_mut().outside_project = true;
        let mut seen = Vec::new();
        // The reader thread starts by loading highlighting, which can outlast a save; once a highlighting
        // answer for the displayed text was applied, it is idle, and the rest lets the startup rereads finish.
        {
            let mut waiting = Waiting::eventually();
            loop {
                {
                    let current = state.borrow();
                    if current.syntax_revision == Some(current.document.revision()) {
                        break;
                    }
                }
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
        watch_texts(&window, Duration::from_millis(300), &mut seen);
        let truncated_at = Instant::now();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&displayed)
            .expect("open for an in-place save");
        // What the navigation records after a new watch, a switch, or lost events: read at the next tick.
        state
            .borrow_mut()
            .refresh
            .changed(SourceChange::Reread, Instant::now());
        watch_texts(&window, PAUSE, &mut seen);
        let held = truncated_at.elapsed();
        file.write_all(NEW.as_bytes()).expect("finish the save");
        drop(file);
        until_text(&window, NEW, &mut seen);
        window.hide().expect("close the window");
        if held >= WRITE_QUIET {
            continue;
        }
        checked += 1;
        assert_eq!(seen, [OLD, NEW], "a save in progress was shown");
        break;
    }
    assert!(
        checked > 0,
        "every trial outlasted the quiet period because the test thread stalled"
    );
}
