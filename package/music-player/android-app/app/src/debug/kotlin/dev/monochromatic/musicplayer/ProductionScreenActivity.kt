// Debug-only host that renders the production player screen over authored sample data, for screenshots.
// It starts no service, engine, or sweep, and it does not touch the production controller.

// What:     `package dev.monochromatic.musicplayer` places the host beside the production screen it renders.
// Why:      The host calls `playerContent`, `isCoverLayout`, and `LOG_TAG` from the same module.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.os.Bundle` types the saved instance state that `onCreate` receives.
// Why:      The activity lifecycle hands its saved state to `onCreate` as a Bundle.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Bundle } from "android/os";
// ```
import android.os.Bundle

// What:     `import android.util.Log` writes the composition line the debug run is checked against.
// Why:      The screenshot run confirms which layout and picker state the host composed.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.activity.ComponentActivity` is the base class of the host activity.
// Why:      The host uses Compose through the same base class as the production activity.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { ComponentActivity } from "androidx/activity";
// ```
import androidx.activity.ComponentActivity

// What:     `import androidx.activity.enableEdgeToEdge` draws the host behind the system bars.
// Why:      The production activity does the same, so the screenshots show the same edges.
//
// In TS you'd write (pseudocode):
// ```ts
// import { enableEdgeToEdge } from "androidx/activity";
// ```
import androidx.activity.enableEdgeToEdge

// What:     `import androidx.activity.compose.setContent` mounts the Compose tree in the activity.
// Why:      The host draws its screen inside `setContent`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { setContent } from "androidx/activity/compose";
// ```
import androidx.activity.compose.setContent

// What:     `import androidx.compose.foundation.layout.BoxWithConstraints` measures the width the layout uses.
// Why:      The host logs the same cover or unfolded choice the production screen makes for that width.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BoxWithConstraints } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding

// What:     `import androidx.compose.foundation.layout.fillMaxSize` makes the host fill the screen.
// Why:      The production screen expects the full window as its space.
//
// In TS you'd write (pseudocode):
// ```ts
// import { fillMaxSize } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.fillMaxSize

// What:     `import androidx.compose.material3.Surface` is the themed background container.
// Why:      The host wraps its scenes in one, as the production activity does, so text inherits the right color.
import androidx.compose.material3.Surface

// What:     `import androidx.compose.runtime.Composable` marks the functions that emit UI.
// Why:      The host's screen function emits UI.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Composable } from "compose/runtime";
// ```
import androidx.compose.runtime.Composable

// What:     `import androidx.compose.runtime.CompositionLocalProvider` hands one value to every composable below it.
// Why:      The scenes name the playing track through the same composition local the production host provides.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Provider } from "react"; // a context provider
// ```
import androidx.compose.runtime.CompositionLocalProvider

// What:     `import androidx.compose.runtime.getValue` and `setValue` let a delegated state be read and written.
// Why:      The host keeps the picker and the selected page in delegated state.
//
// In TS you'd write (pseudocode):
// ```ts
// // State delegates are read and written by property access.
// ```
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

// What:     `import androidx.compose.ui.Modifier` sizes the host's outer box.
// Why:      The outer box fills the window so the width measurement matches the production screen.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Modifier } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `import dev.monochromatic.musicplayer.core.Page` and `paginate` build the authored pages.
// Why:      The sample library goes through the same pagination the production controller uses.
//
// In TS you'd write (pseudocode):
// ```ts
// import { paginate, type Page } from "./core/Pagination";
// ```
import dev.monochromatic.musicplayer.core.MenuAnchorRect
import dev.monochromatic.musicplayer.core.Page
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.PlayerContentModel
import dev.monochromatic.musicplayer.core.buildPlayerContent
import dev.monochromatic.musicplayer.core.paginate

// What:     `import dev.monochromatic.musicplayer.core.SearchState` and the Search and template types the scenes draw.
// Why:      The Search scenes build their state from the sample library, and the template scenes open the editor.
//
// In TS you'd write (pseudocode):
// ```ts
// import { searchLibrary, SearchState, TemplateEditorState, TemplateKind } from "./core";
// ```
import dev.monochromatic.musicplayer.core.SearchState
import dev.monochromatic.musicplayer.core.TemplateEditorState
import dev.monochromatic.musicplayer.core.FeedbackKind
import dev.monochromatic.musicplayer.core.FeedbackMessage
import dev.monochromatic.musicplayer.core.FeedbackQueue
import dev.monochromatic.musicplayer.core.FirstRunState
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.core.searchLibrary
import dev.monochromatic.musicplayer.core.TrashRecord
import dev.monochromatic.musicplayer.core.TrashRoute
import dev.monochromatic.musicplayer.core.trackFileNameOf
import dev.monochromatic.musicplayer.core.trackMenuActionsFor
import dev.monochromatic.musicplayer.core.trashUndoMessage

// What:     `const val EXTRA_PICKER: String = "picker"` is the intent extra that starts the cover picker open.
// Why:      The screenshot runs need the picker open without tapping, so the extra sets the starting state.
//
// In TS you'd write (pseudocode):
// ```ts
// const EXTRA_PICKER = "picker";
// ```
/** Intent extra name for opening the folder picker at launch. */
private const val EXTRA_PICKER: String = "picker"

// What:     `private const val EXTRA_SCENE: String = "scene"` is the intent extra that picks the scene to draw.
// Why:      The screenshot runs reach each Search, Settings, and editor state by name, without tapping through.
//
// In TS you'd write (pseudocode):
// ```ts
// const EXTRA_SCENE = "scene";
// ```
/** Intent extra name for choosing the scene the host draws. */
private const val EXTRA_SCENE: String = "scene"

// What:     `private const val SCENE_PLAYER: String = "player"` is the default scene, the production player.
// Why:      A launch without the scene extra draws the player, as the host always has.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_PLAYER = "player";
// ```
/** Scene name of the production player screen, which is the default. */
private const val SCENE_PLAYER: String = "player"

// What:     `private const val SCENE_SEARCH_RESULTS: String = "search-results"` draws Search with a matching query.
// Why:      The captures need the Results state, with the query "cam" that matches the Camellia sample folder.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_SEARCH_RESULTS = "search-results";
// ```
/** Scene name of the Search page with the Results state. */
private const val SCENE_SEARCH_RESULTS: String = "search-results"

// What:     `private const val SCENE_SEARCH_EMPTY: String = "search-empty"` draws Search with an empty query.
// Why:      The captures need the EmptyQuery invitation, which a blank query shows (D70).
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_SEARCH_EMPTY = "search-empty";
// ```
/** Scene name of the Search page with the EmptyQuery invitation. */
private const val SCENE_SEARCH_EMPTY: String = "search-empty"

// What:     `private const val SCENE_SEARCH_NONE: String = "search-none"` draws Search with a query no name matches.
// Why:      The captures need the NoMatch state, which a finished search with no hit shows (D70).
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_SEARCH_NONE = "search-none";
// ```
/** Scene name of the Search page with the NoMatch state. */
private const val SCENE_SEARCH_NONE: String = "search-none"

// What:     `private const val SCENE_SETTINGS: String = "settings"` draws the Settings page with its two templates.
// Why:      The captures need the Templates heading and the two template rows.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_SETTINGS = "settings";
// ```
/** Scene name of the Settings page. */
private const val SCENE_SETTINGS: String = "settings"

// What:     `private const val SCENE_TEMPLATE_EDITOR: String = "template-editor"` draws the Track rows editor.
// Why:      The captures need the editor open on the Track rows template with the stand-in values.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_TEMPLATE_EDITOR = "template-editor";
// ```
/** Scene name of the template editor opened on the Track rows template. */
private const val SCENE_TEMPLATE_EDITOR: String = "template-editor"

// What:     `private const val SCENE_FIRST_RUN_DECLINED: String = "first-run-declined"` draws the declined pane.
// Why:      The captures need each first-run state reachable by name from the intent extra.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_FIRST_RUN_DECLINED = "first-run-declined";
// ```
/** Scene name of the first-run pane in the declined state. */
private const val SCENE_FIRST_RUN_DECLINED: String = "first-run-declined"

// What:     `private const val SCENE_FIRST_RUN_SYSTEM_EMPTY: String = "first-run-system-empty"` draws the no-audio
//           pane.
// Why:      The captures need the device-library no-audio state reachable by name.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_FIRST_RUN_SYSTEM_EMPTY = "first-run-system-empty";
// ```
/** Scene name of the first-run pane for a device library with no audio. */
private const val SCENE_FIRST_RUN_SYSTEM_EMPTY: String = "first-run-system-empty"

// What:     `private const val SCENE_FIRST_RUN_FOLDER_EMPTY: String = "first-run-folder-empty"` draws the folder pane.
// Why:      The captures need the chosen-folder no-audio state reachable by name.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_FIRST_RUN_FOLDER_EMPTY = "first-run-folder-empty";
// ```
/** Scene name of the first-run pane for a chosen folder with no audio. */
private const val SCENE_FIRST_RUN_FOLDER_EMPTY: String = "first-run-folder-empty"

// What:     `private const val SCENE_FEEDBACK_BOTH: String = "feedback-both"` draws Undo and Error together.
// Why:      The captures need both overlay messages visible to check their order and stacking.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_FEEDBACK_BOTH = "feedback-both";
// ```
/** Scene name of the player with an Undo message above an Error message. */
private const val SCENE_FEEDBACK_BOTH: String = "feedback-both"

// What:     `private const val SCENE_TRACK_MENU: String = "track-menu"` names the scene with the track menu open.
// Why:      The menu capture needs the popup open on the second row of the sample library without any playback.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_TRACK_MENU = "track-menu";
// ```
/** Scene name of the player with the track context menu open on the second row. */
private const val SCENE_TRACK_MENU: String = "track-menu"

// What:     `private const val SCENE_TRASH_UNDO: String = "trash-undo"` names the scene with one Undo message visible.
// Why:      The Undo capture needs the message alone over the player, as the production overlay draws it.
//
// In TS you'd write (pseudocode):
// ```ts
// const SCENE_TRASH_UNDO = "trash-undo";
// ```
/** Scene name of the player with one Undo message from a trash, offering the Undo action. */
private const val SCENE_TRASH_UNDO: String = "trash-undo"

// What:     `private const val SAMPLE_FOLDER_NAME: String = "Camellia"` is the folder the folder pane names.
// Why:      The headline names a real folder, so the capture reads like a user's folder.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_FOLDER_NAME = "Camellia";
// ```
/** Folder name that the folder no-audio scene shows in its headline. */
private const val SAMPLE_FOLDER_NAME: String = "Camellia"

// What:     `private val SAMPLE_OVERLAY_INSET: Dp = 16.dp` is the overlay inset the player host uses.
// Why:      The debug overlay scenes place messages at the same inset as the production host.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_OVERLAY_INSET = 16; // dp
// ```
/** Inset between the overlay messages and the screen edges, matching the player host. */
private val SAMPLE_OVERLAY_INSET: Dp = 16.dp

// What:     `private const val SEARCH_NONE_QUERY: String = "zqxvj"` is a query no sample name contains.
// Why:      The NoMatch scene must search a query that no folder or track name matches, so the letters are unlikely
//           in any sample name.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_NONE_QUERY = "zqxvj";
// ```
/** Query that matches no sample folder or track name, which the NoMatch scene searches. */
private const val SEARCH_NONE_QUERY: String = "zqxvj"

// What:     `private const val SEARCH_RESULTS_QUERY: String = "cam"` is the query the Results scene searches.
// Why:      "cam" matches the Camellia folder and its tracks, as the captures show.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_RESULTS_QUERY = "cam";
// ```
/** Query that the Results scene searches, which matches the Camellia folder. */
private const val SEARCH_RESULTS_QUERY: String = "cam"

// What:     `private const val SAMPLE_POSITION_SEC: Double = 66.0` is the fixed playhead of the sample track.
// Why:      The screenshots show the first track one minute and six seconds in.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_POSITION_SEC = 66;
// ```
/** Fixed playhead position in seconds of the sample playing track, one minute and six seconds in. */
private const val SAMPLE_POSITION_SEC: Double = 66.0

// What:     `private const val SAMPLE_DURATION_SEC: Double = 275.0` is the length of the sample track.
// Why:      The screenshots show the first track four minutes and thirty-five seconds long.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_DURATION_SEC = 275;
// ```
/** Fixed length in seconds of the sample playing track, four minutes and thirty-five seconds. */
private const val SAMPLE_DURATION_SEC: Double = 275.0

// What:     `private val CAMELLIA_TITLES: List<String>` are the authored tracks of the Camellia folder.
// Why:      Camellia is the folder the captures show, so its titles match the reference screenshots.
//
// In TS you'd write (pseudocode):
// ```ts
// const CAMELLIA_TITLES = ["Another Xronixle", "Burning Aquamarine", ...];
// ```
/** Own-folder track titles of the Camellia sample folder, in the order the captures show them. */
private val CAMELLIA_TITLES: List<String> = listOf(
    "Another Xronixle",
    "Burning Aquamarine",
    "Dokuhebi",
    "ENÛMAVELIŠ",
    "Ghost",
    "Hyperflux",
    "Idol Corruption",
    "KillerToy",
    "Nacreous Snowmelt",
    "Mirage Rain",
    "Glasshouse",
    "Halcyon Drift",
    "Paper Lantern",
)

// What:     `private val CAMELLIA_LIVE_TITLES: List<String>` are the tracks of the Camellia/Live subfolder.
// Why:      A subfolder with its own tracks shows a header row between the folder's own tracks and its
//           subfolder tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// const CAMELLIA_LIVE_TITLES = ["Ghost (Live)", "Hyperflux (Live)", "KillerToy (Live)"];
// ```
/** Tracks of the Camellia/Live subfolder, which the track list shows under a subfolder header. */
private val CAMELLIA_LIVE_TITLES: List<String> = listOf("Ghost (Live)", "Hyperflux (Live)", "KillerToy (Live)")

// What:     `private val SAMPLE_FOLDER_NAMES: List<String>` are the other sample folders, spanning the rail
//           from A to M.
// Why:      The rail shows several writing-system cells, so the sample library needs folders under many letters.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_FOLDER_NAMES = ["Adele", "Aphex Twin", ...];
// ```
/** Other sample folders, each holding two tracks, so the rail shows letters from A to M and beyond. */
private val SAMPLE_FOLDER_NAMES: List<String> = listOf(
    "Adele",
    "Aphex Twin",
    "Boards of Canada",
    "Burial",
    "C418",
    "Carpenter Brut",
    "Casiopea",
    "Celldweller",
    "Chicane",
    "CHON",
    "Clark",
    "Clown Core",
    "Coaltar of the Deepers",
    "Com Truise",
    "Cornelius",
    "Covet",
    "Crumb",
    "Crystal Castles",
    "Cult of Luna",
    "Current Value",
    "Cynic",
    "Cö shu Nie",
    "capsule",
    "Charisma.com",
    "Cornelius Live",
    "Cytus Sound Team",
    "Daft Punk",
    "Deadmau5",
    "Enya",
    "Gorillaz",
    "Igorrr",
    "Justice",
    "Kraftwerk",
    "Lamb",
    "Massive Attack",
    "Mogwai",
)

// What:     `private val SAMPLE_LIBRARY_PATHS: List<String>` is the authored library in load order.
// Why:      Camellia comes first so its first track is load index zero, the track the screenshots show playing.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_LIBRARY_PATHS = [...camellia, ...others];
// ```
/** Full display paths of the authored library in load order, with Camellia first. */
private val SAMPLE_LIBRARY_PATHS: List<String> = buildSampleLibraryPaths()

// What:     `private fun buildSampleLibraryPaths(): List<String>` writes the display paths of every sample track.
// Why:      The paths are generated from the folder lists so the library stays consistent with those lists.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildSampleLibraryPaths(): string[] { /* camellia, then each folder's two tracks */ }
// ```
/** Builds the authored display paths: Camellia and its subfolder first, then two tracks in each other folder. */
private fun buildSampleLibraryPaths(): List<String> {
    /** Own tracks of Camellia, given their full display paths. */
    val camellia: List<String> = CAMELLIA_TITLES.map { title -> "Camellia/$title.flac" }
    /** Subfolder tracks of Camellia, given their full display paths. */
    val live: List<String> = CAMELLIA_LIVE_TITLES.map { title -> "Camellia/Live/$title.flac" }
    /** Two tracks in each other folder, given their full display paths. */
    val others: List<String> = SAMPLE_FOLDER_NAMES.flatMap { folder ->
        listOf("$folder/01 Song.flac", "$folder/02 Song.flac")
    }
    return camellia + live + others
}

// What:     `private val SAMPLE_PAGES: List<Page>` are the pages the production pagination makes from the
//           sample library.
// Why:      Using the real pagination keeps the rail, the tab order, and the load indices the same as in production.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_PAGES: Page[] = paginate(SAMPLE_LIBRARY_PATHS);
// ```
/** Pages of the authored library, built by the production pagination. */
private val SAMPLE_PAGES: List<Page> = paginate(SAMPLE_LIBRARY_PATHS)

// What:     `private val SAMPLE_PAGE_LABELS: List<String>` are the page names in tab order.
// Why:      The builder takes the page names, and the picker lists them in this order.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_PAGE_LABELS = SAMPLE_PAGES.map((page) => page.label);
// ```
/** Page names of the authored library in tab order. */
private val SAMPLE_PAGE_LABELS: List<String> = SAMPLE_PAGES.map { page -> page.label }

// What:     `private val SAMPLE_SELECTED_PAGE: Int` is the index of the Camellia page.
// Why:      The screenshots open on Camellia, so the selected page starts at its index.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_SELECTED_PAGE = SAMPLE_PAGE_LABELS.indexOf("Camellia");
// ```
/** Index of the Camellia page in tab order, which the host shows at launch. */
private val SAMPLE_SELECTED_PAGE: Int = SAMPLE_PAGE_LABELS.indexOf("Camellia")

// What:     `private val SAMPLE_CURRENT_INDEX: Int` is the load index of the first Camellia track.
// Why:      The first track is the one playing in the screenshots, so its load index marks the playing row.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_CURRENT_INDEX = SAMPLE_LIBRARY_PATHS.indexOf("Camellia/Another Xronixle.flac");
// ```
/** Load-order index of the first Camellia track, which plays at launch. */
private val SAMPLE_CURRENT_INDEX: Int = SAMPLE_LIBRARY_PATHS.indexOf("Camellia/Another Xronixle.flac")

// What:     `private val SAMPLE_TRACKS: List<Track>` is the authored library as tracks, in load order.
// Why:      Search reads the library as tracks, so the Search scenes search the same sample paths the pages use.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_TRACKS: Track[] = SAMPLE_LIBRARY_PATHS.map((path) => ({ uri: "", displayPath: path }));
// ```
/** Authored library as tracks in load order, with empty URIs because no playback happens in this host. */
private val SAMPLE_TRACKS: List<Track> = SAMPLE_LIBRARY_PATHS.map { path -> Track(uri = "", displayPath = path) }

// What:     `class ProductionScreenActivity : ComponentActivity()` is the debug host activity.
// Why:      A separate debug activity lets the screenshot runs render the production screen without the service.
//
// In TS you'd write (pseudocode):
// ```ts
// class ProductionScreenActivity extends ComponentActivity { /* ... */ }
// ```
/** Debug-only activity that renders the production player screen over authored sample data. */
class ProductionScreenActivity : ComponentActivity() {
    // What:     `override fun onCreate(savedInstanceState: Bundle?)` mounts the sample screen.
    // Why:      The host reads its launch extra and applies the same edge-to-edge and theme setup as the
    //           production activity.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onCreate(savedInstanceState: Bundle | null): void { /* ... */ }
    // ```
    /** Enables edge-to-edge drawing and mounts the production screen with the launch picker state. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        /** Whether the folder picker starts open, from the `picker` intent extra. */
        val startWithPicker: Boolean = intent.getBooleanExtra(EXTRA_PICKER, false)
        /** Scene to draw, from the `scene` intent extra, or the player when the extra is absent. */
        val scene: String = intent.getStringExtra(EXTRA_SCENE) ?: SCENE_PLAYER
        setContent {
            // The production activity's theme: the palette derived from the system accent (A3, D108).
            playerTheme {
                // A Surface supplies the content color and canvas, as the production activity's root does.
                Surface(modifier = Modifier.fillMaxSize()) {
                    // The sample rows carry no playing flag, so the scenes name the playing track as the host does.
                    CompositionLocalProvider(LocalPlayingIndex provides SAMPLE_CURRENT_INDEX) {
                        ProductionScreenHost(startWithPicker = startWithPicker, scene = scene)
                    }
                }
            }
        }
    }
}

// What:     `@Composable private fun ProductionScreenHost(startWithPicker: Boolean, scene: String)` draws one scene.
// Why:      The host keeps the picker and the selected page as local state for the player, and draws the Search,
//           Settings, and editor scenes from fixed sample values.
//
// In TS you'd write (pseudocode):
// ```ts
// function ProductionScreenHost(props: { startWithPicker: boolean; scene: string }): UIElement { /* ... */ }
// ```
/** Renders one scene of the production screens over the authored library with fixed playback values. */
@Composable
private fun ProductionScreenHost(startWithPicker: Boolean, scene: String) {
    /** Whether the folder picker is open on the folded cover, starting from the launch extra. */
    var pickerOpen: Boolean by remember { mutableStateOf(startWithPicker) }
    /** Index of the page the host shows, starting on Camellia. */
    var selectedPage: Int by remember { mutableIntStateOf(SAMPLE_SELECTED_PAGE) }
    /** Screen model built from the authored pages and the fixed playback values. */
    val model: PlayerContentModel = buildPlayerContent(
        pageLabels = SAMPLE_PAGE_LABELS,
        selectedPage = selectedPage,
        pageItems = SAMPLE_PAGES.getOrNull(selectedPage)?.entries.orEmpty(),
        playingItems = SAMPLE_PAGES.getOrNull(selectedPage)?.entries.orEmpty(),
        playingPageIndex = selectedPage,
        currentIndex = SAMPLE_CURRENT_INDEX,
        playing = true,
        mode = PlaybackMode.IN_ORDER,
        positionSec = SAMPLE_POSITION_SEC,
        durationSec = SAMPLE_DURATION_SEC,
    )
    /** Actions that change only the host's own picker and page state; playback actions are inert here. */
    val actions = PlayerContentActions(
        onToggleFolderPicker = { pickerOpen = !pickerOpen },
        onSelectFolder = { folder ->
            /** Page index of the chosen folder in tab order, or -1 when no page has that name. */
            val page: Int = SAMPLE_PAGE_LABELS.indexOf(folder)
            if (page >= 0) {
                selectedPage = page
            }
            pickerOpen = false
        },
        onOpen = {},
        onSettings = {},
        onPlay = {},
        onSearch = {},
        deck = TransportDeckActions(
            onSeek = {},
            onPrevious = {},
            onTogglePlay = {},
            onNext = {},
            onSelectMode = {},
        ),
    )
    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        /** Layout name the production screen would choose at this width, for the composition log. */
        val layout: String = if (isCoverLayout(maxWidth.value)) "cover" else "unfolded"
        Log.i(LOG_TAG, "ProductionScreen.composed layout=$layout pickerOpen=$pickerOpen scene=$scene")
        when (scene) {
            SCENE_PLAYER -> playerContent(model = model, pickerOpen = pickerOpen, actions = actions)
            SCENE_SEARCH_RESULTS -> sampleSearchPage(query = SEARCH_RESULTS_QUERY, model = model, actions = actions)
            SCENE_SEARCH_EMPTY -> sampleSearchPage(query = "", model = model, actions = actions)
            SCENE_SEARCH_NONE -> sampleSearchPage(query = SEARCH_NONE_QUERY, model = model, actions = actions)
            SCENE_SETTINGS -> settingsPageContent(
                templateLines = TemplateEditorState.initial(emptyList()).settingsLines,
                autoPlayOnSelect = false,
                onToggleAutoPlay = {},
                onBack = {},
                onOpenTemplate = {},
                onOpenLicenses = {},
            )
            SCENE_TEMPLATE_EDITOR -> templateEditor(
                state = TemplateEditorState.initial(emptyList()).open(TemplateKind.TRACK_ROWS),
                actions = TemplateEditorActions(
                    onEdit = {},
                    onFocusChanged = {},
                    onInsertField = {},
                    onReset = {},
                    onBack = {},
                ),
            )
            // What:     `SCENE_FIRST_RUN_DECLINED` draws the declined first-run pane with its actions unwired.
            // Why:      The capture shows the approved declined copy and its three buttons over the same layout.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "first-run-declined": return <FirstRunPane state="DECLINED" />;
            // ```
            SCENE_FIRST_RUN_DECLINED -> firstRunPane(
                state = FirstRunState.DECLINED,
                folderName = null,
                actions = sampleFirstRunActions(),
            )
            // What:     `SCENE_FIRST_RUN_SYSTEM_EMPTY` draws the no-audio pane for the device library.
            // Why:      The capture shows the device-library copy with Open a folder as the filled action.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "first-run-system-empty": return <FirstRunPane state="SYSTEM_NO_AUDIO" />;
            // ```
            SCENE_FIRST_RUN_SYSTEM_EMPTY -> firstRunPane(
                state = FirstRunState.SYSTEM_NO_AUDIO,
                folderName = null,
                actions = sampleFirstRunActions(),
            )
            // What:     `SCENE_FIRST_RUN_FOLDER_EMPTY` draws the no-audio pane for a chosen folder.
            // Why:      The capture shows the folder's display name in the headline.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "first-run-folder-empty": return <FirstRunPane state="FOLDER_NO_AUDIO" folderName="Camellia" />;
            // ```
            SCENE_FIRST_RUN_FOLDER_EMPTY -> firstRunPane(
                state = FirstRunState.FOLDER_NO_AUDIO,
                folderName = SAMPLE_FOLDER_NAME,
                actions = sampleFirstRunActions(),
            )
            // What:     `SCENE_FEEDBACK_BOTH` draws the Undo message above the Error message over the player.
            // Why:      The capture shows both overlay messages at once, Undo above Error, at their content width.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case "feedback-both": return <Player overlay={<FeedbackOverlay messages={[undo, error]} />} />;
            // ```
            SCENE_FEEDBACK_BOTH -> sampleOverlayScene(
                model = model,
                actions = actions,
                pickerOpen = pickerOpen,
                messages = sampleFeedbackMessages(),
            )
            SCENE_TRASH_UNDO -> sampleOverlayScene(
                model = model,
                actions = actions,
                pickerOpen = pickerOpen,
                messages = listOf(trashUndoMessage(sampleTrashRecord())),
            )
            SCENE_TRACK_MENU -> sampleTrackMenuScene(model = model, actions = actions, pickerOpen = pickerOpen)
            else -> throw IllegalArgumentException("Unknown ProductionScreenActivity scene: $scene")
        }
    }
}

// What:     `@Composable private fun sampleSearchPage(...)` draws Search for one query over the sample library.
// Why:      The captures need each Search state drawn with the deck and folder browser that the inner layout keeps.
//
// In TS you'd write (pseudocode):
// ```ts
// function SampleSearchPage(props: { query: string; model: Model; actions: Actions }): UIElement { /* ... */ }
// ```
// What:     `private fun sampleFirstRunActions(): FirstRunActions` gives the first-run scenes callbacks that do
//           nothing.
// Why:      The scenes draw the pane for a capture, and nothing in them should act on the sample library.
//
// In TS you'd write (pseudocode):
// ```ts
// const sampleFirstRunActions = (): FirstRunActions => ({ onAllowAccess: () => {}, onOpenFolder: () => {}, onSettings:
//           () => {} });
// ```
/** Returns first-run actions that draw the pane without acting on anything. */
private fun sampleFirstRunActions(): FirstRunActions = FirstRunActions(
    onAllowAccess = {},
    onOpenFolder = {},
    onSettings = {},
)

// What:     `private fun sampleFeedbackMessages(): List<FeedbackMessage>` builds the Undo and Error messages.
// Why:      The feedback scene shows both kinds at once, and the queue orders them Undo above Error as production does.
//
// In TS you'd write (pseudocode):
// ```ts
// const sampleFeedbackMessages = (): FeedbackMessage[] => [undo, error];
// ```
/** Returns the Undo and Error messages that the feedback scene shows together. */
private fun sampleFeedbackMessages(): List<FeedbackMessage> {
    /** Undo message that offers to reverse the last removal from the library. */
    val undo = FeedbackMessage(
        id = "sample-undo",
        text = "Track moved to trash",
        kind = FeedbackKind.UNDO,
        actionLabel = "Undo",
    )
    /** Error message with the two-line folder text that production shows for a folder it cannot keep. */
    val error = FeedbackMessage(
        id = "sample-error",
        text = "That folder can't be kept open. Choose it again, or pick another folder.",
        kind = FeedbackKind.ERROR,
        actionLabel = null,
    )
    return FeedbackQueue.EMPTY.show(message = error, nowMs = 0L).show(message = undo, nowMs = 0L).visible()
}

// What:     `private fun sampleOverlayScene(...)` draws the player with the feedback overlay at the bottom.
// Why:      The overlay floats over the player, so the capture shows the same placement as the production host.
//
// In TS you'd write (pseudocode):
// ```ts
// function SampleOverlayScene(props): UIElement { return
//           <Player><BottomColumn>{feedback}</BottomColumn></Player>; }
// ```
// What:     `private fun sampleTrashRecord(): TrashRecord` builds one Undo record over the second Camellia track.
// Why:      The trash-undo scene shows the production message text for a real sample name, without touching any file.
//
// In TS you'd write (pseudocode):
// ```ts
// function sampleTrashRecord(): TrashRecord { return { id: "sample", displayName: secondTrackName, ... }; }
// ```
/** Returns a pending trash record for the second Camellia track, with no cache copy and no file behind it. */
private fun sampleTrashRecord(): TrashRecord = TrashRecord(
    id = "sample",
    displayName = trackFileNameOf(SAMPLE_CAMELLIA_SECOND_PATH),
    route = TrashRoute.SAF_DELETE_WITH_COPY_BACK,
    trackUri = "",
    treeUri = null,
    parentDocumentId = null,
    mimeType = "audio/mpeg",
    copyPath = null,
    sizeBytes = 0L,
)

// What:     `private val SAMPLE_CAMELLIA_SECOND_PATH: String` is the display path of the second Camellia row.
// Why:      The menu and the Undo message name the same track the second row shows.
//
// In TS you'd write (pseudocode):
// ```ts
// const SAMPLE_CAMELLIA_SECOND_PATH = SAMPLE_PAGES[SAMPLE_SELECTED_PAGE].entries[1].name;
// ```
/** Display path of the second track in the Camellia page, which the menu scene opens on. */
private val SAMPLE_CAMELLIA_SECOND_PATH: String = SAMPLE_PAGES[SAMPLE_SELECTED_PAGE].entries[1].name

// What:     `private fun sampleTrackMenuScene(...)` draws the player with the track context menu open on the second row.
// Why:      The menu capture shows the production popup over the production player, with its actions and heading.
//
// In TS you'd write (pseudocode):
// ```ts
// function SampleTrackMenuScene(props): UIElement { return <Player><TrackMenu title="..." /></Player>; }
// ```
/** Draws the player over the sample library with the track menu open on the second Camellia row. */
@Composable
private fun sampleTrackMenuScene(
    model: PlayerContentModel,
    actions: PlayerContentActions,
    pickerOpen: Boolean,
) {
    Box(modifier = Modifier.fillMaxSize()) {
        playerContent(model = model, pickerOpen = pickerOpen, actions = actions)
        trackMenuPopup(
            actions = trackMenuActionsFor(),
            title = trackFileNameOf(SAMPLE_CAMELLIA_SECOND_PATH),
            detailsValue = null,
            anchor = MenuAnchorRect(left = 0, top = 0, right = 0, bottom = 0),
            onSelect = {},
            onDismiss = {},
        )
    }
}

/** Draws the player over the sample library with the feedback overlay at the bottom. */
@Composable
private fun sampleOverlayScene(
    model: PlayerContentModel,
    actions: PlayerContentActions,
    pickerOpen: Boolean,
    messages: List<FeedbackMessage>,
) {
    Box(modifier = Modifier.fillMaxSize()) {
        playerContent(model = model, pickerOpen = pickerOpen, actions = actions)
        Column(modifier = Modifier.align(Alignment.BottomStart).fillMaxWidth()) {
            if (messages.isNotEmpty()) {
                feedbackOverlay(
                    messages = messages,
                    onDismiss = {},
                    onAction = {},
                    modifier = Modifier.padding(SAMPLE_OVERLAY_INSET),
                )
            }
        }
    }
}

/** Draws Search for one query over the sample library, with the deck and folder browser of the inner layout. */
@Composable
private fun sampleSearchPage(query: String, model: PlayerContentModel, actions: PlayerContentActions) {
    /** Search state the page draws for the query over the sample library. */
    val results: SearchState = searchLibrary(query = query, tracks = SAMPLE_TRACKS, available = true)
    searchPage(
        creaseWidthPx = rememberCrease().widthPxOrNull,
        state = results,
        query = query,
        onQueryChange = {},
        onBack = {},
        onClear = {},
        onOpenResult = {},
        currentTrackIndex = SAMPLE_CURRENT_INDEX,
        deck = { transportDeck(model = model.deck, actions = actions.deck) },
        isPlaying = true,
        folderBrowser = { browserModifier ->
            folderPickerPane(
                sections = model.sections,
                currentFolder = model.currentFolder,
                onSelectFolder = {},
                modifier = browserModifier,
            )
        },
    )
}
