//! The session and reconciliation half of `Controller` (a second `impl` block on
//! the type defined in `controller.rs`).
//!  It also holds the staged-track record,
//!  which a gapless handoff uses,
//!  and the listener that announces a running folder walk.
//!  Split out so each file stays within the
//! line budget.
//!  The methods that install a Source Root are in
//!  `src/controller_audio_source_installation.rs`,
//!  and the ones that pump samples into the output in
//!  `src/controller_audio_playback.rs`.

/// What:
///      `use std::path::PathBuf;`.
///  The owned filesystem-path type.
/// Why:
///       A staged track owns its path,
///  so the record can outlive the folder walk that found it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is just a string in TS
/// ```
use std::path::PathBuf;

/// What:
///      `use std::path::Path;` brings the borrowed filesystem path type.
/// Why:
///      Only `controller_position_tests.rs` names it,
///  through its `use super::*`,
///  so the import exists in the test build alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is just a string in TS
/// ```
#[cfg(test)]
use std::path::Path;

/// What:
///      `use crate::command::Update;`.
///  The engine->UI message enum.
/// Why:
///       These methods emit `NowPlaying`/`Position` updates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Update } from "./command";
/// ```
use crate::command::Update;

/// What:
///      `use crate::controller::Controller;`.
///  The state struct from the sibling
///           module;
///  this file adds a second `impl Controller` block.
/// Why:
///       Name the type we are implementing methods on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Controller } from "./controller";
/// ```
use crate::controller::Controller;

/// What:
///      `use crate::decode::AudioSpec;`.
///  The rate,
///           channels and length a decoder reports.
/// Why:
///       A staged track keeps the format it was opened with,
///  so the handoff can compare it with the audible one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioSpec } from "./decode";
/// ```
use crate::decode::AudioSpec;

/// What:
///      `use crate::session::Session;`.
///  The serializable saved-state record.
/// Why:
///       `current_session` builds one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Session } from "./session";
/// ```
use crate::session::Session;

/// What:
///      `pub(crate) struct StagedTrack { ... }`.
///  A track opened early so it can follow the audible one in the same ring buffer.
///  It owns its path,
/// so it is not `Copy`.
/// Why:
///       When the decoder reaches the end of a track and the next track has the same sample rate and channel
///  count,
/// the controller keeps one output stream and keeps pushing:
/// the next track's first frame follows the
///  last frame of the old one with nothing between them.
///  The queue and the UI keep naming the old track until the output consumes its last frame;
///  this record is what the controller needs to announce the next track at that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedTrack = { path: string; spec: AudioSpec; previousSpec: AudioSpec };
/// ```
pub(crate) struct StagedTrack {
    /// What:
    ///      `path: PathBuf`.
    ///  The file the decoder moved on to.
    /// Why:
    ///       The queue's own next track must still be this file when the handoff happens;
    ///  if the queue changed meanwhile,
    /// the controller reloads instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// path: string;
    /// ```
    pub(crate) path: PathBuf,
    /// What:
    ///      `spec: AudioSpec`.
    ///  The staged track's rate,
    /// channels and length.
    /// Why:
    ///       Announced to the UI (seek-bar length) when the track becomes audible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec: AudioSpec;
    /// ```
    pub(crate) spec: AudioSpec,
    /// What:
    ///      `previous_spec: AudioSpec`.
    ///  The still-audible track's format.
    /// Why:
    ///       A seek in the last second applies to the audible track,
    /// so the controller goes back to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// previousSpec: AudioSpec;
    /// ```
    pub(crate) previous_spec: AudioSpec,
}

/// What:
///      `struct ScanBatchAnnouncer { controller, root, next_index, pause }`.
///  The listener that tells the window about each batch of tracks the folder walk finds.
/// Why:
///       Search works while the library is still being scanned (D111),
/// so each batch is announced with its queue position as the walk goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScanBatchAnnouncer implements BatchListener { onBatch(batch) { emit({ firstIndex: this.nextIndex, names: batchNames(root, batch) }); this.nextIndex += batch.length; } }
/// ```
struct ScanBatchAnnouncer<'controller> {
    /// The controller that sends the updates.
    controller: &'controller Controller,
    /// The opened folder that the names are relative to.
    root: &'controller std::path::Path,
    /// The queue position of the next batch's first track.
    next_index: usize,
    /// The pause after each batch,
    /// which tests lengthen to imitate a slow scan.
    pause: std::time::Duration,
}

/// The announcer receives the batches of the walk.
impl crate::scan_stream::BatchListener for ScanBatchAnnouncer<'_> {
    /// What:
    /// `on_batch` announces one batch and advances the queue position.
    /// Why:
    /// the window and the Search index learn the new names while the walk goes on.
    fn on_batch(
        &mut self,
        batch: &[PathBuf],
    ) {
        self.controller.emit(Update::ScanBatch {
            first_index: self.next_index,
            names: crate::scan_stream::batch_names(self.root, batch),
        });
        self.next_index += batch.len();
        if !self.pause.is_zero() {
            std::thread::sleep(self.pause);
        }
    }
}

/// What:
///      `impl Controller { ... }`.
///  The loading/audio half of the behaviour (a SECOND
///           inherent `impl` block for `Controller`,
///  whose other half is in
///           `controller.rs`).
/// Why:
///       Keep these methods beside the command-handling half without one huge file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Controller { /* current_session, save_session, load_current, install_source, seek, pump_audio, on_track_end, tick_playback, stage_next_track, cross_track_boundary */ }
/// ```
impl Controller {
    /// What:
    ///      `fn current_session(&self) -> Session`.
    ///  Snapshot the playback state into a
    ///           serializable `Session`.
    /// Why:
    ///       Persist where the user left off.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// currentSession(): Session { ... }
    /// ```
    fn current_session(&self) -> Session {
        // What:     `Session::load()` reads the saved template texts and the Auto-play on select setting from the
        //           existing session before the playback snapshot replaces it.
        // Why:      They belong to the UI rather than the audio controller, but controller shutdown must preserve the
        //           independently saved values.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.sessionOver(Session.load());
        // ```
        return self.session_over(Session::load());
    }

    /// What:
    /// `session_over` snapshots the playback state on top of the `stored` session.
    /// Why:
    /// the fields that belong to the UI (the two templates,
    /// the Auto-play on select setting and the text sizes) come from `stored`,
    /// so a save at shutdown cannot erase them;
    /// taking `stored` as an argument lets a test prove that without the
    /// user's session file.
    fn session_over(
        &self,
        stored: Session,
    ) -> Session {
        let position_secs = self.audible_secs();
        // What:     `Session { ... }`. Build the record from the Source Root + state.
        //           `self.source_root.clone()` copies the owned root option;
        //           `self.queue.current_path().cloned()` is the Selected Track's path (or
        //           `None`). The queue itself is NOT stored. Tail -> return.
        // Why:      Bundle only what the next launch needs to re-derive the queue: the root,
        //           the cued track, and the settings.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { sourceRoot: this.sourceRoot, selected: this.queue.currentPath() ?? null,
        //          positionSecs, volume: this.volume, shuffle: this.queue.shuffleMode(),
        //          repeatTrack: this.queue.repeatTrack() };
        // ```
        return Session {
            source_root: self.source_root.clone(),
            selected: self.queue.current_path().cloned(),
            position_secs,
            volume: self.volume,
            playback_mode: self.queue.playback_mode(),
            track_rows_template: stored.track_rows_template,
            playing_template: stored.playing_template,
            auto_play_on_select: stored.auto_play_on_select,
            text_sizes: stored.text_sizes,
        };
    }

    /// What:
    ///      `pub(crate) fn save_session(&self)`.
    ///  Write the current session to disk,
    ///           logging (not propagating) any IO error.
    ///  `pub(crate)` so `engine::run` can
    ///           call it on quit.
    /// Why:
    ///       A failed save should not block shutdown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// saveSession(): void { try { this.currentSession().save(); } catch (e) { console.error(e); } }
    /// ```
    pub(crate) fn save_session(&self) {
        // What:     `if let Err(e) = self.current_session().save() { ... }`. `save` returns
        //           `io::Result<()>`; the `if let Err(e)` runs the body only on the error
        //           variant, binding the error to `e`.
        // Why:      Best-effort persistence.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { this.currentSession().save(); } catch (e) { console.error(e); }
        // ```
        if let Err(e) = self.current_session().save() {
            tracing::warn!(error = %e, "session save failed");
        }
    }

    /// What:
    ///      `pub(crate) fn emit_reconciled(&self)`.
    ///  Emit one `Update::Reconciled` carrying
    ///           the current queue (display paths) PLUS the re-anchored now-playing view (the
    ///           possibly-shifted index,
    ///  its display name,
    ///  and the loaded duration).
    ///  `pub(crate)`
    ///           so the `Rescan` handler in `controller.rs` can call it.
    /// Why:
    ///       A live rescan must refresh the list and highlight together WITHOUT moving the
    ///           user's selected tab (the UI keeps its current page for `Reconciled`,
    ///  unlike a
    ///           `Queue`/`NowPlaying` pair which would reset/follow the page).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emitReconciled(): void {
    ///   const names = this.queue.displayPaths();
    ///   const index = this.queue.currentIndex();
    ///   const name = index != null ? names[index] ?? "" : "";
    ///   const duration = index != null ? this.spec?.durationSecs ?? 0 : 0;
    ///   this.emit({ kind: "reconciled", names, index, name, duration });
    /// }
    /// ```
    pub(crate) fn emit_reconciled(&self) {
        // What:     `let names = self.queue.display_paths();`. The reconciled queue as display
        //           paths, computed once and moved into the update below.
        // Why:      The list itself may have gained or lost rows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const names = this.queue.displayPaths();
        // ```
        let names = self.queue.display_paths();
        // What:     `let index = self.queue.current_index();`. The current track's load-order
        //           position (`Option<usize>`), which may have shifted after the rescan.
        // Why:      Drives the row highlight.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = this.queue.currentIndex();
        // ```
        let index = self.queue.current_index();
        // What:     `let name = match index { Some(position) => names.get(position).cloned().unwrap_or_default(),
        //           None => String::new() };` is the display path at that index (cloned from `names`), or an empty
        //           string when no track stays selected.
        // Why:      The window title and label use this same relative-path string.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = index != null ? names[index] ?? "" : "";
        // ```
        let name = match index {
            Some(position) => names.get(position).cloned().unwrap_or_default(),
            None => String::new(),
        };
        // What:     `let duration = if index.is_some() { self.spec...duration_secs } else { 0.0 };`.
        //           The loaded track's duration when a track stays selected, else 0.0.
        // Why:      A cleared selection (file left the root) has no duration; otherwise the
        //           seek-bar maximum is unchanged (same file).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const duration = index != null ? this.spec?.durationSecs ?? 0 : 0;
        // ```
        let duration = if index.is_some() {
            match self.audible_spec() {
                Some(spec) => spec.duration_secs,
                None => 0.0,
            }
        } else {
            0.0
        };
        // What:     `self.emit(Update::Reconciled { names, index, name, duration });`. Push the
        //           combined list-and-highlight refresh.
        // Why:      One update so the UI keeps the page while updating both the rows and the
        //           highlighted track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "reconciled", names, index, name, duration });
        // ```
        self.emit(Update::Library(self.queue.tracks().to_vec()));
        self.emit(Update::Reconciled {
            names,
            index,
            name,
            duration,
        });
    }
}

/// What:
///      `mod controller_audio_source_installation;` loads the sibling
///           `src/controller_audio_source_installation.rs`,
///  the methods that install a Source Root and walk it into the queue.
/// Why:
///       The path from a folder on disk to a decoder keeps a file of its own,
///  and it is another `impl Controller` block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as controllerAudioSourceInstallation from "./controller_audio_source_installation";
/// ```
#[path = "controller_audio_source_installation.rs"]
mod controller_audio_source_installation;

/// What:
///      `mod controller_audio_playback;` loads the sibling
///           `src/controller_audio_playback.rs`,
///  the methods that announce the playing track and pump its samples.
/// Why:
///       The path from a decoder to the speaker keeps a file of its own,
///  and it is another `impl Controller` block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as controllerAudioPlayback from "./controller_audio_playback";
/// ```
#[path = "controller_audio_playback.rs"]
mod controller_audio_playback;

/// What:
///      `#[cfg(test)] #[path = "controller_position_tests.rs"] mod position_tests;`.
///  Pull in the controller-level tests of the audible position and the track-end handoff;
///  test builds only.
/// Why:
///       Keep these tests beside the code they drive without inflating production code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // controller_position_tests.rs is controller.position.test.ts beside controllerAudio.ts
/// ```
#[cfg(test)]
#[path = "controller_position_tests.rs"]
mod position_tests;
