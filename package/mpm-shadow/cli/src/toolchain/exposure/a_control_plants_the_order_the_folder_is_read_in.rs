//! What:
//! The order the rebuild of the exposure folder reads that folder's entries in,
//! which a build made for controls can set to name order or to its reverse.
//! Why:
//! A report that names the temporary links it keeps has to read the same on every run of the same
//! folder,
//! so the pass sorts those names before it says them;
//! a control can only hold that sort if it can make the order the folder gives differ from name
//! order,
//! and a file system's own order is a hash of each name:
//! on this machine two planted names come back in name order in about two of five fresh folders,
//! so a control that relied on the file system's order stayed green with the sort deleted.
//! A second control needs the opposite:
//! a kept name below a removal the kernel refuses,
//! which only an order it chooses can place there,
//! because a temporary link's name always begins with a dot.
//! A shipped build holds no way to choose the order:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose,
//! as the removal's own plant does (`a_control_plants_a_removal`).
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant it:
//! a control plants one for its own thread and gives the folder back when it ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-the-order-the-folder-is-read-in.ts
//! function readingTheFolderIn(order: 'name order' | 'the reverse of name order'): Guard;  // tests only
//! function inTheOrderItIsRead(entries: DirEntry[]): DirEntry[];  // the entries as given in a shipped build
//! ```

/// Imports `DirEntry` from `std::fs`,
/// the entry type a directory listing yields.
/// Why:
/// The pass gives each entry its own arms,
/// and the plant's ordering sorts the same type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // readdirSync with { withFileTypes: true } yields entries, and throws on the folder instead.
/// ```
use std::fs::DirEntry;
/// Imports `Result` from `std::io`,
/// the answer every entry of a listing is wrapped in.
/// Why:
/// The same type the pass's own loop already matches on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: an entry is an object or the listing throws.
/// ```
use std::io::Result;

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// The order a control can give the folder's entries.
/// Why:
/// Two controls need two orders:
/// one must differ from name order so the report's sort is what puts the names in order,
/// and one must put a dotted name before a name a removal is refused for,
/// which no order but name order can do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheFoldersOrder = 'name order' | 'the reverse of name order';
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TheFoldersOrder {
    /// The entries as name order gives them.
    NameOrder,
    /// The entries as the reverse of name order gives them.
    TheReverseOfNameOrder,
}

#[cfg(test)]
thread_local! {
    /// The order this thread's rebuild reads the folder in,
    /// when a control has chosen one.
    static THE_ORDER: Cell<Option<TheFoldersOrder>> = const { Cell::new(None) };
}

/// What:
/// Forgets a plant,
/// so a rebuild after a control's own reads the folder as the directory lists it.
/// Why:
/// A plant that outlived its control would decide another control's report on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetThePlant(): void
/// ```
#[cfg(test)]
fn forget_the_plant() {
    THE_ORDER.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The release of a plant,
/// taken by the guard `reading_the_folder_in` answers.
/// Why:
/// A control's failing assertion raises a panic,
/// which skips the line that would forget the plant,
/// and a plant left armed would decide the next rebuild on this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlantGuard { [Symbol.dispose]() { forgetThePlant(); } }
/// ```
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct FolderOrderPlantGuard;

/// The release of a plant.
#[cfg(test)]
impl Drop for FolderOrderPlantGuard {
    /// What:
    /// Forgets the plant this thread made,
    /// which is the one release a control's body must reach however it leaves.
    /// Why:
    /// The drop is the only place that runs on the way out of a panic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { forgetThePlant(); }
    /// ```
    fn drop(&mut self) {
        forget_the_plant();
    }
}

/// What:
/// Plants,
/// for this thread's next rebuild,
/// a folder whose entries come in `order`,
/// and answers the guard that forgets it when the control's body leaves,
/// however it leaves.
/// Why:
/// The state to hold is a directory's order chosen by the control rather than hashed by the file
/// system,
/// and the reverse of name order is not name order by construction for any two names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readingTheFolderIn(order: TheFoldersOrder): PlantGuard {
///   THE_ORDER = order;
///   return new PlantGuard();
/// }
/// ```
#[cfg(test)]
pub(crate) fn reading_the_folder_in(order: TheFoldersOrder) -> FolderOrderPlantGuard {
    THE_ORDER.with(|cell| {
        cell.set(Some(order));
    });
    return FolderOrderPlantGuard;
}

/// What:
/// The name an entry carries,
/// for the plant's ordering.
/// Why:
/// The order is by name,
/// so the key is the entry's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aName(entry: DirEntry): string { return entry.fileName; }
/// ```
#[cfg(test)]
fn a_name(entry: &Result<DirEntry>) -> String {
    return entry
        .as_ref()
        .map(|present| return present.file_name().to_string_lossy().to_string())
        .unwrap_or_default();
}

/// What:
/// The entries in the order this thread's rebuild reads them:
/// the order a control chose,
/// and the directory's own order otherwise.
/// Why:
/// A control makes the order what it needs by construction,
/// so the sort that makes a report stable is held by a failing assertion rather than by luck.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inTheOrderItIsRead(entries: DirEntry[]): DirEntry[] {
///   const order = THE_ORDER;
///   if (order === undefined) return entries;
///   return [...entries].sort((a, b) => order === 'name order' ? aName(a).localeCompare(aName(b)) : aName(b).localeCompare(aName(a)));
/// }
/// ```
#[cfg(test)]
pub(super) fn in_the_order_it_is_read(mut entries: Vec<Result<DirEntry>>) -> Vec<Result<DirEntry>> {
    let chosen: Option<TheFoldersOrder> = THE_ORDER.with(|cell| return cell.get());
    let Some(order) = chosen else {
        return entries;
    };
    entries.sort_by(|left, right| {
        return match order {
            TheFoldersOrder::NameOrder => a_name(left).cmp(&a_name(right)),
            TheFoldersOrder::TheReverseOfNameOrder => a_name(right).cmp(&a_name(left)),
        };
    });
    return entries;
}

/// The answer a shipped build gives:
/// the entries as the directory listed them,
/// because nothing can ask for another order.
#[cfg(not(test))]
pub(super) fn in_the_order_it_is_read(entries: Vec<Result<DirEntry>>) -> Vec<Result<DirEntry>> {
    return entries;
}
