package dev.monochromatic.musicplayer

import androidx.media3.common.Player
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** The notification controller loses only the timeline read, so media3 publishes no platform queue. */
class PlatformCommandPolicyTest {
    /** Commands the app advertises that the platform session needs for transport, position and metadata. */
    private val others: List<Int> = listOf(
        Player.COMMAND_PLAY_PAUSE,
        Player.COMMAND_PREPARE,
        Player.COMMAND_STOP,
        Player.COMMAND_SEEK_TO_NEXT,
        Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM,
        Player.COMMAND_SEEK_TO_PREVIOUS,
        Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM,
        Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM,
        Player.COMMAND_SEEK_TO_MEDIA_ITEM,
        Player.COMMAND_GET_CURRENT_MEDIA_ITEM,
        Player.COMMAND_GET_METADATA,
        Player.COMMAND_SET_REPEAT_MODE,
        Player.COMMAND_SET_SHUFFLE_MODE,
    )

    @Test
    fun theNotificationControllerLosesTheTimelineRead() {
        assertFalse(keepsPlatformCommand(isMediaNotificationController = true, code = Player.COMMAND_GET_TIMELINE))
    }

    @Test
    fun theNotificationControllerKeepsEveryOtherCommand() {
        for (code in others) {
            assertTrue("command $code", keepsPlatformCommand(isMediaNotificationController = true, code = code))
        }
    }

    @Test
    fun theNotificationControllerKeepsTheCurrentItemReadAndTheSeekCommands() {
        assertTrue(keepsPlatformCommand(true, Player.COMMAND_GET_CURRENT_MEDIA_ITEM))
        assertTrue(keepsPlatformCommand(true, Player.COMMAND_SEEK_TO_NEXT))
        assertTrue(keepsPlatformCommand(true, Player.COMMAND_SEEK_TO_PREVIOUS))
        assertTrue(keepsPlatformCommand(true, Player.COMMAND_SEEK_TO_MEDIA_ITEM))
    }

    @Test
    fun anotherControllerKeepsTheTimelineRead() {
        assertTrue(keepsPlatformCommand(isMediaNotificationController = false, code = Player.COMMAND_GET_TIMELINE))
    }

    @Test
    fun anotherControllerKeepsEveryOtherCommand() {
        for (code in others) {
            assertTrue("command $code", keepsPlatformCommand(isMediaNotificationController = false, code = code))
        }
    }

    @Test
    fun everyDefinedCommandCodeButTheTimelineReadIsKeptForTheNotificationController() {
        val dropped = (0..40).filter { code ->
            !keepsPlatformCommand(isMediaNotificationController = true, code = code)
        }
        assertTrue("dropped $dropped", dropped == listOf(Player.COMMAND_GET_TIMELINE))
    }

    @Test
    fun noCommandCodeIsDroppedForAnotherController() {
        val dropped = (0..40).filter { code ->
            !keepsPlatformCommand(isMediaNotificationController = false, code = code)
        }
        assertTrue("dropped $dropped", dropped.isEmpty())
    }
}
