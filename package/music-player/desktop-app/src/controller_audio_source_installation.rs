//! Installing a Source Root and walking it into the queue.
//!
//! What:
//! `rewatch_source_root` registers the watcher,
//! `scan_root_into_queue` walks the folder and announces each batch,
//! `load_current` opens the selected track,
//! and `install_source` and `install_source_silently` hand a decoder to the
//! output.
//! Why:
//! those five are the path from a folder on disk to a decoder the audio thread
//! reads,
//! so they are read together,
//! split from `src/controller_audio.rs` to stay within the line budget.

/// What:
///      `use std::path::{Path, PathBuf}` brings the borrowed and owned filesystem
///           path types.
/// Why:
///      A decoder is opened from a borrowed `&Path`,
///  and the walk hands each found file on as an owned `PathBuf`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is just a string in TS
/// ```
use std::path::{
    Path,
    PathBuf,
};

/// What:
///      `use crate::command::Update` brings the engine-to-window message enum.
/// Why:
///      Installing a source announces the new queue and the current track with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Update } from "./command";
/// ```
use crate::command::Update;
/// What:
///      `use crate::controller::Controller` brings the state struct these methods are
///           added to.
/// Why:
///      This file is one `impl Controller` block,
///  so the type must be in scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Controller } from "./controller";
/// ```
use crate::controller::Controller;
/// What:
///      `use crate::decode::Source` brings the decoder trait a source implements.
/// Why:
///      Both install methods take a `Box<dyn Source>`,
///  and its `spec`,
///  `next_chunk` and `seek` methods must be in scope to drive it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Source } from "./decode";
/// ```
use crate::decode::Source;

/// What:
///      `use crate::playback::expand_paths` brings the folder-to-files expansion.
/// Why:
///      A Source Root is a folder,
///  and the queue is the audio files under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { expand_paths } from "./playback";
/// ```
use crate::playback::expand_paths;

/// What:
///      `use super::ScanBatchAnnouncer` brings the listener that announces each batch of
///           a running walk.
/// Why:
///      The walk reports what it found as it goes,
///  so a person can search the library while it is still being scanned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ScanBatchAnnouncer } from "./controller_audio";
/// ```
use super::ScanBatchAnnouncer;

/// What:
///      `impl Controller` with the five methods that install a Source Root,
///  walk it into the queue and hand a decoder to the output.
/// Why:
///       The type is declared in `controller.rs`,
///  and Rust lets one type's methods live in several `impl` blocks,
///  which is what keeps each file within the line budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Controller.prototype.installSource = function (...) { ... }
/// ```
impl Controller {
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
    pub(crate) fn scan_root_into_queue(
        &mut self,
        root: PathBuf,
    ) {
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
            let pause: std::time::Duration = crate::scan_stream::pause_from_environment();
            let mut announcer = ScanBatchAnnouncer {
                controller: &*self,
                root: &root,
                next_index: 0,
                pause,
            };
            crate::scan_stream::scan_streaming(
                &root,
                crate::scan_stream::STREAM_BATCH,
                &mut announcer,
            )
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
    pub(crate) fn install_source(
        &mut self,
        source: Box<dyn Source>,
        path: &Path,
    ) {
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
    ///  Everything `install_source` does except telling the window which track plays;
    /// returns the new file's duration.
    /// Why:
    ///       D113:
    /// a playing file that was replaced is loaded again where it was.
    /// The window already shows that track,
    /// so
    ///  announcing it would move the person's page and flash the seek bar to zero.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// installSourceSilently(source: Source, path: string): number { ...; return spec.durationSecs; }
    /// ```
    pub(crate) fn install_source_silently(
        &mut self,
        source: Box<dyn Source>,
        path: &Path,
    ) -> f64 {
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
        match crate::peakcache::fingerprint(path) {
            Some(key) => self.loaded_file = Some((path.to_path_buf(), key)),
            None => self.loaded_file = None,
        }

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
}
