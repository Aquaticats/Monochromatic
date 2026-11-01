// What:     `package dev.monochromatic.musicplayer` places the Settings page beside the other player composables.
// Why:      The page is a stateless composable the host screen calls without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the template kinds, the stock layout numbers and the pane split rule.
// Why:      Each row opens or selects one template kind, and every number comes from the unit-tested pure rules.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TemplateKind } from "core/TemplateEditorState";
// import { settingsRowRadii, settingsPaneSplit } from "core/SettingsLayout";
// ```
import dev.monochromatic.musicplayer.core.SETTINGS_ICON_CIRCLE_DP
import dev.monochromatic.musicplayer.core.SETTINGS_PANE_FADE_MILLIS
import dev.monochromatic.musicplayer.core.autoPlayOnSelectSupportingText
import dev.monochromatic.musicplayer.core.SETTINGS_ICON_GLYPH_DP
import dev.monochromatic.musicplayer.core.SETTINGS_ICON_TITLE_GAP_DP
import dev.monochromatic.musicplayer.core.SETTINGS_PAGE_MARGIN_DP
import dev.monochromatic.musicplayer.core.SETTINGS_ROW_GAP_DP
import dev.monochromatic.musicplayer.core.SETTINGS_ROW_HEIGHT_DP
import dev.monochromatic.musicplayer.core.SETTINGS_ROW_INNER_PADDING_DP
import dev.monochromatic.musicplayer.core.SettingsPaneSplit
import dev.monochromatic.musicplayer.core.SettingsRowRadii
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.core.settingsPaneSplit
import dev.monochromatic.musicplayer.core.settingsRowRadii

// What:     Imports from `androidx.compose.foundation` bring in backgrounds, clicks, scrolling and insets.
// Why:      The page scrolls its rows, keeps them clear of the navigation bar, and marks each row as a tappable button.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clickable, verticalScroll } from "compose/foundation";
// ```
import android.util.Log
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.material3` and `androidx.compose.ui` bring in the text, icon, colour,
//           clipping, alignment, layout and semantics the page uses.
// Why:      The page draws its own header and rows from the theme roles, and each row announces as a button.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Icon, MaterialTheme, Text } from "material3";
// import { Modifier, Role, heading } from "compose/ui";
// ```
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchColors
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private const val SETTINGS_LOG_TAG: String = "SettingsPage"` names the log source of this page.
// Why:      The wide page logs its pane split under one tag so the layout decisions can be found in logcat.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_LOG_TAG = "SettingsPage";
// ```
/** Log tag of the Settings page. */
private const val SETTINGS_LOG_TAG: String = "SettingsPage"

// What:     `internal val SETTINGS_HEADER_HEIGHT: Dp = 64.dp` is the height of the static top bar.
// Why:      The measured stock top bar is 64 dp, with no divider below it; the editor pane's title row matches it.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_HEADER_HEIGHT = 64;
// ```
/** Height of the static top bar that holds the Back button and the page title. */
internal val SETTINGS_HEADER_HEIGHT: Dp = 64.dp

// What:     `private val SETTINGS_BACK_SIZE: Dp = 56.dp` is the diameter of the round Back button.
// Why:      The measured stock Back circle is 56 dp, which is also larger than the 48 dp minimum touch target.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_BACK_SIZE = 56;
// ```
/** Diameter of the round Back button, which also bounds its press indication. */
private val SETTINGS_BACK_SIZE: Dp = 56.dp

// What:     `private val SETTINGS_TITLE_GAP: Dp = 24.dp` is the space between the Back circle and the page title.
// Why:      Places the title text origin at 96 dp from the page edge, which matches the measured title start
//           on the inner panel.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_TITLE_GAP = 24;
// ```
/** Space between the right edge of the Back button and the start of the page title. */
private val SETTINGS_TITLE_GAP: Dp = 24.dp

// What:     `private val SETTINGS_HEADING_INSET: Dp = 24.dp` is the inset of a section heading from the page edge.
// Why:      Stock Settings starts section headings 24 dp from the edge, which is 8 dp to the left of the card edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_HEADING_INSET = 24;
// ```
/** Distance from the page edge to the start of a section heading. */
private val SETTINGS_HEADING_INSET: Dp = 24.dp

// What:     `private val SETTINGS_HEADING_TOP: Dp = 16.dp` is the space above a section heading.
// Why:      Stock Settings puts 16 dp above each heading, so the heading reads as the start of its group.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_HEADING_TOP = 16;
// ```
/** Space above a section heading. */
private val SETTINGS_HEADING_TOP: Dp = 16.dp

// What:     `private val SETTINGS_HEADING_BOTTOM: Dp = 8.dp` is the space between a section heading and its rows.
// Why:      Stock Settings keeps the heading close to the first row it names.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_HEADING_BOTTOM = 8;
// ```
/** Space between a section heading and the first row under it. */
private val SETTINGS_HEADING_BOTTOM: Dp = 8.dp

// What:     `private val SETTINGS_ROW_VERTICAL_INSET: Dp = 12.dp` is the space above and below a row's two text lines.
// Why:      The two lines together are about 44 dp, so 12 dp above and below keeps the text off the row edges when
//           the text grows.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ROW_VERTICAL_INSET = 12;
// ```
/** Vertical inset above and below the two text lines of a row. */
private val SETTINGS_ROW_VERTICAL_INSET: Dp = 12.dp

// What:     `internal const val AUTO_PLAY_ON_SELECT_TITLE: String = "Auto-play on select"` is the switch row's title.
// Why:      The title is the row's spoken label, so tests and the row share one text.
//
// In TS you'd write (pseudocode):
// ```ts
// const AUTO_PLAY_ON_SELECT_TITLE = "Auto-play on select";
// ```
/** Title of the Auto-play on select row, which TalkBack reads as the switch's label. */
internal const val AUTO_PLAY_ON_SELECT_TITLE: String = "Auto-play on select"

// What:     `internal const val AUTO_PLAY_SWITCH_TAG: String = "auto-play-switch"` names the drawn switch for tests.
// Why:      The row owns the toggle and merges its children into one node, so a test finds the drawn switch by this
//           tag in the unmerged tree to see whether it sits beside or under the text.
//
// In TS you'd write (pseudocode):
// ```ts
// const AUTO_PLAY_SWITCH_TAG = "auto-play-switch";
// ```
/** Test tag of the drawn switch of the Auto-play on select row. */
internal const val AUTO_PLAY_SWITCH_TAG: String = "auto-play-switch"

// What:     `private val SETTINGS_SWITCH_GAP: Dp = 16.dp` is the space between the row text and the trailing switch.
// Why:      Material switch rows keep 16 dp between the text column and the 52 dp switch.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_GAP = 16;
// ```
/** Space between the end of the text column and the switch. */
private val SETTINGS_SWITCH_GAP: Dp = 16.dp

// What:     `private const val SETTINGS_SWITCH_SUPPORTING_LINES: Int = 2` bounds the supporting text of a switch row.
// Why:      The text fits one line on the 443 dp cover; in the narrow list pane of the unfolded screen it wraps to a
//           second line, which grows the row like a larger text size does, instead of cutting the meaning off.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_SUPPORTING_LINES = 2;
// ```
/** Most lines the supporting text of a switch row takes before it is cut with an ellipsis. */
private const val SETTINGS_SWITCH_SUPPORTING_LINES: Int = 2

// What:     `private const val SETTINGS_SWITCH_TITLE_LINES: Int = 2` bounds the title of a switch row.
// Why:      A template row title is one line, but "Auto-play on select" wraps in a narrow list pane, and a cut title
//           would hide the name of the setting.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_TITLE_LINES = 2;
// ```
/** Most lines the title of a switch row takes before it is cut with an ellipsis. */
private const val SETTINGS_SWITCH_TITLE_LINES: Int = 2

// What:     `private val SETTINGS_SWITCH_STACK_BELOW: Dp = 300.dp` is the row width under which the switch moves under
//           the text.
// Why:      Beside the text the switch leaves the text column 148 dp less than the row; the list pane of the unfolded
//           screen is about 213 dp wide, which would leave the title no room. Below this width the switch sits under
//           the supporting text instead, so the text keeps the row's width.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_STACK_BELOW = 300;
// ```
/** Row width below which the switch is placed under the text instead of after it. */
private val SETTINGS_SWITCH_STACK_BELOW: Dp = 300.dp

// What:     `private val SETTINGS_SWITCH_STACKED_GAP: Dp = 8.dp` is the space between the supporting text and a stacked
//           switch.
// Why:      A stacked switch needs the same breathing room as the heading has below its label.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SWITCH_STACKED_GAP = 8;
// ```
/** Space between the supporting text and the switch when the switch sits under the text. */
private val SETTINGS_SWITCH_STACKED_GAP: Dp = 8.dp

// What:     `internal fun settingsPageContent(...)` draws the single-column Settings page.
// Why:      On a window narrower than the player's two-pane width, the page lists each template with the line it
//           yields, and a tap opens that template's editor page.
//
// In TS you'd write (pseudocode):
// ```ts
// function SettingsPage(props: { templateLines: Record<TemplateKind, string>; onBack(): void;
//   onOpenTemplate(kind: TemplateKind): void; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the Settings page as a static header and the Templates rows on the player's canvas color,
 * full width with a 16 dp margin and no maximum width.
 */
@Composable
fun settingsPageContent(
    templateLines: Map<TemplateKind, String>,
    autoPlayOnSelect: Boolean,
    onToggleAutoPlay: () -> Unit,
    onBack: () -> Unit,
    onOpenTemplate: (TemplateKind) -> Unit,
    onOpenLicenses: () -> Unit,
    modifier: Modifier = Modifier,
) {
    settingsListPane(
        templateLines = templateLines,
        selected = null,
        onChoose = onOpenTemplate,
        autoPlayOnSelect = autoPlayOnSelect,
        onToggleAutoPlay = onToggleAutoPlay,
        onOpenLicenses = onOpenLicenses,
        onBack = onBack,
        textEndExtra = 0.dp,
        modifier = modifier.fillMaxSize().background(canvasColor(dark = isSystemInDarkTheme())),
    )
}

// What:     `internal fun settingsWidePage(...)` draws the two-pane Settings page for the unfolded screen.
// Why:      D110 splits the window into the list on the left and the selected template's editor on the right, so a
//           tap on a row selects it without leaving the page.
//
// In TS you'd write (pseudocode):
// ```ts
// function SettingsWidePage(props: { templateLines; selected: TemplateKind; onSelect(kind): void; onBack(): void;
//   creaseWidthPx: number | null; editor(startInset: number): UIElement }): UIElement;
// ```
/**
 * Draws the wide Settings page: the list pane, then the editor pane that [editor] draws.
 * [creaseWidthPx] is null when the device has no crease, so the panes follow the stock ratio and text is not kept
 * out of the center; otherwise each pane is half of the window and text keeps the crease band clear.
 * [editor] receives the template to show and the extra start inset in dp that its text needs beside the center,
 * and the page cross-fades between editors when the selection changes.
 */
@Composable
fun settingsWidePage(
    templateLines: Map<TemplateKind, String>,
    selected: TemplateKind,
    onSelect: (TemplateKind) -> Unit,
    autoPlayOnSelect: Boolean,
    onToggleAutoPlay: () -> Unit,
    onOpenLicenses: () -> Unit,
    onBack: () -> Unit,
    creaseWidthPx: Float?,
    editor: @Composable (kind: TemplateKind, startInset: Dp) -> Unit,
    modifier: Modifier = Modifier,
) {
    /** Canvas color shared by both panes and the list. */
    val canvas: Color = canvasColor(dark = isSystemInDarkTheme())
    /** Physical pixels per dp of the current display, which converts the physical clearance to dp. */
    val density: Float = LocalDensity.current.density
    BoxWithConstraints(modifier = modifier.fillMaxSize().background(canvas)) {
        /** Pane widths in pixels and the crease clearance of each pane for this window. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = creaseWidthPx != null,
            windowWidthPx = constraints.maxWidth,
            creaseWidthPx = creaseWidthPx ?: 0f,
            density = density,
        )
        // What:     `LaunchedEffect(split)` logs the pane widths whenever the split changes.
        // Why:      The pane widths are layout decisions that a screenshot cannot show when both panes share the
        //           canvas color, so the log is the record of what the page placed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // useEffect(() => console.info("settings wide split", split), [split]);
        // ```
        LaunchedEffect(split) {
            Log.i(
                SETTINGS_LOG_TAG,
                "wide split window=${constraints.maxWidth}px list=${split.listWidthPx}px " +
                    "editor=${split.editorWidthPx}px crease=${creaseWidthPx != null} density=$density " +
                    "listEndExtra=${split.listTextEndExtraDp}dp editorStartExtra=${split.editorTextStartExtraDp}dp",
            )
        }
        settingsPaneRow(
            listWidthPx = split.listWidthPx,
            list = {
                settingsListPane(
                    templateLines = templateLines,
                    selected = selected,
                    onChoose = onSelect,
                    autoPlayOnSelect = autoPlayOnSelect,
                    onToggleAutoPlay = onToggleAutoPlay,
                    onOpenLicenses = onOpenLicenses,
                    onBack = onBack,
                    textEndExtra = split.listTextEndExtraDp.dp,
                    modifier = Modifier.fillMaxSize(),
                )
            },
            editor = {
                // What:     `Crossfade` fades the old editor out while the new one fades in, with no slide.
                // Why:      The stock two-pane Settings cross-fades its right pane when a left row is chosen, and the
                //           animation scale in the system settings stretches or removes the fade.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // <CrossFade key={selected} durationMs={150}>{editor(selected)}</CrossFade>
                // ```
                Crossfade(
                    targetState = selected,
                    modifier = Modifier.fillMaxSize(),
                    animationSpec = tween(durationMillis = SETTINGS_PANE_FADE_MILLIS),
                    label = "settingsEditorPane",
                ) { kind -> editor(kind, split.editorTextStartExtraDp.dp) }
            },
        )
    }
}

// What:     `@Composable private fun settingsPaneRow(...)` places two panes side by side at exact pixel widths.
// Why:      The panes must be exactly the widths the split rule gives, and converting through dp could drift a pixel.
//
// In TS you'd write (pseudocode):
// ```ts
// function PaneRow(props: { listWidthPx: number; list(): UIElement; editor(): UIElement }): UIElement;
// ```
/** Lays out the list pane [listWidthPx] wide and the editor pane across the rest of the width, both full height. */
@Composable
private fun settingsPaneRow(listWidthPx: Int, list: @Composable () -> Unit, editor: @Composable () -> Unit) {
    Layout(
        content = {
            // The pane titles name each pane for assistive technology and show up as nodes with the pane's bounds.
            Box(modifier = Modifier.semantics { paneTitle = "Settings list" }) { list() }
            Box(modifier = Modifier.semantics { paneTitle = "Settings editor" }) { editor() }
        },
        modifier = Modifier.fillMaxSize(),
    ) { measurables, constraints ->
        /** Full width the row fills. */
        val width: Int = constraints.maxWidth
        /** Full height the row fills. */
        val height: Int = constraints.maxHeight
        /** Left pane measured at its exact width. */
        val listPlaceable = measurables[0].measure(Constraints.fixed(listWidthPx, height))
        /** Right pane measured across the rest of the width. */
        val editorPlaceable = measurables[1].measure(Constraints.fixed(width - listWidthPx, height))
        layout(width, height) {
            listPlaceable.place(0, 0)
            editorPlaceable.place(listWidthPx, 0)
        }
    }
}

// What:     `@Composable private fun settingsListPane(...)` draws the header and the list of template rows.
// Why:      The single page and the wide page's left pane show the same header and rows, so one function serves both.
//
// In TS you'd write (pseudocode):
// ```ts
// function ListPane(props: { templateLines; selected: TemplateKind | null; onChoose(kind): void;
//   onBack(): void; textEndExtra: number; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the status bar spacer, the header and the scrolling Templates, Playback and About groups.
 * [selected] is the chosen row in the wide form and null in the single form;
 * [autoPlayOnSelect] and [onToggleAutoPlay] drive the Playback group's switch row, which toggles in place and never
 * changes [selected] (D129);
 * [textEndExtra] is the extra end inset that keeps row text clear of the crease.
 */
@Composable
private fun settingsListPane(
    templateLines: Map<TemplateKind, String>,
    selected: TemplateKind?,
    onChoose: (TemplateKind) -> Unit,
    autoPlayOnSelect: Boolean,
    onToggleAutoPlay: () -> Unit,
    onOpenLicenses: () -> Unit,
    onBack: () -> Unit,
    textEndExtra: Dp,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        // What:     `Box` with `windowInsetsTopHeight` is an empty block as tall as the status bar.
        // Why:      The page draws edge to edge, so its header starts below the clock and icons.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <div style={{ height: 'env(safe-area-inset-top)' }}/>
        // ```
        Box(modifier = Modifier.fillMaxWidth().windowInsetsTopHeight(WindowInsets.statusBars))
        settingsHeader(title = "Settings", backDescription = "Back to player", onBack = onBack)
        // What:     `verticalScroll` lets the rows move when text is large, and the navigation-bar inset keeps the
        //           last row clear of the system bar.
        // Why:      Larger text grows the rows, so the list must scroll rather than clip.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <div style={{ flex: 1, overflowY: 'auto', paddingBottom: 'env(safe-area-inset-bottom)' }}>{rows}</div>
        // ```
        Column(
            modifier = Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())
                .windowInsetsPadding(WindowInsets.navigationBars),
        ) {
            // The 16 dp between two groups is the top padding of the next group's heading (D109), so the column
            // adds no gap of its own.
            // What:     `Column` with `spacedBy` stacks the heading and the template rows of the one group.
            // Why:      The 2 dp between rows is page canvas, so the group needs no card fill or divider line.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // <div style={{ display: 'flex', flexDirection: 'column', gap: 2, padding: '0 16px' }}>{rows}</div>
            // ```
            Column {
                settingsSectionHeading(text = "Templates")
                Column(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = SETTINGS_PAGE_MARGIN_DP.dp),
                    verticalArrangement = Arrangement.spacedBy(SETTINGS_ROW_GAP_DP.dp),
                ) {
                    TemplateKind.entries.forEachIndexed { index, template ->
                        settingsTemplateRow(
                            template = template,
                            line = templateLines[template].orEmpty(),
                            radii = settingsRowRadii(
                                index = index,
                                count = TemplateKind.entries.size,
                                selected = selected == template,
                            ),
                            selected = selected?.let { chosen -> chosen == template },
                            textEndExtra = textEndExtra,
                            onChoose = { onChoose(template) },
                        )
                    }
                }
            }
            // What:     `Column` with a heading and one `settingsSwitchRow` is the Playback group.
            // Why:      A switch under the Templates heading would be mislabelled, so the setting is its own group
            //           (D129) 16 dp below the templates, with the same row size and corner rounding.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // <section><h2>Playback</h2><SwitchRow title="Auto-play on select" .../></section>
            // ```
            Column {
                settingsSectionHeading(text = "Playback")
                Column(modifier = Modifier.fillMaxWidth().padding(horizontal = SETTINGS_PAGE_MARGIN_DP.dp)) {
                    settingsSwitchRow(
                        icon = SETTINGS_AUTO_PLAY_ICON,
                        title = AUTO_PLAY_ON_SELECT_TITLE,
                        supportingText = autoPlayOnSelectSupportingText(enabled = autoPlayOnSelect),
                        checked = autoPlayOnSelect,
                        radii = settingsRowRadii(index = 0, count = 1),
                        textEndExtra = textEndExtra,
                        onToggle = onToggleAutoPlay,
                    )
                }
            }
            // What:     `Column` with a heading and one `settingsLinkRow` is the About group.
            // Why:      The Licenses row opens a page of its own, so it is not a template and not a switch; it gets its
            //           own group 16 dp below Playback, with the same row size and corner rounding.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // <section><h2>About</h2><LinkRow title="Licenses" onClick={openLicenses}/></section>
            // ```
            Column {
                settingsSectionHeading(text = "About")
                Column(modifier = Modifier.fillMaxWidth().padding(horizontal = SETTINGS_PAGE_MARGIN_DP.dp)) {
                    settingsLinkRow(
                        icon = SETTINGS_LICENSES_ICON,
                        title = LICENSES_TITLE,
                        supportingText = LICENSES_ROW_SUPPORTING_TEXT,
                        radii = settingsRowRadii(index = 0, count = 1),
                        textEndExtra = textEndExtra,
                        onClick = onOpenLicenses,
                    )
                }
            }
        }
    }
}

// What:     `@Composable private fun settingsLinkRow(...)` draws one row that opens a page of its own.
// Why:      The row has the 72 dp size, the 40 dp icon circle, the title and one supporting line of the other rows; the
//           whole row is the button target (role Button), so a tap, Enter and Space open the page.
//
// In TS you'd write (pseudocode):
// ```ts
// function LinkRow(props: { icon; title; supportingText; radii; textEndExtra; onClick(): void }): UIElement;
// ```
/** Draws one row: a 40 dp icon circle, the title and the supporting line; activating it calls [onClick]. */
@Composable
private fun settingsLinkRow(
    icon: ImageVector,
    title: String,
    supportingText: String,
    radii: SettingsRowRadii,
    textEndExtra: Dp,
    onClick: () -> Unit,
) {
    /** Corner shape of this row, with the group radius at each of its four corners. */
    val shape: RoundedCornerShape = RoundedCornerShape(
        topStart = radii.topDp.dp,
        topEnd = radii.topDp.dp,
        bottomStart = radii.bottomDp.dp,
        bottomEnd = radii.bottomDp.dp,
    )
    Row(
        modifier = Modifier.fillMaxWidth()
            .defaultMinSize(minHeight = SETTINGS_ROW_HEIGHT_DP.dp)
            .focusRing(shape)
            .clip(shape)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(
                start = SETTINGS_ROW_INNER_PADDING_DP.dp,
                end = SETTINGS_ROW_INNER_PADDING_DP.dp + textEndExtra,
                top = SETTINGS_ROW_VERTICAL_INSET,
                bottom = SETTINGS_ROW_VERTICAL_INSET,
            ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        settingsIconCircle(icon = icon, onChosenRow = false)
        Spacer(modifier = Modifier.width(SETTINGS_ICON_TITLE_GAP_DP.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
            Text(
                text = title,
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = SETTINGS_SWITCH_TITLE_LINES,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = supportingText,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = SETTINGS_SWITCH_SUPPORTING_LINES,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

// What:     `@Composable private fun settingsSwitchRow(...)` draws one switch row of the stock Settings form.
// Why:      D129: the 72 dp row with the 40 dp icon circle, the title, one supporting line and a trailing Material
//           switch. The whole row is the toggle target (role Switch, state on or off), so a tap, Space and Enter
//           flip it in place, and it carries no `selected` state because choosing a template row is a different act.
//
// In TS you'd write (pseudocode):
// ```ts
// function SwitchRow(props: {
//   icon; title; supportingText; checked; radii; textEndExtra; onToggle(): void }): UIElement;
// ```
/**
 * Draws one row: a 40 dp icon circle, the title, the supporting line and a trailing switch.
 * The switch shows the state through its thumb position, its track fill and the mark in its thumb.
 */
@Composable
private fun settingsSwitchRow(
    icon: ImageVector,
    title: String,
    supportingText: String,
    checked: Boolean,
    radii: SettingsRowRadii,
    textEndExtra: Dp,
    onToggle: () -> Unit,
) {
    /** Corner shape of this row, with the group radius at each of its four corners. */
    val shape: RoundedCornerShape = RoundedCornerShape(
        topStart = radii.topDp.dp,
        topEnd = radii.topDp.dp,
        bottomStart = radii.bottomDp.dp,
        bottomEnd = radii.bottomDp.dp,
    )
    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        /** True when the row is too narrow for the switch beside the text. */
        val stacked: Boolean = maxWidth < SETTINGS_SWITCH_STACK_BELOW
        Row(
            modifier = Modifier.fillMaxWidth()
                .defaultMinSize(minHeight = SETTINGS_ROW_HEIGHT_DP.dp)
                .focusRing(shape)
                .clip(shape)
                .toggleable(value = checked, role = Role.Switch, onValueChange = { onToggle() })
                .padding(
                    start = SETTINGS_ROW_INNER_PADDING_DP.dp,
                    end = SETTINGS_ROW_INNER_PADDING_DP.dp + textEndExtra,
                    top = SETTINGS_ROW_VERTICAL_INSET,
                    bottom = SETTINGS_ROW_VERTICAL_INSET,
                ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            settingsIconCircle(icon = icon, onChosenRow = false)
            Spacer(modifier = Modifier.width(SETTINGS_ICON_TITLE_GAP_DP.dp))
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
                Text(
                    text = title,
                    color = MaterialTheme.colorScheme.onSurface,
                    style = MaterialTheme.typography.bodyLarge,
                    maxLines = SETTINGS_SWITCH_TITLE_LINES,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = supportingText,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = SETTINGS_SWITCH_SUPPORTING_LINES,
                    overflow = TextOverflow.Ellipsis,
                )
                if (stacked) {
                    Spacer(modifier = Modifier.height(SETTINGS_SWITCH_STACKED_GAP))
                    settingsSwitchControl(checked = checked)
                }
            }
            if (!stacked) {
                Spacer(modifier = Modifier.width(SETTINGS_SWITCH_GAP))
                settingsSwitchControl(checked = checked)
            }
        }
    }
}

// What:     `@Composable private fun settingsSwitchControl(checked: Boolean)` draws the Material 3 switch of a row.
// Why:      The row owns the toggle, so the switch only draws the state (thumb position, track fill and the mark in the
//           thumb) and has no click or semantics of its own. One function serves the beside and the stacked place.
//
// In TS you'd write (pseudocode):
// ```ts
// function SwitchControl(props: { checked: boolean }): UIElement;
// ```
/** Draws the switch for [checked] with the player's colors and a check or cross mark in the thumb. */
@Composable
private fun settingsSwitchControl(checked: Boolean) {
    Switch(
        modifier = Modifier.testTag(AUTO_PLAY_SWITCH_TAG),
        checked = checked,
        onCheckedChange = null,
        thumbContent = {
            Icon(
                imageVector = if (checked) SETTINGS_SWITCH_ON_ICON else SETTINGS_SWITCH_OFF_ICON,
                contentDescription = null,
                modifier = Modifier.size(SwitchDefaults.IconSize),
            )
        },
        colors = settingsSwitchColors(),
    )
}

// What:     `@Composable private fun settingsSwitchColors(): SwitchColors` maps the switch parts onto player roles.
// Why:      The stock switch colors read slots such as the outline and the highest surface container; these explicit
//           picks keep the on state in the accent pair and the off state on the canvas with an outline border,
//           with no new color role (D129).
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsSwitchColors = () => ({ checkedTrack: primary, checkedThumb: onPrimary, uncheckedTrack: canvas, ... });
// ```
/** Switch colors: accent track with an on-accent thumb when on, canvas track with an outline border when off. */
@Composable
private fun settingsSwitchColors(): SwitchColors = SwitchDefaults.colors(
    checkedThumbColor = MaterialTheme.colorScheme.onPrimary,
    checkedTrackColor = MaterialTheme.colorScheme.primary,
    checkedBorderColor = MaterialTheme.colorScheme.primary,
    checkedIconColor = MaterialTheme.colorScheme.primary,
    uncheckedThumbColor = MaterialTheme.colorScheme.outline,
    uncheckedTrackColor = MaterialTheme.colorScheme.surface,
    uncheckedBorderColor = MaterialTheme.colorScheme.outline,
    uncheckedIconColor = MaterialTheme.colorScheme.surface,
)

// What:     `@Composable private fun settingsHeader(...)` draws the static top bar with the round Back button
//           and the title.
// Why:      Stock Settings keeps the Back circle and the page title on the page background, with no divider below.
//
// In TS you'd write (pseudocode):
// ```ts
// function SettingsHeader(props: { title: string; backDescription: string; onBack(): void }): UIElement;
// ```
/** Draws the static top bar on the canvas: a 56 dp round Back target followed by the page title. */
@Composable
internal fun settingsHeader(title: String, backDescription: String, onBack: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().height(SETTINGS_HEADER_HEIGHT)
            .padding(start = SETTINGS_PAGE_MARGIN_DP.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // What:     `Box` with `clip` and `clickable` is the round Back target.
        // Why:      The clip bounds the ripple and hover indication to the circle, and the Back role
        //           announces the button; it has no fill so the header stays the canvas color.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <button style={{ width: 56, height: 56, borderRadius: '50%', background: 'none' }}
        //   aria-label={backDescription}/>
        // ```
        /** Round button that holds the Back arrow and returns to the player. */
        Box(
            modifier = Modifier.size(SETTINGS_BACK_SIZE).focusRing(CircleShape).clip(CircleShape)
                .clickable(role = Role.Button, onClick = onBack),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = DESTINATION_BACK_ICON,
                contentDescription = backDescription,
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        Text(
            text = title,
            modifier = Modifier.weight(1f).padding(start = SETTINGS_TITLE_GAP).semantics { heading() },
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.titleLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

// What:     `@Composable private fun settingsSectionHeading(...)` draws the Templates heading on the canvas.
// Why:      The heading names its group in the primary color, and is marked as a heading for assistive technology.
//
// In TS you'd write (pseudocode):
// ```ts
// function SectionHeading(props: { text: string }): UIElement;
// ```
/** Draws one section heading in the primary label role, 24 dp in from the page edge. */
@Composable
private fun settingsSectionHeading(text: String) {
    Text(
        text = text,
        modifier = Modifier.padding(
            start = SETTINGS_HEADING_INSET,
            top = SETTINGS_HEADING_TOP,
            end = SETTINGS_HEADING_INSET,
            bottom = SETTINGS_HEADING_BOTTOM,
        ).semantics { heading() },
        color = MaterialTheme.colorScheme.primary,
        style = MaterialTheme.typography.labelLarge,
    )
}

// What:     `private fun settingsIconOf(template: TemplateKind): ImageVector` picks the icon of one template row.
// Why:      D109 gives each template its own Material Symbol, so every row has the stock leading icon.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsIconOf = (t: TemplateKind) => t === "track" ? QUEUE_MUSIC : PLAY_CIRCLE;
// ```
/** Returns the leading icon of the row of one template. */
private fun settingsIconOf(template: TemplateKind): ImageVector = when (template) {
    TemplateKind.TRACK_ROWS -> SETTINGS_TRACK_ROWS_ICON
    TemplateKind.PLAYING_TRACK -> SETTINGS_PLAYING_TRACK_ICON
}

// What:     `@Composable private fun settingsTemplateRow(...)` draws one template as a tappable row.
// Why:      The row names the template and shows the line it yields, and the whole row opens or selects its editor.
//           The row has no fill of its own: its rounded shape only bounds the press indication and, in the wide form,
//           carries the selection fill.
//
// In TS you'd write (pseudocode):
// ```ts
// function TemplateRow(props: { template: TemplateKind; line: string; radii: SettingsRowRadii;
//   selected: boolean | null; textEndExtra: number; onChoose(): void }): UIElement;
// ```
/**
 * Draws one row: a 40 dp icon circle, the label and the line it yields now.
 * [selected] is null in the single form; in the wide form the chosen row gets the secondary-container fill and a bold
 * label, and its label is sized by a hidden bold copy so choosing it moves nothing.
 */
@Composable
private fun settingsTemplateRow(
    template: TemplateKind,
    line: String,
    radii: SettingsRowRadii,
    selected: Boolean?,
    textEndExtra: Dp,
    onChoose: () -> Unit,
) {
    /** Corner shape of this row, with the group radius or the joint radius at each of its four corners. */
    val shape: RoundedCornerShape = RoundedCornerShape(
        topStart = radii.topDp.dp,
        topEnd = radii.topDp.dp,
        bottomStart = radii.bottomDp.dp,
        bottomEnd = radii.bottomDp.dp,
    )
    /** Whether this row is the chosen one of the wide form. */
    val isChosen: Boolean = selected == true
    /** Fill of the chosen row: the secondary container role, or no fill for the others. */
    val fill: Color = if (isChosen) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent
    // What:     `Row` with `clip`, `background`, `clickable` and `padding` is one row.
    // Why:      Clipping before the clickable bounds the ripple and hover indication to the row's own shape, and the
    //           padding after it keeps the whole row as the tap target.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // <button style={{ minHeight: 72, padding: '12px 16px', borderRadius: radii, background: fill }}/>
    // ```
    Row(
        modifier = Modifier.fillMaxWidth()
            .defaultMinSize(minHeight = SETTINGS_ROW_HEIGHT_DP.dp)
            .focusRing(shape)
            .clip(shape)
            .background(fill)
            .semantics { if (selected != null) this.selected = selected }
            .clickable(role = Role.Button, onClick = onChoose)
            .padding(
                start = SETTINGS_ROW_INNER_PADDING_DP.dp,
                end = SETTINGS_ROW_INNER_PADDING_DP.dp + textEndExtra,
                top = SETTINGS_ROW_VERTICAL_INSET,
                bottom = SETTINGS_ROW_VERTICAL_INSET,
            ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        settingsIconCircle(icon = settingsIconOf(template), onChosenRow = isChosen)
        Spacer(modifier = Modifier.width(SETTINGS_ICON_TITLE_GAP_DP.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
            settingsRowTitle(text = template.label, bold = isChosen)
            Text(
                text = line,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

// What:     `@Composable private fun settingsIconCircle(...)` draws the leading icon in its 40 dp circle.
// Why:      D109 fills the circle with the secondary container role and draws the icon in on-secondary-container; on
//           the chosen row the row itself is that container, so the circle takes the secondary role and keeps its
//           edge visible.
//
// In TS you'd write (pseudocode):
// ```ts
// function IconCircle(props: { icon: ImageVector; onChosenRow: boolean }): UIElement;
// ```
/**
 * Draws a 40 dp circle holding a 24 dp icon, in the secondary container pair or, on the chosen row,
 * the secondary pair.
 */
@Composable
private fun settingsIconCircle(icon: ImageVector, onChosenRow: Boolean) {
    Box(
        modifier = Modifier.size(SETTINGS_ICON_CIRCLE_DP.dp).clip(CircleShape).background(
            if (onChosenRow) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.secondaryContainer,
        ),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(SETTINGS_ICON_GLYPH_DP.dp),
            tint = if (onChosenRow) {
                MaterialTheme.colorScheme.onSecondary
            } else {
                MaterialTheme.colorScheme.onSecondaryContainer
            },
        )
    }
}

// What:     `@Composable private fun settingsRowTitle(...)` draws a row's label, bold when its row is chosen.
// Why:      A bold label is wider, so a transparent bold copy sizes the label for every state and choosing a row
//           moves no text, the same technique as the Folders panel.
//
// In TS you'd write (pseudocode):
// ```ts
// <span><span style={{ fontWeight: 700, color: 'transparent' }} aria-hidden>{text}</span>{text}</span>
// ```
/** Draws the label at bold weight when [bold], and always sizes it by its bold width. */
@Composable
private fun settingsRowTitle(text: String, bold: Boolean) {
    Box {
        Text(
            text = text,
            modifier = Modifier.clearAndSetSemantics {},
            color = Color.Transparent,
            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = text,
            color = MaterialTheme.colorScheme.onSurface,
            style = if (bold) {
                MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold)
            } else {
                MaterialTheme.typography.bodyLarge
            },
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
