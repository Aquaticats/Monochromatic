//! An ignored measurement:
//!  how often a save that begins when the safety sweep comes due is read
//! unfinished,
//!  because the sweep read is not asked for by the save's notification.

/// The bounded wait for startup state,
///  and the renderer that shows a new selection.
use super::render;
/// Shipped window setup shared with the watch tests.
use super::watch_tests::open;
/// The slow-writer helpers and texts of the asserting write-wait tests.
use super::write_wait_tests::{NEW, OLD, PAUSE, baseline, until_text, watch_texts};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Reading positions,
///  and the shipped intervals named in the measurement output.
use ide_app::{
    document::ReadingPosition,
    refresh_policy::{SAFETY_SWEEP, WRITE_QUIET},
};
/// `ComponentHandle` provides `hide` on the generated window.
use slint::ComponentHandle;
/// What:
///  `OpenOptions` opens a file with chosen flags;
///  `Write` provides `write_all`;
///  `Duration` is a time span.
/// Why:
///  The measurement plays a slow writer whose save begins at a chosen time.
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

/// Each trial starts an in-place save,
///  unfinished for `PAUSE`,
///  at a pseudo-random time within the
/// 400 ms that contain the next sweep read.
///  The write wait cannot hold that read back until the
/// save's notification has reached the schedule.
///  The share of trials that showed the truncated file,
/// times 400 ms,
///  is the time before each sweep read in which a beginning save is read unfinished.
/// A trial counts only when the file stayed truncated for less than the quiet period in real time:
///  on a
/// loaded host the test thread itself can stall mid-save,
///  which makes the save a long one that any read may
/// show by design.
///  Such stretched trials are repeated,
///  up to 60 times,
///  and reported separately.
#[test]
#[ignore = "measurement; run through inspect:refresh-latency with the filter write_wait_timer"]
fn write_wait_timer_coincidence() {
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
    let trials = 120;
    let span_ms = 400_u32;
    let mut truncated_shown = 0;
    let mut shown_wait_ms = Vec::new();
    let mut counted = 0;
    let mut stretched = 0;
    let mut shown_when_stretched = 0;
    // The slowest setup; about 400 ms is usual, and far more means the host stalled during the run.
    let mut slowest_setup = Duration::ZERO;
    // What: xorshift, a three-step shift-and-xor generator over a `u32` (unsigned 32-bit integer).
    // Why: The save times must differ between trials yet repeat between runs of the measurement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5;
    // ```
    let mut seed = 0x9E37_79B9_u32;
    while counted < trials && stretched < 60 {
        let begun = Instant::now();
        baseline(&window, &displayed);
        slowest_setup = slowest_setup.max(begun.elapsed());
        seed ^= seed << 13_u32;
        seed ^= seed >> 17_u32;
        seed ^= seed << 5_u32;
        // What: `%` is the remainder; `u64::from` widens the `u32` without loss (`as` would also permit lossy casts).
        // Why: The baseline read restarted the sweep clock about 170 ms before `baseline` returned, so the
        //      next sweep read is due about 830 ms from now; the wait covers 600 ms to 1000 ms.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const waitMs = 600 + (seed % spanMs);
        // ```
        let wait_ms = u64::from(600 + seed % span_ms);
        let mut ignored = Vec::new();
        watch_texts(&window, Duration::from_millis(wait_ms), &mut ignored);
        state.borrow_mut().document.select(ReadingPosition {
            anchor: 2,
            head: 6,
            viewport: 0,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
        render(&window, &state);
        let mut seen = Vec::new();
        let truncated_at = Instant::now();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&displayed)
            .expect("open for an in-place save");
        watch_texts(&window, PAUSE, &mut seen);
        // How long the file really stayed truncated, open syscall included.
        let held = truncated_at.elapsed();
        file.write_all(NEW.as_bytes())
            .expect("write after the pause");
        drop(file);
        until_text(&window, NEW, &mut seen);
        // What: `iter().any(String::is_empty)` asks whether one recorded text is the empty string.
        // Why: The truncated file is empty, so showing it means the window read mid-save.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (seen.some(text => text === '')) { truncatedShown += 1; shownWaitMs.push(waitMs); }
        // ```
        let shown = seen.iter().any(String::is_empty);
        if held >= WRITE_QUIET {
            stretched += 1;
            if shown {
                shown_when_stretched += 1;
            }
            continue;
        }
        counted += 1;
        if shown {
            truncated_shown += 1;
            shown_wait_ms.push(wait_ms);
        }
    }
    println!(
        "{{\"case\":\"write-wait-timer\",\"sweep_ms\":{},\"quiet_ms\":{},\"span_ms\":{span_ms},\"pause_ms\":{},\"trials\":{counted},\"truncated_shown\":{truncated_shown},\"shown_wait_ms\":{shown_wait_ms:?},\"stretched_trials\":{stretched},\"shown_when_stretched\":{shown_when_stretched},\"slowest_setup_ms\":{}}}",
        SAFETY_SWEEP.as_millis(),
        WRITE_QUIET.as_millis(),
        PAUSE.as_millis(),
        slowest_setup.as_millis()
    );
    window.hide().expect("close measurement window");
}
