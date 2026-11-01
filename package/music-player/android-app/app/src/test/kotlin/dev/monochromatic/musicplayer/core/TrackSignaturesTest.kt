package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

/** The signature that tells an edited file from a measured one. */
class TrackSignaturesTest {
    @Test
    fun aDifferentSizeOrTimeGivesADifferentSignature() {
        val base = TrackSignatures.signatureOf(sizeBytes = 1000L, modifiedMs = 5_000L)
        assertNotEquals(base, TrackSignatures.signatureOf(sizeBytes = 1001L, modifiedMs = 5_000L))
        assertNotEquals(base, TrackSignatures.signatureOf(sizeBytes = 1000L, modifiedMs = 6_000L))
        assertEquals(base, TrackSignatures.signatureOf(sizeBytes = 1000L, modifiedMs = 5_000L))
    }

    @Test
    fun unknownSizeAndTimeGiveZeroSoTheUriAloneIdentifiesTheFile() {
        assertEquals(0L, TrackSignatures.signatureOf(sizeBytes = 0L, modifiedMs = 0L))
    }

    @Test
    fun recordedSignaturesAreReadBackUntilCleared() {
        TrackSignatures.clear()
        TrackSignatures.record("content://a/1", 42L)
        assertEquals(42L, TrackSignatures.of("content://a/1"))
        assertEquals(0L, TrackSignatures.of("content://a/2"))
        TrackSignatures.clear()
        assertEquals(0L, TrackSignatures.of("content://a/1"))
    }
}
