//! The 60fps frame recorder:
//!  a steady,
//!  drift-free capture timer decoupled from the app.
//!
//! On each tick (on the render thread) it reads the current framebuffer back into a pooled
//! buffer and hands it to the encoder pool,
//!  then sends frame callbacks so an animating app
//! keeps drawing.
//!  It never calls `submit`,
//!  so the parent compositor's vsync cannot throttle
//! the capture cadence,
//!  and it composites whatever the app LAST committed,
//!  so a laggy app
//! merely yields repeated frames rather than stalling the sequence.
//!  The schedule is
//! absolute (start + n*period) with resync,
//!  so it does not drift.
//!  The one other sender of frame callbacks while a recording runs is `screen::apply`,
//!  which answers the pending requests right after a new size or scale,
//!  so a client can draw the new size before the next capture.

/// What:
///      Grouped `use` of paths,
///  the free-standing directory creator,
///  and time types.
/// Why:
///       The recorder creates the output directory and schedules absolute deadlines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import fs from "node:fs";
/// ```
use std::{
    fs::create_dir_all,
    path::PathBuf,
    time::{Duration, Instant},
};

/// What:
///      Grouped `use` of the calloop timer,
///  timeout action,
///  and registration token.
/// Why:
///       The recorder registers a rescheduling timer source and removes it on stop.
use smithay::reexports::calloop::{
    timer::{TimeoutAction, Timer},
    RegistrationToken,
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       `start` returns `Result` and annotates directory / timer failures.
use anyhow::{Context, Result};

/// What:
///      `use tracing::{info, warn};`.
///  Log macros.
/// Why:
///       Report recording start/stop and per-frame readback failures.
use tracing::{info, warn};

/// What:
///      `use crate::{encoder::{EncoderPool, Format, Frame}, render, screenshot,
///           state::Compositor};`.
///  The encode pool,
///  the format,
///  the frame carrier,
///  the
///           frame-callback helper,
///  the readback,
///  and the state.
/// Why:
///       A tick reads a frame,
///  submits it to the pool,
///  and sends frame callbacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EncoderPool, Format, Frame } from "./encoder"; import * as screenshot from "./screenshot";
/// ```
use crate::{
    encoder::{EncoderPool, Format, Frame},
    render, screenshot,
    state::Compositor,
};

/// Fewest encoder workers to spawn.
///
/// What:
///      `const MIN_WORKERS: usize = 2;`.
///  Floor on the pool size.
/// Why:
///       Even on a small machine,
///  at least two encoders keep PNG throughput up.
const MIN_WORKERS: usize = 2;

/// Cores left free for the app,
///  render thread,
///  and OS when sizing the encoder pool.
///
/// What:
///      `const RESERVED_CORES: usize = 2;`.
///  Subtracted from the core count.
/// Why:
///       Do not spawn so many encoders that the render thread and app are starved.
const RESERVED_CORES: usize = 2;

/// In-flight buffers per worker (bounds memory and provides queue slack).
///
/// What:
///      `const BUFFERS_PER_WORKER: usize = 3;`.
///  Multiplier for the buffer pool.
/// Why:
///       Enough slack to absorb short encode stalls without dropping,
///  but bounded.
const BUFFERS_PER_WORKER: usize = 3;

/// Summary statistics returned when recording stops.
///
/// What:
///      `pub struct RecordStats { ... }`.
///  Captured/dropped/failed frame counts,
///  the
///           elapsed seconds,
///  and the achieved average fps.
/// Why:
///       Make the 60fps claim measurable:
///  the caller (and tests) read these back.
pub struct RecordStats {
    /// Frames successfully handed to the encoder pool.
    pub captured: u64,
    /// Frames dropped because the pool had no free buffer or its queue was full.
    pub dropped: u64,
    /// Frames the encoder pool failed to write.
    pub failures: u64,
    /// Wall-clock recording duration in seconds.
    pub seconds: f64,
    /// Achieved average capture rate (captured / seconds).
    pub fps: f64,
}

/// The running recorder:
///  the capture schedule,
///  the encoder pool,
///  and counters.
///
/// What:
///      `pub struct Recorder { ... }`.
///  Held in `Compositor::recorder` while recording.
/// Why:
///       Carries everything a tick needs and everything `stop` must tear down.
pub struct Recorder {
    /// Interval between captures (1 / fps).
    period: Duration,
    /// Absolute deadline of the next capture (drift-free schedule).
    next_tick: Instant,
    /// When recording started,
    ///  for the duration report.
    started: Instant,
    /// The capture timer's registration token,
    ///  removed on stop.
    token: RegistrationToken,
    /// The parallel encoder worker pool.
    pool: EncoderPool,
    /// Next frame index (filename sequence number).
    frame_index: u64,
    /// Running captured-frame count.
    captured: u64,
    /// Running dropped-frame count.
    dropped: u64,
}

/// Start recording to `dir` at `fps` frames per second in `format`.
///
/// What:
///      `pub fn start(state: &mut Compositor, dir: PathBuf, fps: f64, format: Format)
///           -> Result<()>`.
///  Creates the directory,
///  sizes and spawns the encoder pool,
///           registers the capture timer,
///  and stores the `Recorder` in the state.
/// Why:
///       The `record` control command's implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function start(state, dir, fps, format): void { ... }
/// ```
pub fn start(state: &mut Compositor, dir: PathBuf, fps: f64, format: Format) -> Result<()> {
    // What:     `if state.recorder.is_some() { anyhow::bail!(...); }`. Reject a second
    //           concurrent recording.
    // Why:      One recorder at a time keeps the model simple.
    if state.recorder.is_some() {
        anyhow::bail!("already recording");
    }

    // What:     `if !(fps > 0.0 && fps <= 1000.0) { anyhow::bail!(...); }`. Bound the rate.
    // Why:      A non-positive or absurd fps would make a nonsense schedule.
    if !(fps > 0.0 && fps <= 1000.0) {
        anyhow::bail!("fps must be between 0 and 1000");
    }

    // What:     `create_dir_all(&dir).with_context(...)?;`. Ensure the output directory
    //           exists (creating parents).
    // Why:      Workers write frame files into it.
    create_dir_all(&dir).with_context(|| format!("creating capture directory {}", dir.display()))?;

    // What:     `let workers = worker_count();`. Choose the encoder thread count.
    // Why:      Size the pool to the machine, leaving cores for the app and render thread.
    let workers = worker_count();

    // What:     `let pool_buffers = workers * BUFFERS_PER_WORKER;`. Buffer pool size.
    // Why:      Bound in-flight memory while giving the queue slack.
    let pool_buffers = workers * BUFFERS_PER_WORKER;

    // What:     `let pool = EncoderPool::new(dir, format, workers, pool_buffers);`. Spawn
    //           the encoder workers (this consumes `dir`).
    // Why:      Stand up the off-thread encode stage.
    let pool = EncoderPool::new(dir, format, workers, pool_buffers);

    // What:     `let period = Duration::from_secs_f64(1.0 / fps);`. The tick interval.
    // Why:      Drive the timer at the requested rate.
    let period = Duration::from_secs_f64(1.0 / fps);

    // What:     `let token = state.loop_handle.insert_source(Timer::from_duration(period),
    //           |_, _, state| tick(state)).map_err(...)?;`. Register the rescheduling timer;
    //           the callback runs `tick` on the render thread.
    // Why:      The steady capture clock.
    let token = state
        .loop_handle
        .insert_source(Timer::from_duration(period), |_, _, event_state: &mut Compositor| {
            return tick(event_state)
        })
        .map_err(|err| anyhow::anyhow!("registering the capture timer failed: {err}"))?;

    // What:     `let now = Instant::now();`. The schedule origin.
    // Why:      Absolute deadlines are computed from here.
    let now = Instant::now();

    // What:     `state.recorder = Some(Recorder { ... });`. Store the running recorder.
    // Why:      Ticks and `stop` read it back from the state.
    state.recorder = Some(Recorder {
        period,
        next_tick: now + period,
        started: now,
        token,
        pool,
        frame_index: 0,
        captured: 0,
        dropped: 0,
    });

    // What:     `info!(...)`. Announce the recording.
    // Why:      Make the configuration visible.
    info!("recording started: {fps} fps, {workers} encoder workers, format {format:?}");

    // What:     `Ok(())`. Success.
    // Why:      Recording is live.
    return Ok(())
}

/// Stop recording:
///  remove the timer,
///  drain and join the encoder pool,
///  and report stats.
///
/// What:
///      `pub fn stop(state: &mut Compositor) -> Option<RecordStats>`.
///  `None` when not
///           recording.
/// Why:
///       The `record stop` command's implementation.
pub fn stop(state: &mut Compositor) -> Option<RecordStats> {
    // What:     `let recorder = state.recorder.take()?;`. Move the recorder out; `?` returns
    //           `None` if not recording.
    // Why:      Consume it to tear it down.
    let recorder = state.recorder.take()?;

    // What:     `state.loop_handle.remove(recorder.token);`. Remove the capture timer source.
    // Why:      Stop further ticks.
    state.loop_handle.remove(recorder.token);

    // What:     `let seconds = recorder.started.elapsed().as_secs_f64();`. Elapsed time.
    // Why:      For the duration and achieved-fps report.
    let seconds = recorder.started.elapsed().as_secs_f64();

    // What:     `let failures = recorder.pool.shutdown();`. Close the queue, join workers,
    //           and get the encode-failure count.
    // Why:      Ensure all queued frames are written before returning.
    let failures = recorder.pool.shutdown();

    // What:     `state.backend.window().request_redraw();`. Resume visible rendering.
    // Why:      The live redraw loop was suppressed during recording; kick it back on.
    state.backend.window().request_redraw();

    // What:     `let fps = recorder.captured as f64 / seconds.max(f64::MIN_POSITIVE);`.
    //           Achieved rate, guarding against divide-by-zero.
    // Why:      Report the measured capture rate.
    let fps = recorder.captured as f64 / seconds.max(f64::MIN_POSITIVE);

    // What:     `Some(RecordStats { ... })`. Bundle the stats (tail expression).
    // Why:      Hand them back for the control response.
    return Some(RecordStats {
        captured: recorder.captured,
        dropped: recorder.dropped,
        failures,
        seconds,
        fps,
    })
}

/// One capture tick:
///  read the frame,
///  submit it,
///  keep the app animating,
///  and reschedule.
///
/// What:
///      `fn tick(state: &mut Compositor) -> TimeoutAction`.
///  Runs on each timer fire.
/// Why:
///       Moves the recorder out of the state so the readback can borrow the rest of the
///           state,
///  then puts it back and returns the next deadline.
fn tick(state: &mut Compositor) -> TimeoutAction {
    // What:     `let Some(mut recorder) = state.recorder.take() else { return
    //           TimeoutAction::Drop; };`. Take ownership of the recorder; if it is gone
    //           (stopped), drop the timer.
    // Why:      Separating `recorder` from `state` lets `read_frame` borrow `state` freely.
    let Some(mut recorder) = state.recorder.take() else {
        return TimeoutAction::Drop;
    };

    // What:     `let action = recorder.capture_and_reschedule(state);`. Do the work.
    // Why:      Encapsulate the per-tick logic on the recorder.
    let action = recorder.capture_and_reschedule(state);

    // What:     `state.recorder = Some(recorder);`. Put the recorder back.
    // Why:      Keep it available for the next tick and for `stop`.
    state.recorder = Some(recorder);

    // What:     `action`. The next-deadline instruction (tail expression).
    // Why:      Tell calloop when to fire again.
    return action
}

/// Per-tick capture logic for the recorder.
///
/// What:
///      `impl Recorder { ... }`.
///  The capture-and-reschedule step and the single-frame
///           readback+submit.
/// Why:
///       Group the tick behaviour on the recorder value moved out of the state.
impl Recorder {
    /// Capture one frame (if a buffer is free),
    ///  send frame callbacks,
    ///  and reschedule.
    ///
    /// What:
    ///      `fn capture_and_reschedule(&mut self, state: &mut Compositor) ->
    ///           TimeoutAction`.
    ///  `self` (the recorder) and `state` are now disjoint owned
    ///           values,
    ///  so the readback can borrow `state` mutably.
    /// Why:
    ///       The body of a tick.
    fn capture_and_reschedule(&mut self, state: &mut Compositor) -> TimeoutAction {
        // What:     `match self.pool.take_buffer() { Some(buffer) => ..., None => ... }`.
        //           Take a free buffer; `None` means every buffer is in flight (encoders are
        //           behind).
        // Why:      Never block the render thread waiting for a buffer.
        match self.pool.take_buffer() {
            Some(mut buffer) => self.capture_into(state, &mut buffer),
            None => {
                // What:     `self.dropped += 1;`. Count the dropped frame.
                // Why:      Preserve cadence rather than stall; the drop count reports the
                //           encode shortfall.
                self.dropped += 1;
            }
        }

        // What:     `render::send_frame_callbacks(state);`. Tell the app its last frame was
        //           shown so it draws the next one.
        // Why:      Keep an animating app producing frames at the capture rate.
        render::send_frame_callbacks(state);

        // What:     `let _ = state.display_handle.flush_clients();`. Flush queued events.
        // Why:      Deliver the frame callbacks.
        let _ = state.display_handle.flush_clients();

        // What:     `let now = Instant::now(); self.next_tick += self.period; if
        //           self.next_tick <= now { self.next_tick = now + self.period; }`. Advance
        //           the absolute schedule, resyncing if we fell a whole period behind.
        // Why:      Steady cadence without accumulating drift or a backlog of catch-up ticks.
        let now = Instant::now();
        self.next_tick += self.period;
        if self.next_tick <= now {
            self.next_tick = now + self.period;
        }

        // What:     `TimeoutAction::ToInstant(self.next_tick)`. Fire again at the absolute
        //           next deadline (tail expression).
        // Why:      Hold the target rate precisely.
        return TimeoutAction::ToInstant(self.next_tick)
    }

    /// Read one frame into `buffer` and submit it to the encoder pool.
    ///
    /// What:
    ///      `fn capture_into(&mut self, state: &mut Compositor, buffer: &mut Vec<u8>)`.
    ///           On success builds a `Frame` and submits it;
    ///  on any failure returns the
    ///           buffer to the pool and counts a drop.
    /// Why:
    ///       Keep the buffer accounting in one place.
    fn capture_into(&mut self, state: &mut Compositor, buffer: &mut Vec<u8>) {
        // What:     `match screenshot::read_frame(state, buffer) { Ok((w, h)) => ..., Err(err)
        //           => ... }`. Render + read back into the pooled buffer.
        // Why:      Fill the buffer or handle a readback failure.
        match screenshot::read_frame(state, buffer) {
            Ok((width, height)) => {
                // What:     `let frame = Frame { index: self.frame_index, width, height,
                //           pixels: std::mem::take(buffer) };`. Move the filled bytes out of
                //           the borrowed buffer into an owned `Frame` (`std::mem::take`
                //           swaps in an empty Vec).
                // Why:      The pool owns the buffer while encoding; `take` avoids a copy.
                let frame = Frame {
                    index: self.frame_index,
                    width,
                    height,
                    pixels: std::mem::take(buffer),
                };

                // What:     `self.frame_index += 1;`. Advance the sequence number.
                // Why:      Next frame gets the next index.
                self.frame_index += 1;

                // What:     `match self.pool.submit(frame) { Ok(()) => self.captured += 1,
                //           Err(frame) => { self.dropped += 1; self.pool.return_buffer(
                //           frame.pixels); } }`. Enqueue; on a full queue drop and recycle.
                // Why:      Backpressure without blocking; recycle the buffer on drop.
                match self.pool.submit(frame) {
                    Ok(()) => self.captured += 1,
                    Err(rejected_frame) => {
                        self.dropped += 1;
                        self.pool.return_buffer(rejected_frame.pixels);
                    }
                }
            }
            Err(err) => {
                // What:     `warn!(...); self.pool.return_buffer(std::mem::take(buffer));
                //           self.dropped += 1;`. Log, recycle the buffer, count a drop.
                // Why:      A readback failure should not leak the buffer or break cadence.
                warn!("frame readback failed: {err:#}");
                self.pool.return_buffer(std::mem::take(buffer));
                self.dropped += 1;
            }
        }
    }
}

/// Choose the encoder worker count for this machine.
///
/// What:
///      `fn worker_count() -> usize`.
///  Available parallelism minus reserved cores,
///           floored at `MIN_WORKERS`.
/// Why:
///       Scale encoding to the host without starving the render thread and app.
fn worker_count() -> usize {
    // What:     `let cores = std::thread::available_parallelism().map(|n| n.get())
    //           .unwrap_or(MIN_WORKERS);`. Core count, or the floor if it cannot be queried.
    //           `n.get()` unwraps the `NonZero<usize>`.
    // Why:      The basis for the pool size.
    let cores = std::thread::available_parallelism()
        .map(|n| return n.get())
        .unwrap_or(MIN_WORKERS);

    // What:     `cores.saturating_sub(RESERVED_CORES).max(MIN_WORKERS)`. Reserve cores, then
    //           enforce the floor. `saturating_sub` avoids underflow on tiny machines.
    // Why:      Leave headroom while guaranteeing at least `MIN_WORKERS` encoders.
    return cores.saturating_sub(RESERVED_CORES).max(MIN_WORKERS)
}
