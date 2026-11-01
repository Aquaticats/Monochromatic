// Wiring cases of the Auto-play on select setting (D129), rendered for real under Robolectric over a controller that
// drives a fake engine: the Settings switch row (default, tap, keys, TalkBack semantics, ring, in-place toggle) and
// what a tap, Enter and a Search result do to playback with the setting on and off.

package dev.monochromatic.musicplayer.wiring

import android.content.Context
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.input.InputModeManager
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isNotSelected
import androidx.compose.ui.test.isSelected
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.requestFocus
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.AUTO_PLAY_SWITCH_TAG
import dev.monochromatic.musicplayer.FeedbackHostState
import dev.monochromatic.musicplayer.PlayerController
import dev.monochromatic.musicplayer.ReportingAudioEngine
import dev.monochromatic.musicplayer.SearchHost
import dev.monochromatic.musicplayer.SessionStore
import dev.monochromatic.musicplayer.SweepProgress
import dev.monochromatic.musicplayer.Track
import dev.monochromatic.musicplayer.focusRingTheme
import dev.monochromatic.musicplayer.core.SEARCH_FIELD_LABEL
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.core.deriveRoles
import dev.monochromatic.musicplayer.playerColorScheme
import dev.monochromatic.musicplayer.playerHost
import dev.monochromatic.musicplayer.playerTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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
private const val PIXELS_PER_DP: Float = 3f

/** Title of the setting row, as the user reads and TalkBack speaks it. */
private const val ROW_TITLE = "Auto-play on select"

/** Spoken label of the top bar's Settings button. */
private const val SETTINGS_BUTTON_LABEL = "Settings"

/** Spoken label of the top bar's Search button. */
private const val SEARCH_BUTTON_LABEL = "Search music"

/** Renders the real host and the Settings switch row, and checks the setting through what the screen shows. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = COVER_LIGHT)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class AutoPlaySettingWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Engine that reports playing changes the way the real one does. */
    private val engine = ReportingAudioEngine()

    /** Controller over [engine], so no native library or audio device is involved. */
    private val controller = PlayerController(engine)

    /** Application context whose preference file holds the setting. */
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** Input mode switch of the composition, to move from touch to keyboard input. */
    private lateinit var inputModes: InputModeManager

    /** Colors the host draws with, derived the way the app derives them. */
    private lateinit var scheme: androidx.compose.material3.ColorScheme

    /** Starts every case with the setting saved off, which most cases need, and with an idle sweep. */
    @Before
    fun resetProcessState() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        SessionStore.saveTemplate(context, TemplateKind.TRACK_ROWS, "\$mi(ext)\$")
        SweepProgress.reset()
    }

    /** Invented tracks of one folder, `Alpha/Track NN.mp3`, with content URIs that name nothing real. */
    private fun alpha(count: Int): List<Track> = (0 until count).map { track ->
        Track(uri = "content://invented/alpha/$track", displayPath = "Alpha/${titleOf(track)}.mp3")
    }

    /** Shows the real host in the app theme with its focus ring, and waits until its first frames are composed. */
    private fun showHost() {
        composeTestRule.setContent {
            inputModes = LocalInputModeManager.current
            playerTheme {
                scheme = androidx.compose.material3.MaterialTheme.colorScheme
                playerHost(controller = controller, feedback = FeedbackHostState(), onChooseFolder = { })
            }
        }
        composeTestRule.waitForIdle()
    }

    /** Opens the Settings page from the top bar. */
    private fun openSettings() {
        composeTestRule.onNodeWithContentDescription(SETTINGS_BUTTON_LABEL).performClick()
        composeTestRule.waitForIdle()
    }

    /** The switch row of the Settings page: the only toggleable node. */
    private fun switchRow(): SemanticsNodeInteraction = composeTestRule.onNode(isToggleable())

    /** Switches the composition to keyboard input, where the ring shows. */
    private fun useKeyboard() {
        composeTestRule.runOnUiThread { inputModes.requestInputMode(InputMode.Keyboard) }
        composeTestRule.waitForIdle()
    }

    /** The list row titled for [track], which carries a selected state. */
    private fun trackRow(track: Int): SemanticsNodeInteraction =
        composeTestRule.onNode(hasText(titleOf(track)) and (isSelected() or isNotSelected()))

    /** Taps the row of [track] and waits for the frames the tap causes. */
    private fun tap(track: Int) {
        trackRow(track).performClick()
        composeTestRule.waitForIdle()
    }

    //region Settings switch row

    /** A fresh install, with nothing saved, shows the switch on (D133). */
    @Test(timeout = CASE_LIMIT_MS)
    fun aFreshInstallShowsTheSwitchOn() {
        context.getSharedPreferences("session", Context.MODE_PRIVATE).edit().remove("auto_play_on_select").commit()
        showHost()
        openSettings()
        switchRow().assertIsOn()
        composeTestRule.onNodeWithText("Select and play at once").assertExists()
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
    }

    /** The row is a switch with the title as its label, and it is off when off was saved. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theRowIsAnOffSwitchLabelledByItsTitle() {
        showHost()
        openSettings()
        switchRow().assertIsOff()
        switchRow().assert(SemanticsMatcher.expectValue(SemanticsProperties.Role, Role.Switch))
        switchRow().assert(hasText(ROW_TITLE))
        composeTestRule.onNodeWithText("Playback").assert(SemanticsMatcher.keyIsDefined(SemanticsProperties.Heading))
        assertFalse(SessionStore.loadAutoPlayOnSelect(context))
    }

    /** The row says what the current choice does, and the text follows the state. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSupportingTextFollowsTheState() {
        showHost()
        openSettings()
        composeTestRule.onNodeWithText("Select first, then Play").assertExists()
        switchRow().performClick()
        composeTestRule.onNodeWithText("Select and play at once").assertExists()
    }

    /** A tap flips the switch in place, saves the choice, and a second tap flips it back. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aTapTogglesTheSwitchInPlaceAndSavesIt() {
        showHost()
        openSettings()
        switchRow().performClick()
        composeTestRule.waitForIdle()
        switchRow().assertIsOn()
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
        switchRow().performClick()
        composeTestRule.waitForIdle()
        switchRow().assertIsOff()
        assertFalse(SessionStore.loadAutoPlayOnSelect(context))
    }

    /** A saved on is on when the host first composes, as after a restart. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aSavedOnIsOnWhenTheHostComposes() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        showHost()
        openSettings()
        switchRow().assertIsOn()
    }

    /** Toggling opens no editor, leaves the page on Settings, and plays nothing. */
    @Test(timeout = CASE_LIMIT_MS)
    fun togglingOpensNoEditorAndPlaysNothing() {
        controller.openLibrary(alpha(count = 4))
        showHost()
        openSettings()
        switchRow().performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription("Back to player").assertExists()
        composeTestRule.onNodeWithText("Templates").assertExists()
        assertTrue(
            "no editor page opened",
            composeTestRule.onAllNodes(hasSetTextAction()).fetchSemanticsNodes().isEmpty(),
        )
        assertFalse(engine.playing)
        assertTrue(engine.loads.isEmpty())
    }

    /** Space and Enter on the focused row flip the setting, and Space does not toggle playback. */
    @Test(timeout = CASE_LIMIT_MS)
    fun spaceAndEnterToggleTheFocusedRow() {
        controller.openLibrary(alpha(count = 4))
        showHost()
        openSettings()
        useKeyboard()
        switchRow().requestFocus()
        switchRow().assertIsFocused()
        switchRow().performKeyInput { pressKey(Key.Spacebar) }
        composeTestRule.waitForIdle()
        switchRow().assertIsOn()
        switchRow().performKeyInput { pressKey(Key.Enter) }
        composeTestRule.waitForIdle()
        switchRow().assertIsOff()
        switchRow().performKeyInput { pressKey(Key.Spacebar) }
        composeTestRule.waitForIdle()
        switchRow().assertIsOn()
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
        assertFalse("Space on the row never toggles playback", engine.playing)
        assertTrue(engine.loads.isEmpty())
    }

    /** Tab reaches the row from the last template row, in order, and the arrows are not needed to find it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun tabReachesTheRowAfterTheTemplateRows() {
        showHost()
        openSettings()
        useKeyboard()
        composeTestRule.onNode(hasClickAction() and hasText("Playing track", substring = true)).requestFocus()
        composeTestRule.onNode(hasClickAction() and hasText("Playing track", substring = true))
            .performKeyInput { pressKey(Key.Tab) }
        composeTestRule.waitForIdle()
        switchRow().assertIsFocused()
    }

    /** With keyboard focus the row carries the two rings outside its edge, and without it none. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theKeyboardRingHugsTheRow() {
        showHost()
        openSettings()
        val bounds = switchRow().getUnclippedBoundsInRoot()
        val middleY: Int = ((bounds.top.value + bounds.bottom.value) / 2f * PIXELS_PER_DP).toInt()
        val nearX: Int = (bounds.left.value * PIXELS_PER_DP).toInt() - 3
        val outerX: Int = (bounds.left.value * PIXELS_PER_DP).toInt() - 8
        val canvas: Int = scheme.surface.toArgb()
        val bare = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("no ring before keyboard focus", canvas, bare[nearX, middleY].toArgb())
        useKeyboard()
        switchRow().requestFocus()
        composeTestRule.waitForIdle()
        val image = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("near ring touches the edge", scheme.primary.toArgb(), image[nearX, middleY].toArgb())
        assertEquals("outer ring around it", scheme.onPrimary.toArgb(), image[outerX, middleY].toArgb())
    }

    /** The switch shows on and off by thumb position and track fill, which are two channels besides the mark. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSwitchShowsTheStateByThumbPositionAndFill() {
        showHost()
        openSettings()
        val bounds = switchRow().getUnclippedBoundsInRoot()
        val middleY: Int = ((bounds.top.value + bounds.bottom.value) / 2f * PIXELS_PER_DP).toInt()
        val switchRight: Float = bounds.right.value - 16f
        val leftProbe: Int = ((switchRight - 52f + 6f) * PIXELS_PER_DP).toInt()
        val rightProbe: Int = ((switchRight - 6f) * PIXELS_PER_DP).toInt()
        val leftThumbEdge: Int = ((switchRight - 52f + 6f) * PIXELS_PER_DP).toInt()
        val off = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("off: thumb on the left", scheme.outline.toArgb(), off[leftThumbEdge, middleY].toArgb())
        assertEquals("off: the track is the canvas", scheme.surface.toArgb(), off[rightProbe, middleY].toArgb())
        switchRow().performClick()
        composeTestRule.waitForIdle()
        val on = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("on: the track is the accent", scheme.primary.toArgb(), on[leftProbe, middleY].toArgb())
        assertEquals("on: thumb on the right", scheme.onPrimary.toArgb(), on[rightProbe, middleY].toArgb())
    }

    /**
     * Counts the pixels of [color] in the upper left of the 16 dp mark inside the thumb, where a cross has a stroke
     * and a check mark has none; [thumbLeft] is the thumb's left edge in dp and [switchTop] the switch's top.
     */
    private fun upperLeftMarkPixels(thumbLeft: Float, switchTop: Float, color: Int): Int {
        val image = composeTestRule.onRoot().captureToImage().toPixelMap()
        val markLeft: Float = thumbLeft + 4f
        val markTop: Float = switchTop + 4f + 4f
        var count = 0
        for (x in ((markLeft + 1.3f) * PIXELS_PER_DP).toInt() until ((markLeft + 6.7f) * PIXELS_PER_DP).toInt()) {
            for (y in ((markTop + 1.3f) * PIXELS_PER_DP).toInt() until ((markTop + 6f) * PIXELS_PER_DP).toInt()) {
                if (image[x, y].toArgb() == color) {
                    count += 1
                }
            }
        }
        return count
    }

    /** The thumb holds a cross when off and a check mark when on, the third channel of the state. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theThumbHoldsACrossWhenOffAndACheckMarkWhenOn() {
        showHost()
        openSettings()
        val bounds = switchRow().getUnclippedBoundsInRoot()
        val switchBounds = composeTestRule.onNode(hasTestTag(AUTO_PLAY_SWITCH_TAG), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        val switchLeft: Float = bounds.right.value - 16f - 52f
        val switchTop: Float = switchBounds.top.value
        val crossStroke: Int = upperLeftMarkPixels(switchLeft + 4f, switchTop, scheme.surface.toArgb())
        assertTrue("off: a cross has a stroke in the upper left of the mark, found $crossStroke", crossStroke > 3)
        switchRow().performClick()
        composeTestRule.waitForIdle()
        val checkStroke: Int = upperLeftMarkPixels(switchLeft + 24f, switchTop, scheme.primary.toArgb())
        assertEquals("on: a check mark has none there", 0, checkStroke)
    }

    /** In dark the page is pure black and the switch keeps its two channels on it. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = COVER_DARK)
    fun theDarkRowStaysOnTrueBlackWithBothStatesVisible() {
        showHost()
        openSettings()
        val bounds = switchRow().getUnclippedBoundsInRoot()
        val middleY: Int = ((bounds.top.value + bounds.bottom.value) / 2f * PIXELS_PER_DP).toInt()
        val switchRight: Float = bounds.right.value - 16f
        val leftProbe: Int = ((switchRight - 52f + 6f) * PIXELS_PER_DP).toInt()
        val rightProbe: Int = ((switchRight - 6f) * PIXELS_PER_DP).toInt()
        val off = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("the page is pure black", 0xFF000000.toInt(), off[4, middleY].toArgb())
        assertEquals("off: the track is pure black", 0xFF000000.toInt(), off[rightProbe, middleY].toArgb())
        assertEquals("off: thumb on the left", scheme.outline.toArgb(), off[leftProbe, middleY].toArgb())
        switchRow().performClick()
        composeTestRule.waitForIdle()
        val on = composeTestRule.onRoot().captureToImage().toPixelMap()
        assertEquals("on: the track is the accent", scheme.primary.toArgb(), on[leftProbe, middleY].toArgb())
        assertEquals("on: thumb on the right", scheme.onPrimary.toArgb(), on[rightProbe, middleY].toArgb())
    }

    /** On the unfolded two-pane Settings the switch changes no selection and leaves the selected editor in place. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER_PANEL)
    fun onTheTwoPaneSettingsTheSwitchLeavesTheSelectedTemplateAndItsEditor() {
        showHost()
        openSettings()
        composeTestRule.onNode(hasText("Track rows") and isSelected() and hasClickAction()).assertExists()
        composeTestRule.onAllNodes(hasSetTextAction()).assertCountEquals(1)
        switchRow().assert(!SemanticsMatcher.keyIsDefined(SemanticsProperties.Selected))
        switchRow().performClick()
        composeTestRule.waitForIdle()
        switchRow().assertIsOn()
        composeTestRule.onNode(hasText("Track rows") and isSelected() and hasClickAction()).assertExists()
        composeTestRule.onNode(hasText("Playing track", substring = true) and isNotSelected() and hasClickAction())
            .assertExists()
        composeTestRule.onAllNodes(hasSetTextAction()).assertCountEquals(1)
        // Choosing the other template still works and the switch keeps its state across that.
        composeTestRule.onNode(hasText("Playing track", substring = true) and hasClickAction()).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNode(hasText("Playing track", substring = true) and isSelected() and hasClickAction())
            .assertExists()
        switchRow().assertIsOn()
        switchRow().performClick()
        composeTestRule.waitForIdle()
        switchRow().assertIsOff()
        composeTestRule.onNode(hasText("Playing track", substring = true) and isSelected() and hasClickAction())
            .assertExists()
        composeTestRule.onAllNodes(hasSetTextAction()).assertCountEquals(1)
    }

    /** On the cover the switch sits after the text, on 72 dp, and in the narrow list pane it sits under the text. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSwitchSitsAfterTheTextOnTheCoverAndUnderItInTheNarrowListPane() {
        showHost()
        openSettings()
        val coverSwitch = composeTestRule.onNode(hasTestTag(AUTO_PLAY_SWITCH_TAG), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        val coverLine = composeTestRule.onNode(hasText("Select first, then Play"), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        assertTrue("beside the text on the cover: $coverSwitch and $coverLine", coverSwitch.left >= coverLine.right)
        val rowBounds = switchRow().getUnclippedBoundsInRoot()
        val rowHeight: Float = rowBounds.bottom.value - rowBounds.top.value
        assertEquals("one line each keeps the 72 dp row", 72f, rowHeight, 0.5f)
    }

    /** In the list pane of the unfolded screen, narrower than 300 dp of row, the switch moves under the text. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER_PANEL)
    fun inTheNarrowListPaneTheSwitchSitsUnderTheSupportingLine() {
        showHost()
        openSettings()
        val narrowSwitch = composeTestRule.onNode(hasTestTag(AUTO_PLAY_SWITCH_TAG), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        val narrowLine = composeTestRule.onNode(hasText("Select first, then Play"), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        assertTrue("under the line in the pane: $narrowSwitch and $narrowLine", narrowSwitch.top >= narrowLine.bottom)
        assertTrue("and it starts where the text starts", narrowSwitch.left == narrowLine.left)
    }

    //endregion

    //region Playback

    /** With the setting on, a tap on another row starts it at once, and a tap on the current row toggles it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun withTheSettingOnATapStartsTheRow() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        controller.openLibrary(alpha(count = 6))
        showHost()
        tap(2)
        assertEquals(2, controller.uiState.currentIndex)
        assertTrue(engine.playing)
        assertEquals(listOf("content://invented/alpha/2" to true), engine.loads)
        tap(4)
        assertEquals(4, controller.uiState.currentIndex)
        assertTrue("the next row plays at once, replacing the first", engine.playing)
        assertEquals("content://invented/alpha/4" to true, engine.loads.last())
        tap(4)
        assertFalse("a tap on the current row pauses it", engine.playing)
        assertEquals(2, engine.loads.size)
    }

    /** With the setting off, the same taps only select, as before. */
    @Test(timeout = CASE_LIMIT_MS)
    fun withTheSettingOffATapOnlySelects() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        tap(2)
        assertEquals(2, controller.uiState.currentIndex)
        assertFalse(engine.playing)
        assertEquals(listOf("content://invented/alpha/2" to false), engine.loads)
    }

    /** Toggling the setting in Settings takes effect for the next tap without a restart. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theToggleTakesEffectForTheNextTap() {
        controller.openLibrary(alpha(count = 6))
        showHost()
        openSettings()
        switchRow().performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription("Back to player").performClick()
        composeTestRule.waitForIdle()
        tap(3)
        assertTrue(engine.playing)
        assertEquals("content://invented/alpha/3" to true, engine.loads.single())
    }

    /** Moves the ring onto the track list with Alt+2, then onto a row with Down, and presses Enter [times] times. */
    private fun enterOnTheRingedRow(times: Int) {
        composeTestRule.onRoot().performKeyInput {
            keyDown(Key.AltLeft)
            pressKey(Key.Two)
            keyUp(Key.AltLeft)
            pressKey(Key.DirectionDown)
        }
        composeTestRule.waitForIdle()
        repeat(times) {
            composeTestRule.onRoot().performKeyInput { pressKey(Key.Enter) }
            composeTestRule.waitForIdle()
        }
    }

    /** With the setting on, Enter on the ringed row starts it, and a second Enter pauses it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun withTheSettingOnEnterStartsTheRingedRow() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        controller.openLibrary(alpha(count = 6))
        showHost()
        enterOnTheRingedRow(times = 1)
        assertTrue("Enter selected a track", controller.uiState.currentIndex != null)
        assertTrue("and started it", engine.playing)
        assertEquals(true, engine.loads.single().second)
        composeTestRule.onRoot().performKeyInput { pressKey(Key.Enter) }
        composeTestRule.waitForIdle()
        assertFalse("the second Enter pauses the current row", engine.playing)
        assertEquals(1, engine.loads.size)
    }

    /** Opens Search, types [title], and taps the first result row that shows it. */
    private fun activateSearchResultTitled(title: String) {
        composeTestRule.onNodeWithContentDescription(SEARCH_BUTTON_LABEL).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInput(title)
        val resultRow = hasClickAction() and hasText(title, substring = true) and !hasSetTextAction()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(resultRow).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onAllNodes(resultRow)[0].performClick()
        composeTestRule.waitForIdle()
    }

    /** With the setting on, the result of another track plays that track at once, and the same result again pauses. */
    @Test(timeout = CASE_LIMIT_MS)
    fun withTheSettingOnASearchResultStartsItsTrack() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        controller.searchHost = SearchHost()
        controller.openLibrary(alpha(count = 8))
        controller.playIndex(1)
        showHost()
        activateSearchResultTitled(titleOf(5))
        assertEquals(5, controller.uiState.currentIndex)
        assertTrue("the result's track plays at once", controller.uiState.playing)
        assertEquals("content://invented/alpha/5" to true, engine.loads.last())
        activateSearchResultTitled(titleOf(5))
        assertEquals(5, controller.uiState.currentIndex)
        assertFalse("the result of the current track toggles", controller.uiState.playing)
    }

    /** Opens Search, types [title], and returns the spoken click label of the first result row that shows it. */
    private fun resultClickLabelFor(title: String): String? {
        composeTestRule.onNodeWithContentDescription(SEARCH_BUTTON_LABEL).performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInput(title)
        val resultRow = hasClickAction() and hasText(title, substring = true) and !hasSetTextAction()
        composeTestRule.waitUntil(timeoutMillis = WAIT_LIMIT_MS) {
            composeTestRule.onAllNodes(resultRow).fetchSemanticsNodes().isNotEmpty()
        }
        return composeTestRule.onAllNodes(resultRow)[0].fetchSemanticsNode().config[SemanticsActions.OnClick].label
    }

    /** The spoken action of another track's result is Select track with the setting off. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theResultActionLabelIsSelectTrackWithTheSettingOff() {
        controller.searchHost = SearchHost()
        controller.openLibrary(alpha(count = 8))
        showHost()
        assertEquals("Select track", resultClickLabelFor(titleOf(5)))
    }

    /** The spoken action of another track's result is Play track again with the setting on. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theResultActionLabelIsPlayTrackWithTheSettingOn() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        controller.searchHost = SearchHost()
        controller.openLibrary(alpha(count = 8))
        showHost()
        assertEquals("Play track", resultClickLabelFor(titleOf(5)))
    }

    //endregion
}
