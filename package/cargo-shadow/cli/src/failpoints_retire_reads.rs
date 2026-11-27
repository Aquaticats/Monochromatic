//! What:
//! The test-only switches of the three reads the last look at a run's scope makes:
//! the kernel's `populated` line,
//! how many processes the group has ever held,
//! and whether the group's own directory is still there.
//! Why:
//! Two of the states those reads decide on cannot be produced from outside:
//! the manager removes a group's directory the moment its last member leaves,
//! which is the window the look's two reads can straddle,
//! and a removed group's own control file answers an error of its own,
//! which no path of this host answers,
//! because a path that is gone answers `ENOENT` before the file is opened.
//! They sit in a file of their own
//! because `src/failpoints.rs` has no code line left (`rust/max-lines`,
//! 300),
//! and they are declared from there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // retire-reads.failpoints.ts (test builds only)
//! export function armPeakReadSeesTheGoneGroup(): void
//! export function firesPeakReadSeesTheGoneGroup(): boolean
//! // and the same pair for the directory check and the `populated` read
//! ```

/// What:
/// `use std::cell::Cell;` imports the single-threaded mutable slot type.
/// Why:
/// Each switch is one boolean that the thread that armed it reads back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let armed = false
/// ```
use std::cell::Cell;

thread_local! {
    /// Whether the next `pids.peak` read of a run's scope answers the absence a group the manager removed answers.
    static PEAK_READ_SEES_THE_GONE_GROUP: Cell<bool> = const { Cell::new(false) };
    /// Whether the next check of a run's scope directory answers that the directory is not there.
    static DIRECTORY_CHECK_SEES_THE_GONE_GROUP: Cell<bool> = const { Cell::new(false) };
    /// Whether the next `populated` read of a run's scope answers the error a removed group's control file answers.
    static POPULATED_READ_SEES_THE_GONE_GROUP: Cell<bool> = const { Cell::new(false) };
}

/// What:
/// `pub(crate) fn arm_peak_read_sees_the_gone_group()` makes the next `pids.peak` read of a run's scope answer the absence
/// a group the manager removed answers.
/// Why:
/// The manager removing the group between the two reads of the look is a window nothing outside holds open,
/// and the arm answers for it once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function armPeakReadSeesTheGoneGroup(): void { peakReadSeesTheGoneGroup = true }
/// ```
pub(crate) fn arm_peak_read_sees_the_gone_group() {
    PEAK_READ_SEES_THE_GONE_GROUP.with(|slot| {
        slot.set(true);
    });
}

/// What:
/// `pub(crate) fn fires_peak_read_sees_the_gone_group() -> bool` says whether that read was armed,
/// and disarms it.
/// Why:
/// A control shows that its fault was reached,
/// and one arm reaches one read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function firesPeakReadSeesTheGoneGroup(): boolean
/// ```
pub(crate) fn fires_peak_read_sees_the_gone_group() -> bool {
    return PEAK_READ_SEES_THE_GONE_GROUP.with(Cell::take);
}

/// What:
/// `pub(crate) fn arm_directory_check_sees_the_gone_group()` makes the next check of a run's scope directory answer that
/// the directory is not there.
/// Why:
/// A directory that is there answers no such thing,
/// and the check after a lost `pids.peak` read is what the look's answer hangs on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function armDirectoryCheckSeesTheGoneGroup(): void { directoryCheckSeesTheGoneGroup = true }
/// ```
pub(crate) fn arm_directory_check_sees_the_gone_group() {
    DIRECTORY_CHECK_SEES_THE_GONE_GROUP.with(|slot| {
        slot.set(true);
    });
}

/// What:
/// `pub(crate) fn fires_directory_check_sees_the_gone_group() -> bool` says whether that check was armed,
/// and disarms it.
/// Why:
/// A control shows that its fault was reached,
/// and one arm reaches one check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function firesDirectoryCheckSeesTheGoneGroup(): boolean
/// ```
pub(crate) fn fires_directory_check_sees_the_gone_group() -> bool {
    return DIRECTORY_CHECK_SEES_THE_GONE_GROUP.with(Cell::take);
}

/// What:
/// `pub(crate) fn arm_populated_read_sees_the_gone_group()` makes the next `populated` read of a run's scope answer the
/// error a removed group's control file answers (`ENODEV`).
/// Why:
/// A removed group's file answers that error and a directory that is gone answers `ENOENT` first,
/// so no fixture on this host can make the first,
/// and the look reads the two as one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function armPopulatedReadSeesTheGoneGroup(): void { populatedReadSeesTheGoneGroup = true }
/// ```
pub(crate) fn arm_populated_read_sees_the_gone_group() {
    POPULATED_READ_SEES_THE_GONE_GROUP.with(|slot| {
        slot.set(true);
    });
}

/// What:
/// `pub(crate) fn fires_populated_read_sees_the_gone_group() -> bool` says whether that read was armed,
/// and disarms it.
/// Why:
/// A control shows that its fault was reached,
/// and one arm reaches one read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function firesPopulatedReadSeesTheGoneGroup(): boolean
/// ```
pub(crate) fn fires_populated_read_sees_the_gone_group() -> bool {
    return POPULATED_READ_SEES_THE_GONE_GROUP.with(Cell::take);
}
