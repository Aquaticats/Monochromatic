// What:     `package dev.monochromatic.musicplayer` places the track menu glyphs beside the other player glyphs.
// Why:      The track menu popup imports the glyphs without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the closed action type.
// Why:      Each action maps to exactly one glyph, so the mapping is checked by the compiler.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TrackMenuAction } from "core/TrackMenuAction";
// ```
import dev.monochromatic.musicplayer.core.TrackMenuAction

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
// Why:      The path data is drawn on the 24 by 24 grid of the Material icon set it was extracted from.
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

// What:     `private const val PLAY_PATH: String` holds the filled play triangle.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAY_PATH = "<svg path data>";
// ```
/** SVG path data for the play glyph on a 24 by 24 grid. */
private const val PLAY_PATH: String = "M8 5 v14 l11 -7 Z"

// What:     `private const val SHUFFLE_PATH: String` holds the two crossing shuffle arrows.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const SHUFFLE_PATH = "<svg path data>";
// ```
/** SVG path data for the shuffle glyph on a 24 by 24 grid. */
private const val SHUFFLE_PATH: String =
    "M10.59 9.17 L5.41 4 L4 5.41 l5.17 5.17 l1.42 -1.41 Z M14.5 4 l2.04 2.04 L4 18.59 L5.41 20 " +
    "L17.96 7.46 L20 9.5 L20 4 h-5.5 Z M14.83 13.41 l-1.41 1.41 l3.13 3.13 L14.5 20 L20 20 v-5.5 " +
    "l-2.04 2.04 l-3.13 -3.13 Z"

// What:     `private const val INFO_PATH: String` holds the outlined information circle.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const INFO_PATH = "<svg path data>";
// ```
/** SVG path data for the outlined information glyph on a 24 by 24 grid. */
private const val INFO_PATH: String =
    "M11 7 h2 v2 h-2 Z M11 11 h2 v6 h-2 Z M12 2 C6.48 2 2 6.48 2 12 s4.48 10 10 10 s10 -4.48 10 -10 " +
    "S17.52 2 12 2 Z M12 20 c-4.41 0 -8 -3.59 -8 -8 s3.59 -8 8 -8 s8 3.59 8 8 s-3.59 8 -8 8 Z"

// What:     `private const val REFRESH_PATH: String` holds the circular re-analysis arrow.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const REFRESH_PATH = "<svg path data>";
// ```
/** SVG path data for the refresh glyph on a 24 by 24 grid. */
private const val REFRESH_PATH: String =
    "M17.65 6.35 C16.2 4.9 14.21 4 12 4 c-4.42 0 -7.99 3.58 -7.99 8 s3.57 8 7.99 8 c3.73 0 6.84 -2.55 " +
    "7.73 -6 h-2.08 c-0.82 2.33 -3.04 4 -5.65 4 c-3.31 0 -6 -2.69 -6 -6 s2.69 -6 6 -6 c1.66 0 3.14 " +
    "0.69 4.22 1.78 L13 11 h7 V4 l-2.35 2.35 Z"

// What:     `private const val FOLDER_OPEN_PATH: String` holds the open folder outline.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_OPEN_PATH = "<svg path data>";
// ```
/** SVG path data for the open folder glyph on a 24 by 24 grid. */
private const val FOLDER_OPEN_PATH: String =
    "M20 6 h-8 l-2 -2 L4 4 c-1.1 0 -1.99 0.9 -1.99 2 L2 18 c0 1.1 0.9 2 2 2 h16 c1.1 0 2 -0.9 2 -2 " +
    "L22 8 c0 -1.1 -0.9 -2 -2 -2 Z M20 18 L4 18 L4 8 h16 v10 Z"

// What:     `private const val CONTENT_COPY_PATH: String` holds the two overlapping sheets.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const CONTENT_COPY_PATH = "<svg path data>";
// ```
/** SVG path data for the copy glyph on a 24 by 24 grid. */
private const val CONTENT_COPY_PATH: String =
    "M16 1 L4 1 c-1.1 0 -2 0.9 -2 2 v14 h2 L4 3 h12 L16 1 Z M19 5 L8 5 c-1.1 0 -2 0.9 -2 2 v14 " +
    "c0 1.1 0.9 2 2 2 h11 c1.1 0 2 -0.9 2 -2 L21 7 c0 -1.1 -0.9 -2 -2 -2 Z M19 21 L8 21 L8 7 h11 v14 Z"

// What:     `private const val DELETE_PATH: String` holds the outlined trash can.
// Why:      The path was extracted mechanically from the Material icons AAR bytecode, so the glyph matches the library.
//
// In TS you'd write (pseudocode):
// ```ts
// const DELETE_PATH = "<svg path data>";
// ```
/** SVG path data for the outlined trash glyph on a 24 by 24 grid. */
private const val DELETE_PATH: String =
    "M16 9 v10 H8 V9 h8 m-1.5 -6 h-5 l-1 1 H5 v2 h14 V4 h-3.5 l-1 -1 Z " +
    "M18 7 H6 v12 c0 1.1 0.9 2 2 2 h8 c1.1 0 2 -0.9 2 -2 V7 Z"

// What:     `private fun trackMenuGlyph(name: String, pathData: String): ImageVector` builds one filled glyph.
// Why:      Every glyph shares the same size, viewport, and fill, so one builder keeps them consistent.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackMenuGlyph(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun trackMenuGlyph(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = ICON_SIZE,
    defaultHeight = ICON_SIZE,
    viewportWidth = ICON_VIEWPORT,
    viewportHeight = ICON_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `internal val TRACK_MENU_PLAY_ICON: ImageVector` is the play glyph.
// Why:      The play action shows this glyph beside its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_PLAY_ICON = trackMenuGlyph("TrackMenu.Play", PLAY_PATH);
// ```
/** Play glyph for the play action. */
internal val TRACK_MENU_PLAY_ICON: ImageVector = trackMenuGlyph(name = "TrackMenu.Play", pathData = PLAY_PATH)

// What:     `internal val TRACK_MENU_SHUFFLE_ICON: ImageVector` is the shuffle glyph.
// Why:      The shuffle action shows this glyph beside its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_SHUFFLE_ICON = trackMenuGlyph("TrackMenu.Shuffle", SHUFFLE_PATH);
// ```
/** Shuffle glyph for the shuffle action. */
internal val TRACK_MENU_SHUFFLE_ICON: ImageVector =
    trackMenuGlyph(name = "TrackMenu.Shuffle", pathData = SHUFFLE_PATH)

// What:     `internal val TRACK_MENU_INFO_ICON: ImageVector` is the outlined information glyph.
// Why:      The file details action shows this glyph beside its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_INFO_ICON = trackMenuGlyph("TrackMenu.Info", INFO_PATH);
// ```
/** Information glyph for the file details action. */
internal val TRACK_MENU_INFO_ICON: ImageVector = trackMenuGlyph(name = "TrackMenu.Info", pathData = INFO_PATH)

// What:     `internal val TRACK_MENU_FOLDER_OPEN_ICON: ImageVector` is the open folder glyph.
// Why:      The show-in-file-manager action shows this glyph beside its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_FOLDER_OPEN_ICON = trackMenuGlyph("TrackMenu.FolderOpen", FOLDER_OPEN_PATH);
// ```
/** Open folder glyph for the show-in-file-manager action. */
internal val TRACK_MENU_FOLDER_OPEN_ICON: ImageVector =
    trackMenuGlyph(name = "TrackMenu.FolderOpen", pathData = FOLDER_OPEN_PATH)

// What:     `internal val TRACK_MENU_CONTENT_COPY_ICON: ImageVector` is the copy glyph.
// Why:      The copy filename action shows this glyph beside its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_CONTENT_COPY_ICON = trackMenuGlyph("TrackMenu.ContentCopy", CONTENT_COPY_PATH);
// ```
/** Copy glyph for the copy filename action. */
internal val TRACK_MENU_CONTENT_COPY_ICON: ImageVector =
    trackMenuGlyph(name = "TrackMenu.ContentCopy", pathData = CONTENT_COPY_PATH)

// What:     `internal val TRACK_MENU_DELETE_ICON: ImageVector` is the outlined trash glyph.
// Why:      The destructive action shows this glyph beside its label, so meaning is not carried by color alone.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_DELETE_ICON = trackMenuGlyph("TrackMenu.Delete", DELETE_PATH);
// ```
/** Trash glyph for the destructive move-to-trash action. */
internal val TRACK_MENU_DELETE_ICON: ImageVector = trackMenuGlyph(name = "TrackMenu.Delete", pathData = DELETE_PATH)

// What:     `internal fun trackMenuIcon(action: TrackMenuAction): ImageVector` selects the glyph for one action.
// Why:      The when expression has no fallback, so adding an action without a glyph fails to compile.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackMenuIcon(action: TrackMenuAction): ImageVector { /* exhaustive switch */ }
// ```
/** Returns the decorative glyph shown beside the label of an action. */
internal fun trackMenuIcon(action: TrackMenuAction): ImageVector = when (action) {
    TrackMenuAction.PLAY -> TRACK_MENU_PLAY_ICON
    TrackMenuAction.SHUFFLE_FROM_HERE -> TRACK_MENU_SHUFFLE_ICON
    TrackMenuAction.DETAILS -> TRACK_MENU_INFO_ICON
    TrackMenuAction.SHOW_IN_FILE_MANAGER -> TRACK_MENU_FOLDER_OPEN_ICON
    TrackMenuAction.COPY_FILENAME -> TRACK_MENU_CONTENT_COPY_ICON
    TrackMenuAction.MOVE_TO_TRASH -> TRACK_MENU_DELETE_ICON
}
