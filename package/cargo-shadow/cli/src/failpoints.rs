//! What:
//! Test-only switches that make one named step fail once, on the thread that set them.
//! Why:
//! Review finding (4c): rollback and finalization failures must be provoked deterministically, also by a process that ignores
//! directory permissions (a root user), so the refusal-precedence tests do not skip.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // failpoints.ts (test builds only)
//! export function arm(point: 'restore' | 'finish'): void
//! export function fires(point: 'restore' | 'finish'): boolean  // true once after arm
//! ```

use std::cell::Cell;

thread_local! {
    /// Whether the next lockfile restore on this thread fails.
    static RESTORE: Cell<bool> = const { Cell::new(false) };
    /// Whether the next journal finalization on this thread fails.
    static FINISH: Cell<bool> = const { Cell::new(false) };
}

/// The steps that can be made to fail.
#[derive(Clone, Copy, Debug)]
pub enum Point {
    /// The restore of lockfiles after a failed or refused step.
    Restore,
    /// The completion of a transaction's journal.
    Finish,
}

/// What:
/// `pub fn arm(point: Point)` makes the next use of the step on this thread fail.
/// Why:
/// Tests provoke the failure without depending on permissions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function arm(point: Point): void
/// ```
pub fn arm(point: Point) {
    match point {
        Point::Restore => RESTORE.with(|slot| { slot.set(true); }),
        Point::Finish => FINISH.with(|slot| { slot.set(true); }),
    }
}

/// What:
/// `pub fn fires(point: Point) -> bool` says whether the step was armed, and disarms it.
/// Why:
/// Each arming fails exactly one use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function fires(point: Point): boolean
/// ```
pub fn fires(point: Point) -> bool {
    return match point {
        Point::Restore => RESTORE.with(Cell::take),
        Point::Finish => FINISH.with(Cell::take),
    };
}
