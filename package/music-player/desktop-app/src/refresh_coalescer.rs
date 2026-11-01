//! At most one refresh sweep runs, and the requests that arrive meanwhile become one follow-up.
//!
//! What: `RefreshCoalescer` is a small state machine over "idle or running one numbered sweep, with work pending or not";
//! `Work` is what a refresh has to look at, either some files or the whole queue.
//! Why: D113 starts a refresh for every settled burst of watcher events. Sweeps that overlap would stat the same files
//! twice and contend for the idle-priority workers, so a request that arrives while a sweep runs joins one pending
//! follow-up that covers the union of everything asked meanwhile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class RefreshCoalescer { running: number | null; pending: Work | null; submit(w: Work): Launch | null; finished(id: number): Launch | null; }
//! ```

/// What: `use std::collections::HashSet;` imports the hash set type from the standard library (`std`). Siblings: `Vec`
/// keeps duplicates and order, and `BTreeSet` keeps its members sorted.
/// Why: a file named twice must be looked at once, and membership is asked often, so a hash set fits; order does not matter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Set is global in TypeScript; nothing to import.
/// ```
use std::collections::HashSet;
/// What: `use std::path::PathBuf;` imports the owned, growable path type. Siblings: `Path` is the borrowed read-only view,
/// and `String` is the text-only owned type, which cannot hold file names that are not valid UTF-8.
/// Why: the sets own their paths, so they hold the owned type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Paths are plain strings in TypeScript.
/// ```
use std::path::PathBuf;

/// What: `use crate::change_set::MAX_DIRTY_PATHS;` imports the cap constant from `src/change_set.rs`. `crate::` means
/// "from the root of this crate".
/// Why: a pending file set uses the same cap as a change set, so the two agree on when a set becomes "everything".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MAX_DIRTY_PATHS } from "./change_set";
/// ```
use crate::change_set::MAX_DIRTY_PATHS;

/// What: `pub(crate) struct Files { check, announce }` is a record of two sets of paths. `pub(crate)` makes the type and
/// its fields visible inside this crate only (siblings: plain `pub` reaches other crates; no keyword is private to the
/// file). `#[derive(Clone, Debug, Default, PartialEq, Eq)]` asks the compiler to write five standard abilities: `Clone`
/// (an explicit `.clone()`), `Debug` (printing with `{:?}`), `Default` (an empty value from `Files::default()`), and
/// `PartialEq` and `Eq` (comparing with `==`).
/// Why: a file that was replaced only needs its fingerprint checked and its peak measured; a file that was added also
/// needs its row announced by a peek at its header, so the row has a length at once. Announcing a replaced file would
/// blank its old peak until the decode ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Files = { check: Set<string>; announce: Set<string> };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Files {
    /// What: `pub(crate) check: HashSet<PathBuf>,` is the set of every file whose fingerprint is checked. `HashSet<T>` is an
    /// owned set (siblings: `Vec<T>` allows duplicates, `BTreeSet<T>` is sorted), and `PathBuf` is an owned path.
    /// Why: a set, not a list, because a file named twice is still one file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// check: Set<string>;
    /// ```
    pub(crate) check: HashSet<PathBuf>,
    /// What: `pub(crate) announce: HashSet<PathBuf>,` is the files among them that are new to the list.
    /// Why: only new rows need their length announced.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// announce: Set<string>;
    /// ```
    pub(crate) announce: HashSet<PathBuf>,
}

/// What: `pub(crate) enum Work { Files(Files), Everything }` is a tagged union of two cases: `Files` carries a `Files`
/// record and `Everything` carries nothing. `#[derive(Clone, Debug, PartialEq, Eq)]` writes the copy, print and `==`
/// abilities (there is no `Default`, because the enum has no obvious first case).
/// Why: a refresh after a few events looks at a few files; one after lost events looks at the whole queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Work = { kind: "files"; files: Files } | { kind: "everything" };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Work {
    /// What: `Files(Files),` is the case that carries the files to look at (a tuple variant: the data sits in the case).
    /// Why: a small burst names a small set of files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "files"; files: Files }
    /// ```
    Files(Files),
    /// What: `Everything,` is the case that carries no data.
    /// Why: lost events or a newly opened folder need every track looked at.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "everything" }
    /// ```
    Everything,
}

/// What: `impl Work { ... }` opens the block that holds the methods of `Work`.
/// Why: the emptiness test, the union and the cap belong to the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Work { /* the methods of the impl block */ }
/// ```
impl Work {
    /// What: `pub(crate) fn is_empty(&self) -> bool` is true for a file set with no file. `&self` is a read-only borrow of
    /// the value the method is called on.
    /// Why: nothing to look at means no sweep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isEmpty(): boolean { return this.kind === "files" && this.files.check.size === 0; }
    /// ```
    pub(crate) fn is_empty(&self) -> bool {
        // What:     `return matches!(self, Work::Files(files) if files.check.is_empty());` asks whether the value fits a pattern.
        //           `matches!` is a macro (the `!` marks code that expands at compile time) that answers true or false;
        //           `Work::Files(files)` matches the `Files` case and names the record inside it, and the `if` after the
        //           pattern is a guard: the match counts only when that condition also holds.
        // Why:      Only a file set with no file is empty; "everything" is never empty.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.kind === "files" && this.files.check.size === 0;
        // ```
        return matches!(self, Work::Files(files) if files.check.is_empty());
    }

    /// What: `fn merge(self, other: Work) -> Work` is the union: "everything" absorbs files, and a file set past the cap
    /// becomes "everything". There is no `pub`, so it is private to this file. `self` and `other` are taken by value, so
    /// the call uses both up and answers a new value.
    /// Why: the pending work of a long sweep must stay bounded however many events arrive.
    /// Gotcha: after `a.merge(b)` neither `a` nor `b` can be used again; in TypeScript both would still exist.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private merge(other: Work): Work { /* either everything -> everything; else union of both sets, then capped */ }
    /// ```
    fn merge(self, other: Work) -> Work {
        // What:     `return match (self, other) { ... };` matches the pair of both values at once, as a tuple. The first arm,
        //           `(Work::Everything, _) | (_, Work::Everything)`, matches when either side is `Everything` (`|` is "or"
        //           between patterns, `_` ignores that side). The second arm names the two file sets `left` (changeable,
        //           from `mut`) and `right`.
        // Why:      "Everything" swallows any files; two file sets are united.
        // Gotcha:   `match` must cover every pair; these two arms do.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (self.kind === "everything" || other.kind === "everything") return { kind: "everything" };
        // ```
        return match (self, other) {
            (Work::Everything, _) | (_, Work::Everything) => Work::Everything,
            (Work::Files(mut left), Work::Files(right)) => {
                // What:     `left.check.extend(right.check);` adds every path of `right.check` into `left.check`. `.extend`
                //           takes the other set by value, so `right.check` is moved out of `right`.
                // Why:      The union of the two sets of files to check.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // for (const p of right.check) left.check.add(p);
                // ```
                left.check.extend(right.check);
                // What:     `left.announce.extend(right.announce);` is the same union for the files whose rows are new.
                // Why:      A file that was new in either request stays new.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // for (const p of right.announce) left.announce.add(p);
                // ```
                left.announce.extend(right.announce);
                // What:     `Work::Files(left).capped()` with no `;` is the value of this arm. `Work::Files(left)` builds the
                //           `Files` case around the united record (moving `left` into it), and `.capped()` applies the cap.
                // Why:      The union may have crossed the cap, and the pending work must stay bounded.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return capped({ kind: "files", files: left });
                // ```
                Work::Files(left).capped()
            }
        };
    }

    /// What: `fn capped(self) -> Work` turns a file set past `MAX_DIRTY_PATHS` into "everything" and leaves any other value
    /// as it was. `self` is taken by value, so the value is used up and returned or replaced.
    /// Why: looking at that many named files costs as much as the whole queue, and the set must stay bounded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private capped(): Work { return this.kind === "files" && this.files.check.size > MAX_DIRTY_PATHS ? { kind: "everything" } : this; }
    /// ```
    fn capped(self) -> Work {
        // What:     `return match self { Work::Files(files) if files.check.len() > MAX_DIRTY_PATHS => Work::Everything, other => other };`
        //           tests the value against two arms. The first matches a file set that is over the cap (the `if` is a
        //           guard that must also hold); the second, `other => other`, matches anything left and gives the value
        //           back unchanged, because a bare name in a pattern accepts every value and binds it.
        // Why:      Over the cap the cheaper and safer meaning is "everything".
        // Gotcha:   The guard arm moves `files` out of `self` only when it matches; the second arm receives the intact value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.kind === "files" && this.files.check.size > MAX_DIRTY_PATHS ? { kind: "everything" } : this;
        // ```
        return match self {
            Work::Files(files) if files.check.len() > MAX_DIRTY_PATHS => Work::Everything,
            other => other,
        };
    }
}

/// What: `pub(crate) struct Launch { id, work }` is a record that tells the caller a sweep to start now. `#[derive(Debug,
/// PartialEq, Eq)]` writes printing and `==` (there is no `Clone`, so a `Launch` is handed over, never copied).
/// Why: the coalescer decides; the controller starts the sweep and passes the number along so its end can be matched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Launch = { id: number; work: Work };
/// ```
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Launch {
    /// What: `pub(crate) id: u64,` is the number of the sweep. `u64` is an unsigned 64-bit integer (siblings: `u32` is
    /// smaller, `i64` can be negative, `usize` follows the pointer width).
    /// Why: unique for the life of the coalescer; a 64-bit counter cannot wrap in any real run, where `u32` could in theory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// id: number;
    /// ```
    pub(crate) id: u64,
    /// What: `pub(crate) work: Work,` is what the sweep looks at.
    /// Why: the controller needs the work together with the number to start the sweep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// work: Work;
    /// ```
    pub(crate) work: Work,
}

/// What: `pub(crate) struct RefreshCoalescer { running, pending, next_id }` is the state of the coalescer: idle, or running
/// one sweep with at most one follow-up pending. `#[derive(Debug, Default)]` writes printing and a ready-made starting value
/// from `RefreshCoalescer::default()`: `None`, `None` and `0`.
/// Why: see the module description. It is touched only by the engine thread, so it needs no lock: a request that arrives
/// after a sweep ended and before its end was handled joins the pending work, and the end then starts it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefreshCoalescer { private running: number | null = null; private pending: Work | null = null; private nextId = 0; }
/// ```
#[derive(Debug, Default)]
pub(crate) struct RefreshCoalescer {
    /// What: `running: Option<u64>,` is the number of the sweep that runs, if any. `Option<T>` is Rust's `T | null`, with
    /// the cases `Some(value)` and `None`.
    /// Why: `None` means idle, and the number lets the end of a stale sweep be told apart from the end of the current one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private running: number | null;
    /// ```
    running: Option<u64>,
    /// What: `pending: Option<Work>,` is what was asked while it ran, or `None` when nothing was.
    /// Why: at most one follow-up is kept, and it is the union of all the requests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private pending: Work | null;
    /// ```
    pending: Option<Work>,
    /// What: `next_id: u64,` is the number the next sweep gets.
    /// Why: every sweep needs a fresh number, and a counter that only rises provides it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private nextId: number;
    /// ```
    next_id: u64,
}

/// What: `impl RefreshCoalescer { ... }` opens the block that holds the methods of `RefreshCoalescer`.
/// Why: the three transitions (submit, finished, reset) and their helper live with the state they change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefreshCoalescer { /* the methods of the impl block */ }
/// ```
impl RefreshCoalescer {
    /// What: `fn start(&mut self, work: Work) -> Launch` marks a new sweep as running and answers the launch. `&mut self` is an
    /// exclusive borrow: the method may change the coalescer and no one else may touch it meanwhile.
    /// Why: both `submit` and `finished` start sweeps the same way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private start(work: Work): Launch { this.nextId += 1; this.running = this.nextId; return { id: this.nextId, work }; }
    /// ```
    fn start(&mut self, work: Work) -> Launch {
        // What:     `self.next_id += 1;` adds one to the counter. `+=` is the same compound assignment as in TypeScript.
        // Why:      The new sweep needs a number nobody used before.
        // Gotcha:   Integer overflow panics in a debug build and wraps in a release build, unlike TypeScript's `number`; a
        //           `u64` counter never gets there.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.nextId += 1;
        // ```
        self.next_id += 1;
        // What:     `self.running = Some(self.next_id);` records the running sweep. `Some(x)` builds the present case of
        //           `Option`, the wrapper that stands in for a non-null value.
        // Why:      From now on the coalescer is not idle, and the number identifies the sweep.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.running = this.nextId;
        // ```
        self.running = Some(self.next_id);
        // What:     `return Launch { id: self.next_id, work };` builds a `Launch` record. `id: self.next_id` sets a field by
        //           name; a bare `work` is shorthand for `work: work`, taking the field name from the variable name. `work` is
        //           moved into the record, so the caller of `start` no longer has it.
        // Why:      The caller must start a sweep with this number and this work.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { id: this.nextId, work };
        // ```
        return Launch { id: self.next_id, work };
    }

    /// What: `pub(crate) fn submit(&mut self, requested: Work) -> Option<Launch>` asks for a refresh: it answers a sweep to start
    /// now when idle, else `None`, and the work joins the pending follow-up. The answer type `Option<Launch>` is Rust's
    /// `Launch | null`.
    /// Why: this is the coalescing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// submit(requested: Work): Launch | null { /* start now if idle, else merge into pending and return null */ }
    /// ```
    pub(crate) fn submit(&mut self, requested: Work) -> Option<Launch> {
        // What:     `let work: Work = requested.capped();` applies the cap to the request. `capped` takes its value by move, so
        //           `requested` is used up and `work` is the result.
        // Why:      A request that is itself past the cap is already "everything".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const work: Work = capped(requested);
        // ```
        let work: Work = requested.capped();
        // What:     `if work.is_empty() { return None; }` leaves with `None`, the absent case of `Option`, when no file is named.
        // Why:      Nothing to look at means no sweep and no pending work.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (work.isEmpty()) return null;
        // ```
        if work.is_empty() {
            return None;
        }
        // What:     `if self.running.is_none() { ... }` asks whether the running slot holds nothing. `.is_none()` is the
        //           `Option` test for "is null".
        // Why:      When idle, the request starts a sweep at once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.running == null) {
        // ```
        if self.running.is_none() {
            // What:     `tracing::debug!(id = self.next_id + 1, "...");` writes a debug log line with one named field. `tracing::debug!`
            //           is a macro (the `!` marks code that expands at compile time), and `self.next_id + 1` is the number
            //           that `start` is about to give.
            // Why:      The repo logs every branch decision, and this one says why a sweep starts at once.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug({ id: this.nextId + 1 }, "refresh starts at once; none is running");
            // ```
            tracing::debug!(id = self.next_id + 1, "refresh starts at once; none is running");
            // What:     `return Some(self.start(work));` starts the sweep and wraps its launch in `Some`, the present case of
            //           `Option`, so the caller sees "start this one now".
            // Why:      The caller is told which sweep to run.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return this.start(work);
            // ```
            return Some(self.start(work));
        }
        // What:     `tracing::debug!("a refresh is running; ...");` writes a debug log line with no fields.
        // Why:      This branch records that the request was queued instead of started.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug("a refresh is running; the request joins the pending follow-up");
        // ```
        tracing::debug!("a refresh is running; the request joins the pending follow-up");
        // What:     `self.pending = Some(match self.pending.take() { Some(waiting) => waiting.merge(work), None => work });`
        //           replaces the pending slot with the union of what waited and the new work. `.take()` empties the slot and
        //           gives its old content (`Option::take` swaps in `None`), `match` handles the two cases, and `Some(...)`
        //           wraps the result back into the slot. `waiting.merge(work)` consumes both values.
        // Why:      At most one follow-up is kept, and it covers everything asked meanwhile.
        // Gotcha:   `.take()` moves the old value out of a field you only borrow, which plain assignment would not allow.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pending = this.pending ? merge(this.pending, work) : work;
        // ```
        self.pending = Some(match self.pending.take() {
            Some(waiting) => waiting.merge(work),
            None => work,
        });
        // What:     `return None;` answers "nothing to start now", the absent case of `Option`.
        // Why:      The request was queued behind the running sweep.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return null;
        // ```
        return None;
    }

    /// What: `pub(crate) fn finished(&mut self, id: u64) -> Option<Launch>` says that the sweep `id` ended: it answers the
    /// follow-up to start now, or `None` and idle. Another number is ignored.
    /// Why: the end of a sweep from before a reset must not release the sweep that runs now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finished(id: number): Launch | null { /* ignore a stale id; else start the pending follow-up if any */ }
    /// ```
    pub(crate) fn finished(&mut self, id: u64) -> Option<Launch> {
        // What:     `if self.running != Some(id) { ... }` compares the running slot with `Some(id)`, an `Option` built from the
        //           ended sweep's number; two `Option`s compare equal only when both are `Some` of equal numbers or both
        //           `None`. `!=` is TypeScript's `!==`.
        // Why:      Only the end of the sweep that is running counts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.running !== id) {
        // ```
        if self.running != Some(id) {
            // What:     `tracing::debug!(id, running = ?self.running, "...");` writes a debug log line. The bare `id` is
            //           shorthand for `id = id`. The `?` before an expression asks for the developer (`Debug`) text, here of
            //           the `Option`; it is a formatting mark in the macro and has nothing to do with the early-return `?`
            //           operator.
            // Why:      The ignored end is worth a line when a race is being read in the log.
            // Gotcha:   `%` asks for the reader-friendly form and `?` for the developer form; both are only log formatting.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug({ id, running: this.running }, "an end of a sweep that is not the running one is ignored");
            // ```
            tracing::debug!(id, running = ?self.running, "an end of a sweep that is not the running one is ignored");
            // What:     `return None;` answers "nothing to start", the absent case of `Option`.
            // Why:      A stale end releases nothing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return null;
            // ```
            return None;
        }
        // What:     `self.running = None;` empties the running slot; `None` is the absent case of `Option`.
        // Why:      The sweep ended, so the coalescer is idle until it starts the follow-up.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.running = null;
        // ```
        self.running = None;
        // What:     `return self.pending.take().map(|waiting| return self.start(waiting));` takes the pending work out of its slot
        //           (leaving `None`) and, only when there was some, starts it. `.map(f)` on an `Option` runs `f` on the value
        //           inside `Some` and wraps the answer back in `Some`; on `None` it gives `None` without calling `f`.
        //           `|waiting| ...` is a closure (an anonymous function), and it calls `self.start`, which needs `&mut self`.
        // Why:      The follow-up starts the moment the running sweep ends, and an empty slot means the coalescer stays idle.
        // Gotcha:   The `.take()` finishes before `.map` runs, so the closure may borrow `self` mutably again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const waiting = this.pending; this.pending = null;
        // return waiting ? this.start(waiting) : null;
        // ```
        return self.pending.take().map(|waiting| return self.start(waiting));
    }

    /// What: `pub(crate) fn reset(&mut self)` forgets the running sweep and the pending work. It returns nothing (`()`, close
    /// to `void`).
    /// Why: a newly opened folder starts its own sweep, and the paths asked before it belong to the old folder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reset(): void { this.running = null; this.pending = null; }
    /// ```
    pub(crate) fn reset(&mut self) {
        // What:     `tracing::debug!(running = ?self.running, had_pending = self.pending.is_some(), "...");` writes a debug log
        //           line with two fields: the running number in developer form (`?`), and whether work was pending
        //           (`.is_some()` is the `Option` test for "is not null").
        // Why:      A reset throws state away, so the log records what was thrown away.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.debug({ running: this.running, hadPending: this.pending != null }, "the refresh state is reset");
        // ```
        tracing::debug!(running = ?self.running, had_pending = self.pending.is_some(), "the refresh state is reset");
        // What:     `self.running = None;` empties the running slot.
        // Why:      The old sweep, if any, no longer counts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.running = null;
        // ```
        self.running = None;
        // What:     `self.pending = None;` empties the pending slot.
        // Why:      Paths asked before the reset belong to the old folder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pending = null;
        // ```
        self.pending = None;
    }

    /// What: `pub(crate) fn is_idle(&self) -> bool` says whether no sweep runs. The attribute `#[cfg(test)]` compiles the
    /// method only when building tests.
    /// Why: tests wait for a sweep to end before the next run.
    /// Gotcha: this method does not exist in the production build, so non-test code cannot call it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isIdle(): boolean { return this.running == null; }
    /// ```
    #[cfg(test)]
    pub(crate) fn is_idle(&self) -> bool {
        // What:     `return self.running.is_none();` answers true when the running slot holds nothing.
        // Why:      An empty running slot is what "idle" means.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.running == null;
        // ```
        return self.running.is_none();
    }

    /// What: `pub(crate) fn pending(&self) -> Option<&Work>` gives the work waiting for the running sweep to end. The answer
    /// type `Option<&Work>` is an optional read-only borrow: `Some(&work)` lends the stored value, `None` says there is none.
    /// Sibling: `Option<Work>` would hand out an owned copy and empty the slot.
    /// Why: tests read what was merged without disturbing it. Only compiled for tests (`#[cfg(test)]`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pending(): Work | null { return this.pending; }
    /// ```
    #[cfg(test)]
    pub(crate) fn pending(&self) -> Option<&Work> {
        // What:     `return self.pending.as_ref();` turns `&Option<Work>` into `Option<&Work>`. `.as_ref()` looks inside the
        //           `Option` without moving or copying the `Work` it holds.
        // Why:      The test only wants to look; it must not take the work.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.pending;
        // ```
        return self.pending.as_ref();
    }
}

/// What: `#[cfg(test)] #[path = "refresh_coalescer_tests.rs"] mod tests;` declares the module `tests` from the file
/// `refresh_coalescer_tests.rs`, compiled only for `cargo test`.
/// Why: the unit tests live in their own file to keep this one short, and stay out of the production build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: like a `refresh_coalescer.test.ts` file that the production bundle never imports.
/// ```
#[cfg(test)]
#[path = "refresh_coalescer_tests.rs"]
mod tests;
