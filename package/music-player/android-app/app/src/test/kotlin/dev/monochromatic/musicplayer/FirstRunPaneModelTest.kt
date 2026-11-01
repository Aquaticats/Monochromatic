// What:     `package dev.monochromatic.musicplayer` places this test beside the first-run pane.
// Why:      The test reaches the module-internal pure helpers without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the label constants the action mapping must accept.
// Why:      The mapping test feeds the same labels the core copy produces.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ALLOW_ACCESS_LABEL, OPEN_FOLDER_LABEL, SETTINGS_LABEL } from "core/FirstRunState";
// ```
import dev.monochromatic.musicplayer.core.ALLOW_ACCESS_LABEL
import dev.monochromatic.musicplayer.core.OPEN_FOLDER_LABEL
import dev.monochromatic.musicplayer.core.SETTINGS_LABEL

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `import androidx.compose.ui.unit.dp` brings in the Dp extension on numbers.
// Why:      The start-inset helper takes a Dp width, so the tests build widths the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// // Widths are plain numbers in pseudocode.
// ```
import androidx.compose.ui.unit.dp

// What:     `class FirstRunPaneModelTest` groups the pure-helper tests for the first-run pane.
// Why:      The start inset and action mapping can be verified on the host JVM without Compose.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("FirstRunPane helpers", () => { ... });
// ```
/** Verifies the pure helpers behind the first-run pane without a Compose runtime. */
class FirstRunPaneModelTest {
    // What:     `narrowWidthKeepsEdgeInset` checks that a narrow pane starts its text at the edge inset.
    // Why:      Below 600dp the text uses the full width, so no left half is left blank.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("narrow", () => expect(firstRunStartPadding(300)).toBe(24));
    // ```
    /** Confirms a width below the wide threshold starts the text at the 24dp edge inset. */
    @Test
    fun narrowWidthKeepsEdgeInset() {
        assertEquals(24.dp, firstRunStartPadding(maxWidth = 300.dp))
    }

    // What:     `wideThresholdStartsPastCentre` checks the inclusive wide-layout boundary.
    // Why:      Exactly 600dp is wide, so the text starts at half the width plus the edge inset.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("threshold", () => expect(firstRunStartPadding(600)).toBe(324));
    // ```
    /** Confirms a width equal to the wide threshold starts the text past the centre. */
    @Test
    fun wideThresholdStartsPastCentre() {
        assertEquals(324.dp, firstRunStartPadding(maxWidth = 600.dp))
    }

    // What:     `wideLayoutLeavesLeftHalfBlank` checks a tablet-width pane.
    // Why:      On wide layouts the text starts at half the width, leaving the left half blank.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wide", () => expect(firstRunStartPadding(840)).toBe(444));
    // ```
    /** Confirms a wide pane starts its text at half the width plus the edge inset. */
    @Test
    fun wideLayoutLeavesLeftHalfBlank() {
        assertEquals(444.dp, firstRunStartPadding(maxWidth = 840.dp))
    }

    // What:     `actionLabelsMapToTheirTargets` checks each label the core copy can produce.
    // Why:      A label that maps to the wrong callback would send the user to the wrong action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("labels", () => expect(firstRunActionTargetFor("Settings")).toBe("SETTINGS"));
    // ```
    /** Confirms each action label maps to the callback target it names. */
    @Test
    fun actionLabelsMapToTheirTargets() {
        assertEquals(FirstRunActionTarget.ALLOW_ACCESS, firstRunActionTargetFor(ALLOW_ACCESS_LABEL))
        assertEquals(FirstRunActionTarget.OPEN_FOLDER, firstRunActionTargetFor(OPEN_FOLDER_LABEL))
        assertEquals(FirstRunActionTarget.SETTINGS, firstRunActionTargetFor(SETTINGS_LABEL))
    }

    // What:     `unknownActionLabelThrows` checks that an unrecognised label is rejected.
    // Why:      An unknown label has no callback, so the pane must fail instead of drawing a dead button.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown", () => expect(() => firstRunActionTargetFor("Rescan")).toThrow());
    // ```
    /** Confirms a label outside the first-run action set throws an IllegalArgumentException. */
    @Test
    fun unknownActionLabelThrows() {
        assertThrows(IllegalArgumentException::class.java) {
            firstRunActionTargetFor("Rescan")
        }
    }
}
