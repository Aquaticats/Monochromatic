// Immutable screen state. Position and duration stay outside this snapshot because
// the seek row polls them frequently, while these structural values change rarely.

// What:     `package ...musicplayer` places the view model beside its controller.
// Why:      Compose can receive this type without a fully qualified name.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     This import brings the page-row value into the current file.
// Why:      The visible page carries entries with display names and load indices.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { PageEntry } from "./core/Page";
// ```
import dev.monochromatic.musicplayer.core.PageEntry

// What:     This import brings the single four-state playback setting into scope.
// Why:      The UI highlights exactly one segmented option from this value.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { PlaybackMode } from "./core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.PlaybackMode

// What:     This import brings the per-track duration and peak value into scope.
// Why:      The screen state carries one measure per track it has read.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TrackMeasure } from "./core/TrackMeasure";
// ```
import dev.monochromatic.musicplayer.core.TrackMeasure

// What:     `data class` generates structural equality and `copy`; each `val` is
//           read-only. `List<T>` is a read-only view, unlike `MutableList<T>`.
// Why:      Compose replaces whole snapshots, allowing a mode change to repaint
//           controls without mutable shuffle or repeat fields hidden behind them.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayerUiState = Readonly<{
//   pageLabels: readonly string[]; selectedPage: number;
//   pageItems: readonly PageEntry[]; currentIndex: number | null;
//   playing: boolean; playbackMode: PlaybackMode; volume: number;
//   queueSize: number; loading: boolean;
// }>;
// ```
/** Supplies all slow-changing values rendered by the Android player screen. */
data class PlayerUiState(
    /** Displayed names of library pages in tab order. */
    val pageLabels: List<String> = emptyList(),

    /** Index of the page currently displayed by the UI. */
    val selectedPage: Int = 0,

    /** Rows belonging to the displayed page. */
    val pageItems: List<PageEntry> = emptyList(),

    /** Current track's load-order index, or null when nothing is selected. */
    val currentIndex: Int? = null,

    /** Tab-order index of the page that holds the current track, or null when nothing is selected. */
    val playingPageIndex: Int? = null,

    /** Every row of the page that holds the current track, which is the folder the deck's title and count read. */
    val playingItems: List<PageEntry> = emptyList(),

    /** Whether audio is currently advancing. */
    val playing: Boolean = false,

    /** Single selected behavior for completion and transport. */
    val playbackMode: PlaybackMode = PlaybackMode.IN_ORDER,

    /** Output gain from zero to one. */
    val volume: Float = 1.0f,

    /** Total loaded-track count. */
    val queueSize: Int = 0,

    /** Whether a source scan is still running. */
    val loading: Boolean = false,

    /** Measures of the tracks read so far, keyed by track URI; a track with no entry has not been read yet. */
    val measures: Map<String, TrackMeasure> = emptyMap(),
)
