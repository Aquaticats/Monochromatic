// What:     `package dev.monochromatic.musicplayer` places the two glyphs beside the other player composables.
// Why:      The template editor and Settings draw a Back arrow and an error mark without the extended icon set.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.ui.graphics` bring in the fill colour and the vector builder helpers.
// Why:      Each glyph is one filled path, tinted by the caller.
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

// What:     `private val GLYPH_SIZE: Dp = 24.dp` is the nominal width and height of each glyph.
// Why:      The path data is drawn on the 24 by 24 grid of the Material icon set it copies.
//
// In TS you'd write (pseudocode):
// ```ts
// const GLYPH_SIZE = 24;
// ```
/** Nominal width and height of each glyph. */
private val GLYPH_SIZE: Dp = 24.dp

// What:     `private const val GLYPH_VIEWPORT: Float = 24f` is the coordinate extent of each glyph path.
// Why:      The path data is written on a 24 by 24 grid, so it scales with `GLYPH_SIZE`.
//
// In TS you'd write (pseudocode):
// ```ts
// const GLYPH_VIEWPORT = 24;
// ```
/** Coordinate extent of each glyph path on both axes. */
private const val GLYPH_VIEWPORT: Float = 24f

// What:     `private const val ARROW_BACK_PATH: String` holds the leftward arrow outline.
// Why:      The path was read from the Material ArrowBack class in the local icon artifacts, so the glyph matches it.
//
// In TS you'd write (pseudocode):
// ```ts
// const ARROW_BACK_PATH = "<svg path data>";
// ```
/** SVG path data for the Back arrow on a 24 by 24 grid. */
private const val ARROW_BACK_PATH: String =
    "M20 11 H7.83 l5.59 -5.59 L12 4 l-8 8 8 8 l1.41 -1.41 L7.83 13 H20 v-2 Z"

// What:     `private const val ERROR_PATH: String` holds the filled circle with an exclamation mark.
// Why:      The path was read from the Material Error class in the local icon artifacts, so the glyph matches it.
//
// In TS you'd write (pseudocode):
// ```ts
// const ERROR_PATH = "<svg path data>";
// ```
/** SVG path data for the error mark on a 24 by 24 grid. */
private const val ERROR_PATH: String =
    "M12 2 C6.48 2 2 6.48 2 12 s4.48 10 10 10 s10 -4.48 10 -10 S17.52 2 12 2 Z " +
    "M13 17 h-2 v-2 h2 v2 Z M13 13 h-2 L11 7 h2 v6 Z"

// What:     `private fun templateEditorIcon(name: String, pathData: String): ImageVector` builds one filled glyph.
// Why:      Both glyphs share one size, viewport and fill, so one builder keeps them consistent.
//
// In TS you'd write (pseudocode):
// ```ts
// function templateEditorIcon(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun templateEditorIcon(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = GLYPH_SIZE,
    defaultHeight = GLYPH_SIZE,
    viewportWidth = GLYPH_VIEWPORT,
    viewportHeight = GLYPH_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `internal val DESTINATION_BACK_ICON: ImageVector` is the Back arrow glyph.
// Why:      Both the editor header and the Settings header use it for the way back.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_BACK_ICON = templateEditorIcon("TemplateEditor.Back", ARROW_BACK_PATH);
// ```
/** Leftward arrow glyph for the way back. */
internal val DESTINATION_BACK_ICON: ImageVector = templateEditorIcon(
    name = "TemplateEditor.Back",
    pathData = ARROW_BACK_PATH,
)

// What:     `internal val TEMPLATE_ERROR_ICON: ImageVector` is the error mark beside each mistake line.
// Why:      The icon is a second channel beside the error colour, so the mistake does not rest on colour alone.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEMPLATE_ERROR_ICON = templateEditorIcon("TemplateEditor.Error", ERROR_PATH);
// ```
/** Filled circle with an exclamation mark, shown beside each mistake line. */
internal val TEMPLATE_ERROR_ICON: ImageVector = templateEditorIcon(name = "TemplateEditor.Error", pathData = ERROR_PATH)
