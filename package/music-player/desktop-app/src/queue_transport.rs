//! The transport of the queue:
//! stepping to the next and the previous track,
//! peeking ahead and rewinding the scope.
//!
//! What:
//!  `impl Queue` methods that move the cursor:
//!  `advance` after a track ended or a manual Next,
//!  `peek_next_natural` to see what a natural end would play,
//!  `prev` for Previous,
//!  and `first_in_scope` and `rewind_scope` for the start of the current scope.
//! Why:
//!  They are the part of the queue that the playback modes act on,
//!  and they are split off the track list and the scope builder so that no file of the queue grows past the line limit.
//!  The module is a child of `queue`,
//! so it reaches the private fields of `Queue` without widening them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Queue { advance(natural) {} peekNextNatural() {} firstInScope() {} rewindScope() {} prev() {} }
//! ```

/// Imports everything the queue module declares,
/// whose `Queue` this module extends.
use super::*;

/// What:
///      `impl Queue { ... }`.
///  The transport half of the queue (a THIRD inherent `impl` block for `Queue`,
///  whose first half is in `queue.rs` and whose second half is in `queue_scope_order.rs`).
/// Why:
///       Keep the cursor movement apart from the track list without one huge file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Queue { /* advance, peek_next_natural, first_in_scope, rewind_scope, prev */ }
/// ```
impl Queue {
    /// What:
    ///      `pub fn advance(&mut self, natural: bool) -> Option<usize>`.
    ///  `natural` is
    ///           true when a track ended on its own,
    ///  false when the user pressed Next.
    /// Why:
    ///       End-of-track and Next share most logic but differ for repeat-track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// advance(natural: boolean): number | null {
    ///   if (this.pos === null) return null;
    ///   const pos = this.pos;
    ///   if (natural && this.repeatTrack) return this.order[pos];
    ///   const next = pos + 1;
    ///   if (next < this.order.length) { this.pos = next; return this.order[next]; }
    ///   if (natural && this.mode === "in_order") return null; // D2: In order stops at the end of the folder
    ///   this.pos = 0; return this.order[0]; // a manual Next loops the scope (page or all)
    /// }
    /// ```
    pub fn advance(&mut self, natural: bool) -> Option<usize> {
        // What:     `let pos = self.pos?;`. The `?` operator on an `Option`: if `self.pos`
        //           is `Some(p)` it unwraps to `p`; if `None` it RETURNS `None` from the
        //           whole function immediately. (This early-return shape is why `advance`
        //           returns `Option<usize>`.)
        // Why:      No cursor means nothing to advance; bail out early.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.pos === null) return null;
        // const pos = this.pos;
        // ```
        let pos = self.pos?;
        // What:     `if natural && self.mode == PlaybackMode::Repeat { ... }`. `&&` is logical AND.
        // Why:      A track that ended under "repeat track" replays itself.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (natural && this.repeatTrack) return this.order[pos];
        // ```
        if natural && self.mode == PlaybackMode::Repeat {
            // What:     `return Some(self.order[pos]);`. Wrap the current track index as the
            //           return value (cursor unchanged).
            // Why:      Signal "play this same track again".
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return this.order[pos];
            // ```
            return Some(self.order[pos]);
        }
        // What:     `if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll { ... }`. The shuffle modes use the
        //           just-in-time path; `Off` falls through to the sequential walk below.
        // Why:      Shuffle has no precomputed order: it retraces history forward, or appends a
        //           fresh random pick when at the history end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle !== "off") { /* retrace or pick */ }
        // ```
        if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll {
            // What:     `if pos + 1 < self.order.len() { ... }`. There is forward history to
            //           retrace (the user pressed `prev` earlier, then `next`).
            // Why:      `next` after `prev` replays the recorded history before drawing anew.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (pos + 1 < this.order.length) { this.pos = pos + 1; return this.order[pos + 1]; }
            // ```
            if pos + 1 < self.order.len() {
                self.pos = Some(pos + 1);
                return Some(self.order[pos + 1]);
            }
            // What:     `let current = self.order[pos];` then `let pick = self.pick_next_shuffle(current);`.
            //           At the history end: choose the next track just in time.
            // Why:      Without replacement within the cycle, reshuffling at cycle end.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const pick = this.pickNextShuffle(this.order[pos]);
            // ```
            let current = self.order[pos];
            let pick = self.pick_next_shuffle(current);
            // What:     `self.order.push(pick); self.pos = Some(self.order.len() - 1);`. Append
            //           the pick to the history and point the cursor at it.
            // Why:      The history grows by one; the new track is current.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order.push(pick); this.pos = this.order.length - 1;
            // ```
            self.order.push(pick);
            self.pos = Some(self.order.len() - 1);
            // What:     `return Some(pick);`. The chosen track.
            // Why:      Hand back what to play next.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return pick;
            // ```
            return Some(pick);
        }
        // What:     `if self.sequential_start > 0 && pos < self.sequential_start { ... }`. The cursor sits on the retained
        //           off-page current track that `rebuild_scope_order` put before the scope (slots before
        //           `sequential_start`), so Next leaves that prelude: `self.order.drain(0..self.sequential_start);`
        //           removes it, `self.sequential_start = 0;` makes the scope start at slot 0, `self.pos = Some(0);`
        //           moves the cursor to the first scope track and `return Some(self.order[0]);` hands it back.
        // Why:      The prelude is the track already playing from outside the scope, so Next moves into the scope and the
        //           prelude is not repeated; after the drain slot 0 is the first track of the displayed page.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.sequentialStart > 0 && pos < this.sequentialStart) {
        //   this.order.splice(0, this.sequentialStart); this.sequentialStart = 0; this.pos = 0; return this.order[0];
        // }
        // ```
        if self.sequential_start > 0 && pos < self.sequential_start {
            self.order.drain(0..self.sequential_start);
            self.sequential_start = 0;
            self.pos = Some(0);
            return Some(self.order[0]);
        }
        // What:     `let next = pos + 1;` compute the following position (Off, sequential).
        // Why:      Try to move forward within the scope.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const next = pos + 1;
        // ```
        let next = pos + 1;
        // What:     `if next < self.order.len() { ... }` bounds check.
        // Why:      There is a track after the current one in this scope.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (next < this.order.length) { this.pos = next; return this.order[next]; }
        // ```
        if next < self.order.len() {
            // What:     `self.pos = Some(next);`. Update the cursor.
            // Why:      Normal forward step.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pos = next;
            // ```
            self.pos = Some(next);
            // What:     `return Some(self.order[next]);`. The new track index.
            // Why:      Hand back what to play next.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return this.order[next];
            // ```
            return Some(self.order[next]);
        }
        // What:     `if natural && self.mode == PlaybackMode::InOrder { return None; }`. A track that ended by itself
        //           at the last slot of the scope has nothing after it; the cursor stays on that track.
        // Why:      D2: "In order = play to the end of the folder and stop". The polish review finding PB4: the scope
        //           looped forever. The caller stops playback and calls `rewind_scope`. A manual Next (`natural` false)
        //           still wraps to the start, because Next must always have a target and `load_current` steps past
        //           unreadable files with it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (natural && this.mode === "in_order") return null;
        // ```
        if natural && self.mode == PlaybackMode::InOrder {
            tracing::info!(slot = pos, "In order reached the end of the scope; nothing follows");
            return None;
        }
        // What:     `self.pos = Some(self.sequential_start);`. Past the end of the scope: wrap to its start. The start
        //           is `self.sequential_start`, the first scope slot, not slot 0.
        // Why:      A manual Next loops the page. When the current track is outside the scope, `rebuild_scope_order`
        //           puts it before the scope (slot 0) and sets `sequential_start` to 1, and the loop must not
        //           return to that track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pos = this.sequentialStart;
        // ```
        self.pos = Some(self.sequential_start);
        // What:     `Some(self.order[self.sequential_start])` tail expression: the wrapped track, the first track of
        //           the scope.
        // Why:      Begin the next loop of the scope.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.order[this.sequentialStart];
        // ```
        return Some(self.order[self.sequential_start])
    }

    /// What:
    ///      `pub fn peek_next_natural(&mut self) -> Option<usize>`.
    ///  The load-order index that `advance(true)` will return next,
    /// without moving the cursor.
    ///  In the shuffle modes,
    /// when the history has no forward entry,
    /// the random pick is appended to the
    ///  history now (the cursor stays),
    /// so the later `advance(true)` steps onto exactly this pick.
    /// Why:
    ///       The player opens the next track a moment before the current one ends so the two play without a gap,
    ///  but the queue must keep pointing at the track that is still audible until then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// peekNextNatural(): number | null {
    ///   if (this.pos === null) return null;
    ///   if (this.repeatTrack) return this.order[this.pos];
    ///   if (this.shuffle) { if (this.pos + 1 >= this.order.length) this.order.push(this.pickNextShuffle(this.order[this.pos])); return this.order[this.pos + 1]; }
    ///   if (this.sequentialStart > 0 && this.pos < this.sequentialStart) return this.order[this.sequentialStart];
    ///   return this.order[this.pos + 1 < this.order.length ? this.pos + 1 : this.sequentialStart];
    /// }
    /// ```
    pub fn peek_next_natural(&mut self) -> Option<usize> {
        let pos = self.pos?;
        if self.mode == PlaybackMode::Repeat {
            return Some(self.order[pos]);
        }
        if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll {
            if pos + 1 >= self.order.len() {
                let current = self.order[pos];
                let pick = self.pick_next_shuffle(current);
                self.order.push(pick);
            }
            return Some(self.order[pos + 1]);
        }
        if self.sequential_start > 0 && pos < self.sequential_start {
            return Some(self.order[self.sequential_start]);
        }
        if pos + 1 < self.order.len() {
            return Some(self.order[pos + 1]);
        }
        // In order stops at the end of the scope, so no track is staged behind the last one; `advance(true)` agrees.
        if self.mode == PlaybackMode::InOrder {
            return None;
        }
        return Some(self.order[self.sequential_start])
    }

    /// What:
    ///      `pub fn first_in_scope(&self) -> Option<usize>` is the first track of the displayed page,
    /// or the first
    ///  track of the queue when no page has been shown to the engine yet;
    /// `None` for an empty queue.
    /// Why:
    ///       Play with nothing selected starts here (the polish review finding EM2):
    /// the button used to flip to Pause
    ///  and nothing played.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// firstInScope(): number | null { return this.pageScope[0] ?? (this.tracks.length ? 0 : null); }
    /// ```
    pub fn first_in_scope(&self) -> Option<usize> {
        if let Some(first) = self.page_scope.first() {
            return Some(*first);
        }
        if self.tracks.is_empty() {
            return None;
        }
        return Some(0);
    }

    /// What:
    ///      `pub fn rewind_scope(&mut self) -> Option<usize>` puts the cursor on the first track of the scope and returns
    ///  it,
    /// or `None` when nothing is queued.
    /// Why:
    ///       After In order has played its scope to the end and stopped,
    /// Play must start the folder again from its first
    ///  track,
    /// as a disc player does;
    /// leaving the cursor on the last track would end again at once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rewindScope(): number | null { if (!this.order.length) return null; this.pos = this.sequentialStart; return this.order[this.pos]; }
    /// ```
    pub fn rewind_scope(&mut self) -> Option<usize> {
        if self.order.is_empty() {
            return None;
        }
        let slot: usize = self.sequential_start.min(self.order.len() - 1);
        self.pos = Some(slot);
        tracing::info!(slot, track = self.order[slot], "the cursor is back on the first track of the scope");
        return Some(self.order[slot]);
    }

    /// What:
    ///      `pub fn prev(&mut self) -> Option<usize>` steps backward.
    ///  In `Off` it walks the
    ///           sequential scope and wraps to the end at the start;
    ///  in the shuffle modes it
    ///           steps back through the play history and stops at its start (no wrap).
    /// Why:
    ///       The user pressed Previous.
    ///  A shuffle history has no meaningful "last" to wrap
    ///           to,
    ///  so going back past its start would invent a track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prev(): number | null {
    ///   if (this.pos === null) return null;
    ///   const pos = this.pos;
    ///   if (this.shuffle !== "off") {
    ///     if (pos > 0) { this.pos = pos - 1; return this.order[pos - 1]; }
    ///     return this.order[pos];
    ///   }
    ///   if (pos > 0) { this.pos = pos - 1; return this.order[pos - 1]; }
    ///   const last = this.order.length - 1; this.pos = last; return this.order[last];
    /// }
    /// ```
    pub fn prev(&mut self) -> Option<usize> {
        // What:     `let pos = self.pos?;`. The `?` operator on the `Option<usize>` cursor:
        //           unwraps `Some(p)` to `p`, or returns `None` from `prev` immediately when
        //           the cursor is `None`.
        // Why:      Nothing to go back to when there is no cursor; bail out early.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.pos === null) return null;
        // const pos = this.pos;
        // ```
        let pos = self.pos?;
        // What:     `if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll { ... }`. Shuffle steps back through
        //           the history and stops at its start; `Off` falls through to the wrap below.
        // Why:      Going back past the start of a random history would invent a track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle !== "off") { if (pos > 0) { this.pos = pos - 1; return this.order[pos - 1]; } return this.order[pos]; }
        // ```
        if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll {
            // What:     `if pos > 0 { self.pos = Some(pos - 1); return Some(self.order[pos - 1]); }`.
            //           Step back one in the history.
            // Why:      Replay the previously-played track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (pos > 0) { this.pos = pos - 1; return this.order[pos - 1]; }
            // ```
            if pos > 0 {
                self.pos = Some(pos - 1);
                return Some(self.order[pos - 1]);
            }
            // What:     `return Some(self.order[pos]);`. Already at the history start: stay put.
            // Why:      No earlier history; report the current track unchanged.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return this.order[pos];
            // ```
            return Some(self.order[pos]);
        }
        // What:     `if self.sequential_start > 0 && pos < self.sequential_start { ... }`. The cursor sits on the retained
        //           off-page current track before the scope, so Previous leaves that prelude: `self.order.drain(...)`
        //           removes it, `self.sequential_start = 0;` makes the scope start at slot 0, and
        //           `let last = self.order.len() - 1;` with `self.pos = Some(last);` and `return Some(self.order[last]);`
        //           moves to the scope's last track and hands it back.
        // Why:      There is no earlier scope track to step to, so Previous wraps to the end of the scope, as it does at
        //           the scope's first slot. The prelude is dropped so it is not repeated.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.sequentialStart > 0 && pos < this.sequentialStart) {
        //   this.order.splice(0, this.sequentialStart); this.sequentialStart = 0;
        //   const last = this.order.length - 1; this.pos = last; return this.order[last];
        // }
        // ```
        if self.sequential_start > 0 && pos < self.sequential_start {
            self.order.drain(0..self.sequential_start);
            self.sequential_start = 0;
            let last = self.order.len() - 1;
            self.pos = Some(last);
            return Some(self.order[last]);
        }
        // What:     `if pos > self.sequential_start { ... }` there is a previous slot in the scope (Off). The bound is
        //           `self.sequential_start`, the first scope slot, not 0, so a prelude slot is never stepped back to.
        // Why:      Normal backward step.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (pos > this.sequentialStart) { this.pos = pos - 1; return this.order[pos - 1]; }
        // ```
        if pos > self.sequential_start {
            // What:     `self.pos = Some(pos - 1);`. Decrement the cursor.
            // Why:      Move to the previous track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pos = pos - 1;
            // ```
            self.pos = Some(pos - 1);
            // What:     `return Some(self.order[pos - 1]);`. That track index.
            // Why:      Hand back the previous track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return this.order[pos - 1];
            // ```
            return Some(self.order[pos - 1]);
        }
        // What:     `let last = self.order.len() - 1;` the last scope index.
        // Why:      At the start of the scope, Previous wraps to its end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const last = this.order.length - 1;
        // ```
        let last = self.order.len() - 1;
        // What:     `self.pos = Some(last);`. Set the cursor to the last slot.
        // Why:      Wrap behaviour (Previous at the start of the scope goes to its last track).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pos = last;
        // ```
        self.pos = Some(last);
        // What:     `Some(self.order[last])` tail expression.
        // Why:      Play the wrapped (last) track of the scope.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.order[last];
        // ```
        return Some(self.order[last])
    }
}
