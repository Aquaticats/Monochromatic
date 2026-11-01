// What:     `package dev.monochromatic.musicplayer` places this test beside the track menu popup.
// Why:      The test reaches the module-internal divider and glyph helpers without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the action type and the D7 list.
// Why:      Divider and glyph tests enumerate real actions rather than inventing group numbers.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TrackMenuAction, trackMenuActionsFor } from "core/TrackMenuAction";
// ```
import dev.monochromatic.musicplayer.core.TrackMenuAction
import dev.monochromatic.musicplayer.core.trackMenuActionsFor

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class TrackMenuPopupModelTest` groups the pure helper tests for the track menu popup.
// Why:      Divider placement and glyph selection can be verified on the host JVM without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("track menu popup helpers", () => { ... });
// ```
/** Verifies divider placement and glyph selection without a Compose runtime. */
class TrackMenuPopupModelTest {
    // What:     `dividersFollowD7Groups` checks the divider flags for the full accepted list.
    // Why:      The D7 list has two group changes, so exactly two dividers must be drawn.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dividers follow D7 groups", () => { ... });
    // ```
    /** Confirms dividers appear before File details and before Show in file manager only. */
    @Test
    fun dividersFollowD7Groups() {
        assertEquals(
            listOf(false, false, true, true, false, false),
            trackMenuDividerFlags(trackMenuActionsFor()),
        )
    }

    // What:     `emptyListHasNoDividers` checks the empty input.
    // Why:      A menu with no actions must produce no divider flags rather than a fallback value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty list", () => expect(dividerFlags([])).toEqual([]));
    // ```
    /** Confirms an empty action list yields an empty flag list. */
    @Test
    fun emptyListHasNoDividers() {
        assertEquals(emptyList<Boolean>(), trackMenuDividerFlags(emptyList()))
    }

    // What:     `singleActionHasNoDivider` checks the one-element input.
    // Why:      The first row never has a neighbour above it, so it never receives a divider.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single action", () => expect(dividerFlags([play])).toEqual([false]));
    // ```
    /** Confirms a single action never receives a divider. */
    @Test
    fun singleActionHasNoDivider() {
        assertEquals(listOf(false), trackMenuDividerFlags(listOf(TrackMenuAction.PLAY)))
    }

    // What:     `sameGroupHasNoDivider` checks two neighbours in the same section.
    // Why:      Neighbours in one section must stay together without a rule between them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same group", () => expect(dividerFlags([play, shuffle])).toEqual([false, false]));
    // ```
    /** Confirms neighbours with equal groups receive no divider. */
    @Test
    fun sameGroupHasNoDivider() {
        assertEquals(
            listOf(false, false),
            trackMenuDividerFlags(listOf(TrackMenuAction.PLAY, TrackMenuAction.SHUFFLE_FROM_HERE)),
        )
    }

    // What:     `everyGroupChangeDrawsADivider` checks a list where each neighbour changes group.
    // Why:      Each change must produce its own divider, so no change is silently merged away.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("every change draws", () => expect(dividerFlags([play, details, copy])).toEqual([false, true, true]));
    // ```
    /** Confirms every neighbour that changes group receives a divider. */
    @Test
    fun everyGroupChangeDrawsADivider() {
        assertEquals(
            listOf(false, true, true),
            trackMenuDividerFlags(
                listOf(TrackMenuAction.PLAY, TrackMenuAction.DETAILS, TrackMenuAction.COPY_FILENAME),
            ),
        )
    }

    // What:     `trackMenuIconMapsEveryAction` checks the glyph chosen for each action.
    // Why:      Each action must show its own glyph, so a swapped mapping fails the named comparison.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("glyph per action", () => { ... });
    // ```
    /** Confirms each of the six actions maps to its named glyph. */
    @Test
    fun trackMenuIconMapsEveryAction() {
        /** Each action paired with the glyph name it must produce. */
        val expected: List<Pair<TrackMenuAction, String>> = listOf(
            Pair(TrackMenuAction.PLAY, "TrackMenu.Play"),
            Pair(TrackMenuAction.SHUFFLE_FROM_HERE, "TrackMenu.Shuffle"),
            Pair(TrackMenuAction.DETAILS, "TrackMenu.Info"),
            Pair(TrackMenuAction.SHOW_IN_FILE_MANAGER, "TrackMenu.FolderOpen"),
            Pair(TrackMenuAction.COPY_FILENAME, "TrackMenu.ContentCopy"),
            Pair(TrackMenuAction.MOVE_TO_TRASH, "TrackMenu.Delete"),
        )
        expected.forEach { (action, glyphName) ->
            assertEquals(glyphName, trackMenuIcon(action).name)
        }
    }

    // What:     `detailsSuffixIsEmptyForNull` checks that an absent value adds no text.
    // Why:      A track without analysis must show File details with no trailing separator.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details suffix null", () => expect(trackMenuDetailsSuffix(null)).toBe(""));
    // ```
    /** Confirms a null peak value produces an empty suffix. */
    @Test
    fun detailsSuffixIsEmptyForNull() {
        assertEquals("", trackMenuDetailsSuffix(null))
    }

    // What:     `detailsSuffixIsEmptyForBlankValue` checks that a whitespace-only value adds no text.
    // Why:      A blank value would otherwise render a dangling middle dot beside File details.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details suffix blank", () => expect(trackMenuDetailsSuffix("   ")).toBe(""));
    // ```
    /** Confirms a blank peak value produces an empty suffix. */
    @Test
    fun detailsSuffixIsEmptyForBlankValue() {
        assertEquals("", trackMenuDetailsSuffix("   "))
    }

    // What:     `detailsSuffixJoinsValueWithSeparator` checks the separator and the value text.
    // Why:      The inline peak must read as the label, a middle dot, and the dB value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details suffix value", () => expect(trackMenuDetailsSuffix("-0.8 dBTP")).toBe(" · -0.8 dBTP"));
    // ```
    /** Confirms a present peak value is appended after a middle-dot separator. */
    @Test
    fun detailsSuffixJoinsValueWithSeparator() {
        assertEquals(" · -0.8 dBTP", trackMenuDetailsSuffix("-0.8 dBTP"))
    }

    // What:     `detailsSuffixKeepsValueExact` checks that the value text is not trimmed or altered.
    // Why:      The analysed value must appear exactly as authored, including its own spacing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("details suffix exact", () => expect(trackMenuDetailsSuffix(" -1.2 dBTP ")).toBe(" ·  -1.2 dBTP "));
    // ```
    /** Confirms surrounding spaces of a present value are kept as authored. */
    @Test
    fun detailsSuffixKeepsValueExact() {
        assertEquals(" ·  -1.2 dBTP ", trackMenuDetailsSuffix(" -1.2 dBTP "))
    }

    // What:     `glyphsAreDistinct` checks that no two actions share a glyph.
    // Why:      A shared glyph would make two actions look identical beside their labels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("glyphs are distinct", () => { ... });
    // ```
    /** Confirms the seven actions each have a different glyph name. */
    @Test
    fun glyphsAreDistinct() {
        /** Glyph name of each action in display order. */
        val names: List<String> = trackMenuActionsFor()
            .map { action -> trackMenuIcon(action).name }
        assertEquals(names.size, names.toSet().size)
    }
}
