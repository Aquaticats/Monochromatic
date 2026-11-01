// What:     `package dev.monochromatic.musicplayer` places the player glyphs beside the other player composables.
// Why:      The top bars and folder picker import the glyphs without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.ui.graphics` bring in the fill color and the vector builder helpers.
// Why:      Each glyph is one filled path drawn in the tint the caller applies.
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
// Why:      The glyph size is a Dp so it scales with the display density like the Material icon set.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Dp, dp } from "compose/ui/unit";
// ```
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private val ICON_SIZE: Dp = 24.dp` is the nominal width and height of each glyph.
// Why:      The path data is drawn on the 24 by 24 grid of the Material icon set it replaces.
//
// In TS you'd write (pseudocode):
// ```ts
// const ICON_SIZE = 24;
// ```
/** Nominal width and height of each glyph. */
private val ICON_SIZE: Dp = 24.dp

// What:     `private const val ICON_VIEWPORT: Float = 24f` is the coordinate extent of each glyph path.
// Why:      The path data is written on a 24 by 24 grid so it scales with `ICON_SIZE`.
//
// In TS you'd write (pseudocode):
// ```ts
// const ICON_VIEWPORT = 24;
// ```
/** Coordinate extent of each glyph path on both axes. */
private const val ICON_VIEWPORT: Float = 24f

// What:     `private const val SEARCH_PATH: String` holds the magnifying glass outline.
// Why:      The path was copied from Material icon bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_PATH = "<svg path data>";
// ```
/** SVG path data for the search glyph on a 24 by 24 grid. */
private const val SEARCH_PATH: String =
    "M15.5 14 h-0.79 l-0.28 -0.27 C15.41 12.59 16 11.11 16 9.5 C16 5.91 13.09 3 9.5 3 S3 5.91 3 9.5 " +
    "S5.91 16 9.5 16 c1.61 0 3.09 -0.59 4.23 -1.57 l0.27 0.28 v0.79 l5 4.99 L20.49 19 l-4.99 -5 ZM9.5 " +
    "14 C7.01 14 5 11.99 5 9.5 S7.01 5 9.5 5 S14 7.01 14 9.5 S11.99 14 9.5 14 Z"

// What:     `private const val SETTINGS_PATH: String` holds the gear outline.
// Why:      The path was copied from Material icon bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_PATH = "<svg path data>";
// ```
/** SVG path data for the settings gear on a 24 by 24 grid. */
private const val SETTINGS_PATH: String =
    "M19.14 12.94 c0.04 -0.3 0.06 -0.61 0.06 -0.94 c0 -0.32 -0.02 -0.64 -0.07 -0.94 l2.03 -1.58 c0.18 " +
    "-0.14 0.23 -0.41 0.12 -0.61 l-1.92 -3.32 c-0.12 -0.22 -0.37 -0.29 -0.59 -0.22 l-2.39 0.96 c-0.5 " +
    "-0.38 -1.03 -0.7 -1.62 -0.94 L14.4 2.81 c-0.04 -0.24 -0.24 -0.41 -0.48 -0.41 h-3.84 c-0.24 0 " +
    "-0.43 0.17 -0.47 0.41 L9.25 5.35 C8.66 5.59 8.12 5.92 7.63 6.29 L5.24 5.33 c-0.22 -0.08 -0.47 0 " +
    "-0.59 0.22 L2.74 8.87 C2.62 9.08 2.66 9.34 2.86 9.48 l2.03 1.58 C4.84 11.36 4.8 11.69 4.8 12 " +
    "s0.02 0.64 0.07 0.94 l-2.03 1.58 c-0.18 0.14 -0.23 0.41 -0.12 0.61 l1.92 3.32 c0.12 0.22 0.37 " +
    "0.29 0.59 0.22 l2.39 -0.96 c0.5 0.38 1.03 0.7 1.62 0.94 l0.36 2.54 c0.05 0.24 0.24 0.41 0.48 " +
    "0.41 h3.84 c0.24 0 0.44 -0.17 0.47 -0.41 l0.36 -2.54 c0.59 -0.24 1.13 -0.56 1.62 -0.94 l2.39 " +
    "0.96 c0.22 0.08 0.47 0 0.59 -0.22 l1.92 -3.32 c0.12 -0.22 0.07 -0.47 -0.12 -0.61 L19.14 12.94 " +
    "ZM12 15.6 c-1.98 0 -3.6 -1.62 -3.6 -3.6 s1.62 -3.6 3.6 -3.6 s3.6 1.62 3.6 3.6 S13.98 15.6 12 " +
    "15.6 Z"

// What:     `private const val FOLDER_OPEN_PATH: String` holds the open folder outline.
// Why:      The path was copied from extended Material icon bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_OPEN_PATH = "<svg path data>";
// ```
/** SVG path data for the open folder glyph on a 24 by 24 grid. */
private const val FOLDER_OPEN_PATH: String =
    "M20 6 h-8 l-2 -2 L4 4 c-1.1 0 -1.99 0.9 -1.99 2 L2 18 c0 1.1 0.9 2 2 2 h16 c1.1 0 2 -0.9 2 -2 " +
    "L22 8 c0 -1.1 -0.9 -2 -2 -2 ZM20 18 L4 18 L4 8 h16 v10 Z"

// What:     `private const val ARROW_DROP_DOWN_PATH: String` holds the downward caret triangle.
// Why:      The path was copied from Material icon bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const ARROW_DROP_DOWN_PATH = "<svg path data>";
// ```
/** SVG path data for the downward caret on a 24 by 24 grid. */
private const val ARROW_DROP_DOWN_PATH: String =
    "M7 10 l5 5 l5 -5 Z"

// What:     `private const val ARROW_DROP_UP_PATH: String` holds the upward caret triangle.
// Why:      The path was copied from extended Material icon bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const ARROW_DROP_UP_PATH = "<svg path data>";
// ```
/** SVG path data for the upward caret on a 24 by 24 grid. */
private const val ARROW_DROP_UP_PATH: String =
    "M7 14 l5 -5 l5 5 Z"

// What:     `private fun playerIcon(name: String, pathData: String): ImageVector` builds one filled glyph.
// Why:      Every glyph shares the same size, viewport, and fill, so one builder keeps them consistent.
//
// In TS you'd write (pseudocode):
// ```ts
// function playerIcon(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun playerIcon(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = ICON_SIZE,
    defaultHeight = ICON_SIZE,
    viewportWidth = ICON_VIEWPORT,
    viewportHeight = ICON_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `internal val SEARCH_ICON: ImageVector` is the search glyph.
// Why:      The top bar uses it for the search action, shown only when a search page exists.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ICON = playerIcon("Player.Search", SEARCH_PATH);
// ```
/** Magnifying glass glyph for the search action. */
internal val SEARCH_ICON: ImageVector = playerIcon(name = "Player.Search", pathData = SEARCH_PATH)

// What:     `internal val SETTINGS_ICON: ImageVector` is the gear glyph.
// Why:      The top bar and the unfolded header use it for the settings action.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON = playerIcon("Player.Settings", SETTINGS_PATH);
// ```
/** Gear glyph for the settings action. */
internal val SETTINGS_ICON: ImageVector = playerIcon(name = "Player.Settings", pathData = SETTINGS_PATH)

// What:     `internal val FOLDER_OPEN_ICON: ImageVector` is the open folder glyph.
// Why:      The Open action pairs this glyph with the label `Open`.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_OPEN_ICON = playerIcon("Player.FolderOpen", FOLDER_OPEN_PATH);
// ```
/** Open folder glyph shown inside the Open action. */
internal val FOLDER_OPEN_ICON: ImageVector = playerIcon(name = "Player.FolderOpen", pathData = FOLDER_OPEN_PATH)

// What:     `internal val ARROW_DROP_DOWN_ICON: ImageVector` is the downward caret.
// Why:      The folder title trigger shows it while the picker is closed, per the design decision D104.
//
// In TS you'd write (pseudocode):
// ```ts
// const ARROW_DROP_DOWN_ICON = playerIcon("Player.ArrowDropDown", ARROW_DROP_DOWN_PATH);
// ```
/** Downward caret shown on the folder title while the picker is closed. */
internal val ARROW_DROP_DOWN_ICON: ImageVector =
    playerIcon(name = "Player.ArrowDropDown", pathData = ARROW_DROP_DOWN_PATH)

// What:     `internal val ARROW_DROP_UP_ICON: ImageVector` is the upward caret.
// Why:      The folder title trigger shows it while the picker is open, per the design decision D104.
//
// In TS you'd write (pseudocode):
// ```ts
// const ARROW_DROP_UP_ICON = playerIcon("Player.ArrowDropUp", ARROW_DROP_UP_PATH);
// ```
/** Upward caret shown on the folder title while the picker is open. */
internal val ARROW_DROP_UP_ICON: ImageVector =
    playerIcon(name = "Player.ArrowDropUp", pathData = ARROW_DROP_UP_PATH)
