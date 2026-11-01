// What:     `package dev.monochromatic.musicplayer` places the feedback glyphs beside the overlay that draws them.
// Why:      The overlay imports the glyphs without reaching into another icon file.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.ui.graphics` bring in the fill color and the vector builder helpers.
// Why:      Each glyph is one filled path, and the caller applies the tint.
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

// What:     `private val FEEDBACK_GLYPH_SIZE: Dp = 24.dp` is the nominal width and height of each feedback glyph.
// Why:      The path data is drawn on the 24 by 24 grid of the Material icon set it was decoded from.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_GLYPH_SIZE = 24;
// ```
/** Nominal width and height of each feedback glyph. */
private val FEEDBACK_GLYPH_SIZE: Dp = 24.dp

// What:     `private const val FEEDBACK_GLYPH_VIEWPORT: Float = 24f` is the coordinate extent of each glyph path.
// Why:      The path data is written on a 24 by 24 grid so it scales with `FEEDBACK_GLYPH_SIZE`.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_GLYPH_VIEWPORT = 24;
// ```
/** Coordinate extent of each glyph path on both axes. */
private const val FEEDBACK_GLYPH_VIEWPORT: Float = 24f

// What:     `private const val FEEDBACK_WARNING_PATH: String` holds the warning triangle with its exclamation cutout.
// Why:      The path was decoded from the Material Filled.Warning class, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_WARNING_PATH = "<svg path data>";
// ```
/** SVG path data for the warning glyph on a 24 by 24 grid. */
private const val FEEDBACK_WARNING_PATH: String =
    "M1 21 h22 L12 2 L1 21 Z M13 18 h-2 v-2 h2 v2 Z M13 14 h-2 v-4 h2 v4 Z"

// What:     `private const val FEEDBACK_CLOSE_PATH: String` holds the close cross.
// Why:      The path was decoded from the Material Filled.Close class, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_CLOSE_PATH = "<svg path data>";
// ```
/** SVG path data for the close glyph on a 24 by 24 grid. */
private const val FEEDBACK_CLOSE_PATH: String =
    "M19 6.41 L17.59 5 L12 10.59 L6.41 5 L5 6.41 L10.59 12 L5 17.59 L6.41 19 L12 13.41 L17.59 19 L19 17.59 L13.41 12 Z"

// What:     `private fun feedbackGlyph(name: String, pathData: String): ImageVector` builds one filled glyph.
// Why:      Both glyphs share the same size, viewport and fill, so one builder keeps them consistent.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackGlyph(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun feedbackGlyph(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = FEEDBACK_GLYPH_SIZE,
    defaultHeight = FEEDBACK_GLYPH_SIZE,
    viewportWidth = FEEDBACK_GLYPH_VIEWPORT,
    viewportHeight = FEEDBACK_GLYPH_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `internal val FEEDBACK_WARNING_ICON: ImageVector` is the warning glyph for error messages.
// Why:      Error state keeps a shape channel beside its container color.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_WARNING_ICON = feedbackGlyph("Feedback.Warning", FEEDBACK_WARNING_PATH);
// ```
/** Warning glyph shown at the start of an error message. */
internal val FEEDBACK_WARNING_ICON: ImageVector =
    feedbackGlyph(name = "Feedback.Warning", pathData = FEEDBACK_WARNING_PATH)

// What:     `internal val FEEDBACK_CLOSE_ICON: ImageVector` is the close glyph for dismissal.
// Why:      The dismiss control names its action through the glyph and its content description.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_CLOSE_ICON = feedbackGlyph("Feedback.Close", FEEDBACK_CLOSE_PATH);
// ```
/** Close glyph shown in the dismiss control of every message. */
internal val FEEDBACK_CLOSE_ICON: ImageVector =
    feedbackGlyph(name = "Feedback.Close", pathData = FEEDBACK_CLOSE_PATH)
