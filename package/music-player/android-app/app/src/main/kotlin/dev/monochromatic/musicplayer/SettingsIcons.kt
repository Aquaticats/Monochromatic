// What:     `package dev.monochromatic.musicplayer` places the Settings row icons beside the other player composables.
// Why:      The Settings rows draw one leading icon each without an icon font or the extended icon set.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.ui.graphics` bring in the fill colour and the vector builder helpers.
// Why:      Each icon is one filled path, tinted by the caller.
//
// In TS you'd write (pseudocode):
// ```ts
// import { SolidColor, Color } from "compose/ui/graphics";
// ```
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes

// What:     Imports from `androidx.compose.ui.unit` bring in the density-independent size type and its builder.
// Why:      The icon size is a Dp so it scales with the display density like the other icons.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Dp, dp } from "compose/ui/unit";
// ```
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private val SETTINGS_ICON_SIZE: Dp = 24.dp` is the nominal width and height of each icon.
// Why:      Material Symbols are drawn at the 24 px optical size, and the icon sits in a 40 dp circle.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON_SIZE = 24;
// ```
/** Nominal width and height of each Settings icon. */
private val SETTINGS_ICON_SIZE: Dp = 24.dp

// What:     `private const val SETTINGS_ICON_VIEWPORT: Float = 960f` is the coordinate extent of each icon path.
// Why:      The upstream SVGs use `viewBox="0 -960 960 960"`, so their path data lives on a 960 unit grid.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON_VIEWPORT = 960;
// ```
/** Coordinate extent of each icon path on both axes, as the upstream `viewBox` gives it. */
private const val SETTINGS_ICON_VIEWPORT: Float = 960f

// What:     `private val QUEUE_MUSIC_PATH: String` holds the Material Symbols `queue_music` outline.
// Why:      The text is the `d` attribute of
//           `symbols/web/queue_music/materialsymbolsoutlined/queue_music_24px.svg` (weight 400, grade 0, optical
//           size 24, unfilled) from google/material-design-icons at commit 49d4db35df873165d6bd6ba09b063c7dafbac2f4,
//           copied unchanged; Apache 2.0, see `LICENSES/Apache-2.0.txt`.
//
// In TS you'd write (pseudocode):
// ```ts
// const QUEUE_MUSIC_PATH = "<svg path data>";
// ```
/** SVG path data for the `queue_music` icon on the upstream 960 unit grid, whose y axis starts at -960. */
private val QUEUE_MUSIC_PATH: String = listOf(
    "M640-160q-50 0-85-35t-35-85q0-50 35-85t85-35q11 0 21 1.5t19 6.5v-328h200v80H760v360q0 50-35 85t-85 35Z",
    "M120-320v-80h320v80H120Zm0-160v-80h480v80H120Zm0-160v-80h480v80H120Z",
).joinToString(separator = "")

// What:     `private val PLAY_CIRCLE_PATH: String` holds the Material Symbols `play_circle` outline.
// Why:      The text is the `d` attribute of
//           `symbols/web/play_circle/materialsymbolsoutlined/play_circle_24px.svg` (weight 400, grade 0, optical
//           size 24, unfilled) from google/material-design-icons at commit 49d4db35df873165d6bd6ba09b063c7dafbac2f4,
//           copied unchanged; Apache 2.0, see `LICENSES/Apache-2.0.txt`.
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAY_CIRCLE_PATH = "<svg path data>";
// ```
/** SVG path data for the `play_circle` icon on the upstream 960 unit grid, whose y axis starts at -960. */
private val PLAY_CIRCLE_PATH: String = listOf(
    "m380-300 280-180-280-180v360ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763",
    "q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5",
    "T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z",
).joinToString(separator = "")

// What:     `private fun settingsIcon(name: String, pathData: String): ImageVector` builds one filled icon.
// Why:      Both icons share one size, viewport and fill, and the group shifts the upstream y range of -960 to 0
//           down onto the 0 to 960 viewport, so the path data stays exactly as published.
//
// In TS you'd write (pseudocode):
// ```ts
// function settingsIcon(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled icon from upstream SVG path data whose `viewBox` starts at y = -960. */
private fun settingsIcon(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = SETTINGS_ICON_SIZE,
    defaultHeight = SETTINGS_ICON_SIZE,
    viewportWidth = SETTINGS_ICON_VIEWPORT,
    viewportHeight = SETTINGS_ICON_VIEWPORT,
).addGroup(
    translationY = SETTINGS_ICON_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).clearGroup().build()

// What:     `internal val SETTINGS_TRACK_ROWS_ICON: ImageVector` is the icon of the Track rows template row.
// Why:      D109 picked `queue_music` for the template that shapes the supporting line of a track row.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_TRACK_ROWS_ICON = settingsIcon("Settings.TrackRows", QUEUE_MUSIC_PATH);
// ```
/** Queue-with-note icon of the Track rows template row. */
internal val SETTINGS_TRACK_ROWS_ICON: ImageVector = settingsIcon(
    name = "Settings.TrackRows",
    pathData = QUEUE_MUSIC_PATH,
)

// What:     `internal val SETTINGS_PLAYING_TRACK_ICON: ImageVector` is the icon of the Playing track template row.
// Why:      D109 picked `play_circle` for the template that shapes the line under the playing track's title.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_PLAYING_TRACK_ICON = settingsIcon("Settings.PlayingTrack", PLAY_CIRCLE_PATH);
// ```
/** Circled play triangle icon of the Playing track template row. */
internal val SETTINGS_PLAYING_TRACK_ICON: ImageVector = settingsIcon(
    name = "Settings.PlayingTrack",
    pathData = PLAY_CIRCLE_PATH,
)

// What:     `private val AUTOPLAY_PATH: String` holds the Material Symbols `autoplay` outline.
// Why:      The text is the `d` attribute of
//           `symbols/web/autoplay/materialsymbolsoutlined/autoplay_24px.svg` (weight 400, grade 0, optical
//           size 24, unfilled) from google/material-design-icons at commit 49d4db35df873165d6bd6ba09b063c7dafbac2f4,
//           copied unchanged; Apache 2.0, see `LICENSES/Apache-2.0.txt`.
//
// In TS you'd write (pseudocode):
// ```ts
// const AUTOPLAY_PATH = "<svg path data>";
// ```
/** SVG path data for the `autoplay` icon on the upstream 960 unit grid, whose y axis starts at -960. */
private val AUTOPLAY_PATH: String = listOf(
    "M380-300v-360l280 180-280 180ZM480-40q-108 0-202.5-49.5T120-228v108H40v-240h240v80h-98q51 75 129.5 117.5T480-120",
    "q115 0 208.5-66T820-361l78 18q-45 136-160 219.5T480-40ZM42-520q7-67 32-128.5T143-762l57 57q-32 41-52 87.5T123-520",
    "H42Zm214-241-57-57q53-44 114-69.5T440-918v80q-51 5-97 25t-87 52Zm449 0q-41-32-87.5-52T520-838v-80q67 6 128.5 31",
    "T762-818l-57 57Zm133 241q-5-51-25-97.5T761-705l57-57q44 52 69 113.5T918-520h-80Z",
).joinToString(separator = "")

// What:     `private const val CHECK_PATH: String` holds the Material Symbols `check` outline.
// Why:      The text is the `d` attribute of `symbols/web/check/materialsymbolsoutlined/check_24px.svg` (weight 400,
//           grade 0, optical size 24, unfilled) from the same commit, copied unchanged; Apache 2.0.
//
// In TS you'd write (pseudocode):
// ```ts
// const CHECK_PATH = "<svg path data>";
// ```
/** SVG path data for the `check` icon on the upstream 960 unit grid, drawn in the thumb of an on switch. */
private const val CHECK_PATH: String = "M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z"

// What:     `private const val CLOSE_PATH: String` holds the Material Symbols `close` outline.
// Why:      The text is the `d` attribute of `symbols/web/close/materialsymbolsoutlined/close_24px.svg` (weight 400,
//           grade 0, optical size 24, unfilled) from the same commit, copied unchanged; Apache 2.0.
//
// In TS you'd write (pseudocode):
// ```ts
// const CLOSE_PATH = "<svg path data>";
// ```
/** SVG path data for the `close` icon on the upstream 960 unit grid, drawn in the thumb of an off switch. */
private const val CLOSE_PATH: String =
    "m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"

// What:     `internal val SETTINGS_AUTO_PLAY_ICON: ImageVector` is the icon of the Auto-play on select row.
// Why:      D129 picked `autoplay` (a play triangle inside a circling arrow), which no other screen uses.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_AUTO_PLAY_ICON = settingsIcon("Settings.AutoPlay", AUTOPLAY_PATH);
// ```
/** Play triangle inside a circular arrow, the icon of the Auto-play on select row. */
internal val SETTINGS_AUTO_PLAY_ICON: ImageVector = settingsIcon(
    name = "Settings.AutoPlay",
    pathData = AUTOPLAY_PATH,
)

// What:     `internal val SETTINGS_SWITCH_ON_ICON: ImageVector` is the mark in the thumb of an on switch.
// Why:      A check in the thumb is a second channel for the on state besides the thumb position and the track fill.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_ON_ICON = settingsIcon("Settings.SwitchOn", CHECK_PATH);
// ```
/** Check mark drawn in the thumb of an on switch. */
internal val SETTINGS_SWITCH_ON_ICON: ImageVector = settingsIcon(name = "Settings.SwitchOn", pathData = CHECK_PATH)

// What:     `internal val SETTINGS_SWITCH_OFF_ICON: ImageVector` is the mark in the thumb of an off switch.
// Why:      A cross in the thumb is a second channel for the off state besides the thumb position and the track fill.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_OFF_ICON = settingsIcon("Settings.SwitchOff", CLOSE_PATH);
// ```
/** Cross drawn in the thumb of an off switch. */
internal val SETTINGS_SWITCH_OFF_ICON: ImageVector = settingsIcon(name = "Settings.SwitchOff", pathData = CLOSE_PATH)

// What:     `private val LICENSE_PATH: String` holds the Material Symbols `license` outline.
// Why:      The text is the `d` attribute of
//           `symbols/web/license/materialsymbolsoutlined/license_24px.svg` (weight 400, grade 0, optical
//           size 24, unfilled) from google/material-design-icons at commit 49d4db35df873165d6bd6ba09b063c7dafbac2f4,
//           copied unchanged; Apache 2.0, see `LICENSES/Apache-2.0.txt`.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSE_PATH = "<svg path data>";
// ```
/** SVG path data for the `license` icon on the upstream 960 unit grid, whose y axis starts at -960. */
private val LICENSE_PATH: String = listOf(
    "M480-440q-50 0-85-35t-35-85q0-50 35-85t85-35q50 0 85 35t35 85q0 50-35 85t-85 35Z",
    "M240-40v-309q-38-42-59-96t-21-115q0-134 93-227t227-93q134 0 227 93t93 227q0 61-21 115t-59 96v309l-240-80-240 80Z",
    "m240-280q100 0 170-70t70-170q0-100-70-170t-170-70q-100 0-170 70t-70 170q0 100 70 170t170 70Z",
    "M320-159l160-41 160 41v-124q-35 20-75.5 31.5T480-240q-44 0-84.5-11.5T320-283v124Zm160-62Z",
).joinToString(separator = "")

// What:     `internal val SETTINGS_LICENSES_ICON: ImageVector` is the icon of the Licenses row.
// Why:      The desktop Licenses row draws the same `license` symbol, so both products show one icon for the row.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_LICENSES_ICON = settingsIcon("Settings.Licenses", LICENSE_PATH);
// ```
/** Ribbon badge on a certificate, the icon of the Licenses row. */
internal val SETTINGS_LICENSES_ICON: ImageVector = settingsIcon(
    name = "Settings.Licenses",
    pathData = LICENSE_PATH,
)
