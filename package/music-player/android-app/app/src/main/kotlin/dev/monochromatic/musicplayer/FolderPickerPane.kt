// What:     `package dev.monochromatic.musicplayer` places the folder picker beside the other player composables.
// Why:      The picker is a stateless composable that screens import without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the rail section and cell models built by `railFor`.
// Why:      The picker draws the rail and names from the same model the library index produces.
//
// In TS you'd write (pseudocode):
// ```ts
// import { RailCell, RailSection } from "core/FolderRail";
// ```
import dev.monochromatic.musicplayer.core.RailCell
import dev.monochromatic.musicplayer.core.RailSection

// What:     Imports from `androidx.compose.foundation` bring in layout, lazy list, scrolling, selection,
//           and background helpers.
// Why:      The rail scrolls on its own, the names list is lazy, and every target is selectable.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LazyColumn, verticalScroll, selectable } from "compose/foundation";
// ```
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.material3` and `runtime` bring in the theme accessor, text,
//           and the coroutine scope used to animate the list.
// Why:      Colors and type come from the Material roles, and a rail tap launches one scroll animation.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MaterialTheme, Text } from "material3";
// ```
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.focus.focusProperties
import dev.monochromatic.musicplayer.core.FocusRingPlacement
import dev.monochromatic.musicplayer.core.HostKeyZone
import dev.monochromatic.musicplayer.core.KeyActivateRow
import dev.monochromatic.musicplayer.core.ringAfter
import dev.monochromatic.musicplayer.core.startFocus
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope

// What:     Imports from `androidx.compose.ui` bring in modifiers, alignment, geometry, colors, and semantics.
// Why:      The indicator is drawn in the draw phase, and each target publishes radio-button semantics.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Color, Offset, Size } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

// What:     `private val RAIL_WIDTH: Dp = 48.dp` is the width of the letter rail column.
// Why:      One 48dp cell fills the column, so the rail is exactly one touch target wide, and the cell's focus ring
//           lies just inside its edge (D120) because the scrolling column clips anything outside it.
//
// In TS you'd write (pseudocode):
// ```ts
// const RAIL_WIDTH = 48;
// ```
/** Width of the letter rail column: one minimum touch target; the focus ring of a cell lies inside the cell. */
private val RAIL_WIDTH: Dp = 48.dp

// What:     `private val RAIL_CELL_SIZE: Dp = 48.dp` is the side of each rail cell.
// Why:      Every rail cell stays a 48dp target so it meets the touch-target minimum.
//
// In TS you'd write (pseudocode):
// ```ts
// const RAIL_CELL_SIZE = 48;
// ```
/** Side length of one rail cell, which is also its minimum touch target. */
private val RAIL_CELL_SIZE: Dp = 48.dp

// What:     `private val RAIL_HIGHLIGHT_SIZE: Dp = 32.dp` is the diameter of the current cell's circle.
// Why:      The circle sits inside the 48dp cell and leaves room for the touch target around it.
//
// In TS you'd write (pseudocode):
// ```ts
// const RAIL_HIGHLIGHT_SIZE = 32;
// ```
/** Diameter of the circle behind the rail label of the current folder's cell. */
private val RAIL_HIGHLIGHT_SIZE: Dp = 32.dp

// What:     `private val SECTION_HAIRLINE_HEIGHT: Dp = 1.dp` is the thickness of the rule between sections.
// Why:      A hairline separates writing systems without adding a heavier divider to the rail.
//
// In TS you'd write (pseudocode):
// ```ts
// const SECTION_HAIRLINE_HEIGHT = 1;
// ```
/** Thickness of the hairline drawn between writing-system sections of the rail. */
private val SECTION_HAIRLINE_HEIGHT: Dp = 1.dp

// What:     `private val RAIL_BOUNDARY_WIDTH: Dp = 1.dp` is the thickness of the rule between the rail and the names.
// Why:      D34 specifies a one-pixel rule so the letter targets read as separate from the folder names.
//
// In TS you'd write (pseudocode):
// ```ts
// const RAIL_BOUNDARY_WIDTH = 1;
// ```
/** Width of the vertical rule drawn between the letter rail and the folder names in light mode. */
private val RAIL_BOUNDARY_WIDTH: Dp = 1.dp

// What:     `private val NAME_TARGET_MIN: Dp = 48.dp` is the minimum width and height of one folder name target.
// Why:      Each name stays a 48dp target even though it takes only its natural text width.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAME_TARGET_MIN = 48;
// ```
/** Number of folder names that Page up and Page down move the focus ring by. */
private const val NAMES_PAGE: Int = 10

/** Number of rail cells that Page up and Page down move the focus ring by. */
private const val RAIL_PAGE_CELLS: Int = 8

/** Minimum width and height of one folder name target. */
private val NAME_TARGET_MIN: Dp = 48.dp

// What:     `private val NAME_SPACING: Dp = 8.dp` is the gap between wrapped folder names.
// Why:      The gap adds to the inner text padding so names read as separate targets.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAME_SPACING = 8;
// ```
/** Horizontal gap between neighboring folder name targets in a wrapped line. */
private val NAME_SPACING: Dp = 8.dp

// What:     `private val NAME_TEXT_PADDING: Dp = 4.dp` is the inner horizontal padding of each name.
// Why:      Padding on both sides of each name keeps the visible gap between names at the design width.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAME_TEXT_PADDING = 4;
// ```
/** Horizontal padding on each side of one folder name's text. */
private val NAME_TEXT_PADDING: Dp = 4.dp

// What:     `private val SELECTED_INDICATOR_HEIGHT: Dp = 2.dp` is the bar under the current folder's target.
// Why:      A bar on the target's bottom edge marks the selection without underlining the label text.
//
// In TS you'd write (pseudocode):
// ```ts
// const SELECTED_INDICATOR_HEIGHT = 2;
// ```
/** Thickness of the bar drawn along the bottom edge of the current folder's target. */
private val SELECTED_INDICATOR_HEIGHT: Dp = 2.dp

// What:     `private val HEADING_START: Dp = 16.dp` is the start padding of a section heading.
// Why:      The heading aligns with the first name of its section and sits above the wrapped names.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_START = 16;
// ```
/** Start padding of a section heading, aligned with the names below it. */
private val HEADING_START: Dp = 16.dp

// What:     `private val HEADING_TOP: Dp = 8.dp` is the space above a section heading's label.
// Why:      The label sits clear of the rule or the previous section's last line of names.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_TOP = 8;
// ```
/** Top padding of a section heading above its label. */
private val HEADING_TOP: Dp = 8.dp

// What:     `private val HEADING_END: Dp = 16.dp` is the end padding of a section heading.
// Why:      The heading keeps the same inset on the right as on the left.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_END = 16;
// ```
/** End padding of a section heading. */
private val HEADING_END: Dp = 16.dp

// What:     `private val HEADING_BOTTOM: Dp = 2.dp` is the space between a section heading and its names.
// Why:      A small gap keeps the label visually attached to the names it introduces.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADING_BOTTOM = 2;
// ```
/** Bottom padding of a section heading, separating it from the names. */
private val HEADING_BOTTOM: Dp = 2.dp

// What:     `private val NAMES_START: Dp = 12.dp` is the start padding of a wrapped line of names.
// Why:      The names start a little left of the heading so their targets line up with the text padding.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAMES_START = 12;
// ```
/** Start padding of a wrapped line of folder names. */
private val NAMES_START: Dp = 12.dp

// What:     `private val NAMES_END: Dp = 16.dp` is the end padding of a wrapped line of names.
// Why:      The last name on a line keeps a margin from the window edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAMES_END = 16;
// ```
/** End padding of a wrapped line of folder names. */
private val NAMES_END: Dp = 16.dp

// What:     `private val NAMES_BOTTOM: Dp = 16.dp` is the space under one section's last line of names.
// Why:      The next section's heading starts after a clear gap, so the sections read as separate groups.
//
// In TS you'd write (pseudocode):
// ```ts
// const NAMES_BOTTOM = 16;
// ```
/** Bottom padding under the last wrapped line of folder names of one section. */
private val NAMES_BOTTOM: Dp = 16.dp

// What:     `internal fun cellsInOrder(sections: List<RailSection>): List<RailCell>` flattens every section's cells.
// Why:      The names list shows one item per cell, and its item indexes follow this flat order.
//
// In TS you'd write (pseudocode):
// ```ts
// function cellsInOrder(sections: RailSection[]): RailCell[] {
//   return sections.flatMap(section => section.cells);
// }
// ```
/** Returns every rail cell in rail order, across all writing-system sections. */
internal fun cellsInOrder(sections: List<RailSection>): List<RailCell> =
    sections.flatMap { section -> section.cells }

// What:     `internal fun cellKeyForFolder(sections, folder): String?` finds the cell that lists a folder.
// Why:      The rail highlights the current folder's cell, and a null folder or an unknown name has no cell.
//
// In TS you'd write (pseudocode):
// ```ts
// function cellKeyForFolder(sections: RailSection[], folder: string | null): string | undefined {
//   return cellsInOrder(sections).find(cell => cell.names.includes(folder ?? ""))?.key;
// }
// ```
/** Returns the key of the cell listing the folder, or null when the folder is null or listed nowhere. */
internal fun cellKeyForFolder(sections: List<RailSection>, folder: String?): String? {
    if (folder == null) {
        return null
    }
    return cellsInOrder(sections).firstOrNull { cell -> folder in cell.names }?.key
}

// What:     `internal fun railHighlightKey(tappedKey, currentKey): String?` picks the rail cell to circle.
// Why:      A tapped letter must show as selected at once, even though the current folder is still in another cell;
//           the tap stands until the current folder changes.
//
// In TS you'd write (pseudocode):
// ```ts
// function railHighlightKey(tappedKey: string | null, currentKey: string | null): string | null {
//   return tappedKey ?? currentKey;
// }
// ```
/** Returns the tapped cell's key when a tap is pending, otherwise the key of the current folder's cell. */
internal fun railHighlightKey(tappedKey: String?, currentKey: String?): String? = tappedKey ?: currentKey

// What:     `internal fun firstNameForCell(sections, cellKey): String?` returns the first folder under a cell.
// Why:      A rail tap scrolls the names to this name, so the list lands on the cell's first folder.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstNameForCell(sections: RailSection[], cellKey: string): string | undefined {
//   return cellsInOrder(sections).find(cell => cell.key === cellKey)?.names[0];
// }
// ```
/** Returns the first folder name under the cell with the key, or null when no such cell has a name. */
internal fun firstNameForCell(sections: List<RailSection>, cellKey: String): String? =
    cellsInOrder(sections).firstOrNull { cell -> cell.key == cellKey }?.names?.firstOrNull()

// What:     `internal fun itemIndexForCell(sections, cellKey): Int?` gives the list position of one cell.
// Why:      The names list uses one item per cell, so this index is what the list scrolls to.
//
// In TS you'd write (pseudocode):
// ```ts
// function itemIndexForCell(sections: RailSection[], cellKey: string | null): number | undefined {
//   const index = cellsInOrder(sections).findIndex(cell => cell.key === cellKey);
//   return index < 0 ? undefined : index;
// }
// ```
/** Returns the list position of the cell with the key, or null when no cell has that key. */
internal fun itemIndexForCell(sections: List<RailSection>, cellKey: String?): Int? {
    /** Position of the cell in the flat cell list, or -1 when the key is unknown. */
    val index: Int = cellsInOrder(sections).indexOfFirst { cell -> cell.key == cellKey }
    if (index < 0) {
        return null
    }
    return index
}

// What:     `internal fun itemIndexForFolder(sections, folder): Int?` gives the list position of a folder's cell.
// Why:      The picker opens on the current folder and scrolls to a rail cell's first name through this lookup.
//
// In TS you'd write (pseudocode):
// ```ts
// function itemIndexForFolder(sections: RailSection[], folder: string | null): number | undefined {
//   return itemIndexForCell(sections, cellKeyForFolder(sections, folder));
// }
// ```
/** Returns the list position of the cell that lists the folder, or null when no cell lists it. */
internal fun itemIndexForFolder(sections: List<RailSection>, folder: String?): Int? =
    itemIndexForCell(sections, cellKeyForFolder(sections, folder))

// What:     `@Composable fun folderPickerPane(...)` draws the letter rail beside the wrapped folder names.
// Why:      The caller owns the library and the current folder, so the picker stays stateless and previewable.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderPickerPane(props: { sections: RailSection[]; currentFolder: string | null;
//   onSelectFolder(name: string): void; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the folder picker: a one-column letter rail on the left and wrapped folder names on the right.
 *
 * The names list opens at the current folder's cell, and a rail tap scrolls it to that cell's first name.
 * The picker takes its height from the parent, so the caller sets the height through `modifier`.
 *
 * @param sections rail model built from the library folder names
 * @param currentFolder folder shown as current, or null when none is selected
 * @param onSelectFolder invoked with the folder name when a name is tapped
 * @param modifier outer modifier applied to the picker row
 */
@Composable
fun folderPickerPane(
    sections: List<RailSection>,
    currentFolder: String?,
    onSelectFolder: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    /** Flat list of every cell, one list item per cell. */
    val cells: List<RailCell> = cellsInOrder(sections)
    /** Key of the cell listing the current folder, used for the rail highlight and the opening scroll. */
    val currentCellKey: String? = cellKeyForFolder(sections, currentFolder)
    /** Scroll state of the names list, opened at the current folder's cell so a long library starts there. */
    val listState = rememberLazyListState(
        initialFirstVisibleItemIndex = itemIndexForCell(sections, currentCellKey) ?: 0,
    )
    /** Coroutine scope that runs the scroll animation started by a rail tap. */
    val scope = rememberCoroutineScope()
    /** Cell the user last tapped on the rail, forgotten when the current folder changes. */
    val tappedCellKey = remember(currentFolder) { mutableStateOf<String?>(null) }
    /** Focus state of the folder name list as one keyboard stop, whose focus ring marks one name. */
    val namesZone: ListZoneState = rememberListZone(HostKeyZone.NAMES)
    /** Every folder name in list order, which is the order the ring walks. */
    val names: List<String> = remember(sections) { cells.flatMap { cell -> cell.names } }
    /** Folder name the focus ring is on, or null when the ring has not started. */
    var ringName: String? by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(namesZone.focused) {
        if (namesZone.focused && ringName !in names) {
            ringName = startFocus(
                selectable = names.map { true },
                preferred = names.indexOf(currentFolder).takeIf { at -> at >= 0 },
            )?.let { at -> names[at] }
        }
    }
    LaunchedEffect(ringName) {
        /** Position of the cell that lists the ringed name, which must be on screen for the ring to be seen. */
        val cellIndex: Int? = itemIndexForFolder(sections, ringName)
        if (cellIndex != null && listState.layoutInfo.visibleItemsInfo.none { item -> item.index == cellIndex }) {
            listState.scrollToItem(cellIndex)
        }
    }
    Row(modifier = modifier.fillMaxSize()) {
        letterRail(
            sections = sections,
            currentCellKey = railHighlightKey(tappedKey = tappedCellKey.value, currentKey = currentCellKey),
            onCellSelected = { cellKey ->
                tappedCellKey.value = cellKey
                /** First folder name under the tapped cell, which the names list scrolls to. */
                val firstName: String? = firstNameForCell(sections, cellKey)
                /** List position of the cell that lists that first name. */
                val targetIndex: Int? = itemIndexForFolder(sections, firstName)
                if (targetIndex != null) {
                    scope.launch { listState.animateScrollToItem(targetIndex) }
                }
            },
        )
        // What:     The light-mode rail boundary is a one-pixel rule in the outline-variant role.
        // Why:      D34 keeps a visible rail line in light mode, and the dark theme keeps its stable black surfaces.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!dark) rail.append(<div style={{ width: 1, background: scheme.outlineVariant }} />);
        // ```
        if (!isSystemInDarkTheme()) {
            Box(
                modifier = Modifier
                    .width(RAIL_BOUNDARY_WIDTH)
                    .fillMaxHeight()
                    .background(MaterialTheme.colorScheme.outlineVariant),
            )
        }
        LazyColumn(
            state = listState,
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .listZone(zone = HostKeyZone.NAMES, state = namesZone) { command ->
                    if (command == KeyActivateRow) {
                        ringName?.let { name -> onSelectFolder(name) }
                        return@listZone ringName != null
                    }
                    /** Name the ring moves to, or null when the command does not move it. */
                    val moved: Int? = ringAfter(
                        command = command,
                        selectable = names.map { true },
                        labels = names,
                        current = names.indexOf(ringName).takeIf { at -> at >= 0 },
                        page = NAMES_PAGE,
                    )
                    if (moved != null) {
                        ringName = names[moved]
                    }
                    moved != null
                },
        ) {
            items(items = cells, key = { cell -> cell.key }) { cell ->
                cellNames(
                    cell = cell,
                    currentFolder = currentFolder,
                    ringName = if (namesZone.focused) ringName else null,
                    onSelectFolder = onSelectFolder,
                )
            }
        }
    }
}

// What:     `@Composable private fun letterRail(...)` draws the one-column rail of writing-system cells.
// Why:      The rail scrolls on its own with no scrollbar, and a hairline separates each writing system.
//
// In TS you'd write (pseudocode):
// ```ts
// function letterRail(props: { sections: RailSection[]; currentCellKey: string | null;
//   onCellSelected(key: string): void }): UIElement;
// ```
/**
 * Draws the rail column, with a hairline before every section after the first.
 *
 * The rail is one keyboard focus stop. While it holds focus a focus ring marks one cell, the arrow, Home, End and
 * page keys move the ring, a typed letter jumps to the first cell with that glyph, and Enter does what a tap on the
 * cell does. The cells themselves take no focus, so Tab passes the rail in one step (D106).
 */
@Composable
private fun letterRail(
    sections: List<RailSection>,
    currentCellKey: String?,
    onCellSelected: (String) -> Unit,
) {
    /** Focus state of the rail as one list. */
    val zone: ListZoneState = rememberListZone(HostKeyZone.FOLDERS)
    /** Every cell in rail order, which is the order the ring walks. */
    val cells: List<RailCell> = cellsInOrder(sections)
    /** Key of the cell the focus ring is on, or null when the ring has not started. */
    var ringKey: String? by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(zone.focused) {
        if (zone.focused && cells.none { cell -> cell.key == ringKey }) {
            ringKey = startFocus(
                selectable = cells.map { true },
                preferred = cells.indexOfFirst { cell -> cell.key == currentCellKey }.takeIf { at -> at >= 0 },
            )?.let { at -> cells[at].key }
        }
    }
    Column(
        modifier = Modifier
            .width(RAIL_WIDTH)
            .fillMaxHeight()
            .verticalScroll(rememberScrollState())
            .selectableGroup()
            .listZone(zone = HostKeyZone.FOLDERS, state = zone) { command ->
                if (command == KeyActivateRow) {
                    ringKey?.let { key -> onCellSelected(key) }
                    return@listZone ringKey != null
                }
                /** Cell the ring moves to, or null when the command does not move it. */
                val moved: Int? = ringAfter(
                    command = command,
                    selectable = cells.map { true },
                    labels = cells.map { cell -> cell.label },
                    current = cells.indexOfFirst { cell -> cell.key == ringKey }.takeIf { at -> at >= 0 },
                    page = RAIL_PAGE_CELLS,
                )
                if (moved != null) {
                    ringKey = cells[moved].key
                }
                moved != null
            },
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        sections.forEachIndexed { sectionIndex, section ->
            if (sectionIndex > 0) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(SECTION_HAIRLINE_HEIGHT)
                        .background(MaterialTheme.colorScheme.outlineVariant),
                )
            }
            section.cells.forEach { cell ->
                railCellTarget(
                    cell = cell,
                    selected = cell.key == currentCellKey,
                    ringed = zone.focused && cell.key == ringKey,
                    onClick = { onCellSelected(cell.key) },
                )
            }
        }
    }
}

// What:     `@Composable private fun railCellTarget(...)` draws one 48dp rail cell.
// Why:      The current folder's cell gets a circle and a bolder label, which is a second channel besides color.
//
// In TS you'd write (pseudocode):
// ```ts
// function railCellTarget(props: { cell: RailCell; selected: boolean; onClick(): void }): UIElement;
// ```
/**
 * Draws one rail label, highlighted with a circle and bold text when it is the current cell.
 *
 * The cell takes no keyboard focus of its own; [ringed] draws the rail's focus ring on it while the rail is focused.
 */
@Composable
private fun railCellTarget(cell: RailCell, selected: Boolean, ringed: Boolean, onClick: () -> Unit) {
    /** Lets the rail scroll the ringed cell into view when a key moves the ring there. */
    val bringIntoView = remember { BringIntoViewRequester() }
    LaunchedEffect(ringed) {
        if (ringed) {
            bringIntoView.bringIntoView()
        }
    }
    Box(
        modifier = Modifier
            .size(RAIL_CELL_SIZE)
            .bringIntoViewRequester(bringIntoView)
            .keyboardFocusRing(visible = ringed, shape = CircleShape, placement = FocusRingPlacement.INSIDE)
            .focusProperties { canFocus = false }
            .selectable(selected = selected, onClick = onClick, role = Role.RadioButton),
        contentAlignment = Alignment.Center,
    ) {
        if (selected) {
            Box(
                modifier = Modifier
                    .size(RAIL_HIGHLIGHT_SIZE)
                    .background(
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        shape = CircleShape,
                    ),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = cell.label,
                    color = MaterialTheme.colorScheme.onSecondaryContainer,
                    style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                )
            }
        } else {
            Text(
                text = cell.label,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelLarge,
            )
        }
    }
}

// What:     `@OptIn(ExperimentalLayoutApi::class)` allows the `FlowRow` used to wrap names.
// Why:      `FlowRow` is marked experimental in this Compose version, so the opt-in sits on the one
//           composable that uses it.
//
// In TS you'd write (pseudocode):
// ```ts
// // Wrapping flex rows need no opt-in in TypeScript.
// ```
/** Draws one cell's heading and its folder names, wrapped across the available width. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun cellNames(
    cell: RailCell,
    currentFolder: String?,
    ringName: String?,
    onSelectFolder: (String) -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = cell.label,
            modifier = Modifier.padding(
                start = HEADING_START,
                top = HEADING_TOP,
                end = paneEndInset(HEADING_END),
                bottom = HEADING_BOTTOM,
            ),
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.titleLarge,
        )
        FlowRow(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = NAMES_START, end = paneEndInset(NAMES_END), bottom = NAMES_BOTTOM)
                .selectableGroup(),
            horizontalArrangement = Arrangement.spacedBy(NAME_SPACING),
        ) {
            cell.names.forEach { name ->
                folderNameTarget(
                    name = name,
                    selected = name == currentFolder,
                    ringed = name == ringName,
                    onClick = { onSelectFolder(name) },
                )
            }
        }
    }
}

// What:     `@Composable private fun folderNameTarget(...)` draws one plain-text folder name as a 48dp target.
// Why:      The selected name is marked by primary color, a medium weight, and a bar on its bottom edge, not by a fill.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderNameTarget(props: { name: string; selected: boolean; onClick(): void }): UIElement;
// ```
/**
 * Draws one folder name with the selection bar on the bottom edge of its target.
 *
 * The target takes no keyboard focus of its own; [ringed] draws the name list's focus ring on it while the list is
 * focused.
 */
@Composable
private fun folderNameTarget(name: String, selected: Boolean, ringed: Boolean, onClick: () -> Unit) {
    /** Color of the name text and the selection bar, both taken from the primary role when selected. */
    val accentColor: Color = MaterialTheme.colorScheme.primary
    /** Text color of the name, primary when it is the current folder and on-surface otherwise. */
    val textColor: Color = if (selected) accentColor else MaterialTheme.colorScheme.onSurface
    /** Text style of the name, with medium weight when it is the current folder. */
    val textStyle = if (selected) {
        MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium)
    } else {
        MaterialTheme.typography.bodyLarge
    }
    Box(
        modifier = Modifier
            .defaultMinSize(minWidth = NAME_TARGET_MIN, minHeight = NAME_TARGET_MIN)
            .keyboardFocusRing(visible = ringed, shape = RectangleShape)
            .focusProperties { canFocus = false }
            .selectable(selected = selected, onClick = onClick, role = Role.RadioButton)
            .drawBehind {
                if (selected) {
                    /** Bar thickness in pixels, converted from the Dp constant for the current density. */
                    val indicatorHeight: Float = SELECTED_INDICATOR_HEIGHT.toPx()
                    drawRect(
                        color = accentColor,
                        topLeft = Offset(0f, size.height - indicatorHeight),
                        size = Size(size.width, indicatorHeight),
                    )
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        // What:     A transparent copy of the name at the selected weight sets the target's width for every name.
        // Why:      The selected name is drawn heavier and so wider; sizing each target by the heavier copy keeps
        //           the flow's line breaks the same whichever folder is selected.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <span style={{ fontWeight: 500, color: 'transparent' }} aria-hidden>{name}</span>
        // ```
        Text(
            text = name,
            modifier = Modifier.padding(horizontal = NAME_TEXT_PADDING).clearAndSetSemantics {},
            color = Color.Transparent,
            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium),
        )
        Text(
            text = name,
            modifier = Modifier.padding(horizontal = NAME_TEXT_PADDING),
            color = textColor,
            style = textStyle,
        )
    }
}
