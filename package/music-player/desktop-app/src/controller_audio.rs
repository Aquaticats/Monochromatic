//! The loading and audio-pumping half of `Controller` (a second `impl` block on
//! the type defined in `controller.rs`).
//!  Split out so each file stays within the
//! line budget.
//!  These methods open decoders,
//!  prepare each track's true-peak swap
//! gain,
//!  push samples into the ring buffer,
//!  and report position.

/// What:
///      `use std::path::{Path, PathBuf};`.
///  Borrowed (`&Path`) and owned (`PathBuf`)
///           filesystem-path types.
/// Why:
///       `install_source` borrows a `&Path` to read the current file;
///           `scan_root_into_queue` takes an owned `PathBuf` root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is just a string in TS
/// ```
use std::path::{Path, PathBuf};

/// What:
///      `use std::time::{Duration, Instant};`.
///  A span of time and a monotonic instant.
/// Why:
///       The end-of-track hold waits a few milliseconds after the ring ran empty;
///  the clock is read here and handed to the pure `PlaybackClock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // numbers of milliseconds from performance.now()
/// ```
use std::time::{Duration, Instant};

/// What:
///      `use ringbuf::traits::{Observer, Producer};`.
///  Brings `push_slice` and `occupied_len` into scope for the
///           producer half of the ring buffer.
/// Why:
///       We push decoded samples into the buffer the output gave us,
///  and read how many of them the output has not taken yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: importing interfaces to unlock .pushSlice() and .length
/// ```
use ringbuf::traits::{Observer, Producer};

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
///      `use crate::decode::Source;`.
///  The decoder trait (so `Box<dyn Source>` is
///           nameable and its `spec`/`next_chunk`/`seek` methods are in scope).
/// Why:
///       `install_source` takes a `Box<dyn Source>` and we drive it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Source } from "./decode";
/// ```
use crate::decode::{AudioSpec, Source};

/// What:
///      `use crate::playback::{expand_paths, file_name_of, frames_to_secs, process_sample};`.
///           Folder-to-file expansion,
///  display-name extraction,
///           frame->seconds conversion,
///           and the per-sample gain+clamp stage.
/// Why:
///       Used by scan_root_into_queue,
///  install_source,
///           and pump_audio.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { expandPaths, fileNameOf, framesToSecs, processSample } from "./playback";
/// ```
use crate::playback::{expand_paths, file_name_of, frames_to_secs, process_sample};

/// What:
///      `use crate::playback_clock::frames_of_samples;`.
///  Interleaved samples to whole frames.
/// Why:
///       The pushed and queued counts are samples;
///  the clock counts frames.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { framesOfSamples } from "./playbackClock";
/// ```
use crate::playback_clock::frames_of_samples;

/// What:
///      `use crate::audio_meter::MeterReporter;`.
///  The once-a-second reporter for the output's underrun counters.
/// Why:
///       The engine loop owns one and passes it to `report_audio_meters`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MeterReporter } from "./audioMeter";
/// ```
use crate::audio_meter::MeterReporter;

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
///  It owns its path, so it is not `Copy`.
/// Why:
///       When the decoder reaches the end of a track and the next track has the same sample rate and channel
///  count, the controller keeps one output stream and keeps pushing: the next track's first frame follows the
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
    ///  if the queue changed meanwhile, the controller reloads instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// path: string;
    /// ```
    pub(crate) path: PathBuf,
    /// What:
    ///      `spec: AudioSpec`.
    ///  The staged track's rate, channels and length.
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
    ///       A seek in the last second applies to the audible track, so the controller goes back to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// previousSpec: AudioSpec;
    /// ```
    pub(crate) previous_spec: AudioSpec,
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

    /// What: `session_over` snapshots the playback state on top of the `stored` session.
    /// Why: the fields that belong to the UI (the two templates, the Auto-play on select setting and the text sizes) come from `stored`,
    /// so a save at shutdown cannot erase them; taking `stored` as an argument lets a test prove that without the
    /// user's session file.
    fn session_over(&self, stored: Session) -> Session {
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
        }
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
        // What:     `let name = index.and_then(|i| names.get(i).cloned()).unwrap_or_default();`.
        //           The display path at that index (borrowing from `names`), or `""`.
        // Why:      The window title and label use this same relative-path string.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = index != null ? names[index] ?? "" : "";
        // ```
        let name = index.and_then(|i| return names.get(i).cloned()).unwrap_or_default();
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
            self.audible_spec().map_or(0.0, |s| return s.duration_secs)
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

    /// What:
    ///      `pub(crate) fn rewatch_source_root(&mut self)`.
    ///  Point the file watcher at the
    ///           current Source Root (if both a root and a watcher exist).
    ///  `pub(crate)` so the
    ///           open/restore handlers in `controller.rs` can call it.
    /// Why:
    ///       Called after every open/restore so on-disk changes to the newly-loaded root
    ///           drive a `Rescan`;
    ///  a no-op in tests (no watcher) and when no root is set.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rewatchSourceRoot(): void { if (this.sourceRoot && this.watcher) this.watcher.watch(this.sourceRoot); }
    /// ```
    pub(crate) fn rewatch_source_root(&mut self) {
        // What:     `if let Some(root) = self.source_root.clone() { ... }`. Clone the root so
        //           the immutable borrow ends before the mutable watcher borrow.
        // Why:      Nothing to watch without a root.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.sourceRoot) { ... }
        // ```
        if let Some(root) = self.source_root.clone() {
            // What:     `if let Some(watcher) = self.watcher.as_mut() { watcher.watch(&root); }`.
            //           Mutably borrow the watcher and re-point it.
            // Why:      Only the running app has a watcher; tests skip this.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.watcher?.watch(root);
            // ```
            if let Some(watcher) = self.watcher.as_mut() {
                watcher.watch(&root);
            }
        }
    }

    /// What:
    ///      `pub(crate) fn scan_root_into_queue(&mut self, root: PathBuf)`.
    ///  Adopt `root`
    ///           as the Source Root:
    ///  remember it,
    ///  re-point the file watcher at it,
    ///  and rebuild
    ///           the queue by scanning it from disk.
    ///  Consumes the owned `root`.
    ///  `pub(crate)`
    ///           so the command-handling half can call it.
    /// Why:
    ///       "The Queue is the scan of the Source Root" (see CONTEXT.md).
    ///  Opening a folder
    ///           and restoring a session both start with this identical projection,
    ///  so it
    ///           lives in one place instead of being duplicated across the two command arms.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanRootIntoQueue(root: string): void {
    ///   this.sourceRoot = root;
    ///   this.rewatchSourceRoot();
    ///   this.queue.setTracks(expandPaths([root]));
    /// }
    /// ```
    pub(crate) fn scan_root_into_queue(&mut self, root: PathBuf) {
        // What:     `self.source_root = Some(root.clone());`. Remember the directory the queue
        //           is scanned from. `.clone()` because `root` is moved into `expand_paths`.
        // Why:      The session, the watcher, and any rescan need the root.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.sourceRoot = root;
        // ```
        // What:     `self.emit(Update::Reading { name });`. Tell the window the walk has started, before the walk blocks this
        //           thread.
        // Why:      Until the queue arrives the window used to say "No folder is open" (EM1); now it can say that the
        //           folder is being read.
        self.emit(Update::Reading {
            name: crate::controller_measures::display_name_of(&root),
        });
        self.source_root = Some(root.clone());
        // A newly opened folder starts with no file reported as unplayable, so its rows are marked again.
        if let Ok(mut reported) = self.reported_unplayable.lock() {
            reported.clear();
        }
        // What:     `self.queue.set_display_root(Some(root.clone()));`. Tell the queue which folder its names and pages are
        //           relative to, before the tracks arrive.
        // Why:      A library with one top-level folder keeps that folder in its names, while loose files stay in the
        //           virtual folders of their first letters, as in the picker and in Search (LS1).
        self.queue.set_display_root(Some(root.clone()));
        // What:     `self.rewatch_source_root();`. Point the file watcher at the new root so
        //           its changes drive live `Rescan`s.
        // Why:      Live updating follows whatever is currently open.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rewatchSourceRoot();
        // ```
        self.rewatch_source_root();
        // What:     `self.queue.set_tracks(expand_paths(vec![root]));`. Scan the single root
        //           directory into its files (folders -> their files, recursively) and replace
        //           the queue with the result (consumes the owned `root`).
        // Why:      The queue holds files, not directories; one root is scanned into the new
        //           playlist.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.queue.setTracks(expandPaths([root]));
        // ```
        // What:     a directory root is walked with `scan_streaming`, which hands every run of found tracks to
        //           `Update::ScanBatch` while it walks; the returned list is the whole walk, in queue order.
        // Why:      D111: Search works while the library is still being scanned. The batch's first position is its
        //           queue position, because the queue is exactly this list.
        let tracks: Vec<PathBuf> = if root.is_dir() {
            let mut next_index: usize = 0;
            let pause: std::time::Duration = crate::scan_stream::pause_from_environment();
            crate::scan_stream::scan_streaming(&root, crate::scan_stream::STREAM_BATCH, &mut |batch| {
                self.emit(Update::ScanBatch {
                    first_index: next_index,
                    names: crate::scan_stream::batch_names(&root, batch),
                });
                next_index += batch.len();
                if !pause.is_zero() {
                    std::thread::sleep(pause);
                }
            })
        } else {
            expand_paths(vec![root])
        };
        self.queue.set_tracks(tracks);
    }

    /// What:
    ///      `pub(crate) fn load_current(&mut self) -> bool`.
    ///  Open the queue's current
    ///           track into a decoder + reconfigure output.
    ///  Returns whether a track was
    ///           loaded.
    ///  Skips past files that fail to open.
    ///  `pub(crate)` so the
    ///           command-handling half can call it.
    /// Why:
    ///       One place that turns "current path" into live playback state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// loadCurrent(): boolean { ... }
    /// ```
    pub(crate) fn load_current(&mut self) -> bool {
        // What:     `let max_attempts = self.queue.len();`. How many opens to try before
        //           giving up.
        // Why:      `advance` now ALWAYS loops its scope (no end-of-queue `None`), so a queue
        //           of all-unreadable files would otherwise spin forever; cap the retries at
        //           the track count. Confinement is intentional: when a whole page is
        //           unreadable, stop rather than jump to another page.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const maxAttempts = this.queue.len();
        // ```
        let max_attempts = self.queue.len();
        // What:     `let mut failed: Vec<PathBuf> = Vec::new();`. The files this call could not open.
        // Why:      They are announced once when the call ends, so the window can name them (PB1).
        let mut failed: Vec<PathBuf> = Vec::new();
        // What:     `let mut attempts = 0;`. Count of failed opens so far.
        // Why:      Compare against `max_attempts` to bound the loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let attempts = 0;
        // ```
        let mut attempts = 0;
        // What:     `loop { ... }`. Iterate the queue, advancing past any unreadable file.
        //           Iterative (not recursive) so a long run of bad files cannot overflow the
        //           stack.
        // Why:      Robustly find the next playable track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { ... }
        // ```
        loop {
            // What:     `let path = match self.queue.current_path() { ... };`. Copy the
            //           current path out (`.clone()` -> owned `PathBuf`), or return `false`
            //           if the queue is empty.
            // Why:      Need an owned path to open and to release the queue borrow.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const path = this.queue.currentPath(); if (!path) return false;
            // ```
            let path = match self.queue.current_path() {
                // What:     `Some(p) => p.clone()`. `.clone()` makes an owned `PathBuf`.
                // Why:      Outlive the borrow into the queue.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // path = currentPath;
                // ```
                Some(p) => p.clone(),
                // What:     `None => return false`. Empty queue.
                // Why:      Nothing to load.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return false;
                // ```
                None => return false,
            };

            // What:     `match crate::decode::open(&path) { ... }`. Try to open a decoder.
            //           `&path` lends it. Returns `Result<Box<dyn Source>, PlayerError>`.
            // Why:      Build the source.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { const source = open(path); ... } catch (e) { ... }
            // ```
            match crate::decode::open(&path) {
                // What:     `Ok(source) => { self.install_source(source, &path); return true; }`.
                //           Loaded: install it and report success.
                // Why:      Begin playing this track.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.installSource(source, path); return true;
                // ```
                Ok(source) => {
                    self.install_source(source, &path);
                    self.report_unplayable(failed);
                    return true;
                }
                // What:     `Err(e) => { ... }`. Could not open this file; bind the error.
                // Why:      Skip to the next track.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // catch (e) { ... }
                // ```
                Err(e) => {
                    // What:     `tracing::warn!(path = %path.display(), error = %e, "cannot open
                    //           file");`. Log the failure as a structured event; `%` formats the
                    //           path and error via Display. The module path is the tag.
                    // Why:      Surface the bad file.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // logger.warn(`cannot open ${path}: ${e}`);
                    // ```
                    tracing::warn!(path = %path.display(), error = %e, "cannot open file");
                    // What:     `attempts += 1;`. Count this failed open.
                    // Why:      Track progress toward the retry cap.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // attempts += 1;
                    // ```
                    attempts += 1;
                    failed.push(path.clone());
                    // What:     `if attempts >= max_attempts { return false; }`. Give up after
                    //           trying every track once (advance loops, so it never returns
                    //           `None` on its own for a non-empty queue).
                    // Why:      Avoid an endless loop when all files are bad.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (attempts >= maxAttempts) return false;
                    // ```
                    if attempts >= max_attempts {
                        self.report_unplayable(failed);
                        return false;
                    }
                    // What:     `self.queue.advance(false);`. Step forward within the scope;
                    //           the returned index is ignored (we only need the cursor
                    //           moved). `advance` returns an `Option` that is not
                    //           `#[must_use]`, so discarding it is fine.
                    // Why:      Retry the loop with the next current track.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.queue.advance(false);
                    // ```
                    self.queue.advance(false);
                    // Otherwise the loop retries with the new current track.
                }
            }
        }
    }

    /// What:
    ///      `fn install_source(&mut self, source: Box<dyn Source>, path: &Path)`.
    ///  Store
    ///           the source,
    ///  reconfigure the output,
    ///  resolve the track's normalization gain,
    ///           reset position,
    ///  and tell the UI what is playing.
    /// Why:
    ///       The common setup after a successful `open`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// installSource(source: Source, path: string): void { ... }
    /// ```
    pub(crate) fn install_source(&mut self, source: Box<dyn Source>, path: &Path) {
        let duration_secs: f64 = self.install_source_silently(source, path);
        // What:     `self.emit_now_playing(path, duration_secs);`. Tell the UI which track plays and put the seek bar at zero.
        // Why:      The same announcement is made when a track that was queued behind the audible one
        //           becomes audible (`cross_track_boundary`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emitNowPlaying(path, durationSecs);
        // ```
        self.emit_now_playing(path, duration_secs);
    }

    /// What:
    ///      `pub(crate) fn install_source_silently(&mut self, source: Box<dyn Source>, path: &Path) -> f64`.
    ///  Everything `install_source` does except telling the window which track plays; returns the new file's duration.
    /// Why:
    ///       D113: a playing file that was replaced is loaded again where it was. The window already shows that track, so
    ///  announcing it would move the person's page and flash the seek bar to zero.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// installSourceSilently(source: Source, path: string): number { ...; return spec.durationSecs; }
    /// ```
    pub(crate) fn install_source_silently(&mut self, source: Box<dyn Source>, path: &Path) -> f64 {
        // What:     `let spec = source.spec();`. Copy the stream's rate/channels/duration
        //           (`AudioSpec` is `Copy`).
        // Why:      Needed to configure the output and the position math.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const spec = source.spec();
        // ```
        let spec = source.spec();

        // What:     `self.staged = None;`. Forget a next track that was opened early.
        // Why:      Loading a track rebuilds the output, so whatever was queued behind the old track is gone,
        //           and the track now being loaded replaces it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.staged = null;
        // ```
        self.staged = None;

        // What:     `self.loaded_file = Some((path, fingerprint))`. Remember which file the decoder was opened on.
        // Why:      A rescan compares it with the file on disk, so a paused track whose file was replaced is loaded again (D113).
        self.loaded_file = crate::peakcache::fingerprint(path).map(|key| return (path.to_path_buf(), key));

        // What:     `self.prepare_peak_for_path(path);`. Start the current-track
        //           peak swap path: cache hit applies immediately; cache miss sets
        //           fallback gain and stores a pending measurement receiver.
        // Why:      Loading no longer blocks on a full-track true-peak decode.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.preparePeakForPath(path);
        // ```
        self.prepare_peak_for_path(path);

        // What:     `if let Some(output) = self.output.as_mut() { ... }`. `.as_mut()` borrows
        //           the `Option<Output>` as `Option<&mut Output>`; reconfigure only when
        //           audio is available (not silent mode).
        // Why:      Skip audio setup when there is no output.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.output) { ... }
        // ```
        if let Some(output) = self.output.as_mut() {
            // What:     `let capacity_frames = spec.rate as usize;`. ~1 second of audio (`as
            //           usize` widens the rate).
            // Why:      Enough buffering to avoid underruns without big latency.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const capacityFrames = spec.rate;
            // ```
            let capacity_frames = spec.rate as usize;
            // What:     `match output.reconfigure(spec.rate, spec.channels, capacity_frames) { ... }`.
            //           Rebuild the stream at this track's format; returns a new producer.
            // Why:      Per-track native rate; fresh buffer flushes old audio.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { this.producer = output.reconfigure(spec.rate, spec.channels, capacityFrames); } catch (e) { ... }
            // ```
            match output.reconfigure(spec.rate, spec.channels, capacity_frames) {
                // What:     `Ok(prod) => self.producer = Some(prod)`. Store the write end.
                // Why:      Push samples here from now on.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.producer = prod;
                // ```
                Ok(prod) => self.producer = Some(prod),
                // What:     `Err(e) => { eprintln!(...); self.producer = None; }`. Log and drop
                //           into silent mode for this track.
                // Why:      Don't crash if a stream fails to connect.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // console.error(e); this.producer = null;
                // ```
                Err(e) => {
                    tracing::warn!(error = %e, "audio reconfigure failed");
                    self.producer = None;
                }
            }
        }

        // What:     `self.source = Some(source);`. Store the decoder (moves it in, wrapped
        //           in `Some`).
        // Why:      Pump decodes from it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.source = source;
        // ```
        self.source = Some(source);
        // What:     `self.spec = Some(spec);`. Cache the format.
        // Why:      Position math + future reconfigure (seek).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.spec = spec;
        // ```
        self.spec = Some(spec);
        // What:     `self.clock.restart(0);`. Count from an empty ring that starts at track
        //           frame 0; this also clears the throttle baseline and any end-of-track drain.
        // Why:      New track starts at 0, and its ring is new (`reconfigure` above dropped the old one).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.clock.restart(0);
        // ```
        self.clock.restart(0);
        // What:     `self.pending.clear(); self.pending_pos = 0;`. Drop leftover samples.
        // Why:      Avoid mixing tracks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pending = []; this.pendingPos = 0;
        // ```
        self.pending.clear();
        self.pending_pos = 0;

        // What:     No wait here. Decoding starts at once with the fallback gain that
        //           `prepare_peak_for_path` set; the engine loop swaps in the exact gain
        //           when the measurement lands.
        // Why:      On a cache miss the new stream's ring buffer is empty, so waiting here
        //           for the measurement was an audible gap at every track change.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // nothing to wait for: the fallback gain is already in place
        // ```
        return spec.duration_secs;
    }

    /// What:
    ///      `fn emit_now_playing(&self, path: &Path, duration_secs: f64)`.
    ///  Announce the queue's current track to the UI (its row, its display name, its length)
    ///  and reset the seek bar to zero.
    /// Why:
    ///       Both a freshly loaded track and a track that follows the audible one without a gap
    ///  are announced this way, at the moment they become audible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emitNowPlaying(path: string, durationSecs: number): void { ... }
    /// ```
    pub(crate) fn emit_now_playing(&self, path: &Path, duration_secs: f64) {
        // What:     `let index = self.queue.current_index();`. Its LOAD-ORDER position in the
        //           queue (an `Option<usize>`).
        // Why:      Lets the UI highlight the current row, and indexes the display paths for
        //           the name below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const index = this.queue.currentIndex();
        // ```
        let index = self.queue.current_index();
        // What:     `let name = index.and_then(|i| self.queue.display_paths().into_iter().nth(i)).unwrap_or_else(|| file_name_of(path));`.
        //           The track's DISPLAY PATH relative to the queue root (the same string the
        //           list row shows, e.g. `r-906/diaLOG/06 V.flac`): `.and_then` indexes
        //           `display_paths()` by the load-order index (`.into_iter().nth(i)`), and
        //           `.unwrap_or_else(...)` falls back to the bare filename if the index is
        //           absent. `display_paths()` runs only when `index` is `Some` (lazily,
        //           inside the closure).
        // Why:      The window title shows this name, so it must match the list row, not just
        //           the filename. Still filesystem-derived (no embedded tags).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = index != null ? this.queue.displayPaths()[index] : fileNameOf(path);
        // ```
        let name = index
            .and_then(|i| return self.queue.display_paths().into_iter().nth(i))
            .unwrap_or_else(|| return file_name_of(path));
        // What:     `self.emit(Update::NowPlaying { index, name, duration: duration_secs });`.
        //           Tell the UI the new track (struct-variant literal).
        // Why:      Update the now-playing label and seek-bar maximum.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "nowPlaying", index, name, duration: durationSecs });
        // ```
        self.emit(Update::NowPlaying {
            index,
            name,
            duration: duration_secs,
        });
        // What:     `self.emit(Update::Position(0.0));`. Reset the seek bar to 0.
        // Why:      New track starts at the beginning.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "position", secs: 0 });
        // ```
        self.emit(Update::Position(0.0));
    }

    /// What:
    ///      `pub(crate) fn seek(&mut self, secs: f64)`.
    ///  Move playback to `secs` and
    ///           flush buffered audio.
    ///  `pub(crate)` so the command-handling half can call it.
    /// Why:
    ///       Seek-bar control.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seek(secs: number): void { ... }
    /// ```
    pub(crate) fn seek(&mut self, secs: f64) {
        // What:     `if self.staged.is_some() && !self.return_to_audible_track() { return; }`. When the next track
        //           was opened early, switch the decoder back to the track the listener still hears.
        // Why:      The seek bar belongs to the audible track; seeking the staged one would play the wrong file.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.staged && !this.returnToAudibleTrack()) return;
        // ```
        if self.staged.is_some() && !self.return_to_audible_track() {
            return;
        }
        // What:     `let spec = match self.spec { Some(s) => s, None => return };`. Copy the
        //           format out, or do nothing if no track is loaded.
        // Why:      Need the rate to recompute the frame position.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const spec = this.spec; if (!spec) return;
        // ```
        let spec = match self.spec {
            // What:     `Some(s) => s`. Copy the spec (`AudioSpec` is `Copy`).
            // Why:      No borrow held.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // spec = this.spec;
            // ```
            Some(s) => s,
            // What:     `None => return`. Nothing loaded.
            // Why:      Ignore the seek.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            None => return,
        };

        // What:     `if let Some(source) = self.source.as_mut() { ... } else { return; }`.
        //           `.as_mut()` borrows the optional source mutably; seek the decoder, or
        //           bail if there is no source.
        // Why:      The decoder must reposition.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.source) return; this.source.seek(secs);
        // ```
        if let Some(source) = self.source.as_mut() {
            // What:     `if let Err(e) = source.seek(secs) { eprintln!(...); return; }`.
            //           Attempt the seek; on the error variant, log and abort.
            // Why:      A failed seek should not corrupt position state.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { source.seek(secs); } catch (e) { console.error(e); return; }
            // ```
            if let Err(e) = source.seek(secs) {
                tracing::warn!(error = %e, "seek failed");
                return;
            }
        } else {
            // What:     `return;`. No source -> nothing to seek.
            // Why:      Guard.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return;
        }

        // What:     `if let Some(output) = self.output.as_mut() { ... }`. Rebuild the stream
        //           at the SAME format to flush stale buffered audio.
        // Why:      Otherwise ~1s of pre-seek audio would still play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.output) { this.producer = output.reconfigure(spec.rate, spec.channels, spec.rate); }
        // ```
        if let Some(output) = self.output.as_mut() {
            // What:     `match output.reconfigure(spec.rate, spec.channels, spec.rate as usize) { ... }`.
            //           Same rate/channels, ~1s buffer; new empty producer.
            // Why:      Clean slate after the jump.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { this.producer = output.reconfigure(spec.rate, spec.channels, spec.rate); } catch (e) { ... }
            // ```
            match output.reconfigure(spec.rate, spec.channels, spec.rate as usize) {
                // What:     `Ok(prod) => self.producer = Some(prod)`. Replace the producer.
                // Why:      Flush.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.producer = prod;
                // ```
                Ok(prod) => self.producer = Some(prod),
                // What:     `Err(e) => eprintln!(...)`. Log a reconfigure failure.
                // Why:      Keep going (position still updates).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // console.error(e);
                // ```
                Err(e) => tracing::warn!(error = %e, "seek reconfigure failed"),
            }
        }

        // What:     `self.pending.clear(); self.pending_pos = 0;`. Drop leftover pre-seek
        //           samples we had not pushed yet.
        // Why:      They belong to the old position.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pending = []; this.pendingPos = 0;
        // ```
        self.pending.clear();
        self.pending_pos = 0;

        // What:     `self.clock.restart((secs * spec.rate as f64) as u64);`. Convert the
        //           target seconds to a frame count (`as u64` truncates the float) and count from the
        //           new, empty ring that starts at that frame; this also sets the throttle baseline to
        //           the target and clears an end-of-track drain.
        // Why:      Keep position reporting consistent after the jump: the sound resumes at `secs`,
        //           so the audible position is `secs` until the output consumes more.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.clock.restart(Math.floor(secs * spec.rate));
        // ```
        self.clock.restart((secs * spec.rate as f64) as u64);
        // What:     `self.emit(Update::Position(secs));`. Snap the UI seek bar.
        // Why:      Reflect the jump immediately.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "position", secs });
        // ```
        self.emit(Update::Position(secs));
    }

    /// What:
    ///      `pub(crate) fn pump_audio(&mut self) -> bool`.
    ///  Push at most one block of
    ///           audio into the ring buffer.
    ///  Returns whether it did meaningful work.
    ///           `pub(crate)` so `engine::run` can call it each loop iteration.
    /// Why:
    ///       The decode->buffer feeding step.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pumpAudio(): boolean { ... }
    /// ```
    pub(crate) fn pump_audio(&mut self) -> bool {
        // What:     `if !self.playing { return false; }`. Paused: no work.
        // Why:      Respect pause.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.playing) return false;
        // ```
        if !self.playing {
            return false;
        }
        // What:     `if self.clock.is_draining() { return false; }`. The decoder already reached the end
        //           of the track; the ring is being played out and `tick_playback` starts the next track.
        // Why:      Decoding again at end of stream would only repeat the end-of-track signal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.clock.isDraining()) return false;
        // ```
        if self.clock.is_draining() {
            return false;
        }
        // What:     `if self.producer.is_none() || self.source.is_none() { return false; }`.
        //           Need both a write end and a decoder. `.is_none()` is true for the empty
        //           `Option`.
        // Why:      Nothing to do otherwise.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.producer || !this.source) return false;
        // ```
        if self.producer.is_none() || self.source.is_none() {
            return false;
        }

        // What:     `if self.pending_pos < self.pending.len() { ... }`. Leftover samples from
        //           last time; push them first.
        // Why:      Finish the previous block before decoding more.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.pendingPos < this.pending.length) { ... }
        // ```
        if self.pending_pos < self.pending.len() {
            // What:     `let pushed = if let Some(producer) = self.producer.as_mut() { producer.push_slice(&self.pending[self.pending_pos..]) } else { 0 };`.
            //           Push the unsent tail; `push_slice` returns how many were accepted.
            //           `&self.pending[a..]` borrows a sub-slice (disjoint from the producer).
            // Why:      Make progress draining `pending`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const pushed = this.producer.pushSlice(this.pending.slice(this.pendingPos));
            // ```
            let pushed = if let Some(producer) = self.producer.as_mut() {
                producer.push_slice(&self.pending[self.pending_pos..])
            } else {
                0
            };
            // What:     `self.pending_pos += pushed;`. Advance the sent cursor.
            // Why:      Track what is left.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pendingPos += pushed;
            // ```
            self.pending_pos += pushed;
            // What:     `if self.pending_pos >= self.pending.len() { self.pending.clear(); self.pending_pos = 0; }`.
            //           Fully drained -> reset the buffer.
            // Why:      Ready to decode next time.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (this.pendingPos >= this.pending.length) { this.pending = []; this.pendingPos = 0; }
            // ```
            if self.pending_pos >= self.pending.len() {
                self.pending.clear();
                self.pending_pos = 0;
            }
            // What:     `self.record_pushed(pushed);`. Count the pushed frames.
            // Why:      The audible position is pushed minus what is still queued in the ring.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.recordPushed(pushed);
            // ```
            self.record_pushed(pushed);
            // What:     `return pushed > 0;`. Did work only if something was pushed.
            // Why:      Tell the caller whether to sleep.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return pushed > 0;
            // ```
            return pushed > 0;
        }

        // What:     `let decoded = if let Some(source) = self.source.as_mut() { source.next_chunk() } else { return false; };`.
        //           Decode the next block; `next_chunk()` returns `Result<Vec<f32>, _>`.
        // Why:      Produce more audio.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const decoded = this.source.nextChunk();
        // ```
        let decoded = if let Some(source) = self.source.as_mut() {
            source.next_chunk()
        } else {
            return false;
        };

        // What:     `let mut chunk = match decoded { ... };`. Unwrap; on error, log, start the
        //           end-of-track drain, and report work done. `mut` because the samples are gained in
        //           place below.
        // Why:      Handle decode failures without crashing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let chunk; try { chunk = decoded; } catch (e) { console.error(e); this.beginTrackEnd(); return true; }
        // ```
        let mut chunk = match decoded {
            // What:     `Ok(c) => c`. The decoded samples.
            // Why:      Continue.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // chunk = c;
            // ```
            Ok(c) => c,
            // What:     `Err(e) => { tracing::warn!(...); self.begin_track_end(); return true; }`. Treat
            //           a decode error as end-of-track.
            // Why:      Move on rather than stall.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch (e) { console.error(e); this.beginTrackEnd(); return true; }
            // ```
            Err(e) => {
                tracing::warn!(error = %e, "decode error; treating as end of track");
                self.begin_track_end();
                return true;
            }
        };

        // What:     `if chunk.is_empty() { self.begin_track_end(); return true; }`. An empty
        //           chunk is the end-of-stream signal.
        // Why:      Every sample of the track is now in the ring; the next track starts after the
        //           output consumed them (`tick_playback`), not now.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (chunk.length === 0) { this.beginTrackEnd(); return true; }
        // ```
        if chunk.is_empty() {
            self.begin_track_end();
            return true;
        }

        // What:     `let gain = self.volume * self.track_gain;`. Combine the user volume with
        //           the track's true-peak normalization gain.
        // Why:      One scalar applied per sample below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gain = this.volume * this.trackGain;
        // ```
        let gain = self.volume * self.track_gain;
        // What:     `for sample in chunk.iter_mut() { *sample = process_sample(*sample, gain); }`.
        //           Run every sample through the output stage in place. `iter_mut` yields
        //           `&mut f32`; the right `*sample` reads, the left `*sample =` writes back.
        // Why:      Apply gain + clamp in one tested place.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (let i = 0; i < chunk.length; i++) chunk[i] = processSample(chunk[i], gain);
        // ```
        for sample in chunk.iter_mut() {
            *sample = process_sample(*sample, gain);
        }

        // What:     `let pushed = if let Some(producer) = self.producer.as_mut() { producer.push_slice(&chunk) } else { 0 };`.
        //           Push the gained chunk; `push_slice` returns the accepted count.
        // Why:      Feed the audio thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pushed = this.producer.pushSlice(chunk);
        // ```
        let pushed = if let Some(producer) = self.producer.as_mut() {
            producer.push_slice(&chunk)
        } else {
            0
        };
        // What:     `self.record_pushed(pushed);`. Count the pushed frames.
        // Why:      The audible position is pushed minus what is still queued in the ring.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.recordPushed(pushed);
        // ```
        self.record_pushed(pushed);
        // What:     `if pushed < chunk.len() { self.pending = chunk; self.pending_pos = pushed; }`.
        //           Stash the remainder if the buffer could not take all of it (MOVES
        //           `chunk` into `self.pending`).
        // Why:      Push the rest next cycle instead of dropping samples.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (pushed < chunk.length) { this.pending = chunk; this.pendingPos = pushed; }
        // ```
        if pushed < chunk.len() {
            self.pending = chunk;
            self.pending_pos = pushed;
        }
        // What:     `true`. We decoded and pushed: work was done. Tail expression -> return.
        // Why:      Caller should not sleep.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return true;
        // ```
        return true
    }

    /// What:
    ///      `pub(crate) fn report_audio_meters(&self, reporter: &mut MeterReporter)`.
    ///  Logs the output's underrun counters once a second, and only when they moved.
    /// Why:
    ///       The realtime callback only counts; this runs on the engine thread, where logging is allowed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reportAudioMeters(reporter: MeterReporter): void { ... }
    /// ```
    pub(crate) fn report_audio_meters(&self, reporter: &mut MeterReporter) {
        let Some(output) = self.output.as_ref() else {
            return;
        };
        let Some(change) = reporter.take_change(std::time::Instant::now(), || return output.meter_snapshot()) else {
            return;
        };
        if change.short_callbacks > 0 {
            tracing::warn!(
                short_callbacks = change.short_callbacks,
                silent_frames = change.silent_frames,
                requested_frames = change.last_requested_frames,
                buffer_frames = change.last_buffer_frames,
                "audio output underran in the last second"
            );
            return;
        }
        tracing::info!(
            requested_frames = change.last_requested_frames,
            buffer_frames = change.last_buffer_frames,
            "audio output buffer sizes"
        );
    }

    /// What:
    ///      `fn on_track_end(&mut self)`.
    ///  Natural end of the current track:
    ///  advance the
    ///           queue (natural end,
    ///  so repeat-one replays) and load,
    ///  or stop.
    /// Why:
    ///       Auto-advance between tracks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// onTrackEnd(): void { const moved = this.queue.advance(true); this.afterMove(moved); }
    /// ```
    pub(crate) fn on_track_end(&mut self) {
        // What:     `let moved = self.queue.advance(true);`. `true` = natural end, letting
        //           repeat-one replay the same track.
        // Why:      Honour the repeat mode.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const moved = this.queue.advance(true);
        // ```
        let moved = self.queue.advance(true);
        // What:     `self.after_move(moved);`. Load the next or stop. `after_move` lives in
        //           the command-handling half but is the same type's method.
        // Why:      Shared follow-up logic.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.afterMove(moved);
        // ```
        self.after_move(moved);
    }

    /// What:
    ///      `fn ring_queued_frames(&self) -> u64`.
    ///  Frames the decoder pushed that the output has not taken from the ring yet.
    ///  Reads the producer's view of the ring (an atomic load of the two indices),
    ///  so it takes no lock and never touches the output callback.
    /// Why:
    ///       The ring's read index is advanced only by the output callback,
    ///  so what remains is exactly the audio that is still to be heard.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ringQueuedFrames(): number { return framesOfSamples(this.producer?.length ?? 0, this.spec?.channels ?? 0); }
    /// ```
    fn ring_queued_frames(&self) -> u64 {
        let channels = self.spec.as_ref().map_or(0, |s| return s.channels);
        let queued_samples = self.producer.as_ref().map_or(0, |producer| return producer.occupied_len());
        return frames_of_samples(queued_samples, channels)
    }

    /// What:
    ///      `fn audible_secs(&self) -> f64`.
    ///  The position (seconds) of the sound the listener hears now:
    ///  frames the output consumed from the ring, at the loaded track's rate (zero with no track).
    /// Why:
    ///       Shared by the saved session and the `Position` updates.
    ///  No device latency is subtracted:
    ///  the output hands audio to the system mixer as soon as it takes it from the ring.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// audibleSecs(): number { return this.clock.audibleSecs(this.ringQueuedFrames(), this.spec?.rate ?? 0); }
    /// ```
    pub(crate) fn audible_secs(&self) -> f64 {
        let rate = self.spec.as_ref().map_or(0, |s| return s.rate);
        return self.clock.audible_secs(self.ring_queued_frames(), rate)
    }

    /// What:
    ///      `fn record_pushed(&mut self, samples_pushed: usize)`.
    ///  Count the frames just written into the ring.
    /// Why:
    ///       Pushed frames minus queued frames is what the output consumed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordPushed(samplesPushed: number): void { this.clock.recordPushed(framesOfSamples(samplesPushed, channels)); }
    /// ```
    fn record_pushed(&mut self, samples_pushed: usize) {
        let channels = self.spec.as_ref().map_or(0, |s| return s.channels);
        self.clock.record_pushed(frames_of_samples(samples_pushed, channels));
    }

    /// What:
    ///      `fn begin_track_end(&mut self)`.
    ///  The decoder reached the end of the track.
    ///  When the next track can follow in the same output stream, open it now and keep pushing its frames behind
    ///  the old track's;
    ///  otherwise stop decoding and let the ring play out, and `tick_playback` starts the next track after the
    ///  last frame was consumed.
    /// Why:
    ///       Starting the next track by rebuilding the output now discards up to one second of the old track's
    ///  unplayed tail, and the rebuilt stream starts silent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// beginTrackEnd(): void { if (!this.stageNextTrack() && this.clock.beginDrain()) { this.output?.setDraining(true); } }
    /// ```
    fn begin_track_end(&mut self) {
        if self.stage_next_track() {
            return;
        }
        if self.clock.begin_drain() {
            if let Some(output) = self.output.as_ref() {
                output.set_draining(true);
            }
            tracing::info!(
                queued_frames = self.ring_queued_frames(),
                "decoder reached the end of the track; playing out the ring"
            );
        }
    }

    /// What:
    ///      `fn stage_next_track(&mut self) -> bool`.
    ///  Open the track the queue will play next and, when it has the same sample rate and channel count as the
    ///  current one, make it the decoder's source so its frames follow the current track's in the same ring.
    ///  Returns whether the next track was staged.
    ///  The queue itself does not move until the output consumes the last frame of the current track.
    /// Why:
    ///       Same-format neighbours (an album) then play with no gap and no rebuilt stream.
    ///  A different format needs a new stream, so it falls back to playing the ring out.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stageNextTrack(): boolean { ... }
    /// ```
    fn stage_next_track(&mut self) -> bool {
        if self.clock.has_boundary() || self.producer.is_none() {
            return false;
        }
        let Some(previous_spec) = self.spec else {
            return false;
        };
        let Some(index) = self.queue.peek_next_natural() else {
            return false;
        };
        let Some(path) = self.queue.tracks().get(index).cloned() else {
            return false;
        };
        let source = match crate::decode::open(&path) {
            Ok(source) => source,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "cannot open the next track early; playing out the ring");
                return false;
            }
        };
        let spec = source.spec();
        if spec.rate != previous_spec.rate || spec.channels != previous_spec.channels {
            tracing::info!(
                path = %path.display(),
                previous_rate = previous_spec.rate,
                previous_channels = previous_spec.channels,
                rate = spec.rate,
                channels = spec.channels,
                "the next track has another format; playing out the ring before the output is rebuilt"
            );
            return false;
        }
        self.prepare_peak_for_path(&path);
        self.source = Some(source);
        self.spec = Some(spec);
        self.clock.mark_boundary();
        tracing::info!(
            path = %path.display(),
            queued_frames = self.ring_queued_frames(),
            "the next track follows in the same stream"
        );
        self.staged = Some(StagedTrack { path, spec, previous_spec });
        return true
    }

    /// What:
    ///      `fn cross_track_boundary(&mut self) -> bool`.
    ///  The output consumed the last frame of the old track and the staged track is audible now:
    ///  move the queue onto it and announce it.
    ///  If the queue no longer names the staged file (the play mode, the page or the folder changed in the last
    ///  second), reload whatever the queue points at instead.
    ///  Returns whether it did anything.
    /// Why:
    ///       The listener hears the new track now, so the title, the row and the seek bar change now,
    ///  not a second earlier when the decoder moved on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// crossTrackBoundary(): boolean { ... }
    /// ```
    fn cross_track_boundary(&mut self) -> bool {
        let Some(staged) = self.staged.take() else {
            self.clock.cross_boundary();
            return false;
        };
        let moved = self.queue.advance(true);
        if self.queue.current_path() != Some(&staged.path) {
            tracing::warn!(
                staged = %staged.path.display(),
                "the queue changed while the next track was staged; reloading the current track"
            );
            self.after_move(moved);
            return true;
        }
        tracing::info!(path = %staged.path.display(), "the next track is audible now");
        self.clock.cross_boundary();
        self.emit_now_playing(&staged.path, staged.spec.duration_secs);
        return true
    }

    /// What:
    ///      `fn return_to_audible_track(&mut self) -> bool`.
    ///  Undo staging: reopen the track the listener still hears (the queue never left it) and make it the
    ///  decoder's source again.
    ///  Returns whether the audible track is the source now (true when nothing was staged).
    /// Why:
    ///       A seek in the last second of a track applies to the track on the seek bar, not the one behind it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// returnToAudibleTrack(): boolean { ... }
    /// ```
    fn return_to_audible_track(&mut self) -> bool {
        let Some(staged) = self.staged.take() else {
            return true;
        };
        let Some(path) = self.queue.current_path().cloned() else {
            self.staged = Some(staged);
            return false;
        };
        let source = match crate::decode::open(&path) {
            Ok(source) => source,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "cannot reopen the audible track to seek in it");
                self.staged = Some(staged);
                return false;
            }
        };
        tracing::info!(path = %path.display(), "seek while the next track was staged; back to the audible track");
        self.prepare_peak_for_path(&path);
        self.source = Some(source);
        self.spec = Some(staged.previous_spec);
        return true
    }

    /// What:
    ///      `fn audible_spec(&self) -> Option<AudioSpec>`.
    ///  The format and length of the track the listener hears:
    ///  the previous track while a staged one waits behind it, else the loaded one.
    /// Why:
    ///       The seek bar's length must belong to the track on screen.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// audibleSpec(): AudioSpec | null { return this.staged?.previousSpec ?? this.spec; }
    /// ```
    fn audible_spec(&self) -> Option<AudioSpec> {
        return self.staged.as_ref().map(|staged| return staged.previous_spec).or(self.spec)
    }

    /// What:
    ///      `fn drain_hold(&self) -> Duration`.
    ///  How long after the ring ran empty the old stream must live on:
    ///  the time the output needs to play the audio it already took from the ring
    ///  (`FILL_FRAMES_FLOOR` frames at the track's rate), or zero without an output.
    /// Why:
    ///       Rebuilding the stream drops what it still holds;
    ///  waiting this long keeps the last milliseconds of the track audible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// drainHold(): number { return this.output ? framesToSecs(FILL_FRAMES_FLOOR, rate) : 0; }
    /// ```
    fn drain_hold(&self) -> Duration {
        if self.output.is_none() {
            return Duration::ZERO;
        }
        let rate = self.audible_spec().map_or(0, |s| return s.rate);
        return Duration::from_secs_f64(frames_to_secs(crate::output::FILL_FRAMES_FLOOR as u64, rate))
    }

    /// What:
    ///      `pub(crate) fn idle_park_timeout(&self, fallback: Duration) -> Duration`.
    ///  How long the engine thread may sleep: `fallback`, or less while an end-of-track hold is running.
    ///  `pub(crate)` so `engine::run` can call it before parking.
    /// Why:
    ///       The output does not wake the engine once the ring is empty,
    ///  so without this the hold would stretch to the idle timeout.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// idleParkTimeout(fallback: number): number { ... }
    /// ```
    pub(crate) fn idle_park_timeout(&self, fallback: Duration) -> Duration {
        return self
            .clock
            .drain_wait(Instant::now(), self.drain_hold())
            .map_or(fallback, |left| return left.max(Duration::from_millis(1)).min(fallback))
    }

    /// What:
    ///      `pub(crate) fn tick_playback(&mut self) -> bool`.
    ///  Emit the audible position when due, announce a staged track once it is audible,
    ///  and start the next track once the last frame of an old one was consumed.
    ///  Returns whether it changed the loaded track (work was done).
    ///  `pub(crate)` so `engine::run` can call it each loop iteration.
    /// Why:
    ///       Position follows the speakers, not the decoder,
    ///  and nothing is pushed while the tail plays out,
    ///  so this cannot live in `pump_audio`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tickPlayback(): boolean { ... }
    /// ```
    pub(crate) fn tick_playback(&mut self) -> bool {
        // Silent mode (no stream) has nothing audible to report.
        if self.producer.is_none() {
            return false;
        }
        let Some(spec) = self.audible_spec() else {
            return false;
        };
        let queued = self.ring_queued_frames();
        if let Some(secs) = self.clock.position_to_emit(queued, spec.rate) {
            self.emit(Update::Position(secs));
        }
        if self.clock.boundary_reached(queued) {
            return self.cross_track_boundary();
        }
        // A paused player keeps its ring, so a drain that is still in progress resumes with play.
        let hold = self.drain_hold();
        if !self.playing || !self.clock.drain_finished(queued, Instant::now(), hold) {
            return false;
        }
        tracing::info!("the last frame of the track was consumed; starting the next track");
        self.clock.finish_drain();
        self.on_track_end();
        return true
    }
}

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
