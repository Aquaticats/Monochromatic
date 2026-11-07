//! Window binding for the Search route:
//! opens and closes it,
//! sends each query edit to the Search service,
//! and
//! activates a result.
//!
//! What:
//! `wire` registers the Search callbacks and starts the Search service;
//! `project_search` sends the current query
//! to the service;
//! the service answers on its own thread and `ui_search_apply` puts the answer in the window.
//! Why:
//! D64 (Back exits),
//! D66 (a new visit starts empty),
//! D72 (a folder result opens without autoplay),
//! D74 (a track
//! result returns to the player),
//! and D111/D112 (search never runs on the window thread,
//! and a library that is still
//! being fed refreshes the open results in place) are applied here.

/// Imports the cell type that holds the service slot on the window thread.
use std::cell::RefCell;
/// Imports the shared ownership type that shares the service with the callbacks.
use std::rc::Rc;
/// Imports the atomic shared ownership type that lets the service be shared with its worker threads.
use std::sync::Arc;
/// Imports the duration type of the pause between the batches that the library feed hands over.
use std::time::Duration;

/// Imports the command that the search sends to the engine.
use music_player::command::Command;
/// Imports the engine that receives the commands.
use music_player::engine::Engine;
/// Imports the library feeder and its batch size.
use music_player::search_feed::{FEED_BATCH, feed_library};
/// Imports the query splitter.
use music_player::search_matcher::search_terms;
/// Imports the result kind and the search state.
use music_player::search_results::{SearchResultKind, SearchState};
/// Imports the answer type and the service that answers queries.
use music_player::search_service::{SearchAnswer, SearchService};
/// Imports the component handle that reaches the window.
use slint::ComponentHandle;

/// Imports the window that shows the search.
use crate::AppWindow;
/// Imports the host through which actions leave the window.
use crate::ui_host::Host;
/// Imports the projection and the state accessors of the screen.
use crate::ui_screen::{project, read_screen, write_screen};
/// Imports the route,
/// the screen state and the folder selection.
use crate::ui_screen_view::{Route, ScreenState, select_folder_label};
/// Imports the application of an answer to the window.
use crate::ui_search_apply::{apply_answer, install_rows_model, show_view};
/// Imports the pure search view builder.
use crate::ui_search_model::search_view_of;

thread_local! {
    /// The window thread's handle to the Search service.
    static SERVICE: RefCell<Option<Arc<SearchService>>> = const { RefCell::new(None) };
}

/// What:
/// `service` returns the Search service,
/// if one was installed.
/// Why:
/// callbacks and the library feed reach the service through the window thread's handle.
pub(crate) fn service() -> Option<Arc<SearchService>> {
    return SERVICE.with(std::cell::RefCell::clone).into_inner();
}

/// What:
/// `ServiceShutdown` stops the Search service's threads when `main` ends,
/// whichever way it ends.
/// Why:
/// the thread-local handle above is dropped after the process started ending,
/// when joining the service's threads
/// panics on Windows (`SearchService::shutdown` has the details).
/// A guard in `main` joins them first.
pub(crate) struct ServiceShutdown;

/// Joins the Search service's threads when the guard is dropped.
impl Drop for ServiceShutdown {
    /// What:
    /// `drop` shuts down the installed Search service,
    /// if any.
    /// Why:
    /// every thread is still alive at this point,
    /// so the joins return.
    fn drop(&mut self) {
        if let Some(running) = service() {
            running.shutdown();
        }
    }
}

/// What:
/// `start_service` starts the Search service for the window,
/// delivering answers through the event loop.
/// Why:
/// `main` starts it before the engine,
/// because the engine thread feeds scan batches to the same service.
pub(crate) fn start_service(app: &AppWindow) -> Arc<SearchService> {
    let weak = app.as_weak();
    install(app, move |answer: SearchAnswer| {
        let target = weak.clone();
        let _posted = slint::invoke_from_event_loop(move || {
            if let Some(window) = target.upgrade() {
                apply_answer(&window, answer);
            }
        });
    });
    return service().expect("the Search service was just installed");
}

/// What:
/// `install` creates the rows model and starts the Search service with the given delivery.
/// Why:
/// production posts answers to the event loop;
/// a test collects them and applies them itself.
pub(crate) fn install(app: &AppWindow, deliver: impl Fn(SearchAnswer) + Send + 'static) {
    install_rows_model(app);
    let started: Arc<SearchService> = Arc::new(SearchService::start(deliver));
    SERVICE.set(Some(started));
}

/// What:
/// `feed_library_names` hands a new library to the Search service.
/// Why:
/// the service indexes it in batches off the window thread,
/// so Search works from the first batch (D111).
pub(crate) fn feed_library_names(names: Arc<Vec<String>>) {
    match service() {
        Some(running) => feed_library(&running, names, FEED_BATCH, Duration::ZERO),
        None => tracing::debug!("no Search service yet; the library is not fed"),
    }
}

/// What:
/// `wire` registers the Search callbacks on the window;
/// the service was started by `start_service`.
/// Why:
/// the Search page reports open,
/// close,
/// query edits and result activation through these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wireSearch(app: AppWindow, engine: Engine): void { ... };
/// ```
pub(crate) fn wire(app: &AppWindow, engine: &Rc<Engine>) {
    wire_route(app);
    wire_open(app, Rc::clone(engine) as Rc<dyn Host>);
}

/// What:
/// `wire_open` registers the callback that opens a result with a host.
/// Why:
/// the window tests open a result with a recording host.
pub(crate) fn wire_open(app: &AppWindow, host: Rc<dyn Host>) {
    let result_weak = app.as_weak();
    app.on_search_open(move |position| {
        if let Some(window) = result_weak.upgrade() {
            activate_result(&window, host.as_ref(), usize::try_from(position).unwrap_or(usize::MAX));
        }
    });
}

/// What:
/// `wire_route` registers open,
/// close,
/// edit and clear.
/// Why:
/// these need no engine,
/// so a test can wire them on a bare window.
pub(crate) fn wire_route(app: &AppWindow) {
    let open_weak = app.as_weak();
    app.on_open_search(move || {
        if let Some(window) = open_weak.upgrade() {
            open_search_route(&window);
        }
    });
    let close_weak = app.as_weak();
    app.on_close_search(move || {
        if let Some(window) = close_weak.upgrade() {
            close_search_route(&window);
        }
    });
    let edit_weak = app.as_weak();
    app.on_search_query_edited(move |query| {
        if let Some(window) = edit_weak.upgrade() {
            {
                let mut state = write_screen();
                state.search_query = query.to_string();
            }
            project_search(&window);
        }
    });
    let clear_weak = app.as_weak();
    app.on_search_clear(move || {
        if let Some(window) = clear_weak.upgrade() {
            {
                let mut state = write_screen();
                state.search_query = String::new();
            }
            project_search(&window);
        }
    });
}

/// What:
/// `open_search_route` shows the Search page with an empty query.
/// Why:
/// D66:
/// a new Search visit starts empty.
pub(crate) fn open_search_route(window: &AppWindow) {
    {
        let mut state = write_screen();
        state.route = Route::Search;
        state.search_query = String::new();
    }
    // The list position is shared with the page through a two-way property, so a new visit starts at the top.
    window.set_search_scroll_y(0.0);
    show_route(window);
}

/// What:
/// `close_search_route` returns to the player.
/// Why:
/// D64:
/// Back exits Search directly.
pub(crate) fn close_search_route(window: &AppWindow) {
    {
        let mut state = write_screen();
        state.route = Route::Player;
    }
    show_route(window);
}

/// What:
/// `show_route` projects the player and sends the Search query.
/// Why:
/// both routes change what the window shows.
fn show_route(window: &AppWindow) {
    project(window);
    project_search(window);
}

/// What:
/// `project_search` sets the query and route on the window and sends the query to the service.
/// Why:
/// the answer arrives later through `apply_answer`;
/// an empty query needs no index,
/// so its prompt is shown at once.
pub(crate) fn project_search(window: &AppWindow) {
    let (query, route) = { let state = read_screen(); (state.search_query.clone(), state.route) };
    window.set_search_query(query.as_str().into());
    window.set_route(route.to_int());
    if route != Route::Search {
        return;
    }
    let Some(running) = service() else {
        tracing::warn!("the Search service is not running; the query is not sent");
        return;
    };
    let sequence: u64 = running.query(&query);
    {
        let mut state = write_screen();
        state.search_ui.sequence = sequence;
    }
    if search_terms(&query).is_empty() {
        let (current_index, playing) = { let state = read_screen(); (state.current_index, state.playing) };
        show_view(window, search_view_of(&SearchState::EmptyQuery, current_index, playing, true), &query, false);
    }
}

/// What:
/// `activate_result` opens the result at `position` of the list the user is looking at.
/// Why:
/// the position refers to the shown list,
/// which was kept from the last answer;
/// searching again could return a
/// different row for the same position while the library is still growing.
pub(crate) fn activate_result(window: &AppWindow, engine: &dyn Host, position: usize) {
    let result = { let state = read_screen(); state.search_ui.results.get(position).cloned() };
    let Some(found) = result else {
        return;
    };
    match found.index_in_library {
        Some(index) if found.kind == SearchResultKind::Track => {
            // D127, D129: a result follows the row click rule (D124): a track that is not current is only selected
            // (and started with Auto-play on select on), the current track toggles play and pause.
            crate::ui_wire::play_row(engine, index);
            {
                let mut state = write_screen();
                state.route = Route::Player;
            }
        }
        _ => {
            let chosen: Option<Vec<usize>> = open_folder_result(&mut write_screen(), &found.name);
            let Some(scope) = chosen else {
                tracing::info!(folder = %found.name, "a folder result was tapped before the library rows exist; Search stays open");
                return;
            };
            engine.send(Command::SetPageScope(scope));
        }
    }
    show_route(window);
}

/// What:
/// `move_ring` moves the highlighted result of the Search page by `delta`,
/// staying inside the list.
/// Why:
/// Up and Down move through the results while the query field keeps the typing focus (D106).
pub(crate) fn move_ring(window: &AppWindow, delta: isize) {
    let count = read_screen().search_ui.results.len();
    if count == 0 {
        return;
    }
    {
        let mut state = write_screen();
        let here = state.search_ring.unwrap_or(0);
        state.search_ring = Some(here.saturating_add_signed(delta).min(count - 1));
    }
    let ring = read_screen().search_ring;
    window.set_search_ring(match ring {
        Some(row) => i32::try_from(row).unwrap_or(-1),
        None => -1,
    });
    tracing::debug!(?ring, "the Search highlight moved");
}

/// What:
/// `open_ringed_result` opens the highlighted result,
/// or the first one when none is highlighted.
/// Why:
/// Enter in the Search page opens the focused result (D106).
pub(crate) fn open_ringed_result(window: &AppWindow, host: &dyn Host) {
    let position = read_screen().search_ring.unwrap_or(0);
    activate_result(window, host, position);
}

/// What:
/// `open_folder_result` selects the folder page named by a result and returns to the player,
/// or does nothing.
/// Why:
/// D111:
/// Search works while the library is still being scanned,
/// and then the pages do not exist yet.
/// Leaving
/// Search for an empty player would lose the user's place,
/// so the tap changes nothing until the folder exists.
pub(crate) fn open_folder_result(state: &mut ScreenState, name: &str) -> Option<Vec<usize>> {
    let scope: Vec<usize> = select_folder_label(state, name)?;
    state.route = Route::Player;
    return Some(scope);
}

/// Folder tap tests in `ui_search_folder_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "ui_search_folder_tests.rs"]
mod tests;

/// Result activation tests in `ui_search_activation_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "ui_search_activation_tests.rs"]
mod activation_tests;
