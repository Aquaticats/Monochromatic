//! Window binding for the library screen:
//! applies engine updates to the UI-thread state and pushes
//! the projected view into `AppWindow`.
//!
//! What:
//! `apply_update` is the single entry for engine messages on the event-loop thread.
//! Why:
//! the page rules and the row and picker models stay in the pure modules;
//! this file only
//! converts them into Slint values and writes window properties.

/// Imports the cell that holds the screen state on the window thread and the guards that borrow it.
use std::cell::{Ref, RefCell, RefMut};
/// Imports the shared ownership type that shares models with the window.
use std::rc::Rc;

/// Imports the update type that engine messages arrive as.
use music_player::command::Update;
/// Imports the default track row template.
use music_player::template_model::DEFAULT_TRACK_TEMPLATE;
/// Imports the function that applies reported measures to the state.
use music_player::track_measure::apply_measures;
/// Imports the supporting line evaluation of a row.
use music_player::track_supporting_line::supporting_line_from_template;
/// Imports the component handle,
/// the model types and the shared string that fill the window.
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

/// Imports the conversion of the playback mode to the window's integer.
use crate::ui_playback::playback_mode_to_int;
/// Imports the scroll request type and its decision.
use crate::ui_screen_scroll::{ScrollRequest, next_scroll_request, scroll_offset_for};
/// Imports the screen state and view types and the projection.
use crate::ui_screen_view::{ScreenState, ScreenView, view};
/// Imports the window,
/// its generated row types and the time formatter.
use crate::{AppWindow, DeckState, FolderName, FolderRailCell, TrackListRow, format_time};

thread_local! {
    /// What:
    /// the UI thread's copy of the engine facts,
    /// read and written only on the event-loop thread.
    /// Why:
    /// `invoke_from_event_loop` runs every update on that thread,
    /// so no lock is needed.
    /// The cell is leaked once per thread so that `screen_cell` can hand out a plain reference,
    /// which lets callers hold a guard instead of passing a closure to `LocalKey::with`.
    static SCREEN: &'static RefCell<ScreenState> = Box::leak(Box::new(RefCell::new(ScreenState::default())));
}

/// What:
/// `apply_update` applies one engine message to the state and the window.
/// Why:
/// queue changes reset the folder selection,
/// playing-track changes follow the playing folder,
/// and position and playing changes only touch their own window properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyUpdate(app: AppWindow, update: Update): void { ... }
/// ```
pub(crate) fn apply_update(app: &AppWindow, update: &Update) {
    match update {
        Update::Library(paths) => {
            {
                let mut state = write_screen();
                state.paths = paths.clone();
            }
        }
        Update::Queue(names) => {
            let library = {
                let mut state = write_screen();
                let library = state.set_names(names);
                state.selected_key = String::new();
                state.loaded = true;
                state.reading = false;
                state.unplayable.clear();
                library
            };
            tracing::info!(names = library.len(), "queue applied; the library rows are projected next");
            crate::ui_search::feed_library_names(library);
            project(app);
            tracing::info!("queue projected");
        }
        Update::NowPlaying { index, name, duration } => {
            set_now_playing(app, *index, name, *duration);
            follow_playing(*index);
            project(app);
            crate::ui_state_log::emit(app, "now-playing");
        }
        Update::Reconciled { names, index, name, duration } => {
            tracing::info!(rows = names.len(), "the library list was reconciled in the window");
            let library = {
                let mut state = write_screen();
                let library = state.set_names_keeping_place(names);
                state.loaded = true;
                state.reading = false;
                library
            };
            crate::ui_search::feed_library_names(library);
            set_now_playing(app, *index, name, *duration);
            if index.is_none() {
                reset_position(app);
            }
            project(app);
        }
        Update::Position(secs) => {
            {
                let mut state = write_screen();
                state.position = *secs;
            }
            app.set_position(*secs as f32);
            app.set_position_text(format_time(*secs).into());
            crate::ui_state_log::emit(app, "position");
        }
        Update::Playing(on) => {
            {
                let mut state = write_screen();
                state.playing = *on;
            }
            app.set_playing(*on);
            crate::ui_state_log::emit(app, "playing");
        }
        Update::Volume(volume) => app.set_volume(*volume),
        Update::Measures(measures) => {
            let shown_changed = {
                let mut state = write_screen();
                apply_measures(&mut state.measures, measures);
                let mut shown_changed = state.measures_touch_selected_page(measures);
                // A file that was marked unplayable and now has a measured peak was replaced by one that decodes (D113),
                // so its mark goes.
                for (path, measure) in measures {
                    if measure.peak_dbtp.is_some() && state.unplayable.remove(path) {
                        tracing::info!(path = %path.display(), "a file marked unplayable now decodes; its mark is removed");
                        shown_changed = true;
                    }
                }
                shown_changed
            };
            if shown_changed {
                crate::ui_screen_throttle::project_soon(app);
            }
        }
        // The window draws no analysis indicator (D107); the system indicator takes this update in main.rs.
        Update::Analysis { .. } => {}
        Update::SourceRoot(name) => {
            {
                let mut state = write_screen();
                state.set_source_root_name(name.clone());
            }
            project(app);
        }
        // The Search index takes scan batches on the engine thread (main.rs); the window has nothing to draw for them.
        Update::ScanBatch { .. } => {}
        Update::Reading { name } => {
            tracing::info!(name = %name, "the folder is being read");
            {
                let mut state = write_screen();
                state.reading = true;
                state.set_source_root_name(Some(name.clone()));
            }
            project(app);
        }
        // The window marks the rows only (D122): the words about these files go to the desktop as a system notification,
        // which main.rs hands to the unplayable notifier on the engine thread, so no message is drawn here.
        Update::Unplayable(paths) => {
            tracing::warn!(files = paths.len(), "files that cannot be played are marked on their rows");
            {
                let mut state = write_screen();
                state.unplayable.extend(paths.iter().cloned());
            }
            project(app);
        }
        Update::AudioUnavailable { reason } => {
            tracing::warn!(%reason, "no audio output is available; the window tells the person");
            crate::ui_overlays::show_notice(app, crate::ui_notices::audio_unavailable_message());
        }
        Update::FileGone { path, was_playing } => {
            tracing::info!(path = %path.display(), was_playing, "the selected track's file left the folder");
            if crate::ui_files::was_just_trashed(path) {
                tracing::debug!(path = %path.display(), "the trash message already explains the missing file");
            } else {
                crate::ui_overlays::show_notice(app, &crate::ui_notices::file_gone_message(path, *was_playing));
            }
        }
        Update::Duration(duration) => {
            {
                let mut state = write_screen();
                state.duration = *duration;
            }
            app.set_duration(*duration as f32);
            app.set_duration_text(format_time(*duration).into());
        }
        Update::PlaybackMode(mode) => {
            {
                let mut state = write_screen();
                state.mode = *mode;
            }
            app.set_playback_mode(playback_mode_to_int(*mode));
            crate::ui_state_log::emit(app, "playback-mode");
        }
    }
}

/// What:
/// `set_now_playing` records and shows the playing track's name,
/// duration and queue index.
/// Why:
/// the window title,
/// the seek bar and the row highlight all read these values.
/// What:
/// `reset_position` puts the seek bar and the elapsed label back to the start.
/// Why:
/// when the playing file leaves the folder no `NowPlaying` follows,
/// and the rate-limited `Position` reset can be
/// dropped,
/// which left `0:02` next to `0:00` (the polish review finding PB2).
fn reset_position(app: &AppWindow) {
    {
        let mut state = write_screen();
        state.position = 0.0;
    }
    app.set_position(0.0);
    app.set_position_text(format_time(0.0).into());
}

/// What:
/// `pub(crate) fn set_now_playing(app, index, name, duration)` records the playing track in the screen state and shows its name,
/// length and queue position in the window.
/// Why:
/// The engine reports each track load once;
/// the deck and the keyboard code then read the same facts from the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setNowPlaying(app: AppWindow, index: number | null, name: string, duration: number): void {
///  state.trackName = name; state.duration = duration; state.currentIndex = index;
///  app.trackName = name; app.duration = duration; app.durationText = formatTime(duration); app.currentIndex = index ?? -1;
/// }
/// ```
pub(crate) fn set_now_playing(app: &AppWindow, index: Option<usize>, name: &str, duration: f64) {
    {
        let mut state = write_screen();
        state.track_name = name.to_string();
        state.duration = duration;
        state.current_index = index;
    }
    app.set_track_name(name.into());
    app.set_duration(duration as f32);
    app.set_duration_text(format_time(duration).into());
    app.set_current_index(match index {
        Some(position) => i32::try_from(position).unwrap_or(-1),
        None => -1,
    });
}

/// What:
/// `follow_playing` moves the selected folder to the folder that holds the playing track.
/// Why:
/// the list keeps the playing track visible when playback moves to another folder.
fn follow_playing(index: Option<usize>) {
    let Some(queue_index) = index else {
        return;
    };
    {
        let mut state = write_screen();
        let page = state.page_key_of_index(queue_index);
        if let Some(page_key) = page {
            state.selected_key = page_key;
        }
    }
}

/// What:
/// `apply_scroll_request` moves the track list for one pending request by writing its offset and bumping the tick.
/// Why:
/// the offset is written before the tick,
/// so the list reads the new offset when the tick changes;
/// without a
/// request the list keeps its scroll position.
fn apply_scroll_request(app: &AppWindow, pending: Option<ScrollRequest>, rows: &[crate::ui_screen_rows::ListRow]) {
    let Some(request) = pending else {
        return;
    };
    app.set_scroll_offset(scroll_offset_for(request, rows));
    app.set_scroll_tick(app.get_scroll_tick() + 1);
}

/// What:
/// `project` rebuilds the window's view models from the state and writes them to the window.
/// Why:
/// every list,
/// picker and deck value comes from one projection,
/// so the window never mixes stale models.
pub(crate) fn project(app: &AppWindow) {
    let started = std::time::Instant::now();
    let (screen, request): (ScreenView, Option<ScrollRequest>) = {
        let mut state = write_screen();
        // The template text is copied so the supporting line does not keep `state` borrowed while the
        // scroll decision below needs it mutably.
        let template = match state.track_rows_template.clone() {
            Some(text) => text,
            None => DEFAULT_TRACK_TEMPLATE.to_string(),
        };
        let screen = {
            let supporting_line = supporting_line_from_template(template.as_str());
            view(&state, &supporting_line)
        };
        let request = next_scroll_request(&mut state, &screen);
        (screen, request)
    };
    // With no folder page (reading, empty or without audio) the folded header would show only a caret (EM5): it shows
    // the library's name instead. The two-pane layout keeps its own heading, which names the library already.
    let header_title: String = if screen.folder_title.is_empty() { read_screen().library_heading() } else { screen.folder_title.clone() };
    app.set_folded_title(header_title.as_str().into());
    app.set_folder_title(screen.folder_title.as_str().into());
    let view_ms = started.elapsed().as_millis();
    app.set_track_rows(model_of(screen.rows.iter().map(track_row).collect()));
    crate::ui_lists::publish_track_layout(app, &screen.rows);
    apply_scroll_request(app, request, &screen.rows);
    let mut rail_cells: Vec<FolderRailCell> = Vec::with_capacity(screen.picker.cells.len());
    for cell in &screen.picker.cells {
        rail_cells.push(FolderRailCell {
            key: cell.key.as_str().into(),
            label: cell.label.as_str().into(),
            section_start: cell.section_start,
        });
    }
    app.set_rail_cells(model_of(rail_cells));
    let mut folder_names: Vec<FolderName> = Vec::with_capacity(screen.picker.names.len());
    for row in &screen.picker.names {
        folder_names.push(FolderName {
            name: row.name.as_str().into(),
            section_start: row.section_start,
            heading: row.heading.as_str().into(),
        });
    }
    app.set_folder_names(model_of(folder_names));
    app.set_current_folder(screen.folder_title.as_str().into());
    app.set_current_cell(screen.picker.current_cell.as_str().into());
    {
        let mut state = write_screen();
        state.shown_rows = screen.rows.clone();
        state.shown_names = screen.picker.names.clone();
        state.shown_folder = screen.folder_title.clone();
    }
    crate::ui_lists::sync_rings(app);
    app.set_library_name(read_screen().library_heading().into());
    app.set_deck_title(screen.deck_title.as_str().into());
    app.set_deck_folder(screen.deck_folder.as_str().into());
    app.set_deck_subtitle(screen.deck_subtitle.as_str().into());
    app.set_shuffle_folder_label(screen.shuffle_folder_label.as_str().into());
    // D1 shortens a long name in the middle; the deck draws the head with an ellipsis and then the last three characters.
    let (shuffle_head, shuffle_tail) = music_player::non_spacing_mark::split_tail_clusters(&screen.shuffle_folder_label, 3);
    app.set_shuffle_folder_head(shuffle_head.as_str().into());
    app.set_shuffle_folder_tail(shuffle_tail.as_str().into());
    app.global::<DeckState>().set_disabled(read_state(ScreenState::deck_disabled));
    app.set_picker_open(read_screen().picker_open);
    app.set_route(read_screen().route.to_int());
    crate::ui_overlays::project(app);
    crate::ui_settings::project_pages(app);
    tracing::debug!(rows = screen.rows.len(), view_ms, total_ms = started.elapsed().as_millis(), "the library screen was projected");
}

/// What:
/// `track_row` converts one plain list row into the window's `TrackListRow`.
/// Why:
/// the window struct carries Slint strings,
/// which the pure row model does not.
fn track_row(row: &crate::ui_screen_rows::ListRow) -> TrackListRow {
    return TrackListRow {
        kind: row.kind,
        title: SharedString::from(row.title.as_str()),
        supporting: SharedString::from(row.supporting.as_str()),
        current: row.current,
        index: row.index,
    };
}

/// What:
/// `model_of` wraps a vector in a shared Slint model.
/// Why:
/// Slint list properties take `ModelRc` values,
/// and the rows are rebuilt on every projection.
pub(crate) fn model_of<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    return Rc::new(VecModel::from(items)).into();
}

/// What:
/// `screen_cell` returns this thread's screen state cell.
/// Why:
/// the cell is leaked once per thread,
/// so it outlives every borrow,
/// so the guards of `read_screen` and `write_screen` need no closure to stay inside the borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const screenCell = (): Cell<ScreenState> => screen;
/// ```
fn screen_cell() -> &'static RefCell<ScreenState> {
    return SCREEN.with(Clone::clone);
}

/// What:
/// `read_screen` borrows the UI-thread state for reading until the guard is dropped.
/// Why:
/// callers read a field or two without wrapping the read in a closure;
/// a guard held across a write would panic,
/// so each guard lives only as long as the statement or block that needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = readScreen(); // a read-only view until it goes out of scope
/// ```
pub(crate) fn read_screen() -> Ref<'static, ScreenState> {
    return screen_cell().borrow();
}

/// What:
/// `write_screen` borrows the UI-thread state for writing until the guard is dropped.
/// Why:
/// all writes go through this one function,
/// so the borrow stays short:
/// each guard lives only in the block that changes the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = writeScreen(); // the one writer until it goes out of scope
/// ```
pub(crate) fn write_screen() -> RefMut<'static, ScreenState> {
    return screen_cell().borrow_mut();
}

/// What:
/// `read_state` runs a read-only closure over the UI-thread state.
/// Why:
/// callers read the state without taking ownership of it.
pub(crate) fn read_state<T>(read: impl FnOnce(&ScreenState) -> T) -> T {
    return read(&read_screen());
}

/// What:
/// `update_state` runs a mutating closure over the UI-thread state.
/// Why:
/// the window tests fill the state through it;
/// production code borrows `write_screen` for the same short span.
#[cfg(test)]
pub(crate) fn update_state(update: impl FnOnce(&mut ScreenState)) {
    update(&mut write_screen());
}
