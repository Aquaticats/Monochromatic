// The deck's background is the derived `deck` role (D118): a gray that takes the accent's hue unless the root color
// is completely neutral. Android used to draw a literal in the dark appearance, so no accent could reach it.

// What:     `package dev.monochromatic.musicplayer` places the test beside `TransportDeck.kt`.
// Why:      The color function it checks is internal to the package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer

import androidx.compose.ui.graphics.Color
import dev.monochromatic.musicplayer.core.Srgb8
import dev.monochromatic.musicplayer.core.ThemePalette
import dev.monochromatic.musicplayer.core.deriveRoles
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Checks that the deck follows the derived roles in both appearances. */
class DeckColorTest {
    /** Whether a color's three channels are equal, which is what a true gray is. */
    private fun isTrueGray(color: Color): Boolean = color.red == color.green && color.green == color.blue

    /** Two accents give two different dark decks, and the deck is the derived role, not a literal. */
    @Test
    fun theDarkDeckChangesWithTheAccent() {
        /** Palette of a green root. */
        val green: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x2E7D32))
        /** Palette of a violet root. */
        val violet: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x6A1B9A))
        /** Dark deck under the green root. */
        val greenDeck: Color = deckContainerColorOf(green.dark)
        /** Dark deck under the violet root. */
        val violetDeck: Color = deckContainerColorOf(violet.dark)
        assertNotEquals(greenDeck, violetDeck)
        assertEquals(green.dark.deck.toComposeColor(), greenDeck)
        assertTrue("a chromatic root tints the deck", !isTrueGray(greenDeck) && !isTrueGray(violetDeck))
    }

    /** A completely neutral root, equal channels, gives a deck whose channels are equal, in both appearances. */
    @Test
    fun aNeutralRootGivesATrueGrayDeck() {
        /** Palette of a mid gray root. */
        val gray: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x808080))
        assertTrue(isTrueGray(deckContainerColorOf(gray.dark)))
        assertTrue(isTrueGray(deckContainerColorOf(gray.light)))
    }

    /** A root with any chroma, however small, still tints the deck: a nearly gray root is not treated as gray. */
    @Test
    fun aNearlyGrayRootStillTintsTheDeck() {
        /** Palette of a root one step off gray. */
        val nearly: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x808081))
        assertTrue(!isTrueGray(deckContainerColorOf(nearly.dark)))
    }
}
