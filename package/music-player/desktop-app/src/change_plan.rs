//! From the paths the watcher named to the edits of the queue, without walking the library.
//!
//! What: `plan_changes` looks at each named path on the disk and in the sorted queue and says which queue runs to remove,
//! which tracks to add and which listed tracks to look at again; `apply_plan` builds the new queue from the old one.
//! Why: D113 makes every watcher event a refresh. Walking and stat'ing a million tracks for one replaced file took
//! seconds of idle-priority work per event; this costs a few binary searches and one `lstat` per named path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planChanges(tracks, root, dirty, disk); const next = applyPlan(tracks, plan, root);
//! ```

/// What: `use std::collections::HashSet;` imports the hash set type from the standard library (`std`). It holds each value
/// at most once and answers "is this in the set" in constant time. Siblings: `Vec<T>` keeps order and duplicates and
/// answers "is this in the list" by scanning; `BTreeSet<T>` keeps the values sorted.
/// Why: the planner asks again and again whether a path was listed or is on disk, and a scan per question would be slow
/// in a folder with thousands of entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HashSet<T> = Set<T>; // a built-in global in TypeScript; Rust imports it
/// ```
use std::collections::HashSet;
/// What: `use std::ops::Range;` imports the half-open interval type `start..end`, which includes `start` and excludes `end`.
/// Why: a run of queue positions is a `Range<usize>`, which can slice the queue directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Range = { start: number; end: number }; // end is exclusive, like Array.prototype.slice(start, end)
/// ```
use std::ops::Range;
/// What: `use std::path::{Path, PathBuf};` imports two path types in one line: `Path` (a borrowed view of a path) and
/// `PathBuf` (an owned, growable path). Siblings: `&str` and `String` are the text-only counterparts.
/// Why: the queue stores owned paths (`PathBuf`) while the planner only looks at them (`Path`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string; type PathBuf = string; // TypeScript has one string type for both
/// ```
use std::path::{Path, PathBuf};

/// What: `use crate::change_set::DirtyPaths;` imports the record of paths the watcher named, from `src/change_set.rs`.
/// `crate::` means "from the root of this crate".
/// Why: it is the input of the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type DirtyPaths } from "./change_set";
/// ```
use crate::change_set::DirtyPaths;
/// What: `use crate::disk_view::{Disk, EntryKind};` imports the trait (an interface) that answers questions about the disk,
/// and the enum of what a path can be there.
/// Why: the planner asks a `Disk`, so tests can answer from memory instead of from the file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Disk, EntryKind } from "./disk_view";
/// ```
use crate::disk_view::{Disk, EntryKind};
/// What: `use crate::walk_order::{direct_files, locate, subtree_range, walk_cmp};` imports the four functions that search the
/// sorted queue by the walk's own order.
/// Why: finding a named path in the queue must cost a binary search, not a scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { directFiles, locate, subtreeRange, walkCmp } from "./walk_order";
/// ```
use crate::walk_order::{direct_files, locate, subtree_range, walk_cmp};

/// What: `pub(crate) struct Plan { removed, added, touched }` is the edits one change set asks of the queue. `pub(crate)`
/// makes it visible inside this crate only (siblings: plain `pub` reaches other crates; no keyword is private to the file).
/// `#[derive(Debug, Default, PartialEq, Eq)]` asks the compiler to write four standard abilities: printing with `{:?}`, an
/// empty value from `Plan::default()`, and `==` comparison.
/// Why: the queue is edited once, and the tracks that exist on both sides are handed to the measurement. Equality lets
/// tests compare a whole plan with `assert_eq!`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { removed: Range[]; added: string[]; touched: string[] };
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    /// What: `pub(crate) removed: Vec<Range<usize>>,` is index runs of the old queue that are gone, sorted and disjoint.
    /// `Vec<T>` is an owned, growable list (siblings: `&[T]` is a borrowed view, `[T; N]` a fixed-size array); `usize` is the
    /// unsigned integer that indexes memory (siblings: `u32`, `u64`).
    /// Why: runs, not single indexes, because a removed folder takes a whole stretch of the queue in one piece.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// removed: Range[];
    /// ```
    pub(crate) removed: Vec<Range<usize>>,
    /// What: `pub(crate) added: Vec<PathBuf>,` is tracks that are on disk and not in the queue, in queue order.
    /// Why: the queue is rebuilt by placing these in order, so they are kept sorted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// added: string[];
    /// ```
    pub(crate) added: Vec<PathBuf>,
    /// What: `pub(crate) touched: Vec<PathBuf>,` is tracks that are listed and still on disk and were named: their file may
    /// have been replaced.
    /// Why: the queue keeps them, but their length and peaks are measured again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// touched: string[];
    /// ```
    pub(crate) touched: Vec<PathBuf>,
}

/// What: `impl Plan { ... }` opens the block that holds the methods of `Plan`. Methods take `&self` (a read-only borrow of
/// the value) the way a TypeScript method reads `this`.
/// Why: asking a plan a question belongs to the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Plan { /* the methods of the impl block */ }
/// ```
impl Plan {
    /// What: `pub(crate) fn changes_the_list(&self) -> bool` says whether the queue gains or loses a track. `-> bool` is the
    /// return type.
    /// Why: a replaced file leaves the list alone, so nothing but the measurement follows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// changesTheList(): boolean { return this.removed.length > 0 || this.added.length > 0; }
    /// ```
    pub(crate) fn changes_the_list(&self) -> bool {
        // What:     `return !self.removed.is_empty() || !self.added.is_empty();` is true when either list has an item. `!` is
        //           "not" and `||` is "or", both as in TypeScript; `.is_empty()` asks a `Vec` whether it has no items.
        // Why:      Removing or adding a track is the only change that alters the list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return !(this.removed.length === 0) || !(this.added.length === 0);
        // ```
        return !self.removed.is_empty() || !self.added.is_empty();
    }
}

/// What: `struct Planner<'a> { tracks, root, disk, plan, walked_until }` is the state of one planning pass. `'a` is a lifetime
/// label: it says that the borrowed fields live at least as long as the `Planner`, so the compiler can prove that none of
/// them is used after the owner is gone. There is no `pub`, so it is private to this file.
/// Why: the helpers share the queue, the root and the disk, and write into one plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Planner { constructor(tracks: string[], root: string, disk: Disk) { /* plan starts empty */ } }
/// ```
struct Planner<'a> {
    /// What: `tracks: &'a [PathBuf],` is the queue, in walk order. `&'a [T]` is a borrowed view of a list, here for the
    /// lifetime `'a`.
    /// Why: the planner reads the queue and never changes it; the plan holds the edits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tracks: readonly string[];
    /// ```
    tracks: &'a [PathBuf],
    /// What: `root: &'a Path,` is the folder the queue was scanned from, borrowed.
    /// Why: the walk order is relative to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// root: string;
    /// ```
    root: &'a Path,
    /// What: `disk: &'a dyn Disk,` is the answers about the disk. `&'a dyn Disk` is a borrow of any value that implements the
    /// `Disk` trait; the real type is picked at run time (siblings: a generic `D: Disk` picks it at compile time).
    /// Why: one planner serves the real file system and the in-memory disks of the tests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// disk: Disk;
    /// ```
    disk: &'a dyn Disk,
    /// What: `plan: Plan,` is the edits so far. The field owns the value (no `&`), so the plan lives and dies with the planner.
    /// Why: the helpers push into it, and `plan_changes` takes it at the end.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan: Plan;
    /// ```
    plan: Plan,
    /// What: `walked_until: Option<PathBuf>,` is the last folder whose whole subtree was read; named paths below it are
    /// already covered. `Option<T>` is Rust's `T | null`: `Some(value)` or `None`.
    /// Why: reading a folder lists everything in it, so a named path inside needs no second visit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// walkedUntil: string | null;
    /// ```
    walked_until: Option<PathBuf>,
}

/// What: `impl Planner<'_> { ... }` opens the block that holds the methods of `Planner`. `'_` means "some lifetime, the
/// compiler fills it in", because no method needs to name it.
/// Why: the visiting steps belong to the planner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Planner { /* the methods of the impl block */ }
/// ```
impl Planner<'_> {
    /// What: `fn remove_path(&mut self, path: &Path)` removes a file or a whole folder's run from the queue. `&mut self` is a
    /// borrow that may change the value; Rust allows one such borrow at a time.
    /// Why: a path that is gone takes everything listed at or under it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// removePath(path: string): void { /* push the run of the path, or its single index */ }
    /// ```
    fn remove_path(&mut self, path: &Path) {
        // What:     `let run: Range<usize> = subtree_range(self.tracks, self.root, path);` asks for the stretch of the queue at or
        //           under the path. The `: Range<usize>` annotation names the type, which would also be inferred.
        // Why:      A folder that is gone has a stretch of tracks; a file has none, because nothing is listed under a file.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const run: Range = subtreeRange(this.tracks, this.root, path);
        // ```
        let run: Range<usize> = subtree_range(self.tracks, self.root, path);
        // What:     `if !run.is_empty() { ... } else if let Ok(index) = locate(...) { ... }` has two branches. The second is
        //           `if let`: it runs only when `locate` answers `Ok(index)`, and it binds the position to `index`. `locate`
        //           answers a `Result`: `Ok(index)` when the path is in the queue and `Err(place)` when it is not.
        //           `index..index + 1` builds the one-item range.
        // Why:      A folder removes its whole run; a file removes its own position if it is listed; a path that is not
        //           listed needs no edit.
        // Gotcha:   `Err` is not thrown; here it only means "not found", and the `if let` skips it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (run.end > run.start) this.plan.removed.push(run);
        // else { const index = locate(this.tracks, this.root, path); if (index !== null) this.plan.removed.push({ start: index, end: index + 1 }); }
        // ```
        if !run.is_empty() {
            self.plan.removed.push(run);
        } else if let Ok(index) = locate(self.tracks, self.root, path) {
            self.plan.removed.push(index..index + 1);
        }
    }

    /// What: `fn visit_track(&mut self, path: &Path)` adds a track that is not listed, or marks a listed one to be looked at
    /// again.
    /// Why: a replaced file keeps its path; a new one needs a row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visitTrack(path: string): void { /* drop a stale run, then touched or added */ }
    /// ```
    fn visit_track(&mut self, path: &Path) {
        // What:     `let run: Range<usize> = subtree_range(self.tracks, self.root, path);` asks for the tracks listed under the
        //           path as if it were a folder.
        // Why:      A folder that was replaced by a file of the same name leaves tracks listed under it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const run: Range = subtreeRange(this.tracks, this.root, path);
        // ```
        let run: Range<usize> = subtree_range(self.tracks, self.root, path);
        // What:     `if !run.is_empty() { self.plan.removed.push(run); }` removes that stale run.
        // Why:      Those tracks are gone, because the path is a file now.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (run.end > run.start) this.plan.removed.push(run);
        // ```
        if !run.is_empty() {
            self.plan.removed.push(run);
        }
        // What:     `match locate(...) { Ok(_) => ..., Err(_) => ... }` takes the answer apart. `Ok(_)` means "found" and `Err(_)`
        //           means "not found"; `_` ignores the carried position. `path.to_path_buf()` makes an owned copy of the borrowed
        //           path, because the plan keeps it after this call returns.
        // Why:      A listed track is touched; one that is not listed is added.
        // Gotcha:   `match` must cover every case of the `Result`; both are present.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (locate(this.tracks, this.root, path) !== null) this.plan.touched.push(path); else this.plan.added.push(path);
        // ```
        match locate(self.tracks, self.root, path) {
            Ok(_) => self.plan.touched.push(path.to_path_buf()),
            Err(_) => self.plan.added.push(path.to_path_buf()),
        }
    }

    /// What: `fn read_subtree(&mut self, folder: &Path)` replaces the run of a folder by what the disk holds under it now.
    /// Why: a folder that is new, renamed in or recreated may hold anything, and no event names its contents.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readSubtree(folder: string): void { /* merge the old run with a fresh walk, both sorted */ }
    /// ```
    fn read_subtree(&mut self, folder: &Path) {
        // What:     `let run: Range<usize> = subtree_range(self.tracks, self.root, folder);` is the stretch of the queue under the
        //           folder.
        // Why:      It is the old side of the comparison.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const run: Range = subtreeRange(this.tracks, this.root, folder);
        // ```
        let run: Range<usize> = subtree_range(self.tracks, self.root, folder);
        // What:     `let fresh: Vec<PathBuf> = self.disk.walk(folder);` asks the disk for every track under the folder, in walk order.
        // Why:      It is the new side of the comparison.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const fresh: string[] = this.disk.walk(folder);
        // ```
        let fresh: Vec<PathBuf> = self.disk.walk(folder);
        // What:     `let old: &[PathBuf] = &self.tracks[run.clone()];` borrows the old run as a view. `&` lends a slice of the queue
        //           and `[range]` picks the stretch; no track is copied. `run.clone()` copies the two numbers of the range, because
        //           indexing takes the range by value and `run` is used again.
        // Why:      Two sorted lists are compared in one pass, so a view is all that is needed.
        // Gotcha:   A slice is a borrow, not a copy: `old` is valid only while the queue is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const old: readonly string[] = this.tracks.slice(run.start, run.end);
        // ```
        let old: &[PathBuf] = &self.tracks[run.clone()];
        // What:     `let (mut left, mut right) = (0_usize, 0_usize);` makes two counters at once by taking a tuple apart. `0_usize` is
        //           the literal zero with its type written as a suffix. `mut` lets each variable change.
        // Why:      One counter walks the old run and the other walks the fresh walk.
        // Gotcha:   `let` without `mut` is TypeScript's `const`, and the default is `const`, the reverse of habit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let left = 0, right = 0;
        // ```
        let (mut left, mut right) = (0_usize, 0_usize);
        // What:     `while left < old.len() || right < fresh.len() { ... }` repeats until both lists are used up. `.len()` is the
        //           number of items.
        // Why:      A merge of two sorted lists stops only when neither has items left.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (left < old.length || right < fresh.length) {
        // ```
        while left < old.len() || right < fresh.len() {
            // What:     `let order = match (old.get(left), fresh.get(right)) { ... };` decides which side goes first. `.get(i)` answers
            //           an `Option`: `Some(item)` inside the list and `None` past its end, instead of the stop that `[i]` would give.
            //           The tuple of two options is matched by four arms. `(Some(a), Some(b)) if a == b` has a guard: the arm
            //           applies only when the condition after `if` holds. `(Some(_), None)` means only the old side has items left,
            //           and `_` (alone) takes every other case, which is only the fresh side having items left.
            // Why:      Equal paths are kept; a path only in the old run is gone (`Less`); a path only in the fresh walk is new
            //           (`Greater`); two different paths are ordered by the walk's own comparison.
            // Gotcha:   The arms are tried in order, so the guarded arm must come first, and `_` must come last.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const a = old[left], b = fresh[right];
            // const order = a !== undefined && b !== undefined ? (a === b ? 0 : walkCmp(root, a, b)) : a !== undefined ? -1 : 1;
            // ```
            let order = match (old.get(left), fresh.get(right)) {
                (Some(a), Some(b)) if a == b => std::cmp::Ordering::Equal,
                (Some(a), Some(b)) => walk_cmp(self.root, a, b),
                (Some(_), None) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Greater,
            };
            // What:     `match order { Equal => ..., Less => ..., Greater => ... }` acts on the three outcomes. `Ordering` is an enum
            //           with exactly these three cases, so `match` covers all of them and needs no fallback arm. `+= 1` adds one to
            //           a counter, and `fresh[right].clone()` copies the owned path because the plan keeps it.
            // Why:      `Equal` marks the track touched and moves both counters. `Less` removes the old position (the run's start plus
            //           the counter) and moves the old counter. `Greater` adds the fresh path and moves the fresh counter.
            // Gotcha:   `.clone()` on a `PathBuf` copies the text; on an `Rc` it would copy only a pointer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (order === 0) { touched.push(fresh[right]); left++; right++; }
            // else if (order < 0) { removed.push({ start: run.start + left, end: run.start + left + 1 }); left++; }
            // else { added.push(fresh[right]); right++; }
            // ```
            match order {
                std::cmp::Ordering::Equal => {
                    self.plan.touched.push(fresh[right].clone());
                    left += 1;
                    right += 1;
                }
                std::cmp::Ordering::Less => {
                    self.plan.removed.push(run.start + left..run.start + left + 1);
                    left += 1;
                }
                std::cmp::Ordering::Greater => {
                    self.plan.added.push(fresh[right].clone());
                    right += 1;
                }
            }
        }
        // What:     `self.walked_until = Some(folder.to_path_buf());` records the folder as read. `Some(...)` wraps the value in the
        //           present case of `Option`, and `.to_path_buf()` makes an owned copy of the borrowed path.
        // Why:      Later named paths below this folder are already covered by the read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.walkedUntil = folder;
        // ```
        self.walked_until = Some(folder.to_path_buf());
    }

    /// What: `fn compare_entries(&mut self, folder: &Path, run: Range<usize>)` compares a folder's direct entries with the
    /// queue's. `run` is taken by value: the caller hands over its range, which is two numbers, so this costs nothing.
    /// Why: an entry appeared or went in a folder that is already listed; only that folder's own entries are read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// compareEntries(folder: string, run: Range): void { /* files by set difference, then subfolders by their runs */ }
    /// ```
    fn compare_entries(&mut self, folder: &Path, run: Range<usize>) {
        // What:     `let entries: Vec<(PathBuf, EntryKind)> = self.disk.entries(folder);` lists the folder's direct entries, each a
        //           tuple of a path and what it is. A tuple is a fixed group like TypeScript's `[string, EntryKind]`.
        // Why:      One directory listing is enough to find new and missing entries.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entries: [string, EntryKind][] = this.disk.entries(folder);
        // ```
        let entries: Vec<(PathBuf, EntryKind)> = self.disk.entries(folder);
        // What:     `let on_disk_files: HashSet<&Path> = entries.iter().filter(...).map(...).collect();` builds the set of file paths.
        //           `.iter()` walks the list by reference, `.filter(|(_, kind)| ...)` keeps the items whose closure answers true
        //           (the closure's parameter takes the tuple apart and ignores the path with `_`), `.map(|(path, _)| ...)` turns each
        //           kept item into a borrowed path, and `.collect()` gathers them into the type the annotation names. `*kind` reads the
        //           kind through its reference, and `path.as_path()` borrows the owned path as a `&Path`.
        // Why:      Membership questions are asked many times, which a set answers in constant time. The set borrows from `entries`,
        //           so no path is copied.
        // Gotcha:   Iterator steps are lazy; nothing runs until `.collect()` pulls the items through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const onDiskFiles = new Set(entries.filter(([, kind]) => kind === EntryKind.Track).map(([path]) => path));
        // ```
        let on_disk_files: HashSet<&Path> = entries.iter().filter(|(_, kind)| return *kind == EntryKind::Track).map(|(path, _)| return path.as_path()).collect();
        // What:     `let on_disk_folders: HashSet<&Path> = ...;` is the same chain for the entries that are folders.
        // Why:      Subfolders are compared by their own set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const onDiskFolders = new Set(entries.filter(([, kind]) => kind === EntryKind.Folder).map(([path]) => path));
        // ```
        let on_disk_folders: HashSet<&Path> = entries.iter().filter(|(_, kind)| return *kind == EntryKind::Folder).map(|(path, _)| return path.as_path()).collect();
        // What:     `let own: Range<usize> = direct_files(self.tracks, folder, run.clone());` finds the part of the run that holds the
        //           folder's own files, before its subfolders. `run.clone()` copies the range, because `run` is used again.
        // Why:      The walk lists a folder's files first, so they form one stretch at the start of the run.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const own: Range = directFiles(this.tracks, folder, run);
        // ```
        let own: Range<usize> = direct_files(self.tracks, folder, run.clone());
        // What:     `let mut listed_files: HashSet<&Path> = HashSet::new();` makes an empty set that the loop fills. `::new()` is an
        //           associated function (a constructor, like a static method).
        // Why:      The files the queue lists are compared with the files on disk.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const listedFiles = new Set<string>();
        // ```
        let mut listed_files: HashSet<&Path> = HashSet::new();
        // What:     `for index in own.clone() { ... }` visits every position of the folder's own files. A `Range` is an iterator, and
        //           `.clone()` copies it because `own` is used again.
        // Why:      Each listed file is noted, and one that is no longer on disk is removed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (let index = own.start; index < own.end; index++) {
        // ```
        for index in own.clone() {
            // What:     `let track: &Path = self.tracks[index].as_path();` borrows the path at that position.
            // Why:      The set and the lookup both take borrowed paths.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const track: string = this.tracks[index];
            // ```
            let track: &Path = self.tracks[index].as_path();
            // What:     `listed_files.insert(track);` adds the path to the set.
            // Why:      Later, files on disk that are missing from it are the new ones.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // listedFiles.add(track);
            // ```
            listed_files.insert(track);
            // What:     `if !on_disk_files.contains(track) { self.plan.removed.push(index..index + 1); }` removes a listed file that is
            //           not on disk. `.contains` asks the set.
            // Why:      The file went away.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!onDiskFiles.has(track)) this.plan.removed.push({ start: index, end: index + 1 });
            // ```
            if !on_disk_files.contains(track) {
                self.plan.removed.push(index..index + 1);
            }
        }
        // What:     `for file in &on_disk_files { ... }` visits each path of the set by reference; `&` lends the set instead of
        //           using it up. `(*file).to_path_buf()` follows the extra reference with `*` and copies the path into an owned one.
        // Why:      A file on disk that the queue does not list is new.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const file of onDiskFiles) if (!listedFiles.has(file)) this.plan.added.push(file);
        // ```
        for file in &on_disk_files {
            if !listed_files.contains(file) {
                self.plan.added.push((*file).to_path_buf());
            }
        }
        // What:     `let mut listed_folders: HashSet<PathBuf> = HashSet::new();` makes an empty set of owned folder paths.
        // Why:      It collects the subfolders the queue lists, which are found one at a time and need owned values.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const listedFolders = new Set<string>();
        // ```
        let mut listed_folders: HashSet<PathBuf> = HashSet::new();
        // What:     `let mut position: usize = own.end;` starts a cursor after the folder's own files.
        // Why:      The rest of the run holds the subfolders' tracks, one subfolder after another.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let position = own.end;
        // ```
        let mut position: usize = own.end;
        // What:     `while position < run.end { ... }` repeats until the cursor leaves the run.
        // Why:      Each pass handles one whole subfolder and jumps past it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (position < run.end) {
        // ```
        while position < run.end {
            // What:     `let Some(child) = ... else { position += 1; continue; };` is `let ... else`: it binds `child` when the value on
            //           the right is `Some(child)`, and otherwise runs the `else` block, which must leave the loop pass. The right side
            //           strips the folder off the track's path (`.strip_prefix(folder)` answers a `Result`), turns it into an `Option`
            //           with `.ok()`, and `.and_then(|below| ...)` continues only when that is `Some`, taking the first name of the rest
            //           with `.components().next()`.
            // Why:      The first name under the folder is the subfolder that holds the track. A track that is not under the folder
            //           cannot be placed, so the cursor skips it.
            // Gotcha:   `continue` jumps to the next pass of the `while`, as in TypeScript; the cursor must move first, or the loop
            //           would never end.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const child = firstNameUnder(folder, this.tracks[position]);
            // if (child === null) { position += 1; continue; }
            // ```
            let Some(child) = self.tracks[position].strip_prefix(folder).ok().and_then(|below| return below.components().next()) else {
                position += 1;
                continue;
            };
            // What:     `let subfolder: PathBuf = folder.join(child.as_os_str());` builds the subfolder's full path. `.join` appends a
            //           name to a path and answers a new owned path; `.as_os_str()` views the step as a name.
            // Why:      Subfolders are compared as full paths.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const subfolder: string = path.join(folder, child);
            // ```
            let subfolder: PathBuf = folder.join(child.as_os_str());
            // What:     `let sub_run: Range<usize> = subtree_range(self.tracks, self.root, &subfolder);` finds the queue stretch under the
            //           subfolder; `&subfolder` lends the path.
            // Why:      The whole stretch is removed or skipped as one piece.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const subRun: Range = subtreeRange(this.tracks, this.root, subfolder);
            // ```
            let sub_run: Range<usize> = subtree_range(self.tracks, self.root, &subfolder);
            // What:     `if !on_disk_folders.contains(subfolder.as_path()) { self.plan.removed.push(sub_run.clone()); }` removes the
            //           stretch of a subfolder that is not on disk any more.
            // Why:      The subfolder was deleted or renamed away.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!onDiskFolders.has(subfolder)) this.plan.removed.push(subRun);
            // ```
            if !on_disk_folders.contains(subfolder.as_path()) {
                self.plan.removed.push(sub_run.clone());
            }
            // What:     `listed_folders.insert(subfolder);` adds the subfolder to the set. The value moves into the set, so `subfolder`
            //           cannot be used after this line.
            // Why:      Later, folders on disk that are missing from the set are the new ones.
            // Gotcha:   Moving a value ends its use in that variable; the compiler rejects a later read of `subfolder`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // listedFolders.add(subfolder);
            // ```
            listed_folders.insert(subfolder);
            // What:     `position = sub_run.end.max(position + 1);` moves the cursor past the subfolder's stretch. `.max(other)` picks the
            //           larger of two numbers.
            // Why:      Taking the larger one guarantees progress even when the stretch is empty, so the loop always ends.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // position = Math.max(subRun.end, position + 1);
            // ```
            position = sub_run.end.max(position + 1);
        }
        // What:     `for subfolder in on_disk_folders { if !listed_folders.contains(subfolder) { self.read_subtree(subfolder); } }` walks
        //           the folders on disk, and reads each one the queue does not list. The loop takes the set by value, so the set is
        //           used up.
        // Why:      A folder that appeared holds tracks that no event names, so it is read in full.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const subfolder of onDiskFolders) if (!listedFolders.has(subfolder)) this.readSubtree(subfolder);
        // ```
        for subfolder in on_disk_folders {
            if !listed_folders.contains(subfolder) {
                self.read_subtree(subfolder);
            }
        }
    }

    /// What: `fn visit(&mut self, path: &Path, structural: bool)` handles one named path. `structural` says that the event was
    /// a rename, creation or removal of the entry itself, not a change inside it.
    /// Why: what the path is now decides what the queue needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// visit(path: string, structural: boolean): void { /* skip covered paths, then dispatch on what the path is now */ }
    /// ```
    fn visit(&mut self, path: &Path, structural: bool) {
        // What:     `if !path.starts_with(self.root) || self.walked_until.as_deref().is_some_and(|folder| return path.starts_with(folder)) { return; }`
        //           leaves early in two cases. `.as_deref()` views the optional owned path as an optional borrowed one, and
        //           `.is_some_and(f)` is true only when the option holds a value for which the closure answers true.
        // Why:      A path outside the root is not the queue's business, and a path inside a folder read in full is already covered.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!path.startsWith(this.root) || (this.walkedUntil !== null && path.startsWith(this.walkedUntil))) return;
        // ```
        if !path.starts_with(self.root) || self.walked_until.as_deref().is_some_and(|folder| return path.starts_with(folder)) {
            return;
        }
        // What:     `match self.disk.kind(path) { ... }` dispatches on what the path is on disk now. `EntryKind::Missing | EntryKind::Other`
        //           is one arm with two patterns joined by `|`. Each arm is an expression, and the arms' values are all `()`, Rust's
        //           empty value.
        // Why:      A missing path, or one that is neither a track nor a folder, is removed. A track is added or touched. A folder is
        //           read.
        // Gotcha:   `match` must cover every `EntryKind`; the compiler rejects a new variant that is not handled.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.disk.kind(path)) { case "Missing": case "Other": this.removePath(path); break; case "Track": this.visitTrack(path); break; case "Folder": ... }
        // ```
        match self.disk.kind(path) {
            EntryKind::Missing | EntryKind::Other => self.remove_path(path),
            EntryKind::Track => self.visit_track(path),
            EntryKind::Folder => {
                // What:     `if let Ok(index) = locate(self.tracks, self.root, path) { ... }` runs only when the folder's own path is in the
                //           queue, and binds its position to `index`.
                // Why:      The queue lists tracks, so a folder's path in it means that a track used to be at that path.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const index = locate(this.tracks, this.root, path);
                // if (index !== null) this.plan.removed.push({ start: index, end: index + 1 });
                // ```
                if let Ok(index) = locate(self.tracks, self.root, path) {
                    self.plan.removed.push(index..index + 1);
                }
                // What:     `let run: Range<usize> = subtree_range(self.tracks, self.root, path);` is the stretch of the queue under the
                //           folder.
                // Why:      Its size decides between a full read and a comparison of the entries.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const run: Range = subtreeRange(this.tracks, this.root, path);
                // ```
                let run: Range<usize> = subtree_range(self.tracks, self.root, path);
                // What:     `if structural || run.is_empty() { self.read_subtree(path); } else { self.compare_entries(path, run); }` picks one
                //           of two readings. `||` is "or".
                // Why:      A folder that is new or was renamed in, or one the queue does not list yet, is read in full. A listed folder
                //           that only changed inside needs only its direct entries compared.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (structural || run.end === run.start) this.readSubtree(path); else this.compareEntries(path, run);
                // ```
                if structural || run.is_empty() {
                    self.read_subtree(path);
                } else {
                    self.compare_entries(path, run);
                }
            }
        }
    }
}

/// What: `fn merge_runs(mut runs: Vec<Range<usize>>) -> Vec<Range<usize>>` sorts index runs and joins the ones that overlap or
/// touch. `mut runs` makes the parameter changeable inside the function; the caller's list is moved in, so it is not seen
/// changing.
/// Why: two named paths can remove the same track, and the queue is rebuilt in one pass over disjoint runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeRuns(runs: Range[]): Range[] { /* sort by start, then extend the last run while the next one starts inside it */ }
/// ```
fn merge_runs(mut runs: Vec<Range<usize>>) -> Vec<Range<usize>> {
    // What:     `runs.sort_by_key(|run| return run.start);` sorts the list in place by the key the closure returns. `|run| ...` is a
    //           closure (an anonymous function) whose parameters sit between the bars.
    // Why:      Overlapping runs must be neighbors before they can be joined in one pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // runs.sort((a, b) => a.start - b.start);
    // ```
    runs.sort_by_key(|run| return run.start);
    // What:     `let mut merged: Vec<Range<usize>> = Vec::new();` makes an empty list that the loop fills.
    // Why:      The joined runs are built in a new list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const merged: Range[] = [];
    // ```
    let mut merged: Vec<Range<usize>> = Vec::new();
    // What:     `for run in runs { ... }` visits each run and takes it out of the list (`runs` is used up).
    // Why:      Each run either extends the last joined run or starts a new one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const run of runs) {
    // ```
    for run in runs {
        // What:     `match merged.last_mut() { Some(last) if run.start <= last.end => ..., _ => merged.push(run) }` looks at the last joined
        //           run. `.last_mut()` answers an `Option` of a changeable borrow of it. The guard `if run.start <= last.end` makes the
        //           first arm apply only for a run that overlaps or touches it; `last.end = last.end.max(run.end)` then stretches it.
        //           `_` takes every other case: the list is empty, or the run starts later.
        // Why:      Runs are sorted by start, so only the last joined run can overlap the next one.
        // Gotcha:   `<=` also joins runs that only touch, such as `0..2` and `2..5`, which is intended.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const last = merged.at(-1);
        // if (last !== undefined && run.start <= last.end) last.end = Math.max(last.end, run.end); else merged.push(run);
        // ```
        match merged.last_mut() {
            Some(last) if run.start <= last.end => last.end = last.end.max(run.end),
            _ => merged.push(run),
        }
    }
    // What:     `return merged;` hands the joined list to the caller; ownership moves out of the function.
    // Why:      The caller gets disjoint runs in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return merged;
    // ```
    return merged;
}

/// What: `pub(crate) fn plan_changes(tracks, root, dirty, disk) -> Plan` plans the queue edits for the named paths. Its full
/// signature is `(tracks: &[PathBuf], root: &Path, dirty: &DirtyPaths, disk: &dyn Disk) -> Plan`; every parameter is a borrow,
/// so the caller keeps its queue and the function only reads.
/// Why: see the module description. Paths are visited in path order so a folder is read before what is inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planChanges(tracks: readonly string[], root: string, dirty: DirtyPaths, disk: Disk): Plan { ... }
/// ```
pub(crate) fn plan_changes(tracks: &[PathBuf], root: &Path, dirty: &DirtyPaths, disk: &dyn Disk) -> Plan {
    // What:     `let mut planner: Planner = Planner { tracks, root, disk, plan: Plan::default(), walked_until: None };` builds the
    //           planner. Bare names such as `tracks` are shorthand for `tracks: tracks`; `Plan::default()` is an empty plan and
    //           `None` is the absent case of `Option`.
    // Why:      The planner shares the inputs with its helpers and collects the plan.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const planner = new Planner(tracks, root, disk);
    // ```
    let mut planner: Planner = Planner { tracks, root, disk, plan: Plan::default(), walked_until: None };
    // What:     `for (path, structural) in dirty.sorted() { planner.visit(&path, structural); }` visits each named path. The loop
    //           variable is a tuple taken apart into two names, and `&path` lends the owned path to `visit`.
    // Why:      Sorted order puts a folder before what is inside it, so `walked_until` can skip the covered paths.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [path, structural] of dirty.sorted()) planner.visit(path, structural);
    // ```
    for (path, structural) in dirty.sorted() {
        planner.visit(&path, structural);
    }
    // What:     `let mut plan: Plan = planner.plan;` moves the plan out of the planner. The planner is not used afterwards.
    // Why:      The tidying that follows changes the plan, so it needs to own it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plan: Plan = planner.plan;
    // ```
    let mut plan: Plan = planner.plan;
    // What:     `plan.removed = merge_runs(plan.removed);` replaces the removed runs by the joined runs. The old value moves into
    //           the function and the answer is stored back.
    // Why:      Two named paths can remove the same track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plan.removed = mergeRuns(plan.removed);
    // ```
    plan.removed = merge_runs(plan.removed);
    // What:     `plan.added.sort_by(|left, right| return walk_cmp(root, left, right));` sorts the new tracks in walk order. `sort_by`
    //           takes a closure that answers an `Ordering` for two items.
    // Why:      The queue is rebuilt by placing the additions in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plan.added.sort((left, right) => walkCmp(root, left, right));
    // ```
    plan.added.sort_by(|left, right| return walk_cmp(root, left, right));
    // What:     `plan.added.dedup();` drops items equal to the one before them.
    // Why:      After sorting, equal paths are neighbors, and two events can name the same new file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plan.added = plan.added.filter((path, index) => index === 0 || path !== plan.added[index - 1]);
    // ```
    plan.added.dedup();
    // What:     `let mut seen: HashSet<PathBuf> = HashSet::new();` makes an empty set of the paths met so far.
    // Why:      It lets the retain call drop repeats without sorting, so the order of `touched` stays.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = new Set<string>();
    // ```
    let mut seen: HashSet<PathBuf> = HashSet::new();
    // What:     `plan.touched.retain(|path| return seen.insert(path.clone()));` keeps the items for which the closure answers true.
    //           `seen.insert(...)` answers true when the value was not in the set yet, so only the first copy of a path stays.
    // Why:      A path can be touched through more than one named path.
    // Gotcha:   The closure changes `seen` as a side effect; `retain` runs it once per item in order, so the first copy wins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plan.touched = plan.touched.filter((path) => { if (seen.has(path)) return false; seen.add(path); return true; });
    // ```
    plan.touched.retain(|path| return seen.insert(path.clone()));
    // What:     `tracing::info!(named = ..., "the named paths are planned");` writes one log event. `!` marks a macro (code that is
    //           expanded at compile time), `name = value` pairs become structured fields, and the string is the message.
    // Why:      The sizes of the plan show, in the log, how much work one refresh asked for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // logger.info({ named: dirty.size, removedRuns: plan.removed.length, added: plan.added.length, touched: plan.touched.length }, "the named paths are planned");
    // ```
    tracing::info!(
        named = dirty.len(),
        removed_runs = plan.removed.len(),
        added = plan.added.len(),
        touched = plan.touched.len(),
        "the named paths are planned"
    );
    // What:     `return plan;` hands the plan to the caller; ownership moves out of the function.
    // Why:      The caller applies it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return plan;
    // ```
    return plan;
}

/// What: `pub(crate) fn apply_plan(old: Vec<PathBuf>, root: &Path, plan: &Plan) -> Vec<PathBuf>` builds the new queue: the old
/// one without the removed runs and with the added tracks in their places. `old` is taken by value, so the caller's queue is
/// moved in and its paths are reused.
/// Why: one pass that moves paths and clones none; the additions are placed by binary search in the old queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyPlan(old: string[], root: string, plan: Plan): string[] { /* merge old, minus removed runs, with added */ }
/// ```
pub(crate) fn apply_plan(old: Vec<PathBuf>, root: &Path, plan: &Plan) -> Vec<PathBuf> {
    // What:     `let places: Vec<usize> = plan.added.iter().map(|path| return locate(&old, root, path).unwrap_or_else(|place| return place)).collect();`
    //           finds, for each added track, the position it takes in the old queue. `locate` answers `Ok(index)` for a found path
    //           and `Err(place)` for the position where it would go; `.unwrap_or_else(f)` takes the `Ok` value, or calls the closure
    //           with the `Err` value and uses what it answers. Here both cases give a position.
    // Why:      A binary search per addition is cheap and keeps the additions in order.
    // Gotcha:   `.unwrap_or_else` is lazy: its closure runs only on `Err`, unlike `.unwrap_or(value)`, which computes the value always.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const places: number[] = plan.added.map((path) => insertionPoint(old, root, path));
    // ```
    let places: Vec<usize> = plan.added.iter().map(|path| return locate(&old, root, path).unwrap_or_else(|place| return place)).collect();
    // What:     `let mut rebuilt: Vec<PathBuf> = Vec::with_capacity(old.len() + plan.added.len());` makes an empty list with room for
    //           the most items it can hold. `Vec::with_capacity(n)` reserves memory for `n` items at once.
    // Why:      One reservation avoids growing the list again and again for a queue of a million tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rebuilt: string[] = []; // JavaScript arrays grow on their own; Rust can reserve room up front
    // ```
    let mut rebuilt: Vec<PathBuf> = Vec::with_capacity(old.len() + plan.added.len());
    // What:     `let mut next_added: usize = 0;` is the index of the next addition to place.
    // Why:      The additions are sorted, so one counter walks them in step with the old queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let nextAdded = 0;
    // ```
    let mut next_added: usize = 0;
    // What:     `let mut runs = plan.removed.iter().peekable();` makes an iterator over the removed runs that can look at the next one
    //           without taking it. `.peekable()` adds `.peek()`.
    // Why:      The loop must know the next run before it decides whether to skip a track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let runIndex = 0; // peek is plan.removed[runIndex]
    // ```
    let mut runs = plan.removed.iter().peekable();
    // What:     `for (index, track) in old.into_iter().enumerate() { ... }` visits each old track with its position. `.into_iter()`
    //           takes the list by value, so each path moves out without a copy, and `.enumerate()` pairs each with a counter from 0.
    // Why:      The positions are what the removed runs and the additions are expressed in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [index, track] of old.entries()) {
    // ```
    for (index, track) in old.into_iter().enumerate() {
        // What:     `while next_added < places.len() && places[next_added] <= index { ... }` pushes every addition that belongs before
        //           this old track, and `+= 1` moves to the next one. `&&` is "and".
        // Why:      An addition placed at position `p` goes right before the old track at `p`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (nextAdded < places.length && places[nextAdded] <= index) rebuilt.push(plan.added[nextAdded++]);
        // ```
        while next_added < places.len() && places[next_added] <= index {
            rebuilt.push(plan.added[next_added].clone());
            next_added += 1;
        }
        // What:     `while runs.peek().is_some_and(|run| return run.end <= index) { runs.next(); }` drops the runs that end before this
        //           track. `.peek()` answers an `Option` of the next run without taking it, `.is_some_and(f)` is true when it holds a
        //           value the closure accepts, and `.next()` takes it.
        // Why:      A run that ended can no longer remove anything, and the next run is the one that matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (runIndex < plan.removed.length && plan.removed[runIndex].end <= index) runIndex++;
        // ```
        while runs.peek().is_some_and(|run| return run.end <= index) {
            runs.next();
        }
        // What:     `if runs.peek().is_some_and(|run| return run.contains(&index)) { continue; }` skips the track when the next run
        //           holds its position. `run.contains(&index)` asks a range whether it includes a number, and `continue` jumps to the
        //           next pass of the `for`.
        // Why:      A track in a removed run is not copied to the new queue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (runIndex < plan.removed.length && index >= plan.removed[runIndex].start) continue;
        // ```
        if runs.peek().is_some_and(|run| return run.contains(&index)) {
            continue;
        }
        // What:     `rebuilt.push(track);` appends the old track. The path moves in; nothing is copied.
        // Why:      A track that no run removed stays.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // rebuilt.push(track);
        // ```
        rebuilt.push(track);
    }
    // What:     `rebuilt.extend(plan.added[next_added..].iter().cloned());` appends the additions that belong after the last old track.
    //           `[next_added..]` is the slice from that position to the end, `.iter()` walks it by reference, and `.cloned()` copies
    //           each item so the plan stays intact.
    // Why:      Additions placed at or past the end of the old queue were never reached by the loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rebuilt.push(...plan.added.slice(nextAdded));
    // ```
    rebuilt.extend(plan.added[next_added..].iter().cloned());
    // What:     `return rebuilt;` hands the new queue to the caller; ownership moves out of the function.
    // Why:      The caller replaces the old queue with it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rebuilt;
    // ```
    return rebuilt;
}

/// What: `#[cfg(test)] #[path = "change_plan_tests.rs"] mod tests;` pulls the unit tests in from a sibling file. `#[cfg(test)]`
/// compiles the module only for `cargo test`, and `#[path = "..."]` names the file, which is otherwise `tests.rs` or
/// `tests/mod.rs`.
/// Why: the tests are long, so they sit in their own file, and the release build never contains them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import "./change_plan_tests"; // loaded only by the test runner
/// ```
#[cfg(test)]
#[path = "change_plan_tests.rs"]
mod tests;
