// What:     `package dev.monochromatic.musicplayer` places the transport deck beside the other
//           player composables of the app.
// Why:      The deck is a stateless composable that screens import without reaching into
//           MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the four-state playback mode type.
// Why:      The deck displays and reports the same mode that playback uses.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlaybackMode } from "core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.RoleSet
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.headingLineBudget
import dev.monochromatic.musicplayer.core.shuffleFolderTextWidthPx
import dev.monochromatic.musicplayer.core.shuffleLabelSlotWidthPx
import dev.monochromatic.musicplayer.core.titleLineBudget

// What:     Imports from `androidx.compose.foundation` bring in layout, selection, scrolling,
//           system inset, and dark-theme helpers.
// Why:      The deck uses them for its background, seek row, mode group, and safe-area padding.
//
// In TS you'd write (pseudocode):
// ```ts
// import { verticalScroll } from "compose/foundation";
// ```
import androidx.compose.foundation.background
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemGestures
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.material3` bring in Material 3 buttons, text,
//           slider, segmented-button defaults, and the theme accessor.
// Why:      The deck uses Material 3 roles so it follows the app theme.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Slider, Text, OutlinedButton, MaterialTheme } from "material3";
// ```
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedIconButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.rememberSliderState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember

// What:     Imports from `androidx.compose.ui` bring in modifiers, colors, vectors, text
//           measurement, density, layout direction, and semantics.
// Why:      The mode control measures labels and publishes radio semantics for each segment.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Color, Density, LayoutDirection } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.layout.SubcomposeLayout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.traversalIndex
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.LayoutDirection

// What:     `private val DECK_VERTICAL_PADDING: Dp = 16.dp` names the deck's outer vertical inset.
// Why:      The accepted deck keeps 16dp above the heading and below the mode control.
//
// In TS you'd write (pseudocode):
// ```ts
// const DECK_VERTICAL_PADDING = 16;
// ```
/** Outer vertical inset above the heading and below the mode control. */
private val DECK_VERTICAL_PADDING: Dp = 16.dp

// What:     `private val DECK_GROUP_SPACING: Dp = 8.dp` names the gap between deck groups.
// Why:      The accepted cover deck uses 8dp between heading, seek, transport, and mode groups.
//
// In TS you'd write (pseudocode):
// ```ts
// const DECK_GROUP_SPACING = 8;
// ```
/** Vertical gap between the heading, seek row, transport row, and mode control. */
private val DECK_GROUP_SPACING: Dp = 8.dp

// What:     `private val DECK_HORIZONTAL_PADDING: Dp = 16.dp` names the deck's outer horizontal inset.
// Why:      On a phone, with no crease, the deck had no side inset at all, so the clocks, the slider and the mode
//           control touched the screen edges while the list rows and the top bar keep 16dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const DECK_HORIZONTAL_PADDING = 16;
// ```
/** Outer horizontal inset of the deck's content, equal to the list rows'; a larger crease clearance wins. */
private val DECK_HORIZONTAL_PADDING: Dp = 16.dp

// What:     `private val STACKED_CONTROL_SPACING: Dp = 16.dp` names the gap between stacked transport and mode groups.
// Why:      At large text the transport buttons sit above the mode control, and at 8dp their outlines read as touching.
//
// In TS you'd write (pseudocode):
// ```ts
// const STACKED_CONTROL_SPACING = 16;
// ```
/** Vertical gap between the transport row and the mode control when they are stacked. */
private val STACKED_CONTROL_SPACING: Dp = 16.dp

// What:     `private val CONTROL_SPACING: Dp = 8.dp` names the horizontal gap in seek and transport rows.
// Why:      The accepted deck spaces time labels, slider, and transport buttons at 8dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const CONTROL_SPACING = 8;
// ```
/** Horizontal gap between time labels, the slider, and transport buttons. */
private val CONTROL_SPACING: Dp = 8.dp

// What:     `private val MIN_TARGET: Dp = 48.dp` names the minimum touch target of each segment.
// Why:      Every mode segment keeps a 48dp minimum target at every layout.
//
// In TS you'd write (pseudocode):
// ```ts
// const MIN_TARGET = 48;
// ```
/** Minimum width and height of one tappable target, such as a mode segment or a heading part. */
private val MIN_TARGET: Dp = 48.dp

// What:     `private val ONE_LINE_CONTROL_GAP: Dp = 16.dp` is the gap between transport and mode when they share a row.
// Why:      Sixteen dp separates the two groups clearly, and it is counted once in the one-line width check.
//
// In TS you'd write (pseudocode):
// ```ts
// const ONE_LINE_CONTROL_GAP = 16;
// ```
/** Minimum horizontal gap between the transport buttons and the mode control when they share one row. */
private val ONE_LINE_CONTROL_GAP: Dp = 16.dp

// What:     `private const val FOLDER_MAX_WIDTH_FRACTION: Float = 0.4f` caps the folder part of the heading.
// Why:      The folder name may take at most 40 percent of the heading width, so the track title keeps the rest.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_MAX_WIDTH_FRACTION = 0.4;
// ```
/** Largest share of the heading width the folder part may take before it is cut with an ellipsis. */
private const val FOLDER_MAX_WIDTH_FRACTION: Float = 0.4f

// What:     `private const val LARGE_TEXT_FOLDER_FRACTION: Float = 0.5f` is the folder part's share at large text.
// Why:      At 150 percent text and more each part wraps, and the folder name gets half the width so it needs fewer
//           lines before it is cut.
//
// In TS you'd write (pseudocode):
// ```ts
// const LARGE_TEXT_FOLDER_FRACTION = 0.5;
// ```
/** Largest share of the heading width the folder part may take at large text sizes. */
private const val LARGE_TEXT_FOLDER_FRACTION: Float = 0.5f

// What:     `private val HEADING_PART_TEXT_PADDING: Dp = 4.dp` keeps a tappable part's text off its focus ring.
// Why:      The focus ring is drawn on the edge of the 48dp cell, and text that touches the cell edge touches the ring.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_PART_TEXT_PADDING = 4;
// ```
/** Horizontal padding between a tappable heading part's text and its cell edge. */
private val HEADING_PART_TEXT_PADDING: Dp = 4.dp

// What:     `private val HEADING_SEPARATOR_PADDING: Dp = 4.dp` is the space on each side of the heading's slash.
// Why:      The spaces of the old ` / ` text are gone, so the slash gets its own small gap, which does not grow
//           with the 48dp cells around the folder and title.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_SEPARATOR_PADDING = 4;
// ```
/** Horizontal padding on each side of the slash between the folder and the title. */
private val HEADING_SEPARATOR_PADDING: Dp = 4.dp

// What:     `private val SEGMENT_OVERLAP: Dp = (-1).dp` shares one hairline between adjacent segments.
// Why:      Connected segments overlap by one dp so the outlines read as one control.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEGMENT_OVERLAP = -1;
// ```
/** Negative spacing that makes adjacent segment outlines share one edge. */
private val SEGMENT_OVERLAP: Dp = (-1).dp

// What:     `private val SEGMENT_OUTER_RADIUS: Dp = 20.dp` rounds only the outside corners.
// Why:      Multi-row layouts keep internal edges square so the group reads as one control.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEGMENT_OUTER_RADIUS = 20;
// ```
/** Corner radius applied only to the outside corners of a multi-row mode group. */
private val SEGMENT_OUTER_RADIUS: Dp = 20.dp

// What:     `private val ONE_ROW_MIN_WIDTH: Dp = 189.dp` is the narrowest width of one row.
// Why:      Four 48dp targets less three shared edges keep every segment at its minimum target.
//
// In TS you'd write (pseudocode):
// ```ts
// const ONE_ROW_MIN_WIDTH = 4 * 48 - 3;
// ```
/** Width floor below which the mode control stops using one row. */
private val ONE_ROW_MIN_WIDTH: Dp = 189.dp

// What:     `private val TWO_ROW_MIN_WIDTH: Dp = 95.dp` is the narrowest width of two rows.
// Why:      Two 48dp targets less one shared edge keep each row segment at its minimum target.
//
// In TS you'd write (pseudocode):
// ```ts
// const TWO_ROW_MIN_WIDTH = 2 * 48 - 1;
// ```
/** Width floor below which the mode control stops using two rows. */
private val TWO_ROW_MIN_WIDTH: Dp = 95.dp

// What:     `private val SEGMENT_ICON_END_SPACING: Dp = 8.dp` separates the check mark from its label.
// Why:      The selected segment keeps its check mark visually attached to the label.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEGMENT_ICON_END_SPACING = 8;
// ```
/** Gap between the check mark and the label of the selected segment. */
private val SEGMENT_ICON_END_SPACING: Dp = 8.dp

// What:     `private val ICON_SIZE: Dp = 24.dp` is the nominal size of each transport glyph.
// Why:      The drawn glyphs use the same 24dp grid as the Material icon set they replace.
//
// In TS you'd write (pseudocode):
// ```ts
// const ICON_SIZE = 24;
// ```
/** Nominal width and height of each drawn transport glyph. */
private val ICON_SIZE: Dp = 24.dp

// What:     `private const val ICON_VIEWPORT: Float = 24f` is the coordinate space of each glyph path.
// Why:      Path data is written on a 24 by 24 grid so it scales with `ICON_SIZE`.
//
// In TS you'd write (pseudocode):
// ```ts
// const ICON_VIEWPORT = 24;
// ```
/** Coordinate extent of each glyph path on both axes. */
private const val ICON_VIEWPORT: Float = 24f

// What:     `private const val SECONDS_PER_MINUTE: Long = 60L` converts whole seconds to minutes.
// Why:      The time labels print `m:ss` from a whole-second count.
//
// In TS you'd write (pseudocode):
// ```ts
// const SECONDS_PER_MINUTE = 60;
// ```
/** Number of seconds in one minute for the clock label. */
private const val SECONDS_PER_MINUTE: Long = 60L

// What:     `private const val SHUFFLE_PREFIX: String = "Shuffle "` is the fixed label prefix.
// Why:      The folder name follows a stable prefix so the verb stays visible when the name truncates.
//
// In TS you'd write (pseudocode):
// ```ts
// const SHUFFLE_PREFIX = "Shuffle ";
// ```
/** Fixed text before the folder name in the shuffle-folder segment. */
private const val SHUFFLE_PREFIX: String = "Shuffle "

// What:     `private const val SHUFFLE_WIDTH_REFERENCE: String = "Camellia"` sets the width cap.
// Why:      Long folder names truncate to the rendered width of this reference name.
//
// In TS you'd write (pseudocode):
// ```ts
// const SHUFFLE_WIDTH_REFERENCE = "Camellia";
// ```
/** Reference folder name whose rendered width caps the shuffle-folder text. */
private const val SHUFFLE_WIDTH_REFERENCE: String = "Camellia"

// What:     `private const val CHECK_PATH: String` holds the check mark outline.
// Why:      The check mark appears on the selected segment without an extended icon dependency.
//
// In TS you'd write (pseudocode):
// ```ts
// const CHECK_PATH = "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z";
// ```
/** SVG path data for the check mark on a 24 by 24 grid. */
private const val CHECK_PATH: String = "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"

// What:     `private const val PLAY_PATH: String` holds the play triangle outline.
// Why:      The play button shows this glyph while playback is paused.
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAY_PATH = "M8 5v14l11-7z";
// ```
/** SVG path data for the play triangle on a 24 by 24 grid. */
private const val PLAY_PATH: String = "M8 5v14l11-7z"

// What:     `private const val PAUSE_PATH: String` holds the two pause bars.
// Why:      The pause button shows this glyph while playback is running.
//
// In TS you'd write (pseudocode):
// ```ts
// const PAUSE_PATH = "M6 19h4V5H6v14zm8-14v14h4V5h-4z";
// ```
/** SVG path data for the two pause bars on a 24 by 24 grid. */
private const val PAUSE_PATH: String = "M6 19h4V5H6v14zm8-14v14h4V5h-4z"

// What:     `private const val SKIP_PREVIOUS_PATH: String` holds the previous-track glyph.
// Why:      The previous button uses a bar and a left-pointing triangle.
//
// In TS you'd write (pseudocode):
// ```ts
// const SKIP_PREVIOUS_PATH = "M6 6h2v12H6zM9.5 12l8.5 6V6z";
// ```
/** SVG path data for the previous-track glyph on a 24 by 24 grid. */
private const val SKIP_PREVIOUS_PATH: String = "M6 6h2v12H6zM9.5 12l8.5 6V6z"

// What:     `private const val SKIP_NEXT_PATH: String` holds the next-track glyph.
// Why:      The next button uses a right-pointing triangle and a bar.
//
// In TS you'd write (pseudocode):
// ```ts
// const SKIP_NEXT_PATH = "M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z";
// ```
/** SVG path data for the next-track glyph on a 24 by 24 grid. */
private const val SKIP_NEXT_PATH: String = "M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"

// What:     `data class TransportDeckModel(...)` is the immutable state the deck displays.
// Why:      The caller owns playback state and passes one snapshot per frame.
//
// In TS you'd write (pseudocode):
// ```ts
// type TransportDeckModel = { title: string; subtitle: string; positionSec: number;
//   durationSec: number; playing: boolean; mode: PlaybackMode; shuffleFolderLabel: string;
//   folderLabel: string };
// ```
/** Snapshot of playback state rendered by [transportDeck]. */
data class TransportDeckModel(
    // What:     `val title: String` is the playing track title.
    // Why:      The heading names the track the deck controls.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // title: string;
    // ```
    /** Title of the playing track shown in the heading. */
    val title: String,
    // What:     `val subtitle: String` is the preformatted line under the title.
    // Why:      The caller formats the position and loudness text, so the deck draws it unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // subtitle: string;
    // ```
    /** Preformatted line drawn under the title, such as the track index and true peak. */
    val subtitle: String,
    // What:     `val positionSec: Double` is the playhead in seconds.
    // Why:      The seek row shows the elapsed time and the slider position from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // positionSec: number;
    // ```
    /** Current playhead position in seconds. */
    val positionSec: Double,
    // What:     `val durationSec: Double` is the track length in seconds.
    // Why:      The seek row shows the total time and scales slider fractions to seconds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // durationSec: number;
    // ```
    /** Total track length in seconds. */
    val durationSec: Double,
    // What:     `val playing: Boolean` records whether playback is running.
    // Why:      The center button shows pause while playing and play while paused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // playing: boolean;
    // ```
    /** True while playback is running. */
    val playing: Boolean,
    // What:     `val mode: PlaybackMode` is the selected playback mode.
    // Why:      The mode control marks exactly one segment as selected.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mode: PlaybackMode;
    // ```
    /** Playback mode whose segment is marked selected. */
    val mode: PlaybackMode,
    // What:     `val shuffleFolderLabel: String` is the folder name shown in the shuffle segment.
    // Why:      The shuffle-folder segment names the folder that shuffling is scoped to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // shuffleFolderLabel: string;
    // ```
    /** Folder name shown after the shuffle prefix in the shuffle-folder segment. */
    val shuffleFolderLabel: String,
    // What:     `val folderLabel: String` is the folder that holds the playing track.
    // Why:      The heading shows it before the track title, and an empty text means nothing is playing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // folderLabel: string;
    // ```
    /** Name of the directory that holds the playing track, or an empty text when no track is playing. */
    val folderLabel: String = "",
)

// What:     `data class TransportDeckActions(...)` bundles the callbacks the deck reports through.
// Why:      The deck never mutates playback itself; the caller decides what each action does.
//
// In TS you'd write (pseudocode):
// ```ts
// type TransportDeckActions = { onSeek(sec: number): void; onPrevious(): void;
//   onTogglePlay(): void; onNext(): void; onSelectMode(mode: PlaybackMode): void;
//   onRevealPlaying?(): void; onRevealFolder?(): void };
// ```
/** Callbacks the deck invokes when the user acts on it. */
data class TransportDeckActions(
    // What:     `val onSeek: (Double) -> Unit` receives an absolute target position.
    // Why:      The slider reports a fraction that the deck converts to seconds before calling it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onSeek: (positionSec: number) => void;
    // ```
    /** Requests a seek to an absolute position in seconds. */
    val onSeek: (Double) -> Unit,
    // What:     `val onPrevious: () -> Unit` reports the previous-track button.
    // Why:      The caller decides which track precedes the playing one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onPrevious: () => void;
    // ```
    /** Requests the previous track. */
    val onPrevious: () -> Unit,
    // What:     `val onTogglePlay: () -> Unit` reports the center play or pause button.
    // Why:      The caller owns the playback state and decides whether to play or pause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onTogglePlay: () => void;
    // ```
    /** Requests a play or pause toggle. */
    val onTogglePlay: () -> Unit,
    // What:     `val onNext: () -> Unit` reports the next-track button.
    // Why:      The caller decides which track follows the playing one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onNext: () => void;
    // ```
    /** Requests the next track. */
    val onNext: () -> Unit,
    // What:     `val onSelectMode: (PlaybackMode) -> Unit` reports a mode-segment selection.
    // Why:      The caller persists the mode, so the deck only reports the choice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onSelectMode: (mode: PlaybackMode) => void;
    // ```
    /** Requests selection of one playback mode. */
    val onSelectMode: (PlaybackMode) -> Unit,
    // What:     `val onRevealPlaying: (() -> Unit)?` reports a tap on the playing title, or is null for no tap target.
    // Why:      A tap on the playing track's name jumps the list to that track; the deck under Search has no list to
    //           jump in, so it passes null and draws the heading as plain text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onRevealPlaying?: () => void;
    // ```
    /** Requests that the list shows the playing track, or null when the heading is not tappable. */
    val onRevealPlaying: (() -> Unit)? = null,
    // What:     `val onRevealFolder: (() -> Unit)?` reports a tap on the folder part of the heading, or is null
    //           for no tap target.
    // Why:      A tap on the folder name scrolls the list to the playing track's subfolder, and the deck under Search
    //           passes null so the folder name is plain text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onRevealFolder?: () => void;
    // ```
    /** Requests that the list shows the folder of the playing track, or null when the folder part is not tappable. */
    val onRevealFolder: (() -> Unit)? = null,
)

// What:     `internal enum class ModeControlLayout` names the three arrangements of the mode control.
// Why:      A pure function can choose the arrangement and tests can assert the choice without Compose.
//
// In TS you'd write (pseudocode):
// ```ts
// type ModeControlLayout = "one-row" | "two-row" | "four-row";
// ```
/** Arrangements the mode control can take, from fewest rows to most rows. */
internal enum class ModeControlLayout {
    // What:     `ONE_ROW` is the single connected row with intrinsic segment widths.
    // Why:      It is the first choice when every full label fits at the current width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const oneRow = "one-row";
    // ```
    /** One connected row of four segments with intrinsic widths. */
    ONE_ROW,

    // What:     `TWO_ROW` is the connected two-by-two block.
    // Why:      It is the second choice when labels fit two per row but not one row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const twoRow = "two-row";
    // ```
    /** Two connected rows of two segments each. */
    TWO_ROW,

    // What:     `FOUR_ROW` is the connected vertical stack of four segments.
    // Why:      It is the final fallback when even two segments cannot share a row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fourRow = "four-row";
    // ```
    /** Four connected rows of one segment each. */
    FOUR_ROW,
}

// What:     `internal fun modeControlLayoutFor(...)` chooses the arrangement from width and overflow.
// Why:      The thresholds and overflow fallbacks are testable without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// function modeControlLayoutFor(widthDp: number, oneRowOverflowed: boolean,
//   twoRowOverflowed: boolean): ModeControlLayout { ... }
// ```
/**
 * Picks the fewest connected rows that fit, honoring text overflow reported by earlier layouts.
 *
 * @param widthDp available width of mode control
 * @param oneRowOverflowed true when one row previously clipped a label at this width
 * @param twoRowOverflowed true when two rows previously clipped a label at this width
 * @return arrangement to draw
 */
internal fun modeControlLayoutFor(
    widthDp: Dp,
    oneRowOverflowed: Boolean,
    twoRowOverflowed: Boolean,
): ModeControlLayout {
    if (widthDp >= ONE_ROW_MIN_WIDTH && !oneRowOverflowed) {
        return ModeControlLayout.ONE_ROW
    }
    if (widthDp >= TWO_ROW_MIN_WIDTH && !twoRowOverflowed) {
        return ModeControlLayout.TWO_ROW
    }
    return ModeControlLayout.FOUR_ROW
}

// What:     `internal fun deckControlsFitOneLine(...)` decides whether transport and mode share one row.
// Why:      The check is pure pixel arithmetic, so the exact boundary is testable without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// function deckControlsFitOneLine(availableWidthPx: number, transportWidthPx: number,
//   modeWidthPx: number, gapPx: number): boolean { ... }
// ```
/**
 * Reports whether the transport buttons, one gap, and the one-row mode control fit in the available width.
 *
 * @param availableWidthPx usable width of the controls group in pixels
 * @param transportWidthPx natural width of transport buttons in pixels
 * @param modeWidthPx natural width of one-row mode control in pixels
 * @param gapPx gap between the two groups in pixels, counted once
 * @return true when transport, gap, and mode together fit within availableWidthPx
 */
internal fun deckControlsFitOneLine(
    availableWidthPx: Int,
    transportWidthPx: Int,
    modeWidthPx: Int,
    gapPx: Int,
): Boolean {
    /** Sum in Long so extreme widths cannot wrap around and report a false fit. */
    val requiredWidthPx: Long = transportWidthPx.toLong() + gapPx.toLong() + modeWidthPx.toLong()
    return availableWidthPx.toLong() >= requiredWidthPx
}

// What:     `private val MODE_SEGMENT_ORDER: List<PlaybackMode>` fixes the left-to-right segment order.
// Why:      Segment position drives the connected corner shapes and the traversal order.
//
// In TS you'd write (pseudocode):
// ```ts
// const MODE_SEGMENT_ORDER: readonly PlaybackMode[] =
//   ["repeat", "in_order", "shuffle_page", "shuffle_all"];
// ```
/** Left-to-right order of the four mode segments. */
private val MODE_SEGMENT_ORDER: List<PlaybackMode> = listOf(
    PlaybackMode.REPEAT,
    PlaybackMode.IN_ORDER,
    PlaybackMode.SHUFFLE_PAGE,
    PlaybackMode.SHUFFLE_ALL,
)

// What:     `internal fun modeLabelFor(...)` maps one playback mode to its visible label.
// Why:      Visible labels are fixed except the folder name, which the caller supplies.
//
// In TS you'd write (pseudocode):
// ```ts
// function modeLabelFor(mode: PlaybackMode, folder: string): string { ... }
// ```
/** Returns the visible segment text for one playback mode. */
internal fun modeLabelFor(mode: PlaybackMode, shuffleFolderLabel: String): String = when (mode) {
    PlaybackMode.REPEAT -> "Repeat"
    PlaybackMode.IN_ORDER -> "In order"
    PlaybackMode.SHUFFLE_PAGE -> SHUFFLE_PREFIX + shuffleFolderLabel
    PlaybackMode.SHUFFLE_ALL -> "Shuffle all"
}

// What:     `internal fun modeAccessibleLabelFor(...)` maps one playback mode to its spoken label.
// Why:      Screen readers hear fuller phrases than the compact visible labels show.
//
// In TS you'd write (pseudocode):
// ```ts
// function modeAccessibleLabelFor(mode: PlaybackMode, folder: string): string { ... }
// ```
/** Returns the spoken description for one playback mode segment. */
internal fun modeAccessibleLabelFor(mode: PlaybackMode, shuffleFolderLabel: String): String =
    when (mode) {
        PlaybackMode.REPEAT -> "Repeat track"
        PlaybackMode.IN_ORDER -> "Play in order"
        PlaybackMode.SHUFFLE_PAGE -> SHUFFLE_PREFIX + shuffleFolderLabel
        PlaybackMode.SHUFFLE_ALL -> "Shuffle all folders"
    }

// What:     `internal fun deckClockLabel(...)` formats whole seconds as `m:ss`.
// Why:      The seek row shows elapsed and total time in one fixed, locale-independent form.
//
// In TS you'd write (pseudocode):
// ```ts
// function deckClockLabel(seconds: number): string {
//   const total = Math.max(0, Math.trunc(seconds));
//   return `${Math.trunc(total / 60)}:${String(total % 60).padStart(2, "0")}`;
// }
// ```
/** Formats a non-negative duration in seconds as minutes and two-digit seconds. */
internal fun deckClockLabel(seconds: Double): String {
    /** Whole seconds with negatives and NaN clamped to zero. */
    val total: Long = seconds.toLong().coerceAtLeast(0L)
    /** Whole minutes in the duration. */
    val minutes: Long = total / SECONDS_PER_MINUTE
    /** Seconds left after whole minutes are removed. */
    val remainder: Long = total % SECONDS_PER_MINUTE
    return "$minutes:${remainder.toString().padStart(2, '0')}"
}

// What:     `internal fun progressFractionFor(...)` converts a position into a slider fraction.
// Why:      The slider needs a value in 0 to 1 even when the duration is zero or unknown.
//
// In TS you'd write (pseudocode):
// ```ts
// function progressFractionFor(positionSec: number, durationSec: number): number {
//   if (!(durationSec > 0)) return 0;
//   return Math.min(1, Math.max(0, positionSec / durationSec));
// }
// ```
/** Returns the playhead as a fraction of the track in the closed range zero to one. */
internal fun progressFractionFor(positionSec: Double, durationSec: Double): Float {
    if (durationSec <= 0.0) {
        return 0f
    }
    /** Raw position divided by duration before clamping. */
    val fraction: Float = (positionSec / durationSec).toFloat()
    if (fraction.isNaN()) {
        return 0f
    }
    return fraction.coerceIn(0f, 1f)
}

// What:     `private fun deckIcon(...)` builds one 24dp glyph from a single path string.
// Why:      The check, play, pause, and skip glyphs need no extended icon artifact in production.
//
// In TS you'd write (pseudocode):
// ```ts
// function deckIcon(name: string, pathData: string): ImageVector { ... }
// ```
/** Builds one filled glyph from SVG path data on the shared 24 by 24 grid. */
private fun deckIcon(name: String, pathData: String): ImageVector = ImageVector.Builder(
    name = name,
    defaultWidth = ICON_SIZE,
    defaultHeight = ICON_SIZE,
    viewportWidth = ICON_VIEWPORT,
    viewportHeight = ICON_VIEWPORT,
).addPath(
    pathData = addPathNodes(pathData),
    fill = SolidColor(Color.Black),
).build()

// What:     `private val CHECK_ICON: ImageVector` is the check mark on the selected segment.
// Why:      Selection needs a second visual channel besides the container fill.
//
// In TS you'd write (pseudocode):
// ```ts
// const CHECK_ICON = deckIcon("Check", CHECK_PATH);
// ```
/** Check mark drawn on the selected mode segment. */
private val CHECK_ICON: ImageVector = deckIcon(name = "Deck.Check", pathData = CHECK_PATH)

// What:     `private val PLAY_ICON: ImageVector` is the play glyph.
// Why:      The center button shows play while playback is paused.
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAY_ICON = deckIcon("PlayArrow", PLAY_PATH);
// ```
/** Play glyph shown on the center button while paused. */
private val PLAY_ICON: ImageVector = deckIcon(name = "Deck.PlayArrow", pathData = PLAY_PATH)

// What:     `private val PAUSE_ICON: ImageVector` is the pause glyph.
// Why:      The center button shows pause while playback is running.
//
// In TS you'd write (pseudocode):
// ```ts
// const PAUSE_ICON = deckIcon("Pause", PAUSE_PATH);
// ```
/** Pause glyph shown on the center button while playing. */
private val PAUSE_ICON: ImageVector = deckIcon(name = "Deck.Pause", pathData = PAUSE_PATH)

// What:     `private val SKIP_PREVIOUS_ICON: ImageVector` is the previous-track glyph.
// Why:      The previous button identifies the backward transport action.
//
// In TS you'd write (pseudocode):
// ```ts
// const SKIP_PREVIOUS_ICON = deckIcon("SkipPrevious", SKIP_PREVIOUS_PATH);
// ```
/** Previous-track glyph on the left transport button. */
private val SKIP_PREVIOUS_ICON: ImageVector =
    deckIcon(name = "Deck.SkipPrevious", pathData = SKIP_PREVIOUS_PATH)

// What:     `private val SKIP_NEXT_ICON: ImageVector` is the next-track glyph.
// Why:      The next button identifies the forward transport action.
//
// In TS you'd write (pseudocode):
// ```ts
// const SKIP_NEXT_ICON = deckIcon("SkipNext", SKIP_NEXT_PATH);
// ```
/** Next-track glyph on the right transport button. */
private val SKIP_NEXT_ICON: ImageVector = deckIcon(name = "Deck.SkipNext", pathData = SKIP_NEXT_PATH)

// What:     `internal fun deckContainerColorOf(roles)` picks the deck ground from the derived roles.
// Why:      The deck is a gray, and every gray carries the accent's hue unless the root is completely neutral
//           (D118), so both appearances read the derived `deck` role instead of a fixed literal.
//
// In TS you'd write (pseudocode):
// ```ts
// const deckContainerColorOf = (roles: RoleSet): Color => toColor(roles.deck);
// ```
/** Returns the deck background of the derived [roles]: a gray with the accent's hue, in both appearances (D118). */
internal fun deckContainerColorOf(roles: RoleSet): Color = roles.deck.toComposeColor()

/** Resolves the deck background from the derived roles of the appearance in use. */
@Composable
private fun deckContainerColor(): Color = deckContainerColorOf(LocalThemeRoles.current)

/**
 * Draws the centered folder and title line above the subtitle, with each tappable part as its own button.
 *
 * Each part reserves the same number of lines for every name (one at ordinary text sizes, two from 150 percent), so
 * the deck keeps its height while tracks change. The folder part sits against the separator and the title part starts
 * at it, so the 48 dp minimum cells put their spare width on the outer sides and the separator never floats in a gap.
 */
@Composable
private fun deckHeading(
    folder: String,
    title: String,
    subtitle: String,
    onRevealFolder: (() -> Unit)?,
    onRevealPlaying: (() -> Unit)?,
) {
    /** Lines each heading part reserves, from the text size. */
    val lines: Int = headingLineBudget(LocalDensity.current.fontScale)
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
            /** Widest the folder part may be: the fixed share of the heading width, and half of it at large text. */
            val folderMaxWidth: Dp = maxWidth * if (lines > 1) LARGE_TEXT_FOLDER_FRACTION else FOLDER_MAX_WIDTH_FRACTION
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Center,
            ) {
                if (folder.isNotEmpty()) {
                    deckHeadingPart(
                        text = folder,
                        style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Medium),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        onClick = onRevealFolder,
                        clickLabel = "Show folder of playing track",
                        modifier = Modifier.widthIn(max = folderMaxWidth),
                        lines = lines,
                        alignment = Alignment.CenterEnd,
                        textAlign = TextAlign.End,
                    )
                    Text(
                        text = "/",
                        modifier = Modifier.padding(horizontal = HEADING_SEPARATOR_PADDING),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                    )
                }
                deckHeadingPart(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = Color.Unspecified,
                    // An idle deck has no title, so its part must not be an unlabeled button (M5).
                    onClick = if (title.isEmpty()) null else onRevealPlaying,
                    clickLabel = "Show playing track",
                    modifier = Modifier.weight(1f, fill = false),
                    lines = titleLineBudget(LocalDensity.current.fontScale),
                    alignment = if (folder.isNotEmpty()) Alignment.CenterStart else Alignment.Center,
                    textAlign = if (folder.isNotEmpty()) TextAlign.Start else TextAlign.Center,
                )
            }
        }
        Text(
            text = subtitle,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}

// What:     `@Composable private fun deckHeadingPart(...)` draws one part of the heading in its own cell.
// Why:      The folder and the title share the same button rules, so one helper draws both. A part with a tap action
//           is a cell with a 48dp minimum width and height and the Button role; the text inside it is aligned toward
//           the separator, so the cell's spare width is padding on the outer side (HZP). A part without one is plain
//           text with no minimum size.
//
// In TS you'd write (pseudocode):
// ```ts
// function deckHeadingPart(props: { text: string; style: TextStyle; color: Color; onClick?: () => void;
//   clickLabel: string; lines: number; alignment: Alignment; textAlign: TextAlign }): UIElement;
// ```
/** Draws one heading part within a fixed number of lines, tappable with the given label when [onClick] is given. */
@Composable
private fun deckHeadingPart(
    text: String,
    style: TextStyle,
    color: Color,
    onClick: (() -> Unit)?,
    clickLabel: String,
    modifier: Modifier,
    lines: Int,
    alignment: Alignment,
    textAlign: TextAlign,
) {
    /** Part modifier: a button with a 48dp minimum width and height when tappable, unchanged otherwise. */
    val partModifier: Modifier = if (onClick == null) {
        modifier
    } else {
        modifier
            .defaultMinSize(minWidth = MIN_TARGET, minHeight = MIN_TARGET)
            .focusRing(RectangleShape)
            .clickable(onClickLabel = clickLabel, role = Role.Button, onClick = onClick)
    }
    Box(modifier = partModifier, contentAlignment = alignment) {
        Text(
            text = text,
            modifier = if (onClick == null) Modifier else Modifier.padding(horizontal = HEADING_PART_TEXT_PADDING),
            color = color,
            style = style,
            textAlign = textAlign,
            maxLines = lines,
            minLines = lines,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

// What:     `@Composable private fun seekRow(...)` draws elapsed time, a slider, and total time.
// Why:      The slider reports a fraction that becomes an absolute position in seconds.
//
// In TS you'd write (pseudocode):
// ```ts
// function seekRow(props: { positionSec: number; durationSec: number;
//   onSeek(sec: number): void }): UIElement;
// ```
/** Draws the seek row with the slider converting its fraction into seconds. */
@Composable
private fun seekRow(positionSec: Double, durationSec: Double, onSeek: (Double) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(CONTROL_SPACING),
    ) {
        /** Tabular-figure style so the clock digits do not shift width while playing. */
        val timeStyle = MaterialTheme.typography.labelMedium.copy(fontFeatureSettings = "tnum")
        Text(
            text = deckClockLabel(positionSec),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = timeStyle,
        )
        /** Slider state whose thumb is re-pinned to the playback fraction on every recomposition. */
        val sliderState = rememberSliderState()
        sliderState.value = progressFractionFor(positionSec, durationSec)
        /** Source the slider and its thumb share, so the thumb can show the focus ring (D117). */
        val sliderSource = remember { MutableInteractionSource() }
        Slider(
            state = sliderState,
            onValueChange = { fraction -> onSeek(fraction.toDouble() * durationSec) },
            modifier = Modifier
                .weight(1f)
                .semantics {
                    contentDescription = "Track position"
                },
            interactionSource = sliderSource,
            thumb = {
                SliderDefaults.Thumb(
                    interactionSource = sliderSource,
                    modifier = Modifier.focusRingOf(sliderSource, CircleShape),
                )
            },
        )
        Text(
            text = deckClockLabel(durationSec),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = timeStyle,
        )
    }
}

// What:     `@Composable private fun secondaryTransportButton(...)` draws one outlined skip button.
// Why:      Skip actions use the accepted outlined treatment, which carries less emphasis than play.
//
// In TS you'd write (pseudocode):
// ```ts
// function secondaryTransportButton(props: { icon: ImageVector; label: string;
//   onClick(): void }): UIElement;
// ```
/** Draws one outlined icon button for a skip action. */
@Composable
private fun secondaryTransportButton(
    imageVector: ImageVector,
    contentDescription: String,
    onClick: () -> Unit,
) {
    OutlinedIconButton(onClick = onClick, modifier = Modifier.size(MIN_TARGET).focusRing(CircleShape)) {
        Icon(imageVector = imageVector, contentDescription = contentDescription)
    }
}

// What:     `@Composable private fun transportControls(...)` draws previous, play or pause, and next.
// Why:      The filled center button carries the strongest emphasis of the three transport actions.
//
// In TS you'd write (pseudocode):
// ```ts
// function transportControls(props: { playing: boolean; onPrevious(): void;
//   onTogglePlay(): void; onNext(): void }): UIElement;
// ```
/**
 * Draws the three transport buttons centered in one row, always laid out left to right.
 *
 * @param playing true while playback runs, which selects the pause glyph
 * @param onPrevious invoked by previous-track button
 * @param onTogglePlay invoked by center play or pause button
 * @param onNext invoked by next-track button
 * @param modifier outer modifier of row, so callers choose its width
 */
@Composable
private fun transportControls(
    playing: Boolean,
    onPrevious: () -> Unit,
    onTogglePlay: () -> Unit,
    onNext: () -> Unit,
    modifier: Modifier,
) {
    CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
        Row(
            modifier = modifier,
            horizontalArrangement = Arrangement.spacedBy(CONTROL_SPACING, Alignment.CenterHorizontally),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            secondaryTransportButton(
                imageVector = SKIP_PREVIOUS_ICON,
                contentDescription = "Previous track",
                onClick = onPrevious,
            )
            FilledIconButton(onClick = onTogglePlay, modifier = Modifier.size(MIN_TARGET).focusRing(CircleShape)) {
                Icon(
                    imageVector = if (playing) PAUSE_ICON else PLAY_ICON,
                    contentDescription = if (playing) "Pause" else "Play",
                )
            }
            secondaryTransportButton(
                imageVector = SKIP_NEXT_ICON,
                contentDescription = "Next track",
                onClick = onNext,
            )
        }
    }
}

// What:     `@Composable private fun shuffleModeLabel(...)` draws `Shuffle ` and a capped folder name.
// Why:      Long names keep the verb and useful characters without widening the segment beyond
//           the reference name.
//
// In TS you'd write (pseudocode):
// ```ts
// function shuffleModeLabel(props: { folderName: string; onUnexpectedOverflow(): void }): UIElement;
// ```
/**
 * Draws the shuffle-folder label, truncating the folder name in the middle when it is too wide.
 *
 * The label always takes the width of the prefix plus the reference name, whatever the folder is called, and centers
 * its content in that slot, so the mode control and the deck keep one shape while the user browses folders.
 */
@Composable
private fun shuffleModeLabel(folderName: String, onUnexpectedOverflow: () -> Unit) {
    /** Label text style shared by the prefix and the folder name. */
    val textStyle = MaterialTheme.typography.labelLarge
    /** Text measurer retained across redraws so measurement does not allocate per frame. */
    val textMeasurer = rememberTextMeasurer()
    /** Density used to convert measured pixel widths to Dp. */
    val density = LocalDensity.current
    /** Pixel width of the reference name, which caps the folder text. */
    val maximumFolderWidthPixels: Int = remember(textMeasurer, textStyle) {
        textMeasurer.measure(text = SHUFFLE_WIDTH_REFERENCE, style = textStyle).size.width
    }
    /** Pixel width of the full folder name before any cap. */
    val naturalFolderWidthPixels: Int = remember(folderName, textMeasurer, textStyle) {
        textMeasurer.measure(text = folderName, style = textStyle).size.width
    }
    /** Folder text width in pixels after applying the cap. */
    val desiredFolderWidthPixels: Int = shuffleFolderTextWidthPx(
        naturalPx = naturalFolderWidthPixels,
        referencePx = maximumFolderWidthPixels,
    )
    /** Folder text width converted to Dp for the layout modifier. */
    val desiredFolderWidth: Dp = with(density) {
        desiredFolderWidthPixels.toDp()
    }
    /** Pixel width of the prefix, which the fixed slot adds to the reference name's width. */
    val prefixWidthPixels: Int = remember(textMeasurer, textStyle) {
        textMeasurer.measure(text = SHUFFLE_PREFIX, style = textStyle).size.width
    }
    /** Width of the whole label slot, the same for every folder name. */
    val slotWidth: Dp = with(density) {
        shuffleLabelSlotWidthPx(prefixPx = prefixWidthPixels, referencePx = maximumFolderWidthPixels).toDp()
    }
    Row(
        modifier = Modifier.width(slotWidth).clearAndSetSemantics {},
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = SHUFFLE_PREFIX,
            maxLines = 1,
            overflow = TextOverflow.Clip,
            style = textStyle,
            onTextLayout = { result ->
                if (result.hasVisualOverflow) {
                    onUnexpectedOverflow()
                }
            },
        )
        Text(
            text = folderName,
            modifier = Modifier.width(desiredFolderWidth),
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
            style = textStyle,
            onTextLayout = { result ->
                if (result.size.width < desiredFolderWidthPixels) {
                    onUnexpectedOverflow()
                }
            },
        )
    }
}

// What:     `@Composable private fun variableWidthModeSegment(...)` draws one connected segment.
// Why:      Each segment sizes to its own label, so unequal labels can share one row.
//
// In TS you'd write (pseudocode):
// ```ts
// function variableWidthModeSegment(props: { mode: PlaybackMode; selected: PlaybackMode;
//   shuffleFolderLabel: string; shape: Shape; onSelect(m: PlaybackMode): void;
//   onOverflow(): void }): UIElement;
// ```
/** Draws one radio-style mode segment with a 48dp minimum target. */
@Composable
private fun variableWidthModeSegment(
    mode: PlaybackMode,
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    shape: Shape,
    modifier: Modifier,
    contentPadding: PaddingValues,
    onSelectMode: (PlaybackMode) -> Unit,
    onOverflow: () -> Unit,
) {
    /** True when this segment is the playback mode in effect. */
    val isSelected: Boolean = mode == selectedMode
    /** Selected segments take the secondary container fill, others stay transparent. */
    val containerColor: Color = if (isSelected) {
        MaterialTheme.colorScheme.secondaryContainer
    } else {
        Color.Transparent
    }
    /** Text color paired with the container fill. */
    val contentColor: Color = if (isSelected) {
        MaterialTheme.colorScheme.onSecondaryContainer
    } else {
        MaterialTheme.colorScheme.onSurface
    }
    OutlinedButton(
        onClick = { onSelectMode(mode) },
        shape = shape,
        modifier = modifier
            .defaultMinSize(minWidth = MIN_TARGET, minHeight = MIN_TARGET)
            .focusRing(shape)
            .semantics {
                contentDescription = modeAccessibleLabelFor(mode, shuffleFolderLabel)
                role = Role.RadioButton
                selected = isSelected
                traversalIndex = MODE_SEGMENT_ORDER.indexOf(mode).toFloat()
            },
        colors = ButtonDefaults.buttonColors(
            containerColor = containerColor,
            contentColor = contentColor,
        ),
        border = BorderStroke(
            width = SegmentedButtonDefaults.BorderWidth,
            color = MaterialTheme.colorScheme.outline,
        ),
        contentPadding = contentPadding,
    ) {
        if (isSelected) {
            Icon(
                imageVector = CHECK_ICON,
                contentDescription = null,
                modifier = Modifier
                    .padding(end = SEGMENT_ICON_END_SPACING)
                    .size(SegmentedButtonDefaults.IconSize),
            )
        }
        if (mode == PlaybackMode.SHUFFLE_PAGE) {
            shuffleModeLabel(
                folderName = shuffleFolderLabel,
                onUnexpectedOverflow = onOverflow,
            )
        } else {
            Text(
                text = modeLabelFor(mode, shuffleFolderLabel),
                modifier = Modifier.clearAndSetSemantics {},
                maxLines = 1,
                overflow = TextOverflow.Clip,
                onTextLayout = { result ->
                    if (result.hasVisualOverflow) {
                        onOverflow()
                    }
                },
            )
        }
    }
}

// What:     `@Composable private fun oneRowModeControl(...)` lays four segments in one row.
// Why:      Intrinsic widths keep one row as the first fitting arrangement.
//
// In TS you'd write (pseudocode):
// ```ts
// function oneRowModeControl(props: ModeControlProps): UIElement;
// ```
/** Draws the four mode segments as one connected horizontal group. */
@Composable
private fun oneRowModeControl(
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    contentPadding: PaddingValues,
    onSelectMode: (PlaybackMode) -> Unit,
    onOverflow: () -> Unit,
) {
    Row(
        modifier = Modifier.selectableGroup(),
        horizontalArrangement = Arrangement.spacedBy(SEGMENT_OVERLAP),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        MODE_SEGMENT_ORDER.forEachIndexed { index, mode ->
            variableWidthModeSegment(
                mode = mode,
                selectedMode = selectedMode,
                shuffleFolderLabel = shuffleFolderLabel,
                shape = SegmentedButtonDefaults.itemShape(index = index, count = MODE_SEGMENT_ORDER.size),
                modifier = Modifier,
                contentPadding = contentPadding,
                onSelectMode = onSelectMode,
                onOverflow = onOverflow,
            )
        }
    }
}

// What:     `@Composable private fun twoRowModeControl(...)` lays four segments in a 2 by 2 block.
// Why:      Enlarged labels fit two per row before the control falls back to four rows.
//
// In TS you'd write (pseudocode):
// ```ts
// function twoRowModeControl(props: ModeControlProps): UIElement;
// ```
/**
 * Draws the four mode segments as a connected two-by-two block.
 *
 * Each row is a plain row of two equal-weight segments, so a label too wide for half the width is clipped and
 * reported through [onOverflow], which moves the control to four rows. A wrapping flow would instead push the wide
 * shuffle segment onto a row of its own and leave a ragged block that nothing reports.
 */
@Composable
private fun twoRowModeControl(
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    contentPadding: PaddingValues,
    onSelectMode: (PlaybackMode) -> Unit,
    onOverflow: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .selectableGroup(),
        verticalArrangement = Arrangement.spacedBy(SEGMENT_OVERLAP),
    ) {
        for (rowStart in MODE_SEGMENT_ORDER.indices step 2) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(SEGMENT_OVERLAP),
            ) {
                for (index in rowStart..rowStart + 1) {
                    /** Outside corner shape for this grid position, with internal corners square. */
                    val shape: Shape = when (index) {
                        0 -> RoundedCornerShape(topStart = SEGMENT_OUTER_RADIUS)
                        1 -> RoundedCornerShape(topEnd = SEGMENT_OUTER_RADIUS)
                        2 -> RoundedCornerShape(bottomStart = SEGMENT_OUTER_RADIUS)
                        else -> RoundedCornerShape(bottomEnd = SEGMENT_OUTER_RADIUS)
                    }
                    variableWidthModeSegment(
                        mode = MODE_SEGMENT_ORDER[index],
                        selectedMode = selectedMode,
                        shuffleFolderLabel = shuffleFolderLabel,
                        shape = shape,
                        modifier = Modifier.weight(1f),
                        contentPadding = contentPadding,
                        onSelectMode = onSelectMode,
                        onOverflow = onOverflow,
                    )
                }
            }
        }
    }
}

// What:     `@Composable private fun fourRowModeControl(...)` stacks four segments vertically.
// Why:      This final fallback keeps full labels when even two segments cannot share a row.
//
// In TS you'd write (pseudocode):
// ```ts
// function fourRowModeControl(props: ModeControlProps): UIElement;
// ```
/** Draws the four mode segments as one connected vertical stack. */
@Composable
private fun fourRowModeControl(
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    contentPadding: PaddingValues,
    onSelectMode: (PlaybackMode) -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .selectableGroup(),
        verticalArrangement = Arrangement.spacedBy(SEGMENT_OVERLAP),
    ) {
        MODE_SEGMENT_ORDER.forEachIndexed { index, mode ->
            /** Only the top and bottom stack positions round their outside corners. */
            val shape: Shape = when (index) {
                0 -> RoundedCornerShape(topStart = SEGMENT_OUTER_RADIUS, topEnd = SEGMENT_OUTER_RADIUS)
                MODE_SEGMENT_ORDER.lastIndex -> RoundedCornerShape(
                    bottomStart = SEGMENT_OUTER_RADIUS,
                    bottomEnd = SEGMENT_OUTER_RADIUS,
                )
                else -> RectangleShape
            }
            variableWidthModeSegment(
                mode = mode,
                selectedMode = selectedMode,
                shuffleFolderLabel = shuffleFolderLabel,
                shape = shape,
                modifier = Modifier.fillMaxWidth(),
                contentPadding = contentPadding,
                onSelectMode = onSelectMode,
                onOverflow = {},
            )
        }
    }
}

// What:     `@Composable private fun modeControl(...)` measures width and picks one of three layouts.
// Why:      The control uses the fewest rows that fit, and text that clips at one width moves the
//           control to the next arrangement.
//
// In TS you'd write (pseudocode):
// ```ts
// function modeControl(props: ModeControlProps & { modifier: Modifier }): UIElement;
// ```
/** Chooses the mode-control arrangement from the available width and recorded text overflow. */
@Composable
private fun modeControl(
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    onSelectMode: (PlaybackMode) -> Unit,
    modifier: Modifier,
) {
    /** Density whose font scale resets the overflow flags when the user changes text size. */
    val density = LocalDensity.current
    /** Standard padding that keeps each segment at its minimum horizontal content width. */
    val contentPadding: PaddingValues = SegmentedButtonDefaults.ContentPadding
    BoxWithConstraints(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        /** True when one row clipped a label at this width and font scale. */
        val oneRowOverflow = remember(maxWidth, density.fontScale) { mutableStateOf(false) }
        /** True when two rows clipped a label at this width and font scale. */
        val twoRowOverflow = remember(maxWidth, density.fontScale) { mutableStateOf(false) }
        /** Arrangement chosen for the current width and recorded overflow. */
        val layout: ModeControlLayout = modeControlLayoutFor(
            widthDp = maxWidth,
            oneRowOverflowed = oneRowOverflow.value,
            twoRowOverflowed = twoRowOverflow.value,
        )
        when (layout) {
            ModeControlLayout.ONE_ROW -> oneRowModeControl(
                selectedMode = selectedMode,
                shuffleFolderLabel = shuffleFolderLabel,
                contentPadding = contentPadding,
                onSelectMode = onSelectMode,
                onOverflow = { oneRowOverflow.value = true },
            )
            ModeControlLayout.TWO_ROW -> twoRowModeControl(
                selectedMode = selectedMode,
                shuffleFolderLabel = shuffleFolderLabel,
                contentPadding = contentPadding,
                onSelectMode = onSelectMode,
                onOverflow = { twoRowOverflow.value = true },
            )
            ModeControlLayout.FOUR_ROW -> fourRowModeControl(
                selectedMode = selectedMode,
                shuffleFolderLabel = shuffleFolderLabel,
                contentPadding = contentPadding,
                onSelectMode = onSelectMode,
            )
        }
    }
}

// What:     `private enum class DeckControlsSlot` names the subcomposition slots of the transport and mode group.
// Why:      Named slots let the group measure its probes and compose only the arrangement it chose.
//
// In TS you'd write (pseudocode):
// ```ts
// type DeckControlsSlot = "transportProbe" | "modeProbe" | "oneLine" | "stacked";
// ```
/** Subcomposition slots used by the transport and mode group. */
private enum class DeckControlsSlot {
    // What:     `TRANSPORT_PROBE` is the hidden measurement of the transport buttons.
    // Why:      Its natural width is needed before the arrangement is chosen, and it is never placed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const transportProbe = "transportProbe";
    // ```
    /** Hidden measurement of the transport buttons at their natural width. */
    TRANSPORT_PROBE,

    // What:     `MODE_PROBE` is the hidden measurement of the one-row mode control.
    // Why:      Its natural width decides whether both groups fit side by side, and it is never placed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const modeProbe = "modeProbe";
    // ```
    /** Hidden measurement of the one-row mode control at its natural width. */
    MODE_PROBE,

    // What:     `ONE_LINE` is the single row that holds the transport buttons, the gap, and the mode control.
    // Why:      It is drawn only when the measured groups fit side by side at the current width and font scale.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const oneLine = "oneLine";
    // ```
    /** Transport buttons and the mode control side by side in one centered row. */
    ONE_LINE,

    // What:     `STACKED` is the column that holds the transport buttons and then the mode control.
    // Why:      It is the arrangement the deck has always drawn, and it is the fallback when the row does not fit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stacked = "stacked";
    // ```
    /** Transport buttons and the mode control in one column, each on its own line. */
    STACKED,
}

// What:     `@Composable private fun deckControlsGroup(...)` draws transport and mode in one row or in a column.
// Why:      Only a measurement made before composition knows whether both groups fit side by side at this width
//           and font scale, so the group measures natural widths first and then composes the chosen arrangement.
//
// In TS you'd write (pseudocode):
// ```ts
// function deckControlsGroup(props: DeckControlsProps): UIElement {
//   const transportWidth = measureNatural(transportControls);
//   const modeWidth = measureNatural(oneRowModeControl);
//   return deckControlsFitOneLine(availableWidth(), transportWidth, modeWidth, GAP)
//     ? oneLine(props) : stacked(props);
// }
// ```
/** Draws transport and mode side by side when both fit the width, and in a column otherwise. */
@Composable
private fun deckControlsGroup(
    playing: Boolean,
    selectedMode: PlaybackMode,
    shuffleFolderLabel: String,
    actions: TransportDeckActions,
) {
    /** Density whose font scale resets the one-line overflow flag when the user changes text size. */
    val density = LocalDensity.current
    /** Standard padding that keeps each segment at its minimum horizontal content width. */
    val contentPadding: PaddingValues = SegmentedButtonDefaults.ContentPadding
    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        /** True when the one-line row clipped a label at this width and font scale, which forces the column. */
        val oneLineOverflow = remember(maxWidth, density.fontScale) { mutableStateOf(false) }
        SubcomposeLayout(modifier = Modifier.fillMaxWidth()) { constraints ->
            /** Natural width of the transport buttons in pixels, measured without a width cap. */
            val transportWidthPx: Int = subcompose(DeckControlsSlot.TRANSPORT_PROBE) {
                transportControls(
                    playing = playing,
                    onPrevious = actions.onPrevious,
                    onTogglePlay = actions.onTogglePlay,
                    onNext = actions.onNext,
                    modifier = Modifier,
                )
            }.first().measure(Constraints()).width
            /** Natural width of the one-row mode control in pixels, measured without a width cap. */
            val modeWidthPx: Int = subcompose(DeckControlsSlot.MODE_PROBE) {
                oneRowModeControl(
                    selectedMode = selectedMode,
                    shuffleFolderLabel = shuffleFolderLabel,
                    contentPadding = contentPadding,
                    onSelectMode = actions.onSelectMode,
                    onOverflow = {},
                )
            }.first().measure(Constraints()).width
            /** Gap between the two groups converted to pixels for the width check. */
            val gapPx: Int = ONE_LINE_CONTROL_GAP.roundToPx()
            /** True when both groups fit side by side and the one-line row has not clipped at this width. */
            val oneLine: Boolean = !oneLineOverflow.value && deckControlsFitOneLine(
                availableWidthPx = constraints.maxWidth,
                transportWidthPx = transportWidthPx,
                modeWidthPx = modeWidthPx,
                gapPx = gapPx,
            )
            /** Arrangement drawn for this pass, measured to the available width. */
            val placed = subcompose(
                if (oneLine) DeckControlsSlot.ONE_LINE else DeckControlsSlot.STACKED,
            ) {
                if (oneLine) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(
                            ONE_LINE_CONTROL_GAP,
                            Alignment.CenterHorizontally,
                        ),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        transportControls(
                            playing = playing,
                            onPrevious = actions.onPrevious,
                            onTogglePlay = actions.onTogglePlay,
                            onNext = actions.onNext,
                            modifier = Modifier,
                        )
                        oneRowModeControl(
                            selectedMode = selectedMode,
                            shuffleFolderLabel = shuffleFolderLabel,
                            contentPadding = contentPadding,
                            onSelectMode = actions.onSelectMode,
                            onOverflow = { oneLineOverflow.value = true },
                        )
                    }
                } else {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(STACKED_CONTROL_SPACING),
                    ) {
                        transportControls(
                            playing = playing,
                            onPrevious = actions.onPrevious,
                            onTogglePlay = actions.onTogglePlay,
                            onNext = actions.onNext,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        modeControl(
                            selectedMode = selectedMode,
                            shuffleFolderLabel = shuffleFolderLabel,
                            onSelectMode = actions.onSelectMode,
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
            }.first().measure(Constraints(maxWidth = constraints.maxWidth))
            layout(width = placed.width, height = placed.height) {
                placed.place(0, 0)
            }
        }
    }
}

// What:     `fun transportDeck(...)` draws the playback deck from a state snapshot and callbacks.
// Why:      The caller owns playback, so the deck stays stateless and previewable.
//
// In TS you'd write (pseudocode):
// ```ts
// function transportDeck(model: TransportDeckModel, actions: TransportDeckActions,
//   modifier?: Modifier): UIElement;
// ```
/**
 * Draws the playback deck: heading, seek row, transport buttons, and playback mode control.
 *
 * The deck takes its content height and never caps it, so the caller sets any height limit
 * through `modifier`.
 *
 * @param model playback snapshot to display
 * @param actions callbacks invoked on user actions
 * @param modifier outer modifier applied before background and padding
 */
@Composable
fun transportDeck(
    model: TransportDeckModel,
    actions: TransportDeckActions,
    modifier: Modifier = Modifier,
) {
    /** Deck background resolved from the derived roles of the current appearance. */
    val containerColor: Color = deckContainerColor()
    Column(
        modifier = modifier
            .background(color = containerColor)
            .windowInsetsPadding(WindowInsets.systemGestures.only(WindowInsetsSides.Horizontal))
            .windowInsetsPadding(WindowInsets.navigationBars)
            .verticalScroll(rememberScrollState())
            .padding(vertical = DECK_VERTICAL_PADDING)
            // Content keeps its own gutter and clears the crease on both sides, so it stays centered on the pane's
            // axis (D18) while the background above still spans the whole pane.
            .padding(
                horizontal = maxOf(
                    paneStartInset(DECK_HORIZONTAL_PADDING),
                    paneEndInset(DECK_HORIZONTAL_PADDING),
                ),
            ),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(DECK_GROUP_SPACING),
    ) {
        deckHeading(
            folder = model.folderLabel,
            title = model.title,
            subtitle = model.subtitle,
            onRevealFolder = actions.onRevealFolder,
            onRevealPlaying = actions.onRevealPlaying,
        )
        CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
            seekRow(
                positionSec = model.positionSec,
                durationSec = model.durationSec,
                onSeek = actions.onSeek,
            )
        }
        deckControlsGroup(
            playing = model.playing,
            selectedMode = model.mode,
            shuffleFolderLabel = model.shuffleFolderLabel,
            actions = actions,
        )
    }
}
