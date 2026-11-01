//! The order of the folder walk as a comparison, so a path can be found in the queue without walking the folder.
//!
//! What: `walk_cmp` orders two paths the way `scan_streaming` lists them (a folder's own files in natural order, then
//! its subfolders in natural order, each subfolder's tree in full), and `locate`, `subtree_range` and `direct_files`
//! use it to search the sorted queue by binary search.
//! Why: D113 refreshes only what the watcher named. Deciding whether a named path is already in a queue of a million
//! tracks must cost a few comparisons, not a walk of the library or a copy of the queue.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const locate = (tracks: string[], root: string, path: string) => bsearch(tracks, (t) => walkCmp(root, t, path));
//! ```

/// What: `use std::cmp::Ordering;` imports the three-way comparison result enum (`Less`, `Equal`, `Greater`) from the
/// standard library (`std`). Siblings: a plain `bool` answers only "less or not", and a TypeScript comparator returns a
/// negative, zero or positive `number`; `Ordering` names those three outcomes.
/// Why: every function here compares paths and answers with one of the three outcomes, which sorting and binary search
/// both consume.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ordering = "Less" | "Equal" | "Greater"; // a comparator would return -1, 0 or 1
/// ```
use std::cmp::Ordering;
/// What: `use std::ffi::OsStr;` imports the borrowed operating-system string type. It is the text of one file or folder
/// name exactly as the file system stores it (siblings: `&str` must be valid UTF-8, and `OsString` is the owned version).
/// Why: file names on Linux may hold bytes that are not valid UTF-8, so a path component cannot always be a `&str`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OsStr = string; // TypeScript has one string type; Rust separates file-system text from UTF-8 text
/// ```
use std::ffi::OsStr;
/// What: `use std::ops::Range;` imports the half-open interval type `start..end`, which includes `start` and excludes `end`.
/// Why: the functions answer "which run of queue positions" as a `Range<usize>`, which can slice the queue directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Range = { start: number; end: number }; // end is exclusive, like Array.prototype.slice(start, end)
/// ```
use std::ops::Range;
/// What: `use std::path::{Component, Path, PathBuf};` imports three path types in one line: `Component` (one step of a
/// path: a name, `..`, `.` or the root), `Path` (a borrowed view of a path) and `PathBuf` (an owned, growable path).
/// Siblings of the last two: `&str` and `String` are the text-only counterparts.
/// Why: the walk order compares paths level by level, which needs the steps (`Component`), and the queue stores owned
/// paths (`PathBuf`) while the functions only look at them (`Path`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { sep } from "node:path"; // paths are plain strings in TS; there is no Component, Path or PathBuf
/// ```
use std::path::{Component, Path, PathBuf};

/// What: `use crate::natural_order::natural_cmp;` imports the repo's natural-order comparison (so `Track 2` sorts before
/// `Track 10`) from `src/natural_order.rs`.
/// Why: the folder walk sorts names naturally, and this module must reproduce that order exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { naturalCmp } from "./natural_order";
/// ```
use crate::natural_order::natural_cmp;

/// What: `fn compare_names(left: &OsStr, right: &OsStr) -> Ordering` orders two path components the way the walk sorts
/// siblings: natural order of the name, then the plain bytes. There is no `pub`, so it is private to this file.
/// `&OsStr` is a borrow: `&` means "I only look at this value, I do not take it". `-> Ordering` is the return type.
/// Why: `sort_naturally` in the folder walk breaks ties with the whole path; siblings share their parent, so the
/// components decide.
/// Gotcha: a `&OsStr` argument is never copied; the caller keeps the name and the function only reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const compareNames = (left: string, right: string): Ordering => naturalCmp(left, right) || bytes(left, right);
/// ```
fn compare_names(left: &OsStr, right: &OsStr) -> Ordering {
    // What:     `return natural_cmp(&left.to_string_lossy(), &right.to_string_lossy()).then_with(|| return left.cmp(right));`
    //           first turns each name into text with `.to_string_lossy()` (a copy-on-demand that replaces bytes that are
    //           not valid UTF-8 with the replacement character) and lends both texts to `natural_cmp` with `&`. The
    //           `Ordering` it answers has `.then_with(...)`, which keeps the answer unless it is `Equal` and only then
    //           runs the closure. `|| return left.cmp(right)` is a closure: `||` is its empty parameter list, and the
    //           body compares the raw names byte by byte with `.cmp`.
    // Why:      Two names that read the same naturally (`a1` and `A1`) must still have a fixed order, so the bytes break
    //           the tie, exactly as the walk does.
    // Gotcha:   The closure is lazy: `.then_with` runs it only on a tie, unlike `.then(value)`, which would compute the
    //           byte comparison every time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return naturalCmp(String(left), String(right)) || compareBytes(left, right);
    // ```
    return natural_cmp(&left.to_string_lossy(), &right.to_string_lossy()).then_with(|| return left.cmp(right));
}

/// What: `fn names_below<'a>(root: &Path, path: &'a Path) -> impl Iterator<Item = &'a OsStr>` gives the names of the path
/// below the root, one per level. `'a` is a lifetime label: it says that the names returned are slices of `path`, so they
/// are valid only as long as `path` is. `impl Iterator<Item = &'a OsStr>` means "some lazy sequence of names", without
/// naming the exact iterator type.
/// Why: every comparison works level by level without building a list; a path outside the root keeps all its names.
/// Gotcha: the returned sequence is lazy; nothing is computed until a caller pulls names from it, like a TypeScript
/// generator and unlike a `map` that builds an array.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function* namesBelow(root: string, path: string): Generator<string> { /* yield each normal path step after root */ }
/// ```
fn names_below<'a>(root: &Path, path: &'a Path) -> impl Iterator<Item = &'a OsStr> {
    // What:     `let below: &Path = path.strip_prefix(root).unwrap_or(path);` removes the root from the front of the path.
    //           `strip_prefix` answers a `Result`: `Ok(rest)` when the path starts with the root and `Err(_)` when it does
    //           not. `.unwrap_or(path)` takes the `Ok` value, or the given fallback (`path` itself) on `Err`. The `: &Path`
    //           annotation says that `below` is a borrowed path.
    // Why:      A path that is not under the root keeps all its steps, so the comparison still has something to work with.
    // Gotcha:   `.unwrap_or(value)` on a `Result` silently drops the error; here that is the intended fallback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const below: string = path.startsWith(root) ? path.slice(root.length) : path;
    // ```
    let below: &Path = path.strip_prefix(root).unwrap_or(path);
    // What:     `return below.components().filter_map(|component| { ... });` splits the path into steps and keeps only the
    //           ones the closure turns into `Some(name)`. `.components()` is a lazy iterator of `Component`s, and
    //           `.filter_map(f)` calls `f` on each one: a `Some(x)` result is kept as `x`, a `None` is dropped.
    //           `|component| { ... }` is a closure (an anonymous function) with one parameter; the parameters sit
    //           between the bars where TypeScript writes them in parentheses before `=>`.
    // Why:      Only real names count as levels; the root marker and `.` or `..` steps are not names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return splitPath(below).flatMap((component) => (component.kind === "normal" ? [component.name] : []));
    // ```
    return below.components().filter_map(|component| {
        // What:     `return match component { Component::Normal(name) => Some(name), _ => None };` takes the step apart
        //           by its kind. `Component::Normal(name)` matches the ordinary-name variant and binds its text to `name`;
        //           `_` matches every other variant. `Some(name)` wraps the name in the present case of `Option`, and
        //           `None` is the absent case; `Option` is Rust's `T | null`.
        // Why:      Keep the names and drop the rest, as `filter_map` asks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return component.kind === "normal" ? component.name : null;
        // ```
        return match component {
            Component::Normal(name) => Some(name),
            _ => None,
        };
    });
}

/// What: `pub(crate) fn walk_cmp(root: &Path, left: &Path, right: &Path) -> Ordering` compares two file paths in the order
/// the folder walk lists them. `pub(crate)` makes it visible inside this crate only (siblings: plain `pub` reaches other
/// crates; no keyword keeps it private to this file).
/// Why: at the first level where the names differ, a file (the last level of its path) comes before a subfolder (a
/// level with more below it), and two of the same kind compare by name. This is exactly the walk: files first, then
/// each subfolder with everything under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walkCmp = (root: string, left: string, right: string): Ordering => { /* compare level by level, files first */ };
/// ```
pub(crate) fn walk_cmp(root: &Path, left: &Path, right: &Path) -> Ordering {
    // What:     `let mut left_names = names_below(root, left).peekable();` builds the lazy sequence of the left path's names
    //           and wraps it with `.peekable()`, which adds `.peek()` (look at the next name without taking it). `mut`
    //           makes the variable changeable; Rust variables are constants unless marked.
    // Why:      The loop below pulls names one level at a time and must also ask "is there another level after this one"
    //           to tell a file from a folder.
    // Gotcha:   `let` without `mut` is TypeScript's `const`, and the default is `const`, the reverse of habit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let leftNames = peekable(namesBelow(root, left));
    // ```
    let mut left_names = names_below(root, left).peekable();
    // What:     The same `peekable` iterator for the right path.
    // Why:      Both paths are walked in step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let rightNames = peekable(namesBelow(root, right));
    // ```
    let mut right_names = names_below(root, right).peekable();
    // What:     `loop { ... }` repeats forever until a `return` leaves the function.
    // Why:      Each pass compares one level, and the comparison ends as soon as one level decides.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) {
    // ```
    loop {
        // What:     `let (a, b) = match (left_names.next(), right_names.next()) { ... };` pulls the next name from each
        //           side and matches the pair, which is a tuple of two `Option`s. `(None, None)` means both paths ended;
        //           the other arms handle one side ending first; `(Some(a), Some(b))` binds both names and the arm's value
        //           `(a, b)` is a new tuple that the `let` splits into `a` and `b`.
        //           `Ordering::Equal`, `Ordering::Less` and `Ordering::Greater` are the variants of the imported enum.
        // Why:      Two equal paths end together; a path that ends first is a folder-prefix of the other, so it sorts
        //           before it.
        // Gotcha:   `.next()` consumes the name; a later `.next()` gives the following level.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const l = leftNames.next(), r = rightNames.next();
        // if (l.done && r.done) return "Equal"; if (l.done) return "Less"; if (r.done) return "Greater";
        // const [a, b] = [l.value, r.value];
        // ```
        let (a, b) = match (left_names.next(), right_names.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) => (a, b),
        };
        // What:     `if a == b { continue; }` skips to the next pass of the loop when the two names are equal.
        // Why:      Equal names say nothing about the order, so the decision lies deeper.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (a === b) continue;
        // ```
        if a == b {
            continue;
        }
        // What:     `return match (left_names.peek().is_none(), right_names.peek().is_none()) { ... };` asks, for each
        //           side, whether this name is the last level (`.peek()` looks at what follows without taking it, and
        //           `.is_none()` is true when nothing follows). The pair of booleans is matched: `(true, false)` is a file
        //           on the left against a folder on the right, `(false, true)` the reverse, and `_` is every other case.
        // Why:      Files come before subfolders at the same level; two of the same kind compare by name with
        //           `compare_names`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const leftIsFile = !leftNames.hasNext(), rightIsFile = !rightNames.hasNext();
        // if (leftIsFile && !rightIsFile) return "Less"; if (!leftIsFile && rightIsFile) return "Greater";
        // return compareNames(a, b);
        // ```
        return match (left_names.peek().is_none(), right_names.peek().is_none()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => compare_names(a, b),
        };
    }
}

/// What: `pub(crate) fn locate(tracks: &[PathBuf], root: &Path, path: &Path) -> Result<usize, usize>` finds a file in the
/// sorted queue: its index, or where it would be inserted. `&[PathBuf]` is a borrowed slice, a read-only view of a run of
/// owned paths (siblings: `Vec<PathBuf>` owns the paths, and `[PathBuf; N]` is a fixed-size array). `Result<usize, usize>`
/// is `Ok(index)` for found and `Err(place)` for not found. `usize` is the unsigned integer wide enough to index memory
/// (siblings: `u32`, `u64`, `i32`).
/// Why: a binary search over the walk order; the queue is exactly the walk of its root, so the order holds. A slice is
/// taken, not a `Vec`, so the caller keeps the queue; `usize` is used, not `u32`, because queue positions index a `Vec`.
/// Gotcha: `Err` here is not a failure; it is the insertion point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const locate = (tracks: string[], root: string, path: string): { found: number } | { insertAt: number } => bsearch(...);
/// ```
pub(crate) fn locate(tracks: &[PathBuf], root: &Path, path: &Path) -> Result<usize, usize> {
    // What:     `return tracks.binary_search_by(|track| return walk_cmp(root, track, path));` runs a binary search where
    //           the closure `|track| ...` tells, for each probed track, how it compares with the wanted path.
    // Why:      The queue is sorted in walk order, so the search needs only about twenty comparisons for a million tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return binarySearchBy(tracks, (track) => walkCmp(root, track, path));
    // ```
    return tracks.binary_search_by(|track| return walk_cmp(root, track, path));
}

/// What: `fn against_folder(root: &Path, track: &Path, folder: &[&OsStr]) -> Ordering` says where a track lies relative
/// to everything under a folder: `Less` before it, `Equal` inside it, `Greater` after it. `&[&OsStr]` is a borrowed slice
/// of borrowed names.
/// Why: the tracks under one folder are one run of the walk, so two partition points bound the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const againstFolder = (root: string, track: string, folder: string[]): Ordering => { /* before, inside or after */ };
/// ```
fn against_folder(root: &Path, track: &Path, folder: &[&OsStr]) -> Ordering {
    // What:     `let mut names = names_below(root, track).peekable();` builds the peekable sequence of the track's names.
    // Why:      The folder's names are compared with the track's names level by level, and the end of the track decides
    //           the last case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let names = peekable(namesBelow(root, track));
    // ```
    let mut names = names_below(root, track).peekable();
    // What:     `for wanted in folder { ... }` visits each name of the folder in order; because `folder` is a borrowed
    //           slice, `wanted` is itself a `&&OsStr`, a borrow of a borrow.
    // Why:      A track is inside the folder only if every level of the folder is the track's level too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const wanted of folder) {
    // ```
    for wanted in folder {
        // What:     `let Some(name) = names.next() else { return Ordering::Less; };` takes the track's next name; when the
        //           track has no more names (`None`) the `else` block leaves with `Less`.
        // Why:      A track that ends before the folder's path does cannot be inside it; it sorts before.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = names.next(); if (name == null) return "Less";
        // ```
        let Some(name) = names.next() else {
            return Ordering::Less;
        };
        // What:     `if name == *wanted { continue; }` compares the track's name with the folder's name. `*wanted` is a
        //           dereference: it follows the borrow to the value, so both sides are `&OsStr`.
        // Why:      Equal names mean this level matches and the next level decides.
        // Gotcha:   `*x` reads through a reference; it is not a pointer multiplication and has no TypeScript equivalent.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (name === wanted) continue;
        // ```
        if name == *wanted {
            continue;
        }
        // What:     `return if names.peek().is_none() { Ordering::Less } else { compare_names(name, wanted) };` is an
        //           `if` expression used as the returned value. When the track's name is its last level the track is a
        //           file here and sorts before the folder; otherwise the two sibling names decide.
        // Why:      Files precede subfolders at one level, as in the walk.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return names.isLast() ? "Less" : compareNames(name, wanted);
        // ```
        return if names.peek().is_none() { Ordering::Less } else { compare_names(name, wanted) };
    }
    // What:     `return if names.peek().is_some() { Ordering::Equal } else { Ordering::Less };` is the tail after every
    //           level of the folder matched. `.is_some()` is true when the track has more names after them.
    // Why:      A track with more names below the folder is inside it (`Equal`); a track that is exactly the folder's path
    //           is not under it, so it sorts before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return names.hasNext() ? "Equal" : "Less";
    // ```
    return if names.peek().is_some() { Ordering::Equal } else { Ordering::Less };
}

/// What: `pub(crate) fn subtree_range(tracks: &[PathBuf], root: &Path, folder: &Path) -> Range<usize>` gives the indexes
/// of every track under `folder`, however deep (empty when there is none), as a half-open `start..end` range.
/// Why: a removed or renamed folder takes its whole run out of the queue, and a new folder is told apart from a known
/// one by whether it has a run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subtreeRange = (tracks: string[], root: string, folder: string): { start: number; end: number } => { ... };
/// ```
pub(crate) fn subtree_range(tracks: &[PathBuf], root: &Path, folder: &Path) -> Range<usize> {
    // What:     `let names: Vec<&OsStr> = names_below(root, folder).collect();` drains the lazy sequence into a growable
    //           list. `.collect()` builds whatever collection the annotation names; here `Vec<&OsStr>` (siblings: `HashSet`
    //           or a slice cannot be built in place). The names stay borrowed from `folder`, so nothing is copied.
    // Why:      The two searches below reuse the folder's names many times, so they are listed once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [...namesBelow(root, folder)];
    // ```
    let names: Vec<&OsStr> = names_below(root, folder).collect();
    // What:     `let start: usize = tracks.partition_point(|track| return against_folder(root, track, &names) == Ordering::Less);`
    //           finds the first position where the closure stops being true, assuming the answers are true and then false
    //           (`partition_point` is a binary search for that border). `&names` lends the `Vec` where a slice
    //           `&[&OsStr]` is expected; the `Vec` coerces to a slice.
    // Why:      Everything before the folder's run sorts `Less`, so the border is the run's start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const start: number = partitionPoint(tracks, (track) => againstFolder(root, track, names) === "Less");
    // ```
    let start: usize = tracks.partition_point(|track| return against_folder(root, track, &names) == Ordering::Less);
    // What:     `let end: usize = tracks.partition_point(|track| return against_folder(root, track, &names) != Ordering::Greater);`
    //           is the same search with `!=`: the border after the last track that is `Less` or `Equal`.
    // Why:      Everything up to the run's last track is not `Greater`, so the border is the run's end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const end: number = partitionPoint(tracks, (track) => againstFolder(root, track, names) !== "Greater");
    // ```
    let end: usize = tracks.partition_point(|track| return against_folder(root, track, &names) != Ordering::Greater);
    // What:     `return start..end.max(start);` builds the range `start..end`, raising `end` to `start` when it is smaller.
    //           `..` is the range constructor; `.max` picks the larger of two integers.
    // Why:      The range is never backwards (`end < start` would be an invalid slice), and an empty run is `start..start`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { start, end: Math.max(end, start) };
    // ```
    return start..end.max(start);
}

/// What: `pub(crate) fn direct_files(tracks: &[PathBuf], folder: &Path, run: Range<usize>) -> Range<usize>` gives the
/// files that sit directly in `folder` within its run: they come first. `run` is taken by value (a `Range` is two
/// integers, cheap to copy).
/// Why: a folder's own files precede its subfolders, so the direct files are a prefix of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directFiles = (tracks: string[], folder: string, run: Range): Range => ...;
/// ```
pub(crate) fn direct_files(tracks: &[PathBuf], folder: &Path, run: Range<usize>) -> Range<usize> {
    // What:     `let inside: &[PathBuf] = &tracks[run.clone()];` takes the part of the queue that the range covers as a
    //           borrowed slice. `tracks[range]` slices by a range, and the leading `&` lends the slice instead of copying
    //           it. `run.clone()` copies the range, because slicing uses it up and the function needs `run.start` later.
    // Why:      Only the folder's own run is searched, not the whole queue.
    // Gotcha:   `.clone()` here copies two integers; it never copies the paths.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inside: string[] = tracks.slice(run.start, run.end);
    // ```
    let inside: &[PathBuf] = &tracks[run.clone()];
    // What:     `let count: usize = inside.partition_point(|track| return track.parent() == Some(folder));` counts the
    //           leading tracks whose parent folder is `folder`. `track.parent()` answers `Option<&Path>`; `Some(folder)`
    //           builds the matching `Option` so that the two can be compared with `==`.
    // Why:      Direct files are a prefix of the run, so a border search finds how many there are.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count: number = partitionPoint(inside, (track) => dirname(track) === folder);
    // ```
    let count: usize = inside.partition_point(|track| return track.parent() == Some(folder));
    // What:     `return run.start..run.start + count;` builds the range of the direct files, which starts where the run
    //           starts and is `count` long.
    // Why:      The caller needs positions in the whole queue, so the run's start is added back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { start: run.start, end: run.start + count };
    // ```
    return run.start..run.start + count;
}

/// What: `#[cfg(test)] #[path = "walk_order_tests.rs"] mod tests;` declares the module `tests` from the file
/// `walk_order_tests.rs`, compiled only for `cargo test`.
/// Why: the unit tests live in their own file to keep this one short, and stay out of the production build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: like a `walk_order.test.ts` file that the production bundle never imports.
/// ```
#[cfg(test)]
#[path = "walk_order_tests.rs"]
mod tests;
