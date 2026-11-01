// Production player screen: the folded cover layout below 600dp and the unfolded two-pane layout above it.
// The composable is stateless; the caller owns the picker state and the controller actions.

// What:     `package dev.monochromatic.musicplayer` places the screen beside the other player composables.
// Why:      The screen composes the top bars, the picker, the track list, and the deck, which share this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import androidx.compose.foundation.background` brings in the modifier that paints a canvas color.
// Why:      The screen paints a true-black canvas in dark mode and a flat canvas in light mode.
//
// In TS you'd write (pseudocode):
// ```ts
// import { background } from "compose/foundation";
// ```
// What:     `import androidx.activity.compose.BackHandler` intercepts system Back while a handler is enabled.
// Why:      Back on the folded cover closes an open folder picker before it leaves the screen.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BackHandler } from "androidx/activity/compose";
// ```
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background

// What:     `import androidx.compose.foundation.isSystemInDarkTheme` reads the system appearance.
// Why:      The canvas and the seams follow the dark or light theme chosen by the system.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isSystemInDarkTheme } from "compose/foundation";
// ```
import androidx.compose.foundation.isSystemInDarkTheme

// What:     `import androidx.compose.foundation.layout.BoxWithConstraints` measures the available width.
// Why:      The layout choice depends on the width the screen was given, not on the device model.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BoxWithConstraints } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints

// What:     `import androidx.compose.foundation.layout.Column` stacks the cover screen vertically.
// Why:      The folded cover is a single column of top row, content, seam, and deck.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Column } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Column

// What:     `import androidx.compose.foundation.layout.Row` places the two unfolded panes side by side.
// Why:      The unfolded screen is one row of two equal-weight columns.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Row } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Row

// What:     `import androidx.compose.foundation.layout.fillMaxHeight` and `fillMaxSize` set the screen extent.
// Why:      Each pane and the whole screen fill the space the parent gives them.
//
// In TS you'd write (pseudocode):
// ```ts
// import { fillMaxHeight, fillMaxSize } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth

// What:     `import androidx.compose.foundation.layout.height` sets the seam thickness.
// Why:      The seam is a one-density-pixel line between the content and the deck.
//
// In TS you'd write (pseudocode):
// ```ts
// import { height } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.height

// What:     `import androidx.compose.material3.MaterialTheme` reads the theme colors for the light canvas and seams.
// Why:      The light canvas and the hairlines come from documented Material roles.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MaterialTheme } from "material3";
// ```
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme

// What:     `import androidx.compose.runtime.Composable` marks functions that emit UI.
// Why:      Every function in this file that draws UI is annotated.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Composable } from "compose/runtime";
// ```
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider

// What:     `import androidx.compose.ui.Modifier` is the chain that sizes, paints, and places each pane.
// Why:      Every composable here takes a modifier, and the panes set their size and background through it.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Modifier } from "compose/ui";
// ```
import androidx.compose.ui.Modifier

// What:     `import androidx.compose.ui.graphics.Color` names the canvas and seam colors.
// Why:      The true-black canvas is a fixed color, while the light canvas and seams are theme roles.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Color } from "compose/ui/graphics";
// ```
import androidx.compose.ui.graphics.Color

// What:     `import androidx.compose.ui.unit.Dp` and `dp` give the seam its thickness.
// Why:      The seam thickness is a density-independent size.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Dp } from "compose/ui/unit";
// ```
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `import dev.monochromatic.musicplayer.core.PlayerContentModel` brings in the screen model.
// Why:      The screen draws the model the builder produced, so it names the type directly.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { PlayerContentModel } from "dev.monochromatic.musicplayer/core/PlayerContentModel";
// ```
import androidx.compose.ui.platform.LocalDensity
import dev.monochromatic.musicplayer.core.PlayerContentModel
import dev.monochromatic.musicplayer.core.leftPaneClearance
import dev.monochromatic.musicplayer.core.rightPaneClearance

// What:     `private const val COVER_LAYOUT_MAX_WIDTH_DP: Float = 600f` is the width below which the
//           cover layout draws.
// Why:      A width under 600dp is a folded or phone-width screen, and 600dp and wider uses the unfolded panes.
//
// In TS you'd write (pseudocode):
// ```ts
// const COVER_LAYOUT_MAX_WIDTH_DP = 600;
// ```
/** Width in density-independent pixels below which the folded cover layout is drawn. */
private const val COVER_LAYOUT_MAX_WIDTH_DP: Float = 600f

// What:     `private val SEAM_THICKNESS: Dp = 1.dp` is the hairline thickness between the content and the deck.
// Why:      One density-independent pixel is the hairline the reference captures show at the seams.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEAM_THICKNESS = "1dp";
// ```
/** Thickness of the seam drawn between content and the transport deck, and between a header and its list. */
private val SEAM_THICKNESS: Dp = 1.dp

// What:     `private val SECTION_GAP: Dp = 16.dp` is the canvas-colored gap above the unfolded transport deck.
// Why:      The reference draws a 16dp canvas-colored gap between the folder picker and the deck, which
//           separates them without a line.
//
// In TS you'd write (pseudocode):
// ```ts
// const SECTION_GAP = "16dp";
// ```
/** Height of the canvas-colored gap between the folder picker and the transport deck on the unfolded screen. */
private val SECTION_GAP: Dp = 16.dp

// What:     `private val TRUE_BLACK: Color = Color(0xFF000000)` is the dark canvas.
// Why:      The dark theme paints a true-black canvas, so the deck's own dark ground stands out against it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRUE_BLACK = "#000000";
// ```
/** True black canvas used behind every pane in dark mode. */
private val TRUE_BLACK: Color = Color(0xFF000000)

// What:     `data class PlayerContentActions(...)` groups the callbacks the production screen can invoke.
// Why:      One value carries every action, so the screen signature stays short and the caller wires each action once.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayerContentActions = {
//   onToggleFolderPicker: () => void; onSelectFolder: (name: string) => void; onOpen: () => void;
//   onSettings: () => void; onPlay: (index: number) => void; deck: TransportDeckActions;
// };
// ```
/** Callbacks the production player screen invokes when the user acts on it. */
data class PlayerContentActions(
    /** Toggles the folder picker on the folded cover, where the title trigger is tapped. */
    val onToggleFolderPicker: () -> Unit,
    /** Selects the folder with the given name, where a folder name is tapped in the picker. */
    val onSelectFolder: (String) -> Unit,
    /** Opens the system folder chooser, where the Open button is tapped. */
    val onOpen: () -> Unit,
    /** Opens the settings page, where the settings action is tapped. */
    val onSettings: () -> Unit,
    /** Handles a tap on the track row of the given load-order index: selects it, or toggles play if it is current. */
    val onPlay: (Int) -> Unit,
    /** Transport deck callbacks for seek, previous, play or pause, next, and mode selection. */
    val deck: TransportDeckActions,
    /** Opens the Search page, where the Search action is tapped, or null when no Search action is offered. */
    val onSearch: (() -> Unit)? = null,
)

// What:     `internal fun isCoverLayout(widthDp: Float): Boolean` picks the folded cover layout for narrow widths.
// Why:      The width test is pure, so the threshold is unit-tested without a composition.
//
// In TS you'd write (pseudocode):
// ```ts
// const isCoverLayout = (widthDp: number): boolean => widthDp < 600;
// ```
/** Whether a width in density-independent pixels is narrow enough for the folded cover layout. */
internal fun isCoverLayout(widthDp: Float): Boolean = widthDp < COVER_LAYOUT_MAX_WIDTH_DP

// What:     `fun playerContent(...)` draws the production player screen for the given model and picker state.
// Why:      The screen is stateless: the caller owns the picker and the controller, so the same function serves
//           the production activity and the debug host.
//
// In TS you'd write (pseudocode):
// ```ts
// function playerContent(props: { model: PlayerContentModel; pickerOpen: boolean; actions: PlayerContentActions;
//   modifier?: Modifier }): UIElement;
// ```
/** Draws the production player screen, choosing the cover or unfolded layout from the available width. */
@Composable
fun playerContent(
    model: PlayerContentModel,
    pickerOpen: Boolean,
    actions: PlayerContentActions,
    modifier: Modifier = Modifier,
    creaseWidthPx: Float? = null,
) {
    /** Whether the system is in dark mode, which selects the true-black canvas and the black seams. */
    val dark: Boolean = isSystemInDarkTheme()
    /** Canvas painted behind every pane of the screen. */
    val canvas: Color = canvasColor(dark = dark)
    // Text and icons that set no color of their own read LocalContentColor, which is black unless a Surface or
    // Scaffold provides one; the screen paints its own canvas, so it provides the theme's on-surface color itself.
    CompositionLocalProvider(LocalContentColor provides MaterialTheme.colorScheme.onSurface) {
        BoxWithConstraints(modifier = modifier.fillMaxSize().background(color = canvas)) {
            if (isCoverLayout(maxWidth.value)) {
                coverLayout(model = model, pickerOpen = pickerOpen, actions = actions, dark = dark)
            } else {
                unfoldedLayout(model = model, actions = actions, canvas = canvas, creaseWidthPx = creaseWidthPx)
            }
        }
    }
}

// What:     `@Composable private fun coverLayout(...)` draws the folded cover: top row, content, seam, and deck.
// Why:      The cover shows one list or one picker at a time, under a single top row, above the deck.
//
// In TS you'd write (pseudocode):
// ```ts
// function CoverLayout(props: { model; pickerOpen; actions; dark }): UIElement {
//   return <Column>{topRow}{pickerOpen ? picker : list}{seam}{deck}</Column>;
// }
// ```
/** Draws the folded cover layout as one column of top row, folder picker or track list, seam, and transport deck. */
@Composable
private fun coverLayout(
    model: PlayerContentModel,
    pickerOpen: Boolean,
    actions: PlayerContentActions,
    dark: Boolean,
) {
    BackHandler(enabled = pickerOpen) { actions.onToggleFolderPicker() }
    /** Seam color for the dark or light theme. */
    val seam: Color = seamColor(dark = dark)
    Column(modifier = Modifier.fillMaxSize()) {
        coverTopRow(
            folderTitle = model.folderTitle,
            pickerOpen = pickerOpen,
            onToggleFolderPicker = actions.onToggleFolderPicker,
            onOpen = actions.onOpen,
            onSettings = actions.onSettings,
            onSearch = actions.onSearch,
        )
        seamLine(color = seam)
        if (pickerOpen) {
            folderPickerPane(
                sections = model.sections,
                currentFolder = model.currentFolder,
                onSelectFolder = actions.onSelectFolder,
                modifier = Modifier.weight(1f),
            )
        } else {
            trackListPane(
                entries = model.entries,
                onPlay = actions.onPlay,
                modifier = Modifier.weight(1f),
            )
        }
        seamLine(color = seam)
        transportDeck(model = model.deck, actions = actions.deck)
    }
}

// What:     `@Composable private fun unfoldedLayout(...)` draws the unfolded screen as two equal-weight panes.
// Why:      The left pane holds folders and the deck, and the right pane holds the track list of the selected folder.
//
// In TS you'd write (pseudocode):
// ```ts
// function UnfoldedLayout(props: { model; actions; dark }): UIElement {
//   return <Row><Column weight={1}>{foldersHeader}{rail}{seam}{deck}</Column>
//     <Column weight={1}>{trackHeader}{list}</Column></Row>;
// }
// ```
/**
 * Draws the unfolded layout: the folder rail with the deck on the left, and the track list on the right.
 *
 * Each pane provides the crease clearance of its own edge (E2, D110), so the text of the left pane ends before the
 * crease band and the text of the right pane starts after it, while backgrounds and containers span the band.
 */
@Composable
private fun unfoldedLayout(
    model: PlayerContentModel,
    actions: PlayerContentActions,
    canvas: Color,
    creaseWidthPx: Float?,
) {
    /** Physical pixels per dp, which converts the physical crease clearance to dp. */
    val density: Float = LocalDensity.current.density
    Row(modifier = Modifier.fillMaxSize()) {
        paneClearanceProvider(clearance = leftPaneClearance(creaseWidthPx = creaseWidthPx, density = density)) {
            Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                unfoldedFoldersHeader(title = model.libraryName, onOpen = actions.onOpen)
                folderPickerPane(
                    sections = model.sections,
                    currentFolder = model.currentFolder,
                    onSelectFolder = actions.onSelectFolder,
                    modifier = Modifier.weight(1f),
                )
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(SECTION_GAP)
                        .background(color = canvas),
                )
                transportDeck(model = model.deck, actions = actions.deck)
            }
        }
        paneClearanceProvider(clearance = rightPaneClearance(creaseWidthPx = creaseWidthPx, density = density)) {
            Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                unfoldedTrackHeader(
                    folderTitle = model.folderTitle,
                    onSettings = actions.onSettings,
                    onSearch = actions.onSearch,
                )
                trackListPane(
                    entries = model.entries,
                    onPlay = actions.onPlay,
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

// What:     `@Composable private fun seamLine(color: Color)` draws the one-density-pixel hairline.
// Why:      Seams separate the top row, the content, and the deck, so each is drawn the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// function SeamLine(props: { color: Color }): UIElement {
//   return <Box width="100%" height="1dp" background={color} />;
// }
// ```
/** Draws one horizontal seam across the full width. */
@Composable
private fun seamLine(color: Color) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(SEAM_THICKNESS)
            .background(color = color),
    )
}

// What:     `@Composable internal fun canvasColor(dark: Boolean): Color` picks the screen canvas for the theme.
// Why:      Dark mode paints true black, and light mode paints the flat Material container role; Settings
//           reads the same canvas so every page is one color.
//
// In TS you'd write (pseudocode):
// ```ts
// const canvasColor = (dark: boolean): Color => (dark ? TRUE_BLACK : scheme.surfaceContainerLowest);
// ```
/** Returns the true-black canvas in dark mode and the flat Material container in light mode. */
@Composable
internal fun canvasColor(dark: Boolean): Color {
    if (dark) {
        return TRUE_BLACK
    }
    return MaterialTheme.colorScheme.surfaceContainerLowest
}

// What:     `@Composable private fun seamColor(dark: Boolean): Color` picks the seam color for the theme.
// Why:      Light seams use the Material outline-variant role, and dark seams stay black so they do not show.
//
// In TS you'd write (pseudocode):
// ```ts
// const seamColor = (dark: boolean): Color => (dark ? TRUE_BLACK : scheme.outlineVariant);
// ```
/** Returns the outline-variant hairline in light mode and the true-black seam in dark mode. */
@Composable
private fun seamColor(dark: Boolean): Color {
    if (dark) {
        return TRUE_BLACK
    }
    return MaterialTheme.colorScheme.outlineVariant
}
