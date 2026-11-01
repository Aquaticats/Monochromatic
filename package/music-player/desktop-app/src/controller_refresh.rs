//! Refreshes the library when the watcher reports changes: the queue edits, the selection and the measurement.
//!
//! What: `impl Controller` methods that turn a `Change` into queue edits (`handle_source_changed`), walk the whole root
//! when events were lost (`rescan_everything`), and start or queue the refresh sweeps through the coalescer.
//! Why: D113 makes the library follow the disk with no action by the person. The cost of a refresh must follow what
//! changed, so a replaced file in a library of a million tracks costs lookups for that one path, not a walk and a stat of
//! every file.

/// What: `use std::collections::HashSet;` imports the hash set type from the standard library (`std`). Siblings: `Vec`
/// keeps duplicates and order, and `BTreeSet` keeps its members sorted.
/// Why: the refresh asks "is this path in the set" for many paths and must not hold a path twice, which a hash set does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Set is global in TypeScript; nothing to import.
/// ```
use std::collections::HashSet;
/// What: `use std::path::{Path, PathBuf};` imports two path types: `Path`, a borrowed read-only view of a path, and
/// `PathBuf`, an owned and growable path. Siblings: `&str` and `String` are the text-only counterparts, which cannot hold
/// file names that are not valid UTF-8.
/// Why: the methods look at a root they are lent (`&Path`) and keep paths they own (`PathBuf`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Paths are plain strings in TypeScript.
/// ```
use std::path::{Path, PathBuf};
/// What: `use std::sync::Arc;` imports the atomically counted shared pointer. Siblings: `Rc<T>` is the single-thread version,
/// `Box<T>` has exactly one owner, and a plain `T` is owned by one variable.
/// Why: a sweep runs on other threads and shares the report sink and the claim set with the controller, so both need an
/// owner count that is safe across threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object is shared by reference and garbage collected; there is nothing to import.
/// ```
use std::sync::Arc;

/// What: `use crate::change_plan::{Plan, apply_plan, plan_changes};` imports one type and two functions from
/// `src/change_plan.rs`. `crate::` means "from the root of this crate", and the braces import several names in one line.
/// Why: the refresh plans the queue edits from named paths and applies them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Plan, applyPlan, planChanges } from "./change_plan";
/// ```
use crate::change_plan::{Plan, apply_plan, plan_changes};
/// What: `use crate::change_set::Change;` imports the "paths or everything" enum.
/// Why: the first method takes a `Change`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Change } from "./change_set";
/// ```
use crate::change_set::Change;
/// What: `use crate::command::Update;` imports the engine-to-window message enum.
/// Why: the selection handling sends messages to the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Update } from "./command";
/// ```
use crate::command::Update;
/// What: `use crate::controller::Controller;` imports the playback state machine type that the methods attach to.
/// Why: Rust allows an `impl` block only on a type named in the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Controller } from "./controller";
/// ```
use crate::controller::Controller;
/// What: `use crate::disk_view::RealDisk;` imports the file-system implementation of the `Disk` trait.
/// Why: the planner asks the real disk about each named path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RealDisk } from "./disk_view";
/// ```
use crate::disk_view::RealDisk;
/// What: `use crate::measure::spawn_queue_measurement;` imports the function that starts a background measurement sweep.
/// Why: a refresh ends by measuring the files that changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnQueueMeasurement } from "./measure";
/// ```
use crate::measure::spawn_queue_measurement;
/// What: `use crate::playback::expand_paths;` imports the function that walks folders and lists the audio files in them.
/// Why: a full rescan needs the whole walk once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { expandPaths } from "./playback";
/// ```
use crate::playback::expand_paths;
/// What: `use crate::refresh_coalescer::{Files, Launch, Work};` imports three types from `src/refresh_coalescer.rs`: the
/// files a sweep looks at, a sweep to start, and the work a request asks for.
/// Why: requests and launches are exchanged with the coalescer as these values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Files, type Launch, type Work } from "./refresh_coalescer";
/// ```
use crate::refresh_coalescer::{Files, Launch, Work};
/// What: `use crate::sweep_gate::SweepGate;` imports the pause gate type of the background sweep.
/// Why: a sweep job carries the shared gate so Pause stops its workers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SweepGate } from "./sweep_gate";
/// ```
use crate::sweep_gate::SweepGate;
/// What: `use crate::sweep_report::SweepJob;` imports the record that holds everything one sweep needs to start.
/// Why: `spawn_queue_measurement` takes exactly this record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type SweepJob } from "./sweep_report";
/// ```
use crate::sweep_report::SweepJob;
/// What: `use crate::walk_order::locate;` imports the binary search over the walk order.
/// Why: the selected track is found in the new queue without scanning it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { locate } from "./walk_order";
/// ```
use crate::walk_order::locate;

/// What: `impl Controller { ... }` opens a block of methods for the `Controller` struct. A struct holds data, and one type
/// may have several `impl` blocks, even in different files.
/// Why: the controller is large, so its methods are split over several files; this block holds the refresh ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Like adding methods to a class in a second file (declaration merging):
/// interface Controller { handleSourceChanged(change: Change): void; /* ... */ }
/// ```
impl Controller {
    /// What: `pub(crate) fn handle_source_changed(&mut self, change: Change)` applies one debounced change of the Source Root:
    /// the named paths, or everything. `pub(crate)` makes it visible inside this crate only (siblings: plain `pub` reaches
    /// other crates; no keyword is private to the file). `&mut self` is an exclusive borrow of the controller: the method may
    /// change it, and no one else may touch it meanwhile. `change` is taken by value, so the caller gives it up.
    /// Why: named paths are planned against the sorted queue and the disk; only an added or removed track changes the list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handleSourceChanged(change: Change): void { if (change.kind === "everything") return this.rescanEverything(); /* plan, apply, refresh */ }
    /// ```
    pub(crate) fn handle_source_changed(&mut self, change: Change) {
        // What:     `let Change::Paths(dirty) = change else { ...; return; };` is a `let ... else`. It matches `change` against
        //           the `Paths` case of the enum and, when it fits, names the set inside it `dirty`; when `change` is
        //           `Everything` the `else` block runs and must leave the function.
        // Why:      "Everything" has no paths to plan, so it goes to the full rescan, and the rest of the method can rely
        //           on `dirty`.
        // Gotcha:   `change` is moved into the pattern here; after the line it cannot be used by name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (change.kind !== "paths") { logger.info(...); this.rescanEverything(); return; }
        // const dirty = change.paths;
        // ```
        let Change::Paths(dirty) = change else {
            // What:     `tracing::info!("...");` writes an info-level log line with no fields. `tracing::info!` is a macro
            //           (the `!` marks code that expands at compile time).
            // Why:      The repo logs every branch decision; this one explains the full rescan.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.info("the watcher lost events; every track is looked at");
            // ```
            tracing::info!("the watcher lost events; every track is looked at");
            // What:     `self.rescan_everything();` calls the method defined later in this block; `self` carries the
            //           exclusive borrow along.
            // Why:      Lost events cannot be turned into paths.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.rescanEverything();
            // ```
            self.rescan_everything();
            // What:     `return;` leaves the method with no value (its return type is `()`, Rust's empty value, close to `void`).
            // Why:      The full rescan did all the work.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return;
        };
        // What:     `let Some(root) = self.source_root.clone() else { ...; return; };` is a `let ... else` over an `Option`.
        //           `self.source_root` is an `Option<PathBuf>` (Rust's `PathBuf | null`: `Some(path)` or `None`); `.clone()`
        //           copies the path out so that `root` does not keep the controller borrowed; `Some(root)` binds the path
        //           when there is one, and the `else` block runs when there is none.
        // Why:      Without an opened folder there is nothing to refresh. The copy lets the method change other controller
        //           fields while it holds `root`.
        // Gotcha:   `.clone()` here is a real copy of the path text, not a second reference to the same string.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const root = this.sourceRoot; if (root == null) { logger.debug(...); return; }
        // ```
        let Some(root) = self.source_root.clone() else {
            // What:     `tracing::debug!("...");` writes a debug-level log line with no fields.
            // Why:      A change with no open folder is unexpected but harmless, so the line explains the early exit.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug("a change was reported with no folder open; ignored");
            // ```
            tracing::debug!("a change was reported with no folder open; ignored");
            return;
        };
        // What:     `tracing::debug!(named = dirty.len(), "...");` writes a debug line with one named field. `dirty.len()` is
        //           the count of distinct named paths, a `usize` (the unsigned integer wide enough to index memory;
        //           siblings: `u32`, `u64`).
        // Why:      The size of the burst explains the cost of what follows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug({ named: dirty.size }, "the watcher named changed paths");
        // ```
        tracing::debug!(named = dirty.len(), "the watcher named changed paths");
        // What:     `if !root.is_dir() { self.rescan_everything(); return; }` asks the file system whether the root is still a
        //           folder. The leading `!` is "not", as in TypeScript.
        // Why:      A root that is gone or replaced by a file cannot be planned against, so the full rescan decides what
        //           the library holds now.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!isDirectory(root)) { this.rescanEverything(); return; }
        // ```
        if !root.is_dir() {
            self.rescan_everything();
            return;
        }
        // What:     `let started: std::time::Instant = std::time::Instant::now();` reads the monotonic clock. `Instant` is a
        //           point on a clock that only moves forward (siblings: `SystemTime` is wall-clock time and can jump). The
        //           path `std::time::Instant` is spelled in full because no `use` imported it.
        // Why:      The log below reports how long planning, editing and settling took.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const started: number = performance.now();
        // ```
        let started: std::time::Instant = std::time::Instant::now();
        // What:     `let plan: Plan = plan_changes(self.queue.tracks(), &root, &dirty, &RealDisk);` asks the planner what to do.
        //           Every argument is lent with `&` (a read-only borrow, "I only look"): `self.queue.tracks()` already answers
        //           a borrowed slice, `&root` and `&dirty` borrow the local values, and `&RealDisk` borrows a fresh value of
        //           the unit struct that carries the real file-system answers.
        // Why:      The planner needs only to read the queue, the root and the set, and must not take them, because they are
        //           used again below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const plan: Plan = planChanges(this.queue.tracks(), root, dirty, new RealDisk());
        // ```
        let plan: Plan = plan_changes(self.queue.tracks(), &root, &dirty, &RealDisk);
        // What:     `let planned: std::time::Duration = started.elapsed();` measures the time since `started`. `Duration` is a
        //           span of time (siblings: `Instant` is a point, not a span).
        // Why:      It is the planning time for the log line.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const planned: number = performance.now() - started;
        // ```
        let planned: std::time::Duration = started.elapsed();
        // What:     `let selected_path: Option<PathBuf> = self.queue.current_path().cloned();` reads the selected track's path as
        //           an owned copy. `current_path()` answers `Option<&PathBuf>` (an optional borrow), and `.cloned()` turns it
        //           into `Option<PathBuf>` by copying the path when there is one.
        // Why:      The selection is looked up again after the list changes, so the path is kept apart from the queue.
        // Gotcha:   `.cloned()` on an `Option` copies the inside; it does not clone the `Option` wrapper itself.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const selectedPath: string | null = this.queue.currentPath();
        // ```
        let selected_path: Option<PathBuf> = self.queue.current_path().cloned();
        // What:     `if plan.changes_the_list() { ... }` runs the block only when the plan adds or removes a track.
        // Why:      A replaced file leaves the list alone, so only a real list change needs the queue edited.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (plan.changesTheList()) {
        // ```
        if plan.changes_the_list() {
            // What:     `self.queue.replace_tracks_with(|old| return apply_plan(old, &root, &plan));` edits the queue's track list.
            //           `|old| return apply_plan(...)` is a closure (an anonymous function whose parameters sit between the
            //           bars); the queue calls it once with the old list handed over by value, and it answers the new list.
            //           `&root` and `&plan` lend the values the closure reads.
            // Why:      The queue can move the paths out of its own list into the new one instead of copying a million paths.
            // Gotcha:   The closure takes `old` by value, so the queue's list is empty while the closure runs.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.queue.replaceTracksWith((old) => applyPlan(old, root, plan));
            // ```
            self.queue.replace_tracks_with(|old| return apply_plan(old, &root, &plan));
            // What:     `let edited: std::time::Duration = started.elapsed();` measures the time since `started`.
            // Why:      It is the edit time for the log line.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const edited: number = performance.now() - started;
            // ```
            let edited: std::time::Duration = started.elapsed();
            // What:     `self.settle_selection(&root, selected_path.clone());` keeps or clears the selection after the edit.
            //           `&root` lends the root; `selected_path.clone()` passes a copy of the `Option<PathBuf>` so that the
            //           original stays usable after the block.
            // Why:      The queue's cursor is stale after an edit, and the selected track must be found again by its path.
            // Gotcha:   `.clone()` here copies the whole path text; the original `selected_path` is needed again below.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.settleSelection(root, selectedPath);
            // ```
            self.settle_selection(&root, selected_path.clone());
            // What:     `tracing::debug!(planned_us = planned.as_micros(), edited_us = ..., settled_us = ..., "...");` writes a
            //           debug line with three named fields. `.as_micros()` turns a `Duration` into whole microseconds.
            // Why:      The three timings show where a refresh spent its time.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug({ plannedUs, editedUs, settledUs }, "the list was edited");
            // ```
            tracing::debug!(planned_us = planned.as_micros(), edited_us = edited.as_micros(), settled_us = started.elapsed().as_micros(), "the list was edited");
        }
        // What:     `if selected_path.as_ref().is_some_and(|selected| return plan.touched.contains(selected)) { ... }` asks
        //           whether the selected track is among the touched files. `.as_ref()` turns `&Option<PathBuf>` into
        //           `Option<&PathBuf>` (a look inside without moving), and `.is_some_and(f)` answers true when the `Option` is
        //           `Some` and the closure `f` is true for its content. `|selected| ...` is a closure.
        //           `plan.touched.contains(selected)` is a list lookup.
        // Why:      Only a replaced selected file needs its decoder reloaded.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (selectedPath != null && plan.touched.includes(selectedPath)) {
        // ```
        if selected_path.as_ref().is_some_and(|selected| return plan.touched.contains(selected)) {
            // What:     `self.reload_current_when_its_file_changed();` calls the method of `controller_reload.rs`.
            // Why:      It swaps in the new file at the same position.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.reloadCurrentWhenItsFileChanged();
            // ```
            self.reload_current_when_its_file_changed();
        }
        // What:     `let announce: HashSet<PathBuf> = plan.added.into_iter().collect();` turns the plan's added list into a set.
        //           `.into_iter()` walks the list by value (consuming `plan.added`, so the paths are moved, not copied), and
        //           `.collect()` gathers them into the annotated type `HashSet<PathBuf>` (sibling: `Vec<PathBuf>` would keep
        //           the list as it was).
        // Why:      The added tracks are the ones whose rows are new and whose headers must be announced; a set answers
        //           membership quickly and drops repeats.
        // Gotcha:   `plan.added` is moved out of `plan`; later code may still use `plan.touched`, because only that field was
        //           taken.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const announce: Set<string> = new Set(plan.added);
        // ```
        let announce: HashSet<PathBuf> = plan.added.into_iter().collect();
        // What:     `let mut check: HashSet<PathBuf> = plan.touched.into_iter().chain(announce.iter().cloned()).collect();` builds
        //           the set of every file to check. `plan.touched.into_iter()` walks the touched files by value;
        //           `.chain(...)` appends a second sequence after the first; `announce.iter().cloned()` walks the announced
        //           set by borrow and copies each path; `.collect()` gathers all into a `HashSet<PathBuf>`. `mut` makes the
        //           set changeable.
        // Why:      A file that was added must also have its fingerprint checked, so `check` is touched plus announced.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const check: Set<string> = new Set([...plan.touched, ...announce]);
        // ```
        let mut check: HashSet<PathBuf> = plan.touched.into_iter().chain(announce.iter().cloned()).collect();
        // What:     `check.extend(self.recorded_failures());` adds the paths that failed to decode in a previous pass. `.extend(list)` takes
        //           any sequence by value and inserts each item.
        // Why:      D113: a failed decode is retried at the next library change, whichever file changed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const p of this.recordedFailures()) check.add(p);
        // ```
        check.extend(self.recorded_failures());
        // What:     `self.request_refresh(Work::Files(Files { check, announce }));` asks for a refresh sweep of these files.
        //           `Files { check, announce }` builds a record, and a field written as a bare name (`check`, `announce`) is
        //           shorthand for `check: check`. `Work::Files(...)` wraps it in the `Files` case of the `Work` enum, a tagged
        //           union whose cases may carry data. Both sets are moved into the value.
        // Why:      The coalescer decides whether to start the sweep now or join it to the pending follow-up.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.requestRefresh({ kind: "files", files: { check, announce } });
        // ```
        self.request_refresh(Work::Files(Files { check, announce }));
    }

    /// What: `pub(crate) fn rescan_everything(&mut self)` walks the Source Root again, replaces the queue when the list
    /// differs, and refreshes every track.
    /// Why: the fallback when the watcher lost events or a registration just ended: nothing names what changed. An
    /// unchanged list starts a quiet refresh sweep, which measures a file whose fingerprint changed (D113) and nothing
    /// else; the watcher ignores the reads it makes, so it cannot loop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rescanEverything(): void { const fresh = expandPaths([root]); if (equal(fresh, tracks)) { refreshAll(); return; } setTracks(fresh); settle(); sweepAll(); }
    /// ```
    pub(crate) fn rescan_everything(&mut self) {
        // What:     `let Some(root) = self.source_root.clone() else { ...; return; };` is the same `let ... else` as in
        //           `handle_source_changed`: it copies the optional root and leaves when there is none.
        // Why:      Without an opened folder there is nothing to walk.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const root = this.sourceRoot; if (root == null) return;
        // ```
        let Some(root) = self.source_root.clone() else {
            // What:     The same debug log call as in `handle_source_changed`, with the message of this method.
            // Why:      The exit is explained in the log.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug("a rescan was requested with no folder open; ignored");
            // ```
            tracing::debug!("a rescan was requested with no folder open; ignored");
            return;
        };
        // What:     `tracing::info!(root = %root.display(), "...");` writes an info line with one named field. `%` asks for the
        //           human-readable (`Display`) text, and `.display()` gives a path such a text, because a path has no built-in one.
        // Why:      A full walk is expensive enough to deserve an info line.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.info({ root }, "the folder is walked again and every track is refreshed");
        // ```
        tracing::info!(root = %root.display(), "the folder is walked again and every track is refreshed");
        // What:     `let fresh: Vec<PathBuf> = expand_paths(vec![root.clone()]);` walks the folder. `vec![root.clone()]` is a macro
        //           (the `!` marks code that expands at compile time) that builds a `Vec` holding one copy of the root;
        //           `expand_paths` takes a `Vec<PathBuf>` by value and answers every audio file under it in queue order.
        //           `Vec<T>` is an owned, growable list (siblings: `&[T]` is a borrowed view, `[T; N]` a fixed-size array).
        // Why:      The walk is the source of truth for what the folder holds now. The root is copied because `expand_paths`
        //           takes ownership of the list and the method needs `root` again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const fresh: string[] = expandPaths([root]);
        // ```
        let fresh: Vec<PathBuf> = expand_paths(vec![root.clone()]);
        // What:     `if fresh.as_slice() == self.queue.tracks() { ... }` compares the two lists. `.as_slice()` views the `Vec` as
        //           a borrowed slice `&[PathBuf]`, the type `tracks()` answers, and `==` compares the slices element by
        //           element, as `JSON.stringify` equality would, not by identity.
        // Why:      An unchanged list needs no queue edit, only a quiet refresh.
        // Gotcha:   `==` on slices compares contents; in TypeScript `===` on arrays would compare identity.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (equalArrays(fresh, this.queue.tracks())) {
        // ```
        if fresh.as_slice() == self.queue.tracks() {
            // What:     `self.reload_current_when_its_file_changed();` swaps in the selected file if it was replaced.
            // Why:      Even with an unchanged list the selected file's bytes may have changed.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.reloadCurrentWhenItsFileChanged();
            // ```
            self.reload_current_when_its_file_changed();
            // What:     `self.request_refresh(Work::Everything);` asks for a refresh of every track. `Work::Everything` is the case
            //           of the `Work` enum that carries no data.
            // Why:      Any file in the queue may have been replaced while events were lost.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.requestRefresh({ kind: "everything" });
            // ```
            self.request_refresh(Work::Everything);
            return;
        }
        // What:     `let selected_path: Option<PathBuf> = self.queue.current_path().cloned();` keeps an owned copy of the selected
        //           path, the same line as in `handle_source_changed`.
        // Why:      The selection is looked up in the new list by its path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const selectedPath: string | null = this.queue.currentPath();
        // ```
        let selected_path: Option<PathBuf> = self.queue.current_path().cloned();
        // What:     `self.queue.set_tracks(fresh);` replaces the queue's track list with the walked one. `fresh` is moved in, so
        //           this method cannot use it afterwards.
        // Why:      The list differs from the walk, so the walk wins.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.queue.setTracks(fresh);
        // ```
        self.queue.set_tracks(fresh);
        // What:     `self.settle_selection(&root, selected_path);` re-anchors the selection. `&root` lends the root, and
        //           `selected_path` is moved into the call.
        // Why:      The cursor is stale after a new list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.settleSelection(root, selectedPath);
        // ```
        self.settle_selection(&root, selected_path);
        // What:     `self.start_queue_measurement();` starts the loading sweep over the new queue.
        // Why:      A different list needs every track measured, not only a refresh of changed files.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.startQueueMeasurement();
        // ```
        self.start_queue_measurement();
    }

    /// What: `fn recorded_failures(&self) -> Vec<PathBuf>` lists the files that failed to decode and have not decoded since,
    /// minus the ones that no longer exist. `&self` is a read-only borrow of the controller. There is no `pub`, so the
    /// method is private to this file.
    /// Why: D113: a failed decode is retried at the next library change, whichever file changed. The set is the files already
    /// reported as unplayable, which is small, so adding it to every refresh keeps the cost bounded. A removed file is
    /// forgotten here so the set cannot grow with deleted files. A file that decodes again leaves the set when its peak is
    /// measured, so it is not retried after that.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private recordedFailures(): string[] { reported.retain((p) => exists(p)); return [...reported]; }
    /// ```
    fn recorded_failures(&self) -> Vec<PathBuf> {
        // What:     `let Ok(mut reported) = self.reported_unplayable.lock() else { return Vec::new(); };` takes the lock that
        //           guards the set. The set sits in a `Mutex`, which allows one thread at a time; `.lock()` answers a
        //           `Result`: `Ok(guard)` when the lock was taken and `Err(_)` when another thread panicked while holding it.
        //           `let Ok(mut reported) = ... else` binds the guard (changeable, because of `mut`) on success and runs the
        //           `else` block on failure. `Vec::new()` is a constructor that makes an empty list.
        // Why:      The sweep threads add to the set, so the controller must lock it to read and prune it. On a poisoned lock
        //           the safe answer is "no failures to retry".
        // Gotcha:   The lock is released when `reported` goes out of scope, at the end of the method, not by an explicit unlock.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const reported = this.reportedUnplayable; // JS has one thread, so no lock is needed
        // ```
        let Ok(mut reported) = self.reported_unplayable.lock() else {
            // What:     `return Vec::new();` leaves with an empty list. `Vec::new()` makes an empty growable list and allocates
            //           nothing until the first push.
            // Why:      No usable set means nothing to retry.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return [];
            // ```
            return Vec::new();
        };
        // What:     `reported.retain(|path| return !matches!(std::fs::symlink_metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound));`
        //           removes every path for which the closure answers false. `|path| ...` is a closure whose parameter is a borrow of
        //           each path. `matches!` is a macro that answers true when a value fits a pattern: here the file-system query
        //           `symlink_metadata(path)` (an `lstat`, which does not follow a link) answered `Err(error)` and the `if` guard
        //           says that the error kind is "not found". The leading `!` negates, so a path stays unless it is gone.
        // Why:      A removed file is forgotten here so the set cannot grow with deleted files.
        // Gotcha:   `retain` keeps the items for which the closure is true, the reverse of a "remove if" test.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const path of [...reported]) if (!existsLstat(path)) reported.delete(path);
        // ```
        reported.retain(|path| return !matches!(std::fs::symlink_metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound));
        // What:     `return reported.iter().cloned().collect();` copies the set into a list. `.iter()` walks by borrow, `.cloned()`
        //           copies each path, and `.collect()` gathers them into the return type `Vec<PathBuf>`.
        // Why:      The caller wants an owned list, and the lock must be released when the method returns.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [...reported];
        // ```
        return reported.iter().cloned().collect();
    }

    /// What: `fn settle_selection(&mut self, root: &Path, selected_path: Option<PathBuf>)` runs after the list changed: it keeps
    /// the selected track by its path, or clears the selection when its file left. `root: &Path` is a borrowed read-only
    /// path; `selected_path: Option<PathBuf>` is an owned optional path taken by value.
    /// Why: audio is owned by the decoder, so a live change to other files must not interrupt playback or move the
    /// person's page; a selected file that left stops playback and says so (PB2, D9, D83).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private settleSelection(root: string, selectedPath: string | null): void { /* find it again, or clear and say it left */ }
    /// ```
    fn settle_selection(&mut self, root: &Path, selected_path: Option<PathBuf>) {
        // What:     `let found: Option<usize> = selected_path.as_ref().and_then(|path| { ... });` looks for the selected track in the
        //           new queue. `.as_ref()` borrows the inside of the `Option`; `.and_then(f)` runs the closure on the path when
        //           there is one and answers the closure's own `Option<usize>` (an index, or `None`), and answers `None` without
        //           calling it when there is no selected path.
        // Why:      The selected track is found again by its path, never by its old position, which an edit may have moved.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const found: number | null = selectedPath == null ? null : findIndex(selectedPath);
        // ```
        let found: Option<usize> = selected_path.as_ref().and_then(|path| {
            // What:     `return match locate(self.queue.tracks(), root, path) { Ok(index) => Some(index), Err(_) => ... };` first tries
            //           the binary search. `locate` answers `Result<usize, usize>`: `Ok(index)` when found, `Err(place)` when not;
            //           the success arm wraps the index in `Some`, the present case of `Option`, and the failure arm (`_` ignores
            //           the insertion point) falls back to a scan.
            // Why:      The binary search is cheap, but it relies on the queue being in walk order; the scan covers a queue that
            //           is not.
            // Gotcha:   `Err` here is not a failure of the program; it is the "not found" answer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const hit = locate(tracks, root, path); if (hit.found != null) return hit.found;
            // ```
            return match locate(self.queue.tracks(), root, path) {
                Ok(index) => Some(index),
                // What:     `Err(_) => self.queue.tracks().iter().position(|track| return track == path),` scans the queue for the
                //           path. `.iter()` walks by borrow and `.position(f)` answers `Some(index)` of the first item for which
                //           the closure is true, or `None`. `track == path` compares two borrowed paths by content.
                // Why:      The fallback finds the track even when the sorted-order search could not.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return tracks.findIndex((track) => track === path); // -1 becomes null
                // ```
                Err(_) => self.queue.tracks().iter().position(|track| return track == path),
            };
        });
        // What:     `match found { Some(index) => { ... } None => { ... } }` takes the optional index apart. `Some(index)` binds
        //           the index when the track was found, and `None` handles the case where it was not.
        // Why:      The two cases are the two things that can happen to the selected track: it stays, or it left.
        // Gotcha:   `match` must cover every case; leaving out `None` would not compile.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (found != null) { ... } else { ... }
        // ```
        match found {
            Some(index) => {
                // What:     `self.queue.restore_index_preserving_page(index);` points the queue's cursor at the found track while keeping
                //           the page. The method answers an `Option<usize>` that this call discards by ending with `;`.
                // Why:      The cursor was stale after the edit.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.queue.restoreIndexPreservingPage(index);
                // ```
                self.queue.restore_index_preserving_page(index);
                // What:     `self.emit_reconciled();` tells the window that the queue and the playing index now agree.
                // Why:      The window must repaint with the cursor's new place.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emitReconciled();
                // ```
                self.emit_reconciled();
            }
            None => {
                // What:     `let was_playing: bool = self.playing;` copies the playing flag into a local. A `bool` is a `Copy` type,
                //           so later changes to `self.playing` do not change the local.
                // Why:      Playback is stopped below, and the message must say whether audio was playing when the file left.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const wasPlaying: boolean = this.playing;
                // ```
                let was_playing: bool = self.playing;
                // What:     `self.queue.clear_selection();` removes the selection from the queue.
                // Why:      The selected file is gone, so no track is selected.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.queue.clearSelection();
                // ```
                self.queue.clear_selection();
                // What:     `self.emit_reconciled();` sends the reconciled message, as in the other arm.
                // Why:      The window drops the highlight and the row.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emitReconciled();
                // ```
                self.emit_reconciled();
                // What:     `self.emit(Update::Position(0.0));` sends a message to the window. `Update::Position(0.0)` builds the
                //           `Position` case of the `Update` enum around the float `0.0`; `::` joins the enum's name and its case.
                // Why:      The elapsed time resets, because the rate-limited position message could otherwise be dropped (PB2).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // this.emit({ kind: "position", secs: 0 });
                // ```
                self.emit(Update::Position(0.0));
                // What:     `if self.playing { self.set_playing(false); }` stops audio when it was playing.
                // Why:      A selected file that left the folder cannot keep playing from the queue's point of view.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (this.playing) this.setPlaying(false);
                // ```
                if self.playing {
                    self.set_playing(false);
                }
                // What:     `if let Some(path) = selected_path { ... }` is a one-arm match: it runs the block only when the `Option` is
                //           `Some`, naming the path inside. `selected_path` is moved into the pattern.
                // Why:      With no selected path before the edit there is no file to report as gone.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (selectedPath != null) {
                // ```
                if let Some(path) = selected_path {
                    // What:     `tracing::info!(path = %path.display(), was_playing, "...");` writes an info line. `%path.display()` is the
                    //           readable text of the path, and a bare `was_playing` is shorthand for `was_playing = was_playing`.
                    // Why:      The loss of the selected file is a notable event.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // logger.info({ path, wasPlaying }, "the selected track's file left the folder");
                    // ```
                    tracing::info!(path = %path.display(), was_playing, "the selected track's file left the folder");
                    // What:     `self.emit(Update::FileGone { path, was_playing });` sends the `FileGone` case, which carries named fields.
                    //           A bare `path` and `was_playing` are shorthand for `path: path` and `was_playing: was_playing`; `path` is moved
                    //           into the message.
                    // Why:      The window names the missing file in a message, so the person knows why playback stopped.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.emit({ kind: "fileGone", path, wasPlaying });
                    // ```
                    self.emit(Update::FileGone { path, was_playing });
                }
            }
        }
    }

    /// What: `pub(crate) fn request_refresh(&mut self, work: Work)` asks for a refresh sweep: it starts now when none runs, else
    /// it joins the pending follow-up.
    /// Why: overlapping sweeps would stat the same files twice; the coalescer merges what arrives meanwhile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// requestRefresh(work: Work): void { const launch = this.refresh.submit(work); if (launch) this.launchRefresh(launch); }
    /// ```
    pub(crate) fn request_refresh(&mut self, work: Work) {
        // What:     `if let Some(launch) = self.refresh.submit(work) { self.launch_refresh(launch); }` is a one-arm match. `submit` answers
        //           `Option<Launch>`: `Some(launch)` when a sweep must start now, and `None` when the work joined the pending
        //           follow-up. The block runs only for `Some`, and `launch` is moved into `launch_refresh`.
        // Why:      The coalescer decides; the controller starts only what it is told to start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const launch = this.refresh.submit(work); if (launch != null) this.launchRefresh(launch);
        // ```
        if let Some(launch) = self.refresh.submit(work) {
            self.launch_refresh(launch);
        }
    }

    /// What: `pub(crate) fn finish_refresh(&mut self, id: u64)` is called when a refresh sweep ended: it starts the follow-up if
    /// work arrived meanwhile. `u64` is an unsigned 64-bit integer (siblings: `u32` is smaller, `i64` can be negative).
    /// Why: the end is reported through the command channel, so the follow-up starts on the engine thread. The sweep's number
    /// is a `u64` because the coalescer counts with one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finishRefresh(id: number): void { const next = this.refresh.finished(id); if (next) this.launchRefresh(next); }
    /// ```
    pub(crate) fn finish_refresh(&mut self, id: u64) {
        // What:     `tracing::debug!(id, "a refresh sweep ended");` writes a debug line; the bare `id` is shorthand for `id = id`.
        // Why:      The end of each sweep is worth a line when a race between sweeps is read in the log.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug({ id }, "a refresh sweep ended");
        // ```
        tracing::debug!(id, "a refresh sweep ended");
        // What:     `if let Some(next) = self.refresh.finished(id) { self.launch_refresh(next); }` runs the follow-up when the coalescer
        //           hands one back. `finished` answers `Option<Launch>`; `None` means idle, and the block is skipped.
        // Why:      Work that arrived while the sweep ran starts the moment the sweep ends.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const next = this.refresh.finished(id); if (next != null) this.launchRefresh(next);
        // ```
        if let Some(next) = self.refresh.finished(id) {
            self.launch_refresh(next);
        }
    }

    /// What: `fn launch_refresh(&mut self, launch: Launch)` starts the sweep the coalescer asked for. `launch` is taken by value.
    /// Why: a listed sweep carries only the changed files; "everything" carries the queue. A sweep with nothing to look at ends
    /// at once so the follow-up is not held back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private launchRefresh(launch: Launch): void { /* loop: build the job, spawn the sweep, or end an empty one at once */ }
    /// ```
    fn launch_refresh(&mut self, launch: Launch) {
        // What:     `let mut next: Option<Launch> = Some(launch);` starts a loop variable. `Some(launch)` wraps the launch in the
        //           present case of `Option`, and `mut` makes the variable changeable.
        // Why:      An empty sweep ends at once and the coalescer may hand back another, so the method loops until it has none.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let next: Launch | null = launch;
        // ```
        let mut next: Option<Launch> = Some(launch);
        // What:     `while let Some(current) = next.take() { ... }` repeats while the variable holds a launch. `.take()` empties
        //           `next` (leaving `None`) and answers the old content, so each pass owns its launch and the next pass starts
        //           with nothing unless the body stores one.
        // Why:      The loop is the only place that starts sweeps, so an empty one cannot leave the follow-up waiting.
        // Gotcha:   `while let` stops as soon as the pattern fails to match, here when `take()` answers `None`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (next != null) { const current = next; next = null; ...
        // ```
        while let Some(current) = next.take() {
            // What:     `let selected: Option<PathBuf> = self.queue.current_path().cloned();` copies the selected path.
            // Why:      The sweep leaves the selected track to the current-track worker.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const selected: string | null = this.queue.currentPath();
            // ```
            let selected: Option<PathBuf> = self.queue.current_path().cloned();
            // What:     `let (tracks, listed, announce): (Vec<PathBuf>, bool, Vec<PathBuf>) = match current.work { ... };` splits the
            //           work into three values at once. The left side is a tuple pattern that names three variables, and the
            //           annotation spells their types: a list of paths, a flag, and a list of paths. The `match` answers the
            //           matching triple, one arm per case of `Work`. `current.work` is moved into the `match`.
            // Why:      The sweep job needs the tracks to look at, whether they are a listed subset, and which to announce.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const [tracks, listed, announce]: [string[], boolean, string[]] = current.work.kind === "everything" ? [...] : [...];
            // ```
            let (tracks, listed, announce): (Vec<PathBuf>, bool, Vec<PathBuf>) = match current.work {
                // What:     `Work::Everything => (self.queue.tracks().to_vec(), false, Vec::new()),` is the arm for "every track".
                //           `.to_vec()` copies the borrowed slice into an owned `Vec`, `false` says that the list is not a subset,
                //           and `Vec::new()` is an empty list of paths to announce.
                // Why:      The whole queue is the list, and a sweep over everything announces nothing by peek.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return [[...this.queue.tracks()], false, []];
                // ```
                Work::Everything => (self.queue.tracks().to_vec(), false, Vec::new()),
                // What:     `Work::Files(files) => { ... }` is the arm for a listed set; it names the `Files` record `files`.
                // Why:      Only the changed files are looked at.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // // files = current.work.files
                // ```
                Work::Files(files) => {
                    // What:     `let mut listed_files: Vec<PathBuf> = files.check.into_iter().collect();` turns the set of files to check
                    //           into a list. `.into_iter()` consumes the set, and `.collect()` gathers into the annotated `Vec`.
                    // Why:      A sorted list lets the sweep claim tracks in a stable order.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const listedFiles: string[] = [...files.check];
                    // ```
                    let mut listed_files: Vec<PathBuf> = files.check.into_iter().collect();
                    // What:     `listed_files.sort();` sorts the list in place by the natural order of paths.
                    // Why:      A set has no order, and a repeatable order makes the sweep and its log reproducible.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // listedFiles.sort();
                    // ```
                    listed_files.sort();
                    // What:     `let mut announced: Vec<PathBuf> = files.announce.into_iter().filter(|path| return Some(path) != selected.as_ref()).collect();`
                    //           turns the announce set into a list without the selected track. `.filter(f)` keeps items for which the
                    //           closure is true; `|path| ...` is a closure whose parameter is a borrow of each path. The condition
                    //           compares `Some(path)` with `selected.as_ref()`, two `Option<&PathBuf>`, with `!=`.
                    // Why:      The selected track is handled by the current-track worker and must not be announced twice.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const announced: string[] = [...files.announce].filter((path) => path !== selected);
                    // ```
                    let mut announced: Vec<PathBuf> = files.announce.into_iter().filter(|path| return Some(path) != selected.as_ref()).collect();
                    // What:     `announced.sort();` sorts the announced list in place.
                    // Why:      Same reason as for the listed files: a repeatable order.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // announced.sort();
                    // ```
                    announced.sort();
                    // What:     `(listed_files, true, announced)` with no `;` is the value of this arm: a tuple of the sorted list, the
                    //           flag `true` (this list is a subset), and the announce list. A block's last expression is its value.
                    // Why:      It is the triple the `let` on the `match` is waiting for.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // return [listedFiles, true, announced];
                    // ```
                    (listed_files, true, announced)
                }
            };
            // What:     `if tracks.is_empty() { next = self.refresh.finished(current.id); continue; }` handles a sweep with nothing to look at.
            //           `.is_empty()` is true for an empty list; the body ends the sweep in the coalescer at once, stores whatever it
            //           hands back in `next`, and `continue` jumps to the next pass of the loop.
            // Why:      An empty sweep must not hold the follow-up back, and the loop then starts the follow-up if there is one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (tracks.length === 0) { next = this.refresh.finished(current.id); continue; }
            // ```
            if tracks.is_empty() {
                next = self.refresh.finished(current.id);
                continue;
            }
            // What:     `tracing::debug!(id = current.id, tracks = tracks.len(), listed, generation = self.analysis_generation, "...");` writes a
            //           debug line with four fields; the bare `listed` is shorthand for `listed = listed`.
            // Why:      One line records what each sweep covers.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug({ id: current.id, tracks: tracks.length, listed, generation: this.analysisGeneration }, "refresh sweep started");
            // ```
            tracing::debug!(id = current.id, tracks = tracks.len(), listed, generation = self.analysis_generation, "refresh sweep started");
            // What:     `spawn_queue_measurement(SweepJob { ... });` starts the background sweep. `SweepJob { ... }` builds a record with
            //           every field named: a bare name (`tracks`, `listed`, `announce`) is shorthand for `name: name`; `current: selected`
            //           names a field and a different variable; `cache: self.peaks.clone()` copies the handle to the cache (a cheap copy of
            //           a handle, not of the data); `sink: Arc::clone(&self.sweep_sink)` makes a second owner of the shared report sink;
            //           `gate: SweepGate::shared()` fetches the shared pause gate; `quiet: true` makes it a refresh sweep;
            //           `refresh_id: Some(current.id)` wraps the number in `Some`, the present case of `Option`.
            // Why:      The sweep thread needs its own handles to the cache, the sink, the gate and the claim set, because it outlives
            //           this call.
            // Gotcha:   `Arc::clone(&x)` copies the pointer and bumps a count; it does not copy what the pointer points at, unlike a
            //           `.clone()` on a plain value. It is spelled `Arc::clone(&x)` instead of `x.clone()` so that a reader sees at
            //           once that the copy is cheap.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // spawnQueueMeasurement({ tracks, current: selected, cache: this.peaks, sink: this.sweepSink, generation: this.analysisGeneration,
            //   gate: SweepGate.shared(), claims: this.sweepClaims, quiet: true, listed, announce, refreshId: current.id });
            // ```
            spawn_queue_measurement(SweepJob {
                tracks,
                current: selected,
                cache: self.peaks.clone(),
                sink: Arc::clone(&self.sweep_sink),
                generation: self.analysis_generation,
                gate: SweepGate::shared(),
                claims: Arc::clone(&self.sweep_claims),
                quiet: true,
                listed,
                announce,
                refresh_id: Some(current.id),
            });
        }
    }
}
