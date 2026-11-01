// Wiring cases of the Search query field's caret, rendered for real under Robolectric: the query is kept by the host
// across a fold change, so the rebuilt header must put the caret after the kept text and typing must continue there;
// the Search shortcut must do the same in its own event, before any later character is read.

// What:     `package ...wiring` groups the Robolectric Compose tests, which share the fixtures of this package.
// Why:      The cases reach `searchPage` and the shortcut port of the host keyboard.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusManager
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextInputSelection
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.unit.dp
import dev.monochromatic.musicplayer.HostKeyLocalSearchFocus
import dev.monochromatic.musicplayer.SearchFocusPort
import dev.monochromatic.musicplayer.core.SEARCH_CLEAR_LABEL
import dev.monochromatic.musicplayer.core.SEARCH_FIELD_LABEL
import dev.monochromatic.musicplayer.core.SearchState
import dev.monochromatic.musicplayer.searchPage
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** Qualifier of the unfolded inner panel: 851 dp wide at three physical pixels per dp. */
private const val INNER_PANEL = "w851dp-h700dp-xxhdpi"

/** Width in dp of the folded cover, which a narrower box gives the page so it draws the cover arrangement. */
private const val COVER_WIDTH_DP: Int = 411

// What:     `class SearchFieldCaretWiringTest` groups the cases of the Search field's caret.
// Why:      The field was fed a bare `String`, so its caret lived in state that the other panel's header dropped; the
//           query stayed (it is kept by the host) but typing after a fold change landed before it (D126). The Search
//           shortcut then moved the caret in an effect two frames later, so a character typed right after the chord
//           still landed before the query; the shortcut now acts in its own event.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("Search field caret", () => { /* type, rebuild the header, type again, read the text and the caret */ });
// ```
/** Renders [searchPage] over a kept query and asserts where typed text lands after a rebuild or a shortcut press. */
@RunWith(RobolectricTestRunner::class)
@Config(qualifiers = INNER_PANEL)
class SearchFieldCaretWiringTest {
    /** Compose test rule that hosts each case's content in an empty activity. */
    @get:Rule
    val composeTestRule = createComposeRule()

    /** Query the host keeps, written by the field and read back by the assertions. */
    private var query: String by mutableStateOf("")

    /** Whether the page is given the folded cover width; flipping it rebuilds the header, as a fold change does. */
    private var onCover: Boolean by mutableStateOf(false)

    /** Port that the host presses on a Search shortcut, which the open page registers its field with. */
    private val shortcutPort: SearchFocusPort = SearchFocusPort()

    /** Whether the Search page is composed; clearing it closes the page, as Back does. */
    private var pageOpen: Boolean by mutableStateOf(true)

    /** Focus manager of the composition, captured so a case can take focus away from the field. */
    private var focusManager: FocusManager? = null

    /** Draws the page over the kept query, with a header that a width change rebuilds. */
    @Composable
    private fun page(keptQuery: String, onQuery: (String) -> Unit) {
        focusManager = LocalFocusManager.current
        MaterialTheme {
            CompositionLocalProvider(HostKeyLocalSearchFocus provides shortcutPort) {
                Box(modifier = if (onCover) Modifier.width(COVER_WIDTH_DP.dp) else Modifier) {
                    if (pageOpen) searchPage(
                        creaseWidthPx = null,
                        state = SearchState.EmptyQuery,
                        query = keptQuery,
                        onQueryChange = onQuery,
                        onBack = { },
                        onClear = { onQuery("") },
                        onOpenResult = { },
                        currentTrackIndex = null,
                        deck = { Box(modifier = Modifier) },
                    )
                }
            }
        }
    }

    /** Shows the page with the host-kept [query] and waits for its first frames. */
    private fun show() {
        composeTestRule.setContent { page(keptQuery = query, onQuery = { text -> query = text }) }
        composeTestRule.waitForIdle()
    }

    /** Types [text] into the field at its caret, as a keyboard does, and waits for the frames. */
    private fun type(text: String) {
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInput(text)
        composeTestRule.waitForIdle()
    }

    /** Moves the field's caret to [offset], as a tap does, and waits for the frames. */
    private fun placeCaret(offset: Int) {
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInputSelection(TextRange(offset))
        composeTestRule.waitForIdle()
    }

    /** Selection of the field as its semantics report it, or null when it reports none. */
    private fun selection(): TextRange? =
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).fetchSemanticsNode().config
            .getOrNull(SemanticsProperties.TextSelectionRange)

    // What:     `@Test fun typingAfterAFoldChangeContinuesAfterTheKeptQuery()` is the reported failure.
    // Why:      "lan" typed again after a fold change gave "lanlan" with the caret at the start (D126); the rebuilt
    //           header must start the field with the caret after the kept query.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("caret after fold", () => { type("lan"); fold(); type("x"); expect(query).toBe("lanx"); });
    // ```
    /** After the header is rebuilt for the cover, the next typed character lands after "lan" and the caret follows. */
    @Test(timeout = CASE_LIMIT_MS)
    fun typingAfterAFoldChangeContinuesAfterTheKeptQuery() {
        show()
        type("lan")
        assertEquals("lan", query)
        composeTestRule.runOnIdle { onCover = true }
        composeTestRule.waitForIdle()
        assertEquals("the fold change keeps the query", "lan", query)
        assertEquals("the rebuilt field puts the caret at the end", TextRange(3), selection())
        type("x")
        assertEquals("lanx", query)
        assertEquals(TextRange(4), selection())
    }

    // What:     `@Test fun theSearchShortcutOnAnOpenPagePutsTheCaretAtTheEndAndSelectsNothing()` is the Ctrl+F case.
    // Why:      A second "lan" typed after Ctrl+F landed in front of the first; the shortcut only refocused the field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ctrl+f caret", () => {
    //   type("lan"); setCaret(0); pressSearchShortcut(); type("x");
    //   expect(query).toBe("lanx");
    // });
    // ```
    /** With the caret at the start, a Search shortcut makes the next typed character land after "lan". */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSearchShortcutOnAnOpenPagePutsTheCaretAtTheEndAndSelectsNothing() {
        show()
        type("lan")
        placeCaret(0)
        assertEquals(TextRange(0), selection())
        composeTestRule.runOnIdle { shortcutPort.press() }
        composeTestRule.waitForIdle()
        assertEquals("the shortcut puts a caret and no selection at the end", TextRange(3), selection())
        type("x")
        assertEquals("lanx", query)
    }

    // What:     `@Test fun aCharacterTypedInTheSameEventAsTheSearchShortcutLandsAfterTheQuery()` is the race case.
    // Why:      The shortcut used to move the caret in an effect two frames after the chord, so a character typed
    //           right after it (7 ms later on the emulator) landed before the query ("ylan"). Pressing the port and
    //           inserting the character in one main-thread block leaves no frame between them: only a caret placed
    //           synchronously by the shortcut passes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("chord then char, no frame between", () => {
    //   type("lan"); setCaret(0);
    //   inOneEvent(() => { pressSearchShortcut(); insertAtCursor("y"); });
    //   expect(query).toBe("lany");
    // });
    // ```
    /** With the caret at the start, a shortcut press and a character inserted in the same event give "lany". */
    @Test(timeout = CASE_LIMIT_MS)
    fun aCharacterTypedInTheSameEventAsTheSearchShortcutLandsAfterTheQuery() {
        show()
        type("lan")
        placeCaret(0)
        // The lookup happens before the press: a semantics lookup inside the block can wait for a frame, which would
        // let a caret that is moved in a later frame arrive before the character.
        /** Action that inserts text at the field's caret, as the keyboard does. */
        val insert = composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).fetchSemanticsNode().config
            .getOrNull(SemanticsActions.InsertTextAtCursor)?.action
        checkNotNull(insert) { "the field offers no insert-at-cursor action" }
        composeTestRule.runOnUiThread {
            shortcutPort.press()
            insert(AnnotatedString("y"))
        }
        composeTestRule.waitForIdle()
        assertEquals("lany", query)
        assertEquals(TextRange(4), selection())
    }

    // What:     `@Test fun aCharacterTypedBeforeTheNextFrameFollowsTheSearchShortcut()` is the paused-clock case.
    // Why:      With the frame clock held, the shortcut press and the typing both happen before any composition; the
    //           typed character must still land after the query.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("chord then char, clock held", () => {
    //   type("lan"); setCaret(0); holdFrames(); pressSearchShortcut(); type("y"); releaseFrames();
    //   expect(query).toBe("lany");
    // });
    // ```
    /** With the frame clock held, a shortcut press followed at once by typed text gives "lany". */
    @Test(timeout = CASE_LIMIT_MS)
    fun aCharacterTypedBeforeTheNextFrameFollowsTheSearchShortcut() {
        show()
        type("lan")
        placeCaret(0)
        composeTestRule.mainClock.autoAdvance = false
        composeTestRule.runOnUiThread { shortcutPort.press() }
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).performTextInput("y")
        composeTestRule.mainClock.autoAdvance = true
        composeTestRule.waitForIdle()
        assertEquals("lany", query)
    }

    // What:     `@Test fun theSearchShortcutRefocusesTheFieldThatLostFocus()` checks the refocus half of the shortcut.
    // Why:      The shortcut exists to bring the person back to the field; moving the caret alone would leave a
    //           field without focus that typing does not reach.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("ctrl+f refocuses", () => { blur(); pressSearchShortcut(); expect(field).toBeFocused(); });
    // ```
    /** After focus leaves the field, the shortcut puts it back on the field. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSearchShortcutRefocusesTheFieldThatLostFocus() {
        show()
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).assertIsFocused()
        composeTestRule.runOnIdle { checkNotNull(focusManager).clearFocus() }
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).assertIsNotFocused()
        composeTestRule.runOnIdle { shortcutPort.press() }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithContentDescription(SEARCH_FIELD_LABEL).assertIsFocused()
    }

    // What:     `@Test fun theSearchShortcutDoesNothingOnceThePageIsClosed()` checks the end of the registration.
    // Why:      A closed page must leave no field behind, or the next shortcut would act on a field that is gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("port empties on close", () => { close(); expect(port.onShortcut).toBeNull(); port.press(); });
    // ```
    /** After the page leaves composition the port holds no action, and pressing it does not fail. */
    @Test(timeout = CASE_LIMIT_MS)
    fun theSearchShortcutDoesNothingOnceThePageIsClosed() {
        show()
        composeTestRule.runOnIdle { pageOpen = false }
        composeTestRule.waitForIdle()
        assertNull("the closed page unregistered its field", shortcutPort.onShortcut)
        composeTestRule.runOnIdle { shortcutPort.press() }
    }

    // What:     `@Test fun aCaretTheUserPlacedInTheMiddleIsKeptWhileTheQueryIsNotRebuilt()` is the control.
    // Why:      A field that put the caret at the end on every frame would pass the cases above and break editing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("caret stays", () => { type("lan"); setCaret(1); type("x"); expect(query).toBe("lxan"); });
    // ```
    /** With the caret moved to offset 1, a typed character lands there, not at the end. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aCaretTheUserPlacedInTheMiddleIsKeptWhileTheQueryIsNotRebuilt() {
        show()
        type("lan")
        placeCaret(1)
        type("x")
        assertEquals("lxan", query)
        assertEquals(TextRange(2), selection())
    }

    // What:     `@Test fun clearingTheQueryThenTypingStartsAFreshQuery()` checks the Clear control.
    // Why:      Clear empties the query (D66 keeps new visits empty the same way); the field must show the empty query
    //           and take the next characters from its start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clear then type", () => { type("lan"); clear(); type("ab"); expect(query).toBe("ab"); });
    // ```
    /** After the Clear control is pressed, typing "ab" gives "ab" with the caret after it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun clearingTheQueryThenTypingStartsAFreshQuery() {
        show()
        type("lan")
        composeTestRule.onNodeWithContentDescription(SEARCH_CLEAR_LABEL).performClick()
        composeTestRule.waitForIdle()
        assertEquals("", query)
        type("ab")
        assertEquals("ab", query)
        assertEquals(TextRange(2), selection())
    }

    // What:     `@Test fun aSavedFieldFromAnEarlierLayoutDoesNotOverwriteTheNewerKeptQuery()` is the unfold case.
    // Why:      A fold or unfold recreates the activity. Compose saves again a value that no composable consumed, so
    //           a header back at its first position was handed the "lan" saved there before "x" was typed on the
    //           cover, and its mirror wrote "lan" over the newer "lanx" (seen on the emulator).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("stale", () => { type("lan"); relaunch(cover); type("x"); relaunch(inner); expect(q).toBe("lanx"); });
    // ```
    /** After inner, cover and inner, each with a recreation, the query is "lanx" and typing goes after it. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aSavedFieldFromAnEarlierLayoutDoesNotOverwriteTheNewerKeptQuery() {
        /** Layout that the next composition draws; a plain value, so only a recreation reads a new one. */
        var coverLayout = false
        /** Restores saved state the way a recreated activity does. */
        val restoration = StateRestorationTester(composeTestRule)
        restoration.setContent {
            /** Query kept as the host keeps it, across recreation. */
            var saved: String by rememberSaveable { mutableStateOf("") }
            if (coverLayout) {
                Box(modifier = Modifier.width(COVER_WIDTH_DP.dp)) {
                    page(keptQuery = saved, onQuery = { text -> saved = text; query = text })
                }
            } else {
                Box(modifier = Modifier) { page(keptQuery = saved, onQuery = { text -> saved = text; query = text }) }
            }
        }
        composeTestRule.waitForIdle()
        type("lan")
        coverLayout = true
        restoration.emulateSavedInstanceStateRestore()
        composeTestRule.waitForIdle()
        assertEquals("lan", query)
        type("x")
        assertEquals("lanx", query)
        coverLayout = false
        restoration.emulateSavedInstanceStateRestore()
        composeTestRule.waitForIdle()
        assertEquals("the newer query survives the return to the first layout", "lanx", query)
        assertEquals(TextRange(4), selection())
        type("v")
        assertEquals("lanxv", query)
    }

    // What:     `@Test fun aRecreatedActivityKeepsTheCaretTheUserPlaced()` restores the saved field state.
    // Why:      The query survives a recreation in the host through saved state; the caret should come back with it
    //           instead of jumping to the end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("recreation keeps the caret", () => {
    //   type("lan"); setCaret(1); recreate(); type("x");
    //   expect(query).toBe("lxan");
    // });
    // ```
    /** With the caret at offset 1 and the state saved and restored, the next typed character lands at offset 1. */
    @Test(timeout = CASE_LIMIT_MS)
    fun aRecreatedActivityKeepsTheCaretTheUserPlaced() {
        /** Restores saved state the way a recreated activity does. */
        val restoration = StateRestorationTester(composeTestRule)
        restoration.setContent {
            /** Query kept as the host keeps it, across recreation. */
            var saved: String by rememberSaveable { mutableStateOf("") }
            page(keptQuery = saved, onQuery = { text -> saved = text; query = text })
        }
        composeTestRule.waitForIdle()
        type("lan")
        placeCaret(1)
        restoration.emulateSavedInstanceStateRestore()
        composeTestRule.waitForIdle()
        assertEquals(TextRange(1), selection())
        type("x")
        assertEquals("lxan", query)
    }
}
