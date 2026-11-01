// Host of the production player screen. It owns the route between the player, the Search page, the Settings page,
// and the template editor, the Search query, the folder picker flag, and the template texts. It sends each action
// to the controller or to its own state, and it saves a template only when the edited text applies.

// What:     `package dev.monochromatic.musicplayer` places the host beside the controller and the screens it hosts.
// Why:      The host calls the screens and the controller from this package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.Context` types the context the template preferences are read through.
// Why:      The host loads and saves templates through `SessionStore`, which takes a Context.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Context } from "android/content";
// ```
import android.content.Context

// What:     `import android.util.Log` writes the route and template decisions to the log.
// Why:      Every route change and template save is logged, so a device run shows which destination was reached.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.activity.compose.BackHandler` intercepts system Back while the host is off the player.
// Why:      Back on Search or Settings returns to the player, and Back in the editor returns to Settings.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BackHandler } from "androidx/activity/compose";
// ```
import androidx.activity.compose.BackHandler

// What:     `import androidx.compose.foundation.layout.Box` and `fillMaxSize` lay out the player and its notice.
// Why:      The player and its notice share one box that fills the screen.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Box } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding

// What:     `import androidx.compose.runtime.LaunchedEffect` runs the feedback expiry keyed on the next expiry.
// Why:      Expiry must wait for the next due message and restart when a message is shown or dismissed.
//
// In TS you'd write (pseudocode):
// ```ts
// import { useEffect } from "react";
// ```
import androidx.compose.runtime.LaunchedEffect

// What:     `import androidx.compose.runtime.Composable` marks the host as a function that emits UI.
// Why:      The host emits the screens for the current route.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Composable } from "compose/runtime";
// ```
import androidx.compose.runtime.Composable

// What:     `import androidx.compose.runtime.getValue` and `setValue` let delegated state be read and written.
// Why:      The host keeps the query, the picker flag, and the templates in delegated state.
//
// In TS you'd write (pseudocode):
// ```ts
// // Delegated state is read and written by property access.
// ```
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue

// What:     `import androidx.compose.runtime.mutableStateOf` and `remember` hold state across recompositions.
// Why:      The templates are loaded once per host, and each edit replaces them.
//
// In TS you'd write (pseudocode):
// ```ts
// import { useState } from "react";
// ```
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember

// What:     `import androidx.compose.runtime.saveable.rememberSaveable` keeps state across configuration changes.
// Why:      The route, the query, and the picker flag survive a rotation.
//
// In TS you'd write (pseudocode):
// ```ts
// // A state hook whose value survives a configuration change.
// ```
import androidx.compose.runtime.saveable.rememberSaveable

// What:     Imports for the track menu, its actions, and the trash Undo flow.
// Why:      The host opens the menu, runs its actions on a coroutine scope, and shows each result in the overlay.
//
// In TS you'd write (pseudocode):
// ```ts
// import { trackMenuPopup, trashRouteFor, trashUndoMessage } from "./track-menu";
// ```
import android.net.Uri
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.MutableIntState
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.onFocusChanged
import dev.monochromatic.musicplayer.core.CreaseState
import dev.monochromatic.musicplayer.core.TrackActionFailure
import dev.monochromatic.musicplayer.core.libraryDisplayName
import dev.monochromatic.musicplayer.core.TrackMenuAction
import dev.monochromatic.musicplayer.core.TrashRecord
import dev.monochromatic.musicplayer.core.TrashRoute
import dev.monochromatic.musicplayer.core.trackFileNameOf
import dev.monochromatic.musicplayer.core.mimeTypeFor
import dev.monochromatic.musicplayer.core.trackActionFailureMessage
import dev.monochromatic.musicplayer.core.trackDetailsMessage
import dev.monochromatic.musicplayer.core.trackLoadFailureMessage
import dev.monochromatic.musicplayer.core.trackMenuActionsFor
import dev.monochromatic.musicplayer.core.trashRouteFor
import dev.monochromatic.musicplayer.core.trashUndoMessage
import dev.monochromatic.musicplayer.core.RailSection
import dev.monochromatic.musicplayer.core.TEMPLATE_PREVIEW_ROW_COUNT
import dev.monochromatic.musicplayer.core.TrackMeasure
import dev.monochromatic.musicplayer.core.TrackRowLineSource
import dev.monochromatic.musicplayer.core.measureDetailsValue
import dev.monochromatic.musicplayer.core.measuresByIndex
import dev.monochromatic.musicplayer.core.railFor
import java.util.UUID
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

// What:     `import androidx.compose.ui.Alignment` aligns the feedback overlay to the bottom start edge.
// Why:      The overlay floats at the bottom of the player, and it never reflows the content.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Alignment } from "compose/ui";
// ```
import androidx.compose.ui.Alignment

// What:     `import androidx.compose.ui.unit.Dp` and `dp` size the feedback overlay inset.
// Why:      The overlay sits 16dp from the edges, and the inset is a named constant.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Dp } from "compose/ui/unit";
// ```
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `import androidx.compose.ui.Modifier` sizes the player box.
// Why:      The box fills the screen so the player content lays out in full.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Modifier } from "compose/ui";
// ```
import androidx.compose.ui.Modifier

// What:     `import androidx.compose.ui.platform.LocalContext` reads the current Android context.
// Why:      The template preferences need a Context, and the composition provides one.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LocalContext } from "compose/ui/platform";
// ```
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity

// What:     `import dev.monochromatic.musicplayer.core.PlayerHostModel` and the core types the host reads.
// Why:      The pure rules and the core values live in the core package and are shared with the tests.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlayerHostModel, SearchActivation } from "./core/PlayerHostModel";
// ```
import dev.monochromatic.musicplayer.core.PlayerContentModel
import dev.monochromatic.musicplayer.core.HostKeyActions
import dev.monochromatic.musicplayer.core.HostKeyContext
import dev.monochromatic.musicplayer.core.HostKeyDispatcher
import dev.monochromatic.musicplayer.core.HostKeyLayout
import dev.monochromatic.musicplayer.core.HostKeyZone
import dev.monochromatic.musicplayer.core.adjacentFolderLabel
import dev.monochromatic.musicplayer.core.rightPaneClearance
import dev.monochromatic.musicplayer.core.NO_VISIBLE_TRACK
import dev.monochromatic.musicplayer.core.pageMeasurePasses
import dev.monochromatic.musicplayer.core.seekTargetSec
import dev.monochromatic.musicplayer.core.MenuAnchorRect
import dev.monochromatic.musicplayer.core.PlayingPlace
import dev.monochromatic.musicplayer.core.playingPlaceIn
import dev.monochromatic.musicplayer.core.playingRowOrderOf
import androidx.compose.ui.platform.LocalConfiguration
import dev.monochromatic.musicplayer.core.HostKeyRoute
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.PlayerHostModel
import dev.monochromatic.musicplayer.core.SearchActivation
import dev.monochromatic.musicplayer.core.SearchAnswer
import dev.monochromatic.musicplayer.core.SearchResult
import dev.monochromatic.musicplayer.core.SearchState
import dev.monochromatic.musicplayer.core.TemplateEditorState
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.FeedbackMessage
import dev.monochromatic.musicplayer.core.buildPlayerContent
import dev.monochromatic.musicplayer.core.SearchResultKind
import dev.monochromatic.musicplayer.core.playingResultId
import dev.monochromatic.musicplayer.core.searchStateFor

// What:     `sealed interface PlayerRoute` names the five destinations the host can show.
// Why:      One closed type lets the host draw exactly one destination and makes Back a total function.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlayerRoute = { kind: "player" } | { kind: "search" } | { kind: "settings" }
//   | { kind: "templateEditor"; template: TemplateKind } | { kind: "licenses" };
// ```
/** The destinations of the player host. */
sealed interface PlayerRoute {
    // What:     `data object Player` is the player screen with the folder picker and the track list.
    // Why:      The player is the start destination and the one Back returns to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "player" }
    // ```
    /** The player screen, which shows the folder picker or the track list. */
    data object Player : PlayerRoute

    // What:     `data object Search` is the Search page reached from the Search button.
    // Why:      Search is a separate destination with its own query, and Back returns to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "search" }
    // ```
    /** The Search page, reached from the Search button. */
    data object Search : PlayerRoute

    // What:     `data object Settings` is the Settings page that lists the two templates.
    // Why:      Settings is a destination of its own, and Back returns to the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "settings" }
    // ```
    /** The Settings page that lists the track-row and playing-track templates. */
    data object Settings : PlayerRoute

    // What:     `data class TemplateEditor(val kind: TemplateKind)` is the editor of one template.
    // Why:      Each template has its own editor, opened from Settings, and Back returns to Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "templateEditor"; template: TemplateKind }
    // ```
    /** The editor of one template, reached from Settings. */
    data class TemplateEditor(
        /** The template the editor is open on. */
        val kind: TemplateKind,
    ) : PlayerRoute

    // What:     `data object Licenses` is the Licenses page reached from the About group of Settings.
    // Why:      The notices are a page of their own on every window width, and Back returns to Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "licenses" }
    // ```
    /** The Licenses page, reached from the Licenses row of Settings. */
    data object Licenses : PlayerRoute
}

// What:     `internal object PlayerRouteRules` holds the pure route rules: Back and the saved text of each route.
// Why:      The rules are testable on the JVM, and the saved text keeps the route across a configuration change
//           without making every route Parcelable.
//
// In TS you'd write (pseudocode):
// ```ts
// const PlayerRouteRules = { backFrom, encode, decode };
// ```
/** Pure route rules: where Back leads, and how a route is saved in the instance state as text. */
internal object PlayerRouteRules {
    // What:     `private const val TOKEN_PLAYER: String = "player"` names the player route in the saved state.
    // Why:      A fixed token keeps the saved text stable across builds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOKEN_PLAYER = "player";
    // ```
    /** Saved token of the player route. */
    private const val TOKEN_PLAYER: String = "player"

    // What:     `private const val TOKEN_SEARCH: String = "search"` names the Search route in the saved state.
    // Why:      A fixed token keeps the saved text stable across builds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOKEN_SEARCH = "search";
    // ```
    /** Saved token of the Search route. */
    private const val TOKEN_SEARCH: String = "search"

    // What:     `private const val TOKEN_SETTINGS: String = "settings"` names the Settings route in the saved state.
    // Why:      A fixed token keeps the saved text stable across builds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOKEN_SETTINGS = "settings";
    // ```
    /** Saved token of the Settings route. */
    private const val TOKEN_SETTINGS: String = "settings"

    // What:     `private const val TOKEN_TEMPLATE_PREFIX: String = "template:"` starts the saved token of an editor.
    // Why:      The template kind's name follows the prefix, so one prefix covers every editor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOKEN_TEMPLATE_PREFIX = "template:";
    // ```
    /** Prefix of the saved token of a template editor, followed by the template kind's name. */
    private const val TOKEN_TEMPLATE_PREFIX: String = "template:"

    // What:     `private const val TOKEN_LICENSES: String = "licenses"` names the Licenses route in the saved state.
    // Why:      A fixed token keeps the saved text stable across builds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TOKEN_LICENSES = "licenses";
    // ```
    /** Saved token of the Licenses route. */
    private const val TOKEN_LICENSES: String = "licenses"

    // What:     `fun backFrom(route: PlayerRoute): PlayerRoute` names the route Back leads to.
    // Why:      Search and Settings return to the player, and an editor returns to Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function backFrom(route: PlayerRoute): PlayerRoute { /* editor -> settings; others -> player */ }
    // ```
    /** Returns the route Back leads to from one route. */
    fun backFrom(route: PlayerRoute): PlayerRoute = when (route) {
        PlayerRoute.Player -> PlayerRoute.Player
        PlayerRoute.Search -> PlayerRoute.Player
        PlayerRoute.Settings -> PlayerRoute.Player
        is PlayerRoute.TemplateEditor -> PlayerRoute.Settings
        PlayerRoute.Licenses -> PlayerRoute.Settings
    }

    // What:     `fun backFromHost(route: PlayerRoute, settingsIsWide: Boolean): PlayerRoute` names where the Back
    //           button and Escape lead on the screen the host draws.
    // Why:      The wide Settings page shows its editor in place, so Back closes Settings from either route and
    //           skips the Settings route; the Licenses page is a page of its own on every width and returns to
    //           Settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function backFromHost(route: PlayerRoute, wide: boolean): PlayerRoute {
    //   if (route.kind === "licenses") return settings;
    //   return wide && route.kind !== "search" ? player : backFrom(route);
    // }
    // ```
    /** Returns the route that Back leads to from [route], given whether Settings is drawn in two panes. */
    fun backFromHost(route: PlayerRoute, settingsIsWide: Boolean): PlayerRoute = when {
        route == PlayerRoute.Licenses -> PlayerRoute.Settings
        settingsIsWide && route != PlayerRoute.Search -> PlayerRoute.Player
        else -> backFrom(route)
    }

    // What:     `fun encode(route: PlayerRoute): String` writes one route as its saved token.
    // Why:      The instance state keeps text, so each route is saved as a stable token.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function encode(route: PlayerRoute): string { /* token per route */ }
    // ```
    /** Returns the saved token of one route. */
    fun encode(route: PlayerRoute): String = when (route) {
        PlayerRoute.Player -> TOKEN_PLAYER
        PlayerRoute.Search -> TOKEN_SEARCH
        PlayerRoute.Settings -> TOKEN_SETTINGS
        is PlayerRoute.TemplateEditor -> TOKEN_TEMPLATE_PREFIX + route.kind.name
        PlayerRoute.Licenses -> TOKEN_LICENSES
    }

    // What:     `fun decode(token: String): PlayerRoute` reads one route from its saved token.
    // Why:      An unknown token restores the player, the same way an unknown stored playback mode falls back
    //           to its default, so a stale saved state never blocks the screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function decode(token: string): PlayerRoute { /* unknown -> player */ }
    // ```
    /** Returns the route of one saved token, or the player route when the token names no route. */
    fun decode(token: String): PlayerRoute = when {
        token == TOKEN_SEARCH -> PlayerRoute.Search
        token == TOKEN_SETTINGS -> PlayerRoute.Settings
        token == TOKEN_LICENSES -> PlayerRoute.Licenses
        token.startsWith(TOKEN_TEMPLATE_PREFIX) -> templateRouteFor(token)
        else -> PlayerRoute.Player
    }

    // What:     `private fun templateRouteFor(token: String): PlayerRoute` reads the editor route of one token.
    // Why:      A template token names a kind, and a name no kind carries falls back to the player route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function templateRouteFor(token: string): PlayerRoute { /* kind by name, else player */ }
    // ```
    /** Returns the editor route of one template token, or the player route when the kind name is unknown. */
    private fun templateRouteFor(token: String): PlayerRoute {
        /** Name of the template kind that follows the token prefix. */
        val kindName: String = token.removePrefix(TOKEN_TEMPLATE_PREFIX)
        /** Template kind with that name, or null when no kind has it. */
        val kind: TemplateKind? = TemplateKind.entries.firstOrNull { entry -> entry.name == kindName }
        if (kind == null) {
            return PlayerRoute.Player
        }
        return PlayerRoute.TemplateEditor(kind)
    }
}

// What:     `private fun loadStoredTemplates(context: Context): Map<TemplateKind, String>` reads every template.
// Why:      The host starts with the saved text of each template, so the rows and Settings show what the user saved.
//
// In TS you'd write (pseudocode):
// ```ts
// function loadStoredTemplates(context: Context): Record<TemplateKind, string> { /* each kind from the store */ }
// ```
/** Reads the stored text of every template, each falling back to its default when nothing valid is stored. */
private fun loadStoredTemplates(context: Context): Map<TemplateKind, String> =
    TemplateKind.entries.associateWith { kind -> SessionStore.loadTemplate(context, kind) }

// What:     `private val PLAYER_HOST_FEEDBACK_INSET: Dp = 16.dp` is the gap between the overlay and the screen edges.
// Why:      The feedback overlay keeps this inset from the start, end, and bottom edges.
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAYER_HOST_FEEDBACK_INSET = 16; // dp
// ```
/** Inset between the feedback overlay and the start, end, and bottom edges of the player. */
private val PLAYER_HOST_FEEDBACK_INSET: Dp = 16.dp

// What:     `private val TRACK_MENU_INNER_INSET: Dp = 12.dp` is the track menu's start inset on the inner panes.
// Why:      The inner panes keep 12dp at their start edge, so the menu's leftmost position leaves that inset clear.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_MENU_INNER_INSET = 12; // dp
// ```
/** Start inset of the track menu on the unfolded inner panes, where the cover uses no inset. */
private val TRACK_MENU_INNER_INSET: Dp = 12.dp

// What:     `private fun hostKeyRouteOf(route: PlayerRoute): HostKeyRoute` names the keyboard route of a host route.
// Why:      The keyboard rules have their own route kinds, so each host route maps to one of them here.
//
// In TS you'd write (pseudocode):
// ```ts
// const hostKeyRouteOf = (route: PlayerRoute): HostKeyRoute => (route.kind === "template" ? "editor" : route.kind);
// ```
/** Maps a host route to the route kind that the keyboard rules use. */
private fun hostKeyRouteOf(route: PlayerRoute): HostKeyRoute = when (route) {
    PlayerRoute.Player -> HostKeyRoute.PLAYER
    PlayerRoute.Search -> HostKeyRoute.SEARCH
    PlayerRoute.Settings -> HostKeyRoute.SETTINGS
    is PlayerRoute.TemplateEditor -> HostKeyRoute.EDITOR
    PlayerRoute.Licenses -> HostKeyRoute.SETTINGS
}

// What:     `@Composable fun playerHost(...)` draws the destination of the current route over the controller state.
// Why:      The player, Search, Settings, and the editor share one route and one set of template texts, so the host
//           holds them together and routes each action to the controller or to its own state.
//
// In TS you'd write (pseudocode):
// ```ts
// function PlayerHost(props: { controller: PlayerController; onChooseFolder: () => void }): UIElement { /* ... */ }
// ```
/**
 * Draws the player host: the route, the Search query, the folder picker flag, the template texts, the scan
 * indicator, and the feedback overlay.
 *
 * @param feedback Feedback queue that the host draws above the player and expires on time.
 * @param emptyNotice Draws the notice over the player while the library is loading or holds no track.
 * @param firstRunContent Draws the first-run pane over the player when the open source needs it, given the
 *     callback that opens the Settings route; null when the player shows.
 * @param libraryName Name of the opened folder, or of the MediaStore library, shown above the unfolded folders.
 * @param readLicenseNotices Reads the text of the Licenses page for a context; the APK's notice asset by default.
 */
@Composable
fun playerHost(
    controller: PlayerController,
    feedback: FeedbackHostState,
    onChooseFolder: () -> Unit,
    emptyNotice: @Composable (PlayerUiState) -> Unit = {},
    firstRunContent: (@Composable (onSettings: () -> Unit) -> Unit)? = null,
    libraryName: String = libraryDisplayName(null),
    readLicenseNotices: suspend (Context) -> String = ::readLicenseNoticesAsset,
) {
    /** Android context that the template preferences are read and written through. */
    val context: Context = LocalContext.current
    /** Saved token of the route the host shows, which survives a configuration change. */
    var routeToken: String by rememberSaveable { mutableStateOf(PlayerRouteRules.encode(PlayerRoute.Player)) }
    /** Route the host shows, decoded from the saved token. */
    val route: PlayerRoute = PlayerRouteRules.decode(routeToken)
    /** Text typed into the Search query, which starts empty on each visit. */
    var query: String by rememberSaveable { mutableStateOf("") }
    /** Whether the folder picker is open on the folded cover. */
    var pickerOpen: Boolean by rememberSaveable { mutableStateOf(false) }
    /** Whether the playback area, which is the host's own focusable root, holds focus itself. */
    var rootFocused: Boolean by remember { mutableStateOf(false) }
    /** Whether the host root or any control inside it holds focus. */
    var hostHasFocus: Boolean by remember { mutableStateOf(false) }
    /** Whether the Search query field holds focus, as the Search page reports it. */
    var searchFieldFocused: Boolean by remember { mutableStateOf(false) }
    /** The list that holds keyboard focus (the folder rail or the track list), or none. */
    var keyZone: HostKeyZone by remember { mutableStateOf(HostKeyZone.NONE) }
    /** The list that a focus chord asked to take keyboard focus and has not yet done so, or null. */
    var pendingFocus: HostKeyZone? by remember { mutableStateOf<HostKeyZone?>(null) }
    /** Callback that the lists report their focus through, kept across frames so readers stay valid. */
    val reportKeyZone: (HostKeyZone, Boolean) -> Unit = remember {
        { zone: HostKeyZone, focused: Boolean ->
            if (focused) {
                keyZone = zone
            } else if (keyZone == zone) {
                keyZone = HostKeyZone.NONE
            }
        }
    }
    /** Callback that a list calls after it took focus for the pending request. */
    val clearPendingFocus: () -> Unit = remember { { pendingFocus = null } }
    /** Link that the Search shortcut presses, which refocuses the open Search page's query field in the same event. */
    val searchFocusPort: SearchFocusPort = remember { SearchFocusPort() }
    /** Count of reveal requests, read by the track list to scroll to and focus the playing row. */
    var revealToken: Int by remember { mutableIntStateOf(0) }
    /** Count of folder reveal requests, read by the track list to scroll to the playing track's folder header. */
    var folderRevealToken: Int by remember { mutableIntStateOf(0) }
    /** Focus requester of the host root, which takes the player's keys when no control holds focus. */
    val rootFocusRequester: FocusRequester = remember { FocusRequester() }
    /** Callback that the Search query field reports its focus through, kept across frames so readers stay valid. */
    val reportSearchFieldFocus: (Boolean) -> Unit = remember { { focused: Boolean -> searchFieldFocused = focused } }
    /** Both templates as typed and as last applied, loaded from the store when the host first composes. */
    var templates: TemplateEditorState by remember {
        mutableStateOf(PlayerHostModel.restoredEditor(stored = loadStoredTemplates(context)))
    }
    /** Auto-play on select (D129): whether selecting a track also starts it, loaded from the store on first compose. */
    var autoPlayOnSelect: Boolean by remember { mutableStateOf(SessionStore.loadAutoPlayOnSelect(context)) }
    /** Template whose editor the wide Settings page shows; Track rows each time Settings opens (D110). */
    var settingsSelection: TemplateKind by rememberSaveable { mutableStateOf(TemplateKind.entries.first()) }
    /** Controller snapshot the host draws this frame. */
    val state: PlayerUiState = controller.uiState
    /** Playback position and duration sampled for this frame. */
    val playbackProgress: PlaybackProgress = rememberPlaybackProgress(controller)
    /** Measures of the previewed rows of the selected page keyed by load-order index, which Settings shows. */
    val previewMeasures: Map<Int, TrackMeasure> = measuresByIndex(
        pageIndices = state.pageItems.take(TEMPLATE_PREVIEW_ROW_COUNT).map { entry -> entry.index },
        uriAt = controller::uriAt,
        measures = state.measures,
    )
    /** Saved template of the track rows, parsed once by the template cache and applied to every row. */
    val trackRowsTemplate: String = templates.lastValid.getValue(TemplateKind.TRACK_ROWS)
    /** Name of the selected page, which titles the folder that the rows belong to. */
    val pageFolderTitle: String = state.pageLabels.getOrNull(state.selectedPage).orEmpty()
    // What:     `remember(keys) { ... }` keeps the built rows until one of its keys changes.
    // Why:      The progress tick recomposes this host several times a second; rebuilding every row on each tick pinned
    //           the main thread for a 1,000-row folder (S2). Rows depend on the page and the folder title, never on
    //           the position, the measures or the playing row: the analysis publishes measures many times a second,
    //           and rebuilding 10,000 rows for each of them was an ANR, and for each track change a stall of tens of
    //           milliseconds. Each visible row reads its own measure through `rowLines` and its own playing flag
    //           through `LocalPlayingIndex` instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rowEntries = useMemo(() => rowEntriesFor(...), [pageItems, title]);
    // ```
    /** Rows of the selected page without supporting lines or a playing flag, rebuilt only when the page changes. */
    val rowEntries: List<TrackListEntry> = remember(
        state.pageItems,
        pageFolderTitle,
    ) {
        PlayerHostModel.rowEntriesFor(
            folderTitle = pageFolderTitle,
            pageItems = state.pageItems,
            currentIndex = null,
            trackRowsTemplate = trackRowsTemplate,
            withSupportingLines = false,
        )
    }
    /** Library index of the first track on screen, which the measure refresh reads first, or the none value. */
    val firstVisibleTrack: MutableIntState = remember { mutableIntStateOf(NO_VISIBLE_TRACK) }
    /** Source of each visible row's supporting line, which reads that row's own measure when the row is drawn. */
    val rowLines: TrackRowLineSource = remember(state.pageItems, trackRowsTemplate, controller) {
        TrackRowLineSource.forPage(
            pageItems = state.pageItems,
            template = trackRowsTemplate,
            measureOf = { index -> controller.uriAt(index)?.let { uri -> controller.uiState.measures[uri] } },
        )
    }
    /** Rail built from the page names, rebuilt only when a page appears, disappears or is renamed. */
    val railSections: List<RailSection> = remember(state.pageLabels) { railFor(state.pageLabels) }
    /** Track rows of the playing folder in list order, built once per folder and not again for each track change. */
    val playingRowOrder: List<Int> = remember(state.playingPageIndex, state.pageLabels, state.playingItems) {
        playingRowOrderOf(
            folderTitle = state.playingPageIndex?.let { index -> state.pageLabels.getOrNull(index) }.orEmpty(),
            playingItems = state.playingItems,
        )
    }
    /** Playing track and its place in its own folder, which the deck reads even when that folder is not on screen. */
    val playingPlace: PlayingPlace = remember(playingRowOrder, state.playingItems, state.currentIndex) {
        playingPlaceIn(
            playingItems = state.playingItems,
            rowOrder = playingRowOrder,
            currentIndex = state.currentIndex,
        )
    }
    /** Player model built from the controller snapshot and the sampled progress, before the templates apply. */
    val builtModel: PlayerContentModel = buildPlayerContent(
        pageLabels = state.pageLabels,
        selectedPage = state.selectedPage,
        pageItems = state.pageItems,
        playingItems = state.playingItems,
        playingPageIndex = state.playingPageIndex,
        currentIndex = state.currentIndex,
        playing = state.playing,
        mode = state.playbackMode,
        positionSec = playbackProgress.position,
        durationSec = playbackProgress.duration,
        libraryName = libraryName,
        entries = rowEntries,
        sections = railSections,
        playingPlace = playingPlace,
    )
    /** Library tracks of the selected page, each keyed by the URI that its measure is stored under. */
    val pageTracks: List<Track> = remember(state.pageItems, controller.urisSnapshot()) {
        state.pageItems.mapNotNull { entry ->
            controller.uriAt(entry.index)?.let { uri -> Track(uri = uri, displayPath = entry.name) }
        }
    }
    // What:     `LaunchedEffect(staleMeasures, pageTracks)` hands the page and its reads to `pageMeasurePasses`, which
    //           reads the unmeasured tracks, a pass per sweep step.
    // Why:      A new page or an invalidated measure restarts the reading. A sweep step (the sweep reports every few
    //           tracks) only asks for another pass after the running one, because restarting at every step cancelled
    //           the batch in flight and began again at the top, so the rows of a 10,000 track page got their
    //           measures only after the sweep had ended.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => {
    //   const ctl = new AbortController();
    //   pageMeasurePasses(sweepDoneChanges(), pageTracks, () => measures, firstVisibleUri, read, publish, ctl.signal);
    //   return () => ctl.abort();
    // }, [pageTracks, stale]);
    // ```
    /** Tracks the analysis measured again, whose shown measures are stale (D113). */
    val staleMeasures: Set<String> = MeasureInvalidation.pending
    LaunchedEffect(staleMeasures, pageTracks) {
        controller.dropMeasures(MeasureInvalidation.take())
        pageMeasurePasses(
            steps = snapshotFlow { SweepProgress.snapshot.done },
            tracks = pageTracks,
            known = { controller.uiState.measures },
            firstVisibleUri = { controller.uriAt(firstVisibleTrack.intValue) },
            read = { batch -> TrackMeasures.readMeasures(context, batch) },
            publish = { updates -> controller.mergeMeasures(updates) },
            log = { line -> Log.i(MEASURE_TAG, line) },
        )
    }
    /** Measure of the playing track, read from the library-wide measures so the deck shows it on any folder. */
    val playingMeasure: TrackMeasure? = state.currentIndex
        ?.let { index -> controller.uriAt(index) }
        ?.let { uri -> state.measures[uri] }
    /** Player model with the saved track-row and playing-track templates applied. */
    val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
        base = builtModel,
        pageItems = state.pageItems,
        currentIndex = state.currentIndex,
        durationSec = playbackProgress.duration,
        trackRowsTemplate = trackRowsTemplate,
        playingTemplate = templates.lastValid.getValue(TemplateKind.PLAYING_TRACK),
        playingPlace = playingPlace,
        playingMeasure = playingMeasure,
        rowEntries = rowEntries,
    )
    /** Editor view with the preview drawn from the selected page and its measured durations and peaks, for Settings. */
    val editor: TemplateEditorState = PlayerHostModel.editorWithPreview(
        editor = templates,
        pageItems = state.pageItems,
        measuresByIndex = previewMeasures,
    )

    // What:     `fun go(next: PlayerRoute)` moves the host to one destination and saves its token.
    // Why:      Every route change goes through here, so each one is logged and saved the same way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const go = (next: PlayerRoute) => setRouteToken(encode(next));
    // ```
    /** Moves the host to one destination and saves the destination in the instance state. */
    fun go(next: PlayerRoute) {
        Log.i(LOG_TAG, "player host route=${PlayerRouteRules.encode(next)}")
        if (next == PlayerRoute.Settings && route != PlayerRoute.Settings) {
            settingsSelection = TemplateKind.entries.first()
        }
        routeToken = PlayerRouteRules.encode(next)
    }

    // What:     `fun applyTemplateChange(next: TemplateEditorState)` keeps an edited template and saves it when valid.
    // Why:      The rows follow the typed text while it applies, and the store only ever receives a valid template.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const applyTemplateChange = (next: EditorState) => { setTemplates(next); if (next.isValid) save(next); };
    // ```
    /** Keeps one edited template state and saves the open template when its text applies to every preview track. */
    fun applyTemplateChange(next: TemplateEditorState) {
        templates = next
        if (!next.isValid) {
            Log.i(LOG_TAG, "template draft kept unsaved kind=${next.kind.name}")
            return
        }
        SessionStore.saveTemplate(context = context, kind = next.kind, text = next.text)
        Log.i(LOG_TAG, "template saved kind=${next.kind.name}")
    }

    // What:     `fun toggleAutoPlayOnSelect()` flips the Auto-play on select setting and saves it.
    // Why:      The Settings switch row toggles in place (D129); the new value takes effect for the next tap, Enter or
    //           Search result, and survives a restart because the store holds it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const toggleAutoPlayOnSelect = () => { setAutoPlay(!autoPlay); store.saveAutoPlayOnSelect(!autoPlay); };
    // ```
    /** Flips Auto-play on select, keeps the new value in the host state and writes it to the store. */
    fun toggleAutoPlayOnSelect() {
        autoPlayOnSelect = !autoPlayOnSelect
        SessionStore.saveAutoPlayOnSelect(context = context, enabled = autoPlayOnSelect)
        Log.i(LOG_TAG, "auto-play on select set to $autoPlayOnSelect")
    }

    // What:     `fun selectFolder(folder: String)` selects a folder by its name without starting a track.
    // Why:      Picking a folder shows its tracks and never autoplays, and the picker closes on the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const selectFolder = (folder: string) => { controller.selectPage(labels.indexOf(folder)); };
    // ```
    /** Selects the folder with the given name, and closes the picker when a page with that name exists. */
    fun selectFolder(folder: String) {
        /** Tab-order index of the chosen folder, or -1 when no page has that name. */
        val page: Int = state.pageLabels.indexOf(folder)
        if (page >= 0) {
            controller.selectPage(page)
        }
        pickerOpen = false
    }

    // What:     `fun openTemplate(kind: TemplateKind)` opens one template in its editor.
    // Why:      The editor opens on the chosen template with its caret at the end, as the editor state defines.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const openTemplate = (kind) => { setTemplates(editor.open(kind)); go(templateEditor(kind)); };
    // ```
    /** Opens the template editor on one template and moves the host to it. */
    fun openTemplate(kind: TemplateKind) {
        templates = editor.open(kind)
        go(PlayerRoute.TemplateEditor(kind))
    }

    // What:     `fun openSearchResult(result: SearchResult)` acts on one Search result and returns to the player.
    // Why:      A track starts or pauses, a folder selects its page, and each one returns to the player (D72 to D74);
    //           a result the player cannot open stays on Search so the page can say so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const openSearchResult = (result: SearchResult) => { /* activation, then back to player */ };
    // ```
    /**
     * Returns a track result with its load-order index in the library the player holds now, found by the track's URI.
     *
     * The index of a result is the track's number in the Search index, which is its arrival number while a scan runs
     * and so differs from its position in the sorted library. A track the library no longer holds comes back without an
     * index, which the activation rules answer by staying in Search (D74).
     */
    fun locateInLibrary(result: SearchResult): SearchResult {
        if (result.kind != SearchResultKind.TRACK) {
            return result
        }
        /** URI of the result's track, or null for an index built without URIs. */
        val uri: String = result.trackUri ?: return result.copy(indexInLibrary = null)
        /** Position of that URI in the library now, or -1. */
        val position: Int = controller.indexOfUri(uri)
        return result.copy(indexInLibrary = if (position < 0) null else position)
    }

    /** Acts on one Search result, then returns to the player unless the result could not be opened. */
    fun openSearchResult(result: SearchResult) {
        /** The result with its track located in the library as it is now, or without a position when it is gone. */
        val located: SearchResult = locateInLibrary(result)
        /** What activating this result does to the player, decided from the result and the playing track. */
        val activation: SearchActivation = PlayerHostModel.searchActivationFor(
            result = located,
            pageLabels = state.pageLabels,
            currentIndex = state.currentIndex,
        )
        when (activation) {
            // D127, D129: a result follows the row rule, so a track that is not current is only selected (paused at 0),
            // or also started when Auto-play on select is on.
            is SearchActivation.SelectTrack -> controller.activateRow(
                index = activation.indexInLibrary,
                autoPlayOnSelect = autoPlayOnSelect,
            )
            SearchActivation.TogglePlayback -> controller.togglePlay()
            is SearchActivation.OpenFolder -> {
                controller.selectPage(activation.page)
                pickerOpen = false
            }
            SearchActivation.StayInSearch -> {
                Log.w(LOG_TAG, "search result not opened kind=${result.kind.name}")
                return
            }
        }
        query = ""
        go(PlayerRoute.Player)
    }

    // What:     `fun returnFocusToPlayback()` gives focus back to the playback area after a tap on the player.
    // Why:      Shortcuts reach the host only along the focused path, so a tapped row or control must not keep focus.
    //           Search and the editor keep their own focus, so the request runs only on the player route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function returnFocusToPlayback(): void { if (route === "player") rootRef.focus(); }
    // ```
    /** Gives focus to the playback area after a tap on the player, and leaves focus alone on other routes. */
    fun returnFocusToPlayback() {
        if (route == PlayerRoute.Player) {
            rootFocusRequester.requestFocus()
        }
    }

    // What:     `val showPlayingFolder` closes the picker and selects the playing track's page.
    // Why:      The picker replaces the list, and the playing track may sit in another folder, so the folder is
    //           selected first. The row reveal and the folder reveal both need this step before they ask the list
    //           to move.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const showPlayingFolder = () => {
    //   setPickerOpen(false); if (page changed) selectPage(playing);
    // };
    // ```
    /** Closes the folder picker and selects the playing track's page when another page is selected. */
    val showPlayingFolder: () -> Unit = {
        pickerOpen = false
        /** Tab-order page of the playing track, or null when no track is playing. */
        val playingPage: Int? = controller.uiState.playingPageIndex
        if (playingPage != null && playingPage != controller.uiState.selectedPage) {
            Log.i(LOG_TAG, "player host reveal shows playing folder page=$playingPage")
            controller.selectPage(playingPage)
        }
    }

    // What:     `val revealPlayingTrack` shows the playing track's folder and asks its list to focus the row.
    // Why:      The picker replaces the list, and the playing track may sit in another folder, so the folder is
    //           selected first, without starting a track, and then the row can take focus. Ctrl+G and a tap on the
    //           deck's title both run it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const revealPlayingTrack = () => {
    //   showPlayingFolder(); bumpRevealToken();
    // };
    // ```
    /** Shows the playing track's folder, then focuses the playing row without starting a track. */
    val revealPlayingTrack: () -> Unit = {
        showPlayingFolder()
        revealToken += 1
    }

    // What:     `val revealPlayingFolder` shows the playing track's folder and asks its list to scroll to the folder.
    // Why:      A tap on the folder part of the deck heading runs it. The list scrolls to the subfolder header that
    //           holds the playing track, or to its top when the track sits directly in the folder, and no row takes
    //           focus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const revealPlayingFolder = () => {
    //   showPlayingFolder(); bumpFolderRevealToken();
    // };
    // ```
    /** Shows the playing track's folder, then scrolls its list to the playing track's subfolder header. */
    val revealPlayingFolder: () -> Unit = {
        showPlayingFolder()
        folderRevealToken += 1
    }

    // What:     `val deckActions` routes the transport actions to the controller.
    // Why:      The player deck and the deck under the Search page send the same actions, and a tapped control
    //           returns focus to the playback area.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const deckActions = { onSeek: (s) => controller.seek(s), onPrevious: () => controller.prev(), ... };
    // ```
    /** Transport actions the player deck and the Search deck send to the controller. */
    val deckActions = TransportDeckActions(
        onSeek = { position -> controller.seek(position) },
        onPrevious = {
            controller.prev()
            returnFocusToPlayback()
        },
        onTogglePlay = {
            controller.togglePlay()
            returnFocusToPlayback()
        },
        onNext = {
            controller.next()
            returnFocusToPlayback()
        },
        onSelectMode = { mode ->
            controller.setPlaybackMode(mode)
            returnFocusToPlayback()
        },
        onRevealPlaying = revealPlayingTrack,
        onRevealFolder = revealPlayingFolder,
    )

    // What:     `val playerActions` routes the player screen's actions.
    // Why:      The folder picker, Settings, Search, and track rows act on the host or the controller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const playerActions = { onSearch: () => go("search"), onSettings: () => go("settings"), ... };
    // ```
    /** Actions the player screen invokes, routed to the controller or to the host's own state. */
    val playerActions = PlayerContentActions(
        onToggleFolderPicker = {
            pickerOpen = !pickerOpen
            returnFocusToPlayback()
        },
        onSelectFolder = { folder ->
            selectFolder(folder)
            returnFocusToPlayback()
        },
        onOpen = onChooseFolder,
        onSettings = { go(PlayerRoute.Settings) },
        onPlay = { index ->
            Log.i(LOG_TAG, "player host row activated index=$index current=${controller.uiState.currentIndex}")
            controller.activateRow(index = index, autoPlayOnSelect = autoPlayOnSelect)
            returnFocusToPlayback()
        },
        deck = deckActions,
        onSearch = {
            query = ""
            go(PlayerRoute.Search)
        },
    )

    // What:     `val scope: CoroutineScope` runs the library reads and the file operations of the track actions.
    // Why:      The file work suspends, and the scope lets it finish off the main thread while the host keeps drawing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const scope = useCoroutineScope();
    // ```
    /** Coroutine scope of the host, which runs the track actions and the library reads they need. */
    val scope: CoroutineScope = rememberCoroutineScope()
    /** Media-library trash request launcher that the activity provides, or null when none is provided. */
    val mediaStoreTrash: TrashMediaStoreRequest? = LocalTrashMediaStoreRequest.current
    /** Track whose context menu is open, bound to its URI at open time, or null when no menu is open. */
    var menuTarget: TrackMenuTarget? by remember { mutableStateOf<TrackMenuTarget?>(null) }
    /** Window-space bounds of the row that opened the menu, or null when no row has reported its bounds. */
    var menuAnchor: MenuAnchorRect? by remember { mutableStateOf<MenuAnchorRect?>(null) }

    // What:     `fun openMenu(index: Int)` binds the menu to the track at a library index.
    // Why:      The track is read once here, so a later rescan that moves the index cannot redirect the action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // openMenu(index: number): void { menuTarget = { indexInLibrary: index, track: library[index] }; }
    // ```
    /**
     * Opens the context menu for the track at the given library index, anchored to the row's window bounds, and
     * logs when the index is out of range.
     */
    fun openMenu(index: Int, anchor: MenuAnchorRect) {
        /** Track at the index in load order, or null when the index is out of range. */
        val track: Track? = controller.trackAt(index)
        if (track == null) {
            Log.w(LOG_TAG, "player host menu index out of range index=$index")
            return
        }
        Log.i(LOG_TAG, "player host menu opened index=$index")
        menuTarget = TrackMenuTarget(indexInLibrary = index, track = track)
        menuAnchor = anchor
    }

    // What:     `fun showFailure(failure: TrackActionFailure, name: String)` shows one failure in the overlay.
    // Why:      Every failure of a track action reaches the user through the same overlay message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // showFailure(failure, name): void { feedback.showFeedback(trackActionFailureMessage(failure, name)); }
    // ```
    /** Shows the failure message for one track action in the feedback overlay. */
    fun showFailure(failure: TrackActionFailure, name: String) {
        feedback.showFeedback(trackActionFailureMessage(failure, name))
    }

    // What:     `fun attempt(name: String, block: () -> Unit)` runs one synchronous action and shows its failure.
    // Why:      The menu actions that do not move files throw TrackActionException, and this keeps the catch in one
    //           place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function attempt(name: string, block: () => void): void { try { block(); } catch (e) { showFailure(e.failure,
    //   name); } }
    // ```
    /** Runs one track action, logging and showing its failure when it throws a track action exception. */
    fun attempt(name: String, block: () -> Unit) {
        try {
            block()
        } catch (expectedFailure: TrackActionException) {
            Log.w(LOG_TAG, "player host track action failed failure=${expectedFailure.failure}", expectedFailure)
            showFailure(expectedFailure.failure, name)
        }
    }

    // What:     `fun commitTrash(record: TrashRecord)` makes one trash the pending Undo and shows its message.
    // Why:      The record is stored before the message is shown, so the cleanup check always sees it as offered.
    //           A replaced record's copy is removed, because its Undo message is gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // commitTrash(record: TrashRecord): void { const old = store.replace(record); remove(old); show(undo(record)); }
    // ```
    /** Stores one trash as the pending Undo, removes the copy of any record it replaces, and shows its Undo message. */
    fun commitTrash(record: TrashRecord) {
        /** Record that this one replaces, whose Undo message is no longer offered. */
        val replaced: TrashRecord? = TrashUndoStore.replace(record)
        if (replaced != null) {
            Log.i(LOG_TAG, "player host replaced pending trash id=${replaced.id}")
            scope.launch(Dispatchers.IO) { removeCopy(replaced) }
        }
        feedback.showFeedback(trashUndoMessage(record))
        Log.i(LOG_TAG, "player host trash undo offered id=${record.id} route=${record.route.name}")
    }

    // What:     `suspend fun reconcile()` reloads the library and reconciles the controller with it.
    // Why:      A trashed track must leave the list, and a restored track must return to it, so the host rescans after
    //           each change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function reconcile(): Promise<void> { controller.reconcileLibrary(await loadLibrary()); }
    // ```
    /** Reloads the library off the main thread, then reconciles the controller on the main thread. */
    suspend fun reconcile() {
        /** Tracks read from the current library after the change. */
        val tracks: List<Track> = withContext(Dispatchers.IO) { LibrarySource.load(context) }
        controller.reconcileFromScan(tracks)
        Log.i(LOG_TAG, "player host library reconciled tracks=${tracks.size}")
    }

    // What:     `suspend fun trashWithCopy(...)` runs the picked-folder trash and reports its outcome.
    // Why:      The copy-back needs the held grant, and any failure before the delete leaves the file where it was.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function trashWithCopy(target, tree): Promise<void> { ... }
    // ```
    /**
     * Trashes one picked-folder track through the copy route, then reconciles and offers Undo, or shows the failure.
     */
    suspend fun trashWithCopy(target: TrackMenuTarget, treeUri: Uri) {
        /** Unique key for this trash, which names its cache folder and its Undo message. */
        val id: String = UUID.randomUUID().toString()
        try {
            /** Trash record of the picked-folder trash, which the Undo message needs. */
            val record: TrashRecord = trashSaf(context = context, track = target.track, treeUri = treeUri, id = id)
            commitTrash(record)
            reconcile()
        } catch (expectedFailure: TrackActionException) {
            Log.w(LOG_TAG, "player host trash failed failure=${expectedFailure.failure}", expectedFailure)
            showFailure(expectedFailure.failure, trackFileNameOf(target.track.displayPath))
        }
    }

    // What:     `fun handleMediaStoreTrashResult(...)` decides what the system's trash answer means for the library.
    // Why:      A confirmed trash is checked against the trashed flag before the app offers Undo, because Android can
    //           report
    //           success before the item moves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function handleTrashResult(track, accepted: boolean, uri): void { ... }
    // ```
    /** Offers Undo when the item is now trashed, shows a failure when it is not, and logs a declined confirmation. */
    fun handleMediaStoreTrashResult(track: Track, accepted: Boolean, uri: Uri) {
        /** Name of the track, used in any failure message. */
        val name: String = trackFileNameOf(track.displayPath)
        if (!accepted) {
            Log.i(LOG_TAG, "player host media store trash declined by user")
            return
        }
        if (isMediaStoreTrashed(context = context, uri = uri) != true) {
            Log.w(LOG_TAG, "player host media store trash accepted but item not trashed")
            showFailure(TrackActionFailure.MEDIASTORE_FAILED, name)
            return
        }
        commitTrash(
            TrashRecord(
                id = UUID.randomUUID().toString(),
                displayName = name,
                route = TrashRoute.MEDIASTORE_TRASH_REQUEST,
                trackUri = track.uri,
                treeUri = null,
                parentDocumentId = null,
                mimeType = mimeTypeFor(name),
                copyPath = null,
                sizeBytes = 0L,
            ),
        )
        scope.launch { reconcile() }
    }

    // What:     `fun trashInMediaStore(target: TrackMenuTarget)` starts Android's trash request for a media item.
    // Why:      The device-wide library has its own trash, so the app asks Android and waits for its result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function trashInMediaStore(target: TrackMenuTarget): void { system.requestTrash(target.uri, onResult); }
    // ```
    /**
     * Launches Android's trash request for the media item, or shows a failure when no launcher or request is available.
     */
    fun trashInMediaStore(target: TrackMenuTarget) {
        /** Name of the track, used in any failure message. */
        val name: String = trackFileNameOf(target.track.displayPath)
        /** Launcher the activity registered for the media trash request, or null when none is provided. */
        val request: TrashMediaStoreRequest = mediaStoreTrash ?: run {
            Log.w(LOG_TAG, "player host has no media store trash launcher")
            showFailure(TrackActionFailure.MEDIASTORE_FAILED, name)
            return
        }
        /** URI of the media item to trash. */
        val uri: Uri = Uri.parse(target.track.uri)
        try {
            /** Pending intent of Android's trash request for this item. */
            val pending = mediaStoreRequest(context = context, uri = uri, trash = true)
            request.launch(pending.intentSender) { accepted ->
                handleMediaStoreTrashResult(track = target.track, accepted = accepted, uri = uri)
            }
        } catch (expectedFailure: TrackActionException) {
            Log.w(LOG_TAG, "player host media store trash could not start", expectedFailure)
            showFailure(expectedFailure.failure, name)
        }
    }

    // What:     `fun moveToTrash(target: TrackMenuTarget)` chooses the trash route for the target and runs it.
    // Why:      The route depends on the source of the track, so the decision is made here from the held grant.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function moveToTrash(target: TrackMenuTarget): void { switch (trashRouteFor(...)) { ... } }
    // ```
    /** Chooses the trash route of the target track from its URI and the held folder grant, then runs that route. */
    fun moveToTrash(target: TrackMenuTarget) {
        /** Name of the track, used in any failure message. */
        val name: String = trackFileNameOf(target.track.displayPath)
        /** Tree of the held folder grant, or null when no folder is held. */
        val heldTree: Uri? = LibraryRoot.heldRoot(context)
        /** Whether the held grant can still change files. */
        val writable: Boolean = writableGrantCovers(context = context, treeUri = heldTree)
        /** Route chosen for this track from its URI and the grant. */
        val route: TrashRoute = trashRouteFor(
            trackUri = target.track.uri,
            treeUri = heldTree?.toString(),
            writableGrant = writable,
        )
        Log.i(LOG_TAG, "player host trash route=${route.name} index=${target.indexInLibrary}")
        when (route) {
            TrashRoute.UNAVAILABLE -> showFailure(TrackActionFailure.ROUTE_UNAVAILABLE, name)
            TrashRoute.SAF_DELETE_WITH_COPY_BACK -> {
                if (heldTree == null) {
                    showFailure(TrackActionFailure.ROUTE_UNAVAILABLE, name)
                    return
                }
                scope.launch { trashWithCopy(target = target, treeUri = heldTree) }
            }
            TrashRoute.MEDIASTORE_TRASH_REQUEST -> trashInMediaStore(target)
        }
    }

    // What:     `fun handleMediaStoreRestoreResult(...)` reports the system restore answer and keeps Undo when it was
    //           declined.
    // Why:      A declined restore leaves the item in the system trash, so the Undo offer must come back rather than be
    //           lost.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function handleRestoreResult(original, restoring, accepted, uri): void { ... }
    // ```
    /** Finishes the restore as restored or failed, or re-offers Undo when the user declined the system confirmation. */
    fun handleMediaStoreRestoreResult(
        original: TrashRecord,
        restoring: TrashRecord,
        accepted: Boolean,
        uri: Uri,
    ) {
        if (!accepted) {
            Log.i(LOG_TAG, "player host media store restore declined, offering Undo again id=${original.id}")
            commitTrash(original)
            return
        }
        /** Whether the item left the system trash, or null when the flag cannot be read. */
        val stillTrashed: Boolean? = isMediaStoreTrashed(context = context, uri = uri)
        if (stillTrashed != false) {
            Log.w(LOG_TAG, "player host media store restore accepted but item still trashed id=${original.id}")
            restoring.finishRestore(restored = false)
            showFailure(TrackActionFailure.MEDIASTORE_RESTORE_FAILED, restoring.displayName)
            return
        }
        /** Restoring record, whose outcome is recorded once the restore answer arrives. */
        val finished: TrashRecord = restoring.finishRestore(restored = true)
        Log.i(LOG_TAG, "player host undo finished state=${finished.state.name} id=${original.id}")
        scope.launch { reconcile() }
    }

    // What:     `fun restoreInMediaStore(...)` starts Android's restore request for a media item in the system trash.
    // Why:      The system trash owns the restore, so the app asks Android and waits for its answer before it reports
    //           success.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function restoreInMediaStore(record: TrashRecord): void { system.requestRestore(record.uri, onResult); }
    // ```
    /** Launches Android's restore request for the trashed media item, and offers Undo again when the user declines. */
    fun restoreInMediaStore(original: TrashRecord, restoring: TrashRecord) {
        /** Launcher the activity registered for media requests, or null when none is provided. */
        val request: TrashMediaStoreRequest = mediaStoreTrash ?: run {
            Log.w(LOG_TAG, "player host has no media store restore launcher")
            showFailure(TrackActionFailure.MEDIASTORE_RESTORE_FAILED, restoring.displayName)
            return
        }
        /** URI of the trashed media item. */
        val uri: Uri = Uri.parse(restoring.trackUri)
        try {
            /** Pending intent of Android's restore request for this item. */
            val pending = mediaStoreRequest(context = context, uri = uri, trash = false)
            request.launch(pending.intentSender) { accepted ->
                handleMediaStoreRestoreResult(
                    original = original,
                    restoring = restoring,
                    accepted = accepted,
                    uri = uri,
                )
            }
        } catch (expectedFailure: TrackActionException) {
            Log.w(LOG_TAG, "player host media store restore could not start", expectedFailure)
            showFailure(TrackActionFailure.MEDIASTORE_RESTORE_FAILED, restoring.displayName)
        }
    }

    // What:     `suspend fun restoreFromCopy(...)` recreates a picked-folder file from its cache copy.
    // Why:      The copy is the only copy of the bytes, so a failed restore keeps it and says so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function restoreFromCopy(original, restoring): Promise<void> { ... }
    // ```
    /** Restores one picked-folder file, reconciles the library on success, and shows the failure otherwise. */
    suspend fun restoreFromCopy(original: TrashRecord, restoring: TrashRecord) {
        try {
            restoreSaf(context = context, record = restoring)
            /** Restoring record, whose outcome is recorded once the restore finishes. */
            val finished: TrashRecord = restoring.finishRestore(restored = true)
            Log.i(LOG_TAG, "player host undo finished state=${finished.state.name} id=${original.id}")
            reconcile()
        } catch (expectedFailure: TrackActionException) {
            restoring.finishRestore(restored = false)
            Log.w(LOG_TAG, "player host undo failed failure=${expectedFailure.failure}", expectedFailure)
            showFailure(expectedFailure.failure, original.displayName)
        }
    }

    // What:     `fun startUndo(messageId: String)` runs the Undo of the pending trash whose message carries the id.
    // Why:      The record leaves the store before its restore starts, so the cleanup check cannot delete its copy
    //           mid-restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // startUndo(messageId: string): void { const record = store.takeForRestore(messageId); ... }
    // ```
    /** Takes the pending record whose message carries the id, dismisses its message, and runs its restore by route. */
    fun startUndo(messageId: String) {
        /** Pending record taken for the restore, or null when the id matches no pending Undo. */
        val original: TrashRecord = TrashUndoStore.takeForRestore(messageId) ?: run {
            Log.w(LOG_TAG, "player host undo ignored, no pending trash for id=$messageId")
            return
        }
        feedback.dismiss(messageId)
        /** The record in the restoring state, whose outcome is recorded once the restore finishes. */
        val restoring: TrashRecord = original.beginRestore()
        Log.i(LOG_TAG, "player host undo started id=${original.id} route=${original.route.name}")
        when (original.route) {
            TrashRoute.SAF_DELETE_WITH_COPY_BACK ->
                scope.launch { restoreFromCopy(original = original, restoring = restoring) }
            TrashRoute.MEDIASTORE_TRASH_REQUEST -> restoreInMediaStore(original = original, restoring = restoring)
            TrashRoute.UNAVAILABLE -> showFailure(TrackActionFailure.ROUTE_UNAVAILABLE, original.displayName)
        }
    }

    // What:     `fun showDetails(target: TrackMenuTarget)` shows the name, size and folder of one track file as a
    //           notice.
    // Why:      M5: choosing File details did nothing. The desktop shows these facts as a notice too; the size comes
    //           from the file provider, so it is read off the main thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function showDetails(target) { feedback.show(trackDetailsMessage(name, await sizeOf(target), path)); }
    // ```
    /** Reads the size of the track file and shows its name, size and folder as a notice. */
    fun showDetails(target: TrackMenuTarget) {
        scope.launch {
            /** Size the provider reports for the file, or null when it reports none. */
            val size: Long? = withContext(Dispatchers.IO) {
                trackFileSizeBytes(context = context, track = target.track)
            }
            Log.i(LOG_TAG, "player host file details shown index=${target.indexInLibrary} sizeKnown=${size != null}")
            feedback.showFeedback(
                trackDetailsMessage(
                    fileName = trackFileNameOf(target.track.displayPath),
                    sizeBytes = size,
                    displayPath = target.track.displayPath,
                ),
            )
        }
    }

    // What:     `fun runMenuAction(...)` routes one selected menu action to its implementation.
    // Why:      Each action has one owner, and the two file-moving actions run on the host's scope while the rest run
    //           here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function runMenuAction(action: TrackMenuAction, target: TrackMenuTarget): void { switch (action) { ... } }
    // ```
    /** Runs the implementation of one selected menu action for the bound target track. */
    fun runMenuAction(action: TrackMenuAction, target: TrackMenuTarget) {
        Log.i(LOG_TAG, "player host menu action=${action.name} index=${target.indexInLibrary}")
        /** Name of the bound track, used in any failure message. */
        val name: String = trackFileNameOf(target.track.displayPath)
        when (action) {
            TrackMenuAction.PLAY -> controller.playIndex(target.indexInLibrary)
            TrackMenuAction.SHUFFLE_FROM_HERE ->
                shuffleFromHere(controller = controller, indexInLibrary = target.indexInLibrary)
            TrackMenuAction.DETAILS -> showDetails(target)
            TrackMenuAction.SHOW_IN_FILE_MANAGER ->
                attempt(name) { showInFileManager(context = context, track = target.track) }
            TrackMenuAction.COPY_FILENAME -> attempt(name) { copyFilename(context = context, track = target.track) }
            TrackMenuAction.MOVE_TO_TRASH -> moveToTrash(target)
        }
    }

    // What:     `LaunchedEffect(controller.loadFailure)` shows a message when the engine could not open a track.
    // Why:      M6: the file open runs off the main thread and can stall, so the engine reports its failure later and
    //           the host words it for the user.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (loadFailure) feedback.show(trackLoadFailureMessage(reason, name)); }, [loadFailure]);
    // ```
    /** The newest failure to open a track file, or null when the last load worked. */
    val loadFailure: LoadFailureNotice? = controller.loadFailure
    LaunchedEffect(loadFailure) {
        if (loadFailure != null) {
            /** Display path of the track that failed, or its URI when the library no longer lists it. */
            val shownName: String = controller.trackAt(controller.indexOfUri(loadFailure.uri))?.displayPath
                ?: loadFailure.uri
            feedback.showFeedback(
                trackLoadFailureMessage(reason = loadFailure.reason, fileName = trackFileNameOf(shownName)),
            )
        }
    }

    // What:     `val visibleIds: Set<String>` holds the ids of the feedback messages shown this frame.
    // Why:      A pending trash whose message is gone has no Undo left, so its copy is released by the effect below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const visibleIds = new Set(feedback.visible().map(m => m.id));
    // ```
    /** Ids of the feedback messages visible in this frame. */
    val visibleIds: Set<String> = feedback.queue.visible().map { message -> message.id }.toSet()
    // What:     `LaunchedEffect(visibleIds)` releases the pending trash when its Undo message is no longer visible.
    // Why:      Expiry, dismissal, and replacement all remove the message, and only then is the copy no longer needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { const dropped = store.dropIfHidden(visibleIds); if (dropped) remove(dropped); }, [visibleIds]);
    // ```
    LaunchedEffect(visibleIds) {
        /** Pending record whose message is gone, or null when its message is still visible or none is pending. */
        val dropped: TrashRecord? = TrashUndoStore.dropIfHidden(visibleIds)
        if (dropped != null) {
            Log.i(LOG_TAG, "player host undo expired, releasing copy id=${dropped.id}")
            withContext(Dispatchers.IO) { removeCopy(dropped) }
        }
    }
    // What:     `LaunchedEffect(Unit)` runs the startup sweep of earlier copies once per process.
    // Why:      Copies left by an earlier process have no Undo message, so they are removed before any new trash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (claimStartupSweep()) purge(); }, []);
    // ```
    LaunchedEffect(Unit) {
        if (TrashUndoStore.claimStartupSweep()) {
            withContext(Dispatchers.IO) { purgeTrashCopies(context) }
        }
    }

    // What:     `val keyLayout: HostKeyLayout` names the layout the screen draws at the current width.
    // Why:      Ctrl+O opens the folder picker only on the folded cover, so the keyboard rules need the layout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyLayout = isCoverLayout(window.innerWidth) ? "folded-cover" : "unfolded";
    // ```
    /** Layout the screen draws at the current window width, which the keyboard rules read. */
    val keyLayout: HostKeyLayout = if (isCoverLayout(LocalConfiguration.current.screenWidthDp.toFloat())) {
        HostKeyLayout.FOLDED_COVER
    } else {
        HostKeyLayout.UNFOLDED
    }
    // What:     `val settingsIsWide: Boolean` says whether Settings draws its two panes.
    // Why:      D110 gives Settings two panes under the same condition as the player's two panes, and the editor then
    //           sits inside the Settings page, so a focused editor field is a text field on that page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const settingsIsWide = !isCoverLayout(window.innerWidth);
    // ```
    /** Whether the window is wide enough for the two-pane Settings page. */
    val settingsIsWide: Boolean = keyLayout == HostKeyLayout.UNFOLDED
    /** Crease of this window from the Jetpack WindowManager layout, shared by Settings and Search (D110, E2). */
    val crease: CreaseState = rememberCrease()
    /** Crease width in physical pixels when the window has a crease, else null. */
    val creaseWidthPx: Float? = crease.widthPxOrNull
    // What:     `val keyContext: HostKeyContext` is the host state that the keyboard rules read for this frame.
    // Why:      The owner, the route rules, the layout, and the mode cycle each need the route, popup, focus, and mode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyContext = { route, menuOpen: menu != null, rootFocused, textFieldFocused, layout, mode };
    // ```
    /** Host state that the keyboard rules read for each key-down in this frame. */
    val keyContext = HostKeyContext(
        route = hostKeyRouteOf(route),
        menuOpen = menuTarget != null,
        textFieldFocused = (route == PlayerRoute.Search && searchFieldFocused) ||
            ((route is PlayerRoute.TemplateEditor || (route == PlayerRoute.Settings && settingsIsWide)) &&
                templates.fieldFocused),
        rootFocused = rootFocused,
        playbackMode = state.playbackMode,
        layout = keyLayout,
        zone = keyZone,
        pickerOpen = pickerOpen,
    )

    // What:     `val keyActions: HostKeyActions` runs each keyboard command through its on-screen control's path.
    // Why:      A shortcut must do exactly what its button does, so each operation reuses that control's own call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyActions = { togglePlayback: () => controller.togglePlay(), openSearch: () => go("search"), ... };
    // ```
    /** Host operations that the keyboard commands run, each through the same path as its on-screen control. */
    val keyActions: HostKeyActions = object : HostKeyActions {
        // What:     `override fun togglePlayback()` runs the deck's play and pause call.
        // Why:      Space and the deck's play button share one controller call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // togglePlayback(): void { controller.togglePlay(); }
        // ```
        /** Plays or pauses the player, as the deck's play button does. */
        override fun togglePlayback() {
            controller.togglePlay()
        }

        // What:     `override fun previousTrack()` runs the deck's previous call.
        // Why:      The backward chord and the deck's previous button share one controller call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // previousTrack(): void { controller.prev(); }
        // ```
        /** Moves to the previous track, as the deck's previous button does. */
        override fun previousTrack() {
            controller.prev()
        }

        // What:     `override fun nextTrack()` runs the deck's next call.
        // Why:      The forward chord and the deck's next button share one controller call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // nextTrack(): void { controller.next(); }
        // ```
        /** Moves to the next track, as the deck's next button does. */
        override fun nextTrack() {
            controller.next()
        }

        // What:     `override fun cycleMode(mode: PlaybackMode)` runs the deck's mode call with the computed mode.
        // Why:      The mode segments and the chord both set the mode through the same controller call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // cycleMode(mode: PlaybackMode): void { controller.setPlaybackMode(mode); }
        // ```
        /** Applies the end-of-track mode that the chord computed. */
        override fun cycleMode(mode: PlaybackMode) {
            controller.setPlaybackMode(mode)
        }

        // What:     `override fun openSearch()` starts a new Search visit with an empty query.
        // Why:      The Search button clears the query and opens Search, and the chord must do the same.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // openSearch(): void { setQuery(""); go("search"); }
        // ```
        /** Opens Search as a new visit with an empty query, as the Search button does. */
        override fun openSearch() {
            query = ""
            go(PlayerRoute.Search)
        }

        // What:     `override fun focusSearch()` asks the Search page to refocus its field without clearing it.
        // Why:      A repeated Search chord keeps the query, so only the focus moves.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // focusSearch(): void { searchFocusPort.press(); }
        // ```
        /** Refocuses the open Search page's query field and puts its caret at the end, keeping the query text. */
        override fun focusSearch() {
            searchFocusPort.press()
        }

        // What:     `override fun toggleFolderPicker()` flips the picker flag, as the folder button does.
        // Why:      The picker button and the chord share one flag, so either one closes what the other opened.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // toggleFolderPicker(): void { setPickerOpen(!pickerOpen); }
        // ```
        /** Opens or closes the folder picker, as the folder button does. */
        override fun toggleFolderPicker() {
            pickerOpen = !pickerOpen
        }

        // What:     `override fun openSettings()` moves the host to the Settings route.
        // Why:      The Settings button and the chord move to the same destination.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // openSettings(): void { go("settings"); }
        // ```
        /** Opens the Settings page, as the Settings button does. */
        override fun openSettings() {
            go(PlayerRoute.Settings)
        }

        // What:     `override fun revealCurrent()` shows the playing track's folder and asks its list to focus the row.
        // Why:      The picker replaces the list, and the playing track may sit in another folder, so the folder is
        //           selected first, without starting a track, and then the row can take focus.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // revealCurrent(): void { setPickerOpen(false); if (page changed) selectPage(playing); bumpRevealToken(); }
        // ```
        /** Shows the playing track's folder, then focuses the playing row without starting a track. */
        override fun revealCurrent() {
            revealPlayingTrack()
        }

        // What:     `override fun backFromSearch()` leaves Search by the same route rule as the system Back button.
        // Why:      Escape and Back must lead to the same destination, so both call the one route rule.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // backFromSearch(): void { go(backFrom(route)); }
        // ```
        /** Leaves Search by the route rule that the system Back handler uses. */
        override fun backFromSearch() {
            go(PlayerRouteRules.backFromHost(route = route, settingsIsWide = settingsIsWide))
        }

        // What:     `override fun closePopup()` clears the bound track so the menu closes.
        // Why:      The menu closes the same way when its dismiss callback runs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // closePopup(): void { setMenuTarget(null); }
        // ```
        /** Closes the open track menu, as its dismiss callback does. */
        override fun closePopup() {
            menuTarget = null
        }

        // What:     `override fun seekBy(seconds: Int)` moves the playhead by a number of seconds.
        // Why:      Alt+Left and Alt+Right seek through the controller, the same call the seek slider makes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // seekBy(seconds: number): void { controller.seek(seekTargetSec(pos, dur, seconds)); }
        // ```
        /** Seeks by [seconds] inside the playing track, and does nothing while no track is loaded. */
        override fun seekBy(seconds: Int) {
            if (state.currentIndex == null) {
                return
            }
            controller.seek(
                seekTargetSec(
                    positionSec = controller.positionSec(),
                    durationSec = controller.durationSec(),
                    deltaSec = seconds,
                ),
            )
        }

        // What:     `override fun restartTrack()` returns the playhead to the start.
        // Why:      Ctrl+Home seeks to zero, the same call as dragging the slider to its start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // restartTrack(): void { controller.seek(0); }
        // ```
        /** Seeks to the start of the playing track, and does nothing while no track is loaded. */
        override fun restartTrack() {
            if (state.currentIndex != null) {
                controller.seek(0.0)
            }
        }

        // What:     `override fun stepFolder(step: Int)` selects the folder a number of places away.
        // Why:      Alt+Up and Alt+Down walk the folder order through the same call as tapping a folder name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // stepFolder(step: number): void {
        //   const label = adjacent(labels, selected, step); if (label) selectFolder(label);
        // }
        // ```
        /** Selects the folder [step] places from the selected one and stops at the first and last folder. */
        override fun stepFolder(step: Int) {
            adjacentFolderLabel(labels = state.pageLabels, selected = state.selectedPage, step = step)
                ?.let { label -> selectFolder(label) }
        }

        // What:     `override fun revealPlayingTrackFolder()` shows the folder of the playing track.
        // Why:      Ctrl+Shift+G runs the same reveal as a tap on the folder part of the deck heading.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // revealPlayingTrackFolder(): void { revealPlayingFolder(); }
        // ```
        /** Shows the playing track's folder and scrolls the track list to it. */
        override fun revealPlayingTrackFolder() {
            revealPlayingFolder()
        }

        // What:     `override fun focusZone(zone: HostKeyZone)` asks a list to take keyboard focus.
        // Why:      Alt+1 and Alt+2 move focus to the folder rail and the track list. On the folded cover only one of
        //           them is drawn at a time, so the chord first shows the one it asks for.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // focusZone(zone: HostKeyZone): void { if (cover) setPickerOpen(zone === "folders"); pendingFocus = zone; }
        // ```
        /** Records the focus request, which the list serves once it is composed. */
        override fun focusZone(zone: HostKeyZone) {
            if (keyLayout == HostKeyLayout.FOLDED_COVER) {
                pickerOpen = zone == HostKeyZone.FOLDERS || zone == HostKeyZone.NAMES
            }
            pendingFocus = zone
        }

        // What:     `override fun openFolderDialog()` opens the system folder chooser.
        // Why:      Ctrl+Shift+O does what the Open button does.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // openFolderDialog(): void { playerActions.onOpen(); }
        // ```
        /** Opens the system folder chooser through the Open button's own action. */
        override fun openFolderDialog() {
            playerActions.onOpen()
        }

        // What:     `override fun closePicker()` closes the folder picker on the folded cover.
        // Why:      Escape and the system Back button close the picker the same way.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // closePicker(): void { setPickerOpen(false); }
        // ```
        /** Closes the folder picker, as the Back button does. */
        override fun closePicker() {
            pickerOpen = false
        }
    }

    // What:     `val keyDispatcher: HostKeyDispatcher` runs the command that each consumed key-down resolves to.
    // Why:      The root keyboard modifier hands each command to this dispatcher, which calls the actions above.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyDispatcher = new HostKeyDispatcher(keyActions);
    // ```
    /** Runs the host command for each consumed key-down. */
    val keyDispatcher = HostKeyDispatcher(actions = keyActions)

    // What:     `val keyBinding: HostKeyBinding` joins the root requester, the host state, and the dispatcher.
    // Why:      The root modifier reads one binding per frame, so each key-down sees the current state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyBinding = { focusRequester: rootRef, context: keyContext, dispatcher: keyDispatcher };
    // ```
    /** Focus requester, host state, and dispatcher that the root keyboard modifier reads. */
    val keyBinding = HostKeyBinding(
        focusRequester = rootFocusRequester,
        context = keyContext,
        dispatcher = keyDispatcher,
    )

    // What:     `LaunchedEffect(route, menuTarget == null)` gives focus to the playback area when the player shows.
    // Why:      Key events reach the host only along the focused path, so the root must hold focus on the player.
    //           It takes focus only when no control holds it, so buttons and rows keep their own focus, and it is
    //           declared before the page, so a page that asks for focus (the Search field) still wins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (route === "player" && !hostHasFocus) rootRef.focus(); }, [route, menuOpen]);
    // ```
    /** Requests focus for the playback area when the player is shown and no control holds focus. */
    LaunchedEffect(route, menuTarget == null) {
        if (route == PlayerRoute.Player && !hostHasFocus) {
            rootFocusRequester.requestFocus()
        }
    }

    BackHandler(enabled = route != PlayerRoute.Player) {
        // The wide Settings page shows the editor in place, so Back closes Settings from either route; the Licenses
        // page returns to Settings.
        go(PlayerRouteRules.backFromHost(route = route, settingsIsWide = settingsIsWide))
    }

    // What:     `LaunchedEffect(nextExpiry)` waits until the earliest visible message is due and then expires it.
    // Why:      The effect restarts whenever the earliest expiry changes, so a shown or dismissed message is timed
    //           again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (nextExpiry != null) feedback.expireAt(nextExpiry); }, [nextExpiry]);
    // ```
    /** Clock value at which the earliest visible message expires, or null when no message is visible. */
    val nextExpiry: Long? = feedback.queue.nextExpiryMs()
    LaunchedEffect(nextExpiry) {
        if (nextExpiry != null) {
            feedback.expireAt(nextExpiry)
        }
    }

    // Root of the player host: it holds the playback-area focus and the keyboard rules, and it provides the
    // Search refocus, reveal, and field-focus locals that the pages read.
    Box(
        modifier = Modifier
            .fillMaxSize()
            .onFocusChanged { focusState ->
                rootFocused = focusState.isFocused
                hostHasFocus = focusState.hasFocus
            }
            .hostKeyboard(binding = keyBinding),
    ) {
        CompositionLocalProvider(
            HostKeyLocalSearchFocus provides searchFocusPort,
            HostKeyLocalReveal provides revealToken,
            HostKeyLocalRevealFolder provides folderRevealToken,
            HostKeyLocalSearchField provides reportSearchFieldFocus,
            HostKeyLocalContext provides keyContext,
            HostKeyLocalZone provides reportKeyZone,
            HostKeyLocalFocusRequest provides pendingFocus,
            HostKeyLocalFocusHandled provides clearPendingFocus,
            LocalTrackRowKeyActions provides { action: TrackMenuAction, index: Int ->
                controller.trackAt(index)?.let { track ->
                    runMenuAction(action = action, target = TrackMenuTarget(indexInLibrary = index, track = track))
                }
            },
        ) {
            when (route) {
                PlayerRoute.Player -> {
                    Box(modifier = Modifier.fillMaxSize()) {
                        if (firstRunContent != null) {
                            // The first-run text starts in the right half, so it reads the right pane's clearance.
                            paneClearanceProvider(
                                clearance = rightPaneClearance(
                                    creaseWidthPx = creaseWidthPx,
                                    density = LocalDensity.current.density,
                                ),
                            ) {
                                firstRunContent { go(PlayerRoute.Settings) }
                            }
                        } else {
                            CompositionLocalProvider(
                                LocalTrackRowMenuOpener provides { index, anchor -> openMenu(index, anchor) },
                                LocalTrackRowLines provides rowLines,
                                LocalPlayingIndex provides state.currentIndex,
                                LocalFirstVisibleTrack provides firstVisibleTrack,
                            ) {
                                playerContent(
                                    model = model,
                                    pickerOpen = pickerOpen,
                                    actions = playerActions,
                                    creaseWidthPx = creaseWidthPx,
                                )
                            }
                            rightPaneOnCreasedScreen(keyLayout = keyLayout, creaseWidthPx = creaseWidthPx) {
                                emptyNotice(state)
                            }
                        }
                        /** Start inset of the track menu: none on the folded cover, the inner-pane inset otherwise. */
                        val menuStartInset: Dp = if (keyLayout == HostKeyLayout.FOLDED_COVER) {
                            0.dp
                        } else {
                            TRACK_MENU_INNER_INSET
                        }
                        /** Track whose context menu is open in this frame, or null when no menu is shown. */
                        val openTarget: TrackMenuTarget? = menuTarget
                        if (openTarget != null) {
                            // The menu sits under the row it opened from, and the window origin stands in when no row
                            // has reported its bounds.
                            trackMenuPopup(
                                actions = trackMenuActionsFor(),
                                title = trackFileNameOf(openTarget.track.displayPath),
                                detailsValue = measureDetailsValue(state.measures[openTarget.track.uri]),
                                anchor = menuAnchor ?: MenuAnchorRect(left = 0, top = 0, right = 0, bottom = 0),
                                startInset = menuStartInset,
                                onSelect = { action ->
                                    menuTarget = null
                                    runMenuAction(action = action, target = openTarget)
                                },
                                onDismiss = { menuTarget = null },
                            )
                        }
                        // The feedback overlay floats at the bottom edge and never changes the layout beneath it.
                        // True-peak analysis shows no in-app indicator (D107): its progress is a system notification,
                        // so nothing here reserves space for it.
                        // On the unfolded screen with a crease the overlay spans the left pane only, ending before the
                        // crease band, so its text never crosses the fold (E2, D110).
                        Column(
                            modifier = Modifier.align(Alignment.BottomStart).feedbackOverlayWidth(
                                keyLayout = keyLayout,
                                creaseWidthPx = creaseWidthPx,
                            ),
                        ) {
                            /**
                             * Messages in display order, Undo above Error, drawn only while at least one is visible.
                             */
                            val messages: List<FeedbackMessage> = feedback.queue.visible()
                            if (messages.isNotEmpty()) {
                                feedbackOverlay(
                                    messages = messages,
                                    onDismiss = { id -> feedback.dismiss(id) },
                                    onAction = { id ->
                                        Log.i(LOG_TAG, "feedback action id=$id")
                                        startUndo(id)
                                    },
                                    modifier = Modifier.padding(
                                        start = PLAYER_HOST_FEEDBACK_INSET,
                                        end = PLAYER_HOST_FEEDBACK_INSET,
                                        bottom = PLAYER_HOST_FEEDBACK_INSET,
                                    ),
                                )
                            }
                        }
                    }
                }
                PlayerRoute.Search -> {
                    /** The process's Search service, or null when nothing can search. */
                    val searchHost: SearchHost? = controller.searchHost
                    // Each visit starts clean (D66), and each edit of the query asks the service for an answer on
                    // its own worker thread, so no keystroke ranks anything in composition.
                    LaunchedEffect(searchHost) { searchHost?.beginVisit() }
                    LaunchedEffect(searchHost, query) { searchHost?.ask(query) }
                    /** Newest answer of this visit, which keeps the page current as a scan adds tracks (D111). */
                    val answer: SearchAnswer? = searchHost?.shownAnswer
                    /** State the page draws for the typed query. */
                    val results: SearchState = searchStateFor(
                        query = query,
                        answer = answer,
                        hasService = searchHost != null,
                    )
                    /** Playing track's URI, which marks its row among the results. */
                    val playingUri: String? = state.currentIndex?.let { index -> controller.uriAt(index) }
                    CompositionLocalProvider(LocalAutoPlayOnSelect provides autoPlayOnSelect) {
                        searchPage(
                            creaseWidthPx = creaseWidthPx,
                            state = results,
                            query = query,
                            onQueryChange = { text -> query = text },
                            onBack = { go(PlayerRoute.Player) },
                            onClear = { query = "" },
                            onOpenResult = { result -> openSearchResult(result) },
                            currentTrackIndex = playingResultId(results, playingUri),
                            deck = {
                                transportDeck(
                                    model = model.deck,
                                    actions = deckActions.copy(onRevealPlaying = null, onRevealFolder = null),
                                )
                            },
                            isPlaying = state.playing,
                            complete = answer?.complete ?: true,
                            folderBrowser = { browserModifier ->
                                // The retained browser keeps its header (D51), which also pads for the status bar, so
                                // the rail letters never draw over the clock while the keyboard shortens the viewport.
                                Column(modifier = browserModifier) {
                                    unfoldedFoldersHeader(title = model.libraryName, onOpen = playerActions.onOpen)
                                    folderPickerPane(
                                        sections = model.sections,
                                        currentFolder = model.currentFolder,
                                        onSelectFolder = { folder -> selectFolder(folder) },
                                        modifier = Modifier.weight(1f),
                                    )
                                }
                            },
                        )
                    }
                }
                PlayerRoute.Settings, is PlayerRoute.TemplateEditor -> {
                    /** Template kind a restored editor route names, or null when the route is Settings itself. */
                    val routeKind: TemplateKind? = (route as? PlayerRoute.TemplateEditor)?.kind
                    /** Template whose editor is shown: the route's own, else the wide page's selection. */
                    val shownKind: TemplateKind = routeKind ?: settingsSelection
                    // What:     `fun actionsFor(opened)` builds the editor callbacks that act on one opened template.
                    // Why:      The wide page keeps two editors alive while it cross-fades, and each one must edit its
                    //           own template.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const actionsFor = (opened) => ({ onEdit: (e) => apply(opened.withEdit(e)), ... });
                    // ```
                    /** Callbacks the editor reports to the host for the template that [opened] is open on. */
                    fun actionsFor(opened: TemplateEditorState) = TemplateEditorActions(
                        onEdit = { edit -> applyTemplateChange(opened.withEdit(edit)) },
                        onFocusChanged = { focused -> templates = opened.withFocus(focused) },
                        onInsertField = { mode -> applyTemplateChange(opened.insertField(mode)) },
                        onReset = { applyTemplateChange(opened.resetToDefault()) },
                        onBack = { go(PlayerRoute.Settings) },
                    )

                    /**
                     * Editor view opened on one template, which differs from the stored editor after a restore.
                     */
                    fun openedOn(kind: TemplateKind): TemplateEditorState =
                        if (editor.kind == kind) editor else editor.open(kind)

                    if (settingsIsWide) {
                        settingsWidePage(
                            templateLines = editor.settingsLines,
                            selected = shownKind,
                            onSelect = { kind ->
                                if (routeKind != null) {
                                    go(PlayerRoute.Settings)
                                }
                                settingsSelection = kind
                                templates = editor.open(kind)
                            },
                            autoPlayOnSelect = autoPlayOnSelect,
                            onToggleAutoPlay = { toggleAutoPlayOnSelect() },
                            onOpenLicenses = { go(PlayerRoute.Licenses) },
                            onBack = { go(PlayerRoute.Player) },
                            creaseWidthPx = creaseWidthPx,
                            editor = { kind, startInset ->
                                /** Editor view of the template this pane child shows. */
                                val paneState: TemplateEditorState = openedOn(kind)
                                templateEditor(
                                    state = paneState,
                                    actions = actionsFor(paneState),
                                    embedded = true,
                                    startInset = startInset,
                                )
                            },
                        )
                    } else if (routeKind != null) {
                        /** Editor view of the route's template. */
                        val pageState: TemplateEditorState = openedOn(shownKind)
                        templateEditor(state = pageState, actions = actionsFor(pageState))
                    } else {
                        settingsPageContent(
                            templateLines = editor.settingsLines,
                            autoPlayOnSelect = autoPlayOnSelect,
                            onToggleAutoPlay = { toggleAutoPlayOnSelect() },
                            onBack = { go(PlayerRoute.Player) },
                            onOpenTemplate = { kind -> openTemplate(kind) },
                            onOpenLicenses = { go(PlayerRoute.Licenses) },
                        )
                    }
                }
                PlayerRoute.Licenses -> {
                    // On a creased unfolded screen the page sits in the right half, so no text crosses the fold.
                    rightPaneOnCreasedScreen(keyLayout = keyLayout, creaseWidthPx = creaseWidthPx) {
                        licensesPage(
                            readNotices = { readLicenseNotices(context) },
                            onBack = { go(PlayerRoute.Settings) },
                        )
                    }
                }
            }
        }
    }
}
