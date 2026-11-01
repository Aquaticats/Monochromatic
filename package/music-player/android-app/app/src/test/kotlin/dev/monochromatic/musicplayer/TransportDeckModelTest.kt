// What:     `package dev.monochromatic.musicplayer` places this test beside the transport deck.
// Why:      The test reaches the module-internal pure helpers without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the playback mode type.
// Why:      Label and layout tests enumerate the four modes the deck displays.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlaybackMode } from "core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.PlaybackMode

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `import androidx.compose.ui.unit.dp` brings in the Dp extension on numbers.
// Why:      The layout helper takes a Dp width, so the tests build widths the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// // Widths are plain numbers in pseudocode.
// ```
import androidx.compose.ui.unit.dp

// What:     `class TransportDeckModelTest` groups the pure-helper tests for the transport deck.
// Why:      The layout, label, clock, and progress helpers can be verified on the host JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("TransportDeck helpers", () => { ... });
// ```
/** Verifies the pure helpers behind the transport deck without a Compose runtime. */
class TransportDeckModelTest {
    // What:     `oneRowAtFloorWidthWithoutOverflow` checks the one-row floor is inclusive.
    // Why:      Exactly 189dp must choose one row, because four 48dp targets fit at that width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one row at the inclusive floor", () => { ... });
    // ```
    /** Confirms a width equal to the one-row floor chooses one row when nothing overflowed. */
    @Test
    fun oneRowAtFloorWidthWithoutOverflow() {
        assertEquals(
            ModeControlLayout.ONE_ROW,
            modeControlLayoutFor(widthDp = 189.dp, oneRowOverflowed = false, twoRowOverflowed = false),
        )
    }

    // What:     `twoRowBelowOneRowFloor` checks a width just under the one-row floor.
    // Why:      Just below 189dp the four labels no longer fit one row, so two rows must be chosen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("two rows just below the one-row floor", () => { ... });
    // ```
    /** Confirms a width just under the one-row floor falls back to two rows. */
    @Test
    fun twoRowBelowOneRowFloor() {
        assertEquals(
            ModeControlLayout.TWO_ROW,
            modeControlLayoutFor(widthDp = 188.9f.dp, oneRowOverflowed = false, twoRowOverflowed = false),
        )
    }

    // What:     `twoRowAtFloorWidth` checks the two-row floor is inclusive.
    // Why:      Exactly 95dp must choose two rows, because two 48dp targets fit at that width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("two rows at the inclusive floor", () => { ... });
    // ```
    /** Confirms a width equal to the two-row floor chooses two rows when nothing overflowed. */
    @Test
    fun twoRowAtFloorWidth() {
        assertEquals(
            ModeControlLayout.TWO_ROW,
            modeControlLayoutFor(widthDp = 95.dp, oneRowOverflowed = false, twoRowOverflowed = false),
        )
    }

    // What:     `fourRowBelowTwoRowFloor` checks a width just under the two-row floor.
    // Why:      Below 95dp even two segments cannot share a row, so four rows must be chosen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("four rows just below the two-row floor", () => { ... });
    // ```
    /** Confirms a width just under the two-row floor falls back to four rows. */
    @Test
    fun fourRowBelowTwoRowFloor() {
        assertEquals(
            ModeControlLayout.FOUR_ROW,
            modeControlLayoutFor(widthDp = 94.9f.dp, oneRowOverflowed = false, twoRowOverflowed = false),
        )
    }

    // What:     `oneRowOverflowDemotesToTwoRows` checks a recorded one-row clip at a wide width.
    // Why:      A label that clipped in one row must not stay in one row at that width and scale.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a one-row clip demotes to two rows", () => { ... });
    // ```
    /** Confirms a recorded one-row overflow moves a wide control to two rows. */
    @Test
    fun oneRowOverflowDemotesToTwoRows() {
        assertEquals(
            ModeControlLayout.TWO_ROW,
            modeControlLayoutFor(widthDp = 300.dp, oneRowOverflowed = true, twoRowOverflowed = false),
        )
    }

    // What:     `bothOverflowFallToFourRows` checks two recorded clips at a wide width.
    // Why:      When one and two rows both clipped, only the four-row stack remains.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("both overflows fall back to four rows", () => { ... });
    // ```
    /** Confirms both recorded overflows force the four-row stack even at a wide width. */
    @Test
    fun bothOverflowFallToFourRows() {
        assertEquals(
            ModeControlLayout.FOUR_ROW,
            modeControlLayoutFor(widthDp = 300.dp, oneRowOverflowed = true, twoRowOverflowed = true),
        )
    }

    // What:     `twoRowOverflowFallsToFourRows` checks a two-row clip at a width that allows two rows.
    // Why:      A clip in the two-row layout must not keep the control in two rows at that width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a two-row clip falls to four rows", () => { ... });
    // ```
    /** Confirms a recorded two-row overflow at a two-row width falls to four rows. */
    @Test
    fun twoRowOverflowFallsToFourRows() {
        assertEquals(
            ModeControlLayout.FOUR_ROW,
            modeControlLayoutFor(widthDp = 120.dp, oneRowOverflowed = false, twoRowOverflowed = true),
        )
    }

    // What:     `oneRowOverflowAtNarrowWidthStillUsesTwoRows` checks a one-row clip below the one-row floor.
    // Why:      A narrow control already excludes one row, so the clip only changes the two-row decision.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a one-row clip below the floor keeps two rows", () => { ... });
    // ```
    /** Confirms a one-row clip at a narrow width still chooses two rows. */
    @Test
    fun oneRowOverflowAtNarrowWidthStillUsesTwoRows() {
        assertEquals(
            ModeControlLayout.TWO_ROW,
            modeControlLayoutFor(widthDp = 100.dp, oneRowOverflowed = true, twoRowOverflowed = false),
        )
    }

    // What:     `visibleLabelsMatchEachMode` checks the four visible segment labels.
    // Why:      The visible labels are fixed text, so each mode must map to its exact label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("visible labels", () => { ... });
    // ```
    /** Confirms every playback mode maps to its visible label, with the folder name in shuffle. */
    @Test
    fun visibleLabelsMatchEachMode() {
        assertEquals("Repeat", modeLabelFor(PlaybackMode.REPEAT, "Camellia"))
        assertEquals("In order", modeLabelFor(PlaybackMode.IN_ORDER, "Camellia"))
        assertEquals("Shuffle Camellia", modeLabelFor(PlaybackMode.SHUFFLE_PAGE, "Camellia"))
        assertEquals("Shuffle all", modeLabelFor(PlaybackMode.SHUFFLE_ALL, "Camellia"))
    }

    // What:     `accessibleLabelsKeepFullFolderName` checks the four spoken descriptions.
    // Why:      Screen readers hear the unshortened folder name and fuller phrases for each mode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("accessible labels", () => { ... });
    // ```
    /** Confirms every playback mode maps to its spoken description with the full folder name. */
    @Test
    fun accessibleLabelsKeepFullFolderName() {
        assertEquals("Repeat track", modeAccessibleLabelFor(PlaybackMode.REPEAT, "Extraordinary"))
        assertEquals("Play in order", modeAccessibleLabelFor(PlaybackMode.IN_ORDER, "Extraordinary"))
        assertEquals(
            "Shuffle Extraordinary",
            modeAccessibleLabelFor(PlaybackMode.SHUFFLE_PAGE, "Extraordinary"),
        )
        assertEquals("Shuffle all folders", modeAccessibleLabelFor(PlaybackMode.SHUFFLE_ALL, "Extraordinary"))
    }

    // What:     `clockLabelFormatsMinutesAndPaddedSeconds` checks typical and boundary durations.
    // Why:      Seconds below ten need a leading zero so the clock width stays steady.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("m:ss formatting", () => { ... });
    // ```
    /** Confirms durations format as minutes and two-digit seconds, including zero and the hour edge. */
    @Test
    fun clockLabelFormatsMinutesAndPaddedSeconds() {
        assertEquals("1:06", deckClockLabel(66.0))
        assertEquals("4:35", deckClockLabel(275.0))
        assertEquals("0:00", deckClockLabel(0.0))
        assertEquals("0:09", deckClockLabel(9.4))
        assertEquals("59:59", deckClockLabel(3599.9))
    }

    // What:     `clockLabelClampsNegativeAndNaN` checks inputs that are not valid durations.
    // Why:      A negative or NaN position from the engine must still render as a valid clock.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clock clamps invalid input", () => { ... });
    // ```
    /** Confirms negative and NaN durations render as zero rather than a malformed label. */
    @Test
    fun clockLabelClampsNegativeAndNaN() {
        assertEquals("0:00", deckClockLabel(-5.0))
        assertEquals("0:00", deckClockLabel(Double.NaN))
    }

    // What:     `progressFractionMatchesPositionRatio` checks an ordinary position.
    // Why:      The slider fraction must equal position over duration for the study's example.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fraction equals position over duration", () => { ... });
    // ```
    /** Confirms the study position of 1:06 inside 4:35 maps to its exact ratio. */
    @Test
    fun progressFractionMatchesPositionRatio() {
        assertEquals(66f / 275f, progressFractionFor(positionSec = 66.0, durationSec = 275.0), 0.0001f)
    }

    // What:     `progressFractionClampsToRange` checks positions beyond either end of the track.
    // Why:      The slider must stay inside zero to one even when the engine overshoots.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fraction clamps", () => { ... });
    // ```
    /** Confirms positions below zero and above the duration clamp to the slider bounds. */
    @Test
    fun progressFractionClampsToRange() {
        assertEquals(0f, progressFractionFor(positionSec = -3.0, durationSec = 275.0), 0f)
        assertEquals(1f, progressFractionFor(positionSec = 300.0, durationSec = 275.0), 0f)
    }

    // What:     `progressFractionGuardsZeroAndNaNDuration` checks durations that cannot be divided.
    // Why:      A zero, negative, or NaN duration must yield zero instead of NaN in the slider.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fraction guards invalid duration", () => { ... });
    // ```
    /** Confirms zero, negative, and NaN durations all yield a zero fraction. */
    @Test
    fun progressFractionGuardsZeroAndNaNDuration() {
        assertEquals(0f, progressFractionFor(positionSec = 10.0, durationSec = 0.0), 0f)
        assertEquals(0f, progressFractionFor(positionSec = 10.0, durationSec = -1.0), 0f)
        assertEquals(0f, progressFractionFor(positionSec = 10.0, durationSec = Double.NaN), 0f)
    }

    // What:     `oneLineFitsExactlyWithGapCountedOnce` checks the boundary where the row just fits.
    // Why:      100 plus 16 plus 200 is 316, so the gap must be added once to reach the exact fit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one line fits exactly at the summed width", () => { ... });
    // ```
    /** Confirms a width equal to transport, one gap, and mode fits side by side. */
    @Test
    fun oneLineFitsExactlyWithGapCountedOnce() {
        assertTrue(
            deckControlsFitOneLine(availableWidthPx = 316, transportWidthPx = 100, modeWidthPx = 200, gapPx = 16),
        )
    }

    // What:     `oneLineRejectsOnePixelShort` checks the width one pixel under the exact fit.
    // Why:      A single pixel short must stack the controls, or the row would clip a 48dp target.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one line rejects one pixel short", () => { ... });
    // ```
    /** Confirms a width one pixel below the summed width does not fit side by side. */
    @Test
    fun oneLineRejectsOnePixelShort() {
        assertFalse(
            deckControlsFitOneLine(availableWidthPx = 315, transportWidthPx = 100, modeWidthPx = 200, gapPx = 16),
        )
    }

    // What:     `oneLineFitsWideWidth` checks a width far above the summed width.
    // Why:      A wide deck, such as an unfolded screen, must keep the controls on one row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one line fits a wide width", () => { ... });
    // ```
    /** Confirms a width well above the summed width fits side by side. */
    @Test
    fun oneLineFitsWideWidth() {
        assertTrue(
            deckControlsFitOneLine(availableWidthPx = 1080, transportWidthPx = 100, modeWidthPx = 200, gapPx = 16),
        )
    }

    // What:     `oneLineGapChangesTheBoundary` checks that a wider gap moves the boundary by its own size.
    // Why:      With a 17 pixel gap the same widths need one more pixel, which proves the gap is in the sum.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a wider gap needs one more pixel", () => { ... });
    // ```
    /** Confirms a gap one pixel wider rejects the width that fit with a gap of 16 pixels. */
    @Test
    fun oneLineGapChangesTheBoundary() {
        assertFalse(
            deckControlsFitOneLine(availableWidthPx = 316, transportWidthPx = 100, modeWidthPx = 200, gapPx = 17),
        )
    }

    // What:     `oneLineZeroWidthsNeedOnlyTheGap` checks controls with no measured width.
    // Why:      With zero-width groups the gap alone decides the fit, so the boundary is exactly the gap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("zero controls need only the gap", () => { ... });
    // ```
    /** Confirms zero-width groups fit exactly when the available width equals the gap. */
    @Test
    fun oneLineZeroWidthsNeedOnlyTheGap() {
        assertTrue(deckControlsFitOneLine(availableWidthPx = 16, transportWidthPx = 0, modeWidthPx = 0, gapPx = 16))
        assertFalse(deckControlsFitOneLine(availableWidthPx = 15, transportWidthPx = 0, modeWidthPx = 0, gapPx = 16))
    }

    // What:     `oneLineZeroAvailableOnlyFitsEmptyControls` checks a zero available width.
    // Why:      Nothing with a positive width fits in zero pixels, while all-zero inputs trivially fit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("zero available width", () => { ... });
    // ```
    /** Confirms zero available width fits only when every width and the gap are zero. */
    @Test
    fun oneLineZeroAvailableOnlyFitsEmptyControls() {
        assertTrue(deckControlsFitOneLine(availableWidthPx = 0, transportWidthPx = 0, modeWidthPx = 0, gapPx = 0))
        assertFalse(deckControlsFitOneLine(availableWidthPx = 0, transportWidthPx = 1, modeWidthPx = 0, gapPx = 0))
    }

    // What:     `oneLineHugeWidthsDoNotWrap` checks inputs whose sum exceeds the Int range.
    // Why:      Summing in Int would wrap negative and report a false fit, so the sum is held in Long.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("huge widths do not wrap", () => { ... });
    // ```
    /** Confirms widths whose sum overflows Int do not report a fit at the largest Int available width. */
    @Test
    fun oneLineHugeWidthsDoNotWrap() {
        assertFalse(
            deckControlsFitOneLine(
                availableWidthPx = Int.MAX_VALUE,
                transportWidthPx = Int.MAX_VALUE,
                modeWidthPx = Int.MAX_VALUE,
                gapPx = Int.MAX_VALUE,
            ),
        )
    }
}
