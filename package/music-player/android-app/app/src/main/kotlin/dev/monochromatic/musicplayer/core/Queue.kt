// What:     `package dev.monochromatic.musicplayer.core` declares which namespace
//           (logical folder of names) every declaration in this file belongs to.
//           In Kotlin/Java the package name must mirror the directory path under
//           `src/main/kotlin`, so this file physically lives at
//           `.../dev/monochromatic/musicplayer/core/Queue.kt`. Other files refer
//           to `Queue` either by importing `dev.monochromatic.musicplayer.core.Queue`
//           or by sitting in this same package (as the sibling `PlaybackMode`,
//           `Page`, and `paginate` do).
// Why:      Without a package line everything would land in the unnamed "root"
//           package, which Kotlin discourages and which collides as the app grows.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — the file path ./core/Queue.ts IS the module name.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import kotlin.random.Random` pulls the standard-library pseudo-random
//           number generator type `Random` into this file by its short name, so we
//           can write `Random` instead of the fully-qualified `kotlin.random.Random`.
//           `kotlin.random` is the package (namespace) the type lives in; `Random`
//           is the specific class.
// Why:      The queue's shuffle needs a SEEDED random source. `kotlin.random.Random`
//           can be constructed from a fixed `Long` seed, which is exactly what makes
//           the shuffle deterministic (same seed -> same order) for the tests and
//           for session restore.
// Gotcha:   This is NOT the desktop port's xorshift64 PRNG. The desktop (`queue.rs`)
//           hand-rolls a 64-bit xorshift with `^=`/`<<`; this Kotlin port instead
//           leans on the platform's seeded `Random`. The two produce DIFFERENT
//           shuffle sequences; only the "same seed -> same order on THIS platform"
//           guarantee is shared (see the class summary below).
//
// In TS you'd write (pseudocode):
// ```ts
// import { Random } from "./seeded-random"; // a seedable PRNG, since Math.random isn't
// ```
import kotlin.random.Random

// =============================================================================
// File summary (folds in the old class KDoc's domain content)
// =============================================================================
//
// This file defines `Queue`: the play queue, an ordered list of tracks plus a
// cursor, with shuffle and "repeat track" behaviour. It is PURE LOGIC: no audio,
// no I/O, so it is fully unit-tested. It is a faithful port of the desktop's
// `queue.rs`.
//
// Playback has a SCOPE that it walks, chosen by the playback mode (the
// sibling `PlaybackMode` enum):
//
//   - `PlaybackMode.IN_ORDER` and `PlaybackMode.SHUFFLE_PAGE` confine playback to the
//     current track's PAGE: its top-level folder under the loaded root, or its
//     A-Z/`#` letter bucket for a root-level track. This is the same grouping the
//     UI tabs use, computed by the sibling `paginate` function. `IN_ORDER` plays the
//     page in load order and STOPS after its last track (D2); `SHUFFLE_PAGE` shuffles the
//     page and starts a new cycle when every track has played. A manual Next at the end
//     of an In order page wraps to its start.
//   - `PlaybackMode.SHUFFLE_ALL` scopes playback to the whole queue, shuffled, and loops
//     the whole queue.
//
// `PlaybackMode.REPEAT` replays a track that ends NATURALLY; a manual Next/Prev still
// moves within the page. Because `IN_ORDER` and `REPEAT` are page-confined, there is
// deliberately NO way to play the whole queue in load order; when not shuffling,
// the user stays inside the current page.
//
// Portability note (a real Gotcha): the desktop's deterministic shuffle uses a
// seeded xorshift64 PRNG. That exact cross-language sequence is not portable, so
// this port shuffles with a seeded `kotlin.random.Random` instead. The same seed
// still yields the same order ON THIS PLATFORM, which is what the tests (and the
// session restore) rely on. Do not assume a given seed reproduces the desktop's
// order; only the within-Kotlin determinism is guaranteed.

// What:     `class Queue private constructor(private val rng: Random) { ... }`
//           declares a class named `Queue`. Several pieces:
//           - `class Queue` is the type itself (a bundle of state + methods).
//           - `private constructor(...)` makes the PRIMARY constructor private:
//             outside code cannot call `Queue(...)` directly; it must go through
//             the `new`/`withRngSeed` factories in the companion object below.
//           - `private val rng: Random` is a constructor parameter that ALSO
//             becomes a private, read-only PROPERTY of the class in one stroke
//             (the `val` keyword on a constructor param is Kotlin shorthand for
//             "store this argument as a field"). `val` = read-only (immutable
//             binding); the sibling `var` would be reassignable. Its type is
//             `Random` (the seeded PRNG imported above).
// Why:      Bundles the queue's state behind methods that keep it consistent, and
//           forces construction through the factories so every queue gets a
//           properly seeded `Random`. Storing `rng` as a field lets `shuffleSlice`
//           draw repeatable random numbers across many calls.
// Gotcha:   The constructor being `private` means `new Queue(rng)` is unreachable
//           from outside; the only doors in are `Queue.new()` and
//           `Queue.withRngSeed(seed)`. (Replaces the old `@constructor`/`@param rng`
//           KDoc: the constructor builds an empty queue driven by the given seeded
//           `Random`; callers use `withRngSeed`/`new` instead of calling it.)
//
// In TS you'd write (pseudocode):
// ```ts
// class Queue {
//   private constructor(private readonly rng: Random) {}
//   // ...state fields and methods below...
// }
// ```
/**
 * Defines queue type for this music-player component; the TypeScript-oriented notes above explain its role.
 */
class Queue private constructor(private val rng: Random) {
    //region State
    // What:     `private var tracks: List<String> = emptyList()` declares a private,
    //           REASSIGNABLE field named `tracks`.
    //           - `var` (not `val`) means the field can be pointed at a new list
    //             later (e.g. when the user opens new files).
    //           - `: List<String>` is the type: a READ-ONLY list (interface) of
    //             strings. Siblings the reader might expect: `MutableList<String>`
    //             (a list you can `.add`/`.removeAt`), or `Array<String>` (a
    //             fixed-size array). `List` is chosen because the field is replaced
    //             wholesale, never mutated element-by-element, so the read-only
    //             interface is enough and prevents accidental in-place edits.
    //           - `= emptyList()` is the initial value: a shared, zero-length
    //             read-only list (see the `emptyList()` note below).
    // Why:      Holds the tracks in the order the user loaded them; the displayed
    //           queue list uses this order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private tracks: readonly string[] = [];
    // ```
    /**
     * Defines tracks value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var tracks: List<String> = emptyList()

    /** Relative display paths of [tracks], or null until first asked for or handed over by a library build. */
    private var cachedPaths: List<String>? = null

    /** Pages of [tracks], or null until first asked for or handed over by a library build. */
    private var cachedPages: List<Page>? = null

    /** Tab-order page of each load-order index, or null until first asked for or handed over by a build. */
    private var cachedLookup: IntArray? = null

    // What:     `private var order: List<Int> = emptyList()` declares a private,
    //           reassignable read-only list of integers.
    //           - `List<Int>` holds LOAD-ORDER INDICES (positions into `tracks`),
    //             not the strings themselves. `Int` is a 32-bit signed integer;
    //             siblings: `Long` (64-bit), `Short`, `Byte`. `Int` is right here
    //             because a queue never holds billions of tracks, so 32 bits is
    //             ample and matches every list-index API.
    //           - `emptyList()` again seeds it empty.
    // Why:      This is the CURRENT SCOPE's playback order: the load-order indices of
    //           the tracks playback walks right now (the current page for
    //           `OFF`/`WITHIN_PAGE`, or the whole queue for `ALL`), sequential or
    //           shuffled. Storing indices (not paths) lets the same scope reference
    //           tracks cheaply and keep the `tracks` list as the single source of
    //           truth for the strings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private order: readonly number[] = [];
    // ```
    /**
     * Defines order value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var order: List<Int> = emptyList()

    // What:     `private var pos: Int? = null` declares a private, reassignable field
    //           whose type is `Int?` — a NULLABLE `Int`. The trailing `?` makes the
    //           type "either an `Int` or `null`". The initial value is `null`.
    //           Sibling the reader might expect: a plain non-null `Int` (which could
    //           never represent "nothing selected").
    // Why:      The cursor's POSITION WITHIN `order` (not within `tracks`). `null`
    //           means the queue is empty / nothing is selected. The nullability is
    //           how "no current track" is modelled without a sentinel like `-1`.
    // Gotcha:   `Int?` forces every read to handle the `null` case (via `?.`, `?:`,
    //           or a null check) — the compiler will not let you index `order[pos]`
    //           until you have proven `pos` is non-null. This is Kotlin's
    //           null-safety, stricter than TS unless `strictNullChecks` is on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private pos: number | null = null;
    // ```
    /**
     * Defines pos value for this music-player component; the TypeScript-oriented notes above explain its source
     * and use.
     */
    private var pos: Int? = null

    // What:     `private var mode: PlaybackMode` stores one of four enum values.
    // Why:      Natural completion, manual transport, shuffle scope, and persistence
    //           must never disagree through separate flags.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private mode: PlaybackMode = "in_order";
    // ```
    /** Single selected playback behavior. */
    private var mode: PlaybackMode = PlaybackMode.IN_ORDER

    // What:     `List<Int>` is a read-only list of load-order indices; its mutable
    //           sibling is `MutableList<Int>`.
    // Why:      The displayed page, not necessarily the current track's page, decides
    //           the scope used by In order and Shuffle page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private pageScope: readonly number[] = [];
    // ```
    /** Load-order indices belonging to the currently displayed page. */
    private var pageScope: List<Int> = emptyList()

    // What:     `Int` is Kotlin's 32-bit signed integer; siblings include `Long`,
    //           `Short`, and `Byte`.
    // Why:      A retained current track may sit before the displayed page temporarily;
    //           this index marks where sequential wrapping begins after that transition.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private sequentialStart = 0;
    // ```
    /** First repeatable position in a sequential order. */
    private var sequentialStart: Int = 0

    // What:     `private var cycleStart: Int = 0` declares a private, reassignable
    //           `Int` field, initialised to `0`. It is an index INTO `order`: the
    //           position at which the CURRENT just-in-time shuffle cycle began.
    // Why:      Just-in-time shuffle plays each scope track once per cycle WITHOUT
    //           precomputing a permutation. The slice `order[cycleStart until order.size]`
    //           is the set already played this cycle; a new pick is drawn only from
    //           scope tracks NOT in that set (without replacement). When a cycle has
    //           exhausted the scope, `cycleStart` advances to `order.size` so the next
    //           pick begins a fresh cycle. Under `PlaybackMode.IN_ORDER` it is unused
    //           (sequential order needs no play history). Mirrors the desktop
    //           `queue.rs` `cycle_start`. See `doc/decision/music-player-jit-shuffle.md`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private cycleStart = 0; // index into `order`: start of the current shuffle cycle
    // ```
    /**
     * Defines cycle start value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var cycleStart: Int = 0
    //endregion

    /**
     * Defines companion object for this music-player component; the TypeScript-oriented notes above explain its
     * shared role.
     */
    //region Factories
    // What:     `companion object { ... }` declares a single, file-private OBJECT
    //           attached to the `Queue` class. A "companion object" is Kotlin's way
    //           to hang STATIC-LIKE members (functions/values that belong to the
    //           class itself, not to an instance) off a class. Members declared
    //           inside it are called as `Queue.new()` / `Queue.withRngSeed(...)`,
    //           exactly like static methods.
    // Why:      It hosts the public factory functions that mirror the desktop Rust
    //           `new` / `with_rng_seed` constructors. Because the primary
    //           constructor is private, these factories are the only way to build a
    //           `Queue`; they live here so they can reach the private constructor.
    // Gotcha:   The companion is itself an object instance (you could name it), but
    //           here it is anonymous and used purely as a static-method bag. Calling
    //           `Queue.new()` does NOT create the companion each time; there is
    //           exactly one companion per class.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // static factory methods on the Queue class:
    // //   static new(): Queue { ... }
    // //   static withRngSeed(seed: bigint): Queue { ... }
    // ```
    companion object {
        // What:     `fun new(): Queue = withRngSeed(System.nanoTime())` declares a
        //           companion (static-like) function named `new` taking no
        //           parameters and returning a `Queue`. The `= expr` form is an
        //           EXPRESSION BODY: the single expression after `=` IS the return
        //           value (no `return` keyword, no braces).
        //           - `System.nanoTime()` is a Java-interop call: `System` is the
        //             java.lang class, `nanoTime()` returns a `Long` (64-bit signed
        //             integer) high-resolution timestamp.
        //           - `withRngSeed(...)` is the sibling factory, called with that
        //             timestamp as the seed; its result is `new`'s result.
        // Why:      Creates an empty queue seeded from the wall clock so first-run
        //           shuffles differ between launches (each launch's `nanoTime` is
        //           different). Mirrors the Rust `Queue::new`.
        // Gotcha:   `System.nanoTime()` returns a `Long`, NOT an `Int`; it is a
        //           nanosecond counter that easily exceeds 32 bits. Feeding it
        //           straight into `withRngSeed(seed: Long)` is why that parameter is
        //           `Long`, not `Int`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // static new(): Queue {
        //   return Queue.withRngSeed(BigInt(performance.now())); // nondeterministic seed
        // }
        // ```
        /**
         * Defines new behavior for this music-player component; the TypeScript-oriented notes above explain its
         * call shape and effects.
         */
        fun new(): Queue = withRngSeed(System.nanoTime())

        // What:     `fun withRngSeed(seed: Long): Queue = Queue(Random(seed))` declares
        //           a companion factory taking one parameter and returning a `Queue`,
        //           again as an expression body (`= expr` is the return).
        //           - `seed: Long` is the parameter: a 64-bit signed integer seed.
        //             Sibling `Int` (32-bit) is declined because `System.nanoTime()`
        //             above overflows 32 bits, and a wider seed gives more shuffle
        //             entropy.
        //           - `Random(seed)` is a CONSTRUCTOR CALL on the imported `Random`
        //             type: it builds a seeded PRNG. (In Kotlin you call a constructor
        //             just like a function, no `new` keyword.)
        //           - `Queue(...)` then calls the PRIVATE primary constructor with
        //             that `Random`, producing the empty queue. The companion can
        //             reach the private constructor because it is nested inside the
        //             class.
        // Why:      Creates an empty queue with a CALLER-CHOSEN PRNG seed, mirroring
        //           the Rust `Queue::with_rng_seed`; tests pass a fixed seed to get a
        //           deterministic shuffle (same seed -> same order).
        // Gotcha:   No `new` keyword anywhere: `Random(seed)` constructs a `Random`,
        //           and `Queue(...)` constructs a `Queue`. A TS reader expects `new`;
        //           Kotlin constructor calls look identical to function calls.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // static withRngSeed(seed: bigint): Queue {
        //   return new Queue(new Random(seed)); // same seed -> same shuffle order
        // }
        // ```
        /**
         * Defines with rng seed behavior for this music-player component; the TypeScript-oriented notes above
         * explain its call shape and effects.
         */
        fun withRngSeed(seed: Long): Queue = Queue(Random(seed))
    }
    //endregion

    //region Read-only accessors
    // What:     `fun len(): Int = tracks.size` declares an INSTANCE method `len`
    //           (no `companion`, so it runs on a `Queue` instance) returning `Int`,
    //           as an expression body. `tracks.size` reads the `.size` property of
    //           the `List<String>` field (its element count, an `Int`); that value
    //           IS the return.
    // Why:      Callers ask how many tracks are in the queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // len(): number { return this.tracks.length; }
    // ```
    /**
     * Defines len behavior for this music-player component; the TypeScript-oriented notes above explain its call
     * shape and effects.
     */
    fun len(): Int = tracks.size

    // What:     `fun tracks(): List<String> = tracks` declares an instance method
    //           `tracks()` returning the read-only `List<String>`, as an expression
    //           body. The body `tracks` (the field) is the return value. Note the
    //           method and the field share the name; `tracks` here resolves to the
    //           field because there is no `tracks(...)` call.
    // Why:      Exposes the tracks in load order (as opened), regardless of shuffle;
    //           the session save persists these. Returning the read-only `List`
    //           interface hands out a view callers cannot mutate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // tracks(): readonly string[] { return this.tracks; }
    // ```
    /**
     * Defines tracks behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun tracks(): List<String> = tracks

    // What:     `fun isEmpty(): Boolean = tracks.isEmpty()` declares an instance
    //           method returning `Boolean`, expression body. `tracks.isEmpty()` is a
    //           stdlib `List` method returning `true` when the list has zero
    //           elements; that boolean is the return.
    // Why:      Convenience predicate for "the queue has no tracks".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // isEmpty(): boolean { return this.tracks.length === 0; }
    // ```
    /**
     * Defines is empty behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun isEmpty(): Boolean = tracks.isEmpty()

    // What:     `fun playbackMode(): PlaybackMode = mode` is an expression-body
    //           accessor returning the enum by value.
    // Why:      UI and persistence mirror the same single mode the queue executes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playbackMode(): PlaybackMode { return this.mode; }
    // ```
    /** Returns the selected completion and transport behavior. */
    fun playbackMode(): PlaybackMode = mode

    // What:     `fun displayPaths(): List<String> = relativeDisplayPaths(tracks)`
    //           declares an instance method returning a read-only `List<String>`,
    //           expression body. `relativeDisplayPaths(tracks)` calls the sibling
    //           package-level function (defined in `RelPath.kt`) with the `tracks`
    //           field; its result IS the return.
    // Why:      Produces the display strings in load order: each track's path
    //           RELATIVE to the queue's common root, so the UI shows the folder a
    //           track lives in and pagination can group by folder. Delegating to the
    //           shared helper keeps one source of truth for the common-prefix
    //           stripping (unit-tested in `RelPath.kt`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // displayPaths(): string[] { return relativeDisplayPaths(this.tracks); }
    // ```
    /**
     * Defines display paths behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun displayPaths(): List<String> = cachedPaths ?: relativeDisplayPaths(tracks).also { paths -> cachedPaths = paths }

    /** Returns the pages of the tracks, built on first use and kept until the tracks change. */
    fun pages(): List<Page> = cachedPages ?: paginate(displayPaths()).also { built -> cachedPages = built }

    /** Returns the tab-order page of a load-order index, or null when no page holds it. */
    private fun pageOf(index: Int): Int? {
        /** Page table of the tracks, built on first use. */
        val table: IntArray = cachedLookup ?: pageLookupOf(pages(), tracks.size).also { built -> cachedLookup = built }
        return table.getOrNull(index)?.takeIf { page -> page >= 0 }
    }

    // What:     `fun currentIndex(): Int? = pos?.let { order[it] }` declares an
    //           instance method returning a NULLABLE `Int?`, expression body. The
    //           body is symbol-dense:
    //           - `pos` is the `Int?` cursor (position within `order`, or `null`).
    //           - `?.` is the SAFE-CALL operator: `pos?.let { ... }` runs the `.let`
    //             ONLY when `pos` is non-null; when `pos` is `null` the whole
    //             expression short-circuits to `null` (so `null` is the return).
    //           - `.let { ... }` is a scope function: it invokes the trailing lambda
    //             with the (now non-null) receiver as its single implicit argument.
    //           - `{ order[it] }` is that TRAILING LAMBDA. `it` is Kotlin's implicit
    //             name for the lambda's single parameter (here the non-null `pos`
    //             value). `order[it]` indexes the `order` list at that position,
    //             yielding the LOAD-ORDER index stored there.
    // Why:      Translates the cursor's position-within-`order` into the load-order
    //           index of the current track (into `tracks`), or `null` when nothing
    //           is selected; the UI highlights this row.
    // Gotcha:   `?.let { order[it] }` is the idiomatic Kotlin way to "map over a
    //           nullable". The `it` is auto-named; it is the non-null `pos`. This is
    //           NOT a loop — `.let` runs the block at most once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // currentIndex(): number | null {
    //   return this.pos === null ? null : this.order[this.pos];
    // }
    // ```
    /**
     * Defines current index behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun currentIndex(): Int? = pos?.let { order[it] }

    // What:     `fun currentPath(): String? = currentIndex()?.let { tracks[it] }`
    //           declares an instance method returning a NULLABLE `String?`,
    //           expression body. Same shape as `currentIndex`:
    //           - `currentIndex()` returns `Int?` (the current track's load-order
    //             index, or `null`).
    //           - `?.let { tracks[it] }` runs only when that index is non-null;
    //             `it` is the non-null index, and `tracks[it]` reads the path string
    //             at that position. When `currentIndex()` is `null`, the whole thing
    //             is `null`.
    // Why:      The path of the current track, or `null`; the engine needs it to open
    //           the file. Chaining off `currentIndex()` reuses the cursor->index
    //           translation rather than duplicating it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // currentPath(): string | null {
    //   const i = this.currentIndex();
    //   return i === null ? null : this.tracks[i];
    // }
    // ```
    /**
     * Defines current path behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun currentPath(): String? = currentIndex()?.let { tracks[it] }

    // What:     `fun playbackOrder(): List<Int> = order` declares an instance method
    //           returning the read-only `List<Int>` of load-order indices, expression
    //           body. It returns the `order` field directly.
    // Why:      The current scope's playback order as load-order indices: the sequence
    //           playback walks right now. This is the same order a MediaSession
    //           timeline must report so its (framework-computed) next/previous
    //           navigation matches this queue; position `i` in the result is timeline
    //           window index `i`. (No `queue.rs` twin: this method is
    //           MediaSession-oriented, specific to the Android port.)
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playbackOrder(): readonly number[] { return this.order; }
    // ```
    /**
     * Defines playback order behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun playbackOrder(): List<Int> = order

    // What:     `fun cursorPosition(): Int? = pos` declares an instance method
    //           returning the nullable `Int?` cursor, expression body. It returns the
    //           `pos` field as-is (no translation, unlike `currentIndex` which maps
    //           through `order`).
    // Why:      The cursor's POSITION WITHIN `playbackOrder` (the current timeline
    //           window index), or `null` when the queue is empty; the MediaSession
    //           reports this as the current media-item index. (Also no `queue.rs`
    //           twin: MediaSession-specific.)
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // cursorPosition(): number | null { return this.pos; }
    // ```
    /**
     * Defines cursor position behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun cursorPosition(): Int? = pos
    //endregion

    //region Mutators
    // What:     `fun setTracks(newTracks: List<String>) { ... }` declares an instance
    //           method taking one read-only `List<String>` parameter `newTracks` and
    //           returning `Unit` (void), block body. The parameter is the replacement
    //           track list in load order.
    // Why:      Replace the queue when the user opens new files, anchoring playback on
    //           the first track (or leaving it empty when there are no tracks).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setTracks(newTracks: readonly string[]): void {
    //   this.tracks = newTracks;
    //   this.rebuildScopeOrder(0);
    // }
    // ```
    /**
     * Defines set tracks behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun setTracks(newTracks: List<String>, built: BuiltLibrary? = null) {
        /** Pages derived from the prior track set. */
        val previousPages: List<Page> = pages()
        /** Prior displayed page position, or null. */
        val previousPage: Int? = pageScope.firstOrNull()?.let { previousIndex ->
            pageOf(previousIndex)
        }
        /** Stable prior page identity, or null. */
        val previousPageIdentity: String? = previousPage?.let { page ->
            pageIdentity(previousPages[page])
        }
        // What:     `tracks = newTracks` reassigns the `var` field to the new list.
        //           Plain assignment; no special punctuation.
        // Why:      Adopt the new track list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.tracks = newTracks;
        // ```
        tracks = newTracks
        cachedPaths = built?.displayPaths
        cachedPages = built?.pages
        cachedLookup = built?.pageOfIndex
        /** Rebuilt pages for resolving the retained displayed page. */
        val rebuiltPages: List<Page> = pages()
        /** Identity match, or numeric fallback when prior page was removed. */
        val rebuiltPage: Page? = rebuiltPages
            .firstOrNull { page -> pageIdentity(page) == previousPageIdentity }
            ?: previousPage?.let { page -> rebuiltPages.getOrNull(minOf(page, rebuiltPages.size - 1)) }
        pageScope = rebuiltPage?.entries?.map { entry -> entry.index } ?: emptyList()
        // What:     `rebuildScopeOrder(0)` calls the private helper with the literal
        //           anchor `0`. Note `0` is a plain non-null `Int` here, which the
        //           helper accepts as its `Int?` parameter (a non-null value is a
        //           valid `Int?`). The helper itself handles the empty-queue case.
        // Why:      Build the scope order around the first track so playback starts
        //           at the first track's page (or whole queue).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rebuildScopeOrder(0);
        // ```
        rebuildScopeOrder(0)
    }

    // What:     `fun clearSelection() { ... }` declares a public (no visibility keyword)
    //           instance method, no params, `Unit` (void) return, block body.
    // Why:      Drop the current-track selection so no track is current and there is no
    //           playback scope until the user taps one. `PlayerController.openLibrary` calls
    //           this after `setTracks` so a freshly opened library auto-selects NOTHING.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // clearSelection(): void { this.rebuildScopeOrder(null); }
    // ```
    /**
     * Defines clear selection behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun clearSelection() {
        // What:     `rebuildScopeOrder(null)` calls the private helper with a `null` anchor,
        //           which (see `rebuildScopeOrder`) empties `order` and nulls `pos`. `null` is
        //           the absent value of the `Int?` parameter.
        // Why:      Reuse the single method that owns the scope/cursor invariant instead of
        //           poking the fields here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.rebuildScopeOrder(null);
        // ```
        rebuildScopeOrder(null)
    }

    // What:     `fun setPlaybackMode(newMode: PlaybackMode)` accepts one enum value.
    // Why:      A mode change rebuilds navigation around the retained track without
    //           loading audio or changing playback position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setPlaybackMode(newMode: PlaybackMode): void { /* retain current, rebuild */ }
    // ```
    /** Selects one playback behavior while retaining the current track. */
    fun setPlaybackMode(newMode: PlaybackMode) {
        if (newMode == mode) return
        /** Track retained across the navigation rebuild, or null. */
        val current: Int? = currentIndex()
        mode = newMode
        rebuildScopeOrder(current)
    }

    // What:     `fun setPageScope(indices: List<Int>)` accepts the displayed page's
    //           load-order indices.
    // Why:      Page-based modes must follow tab changes even when the playing track
    //           remains on a different page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setPageScope(indices: readonly number[]): void { /* retain current, rebuild */ }
    // ```
    /** Updates the displayed-page scope without moving the current track. */
    fun setPageScope(indices: List<Int>) {
        /** Valid page indices in display order. */
        val valid: List<Int> = indices.filter { index -> index in tracks.indices }
        if (valid == pageScope) return
        pageScope = valid
        if (mode == PlaybackMode.SHUFFLE_PAGE || mode == PlaybackMode.SHUFFLE_ALL) return
        /** Track retained while future transport adopts the new page. */
        val current: Int? = currentIndex()
        rebuildScopeOrder(current)
    }

    // What:     `fun playIndex(track: Int): Int? { ... }` declares an instance method
    //           taking one non-null `Int` parameter `track` (a load-order index) and
    //           returning a nullable `Int?`, block body.
    // Why:      Select a specific track as current, switching the playback scope when
    //           the track is on another page; the user clicked a row in the queue
    //           list. Returns the now-current track index, or `null` for an
    //           out-of-range click (which moves nothing).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playIndex(track: number): number | null {
    //   if (track >= this.tracks.length) return null;
    //   if (this.shuffle === PlaybackMode.IN_ORDER) {
    //     const position = this.order.indexOf(track);
    //     if (position >= 0) this.pos = position;
    //     else this.rebuildScopeOrder(track);
    //   } else {
    //     this.rebuildScopeOrder(track);
    //   }
    //   return track;
    // }
    // ```
    /**
     * Defines play index behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun playIndex(track: Int): Int? {
        // What:     `if (track >= tracks.size) return null` is an early-return bounds
        //           check. `tracks.size` is the element count; `>=` compares the
        //           clicked index against it. `return null` exits with the `null`
        //           variant of the `Int?` return type.
        // Why:      Ignore an out-of-range click (clicking past the last track moves
        //           nothing).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (track >= this.tracks.length) return null;
        // ```
        if (track >= tracks.size) return null
        if (track !in pageScope) {
            /** Pages rebuilt from the same display paths shown by the UI. */
            val trackPages: List<Page> = pages()
            /** Page containing the selected track, or null for a defensive fallback. */
            val trackPage: Int? = pageOf(track)
            pageScope = if (trackPage == null) {
                tracks.indices.toList()
            } else {
                trackPages[trackPage].entries.map { entry -> entry.index }
            }
        }
        // What:     The condition distinguishes both sequential modes from shuffle.
        //           the shuffle mode. `==` compares the `shuffle` field against the
        //           `OFF` enum constant (enum value equality, like TS `===`).
        // Why:      Just-in-time shuffle keeps NO reusable precomputed order, so a
        //           jump under shuffle cannot "find the track in the existing order"
        //           the way the deterministic `OFF` order can; it must restart the
        //           cycle at the chosen track. So `OFF` reuses its scope order when it
        //           can, while shuffle always rebuilds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle === PlaybackMode.IN_ORDER) { /* find-or-rebuild */ } else { this.rebuildScopeOrder(track); }
        // ```
        if (mode == PlaybackMode.IN_ORDER || mode == PlaybackMode.REPEAT) {
            // What:     `val position: Int = order.indexOf(track)` declares a read-only
            //           local `position`. `order.indexOf(track)` is a stdlib `List`
            //           method returning the FIRST index at which `track` appears in
            //           `order`, or `-1` if absent.
            // Why:      Under `OFF` the scope order is the deterministic page sequence,
            //           so if the clicked track is already in it we can keep that order
            //           and only move the cursor.
            // Gotcha:   `.indexOf` returns `-1` for "not found" (not `null`); the test
            //           below is `position >= 0`, not a null check.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const position: number = this.order.indexOf(track);
            // ```
            /**
             * Defines position value for this music-player component; the TypeScript-oriented notes above
             * explain its source and use.
             */
            val position: Int = order.indexOf(track)
            // What:     `if (position >= 0) { pos = position } else { rebuildScopeOrder(track) }`.
            //           The condition means "the track is already in the current scope
            //           order"; the `then` moves the cursor, the `else` rebuilds the
            //           scope around the clicked track (it is on another page).
            // Why:      Stay in the same page when possible; switch pages otherwise.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (position >= 0) this.pos = position; else this.rebuildScopeOrder(track);
            // ```
            if (position >= 0) {
                pos = position
            } else {
                rebuildScopeOrder(track)
            }
        } else {
            // What:     `rebuildScopeOrder(track)` rebuilds the scope around the clicked
            //           track. Under shuffle (`rebuildScopeOrder`'s shuffle branch) this
            //           resets `order` to `[track]`, `pos` to 0, and `cycleStart` to 0,
            //           starting a fresh just-in-time cycle at that track.
            // Why:      A deliberate jump under shuffle restarts the without-replacement
            //           cycle from the chosen track (the accepted cycle reset; see
            //           `doc/decision/music-player-jit-shuffle.md`).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.rebuildScopeOrder(track);
            // ```
            rebuildScopeOrder(track)
        }
        // What:     `return track` returns the clicked load-order index (explicit
        //           `return`, the non-null `Int` wrapped as the `Int?` result).
        // Why:      Tell the caller which track is now current so it can load that
        //           index.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return track;
        // ```
        return track
    }

    // What:     `fun advance(natural: Boolean): Int? { ... }` declares an instance
    //           method taking one `Boolean` parameter `natural` and returning a
    //           nullable `Int?`, block body. `natural` is `true` when a track ended
    //           on its own, `false` when the user pressed Next.
    // Why:      Move to the next track within the scope. A track that ends naturally
    //           under Repeat replays itself, and one that ends naturally under In order
    //           at the scope's last track has nothing to follow it (D2: In order plays
    //           to the end of the folder and stops). A manual Next wraps to the scope's
    //           start in every sequential mode. Returns the load-order index of the
    //           track to play next, or `null` when the queue is empty or In order has
    //           reached its end (the cursor stays on the last track).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // advance(natural: boolean): number | null {
    //   if (this.pos === null) return null;
    //   const position = this.pos;
    //   if (natural && this.repeatTrackFlag) return this.order[position];
    //   if (this.shuffle !== PlaybackMode.IN_ORDER) {
    //     if (position + 1 < this.order.length) { this.pos = position + 1; return this.order[position + 1]; }
    //     const pick = this.pickNextShuffle(this.order[position]);
    //     this.order = [...this.order, pick]; this.pos = this.order.length - 1; return pick;
    //   }
    //   const next = position + 1;
    //   if (next < this.order.length) { this.pos = next; return this.order[next]; }
    //   if (natural && this.mode === PlaybackMode.IN_ORDER) return null; // D2: stop at the end
    //   this.pos = 0; return this.order[0];
    // }
    // ```
    /**
     * Defines advance behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun advance(natural: Boolean): Int? {
        // What:     `val position: Int? = pos` snapshots the nullable cursor.
        // Why:      The expression below returns null when the queue has no cursor.
        /** Cursor position before advancing, or null when the queue is empty. */
        val position: Int? = pos
        return if (position == null) {
            null
        } else if (natural && mode == PlaybackMode.REPEAT) {
            order[position]
        } else if (mode == PlaybackMode.SHUFFLE_PAGE || mode == PlaybackMode.SHUFFLE_ALL) {
            if (position + 1 < order.size) {
                pos = position + 1
                order[position + 1]
            } else {
                /** Load-order index currently at the end of shuffle history. */
                val current: Int = order[position]
                /** Newly drawn load-order index for the next shuffle step. */
                val pick: Int = pickNextShuffle(current)
                order = order + pick
                pos = order.size - 1
                pick
            }
        } else if (sequentialStart > 0 && position < sequentialStart) {
            order = order.drop(sequentialStart)
            sequentialStart = 0
            pos = 0
            order[0]
        } else {
            /** Sequential position after the current one. */
            val next: Int = position + 1
            if (next < order.size) {
                pos = next
                order[next]
            } else if (natural && mode == PlaybackMode.IN_ORDER) {
                null
            } else {
                pos = sequentialStart
                order[sequentialStart]
            }
        }
    }

    // What:     `fun rewindScope(): Int?` puts the cursor on the scope's first track and returns
    //           that track's load-order index, or `null` for an empty scope.
    // Why:      After In order has played to the end and stopped, the next Play starts the
    //           folder again from its first track, as a disc player does; a cursor left on the
    //           last track would end again at once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rewindScope(): number | null {
    //   if (this.order.length === 0) return null;
    //   this.pos = Math.min(this.sequentialStart, this.order.length - 1);
    //   return this.order[this.pos];
    // }
    // ```
    /** Moves the cursor to the first track of the scope and returns it, or null when nothing is queued. */
    fun rewindScope(): Int? {
        if (order.isEmpty()) {
            return null
        }
        /** Position of the first track the sequential walk repeats. */
        val first: Int = minOf(sequentialStart, order.size - 1)
        pos = first
        return order[first]
    }

    // What:     `fun moveCursorTo(scopeIndex: Int): Int? { ... }` declares an instance
    //           method taking one non-null `Int` parameter `scopeIndex` (a timeline
    //           window index) and returning a nullable `Int?`, block body.
    // Why:      Move the cursor straight to scope position `scopeIndex` WITHOUT
    //           changing the scope; used when a MediaSession seek (Next/Previous from
    //           the notification, or a jump to a queue item) resolves to an index the
    //           framework already computed against the reported order. Out-of-range
    //           indices move nothing, matching the framework's `C.INDEX_UNSET` no-op.
    //           (No `queue.rs` twin: this is MediaSession-specific to the Android
    //           port.)
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // moveCursorTo(scopeIndex: number): number | null {
    //   if (scopeIndex < 0 || scopeIndex >= this.order.length) return null;
    //   this.pos = scopeIndex;
    //   return this.order[scopeIndex];
    // }
    // ```
    /**
     * Defines move cursor to behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    fun moveCursorTo(scopeIndex: Int): Int? {
        // What:     `if (scopeIndex < 0 || scopeIndex >= order.size) { return null }`
        //           is a bounds-check guard. `||` is logical OR; the condition is true
        //           when `scopeIndex` is negative OR past the last scope slot.
        //           `return null` exits with the `null` variant.
        // Why:      An out-of-range target moves nothing (the `C.INDEX_UNSET` no-op
        //           the framework uses).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (scopeIndex < 0 || scopeIndex >= this.order.length) return null;
        // ```
        if (scopeIndex < 0 || scopeIndex >= order.size) {
            return null
        }
        // What:     `pos = scopeIndex` assigns the (now validated, in-range) index
        //           into the `Int?` cursor field.
        // Why:      Move the cursor directly to the framework-chosen position without
        //           recomputing the scope.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pos = scopeIndex;
        // ```
        pos = scopeIndex
        // What:     `return order[scopeIndex]` indexes `order` at the new position and
        //           returns that load-order index (the `Int?` result).
        // Why:      Tell the caller which track is now current.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.order[scopeIndex];
        // ```
        return order[scopeIndex]
    }

    // What:     `fun prev(): Int? { ... }` declares an instance method taking no
    //           parameters and returning a nullable `Int?`, block body.
    // Why:      Move to the previous track within the scope, wrapping to the scope's
    //           end at the start; the user pressed Previous. Returns the load-order
    //           index of the previous track, or `null` when the queue is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // prev(): number | null {
    //   if (this.pos === null) return null;
    //   const position = this.pos;
    //   if (this.shuffle !== PlaybackMode.IN_ORDER) {
    //     if (position > 0) { this.pos = position - 1; return this.order[position - 1]; }
    //     return this.order[position]; // at history start: stay put
    //   }
    //   if (position > 0) { this.pos = position - 1; return this.order[position - 1]; }
    //   const last = this.order.length - 1;
    //   this.pos = last; return this.order[last];
    // }
    // ```
    /**
     * Defines prev behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun prev(): Int? {
        // What:     `val position: Int? = pos` snapshots the nullable cursor.
        // Why:      The expression below returns null when the queue has no cursor.
        /** Cursor position before moving backward, or null when the queue is empty. */
        val position: Int? = pos
        return if (position == null) {
            null
        } else if (mode == PlaybackMode.SHUFFLE_PAGE || mode == PlaybackMode.SHUFFLE_ALL) {
            if (position > 0) {
                pos = position - 1
                order[position - 1]
            } else {
                order[position]
            }
        } else if (sequentialStart > 0 && position < sequentialStart) {
            order = order.drop(sequentialStart)
            sequentialStart = 0
            pos = order.size - 1
            order[order.size - 1]
        } else if (position > sequentialStart) {
            pos = position - 1
            order[position - 1]
        } else {
            /** Last position in the current non-shuffle scope. */
            val last: Int = order.size - 1
            pos = last
            order[last]
        }
    }

    //endregion

    //region Scope helpers
    // What:     `private fun scopeIndices(anchor: Int): List<Int> { ... }` declares a
    //           PRIVATE instance method (visible only inside this class) taking one
    //           non-null `Int` parameter `anchor` (a load-order index) and returning
    //           a read-only `List<Int>`, block body.
    // Why:      Compute the load-order indices that make up the playback scope around
    //           the `anchor` track, in ascending load order: the whole queue for
    //           `PlaybackMode.SHUFFLE_ALL`, otherwise the anchor's page. Falls back to the
    //           whole queue when the anchor belongs to no page (an empty/invalid
    //           anchor), never producing an empty scope for a real track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private scopeIndices(anchor: number): number[] {
    //   if (this.shuffle === PlaybackMode.SHUFFLE_ALL) {
    //     return [...this.tracks.keys()];
    //   }
    //   const names = this.displayPaths();
    //   const pages = paginate(names);
    //   const page = pageOfIndex(pages, anchor);
    //   return page !== null
    //     ? pages[page].entries.map((e) => e.index)
    //     : [...this.tracks.keys()];
    // }
    // ```
    /**
     * Defines scope indices behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun scopeIndices(anchor: Int): List<Int> {
        // What:     `if (mode == PlaybackMode.SHUFFLE_ALL) { ... }` is a control-flow check.
        //           `==` compares the `shuffle` field against the `PlaybackMode.SHUFFLE_ALL`
        //           enum constant (enum value equality, like TS `===`).
        // Why:      `ALL` ignores pages entirely: the scope is every track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.shuffle === PlaybackMode.SHUFFLE_ALL) return [...this.tracks.keys()];
        // ```
        if (mode == PlaybackMode.SHUFFLE_ALL) {
            // What:     `return tracks.indices.toList()` builds and returns the list of
            //           every load-order index.
            //           - `tracks.indices` is a stdlib property giving the `IntRange`
            //             `0 until tracks.size` (all valid index positions).
            //           - `.toList()` is a type-CONVERSION call: it materialises that
            //             range into a concrete `List<Int>` (the declared return type).
            // Why:      Every load-order index, ascending — the whole-queue scope.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return [...Array(this.tracks.length).keys()];
            // ```
            return tracks.indices.toList()
        }
        if (pageScope.isNotEmpty()) {
            return pageScope
        }
        // What:     `val names: List<String> = displayPaths()` declares a read-only
        //           `List<String>` local `names`, the relative display strings (one
        //           per track, in load order) from `displayPaths()`.
        // Why:      Pagination groups these display strings into pages.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const names: string[] = this.displayPaths();
        // ```
        /**
         * Defines names value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        // What:     `val pages: List<Page> = paginate(names)` declares a read-only
        //           `List<Page>` local `pages`. `paginate(names)` calls the sibling
        //           package-level function (from `Pagination.kt`) that groups the
        //           display strings into `Page` objects (each `Page` has a label and
        //           a list of entries). `Page` is the sibling data class.
        // Why:      We need the set of indices sharing the anchor's page. Using the
        //           SAME `paginate` the UI tab bar uses means the playback scope and
        //           the visible page can never drift apart.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages: Page[] = paginate(names);
        // ```
        /**
         * Defines pages value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val pages: List<Page> = pages()
        // What:     `val page: Int? = pageOfIndex(pages, anchor)` declares a read-only
        //           NULLABLE `Int?` local `page`. `pageOfIndex(pages, anchor)` is the
        //           sibling function (from `Pagination.kt`) that returns the POSITION
        //           of the page holding `anchor`, or `null` when no page holds it.
        // Why:      That page IS the confined scope; `null` signals "anchor not on any
        //           page" (only for an empty/invalid anchor).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const page: number | null = pageOfIndex(pages, anchor);
        // ```
        /**
         * Defines page value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val page: Int? = pageOf(anchor)
        // What:     `return if (page != null) { ... } else { ... }` returns the value
        //           of an IF/ELSE used as an EXPRESSION (it evaluates to one of the
        //           two branch values, like a TS ternary). The condition
        //           `page != null` is a null check that ALSO smart-casts `page` to a
        //           non-null `Int` inside the `then` branch.
        // Why:      If the anchor is on a page, that page's indices are the scope;
        //           otherwise fall back to the whole queue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return page !== null
        //   ? pages[page].entries.map((e) => e.index)
        //   : [...Array(this.tracks.length).keys()];
        // ```
        return if (page != null) {
            // What:     `pages[page].entries.map { it.index }` is the `then`-branch
            //           value.
            //           - `pages[page]` indexes the pages list at the (smart-cast
            //             non-null) `page` position, giving one `Page`.
            //           - `.entries` is that page's list of `PageEntry` (each entry
            //             pairs a load-order `index` with a display `name`).
            //           - `.map { it.index }` runs a TRAILING LAMBDA over each entry,
            //             pulling out its `index` field. `it` is the implicit lambda
            //             parameter (one `PageEntry`). The result is a `List<Int>` of
            //             load-order indices, already in ascending load order because
            //             pagination preserves order. This is the branch's value and
            //             thus part of the returned expression.
            // Why:      The page's track indices form the confined scope.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // pages[page].entries.map((entry) => entry.index)
            // ```
            pages[page].entries.map { it.index }
        } else {
            // What:     `tracks.indices.toList()` is the `else`-branch value: the whole
            //           queue's load-order indices, same construction as the `ALL`
            //           branch above (`tracks.indices` is the index range; `.toList()`
            //           materialises it into a `List<Int>`).
            // Why:      Defensive fallback: never produce an empty scope for a real
            //           track when the anchor matched no page.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // [...Array(this.tracks.length).keys()]
            // ```
            tracks.indices.toList()
        }
    }

    // What:     `private fun pickNextShuffle(current: Int): Int { ... }` declares a
    //           private instance method taking the CURRENT track's load-order index
    //           and returning the next just-in-time shuffle pick (also a load-order
    //           index), block body.
    // Why:      The just-in-time draw, replacing the old precomputed Fisher-Yates
    //           permutation. It returns one random scope track NOT yet played this
    //           cycle (without replacement); when the cycle has exhausted the scope it
    //           starts a fresh cycle (advancing `cycleStart`) that avoids an immediate
    //           repeat of `current`. Mirrors desktop `queue.rs` `pick_next_shuffle`.
    //           See `doc/decision/music-player-jit-shuffle.md`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private pickNextShuffle(current: number): number {
    //   const scope = this.scopeIndices(current);
    //   const played = new Set(this.order.slice(this.cycleStart));
    //   let remaining = scope.filter((i) => !played.has(i));
    //   if (remaining.length === 0) {
    //     this.cycleStart = this.order.length;
    //     remaining = scope.filter((i) => i !== current);
    //     if (remaining.length === 0) remaining = scope;
    //   }
    //   return remaining[this.rng.nextInt(remaining.length)];
    // }
    // ```
    /**
     * Defines pick next shuffle behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun pickNextShuffle(current: Int): Int {
        // What:     `val scope: List<Int> = scopeIndices(current)` declares a read-only
        //           `List<Int>` of the load-order indices eligible this cycle (the
        //           anchor's page for `WITHIN_PAGE`, the whole queue for `ALL`).
        // Why:      The pool the pick is drawn from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const scope = this.scopeIndices(current);
        // ```
        /**
         * Defines scope value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val scope: List<Int> = scopeIndices(current)
        // What:     `val played: Set<Int> = order.subList(cycleStart, order.size).toHashSet()`
        //           declares a read-only `Set<Int>` of the indices already played THIS
        //           cycle.
        //           - `order.subList(cycleStart, order.size)` is a VIEW of `order` from
        //             `cycleStart` (inclusive) to the end (exclusive of `order.size`,
        //             which is one past the last). This is the current cycle's history.
        //           - `.toHashSet()` copies that view into a hash `Set<Int>` for O(1)
        //             membership tests below.
        // Why:      "Without replacement" means excluding everything already played this
        //           cycle; a set makes the exclusion test cheap.
        // Gotcha:   `subList(from, to)` is a half-open range like TS `slice(from, to)`;
        //           `to = order.size` takes through the last element.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const played = new Set(this.order.slice(this.cycleStart));
        // ```
        /**
         * Defines played value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val played: Set<Int> = order.subList(cycleStart, order.size).toHashSet()
        // What:     `var remaining: List<Int> = scope.filter { it !in played }` declares
        //           a REASSIGNABLE (`var`) `List<Int>` of scope tracks not yet played
        //           this cycle.
        //           - `.filter { ... }` keeps elements for which the trailing lambda is
        //             true; `it` is each scope index.
        //           - `it !in played` is the negated MEMBERSHIP operator (`!in` is
        //             `!played.contains(it)`).
        //           `var` (not `val`) because the empty-cycle branch below reassigns it.
        // Why:      The eligible picks: scope minus this cycle's history.
        // Gotcha:   Kotlin's `in`/`!in` is MEMBERSHIP (`.contains`), NOT JS `in`
        //           (property-key); translate to `.has(...)`/`!.has(...)`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let remaining = scope.filter((i) => !played.has(i));
        // ```
        /**
         * Defines remaining value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        var remaining: List<Int> = scope.filter { it !in played }
        // What:     `if (remaining.isEmpty()) { ... }` runs when every scope track has
        //           been played this cycle (the cycle is exhausted).
        // Why:      Start a FRESH cycle: mark the boundary and reseed the eligible pool.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (remaining.length === 0) { /* start a fresh cycle */ }
        // ```
        if (remaining.isEmpty()) {
            // What:     `cycleStart = order.size` advances the cycle boundary to the
            //           current end of history, so the new cycle's "played" set starts
            //           empty (the next `subList(cycleStart, order.size)` is empty until
            //           picks are appended).
            // Why:      Begin counting a new without-replacement cycle from here.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.cycleStart = this.order.length;
            // ```
            cycleStart = order.size
            // What:     `remaining = scope.filter { it != current }` reseeds the pool to
            //           the whole scope EXCEPT the current track, so the fresh cycle does
            //           not immediately replay the track that just ended.
            // Why:      Avoid a jarring back-to-back repeat across the cycle boundary.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // remaining = scope.filter((i) => i !== current);
            // ```
            remaining = scope.filter { it != current }
            // What:     `if (remaining.isEmpty()) { remaining = scope }` handles a
            //           single-track scope, where excluding `current` leaves nothing; in
            //           that case the only option is to replay `current`.
            // Why:      A one-track scope must still yield a pick.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (remaining.length === 0) remaining = scope;
            // ```
            if (remaining.isEmpty()) {
                remaining = scope
            }
        }
        // What:     `return remaining[rng.nextInt(remaining.size)]` draws a uniformly
        //           random element of `remaining`. `rng.nextInt(remaining.size)` returns
        //           an `Int` in `0 until remaining.size`; indexing yields the chosen
        //           load-order index, which is returned.
        // Why:      The actual random pick from the eligible pool.
        // Gotcha:   This is the KOTLIN seeded RNG (`kotlin.random.Random.nextInt`), NOT
        //           the desktop's xorshift64; only within-Kotlin determinism is shared.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return remaining[this.rng.nextInt(remaining.length)];
        // ```
        return remaining[rng.nextInt(remaining.size)]
    }

    // What:     `private fun rebuildScopeOrder(anchor: Int?) { ... }` declares a
    //           private instance method taking one NULLABLE `Int?` parameter `anchor`
    //           and returning `Unit` (void), block body.
    // Why:      Recompute the scope `order` and cursor `pos` so the `anchor` track
    //           stays current; called whenever the scope might change (`setTracks`,
    //           `setPlaybackMode`, `playIndex` to another page). A `null` anchor defaults
    //           to the first track; a stale index past the end is clamped into range;
    //           an empty queue clears the order and cursor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private rebuildScopeOrder(anchor: number | null): void {
    //   if (this.tracks.length === 0) { this.order = []; this.pos = null; return; }
    //   if (anchor === null) { this.order = []; this.pos = null; return; }
    //   const clamped = Math.min(anchor, this.tracks.length - 1);
    //   if (this.shuffle === PlaybackMode.IN_ORDER) {
    //     const scope = this.scopeIndices(clamped);
    //     const found = scope.indexOf(clamped);
    //     this.order = scope;
    //     this.pos = found < 0 ? 0 : found;
    //   } else {
    //     this.order = [clamped]; this.pos = 0; this.cycleStart = 0; // just-in-time: history grows in advance()
    //   }
    // }
    // ```
    /**
     * Defines rebuild scope order behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun rebuildScopeOrder(anchor: Int?) {
        // What:     `if (tracks.isEmpty()) { ... }` is a control-flow check using the
        //           `List.isEmpty()` predicate (true when there are zero tracks).
        // Why:      An empty queue has no order and no cursor; guard the index math
        //           below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.tracks.length === 0) { this.order = []; this.pos = null; return; }
        // ```
        if (tracks.isEmpty()) {
            sequentialStart = 0
            // What:     `order = emptyList()` assigns the shared zero-length read-only
            //           list (see the `emptyList()` note) to the `order` field.
            // Why:      No tracks means no playback order.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order = [];
            // ```
            order = emptyList()
            // What:     `pos = null` clears the cursor field (assigning the `null`
            //           variant to the `Int?` field).
            // Why:      Nothing is selected in an empty queue.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pos = null;
            // ```
            pos = null
            // What:     `return` exits the method early (returning `Unit`/void). Bare
            //           `return` with no value, legal because the method returns `Unit`.
            // Why:      The empty-queue case is fully handled; skip the rest.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return
        }
        // What:     `if (anchor == null) { order = emptyList(); pos = null; return }`. A `null`
        //           anchor means "NO current track": assign the shared zero-length read-only
        //           list to `order`, null the `pos` cursor, and `return` early. The `== null`
        //           check also SMART-CASTS `anchor` to a non-null `Int` for the lines below.
        // Why:      `setTracks` anchors `0`, but `clearSelection` (and toggling shuffle while
        //           nothing is selected) passes `null` to DESELECT, so a freshly opened library
        //           highlights nothing until the user taps a track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (anchor === null) { this.order = []; this.pos = null; return; }
        // ```
        if (anchor == null) {
            order = emptyList()
            pos = null
            sequentialStart = 0
            return
        }
        // What:     `val clamped: Int = minOf(anchor, tracks.size - 1)` declares a read-only
        //           (`val`) `Int` local `clamped`. `anchor` is now smart-cast to a non-null
        //           `Int` (the null case returned above); `minOf(x, y)` returns the smaller of
        //           two values, CLAMPING the anchor to at most `tracks.size - 1` (the last
        //           valid index).
        // Why:      Defensive: a stale index must not point past the end of the queue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const clamped: number = Math.min(anchor, this.tracks.length - 1);
        // ```
        /**
         * Defines clamped value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val clamped: Int = minOf(anchor, tracks.size - 1)
        // What:     The condition sends Repeat and In order through sequential rebuild.
        //           rebuild on shuffle mode. `==` is enum value equality.
        // Why:      `OFF` builds the FULL sequential page scope up front (it is
        //           deterministic and needs no play history). The shuffle modes build no
        //           permutation: `order` starts as just the anchor and grows as
        //           just-in-time picks are drawn (see `advance`), so the rebuild only
        //           seeds the single anchor and resets the cycle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // sequential ? rebuildFullScope(anchor) : resetShuffleHistory(anchor);
        // ```
        if (mode == PlaybackMode.IN_ORDER || mode == PlaybackMode.REPEAT) {
            // What:     `val scope: List<Int> = scopeIndices(clamped)` is the page's
            //           indices in ascending load order (the whole sequential scope).
            // Why:      `OFF` plays the page in load order, so the scope IS the order.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const scope = this.scopeIndices(clamped);
            // ```
            /**
             * Defines scope value for this music-player component; the TypeScript-oriented notes above explain
             * its source and use.
             */
            val scope: List<Int> = scopeIndices(clamped)
            // What:     `val found: Int = scope.indexOf(clamped)` locates the anchor's
            //           position within the scope, or `-1` if absent.
            // Why:      The cursor must point at the anchor after the rebuild.
            // Gotcha:   `.indexOf` returns `-1` (not `null`) when not found; the test
            //           below is `found < 0`, not a null check.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const found = scope.indexOf(clamped);
            // ```
            /**
             * Defines found value for this music-player component; the TypeScript-oriented notes above explain
             * its source and use.
             */
            val found: Int = scope.indexOf(clamped)
            if (found < 0) {
                // Retain an out-of-page current track as a one-step prelude. Manual
                // transport removes it before looping the displayed page.
                order = listOf(clamped) + scope
                pos = 0
                sequentialStart = 1
            } else {
                order = scope
                pos = found
                sequentialStart = 0
            }
        } else {
            // What:     `order = listOf(clamped)` seeds the play history with JUST the
            //           anchor. `listOf(x)` builds a one-element read-only `List<Int>`.
            // Why:      Just-in-time shuffle does not precompute a permutation; the
            //           history starts at the anchor and grows via `advance`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.order = [clamped];
            // ```
            order = listOf(clamped)
            sequentialStart = 0
            // What:     `pos = 0` points the cursor at that single seeded entry.
            // Why:      The anchor is the current track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pos = 0;
            // ```
            pos = 0
            // What:     `cycleStart = 0` resets the cycle boundary to the start of the
            //           freshly seeded history, so the new cycle's "played" set is just
            //           the anchor.
            // Why:      A rebuild (open, restore, shuffle toggle, jump) begins a new
            //           without-replacement cycle at the anchor (the accepted cycle
            //           reset; see `doc/decision/music-player-jit-shuffle.md`).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.cycleStart = 0;
            // ```
            cycleStart = 0
        }
    }
    //endregion
}
