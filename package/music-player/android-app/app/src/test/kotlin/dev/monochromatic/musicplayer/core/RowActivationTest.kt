package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Test

/** Pins the rule of D129: the current track toggles, any other track is selected and starts only with the setting on. */
class RowActivationTest {
    /** The current track toggles whether or not Auto-play on select is on. */
    @Test
    fun theCurrentTrackAlwaysToggles() {
        assertEquals(RowActivation.TOGGLE_PLAY, rowActivationFor(autoPlayOnSelect = false, isCurrent = true))
        assertEquals(RowActivation.TOGGLE_PLAY, rowActivationFor(autoPlayOnSelect = true, isCurrent = true))
    }

    /** With the setting off, another track is only selected. */
    @Test
    fun anotherTrackIsOnlySelectedWithTheSettingOff() {
        assertEquals(RowActivation.SELECT_ONLY, rowActivationFor(autoPlayOnSelect = false, isCurrent = false))
    }

    /** With the setting on, another track is selected and starts. */
    @Test
    fun anotherTrackStartsWithTheSettingOn() {
        assertEquals(RowActivation.SELECT_AND_PLAY, rowActivationFor(autoPlayOnSelect = true, isCurrent = false))
    }

    /** The supporting line of the setting differs by state and each text states what selecting does. */
    @Test
    fun theSupportingTextNamesTheCurrentChoice() {
        assertEquals("Select first, then Play", autoPlayOnSelectSupportingText(enabled = false))
        assertEquals("Select and play at once", autoPlayOnSelectSupportingText(enabled = true))
    }
}
