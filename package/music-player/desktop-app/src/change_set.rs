//! What the file watcher saw change, as a set of paths or as "everything".
//!
//! What: `Change` is either the union of the paths the watcher named (`Paths`) or the admission that the watcher cannot
//! say what changed (`Everything`); `DirtyPaths` is the set, with a flag per path that tells an entry appearing or
//! disappearing from an entry that was only written.
//! Why: D113 refreshes only what changed. The debounced watcher events name the changed paths, so the refresh can look
//! at those paths and fall back to a whole-library sweep only when the watcher reports lost events (an overflow of the
//! kernel queue, an error) or when a person opens a folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Change = { kind: "paths"; paths: Map<string, boolean> } | { kind: "everything" };
//! ```

/// What: `use std::collections::HashMap;` imports the hash map type from the standard library (`std`). Siblings:
/// `BTreeMap` keeps its keys sorted, and `HashSet` stores keys with no values.
/// Why: the set of named paths needs a lookup by path with a value per path, in constant time; sorted order is not needed
/// here because `sorted` sorts a copy when the planner asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map is global in TypeScript; nothing to import.
/// ```
use std::collections::HashMap;
/// What: `use std::path::PathBuf;` imports the owned, growable path type. Siblings: `Path` is the borrowed read-only view,
/// and `String` is the text-only owned type, which cannot hold file names that are not valid UTF-8.
/// Why: the set owns its paths, so it must hold the owned type, not a borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Paths are plain strings in TypeScript.
/// ```
use std::path::PathBuf;

/// What: `use notify_debouncer_mini::notify::{Event, EventKind, event::ModifyKind};` imports three names from the
/// `notify` crate (the file-watching library) that the debouncer crate re-exports: `Event` (one raw watcher event),
/// `EventKind` (what happened: access, create, modify, remove) and `ModifyKind` (the kinds of modification). The braces
/// import several names from one path in a single line, and `event::ModifyKind` reaches into a submodule.
/// Why: the change set is built from raw events, and it must tell a read from a write from a rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Event, type EventKind, ModifyKind } from "notify-debouncer-mini";
/// ```
use notify_debouncer_mini::notify::{Event, EventKind, event::ModifyKind};

/// What: `pub(crate) const MAX_DIRTY_PATHS: usize = 100_000;` is a compile-time constant. `pub(crate)` makes it visible
/// inside this crate only (siblings: plain `pub` reaches other crates, and no keyword is private to the file). `usize` is
/// the unsigned integer wide enough to index memory (siblings: `u32`, `u64`, `i32`). `100_000` is the number with a
/// readability underscore, which Rust ignores.
/// Why: past a tenth of the largest supported library (a copy of a hundred thousand files), looking at each named path
/// costs as much as walking the library once, and the memory of the set must stay bounded in a storm of events. It is a
/// `usize`, not a `u32`, because it is compared with `len()`, which answers a `usize`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_DIRTY_PATHS = 100_000;
/// ```
pub(crate) const MAX_DIRTY_PATHS: usize = 100_000;

/// What: `pub struct DirtyPaths { entries }` is a record type with one field. `#[derive(Clone, Debug, Default, PartialEq, Eq)]`
/// asks the compiler to write five standard abilities: `Clone` (an explicit `.clone()` that copies the whole set), `Debug`
/// (printing with `{:?}`), `Default` (a ready-made empty value from `DirtyPaths::default()`), and `PartialEq` and `Eq`
/// (comparing two sets with `==`).
/// Why: a path that only had its data written needs its file looked at; a path where an entry was created, removed or
/// renamed also needs, when it is a folder, a look at what is inside it.
/// Gotcha: the field is private (no `pub`), so only this file can touch `entries`; callers go through the methods.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtyPaths = Map<string, boolean>; // path -> structural
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirtyPaths {
    /// What: `entries: HashMap<PathBuf, bool>,` is the one field: a map from each named path (`PathBuf`, an owned path;
    /// sibling `&Path` would only borrow it) to a `bool` that says whether an entry appeared or disappeared there.
    /// Why: the path is the key because the same path may be named many times and must be stored once; the owned type
    /// is needed because the set outlives the event that named the path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries: Map<string, boolean>;
    /// ```
    entries: HashMap<PathBuf, bool>,
}

/// What: `impl DirtyPaths { ... }` opens the block that holds the methods of `DirtyPaths`. A struct holds data; its methods
/// are written in an `impl` block.
/// Why: all the set's operations (add, count, sort, merge) live together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DirtyPaths { /* the methods of the impl block */ }
/// ```
impl DirtyPaths {
    /// What: `pub(crate) fn single(path: PathBuf, structural: bool) -> DirtyPaths` builds a set of one path. The attribute
    /// `#[cfg(test)]` makes the compiler include the function only when building tests. The parameters are taken by value:
    /// `path` is moved in, so the caller cannot use it afterwards.
    /// Why: tests and the controller build small sets by hand.
    /// Gotcha: this function does not exist in the production build, so non-test code cannot call it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static single(path: string, structural: boolean): DirtyPaths { const set = new DirtyPaths(); set.insert(path, structural); return set; }
    /// ```
    #[cfg(test)]
    pub(crate) fn single(path: PathBuf, structural: bool) -> DirtyPaths {
        // What:     `let mut set: DirtyPaths = DirtyPaths::default();` makes a new empty set. `DirtyPaths::default()` calls
        //           the `Default` ability that the `derive` wrote (an empty map). `mut` allows the variable to change;
        //           Rust variables are constants unless marked, and the `: DirtyPaths` annotation names the type.
        // Why:      The set is filled in the next statement, so it must be changeable.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const set: DirtyPaths = new DirtyPaths();
        // ```
        let mut set: DirtyPaths = DirtyPaths::default();
        // What:     `set.insert(path, structural);` calls the method defined later in this block; `path` is moved into the set.
        // Why:      Put the single path into the set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // set.insert(path, structural);
        // ```
        set.insert(path, structural);
        // What:     `return set;` hands the set to the caller; ownership moves out of the function.
        // Why:      The caller wants the finished set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return set;
        // ```
        return set;
    }

    /// What: `pub(crate) fn insert(&mut self, path: PathBuf, structural: bool)` adds a path; a path named twice stays
    /// structural if either naming was. `&mut self` is an exclusive borrow of the set: the method may change it, and no one
    /// else may touch it while it runs.
    /// Why: a file replaced by a rename shows up as a write and as a rename; the stronger meaning must survive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// insert(path: string, structural: boolean): void { this.entries.set(path, (this.entries.get(path) ?? false) || structural); }
    /// ```
    pub(crate) fn insert(&mut self, path: PathBuf, structural: bool) {
        // What:     `let entry: &mut bool = self.entries.entry(path).or_insert(false);` finds the map slot for the path and
        //           creates it holding `false` when it is missing. `.entry(path)` is a lookup that gives access to the slot;
        //           `.or_insert(false)` fills an empty slot and answers `&mut bool`, a mutable borrow of the stored flag.
        // Why:      A single lookup serves both the "new path" and "seen path" cases.
        // Gotcha:   `entry` is not a copy of the flag; it points into the map, so writing through it changes the map.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.entries.has(path)) this.entries.set(path, false);
        // ```
        let entry: &mut bool = self.entries.entry(path).or_insert(false);
        // What:     `*entry = *entry || structural;` reads through the borrow (`*entry` on the right), combines it with the new
        //           flag, and writes the result back through the borrow (`*entry` on the left). The `*` follows a reference
        //           to the value it points at; it is not multiplication.
        // Why:      Once a path has been marked structural it stays structural.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.entries.set(path, this.entries.get(path)! || structural);
        // ```
        *entry = *entry || structural;
    }

    /// What: `pub(crate) fn len(&self) -> usize` counts the distinct paths named. `&self` is a read-only borrow.
    /// Why: the cap on a change set and the tests read it. `usize` is the type `HashMap::len` answers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// len(): number { return this.entries.size; }
    /// ```
    pub(crate) fn len(&self) -> usize {
        // What:     `return self.entries.len();` answers the map's own length.
        // Why:      The map already knows its size.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.entries.size;
        // ```
        return self.entries.len();
    }

    /// What: `pub(crate) fn is_empty(&self) -> bool` says whether no path is named.
    /// Why: an empty set means there is nothing to refresh.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isEmpty(): boolean { return this.entries.size === 0; }
    /// ```
    pub(crate) fn is_empty(&self) -> bool {
        // What:     `return self.entries.is_empty();` answers the map's own emptiness test.
        // Why:      The map already knows whether it holds anything.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.entries.size === 0;
        // ```
        return self.entries.is_empty();
    }

    /// What: `pub(crate) fn sorted(&self) -> Vec<(PathBuf, bool)>` lists the paths in path order, so a folder comes before
    /// everything inside it. `Vec<T>` is an owned, growable list (siblings: `&[T]` is a borrowed view, `[T; N]` a fixed
    /// array); `(PathBuf, bool)` is a tuple, a fixed pair like TypeScript's `[string, boolean]`.
    /// Why: the planner handles a folder first and skips what its walk already covered. A `Vec` is returned, not a slice,
    /// because the list is built here and the caller keeps it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sorted(): [string, boolean][] { return [...this.entries].sort(byPath); }
    /// ```
    pub(crate) fn sorted(&self) -> Vec<(PathBuf, bool)> {
        // What:     `let mut all: Vec<(PathBuf, bool)> = self.entries.iter().map(|(path, structural)| return (path.clone(), *structural)).collect();`
        //           copies the map into a list of pairs. `.iter()` walks the entries as borrowed pairs; `.map(f)` turns
        //           each into a new value; `|(path, structural)| ...` is a closure (an anonymous function; its parameters sit
        //           between bars) whose parameter is a pair split into two names. `path.clone()` copies the path into an owned
        //           `PathBuf`, `*structural` reads the flag through its borrow, and `(a, b)` builds a tuple. `.collect()`
        //           gathers the results into the annotated type, `Vec<(PathBuf, bool)>`.
        // Why:      A map has no order, and the planner needs one, so the entries are copied out and sorted without
        //           disturbing the set.
        // Gotcha:   Nothing is computed until `.collect()` pulls the values through the lazy `.iter().map(...)` chain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const all: [string, boolean][] = [...this.entries].map(([path, structural]) => [path, structural]);
        // ```
        let mut all: Vec<(PathBuf, bool)> = self.entries.iter().map(|(path, structural)| return (path.clone(), *structural)).collect();
        // What:     `all.sort();` sorts the list in place by the natural order of its pairs: first by path, then by flag.
        //           Tuples compare field by field from the left.
        // Why:      Path order puts a folder before everything inside it, which is what the planner needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // all.sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
        // ```
        all.sort();
        // What:     `return all;` hands the sorted list to the caller; ownership moves out.
        // Why:      The caller owns the list from here on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return all;
        // ```
        return all;
    }

    /// What: `fn absorb(&mut self, other: DirtyPaths)` adds every path of another set. There is no `pub`, so the method is
    /// private to this file. `other` is taken by value, which moves the other set in: the caller can no longer use it.
    /// Why: two bursts of events become one change.
    /// Gotcha: after the call the second set is gone; in TypeScript both sets would still exist.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private absorb(other: DirtyPaths): void { for (const [path, structural] of other.entries) this.insert(path, structural); }
    /// ```
    fn absorb(&mut self, other: DirtyPaths) {
        // What:     `for (path, structural) in other.entries { ... }` visits every pair of the other set's map. Iterating the
        //           map by value (no `&`) hands out each owned `PathBuf` and flag, consuming the map; the parenthesised
        //           pattern splits each pair into two names.
        // Why:      Each path is moved, not copied, into this set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [path, structural] of other.entries) {
        // ```
        for (path, structural) in other.entries {
            // What:     `self.insert(path, structural);` calls the method above, so the "stays structural" rule applies.
            // Why:      Merging uses the same rule as adding one path.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.insert(path, structural);
            // ```
            self.insert(path, structural);
        }
    }
}

/// What: `pub enum Change { Paths(DirtyPaths), Everything }` is a tagged union of two cases: `Paths` carries a set, and
/// `Everything` carries nothing. `#[derive(Clone, Debug, PartialEq, Eq)]` writes the copy, print and `==` abilities.
/// Why: lost events cannot be turned into paths; they are the one case that needs the whole library looked at again.
/// Gotcha: `Paths(DirtyPaths)` is a tuple variant: the data sits in the variant, and `match` is how it is read back out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { kind: "paths"; paths: DirtyPaths } | { kind: "everything" };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// What: `Paths(DirtyPaths),` is the case that carries the set of named paths.
    /// Why: most bursts of events name what changed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "paths"; paths: DirtyPaths }
    /// ```
    Paths(DirtyPaths),
    /// What: `Everything,` is the case that carries no data.
    /// Why: the watcher lost events or the library was just opened, so only a full look is safe.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "everything" }
    /// ```
    Everything,
}

/// What: `event_is_a_plain_write` says whether a notify event kind means that an entry was only written, not created, removed
/// or renamed.
/// Why: A plain write needs no look inside the entry; a structural event reads the whole subtree of a folder. Linux and macOS
///      report writes as `Modify(Data)` or `Modify(Metadata)`. The notify Windows backend (`notify` 8.2.0, `windows.rs`)
///      maps every `FILE_ACTION_MODIFIED` to `Modify(Any)` and never to those two, and Windows also reports that action for a
///      folder whose entries were just written. Treating it as structural made each such event read a whole folder, so on
///      Windows `Modify(Any)` is a plain write. Renames arrive as `Modify(Name)`, creations as `Create`, removals as `Remove`.
pub(crate) fn event_is_a_plain_write(kind: &EventKind) -> bool {
    return match kind {
        EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Metadata(_)) => true,
        EventKind::Modify(ModifyKind::Any) => cfg!(windows),
        _ => false,
    };
}

/// What: `folder_was_only_read_or_touched` is true on Windows for an unspecific "modified" event about a folder that exists.
/// Why: Windows reports a folder as modified when its entries are merely read (the last-access time moves; measured on a
///      GitHub Windows runner on 2026-10-15: `read_dir` of a watched folder produced `Modify(Any)` for that folder and
///      nothing else). Real changes inside the folder come as their own events (`Create`, `Remove`, rename, or a modified
///      file), so the folder event adds nothing and, left in, makes every refresh request another one. Elsewhere it is false.
pub(crate) fn folder_was_only_read_or_touched(kind: &EventKind, path: &std::path::Path) -> bool {
    return cfg!(windows) && matches!(kind, EventKind::Modify(ModifyKind::Any)) && path.is_dir();
}

/// What: `impl Change { ... }` opens the block that holds the methods of `Change`.
/// Why: the translation from a raw event and the merge of two changes belong to the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Change { /* the functions below */ }
/// ```
impl Change {
    /// What: `pub(crate) fn of_event(result: &notify_debouncer_mini::notify::Result<Event>) -> Option<Change>` says what one
    /// raw watcher event means: nothing for a read, everything for lost events and errors, else its paths. The argument is
    /// a borrow (`&`) of the watcher's `Result`, which is `Ok(event)` or `Err(error)`. `Option<Change>` is Rust's
    /// `Change | null`: `Some(change)` or `None`.
    /// Why: the rescan's own walk reads every directory, and reads must never count as changes or each refresh would
    /// request another; an overflow or an error means events were lost.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static ofEvent(result: Result<Event>): Change | null { ... }
    /// ```
    pub(crate) fn of_event(result: &notify_debouncer_mini::notify::Result<Event>) -> Option<Change> {
        // What:     `let event: &Event = match result { Ok(event) => event, Err(_) => return Some(Change::Everything) };`
        //           takes the answer apart. `Ok(event)` binds the event and the arm's value is that same event; `Err(_)`
        //           matches any failure (the `_` means "I do not need the error value") and leaves the function early.
        //           `Some(Change::Everything)` wraps the case in the present form of `Option`; `Change::Everything` joins the
        //           enum's name and its case with `::`.
        // Why:      A watcher error means events may have been lost, so the only safe meaning is "everything".
        // Gotcha:   `match` must cover every case; `Result` has exactly `Ok` and `Err`, and both are present.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (result.error) return { kind: "everything" };
        // const event = result.value;
        // ```
        let event: &Event = match result {
            Ok(event) => event,
            Err(_) => return Some(Change::Everything),
        };
        // What:     `if event.kind.is_access() { return None; }` leaves with `None` when the event was only a read.
        //           `None` is the absent case of `Option`.
        // Why:      The refresh's own walk reads every directory; counting reads would make each refresh ask for another.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (event.kind.isAccess()) return null;
        // ```
        if event.kind.is_access() {
            return None;
        }
        // What:     `if event.need_rescan() { return Some(Change::Everything); }` leaves with "everything" when the event
        //           itself says that events were lost (a kernel queue overflow).
        // Why:      Lost events cannot be turned into paths.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (event.needRescan()) return { kind: "everything" };
        // ```
        if event.need_rescan() {
            return Some(Change::Everything);
        }
        // What:     `let structural: bool = !event_is_a_plain_write(&event.kind);` asks whether the event is anything other than
        //           a write. The leading `!` negates the answer.
        // Why:      An entry that appeared, disappeared or was renamed is structural; a file whose bytes were only written is
        //           not, and needs no look inside it. Windows reports writes under another event kind than Unix does, which
        //           `event_is_a_plain_write` accounts for.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const structural: boolean = !eventIsAPlainWrite(event.kind);
        // ```
        let structural: bool = !event_is_a_plain_write(&event.kind);
        // What:     `let mut set: DirtyPaths = DirtyPaths::default();` makes a new empty, changeable set.
        // Why:      The event's paths are added to it in the loop that follows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const set = new DirtyPaths();
        // ```
        let mut set: DirtyPaths = DirtyPaths::default();
        // What:     `for path in &event.paths { ... }` visits every path of the event. `&event.paths` lends the list, so each
        //           `path` is a borrowed `&PathBuf` and the event keeps its list.
        // Why:      One event may name several paths (a rename names two).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const path of event.paths) {
        // ```
        for path in &event.paths {
            // What:     `if folder_was_only_read_or_touched(&event.kind, path) { continue; }` skips a folder that was merely
            //           touched, here and only on Windows.
            // Why:      Reading a folder's entries makes Windows report that folder as modified (its last-access time moved),
            //           so each refresh, whose walk reads every folder, would request the next one without end. The entries
            //           of a folder that really changed arrive as events of their own.
            if folder_was_only_read_or_touched(&event.kind, path) {
                continue;
            }
            // What:     `set.insert(path.clone(), structural);` copies the borrowed path into an owned one with `.clone()`
            //           and adds it. `.clone()` duplicates the path text.
            // Why:      The set must own its paths, but the loop only borrows them.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // set.insert(path, structural);
            // ```
            set.insert(path.clone(), structural);
        }
        // What:     `if set.is_empty() && !event.paths.is_empty() { return None; }` drops an event whose every path was a
        //           folder only touched.
        // Why:      Such an event changes nothing the list shows.
        if set.is_empty() && !event.paths.is_empty() {
            return None;
        }
        // What:     `if set.is_empty() { return Some(Change::Everything); }` leaves with "everything" when the event named no
        //           path at all.
        // Why:      An event with no paths cannot say what changed, so it is treated like lost events.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (set.isEmpty()) return { kind: "everything" };
        // ```
        if set.is_empty() {
            return Some(Change::Everything);
        }
        // What:     `return Some(Change::Paths(set));` wraps the set in the `Paths` case and then in `Some`, the present
        //           form of `Option`. `set` is moved into the value.
        // Why:      This is the normal answer: the event names these paths.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "paths", paths: set };
        // ```
        return Some(Change::Paths(set));
    }

    /// What: `pub(crate) fn merge(self, other: Change) -> Change` is the union of two changes; "everything" absorbs paths,
    /// and a set past the cap becomes "everything". `self` (with no `&`) takes the first change by value, so the call uses
    /// it up; `other` is taken by value too.
    /// Why: a storm of events must end in a bounded set, and a request that arrives while a refresh runs joins the one
    /// that follows it.
    /// Gotcha: after `a.merge(b)` neither `a` nor `b` can be used; the result is a new value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// merge(other: Change): Change { if (either is everything) return everything; absorb; if (size > MAX) return everything; return paths; }
    /// ```
    pub(crate) fn merge(self, other: Change) -> Change {
        // What:     `return match (self, other) { ... };` matches the pair of both changes at once, as a tuple. The first
        //           arm, `(Change::Everything, _) | (_, Change::Everything)`, matches when either side is `Everything` (`|`
        //           means "or" between patterns, and `_` ignores that side). The second arm,
        //           `(Change::Paths(mut left), Change::Paths(right))`, matches two sets and names them `left` (changeable,
        //           from `mut`) and `right`.
        // Why:      "Everything" swallows any paths; two sets are united, with a cap on the union.
        // Gotcha:   `match` must cover every pair; these two arms do, because a pair is either with an `Everything` or with
        //           two `Paths`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (self.kind === "everything" || other.kind === "everything") return { kind: "everything" };
        // const left = self.paths, right = other.paths;
        // ```
        return match (self, other) {
            (Change::Everything, _) | (_, Change::Everything) => Change::Everything,
            (Change::Paths(mut left), Change::Paths(right)) => {
                // What:     `left.absorb(right);` adds every path of `right` to `left`; `right` is moved in and is gone.
                // Why:      The two sets become one.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // left.absorb(right);
                // ```
                left.absorb(right);
                // What:     `if left.len() > MAX_DIRTY_PATHS { ... }` compares the count (a `usize`) with the constant.
                // Why:      A set past the cap would cost as much as a full walk and must not grow without bound.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (left.len() > MAX_DIRTY_PATHS) {
                // ```
                if left.len() > MAX_DIRTY_PATHS {
                    // What:     `tracing::info!(paths = left.len(), "...");` writes an info log line with one named field,
                    //           `paths`. `tracing::info!` is a macro (the `!` marks it).
                    // Why:      The repo logs every branch decision, and this one explains why a path set became "everything".
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // logger.info({ paths: left.len() }, "too many changed paths; the whole library is looked at instead");
                    // ```
                    tracing::info!(paths = left.len(), "too many changed paths; the whole library is looked at instead");
                    // What:     `return Change::Everything;` leaves the whole function (not just the block) with the case
                    //           that carries no data.
                    // Why:      The over-cap set is dropped and replaced by "everything".
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // return { kind: "everything" };
                    // ```
                    return Change::Everything;
                }
                // What:     `Change::Paths(left)` with no `;` is the value of this arm: a block's last expression is what the
                //           block produces. It builds the `Paths` case around the merged set, moving `left` into it.
                // Why:      This is the normal result: the united set.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return { kind: "paths", paths: left };
                // ```
                Change::Paths(left)
            }
        };
    }
}

/// What: `#[cfg(test)] #[path = "change_set_tests.rs"] mod tests;` declares the module `tests` from the file
/// `change_set_tests.rs`, compiled only for `cargo test`.
/// Why: the unit tests live in their own file to keep this one short, and stay out of the production build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: like a `change_set.test.ts` file that the production bundle never imports.
/// ```
#[cfg(test)]
#[path = "change_set_tests.rs"]
mod tests;
