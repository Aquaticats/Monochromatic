//! D111 on the testing backend: Search while a deliberately slow scan adds tracks in batches.
//!
//! What: the real window, the real Search service and its feeder run on the testing backend. A query is typed through
//! the window's own callback while the feeder adds a batch every 80 ms; answers are applied the way the event loop
//! would apply them.
//! Why: D111 says the open Search page must show results for the tracks known at that moment, then for the tracks
//! that arrive, keep the query, update the one rows model in place, and not move the row the user is about to tap.
//! A scan that has not finished must not claim "no results".

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use i_slint_backend_testing::init_no_event_loop;
use music_player::search_feed::feed_library;
use music_player::search_service::SearchAnswer;
use slint::{ComponentHandle, Model};

use crate::AppWindow;
use crate::ui_screen::read_state;
use crate::ui_screen_view::Route;
use crate::ui_search::{install, service, wire_route};
use crate::ui_search_apply::apply_answer;
use crate::ui_search_model::{SEARCH_SCANNING_TITLE};

/// How long a test waits for the window to show what it expects.
const PATIENCE: Duration = Duration::from_secs(20);

/// What: `Scan` is the window under test with the answers its Search service produced and not yet applied.
/// Why: the event loop is not running, so the test applies answers itself, in the order they were produced.
struct Scan {
    /// The window.
    app: AppWindow,
    /// Answers waiting to be applied.
    inbox: Arc<Mutex<Vec<SearchAnswer>>>,
}

/// Methods of `Scan`.
impl Scan {
    /// What: `new` builds the window on the testing backend, starts the service and opens the Search route.
    /// Why: every case starts from an open, empty Search page.
    fn new() -> Scan {
        init_no_event_loop();
        let app: AppWindow = AppWindow::new().expect("the window builds on the testing backend");
        app.window().set_size(slint::LogicalSize::new(850.0, 900.0));
        let inbox: Arc<Mutex<Vec<SearchAnswer>>> = Arc::new(Mutex::new(Vec::new()));
        let sink: Arc<Mutex<Vec<SearchAnswer>>> = Arc::clone(&inbox);
        install(&app, move |answer| {
            if let Ok(mut waiting) = sink.lock() {
                waiting.push(answer);
            }
        });
        wire_route(&app);
        app.invoke_open_search();
        assert_eq!(app.get_route(), Route::Search.to_int());
        return Scan { app, inbox };
    }

    /// What: `pump` applies waiting answers until `done` is true.
    /// Why: it stands in for the event loop that delivers answers in production.
    fn pump(&self, done: impl Fn(&AppWindow) -> bool) {
        let deadline: Instant = Instant::now() + PATIENCE;
        loop {
            let waiting: Vec<SearchAnswer> = match self.inbox.lock() {
                Ok(mut inbox) => std::mem::take(&mut *inbox),
                Err(_) => Vec::new(),
            };
            for answer in waiting {
                apply_answer(&self.app, answer);
            }
            if done(&self.app) {
                return;
            }
            assert!(Instant::now() < deadline, "the window never showed the expected state");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// What: `type_query` types a query through the window's own callback.
    /// Why: this is the path a keystroke takes.
    fn type_query(&self, text: &str) {
        self.app.invoke_search_query_edited(text.into());
    }

    /// What: `titles` lists the first segment of every shown row title.
    /// Why: the tests compare what is shown, not how it is stored.
    fn titles(&self) -> Vec<String> {
        let rows = self.app.get_search_rows();
        return (0..rows.row_count())
            .filter_map(|position| return rows.row_data(position))
            .map(|row| {
                return row
                    .title_segments
                    .iter()
                    .map(|segment| return segment.text.to_string())
                    .collect::<String>();
            })
            .collect();
    }
}

/// What: `library` builds `batches` batches of `per_batch` tracks named `Artist/Trk NNNN.flac`, with a track named
/// plainly `Trk` placed in batch index `best_in`.
/// Why: the plain name is a whole-name match, which ranks above every numbered name, so it lands at the top when it arrives.
fn library(batches: usize, per_batch: usize, best_in: usize) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for number in 0..batches * per_batch {
        if number == best_in * per_batch {
            names.push("Artist/Trk.flac".to_string());
        } else {
            names.push(format!("Artist/Trk {number:04}.flac"));
        }
    }
    return names;
}

/// What: Typing while the scan runs shows the known tracks, then the new ones, in the same rows model, with the query kept.
/// Why: D111: results for the tracks already known and later for the new ones, refreshed in place.
#[test]
fn results_grow_in_place_while_a_slow_scan_adds_tracks() {
    let scan: Scan = Scan::new();
    let running = service().expect("the service is installed");
    feed_library(&running, Arc::new(library(6, 100, 99)), 100, Duration::from_millis(80));
    scan.pump(|_| return running.track_count() > 0);
    scan.type_query("trk");
    scan.pump(|app| return app.get_search_rows().row_count() > 0);
    let early_rows = scan.app.get_search_rows();
    let early_count: usize = early_rows.row_count();
    assert!(early_count > 0 && early_count < 500, "the first answer showed {early_count} rows");
    scan.pump(|app| return app.get_search_rows().row_count() == 500);
    assert_eq!(early_rows.row_count(), 500, "the same rows model grew; it was updated in place");
    assert_eq!(scan.app.get_search_query().as_str(), "trk", "the refresh leaves the query alone");
    let results = read_state(|state| return state.search_ui.results.len());
    assert_eq!(results, 500, "the cap of five hundred applies to the grown library");
}

/// What: A query that matches nothing yet shows the scanning copy while the scan runs, and the verdict after it ends.
/// Why: D69: a partial scan cannot establish that no name matches.
#[test]
fn no_match_is_not_claimed_before_the_scan_is_complete() {
    let scan: Scan = Scan::new();
    let running = service().expect("the service is installed");
    feed_library(&running, Arc::new(library(5, 100, 99)), 100, Duration::from_millis(100));
    scan.pump(|_| return running.track_count() > 0);
    scan.type_query("zzzz");
    scan.pump(|app| return app.get_search_status_title().as_str() == SEARCH_SCANNING_TITLE);
    scan.pump(|app| return app.get_search_status_title().as_str().starts_with("No results for"));
    assert_eq!(scan.titles().len(), 0);
}

/// What: When a better match arrives while the user has scrolled, the list is anchored so the first visible row keeps its place.
/// Why: D111: the row the user is about to tap must not move.
#[test]
fn a_better_match_arriving_does_not_move_the_row_in_view() {
    let scan: Scan = Scan::new();
    let running = service().expect("the service is installed");
    feed_library(&running, Arc::new(library(4, 100, 3)), 100, Duration::from_millis(150));
    scan.pump(|_| return running.track_count() >= 100);
    scan.type_query("trk");
    scan.pump(|app| return app.get_search_rows().row_count() >= 100);
    // The user has scrolled two rows down.
    scan.app.set_search_scroll_y(-144.0);
    let tick_before: i32 = scan.app.get_search_anchor_tick();
    scan.pump(|app| return app.get_search_rows().row_count() == 400);
    assert_eq!(scan.titles()[0], "Trk", "the whole-name match ranks first once it arrives");
    assert!(scan.app.get_search_anchor_tick() > tick_before, "the list was anchored after the re-rank");
    assert_eq!(scan.app.get_search_anchor_offset(), 144.0 + 72.0, "one row was inserted above the first visible row");
}

/// What: An answer to an older query is dropped.
/// Why: a slow ranking must never overwrite the rows of a newer query.
#[test]
fn an_answer_to_an_older_query_is_dropped() {
    let scan: Scan = Scan::new();
    let running = service().expect("the service is installed");
    feed_library(&running, Arc::new(library(1, 50, 0)), 50, Duration::ZERO);
    scan.pump(|_| return running.track_count() == 50);
    scan.type_query("trk");
    scan.pump(|app| return app.get_search_rows().row_count() == 50);
    let old_answer: SearchAnswer = SearchAnswer {
        sequence: 0,
        query: "trk".to_string(),
        state: music_player::search_results::SearchState::NoMatch("trk".to_string()),
        known_tracks: 50,
        complete: true,
        refresh: false,
    };
    apply_answer(&scan.app, old_answer);
    assert_eq!(scan.app.get_search_rows().row_count(), 50);
}
