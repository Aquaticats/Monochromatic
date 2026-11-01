//! Controller side of the playback flow around the edges: starting from nothing, stopping at the end of a folder, and
//! reporting files that could not be played.
//!
//! What: `impl Controller` methods for Play with nothing selected, the stop that In order makes at the end of its
//! folder, and the list of files that failed to open.
//! Why: the polish review found Play flipping to Pause with nothing playing (EM2), In order looping forever (PB4) and
//! unreadable files skipped without a word (PB1). Each is a small decision the controller owns, kept apart from the
//! large command handler.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Controller { startPlaying() {} togglePlaying() {} stopAtScopeEnd() {} reportUnplayable(paths) {} }
//! ```

use std::path::PathBuf;

use crate::command::Update;
use crate::controller::Controller;

impl Controller {
    /// What:
    ///      `pub(crate) fn start_playing(&mut self)`.
    ///  Play: resume the loaded track, or, with nothing loaded, select the first track of the displayed page, load it
    ///  and play it. An empty queue stays paused.
    /// Why:
    ///       With a library open and no track selected the Play button used to flip to Pause while nothing played
    ///  (`playing` true, no current track, length zero). The button now starts the folder from its first track, and
    ///  tells the window the truth when there is nothing to play.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// startPlaying(): void {
    ///   if (this.source) return this.setPlaying(true);
    ///   const first = this.queue.currentIndex() ?? this.queue.firstInScope();
    ///   if (first === null) return this.setPlaying(false);
    ///   this.queue.playIndex(first); this.setPlaying(this.loadCurrent());
    /// }
    /// ```
    pub(crate) fn start_playing(&mut self) {
        if self.source.is_some() {
            self.set_playing(true);
            return;
        }
        let selected: Option<usize> = self.queue.current_index();
        let first: Option<usize> = selected.or_else(|| return self.queue.first_in_scope());
        let Some(index) = first else {
            tracing::info!("Play was asked with nothing queued; staying paused");
            self.set_playing(false);
            return;
        };
        // D128: a selected row that is not loaded (restored, or its open failed) is tried again where it stands; only
        // a Play with nothing selected walks past unreadable files to the first playable one.
        let loaded: bool = if selected.is_some() {
            tracing::info!(index, "Play was asked with a selected row that is not loaded; loading that row");
            self.load_selected()
        } else {
            tracing::info!(index, "Play was asked with nothing loaded; starting the first track of the page");
            self.queue.play_index(index);
            self.load_current()
        };
        self.set_playing(loaded);
    }

    /// What:
    ///      `pub(crate) fn load_selected(&mut self) -> bool`.
    ///  Open the selected row's file and nothing else: on success install it paused at its start; on failure keep the
    ///  row selected with nothing loaded, report the file, and tell the window the row is current with no length.
    /// Why:
    ///       D128: a click, Enter or Search activation chooses a row, so an unreadable row stays the current row (as on
    ///  Android) and the next click retries it. Playback-driven moves (Next, Previous, the end of a track) keep using
    ///  `load_current`, which walks past unreadable files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// loadSelected(): boolean {
    ///   const path = this.queue.currentPath(); if (!path) return false;
    ///   try { this.installSource(open(path), path); return true; }
    ///   catch { this.unloadTrack(); this.reportUnplayable([path]); this.emitNowPlaying(path, 0); return false; }
    /// }
    /// ```
    pub(crate) fn load_selected(&mut self) -> bool {
        let Some(path) = self.queue.current_path().cloned() else {
            tracing::info!("a row was selected but the queue has no current path");
            return false;
        };
        match crate::decode::open(&path) {
            Ok(source) => {
                self.install_source(source, &path);
                return true;
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), error = %error, "cannot open the selected file; the row stays selected");
                self.unload_track();
                self.report_unplayable(vec![path.clone()]);
                self.emit_now_playing(&path, 0.0);
                return false;
            }
        }
    }

    /// What:
    ///      `fn unload_track(&mut self)`.
    ///  Drop the decoder and everything tied to it: the track queued behind it, the samples waiting to be written, the
    ///  file fingerprint, the pending peak measurement and the clock.
    /// Why:
    ///       D128: after a failed open of the selected row nothing may stay loaded, or the next toggle would resume the
    ///  previous track under the new row instead of trying to open the new row's file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// unloadTrack(): void { this.source = null; this.spec = null; this.staged = null; this.pending = []; this.loadedFile = null; }
    /// ```
    fn unload_track(&mut self) {
        self.source = None;
        self.spec = None;
        self.staged = None;
        self.loaded_file = None;
        self.pending.clear();
        self.pending_pos = 0;
        self.pending_peak = None;
        self.peak_generation = self.peak_generation.wrapping_add(1);
        self.clock.restart(0);
    }

    /// What:
    ///      `pub(crate) fn toggle_playing(&mut self)`.
    ///  The play/pause button: pause when playing, otherwise `start_playing`.
    /// Why:
    ///       The same rule must hold for the button, the keyboard and the menu, so every toggle goes through here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// togglePlaying(): void { if (this.playing) this.setPlaying(false); else this.startPlaying(); }
    /// ```
    pub(crate) fn toggle_playing(&mut self) {
        if self.playing {
            self.set_playing(false);
            return;
        }
        self.start_playing();
    }

    /// What:
    ///      `pub(crate) fn stop_at_scope_end(&mut self)`.
    ///  Playback ran out of tracks: stop, put the cursor on the first track of the scope, and load it paused.
    /// Why:
    ///       D2: In order plays to the end of the folder and stops (PB4). The first track is loaded so the next Play
    ///  starts the folder over, as a disc player does; a decoder left at the end of the last file would end again at
    ///  once and Play would look broken.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stopAtScopeEnd(): void { this.setPlaying(false); if (this.queue.rewindScope() !== null) this.loadCurrent(); }
    /// ```
    pub(crate) fn stop_at_scope_end(&mut self) {
        tracing::info!("nothing follows in the scope; playback stops");
        self.set_playing(false);
        if self.queue.rewind_scope().is_none() {
            return;
        }
        if !self.load_current() {
            self.emit_no_track();
        }
    }

    /// What:
    ///      `pub(crate) fn report_unplayable(&self, failed: Vec<PathBuf>)`.
    ///  Tell the window which files could not be opened, if any, and has not been told about already.
    /// Why:
    ///       D83: a failure is explained, not silent (PB1). One update carries every file of one load, so the report is
    ///  one notification with a count (D9, D122) and not one per file; D113: each file is told once, however often it is tried.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reportUnplayable(failed: string[]): void { this.reportUnplayableOnce(failed); }
    /// ```
    pub(crate) fn report_unplayable(&self, failed: Vec<PathBuf>) {
        self.report_unplayable_once(failed);
    }

    /// What:
    ///      `pub(crate) fn emit_unplayable(&self, failed: Vec<PathBuf>)`.
    ///  Sends one update that names the files that could not be played.
    /// Why:
    ///       The window marks the rows and the desktop gets one notification for the whole list (D122); callers have already left out the files
    ///  the person was told about.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emitUnplayable(failed: string[]): void { if (failed.length) this.emit({ kind: "unplayable", paths: failed }); }
    /// ```
    pub(crate) fn emit_unplayable(&self, failed: Vec<PathBuf>) {
        if failed.is_empty() {
            return;
        }
        tracing::warn!(files = failed.len(), "files that could not be opened are reported to the window");
        self.emit(Update::Unplayable(failed));
    }
}

#[cfg(test)]
#[path = "controller_flow_tests.rs"]
mod tests;
