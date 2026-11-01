// Shortcut classifier cases mirrored one to one from the keyboard-map prototype test.
// Each assertion message names the prototype row it mirrors, so a failure points back to the JavaScript.

// What:     This package matches the classifier's package.
// Why:      The tests call the classifier and its types directly, with no package prefix.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import org.junit.Assert.assertEquals`, `assertThrows`, and `org.junit.Test` bring in
//           JUnit 4's value check, exception check, and test annotation.
// Why:      Every row needs an exact value check, and the platform parser needs an exception check.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `class KeyboardMapTest` groups the test methods for this file.
// Why:      JUnit runs every `@Test` method inside this class.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("KeyboardMap", () => { /* tests */ });
// ```
/** Mirrors every prototype classifier row and the platform parser rejection. */
class KeyboardMapTest {
    // What:     `private fun baseRequest(): ShortcutRequest` builds the prototype's base request.
    // Why:      Each row changes only the fields it names, so one shared starting request keeps rows honest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const base = { platform: "windows-linux", key: " ", ctrl: false, ... owner: "player", ... };
    // ```
    /** Returns the Windows and Linux Space key pressed by the player with no other state. */
    private fun baseRequest(): ShortcutRequest {
        return ShortcutRequest(
            platform = KeyboardPlatform.WINDOWS_LINUX,
            key = " ",
            ctrl = false,
            meta = false,
            alt = false,
            shift = false,
            repeat = false,
            owner = ShortcutOwner.PLAYER,
            inPopup = false,
            inSearch = false,
            composing = false,
        )
    }

    // What:     `private fun assertAction(row: String, expected: ShortcutAction, request: ShortcutRequest)`
    //           classifies one request and compares the result to the expected action.
    // Why:      One helper keeps each row to a single line, and the row text appears in any failure message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function assertAction(row: string, expected: ShortcutAction, request: ShortcutRequest): void {
    //   expect(shortcutAction(request), row).toBe(expected);
    // }
    // ```
    /** Checks one classified request against its expected action and names the row on failure. */
    private fun assertAction(row: String, expected: ShortcutAction, request: ShortcutRequest) {
        assertEquals(row, expected, shortcutAction(request))
    }

    // What:     `private fun assertRejectsPlatform(name: String)` calls the parser with a bad name.
    // Why:      The prototype throws for unknown platforms, and the enum version must throw too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function assertRejectsPlatform(name: string): void {
    //   expect(() => parsePlatform(name)).toThrow("Unknown keyboard-map platform.");
    // }
    // ```
    /** Checks that the parser rejects one name with the prototype's exact message. */
    private fun assertRejectsPlatform(name: String) {
        // What:     `assertThrows(IllegalArgumentException::class.java) { ... }` runs the block and
        //           returns the exception it throws, failing the test if nothing is thrown.
        // Why:      The message must be checked, so the error object is kept in `error`.
        //
        // Gotcha:   Unlike a plain `try` block, this check fails the test when no exception happens.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let error: unknown;
        // try { parsePlatform(name); } catch (caught) { error = caught; }
        // ```
        val error = assertThrows(IllegalArgumentException::class.java) {
            KeyboardPlatform.fromName(name)
        }
        assertEquals("message for platform name '$name'", "Unknown keyboard-map platform.", error.message)
    }

    // What:     `@Test` marks the next function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks the player-owner guard: a surface key stays unbound. */
    @Test
    fun guardKeepsSurfaceKeysUnbound() {
        assertAction(
            "guard 1: surface owner stays unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(owner = ShortcutOwner.SURFACE),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks the composition guard: a composing key stays with its owner. */
    @Test
    fun guardKeepsCompositionKeysWithTheirOwner() {
        assertAction(
            "guard 2: composing Ctrl+F stays with its owner",
            ShortcutAction.OWNER,
            baseRequest().copy(key = "f", ctrl = true, composing = true),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks the popup guard: a key pressed while a popup is open stays with the popup. */
    @Test
    fun guardKeepsPopupKeysWithTheirOwner() {
        assertAction(
            "guard 3: popup key stays with its owner",
            ShortcutAction.OWNER,
            baseRequest().copy(inPopup = true),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks common player and control keys: toggle, owner, repeat and unbound arrows. */
    @Test
    fun commonPlayerRowsClassifyAsExpected() {
        assertAction("row 1: Space toggles", ShortcutAction.TOGGLE, baseRequest())
        assertAction(
            "row 2: editor keeps its key",
            ShortcutAction.OWNER,
            baseRequest().copy(owner = ShortcutOwner.EDITOR),
        )
        assertAction(
            "row 3: control keeps its key",
            ShortcutAction.OWNER,
            baseRequest().copy(owner = ShortcutOwner.CONTROL),
        )
        assertAction(
            "row 4: repeated player key is ignored",
            ShortcutAction.IGNORE_REPEAT,
            baseRequest().copy(repeat = true),
        )
        assertAction(
            "row 11: bare ArrowDown is unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(key = "ArrowDown"),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks the Windows and Linux shortcuts for Search, picker, Settings, reveal, transport and mode. */
    @Test
    fun windowsLinuxRowsClassifyAsExpected() {
        assertAction("row 5: Ctrl+F opens Search", ShortcutAction.SEARCH, baseRequest().copy(key = "f", ctrl = true))
        assertAction("row 6: Ctrl+O opens picker", ShortcutAction.PICKER, baseRequest().copy(key = "o", ctrl = true))
        assertAction(
            "row 7: Ctrl+Alt+S opens Settings",
            ShortcutAction.SETTINGS,
            baseRequest().copy(key = "s", ctrl = true, alt = true),
        )
        assertAction(
            "row 8: Ctrl+ArrowLeft goes to previous",
            ShortcutAction.PREVIOUS,
            baseRequest().copy(key = "ArrowLeft", ctrl = true),
        )
        assertAction(
            "row 9: Ctrl+ArrowRight goes to next",
            ShortcutAction.NEXT,
            baseRequest().copy(key = "ArrowRight", ctrl = true),
        )
        assertAction(
            "row 10: editor keeps Ctrl+ArrowLeft",
            ShortcutAction.OWNER,
            baseRequest().copy(key = "ArrowLeft", ctrl = true, owner = ShortcutOwner.EDITOR),
        )
        assertAction(
            "row 22: Ctrl+G reveals current track",
            ShortcutAction.REVEAL,
            baseRequest().copy(key = "g", ctrl = true),
        )
        assertAction(
            "row 23: Ctrl+M cycles mode",
            ShortcutAction.MODE,
            baseRequest().copy(key = "m", ctrl = true),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks the macOS shortcuts for Search, picker, Settings, reveal, transport, and unbound modifier mixes. */
    @Test
    fun macRowsClassifyAsExpected() {
        val mac = KeyboardPlatform.MAC
        assertAction(
            "row 12: mac Control+ArrowRight is unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(platform = mac, key = "ArrowRight", ctrl = true),
        )
        assertAction(
            "row 13: mac Command+F opens Search",
            ShortcutAction.SEARCH,
            baseRequest().copy(platform = mac, key = "f", meta = true),
        )
        assertAction(
            "row 14: mac Command+O opens picker",
            ShortcutAction.PICKER,
            baseRequest().copy(platform = mac, key = "o", meta = true),
        )
        assertAction(
            "row 15: mac Command+Comma opens Settings",
            ShortcutAction.SETTINGS,
            baseRequest().copy(platform = mac, key = ",", meta = true),
        )
        assertAction(
            "row 16: mac Command+L reveals current track",
            ShortcutAction.REVEAL,
            baseRequest().copy(platform = mac, key = "l", meta = true),
        )
        assertAction(
            "row 17: mac Command+Shift+[ goes to previous",
            ShortcutAction.PREVIOUS,
            baseRequest().copy(platform = mac, key = "[", meta = true, shift = true),
        )
        assertAction(
            "row 18: mac Command+Shift+] goes to next",
            ShortcutAction.NEXT,
            baseRequest().copy(platform = mac, key = "]", meta = true, shift = true),
        )
        assertAction(
            "row 24: mac Command+M is unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(platform = mac, key = "m", meta = true),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks popup, Search, and unbound rows, including mixed-modifier letter combinations. */
    @Test
    fun popupSearchAndUnboundRowsClassifyAsExpected() {
        assertAction(
            "row 19: Escape closes popup",
            ShortcutAction.CLOSE_POPUP,
            baseRequest().copy(key = "Escape", inPopup = true),
        )
        assertAction(
            "row 20: Ctrl+F in popup stays with the popup",
            ShortcutAction.OWNER,
            baseRequest().copy(key = "f", ctrl = true, inPopup = true),
        )
        assertAction(
            "row 21: Escape in Search from an editor returns back",
            ShortcutAction.BACK,
            baseRequest().copy(key = "Escape", owner = ShortcutOwner.EDITOR, inSearch = true),
        )
        assertAction(
            "Escape in the wide Settings page from an editor goes back",
            ShortcutAction.BACK,
            baseRequest().copy(key = "Escape", owner = ShortcutOwner.EDITOR, inWideSettings = true),
        )
        assertAction(
            "Escape in the single-column Settings page from an editor stays with the editor",
            ShortcutAction.OWNER,
            baseRequest().copy(key = "Escape", owner = ShortcutOwner.EDITOR, inWideSettings = false),
        )
        assertAction(
            "row 25: Ctrl+Command+F is unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(key = "f", ctrl = true, meta = true),
        )
        assertAction(
            "row 26: Ctrl+Shift+F is unbound",
            ShortcutAction.UNBOUND,
            baseRequest().copy(key = "f", ctrl = true, shift = true),
        )
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks that the platform names used by the prototype parse to the matching enum entries. */
    @Test
    fun parsesStoredPlatformNamesToTheirMaps() {
        assertEquals(
            "platform name windows-linux",
            KeyboardPlatform.WINDOWS_LINUX,
            KeyboardPlatform.fromName("windows-linux"),
        )
        assertEquals("platform name mac", KeyboardPlatform.MAC, KeyboardPlatform.fromName("mac"))
    }

    // What:     `@Test` marks this function as a JUnit test case.
    // Why:      JUnit runs only annotated functions.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("...", () => { /* body */ });
    // ```
    /** Checks that unknown, differently cased, and empty platform names are rejected. */
    @Test
    fun rejectsUnknownPlatformNamesWithThePrototypeMessage() {
        assertRejectsPlatform("unknown")
        assertRejectsPlatform("Mac")
        assertRejectsPlatform("")
    }
}
