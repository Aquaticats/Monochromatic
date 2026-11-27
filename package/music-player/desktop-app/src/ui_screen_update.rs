//! Applying the engine's messages to the state and the window.
//!
//! What:
//! `apply_update` is the one place an engine message changes the screen state,
//! `set_now_playing` records the playing track,
//! `follow_playing` moves the folder selection after it,
//! and `reset_position` clears the seek bar for a new track.
//! Why:
//! every queue,
//! playing and position change arrives here,
//! so the rules that follow one message are read together,
//! split from `src/ui_screen.rs` to stay within the line budget.

/// What:
///      `use music_player::command::Update` brings the message an engine update arrives
///           as.
/// Why:
///      The function matches on it,
///  and the enum is the library's only description of an update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Update } from "music_player/command";
/// ```
use music_player::command::Update;
/// What:
///      `use music_player::track_measure::apply_measures` brings the function that
///           writes reported measures into the state.
/// Why:
///      A measure message is applied by the library,
///  so this binding only forwards it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { apply_measures } from "music_player/track_measure";
/// ```
use music_player::track_measure::apply_measures;

/// What:
///      `use crate::ui_playback::playback_mode_to_int` brings the conversion of a
///           playback mode to the window's integer.
/// Why:
///      The window stores the mode as an integer,
///  and the library type is what an update carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { playback_mode_to_int } from "./ui_playback";
/// ```
use crate::ui_playback::playback_mode_to_int;

/// What:
///      `use crate::{...}` brings the window and the time formatter the update writes
///           through.
/// Why:
///      Every arm of the match ends in a window property,
///  and the duration text is formatted by one shared function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AppWindow, format_time } from "./main";
/// ```
use crate::{
    AppWindow,
    format_time,
};

/// What:
///      `use super::{project, write_screen}` brings the projection and the state writer.
/// Why:
///      An update writes the state first and then projects it,
///  and both live in the parent module and its projection child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { project, write_screen } from "./ui_screen";
/// ```
use super::{
    project,
    write_screen,
};

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
pub(crate) fn apply_update(
    app: &AppWindow,
    update: &Update,
) {
    match update {
        Update::Library(paths) => {
            let mut state = write_screen();
            state.paths = paths.clone();
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
            tracing::info!(
                names = library.len(),
                "queue applied; the library rows are projected next"
            );
            crate::ui_search::feed_library_names(library);
            project(app);
            tracing::info!("queue projected");
        }
        Update::NowPlaying {
            index,
            name,
            duration,
        } => {
            set_now_playing(app, *index, name, *duration);
            follow_playing(*index);
            project(app);
            crate::ui_state_log::emit(app, "now-playing");
        }
        Update::Reconciled {
            names,
            index,
            name,
            duration,
        } => {
            tracing::info!(
                rows = names.len(),
                "the library list was reconciled in the window"
            );
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
        Update::Analysis {
            ..
        } => {}
        Update::SourceRoot(name) => {
            {
                let mut state = write_screen();
                state.set_source_root_name(name.clone());
            }
            project(app);
        }
        // The Search index takes scan batches on the engine thread (main.rs); the window has nothing to draw for them.
        Update::ScanBatch {
            ..
        } => {}
        Update::Reading {
            name,
        } => {
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
            tracing::warn!(
                files = paths.len(),
                "files that cannot be played are marked on their rows"
            );
            {
                let mut state = write_screen();
                state.unplayable.extend(paths.iter().cloned());
            }
            project(app);
        }
        Update::AudioUnavailable {
            reason,
        } => {
            tracing::warn!(%reason, "no audio output is available; the window tells the person");
            crate::ui_overlays::show_notice(app, crate::ui_notices::audio_unavailable_message());
        }
        Update::FileGone {
            path,
            was_playing,
        } => {
            tracing::info!(path = %path.display(), was_playing, "the selected track's file left the folder");
            if crate::ui_files::was_just_trashed(path) {
                tracing::debug!(path = %path.display(), "the trash message already explains the missing file");
            } else {
                crate::ui_overlays::show_notice(
                    app,
                    &crate::ui_notices::file_gone_message(path, *was_playing),
                );
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
pub(crate) fn set_now_playing(
    app: &AppWindow,
    index: Option<usize>,
    name: &str,
    duration: f64,
) {
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
