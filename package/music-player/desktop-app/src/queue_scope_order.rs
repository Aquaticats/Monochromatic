//! The scope and the play order of the queue:
//! which tracks a playback mode walks and in what order.
//!
//! What:
//!  `impl Queue` methods that compute the scope of an anchor track,
//!  draw the next shuffle pick without replacement,
//!  and rebuild the order of the scope when the mode,
//!  the page or the tracks change.
//! Why:
//!  They are the part of the queue that the shuffle history rests on,
//!  and they are split off the cursor and transport methods in `queue.rs` so that neither file grows past the line limit.
//!  The module is a child of `queue`,
//! so it reaches the private fields of `Queue` without widening them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Queue { scopeIndices(anchor) {} pickNextShuffle(current) {} rebuildScopeOrder(anchor) {} }
//! ```

/// Imports everything the queue module declares,
/// whose `Queue` this module extends.
use super::*;

/// What:
///      `impl Queue { ... }`.
///  The scope and order half of the queue (a SECOND inherent `impl` block for `Queue`,
///  whose first half is in `queue.rs`).
/// Why:
///       Keep the shuffle history apart from the cursor methods without one huge file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Queue { /* scope_indices, pick_next_shuffle, rebuild_scope_order */ }
/// ```
impl Queue {
    /// What:
    ///      `fn scope_indices(&self, anchor: usize) -> Vec<usize>` returns the
    ///           load-order indices that make up the playback scope around the `anchor`
    ///           track,
    ///  in ascending load order.
    ///  Private helper.
    /// Why:
    ///       `All` scopes the whole queue;
    ///  `Off`/`WithinPage` scope the anchor's page
    ///           (its top-level folder / letter bucket),
    ///  so playback stays inside one folder
    ///           unless shuffling everything.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private scopeIndices(anchor: number): number[] {
    ///   if (this.shuffle === "all") return [...Array(this.tracks.length).keys()];
    ///   const pages = paginate(this.displayPaths());
    ///   const p = pageOfIndex(pages, anchor);
    ///   return p === null
    ///     ? [...Array(this.tracks.length).keys()]
    ///     : pages[p].entries.map(e => e.index);
    /// }
    /// ```
    pub(super) fn scope_indices(&self, anchor: usize) -> Vec<usize> {
        // What:     `if self.mode == PlaybackMode::ShuffleAll { ... }`. `==` compares the `Copy`
        //           enum (it derives `PartialEq`).
        // Why:      `All` ignores pages: the scope is every track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle === "all") return [...Array(this.tracks.length).keys()];
        // ```
        if self.mode == PlaybackMode::ShuffleAll {
            // What:     `(0..self.tracks.len()).collect()`. `(a..b)` is a half-open RANGE;
            //           `.collect()` gathers it into a `Vec<usize>` (the return type fixes
            //           the element type). `return` makes this the function's value.
            // Why:      Every load-order index, ascending.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return [...Array(this.tracks.length).keys()];
            // ```
            return (0..self.tracks.len()).collect();
        }
        if !self.page_scope.is_empty() {
            return self.page_scope.clone();
        }
        // What:     the positions on the anchor's page come from `relpath::page_scope_for_anchor`, which compares page
        //           keys track by track instead of grouping every display name into pages.
        // Why:      the playback scope and the visible page still agree (same grouping rules, tested against
        //           `paginate`), without the allocation per name that cost over a second at one million tracks.
        match crate::relpath::page_scope_for_anchor_in(&self.tracks, anchor, self.display_root.as_deref()) {
            Some(indices) => return indices,
            None => return (0..self.tracks.len()).collect(),
        }
    }

    /// What:
    ///      `fn pick_next_shuffle(&mut self, current: usize) -> usize`.
    ///  Choose the next
    ///           shuffle track JUST IN TIME and WITHOUT REPLACEMENT:
    ///  a uniformly random scope
    ///           track not yet played this cycle;
    ///  when the cycle is exhausted,
    ///  start a fresh one
    ///           (avoiding an immediate repeat of `current` unless the scope has one track).
    /// Why:
    ///       Replaces the precomputed shuffled permutation with one pick at a time,
    ///  so live
    ///           queue changes need no order bookkeeping.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private pickNextShuffle(current: number): number {
    ///   const scope = this.scopeIndices(current);
    ///   const played = new Set(this.order.slice(this.cycleStart));
    ///   let remaining = scope.filter((i) => !played.has(i));
    ///   if (remaining.length === 0) {
    ///     this.cycleStart = this.order.length;
    ///     remaining = scope.filter((i) => i !== current);
    ///     if (remaining.length === 0) remaining = scope;
    ///   }
    ///   return remaining[Number(this.nextRand() % BigInt(remaining.length))];
    /// }
    /// ```
    pub(super) fn pick_next_shuffle(&mut self, current: usize) -> usize {
        // What:     `let scope = self.scope_indices(current);`. The scope's load-order indices
        //           (the page for `WithinPage`, the whole queue for `All`).
        // Why:      The pool to pick from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const scope = this.scopeIndices(current);
        // ```
        let scope = self.scope_indices(current);
        // What:     `let played: HashSet<usize> = self.order[self.cycle_start..].iter().copied().collect();`.
        //           The tracks played since the current cycle began. `.copied()` turns
        //           `&usize` into `usize`; `.collect()` builds an owned set (so it does not
        //           keep borrowing `self.order`).
        // Why:      Excluded from this cycle's remaining picks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const played = new Set(this.order.slice(this.cycleStart));
        // ```
        let played: HashSet<usize> = self.order[self.cycle_start..].iter().copied().collect();
        // What:     `let mut remaining: Vec<usize> = scope.iter().copied().filter(|i| !played.contains(i)).collect();`.
        //           Scope tracks not yet played this cycle.
        // Why:      The candidates for the next pick.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let remaining = scope.filter((i) => !played.has(i));
        // ```
        let mut remaining: Vec<usize> = Vec::new();
        for index in scope.iter().copied() {
            if !played.contains(&index) {
                remaining.push(index);
            }
        }
        // What:     `if remaining.is_empty() { ... }`. Cycle exhausted: begin a new one.
        // Why:      Without replacement means a full cycle covers the whole scope; then reshuffle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!remaining.length) { ... }
        // ```
        if remaining.is_empty() {
            // What:     `self.cycle_start = self.order.len();`. The new cycle starts after the
            //           history written so far.
            // Why:      Subsequent picks measure "played this cycle" from here.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.cycleStart = this.order.length;
            // ```
            self.cycle_start = self.order.len();
            // What:     `remaining = scope.iter().copied().filter(|&i| i != current).collect();`.
            //           All scope tracks except the one that just finished.
            // Why:      A fresh cycle should not immediately replay the current track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // remaining = scope.filter((i) => i !== current);
            // ```
            remaining = Vec::new();
            for index in scope.iter().copied() {
                if index != current {
                    remaining.push(index);
                }
            }
            // What:     `if remaining.is_empty() { remaining = scope; }`. A single-track scope
            //           has nothing else; replay it.
            // Why:      Avoid an empty candidate list when the scope is one track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!remaining.length) remaining = scope;
            // ```
            if remaining.is_empty() {
                remaining = scope;
            }
        }
        // What:     `let j = (self.next_rand() % remaining.len() as u64) as usize;`. A uniform
        //           index into `remaining`.
        // Why:      Random choice among the candidates.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const j = Number(this.nextRand() % BigInt(remaining.length));
        // ```
        let j = (self.next_rand() % remaining.len() as u64) as usize;
        // What:     `remaining[j]`. The chosen load-order index. Tail -> return.
        // Why:      Hand back the next track to play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return remaining[j];
        // ```
        return remaining[j]
    }

    /// What:
    ///      `fn rebuild_scope_order(&mut self, anchor: Option<usize>)`.
    ///  Recompute the
    ///           scope `order` (and the cursor `pos`) so that the `anchor` track stays
    ///           current.
    ///  Private helper used whenever the scope might change (set_tracks,
    ///           set_playback_mode,
    ///  play_index to another page).
    /// Why:
    ///       Centralise the "what plays next,
    ///  in what order" rebuild.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private rebuildScopeOrder(anchor: number | null): void {
    ///   if (this.tracks.length === 0) { this.order = []; this.pos = null; return; }
    ///   const a = Math.min(anchor ?? 0, this.tracks.length - 1);
    ///   let scope = this.scopeIndices(a);
    ///   if (this.shuffle !== "off") this.shuffleSlice(scope);
    ///   const p = scope.indexOf(a);
    ///   this.order = scope;
    ///   this.pos = p < 0 ? 0 : p;
    /// }
    /// ```
    pub(super) fn rebuild_scope_order(&mut self, anchor: Option<usize>) {
        // What:     `if self.tracks.is_empty() { self.order = Vec::new(); self.pos = None; return; }`.
        //           Empty queue: no order, no cursor.
        // Why:      Nothing to play; guard the index math below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.tracks.length) { this.order = []; this.pos = null; return; }
        // ```
        if self.tracks.is_empty() {
            self.order = Vec::new();
            self.pos = None;
            self.sequential_start = 0;
            return;
        }
        // What:     `let anchor_index = match anchor { Some(a) => a, None => { ...; return; } };`.
        //           Turn the `Option<usize>` parameter into a plain `usize`, but treat a
        //           `None` anchor as "NO current track": clear the order and cursor and RETURN
        //           early (handled in the `None` arm below). `Some(a) => a` keeps a real index.
        //           (The unwrapped index gets its own name; it never shadows the parameter.)
        // Why:      `set_tracks` anchors `Some(0)`, but `clear_selection` (and toggling shuffle
        //           while nothing is selected) passes `None` to DESELECT, so a freshly opened
        //           library highlights and loads nothing until the user picks a track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (anchor === null) { this.order = []; this.pos = null; return; }
        // const a0 = anchor;
        // ```
        let anchor_index = match anchor {
            // What:     `Some(a) => a`. Unwrap a present anchor to its `usize` index.
            // Why:      We have a real track to centre the scope on.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const a0 = anchor;
            // ```
            Some(a) => a,
            // What:     `None => { self.order = Vec::new(); self.pos = None; return; }`. No
            //           anchor: `Vec::new()` builds a fresh empty owned vector for `order`,
            //           `self.pos = None` nulls the cursor, and `return` exits the function.
            // Why:      Express "nothing selected" for a loaded (non-empty) queue; a later
            //           `play_index` rebuilds a real scope when the user taps a track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order = []; this.pos = null; return;
            // ```
            None => {
                self.order = Vec::new();
                self.pos = None;
                self.sequential_start = 0;
                return;
            }
        };
        // What:     `let clamped_anchor = anchor_index.min(self.tracks.len() - 1);`. Clamp the
        //           anchor into range. `.min(x)` returns the smaller of the two. (The clamped
        //           value gets its own name too.)
        // Why:      Defensive: a stale index must not point past the tracks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const a = Math.min(a0, this.tracks.length - 1);
        // ```
        let clamped_anchor = anchor_index.min(self.tracks.len() - 1);
        // What:     `if self.mode == PlaybackMode::InOrder || self.mode == PlaybackMode::Repeat { ... } else { ... }`. Off builds the
        //           full sequential scope order; the shuffle modes start a fresh play history.
        // Why:      Off is a deterministic in-order walk of the scope, while shuffle picks just
        //           in time, so its `order` begins as only the anchor and grows on `advance`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle === "off") { /* full sequential scope */ } else { /* [anchor] */ }
        // ```
        if self.mode == PlaybackMode::InOrder || self.mode == PlaybackMode::Repeat {
            // What:     `let scope = self.scope_indices(clamped_anchor);`. The scope's indices
            //           in ascending load order.
            // Why:      Off plays the scope in load order.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const scope = this.scopeIndices(anchor);
            // ```
            let scope = self.scope_indices(clamped_anchor);
            // What:     `let pos = scope.iter().position(|&x| x == clamped_anchor);`. The
            //           anchor's slot in the scope.
            // Why:      The cursor must point at the anchor after the rebuild.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const p = scope.indexOf(anchor);
            // ```
            let mut pos: Option<usize> = None;
            for (at, member) in scope.iter().enumerate() {
                if *member == clamped_anchor {
                    pos = Some(at);
                    break;
                }
            }
            // What:     `self.order = scope; self.pos = pos.or(Some(0));`. Adopt the sequential
            //           order and point the cursor at the anchor (or the start).
            // Why:      Off's `order` is the full scope, walked sequentially with looping.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order = scope; this.pos = p < 0 ? 0 : p;
            // ```
            if let Some(position) = pos {
                self.order = scope;
                self.pos = Some(position);
                self.sequential_start = 0;
            } else {
                self.order = vec![clamped_anchor];
                self.order.extend(scope);
                self.pos = Some(0);
                self.sequential_start = 1;
            }
        } else {
            // What:     `self.order = vec![clamped_anchor]; self.pos = Some(0); self.cycle_start = 0;`.
            //           Begin the play history with just the anchor and open a fresh cycle.
            // Why:      Shuffle does not precompute a permutation; `advance` appends each
            //           just-in-time pick to `order`, and the anchor is the first track played.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order = [anchor]; this.pos = 0; this.cycleStart = 0;
            // ```
            self.order = vec![clamped_anchor];
            self.pos = Some(0);
            self.cycle_start = 0;
            self.sequential_start = 0;
        }
    }

}
