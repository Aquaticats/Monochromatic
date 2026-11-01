// File summary (folding in the old KDoc domain content):
// This file defines two plain "record"-style data containers used by the
// Android music player. They carry a frozen, point-in-time copy of what the
// player is doing right now, so the Android system UI (the notification and
// the lock-screen media controls, driven by a `MediaSession`) can be drawn
// without reaching back into the live player on every animation frame.
//   - `SnapshotItem`  = one track in the current playback scope (one row of
//                       the timeline the system shows).
//   - `PlaybackSnapshot` = the whole frozen view: the ordered track list plus
//                       the transport state (playing/paused, volume, position,
//                       duration). It is read fresh on each `getState()` pull;
//                       position/duration are sampled once here and the
//                       `MediaSession` extrapolates them between pulls, so the
//                       snapshot is NOT rebuilt every frame.
// Both classes are immutable value bags: build one, read it, throw it away.

// What:     `package dev.monochromatic.musicplayer` declares which "folder of
//           names" every type in this file belongs to. `package` is Kotlin's
//           keyword for a namespace; the dotted path
//           `dev.monochromatic.musicplayer` is that namespace's full name.
//           Other files in the same package can refer to `SnapshotItem` and
//           `PlaybackSnapshot` without importing them.
// Why:      We need it so these classes share an identity with the rest of the
//           app's code (`PlayerController`, `BrainPlayer`, `PlaybackService`,
//           …) and so the build tool knows where this file's types live.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — the folder path dev/monochromatic/musicplayer
// // IS the namespace; TS files just export and import by relative path.
// ```
package dev.monochromatic.musicplayer

// What:     `data class SnapshotItem( ... )` declares a class named
//           `SnapshotItem`. The `data` modifier tells Kotlin "this is a value
//           record": the compiler auto-generates structural `equals`/`hashCode`
//           (two items with the same field values count as equal), a readable
//           `toString`, and a `copy(...)` method. The `( ... )` right after the
//           name is the PRIMARY CONSTRUCTOR: the parameters listed inside it
//           both define the constructor arguments AND, because each is marked
//           `val`, become read-only public fields of the class.
// Why:      We need one of these per track so a `PlaybackSnapshot` can hold an
//           ordered list of them; this is the unit the system timeline shows.
// Gotcha:   "value equality" is the trap for a TS reader: two distinct
//           `SnapshotItem` objects with identical fields are `==`/`equals` in
//           Kotlin. In TS `{a:1} !== {a:1}`. Don't assume reference identity.
//
// In TS you'd write (pseudocode):
// ```ts
// type SnapshotItem = {
//   uri: string;
//   title: string;
//   loadIndex: number;
// };
// // Note: TS objects compare by reference; Kotlin `data class` compares by value.
// ```
/**
 * Defines snapshot item type for this music-player component; the TypeScript-oriented notes above explain its
 * role.
 */
data class SnapshotItem(
    // What:     `val uri: String`. `val` means "read-only binding" (assign
    //           once, never reassign). `uri` is the field name. `String` is
    //           Kotlin's heap-allocated, immutable UTF-16 text type. Sibling
    //           types the reader might expect: `CharSequence` (a read-only text
    //           interface `String` implements) and `StringBuilder` (a mutable
    //           text buffer). We picked plain `String`, the simplest, because
    //           the value never changes and is just handed around.
    // Why:      Holds the `content://` or `file://` URI the playback engine
    //           opens to actually read this track's bytes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly uri: string;
    // ```
    val uri: String,
    // What:     `val title: String`. Same as above: a read-only (`val`) field
    //           named `title` of immutable text type `String` (not the
    //           `CharSequence` interface, not the mutable `StringBuilder`).
    // Why:      Holds the folder-relative display path shown as the track title
    //           on the Android notification and lock screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly title: string;
    // ```
    val title: String,
    // What:     `val loadIndex: Int`. A read-only (`val`) field named
    //           `loadIndex`. `Int` is Kotlin's 32-bit signed integer (range
    //           roughly ±2.1 billion). Siblings the reader might expect:
    //           `Long` (64-bit signed integer), `Short` (16-bit), `Byte`
    //           (8-bit), and the unsigned variants `UInt`/`ULong`. We picked
    //           `Int` (not `Long`) because a load-order position over a
    //           folder of tracks fits easily in 32 bits, and `Int` is the
    //           default integer the Android/Kotlin APIs expect for indices.
    // Why:      The stable load-order index of this track, so the UI can map the
    //           system's "current media item" back to the row it came from. It is
    //           NOT unique among the items: a shuffle history repeats a track, and
    //           [entryId] is the timeline-window uid.
    // Gotcha:   `Int` is NOT TS's `number`. It is a fixed-width 32-bit signed
    //           integer; arithmetic wraps around on overflow rather than
    //           silently widening, and there is no fractional part.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly loadIndex: number;
    // ```
    val loadIndex: Int,
    // What:     `val entryId: Long` is a read-only 64-bit id.
    // Why:      The timeline-window uid: unique among the items even when the same
    //           track appears twice in a shuffle history (see `playbackEntryIds`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly entryId: bigint;
    // ```
    val entryId: Long,
    // What:     `val artist: String? = null` is the artist tag of the track when it has been read, else null.
    // Why:      The media notification shows an artist line under the title when tags exist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly artist: string | null;
    // ```
    val artist: String? = null,
)

// What:     `data class PlaybackSnapshot( ... )` declares another immutable
//           value record (same `data class` machinery as `SnapshotItem`:
//           auto value-equality, `toString`, `copy`). Its primary constructor
//           `( ... )` lists six `val` fields that together form one frozen
//           view of the player.
// Why:      We need a single object the player can build on demand and hand to
//           the `MediaSession`/`BrainPlayer` projection, so the Android system
//           UI can be drawn from a stable copy instead of querying the live,
//           constantly-changing player.
// Gotcha:   Same value-vs-reference trap as `SnapshotItem`: two snapshots with
//           identical contents are `equals` in Kotlin but `!==` in TS.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlaybackSnapshot = {
//   items: readonly SnapshotItem[];
//   currentIndex: number | null;
//   playWhenReady: boolean;
//   volume: number;
//   durationMs: number;
//   positionMs: number;
// };
// ```
/**
 * Defines playback snapshot type for this music-player component; the TypeScript-oriented notes above explain
 * its role.
 */
data class PlaybackSnapshot(
    // What:     `val items: List<SnapshotItem>`. A read-only (`val`) field
    //           named `items`. `List<SnapshotItem>` is Kotlin's read-only list
    //           interface holding elements of type `SnapshotItem`; the
    //           `<SnapshotItem>` part is a generic type argument (it says "a
    //           list OF SnapshotItems"). Siblings the reader might expect:
    //           `MutableList<SnapshotItem>` (a list you can add to / remove
    //           from) and `Array<SnapshotItem>` (a fixed-size primitive array).
    //           We picked the read-only `List` because a snapshot is frozen:
    //           nobody should mutate the track order after it is built.
    // Why:      Holds the current scope's tracks in playback (timeline-window)
    //           order, which is the row list the system UI renders.
    // Gotcha:   Kotlin's `List` is read-only by INTERFACE, but the underlying
    //           object could still be a `MutableList` someone else holds; it is
    //           a "no-write view", not a deep-frozen guarantee. Similar to a TS
    //           `readonly T[]` cast over a real mutable array.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly items: readonly SnapshotItem[];
    // ```
    val items: List<SnapshotItem>,
    // What:     `val currentIndex: Int?`. A read-only (`val`) field. The type
    //           is `Int?` — the trailing `?` makes it a NULLABLE `Int`, meaning
    //           the value is either a 32-bit signed integer or the special
    //           value `null`. Without the `?`, a Kotlin `Int` can never be
    //           null. Sibling to `Int` here is the non-nullable `Int` (and the
    //           wider `Long?`); we want the nullable form specifically so
    //           "no current track" is representable.
    // Why:      The position within `items` of the currently playing track, or
    //           `null` when the queue is empty and nothing is selected.
    // Gotcha:   `Int?` is enforced by the compiler: you cannot read it as a
    //           plain `Int` without first handling the null case (with `?.`,
    //           `?:`, or `!!`). TS's `strictNullChecks` is the analogue, but
    //           Kotlin enforces it for every type by default.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly currentIndex: number | null;
    // ```
    val currentIndex: Int?,
    // What:     `val playWhenReady: Boolean`. A read-only (`val`) field named
    //           `playWhenReady` of type `Boolean` (Kotlin's true/false type).
    //           `Boolean` has no confusing siblings here; it is the simplest
    //           possible type.
    // Why:      The play INTENT, not the actual sound: `true` while playing or
    //           buffering, `false` only when paused. The session reports this
    //           so the notification's play/pause icon does not flicker while a
    //           track is still buffering (distinct from whether audio is
    //           literally coming out of the speaker right now).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly playWhenReady: boolean;
    // ```
    val playWhenReady: Boolean,
    // What:     `val volume: Float`. A read-only (`val`) field named `volume`.
    //           `Float` is Kotlin's 32-bit single-precision floating-point
    //           number. The sibling the reader might expect is `Double`, the
    //           64-bit double-precision float (which is what JS/TS `number`
    //           actually is). We picked `Float` (not `Double`) because Android
    //           audio gain APIs take 32-bit floats, so matching the type avoids
    //           a needless widen/narrow at the boundary.
    // Why:      Holds the output gain in the range `0.0..1.0` (silent to full).
    // Gotcha:   `Float` is LESS precise than TS's `number`. A literal like
    //           `0.1f` cannot represent the exact decimal; expect tiny rounding
    //           differences if you compare a `Float` against a TS `number`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly volume: number;
    // ```
    val volume: Float,
    // What:     `val durationMs: Long`. A read-only (`val`) field named
    //           `durationMs`. `Long` is Kotlin's 64-bit signed integer (range
    //           roughly ±9.2 quintillion). Siblings the reader might expect:
    //           `Int` (32-bit) and `Short`/`Byte` (narrower). We picked `Long`
    //           (not `Int`) because Android's media APIs express times in
    //           milliseconds as 64-bit values; a 32-bit `Int` of milliseconds
    //           would overflow after only ~24.8 days, so `Long` is the safe,
    //           API-matching choice.
    // Why:      Holds the current track's total duration in milliseconds, or
    //           `0` when the duration is not yet known (still being probed).
    // Gotcha:   A `Long` can hold integers larger than TS `number` can
    //           represent exactly (beyond 2^53). For millisecond durations this
    //           never matters, but don't assume every `Long` round-trips
    //           losslessly through a TS `number`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly durationMs: number;
    // ```
    val durationMs: Long,
    // What:     `val positionMs: Long`. Same as `durationMs`: a read-only
    //           (`val`) field of Kotlin's 64-bit signed integer type `Long`
    //           (not the 32-bit `Int`), measured in milliseconds.
    // Why:      Holds the current playback position (how far into the track we
    //           are) in milliseconds, sampled at the moment the snapshot is
    //           built; the `MediaSession` extrapolates it forward between pulls.
    // Gotcha:   Same 64-bit-vs-2^53 caveat as `durationMs`; harmless for
    //           millisecond positions but worth knowing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly positionMs: number;
    // ```
    val positionMs: Long,
)
