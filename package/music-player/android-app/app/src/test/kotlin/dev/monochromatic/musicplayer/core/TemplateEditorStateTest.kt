// What:     `package dev.monochromatic.musicplayer.core` places these tests beside the editor state they check.
// Why:      The tests call the state's public API directly, as the editor composable does.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's equality and truth assertions and the test annotation register each check.
// Why:      Each editor transition is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the editor state's transitions, previews, validation, notes, fields and help. */
class TemplateEditorStateTest {
    /** Sentence for the misspelt field peek. */
    private val PEEK_SENTENCE: String = "mi has no field called peek. Pick a name from the Fields list."

    /** Sentence for the misspelt field seek. */
    private val SEEK_SENTENCE: String = "mi has no field called seek. Pick a name from the Fields list."

    // What:     `private val anotherXronixle` is the first authored library track of the study.
    // Why:      The expected preview lines in these tests come from the study's copy of this track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const anotherXronixle = { title: "Another Xronixle", len: 275, peak: "\u22121.2 dBTP", place: 1, total: 16 };
    // ```
    /** The analysed library track the study previews first. */
    private val anotherXronixle = PlayingTrackFields(
        track = TrackFields(
            title = "Another Xronixle",
            file = "かめりあ(Camellia) - Another Xronixle",
            ext = "flac",
            folder = "Camellia",
            path = "Camellia/かめりあ(Camellia) - Another Xronixle.flac",
            len = 275.0,
            peak = "−1.2 dBTP",
        ),
        placeInFolder = 1,
        tracksInFolder = 16,
    )

    // What:     `private val burningAquamarine` is the second authored library track, not yet analysed.
    // Why:      Its missing peak shows that an empty field leaves the literal text around it in place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const burningAquamarine = { title: "Burning Aquamarine", len: 312, peak: undefined, place: 2, total: 16 };
    // ```
    /** The library track that has no true peak yet. */
    private val burningAquamarine = PlayingTrackFields(
        track = TrackFields(
            title = "Burning Aquamarine",
            file = "Burning Aquamarine",
            ext = "flac",
            folder = "Camellia",
            path = "Camellia/Burning Aquamarine.flac",
            len = 312.0,
            peak = null,
        ),
        placeInFolder = 2,
        tracksInFolder = 16,
    )

    // What:     `private val library` is the two authored library tracks in folder order.
    // Why:      Most tests start from the same two tracks the study previews.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const library = [anotherXronixle, burningAquamarine];
    // ```
    /** The two authored library tracks the editor previews. */
    private val library: List<PlayingTrackFields> = listOf(anotherXronixle, burningAquamarine)

    // What:     `private fun TemplateEditorState.typed(text: String)` types a whole text with the caret at its end.
    // Why:      Most tests only need the text replaced, so one call keeps them short.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const typed = (state, text) => state.withEdit({ text, selectionStart: text.length, selectionEnd: text.length });
    // ```
    /** Types a whole template text with the caret at its end. */
    private fun TemplateEditorState.typed(text: String): TemplateEditorState =
        withEdit(TemplateEditorEdit(text = text, selectionStart = text.length, selectionEnd = text.length))

    // What:     `fun startsOnTrackRowsAtItsDefault` checks the editor's opening template and text.
    // Why:      The editor opens on the track rows template, at its default, with nothing to reset.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("starts on track rows at its default", () => { ... });
    // ```
    /** The editor opens on the track rows template at its default text. */
    @Test
    fun startsOnTrackRowsAtItsDefault() {
        /** The editor state at its first opening with the study's library. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
        assertEquals(TemplateKind.TRACK_ROWS, state.kind)
        assertEquals(DEFAULT_TRACK_TEMPLATE, state.text)
        assertFalse(state.canReset)
        assertTrue(state.isValid)
    }

    // What:     `fun settingsLinesShowEachTemplateLine` checks what Settings lists at opening.
    // Why:      Settings lists both templates with the line each yields for the first library track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("settings lines", () => expect(state.settingsLines).toEqual({ ... }));
    // ```
    /** Each template's Settings line matches the study's copy. */
    @Test
    fun settingsLinesShowEachTemplateLine() {
        /** The editor state at its first opening. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
        assertEquals(
            mapOf(
                TemplateKind.TRACK_ROWS to "4:35 −1.2 dBTP",
                TemplateKind.PLAYING_TRACK to "1 of 16 −1.2 dBTP",
            ),
            state.settingsLines,
        )
    }

    // What:     `fun trackRowPreviewMatchesStudyCopy` checks the track rows preview against the study.
    // Why:      The default line of the analysed track has no trailing space, and the unanalysed one keeps its space.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track row preview", () => expect(state.previewRows).toEqual([...]));
    // ```
    /** The two preview rows read the study's authored lines exactly. */
    @Test
    fun trackRowPreviewMatchesStudyCopy() {
        /** The editor state at its first opening. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
        assertEquals(
            listOf(
                TemplatePreviewRow("Another Xronixle", "4:35 −1.2 dBTP"),
                TemplatePreviewRow("Burning Aquamarine", "5:12 "),
            ),
            state.previewRows,
        )
    }

    // What:     `fun playingTrackPreviewMatchesStudyCopy` checks the playing track's preview lines.
    // Why:      The playing track reads its place and folder total, so its lines differ from the track rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing preview", () => expect(...).toEqual([...]));
    // ```
    /** The playing track's two preview rows read the study's authored lines exactly. */
    @Test
    fun playingTrackPreviewMatchesStudyCopy() {
        /** The editor state opened on the playing track. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).open(TemplateKind.PLAYING_TRACK)
        assertEquals(
            listOf(
                TemplatePreviewRow("Another Xronixle", "1 of 16 −1.2 dBTP"),
                TemplatePreviewRow("Burning Aquamarine", "2 of 16 "),
            ),
            state.previewRows,
        )
    }

    // What:     `fun libraryPreviewNoteNamesTheLibrary` checks the note while the library has tracks.
    // Why:      The user should see that the rows come from the library, not from sample values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("library note", () => expect(state.previewNotes).toEqual(["From your library."]));
    // ```
    /** A library with tracks draws the library note and no sample note. */
    @Test
    fun libraryPreviewNoteNamesTheLibrary() {
        /** The editor state with the study's library. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
        assertEquals(listOf(TEMPLATE_LIBRARY_NOTE), state.previewNotes)
        assertFalse(state.usesSampleValues)
    }

    // What:     `fun emptyLibraryUsesSampleValues` checks the stand-in preview while the library holds no track.
    // Why:      D95 requires sample values and the note that says so while the open library is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library", () => expect(initial([]).previewNotes).toEqual([SAMPLE_NOTE]));
    // ```
    /** An empty library previews the stand-ins and draws the sample note. */
    @Test
    fun emptyLibraryUsesSampleValues() {
        /** The editor state for a library with no tracks. */
        val state: TemplateEditorState = TemplateEditorState.initial(emptyList())
        assertTrue(state.usesSampleValues)
        assertEquals(listOf(TEMPLATE_SAMPLE_NOTE), state.previewNotes)
        assertEquals(
            listOf(
                TemplatePreviewRow("Track title", "3:20 −1.0 dBTP"),
                TemplatePreviewRow("Track not analyzed yet", "3:20 "),
            ),
            state.previewRows,
        )
    }

    // What:     `fun emptyLibraryFieldsShowStandIns` checks the field list while the library is empty.
    // Why:      The field values come from the stand-in first track, as the study's empty-library scene does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("stand-in fields", () => expect(initial([]).fields[0].value).toBe("Track title"));
    // ```
    /** The first field value is the stand-in title while the library is empty. */
    @Test
    fun emptyLibraryFieldsShowStandIns() {
        /** The field list of the editor with an empty library. */
        val fields: List<TemplateEditorFieldEntry> = TemplateEditorState.initial(emptyList()).fields
        assertEquals("Track title", fields[0].value)
        assertEquals("3:20", fields[5].value)
    }

    // What:     `fun libraryPreviewTakesFirstTwoTracks` checks that the preview draws two rows from a longer library.
    // Why:      The preview shows two files, so a longer library must not add rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("two rows", () => expect(initial([a, b, c]).previewTracks.length).toBe(2));
    // ```
    /** A library of three tracks previews only the first two. */
    @Test
    fun libraryPreviewTakesFirstTwoTracks() {
        /** A third library track that the preview must not draw. */
        val third: PlayingTrackFields = anotherXronixle.copy(placeInFolder = 3)
        /** The editor state with three library tracks. */
        val state: TemplateEditorState = TemplateEditorState.initial(library + third)
        assertEquals(2, state.previewRows.size)
    }

    // What:     `fun trackRowFieldsListSevenWithFirstTrackValues` checks the track rows field list.
    // Why:      The list shows the seven track fields, each with the value of the first preview track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track row fields", () => expect(state.fields.length).toBe(7));
    // ```
    /** The track rows list has seven fields whose values come from the first track. */
    @Test
    fun trackRowFieldsListSevenWithFirstTrackValues() {
        /** The field list of the track rows editor. */
        val fields: List<TemplateEditorFieldEntry> = TemplateEditorState.initial(library).fields
        assertEquals(7, fields.size)
        assertEquals(TemplateEditorFieldEntry("Title", "title", "mi(title)", "Another Xronixle"), fields[0])
        assertEquals("4:35", fields[5].value)
        assertEquals("−1.2 dBTP", fields[6].value)
    }

    // What:     `fun playingFieldsAddPlaceAndTotal` checks the playing track's two extra fields.
    // Why:      The playing track reads its place in the folder and the folder's track count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing fields", () => expect(fields.slice(-2)).toEqual([...]));
    // ```
    /** The playing track's field list ends with place in folder and tracks in folder. */
    @Test
    fun playingFieldsAddPlaceAndTotal() {
        /** The field list of the playing track editor. */
        val fields: List<TemplateEditorFieldEntry> = TemplateEditorState.initial(library)
            .open(TemplateKind.PLAYING_TRACK)
            .fields
        assertEquals(9, fields.size)
        assertEquals(TemplateEditorFieldEntry("Place in folder", "track", "mi(track)", "1"), fields[7])
        assertEquals(TemplateEditorFieldEntry("Tracks in folder", "total", "mi(total)", "16"), fields[8])
    }

    // What:     `fun missingPeakFieldIsEmpty` checks a field whose track has no value.
    // Why:      A field with no value must show as empty so the list can say so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("missing peak", () => expect(fields.at(6).value).toBe(""));
    // ```
    /** A first track without a peak gives its peak field an empty value. */
    @Test
    fun missingPeakFieldIsEmpty() {
        /** The field list when the unanalysed track is first. */
        val fields: List<TemplateEditorFieldEntry> = TemplateEditorState.initial(
            listOf(burningAquamarine, anotherXronixle),
        ).fields
        assertEquals("", fields[6].value)
    }

    // What:     `fun customTemplateChangesBothRows` types a valid custom template and reads both rows.
    // Why:      A template that applies changes every row at once, and the study's custom scene shows it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("custom", () => expect(state.typed(custom).previewRows).toEqual([...]));
    // ```
    /** The custom template replaces both preview lines and offers the way back. */
    @Test
    fun customTemplateChangesBothRows() {
        /** The editor state after the custom template is typed. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed(
            "\$tc(up, mi(ext))\$ · \$tf(mi(len), m:ss)\$ · \$mi(peak)\$",
        )
        assertTrue(state.isValid)
        assertTrue(state.canReset)
        assertEquals(
            listOf(
                TemplatePreviewRow("Another Xronixle", "FLAC · 4:35 · −1.2 dBTP"),
                TemplatePreviewRow("Burning Aquamarine", "FLAC · 5:12 · "),
            ),
            state.previewRows,
        )
    }

    // What:     `fun unknownFieldIsRefusedAndRowsKeepDefault` types a misspelt field.
    // Why:      A mistake is reported on its own line while the rows keep the last template that applied.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown field", () => expect(state.typed(bad).mistakeLines).toEqual(["mi: unknown field peek"]));
    // ```
    /** A misspelt field gives one error line and leaves the rows on the default template. */
    @Test
    fun unknownFieldIsRefusedAndRowsKeepDefault() {
        /** The editor state after the misspelt field is typed. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$tf(mi(len), m:ss)\$ \$mi(peek)\$")
        assertFalse(state.isValid)
        assertEquals(listOf(PEEK_SENTENCE), state.mistakeLines)
        assertEquals(TemplateEditorState.initial(library).previewRows, state.previewRows)
        assertEquals(listOf(TEMPLATE_LIBRARY_NOTE, TEMPLATE_KEPT_NOTE), state.previewNotes)
    }

    // What:     `fun openFormulaIsReported` types a formula with no closing dollar sign.
    // Why:      A formula left open is reported, never shown as literal text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("open formula", () => expect(state.typed("$tf(mi(len), m:ss)").mistakeLines).toEqual([...]));
    // ```
    /** A formula with no closing dollar sign gives the study's error line. */
    @Test
    fun openFormulaIsReported() {
        /** The editor state after the open formula is typed. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed("\$tf(mi(len), m:ss)")
        assertEquals(
            listOf("The \$ at character 1 has no closing \$. Add a \$ where the formula ends."),
            state.mistakeLines,
        )
    }

    // What:     `fun trackOnlyFieldIsRefusedInTrackRows` types the playing track's field into the track rows template.
    // Why:      Track rows never see the folder place, so the field is refused there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track field refused", () => expect(state.typed("$mi(track)$").isValid).toBe(false));
    // ```
    /** The place field is an unknown field in the track rows template. */
    @Test
    fun trackOnlyFieldIsRefusedInTrackRows() {
        /** The editor state after the playing track's field is typed into track rows. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed("\$mi(track)\$")
        assertEquals(listOf("mi has no field called track. Pick a name from the Fields list."), state.mistakeLines)
    }

    // What:     `fun validEditBecomesAppliedText` types a valid template that differs from the custom one.
    // Why:      A valid text becomes the applied template, which the Settings line then shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("valid edit", () => expect(state.typed("$mi(len)$").settingsLine("track")).toBe("275"));
    // ```
    /** A valid edit replaces the applied template, so the Settings line follows it. */
    @Test
    fun validEditBecomesAppliedText() {
        /** The editor state after a valid edit. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed("\$mi(len)\$")
        assertEquals("275", state.settingsLine(TemplateKind.TRACK_ROWS))
    }

    // What:     `fun invalidEditKeepsLastValidForSettings` types a mistake after a valid custom template.
    // Why:      While the text is invalid the Settings line and the rows keep the last template that applied.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kept", () => expect(state.typed(custom).typed(bad).settingsLine("track")).toBe(customLine));
    // ```
    /** A mistake leaves the applied template, and the Settings line, unchanged. */
    @Test
    fun invalidEditKeepsLastValidForSettings() {
        /** The editor state after the custom template and then a mistake. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$tc(up, mi(ext))\$ · \$tf(mi(len), m:ss)\$ · \$mi(peak)\$")
            .typed("\$mi(peek)\$")
        assertEquals("FLAC · 4:35 · −1.2 dBTP", state.settingsLine(TemplateKind.TRACK_ROWS))
        assertEquals("\$mi(peek)\$", state.text)
    }

    // What:     `fun editsStayWithTheirOwnTemplate` changes the playing track and then opens the track rows.
    // Why:      Each template keeps its own text, so opening one must not change the other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("separate", () => expect(state.open("track").text).toBe(DEFAULT_TRACK_TEMPLATE));
    // ```
    /** Editing the playing track leaves the track rows text at its default. */
    @Test
    fun editsStayWithTheirOwnTemplate() {
        /** The editor state after the playing track is edited and then the track rows are opened. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .open(TemplateKind.PLAYING_TRACK)
            .typed("\$mi(total)\$")
            .open(TemplateKind.TRACK_ROWS)
        assertEquals(DEFAULT_TRACK_TEMPLATE, state.text)
        assertEquals("\$mi(total)\$", state.open(TemplateKind.PLAYING_TRACK).text)
    }

    // What:     `fun openPutsCaretAtEndWithoutFocus` checks what opening a template leaves focused.
    // Why:      A template opened from Settings starts unfocused, with the caret at its end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("open", () => expect(state.open("playing")).toMatchObject({ fieldFocused: false }));
    // ```
    /** Opening a template puts the caret at the end of its text and clears focus. */
    @Test
    fun openPutsCaretAtEndWithoutFocus() {
        /** The editor state after the playing track is opened with focus cleared. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .withFocus(true)
            .open(TemplateKind.PLAYING_TRACK)
        assertFalse(state.fieldFocused)
        assertEquals(TemplateKind.PLAYING_TRACK.defaultTemplate.length, state.selectionStart)
    }

    // What:     `fun insertFieldOutsideFormulaWrapsCall` inserts a field at a caret outside any formula.
    // Why:      A call outside a formula would show as literal text, so the editor adds the dollar signs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("insert outside", () => expect(state.insertField("len").text).toBe("$mi(len)$"));
    // ```
    /** A field inserted outside a formula is wrapped in dollar signs with the caret before the closing one. */
    @Test
    fun insertFieldOutsideFormulaWrapsCall() {
        /** The editor state with an empty template and the caret at its start. */
        val start: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = "", selectionStart = 0, selectionEnd = 0))
        /** The editor state after the field is inserted. */
        val state: TemplateEditorState = start.insertField("len")
        assertEquals("\$mi(len)\$", state.text)
        assertEquals(8, state.selectionStart)
        assertEquals(8, state.selectionEnd)
    }

    // What:     `fun insertFieldInsideFormulaWritesBareCall` inserts a field at a caret inside a formula.
    // Why:      Inside a formula the call is written bare, so the formula does not gain nested delimiters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("insert inside", () => expect(state.insertField("len").text).toBe("$tf(mi(len), m:ss)$"));
    // ```
    /** A field inserted inside a formula is written bare, with the caret after its closing bracket. */
    @Test
    fun insertFieldInsideFormulaWritesBareCall() {
        /** The editor state with the caret after the open bracket of the duration call. */
        val start: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = "\$tf(, m:ss)\$", selectionStart = 4, selectionEnd = 4))
        /** The editor state after the field is inserted. */
        val state: TemplateEditorState = start.insertField("len")
        assertEquals("\$tf(mi(len), m:ss)\$", state.text)
        assertEquals(11, state.selectionStart)
    }

    // What:     `fun insertFieldReplacesSelection` inserts a field over a selected word.
    // Why:      A selection is replaced by the inserted call, as a text editor does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("replace selection", () => expect(state.insertField("peak").text).toBe("$tf(mi(len), mi(peak))$"));
    // ```
    /** A selected word is replaced by the inserted call. */
    @Test
    fun insertFieldReplacesSelection() {
        /** The editor state with the format word selected inside the duration call. */
        val start: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = "\$tf(mi(len), m:ss)\$", selectionStart = 13, selectionEnd = 17))
        /** The editor state after the peak field replaces the selection. */
        val state: TemplateEditorState = start.insertField("peak")
        assertEquals("\$tf(mi(len), mi(peak))\$", state.text)
        assertEquals(21, state.selectionStart)
    }

    // What:     `fun insertFunctionPlacesCaretInsideBrackets` inserts an empty call of a function.
    // Why:      The caret lands inside the brackets so the user types the arguments next.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("insert function", () => expect(state.insertFunction("tf").text).toBe("$tf()$"));
    // ```
    /** A function inserted outside a formula gives an empty call with the caret inside its brackets. */
    @Test
    fun insertFunctionPlacesCaretInsideBrackets() {
        /** The editor state with an empty template and the caret at its start. */
        val start: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = "", selectionStart = 0, selectionEnd = 0))
        /** The editor state after the function is inserted. */
        val state: TemplateEditorState = start.insertFunction("tf")
        assertEquals("\$tf()\$", state.text)
        assertEquals(4, state.selectionStart)
    }

    // What:     `fun resetToDefaultRestoresTheTemplate` returns an edited template to its default.
    // Why:      The way back to the default must restore the text, the rows and the caret position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("reset", () => expect(state.typed(custom).resetToDefault().text).toBe(DEFAULT));
    // ```
    /** Resetting restores the default text, the default line and the caret at its end. */
    @Test
    fun resetToDefaultRestoresTheTemplate() {
        /** The editor state after a custom template is reset. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$mi(len)\$")
            .resetToDefault()
        assertEquals(DEFAULT_TRACK_TEMPLATE, state.text)
        assertFalse(state.canReset)
        assertEquals("4:35 −1.2 dBTP", state.settingsLine(TemplateKind.TRACK_ROWS))
        assertEquals(DEFAULT_TRACK_TEMPLATE.length, state.selectionStart)
    }

    // What:     `fun canResetOnlyWhenTextDiffers` checks when the way back is offered.
    // Why:      The way back is shown only when the template differs from its default.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("canReset", () => expect(initial.canReset).toBe(false));
    // ```
    /** The way back is offered after an edit and not at the default. */
    @Test
    fun canResetOnlyWhenTextDiffers() {
        /** The editor state at its default text. */
        val atDefault: TemplateEditorState = TemplateEditorState.initial(library)
        assertFalse(atDefault.canReset)
        assertTrue(atDefault.typed("\$mi(len)\$").canReset)
    }

    // What:     `fun helpShowsOnlyWhileFieldIsFocused` checks the typing help against focus.
    // Why:      Help appears only while the field has focus, because only then is the caret typing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("help focus", () => expect(state.help).toBeUndefined());
    // ```
    /** The help names the duration's format argument only while the field has focus. */
    @Test
    fun helpShowsOnlyWhileFieldIsFocused() {
        /** The editor state with the caret inside the duration call's format argument. */
        val unfocused: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = DEFAULT_TRACK_TEMPLATE, selectionStart = 15, selectionEnd = 15))
        assertNull(unfocused.help)
        /** The same state with the field focused. */
        val focused: TemplateEditorState = unfocused.withFocus(true)
        /** The help for the call around the caret. */
        val help: TemplateHelp? = focused.help
        assertEquals("tf", help?.name)
        assertEquals("tf(seconds, [format])", help?.signature)
        assertEquals("format", help?.parameter)
    }

    // What:     `fun helpIsNullOutsideCalls` checks the help with the caret outside every call.
    // Why:      With the caret outside a call there is no signature to show.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("outside", () => expect(helpAt(text, 0)).toBeUndefined());
    // ```
    /** A caret before the first call's bracket gives no help. */
    @Test
    fun helpIsNullOutsideCalls() {
        /** The editor state with the caret at the template's first character, outside any call. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = DEFAULT_TRACK_TEMPLATE, selectionStart = 0, selectionEnd = 0))
            .withFocus(true)
        assertNull(state.help)
    }

    // What:     `fun selectionIsClampedToTheText` checks an edit whose caret runs past the end.
    // Why:      The state must never hold a caret position outside its text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clamp", () => expect(withEdit({ text: "abc", selectionStart: 10 }).selectionStart).toBe(3));
    // ```
    /** Caret positions past the end of the text are brought back to its end. */
    @Test
    fun selectionIsClampedToTheText() {
        /** The editor state after an edit whose caret is past the end of its text. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .withEdit(TemplateEditorEdit(text = "abc", selectionStart = 10, selectionEnd = 12))
        assertEquals(3, state.selectionStart)
        assertEquals(3, state.selectionEnd)
    }

    // What:     `fun errorLinesKeepTemplateOrder` checks that two mistakes stay in template order.
    // Why:      The editor lists mistakes where they sit in the template, so the user reads them top down.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("order", () => expect(state.mistakeLines[0]).toContain("peek"));
    // ```
    /** Two unknown fields give two lines in the order they appear. */
    @Test
    fun errorLinesKeepTemplateOrder() {
        /** The editor state after a template with two unknown fields. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed("\$mi(peek)\$ \$mi(seek)\$")
        assertEquals(listOf(PEEK_SENTENCE, SEEK_SENTENCE), state.mistakeLines)
    }

    /** A template with an unknown field and a missing bracket lists both mistakes, not only the first. */
    @Test
    fun everyMistakeIsListedAsAPlainSentence() {
        /** The editor state after the template of the polish review's example. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$mi(bogus)\$ and \$tf(mi(len)\$")
        assertEquals(
            listOf(
                "mi has no field called bogus. Pick a name from the Fields list.",
                "tf is missing its closing bracket. Add a ) after its last value.",
            ),
            state.mistakeLines,
        )
    }

    /** One formula with several slips gets one sentence, so a single typo is not listed three times. */
    @Test
    fun aFormulaWithSeveralSlipsGetsOneSentence() {
        /** The editor state after one formula with an unknown field and an unknown mode. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$tc(zap, mi(bogus))\$")
        assertEquals(1, state.mistakeLines.size)
    }

    /** A format letter the desktop rejects is rejected here too, with a sentence that says what to use. */
    @Test
    fun anUnknownFormatLetterIsRefusedWithAPlainSentence() {
        /** The editor state after a duration format with a stray letter. */
        val state: TemplateEditorState = TemplateEditorState.initial(library).typed("\$tf(mi(len), q:q)\$")
        assertFalse(state.isValid)
        assertEquals(
            listOf("tf does not know the format letter q. Use h, m or s, and put any other words between apostrophes."),
            state.mistakeLines,
        )
    }

    /** Sentences are listed in template order whichever mistake the evaluator found first. */
    @Test
    fun aParseMistakeBeforeAnUnknownFieldKeepsTemplateOrder() {
        /** The editor state after a missing bracket followed by an unknown field. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$tf(mi(len)\$ and \$mi(bogus)\$")
        assertEquals(
            listOf(
                "tf is missing its closing bracket. Add a ) after its last value.",
                "mi has no field called bogus. Pick a name from the Fields list.",
            ),
            state.mistakeLines,
        )
    }

    /** While the draft is invalid the Settings line keeps showing the last valid template's text. */
    @Test
    fun settingsLineKeepsTheLastValidTemplateWhileTheDraftIsInvalid() {
        /** The editor after one valid change and then an invalid draft. */
        val valid: TemplateEditorState = TemplateEditorState.initial(library).typed("\$mi(title)\$ here")
        val invalid: TemplateEditorState = valid.typed("\$mi(bogus)\$ and \$tf(mi(len)\$")
        assertFalse(invalid.isValid)
        assertEquals("Another Xronixle here", valid.settingsLine(TemplateKind.TRACK_ROWS))
        assertEquals(valid.settingsLine(TemplateKind.TRACK_ROWS), invalid.settingsLine(TemplateKind.TRACK_ROWS))
        assertEquals(valid.previewRows, invalid.previewRows)
    }

    // What:     `fun resetWithoutMistakeHidesKeptNote` checks the note once the template applies again.
    // Why:      The kept-template note must disappear as soon as the text is valid.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("note", () => expect(state.previewNotes).not.toContain(KEPT_NOTE));
    // ```
    /** A valid template removes the kept-template note from the preview. */
    @Test
    fun resetWithoutMistakeHidesKeptNote() {
        /** The editor state after a mistake and then a valid template. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
            .typed("\$mi(peek)\$")
            .typed("\$mi(len)\$")
        assertFalse(state.previewNotes.contains(TEMPLATE_KEPT_NOTE))
    }

    // What:     `fun settingsLineForEmptyLibraryUsesStandIns` checks the Settings line with no library tracks.
    // Why:      The Settings page shows the same stand-in line the editor preview shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty settings", () => expect(initial([]).settingsLine("track")).toBe("3:20 \u22121.0 dBTP"));
    // ```
    /** The stand-in track's line is what Settings shows for an empty library. */
    @Test
    fun settingsLineForEmptyLibraryUsesStandIns() {
        assertEquals("3:20 −1.0 dBTP", TemplateEditorState.initial(emptyList()).settingsLine(TemplateKind.TRACK_ROWS))
    }

    // What:     `fun focusChangeKeepsText` checks that focus changes do not touch the text.
    // Why:      The field reports focus separately from edits, so focus must leave the template alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("focus", () => expect(state.withFocus(true).text).toBe(state.text));
    // ```
    /** Focusing and unfocusing the field leaves the template text unchanged. */
    @Test
    fun focusChangeKeepsText() {
        /** The editor state at its default text. */
        val state: TemplateEditorState = TemplateEditorState.initial(library)
        assertEquals(state.text, state.withFocus(true).withFocus(false).text)
    }

    // What:     `fun trackWithoutDurationOrPeakKeepsDefaultRows` opens the editor on a track with no metadata.
    // Why:      Production tracks usually carry no duration, so the defaults must still apply without throwing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("bare track", () => expect(initial([bare]).settingsLines).toBeDefined());
    // ```
    /** A track with no duration, peak, place or folder still gives both default lines without an error. */
    @Test
    fun trackWithoutDurationOrPeakKeepsDefaultRows() {
        /** A track whose duration, peak, place and folder total are all unknown. */
        val bare: PlayingTrackFields = PlayingTrackFields(
            track = TrackFields(
                title = "Bare",
                file = "Bare",
                ext = "mp3",
                folder = "",
                path = "Bare.mp3",
                len = null,
                peak = null,
            ),
            placeInFolder = null,
            tracksInFolder = null,
        )
        /** The editor state opened on the bare track. */
        val state: TemplateEditorState = TemplateEditorState.initial(listOf(bare))
        assertTrue(state.isValid)
        assertEquals(listOf(TemplatePreviewRow("Bare", " ")), state.previewRows)
        assertEquals(" of  ", state.open(TemplateKind.PLAYING_TRACK).settingsLine(TemplateKind.PLAYING_TRACK))
    }
}
