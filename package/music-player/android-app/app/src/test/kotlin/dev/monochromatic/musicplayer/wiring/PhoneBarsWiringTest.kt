// Wiring cases of the phone's top bar and deck at large text, rendered for real under Robolectric with native graphics:
// the deck keeps a gutter on both sides, its transport row and mode control do not touch, and a short folder title
// keeps its caret beside it instead of under it.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `coverTopRow` and `transportDeck` of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Spacer
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.unit.Density
import dev.monochromatic.musicplayer.TransportDeckActions
import dev.monochromatic.musicplayer.TransportDeckModel
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.coverTopRow
import dev.monochromatic.musicplayer.transportDeck
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** Qualifier of a 485 dp wide phone at one physical pixel per dp, so a pixel is a dp. */
private const val PHONE_QUALIFIER = "w485dp-h1080dp-mdpi"

/** Width of the phone in dp, from the qualifier above. */
private const val PHONE_WIDTH_DP: Float = 485f

/** Font scale at which the stacked deck and the two-line top bar apply, as on the reporting phone. */
private const val LARGE_TEXT_SCALE: Float = 1.5f

/** Least gutter in dp between the deck's content and the screen edge, equal to the list rows' inset. */
private const val DECK_GUTTER_DP: Float = 16f

/** Least gap in dp between the transport buttons and the mode control. */
private const val CONTROL_GAP_DP: Float = 16f

/** Slack in dp for sizes that must be equal; layout rounds each edge to a pixel, and a pixel is a dp here. */
private const val EDGE_SLACK_DP: Float = 0.6f

/** Tallest the folder trigger may be for a one-line title at large text: the 48 dp target with room to round. */
private const val ONE_LINE_TRIGGER_MAX_DP: Float = 60f

/** Name of the invented folder, short enough to fit one line at large text. */
private const val SHORT_FOLDER = "Music"

/** Name of the invented folder, too long to fit beside the three actions at large text. */
private const val LONG_FOLDER = "Aster Vale with a very long invented folder name that cannot fit beside the actions"

// What:     `class PhoneBarsWiringTest` groups the cases of the phone's deck and top bar at large text.
// Why:      The layout rules are pure and tested on the JVM; these cases pin that the composables really apply them,
//           through the bounds the screen shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("phone bars wiring", () => { /* cases rendered with a test renderer and read back as bounds */ });
// ```
/** Renders the cover top row and the deck at the reporting phone's width and font scale and asserts their bounds. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(qualifiers = PHONE_QUALIFIER)
class PhoneBarsWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Renders the top row, titled [folder], above the deck, the way the phone's cover screen stacks them. */
    private fun renderPhoneScreen(folder: String = SHORT_FOLDER) {
        composeTestRule.setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                CompositionLocalProvider(
                    LocalDensity provides Density(density = 1f, fontScale = LARGE_TEXT_SCALE),
                ) {
                    Column(modifier = Modifier.fillMaxSize()) {
                        coverTopRow(
                            folderTitle = folder,
                            pickerOpen = false,
                            onToggleFolderPicker = { },
                            onOpen = { },
                            onSettings = { },
                            onSearch = { },
                        )
                        Spacer(modifier = Modifier.weight(1f))
                        transportDeck(
                            model = TransportDeckModel(
                                title = "",
                                subtitle = "",
                                positionSec = 0.0,
                                durationSec = 0.0,
                                playing = false,
                                mode = PlaybackMode.IN_ORDER,
                                shuffleFolderLabel = SHORT_FOLDER,
                            ),
                            actions = TransportDeckActions(
                                onSeek = { },
                                onPrevious = { },
                                onTogglePlay = { },
                                onNext = { },
                                onSelectMode = { },
                            ),
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
            }
        }
    }

    /**
     * Returns the bounds in pixels (dp here) of the one node labelled [description] that the screen shows.
     *
     * The deck measures a probe of its transport and mode controls before it picks an arrangement, and the probe's
     * semantics stay in the tree unplaced, so a label matches twice and only the placed node is on screen.
     */
    private fun placedBounds(description: String): Rect = composeTestRule
        .onAllNodesWithContentDescription(description)
        .fetchSemanticsNodes()
        .single { node -> node.layoutInfo.isPlaced }
        .boundsInRoot

    @Test
    fun theDeckKeepsAGutterBesideTheClocksAndTheModeControl() {
        renderPhoneScreen()
        val elapsed = composeTestRule.onAllNodes(hasText("0:00")).onFirst().getUnclippedBoundsInRoot()
        val total = composeTestRule.onAllNodes(hasText("0:00")).onLast().getUnclippedBoundsInRoot()
        val firstSegment = placedBounds("Repeat track")
        val lastSegment = placedBounds("Shuffle all folders")
        assertTrue(
            "elapsed clock starts ${elapsed.left.value} dp from the edge",
            elapsed.left.value >= DECK_GUTTER_DP - EDGE_SLACK_DP,
        )
        assertTrue(
            "total clock ends ${PHONE_WIDTH_DP - total.right.value} dp from the edge",
            PHONE_WIDTH_DP - total.right.value >= DECK_GUTTER_DP - EDGE_SLACK_DP,
        )
        assertTrue(
            "mode control starts ${firstSegment.left} dp from the edge",
            firstSegment.left >= DECK_GUTTER_DP - EDGE_SLACK_DP,
        )
        assertTrue(
            "mode control ends ${PHONE_WIDTH_DP - lastSegment.right} dp from the edge",
            PHONE_WIDTH_DP - lastSegment.right >= DECK_GUTTER_DP - EDGE_SLACK_DP,
        )
    }

    @Test
    fun theTransportButtonsAndTheModeControlAreSeparated() {
        renderPhoneScreen()
        val next = placedBounds("Next track")
        val firstSegment = placedBounds("Repeat track")
        val gap = firstSegment.top - next.bottom
        assertTrue("transport buttons end $gap dp above the mode control", gap >= CONTROL_GAP_DP - EDGE_SLACK_DP)
    }

    @Test
    fun aShortFolderTitleSharesTheRowWithTheActions() {
        renderPhoneScreen()
        val trigger = placedBounds("Folder: $SHORT_FOLDER")
        val settings = placedBounds("Settings")
        assertTrue(
            "settings starts ${settings.top} dp down, below the title's bottom at ${trigger.bottom} dp",
            settings.top < trigger.bottom,
        )
    }

    @Test
    fun aLongFolderTitleStaysOnOneLineBesideTheActions() {
        renderPhoneScreen(folder = LONG_FOLDER)
        val trigger = placedBounds("Folder: $LONG_FOLDER")
        val search = placedBounds("Search music")
        val settings = placedBounds("Settings")
        val height = trigger.bottom - trigger.top
        assertTrue("folder trigger is $height dp tall for a long title", height <= ONE_LINE_TRIGGER_MAX_DP)
        assertTrue(
            "settings starts ${settings.top} dp down, below the title's bottom at ${trigger.bottom} dp",
            settings.top < trigger.bottom,
        )
        assertTrue(
            "title ends ${trigger.right} dp across, past the search button at ${search.left} dp",
            trigger.right <= search.left + EDGE_SLACK_DP,
        )
    }

    @Test
    fun aShortFolderTitleDoesNotReserveASecondLine() {
        renderPhoneScreen()
        val trigger = composeTestRule.onNodeWithContentDescription("Folder: $SHORT_FOLDER").getUnclippedBoundsInRoot()
        val height = trigger.bottom.value - trigger.top.value
        assertTrue("folder trigger is $height dp tall for a one-line title", height <= ONE_LINE_TRIGGER_MAX_DP)
    }
}
