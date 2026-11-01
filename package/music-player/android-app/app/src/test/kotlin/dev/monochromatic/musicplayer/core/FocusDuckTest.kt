package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Test

/** Pins the volume rule for a short interruption that allows ducking (D134). */
class FocusDuckTest {
    /** Without a duck the engine plays at the user volume, whatever it is. */
    @Test
    fun anUnduckedPlayerKeepsTheUserVolume() {
        assertEquals(1.0f, volumeWhileDucked(userVolume = 1.0f, ducked = false), 0.0f)
        assertEquals(0.35f, volumeWhileDucked(userVolume = 0.35f, ducked = false), 0.0f)
        assertEquals(0.0f, volumeWhileDucked(userVolume = 0.0f, ducked = false), 0.0f)
    }

    /** A duck lowers the user volume to a fifth, the level Android's own ducking reaches. */
    @Test
    fun aDuckedPlayerPlaysAtAFifthOfTheUserVolume() {
        assertEquals(0.2f, volumeWhileDucked(userVolume = 1.0f, ducked = true), 1e-6f)
        assertEquals(0.1f, volumeWhileDucked(userVolume = 0.5f, ducked = true), 1e-6f)
        assertEquals(0.0f, volumeWhileDucked(userVolume = 0.0f, ducked = true), 0.0f)
    }

    /** The factor is the framework's duck level, so a change to it fails here and is noticed. */
    @Test
    fun theFactorIsTheFrameworkDuckLevel() {
        assertEquals(0.2f, DUCKED_VOLUME_FACTOR, 0.0f)
    }
}
