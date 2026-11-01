// The Licenses page, opened from the About group of Settings. It shows the license notices that the build packs into
// the APK as assets/THIRD-PARTY-LICENSES.txt (about 1.7 MB in a release build, a short note in a development build).
// The text is read and cut into blocks off the main thread (core/LicenseNoticeBlocks.kt) and listed lazily, so only
// the blocks on screen are measured and drawn.

// What:     `package dev.monochromatic.musicplayer` places the Licenses page beside the Settings page.
// Why:      The page reuses the Settings header and canvas color, which are internal to this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.content.Context
import android.os.SystemClock
import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.monochromatic.musicplayer.core.FocusRingPlacement
import dev.monochromatic.musicplayer.core.LicenseBlock
import dev.monochromatic.musicplayer.core.LicenseBlockKind
import dev.monochromatic.musicplayer.core.splitLicenseNotices
import kotlin.coroutines.cancellation.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

// What:     `internal const val LICENSES_TITLE: String = "Licenses"` is the title of the row and of the page.
// Why:      The row's spoken label, the page header and the tests share one text.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_TITLE = "Licenses";
// ```
/** Title of the Licenses row in Settings and of the Licenses page. */
internal const val LICENSES_TITLE: String = "Licenses"

// What:     `internal const val LICENSES_ROW_SUPPORTING_TEXT: String` is the line under the row's title.
// Why:      It says what the page holds, so a user who has not opened it knows what the row leads to.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_ROW_SUPPORTING_TEXT = "Open-source libraries used by this app";
// ```
/** Supporting line of the Licenses row in Settings. */
internal const val LICENSES_ROW_SUPPORTING_TEXT: String = "Open-source libraries used by this app"

// What:     `internal const val LICENSES_BACK_DESCRIPTION: String = "Back to Settings"` is the spoken label of Back.
// Why:      Back on this page leads to Settings, not to the player, and TalkBack says so.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_BACK_DESCRIPTION = "Back to Settings";
// ```
/** Spoken label of the Back button of the Licenses page. */
internal const val LICENSES_BACK_DESCRIPTION: String = "Back to Settings"

// What:     `internal const val LICENSES_LIST_TAG: String = "licenses-list"` names the scrolling list for tests.
// Why:      Tests scroll the list and read its state by this tag.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_LIST_TAG = "licenses-list";
// ```
/** Test tag of the lazy list that holds the notice blocks. */
internal const val LICENSES_LIST_TAG: String = "licenses-list"

// What:     `internal const val LICENSES_FAILED_TEXT: String` is the message shown when the notices cannot be read.
// Why:      A page that stays empty would look like a bug; the text names what failed and where the full notices are.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_FAILED_TEXT = "The license notices could not be read from this app.";
// ```
/** Message of the Licenses page when the notice asset cannot be read. */
internal const val LICENSES_FAILED_TEXT: String =
    "The license notices could not be read from this app. Every release also lists them in a file next to the app."

// What:     `internal const val LICENSES_LOADING_TEXT: String = "Loading licenses"` is the text beside the spinner.
// Why:      The spinner alone is one channel; the words are the second, and TalkBack reads them.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSES_LOADING_TEXT = "Loading licenses";
// ```
/** Text shown with the progress indicator while the notices load. */
internal const val LICENSES_LOADING_TEXT: String = "Loading licenses"

// What:     `private const val LICENSE_NOTICES_ASSET: String = "THIRD-PARTY-LICENSES.txt"` names the packed asset.
// Why:      The build task `thirdPartyLicensesAsset` writes this file name into the APK's assets.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSE_NOTICES_ASSET = "THIRD-PARTY-LICENSES.txt";
// ```
/** File name of the notices inside the APK's assets. */
private const val LICENSE_NOTICES_ASSET: String = "THIRD-PARTY-LICENSES.txt"

// What:     `private const val LICENSES_LOG_TAG: String = "LicensesPage"` names the log source of the page.
// Why:      The load time and the first frame are read from logcat under one tag.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSES_LOG_TAG = "LicensesPage";
// ```
/** Log tag of the Licenses page. */
private const val LICENSES_LOG_TAG: String = "LicensesPage"

// What:     `private val LICENSES_PAGE_MARGIN: Dp = 16.dp` is the inset of the list from the page edges.
// Why:      The Settings rows use the same 16 dp page margin, so the two pages line up.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSES_PAGE_MARGIN = 16; // dp
// ```
/** Distance from the page edge to the text of the list. */
private val LICENSES_PAGE_MARGIN: Dp = 16.dp

// What:     `private val LICENSES_KEY_STEP: Dp = 48.dp` is how far Up and Down scroll the list.
// Why:      About two lines of body text per key press, so a held key moves at a readable speed.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSES_KEY_STEP = 48; // dp
// ```
/** Distance that one Up or Down key press scrolls the list. */
private val LICENSES_KEY_STEP: Dp = 48.dp

// What:     `private const val LICENSES_PAGE_SHARE: Float = 0.9f` is the share of the viewport that Page Up/Down moves.
// Why:      A little less than a full page keeps the last line of the old page in view, so the reader keeps
//           their place.
//
// In TS you'd write (pseudocode):
// ```ts
// const LICENSES_PAGE_SHARE = 0.9;
// ```
/** Share of the viewport height that one Page Up or Page Down press scrolls. */
private const val LICENSES_PAGE_SHARE: Float = 0.9f

// What:     `internal sealed interface LicensesLoad` is the three states of the page's content.
// Why:      The page draws a progress indicator, the list or a message, and one closed type makes `when` total.
//
// In TS you'd write (pseudocode):
// ```ts
// type LicensesLoad = { kind: "loading" } | { kind: "loaded"; blocks: LicenseBlock[] } | { kind: "failed" };
// ```
/** What the Licenses page shows: nothing yet, the blocks, or the failure. */
internal sealed interface LicensesLoad {
    /** The notices are being read. */
    data object Loading : LicensesLoad

    /** The notices were read and cut into [blocks]. */
    data class Loaded(
        /** The list items, in file order. */
        val blocks: List<LicenseBlock>,
    ) : LicensesLoad

    /** The notices could not be read. */
    data object Failed : LicensesLoad
}

// What:     `internal enum class LicenseScroll` names the six ways the keyboard moves the list.
// Why:      The key-to-move mapping is pure and tested apart from the scrolling itself.
//
// In TS you'd write (pseudocode):
// ```ts
// type LicenseScroll = "lineUp" | "lineDown" | "pageUp" | "pageDown" | "top" | "bottom";
// ```
/** A keyboard move of the Licenses list. */
internal enum class LicenseScroll {
    /** One key step up. */
    LINE_UP,

    /** One key step down. */
    LINE_DOWN,

    /** One page up. */
    PAGE_UP,

    /** One page down. */
    PAGE_DOWN,

    /** To the first block. */
    TOP,

    /** To the last block. */
    BOTTOM,
}

// What:     `internal fun licenseScrollOf(key: Key): LicenseScroll?` maps a key to the list move it means.
// Why:      A keyboard user reads the page by arrows, page keys and Home and End, as in any scrolling document.
//
// In TS you'd write (pseudocode):
// ```ts
// const licenseScrollOf = (key: string): LicenseScroll | null => ({ ArrowUp: "lineUp", ... })[key] ?? null;
// ```
/** Returns the list move that [key] means, or null when the key is not a scroll key. */
internal fun licenseScrollOf(key: Key): LicenseScroll? = when (key) {
    Key.DirectionUp -> LicenseScroll.LINE_UP
    Key.DirectionDown -> LicenseScroll.LINE_DOWN
    Key.PageUp -> LicenseScroll.PAGE_UP
    Key.PageDown -> LicenseScroll.PAGE_DOWN
    Key.MoveHome -> LicenseScroll.TOP
    Key.MoveEnd -> LicenseScroll.BOTTOM
    else -> null
}

// What:     `suspend fun readLicenseNoticesAsset(context: Context): String` reads the packed notice file.
// Why:      The file is 1.7 MB in a release build, so it is read on the IO dispatcher and never on the main thread.
//
// In TS you'd write (pseudocode):
// ```ts
// async function readLicenseNoticesAsset(): Promise<string> {
//   return await fetch("asset:THIRD-PARTY-LICENSES.txt").then((r) => r.text());
// }
// ```
/** Reads the APK asset `THIRD-PARTY-LICENSES.txt` as UTF-8 text on the IO dispatcher. */
suspend fun readLicenseNoticesAsset(context: Context): String = withContext(Dispatchers.IO) {
    context.assets.open(LICENSE_NOTICES_ASSET).use { stream -> stream.readBytes().decodeToString() }
}

// What:     `internal suspend fun loadLicenses(readNotices)` reads the notices and cuts them into blocks.
// Why:      The page needs one call that never throws: a failure becomes the `Failed` state and is logged with its
//           cause.
//
// In TS you'd write (pseudocode):
// ```ts
// async function loadLicenses(read: () => Promise<string>): Promise<LicensesLoad> {
//   try { ... } catch (e) { console.error(e); return failed; }
// }
// ```
/** Reads the notice text with [readNotices] and splits it on the default dispatcher; logs the sizes and the time. */
internal suspend fun loadLicenses(readNotices: suspend () -> String): LicensesLoad {
    /** Clock reading when loading started. */
    val started: Long = SystemClock.elapsedRealtime()
    try {
        /** The whole notice text. */
        val text: String = readNotices()
        /** Clock reading when the text was in memory. */
        val read: Long = SystemClock.elapsedRealtime()
        /** The list items. */
        val blocks: List<LicenseBlock> = withContext(Dispatchers.Default) { splitLicenseNotices(text) }
        Log.i(
            LICENSES_LOG_TAG,
            "licenses loaded chars=${text.length} blocks=${blocks.size} " +
                "readMs=${read - started} splitMs=${SystemClock.elapsedRealtime() - read}",
        )
        return LicensesLoad.Loaded(blocks)
    } catch (error: CancellationException) {
        throw error
    } catch (expectedFailure: Exception) {
        Log.e(LICENSES_LOG_TAG, "licenses could not be read", expectedFailure)
        return LicensesLoad.Failed
    }
}

// What:     `@Composable fun licensesPage(...)` draws the Licenses page: header, then the notices as a lazy list.
// Why:      D109 and D110 give Settings its header, canvas and margins; this page keeps them, and its body is the
//           notice text, which is far too large for one text node.
//
// In TS you'd write (pseudocode):
// ```ts
// function LicensesPage(props: { readNotices(): Promise<string>; onBack(): void }): UIElement;
// ```
/**
 * Draws the Licenses page on the player's canvas color: the Settings header with a Back button to Settings, then the
 * blocks of the notice text in a lazy list that holds one keyboard focus stop.
 *
 * @param readNotices reads the whole notice text; called once per visit, off the main thread.
 * @param onBack leaves the page for Settings.
 */
@Composable
fun licensesPage(readNotices: suspend () -> String, onBack: () -> Unit, modifier: Modifier = Modifier) {
    /** Clock reading when the page entered the composition, which the first frame is measured from. */
    val opened: Long = remember { SystemClock.elapsedRealtime() }
    /** Latest reader, so a new lambda on recomposition does not restart the load. */
    val reader by rememberUpdatedState(readNotices)
    /** What the page shows now. */
    var load: LicensesLoad by remember { mutableStateOf(LicensesLoad.Loading) }
    LaunchedEffect(Unit) {
        load = loadLicenses(readNotices = { reader() })
    }
    LaunchedEffect(load) {
        /** The state this effect run saw, so the check below is about one value. */
        val seen: LicensesLoad = load
        if (seen is LicensesLoad.Loaded) {
            androidx.compose.runtime.withFrameNanos { }
            Log.i(
                LICENSES_LOG_TAG,
                "licenses first frame blocks=${seen.blocks.size} afterOpenMs=${SystemClock.elapsedRealtime() - opened}",
            )
        }
    }
    /** The load state of this composition, read once so the branches below see one value. */
    val shown: LicensesLoad = load
    Column(
        modifier = modifier.fillMaxSize().background(canvasColor(dark = isSystemInDarkTheme()))
            .semantics { paneTitle = LICENSES_TITLE },
    ) {
        Box(modifier = Modifier.fillMaxWidth().windowInsetsTopHeight(WindowInsets.statusBars))
        settingsHeader(title = LICENSES_TITLE, backDescription = LICENSES_BACK_DESCRIPTION, onBack = onBack)
        Box(
            modifier = Modifier.weight(1f).fillMaxWidth().windowInsetsPadding(WindowInsets.navigationBars),
        ) {
            when (shown) {
                LicensesLoad.Loading -> licensesLoading()
                LicensesLoad.Failed -> licensesMessage(text = LICENSES_FAILED_TEXT)
                is LicensesLoad.Loaded -> licensesList(blocks = shown.blocks)
            }
        }
    }
}

// What:     `@Composable private fun licensesLoading()` draws a spinner with the words "Loading licenses".
// Why:      The page shows something at once; the words are a second channel and are read by TalkBack.
//
// In TS you'd write (pseudocode):
// ```ts
// function Loading(): UIElement { return <div><Spinner/><span>Loading licenses</span></div>; }
// ```
/** Centered progress indicator and its label, shown while the notices load. */
@Composable
private fun licensesLoading() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        CircularProgressIndicator(color = MaterialTheme.colorScheme.primary)
        Text(
            text = LICENSES_LOADING_TEXT,
            modifier = Modifier.padding(top = 16.dp),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

// What:     `@Composable private fun licensesMessage(text: String)` draws one centered message.
// Why:      The failure case needs words, not an empty page.
//
// In TS you'd write (pseudocode):
// ```ts
// function Message(props: { text: string }): UIElement { return <p>{props.text}</p>; }
// ```
/** Draws [text] on the page body with the page margin. */
@Composable
private fun licensesMessage(text: String) {
    Text(
        text = text,
        modifier = Modifier.fillMaxWidth().padding(LICENSES_PAGE_MARGIN),
        color = MaterialTheme.colorScheme.onSurface,
        style = MaterialTheme.typography.bodyMedium,
    )
}

// What:     `@Composable private fun licensesList(blocks: List<LicenseBlock>)` draws the blocks as a lazy list.
// Why:      Only the blocks on screen are measured and drawn, so a 35,000-line text scrolls without the cost of all of
//           it; the list is one focus stop that Up, Down, Page and Home and End scroll.
//
// In TS you'd write (pseudocode):
// ```ts
// function List(props: { blocks: LicenseBlock[] }): UIElement {
//   return <VirtualList items={blocks} tabIndex={0} onKeyDown={scrollKeys}/>;
// }
// ```
/** Draws [blocks] in a lazy list with a keyboard ring inside its edge while the list holds keyboard focus. */
@Composable
private fun licensesList(blocks: List<LicenseBlock>) {
    /** Scroll position of the list. */
    val listState: LazyListState = rememberLazyListState()
    /** Scope that keyboard scrolls run in. */
    val scope: CoroutineScope = rememberCoroutineScope()
    /** Pixels that one Up or Down press scrolls at this density. */
    val stepPx: Float = with(LocalDensity.current) { LICENSES_KEY_STEP.toPx() }
    LazyColumn(
        modifier = Modifier.fillMaxSize().testTag(LICENSES_LIST_TAG)
            .focusRing(shape = RectangleShape, placement = FocusRingPlacement.INSIDE)
            .focusable()
            .onKeyEvent { event ->
                licensesKeyHandled(event = event, state = listState, scope = scope, stepPx = stepPx)
            },
        state = listState,
        contentPadding = PaddingValues(horizontal = LICENSES_PAGE_MARGIN, vertical = 8.dp),
    ) {
        itemsIndexed(
            items = blocks,
            key = { index, _ -> index },
            contentType = { _, block -> block.kind },
        ) { _, block ->
            licenseBlockItem(block = block)
        }
    }
}

// What:     `private fun licensesKeyHandled(...)` scrolls the list for a scroll key and reports whether it consumed it.
// Why:      A lazy list does not scroll by keys on its own; the page maps the six document keys to scroll calls.
//
// In TS you'd write (pseudocode):
// ```ts
// function onKeyDown(e: KeyboardEvent): boolean {
//   const move = licenseScrollOf(e.key);
//   if (!move) return false;
//   scrollBy(...);
//   return true;
// }
// ```
/** Scrolls [state] for a key-down that [licenseScrollOf] knows; any other key goes on to the host. */
private fun licensesKeyHandled(event: KeyEvent, state: LazyListState, scope: CoroutineScope, stepPx: Float): Boolean {
    if (event.type != KeyEventType.KeyDown) {
        return false
    }
    /** The move this key means, or null for any other key. */
    val move: LicenseScroll = licenseScrollOf(event.key) ?: return false
    /** Height of the visible part of the list in pixels. */
    val viewportPx: Float = state.layoutInfo.viewportSize.height.toFloat()
    scope.launch {
        when (move) {
            LicenseScroll.LINE_UP -> state.scrollBy(-stepPx)
            LicenseScroll.LINE_DOWN -> state.scrollBy(stepPx)
            LicenseScroll.PAGE_UP -> state.scrollBy(-viewportPx * LICENSES_PAGE_SHARE)
            LicenseScroll.PAGE_DOWN -> state.scrollBy(viewportPx * LICENSES_PAGE_SHARE)
            LicenseScroll.TOP -> state.scrollToItem(0)
            LicenseScroll.BOTTOM -> state.scrollToItem(maxOf(state.layoutInfo.totalItemsCount - 1, 0))
        }
    }
    return true
}

// What:     `@Composable private fun licenseBlockItem(block: LicenseBlock)` draws one list item by its kind.
// Why:      Section titles, subsection titles, wrapped prose and preformatted lines each have their own look; title
//           kinds also differ by size, weight and color and carry the heading role (two channels, not color alone).
//
// In TS you'd write (pseudocode):
// ```ts
// function BlockItem(props: { block: LicenseBlock }): UIElement { switch (props.block.kind) { ... } }
// ```
/** Draws one block: a section title, a subsection title, a paragraph or a group of preformatted lines. */
@Composable
private fun licenseBlockItem(block: LicenseBlock) {
    when (block.kind) {
        LicenseBlockKind.SECTION -> Text(
            text = block.text,
            modifier = Modifier.fillMaxWidth().padding(top = 24.dp, bottom = 8.dp).semantics { heading() },
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.titleMedium,
        )
        LicenseBlockKind.SUBSECTION -> Text(
            text = block.text,
            modifier = Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 8.dp).semantics { heading() },
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelLarge,
        )
        LicenseBlockKind.PROSE -> Text(
            text = block.text,
            modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodyMedium,
        )
        LicenseBlockKind.LINES -> Text(
            text = block.text,
            modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
            fontFamily = FontFamily.Monospace,
        )
    }
}
