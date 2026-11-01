// Wiring cases of the analysis-failure report (D123): the player host never turns it into an in-app message, while
// the tap-time failure to open a track (D83) still shows one.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `playerHost`, `FeedbackHostState` and the report entry point of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import android.Manifest
import android.app.Application
import android.app.Notification
import android.app.NotificationManager
import android.content.Context
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.AnalysisFailures
import dev.monochromatic.musicplayer.FakeAudioEngine
import dev.monochromatic.musicplayer.FeedbackHostState
import dev.monochromatic.musicplayer.LoadFailure
import dev.monochromatic.musicplayer.PlayerController
import dev.monochromatic.musicplayer.Track
import dev.monochromatic.musicplayer.core.FeedbackMessage
import dev.monochromatic.musicplayer.core.TRACK_LOAD_ERROR_ID
import dev.monochromatic.musicplayer.playerHost
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config

// What:     `class AnalysisFailureWiringTest` groups the cases of a failed analysis reaching the real player host.
// Why:      The host used to turn a pending failure into an in-app message in a `LaunchedEffect`; a mutant or a
//           revert that brings that path back keeps every pure-function test green, so each case reads the feedback
//           queue and the screen of the real host after a report.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("analysis failure wiring", () => { /* host rendered, failure reported, queue and notifications read */ });
// ```
/** Renders [playerHost], reports a failed analysis, and reads the feedback queue, the screen and the notifications. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = "w411dp-h891dp-xxhdpi")
class AnalysisFailureWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Application context of the Robolectric app, which the report posts its notification through. */
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** Fake engine, kept so a case can make a tapped track fail to open. */
    private val engine = FakeAudioEngine()

    /** Controller over the fake engine, so no native library or audio device is involved. */
    private val controller = PlayerController(engine)

    /** Feedback host that the player host shows its messages through; the cases read its queue. */
    private val feedback = FeedbackHostState()

    /** Grants the notification permission, opens a one-track invented library and shows the real host over it. */
    @Before
    fun showHost() {
        shadowOf(context as Application).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
        controller.openLibrary(listOf(Track(uri = "content://invented/a/0", displayPath = "Alpha/Track 00.mp3")))
        composeTestRule.setContent {
            MaterialTheme { playerHost(controller = controller, feedback = feedback, onChooseFolder = { }) }
        }
        composeTestRule.waitForIdle()
    }

    /** Reports [names] as failed analyses on the UI thread and waits for the frames it causes. */
    private fun report(names: List<String>) {
        composeTestRule.runOnIdle { AnalysisFailures.report(context, names) }
        composeTestRule.waitForIdle()
    }

    /** Texts of the messages the host has put in the feedback queue. */
    private fun messages(): List<FeedbackMessage> = feedback.queue.visible()

    /** Count of nodes on the screen whose text contains the analysis wording. */
    private fun analysisNodes(): Int =
        composeTestRule.onAllNodes(hasText("Could not analyze", substring = true)).fetchSemanticsNodes().size

    /** Notifications the app has posted so far. */
    private fun posted(): List<Notification> =
        shadowOf(context.getSystemService(NotificationManager::class.java)).allNotifications

    @Test(timeout = CASE_LIMIT_MS)
    fun aReportedAnalysisFailureIsASystemNotificationAndNotAnInAppMessage() {
        report(listOf("broken.mp3"))
        assertEquals(emptyList<FeedbackMessage>(), messages())
        assertEquals(0, analysisNodes())
        assertEquals(
            listOf("Could not analyze broken.mp3."),
            posted().map { notification -> notification.extras.getCharSequence(Notification.EXTRA_TITLE).toString() },
        )
    }

    @Test(timeout = CASE_LIMIT_MS)
    fun withNotificationsOffNothingIsPostedAndNothingAppearsInTheApp() {
        shadowOf(context.getSystemService(NotificationManager::class.java)).setNotificationsEnabled(false)
        report(listOf("broken.mp3", "other.mp3"))
        assertEquals(emptyList<FeedbackMessage>(), messages())
        assertEquals(0, analysisNodes())
        assertEquals(0, posted().size)
    }

    @Test(timeout = CASE_LIMIT_MS)
    fun theTapTimeFailureToPlayStillShowsInTheAppAloneBesideAReport() {
        report(listOf("broken.mp3"))
        composeTestRule.runOnIdle { engine.failLoad(uri = "content://invented/a/0", reason = LoadFailure.UNREADABLE) }
        composeTestRule.waitForIdle()
        assertEquals(listOf(TRACK_LOAD_ERROR_ID), messages().map { message -> message.id })
        assertEquals("Could not play Track 00.mp3.\nIts file could not be opened.", messages().single().text)
        assertEquals(0, analysisNodes())
    }
}
