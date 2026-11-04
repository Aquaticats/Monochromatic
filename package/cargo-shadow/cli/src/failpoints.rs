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
    static OUTPUT_FILE_SYNC: Cell<bool> = const { Cell::new(false) };
    static OUTPUT_DIRECTORY_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static ACKNOWLEDGEMENT_PUBLISH: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static ACKNOWLEDGEMENT_DURABILITY: Cell<bool> = const { Cell::new(false) };
    /// Whether the next sync of the acknowledgement file by a reader on this thread fails.
    static ACKNOWLEDGEMENT_FILE_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next sync of the directory that lists the acknowledgement, by a reader on this thread, fails.
    static ACKNOWLEDGEMENT_DIRECTORY_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next wait for a killed discovery program on this thread fails with an error.
    static REAP_ERROR: Cell<bool> = const { Cell::new(false) };
    /// Whether the next sync of a file that is about to be published whole, on this thread, fails.
    static PUBLICATION_FILE_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next stop of a scope on this thread finds a member that survives every signal until its deadline.
    static CONTAINMENT_SURVIVOR: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static RECORD_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static RECORD_WRITE: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static REAP_STUCK: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static COMPLETION_SYNC: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static COMPLETION_RENAME: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static LISTING_ENTRY_NOT_FOUND: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static LISTING_INSPECTION: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static LISTING_ITERATION: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static LISTING_OPEN: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static CONTAINMENT_STOP_STUCK: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static CONTAINMENT_STOP_ERROR: Cell<bool> = const { Cell::new(false) };
    /// Whether the next use of this step on this thread fails.
    static CONTAINMENT_INSPECTION: Cell<bool> = const { Cell::new(false) };
    /// Whether the next publication of a journal's durable completion marker on this thread fails.
    static FINISH_COMPLETION: Cell<bool> = const { Cell::new(false) };
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
    /// The sync of an output file inside `make_durable`, the real barrier that finishing and recovery use for lockfiles and manifests.
    OutputFileSync,
    /// The sync of the directory that lists an output file inside `make_durable`.
    OutputDirectorySync,
    /// The publication of an acknowledgement fails before it is visible.
    AcknowledgementPublish,
    /// The sync that makes an acknowledgement durable fails.
    AcknowledgementDurability,
    /// The sync of the acknowledgement file by a reader (a recovery) fails.
    AcknowledgementFileSync,
    /// The sync of the directory that lists the acknowledgement, by a reader (a recovery), fails.
    AcknowledgementDirectorySync,
    /// The wait for a killed discovery program reports an error instead of a status.
    ReapError,
    /// The sync of the temporary file of an atomic publication (the writer's side of an acknowledgement) fails.
    PublicationFileSync,
    /// The sync that makes a containment record durable fails.
    RecordSync,
    /// The write of a containment record fails.
    RecordWrite,
    /// The reaping of a direct child reports that it could not be reaped within its bound.
    ReapStuck,
    /// The sync of the directory that lists the completion marker fails (after the real rename).
    CompletionSync,
    /// The rename that publishes the completion marker fails (the real rename is not made).
    CompletionRename,
    /// The examination of the second entry of the journal directory reports that it vanished.
    ListingEntryNotFound,
    /// The examination of the second entry of the journal directory fails.
    ListingInspection,
    /// The reading of the second entry of the journal directory fails.
    ListingIteration,
    /// The opening of the journal directory for listing fails.
    ListingOpen,
    /// The stop of a scope reports that a member could not be stopped within the bound.
    ContainmentStopStuck,
    /// The stop of a scope after the program started finds a member that survives every signal: the stop runs its real loop to its
    /// deadline and then reports that the member could not be stopped.
    ContainmentSurvivor,
    /// The stop of a scope after the program started fails with an error.
    ContainmentStopError,
    /// The listing of the members of a scope after the program started fails.
    ContainmentInspection,
    /// The publication of a journal's durable completion marker (the rename to its done name and the sync of the directory that lists it).
    FinishCompletion,
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
        Point::OutputFileSync => OUTPUT_FILE_SYNC.with(|slot| { slot.set(true); }),
        Point::OutputDirectorySync => OUTPUT_DIRECTORY_SYNC.with(|slot| { slot.set(true); }),
        Point::AcknowledgementPublish => ACKNOWLEDGEMENT_PUBLISH.with(|slot| { slot.set(true); }),
        Point::AcknowledgementDurability => ACKNOWLEDGEMENT_DURABILITY.with(|slot| { slot.set(true); }),
        Point::AcknowledgementFileSync => ACKNOWLEDGEMENT_FILE_SYNC.with(|slot| { slot.set(true); }),
        Point::AcknowledgementDirectorySync => ACKNOWLEDGEMENT_DIRECTORY_SYNC.with(|slot| { slot.set(true); }),
        Point::ContainmentSurvivor => CONTAINMENT_SURVIVOR.with(|slot| { slot.set(true); }),
        Point::ReapError => REAP_ERROR.with(|slot| { slot.set(true); }),
        Point::PublicationFileSync => PUBLICATION_FILE_SYNC.with(|slot| { slot.set(true); }),
        Point::RecordSync => RECORD_SYNC.with(|slot| { slot.set(true); }),
        Point::RecordWrite => RECORD_WRITE.with(|slot| { slot.set(true); }),
        Point::ReapStuck => REAP_STUCK.with(|slot| { slot.set(true); }),
        Point::CompletionSync => COMPLETION_SYNC.with(|slot| { slot.set(true); }),
        Point::CompletionRename => COMPLETION_RENAME.with(|slot| { slot.set(true); }),
        Point::ListingEntryNotFound => LISTING_ENTRY_NOT_FOUND.with(|slot| { slot.set(true); }),
        Point::ListingInspection => LISTING_INSPECTION.with(|slot| { slot.set(true); }),
        Point::ListingIteration => LISTING_ITERATION.with(|slot| { slot.set(true); }),
        Point::ListingOpen => LISTING_OPEN.with(|slot| { slot.set(true); }),
        Point::ContainmentStopStuck => CONTAINMENT_STOP_STUCK.with(|slot| { slot.set(true); }),
        Point::ContainmentStopError => CONTAINMENT_STOP_ERROR.with(|slot| { slot.set(true); }),
        Point::ContainmentInspection => CONTAINMENT_INSPECTION.with(|slot| { slot.set(true); }),
        Point::FinishCompletion => FINISH_COMPLETION.with(|slot| { slot.set(true); }),
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
        Point::OutputFileSync => OUTPUT_FILE_SYNC.with(Cell::take),
        Point::OutputDirectorySync => OUTPUT_DIRECTORY_SYNC.with(Cell::take),
        Point::AcknowledgementPublish => ACKNOWLEDGEMENT_PUBLISH.with(Cell::take),
        Point::AcknowledgementDurability => ACKNOWLEDGEMENT_DURABILITY.with(Cell::take),
        Point::AcknowledgementFileSync => ACKNOWLEDGEMENT_FILE_SYNC.with(Cell::take),
        Point::AcknowledgementDirectorySync => ACKNOWLEDGEMENT_DIRECTORY_SYNC.with(Cell::take),
        Point::ContainmentSurvivor => CONTAINMENT_SURVIVOR.with(Cell::take),
        Point::ReapError => REAP_ERROR.with(Cell::take),
        Point::PublicationFileSync => PUBLICATION_FILE_SYNC.with(Cell::take),
        Point::RecordSync => RECORD_SYNC.with(Cell::take),
        Point::RecordWrite => RECORD_WRITE.with(Cell::take),
        Point::ReapStuck => REAP_STUCK.with(Cell::take),
        Point::CompletionSync => COMPLETION_SYNC.with(Cell::take),
        Point::CompletionRename => COMPLETION_RENAME.with(Cell::take),
        Point::ListingEntryNotFound => LISTING_ENTRY_NOT_FOUND.with(Cell::take),
        Point::ListingInspection => LISTING_INSPECTION.with(Cell::take),
        Point::ListingIteration => LISTING_ITERATION.with(Cell::take),
        Point::ListingOpen => LISTING_OPEN.with(Cell::take),
        Point::ContainmentStopStuck => CONTAINMENT_STOP_STUCK.with(Cell::take),
        Point::ContainmentStopError => CONTAINMENT_STOP_ERROR.with(Cell::take),
        Point::ContainmentInspection => CONTAINMENT_INSPECTION.with(Cell::take),
        Point::FinishCompletion => FINISH_COMPLETION.with(Cell::take),
    };
}
