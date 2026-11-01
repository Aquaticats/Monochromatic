// What:     `package dev.monochromatic.musicplayer` places this test beside the Compose key reading it checks.
// Why:      `hostKeyNameOf` is internal to the app package, so the test lives in the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in the Compose key constants and the JUnit assertions.
// Why:      Each Compose key must map to the exact key text that the D106 table lists.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import androidx.compose.ui.input.key.Key
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class HostKeyNamesTest` groups the cases of the Compose key to key text mapping.
// Why:      The table matches key text, so a wrong name here silently disables a chord on a real keyboard.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("hostKeyNameOf", () => { /* cases */ });
// ```
/** Verifies that every key of the D106 table is named the way the table spells it. */
class HostKeyNamesTest {
    /** The arrow keys map to the DOM-style arrow names. */
    @Test
    fun arrowKeysUseArrowNames() {
        assertEquals("ArrowLeft", hostKeyNameOf(Key.DirectionLeft))
        assertEquals("ArrowRight", hostKeyNameOf(Key.DirectionRight))
        assertEquals("ArrowUp", hostKeyNameOf(Key.DirectionUp))
        assertEquals("ArrowDown", hostKeyNameOf(Key.DirectionDown))
    }

    /** The keyboard's Home key is MoveHome, and the system Home button names nothing. */
    @Test
    fun homeIsTheKeyboardHomeKeyNotTheSystemButton() {
        assertEquals("Home", hostKeyNameOf(Key.MoveHome))
        assertEquals("End", hostKeyNameOf(Key.MoveEnd))
        assertEquals("", hostKeyNameOf(Key.SystemHome))
    }

    /** The page keys, Enter (both Enters), Delete and the Menu key map to their table names. */
    @Test
    fun navigationKeysUseTableNames() {
        assertEquals("PageUp", hostKeyNameOf(Key.PageUp))
        assertEquals("PageDown", hostKeyNameOf(Key.PageDown))
        assertEquals("Enter", hostKeyNameOf(Key.Enter))
        assertEquals("Enter", hostKeyNameOf(Key.NumPadEnter))
        assertEquals("Delete", hostKeyNameOf(Key.Delete))
        assertEquals("ContextMenu", hostKeyNameOf(Key.Menu))
    }

    /** Backspace is not Delete: the table's Delete is the forward-delete key. */
    @Test
    fun backspaceIsNotDelete() {
        assertEquals("", hostKeyNameOf(Key.Backspace))
    }

    /** Space and Escape map to their table names, and the function keys name nothing. */
    @Test
    fun spaceEscapeAndFunctionKeysUseTableNames() {
        assertEquals(" ", hostKeyNameOf(Key.Spacebar))
        assertEquals("Escape", hostKeyNameOf(Key.Escape))
        assertEquals("", hostKeyNameOf(Key.F5))
        assertEquals("", hostKeyNameOf(Key.F10))
    }

    /** Letters map to one lowercase letter, so Shift never changes the name. */
    @Test
    fun lettersMapToLowercaseLetters() {
        assertEquals("a", hostKeyNameOf(Key.A))
        assertEquals("g", hostKeyNameOf(Key.G))
        assertEquals("z", hostKeyNameOf(Key.Z))
    }

    /** Digits map to their digit, including the ones that Ctrl+Shift and Alt chords use. */
    @Test
    fun digitsMapToDigits() {
        assertEquals("0", hostKeyNameOf(Key.Zero))
        assertEquals("1", hostKeyNameOf(Key.One))
        assertEquals("4", hostKeyNameOf(Key.Four))
        assertEquals("9", hostKeyNameOf(Key.Nine))
    }

    /** Modifier keys and Tab name nothing, so they never reach the table. */
    @Test
    fun modifierKeysAndTabNameNothing() {
        assertEquals("", hostKeyNameOf(Key.CtrlLeft))
        assertEquals("", hostKeyNameOf(Key.AltLeft))
        assertEquals("", hostKeyNameOf(Key.ShiftLeft))
        assertEquals("", hostKeyNameOf(Key.Tab))
    }
}
