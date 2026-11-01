// Mirrors every case of package/music-player/design/template-reference-test.mjs, grouped by its regions.
// Each reference case keeps its input and expected output; the group names follow the reference's regions.

// What:     This package matches TrackTemplate's package.
// Why:      The tests call the template engine's public API directly, as a consumer would.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's equality and truth assertions and the test annotation register each template check.
// Why:      Each reference case becomes one assertion with the same input and expected output.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Mirrors each case of the template reference's test file, one assertion per case. */
class TrackTemplateTest {
    // What:     `private val analysed = TrackFields(...)` is a track whose true peak is already analysed.
    // Why:      Most reference cases read a complete track, so one shared fixture keeps them comparable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const analysed = { title: "Another Xronixle", ..., len: 275, peak: "\u22120.3 dBTP" };
    // ```
    /** A track with every field set, including an analysed true peak. */
    private val analysed = TrackFields(
        title = "Another Xronixle",
        file = "かめりあ(Camellia) - Another Xronixle",
        ext = "flac",
        folder = "Camellia",
        path = "Camellia/かめりあ(Camellia) - Another Xronixle.flac",
        len = 275.0,
        peak = "\u22120.3 dBTP",
    )

    // What:     `private val waiting = analysed.copy(...)` is a track whose peak is not yet analysed.
    // Why:      The reference shows that an empty peak leaves its surrounding text in place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const waiting = { ...analysed, title: "Exit This Earth's Atomosphere", peak: undefined };
    // ```
    /** A track with a title of its own and no peak yet. */
    private val waiting = analysed.copy(title = "Exit This Earth's Atomosphere", peak = null)

    // What:     `private val bare = TrackFields(...)` is a track with nothing but a path.
    // Why:      Missing values must show as empty text rather than as placeholders.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bare = { title: "x", file: "x", ext: "mp3", folder: "", path: "x.mp3", len: undefined, peak: undefined };
    // ```
    /** A track with no duration and no peak. */
    private val bare = TrackFields(
        title = "x",
        file = "x",
        ext = "mp3",
        folder = "",
        path = "x.mp3",
        len = null,
        peak = null,
    )

    // What:     `private val playing = PlayingTrackFields(...)` is the analysed track at place 1 of 16.
    // Why:      The playing template reads the place and total as well as the track's own fields.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const playing = { ...analysed, track: 1, total: 16 };
    // ```
    /** The analysed track, first of sixteen in its folder. */
    private val playing = PlayingTrackFields(track = analysed, placeInFolder = 1, tracksInFolder = 16)

    // What:     `private fun assertShows(...)` asserts that a template shows the expected text.
    // Why:      One helper keeps each shown case a single readable line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function shows({ text, track, expected }: ShowCase): void { ... }
    // ```
    /** Asserts that the template applies and shows exactly the expected text. */
    private fun assertShows(
        text: String,
        track: TemplateTrack,
        expected: String,
        fields: List<TemplateField> = trackTemplateFields,
    ) {
        assertEquals(text, TemplateResult.Shown(expected), evaluateTemplate(text, track, fields))
    }

    // What:     `private fun refusedErrors(...)` returns the mistakes of a template that must be refused.
    // Why:      A template that unexpectedly applies fails here with a clear message before its mistakes are compared.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function refusedErrors(text: string, track: TemplateTrack): TemplateError[] { ... }
    // ```
    /** Returns the mistakes of a refused template, failing when the template applies instead. */
    private fun refusedErrors(
        text: String,
        track: TemplateTrack = analysed,
        fields: List<TemplateField> = trackTemplateFields,
    ): List<TemplateError> {
        val result = evaluateTemplate(text, track, fields)
        if (result !is TemplateResult.Refused) {
            throw AssertionError("Expected a refusal for: " + text)
        }
        return result.errors
    }

    // What:     `private fun assertRefuses(...)` asserts the template is refused with exactly these mistake lines.
    // Why:      The lines name each mistake and its order, so both the wording and the position are checked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function refuses({ text, track, lines }: RefuseCase): void { ... }
    // ```
    /** Asserts that the template is refused and that its mistake lines match exactly. */
    private fun assertRefuses(
        text: String,
        lines: List<String>,
        track: TemplateTrack = analysed,
    ) {
        assertEquals(text, lines, errorLines(refusedErrors(text, track)))
    }

    // What:     `private fun helpLabel(text: String): String?` runs help with the caret at the `|` marker.
    // Why:      Writing the caret as a marker keeps each help case readable beside its expected label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function help({ text, marker = "|" }: HelpCase): string | undefined { ... }
    // ```
    /** The signature and argument of the help at the marker, or null when no help applies there. */
    private fun helpLabel(text: String): String? {
        val caret = text.indexOf('|')
        assertTrue(text, caret >= 0)
        val found = helpAt(text.replaceFirst("|", ""), caret)
        return found?.let { help -> help.signature + " / " + help.parameter }
    }

    // What:     `fun textFieldsAndDefaultLineShowSubstitutions()` checks plain text, the default line, and each field.
    // Why:      Text outside formulas is always shown, and a missing value leaves its surrounding text in place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("text, fields and the default line", () => { ... });
    // ```
    /** Covers the text and fields group of the reference. */
    @Test
    fun textFieldsAndDefaultLineShowSubstitutions() {
        assertShows(text = "", track = analysed, expected = "")
        assertShows(text = "plain text, no formula", track = analysed, expected = "plain text, no formula")
        assertShows(text = DEFAULT_TRACK_TEMPLATE, track = analysed, expected = "4:35 \u22120.3 dBTP")
        assertShows(text = DEFAULT_TRACK_TEMPLATE, track = waiting, expected = "4:35 ")
        assertShows(text = DEFAULT_TRACK_TEMPLATE, track = bare, expected = " ")
        assertEquals("\$tf(mi(len), m:ss)\$ \$mi(peak)\$", DEFAULT_TRACK_TEMPLATE)
        val expectedValues = mapOf(
            "title" to "Another Xronixle",
            "file" to "かめりあ(Camellia) - Another Xronixle",
            "ext" to "flac",
            "folder" to "Camellia",
            "path" to "Camellia/かめりあ(Camellia) - Another Xronixle.flac",
            "len" to "275",
            "peak" to "\u22120.3 dBTP",
        )
        for (field in trackTemplateFields) {
            assertShows(
                text = "[\$mi(" + field.mode + ")\$]",
                track = analysed,
                expected = "[" + expectedValues.getValue(field.mode) + "]",
            )
        }
        assertShows(
            text = "\$mi(folder)\$/\$mi(file)\$.\$mi(ext)\$",
            track = analysed,
            expected = "Camellia/かめりあ(Camellia) - Another Xronixle.flac",
        )
        assertShows(text = "\$mi(peak)\$", track = waiting, expected = "")
        assertShows(text = "\$mi(title)\$\$mi(ext)\$", track = analysed, expected = "Another Xronixleflac")
    }

    // What:     `fun durationFormatsFollowTheFormatLetters()` checks each duration format and its refusals.
    // Why:      Letters, doubling, and apostrophes change the output, so each rule has its own case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("duration formats", () => { ... });
    // ```
    /** Covers the duration formats group of the reference. */
    @Test
    fun durationFormatsFollowTheFormatLetters() {
        assertShows(text = "\$tf(mi(len))\$", track = analysed, expected = "4:35")
        assertShows(text = "\$tf(mi(len), mm:ss)\$", track = analysed, expected = "04:35")
        assertShows(text = "\$tf(mi(len), h:mm:ss)\$", track = analysed, expected = "0:04:35")
        assertShows(text = "\$tf(4000, m:ss)\$", track = analysed, expected = "66:40")
        assertShows(text = "\$tf(4000, h:mm:ss)\$", track = analysed, expected = "1:06:40")
        assertShows(text = "\$tf(4000, hh:mm:ss)\$", track = analysed, expected = "01:06:40")
        assertShows(text = "\$tf(59.9, m:ss)\$", track = analysed, expected = "0:59")
        // TE5 (desktop parity): a letter outside apostrophes that is not h, m or s is a mistake, not text.
        assertRefuses(text = "\$tf(mi(len), \"m min\")\$", lines = listOf("tf: unknown format letter i"))
        assertRefuses(text = "\$tf(mi(len), \"MM:SS\")\$", lines = listOf("tf: unknown format letter M"))
        assertRefuses(text = "\$tf(mi(len), q:q)\$", lines = listOf("tf: unknown format letter q"))
        assertRefuses(text = "\$tf(mi(len), q:q)\$", lines = listOf("tf: unknown format letter q"), track = waiting)
        assertShows(text = "\$tf(mi(len), \"m'.'ss\")\$", track = analysed, expected = "4.35")
        assertShows(text = "\$tf(mi(len), \"m / ss\")\$", track = analysed, expected = "4 / 35")
        assertShows(text = "\$tf(mi(len), \"m' min'\")\$", track = analysed, expected = "4 min")
        assertShows(
            text = "\$tf(4000, \"h' hours and 'm' minutes'\")\$",
            track = analysed,
            expected = "1 hours and 6 minutes",
        )
        assertShows(text = "\$tf(mi(len), m:ss)\$", track = bare, expected = "")
        assertRefuses(
            text = "\$tf(mi(title))\$",
            lines = listOf("tf: needs a number of seconds, not Another Xronixle"),
        )
        assertRefuses(text = "\$tf()\$", lines = listOf("tf: takes 1 or 2 values, as in tf(seconds, [format])"))
        assertRefuses(text = "\$tf(1, m, s)\$", lines = listOf("tf: takes 1 or 2 values, as in tf(seconds, [format])"))
    }

    // What:     `fun textConversionCoversEachMode()` checks low, up, cap, and cut, and their refusals.
    // Why:      Each conversion mode has its own rule, and the cut length has its own refusal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("text conversion", () => { ... });
    // ```
    /** Covers the text conversion group of the reference. */
    @Test
    fun textConversionCoversEachMode() {
        assertShows(text = "\$tc(low, mi(title))\$", track = analysed, expected = "another xronixle")
        assertShows(text = "\$tc(up, mi(ext))\$", track = analysed, expected = "FLAC")
        assertShows(text = "\$tc(cap, \"hello there\")\$", track = analysed, expected = "Hello there")
        assertShows(text = "\$tc(cap, \"\")\$", track = analysed, expected = "")
        assertShows(text = "\$tc(cut, mi(title), 7)\$", track = analysed, expected = "Another")
        assertShows(text = "\$tc(cut, mi(file), 4)\$", track = analysed, expected = "かめりあ")
        assertShows(text = "\$tc(cut, mi(title), 0)\$", track = analysed, expected = "")
        assertShows(text = "\$tc(cut, mi(title), 99)\$", track = analysed, expected = "Another Xronixle")
        assertRefuses(
            text = "\$tc(cut, mi(title))\$",
            lines = listOf("tc: cut needs a length, as in tc(cut, text, 10)"),
        )
        assertRefuses(
            text = "\$tc(cut, mi(title), -1)\$",
            lines = listOf("tc: cut needs a length, as in tc(cut, text, 10)"),
        )
        assertRefuses(text = "\$tc(ell, mi(title), 5)\$", lines = listOf("tc: unknown mode ell"))
        assertRefuses(text = "\$tc(up)\$", lines = listOf("tc: takes 2 or 3 values, as in tc(mode, text, [length])"))
    }

    // What:     `fun joiningAndNoConditionalFollowTheReference()` checks `+`, separators next to empty fields,
    //           comparison signs as text, and the refused conditional.
    // Why:      The template has no conditional, so a separator never drops out with an empty field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("joining, and no conditional", () => { ... });
    // ```
    /** Covers the joining group of the reference. */
    @Test
    fun joiningAndNoConditionalFollowTheReference() {
        assertShows(text = "\$\"a\" + \"b\" + mi(ext)\$", track = analysed, expected = "abflac")
        assertShows(text = "\$1 + 2\$", track = analysed, expected = "12")
        assertShows(text = "\$mi(folder)\$ \u00B7 \$mi(title)\$", track = bare, expected = " \u00B7 x")
        assertShows(text = "\$mi(peak)\$ dBTP", track = waiting, expected = " dBTP")
        assertShows(text = "\$tc(up, a<b>=c!=d)\$", track = analysed, expected = "A<B>=C!=D")
        assertRefuses(text = "\$if(mi(peak), yes, no)\$", lines = listOf("if: unknown function"))
        assertRefuses(text = "\$if(1, fine, mi(nope))\$", lines = listOf("if: unknown function"))
        assertRefuses(text = "\$mi(peak) != \"\"\$", lines = listOf("formula: unexpected !="))
        assertRefuses(text = "\$mi(len) = 275\$", lines = listOf("formula: unexpected ="))
        assertRefuses(text = "\$mi(len) > 1\$", lines = listOf("formula: unexpected >"))
        assertFalse(templateFunctions.containsKey("if"))
    }

    // What:     `fun templateKindsHaveTheirOwnDefaultsAndFields()` checks the two templates' defaults, fields,
    //           and help for the playing track.
    // Why:      The track rows and the playing track must not share field lists or default text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("each template has its own default and fields", () => { ... });
    // ```
    /** Covers the template kinds group of the reference. */
    @Test
    fun templateKindsHaveTheirOwnDefaultsAndFields() {
        assertEquals(listOf("TRACK_ROWS", "PLAYING_TRACK"), TemplateKind.entries.map { kind -> kind.name })
        assertEquals(DEFAULT_TRACK_TEMPLATE, TemplateKind.TRACK_ROWS.defaultTemplate)
        assertEquals(trackTemplateFields, TemplateKind.TRACK_ROWS.fields)
        assertEquals(playingTemplateFields, TemplateKind.PLAYING_TRACK.fields)
        assertEquals(
            listOf("title", "file", "ext", "folder", "path", "len", "peak", "track", "total"),
            playingTemplateFields.map { field -> field.mode },
        )
        assertShows(
            text = TemplateKind.PLAYING_TRACK.defaultTemplate,
            track = playing,
            expected = "1 of 16 \u22120.3 dBTP",
            fields = playingTemplateFields,
        )
        assertShows(
            text = TemplateKind.PLAYING_TRACK.defaultTemplate,
            track = playing.copy(track = playing.track.copy(peak = null)),
            expected = "1 of 16 ",
            fields = playingTemplateFields,
        )
        assertShows(
            text = "\$mi(title)\$ (\$mi(track)\$/\$mi(total)\$)",
            track = playing,
            expected = "Another Xronixle (1/16)",
            fields = playingTemplateFields,
        )
        assertRefuses(
            text = "\$mi(track)\$ of \$mi(total)\$",
            lines = listOf("mi: unknown field track", "mi: unknown field total"),
        )
        assertEquals(
            "Field: one of title, file, ext, folder, path, len, peak, track, total.",
            helpAt("\$mi()\$", 4, playingTemplateFields)?.description,
        )
        assertEquals(
            "Field: one of title, file, ext, folder, path, len, peak.",
            helpAt("\$mi()\$", 4)?.description,
        )
    }

    // What:     `fun mistakesAreNamedOrderedAndNeverShownAsText()` checks each mistake's wording, order,
    //           and the rule that broken templates show nothing.
    // Why:      A mistake must name its function, sit in template order, and never leak into the shown line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mistakes are named, ordered, and never shown as text", () => { ... });
    // ```
    /** Covers the mistakes group of the reference. */
    @Test
    fun mistakesAreNamedOrderedAndNeverShownAsText() {
        assertRefuses(
            text = "\$xx(1)\$ and \$mi(nope)\$",
            lines = listOf("xx: unknown function", "mi: unknown field nope"),
        )
        assertRefuses(text = "\$mi(title)", lines = listOf("formula: the \$ at character 1 has no closing \$"))
        assertRefuses(
            text = "a \$mi(title) - text",
            lines = listOf("formula: the \$ at character 3 has no closing \$", "formula: unexpected -"),
        )
        assertRefuses(text = "\$mi(title\$", lines = listOf("mi: a closing bracket is missing"))
        assertRefuses(
            text = "\$mi(\"title)\$",
            lines = listOf("mi: a closing bracket is missing", "text: a quotation mark is not closed"),
        )
        assertRefuses(text = "\$\$", lines = listOf("formula: a value is missing"))
        assertRefuses(
            text = "\$+\$",
            lines = listOf("formula: a value is missing before +", "formula: a value is missing"),
        )
        assertRefuses(text = "\$mi(title) mi(ext)\$", lines = listOf("formula: unexpected mi"))
        assertRefuses(
            text = "\$mi(,)\$",
            lines = listOf("formula: a value is missing before ,", "formula: a value is missing before )"),
        )
        assertRefuses(text = "\$mi()\$", lines = listOf("mi: takes 1 value, as in mi(field)"))
        assertRefuses(
            text = "\$" + "tc(up, ".repeat(13) + "x" + ")".repeat(13) + "\$",
            lines = listOf("tc: calls are nested too deeply"),
        )
        assertRefuses(text = "\$constructor(1)\$", lines = listOf("constructor: unknown function"))
        assertRefuses(text = "\$mi(__proto__)\$", lines = listOf("mi: unknown field __proto__"))
        assertEquals(
            TemplateResult.Shown("X"),
            evaluateTemplate("\$" + "tc(up, ".repeat(12) + "x" + ")".repeat(12) + "\$", analysed),
        )
        assertShows(
            text = "<b>&amp; \"quoted\" \\n {x} `y` ; rm -rf ../",
            track = analysed,
            expected = "<b>&amp; \"quoted\" \\n {x} `y` ; rm -rf ../",
        )
        assertShows(text = "line one\nline two \$mi(ext)\$", track = analysed, expected = "line one\nline two flac")
        assertShows(text = "\$\"<script>\" + mi(ext)\$", track = analysed, expected = "<script>flac")
    }

    // What:     `fun helpFollowsTheCaretIntoTheInnermostCall()` checks which call and argument the caret reports.
    // Why:      The editor shows help for the innermost call around the caret, and nothing outside a call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("help follows the caret into the innermost call", () => { ... });
    // ```
    /** Covers the help group of the reference. */
    @Test
    fun helpFollowsTheCaretIntoTheInnermostCall() {
        assertEquals(null, helpLabel("plain| text"))
        assertEquals(null, helpLabel("\$|mi(title)\$"))
        assertEquals(null, helpLabel("\$mi|(title)\$"))
        assertEquals("mi(field) / field", helpLabel("\$mi(|title)\$"))
        assertEquals("mi(field) / field", helpLabel("\$mi(title|)\$"))
        assertEquals(null, helpLabel("\$mi(title)|\$"))
        assertEquals("tf(seconds, [format]) / seconds", helpLabel("\$tf(|"))
        assertEquals("mi(field) / field", helpLabel("\$tf(mi(|len), m:ss)\$"))
        assertEquals("tf(seconds, [format]) / seconds", helpLabel("\$tf(mi(len)|, m:ss)\$"))
        assertEquals("tf(seconds, [format]) / format", helpLabel("\$tf(mi(len), m:|ss)\$"))
        assertEquals("mi(field) / field", helpLabel("\$tf(mi(len), m:ss)\$ \$mi(peak|)\$"))
        assertEquals("tc(mode, text, [length]) / length", helpLabel("\$tc(cut, mi(title), |5)\$"))
        assertEquals("tc(mode, text, [length]) / length", helpLabel("\$tc(a, b, c, |d)\$"))
        assertEquals(null, helpLabel("\$if(|1)\$"))
        assertEquals(null, helpLabel("\$xx(|1)\$"))
        assertEquals("tc(mode, text, [length]) / text", helpLabel("\$mi(title)\$ and \$tc(up, |"))
        assertEquals(
            templateFunctions.getValue("tf").parameters[0].description,
            helpAt("\$tf(", 4)?.description,
        )
    }

    // What:     `fun parseKeepsWhereEachFormulaSits()` checks the start, end, and closed flag of each formula.
    // Why:      Help and error positions depend on these offsets, so they are checked directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("the parse keeps where each formula sits", () => { ... });
    // ```
    /** Covers the parse positions group of the reference. */
    @Test
    fun parseKeepsWhereEachFormulaSits() {
        val described = parseTemplate("a\$mi(ext)\$b\$mi(").parts.map { part ->
            when (part) {
                is LiteralPart -> "literal:" + part.value
                is FormulaPart -> "formula:" + part.start + "-" + part.end + (if (part.closed) "" else " open")
            }
        }
        assertEquals(listOf("literal:a", "formula:1-10", "literal:b", "formula:11-15 open"), described)
    }
}
