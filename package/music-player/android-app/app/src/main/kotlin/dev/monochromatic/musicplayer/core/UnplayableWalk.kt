// How a track load was started, and what the player remembers while it walks past files it cannot open (D122, D123).

// What:     `package ...core` places the walk rules beside the other platform-independent logic.
// Why:      Everything here is plain code with no Android type in it, so JVM tests reach every branch without a device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class LoadOrigin` names the four ways the controller starts a track load.
// Why:      A track the person chose answers a failure with the in-app "Could not play" message (D83, D123). A track
//           the player moved to by itself answers a failure by walking past it and reporting it (D122, D128), so the
//           controller keeps the origin of the load in flight and reads it when the failure arrives.
//
// In TS you'd write (pseudocode):
// ```ts
// type LoadOrigin = "tap" | "forward" | "backward" | "naturalEnd";
// ```
/** How the load now in flight was started, which decides what its failure does. */
enum class LoadOrigin {
    /** The person chose this track: a tap, Enter, a Search result, a menu Play or the Play button on a current row. */
    TAP,

    /** A move forward the person named no row for: Next, Play with nothing selected, the restore, the readied start. */
    FORWARD,

    /** A move back to the previous track. */
    BACKWARD,

    /** The end of a track that played to its end. */
    NATURAL_END,
}

// What:     `data class UnplayableReport(fileNames, timedOut)` is what the player tells the host about a walk.
// Why:      The host posts one system notification from it; the controller cannot post one because it has no
//           Android types. The list grows while the walk goes on, and the host replaces the earlier notification.
//
// In TS you'd write (pseudocode):
// ```ts
// type UnplayableReport = { fileNames: string[]; timedOut: boolean };
// ```
/** The files one walk has passed so far, by bare file name, and whether the walk ended on a provider that stalled. */
data class UnplayableReport(
    /** Bare file names in the order the walk met them, each once. */
    val fileNames: List<String>,
    /** Whether the last open timed out, which says the music library was slow, not that the file is broken. */
    val timedOut: Boolean,
)

// What:     `class UnplayableWalk` remembers which files one run of failed loads has met.
// Why:      The player tries the next track when a moved-to track cannot be opened. A library of files that all fail
//           would walk for ever, so the count of attempts is capped by the caller's limit, and the files are kept
//           once each in the order met so the report names the first of them.
//
// In TS you'd write (pseudocode):
// ```ts
// class UnplayableWalk {
//   private failed = new Set<string>(); private attempts = 0;
//   reset(): void { ... }
//   record(uri: string, limit: number): boolean { ... }
//   failedUris(): string[] { ... }
// }
// ```
/** The failed loads of one walk past unreadable files. */
class UnplayableWalk {
    // What:     `private val failed: LinkedHashSet<String>` holds the URIs met, once each, in order.
    // Why:      A repeat-one track or a shuffle can meet the same file twice; the report names it once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private failed = new Set<string>();
    // ```
    /** URIs of the files that failed in this walk, in the order met. */
    private val failed: LinkedHashSet<String> = LinkedHashSet()

    // What:     `private var attempts: Int` counts every failed load of this walk.
    // Why:      The cap counts attempts, not distinct files, so a shuffle that keeps drawing one bad file ends too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private attempts = 0;
    // ```
    /** Number of failed loads in this walk, repeats included. */
    private var attempts: Int = 0

    // What:     `fun reset()` forgets the walk.
    // Why:      A load the person or the player starts afresh begins a new walk, and its report replaces the old one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // reset(): void { this.failed.clear(); this.attempts = 0; }
    // ```
    /** Starts a new walk. */
    fun reset() {
        failed.clear()
        attempts = 0
    }

    // What:     `fun record(uri: String, limit: Int): Boolean` notes one failed load and says whether to try another.
    // Why:      After `limit` failures every track of the queue has had one try, so the walk stops there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // record(uri: string, limit: number): boolean {
    //   this.failed.add(uri); this.attempts += 1; return this.attempts < limit;
    // }
    // ```
    /**
     * Notes that the load of [uri] failed.
     *
     * @param uri the track whose load failed
     * @param limit number of failed loads after which the walk gives up, normally the length of the queue
     * @return true while the walk may try another track
     */
    fun record(uri: String, limit: Int): Boolean {
        failed += uri
        attempts += 1
        return attempts < limit
    }

    // What:     `fun failedUris(): List<String>` copies the URIs met so far.
    // Why:      The controller turns them into file names for the report; a copy cannot change under it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // failedUris(): string[] { return [...this.failed]; }
    // ```
    /** The failed URIs in the order met, each once. */
    fun failedUris(): List<String> = failed.toList()
}
