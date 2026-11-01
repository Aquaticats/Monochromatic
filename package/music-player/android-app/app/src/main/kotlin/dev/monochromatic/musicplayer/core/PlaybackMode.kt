// Playback has one persisted mode. This replaces the former independent shuffle
// enum and repeat-track boolean so impossible combinations cannot survive in memory.

// What:     `package ...core` places this enum beside Queue and Session.
// Why:      Playback logic and migration can share one platform-independent type.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class PlaybackMode(val storedName: String)` declares four closed
//           choices. Each value owns an explicit persistence string rather than using
//           Kotlin's rename-sensitive generated `.name`.
// Why:      Exactly one choice controls natural completion, manual transport scope,
//           ordering, UI selection, and saved state.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlaybackMode = "repeat" | "in_order" | "shuffle_page" | "shuffle_all";
// ```
/** Defines the single playback behavior selected throughout the player. */
enum class PlaybackMode(
    /** Stable text written to device preferences. */
    val storedName: String,
) {
    /** Replays the current track after a natural completion. */
    REPEAT("repeat"),

    /** Plays the displayed page in order and stops after its last track (D2); a manual Next wraps to the start. */
    IN_ORDER("in_order"),

    /** Shuffles the displayed page without replacement in repeated cycles. */
    SHUFFLE_PAGE("shuffle_page"),

    /** Shuffles the complete library without replacement in repeated cycles. */
    SHUFFLE_ALL("shuffle_all");

    // What:     `companion object` is Kotlin's static-member container.
    // Why:      Persistence decoding and one-time legacy migration belong beside the
    //           values whose wire forms they produce.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // namespace PlaybackMode { /* decoding helpers */ }
    // ```
    /** Holds storage-boundary decoding helpers. */
    companion object {
        // What:     `fun fromStoredName(storedName: String): PlaybackMode` finds a
        //           known explicit wire value and otherwise returns In order.
        // Why:      Unknown future or corrupt values must degrade safely without
        //           consulting stale legacy fields.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // function fromStoredName(value: string): PlaybackMode {
        //   return entries.find(mode => mode.storedName === value) ?? "in_order";
        // }
        // ```
        /** Decodes current persisted text with an in-order fallback. */
        fun fromStoredName(storedName: String): PlaybackMode =
            entries.firstOrNull { mode -> mode.storedName == storedName } ?: IN_ORDER

        // What:     `fun fromLegacy(shuffleName: String?, repeatTrack: Boolean)`
        //           accepts only the former storage fields at the migration boundary.
        // Why:      All six old shuffle/repeat combinations collapse into the required
        //           four-state model before they enter the player.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // function fromLegacy(shuffle: string | null, repeat: boolean): PlaybackMode {
        //   if (repeat) return "repeat";
        //   if (shuffle === "WITHIN_PAGE") return "shuffle_page";
        //   if (shuffle === "ALL") return "shuffle_all";
        //   return "in_order";
        // }
        // ```
        /** Migrates one former shuffle and repeat-track combination. */
        fun fromLegacy(shuffleName: String?, repeatTrack: Boolean): PlaybackMode {
            if (repeatTrack) {
                return REPEAT
            }
            if (shuffleName == "WITHIN_PAGE") {
                return SHUFFLE_PAGE
            }
            if (shuffleName == "ALL") {
                return SHUFFLE_ALL
            }
            return IN_ORDER
        }
    }
}
