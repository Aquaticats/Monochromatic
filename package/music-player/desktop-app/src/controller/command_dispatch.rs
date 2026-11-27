//! The controller's command dispatch:
//!  every UI `Command`,
//!  applied to the state machine in one exhaustive `match`.
//!
//! What:
//! `handle_command` is the single entry for a command that arrives from the window,
//!  run on the engine's own thread.
//! Why:
//! the dispatch is one long decision table that reads on its own,
//!  so it lives beside the state it mutates rather than inside `controller.rs`;
//! it reaches the same `Controller` type
//! through `crate::controller`.

/// Imports the UI-to-engine message enum whose every variant this dispatch answers.
use crate::command::Command;
/// Imports the engine-to-UI message enum that the arms emit back to the window.
use crate::command::Update;

/// Imports the controller whose command handling this module extends.
use crate::controller::Controller;

/// What:
///      `impl Controller { ... }`.
///  The command half of the controller's behaviour.
/// Why:
///       Every window action lands here and becomes state changes,
///  loads,
///  or emits,
///  kept apart from the state the struct holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Controller { handleCommand(command: Command): void { /* one switch over every command */ } }
/// ```
impl Controller {
    /// What:
    ///      `pub(crate) fn handle_command(&mut self, command: Command)`.
    ///  Apply one UI
    ///           command.
    ///  `pub(crate)` so `engine::run` can call it.
    /// Why:
    ///       The core of UI control.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handleCommand(command: Command): void { ... }
    /// ```
    pub(crate) fn handle_command(
        &mut self,
        command: Command,
    ) {
        // What:     `match command { ... }`. Dispatch on the command variant (exhaustive
        //           over every `Command`).
        // Why:      Each command does a different thing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (command.kind) { ... }
        // ```
        match command {
            // What:     `Command::OpenPaths { paths, play } => { ... }`. STRUCT-variant
            //           pattern: destructure `paths` and `play`. Replace the queue with the
            //           given files/folders, load the first track, and play it only when
            //           `play` is true.
            // Why:      Opening replaces the queue; the launch auto-load loads paused.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "openPaths": { const { paths, play } = command; ... }
            // ```
            Command::OpenRoot {
                root,
                select,
                play,
            } => {
                // What:     `self.scan_root_into_queue(root);`. Remember `root` as the Source
                //           Root, re-point the watcher at it, and rebuild the queue by scanning
                //           it (defined in `controller_audio.rs`). Consumes the owned `root`.
                // Why:      Opening a folder replaces the queue with the scan of that root; this
                //           shared projection lives in one place so Restore stays identical.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.scanRootIntoQueue(root);
                // ```
                self.scan_root_into_queue(root);
                // What:     `self.emit(Update::Queue(self.queue.display_paths()));`. Send
                //           the relative-path list to the UI.
                // Why:      Render the queue list (grouped by folder / first letter).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "queue", names: this.queue.displayPaths() });
                // ```
                self.emit(Update::Library(self.queue.tracks().to_vec()));
                self.emit(Update::Queue(self.queue.display_paths()));
                self.emit_source_root();
                // What:     `match select { Some(sel) => ..., None => ... }`. A single-file
                //           launch carries `Some(file)` to preselect; a folder open or
                //           auto-load carries `None`.
                // Why:      Preselect the named file inside its folder, else fall back to the
                //           normal open behavior (select nothing unless `--start-playing`).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (select) { /* preselect */ } else { /* normal open */ }
                // ```
                match select {
                    // What:     `Some(sel) => match self.queue.position_of_path(&sel) { ... }`. Find `sel` in the
                    //           scanned queue and select it; `position_of_path` returns the matching index or
                    //           `None`.
                    // Why:      Cue the file the launch named, loaded paused unless `play`.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const idx = tracks.indexOf(sel);
                    // ```
                    Some(sel) => match self.queue.position_of_path(&sel) {
                        // What:     `Some(idx) => { ... }`. The file is in the scan: select,
                        //           load, and play only when `play` AND the load succeeded.
                        // Why:      A preselected file is cued; `--start-playing` plays it.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // this.queue.playIndex(idx); const ok = this.loadCurrent(); this.setPlaying(play && ok);
                        // ```
                        Some(idx) => {
                            self.queue.play_index(idx);
                            let ok = self.load_current();
                            self.set_playing(play && ok);
                        }
                        // What:     `None => { ... }`. The named file is not in the scan (e.g.
                        //           it vanished): open with nothing selected, paused.
                        // Why:      Match the "selected track missing" rule: clear the cue.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // this.queue.clearSelection(); this.emitNoTrack(); this.setPlaying(false);
                        // ```
                        None => {
                            self.queue.clear_selection();
                            self.emit_no_track();
                            self.set_playing(false);
                        }
                    },
                    // What:     `None => { ... }`. No preselect. Only a `--start-playing`
                    //           launch with a non-empty queue auto-plays the anchored first
                    //           track; every other open selects and loads NOTHING.
                    // Why:      A normal open must not auto-select or blast audio.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (play && this.queue.currentPath()) { ... } else { ... }
                    // ```
                    None => {
                        if play && self.queue.current_path().is_some() {
                            let ok = self.load_current();
                            self.set_playing(ok);
                        } else {
                            self.queue.clear_selection();
                            self.emit_no_track();
                            self.set_playing(false);
                        }
                    }
                }
                // What:     `self.start_queue_measurement();`. Pre-measure the whole queue in the background (true-peak
                //           normalization cache), AFTER the open has decided which track is current.
                // Why:      Every queue load warms the peak cache, and the sweep skips the current track because the
                //           playback path measures it. `set_tracks` leaves the cursor on the first track until the
                //           branch above clears it, so a sweep started earlier skipped the first track of every
                //           plain open and its row never got a true peak until it played (LS9).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.startQueueMeasurement(); // after the selection is settled
                // ```
                self.start_queue_measurement();
            }
            // What:     `Command::TogglePlay => self.toggle_playing()`. Pause when playing, otherwise start (see
            //           `controller_flow.rs`).
            // Why:      The play/pause button; with nothing loaded it starts the first track instead of flipping to Pause.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "togglePlay": this.setPlaying(!this.playing); break;
            // ```
            Command::TogglePlay => self.toggle_playing(),
            // What:     `Command::Play => self.start_playing()`. Explicit play.
            // Why:      Explicit play command; with nothing loaded it starts the first track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "play": this.setPlaying(true); break;
            // ```
            Command::Play => self.start_playing(),
            // What:     `Command::Pause => self.set_playing(false)`. Explicit pause.
            // Why:      Explicit pause command.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "pause": this.setPlaying(false); break;
            // ```
            Command::Pause => self.set_playing(false),
            // What:     `Command::Next => { ... }`. Advance (not a natural end) and load.
            // Why:      Next button.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "next": { const moved = this.queue.advance(false); this.afterMove(moved); break; }
            // ```
            Command::Next => {
                // What:     `let moved = self.queue.advance(false);`. Step forward. `false`
                //           means "not a natural track end".
                // Why:      Decide whether to load or stop.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const moved = this.queue.advance(false);
                // ```
                let moved = self.queue.advance(false);
                // What:     `self.after_move(moved);`. Load the new current or stop.
                // Why:      Shared follow-up.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.afterMove(moved);
                // ```
                self.after_move(moved);
            }
            // What:     `Command::Prev => { ... }`. Step backward and load.
            // Why:      Previous button.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "prev": { const moved = this.queue.prev(); this.afterMove(moved); break; }
            // ```
            Command::Prev => {
                // What:     `let moved = self.queue.prev();`. Step back.
                // Why:      Get the previous index, if any.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const moved = this.queue.prev();
                // ```
                let moved = self.queue.prev();
                // What:     `self.after_move(moved);`. Load or stop.
                // Why:      Shared follow-up.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.afterMove(moved);
                // ```
                self.after_move(moved);
            }
            // What:     `Command::SelectIndex(index) => { ... }`. Tuple-variant pattern
            //           binding `index`. Make a queue slot current and load it PAUSED,
            //           without starting playback.
            // Why:      A single click selects (and pauses); only a click on the
            //           already-selected row plays it (via a follow-up `TogglePlay`).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "selectIndex": { const { index } = command; if (this.queue.playIndex(index) != null) { ... } break; }
            // ```
            Command::SelectIndex(index) => {
                // What:     `if self.queue.play_index(index).is_some() { ... }`. Act only on
                //           a valid index. (`Queue::play_index` only moves the cursor and
                //           rebuilds the page scope; it does not start audio.)
                // Why:      Ignore out-of-range clicks.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (this.queue.playIndex(index) != null) { ... }
                // ```
                if self.queue.play_index(index).is_some() {
                    // What:     `self.load_selected();`. Load the chosen track, and only that track, so its
                    //           name/duration show and its row highlights. The bool result
                    //           is unused: we pause either way.
                    // Why:      Make the track ready and current, without playing it. D128: an unreadable
                    //           file keeps its row selected (reported, nothing loaded); `load_current`
                    //           would have moved the selection to the next playable file.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.loadSelected();
                    // ```
                    self.load_selected();
                    // What:     `self.set_playing(false);`. Force PAUSED. If audio was
                    //           playing (this or another track), it stops here.
                    // Why:      "Single click merely selects (if currently playing, set to
                    //           paused)" — selecting never auto-plays.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.setPlaying(false);
                    // ```
                    self.set_playing(false);
                }
            }
            // What:     `Command::Seek(secs) => self.seek(secs)`. Tuple-variant binding
            //           `secs`; jump within the track.
            // Why:      Seek bar drag.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "seek": this.seek(command.secs); break;
            // ```
            Command::Seek(secs) => self.seek(secs),
            // What:     `Command::SetVolume(v) => { ... }`. Update the gain and mirror it.
            // Why:      Volume slider.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "setVolume": { this.volume = command.volume; this.emit({ kind: "volume", v: command.volume }); break; }
            // ```
            Command::SetVolume(v) => {
                // What:     `self.volume = v;`. Store the new gain.
                // Why:      Applied to subsequently decoded samples.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.volume = v;
                // ```
                self.volume = v;
                // What:     `self.emit(Update::Volume(v));`. Mirror to the UI.
                // Why:      Keep the slider in sync.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "volume", v });
                // ```
                self.emit(Update::Volume(v));
            }
            // What:     `Command::SetPlaybackMode(mode) => { ... }`. Set the shuffle mode.
            // Why:      Shuffle radio group.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "setShuffle": { this.queue.setShuffle(command.mode); this.emit({ kind: "shuffle", mode: command.mode }); break; }
            // ```
            Command::SetPlaybackMode(mode) => {
                self.queue.set_playback_mode(mode);
                self.emit(Update::PlaybackMode(mode));
            }
            Command::SetPageScope(indices) => {
                self.queue.set_page_scope(indices);
            }
            // What:     `Command::Restore { tracks, current, position, volume, shuffle, repeat_track } => { ... }`.
            //           STRUCT-variant pattern destructuring all six saved fields. Reinstate
            //           a saved session, loading the current track PAUSED at the saved
            //           position.
            // Why:      Resume where the user left off, on launch.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "restore": { const { tracks, current, position, volume, shuffle, repeatTrack } = command; ... }
            // ```
            Command::Restore {
                root,
                selected,
                position,
                volume,
                playback_mode,
            } => {
                // What:     `self.volume = volume;`. Restore the saved gain.
                // Why:      Applied to decoded samples.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.volume = volume;
                // ```
                self.volume = volume;
                // What:     `self.scan_root_into_queue(root);`. Remember `root` as the Source
                //           Root, re-point the watcher at it, and rebuild the queue by SCANNING
                //           it fresh from disk (defined in `controller_audio.rs`), not from a
                //           saved track list. Consumes the owned `root`.
                // Why:      The queue is a projection of the Source Root; re-scanning is the
                //           restore auto-correction (added/removed/renamed files sort themselves
                //           out), and it is the very same projection a normal open performs.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.scanRootIntoQueue(root);
                // ```
                self.scan_root_into_queue(root);
                // Restore the single playback mode after the fresh queue scan.
                self.queue.set_playback_mode(playback_mode);
                // What:     `match selected { Some(sel) => ..., None => ... }`. Re-select the
                //           saved track BY PATH: `Some(sel)` looks it up in the fresh scan,
                //           `None` (nothing was cued) clears the anchored selection.
                // Why:      Resume on the saved track when the scan still contains it; if it
                //           moved or vanished, clear the cue (the "selected track missing"
                //           rule). Done before the background sweep so it skips the current.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const idx = selected ? tracks.indexOf(selected) : -1;
                // if (idx >= 0) this.queue.playIndex(idx); else this.queue.clearSelection();
                // ```
                let saved_position: Option<usize> = match selected {
                    Some(sel) => self.queue.position_of_path(&sel),
                    None => None,
                };
                match saved_position {
                    // What:     `Some(idx) => { self.queue.play_index(idx); }`. The saved track
                    //           is present: select it (rebuilding the scope around it).
                    // Why:      Resume where the user left off.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.queue.playIndex(idx);
                    // ```
                    Some(idx) => {
                        self.queue.play_index(idx);
                    }
                    // What:     `None => { self.queue.clear_selection(); }`. No saved track, or
                    //           it is absent from the fresh scan: deselect.
                    // Why:      A never-cued or moved-away selection reopens with nothing
                    //           selected, not auto-selecting the first track.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.queue.clearSelection();
                    // ```
                    None => {
                        self.queue.clear_selection();
                    }
                }
                // What:     `self.start_queue_measurement();`. Pre-measure the restored
                //           queue in the background, like any other queue load.
                // Why:      Warm the peak cache for restored tracks while leaving the
                //           current track to its dedicated swap measurement.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.startQueueMeasurement();
                // ```
                self.start_queue_measurement();
                // What:     `self.emit(Update::Queue(self.queue.display_paths()));`. Push the
                //           relative-path list to the UI.
                // Why:      Render the restored queue (grouped by folder / first letter).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "queue", names: this.queue.displayPaths() });
                // ```
                self.emit(Update::Library(self.queue.tracks().to_vec()));
                self.emit(Update::Queue(self.queue.display_paths()));
                self.emit_source_root();
                // What:     `self.emit(Update::Volume(volume));`. Mirror volume.
                // Why:      Sync the slider.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "volume", volume });
                // ```
                self.emit(Update::Volume(volume));
                // What:     `self.emit(Update::Shuffle(self.queue.playback_mode()));`. Mirror
                //           the shuffle mode.
                // Why:      Sync the radio group.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "shuffle", mode: this.queue.shuffleMode() });
                // ```
                self.emit(Update::PlaybackMode(self.queue.playback_mode()));
                // What:     `self.playing = false;`. Restore PAUSED.
                // Why:      Resuming should not blast audio on launch.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.playing = false;
                // ```
                self.playing = false;
                // What:     `let loaded = self.load_current();`. Load the current track.
                // Why:      Make it ready to play from the saved position.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const loaded = this.loadCurrent();
                // ```
                let loaded = self.load_current();
                // What:     `if !loaded { self.emit_no_track(); }`. When NO track loaded (the
                //           session had no current track, or every saved file was unreadable),
                //           push the cleared now-playing view. `!` negates the bool.
                // Why:      `load_current` emits `NowPlaying` only when it actually loads a
                //           track, so a no-current restore would otherwise leave a stale label
                //           and row highlight on screen.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (!loaded) this.emitNoTrack();
                // ```
                if !loaded {
                    self.emit_no_track();
                }
                // What:     `self.emit(Update::Playing(false));`. Mirror paused state.
                // Why:      Show the Play button.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "playing", on: false });
                // ```
                self.emit(Update::Playing(false));
                // What:     `if loaded && position > 0.0 { self.seek(position); }`. Jump to
                //           the saved position if a track loaded. `&&` short-circuits.
                // Why:      Resume mid-track.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (loaded && position > 0) this.seek(position);
                // ```
                if loaded && position > 0.0 {
                    self.seek(position);
                }
            }
            // What:     `Command::Rescan => self.rescan_everything()`. Walk the Source Root again and refresh every track.
            // Why:      The watcher sends it after lost events and after a registration; see `controller_refresh.rs`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "rescan": this.rescanEverything(); break;
            // ```
            Command::Rescan => self.rescan_everything(),
            // What:     `Command::SourceChanged(change) => self.handle_source_changed(change)`. Refresh what the watcher named.
            // Why:      D113: the cost of a refresh follows what changed, not the size of the library.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "sourceChanged": this.handleSourceChanged(command.change); break;
            // ```
            Command::SourceChanged(change) => self.handle_source_changed(change),
            // What:     `Command::Quit => {}`. Empty arm: handled in `run`'s drain loop;
            //           this keeps the match exhaustive.
            // Why:      Rust requires every variant to be matched.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "quit": break; // handled in run()'s drain loop
            // ```
            Command::SweepReport(report) => self.handle_sweep_report(report),
            // What:     `Command::NoLibrary => self.emit_source_root()`. Say that no folder is open.
            // Why:      The launch found neither a saved folder nor a music folder; the window then offers to open one.
            Command::NoLibrary => self.emit_source_root(),
            Command::Quit => {}
        }
    }
}
