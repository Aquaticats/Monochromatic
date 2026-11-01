// Cases of `rowWithPlayingFlag`: a row learns whether it is the playing track from the playing library index alone.

// What:     `package ...core` places the test beside `TrackListEntries.kt`, which declares the function.
// Why:      The test reaches `rowWithPlayingFlag` and `TrackRowModel` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Test

// What:     `class RowPlayingFlagTest` groups the cases of the playing flag a visible row computes for itself.
// Why:      The host builds the rows of a page once and no longer rebuilds them when the playing track changes, so this
//           one comparison is the only place a row learns it is playing.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("rowWithPlayingFlag", () => { /* cases */ });
// ```
/** Verifies the flag of a row follows the playing library index, and that an unchanged row is returned as it is. */
class RowPlayingFlagTest {
    /** Builds one row of the given library index and flag, with a fixed title and no supporting line. */
    private fun row(index: Int, current: Boolean): TrackRowModel =
        TrackRowModel(indexInLibrary = index, title = "Track $index", supportingLine = "line", isCurrent = current)

    // What:     `@Test fun rowWithThePlayingIndexBecomesCurrent()` checks a row that is the playing track.
    // Why:      A track change must light the new playing row without anyone rebuilding the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing row", () => expect(rowWithPlayingFlag(row(3, false), 3).isCurrent).toBe(true));
    // ```
    /** A row whose index is the playing index is returned flagged current, with its other fields kept. */
    @Test
    fun rowWithThePlayingIndexBecomesCurrent() {
        assertEquals(row(3, current = true), rowWithPlayingFlag(row(3, current = false), playingIndex = 3))
    }

    // What:     `@Test fun rowWithAnotherIndexStopsBeingCurrent()` checks a row that was playing and no longer is.
    // Why:      The previous playing row must go back to normal when the next track starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no longer playing", () => expect(rowWithPlayingFlag(row(3, true), 4).isCurrent).toBe(false));
    // ```
    /** A flagged row whose index is not the playing index is returned unflagged. */
    @Test
    fun rowWithAnotherIndexStopsBeingCurrent() {
        assertEquals(row(3, current = false), rowWithPlayingFlag(row(3, current = true), playingIndex = 4))
    }

    // What:     `@Test fun noPlayingIndexClearsTheFlag()` checks the state with no track loaded.
    // Why:      With nothing playing no row may look playing, whatever flag it was built with.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nothing playing", () => expect(rowWithPlayingFlag(row(3, true), null).isCurrent).toBe(false));
    // ```
    /** A null playing index clears the flag of a row. */
    @Test
    fun noPlayingIndexClearsTheFlag() {
        assertEquals(row(3, current = false), rowWithPlayingFlag(row(3, current = true), playingIndex = null))
    }

    // What:     `@Test fun unchangedRowIsReturnedAsTheSameInstance()` checks that nothing is copied when nothing changes.
    // Why:      A visible row compares the result by identity to skip recomposition, and every row of a page asks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same instance", () => { const r = row(3, false); expect(rowWithPlayingFlag(r, 4)).toBe(r); });
    // ```
    /** A row that already has the right flag is returned as the same instance, for either flag value. */
    @Test
    fun unchangedRowIsReturnedAsTheSameInstance() {
        /** A row that is not playing, asked about another playing index. */
        val idle: TrackRowModel = row(3, current = false)
        /** A row that is playing, asked about its own index. */
        val playing: TrackRowModel = row(3, current = true)
        assertSame(idle, rowWithPlayingFlag(idle, playingIndex = 4))
        assertSame(playing, rowWithPlayingFlag(playing, playingIndex = 3))
    }
}
