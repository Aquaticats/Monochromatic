//! Window binding for the library screen: applies engine updates to the UI-thread state and pushes
//! the projected view into `AppWindow`.
//!
//! What: `apply_update` is the single entry for engine messages on the event-loop thread.
//! Why: the page rules and the row and picker models stay in the pure modules; this file only
//! converts them into Slint values and writes window properties.

use std::cell::RefCell;
use std::rc::Rc;

use music_player::command::Update;
use music_player::template_model::DEFAULT_TRACK_TEMPLATE;
use music_player::track_measure::apply_measures;
use music_player::track_supporting_line::supporting_line_from_template;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::ui_playback::playback_mode_to_int;
use crate::ui_screen_scroll::{ScrollRequest, next_scroll_request, scroll_offset_for};
use crate::ui_screen_view::{ScreenState, ScreenView, view};
use crate::{AppWindow, DeckState, FolderName, FolderRailCell, TrackListRow, format_time};

thread_local! {
    /// What: the UI thread's copy of the engine facts, read and written only on the event-loop thread.
    /// Why: `invoke_from_event_loop` runs every update on that thread, so no lock is needed.
    static SCREEN: RefCell<ScreenState> = RefCell::new(ScreenState::default());
}

/// What: `apply_update` applies one engine message to the state and the window.
/// Why: queue changes reset the folder selection, playing-track changes follow the playing folder,
/// and position and playing changes only touch their own window properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyUpdate(app: AppWindow, update: Update): void { ... }
/// ```
pub(crate) fn apply_update(app: &AppWindow, update: &Update) {
    match update {
        Update::Library(paths) => {
            update_state(|state| {
                state.paths = paths.clone();
            });
        }
        Update::Queue(names) => {
            let mut shared = None;
            update_state(|state| {
                shared = Some(state.set_names(names));
                state.selected_key = String::new();
                state.loaded = true;
                state.reading = false;
                state.unplayable.clear();
            });
            if let Some(library) = shared {
                tracing::info!(names = library.len(), "queue applied; the library rows are projected next");
                crate::ui_search::feed_library_names(library);
            }
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
            let mut shared = None;
            update_state(|state| {
                shared = Some(state.set_names_keeping_place(names));
                state.loaded = true;
                state.reading = false;
            });
            if let Some(library) = shared {
                crate::ui_search::feed_library_names(library);
            }
            set_now_playing(app, *index, name, *duration);
            if index.is_none() {
                reset_position(app);
            }
            project(app);
        }
        Update::Position(secs) => {
            update_state(|state| {
                state.position = *secs;
            });
            app.set_position(*secs as f32);
            app.set_position_text(format_time(*secs).into());
            crate::ui_state_log::emit(app, "position");
        }
        Update::Playing(on) => {
            update_state(|state| {
                state.playing = *on;
            });
            app.set_playing(*on);
            crate::ui_state_log::emit(app, "playing");
        }
        Update::Volume(volume) => app.set_volume(*volume),
        Update::Measures(measures) => {
            let mut shown_changed = false;
            update_state(|state| {
                apply_measures(&mut state.measures, measures);
                shown_changed = state.measures_touch_selected_page(measures);
                // A file that was marked unplayable and now has a measured peak was replaced by one that decodes (D113),
                // so its mark goes.
                for (path, measure) in measures {
                    if measure.peak_dbtp.is_some() && state.unplayable.remove(path) {
                        tracing::info!(path = %path.display(), "a file marked unplayable now decodes; its mark is removed");
                        shown_changed = true;
                    }
                }
            });
            if shown_changed {
                crate::ui_screen_throttle::project_soon(app);
            }
        }
        // The window draws no analysis indicator (D107); the system indicator takes this update in main.rs.
        Update::Analysis { .. } => {}
        Update::SourceRoot(name) => {
            update_state(|state| {
                state.set_source_root_name(name.clone());
            });
            project(app);
        }
        // The Search index takes scan batches on the engine thread (main.rs); the window has nothing to draw for them.
        Update::ScanBatch { .. } => {}
        Update::Reading { name } => {
            tracing::info!(name = %name, "the folder is being read");
            update_state(|state| {
                state.reading = true;
                state.set_source_root_name(Some(name.clone()));
            });
            project(app);
        }
        // The window marks the rows only (D122): the words about these files go to the desktop as a system notification,
        // which main.rs hands to the unplayable notifier on the engine thread, so no message is drawn here.
        Update::Unplayable(paths) => {
            tracing::warn!(files = paths.len(), "files that cannot be played are marked on their rows");
            update_state(|state| {
                state.unplayable.extend(paths.iter().cloned());
            });
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
            update_state(|state| {
                state.duration = *duration;
            });
            app.set_duration(*duration as f32);
            app.set_duration_text(format_time(*duration).into());
        }
        Update::PlaybackMode(mode) => {
            update_state(|state| {
                state.mode = *mode;
            });
            app.set_playback_mode(playback_mode_to_int(*mode));
            crate::ui_state_log::emit(app, "playback-mode");
        }
    }
}

/// What: `set_now_playing` records and shows the playing track's name, duration and queue index.
/// Why: the window title, the seek bar and the row highlight all read these values.
/// What: `reset_position` puts the seek bar and the elapsed label back to the start.
/// Why: when the playing file leaves the folder no `NowPlaying` follows, and the rate-limited `Position` reset can be
/// dropped, which left `0:02` next to `0:00` (the polish review finding PB2).
fn reset_position(app: &AppWindow) {
    update_state(|state| {
        state.position = 0.0;
    });
    app.set_position(0.0);
    app.set_position_text(format_time(0.0).into());
}

pub(crate) fn set_now_playing(app: &AppWindow, index: Option<usize>, name: &str, duration: f64) {
    update_state(|state| {
        state.track_name = name.to_string();
        state.duration = duration;
        state.current_index = index;
    });
    app.set_track_name(name.into());
    app.set_duration(duration as f32);
    app.set_duration_text(format_time(duration).into());
    app.set_current_index(index.map_or(-1, |i| return i32::try_from(i).unwrap_or(-1)));
}

/// What: `follow_playing` moves the selected folder to the folder that holds the playing track.
/// Why: the list keeps the playing track visible when playback moves to another folder.
fn follow_playing(index: Option<usize>) {
    let Some(queue_index) = index else {
        return;
    };
    update_state(|state| {
        let page = state.page_key_of_index(queue_index);
        if let Some(page_key) = page {
            state.selected_key = page_key;
        }
    });
}

/// What: `apply_scroll_request` moves the track list for one pending request by writing its offset and bumping the tick.
/// Why: the offset is written before the tick, so the list reads the new offset when the tick changes; without a
/// request the list keeps its scroll position.
fn apply_scroll_request(app: &AppWindow, pending: Option<ScrollRequest>, rows: &[crate::ui_screen_rows::ListRow]) {
    let Some(request) = pending else {
        return;
    };
    app.set_scroll_offset(scroll_offset_for(request, rows));
    app.set_scroll_tick(app.get_scroll_tick() + 1);
}

/// What: `project` rebuilds the window's view models from the state and writes them to the window.
/// Why: every list, picker and deck value comes from one projection, so the window never mixes stale models.
pub(crate) fn project(app: &AppWindow) {
    let started = std::time::Instant::now();
    let (screen, request): (ScreenView, Option<ScrollRequest>) = SCREEN.with(|cell| {
        let mut state = cell.borrow_mut();
        // The template text is copied so the supporting-line closure does not keep `state` borrowed while the
        // scroll decision below needs it mutably.
        let template = state.track_rows_template.clone().unwrap_or_else(|| return DEFAULT_TRACK_TEMPLATE.to_string());
        let screen = {
            let supporting_line = supporting_line_from_template(template.as_str());
            view(&state, &supporting_line)
        };
        let request = next_scroll_request(&mut state, &screen);
        return (screen, request);
    });
    // With no folder page (reading, empty or without audio) the folded header would show only a caret (EM5): it shows
    // the library's name instead. The two-pane layout keeps its own heading, which names the library already.
    let header_title: String = if screen.folder_title.is_empty() { read_state(|state| return state.library_heading()) } else { screen.folder_title.clone() };
    app.set_folded_title(header_title.as_str().into());
    app.set_folder_title(screen.folder_title.as_str().into());
    let view_ms = started.elapsed().as_millis();
    app.set_track_rows(model_of(screen.rows.iter().map(track_row).collect()));
    crate::ui_lists::publish_track_layout(app, &screen.rows);
    apply_scroll_request(app, request, &screen.rows);
    app.set_rail_cells(model_of(
        screen
            .picker
            .cells
            .iter()
            .map(|cell| return FolderRailCell {
                key: cell.key.as_str().into(),
                label: cell.label.as_str().into(),
                section_start: cell.section_start,
            })
            .collect(),
    ));
    app.set_folder_names(model_of(
        screen
            .picker
            .names
            .iter()
            .map(|row| return FolderName {
                name: row.name.as_str().into(),
                section_start: row.section_start,
                heading: row.heading.as_str().into(),
            })
            .collect(),
    ));
    app.set_current_folder(screen.folder_title.as_str().into());
    app.set_current_cell(screen.picker.current_cell.as_str().into());
    update_state(|state| {
        state.shown_rows = screen.rows.clone();
        state.shown_names = screen.picker.names.clone();
        state.shown_folder = screen.folder_title.clone();
    });
    crate::ui_lists::sync_rings(app);
    app.set_library_name(read_state(|state| return state.library_heading()).into());
    app.set_deck_title(screen.deck_title.as_str().into());
    app.set_deck_folder(screen.deck_folder.as_str().into());
    app.set_deck_subtitle(screen.deck_subtitle.as_str().into());
    app.set_shuffle_folder_label(screen.shuffle_folder_label.as_str().into());
    // D1 shortens a long name in the middle; the deck draws the head with an ellipsis and then the last three characters.
    let (shuffle_head, shuffle_tail) = music_player::non_spacing_mark::split_tail_clusters(&screen.shuffle_folder_label, 3);
    app.set_shuffle_folder_head(shuffle_head.as_str().into());
    app.set_shuffle_folder_tail(shuffle_tail.as_str().into());
    app.global::<DeckState>().set_disabled(read_state(ScreenState::deck_disabled));
    app.set_picker_open(read_state(|state| return state.picker_open));
    app.set_route(read_state(|state| return state.route.to_int()));
    crate::ui_overlays::project(app);
    crate::ui_settings::project_pages(app);
    tracing::debug!(rows = screen.rows.len(), view_ms, total_ms = started.elapsed().as_millis(), "the library screen was projected");
}

/// What: `track_row` converts one plain list row into the window's `TrackListRow`.
/// Why: the window struct carries Slint strings, which the pure row model does not.
fn track_row(row: &crate::ui_screen_rows::ListRow) -> TrackListRow {
    return TrackListRow {
        kind: row.kind,
        title: SharedString::from(row.title.as_str()),
        supporting: SharedString::from(row.supporting.as_str()),
        current: row.current,
        index: row.index,
    };
}

/// What: `model_of` wraps a vector in a shared Slint model.
/// Why: Slint list properties take `ModelRc` values, and the rows are rebuilt on every projection.
pub(crate) fn model_of<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    return Rc::new(VecModel::from(items)).into();
}

/// What: `read_state` runs a read-only closure over the UI-thread state.
/// Why: callers read the state without taking ownership of it.
pub(crate) fn read_state<T>(read: impl FnOnce(&ScreenState) -> T) -> T {
    return SCREEN.with(|cell| return read(&cell.borrow()));
}

/// What: `update_state` runs a mutating closure over the UI-thread state.
/// Why: all writes go through this one function, so the borrow stays short.
pub(crate) fn update_state(update: impl FnOnce(&mut ScreenState)) {
    SCREEN.with(|cell| {
        update(&mut cell.borrow_mut());
    });
}
