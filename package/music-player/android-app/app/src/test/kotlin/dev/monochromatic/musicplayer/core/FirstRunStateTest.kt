// What:     `package dev.monochromatic.musicplayer.core` places this test beside the first-run copy.
// Why:      Same-package tests call `firstRunCopy` and the label constants without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit assertions and the test annotation register each copy check.
// Why:      Every string and action order is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test, throws } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class FirstRunStateTest` groups the copy tests for the three first-run states.
// Why:      The copy is the approved text, so each state's strings and actions are checked in one place.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("firstRunCopy", () => { ... });
// ```
/** Verifies the exact first-run copy, action order, and analysis rules for every state. */
class FirstRunStateTest {
    // What:     `declinedTitleIsExact` checks the declined headline word for word.
    // Why:      D101 approves this headline, so any drift from it is a regression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined title", () => expect(copy.title).toBe("The device music library can't be read"));
    // ```
    /** Confirms the declined headline matches the approved text exactly. */
    @Test
    fun declinedTitleIsExact() {
        assertEquals(
            "The device music library can't be read",
            firstRunCopy(state = FirstRunState.DECLINED, folderName = null).title,
        )
    }

    // What:     `declinedBodyIsExact` checks the declined body word for word.
    // Why:      The body names only the assessed source and the two ways forward.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined body", () => expect(copy.body).toBe("It is open as your library, ..."));
    // ```
    /** Confirms the declined body matches the approved text exactly. */
    @Test
    fun declinedBodyIsExact() {
        assertEquals(
            "It is open as your library, but access to music on this device was not granted. " +
                "Allow access, or open a folder instead.",
            firstRunCopy(state = FirstRunState.DECLINED, folderName = null).body,
        )
    }

    // What:     `declinedOffersAllowAccessFirst` checks the filled action and the outlined order.
    // Why:      D101 puts Allow access first and filled, then Open a folder and Settings outlined.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined actions", () => expect(copy.primaryAction).toBe("Allow access"));
    // ```
    /** Confirms the declined state fills Allow access and outlines Open a folder then Settings. */
    @Test
    fun declinedOffersAllowAccessFirst() {
        val copy: FirstRunCopy = firstRunCopy(state = FirstRunState.DECLINED, folderName = null)
        assertEquals("Allow access", copy.primaryAction)
        assertEquals(listOf("Open a folder", "Settings"), copy.secondaryActions)
    }

    // What:     `declinedExplainsAnalysis` checks that only the declined state carries the explanation.
    // Why:      D10 and D101 place the true-peak explanation under the declined buttons.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined analysis", () => expect(copy.explainsAnalysis).toBe(true));
    // ```
    /** Confirms the declined state shows the true-peak explanation. */
    @Test
    fun declinedExplainsAnalysis() {
        assertTrue(firstRunCopy(state = FirstRunState.DECLINED, folderName = null).explainsAnalysis)
    }

    // What:     `systemNoAudioTitleIsExact` checks the device library headline word for word.
    // Why:      The study's headline is the approved copy for the device library with no audio.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("system title", () => expect(copy.title).toBe("No audio found in the device music library"));
    // ```
    /** Confirms the device library no-audio headline matches the study text exactly. */
    @Test
    fun systemNoAudioTitleIsExact() {
        assertEquals(
            "No audio found in the device music library",
            firstRunCopy(state = FirstRunState.SYSTEM_NO_AUDIO, folderName = null).title,
        )
    }

    // What:     `systemNoAudioBodyIsExact` checks the device library body word for word.
    // Why:      The body says the library was checked completely and names a folder alternative.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("system body", () => expect(copy.body).toBe("The device music library was checked completely. ..."));
    // ```
    /** Confirms the device library no-audio body matches the study text exactly. */
    @Test
    fun systemNoAudioBodyIsExact() {
        assertEquals(
            "The device music library was checked completely. You can open a folder that is not listed there.",
            firstRunCopy(state = FirstRunState.SYSTEM_NO_AUDIO, folderName = null).body,
        )
    }

    // What:     `systemNoAudioOffersFolderThenSettings` checks the filled and outlined actions.
    // Why:      The filled button already opens a folder, so no second folder button is drawn.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("system actions", () => expect(copy.primaryAction).toBe("Open a folder"));
    // ```
    /** Confirms the device no-audio state fills Open a folder and outlines only Settings. */
    @Test
    fun systemNoAudioOffersFolderThenSettings() {
        val copy: FirstRunCopy = firstRunCopy(state = FirstRunState.SYSTEM_NO_AUDIO, folderName = null)
        assertEquals("Open a folder", copy.primaryAction)
        assertEquals(listOf("Settings"), copy.secondaryActions)
    }

    // What:     `folderNoAudioInsertsFolderName` checks that the chosen folder name appears in the headline.
    // Why:      The headline must name the folder that was checked, not a fixed sample name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder title", () => expect(copy.title).toBe("No audio found in Cult of Luna"));
    // ```
    /** Confirms the chosen folder name is inserted into the folder no-audio headline. */
    @Test
    fun folderNoAudioInsertsFolderName() {
        assertEquals(
            "No audio found in Cult of Luna",
            firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = "Cult of Luna").title,
        )
    }

    // What:     `folderNoAudioUsesTheSuppliedName` checks that a different folder name replaces the first one.
    // Why:      The headline is built from the argument, so it must follow whatever folder is passed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder name", () => expect(firstRunCopy("FOLDER_NO_AUDIO", "Albums").title)
    //   .toBe("No audio found in Albums"));
    // ```
    /** Confirms a different folder name is inserted instead of a remembered one. */
    @Test
    fun folderNoAudioUsesTheSuppliedName() {
        assertEquals(
            "No audio found in Albums",
            firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = "Albums").title,
        )
    }

    // What:     `folderNoAudioBodyIsExact` checks the folder body word for word.
    // Why:      The body states that the folder was checked completely and that changing it changes the source.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder body", () => expect(copy.body).toBe("The chosen folder was checked completely ..."));
    // ```
    /** Confirms the folder no-audio body matches the study text exactly. */
    @Test
    fun folderNoAudioBodyIsExact() {
        assertEquals(
            "The chosen folder was checked completely and contains no supported audio files. " +
                "Opening a different folder changes the music source.",
            firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = "Albums").body,
        )
    }

    // What:     `folderNoAudioOffersFolderThenSettings` checks the folder state's filled and outlined actions.
    // Why:      The folder state mirrors the device state: Open a folder filled, Settings outlined.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder actions", () => expect(copy.secondaryActions).toEqual(["Settings"]));
    // ```
    /** Confirms the folder no-audio state fills Open a folder and outlines only Settings. */
    @Test
    fun folderNoAudioOffersFolderThenSettings() {
        val copy: FirstRunCopy = firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = "Albums")
        assertEquals("Open a folder", copy.primaryAction)
        assertEquals(listOf("Settings"), copy.secondaryActions)
    }

    // What:     `folderNoAudioRejectsMissingName` checks that a null folder name throws.
    // Why:      A headline without a folder name would be wrong, so the state refuses to draw it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("missing name", () => expect(() => firstRunCopy("FOLDER_NO_AUDIO", null)).toThrow());
    // ```
    /** Confirms a missing folder name is rejected for the folder no-audio state. */
    @Test
    fun folderNoAudioRejectsMissingName() {
        assertThrows(IllegalArgumentException::class.java) {
            firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = null)
        }
    }

    // What:     `folderNoAudioRejectsBlankName` checks that a whitespace-only folder name throws.
    // Why:      A blank name would produce a headline that ends with nothing after "in".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank name", () => expect(() => firstRunCopy("FOLDER_NO_AUDIO", "   ")).toThrow());
    // ```
    /** Confirms a blank folder name is rejected for the folder no-audio state. */
    @Test
    fun folderNoAudioRejectsBlankName() {
        assertThrows(IllegalArgumentException::class.java) {
            firstRunCopy(state = FirstRunState.FOLDER_NO_AUDIO, folderName = "   ")
        }
    }

    // What:     `folderNameIsIgnoredByOtherStates` checks that the device states do not use the folder name.
    // Why:      A device headline must never name a folder, even when one is passed in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ignored name", () => expect(firstRunCopy("SYSTEM_NO_AUDIO", "Albums").title).toBe(...));
    // ```
    /** Confirms the device no-audio headline ignores a supplied folder name. */
    @Test
    fun folderNameIsIgnoredByOtherStates() {
        assertEquals(
            "No audio found in the device music library",
            firstRunCopy(state = FirstRunState.SYSTEM_NO_AUDIO, folderName = "Albums").title,
        )
    }

    // What:     `noStateOffersAnalysisChoice` checks that no action label asks whether to analyse.
    // Why:      D84 makes analysis automatic, so no first-run action may ask about it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no analysis choice", () => expect(labels.some(l => /analys/i.test(l))).toBe(false));
    // ```
    /** Confirms no action label in any state offers a choice about analysis. */
    @Test
    fun noStateOffersAnalysisChoice() {
        val folderNames: List<String> = listOf("Albums")
        FirstRunState.entries.forEach { state ->
            folderNames.forEach { folder ->
                val copy: FirstRunCopy = firstRunCopy(state = state, folderName = folder)
                val labels: List<String> = listOfNotNull(copy.primaryAction) + copy.secondaryActions
                labels.forEach { label ->
                    assertFalse(
                        "$state offered an analysis action: $label",
                        label.contains("analys", ignoreCase = true),
                    )
                }
            }
        }
    }

    // What:     `onlyDeclinedExplainsAnalysis` checks that the analysis explanation appears only once.
    // Why:      D101 places the explanation under the declined buttons, and no other state draws it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("analysis flag", () => expect(states.filter(s => s.explainsAnalysis)).toEqual(["DECLINED"]));
    // ```
    /** Confirms the true-peak explanation is shown only by the declined state. */
    @Test
    fun onlyDeclinedExplainsAnalysis() {
        val explaining: List<FirstRunState> = FirstRunState.entries.filter { state ->
            firstRunCopy(state = state, folderName = "Albums").explainsAnalysis
        }
        assertEquals(listOf(FirstRunState.DECLINED), explaining)
    }

    // What:     `everyStateOffersSettings` checks that each state keeps a path to in-app Settings.
    // Why:      No first-run state may lack a way forward, so Settings stays in every state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("settings", () => states.forEach(s => expect(copy(s).secondaryActions).toContain("Settings")));
    // ```
    /** Confirms every first-run state lists Settings among its actions. */
    @Test
    fun everyStateOffersSettings() {
        FirstRunState.entries.forEach { state ->
            val copy: FirstRunCopy = firstRunCopy(state = state, folderName = "Albums")
            assertTrue("$state has no Settings action", copy.secondaryActions.contains(SETTINGS_LABEL))
        }
    }

    // What:     `analysisExplanationIsExact` checks the true-peak sentence word for word.
    // Why:      The study sentence is the approved explanation and must not be paraphrased.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("analysis text", () => expect(ANALYSIS_EXPLANATION).toBe("True peak is measured automatically ..."));
    // ```
    /** Confirms the true-peak explanation matches the study sentence exactly. */
    @Test
    fun analysisExplanationIsExact() {
        assertEquals(
            "True peak is measured automatically for every audio file. " +
                "Analysis uses CPU and battery; playback remains available while it runs.",
            ANALYSIS_EXPLANATION,
        )
    }
}
