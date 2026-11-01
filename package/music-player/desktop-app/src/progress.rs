//! Progress-surface debounce helpers.
//!
//! The engine already throttles media-position messages by playback seconds,
//!  but
//! short or zero-duration tracks can still create rapid progress resets.
//!  This
//! module keeps the UI seek bar and the desktop taskbar progress from being
//! repainted for every reset while still allowing play-state changes through
//! immediately.

/// What:
///      `use std::time::Duration;`.
///  A standard-library span of time.
///  Siblings a
///           TS reader might expect:
///  `Instant` (a monotonic timestamp,
///  an absolute
///           point) and a raw integer count of milliseconds.
/// Why:
///       Debouncing compares elapsed spans;
///  `Duration` keeps the unit explicit
///           and avoids mixing seconds with milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a Duration is just a number of milliseconds in TS
/// ```
use std::time::Duration;

/// What:
///      `pub const PROGRESS_UPDATE_DEBOUNCE_INTERVAL: Duration = Duration::from_millis(250);`.
///           A public constant holding the minimum gap between ordinary progress
///           repaints.
///  `Duration::from_millis` builds the time span from an integer
///           millisecond count,
///  evaluated at compile time.
/// Why:
///       A quarter-second cap is fast enough for a seek bar but slow enough to
///           prevent sub-second tracks from flashing the UI and taskbar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PROGRESS_UPDATE_DEBOUNCE_INTERVAL = 250; // ms
/// ```
pub const PROGRESS_UPDATE_DEBOUNCE_INTERVAL: Duration = Duration::from_millis(250);

// What:     `#[derive(Clone, Copy, Debug, Eq, PartialEq)]` auto-implements five
//           traits for the enum below: `Clone`/`Copy` (it is a trivial value that
//           can be duplicated by bit-copy, so passing it never moves it away),
//           `Debug` (`{:?}` printing), and `Eq`/`PartialEq` (so `==` works, used in
//           `should_surface`).
// Why:      The kind is compared with `==` and copied freely; deriving these makes
//           it behave like a plain enum value rather than something you must clone.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation: "debounced" | "immediate" is already == comparable
// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// What:
///      `pub enum ProgressUpdateKind { ... }` declares which debounce rule an
///           update should use.
///  It is public so the binary can classify Slint updates
///           before asking the debouncer.
/// Why:
///       Position ticks can wait;
///  play/pause visibility changes must surface
///           immediately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProgressUpdateKind = "debounced" | "immediate";
/// ```
pub enum ProgressUpdateKind {
    /// What:
    ///      `Debounced` a fieldless enum variant (carries no data).
    /// Why:
    ///       Ordinary progress movement or track-reset progress that may be
    ///           rate-limited.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "debounced"
    /// ```
    Debounced,
    /// What:
    ///      `Immediate` a fieldless enum variant.
    /// Why:
    ///       State transitions that must update visible/hidden taskbar state now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "immediate"
    /// ```
    Immediate,
}

// What:     `#[derive(Debug, Default)]` auto-implements `Debug` (`{:?}` printing) and
//           `Default` (a zero-argument constructor that fills each field with its
//           own default, here `None`) for the struct below.
// Why:      `Default` gives the empty-state constructor `new()` reuses, and `Debug`
//           helps test failure output.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation; the constructor below seeds lastSurfaceAt = null
// ```
#[derive(Debug, Default)]
/// What:
///      `pub struct ProgressDebouncer { ... }` stores the last time a progress
///           surface was allowed to repaint.
/// Why:
///       One small state object gates both the Slint seek bar and taskbar
///           progress through the same timing rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProgressDebouncer { lastSurfaceAt: number | null = null; }
/// ```
pub struct ProgressDebouncer {
    /// What:
    ///      `last_surface_at: Option<Duration>` remembers the elapsed time of the
    ///           last accepted progress update.
    ///  `Option<Duration>` is either
    ///           `Some(duration)` or `None`;
    ///  `None` means no update has surfaced yet
    ///           (Rust has no `null`,
    ///  so absence is an `Option`).
    /// Why:
    ///       The next debounced update compares against this baseline.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastSurfaceAt: number | null;
    /// ```
    last_surface_at: Option<Duration>,
}

/// What:
///      `impl ProgressDebouncer { ... }` defines methods on the debounce state
///           object (an `impl` block is where a type's methods live).
/// Why:
///       Keep the timing rule next to the state it mutates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProgressDebouncer { /* methods */ }
/// ```
impl ProgressDebouncer {
    /// What:
    ///      `pub fn new() -> Self`.
    ///  Build a fresh debouncer.
    ///  `Self` is an alias
    ///           for `ProgressDebouncer` inside this impl block.
    /// Why:
    ///       Callers should not construct the internal field directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor() { this.lastSurfaceAt = null; }
    /// ```
    pub fn new() -> Self {
        // What:     `Self::default()` calls the derived `Default`, which sets
        //           `last_surface_at` to `None`. Tail expression -> return value.
        // Why:      Reuse the generated empty-state constructor instead of repeating
        //           the field.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new ProgressDebouncer();
        // ```
        return Self::default()
    }

    /// What:
    ///      `pub fn should_surface(&mut self, now: Duration, kind: ProgressUpdateKind) -> bool`.
    ///           `&mut self` borrows the debouncer MUTABLY (this method updates the
    ///           stored baseline);
    ///  `now` is the current elapsed span;
    ///  `kind` is the
    ///           update classification;
    ///  returns a `bool` decision.
    /// Why:
    ///       This is the single debounce decision used by UI and taskbar code.
    /// Gotcha:
    ///    `&mut self` means only one caller may hold this borrow at a time;
    ///           there is no shared-mutable aliasing like a plain JS method has.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shouldSurface(now: number, kind: ProgressUpdateKind): boolean { ... }
    /// ```
    pub fn should_surface(&mut self, now: Duration, kind: ProgressUpdateKind) -> bool {
        // What:     `if kind == ProgressUpdateKind::Immediate { ... }`. Compare the
        //           passed kind against the immediate variant with `==` (available
        //           because the enum derives `PartialEq`).
        // Why:      Play/pause visibility changes should never be held back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (kind === "immediate") { ... }
        // ```
        if kind == ProgressUpdateKind::Immediate {
            // What:     `self.last_surface_at = Some(now);`. Store this accepted update
            //           time, wrapped in `Some` (the present case of `Option`).
            // Why:      A following position reset still waits for the debounce gap.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.lastSurfaceAt = now;
            // ```
            self.last_surface_at = Some(now);
            // What:     `return true;`. Early return leaving the function immediately.
            // Why:      The caller should repaint now.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return true;
            // ```
            return true;
        }

        // What:     `let interval_elapsed = match self.last_surface_at { ... };`. A
        //           `match` EXPRESSION over the `Option` baseline, assigned to a bool.
        // Why:      The first debounced update should surface; later ones must wait.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const intervalElapsed = this.lastSurfaceAt === null
        //   ? true
        //   : now - this.lastSurfaceAt >= PROGRESS_UPDATE_DEBOUNCE_INTERVAL;
        // ```
        let interval_elapsed = match self.last_surface_at {
            // What:     `Some(last_surface_at) => { now.saturating_sub(last_surface_at) >= PROGRESS_UPDATE_DEBOUNCE_INTERVAL }`.
            //           Destructure the present baseline; `saturating_sub` subtracts
            //           and returns ZERO instead of underflowing when `now` is equal
            //           to or earlier than the baseline (e.g. a clock reset); compare
            //           the gap against the interval.
            // Why:      Compare elapsed time safely without panicking on underflow.
            // Gotcha:   `saturating_sub` clamps at zero; a plain `-` on `Duration`
            //           would PANIC on underflow, unlike TS numbers going negative.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Math.max(0, now - lastSurfaceAt) >= PROGRESS_UPDATE_DEBOUNCE_INTERVAL
            // ```
            Some(last_surface_at) => {
                now.saturating_sub(last_surface_at) >= PROGRESS_UPDATE_DEBOUNCE_INTERVAL
            }
            // What:     `None => true`. No previous progress update existed.
            // Why:      The first progress value must be visible.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // true
            // ```
            None => true,
        };

        // What:     `if interval_elapsed { ... }`. Only accepted updates refresh the
        //           stored baseline.
        // Why:      Suppressed rapid updates should not push the window forward (else
        //           a flood of resets would forever postpone the next repaint).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (intervalElapsed) { this.lastSurfaceAt = now; }
        // ```
        if interval_elapsed {
            // What:     `self.last_surface_at = Some(now);`. Record this accepted
            //           repaint time, wrapped in `Some`.
            // Why:      Start a new debounce interval from here.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.lastSurfaceAt = now;
            // ```
            self.last_surface_at = Some(now);
        }

        // What:     `interval_elapsed`. The bare bool is the tail expression, so it is
        //           returned.
        // Why:      The caller receives the decision.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return intervalElapsed;
        // ```
        return interval_elapsed
    }
}

/// What:
///      `#[cfg(test)] #[path = "progress_tests.rs"] mod tests;` declares a
///           test-only submodule whose code lives in the sibling file
///           `progress_tests.rs`.
///  `#[cfg(test)]` gates it to test builds only;
///           `#[path = "..."]` aims the module at a flat sibling file instead of the
///           default `progress/tests.rs` subdirectory lookup.
///  The file stays the
///           `tests` CHILD of progress,
///  so its `use super::*` reaches the module
///           items (including private ones) unchanged.
/// Why:
///       Keep `progress.rs` to production code;
///  the tests live beside it without
///           inflating this file or its max-lines budget (sibling `*_tests.rs` files
///           are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // progress.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
