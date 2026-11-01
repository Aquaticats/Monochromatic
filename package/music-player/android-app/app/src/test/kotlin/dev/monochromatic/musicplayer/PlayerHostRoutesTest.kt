// What:     `package dev.monochromatic.musicplayer` places the route tests beside the route rules they check.
// Why:      `PlayerRouteRules` and `PlayerRoute` live in this package, so the tests reach them without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import dev.monochromatic.musicplayer.core.TemplateKind` names the template kinds the editor route carries.
// Why:      The editor route is tested for each kind, so the kinds must be importable.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TemplateKind } from "./core/TrackTemplate";
// ```
import dev.monochromatic.musicplayer.core.TemplateKind

// What:     `import org.junit.Assert.assertEquals` imports the value-equality assertion.
// Why:      Each route rule is checked by comparing the expected route to the actual one.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertEquals } from "@junit/assert";
// ```
import org.junit.Assert.assertEquals

// What:     `import org.junit.Test` marks each function as a test.
// Why:      JUnit runs every annotated function in this class.
//
// In TS you'd write (pseudocode):
// ```ts
// // test("...", () => { ... });
// ```
import org.junit.Test

// What:     `class PlayerHostRoutesTest` groups the tests of the pure route rules.
// Why:      Back and the saved token are the route rules the host depends on, and each is checked on its own.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("PlayerRouteRules", () => { /* ... */ });
// ```
/** Tests for where Back leads and how each route is saved and restored. */
class PlayerHostRoutesTest {
    // What:     `fun backFromPlayerStaysOnPlayer()` checks that Back on the player keeps the player.
    // Why:      The player is the root destination, so Back there must not move the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("back from player stays on player", () => expect(backFrom("player")).toBe("player"));
    // ```
    /** Back on the player leaves the host on the player. */
    @Test
    fun backFromPlayerStaysOnPlayer() {
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFrom(PlayerRoute.Player))
    }

    // What:     `fun backFromSearchReturnsToPlayer()` checks Back from Search.
    // Why:      Search returns to the player (D64), so Back from it must land there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("back from search returns to player", () => expect(backFrom("search")).toBe("player"));
    // ```
    /** Back on the Search page returns to the player. */
    @Test
    fun backFromSearchReturnsToPlayer() {
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFrom(PlayerRoute.Search))
    }

    // What:     `fun backFromSettingsReturnsToPlayer()` checks Back from Settings.
    // Why:      Settings is reached from the player, so Back returns there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("back from settings returns to player", () => expect(backFrom("settings")).toBe("player"));
    // ```
    /** Back on the Settings page returns to the player. */
    @Test
    fun backFromSettingsReturnsToPlayer() {
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFrom(PlayerRoute.Settings))
    }

    // What:     `fun backFromTemplateEditorReturnsToSettings()` checks Back from an editor.
    // Why:      Each editor is reached from Settings, so Back returns to Settings and not to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("back from editor", () => expect(backFrom(editor)).toBe("settings"));
    // ```
    /** Back in a template editor returns to the Settings page for either template. */
    @Test
    fun backFromTemplateEditorReturnsToSettings() {
        TemplateKind.entries.forEach { kind ->
            assertEquals(PlayerRoute.Settings, PlayerRouteRules.backFrom(PlayerRoute.TemplateEditor(kind)))
        }
    }

    // What:     `fun backFromLicensesReturnsToSettings()` checks Back from the Licenses page.
    // Why:      The page is reached from Settings, so Back returns there and not to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("back from licenses returns to settings", () => expect(backFrom("licenses")).toBe("settings"));
    // ```
    /** Back on the Licenses page returns to Settings. */
    @Test
    fun backFromLicensesReturnsToSettings() {
        assertEquals(PlayerRoute.Settings, PlayerRouteRules.backFrom(PlayerRoute.Licenses))
    }

    // What:     `fun hostBackFromLicensesReturnsToSettingsOnEveryWidth()` checks the Back rule of the drawn screen.
    // Why:      The wide Settings page sends Back straight to the player, and Licenses must not take that shortcut.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("host back from licenses", () => [false, true].forEach((w) => expect(backFromHost("licenses", w)).toBe("settings")));
    // ```
    /** On the cover and on the unfolded screen, Back on the Licenses page returns to Settings. */
    @Test
    fun hostBackFromLicensesReturnsToSettingsOnEveryWidth() {
        assertEquals(PlayerRoute.Settings, PlayerRouteRules.backFromHost(PlayerRoute.Licenses, settingsIsWide = false))
        assertEquals(PlayerRoute.Settings, PlayerRouteRules.backFromHost(PlayerRoute.Licenses, settingsIsWide = true))
    }

    // What:     `fun hostBackKeepsTheOlderRulesOnEveryWidth()` checks the Back rule for the other routes.
    // Why:      Moving the wide-Settings shortcut into the rule must not change where Search, Settings and an editor go.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("host back", () => { /* wide: settings and editor go to player, search goes to player; narrow: backFrom */ });
    // ```
    /** The wide screen sends Settings and an editor to the player; Search and the narrow screen follow `backFrom`. */
    @Test
    fun hostBackKeepsTheOlderRulesOnEveryWidth() {
        /** An editor route, which the wide screen treats like Settings. */
        val editor = PlayerRoute.TemplateEditor(TemplateKind.TRACK_ROWS)
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFromHost(PlayerRoute.Settings, settingsIsWide = true))
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFromHost(editor, settingsIsWide = true))
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFromHost(PlayerRoute.Search, settingsIsWide = true))
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFromHost(PlayerRoute.Settings, settingsIsWide = false))
        assertEquals(PlayerRoute.Settings, PlayerRouteRules.backFromHost(editor, settingsIsWide = false))
        assertEquals(PlayerRoute.Player, PlayerRouteRules.backFromHost(PlayerRoute.Search, settingsIsWide = false))
    }

    // What:     `fun encodeDecodeRoundTripsEveryRoute()` checks that each route survives a save and restore.
    // Why:      The host restores its destination after a configuration change, so every route must decode to itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("routes round trip", () => routes.forEach((r) => expect(decode(encode(r))).toEqual(r)));
    // ```
    /** Every route, including each template editor, decodes from its own saved token to the same route. */
    @Test
    fun encodeDecodeRoundTripsEveryRoute() {
        /** Every route the host can show, with an editor for each template kind. */
        val routes: List<PlayerRoute> = listOf(PlayerRoute.Player, PlayerRoute.Search, PlayerRoute.Settings, PlayerRoute.Licenses) +
            TemplateKind.entries.map { kind -> PlayerRoute.TemplateEditor(kind) }
        routes.forEach { route ->
            assertEquals(route, PlayerRouteRules.decode(PlayerRouteRules.encode(route)))
        }
    }

    // What:     `fun decodeUnknownTokenRestoresPlayer()` checks a token that names no route.
    // Why:      A stale saved state restores the player, as an unknown stored playback mode falls back to its default.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown token restores player", () => expect(decode("bogus")).toBe("player"));
    // ```
    /** A token that names no route restores the player route. */
    @Test
    fun decodeUnknownTokenRestoresPlayer() {
        assertEquals(PlayerRoute.Player, PlayerRouteRules.decode("bogus"))
    }

    // What:     `fun decodeEditorTokenWithUnknownKindRestoresPlayer()` checks an editor token with a bad kind name.
    // Why:      A kind name that no template carries cannot open an editor, so the host falls back to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("editor token with unknown kind restores player", () => expect(decode("template:NOPE")).toBe("player"));
    // ```
    /** An editor token whose kind name matches no template restores the player route. */
    @Test
    fun decodeEditorTokenWithUnknownKindRestoresPlayer() {
        assertEquals(PlayerRoute.Player, PlayerRouteRules.decode("template:NOPE"))
    }
}
