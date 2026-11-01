// What:     `package dev.monochromatic.musicplayer` places this test beside the template editor it checks.
// Why:      The test reaches the module-internal pure helpers of the editor without exposing them publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the state and the help record the helpers read.
// Why:      The helpers convert the state's values for the field and the help's signature for bold text.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TemplateEditorState, TemplateEditorEdit, TemplateHelp } from "core/TemplateEditorState";
// ```
import dev.monochromatic.musicplayer.core.TemplateEditorEdit
import dev.monochromatic.musicplayer.core.TemplateEditorState
import dev.monochromatic.musicplayer.core.TemplateHelp

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each helper outcome is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

// What:     Imports from `androidx.compose.ui.text` bring in the field value and its selection type.
// Why:      The field's value is the text together with its selection, and the tests check both.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TextFieldValue, TextRange } from "compose/ui/text";
// ```
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue

// What:     `class TemplateEditorModelTest` groups the pure-helper tests for the template editor.
// Why:      The field value, the empty-value wording and the signature span run on the host JVM without Compose.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("TemplateEditor helpers", () => { ... });
// ```
/** Verifies the pure helpers behind the template editor without a Compose runtime. */
class TemplateEditorModelTest {
    // What:     `fun emptyValueIsNamed` checks the wording for a field with no value.
    // Why:      A blank value must say that nothing is there yet rather than draw an empty line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty value", () => expect(fieldValueText("")).toBe("No value yet"));
    // ```
    /** An empty field value is shown as the words no value yet. */
    @Test
    fun emptyValueIsNamed() {
        assertEquals("No value yet", templateEditorFieldValueText(""))
    }

    // What:     `fun presentValueIsKeptAsIs` checks that a value with text passes through unchanged.
    // Why:      Only a missing value is renamed, so real values reach the list exactly as the engine wrote them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("present value", () => expect(fieldValueText("275")).toBe("275"));
    // ```
    /** A field value with text is shown exactly as given. */
    @Test
    fun presentValueIsKeptAsIs() {
        assertEquals("275", templateEditorFieldValueText("275"))
    }

    // What:     `fun textValueCarriesTextAndSelection` checks the value the field draws from the state.
    // Why:      The field must show the template text with the selection the state holds,
    //           so the caret stays where the user left it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("text value", () => expect(textValue(state).selection).toEqual({ start: 2, end: 5 }));
    // ```
    /** The field value carries the state's text and its selection range. */
    @Test
    fun textValueCarriesTextAndSelection() {
        /** The state after an edit that selects characters two to five of the template. */
        val state: TemplateEditorState = TemplateEditorState.initial(emptyList())
            .withEdit(TemplateEditorEdit(text = "\$tf(mi(len))\$", selectionStart = 2, selectionEnd = 5))
        /** The field value the composable draws for that state. */
        val value: TextFieldValue = templateEditorTextValue(state)
        assertEquals("\$tf(mi(len))\$", value.text)
        assertEquals(TextRange(2, 5), value.selection)
    }

    // What:     `fun initialTextValueCaretIsAtEnd` checks the field value of a freshly opened editor.
    // Why:      A new editor shows its default with the caret at the end, as the study's default scene does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("initial caret", () => expect(textValue(initial).selection.start).toBe(defaultLength));
    // ```
    /** A freshly opened editor draws its default with a collapsed caret at the end of the text. */
    @Test
    fun initialTextValueCaretIsAtEnd() {
        /** The field value of an editor opened with no library tracks. */
        val value: TextFieldValue = templateEditorTextValue(TemplateEditorState.initial(emptyList()))
        assertEquals(value.text.length, value.selection.start)
        assertEquals(value.selection.start, value.selection.end)
    }

    // What:     `fun editRoundTripsTextAndSelection` checks that a reported field value becomes the same edit.
    // Why:      The host applies the edit exactly as the field reported it, so no text or offset may change on the way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("edit round trip", () => expect(editOf({ text, selection: [1, 2] })).toEqual({ text, 1, 2 }));
    // ```
    /** A reported field value becomes an edit with the same text and selection offsets. */
    @Test
    fun editRoundTripsTextAndSelection() {
        /** The value the field reports after the user selects one character. */
        val reported: TextFieldValue = TextFieldValue(text = "abc", selection = TextRange(1, 2))
        /** The edit the helper builds from the reported value. */
        val edit: TemplateEditorEdit = templateEditorEditOf(reported)
        assertEquals(TemplateEditorEdit(text = "abc", selectionStart = 1, selectionEnd = 2), edit)
    }

    // What:     `fun parameterSpanFindsTheArgumentName` checks where the current argument sits in its signature.
    // Why:      The argument's name is set bold inside the signature, so its character range must be exact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("span", () => expect(parameterSpan(help)).toEqual([13, 19]));
    // ```
    /** The argument name's range inside the signature starts at its first character and runs its length. */
    @Test
    fun parameterSpanFindsTheArgumentName() {
        /** The help for the format argument of the seconds formatter. */
        val help = TemplateHelp(
            name = "tf",
            signature = "tf(seconds, [format])",
            parameter = "format",
            description = "Format: h, m and s.",
        )
        /** The character range of the argument name inside the signature. */
        val span: IntRange? = templateEditorParameterSpan(help)
        assertEquals(13, span?.first)
        assertEquals(18, span?.last)
    }

    // What:     `fun parameterSpanIsNullWhenSignatureOmitsIt` checks a signature that does not name the argument.
    // Why:      A mismatch must draw the signature plainly rather than fail, as the study does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("span absent", () => expect(parameterSpan(help)).toBeUndefined());
    // ```
    /** A signature that does not name the argument yields no bold range. */
    @Test
    fun parameterSpanIsNullWhenSignatureOmitsIt() {
        /** A help whose argument name is absent from its signature. */
        val help = TemplateHelp(
            name = "tf",
            signature = "tf(seconds)",
            parameter = "format",
            description = "Format.",
        )
        assertNull(templateEditorParameterSpan(help))
    }
}
