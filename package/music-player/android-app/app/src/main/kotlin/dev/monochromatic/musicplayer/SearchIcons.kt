// Glyphs the Search page draws for its Back, Clear, and track-kind marks. Each path was copied from the
// Material icon bytecode in the bannerfit Gradle cache, the same way the player glyphs were copied.

// What:     `package dev.monochromatic.musicplayer` places the Search glyphs beside the player glyphs.
// Why:      The Search page imports these vectors without reaching into another composable file.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import androidx.compose.ui.graphics.Color` supplies the black fill of each glyph.
// Why:      Each glyph is one filled path, and the caller tints it through the icon composable.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Color } from "compose/ui/graphics";
// ```
import androidx.compose.ui.graphics.Color

// What:     `import androidx.compose.ui.graphics.SolidColor` wraps the black fill as a brush.
// Why:      The path builder takes a brush, and a solid fill keeps the glyph tintable.
//
// In TS you'd write (pseudocode):
// ```ts
// import { SolidColor } from "compose/ui/graphics";
// ```
import androidx.compose.ui.graphics.SolidColor

// What:     `import androidx.compose.ui.graphics.vector.ImageVector` names the glyph type.
// Why:      Icon composables accept an ImageVector, so each glyph is built as one.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { ImageVector } from "compose/ui/graphics/vector";
// ```
import androidx.compose.ui.graphics.vector.ImageVector

// What:     `import androidx.compose.ui.graphics.vector.addPathNodes` parses SVG path data into path nodes.
// Why:      The copied path strings are written in the SVG path grammar.
//
// In TS you'd write (pseudocode):
// ```ts
// import { addPathNodes } from "compose/ui/graphics/vector";
// ```
import androidx.compose.ui.graphics.vector.addPathNodes

// What:     `import androidx.compose.ui.unit.Dp` and `dp` give each glyph its density-independent size.
// Why:      The glyph size scales with display density, like the Material icon set it copies.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Dp, dp } from "compose/ui/unit";
// ```
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private val SEARCH_ICON_SIZE: Dp = 24.dp` is the nominal width and height of each glyph.
// Why:      The path data is drawn on the 24 by 24 grid of the Material icon set.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ICON_SIZE = 24;
// ```
/** Nominal width and height of each Search glyph. */
private val SEARCH_ICON_SIZE: Dp = 24.dp

// What:     `private const val SEARCH_ICON_VIEWPORT: Float = 24f` is the coordinate extent of each glyph path.
// Why:      The copied path data is written on a 24 by 24 grid, so it scales with `SEARCH_ICON_SIZE`.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ICON_VIEWPORT = 24;
// ```
/** Coordinate extent of each Search glyph path on both axes. */
private const val SEARCH_ICON_VIEWPORT: Float = 24f

// What:     `private const val SEARCH_BACK_PATH: String` holds the filled arrow-back outline.
// Why:      Copied from `Icons.Filled.ArrowBack` in material-icons-core 1.7.8 bytecode.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_BACK_PATH = "M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z";
// ```
/** SVG path data for the Back arrow on a 24 by 24 grid. */
private const val SEARCH_BACK_PATH: String = "M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"

// What:     `private const val SEARCH_CLEAR_PATH: String` holds the filled close cross.
// Why:      Copied from `Icons.Filled.Close` in material-icons-core 1.7.8 bytecode.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_CLEAR_PATH = "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 ...";
// ```
/** SVG path data for the Clear cross on a 24 by 24 grid. */
private const val SEARCH_CLEAR_PATH: String =
    "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"

// What:     `private const val SEARCH_TRACK_PATH: String` holds the filled music-note outline.
// Why:      Copied from `Icons.Filled.MusicNote` in material-icons-extended 1.7.8 bytecode, so track rows
//           carry the music glyph that D58 aligns with the folder glyph.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_TRACK_PATH = "M12 3v10.55c-0.59-0.34-1.27-0.55-2-0.55-2.21 0-4 1.79-4 4s1.79 4 ...";
// ```
/** SVG path data for the track-kind music note on a 24 by 24 grid. */
private const val SEARCH_TRACK_PATH: String =
    "M12 3v10.55c-0.59-0.34-1.27-0.55-2-0.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"

// What:     `private fun searchGlyph(name: String, pathData: String): ImageVector` builds one filled glyph.
// Why:      Every Search glyph shares its size, viewport, and fill, so one builder keeps them consistent.
//
// In TS you'd write (pseudocode):
// ```ts
// function searchGlyph(name: string, pathData: string): ImageVector { /* ... */ }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun searchGlyph(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = SEARCH_ICON_SIZE,
    defaultHeight = SEARCH_ICON_SIZE,
    viewportWidth = SEARCH_ICON_VIEWPORT,
    viewportHeight = SEARCH_ICON_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `internal val SEARCH_BACK_ICON: ImageVector` is the Back arrow of the Search header.
// Why:      The visible Back control returns to the player (D64), so it needs its own glyph.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_BACK_ICON = searchGlyph("Search.Back", SEARCH_BACK_PATH);
// ```
/** Back arrow glyph for the Search header. */
internal val SEARCH_BACK_ICON: ImageVector = searchGlyph(name = "Search.Back", pathData = SEARCH_BACK_PATH)

// What:     `internal val SEARCH_CLEAR_ICON: ImageVector` is the Clear cross of the Search header.
// Why:      Clear erases the query (D65), and its glyph is shown only while the query is not blank.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_CLEAR_ICON = searchGlyph("Search.Clear", SEARCH_CLEAR_PATH);
// ```
/** Clear cross glyph for the Search header. */
internal val SEARCH_CLEAR_ICON: ImageVector = searchGlyph(name = "Search.Clear", pathData = SEARCH_CLEAR_PATH)

// What:     `internal val SEARCH_TRACK_ICON: ImageVector` is the music-note mark of a track result.
// Why:      D58 aligns track glyphs with folder glyphs, so track rows need a note of their own.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_TRACK_ICON = searchGlyph("Search.Track", SEARCH_TRACK_PATH);
// ```
/** Music-note glyph for a track result row. */
internal val SEARCH_TRACK_ICON: ImageVector = searchGlyph(name = "Search.Track", pathData = SEARCH_TRACK_PATH)
