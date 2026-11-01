//! How often a folder is reread by timers alone:
//!  the safety sweep for watched folders,
//!  and the
//! unwatched timer plus the sweep for folders without a live watch.
//!  Simulated in 20 ms native ticks.

/// What:
///  both schedules,
///  the change classification,
///  and the intervals they combine.
/// Why:
///  The simulation calls the shipped schedule,
///  so the printed intervals are its real behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DirectoryRefresh, SourceRefresh, SAFETY_SWEEP } from 'ide/refresh_policy';
/// ```
use ide_app::{
    change_watch::SourceChange,
    refresh_policy::{
        DirectoryRefresh, SAFETY_SWEEP, SourceRefresh, UNWATCHED_DIRECTORY_POLL,
        UNWATCHED_SOURCE_POLL, WRITE_QUIET,
    },
};
/// What:
///  `BTreeMap` maps each folder to its last read time;
///  `Duration`/`Instant` are spans and time points.
/// Why:
///  The longest gap between two reads of one folder is how stale an unreported change can get.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastRead = new Map<string, number>();
/// ```
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{Duration, Instant},
};

/// One native timer tick;
///  the single directory reader can start at most one read per tick.
const TICK: Duration = Duration::from_millis(20);

/// Longest time any of `folders` shown folders goes without a reread when nothing is notified.
/// `watched` false gives the folders no live watch,
///  so the unwatched timer runs beside the sweep.
fn longest_reread_interval(folders: u32, watched: bool) -> Duration {
    let start = Instant::now();
    let mut shown = Vec::new();
    for index in 0..folders {
        shown.push(PathBuf::from(format!("/p/{index:03}")));
    }
    // What: `if ... { a } else { b }` is an expression; both arms yield a `BTreeSet<PathBuf>`.
    // Why: An empty live-watch set makes every shown folder unwatched.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const live = watched ? new Set(shown) : new Set<string>();
    // ```
    let live: BTreeSet<PathBuf> = if watched {
        shown.iter().cloned().collect()
    } else {
        BTreeSet::new()
    };
    let mut refresh = DirectoryRefresh::default();
    let mut last_read: BTreeMap<PathBuf, Instant> = BTreeMap::new();
    let mut longest = Duration::ZERO;
    // The first three seconds settle the timers' phases; the next twenty are measured.
    let measured_from = start + Duration::from_secs(3);
    let mut now = start;
    while now < measured_from + Duration::from_secs(20) {
        refresh.start_sweep_if_due(&shown, now);
        if let Some(path) = refresh.next(&shown, &live, now) {
            // What: `insert` returns the previous read time of this folder, if it was read before.
            // Why: The gap since that read is one sample of how long the folder went unread.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const previous = lastRead.get(path); lastRead.set(path, now);
            // ```
            if let Some(previous) = last_read.insert(path, now)
                && previous >= measured_from
            {
                longest = longest.max(now.saturating_duration_since(previous));
            }
        }
        now += TICK;
    }
    // What: `map_or` reads the folder's last read time, or the start of the measured span if it had none.
    // Why: A folder the schedule stopped reading has no second read to measure a gap from, so the time
    //      from its last read to the end of the simulation counts as its interval.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const path of shown) longest = Math.max(longest, now - Math.max(lastRead.get(path) ?? 0, from));
    // ```
    for path in &shown {
        let last = match last_read.get(path) {
            Some(at) => (*at).max(measured_from),
            None => measured_from,
        };
        longest = longest.max(now.saturating_duration_since(last));
    }
    return longest;
}

/// One pass over more folders than the sweep interval can read (one read per 20 ms tick) outlasts the
/// interval.
///  The next pass then waits for the unfinished one,
///  so every folder is reread once per pass;
/// a pass restarting from the top each interval would never reach the folders late in visible order.
#[test]
fn sweep_rereads_every_folder_when_one_pass_outlasts_the_interval() {
    let folders = 120;
    let pass = TICK * folders;
    assert!(
        pass > SAFETY_SWEEP,
        "the pass no longer outlasts the interval"
    );
    let longest = longest_reread_interval(folders, true);
    println!(
        "folders={folders} pass_ms={} longest_unread_ms={}",
        pass.as_millis(),
        longest.as_millis()
    );
    assert!(
        longest <= pass + TICK,
        "a shown folder went unread for {} ms while sweeps restarted from the top",
        longest.as_millis()
    );
}

/// The safety sweep comes due every second,
///  so it regularly coincides with a save in progress.
/// While an unfinished write is waiting,
///  neither the timer nor a highlighting request reads the file;
/// the write wait reads it once the writer goes quiet.
#[test]
fn timers_do_not_read_a_file_whose_write_is_unfinished() {
    let start = Instant::now();
    let mut refresh = SourceRefresh::default();
    refresh.requested(start);
    refresh.set_watched(true);
    // The write starts 10 ms before the sweep comes due.
    let written = start + SAFETY_SWEEP - Duration::from_millis(10);
    refresh.changed(SourceChange::Unsettled, written);
    assert!(
        !refresh.due(start + SAFETY_SWEEP, false),
        "the sweep read a file 10 ms into an unfinished write"
    );
    assert!(
        !refresh.due(start + SAFETY_SWEEP, true),
        "a highlighting request read a file 10 ms into an unfinished write"
    );
    assert!(
        refresh.due(written + WRITE_QUIET, false),
        "the write was not read once it went quiet"
    );
}

/// Watched folders are reread once per sweep.
///  The unwatched timer shortens that only for a single
/// unwatched folder;
///  from the folder count where one sweep already outpaces the round robin,
///  it adds
/// reads without shortening the longest interval.
///  Run with `--nocapture` to print the intervals.
#[test]
fn unwatched_timer_shortens_the_sweep_interval_only_for_few_folders() {
    for folders in [1, 2, 3, 4, 8, 40] {
        println!(
            "folders={folders} sweep_only_ms={} unwatched_timer_and_sweep_ms={}",
            longest_reread_interval(folders, true).as_millis(),
            longest_reread_interval(folders, false).as_millis()
        );
    }
    assert_eq!(
        longest_reread_interval(1, true),
        SAFETY_SWEEP,
        "a watched folder is not reread once per sweep"
    );
    assert_eq!(
        longest_reread_interval(1, false),
        UNWATCHED_DIRECTORY_POLL.min(SAFETY_SWEEP),
        "a single unwatched folder is not reread on its own timer"
    );
    for folders in [1, 2, 3, 4, 8, 40] {
        assert!(
            longest_reread_interval(folders, false) <= longest_reread_interval(folders, true),
            "the unwatched timer made {folders} folders staler than the sweep alone"
        );
    }
}

/// A displayed file outside the project has no watch on purpose,
///  so the safety sweep rereads it,
/// not the 250 ms timer that stands in for a watch that failed.
#[test]
fn a_file_outside_the_project_is_reread_on_the_sweep() {
    let start = Instant::now();
    let mut refresh = SourceRefresh::default();
    refresh.requested(start);
    refresh.set_watched(false);
    refresh.set_outside_project(true);
    assert!(
        !refresh.due(start + UNWATCHED_SOURCE_POLL, false),
        "a file outside the project was polled like a failed watch"
    );
    assert!(
        refresh.due(start + SAFETY_SWEEP, false),
        "a file outside the project was not reread on the sweep"
    );
}
