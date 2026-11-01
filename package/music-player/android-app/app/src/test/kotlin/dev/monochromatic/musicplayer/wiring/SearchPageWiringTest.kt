// Wiring cases of the Search page restyle (D126), rendered for real under Robolectric with native graphics so that the
// pixels the page paints can be read back: no gray header band, the divider role, the 40 dp result circle, the status
// circle lifted above the keyboard, the D58 columns, and the keyboard ring just inside a rounded row.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `searchPage` and the theme functions of the app module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import android.view.View
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.input.InputModeManager
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.requestFocus
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpRect
import androidx.compose.ui.unit.height
import androidx.compose.ui.unit.width
import androidx.compose.ui.unit.dp
import androidx.core.graphics.Insets
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import dev.monochromatic.musicplayer.LocalThemeRoles
import dev.monochromatic.musicplayer.SEARCH_DIVIDER_TAG
import dev.monochromatic.musicplayer.SEARCH_HEADER_TAG
import dev.monochromatic.musicplayer.SEARCH_ROW_CIRCLE_TAG
import dev.monochromatic.musicplayer.SEARCH_STATUS_CIRCLE_TAG
import dev.monochromatic.musicplayer.SEARCH_STATUS_STRIP_TAG
import dev.monochromatic.musicplayer.SEARCH_STATUS_TAG
import dev.monochromatic.musicplayer.core.SearchHighlight
import dev.monochromatic.musicplayer.core.SearchResult
import dev.monochromatic.musicplayer.core.SearchResultKind
import dev.monochromatic.musicplayer.core.SearchState
import dev.monochromatic.musicplayer.core.SEARCH_BACK_LABEL
import dev.monochromatic.musicplayer.core.SEARCH_CLEAR_LABEL
import dev.monochromatic.musicplayer.core.SEARCH_FIELD_LABEL
import dev.monochromatic.musicplayer.core.SEARCH_NO_MATCH_DETAIL
import dev.monochromatic.musicplayer.core.SEARCH_PROMPT_DETAIL
import dev.monochromatic.musicplayer.core.SEARCH_PROMPT_TITLE
import dev.monochromatic.musicplayer.core.deriveRoles
import dev.monochromatic.musicplayer.core.searchRowSemantics
import dev.monochromatic.musicplayer.focusRingTheme
import dev.monochromatic.musicplayer.playerColorScheme
import dev.monochromatic.musicplayer.searchCreaseHalfDp
import dev.monochromatic.musicplayer.searchPage
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** Qualifier of the folded cover: 411 dp wide at three physical pixels per dp. */
private const val COVER = "w411dp-h891dp-xxhdpi"

/** Qualifier of the folded cover in the dark scheme. */
private const val COVER_DARK = "w411dp-h891dp-night-xxhdpi"

/** Qualifier of the unfolded inner panel: 851 dp wide at three physical pixels per dp. */
private const val INNER = "w851dp-h700dp-xxhdpi"

/** Height in pixels of the invented keyboard inset, 300 dp at three pixels per dp. */
private const val KEYBOARD_PX: Int = 900

/** Height in pixels of the invented status bar inset, 24 dp at three pixels per dp. */
private const val STATUS_BAR_PX: Int = 72

/** Slack in dp for the pixel that the two equal panes split, which is at most one pixel at three per dp. */
private const val PANE_ROUNDING_DP: Float = 0.5f

/** Pixels per dp of the qualifiers above. */
private const val PX_PER_DP: Float = 3f

// What:     `class SearchPageWiringTest` groups the cases of the real Search page.
// Why:      The restyle is a set of modifiers and colors at Compose call sites; a mistake at any of them keeps every
//           pure-function test green, so each case pins one through the pixels or bounds the page shows.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("SearchPage wiring", () => { /* cases rendered with a test renderer and read back as pixels */ });
// ```
/** Renders [searchPage] over invented results and asserts the header, divider, circles, columns, ring and insets. */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(qualifiers = COVER)
class SearchPageWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Material scheme the case was rendered with, so the expected colors are the ones the page was given. */
    private lateinit var scheme: ColorScheme

    /** Input mode manager of the composition, which the keyboard cases switch to keyboard mode. */
    private lateinit var inputModes: InputModeManager

    /** Root view of the composition, which receives the invented window insets. */
    private lateinit var composeView: View

    /** Invented track result whose second line is its parent folder. */
    private fun track(name: String, index: Int, highlights: List<SearchHighlight> = emptyList()): SearchResult =
        SearchResult(
            kind = SearchResultKind.TRACK,
            name = name,
            parentContext = "Aster Vale/Salt Garden",
            indexInLibrary = index,
            folderName = "Aster Vale",
            tier = 0,
            nameHighlights = highlights,
            parentHighlights = emptyList(),
        )

    /** Invented folder result. */
    private fun folder(name: String): SearchResult = SearchResult(
        kind = SearchResultKind.FOLDER,
        name = name,
        parentContext = null,
        indexInLibrary = null,
        folderName = name,
        tier = 0,
        nameHighlights = emptyList(),
        parentHighlights = emptyList(),
    )

    /** Three invented results: two tracks and a folder. */
    private fun threeResults(): SearchState.Results = SearchState.Results(
        items = listOf(track("Dune Lantern", 1000), track("Lantern Season", 1001), folder("Lantern Hall")),
        truncated = false,
    )

    /** Shows the real page for [state] and [query] and waits until its first frames are composed. */
    private fun show(
        state: SearchState,
        query: String,
        crease: Float? = null,
        fontScale: Float = 1f,
        ringTheme: Boolean = true,
    ) {
        composeTestRule.setContent {
            composeView = LocalView.current
            inputModes = LocalInputModeManager.current
            /** Whether the configuration is night, as the page itself reads it. */
            val dark: Boolean = isSystemInDarkTheme()
            /** Roles of both appearances, derived from the baseline root. */
            val palette = deriveRoles(null)
            scheme = playerColorScheme(palette, dark)
            CompositionLocalProvider(
                LocalThemeRoles provides if (dark) palette.dark else palette.light,
                LocalDensity provides Density(LocalDensity.current.density, fontScale),
            ) {
                MaterialTheme(colorScheme = scheme) {
                    ringThemeIf(ringTheme) {
                        searchPage(
                            creaseWidthPx = crease,
                            state = state,
                            query = query,
                            onQueryChange = { },
                            onBack = { },
                            onClear = { },
                            onOpenResult = { },
                            currentTrackIndex = null,
                            deck = { Box(modifier = Modifier) },
                        )
                    }
                }
            }
        }
        composeTestRule.waitForIdle()
    }

    /** Wraps [content] in the app's focus ring theme when [enabled], and leaves Material's focus layer on otherwise. */
    @Composable
    private fun ringThemeIf(enabled: Boolean, content: @Composable () -> Unit) {
        if (enabled) {
            focusRingTheme(content)
        } else {
            content()
        }
    }

    /** Switches to keyboard input and moves focus from the Clear button to the first result row by Tab. */
    private fun focusFirstRowByKeyboard() {
        // Switches the input mode from touch to keyboard, in which buttons and rows can take focus.
        composeTestRule.runOnUiThread { inputModes.requestInputMode(InputMode.Keyboard) }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SEARCH_CLEAR_LABEL).requestFocus()
        composeTestRule.onNodeWithContentDescription(SEARCH_CLEAR_LABEL).assertIsFocused()
        composeTestRule.onNodeWithContentDescription(SEARCH_CLEAR_LABEL).performKeyInput { pressKey(Key.Tab) }
        composeTestRule.onNode(hasClickAction().and(hasText("Dune Lantern", substring = true))).assertIsFocused()
        composeTestRule.waitForIdle()
    }

    /** Dispatches invented window insets (a status bar and a keyboard) to the composition and waits for the frames. */
    private fun insets(statusBarPx: Int, keyboardPx: Int) {
        composeTestRule.runOnUiThread {
            ViewCompat.dispatchApplyWindowInsets(
                composeView,
                WindowInsetsCompat.Builder()
                    .setInsets(WindowInsetsCompat.Type.statusBars(), Insets.of(0, statusBarPx, 0, 0))
                    .setInsets(WindowInsetsCompat.Type.ime(), Insets.of(0, 0, 0, keyboardPx))
                    .setVisible(WindowInsetsCompat.Type.ime(), keyboardPx > 0)
                    .build(),
            )
        }
        composeTestRule.waitForIdle()
    }

    /** ARGB value of the pixel at ([x], [y]) of the tagged node's image. */
    private fun pixelOf(tag: String, x: Int, y: Int): Int =
        composeTestRule.onAllNodesWithTag(tag, useUnmergedTree = true)[0].captureToImage().toPixelMap()[x, y].toArgb()

    /** Bounds of the tagged node in the unmerged tree. */
    private fun boundsOf(tag: String): DpRect =
        composeTestRule.onAllNodesWithTag(tag, useUnmergedTree = true)[0].getUnclippedBoundsInRoot()

    /** Bounds of the node with the given description in the unmerged tree. */
    private fun boundsOfDescription(description: String): DpRect =
        composeTestRule.onNodeWithContentDescription(description, useUnmergedTree = true).getUnclippedBoundsInRoot()

    /** Bounds of the merged row that holds [title] and can be clicked. */
    private fun rowBounds(title: String): DpRect =
        composeTestRule.onNode(hasClickAction().and(hasText(title, substring = true))).getUnclippedBoundsInRoot()

    // What:     `@Test fun headerAndStatusStripShareTheCanvasColorInLight()` checks that no gray band remains.
    // Why:      The header used `surfaceContainerHigh`, so a white status strip sat over a gray band; D126 makes the
    //           header the canvas, so strip, header and the page below it are one tone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no band", () => {
    //   render(search);
    //   expect(pixel("header")).toBe(canvas);
    //   expect(pixel("strip")).toBe(canvas);
    // });
    // ```
    /** In the light scheme the strip and the header read the canvas color, which differs from the old band color. */
    @Test(timeout = CASE_LIMIT_MS)
    fun headerAndStatusStripShareTheCanvasColorInLight() {
        show(SearchState.EmptyQuery, query = "")
        insets(statusBarPx = STATUS_BAR_PX, keyboardPx = 0)
        /** The old band color, which the test must be able to tell from the canvas. */
        val oldBand: Int = scheme.surfaceContainerHigh.toArgb()
        /** The canvas the page paints in the light scheme. */
        val canvas: Int = scheme.surfaceContainerLowest.toArgb()
        assertNotEquals("the old band and the canvas must differ for this check to mean anything", oldBand, canvas)
        assertEquals(canvas, pixelOf(SEARCH_HEADER_TAG, x = 900, y = 20))
        assertEquals(canvas, pixelOf(SEARCH_HEADER_TAG, x = 900, y = 200))
        assertEquals(canvas, pixelOf(SEARCH_STATUS_STRIP_TAG, x = 10, y = 10))
    }

    // What:     `@Test fun headerAndStatusStripAreTrueBlackInDark()` checks the dark canvas.
    // Why:      B1 makes dark pure black, and the old band was a lighter gray on it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dark header is black", () => {
    //   render(search, { dark: true });
    //   expect(pixel("header")).toBe(0xff000000);
    // });
    // ```
    /** In the dark scheme the strip and the header are pure black. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = COVER_DARK)
    fun headerAndStatusStripAreTrueBlackInDark() {
        show(SearchState.EmptyQuery, query = "")
        insets(statusBarPx = STATUS_BAR_PX, keyboardPx = 0)
        assertNotEquals(Color.Black.toArgb(), scheme.surfaceContainerHigh.toArgb())
        assertEquals(Color.Black.toArgb(), pixelOf(SEARCH_HEADER_TAG, x = 900, y = 20))
        assertEquals(Color.Black.toArgb(), pixelOf(SEARCH_STATUS_STRIP_TAG, x = 10, y = 10))
    }

    // What:     `@Test fun dividerIsOutlineVariantInBothSchemes()` checks the divider role.
    // Why:      The divider moved from the heavy `outline` role to the light `outlineVariant` role.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("divider role", () => { expect(pixel("divider")).toBe(outlineVariant); });
    // ```
    /** The divider's middle pixel is the outline-variant color, which differs from the outline color. */
    @Test(timeout = CASE_LIMIT_MS)
    fun dividerIsOutlineVariantInLight() {
        show(SearchState.EmptyQuery, query = "")
        assertNotEquals(scheme.outline.toArgb(), scheme.outlineVariant.toArgb())
        assertEquals(scheme.outlineVariant.toArgb(), pixelOf(SEARCH_DIVIDER_TAG, x = 400, y = 1))
        assertEquals(1f, boundsOf(SEARCH_DIVIDER_TAG).height.value, 0.01f)
    }

    /** The same holds in the dark scheme on the true-black canvas. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = COVER_DARK)
    fun dividerIsOutlineVariantInDark() {
        show(SearchState.EmptyQuery, query = "")
        assertNotEquals(scheme.outline.toArgb(), scheme.outlineVariant.toArgb())
        assertEquals(scheme.outlineVariant.toArgb(), pixelOf(SEARCH_DIVIDER_TAG, x = 400, y = 1))
    }

    // What:     `@Test fun resultGlyphSitsInAFortyDpSecondaryContainerCircle()` checks the circle's size and fill.
    // Why:      D126 replaces the bare 24 dp glyph with the D109 circle, 40 dp wide, centered in the 48 dp slot.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("40dp circle", () => {
    //   expect(size("rowCircle")).toBe(40);
    //   expect(corner).toBe(canvas);
    //   expect(edge).toBe(fill);
    // });
    // ```
    /** The circle is 40 dp wide, filled with the secondary container role, round at its corners, and centered. */
    @Test(timeout = CASE_LIMIT_MS)
    fun resultGlyphSitsInAFortyDpSecondaryContainerCircle() {
        show(threeResults(), query = "lan")
        /** Bounds of the first row's circle. */
        val circle = boundsOf(SEARCH_ROW_CIRCLE_TAG)
        assertEquals(40f, circle.width.value, 0.01f)
        assertEquals(40f, circle.height.value, 0.01f)
        assertEquals(scheme.secondaryContainer.toArgb(), pixelOf(SEARCH_ROW_CIRCLE_TAG, x = 12, y = 60))
        assertEquals(scheme.surfaceContainerLowest.toArgb(), pixelOf(SEARCH_ROW_CIRCLE_TAG, x = 0, y = 0))
        /** Bounds of the Back button's glyph, whose center column the result glyph must share (D58). */
        val back = boundsOfDescription(SEARCH_BACK_LABEL)
        assertEquals(back.left.value + back.width.value / 2f, circle.left.value + circle.width.value / 2f, 0.01f)
    }

    // What:     `@Test fun resultCircleHoldsTheOnlyDescribedGlyph()` checks the merged speech of a row.
    // Why:      The kind label must stay on the glyph and not move to the circle, so TalkBack reads the same row (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("speech unchanged", () => {
    //   expect(circle.contentDescription).toBeUndefined();
    //   expect(glyph.contentDescription).toBe("Track");
    // });
    // ```
    /** The circle has no description and the glyph keeps the row's kind label. */
    @Test(timeout = CASE_LIMIT_MS)
    fun resultCircleHoldsTheOnlyDescribedGlyph() {
        /** The first invented result. */
        val first: SearchResult = track("Dune Lantern", 1000)
        show(SearchState.Results(items = listOf(first), truncated = false), query = "lan")
        composeTestRule.onAllNodesWithTag(SEARCH_ROW_CIRCLE_TAG, useUnmergedTree = true)[0]
            .assert(SemanticsMatcher.keyNotDefined(SemanticsProperties.ContentDescription))
        composeTestRule.onNodeWithContentDescription(
            searchRowSemantics(first, currentTrackIndex = null, isPlaying = null).kindLabel,
            useUnmergedTree = true,
        ).assertExists()
    }

    // What:     `@Test fun resultTitleAndGlyphKeepTheHeaderColumnsOnTheCover()` checks D58 on the folded cover.
    // Why:      The rounded shape starts 8 dp before the content; the glyph must still share Back's center and the
    //           title must still start where the query starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("D58 on the cover", () => { expect(titleLeft).toBe(fieldLeft); expect(glyphCenter).toBe(backCenter); });
    // ```
    /** On the cover the title starts at the query's start and the shape starts 8 dp from each side. */
    @Test(timeout = CASE_LIMIT_MS)
    fun resultTitleAndGlyphKeepTheHeaderColumnsOnTheCover() {
        show(threeResults(), query = "lan")
        /** Bounds of the query field. */
        val field = boundsOfDescription(SEARCH_FIELD_LABEL)
        /** Bounds of the first title. */
        val title = composeTestRule.onNodeWithText("Dune Lantern", useUnmergedTree = true).getUnclippedBoundsInRoot()
        assertEquals(field.left.value, title.left.value, 0.01f)
        assertEquals(64f, title.left.value, 0.01f)
        /** Bounds of the first row's rounded shape. */
        val row = rowBounds("Dune Lantern")
        assertEquals(8f, row.left.value, 0.01f)
        assertEquals(411f - 8f, row.right.value, 0.01f)
        assertEquals(72f, row.height.value, 0.01f)
    }

    // What:     `@Test fun resultShapeStaysOutsideTheCreaseOnTheInnerPanel()` checks D58 and E2 on the inner panel.
    // Why:      The pane starts at the screen center and its content at half the crease plus 16 dp, so the rounded
    //           shape, 8 dp before the content, must not enter the crease clear zone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("inner shape clear of crease", () => { expect(row.left).toBeGreaterThanOrEqual(paneLeft + halfCrease); });
    // ```
    /** On the inner panel the title still starts at the query's start and the shape starts after the clear zone. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER)
    fun resultShapeStaysOutsideTheCreaseOnTheInnerPanel() {
        /** Crease width in pixels as a platform would report it. */
        val creasePx = 60f
        /** Half the clear zone in dp. */
        val halfCrease: Float = searchCreaseHalfDp(densityPerDp = PX_PER_DP, creaseWidthPx = creasePx)
        show(threeResults(), query = "lan", crease = creasePx)
        /** Bounds of the query field. */
        val field = boundsOfDescription(SEARCH_FIELD_LABEL)
        /** Bounds of the first title. */
        val title = composeTestRule.onNodeWithText("Dune Lantern", useUnmergedTree = true).getUnclippedBoundsInRoot()
        assertEquals(field.left.value, title.left.value, 0.01f)
        /** Left edge of the right pane in dp. */
        val paneLeft = 851f / 2f
        assertEquals(paneLeft + halfCrease + 16f + 48f, title.left.value, PANE_ROUNDING_DP)
        /** Bounds of the first row's rounded shape. */
        val row = rowBounds("Dune Lantern")
        assertEquals(paneLeft + halfCrease + 8f, row.left.value, PANE_ROUNDING_DP)
        assertTrue(row.left.value >= paneLeft + halfCrease)
    }

    // What:     `@Test fun emptyStateShowsA72dpCircleAboveTheKeyboard()` checks the status circle and its lift.
    // Why:      D126 puts the glyph in a 72 dp circle and pads the block by the keyboard inset, as the result list is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("status above keyboard", () => {
    //   setKeyboard(300);
    //   expect(status.bottom).toBeLessThanOrEqual(height - 300);
    // });
    // ```
    /** The circle is 72 dp and filled; with a 300 dp keyboard the status block ends above it and shows all its text. */
    @Test(timeout = CASE_LIMIT_MS)
    fun emptyStateShowsA72dpCircleAboveTheKeyboard() {
        show(SearchState.EmptyQuery, query = "")
        insets(statusBarPx = STATUS_BAR_PX, keyboardPx = KEYBOARD_PX)
        /** Bounds of the circle. */
        val circle = boundsOf(SEARCH_STATUS_CIRCLE_TAG)
        assertEquals(72f, circle.width.value, 0.01f)
        assertEquals(72f, circle.height.value, 0.01f)
        assertEquals(scheme.secondaryContainer.toArgb(), pixelOf(SEARCH_STATUS_CIRCLE_TAG, x = 20, y = 108))
        /** Height of the root in dp. */
        val rootHeight: Dp = composeTestRule.onRoot().getUnclippedBoundsInRoot().height
        /** Top of the keyboard in dp. */
        val keyboardTop: Float = rootHeight.value - 300f
        assertTrue(boundsOf(SEARCH_STATUS_TAG).bottom.value <= keyboardTop + 0.01f)
        /** Bottom of the support line in dp. */
        val detailBottom: Float =
            composeTestRule.onNodeWithText(SEARCH_PROMPT_DETAIL).getUnclippedBoundsInRoot().bottom.value
        assertTrue(detailBottom <= keyboardTop)
        composeTestRule.onNodeWithText(SEARCH_PROMPT_TITLE).assertExists()
    }

    /** Without a keyboard the status block reaches the bottom of the page, so the lift is the keyboard's doing. */
    @Test(timeout = CASE_LIMIT_MS)
    fun emptyStateBlockReachesTheBottomWithoutAKeyboard() {
        show(SearchState.EmptyQuery, query = "")
        insets(statusBarPx = STATUS_BAR_PX, keyboardPx = 0)
        /** Height of the root in dp. */
        val rootHeight: Float = composeTestRule.onRoot().getUnclippedBoundsInRoot().height.value
        assertEquals(rootHeight, boundsOf(SEARCH_STATUS_TAG).bottom.value, 0.01f)
    }

    /** At 200 percent text the whole prompt still ends above a 300 dp keyboard. */
    @Test(timeout = CASE_LIMIT_MS)
    fun emptyStateStaysAboveTheKeyboardAtDoubleTextSize() {
        show(SearchState.NoMatch(query = "zzq"), query = "zzq", fontScale = 2f)
        insets(statusBarPx = STATUS_BAR_PX, keyboardPx = KEYBOARD_PX)
        /** Height of the root in dp. */
        val rootHeight: Float = composeTestRule.onRoot().getUnclippedBoundsInRoot().height.value
        /** Top of the keyboard in dp. */
        val keyboardTop: Float = rootHeight - 300f
        assertTrue(boundsOf(SEARCH_STATUS_TAG).bottom.value <= keyboardTop + 0.01f)
        /** Bottom of the support line in dp. */
        val detailBottom: Float =
            composeTestRule.onNodeWithText(SEARCH_NO_MATCH_DETAIL).getUnclippedBoundsInRoot().bottom.value
        assertTrue(detailBottom <= keyboardTop)
    }

    // What:     `@Test fun keyboardRingSitsJustInsideARoundedRow()` checks the ring after Tab.
    // Why:      D117 and D120 draw the ring only for keyboard focus and just inside the row's edge; the rounded shape
    //           must be the one the ring follows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ring inside row", () => {
    //   press("Tab");
    //   expect(pixel(row, 1.5, mid)).toBe(primary);
    //   expect(pixel(row, 0, 0)).toBe(canvas);
    // });
    // ```
    /** After Tab the row's edge pixel is the outer ring, the next ones the near ring, and the corner stays bare. */
    @Test(timeout = CASE_LIMIT_MS)
    fun keyboardRingSitsJustInsideARoundedRow() {
        show(threeResults(), query = "lan")
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).assertIsFocused()
        /** The row node, whose bounds are the rounded shape. */
        val rowNode = composeTestRule.onNode(hasClickAction().and(hasText("Dune Lantern", substring = true)))
        assertEquals(
            "no ring before keyboard focus",
            scheme.surfaceContainerLowest.toArgb(),
            rowNode.captureToImage().toPixelMap()[4, 108].toArgb(),
        )
        focusFirstRowByKeyboard()
        /** Pixels of the focused row. */
        val image = rowNode.captureToImage().toPixelMap()
        /** Middle of the row's height in pixels. */
        val middle: Int = (72f * PX_PER_DP / 2f).toInt()
        assertEquals("outer ring at the edge", scheme.onPrimary.toArgb(), image[1, middle].toArgb())
        assertEquals("near ring just inside", scheme.primary.toArgb(), image[4, middle].toArgb())
        assertEquals("bare inside the rings", scheme.surfaceContainerLowest.toArgb(), image[12, middle].toArgb())
        assertEquals("rounded corner stays bare", scheme.surfaceContainerLowest.toArgb(), image[1, 1].toArgb())
        // The outer ring is white like the canvas in light, so the near ring (row 4 from the top) is what is probed.
        // A square ring would still cover the top edge 6 dp from the corner, where the curve has already left it.
        assertEquals(
            "the ring follows the curve, not a square",
            scheme.surfaceContainerLowest.toArgb(),
            image[(6f * PX_PER_DP).toInt(), 4].toArgb(),
        )
        assertEquals(
            "the straight top edge carries the near ring",
            scheme.primary.toArgb(),
            image[(30f * PX_PER_DP).toInt(), 4].toArgb(),
        )
        assertEquals(8f, rowNode.getUnclippedBoundsInRoot().left.value, 0.01f)
    }

    // What:     `@Test fun indicationLayerIsClippedToTheRoundedShape()` checks the row's clip.
    // Why:      The row clips its indication to the rounded shape; without the clip a square state layer would reach
    //           the corner pixels. Material's own focus layer is drawn here because the test host cannot draw a press
    //           ripple, and it goes through the same indication as a press.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("indication shape", () => {
    //   focusByKeyboard(row);
    //   expect(pixel(row, middle)).not.toBe(canvas);
    //   expect(pixel(row, corner)).toBe(canvas);
    // });
    // ```
    /** With Material's focus layer left on, the row's inside shows the layer and its rounded corner stays bare. */
    @Test(timeout = CASE_LIMIT_MS)
    fun indicationLayerIsClippedToTheRoundedShape() {
        show(threeResults(), query = "lan", ringTheme = false)
        focusFirstRowByKeyboard()
        /** Pixels of the focused row. */
        val image = composeTestRule.onNode(hasClickAction().and(hasText("Dune Lantern", substring = true)))
            .captureToImage().toPixelMap()
        /** Canvas color the layer is drawn over. */
        val canvas: Int = scheme.surfaceContainerLowest.toArgb()
        assertNotEquals("the focus layer tints the inside", canvas, image[(300 * PX_PER_DP).toInt(), 20].toArgb())
        assertEquals("the rounded corner stays bare", canvas, image[1, 1].toArgb())
    }

    // What:     `@Test fun statusTitleUsesTitleMediumAtMediumWeight()` checks the lighter headline.
    // Why:      D126 lightens the headline from semibold title large to medium title medium.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("status title", () => {
    //   expect(style(title).fontWeight).toBe(500);
    //   expect(style(title).fontSize).toBe(16);
    // });
    // ```
    /** The empty-state headline is title medium at weight 500 on the cover and on the inner panel. */
    @Test(timeout = CASE_LIMIT_MS)
    fun statusTitleUsesTitleMediumAtMediumWeight() {
        show(SearchState.EmptyQuery, query = "")
        /** Style the headline was laid out with. */
        val style = titleStyle()
        assertEquals(FontWeight.Medium, style.fontWeight)
        assertEquals(Typography().titleMedium.fontSize, style.fontSize)
    }

    /** Layout style of the empty-state headline, read from its text layout result. */
    private fun titleStyle(): TextStyle {
        /** Layout results the headline reports. */
        val results = mutableListOf<TextLayoutResult>()
        composeTestRule.onNodeWithText(SEARCH_PROMPT_TITLE).fetchSemanticsNode().config
            .getOrNull(SemanticsActions.GetTextLayoutResult)?.action?.invoke(results)
        return results.single().layoutInput.style
    }

    // What:     `@Test fun statusBlockStartsSeventyTwoDpUnderTheDividerAndScrolls()` checks the block's anchor.
    // Why:      The block is anchored a fixed distance under the divider, not centered, and scrolls when the space
    //           above the keyboard is shorter than the block.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("anchor", () => { expect(circle.top - divider.bottom).toBe(72); expect(status).toHaveScrollAction(); });
    // ```
    /** The circle's top is 72 dp under the divider on the cover, and the block offers a scroll action. */
    @Test(timeout = CASE_LIMIT_MS)
    fun statusBlockStartsSeventyTwoDpUnderTheDividerAndScrolls() {
        show(SearchState.EmptyQuery, query = "")
        /** Distance from the divider's bottom to the circle's top in dp. */
        val gap: Float = boundsOf(SEARCH_STATUS_CIRCLE_TAG).top.value - boundsOf(SEARCH_DIVIDER_TAG).bottom.value
        assertEquals(72f, gap, 0.01f)
        composeTestRule.onNodeWithTag(SEARCH_STATUS_TAG, useUnmergedTree = true).assert(hasScrollAction())
    }

    // What:     `@Test fun innerPanelStatusHasTheCircleAtTheContentStart()` checks the inner panel's status.
    // Why:      The inner status gets the same circle, left aligned at the start inset beside the crease (E2).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("inner status", () => {
    //   expect(circle.left).toBe(paneLeft + halfCrease + 16);
    //   expect(circle.width).toBe(72);
    // });
    // ```
    /** On the inner panel the circle is 72 dp wide and starts at the content inset after the crease clear zone. */
    @Test(timeout = CASE_LIMIT_MS)
    @Config(qualifiers = INNER)
    fun innerPanelStatusHasTheCircleAtTheContentStart() {
        /** Crease width in pixels as a platform would report it. */
        val creasePx = 60f
        /** Half the clear zone in dp. */
        val halfCrease: Float = searchCreaseHalfDp(densityPerDp = PX_PER_DP, creaseWidthPx = creasePx)
        show(SearchState.EmptyQuery, query = "", crease = creasePx)
        /** Bounds of the status circle. */
        val circle = boundsOf(SEARCH_STATUS_CIRCLE_TAG)
        assertEquals(72f, circle.width.value, 0.01f)
        assertEquals(851f / 2f + halfCrease + 16f, circle.left.value, PANE_ROUNDING_DP)
        assertEquals(72f, circle.top.value - boundsOf(SEARCH_DIVIDER_TAG).bottom.value, 0.01f)
        assertEquals(scheme.secondaryContainer.toArgb(), pixelOf(SEARCH_STATUS_CIRCLE_TAG, x = 20, y = 108))
        // The header and the divider of the right pane are the canvas and the soft divider too.
        assertEquals(scheme.surfaceContainerLowest.toArgb(), pixelOf(SEARCH_HEADER_TAG, x = 1200, y = 20))
        assertEquals(scheme.outlineVariant.toArgb(), pixelOf(SEARCH_DIVIDER_TAG, x = 400, y = 1))
    }

    // What:     `@Test fun glyphsAreDrawnInOnSecondaryContainer()` checks the glyph tint in both circles.
    // Why:      The glyph must use the role that pairs with the circle fill, so some pixel of each circle is it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("glyph tint", () => { expect(pixels(rowCircle)).toContain(onSecondaryContainer); });
    // ```
    /** Each circle holds at least one pixel of the on-secondary-container color. */
    @Test(timeout = CASE_LIMIT_MS)
    fun glyphsAreDrawnInOnSecondaryContainer() {
        show(threeResults(), query = "lan")
        assertTrue(hasPixel(SEARCH_ROW_CIRCLE_TAG, scheme.onSecondaryContainer.toArgb()))
    }

    /** The status circle holds at least one pixel of the on-secondary-container color. */
    @Test(timeout = CASE_LIMIT_MS)
    fun statusGlyphIsDrawnInOnSecondaryContainer() {
        show(SearchState.EmptyQuery, query = "")
        assertTrue(hasPixel(SEARCH_STATUS_CIRCLE_TAG, scheme.onSecondaryContainer.toArgb()))
    }

    /** Whether any pixel of the first node with [tag] has exactly the color [argb]. */
    private fun hasPixel(tag: String, argb: Int): Boolean {
        /** Pixels of the node. */
        val image = composeTestRule.onAllNodesWithTag(tag, useUnmergedTree = true)[0].captureToImage().toPixelMap()
        return (0 until image.height).any { y -> (0 until image.width).any { x -> image[x, y].toArgb() == argb } }
    }
}
