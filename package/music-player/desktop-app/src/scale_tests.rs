//! Scale tests for the library window at 1k, 10k, 100k and 1M tracks, with no window and no filesystem.
//!
//! What: a deterministic in-memory library projected through the functions the window calls on every update: the page
//! index built when a queue arrives, the view of the first, middle and last page, the page lookup of the playing track,
//! the folder picker, the measure reports of the analysis sweep, and the Search service fed with the whole library.
//! Why: 1,000 artists and 1,000,000 tracks are normal operation (D112). The window used to regroup every name on every
//! projection, copy the measure map on every sweep chunk and search on the window thread; these tests print the shape
//! of each stage and fail when a stage grows with the library instead of with the page.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test.each([1_000, 10_000, 100_000, 1_000_000])("projects %i tracks", (size) => {
//!   expect(viewMs(syntheticLibrary(size), middlePage)).toBeLessThan(100);
//! });
//! ```

use std::hint::black_box;
use std::time::{Duration, Instant};

use music_player::template_model::DEFAULT_TRACK_TEMPLATE;
use music_player::track_measure::{TrackMeasure, apply_measures};
use music_player::track_supporting_line::supporting_line_from_template;

use crate::scale_library_tests::{SyntheticLibrary, build_library};
use crate::ui_screen_rows::picker;
use crate::ui_screen_view::{ScreenState, selected_page_scope, view};

/// What: the tracks each artist folder holds, as the generator makes them.
/// Why: a page then holds about the same rows at every size.
const TRACKS_PER_ARTIST: usize = 1000;
/// What: runs per stage when a median is taken.
/// Why: the host is shared; five runs show the spread a median hides. A debug build runs once, because its timings are
/// only informational and the suite must stay short.
const REPEATS: usize = if cfg!(debug_assertions) { 1 } else { 5 };
/// What: the view bar at 1M, in release builds.
/// Why: the orchestrator's target: a projection under 100 ms at one million tracks.
const VIEW_BAR_RELEASE: Duration = Duration::from_millis(100);
/// What: the cost allowed per row of a very large page, in release builds.
/// Why: a view builds every row of the one selected page, so it is bounded by the page and not by the library. A page
/// of 30,000 loose root tracks (the largest in the generated library) measured 77 to 101 ms, about 2.8 microseconds a
/// row, after the row template was parsed once instead of once per row (it was 118 to 166 ms before). The bar is the
/// larger of the 100 ms target and 5 microseconds per row, so only pages above about 20,000 rows get the extra room, and
/// a regression that grows with the library still breaks it by orders of magnitude.
const PER_ROW_BAR: Duration = if cfg!(debug_assertions) { Duration::from_micros(150) } else { Duration::from_micros(5) };
/// What: the same bar in debug builds, which run unoptimized code about twenty times slower.
/// Why: the normal suite runs in debug; the bar still catches a stage that grows with the library.
const VIEW_BAR_DEBUG: Duration = Duration::from_millis(2000);

/// What: `Stage` is one timed stage with its spread.
/// Why: the report prints the median and the min to max range of five runs.
struct Stage {
    /// Stage name as printed.
    name: String,
    /// Median of the runs.
    median: Duration,
    /// Fastest run.
    min: Duration,
    /// Slowest run.
    max: Duration,
}

/// What: `timed` runs `stage` `REPEATS` times and records its spread under `name`.
/// Why: every stage uses it, so the numbers are comparable.
fn timed<T>(stages: &mut Vec<Stage>, name: &str, mut stage: impl FnMut() -> T) -> T {
    let mut samples: Vec<Duration> = Vec::new();
    let mut last: Option<T> = None;
    for _ in 0..REPEATS {
        let started: Instant = Instant::now();
        let value: T = stage();
        samples.push(started.elapsed());
        last = Some(black_box(value));
    }
    samples.sort();
    stages.push(Stage {
        name: name.to_string(),
        median: samples[samples.len() / 2],
        min: samples[0],
        max: samples[samples.len() - 1],
    });
    return last.expect("at least one repeat ran");
}

/// What: `read_kb` reads one kilobyte field of `/proc/self/status`.
/// Why: resident memory before and after the stages shows what the projection keeps alive.
fn read_kb(field: &str) -> u64 {
    let status: String = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    return status
        .lines()
        .find_map(|line| {
            let rest: &str = line.strip_prefix(field)?;
            return rest.split_whitespace().next()?.parse::<u64>().ok();
        })
        .unwrap_or(0);
}

/// What: `screen_state` builds the state the window holds after a queue update, with every track measured.
/// Why: a finished sweep leaves one measure per path, which is the heaviest state a projection reads.
fn screen_state(library: &SyntheticLibrary, measured: bool) -> ScreenState {
    let middle: usize = library.names.len() / 2;
    let all_paths: Vec<std::path::PathBuf> = library.paths();
    let mut state: ScreenState = ScreenState {
        paths: all_paths.clone(),
        current_index: Some(middle),
        track_name: library.names[middle].clone(),
        playing: true,
        loaded: true,
        ..ScreenState::default()
    };
    let _shared = state.set_names(&library.names);
    if measured {
        let measure: TrackMeasure = TrackMeasure { duration_secs: Some(200.0), peak_dbtp: Some(-1.5), peak_exact: true };
        let measured_count: usize = if cfg!(debug_assertions) { all_paths.len().min(200_000) } else { all_paths.len() };
        for chunk in all_paths[..measured_count].chunks(1024) {
            let reports: Vec<(std::path::PathBuf, TrackMeasure)> = chunk.iter().map(|path| return (path.clone(), measure)).collect();
            apply_measures(&mut state.measures, &reports);
        }
    }
    return state;
}

/// What: `identity_at` is the identity of the page at a display position.
/// Why: the stages select pages by position.
fn identity_at(state: &ScreenState, position: usize) -> String {
    return state.current_pages().page(position).map(|page| return page.identity.clone()).unwrap_or_default();
}

/// What: `print_stages` prints the stage table of one size.
/// Why: the report quotes these lines.
fn print_stages(label: &str, total: usize, stages: &[Stage], extra: &[String]) {
    println!("== window scale {label} ({total} names) ==");
    for stage in stages {
        println!(
            "  {:<28} {:>10.3} ms (min {:.3}, max {:.3})",
            stage.name,
            stage.median.as_secs_f64() * 1000.0,
            stage.min.as_secs_f64() * 1000.0,
            stage.max.as_secs_f64() * 1000.0
        );
    }
    for line in extra {
        println!("  {line}");
    }
}

/// What: `measure_window` runs every window stage at one size and checks the bounds that hold at any size.
/// Why: the page bound and the bar are checked where the outputs exist.
fn measure_window(label: &str, library: &SyntheticLibrary, view_bar: Duration) {
    let rss_before: u64 = read_kb("VmRSS:");
    let mut stages: Vec<Stage> = Vec::new();
    let mut extra: Vec<String> = Vec::new();
    let mut state: ScreenState = screen_state(library, true);
    timed(&mut stages, "set names (page index)", || {
        let mut fresh: ScreenState = ScreenState::default();
        return fresh.set_names(&library.names).len();
    });
    let page_count: usize = state.current_pages().page_count();
    assert!(page_count > 0 && page_count <= library.artists + 64, "pages {page_count} for {} artists", library.artists);
    let supporting_line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE);
    for (name, position) in [("view first page", 0), ("view middle page", page_count / 2), ("view last page", page_count - 1)] {
        state.selected_key = identity_at(&state, position);
        let screen = timed(&mut stages, name, || return view(&state, &supporting_line));
        let page_len: usize = state.selected_page_scope_indices().len();
        assert!(screen.rows.len() <= 2 * page_len, "{name}: rows {} exceed twice the page of {page_len}", screen.rows.len());
        let slowest: Duration = stages.last().map_or(Duration::ZERO, |stage| return stage.median);
        let allowed: Duration = view_bar.max(PER_ROW_BAR * u32::try_from(page_len).unwrap_or(u32::MAX));
        assert!(slowest < allowed, "{name} took {slowest:?} at {label}, over the {allowed:?} bar (page of {page_len})");
        extra.push(format!("{name}: page_len={page_len} rows={}", screen.rows.len()));
    }
    let middle: usize = library.names.len() / 2;
    timed(&mut stages, "page of playing track", || return state.page_key_of_index(middle));
    timed(&mut stages, "selected page scope", || return selected_page_scope(&state));
    let labels: Vec<String> = state.current_pages().pages().map(|page| return page.label.clone()).collect();
    timed(&mut stages, "picker", || {
        let sections = music_player::folder_rail::rail_for(&labels);
        return picker(&sections, &labels[labels.len() / 2]);
    });
    let on_page: Vec<std::path::PathBuf> = state.selected_page_scope_indices().iter().take(128).map(|index| return state.paths[*index].clone()).collect();
    let chunk_on_page: Vec<(std::path::PathBuf, TrackMeasure)> = on_page.into_iter().map(|path| return (path, TrackMeasure::default())).collect();
    let off_page_path: std::path::PathBuf = std::path::PathBuf::from("/library/not on any page.flac");
    let chunk_off_page: Vec<(std::path::PathBuf, TrackMeasure)> = vec![(off_page_path, TrackMeasure::default()); 128];
    timed(&mut stages, "measure chunk (on page)", || {
        apply_measures(&mut state.measures, &chunk_on_page);
        return state.measures_touch_selected_page(&chunk_on_page);
    });
    assert!(state.measures_touch_selected_page(&chunk_on_page), "a measure on the shown page must repaint");
    timed(&mut stages, "measure chunk (off page)", || {
        apply_measures(&mut state.measures, &chunk_off_page);
        return state.measures_touch_selected_page(&chunk_off_page);
    });
    assert!(!state.measures_touch_selected_page(&chunk_off_page), "a measure off the shown page must not repaint");
    let rss_after: u64 = read_kb("VmRSS:");
    extra.push(format!("pages={page_count} artists={} rss before {rss_before} kB, after {rss_after} kB", library.artists));
    print_stages(label, library.names.len(), &stages, &extra);
}

/// What: `window_scale` runs the window and search stages for one size.
/// Why: each size is its own test so a failure names the size.
fn window_scale(label: &str, artists: usize) {
    let library: SyntheticLibrary = build_library(artists, TRACKS_PER_ARTIST);
    let view_bar: Duration = if cfg!(debug_assertions) { VIEW_BAR_DEBUG } else { VIEW_BAR_RELEASE };
    measure_window(label, &library, view_bar);
}

/// What: `scale_1k_window_is_bounded_by_the_page` projects 1,000 tracks.
/// Why: the smallest size, where the page is most of the library.
#[test]
fn scale_1k_window_is_bounded_by_the_page() {
    window_scale("1k", 1);
}

/// What: `scale_10k_window_is_bounded_by_the_page` projects 10,000 tracks.
/// Why: the middle size checks that the page bound holds as artists are added.
#[test]
fn scale_10k_window_is_bounded_by_the_page() {
    window_scale("10k", 10);
}

/// What: `scale_100k_window_is_bounded_by_the_page` projects 100,000 tracks.
/// Why: 100k is the size the 1M numbers are compared with.
#[test]
fn scale_100k_window_is_bounded_by_the_page() {
    window_scale("100k", 100);
}

/// What: `scale_1m_window_is_bounded_by_the_page` projects 1,000,000 tracks.
/// Why: 1,000 artists and 1,000,000 tracks are normal operation, so this test is in the normal suite.
#[test]
fn scale_1m_window_is_bounded_by_the_page() {
    window_scale("1M", 1000);
}
