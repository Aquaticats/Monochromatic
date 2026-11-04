//! What:
//! Test-only switches that make one named step fail once,
//! on the thread that set them.
//! Why:
//! Review finding (4c):
//! rollback and finalization failures must be provoked deterministically,
//! also by a process that ignores
//! directory permissions (a root user),
//! so the refusal-precedence tests do not skip.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // failpoints.ts (test builds only)
//! export function arm(point: 'restore' | 'finish'): void
//! export function fires(point: 'restore' | 'finish'): boolean  // true once after arm
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::cell::Cell` for this file.
use std::cell::Cell;

thread_local! {
    /// Whether the next lockfile restore on this thread fails.
    static RESTORE: Cell<bool> = const { Cell::new(false) };
    /// Whether the next journal finalization on this thread fails.
    static FINISH: Cell<bool> = const { Cell::new(false) };
    /// Whether the next journal cleanup after a durable completion on this thread fails.
    static FINISH_CLEANUP: Cell<bool> = const { Cell::new(false) };
    /// Whether the next durability barrier of a recovery on this thread fails.
    static RECOVER_DURABILITY: Cell<bool> = const { Cell::new(false) };
}

/// The steps that can be made to fail.
#[derive(Clone, Copy, Debug)]
pub enum Point {
    /// The restore of lockfiles after a failed or refused step.
    Restore,
    /// The forcing of a transaction's outputs to disk before its completion marker.
    FinishDurability,
    /// The removal of a transaction's journal after the completion marker was made durable.
    FinishCleanup,
    /// The forcing of live files to disk before a recovery deletes the durable copies that made them recoverable.
    RecoverDurability,
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
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    match point {
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        Point::Restore => RESTORE.with(|slot| { slot.set(true); }),
        // Closure `|slot|`: an anonymous function, like a TS arrow function.
        Point::FinishDurability => FINISH.with(|slot| { slot.set(true); }),
        Point::FinishCleanup => FINISH_CLEANUP.with(|slot| { slot.set(true); }),
        Point::RecoverDurability => RECOVER_DURABILITY.with(|slot| { slot.set(true); }),
    }
}

/// What:
/// `pub fn fires(point: Point) -> bool` says whether the step was armed,
/// and disarms it.
/// Why:
/// Each arming fails exactly one use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function fires(point: Point): boolean
/// ```
pub fn fires(point: Point) -> bool {
    // `match`: picks the arm whose pattern fits the value.
    return match point {
        Point::Restore => RESTORE.with(Cell::take),
        Point::FinishDurability => FINISH.with(Cell::take),
        Point::FinishCleanup => FINISH_CLEANUP.with(Cell::take),
        Point::RecoverDurability => RECOVER_DURABILITY.with(Cell::take),
    };
}
