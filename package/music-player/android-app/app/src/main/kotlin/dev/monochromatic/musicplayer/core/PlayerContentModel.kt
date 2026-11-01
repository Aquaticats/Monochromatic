// Pure model for the production player screen: the folder rail, the selected page's track list, and the
// transport deck. This file has no Compose imports, so the builder is unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places the builder beside the rail, folder index,
//           and page types it combines.
// Why:      The builder uses those helpers directly, so it shares their package and needs no extra imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type.
// Why:      Page entries are wrapped as tracks so the folder index can group them.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     `import dev.monochromatic.musicplayer.TransportDeckModel` brings in the deck snapshot type.
// Why:      The builder fills the deck snapshot that the transport deck draws.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TransportDeckModel } from "dev.monochromatic.musicplayer/TransportDeck";
// ```
import dev.monochromatic.musicplayer.TransportDeckModel

// What:     `data class PlayerContentModel(...)` is everything the production player screen draws.
// Why:      One plain snapshot lets the screen be composed without reading the controller.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayerContentModel = {
//   sections: RailSection[]; currentFolder: string | null; folderTitle: string;
//   entries: TrackListEntry[]; deck: TransportDeckModel;
// };
// ```
/** Everything the production player screen draws for one moment of browsing and playback. */
data class PlayerContentModel(
    /** Rail sections built from the page names in tab order. */
    val sections: List<RailSection>,
    /** Name of the selected page, or null when the selected index is outside the page list. */
    val currentFolder: String?,
    /** Heading text for the folder, which is the page name or an empty text when no page is selected. */
    val folderTitle: String,
    /** Track rows and subfolder headers of the selected page, in playback order. */
    val entries: List<TrackListEntry>,
    /** Snapshot the transport deck draws. */
    val deck: TransportDeckModel,
    /** Name of the opened folder, or the name of the MediaStore library, which heads the unfolded folder pane. */
    val libraryName: String = libraryDisplayName(null),
)

// What:     `fun buildPlayerContent(...)` turns the controller's page and playback values into the screen model.
// Why:      The production screen and its debug host both build the model here, so they draw the same content.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildPlayerContent(input: PlayerInput): PlayerContentModel { /* rail, page entries, deck */ }
// ```
/**
 * Builds the production player screen model from the page list, the selected page, and playback values.
 *
 * @param entries rows the caller already built for the selected page (with its templates), so they are not built twice
 * @param sections rail the caller already built from the page names, so it is not built on every position tick
 * @param playingPlace place the caller already built for the playing track, so the playing folder is not indexed
 *     again on every position tick
 */
fun buildPlayerContent(
    pageLabels: List<String>,
    selectedPage: Int,
    pageItems: List<PageEntry>,
    playingItems: List<PageEntry>,
    playingPageIndex: Int?,
    currentIndex: Int?,
    playing: Boolean,
    mode: PlaybackMode,
    positionSec: Double,
    durationSec: Double,
    libraryName: String = libraryDisplayName(null),
    entries: List<TrackListEntry>? = null,
    sections: List<RailSection>? = null,
    playingPlace: PlayingPlace? = null,
): PlayerContentModel {
    /** Rail sections for the picker, built from every page name unless the caller already built them. */
    val railSections: List<RailSection> = sections ?: railFor(pageLabels)
    /** Name of the selected page, or null when the selected index is outside the page list. */
    val currentFolder: String? = pageLabels.getOrNull(selectedPage)
    /** Heading text for the selected page, empty when no page is selected. */
    val folderTitle: String = currentFolder.orEmpty()
    /** Track rows and subfolder headers of the selected page, in playback order; the caller's own when it has them. */
    val pageEntries: List<TrackListEntry> = entries ?: trackListEntries(
        folder = pageFolderFor(folderTitle, pageItems),
        currentIndexInLibrary = currentIndex,
    )
    /** Label of the playing page, or an empty text when the playing page index is outside the page list. */
    val playingPageLabel: String = playingPageIndex?.let { index -> pageLabels.getOrNull(index) }.orEmpty()
    /** Playing track and its place in its own folder, which may differ from the folder on screen. */
    val place: PlayingPlace = playingPlace ?: playingPlaceOf(
        folderTitle = playingPageLabel,
        playingItems = playingItems,
        currentIndex = currentIndex,
    )
    /** Title of the playing track, or an empty text when no track is playing. */
    val title: String = place.entry?.let { entry -> trackTitleFor(entry.name) }.orEmpty()
    /** Folder that holds the playing track, or an empty text when no track is playing. */
    val folderLabel: String = deckFolderLabelFor(
        displayPath = place.entry?.name.orEmpty(),
        pageLabel = playingPageLabel,
    )
    /** Transport deck snapshot with the playing folder and title, its place in its folder, and the playback values. */
    val deck = TransportDeckModel(
        title = title,
        subtitle = subtitleFor(rowPosition = place.rowPosition, pageTrackCount = place.folderTrackCount),
        positionSec = positionSec,
        durationSec = durationSec,
        playing = playing,
        mode = mode,
        shuffleFolderLabel = folderTitle,
        folderLabel = folderLabel,
    )
    return PlayerContentModel(
        sections = railSections,
        currentFolder = currentFolder,
        folderTitle = folderTitle,
        entries = pageEntries,
        deck = deck,
        libraryName = libraryName,
    )
}

// What:     `data class PlayingPlace(...)` is the playing track's entry and its place in its own folder.
// Why:      The deck names the playing track even when its folder is not on screen, so the place is kept apart
//           from the rows of the folder on screen.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayingPlace = { entry: PageEntry | null; rowPosition: number; folderTrackCount: number };
// ```
/** The playing track's entry, its zero-based row position in its folder's list, and the folder's track count. */
data class PlayingPlace(
    /** Entry of the playing track in its folder, or null when no track is playing. */
    val entry: PageEntry?,
    /** Zero-based position of the playing row among the folder's track rows, or -1 when no row matches. */
    val rowPosition: Int,
    /** Number of tracks in the playing track's folder. */
    val folderTrackCount: Int,
)

// What:     `fun playingRowOrderOf(...)` lists the library indices of one folder's track rows in list order.
// Why:      Finding the playing row's position needs the folder's list order, and building that order indexes the whole
//           folder. The host builds it once per folder and reuses it for every track change, because rebuilding it per
//           change cost about 15 ms of main thread on a folder of 10,000 tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// function playingRowOrderOf(folderTitle: string, items: PageEntry[]): number[] {
//   return pageFolderFor(folderTitle, items).items.filter(isTrack).map((item) => item.row.indexInLibrary);
// }
// ```
/** Library indices of the folder's track rows in the order the folder's list shows them, headers left out. */
fun playingRowOrderOf(folderTitle: String, playingItems: List<PageEntry>): List<Int> =
    pageFolderFor(folderTitle = folderTitle, pageItems = playingItems)
        .items
        .filterIsInstance<FolderListItem.TrackItem>()
        .map { item -> item.row.indexInLibrary }

// What:     `fun playingPlaceIn(...)` finds the playing track's entry and row position from a prepared row order.
// Why:      A track change only needs two lookups in lists that already exist, so it must not index the folder again.
//
// In TS you'd write (pseudocode):
// ```ts
// function playingPlaceIn(items: PageEntry[], rowOrder: number[], current: number | null): PlayingPlace {
//   return { entry: items.find((i) => i.index === current) ?? null, rowPosition: rowOrder.indexOf(current ?? -1),
//     folderTrackCount: items.length };
// }
// ```
/** Builds the playing track's place from the folder's entries, its prepared row order and the current index. */
fun playingPlaceIn(playingItems: List<PageEntry>, rowOrder: List<Int>, currentIndex: Int?): PlayingPlace {
    /** Entry of the playing track within its folder, or null when the current index is not in that folder. */
    val entry: PageEntry? = playingItems.firstOrNull { item -> item.index == currentIndex }
    /** Position of the playing row among the track rows, or -1 when the playing track is not in this folder. */
    val rowPosition: Int = if (currentIndex == null) -1 else rowOrder.indexOf(currentIndex)
    return PlayingPlace(entry = entry, rowPosition = rowPosition, folderTrackCount = playingItems.size)
}

// What:     `fun playingPlaceOf(...)` finds the playing track's entry and row position in its own folder.
// Why:      The deck title and the `N of M` line must come from the whole queue, so they read the playing folder
//           and not the folder on screen.
//
// In TS you'd write (pseudocode):
// ```ts
// function playingPlaceOf(folderTitle: string, items: PageEntry[], current: number | null): PlayingPlace {
//   return playingPlaceIn(items, playingRowOrderOf(folderTitle, items), current);
// }
// ```
/** Builds the playing track's place from the playing folder's entries and the current load-order index. */
fun playingPlaceOf(folderTitle: String, playingItems: List<PageEntry>, currentIndex: Int?): PlayingPlace =
    playingPlaceIn(
        playingItems = playingItems,
        rowOrder = playingRowOrderOf(folderTitle = folderTitle, playingItems = playingItems),
        currentIndex = currentIndex,
    )

// What:     `internal fun pageFolderFor(folderTitle: String, pageItems: List<PageEntry>): FolderEntry` makes the
//           single flat folder entry of one page.
// Why:      The track list consumes a `FolderEntry`, but the page's own load-order indices must replace the
//           positions the folder index reports, so the playing row and its play action use the real index.
//
// In TS you'd write (pseudocode):
// ```ts
// function pageFolderFor(title: string, items: PageEntry[]): FolderEntry { /* index, then remap indices */ }
// ```
/** Builds one flat folder entry for a page, with each track carrying its load-order index. */
internal fun pageFolderFor(folderTitle: String, pageItems: List<PageEntry>): FolderEntry {
    /** Page rows as tracks named by their full display path, in the page's own order. */
    val tracks: List<Track> = pageItems.map { item -> Track(uri = "", displayPath = item.name) }
    /** Flat items of every folder the index makes from the page's tracks, in the index's own order. */
    val items: List<FolderListItem> = folderIndex(tracks).flatMap { folder -> folder.items }
    return FolderEntry(
        name = folderTitle,
        items = withLoadIndices(items = items, pageItems = pageItems),
        trackCount = pageItems.size,
    )
}

// What:     `internal fun withLoadIndices(...)` replaces each track item's page position with its entry's load index.
// Why:      The folder index numbers tracks by their position in the list it was given. The playing check and the
//           play action need the library's load-order index, which is stored on each `PageEntry`.
//
// In TS you'd write (pseudocode):
// ```ts
// const withLoadIndices = (items: FolderListItem[], page: PageEntry[]): FolderListItem[] =>
//   items.map((item) => item.kind === "track" ? { ...item, indexInLibrary: page[item.indexInLibrary].index } : item);
// ```
/** Rewrites each track item's list position to the load-order index of its page entry, leaving headers unchanged. */
internal fun withLoadIndices(items: List<FolderListItem>, pageItems: List<PageEntry>): List<FolderListItem> =
    items.map { item -> withLoadIndex(item = item, pageItems = pageItems) }

// What:     `private fun withLoadIndex(...)` rewrites one list item's track position, or returns a header as it is.
// Why:      Headers carry no track index, so only track items need the page entry lookup.
//
// In TS you'd write (pseudocode):
// ```ts
// function withLoadIndex(item: FolderListItem, page: PageEntry[]): FolderListItem { /* ... */ }
// ```
/** Returns one list item with its track position replaced by the load-order index of its page entry. */
private fun withLoadIndex(item: FolderListItem, pageItems: List<PageEntry>): FolderListItem = when (item) {
    is FolderListItem.TrackItem -> FolderListItem.TrackItem(
        FolderTrackRow(
            track = item.row.track,
            indexInLibrary = pageItems[item.row.indexInLibrary].index,
        ),
    )
    is FolderListItem.SubfolderHeader -> item
}

// What:     `internal fun subtitleFor(rowPosition: Int, pageTrackCount: Int): String` writes the deck's "N of M" line.
// Why:      The deck shows where the playing track sits in the page, and nothing when it is not on the page.
//
// In TS you'd write (pseudocode):
// ```ts
// const subtitleFor = (row: number, total: number): string => (row < 0 ? "" : `${row + 1} of ${total}`);
// ```
/** Returns the one-based position of the playing row followed by the page track count, or an empty text. */
internal fun subtitleFor(rowPosition: Int, pageTrackCount: Int): String {
    if (rowPosition < 0) {
        return ""
    }
    return "${rowPosition + 1} of $pageTrackCount"
}
