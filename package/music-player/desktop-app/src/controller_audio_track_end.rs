//! Controller side of the end of a track:
//! the natural end,
//! the staged next track,
//! the drain hold and the playback tick.
//!
//! What:
//!  `impl Controller` methods that decide what happens when the audible track runs out:
//!  advance the queue,
//!  stage the next track behind the one still sounding,
//!  cross the boundary when its frames have been heard,
//!  and hold the drain until the ring buffer is empty.
//! Why:
//!  These methods are the second half of the audio flow of `controller_audio.rs`;
//!  they are split off so that neither file grows past the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Controller { onTrackEnd() {} stageNextTrack() {} crossTrackBoundary() {} tickPlayback() {} }
//! ```

/// Imports the duration and instant types that time the drain hold and the idle park.
use std::time::{Duration, Instant};

/// Imports the ring buffer trait that reads how many samples the output has not taken yet.
use ringbuf::traits::Observer;

/// Imports the update that the controller sends to the window.
use crate::command::Update;

/// Imports the controller whose end-of-track behavior this file extends.
use crate::controller::Controller;

/// Imports the staged track that waits behind the sounding one.
use crate::controller_audio::StagedTrack;

/// Imports the audio format of the audible track.
use crate::decode::AudioSpec;

/// Imports the conversion from frames to seconds.
use crate::playback::frames_to_secs;

/// Imports the conversion from samples to frames.
use crate::playback_clock::frames_of_samples;


/// What:
///      `impl Controller { ... }`.
///  The end-of-track half of the audio behaviour (a THIRD inherent `impl` block for `Controller`,
///  whose first halves are in `controller.rs` and `controller_audio.rs`).
/// Why:
///       Keep the handoff between tracks apart from loading and pumping without one huge file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Controller { /* on_track_end, begin_track_end, stage_next_track, cross_track_boundary, return_to_audible_track, drain_hold, idle_park_timeout, tick_playback */ }
/// ```
impl Controller {
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
    pub(crate) fn ring_queued_frames(&self) -> u64 {
        let channels = self.spec.as_ref().map_or(0, |s| return s.channels);
        let queued_samples = self.producer.as_ref().map_or(0, |producer| return producer.occupied_len());
        return frames_of_samples(queued_samples, channels)
    }

    /// What:
    ///      `fn audible_secs(&self) -> f64`.
    ///  The position (seconds) of the sound the listener hears now:
    ///  frames the output consumed from the ring,
    /// at the loaded track's rate (zero with no track).
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
    pub(crate) fn record_pushed(&mut self, samples_pushed: usize) {
        let channels = self.spec.as_ref().map_or(0, |s| return s.channels);
        self.clock.record_pushed(frames_of_samples(samples_pushed, channels));
    }

    /// What:
    ///      `fn begin_track_end(&mut self)`.
    ///  The decoder reached the end of the track.
    ///  When the next track can follow in the same output stream,
    /// open it now and keep pushing its frames behind
    ///  the old track's;
    ///  otherwise stop decoding and let the ring play out,
    /// and `tick_playback` starts the next track after the
    ///  last frame was consumed.
    /// Why:
    ///       Starting the next track by rebuilding the output now discards up to one second of the old track's
    ///  unplayed tail,
    /// and the rebuilt stream starts silent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// beginTrackEnd(): void { if (!this.stageNextTrack() && this.clock.beginDrain()) { this.output?.setDraining(true); } }
    /// ```
    pub(crate) fn begin_track_end(&mut self) {
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
    ///  Open the track the queue will play next and,
    /// when it has the same sample rate and channel count as the
    ///  current one,
    /// make it the decoder's source so its frames follow the current track's in the same ring.
    ///  Returns whether the next track was staged.
    ///  The queue itself does not move until the output consumes the last frame of the current track.
    /// Why:
    ///       Same-format neighbours (an album) then play with no gap and no rebuilt stream.
    ///  A different format needs a new stream,
    /// so it falls back to playing the ring out.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stageNextTrack(): boolean { ... }
    /// ```
    pub(crate) fn stage_next_track(&mut self) -> bool {
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
    ///  If the queue no longer names the staged file (the play mode,
    /// the page or the folder changed in the last
    ///  second),
    /// reload whatever the queue points at instead.
    ///  Returns whether it did anything.
    /// Why:
    ///       The listener hears the new track now,
    /// so the title,
    /// the row and the seek bar change now,
    ///  not a second earlier when the decoder moved on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// crossTrackBoundary(): boolean { ... }
    /// ```
    pub(crate) fn cross_track_boundary(&mut self) -> bool {
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
    ///  Undo staging:
    /// reopen the track the listener still hears (the queue never left it) and make it the
    ///  decoder's source again.
    ///  Returns whether the audible track is the source now (true when nothing was staged).
    /// Why:
    ///       A seek in the last second of a track applies to the track on the seek bar,
    /// not the one behind it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// returnToAudibleTrack(): boolean { ... }
    /// ```
    pub(crate) fn return_to_audible_track(&mut self) -> bool {
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
    ///  the previous track while a staged one waits behind it,
    /// else the loaded one.
    /// Why:
    ///       The seek bar's length must belong to the track on screen.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// audibleSpec(): AudioSpec | null { return this.staged?.previousSpec ?? this.spec; }
    /// ```
    pub(crate) fn audible_spec(&self) -> Option<AudioSpec> {
        return self.staged.as_ref().map(|staged| return staged.previous_spec).or(self.spec)
    }

    /// What:
    ///      `fn drain_hold(&self) -> Duration`.
    ///  How long after the ring ran empty the old stream must live on:
    ///  the time the output needs to play the audio it already took from the ring
    ///  (`FILL_FRAMES_FLOOR` frames at the track's rate),
    /// or zero without an output.
    /// Why:
    ///       Rebuilding the stream drops what it still holds;
    ///  waiting this long keeps the last milliseconds of the track audible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// drainHold(): number { return this.output ? framesToSecs(FILL_FRAMES_FLOOR, rate) : 0; }
    /// ```
    pub(crate) fn drain_hold(&self) -> Duration {
        if self.output.is_none() {
            return Duration::ZERO;
        }
        let rate = self.audible_spec().map_or(0, |s| return s.rate);
        return Duration::from_secs_f64(frames_to_secs(crate::output::FILL_FRAMES_FLOOR as u64, rate))
    }

    /// What:
    ///      `pub(crate) fn idle_park_timeout(&self, fallback: Duration) -> Duration`.
    ///  How long the engine thread may sleep:
    /// `fallback`,
    /// or less while an end-of-track hold is running.
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
    ///  Emit the audible position when due,
    /// announce a staged track once it is audible,
    ///  and start the next track once the last frame of an old one was consumed.
    ///  Returns whether it changed the loaded track (work was done).
    ///  `pub(crate)` so `engine::run` can call it each loop iteration.
    /// Why:
    ///       Position follows the speakers,
    /// not the decoder,
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
