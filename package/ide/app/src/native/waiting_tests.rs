//! Bounded waiting for the native window tests:
//!  advancing native timers and animations until a state the test checks holds.
//!
//! A test writes the check itself,
//!  as the condition of a `while` loop,
//!  and calls `Waiting::step` in the body.
//!  A condition that never holds fails the test with the message given to `step`.

/// The window and the source state whose facts a failed wait names.
use super::{AppWindow, State, tile_build::FAIL_NEXT_BUILD};
/// Native timers and animations run only when the test advances them.
use slint::platform::update_timers_and_animations;
/// Deadlines and pauses between checks.
use std::{
    cell::RefCell,
    fmt::{self, Display},
    rc::Rc,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

/// Longest wait for most expected states.
const PATIENCE: Duration = Duration::from_secs(5);

/// Longest wait for a native language test to see its expected state.
const LANGUAGE_PATIENCE: Duration = Duration::from_secs(10);

/// Longest wait for a window to take over a tile built for the wrapped view.
const SLOW_PATIENCE: Duration = Duration::from_secs(90);

/// Every observed change must arrive within this bound.
///  Each change is made right after a reread of the same item was observed,
///  so without a notification its next reread is the following 1 s safety sweep,
///  about 800 ms later for five shown directories;
///  only the first change of a test can meet a sweep by chance.
const PROMPT_PATIENCE: Duration = Duration::from_millis(400);

/// The message of a navigation wait that timed out.
pub(super) const NAVIGATION_UNREACHED: &str = "native navigation did not reach the expected state";

/// Pause between checks of a state.
const PAUSE: Duration = Duration::from_millis(2);

/// Pause between checks when the wait itself is measured.
const FINE_PAUSE: Duration = Duration::from_millis(1);

/// What:
///  One wait for a state:
///  when it began,
///  how long it may last,
///  and how long to pause between checks.
/// Why:
///  The check stays in the test as a plain condition,
///  and the clock,
///  the pause,
///  and the failure live here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Waiting { constructor(readonly started: number, readonly patience: number, readonly pause: number) {} }
/// ```
pub(super) struct Waiting {
    /// When the wait began.
    started: Instant,
    /// Longest time the wait may last.
    patience: Duration,
    /// Time to sleep between checks.
    pause: Duration,
}

impl Waiting {
    /// What:
    ///  Begin a wait that may last five seconds,
    ///  after one pass of the native timers.
    /// Why:
    ///  The first check must see the effect of timers already due.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function eventually(): Waiting { updateTimers(); return new Waiting(now(), 5_000, 2); }
    /// ```
    pub(super) fn eventually() -> Waiting {
        return Waiting::begin(PATIENCE, PAUSE);
    }

    /// Begin a wait that may last ten seconds,
    ///  after one pass of the native timers.
    pub(super) fn language() -> Waiting {
        return Waiting::begin(LANGUAGE_PATIENCE, PAUSE);
    }

    /// Begin a wait that may last ninety seconds,
    ///  after one pass of the native timers.
    pub(super) fn slow() -> Waiting {
        return Waiting::begin(SLOW_PATIENCE, PAUSE);
    }

    /// Begin a wait for a change that must arrive within a fraction of a second,
    ///  after one pass of the native timers.
    pub(super) fn prompt() -> Waiting {
        return Waiting::begin(PROMPT_PATIENCE, PAUSE);
    }

    /// Begin a wait that may last `patience`,
    ///  after one pass of the native timers.
    pub(super) fn within(patience: Duration) -> Waiting {
        return Waiting::begin(patience, PAUSE);
    }

    /// Begin a wait that may last `patience` and checks about once per millisecond,
    ///  after one pass of the native timers.
    pub(super) fn closely(patience: Duration) -> Waiting {
        return Waiting::begin(patience, FINE_PAUSE);
    }

    /// Start the clock and then run the native timers once,
    ///  so the first pass counts against the patience.
    fn begin(patience: Duration, pause: Duration) -> Waiting {
        let started = Instant::now();
        update_timers_and_animations();
        return Waiting {
            started,
            patience,
            pause,
        };
    }

    /// What:
    ///  Fail with `message` when the wait has lasted its patience,
    ///  otherwise sleep for the pause and run the native timers again.
    /// Why:
    ///  Called after a check found the state missing,
    ///  so the next check sees what the timers produced.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// step(message: string): void {
    ///   if (now() - this.started >= this.patience) throw new Error(message);
    ///   sleep(this.pause); updateTimers();
    /// }
    /// ```
    pub(super) fn step(&mut self, message: impl Display) {
        assert!(self.started.elapsed() < self.patience, "{message}");
        std::thread::sleep(self.pause);
        update_timers_and_animations();
    }

    /// Like `step`,
    ///  with the message "`what` did not appear within" the patience.
    pub(super) fn step_missing(&mut self, what: impl Display) {
        let patience = self.patience;
        self.step(format_args!("{what} did not appear within {patience:?}"));
    }

    /// Time since the wait began.
    pub(super) fn elapsed(&self) -> Duration {
        return self.started.elapsed();
    }
}

/// What:
///  The tile and worker facts of a window,
///  written when a failure message is formatted.
/// Why:
///  A flake that cannot be reproduced on demand must name,
///  at the moment it fails,
///  which tile was shown,
///  which
///  job and re-center were pending,
///  how many tiles each side had built,
///  and whether the failure switch was
///  consumed,
///  or the next occurrence says only that a number differed.
///  The text is built only when the assertion fails,
///  so a passing wait pays nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Facts { toString(): string { return describe(this.window, this.source); } }
/// ```
pub(super) struct Facts<'a> {
    /// The window whose properties are read.
    pub(super) window: &'a AppWindow,
    /// The source state of that window.
    pub(super) source: &'a Rc<RefCell<State>>,
}

/// The facts as one line.
impl Display for Facts<'_> {
    /// Write the facts,
    ///  or say that the state was borrowed.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ok(current) = self.source.try_borrow() else {
            return write!(formatter, "[state borrowed]");
        };
        return write!(
            formatter,
            "[scroll_y {} offset {} image_y {} first {} count {} shown {}+{} tile_job {} scroll_wanted {} \
             renders {} tiles_painted {} images_set {} tiles_rendered {} gpu_pending {} worker_tiles {} \
             applied {} discarded {} worker_failures {} fail_next_build {} error {:?}]",
            self.window.get_scroll_y(),
            current.offset,
            self.window.get_image_y(),
            current.first,
            current.count,
            current.shown.first,
            current.shown.count,
            current.tile_job.is_some(),
            current.scroll_wanted.is_some(),
            current.renders,
            current.tiles_painted,
            current.images_set,
            current.gpu.tiles_rendered(),
            current.gpu.has_pending(),
            current.worker_tiles,
            current.tiles_applied,
            current.tiles_discarded,
            current.worker_failures.len(),
            FAIL_NEXT_BUILD.load(Ordering::SeqCst),
            self.window.get_error_message().to_string(),
        );
    }
}
