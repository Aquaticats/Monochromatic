// Wiring cases of the template editor's field rows (D58, D110, D120, D130), rendered for real under Robolectric: the
// text of a field row starts in the column of the section headings, with and without the crease inset of the right
// pane, and the ring that lies inside the row's edge keeps its gap to that text.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `templateEditor` and the state it draws.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.test.getUnclippedBoundsInRoot
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpRect
import androidx.compose.ui.unit.dp
import dev.monochromatic.musicplayer.TemplateEditorActions
import dev.monochromatic.musicplayer.core.TemplateEditorState
import dev.monochromatic.musicplayer.templateEditor
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** Qualifier of a 400 dp wide pane at one physical pixel per dp, so a pixel is a dp. */
private const val EDITOR_QUALIFIER = "w400dp-h800dp-mdpi"

/** Width of the pane in dp, from the qualifier above. */
private const val EDITOR_PANE_WIDTH_DP: Float = 400f

/** Slack in dp for two sizes that must be equal; layout rounds each edge to a pixel, and a pixel is a dp here. */
private const val COLUMN_SLACK_DP: Float = 0.6f

/** Depth in dp of the two focus rings inside the row's edge. */
private const val EDITOR_RING_DEPTH_DP: Float = 3f

/** Least gap in dp from the ring's inner edge to the text (D120). */
private const val EDITOR_RING_TEXT_GAP_DP: Float = 8f

/** Crease inset of the right Settings pane at the Fold inner density: the 22.64 dp clearance less the 16 dp inset. */
private const val CREASE_INSET_DP: Float = 6.64f

// What:     `class TemplateEditorAlignmentWiringTest` groups the cases of the field rows' text column.
// Why:      A row that pads its content for the ring puts the text right of the headings; the cases read the bounds
//           of the real editor to pin the column.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("template editor alignment", () => { /* cases rendered with a test renderer */ });
// ```
/** Renders [templateEditor] over the default templates and compares the field rows with the section headings. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = EDITOR_QUALIFIER)
class TemplateEditorAlignmentWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Editor state on the default track rows template, previewing the invented stand-in tracks. */
    private val state: TemplateEditorState = TemplateEditorState.initial(emptyList())

    /** Shows the editor as the right Settings pane with [startInset] moved off the pane's edge. */
    private fun show(startInset: Dp) {
        composeTestRule.setContent {
            MaterialTheme {
                templateEditor(
                    state = state,
                    actions = TemplateEditorActions(
                        onEdit = { },
                        onFocusChanged = { },
                        onInsertField = { },
                        onReset = { },
                        onBack = { },
                    ),
                    embedded = true,
                    startInset = startInset,
                )
            }
        }
        composeTestRule.waitForIdle()
    }

    /** Bounds of the text node of the first field row's label, read from the unmerged tree. */
    private fun firstFieldLabelBounds(): DpRect = composeTestRule
        .onAllNodes(
            hasText(state.fields.first().label).and(hasAnyAncestor(hasClickAction())),
            useUnmergedTree = true,
        )[0]
        .getUnclippedBoundsInRoot()

    /** Bounds of the first field row, which is the clickable row that holds the first field's label. */
    private fun firstFieldRowBounds(): DpRect = composeTestRule
        .onAllNodes(hasClickAction().and(hasText(state.fields.first().label)))[0]
        .getUnclippedBoundsInRoot()

    /** Asserts the first field row's text starts where the "Fields" heading starts. */
    private fun assertFieldTextUnderHeading(startInset: Dp) {
        show(startInset)
        /** Bounds of the section heading above the rows. */
        val heading: DpRect = composeTestRule.onNode(hasText("Fields"), useUnmergedTree = true)
            .getUnclippedBoundsInRoot()
        assertEquals("heading column", 16f + startInset.value, heading.left.value, COLUMN_SLACK_DP)
        assertEquals("field text column", heading.left.value, firstFieldLabelBounds().left.value, COLUMN_SLACK_DP)
    }

    // What:     `fieldTextSitsUnderTheHeadingOnTheCover` checks the editor without a crease inset.
    // Why:      The ring used to reserve 3 dp of padding inside the row, which put the field text at 19 dp while the
    //           headings stayed at 16 dp.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("field text under the heading", () => { expect(label.left).toBe(heading.left); });
    // ```
    /** With no inset the field's label starts at the heading's x. */
    @Test(timeout = CASE_LIMIT_MS)
    fun fieldTextSitsUnderTheHeadingOnTheCover() {
        assertFieldTextUnderHeading(0.dp)
    }

    // What:     `fieldTextSitsUnderTheHeadingBesideTheCrease` checks the editor with the crease inset.
    // Why:      The crease inset moves the whole editor column, so the label and the heading must move together.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("field text under the heading beside the crease", () => { expect(label.left).toBe(heading.left); });
    // ```
    /** With the crease inset of the inner display the label starts at the heading's x. */
    @Test(timeout = CASE_LIMIT_MS)
    fun fieldTextSitsUnderTheHeadingBesideTheCrease() {
        assertFieldTextUnderHeading(CREASE_INSET_DP.dp)
    }

    // What:     `fieldTextKeepsItsGapToTheRingInsideTheRow` checks the room between the ring and the text.
    // Why:      The ring lies inside the row's edge (D120) and the text must keep 8 dp from its inner edge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("gap to the ring", () => { expect(label.left - row.left - 3).toBeGreaterThanOrEqual(8); });
    // ```
    /** The label starts at least 8 dp past the 3 dp ring from the row's edge. */
    @Test(timeout = CASE_LIMIT_MS)
    fun fieldTextKeepsItsGapToTheRingInsideTheRow() {
        show(CREASE_INSET_DP.dp)
        /** Distance from the ring's inner edge to the start of the label. */
        val gap: Float = firstFieldLabelBounds().left.value - firstFieldRowBounds().left.value - EDITOR_RING_DEPTH_DP
        assertTrue("gap from ring to text is $gap dp", gap >= EDITOR_RING_TEXT_GAP_DP)
    }

    // What:     `fieldRowBoxReachesThePaneEdgeBesideTheCrease` checks the box of a field row with the crease inset.
    // Why:      The crease moves text only (D130): the row's box, its state layer and its inside ring keep the pane's
    //           edges, so the row spans the pane while its label starts at the heading's x.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("field row box spans the pane", () => { expect(row.left).toBe(0); expect(row.right).toBe(paneWidth); });
    // ```
    /** With the crease inset the first field row still starts at x 0 and ends at the pane's end. */
    @Test(timeout = CASE_LIMIT_MS)
    fun fieldRowBoxReachesThePaneEdgeBesideTheCrease() {
        show(CREASE_INSET_DP.dp)
        /** Bounds of the first field row's box. */
        val box: DpRect = firstFieldRowBounds()
        assertEquals("row box start", 0f, box.left.value, COLUMN_SLACK_DP)
        assertEquals("row box end", EDITOR_PANE_WIDTH_DP, box.right.value, COLUMN_SLACK_DP)
    }

    // What:     `previewTextSitsUnderTheHeadingBesideTheCrease` checks the preview rows after the inset moved.
    // Why:      The inset used to pad the whole scrolling column and is now added to each text, so every text must
    //           still start in the heading column.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("preview under the heading", () => { expect(previewTitle.left).toBe(previewHeading.left); });
    // ```
    /** The first preview row's title starts at the x of the `Preview` heading. */
    @Test(timeout = CASE_LIMIT_MS)
    fun previewTextSitsUnderTheHeadingBesideTheCrease() {
        show(CREASE_INSET_DP.dp)
        /** Bounds of the `Preview` heading. */
        val heading: DpRect = composeTestRule.onAllNodes(hasText("Preview"), useUnmergedTree = true)[0]
            .getUnclippedBoundsInRoot()
        /** Bounds of the first preview row's title, the topmost node with that text. */
        val title: DpRect = composeTestRule
            .onAllNodes(hasText(state.previewRows.first().title), useUnmergedTree = true)[0]
            .getUnclippedBoundsInRoot()
        assertEquals("heading column", 16f + CREASE_INSET_DP, heading.left.value, COLUMN_SLACK_DP)
        assertEquals("preview title column", heading.left.value, title.left.value, COLUMN_SLACK_DP)
    }
}
