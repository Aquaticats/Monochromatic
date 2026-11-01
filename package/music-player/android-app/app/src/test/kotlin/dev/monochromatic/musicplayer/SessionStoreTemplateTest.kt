// What:     `package dev.monochromatic.musicplayer` places the template store tests beside `SessionStore`.
// Why:      The fallback rule is an internal function of `SessionStore`, so the tests reach it without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import dev.monochromatic.musicplayer.core.TemplateKind` names the template kinds the store keeps.
// Why:      Each test checks the fallback for each kind, so the kinds must be importable.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TemplateKind } from "./core/TrackTemplate";
// ```
import dev.monochromatic.musicplayer.core.TemplateKind

// What:     `import dev.monochromatic.musicplayer.core.parseTemplate` checks that a sample text really is malformed.
// Why:      A test for the parse fallback must first prove its text fails to parse, so the test is not vacuous.
//
// In TS you'd write (pseudocode):
// ```ts
// import { parseTemplate } from "./core/TrackTemplate";
// ```
import dev.monochromatic.musicplayer.core.parseTemplate

// What:     `import org.junit.Assert` brings in the assertions the tests use.
// Why:      Each fallback is checked by comparing the stored-text result to the expected text.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assert } from "@junit/assert";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private const val KEPT_TRACK_ROWS_TEXT: String` is a track-row text that applies to every track.
// Why:      A valid stored text must come back unchanged, so the test needs a text that the engine accepts.
//
// In TS you'd write (pseudocode):
// ```ts
// const KEPT_TRACK_ROWS_TEXT = "$mi(title)$ by $mi(folder)$";
// ```
/** A track-row text made of the title and the folder, which every validation track shows. */
private const val KEPT_TRACK_ROWS_TEXT: String = "\$mi(title)\$ by \$mi(folder)\$"

// What:     `private const val KEPT_PLAYING_TEXT: String` is a playing-track text that uses the track's place.
// Why:      The playing template may read the place and folder total, which the track rows may not.
//
// In TS you'd write (pseudocode):
// ```ts
// const KEPT_PLAYING_TEXT = "$mi(track)$ of $mi(total)$";
// ```
/** A playing-track text that reads the place in the folder and the folder total. */
private const val KEPT_PLAYING_TEXT: String = "\$mi(track)\$ of \$mi(total)\$"

// What:     `private const val MALFORMED_TEXT: String` is a text whose formula never closes.
// Why:      An unclosed formula is a parse mistake, so the store must fall back to the default for it.
//
// In TS you'd write (pseudocode):
// ```ts
// const MALFORMED_TEXT = "$mi(title";
// ```
/** A template text with an unclosed formula, which fails to parse. */
private const val MALFORMED_TEXT: String = "\$mi(title"

// What:     `class SessionStoreTemplateTest` groups the tests of the template fallback rule.
// Why:      The store must never hand a screen a template that would be refused, so the rule gets its own tests.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("SessionStore template fallback", () => { /* ... */ });
// ```
/** Tests for which stored template texts the store keeps and which fall back to the default. */
class SessionStoreTemplateTest {
    // What:     `fun fallsBackToDefaultWhenNothingIsStored()` checks the case with no saved text.
    // Why:      A fresh install has no saved template, so each kind must start from its default.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nothing stored", () => expect(templateOrDefault(null, kind)).toBe(kind.defaultTemplate));
    // ```
    /** A missing stored text gives each template kind its default. */
    @Test
    fun fallsBackToDefaultWhenNothingIsStored() {
        TemplateKind.entries.forEach { kind ->
            assertEquals(kind.defaultTemplate, SessionStore.templateOrDefault(stored = null, kind = kind))
        }
    }

    // What:     `fun defaultsApplyToEveryValidationTrack()` checks that each default text passes the store's rule.
    // Why:      A default that the store refused would make every fallback a refusal too, so the defaults must apply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("defaults apply", () => kinds.forEach((k) => expect(templateApplies(k.defaultTemplate, k)).toBe(true)));
    // ```
    /** The default text of every template kind applies to every validation track, so it is kept as stored. */
    @Test
    fun defaultsApplyToEveryValidationTrack() {
        TemplateKind.entries.forEach { kind ->
            assertTrue(SessionStore.templateApplies(text = kind.defaultTemplate, kind = kind))
        }
    }

    // What:     `fun keepsStoredTextThatApplies()` checks that a valid stored text is returned unchanged.
    // Why:      The user's own template must come back as typed, or the editor's saved work is lost.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("valid kept", () => expect(templateOrDefault(text, "trackRows")).toBe(text));
    // ```
    /** A valid stored text for the track rows is returned as stored. */
    @Test
    fun keepsStoredTextThatApplies() {
        assertEquals(
            KEPT_TRACK_ROWS_TEXT,
            SessionStore.templateOrDefault(stored = KEPT_TRACK_ROWS_TEXT, kind = TemplateKind.TRACK_ROWS),
        )
    }

    // What:     `fun fallsBackWhenStoredTextDoesNotParse()` checks a text with an unclosed formula.
    // Why:      A text that cannot be parsed shows nothing useful, so the default replaces it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unparsable falls back", () => expect(templateOrDefault("$mi(title", k)).toBe(k.defaultTemplate));
    // ```
    /** A stored text that does not parse falls back to the default for its kind. */
    @Test
    fun fallsBackWhenStoredTextDoesNotParse() {
        assertTrue(parseTemplate(MALFORMED_TEXT).errors.isNotEmpty())
        assertEquals(
            TemplateKind.TRACK_ROWS.defaultTemplate,
            SessionStore.templateOrDefault(stored = MALFORMED_TEXT, kind = TemplateKind.TRACK_ROWS),
        )
    }

    // What:     `fun fallsBackWhenFieldIsUnknown()` checks a field name that the template kind does not have.
    // Why:      A name outside the kind's fields is refused on every track, so it must not be kept.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown field falls back", () => expect(templateOrDefault("$mi(nope)$", k)).toBe(k.defaultTemplate));
    // ```
    /** A stored text that names a field the template kind does not have falls back to the default. */
    @Test
    fun fallsBackWhenFieldIsUnknown() {
        assertEquals(
            TemplateKind.TRACK_ROWS.defaultTemplate,
            SessionStore.templateOrDefault(stored = "\$mi(nope)\$", kind = TemplateKind.TRACK_ROWS),
        )
    }

    // What:     `fun trackRowsRejectTheTrackFieldButPlayingTrackKeepsIt()` checks the field sets of the two kinds.
    // Why:      The playing track has the place field and the track rows do not, so the same text is refused for one
    //           kind and kept for the other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track field per kind", () => { trackRows(p) is default; playing(p) is p });
    // ```
    /** The track field makes a text fall back for the track rows, while the playing track keeps the same text. */
    @Test
    fun trackRowsRejectTheTrackFieldButPlayingTrackKeepsIt() {
        assertEquals(
            TemplateKind.TRACK_ROWS.defaultTemplate,
            SessionStore.templateOrDefault(stored = KEPT_PLAYING_TEXT, kind = TemplateKind.TRACK_ROWS),
        )
        assertEquals(
            KEPT_PLAYING_TEXT,
            SessionStore.templateOrDefault(stored = KEPT_PLAYING_TEXT, kind = TemplateKind.PLAYING_TRACK),
        )
    }

    // What:     `fun playingTemplateFallsBackWhenItNamesNoField()` checks a playing text with an unknown name.
    // Why:      The playing template also falls back on an unknown name, so a bad saved text never reaches the deck.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing unknown falls back", () => expect(templateOrDefault("$mi(nope)$", playing)).toBe(playing.default));
    // ```
    /** A playing-track text with an unknown name falls back to the playing-track default. */
    @Test
    fun playingTemplateFallsBackWhenItNamesNoField() {
        assertEquals(
            TemplateKind.PLAYING_TRACK.defaultTemplate,
            SessionStore.templateOrDefault(stored = "\$mi(nope)\$", kind = TemplateKind.PLAYING_TRACK),
        )
    }
}
