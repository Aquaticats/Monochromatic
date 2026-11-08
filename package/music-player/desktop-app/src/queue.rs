//! The play queue:
//!  an ordered list of tracks plus a cursor,
//!  with shuffle and
//! "repeat track" behaviour.
//!  Pure logic,
//!  no audio,
//!  no I/O,
//!  so it is fully
//! unit-tested.
//!
//! Playback has a SCOPE that it walks,
//!  chosen by the playback mode:
//!
//! - `PlaybackMode::InOrder` and `PlaybackMode::ShufflePage` confine playback to the
//!   current track's PAGE (its top-level folder under the opened folder,
//!    or the page of the
//!   loose files for a track directly in it;
//!    the same grouping the UI tabs
//!   use,
//!    computed by the `pagination` module).
//!    `InOrder` plays the page in load
//!   order and STOPS at its end (D2:
//!    "In order = play to the end of the folder and stop");
//!    `ShufflePage` shuffles the page and,
//!    when every track has played,
//!    starts a new cycle.
//!    A manual Next at the end of the page wraps to its start.
//! - `PlaybackMode::ShuffleAll` scopes playback to the whole queue,
//!    shuffled,
//!    and loops
//!   the whole queue.
//!
//! Shuffle is JUST IN TIME and WITHOUT REPLACEMENT:
//!  there is no precomputed
//! permutation.
//!  Each cycle plays every track in the scope once,
//!  in a random order
//! chosen one pick at a time,
//!  then starts a fresh cycle.
//!  `order` doubles as the
//! play history,
//!  so `prev` steps back through it and a `next` after `prev`
//! retraces forward before drawing a new random pick.
//!  See
//! `doc/decision/music-player-jit-shuffle.md`.
//!
//! "Repeat track" is independent:
//!  when on,
//!  a track that ends NATURALLY replays
//! itself;
//!  a manual Next/Prev still moves within the scope.
//!
//! Design decision (deliberate):
//!  because `InOrder` is page-confined
//! and stops at the end of the page,
//!  there is no way to play the whole queue in load
//! order and loop the whole queue (non-shuffle + repeat-all).
//!  When not
//! shuffling,
//!  the user stays inside the current folder/page on purpose.

/// What:
///      `use std::path::PathBuf;` imports the OWNED filesystem-path type
///           (heap-allocated,
///  growable).
///  Sibling:
///  `&Path`,
///  a borrowed view.
/// Why:
///       The queue stores the actual file paths it will hand to the decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// What:
///      `use std::collections::HashSet;` the hash-set type.
/// Why:
///       The just-in-time shuffle pick excludes tracks already played this cycle;
///  a set
///           gives O(1) membership over a possibly large (whole-queue) scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const played = new Set<number>();
/// ```
use std::collections::HashSet;

/// What:
///      `use crate::command::PlaybackMode;` imports our own enum from the sibling
///           module.
///  `crate::` means "from the root of this package".
/// Why:
///       The queue's scope and ordering depend on the shuffle mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlaybackMode } from "./command";
/// ```
use crate::command::PlaybackMode;

/// What:
///      `pub struct Queue { ... }` declares a public record type with named fields.
///           The fields are private (no `pub`),
///  so only this module can touch them
///           directly.
/// Why:
///       Bundles the queue's state behind methods that keep it consistent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Queue {
///   private tracks: string[];
///   private order: number[];
///   private pos: number | null;
///   private mode: PlaybackMode;
///   private pageScope: number[];
///   private rngState: bigint;
/// }
/// ```
pub struct Queue {
    /// What:
    ///      `tracks: Vec<PathBuf>`.
    ///  An OWNED,
    ///  growable array of owned paths.
    ///  Sibling:
    ///           `&[PathBuf]`,
    ///  a borrowed slice that owns nothing.
    /// Why:
    ///       The tracks in the order the user loaded them;
    ///  the displayed queue list
    ///           uses this order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private tracks: string[];
    /// ```
    tracks: Vec<PathBuf>,
    /// The opened folder that display names and pages are relative to,
    /// or `None` to strip the shared directory.
    display_root: Option<PathBuf>,
    /// What:
    ///      `order: Vec<usize>`.
    ///  A growable array of indices into `tracks`.
    ///  `usize` is
    ///           the pointer-sized unsigned int used for indexing (siblings:
    ///  `u32`,
    ///  `u64`).
    /// Why:
    ///       The CURRENT SCOPE's playback order:
    ///  the load-order indices of the tracks
    ///           playback walks right now (the current page for Off/WithinPage,
    ///  or the whole
    ///           queue for All),
    ///  sequential or shuffled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private order: number[];
    /// ```
    order: Vec<usize>,
    /// What:
    ///      `pos: Option<usize>`.
    ///  "maybe an index":
    ///  `Some(p)` or `None`.
    /// Why:
    ///       The cursor's position WITHIN `order`.
    ///  `None` means the queue is empty /
    ///           nothing selected.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private pos: number | null;
    /// ```
    pos: Option<usize>,
    /// One selected completion,
    ///  ordering,
    ///  and scope behavior.
    mode: PlaybackMode,
    /// Load-order indices belonging to the page currently displayed by the UI.
    page_scope: Vec<usize>,
    /// First repeatable slot after any retained off-page current-track prelude.
    sequential_start: usize,
    /// What:
    ///      `rng_state: u64`.
    ///  An unsigned 64-bit integer (siblings:
    ///  `u32`,
    ///  `usize`,
    ///           `i64`).
    ///  Used as the running state of a tiny PRNG.
    /// Why:
    ///       Shuffling needs randomness;
    ///  a self-contained PRNG avoids a dependency and
    ///           stays seedable for deterministic tests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private rngState: bigint;
    /// ```
    rng_state: u64,
    /// What:
    ///      `cycle_start: usize`.
    ///  An index into `order` marking where the current shuffle
    ///           CYCLE began (a cycle plays every scope track once before repeating).
    ///  Only
    ///           meaningful in the shuffle modes;
    ///  `0` and unused for `Off`.
    /// Why:
    ///       The just-in-time without-replacement pick excludes tracks played since this
    ///           point;
    ///  when the scope is exhausted,
    ///  `cycle_start` jumps forward to start a new
    ///           cycle.
    ///  `order[cycle_start..]` is "played this cycle".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private cycleStart: number;
    /// ```
    cycle_start: usize,
}

/// What:
///      `impl Queue { ... }`.
///  The queue's methods (an `impl` block holds a type's
///           behaviour).
/// Why:
///       Group the queue's operations with its state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Queue { /* methods */ }
/// ```
impl Queue {
    /// What:
    ///      `pub fn new() -> Queue` is the public constructor.
    ///  `-> Queue` is the return
    ///           type.
    /// Why:
    ///       Creates an empty queue seeded from the clock so first-run shuffles differ
    ///           between launches.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): Queue { return new Queue(seedFromClock()); }
    /// ```
    pub fn new() -> Queue {
        // What:     `let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH);` and the `match` below turn the
        //           wall clock into a 64-bit seed: the nanoseconds since the epoch truncated to 64 bits, or a fixed
        //           non-zero constant when the clock is before the epoch.
        // Why:      We derive a changing seed from the current time.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const seed = BigInt(Date.now());
        // ```
        // Gotcha:   A clock before the epoch gives the fixed non-zero constant `0x9e3779b97f4a7c15`, because xorshift must never
        //           start at 0; otherwise the seed is the nanoseconds since the epoch truncated to 64 bits.
        let since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let seed: u64 = match since_epoch {
            Ok(elapsed) => elapsed.as_nanos() as u64,
            Err(_clock_before_epoch) => 0x9e3779b97f4a7c15,
        };
        // What:     `Queue::with_rng_seed(seed)` calls the seeded constructor. No trailing
        //           `;`, so it is the tail expression / return.
        // Why:      Share construction logic with the test constructor.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Queue.withRngSeed(seed);
        // ```
        return Queue::with_rng_seed(seed)
    }

    /// What:
    ///      `pub fn with_rng_seed(seed: u64) -> Queue` builds a queue with a
    ///           caller-chosen PRNG seed.
    /// Why:
    ///       Tests pass a fixed seed to get a deterministic shuffle.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static withRngSeed(seed: bigint): Queue { ... }
    /// ```
    pub fn with_rng_seed(seed: u64) -> Queue {
        // What:     `Queue { ... }`. A struct literal constructs the record. `Vec::new()`
        //           makes an empty owned array; `None` is the empty `Option`.
        //           `PlaybackMode::InOrder` is the path-qualified variant. No `;`, so this is the
        //           return value.
        // Why:      Start empty, not shuffled, repeat-track off.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { tracks: [], order: [], pos: null, shuffle: "off",
        //          repeatTrack: false, rngState: seed === 0n ? 1n : seed };
        // ```
        return Queue {
            tracks: Vec::new(),
            display_root: None,
            order: Vec::new(),
            pos: None,
            mode: PlaybackMode::InOrder,
            page_scope: Vec::new(),
            sequential_start: 0,
            // What:     `rng_state: if seed == 0 { 1 } else { seed }`. An `if/else`
            //           EXPRESSION that evaluates to one of the two branch values (no `;`).
            // Why:      xorshift gets stuck forever at state 0, so forbid it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // rngState: seed === 0n ? 1n : seed,
            // ```
            rng_state: if seed == 0 { 1 } else { seed },
            // What:     `cycle_start: 0`. No shuffle cycle has begun yet.
            // Why:      Set properly by `rebuild_scope_order` the first time a shuffle scope is
            //           anchored.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // cycleStart: 0,
            // ```
            cycle_start: 0,
        }
    }

    /// What:
    ///      `fn next_rand(&mut self) -> u64`.
    ///  `&mut self` is a MUTABLE borrow of the
    ///           queue:
    ///  the method may change `self`'s fields but does not own/consume it.
    ///           Private (no `pub`).
    /// Why:
    ///       Advances and returns the PRNG state (xorshift64).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private nextRand(): bigint { /* xorshift on this.rngState */ }
    /// ```
    fn next_rand(&mut self) -> u64 {
        // What:     `let mut x = self.rng_state;` binds a LOCAL MUTABLE copy. `mut` marks
        //           it reassignable; without it, bindings are read-only by default.
        // Why:      We mutate a local then store it back, the classic xorshift.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = this.rngState;
        // ```
        let mut x = self.rng_state;
        // What:     `x ^= x << 13;` is xor-assign with a left-shift. `^` is bitwise XOR,
        //           `<<` is bitwise left shift. On `u64` these are plain wrapping bit ops
        //           (no overflow concept for shifts).
        // Why:      One of the three xorshift mixing steps.
        // Gotcha:   `<<` on a 64-bit Rust int wraps within 64 bits; in TS you must mask
        //           with `BigInt.asUintN(64, ...)` to match.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // x = BigInt.asUintN(64, x ^ (x << 13n));
        // ```
        x ^= x << 13;
        // What:     `x ^= x >> 7;` xor-assign with a right shift.
        // Why:      Second xorshift step.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // x = BigInt.asUintN(64, x ^ (x >> 7n));
        // ```
        x ^= x >> 7;
        // What:     `x ^= x << 17;` xor-assign with a left shift.
        // Why:      Third xorshift step.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // x = BigInt.asUintN(64, x ^ (x << 17n));
        // ```
        x ^= x << 17;
        // What:     `self.rng_state = x;` writes the new state back through the mutable
        //           borrow.
        // Why:      Persist the PRNG progress for the next call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rngState = x;
        // ```
        self.rng_state = x;
        // What:     `x` alone on the last line is the tail expression: its value is returned.
        // Why:      Caller uses the fresh random number.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return x;
        // ```
        return x
    }

    /// Number of tracks in the queue.
    // What:     `pub fn len(&self) -> usize`. `&self` is a read-only (shared) borrow; the
    //           method cannot mutate the queue.
    // Why:      Callers ask how many tracks there are.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get length(): number { return this.tracks.length; }
    // ```
    pub fn len(&self) -> usize {
        // What:     `self.tracks.len()` returns the array length as `usize`. Tail expression
        //           -> return value.
        // Why:      Report the count.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.tracks.length;
        // ```
        return self.tracks.len()
    }

    /// Tracks in load order (as opened),
    ///  regardless of shuffle.
    // What:     `pub fn tracks(&self) -> &[PathBuf]`. Returns a BORROWED slice (`&[PathBuf]`,
    //           sibling of the owned `Vec<PathBuf>`) of the load-order paths. Read-only
    //           borrow; the caller may not mutate them.
    // Why:      The session save needs the queue's file paths to persist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get tracks(): readonly string[] { return this.tracks; }
    // ```
    pub fn tracks(&self) -> &[PathBuf] {
        // What:     `&self.tracks` borrows the `Vec` as a slice (the `Vec` derefs to
        //           `&[PathBuf]`). Tail expression -> return value.
        // Why:      Hand out a read-only view of the paths.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.tracks;
        // ```
        return &self.tracks
    }

    /// Whether the queue has no tracks.
    // What:     `pub fn is_empty(&self) -> bool`. Read-only borrow.
    // Why:      Convenience predicate; clippy also prefers this beside `len`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get isEmpty(): boolean { return this.tracks.length === 0; }
    // ```
    pub fn is_empty(&self) -> bool {
        // What:     `self.tracks.is_empty()` -> bool. Tail expression.
        // Why:      Report emptiness.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.tracks.length === 0;
        // ```
        return self.tracks.is_empty()
    }

    /// Current completion and transport mode.
    // What:     `pub fn playback_mode(&self) -> PlaybackMode`. Read-only borrow.
    // Why:      The engine mirrors the mode to the UI radio group.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // shuffleMode(): PlaybackMode { return this.shuffle; }
    // ```
    pub fn playback_mode(&self) -> PlaybackMode {
        // Return the same enum value used by navigation and persistence.
        return self.mode
    }

    /// What:
    ///      `pub fn display_paths(&self) -> Vec<String>` returns owned display strings
    ///           in load order:
    ///  each track's path relative to the queue's common root (e.g.
    ///           `Artist/Album/01.flac`,
    ///  or just `01.flac` when the whole queue is one
    ///           folder).
    /// Why:
    ///       The UI shows the folder a track lives in,
    ///  not just its filename,
    ///  so
    ///           pagination can group by folder;
    ///  the absolute prefix is stripped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// displayPaths(): string[] { return relativeDisplayPaths(this.tracks); }
    /// ```
    pub fn display_paths(&self) -> Vec<String> {
        // What:     `crate::relpath::relative_display_paths(&self.tracks)`. Call the pure
        //           path helper. `crate::` means "from this package's root"; `&self.tracks`
        //           lends the `Vec<PathBuf>` (which coerces to the `&[PathBuf]` slice the
        //           helper takes) read-only. Tail expression -> return.
        // Why:      One source of truth for the common-prefix stripping, reused and
        //           unit-tested in the `relpath` module.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return relativeDisplayPaths(this.tracks);
        // ```
        return crate::relpath::relative_display_paths_in(&self.tracks, self.display_root.as_deref())
    }

    /// What:
    ///      `pub fn set_display_root(&mut self, root: Option<PathBuf>)` names the opened folder that display names and
    ///  pages are relative to.
    /// Why:
    ///       A library with one top-level folder must keep that folder in its names,
    /// and loose files must form one
    ///  page;
    /// both need the opened folder,
    /// not the longest directory the tracks share (LS1).
    /// Set it before the tracks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setDisplayRoot(root: string | null): void { this.displayRoot = root; }
    /// ```
    pub fn set_display_root(&mut self, root: Option<PathBuf>) {
        tracing::debug!(root = ?root, "the queue's display root is set");
        self.display_root = root;
    }

    /// What:
    ///      `fn order_position_of(&self, track: usize) -> Option<usize>`.
    ///  The position of a track in the play order,
    /// or nothing when the order does not hold it.
    /// Why:
    ///       Playing a track inside the current scope moves the cursor to its place in the order,
    /// and a track outside the scope rebuilds the order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// orderPositionOf(track: number): number | null { const i = this.order.indexOf(track); return i < 0 ? null : i; }
    /// ```
    fn order_position_of(&self, track: usize) -> Option<usize> {
        for (position, candidate) in self.order.iter().enumerate() {
            if *candidate == track {
                return Some(position);
            }
        }
        return None;
    }

    /// What:
    /// `position_of_path` gives the load-order index of the track with this path,
    /// or nothing when the queue does not hold it.
    /// Why:
    /// a saved or preselected file is found in the scanned queue by its path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// positionOfPath(path: string): number | null { const i = this.tracks.indexOf(path); return i < 0 ? null : i; }
    /// ```
    pub fn position_of_path(&self, wanted: &std::path::Path) -> Option<usize> {
        for (index, track) in self.tracks.iter().enumerate() {
            if track.as_path() == wanted {
                return Some(index);
            }
        }
        return None;
    }

    /// What:
    ///      `pub fn current_index(&self) -> Option<usize>` returns the LOAD-ORDER index
    ///           of the current track (into `tracks`),
    ///  or None.
    /// Why:
    ///       The UI highlights this row;
    ///  NowPlaying carries it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// currentIndex(): number | null {
    ///   return this.pos === null ? null : this.order[this.pos];
    /// }
    /// ```
    pub fn current_index(&self) -> Option<usize> {
        // What:     `let position = self.pos?;` returns `None` when no track is selected, and
        //           `Some(self.order[position])` indexes the order with the cursor.
        // Why:      Translate the cursor's order-position into a track index.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.pos === null ? null : this.order[this.pos];
        // ```
        let position = self.pos?;
        return Some(self.order[position])
    }

    /// What:
    ///      `pub fn current_path(&self) -> Option<&PathBuf>` returns a BORROWED
    ///           reference to the current path,
    ///  or None.
    ///  `&PathBuf` is a shared borrow tied
    ///           to `self`'s lifetime.
    /// Why:
    ///       The engine needs the path to open the file,
    ///  without copying it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// currentPath(): string | null {
    ///   const i = this.currentIndex();
    ///   return i === null ? null : this.tracks[i];
    /// }
    /// ```
    pub fn current_path(&self) -> Option<&PathBuf> {
        // What:     `match self.current_index() { Some(index) => Some(&self.tracks[index]), None => None }` borrows
        //           the current track from the list, or answers `None` when no track is selected.
        // Why:      Avoid cloning the path on every access.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const i = this.currentIndex(); return i === null ? null : this.tracks[i];
        // ```
        return match self.current_index() {
            Some(index) => Some(&self.tracks[index]),
            None => None,
        }
    }

    /// What:
    ///      `pub fn set_tracks(&mut self, tracks: Vec<PathBuf>)`.
    ///  The parameter is taken
    ///           BY VALUE (ownership moves into the queue).
    /// Why:
    ///       Replacing the queue when the user opens new files.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setTracks(tracks: string[]): void {
    ///   this.tracks = tracks;
    ///   this.rebuildScopeOrder(tracks.length ? 0 : null);
    /// }
    /// ```
    pub fn set_tracks(&mut self, tracks: Vec<PathBuf>) {
        // What:     with no page selected there is nothing to carry over, so the new list is stored and the order rebuilt.
        // Why:      the carry-over below rebuilds the display names and the pages of the whole list, about a second at one
        //           million tracks, and its result is an empty scope when no page was selected.
        if self.page_scope.is_empty() {
            self.tracks = tracks;
            self.page_scope = Vec::new();
            self.rebuild_scope_order(Some(0));
            return;
        }
        let previous_pages = crate::page_index::PageIndex::build(&self.display_paths());
        let previous_page: Option<usize> = match self.page_scope.first() {
            Some(index) => previous_pages.page_of_index(*index),
            None => None,
        };
        let previous_page_identity: Option<String> = match previous_page {
            Some(page) => previous_pages.identity_at(page),
            None => None,
        };
        self.tracks = tracks;
        let rebuilt_pages = crate::page_index::PageIndex::build(&self.display_paths());
        let found_by_identity: Option<usize> = match previous_page_identity {
            Some(identity) => rebuilt_pages.position_of_identity(&identity),
            None => None,
        };
        let rebuilt_page: Option<usize> = match (found_by_identity, previous_page) {
            (Some(position), _) => Some(position),
            (None, Some(page)) => {
                let fallback = page.min(rebuilt_pages.page_count().saturating_sub(1));
                if fallback < rebuilt_pages.page_count() { Some(fallback) } else { None }
            }
            (None, None) => None,
        };
        self.page_scope = match rebuilt_page {
            Some(position) => match rebuilt_pages.page(position) {
                Some(group) => {
                    let mut scope: Vec<usize> = Vec::with_capacity(group.indices().len());
                    for index in group.indices() {
                        scope.push(*index as usize);
                    }
                    scope
                }
                None => Vec::new(),
            },
            None => Vec::new(),
        };
        // What:     `self.rebuild_scope_order(Some(0));` builds the scope order around the
        //           first track. `Some(0)` wraps index 0; the helper handles the empty queue
        //           (order empty, cursor None).
        // Why:      Start playback at the first track's page (or whole queue).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rebuildScopeOrder(0);
        // ```
        self.rebuild_scope_order(Some(0));
    }

    /// What:
    ///      `pub(crate) fn tracks_for_edit(&mut self) -> Vec<PathBuf>`.
    ///  The track list to edit:
    /// moved out when no page scope is set,
    ///  copied otherwise.
    /// Why:
    ///       An edit of a library of a million tracks must not copy it when nothing else reads the queue meanwhile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tracksForEdit(): string[] { return this.pageScope.length === 0 ? this.tracks.splice(0) : [...this.tracks]; }
    /// ```
    pub(crate) fn tracks_for_edit(&mut self) -> Vec<PathBuf> {
        if self.page_scope.is_empty() {
            return std::mem::take(&mut self.tracks);
        }
        return self.tracks.clone();
    }

    /// What:
    ///      `pub(crate) fn install_edited_tracks(&mut self, edited: Vec<PathBuf>)`.
    ///  Puts an edited track list back:
    /// directly when no page scope is set,
    ///  through `set_tracks` otherwise.
    /// Why:
    ///       A page scope refers to track positions,
    /// so a list that changed under it must rebuild the scope.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// installEditedTracks(edited: string[]): void { if (this.pageScope.length === 0) this.tracks = edited; else this.setTracks(edited); }
    /// ```
    pub(crate) fn install_edited_tracks(&mut self, edited: Vec<PathBuf>) {
        if self.page_scope.is_empty() {
            self.tracks = edited;
            return;
        }
        self.set_tracks(edited);
    }

    /// What:
    ///      `pub fn clear_selection(&mut self)`.
    ///  Drop the current-track selection:
    ///  after
    ///           this,
    ///  no track is current and there is no playback scope until the user picks
    ///           one.
    ///  `&mut self` is a MUTABLE borrow of the queue (we reassign its fields).
    /// Why:
    ///       Opening a library should auto-select NOTHING.
    ///  The controller calls this after
    ///           `set_tracks` on a normal open,
    ///  and the restore path calls it when the saved
    ///           session had no current track,
    ///  so a fresh queue highlights and loads nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// clearSelection(): void { this.rebuildScopeOrder(null); }
    /// ```
    pub fn clear_selection(&mut self) {
        // What:     `self.rebuild_scope_order(None);`. Rebuild the scope with a `None` anchor;
        //           `None` is the absent variant of `Option<usize>`, which `rebuild_scope_order`
        //           treats as "no current track" (empties `order`, sets `pos = None`).
        // Why:      Reuse the single method that owns the scope/cursor invariant instead of
        //           poking the fields here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rebuildScopeOrder(null);
        // ```
        self.rebuild_scope_order(None);
    }

    /// What:
    ///      `pub fn set_playback_mode(&mut self, mode: PlaybackMode)` changes the shuffle/scope
    ///           mode while keeping the currently-playing track current.
    /// Why:
    ///       Switching shuffle should not interrupt the current song.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setShuffle(mode: PlaybackMode): void {
    ///   if (mode === this.shuffle) return;
    ///   const cur = this.currentIndex();
    ///   this.shuffle = mode;
    ///   this.rebuildScopeOrder(cur);
    /// }
    /// ```
    pub fn set_playback_mode(&mut self, mode: PlaybackMode) {
        // What:     `if mode == self.mode { return; }`. Early return when nothing
        //           changes. `==` compares the enum.
        // Why:      Avoid reshuffling (and moving the cursor) on a no-op.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (mode === this.shuffle) return;
        // ```
        if mode == self.mode {
            return;
        }
        // What:     `let current = self.current_index();` remembers the playing track
        //           (Option<usize>) before we rebuild the scope.
        // Why:      So the rebuild can keep the cursor on the same track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const current = this.currentIndex();
        // ```
        let current = self.current_index();
        // What:     `self.mode = mode;` record the new mode.
        // Why:      `rebuild_scope_order`/`scope_indices` read it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.shuffle = mode;
        // ```
        self.mode = mode;
        // What:     `self.rebuild_scope_order(current);` rebuild the scope order anchored on
        //           the previously playing track.
        // Why:      Apply the new mode without losing the current track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rebuildScopeOrder(current);
        // ```
        self.rebuild_scope_order(current);
    }

    /// Updates the displayed page's load-order scope while retaining current audio.
    pub fn set_page_scope(&mut self, indices: Vec<usize>) {
        let mut valid: Vec<usize> = Vec::new();
        for index in indices {
            if index < self.tracks.len() {
                valid.push(index);
            }
        }
        if valid == self.page_scope {
            return;
        }
        self.page_scope = valid;
        if self.mode == PlaybackMode::ShufflePage || self.mode == PlaybackMode::ShuffleAll {
            return;
        }
        let current = self.current_index();
        self.rebuild_scope_order(current);
    }

    /// What:
    ///      `pub fn play_index(&mut self, track: usize) -> Option<usize>` selects a
    ///           specific track (load-order index) as current,
    ///  switching the playback scope
    ///           if the track is on another page.
    /// Why:
    ///       The user clicked a row in the queue list.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playIndex(track: number): number | null {
    ///   if (track >= this.tracks.length) return null;
    ///   const p = this.order.indexOf(track);
    ///   if (p >= 0) this.pos = p;          // already in the current scope
    ///   else this.rebuildScopeOrder(track); // jumped to another page
    ///   return track;
    /// }
    /// ```
    pub fn play_index(&mut self, track: usize) -> Option<usize> {
        // What:     `if track >= self.tracks.len() { return None; }` bounds check.
        // Why:      Ignore an out-of-range click.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (track >= this.tracks.length) return null;
        // ```
        if track >= self.tracks.len() {
            return None;
        }
        if !self.page_scope.contains(&track) {
            self.page_scope = match crate::relpath::page_scope_for_anchor_in(&self.tracks, track, self.display_root.as_deref()) {
                Some(indices) => indices,
                None => (0..self.tracks.len()).collect(),
            };
        }
        // What:     `if self.mode == PlaybackMode::InOrder || self.mode == PlaybackMode::Repeat { ...find or rebuild... } else { rebuild }`.
        //           In `Off`, stay in the current scope when the track is already in it and
        //           only rebuild on a jump to another page; in the shuffle modes, always
        //           rebuild so the play history restarts from the clicked track.
        // Why:      For `Off`, `order` is the full sequential scope, so the track's slot is a
        //           valid cursor; for shuffle, `order` is the play history, and clicking a
        //           track should begin a fresh cycle at it rather than retrace into history.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle === "off") {
        //   const p = this.order.indexOf(track);
        //   if (p >= 0) this.pos = p; else this.rebuildScopeOrder(track);
        // } else this.rebuildScopeOrder(track);
        // ```
        if self.mode == PlaybackMode::InOrder || self.mode == PlaybackMode::Repeat {
            // What:     `match self.order_position_of(track) { ... }` finds the track in the play order: `Some(p)`
            //           moves the cursor there, and `None` (another page) rebuilds the scope around the track.
            // Why:      Stay in scope when possible; rebuild only on a jump to another page.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const p = this.order.indexOf(track);
            // ```
            match self.order_position_of(track) {
                // What:     `Some(p) => self.pos = Some(p)`. Already in scope: move the cursor.
                // Why:      Clicking another track on the same page keeps the scope intact.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (p >= 0) this.pos = p;
                // ```
                Some(p) => self.pos = Some(p),
                // What:     `None => self.rebuild_scope_order(Some(track))`. Another page:
                //           rebuild the scope around the clicked track.
                // Why:      Switch playback to the clicked track's page.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // else this.rebuildScopeOrder(track);
                // ```
                None => self.rebuild_scope_order(Some(track)),
            }
        } else {
            // What:     `self.rebuild_scope_order(Some(track));`. Restart the shuffle history at
            //           the clicked track (order = [track], a fresh cycle).
            // Why:      Selecting a track under shuffle should make it current and shuffle
            //           onward from there, not replay recorded history.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.rebuildScopeOrder(track);
            // ```
            self.rebuild_scope_order(Some(track));
        }
        // What:     `Some(track)` tail expression: report the now-current track.
        // Why:      The caller loads this index.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return track;
        // ```
        return Some(track)
    }

    /// Restores a relocated selected track without replacing the displayed-page scope.
    pub fn restore_index_preserving_page(&mut self, track: usize) -> Option<usize> {
        if track >= self.tracks.len() {
            return None;
        }
        self.rebuild_scope_order(Some(track));
        return Some(track);
    }

}

/// What:
///      `impl Default for Queue { ... }` lets `Queue::default()` work and satisfies
///           clippy's "type with new() should impl Default" lint.
/// Why:
///       Idiomatic;
///  some generic code expects `Default`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Queue.default() === Queue.new()
/// ```
impl Default for Queue {
    /// What:
    ///      `fn default() -> Queue`.
    ///  The single method `Default` requires.
    /// Why:
    ///       Provide the zero-argument construction generic code expects.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): Queue { return Queue.new(); }
    /// ```
    fn default() -> Queue {
        // What:     `Queue::new()`. Delegate to the seeded constructor. Tail expression.
        // Why:      One source of truth for construction.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Queue.new();
        // ```
        return Queue::new()
    }
}

/// What:
///      `#[path = "queue_transport.rs"] mod transport;` declares a child module
///           whose code lives in the sibling file `queue_transport.rs`.
/// Why:
///       The cursor movement methods fill a file of their own,
///  and a child module reaches the private fields of `Queue` that they move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // queueTransport.ts: more methods of the same Queue class
/// ```
#[path = "queue_transport.rs"]
mod transport;

/// What:
///      `#[path = "queue_scope_order.rs"] mod scope_order;` declares a child module
///           whose code lives in the sibling file `queue_scope_order.rs`.
/// Why:
///       The scope and shuffle-order methods fill a file of their own,
///  and a child module reaches the private fields of `Queue` that they read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // queueScopeOrder.ts: more methods of the same Queue class
/// ```
#[path = "queue_scope_order.rs"]
mod scope_order;

/// What:
///      `#[cfg(test)] #[path = "queue_tests.rs"] mod tests;` declares a test-only
///           submodule whose code lives in the sibling file `queue_tests.rs`.
///  `#[cfg(test)]`
///           gates it to test builds only;
///  `#[path = "..."]` aims the module at a flat
///           sibling file instead of the default `queue/tests.rs` subdirectory lookup.
///  The
///           file stays the `tests` CHILD of queue,
///  so its `use super::*` reaches the module
///           items (including private ones) unchanged.
/// Why:
///       Keep `queue.rs` to production code;
///  the tests live beside it without inflating
///           this file or its max-lines budget (sibling `*_tests.rs` files are exempt from
///           the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // queue.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "queue_tests.rs"]
mod tests;
