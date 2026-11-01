//! The tile worker's thread:
//!  the loop that builds jobs and carries on the whole-text pass between them,
//!  and the shaper and raster it owns.

/// The tile job and result types,
///  and the construction every build runs.
use super::job::{TileBuilt, TileJob, build_tile, same_typography};
/// The messages and shared slots the thread's owner also reads.
use super::{Command, Shared, WorkerReply};
/// The background measurement of the whole text.
use crate::native::tile_pass::{PassJob, PassOutput, PassRun};
/// Worker-start and build failures stay visible diagnostics.
use anyhow::Result;
/// Shaping choices a worker shaper must match.
use ide_app::shaped_text::TextShaper;
/// Syntax classifications,
///  compared by shared identity.
use ide_app::source_style::SourceStyles;
/// Shaping choices a worker shaper must match.
use ide_app::source_typography::SourceTypography;
/// Owned snapshots are shared between the UI and worker threads.
use std::sync::{
    Arc,
    atomic::Ordering,
    mpsc::{Receiver, SyncSender, TryRecvError},
};

/// What:
///  Most lines the worker's row cache keeps.
/// Why:
///  The whole-file pass keeps every line it shaped so that a resize breaks them again instead of shaping them again;
///  measured on 2026-10-11,
/// 50,000 kept layouts hold about 158 MB (3.2 KB a line),
/// re-breaking them takes 40 to 80 ms
///  where shaping them takes 1 to 2 s.
///  A file longer than this keeps the lines nearest the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEPT_LAYOUTS = 50_000;
/// ```
pub(in crate::native) const KEPT_LAYOUTS: usize = 50_000;

/// What:
///  The worker's own shaper,
/// plus what the shaper's row cache was built for.
/// Why:
///  The row cache keys on revision and scale and tags each row with its wrap width,
///  so a change of styles or typography must clear or replace it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Tiler { shaper: TextShaper; styles?: Styles }
/// ```
struct Tiler {
    /// Font and layout resources owned by the worker thread.
    shaper: TextShaper,
    /// Typography the shaper was built with.
    typography: SourceTypography,
    /// Syntax classifications the cached rows were shaped with,
    /// by identity.
    styles: Option<SourceStyles>,
}

/// Build one job with the worker's resources,
/// clearing its row cache when styles or typography changed.
impl Tiler {
    /// Start from a shaper forked from the window's own,
    /// so the worker shares its fonts and typography.
    fn new(mut shaper: TextShaper) -> Self {
        let typography = shaper.typography().clone();
        shaper.keep_layouts(KEPT_LAYOUTS);
        return Self {
            shaper,
            typography,
            styles: None,
        };
    }

    /// Make the shaper match `typography` and the cached rows match `styles`.
    fn adopt(&mut self, typography: &SourceTypography, styles: &SourceStyles) -> Result<()> {
        if !same_typography(&self.typography, typography) {
            self.shaper = TextShaper::with_typography(typography.clone())?;
            self.shaper.keep_layouts(KEPT_LAYOUTS);
            self.typography = typography.clone();
            self.styles = None;
        }
        // A new styles snapshot can change glyph classes without changing the revision, so cached rows are dropped.
        let same_styles = match self.styles.as_ref() {
            Some(known) => Arc::ptr_eq(known, styles),
            None => false,
        };
        if !same_styles {
            self.shaper.forget_rows();
            self.styles = Some(Arc::clone(styles));
        }
        return Ok(());
    }

    /// Build `job`,
    /// first making the shaper match its typography and styles.
    fn build(&mut self, job: TileJob) -> Result<TileBuilt> {
        self.adopt(&job.typography, &job.styles)?;
        return build_tile(&mut self.shaper, job);
    }
}

/// Test-only gate:
/// the next build waits for one message before it starts,
/// so a test can hold a job in flight.
#[cfg(test)]
pub(in crate::native) static BUILD_GATE: std::sync::Mutex<Option<Receiver<()>>> =
    std::sync::Mutex::new(None);

/// Test-only gate:
/// each chunk of the pass waits for one message before it starts,
/// so a test can release chunks
/// one at a time.
/// Closing the sender releases the pass for good.
#[cfg(test)]
pub(in crate::native) static PASS_GATE: std::sync::Mutex<Option<Receiver<()>>> =
    std::sync::Mutex::new(None);

/// Wait on the test gate once,
/// if a test installed one;
/// the sender stays with the test.
#[cfg(test)]
fn hold_at_test_gate() {
    let gate = match BUILD_GATE.lock() {
        Ok(mut slot) => slot.take(),
        Err(_poisoned) => None,
    };
    if let Some(pending) = gate
        && let Err(error) = pending.recv()
    {
        tracing::debug!(%error, "test build gate closed");
    }
}

/// Whether the next chunk of the pass may run:
/// always,
/// unless a test installed a gate and holds back its token.
/// A token lets one chunk run;
/// a closed gate lets the pass run freely.
/// The worker keeps serving tile jobs meanwhile.
#[cfg(test)]
fn pass_gate_open() -> bool {
    let Ok(mut slot) = PASS_GATE.lock() else {
        return true;
    };
    let Some(pending) = slot.take() else {
        return true;
    };
    match pending.try_recv() {
        Ok(()) => {
            *slot = Some(pending);
            return true;
        }
        Err(TryRecvError::Empty) => {
            *slot = Some(pending);
            return false;
        }
        Err(TryRecvError::Disconnected) => return true,
    }
}

/// Own the blocking receive loop entirely outside native state.
pub(super) fn run(
    shaper: TextShaper,
    requests: Receiver<Command>,
    replies: SyncSender<WorkerReply>,
    shared: Shared,
) {
    tracing::debug!("source tile worker started");
    let mut tiler = Tiler::new(shaper);
    if let Err(error) = replies.send(WorkerReply::Ready) {
        tracing::debug!(%error, "source tile reply receiver closed before the worker was ready");
        return;
    }
    let mut seen = 0_u64;
    let mut pass: Option<PassRun> = None;
    loop {
        // What: An idle worker waits for a command; a worker with a pass to carry on only glances at the channel.
        // Why: A waiting tile job always goes first, and the pass fills the time between jobs.
        let received = if pass.is_some() {
            match requests.try_recv() {
                Ok(command) => Some(command),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    tracing::debug!("source tile request channel closed");
                    return;
                }
            }
        } else {
            match requests.recv() {
                Ok(command) => Some(command),
                Err(error) => {
                    tracing::debug!(%error, "source tile request channel closed");
                    return;
                }
            }
        };
        if received.is_some() {
            shared.wake_pending.store(false, Ordering::SeqCst);
        }
        let waiting = received.is_some();
        if let Some(Command::Tile(job)) = received {
            #[cfg(test)]
            hold_at_test_gate();
            // A tile far from the walk means the reader jumped; the walk goes to the new view next.
            if let Some(run) = pass.as_mut() {
                run.refocus(job.bounds.first);
            }
            let result = tiler.build(*job);
            // A closed receiver means the window is gone; the thread stops instead of retrying.
            if let Err(error) = replies.send(WorkerReply::Built(Box::new(result))) {
                tracing::debug!(%error, "source tile reply receiver closed");
                return;
            }
        }
        if let Some(job) = shared.control.take_newer(&mut seen) {
            match tiler.adopt(&job.typography, &job.styles) {
                Ok(()) => pass = Some(PassRun::new(job)),
                Err(error) => {
                    tracing::error!(%error, "source pass could not adopt its typography");
                    pass = None;
                }
            }
        }
        if waiting {
            // Look for another command before spending time on a chunk.
            continue;
        }
        if let Some(run) = pass.as_mut() {
            #[cfg(test)]
            if !pass_gate_open() {
                // A held-back chunk waits for its token without keeping tile jobs waiting.
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            let output = run.step(&mut tiler.shaper);
            let finished = output.done;
            shared.outbox.push(output);
            if finished {
                pass = None;
            }
        }
    }
}

/// Compiles only for a type that is `Send`.
fn assert_send<T: Send>() {}

/// Names every type that crosses to the worker thread.
fn assert_crossing_types_are_send() {
    assert_send::<TileJob>();
    assert_send::<PassJob>();
    assert_send::<PassOutput>();
    assert_send::<Result<TileBuilt>>();
    assert_send::<TextShaper>();
}

/// Compile-time proof that jobs and results may cross to the worker thread.
const _: fn() = assert_crossing_types_are_send;
