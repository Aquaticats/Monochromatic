// What:     `package dev.monochromatic.musicplayer.core` places the editor state beside the template engine.
// Why:      The editor's values and transitions need no Compose runtime, so the host JVM can test them.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

//region Constants and stand-in tracks
// What:     `const val TEMPLATE_PREVIEW_ROW_COUNT: Int = 2` is how many library tracks the preview draws.
// Why:      Two rows, one analysed and one not, show what a template yields with and without a true peak.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_PREVIEW_ROW_COUNT = 2;
// ```
/** Number of library tracks the editor preview draws as rows. */
const val TEMPLATE_PREVIEW_ROW_COUNT: Int = 2

// What:     `const val TEMPLATE_SAMPLE_NOTE: String` says the preview shows stand-in values.
// Why:      D95 requires the editor to say so whenever the open library holds no track.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_SAMPLE_NOTE = "Your library has no tracks yet. These are sample values.";
// ```
/** Note drawn under the preview while the open library holds no track. */
const val TEMPLATE_SAMPLE_NOTE: String = "Your library has no tracks yet. These are sample values."

// What:     `const val TEMPLATE_UNREAD_NOTE: String` says the preview shows stand-in values until the tracks are read.
// Why:      A library with tracks but no read duration or peak must not claim it is empty, so it says what is missing.
//
/** Note drawn under the preview while the open library's tracks have no duration or peak read yet. */
const val TEMPLATE_UNREAD_NOTE: String = "Track durations and peaks are not read yet. These are sample values."

// What:     `const val TEMPLATE_LIBRARY_NOTE: String` says the preview rows come from the open library.
// Why:      The user should know whose tracks the preview is showing.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_LIBRARY_NOTE = "From your library.";
// ```
/** Note drawn under the preview while it shows tracks from the open library. */
const val TEMPLATE_LIBRARY_NOTE: String = "From your library."

// What:     `const val TEMPLATE_KEPT_NOTE: String` says the rows keep the last template that applied.
// Why:      While the text is invalid the rows do not change, and the note explains why.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_KEPT_NOTE = "Rows keep the last valid template.";
// ```
/** Note drawn under the preview while the typed template does not apply. */
const val TEMPLATE_KEPT_NOTE: String = "Rows keep the last valid template."

// What:     `val TEMPLATE_SAMPLE_TRACKS: List<PlayingTrackFields>` holds the stand-in tracks.
// Why:      D95 uses sample values only while the open library holds no track, and these two show
//           an analysed file and one not yet analysed.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_SAMPLE_TRACKS = [
//   { title: "Track title", len: 200, peak: "\u22121.0 dBTP", place: 1, total: 16 },
//   { title: "Track not analyzed yet", len: 200, peak: undefined, place: 2, total: 16 },
// ];
// ```
/** Stand-in tracks the preview uses while the open library holds no track. */
val TEMPLATE_SAMPLE_TRACKS: List<PlayingTrackFields> = listOf(
    PlayingTrackFields(
        track = TrackFields(
            title = "Track title",
            file = "File name",
            ext = "flac",
            folder = "Folder",
            path = "Folder/File name.flac",
            len = 200.0,
            peak = "−1.0 dBTP",
        ),
        placeInFolder = 1,
        tracksInFolder = 16,
    ),
    PlayingTrackFields(
        track = TrackFields(
            title = "Track not analyzed yet",
            file = "File name",
            ext = "flac",
            folder = "Folder",
            path = "Folder/File name.flac",
            len = 200.0,
            peak = null,
        ),
        placeInFolder = 2,
        tracksInFolder = 16,
    ),
)
//endregion

//region Values the editor passes around
// What:     `data class TemplateEditorEdit(...)` is one edit the field reports: the new text and selection.
// Why:      The field reports text and selection together, so one value carries both into the state.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateEditorEdit = { text: string; selectionStart: number; selectionEnd: number };
// ```
/** One edit the template field reports: the text after it and where the selection sits. */
data class TemplateEditorEdit(
    /** Whole template text after the edit. */
    val text: String,
    /** Selection start after the edit, in characters. */
    val selectionStart: Int,
    /** Selection end after the edit; equal to the start when there is only a caret. */
    val selectionEnd: Int,
)

// What:     `data class TemplatePreviewRow(...)` is one row the preview draws.
// Why:      The editor shows each track's title with the line the applied template yields for it.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplatePreviewRow = { title: string; supporting: string };
// ```
/** One preview row: a track's title and the line the applied template yields for it. */
data class TemplatePreviewRow(
    /** The track's title as the row shows it. */
    val title: String,
    /** The applied template's line for the track; empty when the template yields nothing. */
    val supporting: String,
)

// What:     `data class TemplateEditorFieldEntry(...)` is one insertable field of the field list.
// Why:      The list shows each field's name, the call a tap inserts, and a value to show what it holds.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateEditorFieldEntry = { label: string; mode: string; insert: string; value: string };
// ```
/** One insertable field: its label, its mode word, the call a tap inserts, and its value. */
data class TemplateEditorFieldEntry(
    /** Label the field list shows for the field. */
    val label: String,
    /** Mode word used inside the call, such as `len`. */
    val mode: String,
    /** Call text a tap inserts at the caret, such as `mi(len)`. */
    val insert: String,
    /** Value of the field for the first preview track; empty when that track has none. */
    val value: String,
)

// What:     `class TemplateEditorStateError(message: String)` signals a broken editor invariant.
// Why:      A stored template that stopped applying must fail loudly rather than show a wrong preview.
//
// In TS you'd write (pseudocode):
// ```ts
// class TemplateEditorStateError extends Error {}
// ```
/** Thrown when a stored editor value no longer holds the invariant the editor depends on. */
class TemplateEditorStateError(message: String) : IllegalStateException(message)
//endregion

//region The editor state
// What:     `data class TemplateEditorState(...)` holds both templates, the open one, its caret, and the preview.
// Why:      One immutable value lets the screen redraw from it and lets tests check each transition.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateEditorState = Readonly<{ kind: TemplateKind; drafts: Record<TemplateKind, string>;
//   lastValid: Record<TemplateKind, string>; selectionStart: number; selectionEnd: number;
//   fieldFocused: boolean; previewTracks: PlayingTrackFields[]; usesSampleValues: boolean }>;
// ```
/** The editor's whole state: both templates, the open one, its caret, focus, and the tracks it previews. */
data class TemplateEditorState(
    /** The template the editor is open on. */
    val kind: TemplateKind,
    /** Current text of each template as typed, including text that does not apply. */
    val drafts: Map<TemplateKind, String>,
    /** Last text of each template that applied; the rows show these. */
    val lastValid: Map<TemplateKind, String>,
    /** Selection start in the open template, in characters. */
    val selectionStart: Int,
    /** Selection end in the open template, in characters. */
    val selectionEnd: Int,
    /** Whether the template field has focus, which decides whether typing help is shown. */
    val fieldFocused: Boolean,
    /** The tracks the preview and validation read: library tracks, or stand-ins while the library is empty. */
    val previewTracks: List<PlayingTrackFields>,
    /** Whether the preview shows stand-in values because the open library holds no track. */
    val usesSampleValues: Boolean,
    /** Note drawn under the preview while it shows stand-in values, which says why they are shown. */
    val sampleNote: String = TEMPLATE_SAMPLE_NOTE,
) {
    // What:     `val text: String` is the open template's current text.
    // Why:      The field, the validation and the help all read the same text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get text(): string { return this.drafts[this.kind]; }
    // ```
    /** The open template's current text, as typed. */
    val text: String
        get() = drafts.getValue(kind)

    // What:     `val errors: List<TemplateError>` lists every mistake of the open template's current text.
    // Why:      A template is valid only if it applies to every preview track, so each track is checked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get errors(): TemplateError[] { return unique(previewTracks.flatMap(t => refusals(evaluate(text, t)))); }
    // ```
    /** Every distinct mistake of the open template's current text, over all preview tracks. */
    val errors: List<TemplateError>
        get() = previewTracks
            .flatMap { track -> templateMistakes(text, templateTrackFor(kind, track), kind.fields) }
            .distinct()

    // What:     `val isValid: Boolean` tells whether the open template's current text applies.
    // Why:      Validity decides whether the text becomes the applied template and whether errors show.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get isValid(): boolean { return this.errors.length === 0; }
    // ```
    /** Whether the open template's current text applies to every preview track. */
    val isValid: Boolean
        get() = errors.isEmpty()

    // What:     `val mistakeLines: List<String>` formats the current mistakes as one line each.
    // Why:      The editor shows the mistakes in template order, named by the function at fault.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get mistakeLines(): string[] { return errorLines(this.errors); }
    // ```
    /** One line per mistake of the open template's current text, in template order. */
    val mistakeLines: List<String>
        get() = errorSentences(errors, ParsedTemplateCache.parsedFor(text))

    // What:     `val previewRows: List<TemplatePreviewRow>` gives each preview track its applied line.
    // Why:      The rows keep the last template that applied, so a mistake never changes what the rows show.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get previewRows() { return this.previewTracks.map(t => ({ title: t.track.title, supporting: applied(t) })); }
    // ```
    /** Rows the preview draws, each with the last applied template's line for its track. */
    val previewRows: List<TemplatePreviewRow>
        get() = previewTracks.map { track ->
            TemplatePreviewRow(title = track.track.title, supporting = appliedLine(kind, track))
        }

    // What:     `val previewNotes: List<String>` gives the notes drawn under the preview.
    // Why:      The note says where the rows come from and, while a mistake stands, that they keep the last template.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get previewNotes(): string[] { return [usesSample ? SAMPLE_NOTE : LIBRARY_NOTE, ...(valid ? [] : [KEPT_NOTE])]; }
    // ```
    /** Notes under the preview: its source first, then the kept-template note while a mistake stands. */
    val previewNotes: List<String>
        get() = buildList {
            add(if (usesSampleValues) sampleNote else TEMPLATE_LIBRARY_NOTE)
            if (!isValid) {
                add(TEMPLATE_KEPT_NOTE)
            }
        }

    // What:     `val fields: List<TemplateEditorFieldEntry>` lists the open template's insertable fields.
    // Why:      The field list shows each field's value for the first preview track, so the user sees what it holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get fields() { return kind.fields.map(f => ({ label, mode, insert: `mi(${mode})`, value })); }
    // ```
    /** The insertable fields of the open template, each with its value for the first preview track. */
    val fields: List<TemplateEditorFieldEntry>
        get() {
            /** The first preview track, whose field values the list shows. */
            val firstTrack: PlayingTrackFields = previewTracks.first()
            /** The first track as the open template reads it. */
            val source: TemplateTrack = templateTrackFor(kind, firstTrack)
            return kind.fields.map { field ->
                TemplateEditorFieldEntry(
                    label = field.label,
                    mode = field.mode,
                    insert = "mi(" + field.mode + ")",
                    value = fieldValueText(field.mode, source),
                )
            }
        }

    // What:     `val help: TemplateHelp?` is the typing help for the call around the caret.
    // Why:      Help is shown only while the field has focus, because only then is the caret typing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get help() { return this.fieldFocused ? helpAt(this.text, this.selectionEnd, fields) : undefined; }
    // ```
    /** Help for the call around the caret, or null when the field has no focus or the caret is outside a call. */
    val help: TemplateHelp?
        get() = if (fieldFocused) helpAt(text, selectionEnd, kind.fields) else null

    // What:     `val canReset: Boolean` tells whether the open template differs from its default.
    // Why:      The way back to the default is offered only when there is something to go back from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get canReset(): boolean { return this.text !== this.kind.defaultTemplate; }
    // ```
    /** Whether the way back to the default is offered, which is when the text differs from the default. */
    val canReset: Boolean
        get() = text != kind.defaultTemplate

    // What:     `val settingsLines: Map<TemplateKind, String>` gives each template's line for Settings.
    // Why:      The Settings page lists each template with the line it currently produces.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get settingsLines() { return Object.fromEntries(kinds.map(k => [k, settingsLine(k)])); }
    // ```
    /** The line each template currently yields for the first preview track, keyed by template. */
    val settingsLines: Map<TemplateKind, String>
        get() = TemplateKind.entries.associateWith { template -> settingsLine(template) }

    // What:     `fun settingsLine(template: TemplateKind): String` gives one template's current line.
    // Why:      Settings shows the applied template on the first preview track, so its effect is visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // settingsLine(template: TemplateKind): string { return applied(template, this.previewTracks[0]); }
    // ```
    /** The line one template currently yields for the first preview track. */
    fun settingsLine(template: TemplateKind): String = appliedLine(template, previewTracks.first())

    // What:     `fun withEdit(edit: TemplateEditorEdit): TemplateEditorState` applies one field edit.
    // Why:      The typed text always becomes the draft, and it becomes the applied template only when it applies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // withEdit(edit: Edit): State { /* draft = text; applied = valid ? text : applied; clamp caret */ }
    // ```
    /** Applies one edit from the field; the last template that applied stays when the new text does not. */
    fun withEdit(edit: TemplateEditorEdit): TemplateEditorState {
        /** Whether the edited text applies to every preview track. */
        val applies: Boolean = resultsFor(kind, edit.text).all { result -> result is TemplateResult.Shown }
        return copy(
            drafts = drafts + (kind to edit.text),
            lastValid = if (applies) lastValid + (kind to edit.text) else lastValid,
            selectionStart = clampOffset(edit.selectionStart, edit.text),
            selectionEnd = clampOffset(edit.selectionEnd, edit.text),
        )
    }

    // What:     `fun insertField(mode: String): TemplateEditorState` puts one field's call at the caret.
    // Why:      The field list inserts at the caret, so the user builds a template without typing a name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // insertField(mode: string): State { return this.insertCall(`mi(${mode})`, caretInCall); }
    // ```
    /** Puts the call that reads one field at the caret, replacing any selection. */
    fun insertField(mode: String): TemplateEditorState {
        /** The call text that reads the field. */
        val call: String = "mi(" + mode + ")"
        return insertCall(call = call, caretInCall = call.length)
    }

    // What:     `fun insertFunction(name: String): TemplateEditorState` puts an empty call at the caret.
    // Why:      The editor can insert any template function by name, with the caret inside its brackets.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // insertFunction(name: string): State { return this.insertCall(`${name}()`, name.length + 1); }
    // ```
    /** Puts an empty call of one function at the caret, with the caret placed inside its brackets. */
    fun insertFunction(name: String): TemplateEditorState =
        insertCall(call = name + "()", caretInCall = name.length + 1)

    // What:     `fun resetToDefault(): TemplateEditorState` restores the open template's default text.
    // Why:      The way back to the default must always work, and the default always applies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // resetToDefault(): State { return this.withEdit({ text: this.kind.defaultTemplate, ... }); }
    // ```
    /** Replaces the open template's text with its default and puts the caret at its end. */
    fun resetToDefault(): TemplateEditorState {
        /** The default text of the open template. */
        val defaultText: String = kind.defaultTemplate
        return withEdit(
            TemplateEditorEdit(
                text = defaultText,
                selectionStart = defaultText.length,
                selectionEnd = defaultText.length,
            ),
        )
    }

    // What:     `fun open(template: TemplateKind): TemplateEditorState` opens one template for editing.
    // Why:      Settings opens an editor per template, and each keeps its own text while the other is open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // open(template: TemplateKind): State { return { ...this, kind: template, fieldFocused: false }; }
    // ```
    /** Opens one template with the caret at the end of its text and no focus in the field. */
    fun open(template: TemplateKind): TemplateEditorState {
        /** Length of the template's current text, where the caret rests. */
        val length: Int = drafts.getValue(template).length
        return copy(kind = template, selectionStart = length, selectionEnd = length, fieldFocused = false)
    }

    // What:     `fun withFocus(focused: Boolean): TemplateEditorState` records whether the field has focus.
    // Why:      Typing help appears only with focus, so focus changes must reach the state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // withFocus(focused: boolean): State { return { ...this, fieldFocused: focused }; }
    // ```
    /** Records whether the template field has keyboard focus. */
    fun withFocus(focused: Boolean): TemplateEditorState = copy(fieldFocused = focused)

    // What:     `private fun insertCall(...)` replaces the selection with a call.
    // Why:      Outside a formula the call is wrapped in dollar signs, or it would show as literal text.
    // Why:      A call written outside a formula would show as literal text, so the editor adds the delimiters itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function insertCall(text, start, end, call, caretInCall) { /* wrap with $ outside a formula */ }
    // ```
    /** Replaces the selection with the call, wrapping it in dollar signs when the caret is outside a formula. */
    private fun insertCall(call: String, caretInCall: Int): TemplateEditorState {
        /** Lower end of the selection, where the inserted text starts. */
        val start: Int = clampOffset(minOf(selectionStart, selectionEnd), text)
        /** Upper end of the selection, which the inserted text replaces. */
        val end: Int = clampOffset(maxOf(selectionStart, selectionEnd), text)
        /** Whether the insertion point is inside a formula, where the call needs no delimiters. */
        val inside: Boolean = insideFormula(text, start)
        /** The text that replaces the selection, with delimiters when the insertion point is outside a formula. */
        val inserted: String = if (inside) call else "\$" + call + "\$"
        /** Characters before the call inside the inserted text: one opening dollar sign when wrapped. */
        val prefix: Int = if (inside) 0 else 1
        /** The whole template text after the insertion. */
        val next: String = text.substring(0, start) + inserted + text.substring(end)
        /** The caret offset just inside the call's own text. */
        val caret: Int = start + prefix + caretInCall
        return withEdit(TemplateEditorEdit(text = next, selectionStart = caret, selectionEnd = caret))
    }

    // What:     `private fun resultsFor(...)` evaluates one template text against every preview track.
    // Why:      Validation and the preview both need the engine's result for each track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function resultsFor(template, text) { return previewTracks.map(t => evaluate(text, t, template.fields)); }
    // ```
    /** The engine's result for one template text on each preview track. */
    private fun resultsFor(template: TemplateKind, text: String): List<TemplateResult> =
        previewTracks.map { track ->
            evaluateTemplate(text, templateTrackFor(template, track), template.fields)
        }

    // What:     `private fun appliedLine(...)` gives the last applied template's line for one track.
    // Why:      The applied template always applies, so a refusal here means the stored state is broken.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function appliedLine(template, track): string { /* the shown text, or throw */ }
    // ```
    /** The last applied template's line for one track; throws when a stored template no longer applies. */
    private fun appliedLine(template: TemplateKind, track: PlayingTrackFields): String {
        /** The engine's result for the last applied text on this track. */
        val result: TemplateResult = evaluateTemplate(
            lastValid.getValue(template),
            templateTrackFor(template, track),
            template.fields,
        )
        return when (result) {
            is TemplateResult.Shown -> result.text
            is TemplateResult.Refused -> throw TemplateEditorStateError(
                "The applied " + template.label + " template no longer applies: " +
                    errorLines(result.errors).joinToString("; "),
            )
        }
    }

    // What:     `companion object` holds the constructor that starts an editor.
    // Why:      Starting the editor needs the library's tracks, which only the caller knows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // namespace TemplateEditorState { function initial(libraryTracks): State { ... } }
    // ```
    /** Holds the function that starts an editor from the library's tracks. */
    companion object {
        // What:     `fun initial(libraryTracks: List<PlayingTrackFields>): TemplateEditorState` starts the editor.
        // Why:      The editor opens on the track rows template at its default, previewing the library or stand-ins.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // function initial(libraryTracks: PlayingTrackFields[]): State { ... }
        // ```
        /** Starts the editor with both templates at their defaults and the preview drawn from the library. */
        fun initial(libraryTracks: List<PlayingTrackFields>): TemplateEditorState {
            /** Whether the open library holds no track, so the preview uses stand-in values. */
            val usesSample: Boolean = libraryTracks.isEmpty()
            /** Tracks the preview and validation read: the first library tracks, or the stand-ins. */
            val tracks: List<PlayingTrackFields> = if (usesSample) {
                TEMPLATE_SAMPLE_TRACKS
            } else {
                libraryTracks.take(TEMPLATE_PREVIEW_ROW_COUNT)
            }
            /** Each template's default text, which is both its current text and its last applied text. */
            val defaults: Map<TemplateKind, String> = TemplateKind.entries.associateWith { template ->
                template.defaultTemplate
            }
            /** Caret offset at the end of the track rows default, where the editor starts. */
            val end: Int = TemplateKind.TRACK_ROWS.defaultTemplate.length
            return TemplateEditorState(
                kind = TemplateKind.TRACK_ROWS,
                drafts = defaults,
                lastValid = defaults,
                selectionStart = end,
                selectionEnd = end,
                fieldFocused = false,
                previewTracks = tracks,
                usesSampleValues = usesSample,
            )
        }
    }
}
//endregion

//region Private helpers
// What:     `private fun templateTrackFor(...)` picks the track value a template reads.
// Why:      Track rows read the track alone, and the playing track also reads its place in the folder.
//
// In TS you'd write (pseudocode):
// ```ts
// function templateTrackFor(kind, track) { return kind === "playing" ? track : track.track; }
// ```
/** The source of field text that one template reads for a preview track. */
private fun templateTrackFor(template: TemplateKind, track: PlayingTrackFields): TemplateTrack = when (template) {
    TemplateKind.TRACK_ROWS -> track.track
    TemplateKind.PLAYING_TRACK -> track
}

// What:     `private fun clampOffset(...)` keeps a caret offset inside the text.
// Why:      The field can report offsets past the end after an edit, and the state must never hold them.
//
// In TS you'd write (pseudocode):
// ```ts
// const clampOffset = (offset: number, text: string) => Math.min(Math.max(offset, 0), text.length);
// ```
/** Limits a caret offset to the range from the start of the text to its end. */
private fun clampOffset(offset: Int, text: String): Int = offset.coerceIn(0, text.length)

// What:     `private fun insideFormula(...)` tells whether a caret offset lies inside a formula.
// Why:      Field calls go bare inside a formula and wrapped in dollar signs outside one.
//
// In TS you'd write (pseudocode):
// ```ts
// const insideFormula = (text, caret) => parse(text).parts.some(p => isFormula(p) && inSpan(caret, p));
// ```
/** Whether the caret offset sits inside one of the text's formulas. */
private fun insideFormula(text: String, caret: Int): Boolean = parseTemplate(text).parts.any { part ->
    part is FormulaPart && caret > part.start && caret <= (if (part.closed) part.end - 1 else part.end)
}
//endregion

// What:     `private fun fieldValueText(mode: String, source: TemplateTrack): String` gives the sample value of a
//           field.
// Why:      The Duration field holds a number of seconds in templates, but the list of fields shows it as minutes and
//           seconds, the way the rows show it.
//
// In TS you'd write (pseudocode):
// ```ts
// const fieldValueText = (mode: string, source: TemplateTrack) => mode === "len" ? duration(source.len) :
// source.field(mode) ?? "";
// ```
/** Sample value of one field: the duration as m:ss, everything else as the track states it. */
private fun fieldValueText(mode: String, source: TemplateTrack): String {
    /** The field as the track states it, or null when the track has no value. */
    val raw: String? = source.fieldText(mode)
    if (mode == "len") {
        return raw?.toDoubleOrNull()?.let { seconds -> durationText(seconds) }.orEmpty()
    }
    return raw.orEmpty()
}
