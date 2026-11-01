// Track list pane for one folder's flat list (D5, D6, D31, D42). It draws track rows and subfolder headers
// from the entries the core layer builds. The list moves only on entry, on a page change, and on a reveal request,
// never because the playing track changed.

// What:     `package dev.monochromatic.musicplayer` places the track list pane beside the other player composables.
// Why:      The pane is a stateless composable that screens import without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.foundation` bring in clicking, layout, and the lazy list.
// Why:      The pane draws a scrolling list of rows that play a track when tapped.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clickable, LazyColumn } from "compose/foundation";
// ```
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState

// What:     Imports from `androidx.compose.material3` and `runtime` bring in the theme roles, text, and effects.
// Why:      Rows read Material roles so they follow the app theme, and the scroll effect runs as a coroutine.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MaterialTheme, Text } from "material3";
// import { LaunchedEffect, Composable } from "compose/runtime";
// ```
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.MutableIntState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import kotlin.math.roundToInt
import dev.monochromatic.musicplayer.core.MenuAnchorRect

// What:     Imports from `androidx.compose.ui` bring in the modifier, color, font weight, semantics, and dp units.
// Why:      The current row is bold, uses the surface role as its background, and publishes its selected state.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Color, FontWeight, Role, Dp } from "compose/ui";
// ```
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.input.pointer.PointerEvent
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.isSecondaryPressed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import kotlinx.coroutines.flow.first
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the entry types the pane draws.
// Why:      The pane renders the pure entries built by the core layer and never builds them itself.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TrackListEntry, TrackRowModel } from "core/TrackListEntries";
// ```
import dev.monochromatic.musicplayer.core.FocusRingPlacement
import dev.monochromatic.musicplayer.core.HostKeyZone
import dev.monochromatic.musicplayer.core.KeyActivateRow
import dev.monochromatic.musicplayer.core.KeyOpenRowMenu
import dev.monochromatic.musicplayer.core.KeyRowAction
import dev.monochromatic.musicplayer.core.ListKeyCommand
import dev.monochromatic.musicplayer.core.TRACK_ROW_RING_TEXT_GAP_DP
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.TrackMenuAction
import dev.monochromatic.musicplayer.core.ringAfter
import dev.monochromatic.musicplayer.core.startFocus
import dev.monochromatic.musicplayer.core.TrackRowLineSource
import dev.monochromatic.musicplayer.core.TrackRowColumns
import dev.monochromatic.musicplayer.core.TrackRowModel
import dev.monochromatic.musicplayer.core.currentPositionOf
import dev.monochromatic.musicplayer.core.entryKeyOf
import dev.monochromatic.musicplayer.core.isSamePageUpdate
import dev.monochromatic.musicplayer.core.pageEntryScrollTarget
import dev.monochromatic.musicplayer.core.pageIdentityOf
import dev.monochromatic.musicplayer.core.reportFirstVisibleTrack
import dev.monochromatic.musicplayer.core.rowWithPlayingFlag
import dev.monochromatic.musicplayer.core.trackRowColumns

// What:     `private val ROW_MIN_HEIGHT: Dp = 48.dp` is the smallest height of a track row and a header.
// Why:      The row meets the 48dp touch target, and a row with a supporting line grows past it.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_MIN_HEIGHT = 48;
// ```
/** Smallest height of one track row or subfolder header, matching the 48dp touch target. */
private val ROW_MIN_HEIGHT: Dp = 48.dp

// What:     `private val ROW_HORIZONTAL_PADDING: Dp = 16.dp` insets a row's text from the pane edges.
// Why:      The text sits clear of the edges so it does not touch the highlight of the current row.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_HORIZONTAL_PADDING = 16;
// ```
/** Horizontal inset of a row's text from the pane edges. */
private val ROW_HORIZONTAL_PADDING: Dp = 16.dp

// What:     `private val ROW_VERTICAL_PADDING: Dp = 12.dp` insets a row's text from its top and bottom.
// Why:      A one-line row with this inset reaches the 48dp minimum height.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_VERTICAL_PADDING = 12;
// ```
/** Vertical inset of a row's text from its top and bottom edges. */
private val ROW_VERTICAL_PADDING: Dp = 12.dp

// What:     `private val TEXT_SPACING: Dp = 2.dp` separates a row's title from its supporting line.
// Why:      A small gap keeps the two lines readable as one row.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEXT_SPACING = 2;
// ```
/** Gap between a row's title and its supporting line. */
private val TEXT_SPACING: Dp = 2.dp

// What:     `private val HEADER_TOP_PADDING: Dp = 16.dp` sets the space above each subfolder header.
// Why:      The space sets a header apart from the tracks above it, which distinguishes it from a track row.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADER_TOP_PADDING = 16;
// ```
/** Space above a subfolder header, which sets it apart from the track rows above it. */
private val HEADER_TOP_PADDING: Dp = 16.dp

// What:     `internal val LocalTrackRowMenuOpener` carries the menu opener from the player host to every track row.
// Why:      The track list is drawn through the player content, which takes no menu parameter, so the host provides the
//           opener through this local instead of threading a new parameter through every screen.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalTrackRowMenuOpener = createContext<((index: number) => void) | null>(null);
// ```
/** Composition local holding the track menu opener, or null when no host provides one. */
internal val LocalTrackRowMenuOpener: ProvidableCompositionLocal<((Int, MenuAnchorRect) -> Unit)?> =
    staticCompositionLocalOf { null }

// What:     `internal val LocalTrackRowLines` carries the supporting-line source from the host to every track row.
// Why:      The analysis publishes measures many times a second. The host keeps the list of rows unchanged and each
//           visible row reads its own line from this source, so a new measure recomposes only the rows it belongs to
//           and never rebuilds a list of thousands of rows (ANR on the main thread).
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalTrackRowLines = createContext<TrackRowLineSource | null>(null);
// ```
/** Composition local holding the source of the rows' supporting lines, or null when the entries carry their own. */
internal val LocalTrackRowLines: ProvidableCompositionLocal<TrackRowLineSource?> =
    staticCompositionLocalOf { null }

// What:     `internal val LocalPlayingIndex` carries the library index of the playing track from the host to the list.
// Why:      The host builds the rows of a page once, so a track change must not rebuild thousands of rows. The list and
//           each visible row read the playing index from here instead, and a change recomposes only what shows it.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalPlayingIndex = createContext<number | null>(null);
// ```
/** Composition local holding the playing track's library index, or null when no track is loaded. */
internal val LocalPlayingIndex: ProvidableCompositionLocal<Int?> = compositionLocalOf { null }

// What:     `internal val LocalFirstVisibleTrack` carries the cell where the list reports its first visible track.
// Why:      The measure refresh reads the tracks on screen first, so it needs to know which ones they are. The list
//           writes the library index of its first visible track here, and the host reads it between batches.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalFirstVisibleTrack = createContext<{ value: number } | null>(null);
// ```
/** Composition local holding the cell the list writes its first visible track's library index to, or null. */
internal val LocalFirstVisibleTrack: ProvidableCompositionLocal<MutableIntState?> = staticCompositionLocalOf { null }

// What:     `val HostKeyLocalRevealFolder: ProvidableCompositionLocal<Int>` carries the folder reveal count.
// Why:      The host bumps the count when the deck's folder part is tapped, and the track list scrolls to the playing
//           track's subfolder header. It sits here, not beside HostKeyLocalReveal, because the list is its only reader.
//
// In TS you'd write (pseudocode):
// ```ts
// const HostKeyLocalRevealFolder = createContext(0);
// ```
/** Count of folder reveal requests, read by the track list to scroll to the playing track's folder header. */
val HostKeyLocalRevealFolder: ProvidableCompositionLocal<Int> = compositionLocalOf { 0 }

// What:     `fun trackListPane(...)` draws the folder's flat list and moves it only on entry, page change, or reveal.
// Why:      The caller owns the entries and the playback action, so the pane keeps only its scroll state. A play, next,
//           previous, or measure update must not move the list, so the page effect is keyed on the page identity.
//
// In TS you'd write (pseudocode):
// ```ts
// function TrackListPane(props: { entries: TrackListEntry[]; onPlay: (indexInLibrary: number) => void }): UIElement;
// ```
/**
 * Draws track rows and subfolder headers in one scrolling list, playing a row by its library index when tapped.
 *
 * @param onOpenMenu Opens the track context menu for a row's library index on long press, secondary click, or the
 *     menu key; defaults to the opener the host provides through [LocalTrackRowMenuOpener], and null disables it.
 */
@Composable
fun trackListPane(
    entries: List<TrackListEntry>,
    onPlay: (Int) -> Unit,
    onOpenMenu: ((Int, MenuAnchorRect) -> Unit)? = LocalTrackRowMenuOpener.current,
    modifier: Modifier = Modifier,
) {
    /** Scroll state of the list, which the page and reveal effects move. */
    val listState = rememberLazyListState()
    /** Source of each row's supporting line, or null when the entries already carry theirs. */
    val rowLines: TrackRowLineSource? = LocalTrackRowLines.current
    /** Library index of the playing track, which the entries do not carry so that a track change keeps them. */
    val playingIndex: Int? = LocalPlayingIndex.current
    /** Position of the playing row among the entries, or null when no row in this list is playing. */
    val currentPosition: Int? = currentPositionOf(entries, playingIndex)
    /** Cell the host reads the first visible track from, or null when no host asks for it. */
    val firstVisibleSink: MutableIntState? = LocalFirstVisibleTrack.current
    /** Page identity the page effect last handled, or null before its first run. */
    var shownPage: List<String>? by remember { mutableStateOf<List<String>?>(null) }
    /** Identity of the shown page, which follows the folder's tracks and not the playing row, measures, or indexes. */
    val pageIdentity: List<String> = remember(entries) { pageIdentityOf(entries) }
    // Declared before the reveal effects, so when a reveal and a page change launch in the same composition,
    // the reveal's scroll is the one launched last. Keyed on the identity, so a play or measure change never runs it.
    LaunchedEffect(pageIdentity) {
        /** Whether this identity is the page already handled, so the list keeps its position. */
        val sameAsHandled: Boolean = isSamePageUpdate(previous = shownPage, next = pageIdentity)
        shownPage = pageIdentity
        if (!sameAsHandled) {
            listState.scrollToItem(pageEntryScrollTarget(entries, playingIndex))
        }
    }
    /** Focus state of the list as one keyboard stop, whose focus ring marks one row. */
    val zone: ListZoneState = rememberListZone(HostKeyZone.TRACKS)
    /** Position among the entries that the focus ring is on, or null when the ring has not started. */
    var ringPosition: Int? by remember(pageIdentity) { mutableStateOf<Int?>(null) }
    /** One flag per entry, true for the track rows, which are the entries that can hold the ring. */
    val selectable: List<Boolean> = remember(entries) { entries.map { entry -> entry is TrackListEntry.RowEntry } }
    /** Window-space bounds of the list, from which the menu anchor of the ringed row is computed. */
    var listBounds: Rect? by remember { mutableStateOf<Rect?>(null) }
    /** Runs a track menu action chosen by key for a library index, or null when no host provides one. */
    val rowKeyActions: ((TrackMenuAction, Int) -> Unit)? = LocalTrackRowKeyActions.current
    LaunchedEffect(zone.focused) {
        if (zone.focused && ringPosition?.let { at -> selectable.getOrElse(at) { false } } != true) {
            ringPosition = startFocus(selectable = selectable, preferred = currentPosition)
        }
    }
    LaunchedEffect(ringPosition) {
        /** Entry the ring is on now. */
        val at: Int? = ringPosition
        if (at != null && listState.layoutInfo.visibleItemsInfo.none { item -> item.index == at }) {
            listState.scrollToItem(at)
        }
    }
    LaunchedEffect(listState, entries, firstVisibleSink) {
        if (firstVisibleSink != null) {
            reportFirstVisibleTrack(
                positions = snapshotFlow { listState.firstVisibleItemIndex },
                entries = entries,
                report = { index -> firstVisibleSink.intValue = index },
            )
        }
    }
    /** Count of reveal requests from the host, each of which scrolls to and focuses the playing row. */
    val revealToken: Int = HostKeyLocalReveal.current
    /** Reveal count when this list entered composition, so that entering the list never reveals by itself. */
    val revealAtEntry: Int = remember { revealToken }
    LaunchedEffect(revealToken) {
        if (revealToken != revealAtEntry && currentPosition != null) {
            listState.scrollToItem(currentPosition)
            snapshotFlow { listState.layoutInfo.visibleItemsInfo.any { item -> item.index == currentPosition } }
                .first { visible -> visible }
            ringPosition = currentPosition
            zone.requester.requestFocus()
        }
    }
    /** Count of folder reveal requests from the host, each of which scrolls to the playing track's folder header. */
    val folderRevealToken: Int = HostKeyLocalRevealFolder.current
    /** Folder reveal count when this list entered composition, so that entering the list never scrolls by itself. */
    val folderRevealAtEntry: Int = remember { folderRevealToken }
    /** Item a folder reveal scrolls to: the playing track's subfolder header, or the first item for a root track. */
    val folderTargetIndex: Int = folderScrollIndexFor(entries, playingIndex)
    LaunchedEffect(folderRevealToken) {
        if (folderRevealToken != folderRevealAtEntry && currentPosition != null) {
            listState.scrollToItem(folderTargetIndex)
        }
    }
    LazyColumn(
        state = listState,
        modifier = modifier
            .onGloballyPositioned { coordinates -> listBounds = coordinates.boundsInWindow() }
            .listZone(zone = HostKeyZone.TRACKS, state = zone) { command ->
                trackListCommand(
                    command = command,
                    entries = entries,
                    selectable = selectable,
                    ringPosition = ringPosition,
                    page = listState.layoutInfo.visibleItemsInfo.size,
                    onRing = { at -> ringPosition = at },
                    onPlay = onPlay,
                    refocus = { zone.requester.requestFocus() },
                    openMenuAt = { at ->
                        /** Layout of the ringed row, whose window bounds anchor the menu. */
                        val item = listState.layoutInfo.visibleItemsInfo.firstOrNull { info -> info.index == at }
                        /** Window-space bounds of the list, or null before its first layout. */
                        val bounds: Rect? = listBounds
                        /** The ringed entry when it is a track row, else null. */
                        val row = entries[at] as? TrackListEntry.RowEntry
                        if (item != null && bounds != null && row != null) {
                            onOpenMenu?.invoke(
                                row.row.indexInLibrary,
                                menuAnchorOfRect(
                                    Rect(
                                        left = bounds.left,
                                        top = bounds.top + item.offset,
                                        right = bounds.right,
                                        bottom = bounds.top + item.offset + item.size,
                                    ),
                                ),
                            )
                        }
                    },
                    runRowAction = rowKeyActions,
                )
            },
    ) {
        itemsIndexed(items = entries, key = { _, entry -> entryKeyOf(entry) }) { position, entry ->
            when (entry) {
                is TrackListEntry.RowEntry -> trackRow(
                    row = rowWithLine(
                        row = rowWithPlayingFlag(row = entry.row, playingIndex = playingIndex),
                        lines = rowLines,
                    ),
                    onPlay = onPlay,
                    onOpenMenu = onOpenMenu,
                    ringed = zone.focused && ringPosition == position,
                )
                is TrackListEntry.HeaderEntry -> subfolderHeader(name = entry.name, trackCount = entry.trackCount)
            }
        }
    }
}

// What:     `internal fun folderScrollIndexFor(entries: List<TrackListEntry>, playingIndex: Int?): Int` finds the item
//           the folder reveal scrolls to.
// Why:      The folder's own tracks come before its subfolder headers, and each subfolder's tracks follow its header.
//           So the nearest header above the playing row is the subfolder that holds it. A playing row with no header
//           above it sits in the folder itself, and the first item is the target.
//
// In TS you'd write (pseudocode):
// ```ts
// const folderScrollIndexFor = (entries: TrackListEntry[], playingIndex: number | null): number => {
//   const current = currentPositionOf(entries, playingIndex);
//   if (current === null) return 0;
//   for (let i = current; i >= 0; i--) if (entries[i].kind === "header") return i;
//   return 0;
// };
// ```
/**
 * Runs one list command of the key table on the track list and reports whether it consumed the key.
 *
 * Movement keys put the focus ring on another row; Enter acts on the ringed row like a tap, the menu keys open its
 * menu, and a row chord runs its menu action, all on the row under the ring.
 *
 * @param command list command that the key resolved to.
 * @param entries entries of the shown page, rows and subfolder headers.
 * @param selectable one flag per entry, true for rows, which can hold the ring.
 * @param ringPosition entry the ring is on now, or null.
 * @param page rows that a page key moves by.
 * @param onRing moves the ring to an entry.
 * @param onPlay handles a tap or Enter on a track by library index: select it, or toggle play on the current row.
 * @param refocus gives keyboard focus back to the list, which the host's play path hands to the playback area.
 * @param openMenuAt opens the context menu of the row at an entry position.
 * @param runRowAction runs a menu action for a library index, or null when the host provides none.
 */
private fun trackListCommand(
    command: ListKeyCommand,
    entries: List<TrackListEntry>,
    selectable: List<Boolean>,
    ringPosition: Int?,
    page: Int,
    onRing: (Int) -> Unit,
    onPlay: (Int) -> Unit,
    refocus: () -> Unit,
    openMenuAt: (Int) -> Unit,
    runRowAction: ((TrackMenuAction, Int) -> Unit)?,
): Boolean {
    /** Entry the ring moves to when the command is a movement. */
    val moved: Int? = ringAfter(
        command = command,
        selectable = selectable,
        labels = emptyList(),
        current = ringPosition,
        page = page,
    )
    if (moved != null) {
        onRing(moved)
        return true
    }
    /** Ringed entry, which must be a row for the remaining commands. */
    val at: Int = ringPosition ?: return false
    /** The ringed row, or null when the ring is on no row. */
    val row: TrackRowModel = (entries.getOrNull(at) as? TrackListEntry.RowEntry)?.row ?: return false
    return when (command) {
        KeyActivateRow -> {
            onPlay(row.indexInLibrary)
            refocus()
            true
        }
        KeyOpenRowMenu -> {
            openMenuAt(at)
            true
        }
        is KeyRowAction -> {
            runRowAction?.invoke(command.action, row.indexInLibrary)
            runRowAction != null
        }
        else -> false
    }
}

/** Index of the subfolder header that holds the playing row, or 0 when no header holds it or no row is playing. */
internal fun folderScrollIndexFor(entries: List<TrackListEntry>, playingIndex: Int?): Int {
    /** Position of the playing row among the entries, or null when no row in this list is playing. */
    val current: Int? = currentPositionOf(entries, playingIndex)
    if (current == null) {
        return 0
    }
    /** Index of the nearest header at or before the playing row, or null when no header comes before it. */
    val headerIndex: Int? = (current downTo 0).firstOrNull { index ->
        entries[index] is TrackListEntry.HeaderEntry
    }
    return headerIndex ?: 0
}

// What:     `private fun menuAnchorOfRect(bounds: Rect): MenuAnchorRect` converts window bounds to whole pixels.
// Why:      The placement rule works on integer pixels, and rounding happens once here at the Compose boundary.
//
// In TS you'd write (pseudocode):
// ```ts
// const menuAnchorOfRect = (b: Rect): MenuAnchorRect => ({ left: Math.round(b.left), top: Math.round(b.top), ... });
// ```
/** Converts window-space float bounds of a row into the whole-pixel rectangle the menu placement reads. */
private fun menuAnchorOfRect(bounds: Rect): MenuAnchorRect = MenuAnchorRect(
    left = bounds.left.roundToInt(),
    top = bounds.top.roundToInt(),
    right = bounds.right.roundToInt(),
    bottom = bounds.bottom.roundToInt(),
)

// What:     `@Composable private fun rowWithLine(row: TrackRowModel, lines: TrackRowLineSource?): TrackRowModel` gives
//           the row the supporting line that its own measure produces.
// Why:      The line is read through `derivedStateOf`, so a measure that arrives for another track does not recompose
//           this row, and a measure that leaves this row's line unchanged does not either.
//
// In TS you'd write (pseudocode):
// ```ts
// function rowWithLine(row: TrackRowModel, lines: TrackRowLineSource | null): TrackRowModel {
//   const line = useDerived(() => lines?.lineOf(row.indexInLibrary));
//   return lines === null ? row : { ...row, supportingLine: line };
// }
// ```
/** Returns the row with the line [lines] computes for it from its own measure, or the row itself without a source. */
@Composable
private fun rowWithLine(row: TrackRowModel, lines: TrackRowLineSource?): TrackRowModel {
    if (lines == null) {
        return row
    }
    /** Line of this row, recomputed when its measure changes and published only when the text differs. */
    val line: String? by remember(lines, row.indexInLibrary) {
        derivedStateOf { lines.lineOf(row.indexInLibrary) }
    }
    return row.copy(supportingLine = line)
}

// What:     `@Composable private fun trackRow(...)` draws one playable track row.
// Why:      The current row gets the D42 surface background and a bold title, and a tap selects a row (and starts it
//           with Auto-play on select on, D129), or toggles play on the current one (D124). Its box spans the pane
//           and its text sits in the heading column (D130).
//
// In TS you'd write (pseudocode):
// ```ts
// function TrackRow(props: { row: TrackRowModel; onPlay: (index: number) => void }): UIElement;
// ```
/**
 * Draws one track row with its title, optional supporting line, and current-row treatment.
 * Tap selects the row (and starts it with Auto-play on select on, D129), or toggles play and pause on the current
 * row (D124); long press, a secondary-button press, or the menu key opens its context menu.
 */
@Composable
private fun trackRow(
    row: TrackRowModel,
    onPlay: (Int) -> Unit,
    onOpenMenu: ((Int, MenuAnchorRect) -> Unit)?,
    ringed: Boolean,
) {
    /** Surface role behind the current row (D42), and transparent behind every other row. */
    val rowBackground: Color = if (row.isCurrent) {
        MaterialTheme.colorScheme.surfaceContainerLow
    } else {
        Color.Transparent
    }
    /** Weight of the title, bold for the current row and the body default for the rest. */
    val titleWeight: FontWeight? = if (row.isCurrent) FontWeight.Bold else null
    /** Window-space bounds of this row from its last layout, or null before the row is first placed. */
    var rowBounds: MenuAnchorRect? by remember { mutableStateOf<MenuAnchorRect?>(null) }
    /**
     * Box and text insets of this row (D130): the box (fill, state layers, the ring just inside its edge, D120) spans
     * the pane even beside the crease, and the text takes the heading column, the only thing the crease moves.
     */
    val columns: TrackRowColumns = trackRowColumns(
        startClearanceDp = LocalPaneStartClearance.current.value,
        endClearanceDp = LocalPaneEndClearance.current.value,
        ordinaryInsetDp = ROW_HORIZONTAL_PADDING.value,
        ringDepthDp = FOCUS_RING_OUTSET.value,
        ringTextGapDp = TRACK_ROW_RING_TEXT_GAP_DP,
    )
    /** Action that opens this row's menu anchored to the row, or null when no menu opener is provided. */
    val openMenuForRow: (() -> Unit)? = onOpenMenu?.let { open ->
        {
            rowBounds?.let { bounds -> open(row.indexInLibrary, bounds) }
        }
    }
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .onGloballyPositioned { coordinates -> rowBounds = menuAnchorOfRect(coordinates.boundsInWindow()) }
            .padding(start = columns.boxStartDp.dp, end = columns.boxEndDp.dp)
            .heightIn(min = ROW_MIN_HEIGHT)
            .background(rowBackground)
            .keyboardFocusRing(visible = ringed, shape = RectangleShape, placement = FocusRingPlacement.INSIDE)
            .focusProperties { canFocus = false }
            .combinedClickable(
                role = Role.Button,
                onClick = { onPlay(row.indexInLibrary) },
                onLongClick = openMenuForRow,
            )
            .pointerInput(row.indexInLibrary, onOpenMenu) {
                awaitPointerEventScope {
                    while (true) {
                        /** Next pointer event on this row, read before the row's own tap handling sees it. */
                        val event: PointerEvent = awaitPointerEvent()
                        if (event.type == PointerEventType.Press && event.buttons.isSecondaryPressed) {
                            openMenuForRow?.invoke()
                        }
                    }
                }
            }
            .semantics { selected = row.isCurrent }
            .padding(
                start = columns.textStartDp.dp - columns.boxStartDp.dp,
                end = columns.textEndDp.dp - columns.boxEndDp.dp,
                top = ROW_VERTICAL_PADDING,
                bottom = ROW_VERTICAL_PADDING,
            ),
        verticalArrangement = Arrangement.spacedBy(TEXT_SPACING),
    ) {
        Text(
            text = row.title,
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = titleWeight,
        )
        if (row.supportingLine != null) {
            Text(
                text = row.supportingLine,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

// What:     `@Composable private fun subfolderHeader(...)` draws one subfolder's name and track count.
// Why:      The header is not clickable, and its primary-colored name and count line set it apart from track rows.
//
// In TS you'd write (pseudocode):
// ```ts
// function SubfolderHeader(props: { name: string; trackCount: number }): UIElement;
// ```
/** Draws a non-clickable subfolder header with its name and the number of tracks under it. */
@Composable
private fun subfolderHeader(name: String, trackCount: Int) {
    /** Count line that reads one track in the singular and any other count in the plural. */
    val countText: String = if (trackCount == 1) "1 track" else "$trackCount tracks"
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = ROW_MIN_HEIGHT)
            .padding(
                start = paneStartInset(ROW_HORIZONTAL_PADDING),
                end = paneEndInset(ROW_HORIZONTAL_PADDING),
                top = HEADER_TOP_PADDING,
            )
            .semantics { heading() },
        verticalArrangement = Arrangement.spacedBy(TEXT_SPACING),
    ) {
        Text(
            text = name,
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.titleSmall,
        )
        Text(
            text = countText,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}
