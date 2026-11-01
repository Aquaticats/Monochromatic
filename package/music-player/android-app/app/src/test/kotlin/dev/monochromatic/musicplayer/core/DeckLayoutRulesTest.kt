// What:     `package dev.monochromatic.musicplayer.core` places this test beside the deck layout rules.
// Why:      The test reaches the internal pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each width and line budget is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class DeckLayoutRulesTest` groups the tests of the deck stability rules.
// Why:      The deck must not move with the folder name, and headings must wrap only at large text.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("deckLayoutRules", () => { ... });
// ```
/** Verifies the shuffle label slot width and the heading line budget. */
class DeckLayoutRulesTest {
    // What:     `slotWidthIsTheSameForEveryFolderName` checks the fixed slot over short, exact and long names.
    // Why:      The slot width feeds the layout decisions, so it must not depend on the name's natural width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("slot width ignores the name", () => { ... });
    // ```
    /** Confirms the slot is the prefix plus the reference width for any natural name width. */
    @Test
    fun slotWidthIsTheSameForEveryFolderName() {
        for (naturalPx in listOf(0, 12, 120, 400, 4000)) {
            /** Name text width for this natural width. */
            val nameWidth: Int = shuffleFolderTextWidthPx(naturalPx = naturalPx, referencePx = 200)
            assertEquals(minOf(naturalPx, 200), nameWidth)
            assertEquals(300, shuffleLabelSlotWidthPx(prefixPx = 100, referencePx = 200))
        }
    }

    // What:     `titleGetsOneLineMoreAtLargeText` checks the title's line budget.
    // Why:      The title shares the line with the folder and reserves one more line once headings wrap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title reserves an extra line from 1.5", () => { ... });
    // ```
    /** Confirms the title reserves one line below 150 percent text and three lines from it. */
    @Test
    fun titleGetsOneLineMoreAtLargeText() {
        assertEquals(1, titleLineBudget(1.0f))
        assertEquals(1, titleLineBudget(1.3f))
        assertEquals(3, titleLineBudget(1.5f))
        assertEquals(3, titleLineBudget(2.0f))
    }

    // What:     `longNamesAreCappedAtTheReferenceWidth` checks the D1 cap.
    // Why:      A name wider than `Camellia` is cut in the middle at the reference width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("long names cap at the reference", () => { ... });
    // ```
    /** Confirms a name wider than the reference is capped and a narrower one keeps its width. */
    @Test
    fun longNamesAreCappedAtTheReferenceWidth() {
        assertEquals(200, shuffleFolderTextWidthPx(naturalPx = 900, referencePx = 200))
        assertEquals(30, shuffleFolderTextWidthPx(naturalPx = 30, referencePx = 200))
    }

    // What:     `headingsWrapOnlyAtLargeText` checks the line budget on the Android font scale ladder.
    // Why:      One line at the ordinary steps keeps the deck height fixed, and two lines from 150 percent keep it
    //           fixed there while a long name wraps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("two lines from 1.5", () => { ... });
    // ```
    /** Confirms the budget is one line below 150 percent text and two lines from it. */
    @Test
    fun headingsWrapOnlyAtLargeText() {
        for (scale in listOf(0.85f, 1.0f, 1.15f, 1.3f)) {
            assertEquals("scale $scale", 1, headingLineBudget(scale))
        }
        for (scale in listOf(1.5f, 1.8f, 2.0f)) {
            assertEquals("scale $scale", 2, headingLineBudget(scale))
        }
    }
}
