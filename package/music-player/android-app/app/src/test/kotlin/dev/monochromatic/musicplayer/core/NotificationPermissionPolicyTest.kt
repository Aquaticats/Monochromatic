package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies when the notification permission prompt is shown, across every input that the rule reads. */
class NotificationPermissionPolicyTest {
    /** Confirms the prompt is shown on Android 13 for a first analysis that has a total. */
    @Test
    fun asksOnApi33ForTheFirstAnalysis() {
        assertTrue(ask(sdkInt = 33, granted = false, askedBefore = false, total = 1218))
        assertTrue(ask(sdkInt = 36, granted = false, askedBefore = false, total = 1))
    }

    /** Confirms nothing is asked below Android 13, where notifications need no permission. */
    @Test
    fun neverAsksBelowApi33() {
        assertFalse(ask(sdkInt = 32, granted = false, askedBefore = false, total = 1218))
        assertFalse(ask(sdkInt = 26, granted = false, askedBefore = false, total = 1218))
    }

    /** Confirms nothing is asked when the permission is already held. */
    @Test
    fun neverAsksWhenGranted() {
        assertFalse(ask(sdkInt = 34, granted = true, askedBefore = false, total = 1218))
    }

    /** Confirms a recorded earlier ask ends asking, so a denial is not asked again. */
    @Test
    fun neverAsksTwice() {
        assertFalse(ask(sdkInt = 34, granted = false, askedBefore = true, total = 1218))
    }

    /** Confirms nothing is asked before an analysis has a total. */
    @Test
    fun waitsForTheAnalysisToStart() {
        assertFalse(ask(sdkInt = 34, granted = false, askedBefore = false, total = 0))
        assertFalse(ask(sdkInt = 34, granted = false, askedBefore = false, total = -1))
    }

    /** Confirms the first level that prompts is the one the constant names. */
    @Test
    fun thresholdIsApi33() {
        assertTrue(NOTIFICATION_PERMISSION_MIN_SDK == 33)
    }

    /** Calls the rule with named arguments. */
    private fun ask(sdkInt: Int, granted: Boolean, askedBefore: Boolean, total: Int): Boolean =
        shouldAskForNotificationPermission(
            sdkInt = sdkInt,
            granted = granted,
            askedBefore = askedBefore,
            analysisTotal = total,
        )
}
