// Wiring cases of the Licenses row and page, rendered for real under Robolectric over a controller that drives a fake
// engine: the About group at the end of Settings, the row as a button, opening and leaving the page by the Back button,
// the system Back and Escape, the notice text in the lazy list, the loading and failure states, the keyboard scroll,
// and the packed asset. The notice text of the page cases is an invented fixture in the format of the real file.

package dev.monochromatic.musicplayer.wiring

import androidx.activity.ComponentActivity
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.input.InputModeManager
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.requestFocus
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.FeedbackHostState
import dev.monochromatic.musicplayer.LICENSES_BACK_DESCRIPTION
import dev.monochromatic.musicplayer.LICENSES_FAILED_TEXT
import dev.monochromatic.musicplayer.LICENSES_LIST_TAG
import dev.monochromatic.musicplayer.LICENSES_LOADING_TEXT
import dev.monochromatic.musicplayer.LICENSES_ROW_SUPPORTING_TEXT
import dev.monochromatic.musicplayer.LICENSES_TITLE
import dev.monochromatic.musicplayer.PlayerController
import dev.monochromatic.musicplayer.ReportingAudioEngine
import dev.monochromatic.musicplayer.SessionStore
import dev.monochromatic.musicplayer.SweepProgress
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.playerHost
import dev.monochromatic.musicplayer.playerTheme
import dev.monochromatic.musicplayer.readLicenseNoticesAsset
import java.io.IOException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The folded cover in light: 411 dp wide, 3 px per dp. */
private const val COVER_LIGHT = "w411dp-h891dp-xxhdpi"

/** The folded cover in dark. */
private const val COVER_DARK = "w411dp-h891dp-night-xxhdpi"

/** The unfolded inner panel, where Settings shows two panes. */
private const val INNER_PANEL = "w851dp-h700dp-xxhdpi"

/** Pixels per dp of the qualifiers above. */
private const val PIXELS_PER_DP: Int = 3

/** Spoken label of the top bar's Settings button. */
private const val SETTINGS_BUTTON_LABEL = "Settings"

/** A library coordinate that the real notice file lists, so the page is checked against a name a user can look for. */
private const val KNOWN_LIBRARY = "androidx.media3:media3-session:1.11.1"

/** A library that exists nowhere, the only other entry of the fixture. */
private const val INVENTED_LIBRARY = "invented.example:inventedlib:1.0.0"

/** Text of the last block of the fixture, far below the first screen. */
private const val LAST_ENTRY = "Final invented entry"

/** Rule of equal signs, as wide as the real file draws it. */
private val EQUALS_RULE: String = "=".repeat(78)

/** Rule of hyphens, as wide as the real file draws it. */
private val HYPHEN_RULE: String = "-".repeat(78)

/**
 * Notice text in the format of the real file: a title section, an Android libraries section with the two entries, and
 * 400 invented paragraphs that push [LAST_ENTRY] far off screen.
 */
private val FIXTURE_NOTICES: String = buildString {
    appendLine(EQUALS_RULE)
    appendLine("Invented Player 0.0.0 for Android: license notices")
    appendLine(EQUALS_RULE)
    appendLine()
    appendLine("This invented file lists the licenses of an invented player and of nothing else that anyone has made.")
    appendLine("Its lines are wrapped the way the real file wraps prose, so the page joins them into one paragraph.")
    appendLine()
    appendLine(EQUALS_RULE)
    appendLine("Android libraries (2 artifacts)")
    appendLine(EQUALS_RULE)
    appendLine()
    appendLine("Under Apache-2.0: 2 artifacts")
    appendLine(HYPHEN_RULE)
    appendLine()
    appendLine(KNOWN_LIBRARY)
    appendLine(INVENTED_LIBRARY)
    appendLine()
    (1..400).forEach { number ->
        appendLine("Invented paragraph $number says nothing true about any license, and it is long enough to wrap.")
        appendLine("Its second line is also long enough to count as a wrapped line of prose in the page's reader.")
        appendLine("end of paragraph $number")
        appendLine()
    }
    appendLine(LAST_ENTRY)
}

/** Renders the real host and the Licenses row and page, and checks them through what the screen shows. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = COVER_LIGHT)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class LicensesWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity and exposes it for system Back. */
    @get:Rule
    val composeTestRule = createAndroidComposeRule<ComponentActivity>()

    /** Engine that reports playing changes the way the real one does. */
    private val engine = ReportingAudioEngine()

    /** Controller over [engine], so no native library or audio device is involved. */
    private val controller = PlayerController(engine)

    /** Input mode switch of the composition, to move from touch to keyboard input. */
    private lateinit var inputModes: InputModeManager

    /** Starts every case with a fresh install's settings and an idle sweep. */
    @Before
    fun resetProcessState() {
        /** Application context whose preference file holds the settings. */
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        SessionStore.saveTemplate(context, TemplateKind.TRACK_ROWS, "\$mi(ext)\$")
        SweepProgress.reset()
    }

    /** Shows the real host, whose Licenses page reads its text with [notices], and waits for its first frames. */
    private fun showHost(notices: suspend () -> String = { FIXTURE_NOTICES }) {
        composeTestRule.setContent {
            inputModes = LocalInputModeManager.current
            playerTheme {
                playerHost(
                    controller = controller,
                    feedback = FeedbackHostState(),
                    onChooseFolder = { },
                    readLicenseNotices = { _ -> notices() },
                )
            }
        }
        composeTestRule.waitForIdle()
    }

    /** Opens the Settings page from the top bar. */
    private fun openSettings() {
        composeTestRule.onNodeWithContentDescription(SETTINGS_BUTTON_LABEL).performClick()
        composeTestRule.waitForIdle()
    }

    /** The Licenses row of Settings: the clickable node that carries the title and the supporting line. */
    private fun licensesRow(): SemanticsNodeInteraction =
        composeTestRule.onNode(hasText(LICENSES_TITLE) and hasText(LICENSES_ROW_SUPPORTING_TEXT) and hasClickAction())

    /** Taps the Licenses row and waits for the frames the tap causes. */
    private fun openLicenses() {
        licensesRow().performClick()
        composeTestRule.waitForIdle()
    }

    /** Waits until the list shows the fixture's known library, which the page reads off the main thread. */
    private fun waitForNotices() {
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodesWithText(KNOWN_LIBRARY, substring = true).fetchSemanticsNodes().isNotEmpty()
        }
    }

    /** Switches the composition to keyboard input, where the ring shows. */
    private fun useKeyboard() {
        composeTestRule.runOnUiThread { inputModes.requestInputMode(InputMode.Keyboard) }
        composeTestRule.waitForIdle()
    }

    /** True when a node with exactly [text] is on screen. */
    private fun hasNodeWithText(text: String): Boolean =
        composeTestRule.onAllNodesWithText(text).fetchSemanticsNodes().isNotEmpty()

    //region The row

    /** The About group comes after the Playback group, and its row is a button labelled by its title. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theRowSitsInAnAboutGroupAfterPlayback() {
        showHost()
        openSettings()
        composeTestRule.onNodeWithText("About").assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Heading))
        licensesRow().assert(SemanticsMatcher.expectValue(SemanticsProperties.Role, Role.Button))
        val aboutTop = composeTestRule.onNodeWithText("About").getUnclippedBoundsInRoot().top
        val switchBottom = composeTestRule.onNode(isToggleable()).getUnclippedBoundsInRoot().bottom
        assertTrue("About starts below the Playback row: $aboutTop and $switchBottom", aboutTop >= switchBottom)
        val rowTop = licensesRow().getUnclippedBoundsInRoot().top
        assertTrue("the row sits under its heading: $rowTop and $aboutTop", rowTop >= aboutTop)
    }

    /** The row is 72 dp tall, like the rows above it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theRowIsAsTallAsTheOtherRows() {
        showHost()
        openSettings()
        val bounds = licensesRow().getUnclippedBoundsInRoot()
        assertEquals("one line each keeps the 72 dp row", 72f, bounds.bottom.value - bounds.top.value, 0.5f)
    }

    /** Tab reaches the row from the switch row of Playback, and Enter on it opens the page. */
    @Test(timeout = CASE_LIMIT_MS)
    fun tabReachesTheRowAfterTheSwitchAndEnterOpensThePage() {
        showHost()
        openSettings()
        useKeyboard()
        composeTestRule.onNode(isToggleable()).requestFocus()
        composeTestRule.onNode(isToggleable()).performKeyInput { pressKey(Key.Tab) }
        composeTestRule.waitForIdle()
        licensesRow().assertIsFocused()
        licensesRow().performKeyInput { pressKey(Key.Enter) }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(LICENSES_BACK_DESCRIPTION).assertExists()
    }

    //endregion

    //region The page

    /** A tap opens the page: its header, its Back label, the section titles and a known library are there. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTapOpensThePageWithTheNotices() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        composeTestRule.onNodeWithContentDescription(LICENSES_BACK_DESCRIPTION).assertExists()
        composeTestRule.onNodeWithText(LICENSES_TITLE).assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Heading))
        composeTestRule.onNodeWithText("Android libraries (2 artifacts)")
            .assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Heading))
        composeTestRule.onNodeWithText("Under Apache-2.0: 2 artifacts")
            .assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Heading))
        composeTestRule.onNodeWithText(KNOWN_LIBRARY, substring = true).assertExists()
        composeTestRule.onNodeWithText(INVENTED_LIBRARY, substring = true).assertExists()
        assertTrue("the Settings groups are gone", !hasNodeWithText("Templates"))
    }

    /** The Back button returns to Settings, whose groups show again. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theBackButtonReturnsToSettings() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        composeTestRule.onNodeWithContentDescription(LICENSES_BACK_DESCRIPTION).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
        licensesRow().assertExists()
        composeTestRule.onNodeWithContentDescription("Back to player").assertExists()
    }

    /** The system Back returns to Settings, and a second system Back returns to the player. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSystemBackReturnsToSettingsThenToThePlayer() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        composeTestRule.runOnUiThread { composeTestRule.activity.onBackPressedDispatcher.onBackPressed() }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
        composeTestRule.runOnUiThread { composeTestRule.activity.onBackPressedDispatcher.onBackPressed() }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SETTINGS_BUTTON_LABEL).assertExists()
        assertTrue("Settings is gone", !hasNodeWithText("Templates"))
    }

    /** The page shows a progress indicator with words while it loads, then the list. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theLoadingStateShowsWordsUntilTheTextArrives() {
        /** Text that arrives when the test releases it. */
        val gate = CompletableDeferred<String>()
        showHost(notices = { gate.await() })
        openSettings()
        openLicenses()
        composeTestRule.onNodeWithText(LICENSES_LOADING_TEXT).assertExists()
        gate.complete(FIXTURE_NOTICES)
        waitForNotices()
        assertTrue("the loading words are gone", !hasNodeWithText(LICENSES_LOADING_TEXT))
    }

    /** A notice file that cannot be read shows the failure message and keeps Back working. */
    @Test(timeout = CASE_LIMIT_MS)
    fun anUnreadableFileShowsTheFailureMessage() {
        showHost(notices = { throw IOException("invented failure") })
        openSettings()
        openLicenses()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) { hasNodeWithText(LICENSES_FAILED_TEXT) }
        composeTestRule.onNodeWithContentDescription(LICENSES_BACK_DESCRIPTION).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
    }

    /** The list draws only the blocks near the top: the last entry of the 400 paragraphs is not composed yet. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theListIsLazy() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        assertTrue("the last entry is not composed", !hasNodeWithText(LAST_ENTRY))
    }

    /** The list is one keyboard stop: End scrolls to the last entry and Home returns to the first. */
    @Test(timeout = CASE_LIMIT_MS)
    fun endAndHomeScrollTheFocusedList() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        useKeyboard()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).requestFocus()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).assertIsFocused()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).performKeyInput { pressKey(Key.MoveEnd) }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText(LAST_ENTRY, substring = true).assertExists()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).performKeyInput { pressKey(Key.MoveHome) }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText(KNOWN_LIBRARY, substring = true).assertExists()
    }

    /** Page Down moves the list by most of a screen, so the first entry leaves the screen. */
    @Test(timeout = CASE_LIMIT_MS)
    fun pageDownMovesTheListByAScreen() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        useKeyboard()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).requestFocus()
        repeat(times = 3) {
            composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).performKeyInput { pressKey(Key.PageDown) }
            composeTestRule.waitForIdle()
        }
        assertTrue("the first entries scrolled away", !hasNodeWithText("Android libraries (2 artifacts)"))
    }

    /** In dark the page is pure black, like the Settings page it opens from. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = COVER_DARK)
    fun theDarkPageIsTrueBlack() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        val image = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("the page is pure black", 0xFF000000.toInt(), image[4, 40 * PIXELS_PER_DP].toArgb())
    }

    /** On the unfolded two-pane screen the row opens the same page and Back returns to the two panes. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER_PANEL)
    fun onTheUnfoldedScreenBackReturnsToTheTwoPaneSettings() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        composeTestRule.onNodeWithContentDescription(LICENSES_BACK_DESCRIPTION).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
        composeTestRule.onNode(hasText("Track rows") and hasClickAction()).assertExists()
    }

    /** On the unfolded screen the system Back leaves the page for Settings, not for the player. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER_PANEL)
    fun onTheUnfoldedScreenTheSystemBackReturnsToSettings() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        composeTestRule.runOnUiThread { composeTestRule.activity.onBackPressedDispatcher.onBackPressed() }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
    }

    /** On the unfolded screen Escape leaves the page for Settings, as it closes the wide Settings page. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER_PANEL)
    fun onTheUnfoldedScreenEscapeReturnsToSettings() {
        showHost()
        openSettings()
        openLicenses()
        waitForNotices()
        useKeyboard()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).requestFocus()
        composeTestRule.onNodeWithTag(LICENSES_LIST_TAG).performKeyInput { pressKey(Key.Escape) }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Templates").assertExists()
    }

    //endregion

    //region The packed asset

    /** The asset the build packs into every variant opens, and holds the notices or the development note. */
    @Test(timeout = CASE_LIMIT_MS)
    fun thePackedAssetIsReadable() {
        val text: String = runBlocking {
            readLicenseNoticesAsset(ApplicationProvider.getApplicationContext())
        }
        assertTrue(
            "the asset holds the notices or the note of a development build, but starts: ${text.take(80)}",
            text.contains("Android libraries (") || text.contains("carries no third-party license notices"),
        )
    }

    //endregion
}
