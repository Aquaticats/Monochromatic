// Layout threshold cases for the production player screen: which widths draw the folded cover layout.

// What:     `package dev.monochromatic.musicplayer` places these tests beside the composable they cover.
// Why:      The test reaches the module-internal `isCoverLayout` without exposing it publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     JUnit's assertions and test annotation register each threshold case.
// Why:      Each case compares a boolean at one width, so the boundary is checked exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class PlayerContentLayoutTest` groups the width threshold cases.
// Why:      The threshold is the only branch point of the screen, so every side of it is covered here.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("isCoverLayout", () => { /* cases */ });
// ```
/** Verifies that widths under 600dp draw the cover layout and widths from 600dp draw the unfolded layout. */
class PlayerContentLayoutTest {
    // What:     `@Test fun narrowWidthsDrawCoverLayout()` checks widths well under the threshold.
    // Why:      A folded or phone-width screen must always get the cover layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("narrow widths draw the cover layout", () => { /* ... */ });
    // ```
    /** Widths of zero and of a phone-width screen both draw the cover layout. */
    @Test
    fun narrowWidthsDrawCoverLayout() {
        assertTrue(isCoverLayout(0f))
        assertTrue(isCoverLayout(360f))
    }

    // What:     `@Test fun widthJustUnderThresholdDrawsCoverLayout()` checks the largest cover width.
    // Why:      The threshold is exclusive, so a width just under 600dp must still be the cover layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("width just under the threshold draws the cover layout", () => { /* ... */ });
    // ```
    /** A width of 599.9dp still draws the cover layout. */
    @Test
    fun widthJustUnderThresholdDrawsCoverLayout() {
        assertTrue(isCoverLayout(599.9f))
    }

    // What:     `@Test fun thresholdWidthDrawsUnfoldedLayout()` checks the threshold itself.
    // Why:      A width of exactly 600dp is not under 600dp, so it draws the unfolded layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("threshold width draws the unfolded layout", () => { /* ... */ });
    // ```
    /** A width of exactly 600dp draws the unfolded layout. */
    @Test
    fun thresholdWidthDrawsUnfoldedLayout() {
        assertFalse(isCoverLayout(600f))
    }

    // What:     `@Test fun wideWidthsDrawUnfoldedLayout()` checks widths well above the threshold.
    // Why:      An unfolded inner screen must always get the two-pane layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wide widths draw the unfolded layout", () => { /* ... */ });
    // ```
    /** Widths of 600dp and above, including a tablet-width screen, draw the unfolded layout. */
    @Test
    fun wideWidthsDrawUnfoldedLayout() {
        assertFalse(isCoverLayout(601f))
        assertFalse(isCoverLayout(1200f))
    }
}
