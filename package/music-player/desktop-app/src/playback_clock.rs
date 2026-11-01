//! The audible playback clock: which second of the track the listener hears now.
//!  Pure arithmetic over four numbers (frames pushed into the ring buffer, frames still waiting in it,
//!  the track frame the ring started at, and the last position that was reported); no audio device and no threads.
//! Why:
//!  The decoder runs ahead of the speakers by up to one full ring (one second).
//!  Counting pushed frames as played put the seek bar a second ahead of the sound and let the next track start
//!  while the old track's last second was still unplayed.
//!  The ring's own read side is the only place that knows what the output has actually taken,
//!  so the clock subtracts the frames still queued there from the frames pushed.
//!
//! Mental model for a TypeScript reader:
//!  `playedFrames = base + pushed - queued`,
//!  where `queued` is `ring.length` read lock-free from the producer end.
//!  No realtime code is involved: the output callback already advances the ring's read index,
//!  and the engine thread only reads it.

use std::time::{Duration, Instant};

use crate::playback::frames_to_secs;

/// What:
///      `pub(crate) const POSITION_EMIT_INTERVAL_SECS: f64 = 0.1;`.
///  Minimum seconds of audible progress between two `Position` updates.
///  `f64` matches the time contract.
/// Why:
///       Throttle position updates to about ten per second instead of one per output callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POSITION_EMIT_INTERVAL_SECS = 0.1;
/// ```
pub(crate) const POSITION_EMIT_INTERVAL_SECS: f64 = 0.1;

/// What:
///      `pub(crate) struct PlaybackClock { ... }`.
///  The counters behind the audible position of the current ring buffer.
///  `Copy` because it is a handful of small numbers.
/// Why:
///       One value holds everything the position and the track-end decision need,
///  so the controller keeps a single field instead of loose counters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlaybackClock = { baseFrames: number; pushedFrames: number; consumedOffset: number; boundaryFrames: number | null; lastEmitFrames: number; drainedSince: number | null; draining: boolean };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlaybackClock {
    /// What:
    ///      `base_frames: u64`.
    ///  The track frame that the first frame of the current ring stands for (zero for a fresh track,
    ///  the seek target after a seek).
    /// Why:
    ///       A seek reconfigures the output with an empty ring,
    ///  so the ring's own count starts at zero while the track position does not.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// baseFrames: number;
    /// ```
    base_frames: u64,
    /// What:
    ///      `consumed_offset: u64`.
    ///  Ring frames that belong to tracks before the current one (the pushed count at the last track boundary).
    /// Why:
    ///       Gapless playback keeps one ring across two tracks;
    ///  the new track's position counts from its first frame, not from the start of the ring.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// consumedOffset: number;
    /// ```
    consumed_offset: u64,
    /// What:
    ///      `boundary_frames: Option<u64>`.
    ///  The pushed count at which the next track's first frame sits in the ring,
    ///  or `None` when no next track was queued behind the current one.
    /// Why:
    ///       The next track is announced when the output consumes up to this frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// boundaryFrames: number | null;
    /// ```
    boundary_frames: Option<u64>,
    /// What:
    ///      `pushed_frames: u64`.
    ///  Frames written into the current ring since it was created.
    /// Why:
    ///       Pushed minus still queued is what the output has consumed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pushedFrames: number;
    /// ```
    pushed_frames: u64,
    /// What:
    ///      `last_emit_frames: u64`.
    ///  The track frame of the most recent `Position` update.
    /// Why:
    ///       The next update waits for `POSITION_EMIT_INTERVAL_SECS` of further audible progress;
    ///  counting frames keeps that comparison exact where float seconds would round.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastEmitFrames: number;
    /// ```
    last_emit_frames: u64,
    /// What:
    ///      `drained_since: Option<Instant>`.
    ///  When the ring was first seen empty during a drain, or `None` while frames are still queued.
    /// Why:
    ///       The output holds some already-consumed audio of its own;
    ///  the next track may only start after that has played out too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// drainedSince: number | null;
    /// ```
    drained_since: Option<Instant>,
    /// What:
    ///      `draining: bool`.
    ///  Whether the decoder reached the end of the track and the ring is being played out.
    /// Why:
    ///       The next track may only start once the last frame of this one was consumed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// draining: boolean;
    /// ```
    draining: bool,
}

/// What:
///      `pub(crate) fn frames_of_samples(samples: usize, channels: u16) -> u64`.
///  Convert a count of interleaved samples (pushed, or still waiting in the ring) into whole frames,
///  returning zero for an unknown channel count.
/// Why:
///       The ring counts samples,
///  the clock counts frames;
///  one helper keeps the divide-by-zero guard in a single place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function framesOfSamples(samples: number, channels: number): number {
///   return channels === 0 ? 0 : Math.floor(samples / channels);
/// }
/// ```
pub(crate) fn frames_of_samples(samples: usize, channels: u16) -> u64 {
    if channels == 0 {
        return 0;
    }
    return samples as u64 / u64::from(channels)
}

impl PlaybackClock {
    /// What:
    ///      `pub(crate) fn new() -> PlaybackClock`.
    ///  A clock at the start of a track with nothing pushed.
    /// Why:
    ///       The controller starts with no track loaded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(): PlaybackClock { return { baseFrames: 0, pushedFrames: 0, consumedOffset: 0, boundaryFrames: null, lastEmitFrames: 0, drainedSince: null, draining: false }; }
    /// ```
    pub(crate) fn new() -> PlaybackClock {
        return PlaybackClock {
            base_frames: 0,
            pushed_frames: 0,
            consumed_offset: 0,
            boundary_frames: None,
            last_emit_frames: 0,
            drained_since: None,
            draining: false,
        }
    }

    /// What:
    ///      `pub(crate) fn restart(&mut self, base_frames: u64)`.
    ///  Start counting for a new, empty ring whose first frame is track frame `base_frames`.
    ///  Clears the pushed count, any track boundary, and any drain in progress.
    /// Why:
    ///       Loading a track and seeking both rebuild the ring,
    ///  and the old ring's pending audio is dropped with it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// restart(baseFrames: number): void { ... }
    /// ```
    pub(crate) fn restart(&mut self, base_frames: u64) {
        self.base_frames = base_frames;
        self.pushed_frames = 0;
        self.consumed_offset = 0;
        self.boundary_frames = None;
        self.last_emit_frames = base_frames;
        self.drained_since = None;
        self.draining = false;
    }

    /// What:
    ///      `pub(crate) fn record_pushed(&mut self, frames: u64)`.
    ///  Count frames the decoder just wrote into the ring.
    /// Why:
    ///       The consumed count is derived from this total.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordPushed(frames: number): void { this.pushedFrames += frames; }
    /// ```
    pub(crate) fn record_pushed(&mut self, frames: u64) {
        self.pushed_frames += frames;
    }

    /// What:
    ///      `pub(crate) fn audible_frames(&self, queued_frames: u64) -> u64`.
    ///  The track frame the listener hears now:
    ///  the track frame at the start of the ring plus the frames the output consumed since the current track began,
    ///  where consumed is pushed minus `queued_frames` (never below zero).
    /// Why:
    ///       This replaces the pushed count,
    ///  which ran a whole ring ahead of the sound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// audibleFrames(queuedFrames: number): number {
    ///   const consumed = Math.max(0, this.pushedFrames - queuedFrames);
    ///   return this.baseFrames + Math.max(0, consumed - this.consumedOffset);
    /// }
    /// ```
    pub(crate) fn audible_frames(&self, queued_frames: u64) -> u64 {
        let consumed = self.pushed_frames.saturating_sub(queued_frames);
        return self.base_frames + consumed.saturating_sub(self.consumed_offset)
    }

    /// What:
    ///      `pub(crate) fn audible_secs(&self, queued_frames: u64, rate: u32) -> f64`.
    ///  The audible position in seconds (zero for an unknown rate).
    /// Why:
    ///       The session snapshot and the `Position` updates both want seconds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// audibleSecs(queuedFrames: number, rate: number): number { return framesToSecs(this.audibleFrames(queuedFrames), rate); }
    /// ```
    pub(crate) fn audible_secs(&self, queued_frames: u64, rate: u32) -> f64 {
        return frames_to_secs(self.audible_frames(queued_frames), rate)
    }

    /// What:
    ///      `pub(crate) fn position_to_emit(&mut self, queued_frames: u64, rate: u32) -> Option<f64>`.
    ///  The audible position when at least `POSITION_EMIT_INTERVAL_SECS` of progress has accumulated since the
    ///  last update,
    ///  remembering it as the new baseline;
    ///  otherwise `None`.
    /// Why:
    ///       Keeps the seek bar moving with the sound without flooding the UI.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// positionToEmit(queuedFrames: number, rate: number): number | null { ... }
    /// ```
    pub(crate) fn position_to_emit(&mut self, queued_frames: u64, rate: u32) -> Option<f64> {
        if rate == 0 {
            return None;
        }
        let frames = self.audible_frames(queued_frames);
        let interval_frames = (POSITION_EMIT_INTERVAL_SECS * f64::from(rate)).round() as u64;
        if frames < self.last_emit_frames + interval_frames {
            return None;
        }
        self.last_emit_frames = frames;
        return Some(frames_to_secs(frames, rate))
    }

    /// What:
    ///      `pub(crate) fn mark_boundary(&mut self)`.
    ///  Record that the next track's first frame is the next frame the decoder pushes.
    /// Why:
    ///       The decoder moves straight on to the next track inside the same ring,
    ///  so nothing in the output changes at the join;
    ///  the clock needs the join's place to know when the next track becomes audible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// markBoundary(): void { this.boundaryFrames = this.pushedFrames; }
    /// ```
    pub(crate) fn mark_boundary(&mut self) {
        self.boundary_frames = Some(self.pushed_frames);
    }

    /// What:
    ///      `pub(crate) fn has_boundary(&self) -> bool`.
    ///  Whether a next track is queued behind the current one and not yet audible.
    /// Why:
    ///       A second join must not be marked before the first one was crossed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hasBoundary(): boolean { return this.boundaryFrames !== null; }
    /// ```
    pub(crate) fn has_boundary(&self) -> bool {
        return self.boundary_frames.is_some()
    }

    /// What:
    ///      `pub(crate) fn boundary_reached(&self, queued_frames: u64) -> bool`.
    ///  Whether the output consumed every frame before the join, so the next track is audible now.
    /// Why:
    ///       This is the moment to announce the next track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// boundaryReached(queuedFrames: number): boolean { return this.boundaryFrames !== null && this.pushedFrames - queuedFrames >= this.boundaryFrames; }
    /// ```
    pub(crate) fn boundary_reached(&self, queued_frames: u64) -> bool {
        let consumed = self.pushed_frames.saturating_sub(queued_frames);
        return self.boundary_frames.is_some_and(|boundary| return consumed >= boundary)
    }

    /// What:
    ///      `pub(crate) fn cross_boundary(&mut self)`.
    ///  The next track became audible:
    ///  count its position from its first frame, and forget the join.
    /// Why:
    ///       The ring keeps running, so the new track's position is the frames consumed past the join.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// crossBoundary(): void { ... }
    /// ```
    pub(crate) fn cross_boundary(&mut self) {
        let Some(boundary) = self.boundary_frames.take() else {
            return;
        };
        self.consumed_offset = boundary;
        self.base_frames = 0;
        self.last_emit_frames = 0;
    }

    /// What:
    ///      `pub(crate) fn begin_drain(&mut self) -> bool`.
    ///  Mark the decoder as finished so the ring is played out;
    ///  returns whether this call started the drain (false when it was already draining).
    /// Why:
    ///       The caller logs the transition once and stops decoding.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// beginDrain(): boolean { const started = !this.draining; this.draining = true; return started; }
    /// ```
    pub(crate) fn begin_drain(&mut self) -> bool {
        let started = !self.draining;
        self.draining = true;
        return started
    }

    /// What:
    ///      `pub(crate) fn finish_drain(&mut self)`.
    ///  Clear the drain flag once the track-end handoff was taken.
    /// Why:
    ///       When no further track loads (end of queue, unreadable files) the clock must not keep reporting a drain,
    ///  or the handoff would run again on the next play.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finishDrain(): void { this.draining = false; }
    /// ```
    pub(crate) fn finish_drain(&mut self) {
        self.drained_since = None;
        self.draining = false;
    }

    /// What:
    ///      `pub(crate) fn is_draining(&self) -> bool`.
    ///  Whether the decoder reached the end of the track and the ring is being played out.
    /// Why:
    ///       Pumping stops while draining.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isDraining(): boolean { return this.draining; }
    /// ```
    pub(crate) fn is_draining(&self) -> bool {
        return self.draining
    }

    /// What:
    ///      `pub(crate) fn drain_finished(&mut self, queued_frames: u64, now: Instant, hold: Duration) -> bool`.
    ///  Whether a drain is in progress, the output consumed every pushed frame, and `hold` has passed since the ring
    ///  was first seen empty.
    ///  `now` is passed in so tests control the clock.
    /// Why:
    ///       This is the moment the last sample of the track has left the ring and the output's own buffer,
    ///  so the next track may start;
    ///  starting earlier discards the unplayed tail.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// drainFinished(queuedFrames: number, now: number, hold: number): boolean { ... }
    /// ```
    pub(crate) fn drain_finished(&mut self, queued_frames: u64, now: Instant, hold: Duration) -> bool {
        if !self.draining {
            return false;
        }
        if queued_frames != 0 {
            self.drained_since = None;
            return false;
        }
        let since = *self.drained_since.get_or_insert(now);
        return now.saturating_duration_since(since) >= hold
    }

    /// What:
    ///      `pub(crate) fn drain_wait(&self, now: Instant, hold: Duration) -> Option<Duration>`.
    ///  The time left of the hold after the ring was seen empty, or `None` when no hold is running.
    /// Why:
    ///       The engine thread sleeps at most this long instead of its idle timeout, so the hold ends on time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// drainWait(now: number, hold: number): number | null { ... }
    /// ```
    pub(crate) fn drain_wait(&self, now: Instant, hold: Duration) -> Option<Duration> {
        let since = self.drained_since?;
        return Some(hold.saturating_sub(now.saturating_duration_since(since)))
    }
}

impl Default for PlaybackClock {
    /// What:
    ///      `fn default() -> PlaybackClock`.
    ///  Same as `PlaybackClock::new`.
    /// Why:
    ///       Lets callers build the clock with `Default`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): PlaybackClock { return PlaybackClock.create(); }
    /// ```
    fn default() -> PlaybackClock {
        return PlaybackClock::new()
    }
}

#[cfg(test)]
#[path = "playback_clock_tests.rs"]
mod tests;
