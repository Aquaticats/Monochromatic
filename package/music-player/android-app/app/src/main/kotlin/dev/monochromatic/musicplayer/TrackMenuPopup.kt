// What:     `package dev.monochromatic.musicplayer` places the track menu popup beside the other player composables.
// Why:      The popup is a stateless composable that the track list imports without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the action type, its wording, and the placement.
// Why:      The popup renders the closed action list and the accepted labels without owning either, and it places
//           itself with the pure placement rule.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TrackMenuAction, label } from "core/TrackMenuAction";
// import { MenuAnchorRect, menuAnchorOffset } from "core/MenuAnchor";
// ```
import androidx.compose.ui.input.key.key
import dev.monochromatic.musicplayer.core.menuRowAfter
import androidx.compose.runtime.setValue
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.graphics.Color
import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.foundation.BorderStroke
import dev.monochromatic.musicplayer.core.MenuAnchorOffset
import dev.monochromatic.musicplayer.core.MenuAnchorRect
import dev.monochromatic.musicplayer.core.MenuAnchorRequest
import dev.monochromatic.musicplayer.core.TrackMenuAction
import dev.monochromatic.musicplayer.core.accessKey
import dev.monochromatic.musicplayer.core.label
import dev.monochromatic.musicplayer.core.menuAnchorOffset
import dev.monochromatic.musicplayer.core.menuRowActionFor
import dev.monochromatic.musicplayer.core.trackMenuActionForAccessKey
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type

// What:     Imports from `androidx.compose.foundation` bring in layout size, shape, scrolling, and the layout column.
// Why:      The popup caps its width, gives each row a 48dp target, scrolls long lists, and rounds the container.
//
// In TS you'd write (pseudocode):
// ```ts
// import { widthIn, heightIn, padding, roundedCorners, verticalScroll } from "compose/foundation";
// ```
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.foundation.layout` bring in the row content padding type.
// Why:      Each row reads its content with the 16dp side and 8dp vertical insets of the accepted study.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PaddingValues } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.PaddingValues

// What:     Imports from `androidx.compose.material3` bring in the surface, dividers, items, icons, and text.
// Why:      The menu container is a tonal surface, and each row is a menu item that takes the theme's colors.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Surface, DropdownMenuItem, HorizontalDivider, Icon, Text } from "material3";
// ```
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text

// What:     Imports from `androidx.compose.runtime` bring in the composable marker and the remembered provider.
// Why:      The popup emits UI, and it keeps one position provider for each row anchor and inset.
//
// In TS you'd write (pseudocode):
// ```ts
// // Composable functions are plain functions in pseudocode.
// ```
import androidx.compose.foundation.focusable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.runtime.remember

// What:     Imports from `androidx.compose.ui` bring in the modifier, the density, and the geometry and unit types.
// Why:      The popup accepts a caller modifier, reads the density for the inset, and returns its window offset.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Dp, IntOffset } from "compose/ui";
// ```
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntRect
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp

// What:     Imports from `androidx.compose.ui.window` bring in the popup, its position provider, and its properties.
// Why:      The menu is its own popup window positioned at its row, and it states its focus and dismissal flags.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Popup, PopupPositionProvider, PopupProperties } from "compose/ui/window";
// ```
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupPositionProvider
import androidx.compose.ui.window.PopupProperties

// What:     Imports from `androidx.compose.ui.text` bring in annotated text, a color span, and ellipsis overflow.
// Why:      The File details row draws its dB value in the secondary role; the heading ends in one ellipsized line.
//
// In TS you'd write (pseudocode):
// ```ts
// import { buildAnnotatedString, withStyle, SpanStyle, TextOverflow } from "compose/ui/text";
// ```
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle

// What:     `private val MENU_MAX_WIDTH: Dp = 280.dp` caps the menu width.
// Why:      The accepted candidate keeps the menu narrower than the full panel.
//
// In TS you'd write (pseudocode):
// ```ts
// const MENU_MAX_WIDTH = 280;
// ```
/** Maximum width of the popup container in density-independent units. */
private val MENU_MAX_WIDTH: Dp = 280.dp

// What:     `private val MENU_CORNER_RADIUS: Dp = 12.dp` rounds the menu container.
// Why:      The accepted candidate uses a 12dp container radius.
//
// In TS you'd write (pseudocode):
// ```ts
// const MENU_CORNER_RADIUS = 12;
// ```
/** Corner radius of the popup container. */
private val MENU_CORNER_RADIUS: Dp = 12.dp

// What:     `private val MENU_ELEVATION: Dp = 3.dp` lifts the menu container above the rows behind it.
// Why:      The menu is a floating surface, so it casts a shadow the way the platform menus do.
//
// In TS you'd write (pseudocode):
// ```ts
// const MENU_ELEVATION = 3;
// ```
/** Shadow elevation of the popup container. */
private val MENU_ELEVATION: Dp = 3.dp

// What:     `private val MENU_OUTLINE_WIDTH: Dp = 1.dp` is the thickness of the menu's outline.
// Why:      D119 puts the menu on the canvas color, so a black menu over a black page needs an edge besides its
//           shadow; the outline-variant role at 1 dp is that edge in both appearances.
//
// In TS you'd write (pseudocode):
// ```ts
// const MENU_OUTLINE_WIDTH = "1dp";
// ```
/** Thickness of the outline that marks the menu's edge. */
private val MENU_OUTLINE_WIDTH: Dp = 1.dp

// What:     `internal fun trackMenuContainerColor(scheme)` names the color the menu is filled with.
// Why:      D119 puts the menu on the pure canvas color (white in light, black in dark) with no tint; naming the choice
//           lets a test pin it to the canvas slot, which the theme tests already pin to pure white and pure black.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackMenuContainerColor = (scheme: ColorScheme) => scheme.surface;
// ```
/** Returns the fill of the track menu: the scheme's canvas slot, untinted. */
internal fun trackMenuContainerColor(scheme: ColorScheme): Color = scheme.surface

// What:     `private val ITEM_MIN_TARGET: Dp = 48.dp` is the minimum touch target of one action row.
// Why:      Every action keeps a 48dp minimum target, matching the touch-target rule of the design study.
//
// In TS you'd write (pseudocode):
// ```ts
// const ITEM_MIN_TARGET = 48;
// ```
/** Minimum width and height of one action row. */
private val ITEM_MIN_TARGET: Dp = 48.dp

// What:     `private val ROW_HORIZONTAL_PADDING: Dp = 16.dp` insets the row content from both sides.
// Why:      The accepted study uses 16dp on the start and end of each action row.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_HORIZONTAL_PADDING = 16;
// ```
/** Start and end inset of the content of one action row. */
private val ROW_HORIZONTAL_PADDING: Dp = 16.dp

// What:     `private val ROW_VERTICAL_PADDING: Dp = 8.dp` insets the row content from top and bottom.
// Why:      The accepted study uses 8dp above and below the content of each action row.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_VERTICAL_PADDING = 8;
// ```
/** Top and bottom inset of the content of one action row. */
private val ROW_VERTICAL_PADDING: Dp = 8.dp

// What:     `private val DIVIDER_HORIZONTAL_INSET: Dp = 8.dp` insets a divider from the container edges.
// Why:      The accepted candidate keeps dividers short of the menu edges.
//
// In TS you'd write (pseudocode):
// ```ts
// const DIVIDER_HORIZONTAL_INSET = 8;
// ```
/** Horizontal inset of a group divider from the container edges. */
private val DIVIDER_HORIZONTAL_INSET: Dp = 8.dp

// What:     `private val DIVIDER_VERTICAL_INSET: Dp = 5.dp` spaces a divider between action rows.
// Why:      The accepted candidate separates groups with a small vertical margin around the rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const DIVIDER_VERTICAL_INSET = 5;
// ```
/** Vertical margin above and below a group divider. */
private val DIVIDER_VERTICAL_INSET: Dp = 5.dp

// What:     `internal fun trackMenuDividerFlags(actions: List<TrackMenuAction>): List<Boolean>` marks group starts.
// Why:      A divider belongs before an action whose group differs from the one before it, never before the first.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackMenuDividerFlags(actions: readonly TrackMenuAction[]): boolean[] {
//   return actions.map((action, index) => index > 0 && action.group !== actions[index - 1].group);
// }
// ```
/** Returns for each action whether a divider is drawn immediately before it. */
internal fun trackMenuDividerFlags(actions: List<TrackMenuAction>): List<Boolean> =
    actions.mapIndexed { index, action -> index > 0 && action.group != actions[index - 1].group }

// What:     `internal fun trackMenuDetailsSuffix(detailsValue: String?): String` builds the text after File details.
// Why:      The dB value joins the label with a middle dot, and an absent or blank value adds no text at all.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackMenuDetailsSuffix(detailsValue: string | null): string {
//   return detailsValue === null || detailsValue.trim() === "" ? "" : ` · ${detailsValue}`;
// }
// ```
/** Returns the separator and value appended to File details, or an empty string when no value is present. */
internal fun trackMenuDetailsSuffix(detailsValue: String?): String =
    if (detailsValue.isNullOrBlank()) "" else " · $detailsValue"

// What:     `internal fun trackMenuPositionProvider(...)` returns the provider that places the menu at its row.
// Why:      The popup window is placed by this provider, which feeds the measured menu size to the placement rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackMenuPositionProvider = (anchor, startInsetPx) => (popupSize, window) =>
//   menuAnchorOffset({ anchor, popupWidth: popupSize.w, popupHeight: popupSize.h, windowWidth: window.w,
//     windowHeight: window.h, startInset: startInsetPx });
// ```
/**
 * Returns a provider that places the menu below its row, above it when it would overflow, and inside the window.
 *
 * The provider takes the row bounds from [anchor] and ignores the parent bounds that the popup reports, because the
 * popup is composed in the host and not inside the row. Both are in window pixels.
 */
internal fun trackMenuPositionProvider(anchor: MenuAnchorRect, startInsetPx: Int): PopupPositionProvider =
    object : PopupPositionProvider {
        // What:     `override fun calculatePosition(...)` returns the window offset of the menu for this measurement.
        // Why:      The popup asks for its position after measuring its content, so the menu size is known here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // calculatePosition(parent, window, dir, popupSize): Offset { return menuAnchorOffset(...); }
        // ```
        /** Window offset of the menu's top-left corner for the measured container size. */
        override fun calculatePosition(
            anchorBounds: IntRect,
            windowSize: IntSize,
            layoutDirection: LayoutDirection,
            popupContentSize: IntSize,
        ): IntOffset {
            /** Placement of the menu computed from the row, the measured menu, and the window. */
            val placed: MenuAnchorOffset = menuAnchorOffset(
                MenuAnchorRequest(
                    anchor = anchor,
                    popupWidth = popupContentSize.width,
                    popupHeight = popupContentSize.height,
                    windowWidth = windowSize.width,
                    windowHeight = windowSize.height,
                    startInset = startInsetPx,
                ),
            )
            return IntOffset(x = placed.x, y = placed.y)
        }
    }

// What:     `fun trackMenuPopup(...)` draws the stateless track menu anchored at its row.
// Why:      The caller decides when the menu is open and where its row sits; this composable only renders and reports.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackMenuPopup(input: {
//   actions: readonly TrackMenuAction[];
//   anchor: MenuAnchorRect;
//   onSelect: (action: TrackMenuAction) => void;
//   onDismiss: () => void;
// }): UIElement;
// ```
/**
 * Draws the track context menu in its own popup window, placed against the row bounds in [anchor].
 *
 * Escape and Back dismiss through the popup window, which calls `onDismiss` when `dismissOnBackPress` is true, as set
 * here. Outside taps dismiss through `dismissOnClickOutside`. Selecting an action only reports it.
 *
 * @param title Track name shown as the one-line heading; longer text ends in an ellipsis.
 * @param detailsValue Peak value drawn after File details in the secondary role; null draws nothing.
 * @param anchor Window-space bounds of the row that opened the menu, which the menu sits under or over.
 * @param startInset Extra start inset for the heading, rows, dividers, and the leftmost edge of the menu;
 *     12dp on the inner panel, 0 on the cover.
 * @param expanded Opens or closes the menu; a closed menu draws nothing.
 */
@Composable
fun trackMenuPopup(
    actions: List<TrackMenuAction>,
    title: String,
    detailsValue: String?,
    anchor: MenuAnchorRect,
    onSelect: (TrackMenuAction) -> Unit,
    onDismiss: () -> Unit,
    startInset: Dp = 0.dp,
    expanded: Boolean = true,
    modifier: Modifier = Modifier,
) {
    if (!expanded) {
        return
    }
    /** Density used to turn the start inset from density-independent units into pixels. */
    val density = LocalDensity.current
    /** Start inset in window pixels, which the placement rule uses as the menu's leftmost edge. */
    val startInsetPx: Int = with(density) { startInset.roundToPx() }
    /** Position provider that places the menu at its row, rebuilt only when the row or the inset changes. */
    val positionProvider: PopupPositionProvider = remember(anchor, startInsetPx) {
        trackMenuPositionProvider(anchor = anchor, startInsetPx = startInsetPx)
    }
    // What:     `val dividers` holds one divider flag per action for this render pass.
    // Why:      Each row reads its own flag, so the divider decision is computed once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const dividers = trackMenuDividerFlags(actions);
    // ```
    /** Divider flags aligned with the action list, computed once per composition. */
    val dividers = trackMenuDividerFlags(actions)
    // What:     `val secondary` is the role for the File details value.
    // Why:      The peak reads as secondary text so the action label stays primary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const secondary = scheme.onSurfaceVariant;
    // ```
    /** Secondary text color used for the dB value on the File details row. */
    val secondary = MaterialTheme.colorScheme.onSurfaceVariant
    /** Row that holds the keyboard ring, or null until the first Down or Up (D114). */
    var activeRow: Int? by remember { mutableStateOf<Int?>(null) }
    /** Focus target of the popup, so the menu's own key handler receives the keys of a hardware keyboard. */
    val menuFocus: FocusRequester = remember { FocusRequester() }
    LaunchedEffect(Unit) {
        menuFocus.requestFocus()
    }
    Popup(
        popupPositionProvider = positionProvider,
        onDismissRequest = onDismiss,
        properties = PopupProperties(focusable = true, dismissOnBackPress = true, dismissOnClickOutside = true),
    ) {
        Surface(
            modifier = modifier
                .widthIn(max = MENU_MAX_WIDTH)
                // The row chords of D106 (Delete, Alt+Enter and the Ctrl+Shift chords) act on the row the menu is
                // open on, so the popup window, which receives its own keys, asks the key table about them.
                .onPreviewKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown) {
                        when (event.key) {
                            Key.DirectionDown, Key.DirectionUp -> {
                                activeRow = menuRowAfter(
                                    current = activeRow,
                                    count = actions.size,
                                    step = if (event.key == Key.DirectionDown) 1 else -1,
                                )
                                return@onPreviewKeyEvent true
                            }
                            Key.Enter, Key.NumPadEnter -> {
                                /** Row that holds the ring, which Enter runs. */
                                val ringed: Int? = activeRow
                                if (ringed != null) {
                                    onSelect(actions[ringed])
                                    return@onPreviewKeyEvent true
                                }
                            }
                            else -> Unit
                        }
                    }
                    /**
                     * Menu action of the row chord or the access letter (D114) that this key-down presses, or null
                     * when it presses neither.
                     */
                    val chord: TrackMenuAction? = if (event.type == KeyEventType.KeyDown) {
                        hostKeyInputOf(event).let { input ->
                            menuRowActionFor(input) ?: trackMenuActionForAccessKey(input)
                        }
                    } else {
                        null
                    }
                    if (chord != null) {
                        onSelect(chord)
                    }
                    chord != null
                }
                .focusRequester(menuFocus)
                .focusable(),
            shape = RoundedCornerShape(MENU_CORNER_RADIUS),
            // D119: the menu sits on the pure canvas color, white in light and black in dark, with no tint; its edge
            // is the shadow and a 1 dp outline-variant outline, because a black menu over a black page has no shadow.
            color = trackMenuContainerColor(MaterialTheme.colorScheme),
            border = BorderStroke(width = MENU_OUTLINE_WIDTH, color = MaterialTheme.colorScheme.outlineVariant),
            shadowElevation = MENU_ELEVATION,
        ) {
            Column(
                modifier = Modifier
                    .verticalScroll(rememberScrollState())
                    .padding(bottom = FOCUS_RING_OUTSET),
            ) {
                // What:     `Text` draws the track name as the heading above the action rows.
                // Why:      D7 headed the menu with the track name, kept to one line with an ellipsis.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // heading({ text: title, maxLines: 1, overflow: "ellipsis" });
                // ```
                Text(
                    text = title,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.fillMaxWidth().padding(
                        start = ROW_HORIZONTAL_PADDING + startInset,
                        end = ROW_HORIZONTAL_PADDING,
                        top = ROW_VERTICAL_PADDING,
                        bottom = ROW_VERTICAL_PADDING,
                    ),
                )
                for ((index, action) in actions.withIndex()) {
                    if (dividers[index]) {
                        HorizontalDivider(
                            modifier = Modifier.padding(
                                start = DIVIDER_HORIZONTAL_INSET + startInset,
                                end = DIVIDER_HORIZONTAL_INSET,
                                top = DIVIDER_VERTICAL_INSET,
                                bottom = DIVIDER_VERTICAL_INSET,
                            ),
                        )
                    }
                    // What:     `val foreground` selects the text and icon role for one row.
                    // Why:      The destructive row uses the error role in addition to its label and trash glyph.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const foreground = action.destructive ? scheme.error : scheme.onSurface;
                    // ```
                    /** Text and icon color role of this row; the error role marks the destructive action. */
                    val foreground = if (action.destructive) {
                        MaterialTheme.colorScheme.error
                    } else {
                        MaterialTheme.colorScheme.onSurface
                    }
                    // What:     `val rowText` appends the dB value to File details in the secondary role.
                    // Why:      Other rows have a single plain label, so only the details row builds a span.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const rowText = [label(action), action === "details" ? span(secondary, suffix) : null];
                    // ```
                    /** Visible text of this row: access letter underlined, File details value in the secondary role. */
                    val rowText = buildAnnotatedString {
                        append(label(action))
                        addStyle(
                            SpanStyle(textDecoration = TextDecoration.Underline),
                            accessKey(action).labelIndex,
                            accessKey(action).labelIndex + 1,
                        )
                        if (action == TrackMenuAction.DETAILS) {
                            withStyle(SpanStyle(color = secondary)) {
                                append(trackMenuDetailsSuffix(detailsValue))
                            }
                        }
                    }
                    DropdownMenuItem(
                        text = { Text(text = rowText) },
                        onClick = { onSelect(action) },
                        leadingIcon = { Icon(imageVector = trackMenuIcon(action), contentDescription = null) },
                        colors = MenuDefaults.itemColors(textColor = foreground, leadingIconColor = foreground),
                        contentPadding = PaddingValues(
                            start = ROW_HORIZONTAL_PADDING + startInset,
                            end = ROW_HORIZONTAL_PADDING,
                            top = ROW_VERTICAL_PADDING,
                            bottom = ROW_VERTICAL_PADDING,
                        ),
                        modifier = Modifier
                            .padding(horizontal = FOCUS_RING_OUTSET)
                            .widthIn(min = ITEM_MIN_TARGET)
                            .heightIn(min = ITEM_MIN_TARGET)
                            .keyboardFocusRing(visible = activeRow == index, shape = RectangleShape),
                    )
                }
            }
        }
    }
}
