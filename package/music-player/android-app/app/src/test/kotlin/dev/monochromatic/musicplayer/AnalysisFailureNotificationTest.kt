package dev.monochromatic.musicplayer

import android.Manifest
import android.app.Application
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.util.Log
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.core.UnplayableReport
import dev.monochromatic.musicplayer.core.analysisNotificationContent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.shadows.ShadowLog

/** D123: the report posts one notification on its own channel, or logs one warning and posts nothing. */
@RunWith(RobolectricTestRunner::class)
class AnalysisFailureNotificationTest {
    /** Application context of the Robolectric app. */
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** The system notification service of the Robolectric app. */
    private val manager: NotificationManager = context.getSystemService(NotificationManager::class.java)

    /** Grants the notification permission, which a Robolectric app does not hold at API 33 and later by default. */
    @Before
    fun grantTheNotificationPermission() {
        shadowOf(context as Application).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
    }

    /** Notifications the app has posted now, as the shadow records them. */
    private fun posted(): List<Notification> = shadowOf(manager).allNotifications

    /** Text of an extra of [notification] as plain text. */
    private fun extra(notification: Notification, key: String): String =
        notification.extras.getCharSequence(key).toString()

    /** Warning lines the app logged under the sweep tag. */
    private fun warnings(): List<String> = ShadowLog.getLogs()
        .filter { entry -> entry.type == Log.WARN && entry.tag == "PeakSweep" }
        .map { entry -> entry.msg }

    @Test(timeout = 60_000)
    fun oneNotificationNamesTheFileAndOpensTheAppWhenTapped() {
        AnalysisFailures.report(context, listOf("broken.mp3"))
        val notification: Notification = posted().single()
        assertEquals("Could not analyze broken.mp3.", extra(notification, Notification.EXTRA_TITLE))
        assertEquals(
            "Its file could not be decoded. The app tries again when your library changes.",
            extra(notification, Notification.EXTRA_TEXT),
        )
        assertEquals(
            "Its file could not be decoded. The app tries again when your library changes.",
            extra(notification, Notification.EXTRA_BIG_TEXT),
        )
        assertEquals(Notification.CATEGORY_ERROR, notification.category)
        assertEquals(ANALYSIS_FAILURE_CHANNEL_ID, notification.channelId)
        assertEquals(R.drawable.ic_stat_analysis, notification.smallIcon.resId)
        assertTrue(notification.flags and Notification.FLAG_AUTO_CANCEL != 0)
        assertFalse(notification.flags and Notification.FLAG_ONGOING_EVENT != 0)
        assertTrue(notification.actions == null || notification.actions.isEmpty())
        assertNotNull(notification.contentIntent)
        assertNotNull(shadowOf(manager).getNotification(ANALYSIS_FAILURE_NOTIFICATION_ID))
    }

    @Test(timeout = 60_000)
    fun theChannelIsLowImportanceAndSeparateFromTheProgressChannel() {
        AnalysisFailures.report(context, listOf("broken.mp3"))
        val channel: NotificationChannel = manager.getNotificationChannel(ANALYSIS_FAILURE_CHANNEL_ID)
        assertEquals(NotificationManager.IMPORTANCE_LOW, channel.importance)
        assertEquals("Analysis problems", channel.name.toString())
        assertFalse(ANALYSIS_FAILURE_CHANNEL_ID == ANALYSIS_CHANNEL_ID)
        assertFalse(ANALYSIS_FAILURE_NOTIFICATION_ID == ANALYSIS_NOTIFICATION_ID)
    }

    @Test(timeout = 60_000)
    fun severalFilesOfOneReportAreOneNotification() {
        AnalysisFailures.report(context, listOf("a.mp3", "b.mp3", "c.mp3"))
        val notification: Notification = posted().single()
        assertEquals("Could not analyze 3 files, including a.mp3.", extra(notification, Notification.EXTRA_TITLE))
    }

    @Test(timeout = 60_000)
    fun aLaterReportReplacesAnUnreadOneUnderTheSameId() {
        AnalysisFailures.report(context, listOf("a.mp3"))
        AnalysisFailures.report(context, listOf("b.mp3"))
        assertEquals("Could not analyze b.mp3.", extra(posted().single(), Notification.EXTRA_TITLE))
    }

    @Test(timeout = 60_000)
    fun theProgressNotificationIsNeitherReplacedNorRemoved() {
        /** Content of a running analysis, as the sweep service posts it. */
        val progress = analysisNotificationContent(done = 1, total = 4, paused = false)
        manager.notify(ANALYSIS_NOTIFICATION_ID, buildAnalysisNotification(context, progress))
        AnalysisFailures.report(context, listOf("a.mp3"))
        assertEquals(
            setOf(ANALYSIS_NOTIFICATION_ID, ANALYSIS_FAILURE_NOTIFICATION_ID),
            shadowOf(manager).activeNotifications.map { active -> active.id }.toSet(),
        )
    }

    @Test(timeout = 60_000)
    fun anEmptyReportPostsAndLogsNothing() {
        ShadowLog.clear()
        AnalysisFailures.report(context, emptyList())
        assertTrue(posted().isEmpty())
        assertTrue(warnings().isEmpty())
    }

    @Test(timeout = 60_000)
    fun aDeniedPermissionPostsNothingAndLogsOneWarningWithoutTheFileName() {
        shadowOf(context as Application).denyPermissions(Manifest.permission.POST_NOTIFICATIONS)
        ShadowLog.clear()
        AnalysisFailures.report(context, listOf("secret-name.mp3"))
        assertTrue(posted().isEmpty())
        assertEquals(1, warnings().size)
        assertTrue(warnings().single().contains("PERMISSION_DENIED"))
        assertFalse(warnings().single().contains("secret-name"))
    }

    @Test(timeout = 60_000)
    fun notificationsTurnedOffPostNothingAndLogOneWarning() {
        shadowOf(manager).setNotificationsEnabled(false)
        ShadowLog.clear()
        AnalysisFailures.report(context, listOf("secret-name.mp3"))
        assertTrue(posted().isEmpty())
        assertEquals(1, warnings().size)
        assertTrue(warnings().single().contains("NOTIFICATIONS_DISABLED"))
    }

    @Test(timeout = 60_000)
    fun aChannelTurnedOffPostsNothingAndLogsOneWarning() {
        manager.createNotificationChannel(
            NotificationChannel(ANALYSIS_FAILURE_CHANNEL_ID, "Analysis problems", NotificationManager.IMPORTANCE_NONE),
        )
        ShadowLog.clear()
        AnalysisFailures.report(context, listOf("secret-name.mp3"))
        assertTrue(posted().isEmpty())
        assertEquals(1, warnings().size)
        assertTrue(warnings().single().contains("CHANNEL_DISABLED"))
    }

    /** The report of files the player passed is one notification on the same channel, apart from the analysis one. */
    @Test(timeout = 60_000)
    fun anUnplayableReportIsOneNotificationOnTheSameChannelWithItsOwnId() {
        postUnplayableFilesNotification(context, UnplayableReport(listOf("a.mp3", "b.mp3"), timedOut = false))
        val notification: Notification = posted().single()
        assertEquals("Could not play 2 files, including a.mp3.", extra(notification, Notification.EXTRA_TITLE))
        assertEquals("They could not be opened.", extra(notification, Notification.EXTRA_TEXT))
        assertEquals(ANALYSIS_FAILURE_CHANNEL_ID, notification.channelId)
        assertTrue(notification.flags and Notification.FLAG_AUTO_CANCEL != 0)
        assertNotNull(notification.contentIntent)
        assertNotNull(shadowOf(manager).getNotification(UNPLAYABLE_FILES_NOTIFICATION_ID))
        assertFalse(UNPLAYABLE_FILES_NOTIFICATION_ID == ANALYSIS_FAILURE_NOTIFICATION_ID)
        assertFalse(UNPLAYABLE_FILES_NOTIFICATION_ID == ANALYSIS_NOTIFICATION_ID)
    }

    /** A growing report replaces the earlier one under the same id, so one notification stays. */
    @Test(timeout = 60_000)
    fun aLaterUnplayableReportReplacesTheEarlierOne() {
        postUnplayableFilesNotification(context, UnplayableReport(listOf("a.mp3"), timedOut = false))
        postUnplayableFilesNotification(context, UnplayableReport(listOf("a.mp3", "b.mp3"), timedOut = false))
        assertEquals(
            "Could not play 2 files, including a.mp3.",
            extra(posted().single(), Notification.EXTRA_TITLE),
        )
    }

    /** An unplayable report and an analysis failure are two notifications, neither replacing the other. */
    @Test(timeout = 60_000)
    fun anUnplayableReportAndAnAnalysisFailureCoexist() {
        AnalysisFailures.report(context, listOf("x.mp3"))
        postUnplayableFilesNotification(context, UnplayableReport(listOf("a.mp3"), timedOut = false))
        assertEquals(
            setOf(ANALYSIS_FAILURE_NOTIFICATION_ID, UNPLAYABLE_FILES_NOTIFICATION_ID),
            shadowOf(manager).activeNotifications.map { active -> active.id }.toSet(),
        )
    }

    /** With the permission denied, nothing is posted and nothing is shown in the app: one warning, no file name. */
    @Test(timeout = 60_000)
    fun anUnplayableReportThatCannotBePostedLogsOneWarningWithoutTheFileName() {
        shadowOf(context as Application).denyPermissions(Manifest.permission.POST_NOTIFICATIONS)
        ShadowLog.clear()
        postUnplayableFilesNotification(context, UnplayableReport(listOf("secret-name.mp3"), timedOut = false))
        assertTrue(posted().isEmpty())
        /** Warning lines the app logged under the unplayable-files tag. */
        val lines: List<String> = ShadowLog.getLogs()
            .filter { entry -> entry.type == Log.WARN && entry.tag == "UnplayableFiles" }
            .map { entry -> entry.msg }
        assertEquals(1, lines.size)
        assertTrue(lines.single().contains("PERMISSION_DENIED"))
        assertFalse(lines.single().contains("secret-name"))
    }
}
