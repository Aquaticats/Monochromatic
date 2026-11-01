// Playback history can name the same track more than once, but the system media timeline needs one distinct id per
// timeline entry.

// What:     `package ...core` places this helper beside Queue, whose playback order it describes.
// Why:      The rule is pure logic, so unit tests reach it without Android.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val OCCURRENCE_SHIFT: Int = 32` is the bit position where the repeat count starts inside a
//           64-bit entry id; the load-order index fills the low 32 bits.
// Why:      A `Long` holds both numbers without collisions, so the first play of a track keeps its plain load index.
//
// In TS you'd write (pseudocode):
// ```ts
// const OCCURRENCE_SHIFT = 32n;
// ```
/** Bit position of the repeat count inside an entry id. */
private const val OCCURRENCE_SHIFT: Int = 32

// What:     `fun playbackEntryIds(order: List<Int>): List<Long>` maps each position of a playback order (a list of
//           load-order indices that may repeat) to an id that is unique among the positions.
// Why:      A shuffle history repeats a track once every track has played (a one-track folder repeats at once), and the
//           media timeline rejects two entries with one id (`Duplicate MediaItemData UID in playlist`). The first play
//           of a track keeps its load-order index as id and each later play adds the count of earlier plays in the
//           high bits, so ids stay stable while the history only grows at its end.
//
// In TS you'd write (pseudocode):
// ```ts
// function playbackEntryIds(order: readonly number[]): bigint[] {
//   const seen = new Map<number, number>();
//   return order.map((index) => {
//     const earlier = (seen.get(index) ?? 0);
//     seen.set(index, earlier + 1);
//     return (BigInt(earlier) << 32n) | BigInt(index);
//   });
// }
// ```
/**
 * Gives every position of [order] an id that no other position shares.
 *
 * @param order load-order indices in playback order, repeats allowed because a shuffle history revisits tracks
 * @return one id per position; a track's first position gets its load-order index, later ones add the earlier count
 */
fun playbackEntryIds(order: List<Int>): List<Long> {
    /** Number of earlier positions already given to each load-order index. */
    val earlierPlays: HashMap<Int, Int> = HashMap()
    return order.map { loadIndex ->
        /** How many positions before this one name the same track. */
        val earlier: Int = earlierPlays.getOrDefault(loadIndex, 0)
        earlierPlays[loadIndex] = earlier + 1
        (earlier.toLong() shl OCCURRENCE_SHIFT) or loadIndex.toLong()
    }
}
