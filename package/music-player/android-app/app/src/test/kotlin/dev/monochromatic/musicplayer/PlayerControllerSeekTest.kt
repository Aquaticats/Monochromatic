package dev.monochromatic.musicplayer

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * A seek made from the app's own scrubber reaches the media session projection. The scrubber, the seek buttons and
 * restart call `PlayerController.seek` directly, so without the announcement the session kept the position from
 * before a seek made while paused.
 */
class PlayerControllerSeekTest {
    /** Engine double that records the last seek. */
    private val engine = FakeAudioEngine()

    /** Controller under test, over [engine]. */
    private val controller = PlayerController(engine)

    /** A seek announces a state change once. */
    @Test
    fun aSeekAnnouncesOneStateChange() {
        var announcements = 0
        controller.onStateChanged = { announcements += 1 }
        controller.seek(42.0)
        assertEquals(1, announcements)
    }

    /** The announcement comes after the engine was told, so the pull that follows sees the seek. */
    @Test
    fun theEngineIsToldBeforeTheAnnouncement() {
        var seekSeenByTheAnnouncement: Double? = null
        controller.onStateChanged = { seekSeenByTheAnnouncement = engine.lastSeek }
        controller.seek(42.0)
        assertEquals(42.0, seekSeenByTheAnnouncement)
    }

    /** A seek does not save the session: the scrubber may seek many times in one drag. */
    @Test
    fun aSeekDoesNotSaveTheSession() {
        var saves = 0
        controller.onPersist = { saves += 1 }
        controller.seek(42.0)
        assertEquals(0, saves)
    }

    /** Without a media session projection a seek still reaches the engine. */
    @Test
    fun aSeekWithoutAProjectionStillReachesTheEngine() {
        controller.seek(7.5)
        assertEquals(7.5, engine.lastSeek)
    }
}
