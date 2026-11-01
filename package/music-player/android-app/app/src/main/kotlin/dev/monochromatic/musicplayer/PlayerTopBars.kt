// What:     `package dev.monochromatic.musicplayer` places the top bars beside the other player composables.
// Why:      The bars are stateless composables that screens import without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.foundation` bring in layout, clicking, and inset helpers.
// Why:      The bars pad for the status bar, size their touch targets, and make the folder title clickable.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clickable } from "compose/foundation";
// ```
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding

// What:     Imports from `androidx.compose.material3` bring in buttons, icon buttons, text, and the theme accessor.
// Why:      The bars use Material 3 roles and the tonal Open button, so they follow the app theme.
//
// In TS you'd write (pseudocode):
// ```ts
// import { FilledTonalButton, IconButton, Text, MaterialTheme } from "material3";
// ```
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

// What:     Imports from `androidx.compose.ui` bring in the modifier, semantics, role, and overflow helpers.
// Why:      The folder title publishes its name and expanded state, and the headings publish heading semantics.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Role, heading } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private val TOP_BAR_MIN_HEIGHT: Dp = 56.dp` is the minimum height of each top bar row.
// Why:      The Material app bar uses 56dp, so the bars match the standard height.
//
// In TS you'd write (pseudocode):
// ```ts
// const TOP_BAR_MIN_HEIGHT = 56;
// ```
/** Minimum height of one top bar row, below the status bar inset. */
private val TOP_BAR_MIN_HEIGHT: Dp = 56.dp

// What:     `private val TOUCH_TARGET: Dp = 48.dp` is the minimum side of each tappable control in the bars.
// Why:      Every icon button and the folder trigger keep a 48dp target.
//
// In TS you'd write (pseudocode):
// ```ts
// const TOUCH_TARGET = 48;
// ```
/** Minimum width and height of each tappable control in the bars. */
private val TOUCH_TARGET: Dp = 48.dp

// What:     `private val BAR_START_PADDING: Dp = 16.dp` is the start inset of each bar row.
// Why:      The row starts at the same 16dp inset as the folder names and the transport deck.
//
// In TS you'd write (pseudocode):
// ```ts
// const BAR_START_PADDING = 16;
// ```
/** Start padding of each top bar row. */
private val BAR_START_PADDING: Dp = 16.dp

// What:     `private val BAR_END_PADDING: Dp = 8.dp` is the end inset of each bar row.
// Why:      The trailing action sits closer to the edge than the leading title, as in the Material app bar.
//
// In TS you'd write (pseudocode):
// ```ts
// const BAR_END_PADDING = 8;
// ```
/** End padding of each top bar row. */
private val BAR_END_PADDING: Dp = 8.dp

// What:     `private val ACTION_SPACING: Dp = 8.dp` is the gap between the bar's controls.
// Why:      The accepted cover bar spaces its title, Open button, and settings icon at 8dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const ACTION_SPACING = 8;
// ```
/** Horizontal gap between the title and each control in a top bar row. */
private val ACTION_SPACING: Dp = 8.dp

// What:     `internal fun folderTriggerContentDescription(folderTitle: String): String` builds the spoken
//           name of the trigger.
// Why:      Screen readers hear the folder name with the word Folder, as the design study does.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderTriggerContentDescription(title: string): string { return `Folder: ${title}`; }
// ```
/** Returns the spoken name of the folder title trigger. */
internal fun folderTriggerContentDescription(folderTitle: String): String = "Folder: $folderTitle"

// What:     `internal fun folderTriggerStateDescription(pickerOpen: Boolean): String` names the trigger's state.
// Why:      Screen readers announce whether the picker is expanded or collapsed, matching the caret direction.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderTriggerStateDescription(open: boolean): string { return open ? "Expanded" : "Collapsed"; }
// ```
/** Returns the spoken expanded or collapsed state of the folder title trigger. */
internal fun folderTriggerStateDescription(pickerOpen: Boolean): String =
    if (pickerOpen) "Expanded" else "Collapsed"

// What:     `@Composable private fun folderTitleTrigger(...)` draws the folder title with a caret and no container.
// Why:      Decision D104 makes the app-bar title itself the trigger, with the caret pointing up while
//           open and down while closed.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderTitleTrigger(props: { title: string; pickerOpen: boolean; onToggle(): void }): UIElement;
// ```
/** Draws the folder title and caret as one button-role target that toggles the picker. */
@Composable
private fun folderTitleTrigger(
    folderTitle: String,
    pickerOpen: Boolean,
    onToggleFolderPicker: () -> Unit,
    modifier: Modifier,
) {
    Row(
        modifier = modifier
            .defaultMinSize(minHeight = TOUCH_TARGET)
            .focusRing(RectangleShape)
            .clickable(role = Role.Button, onClick = onToggleFolderPicker)
            .semantics {
                contentDescription = folderTriggerContentDescription(folderTitle)
                stateDescription = folderTriggerStateDescription(pickerOpen)
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // The title is always one line and a name too long for the room loses its middle, so both ends stay readable.
        // A second line, or reserving one, left the caret under a short name and the actions on a row of their own
        // on a phone at large text (PhoneBarsWiringTest).
        Text(
            text = folderTitle,
            modifier = Modifier.weight(1f, fill = false),
            style = MaterialTheme.typography.titleMedium,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
        )
        Icon(
            imageVector = if (pickerOpen) ARROW_DROP_UP_ICON else ARROW_DROP_DOWN_ICON,
            contentDescription = null,
        )
    }
}

// What:     `@Composable private fun openButton(...)` draws the tonal Open action with its folder glyph.
// Why:      Open changes the folder, so it is a visible labeled action, shared by both headers.
//
// In TS you'd write (pseudocode):
// ```ts
// function openButton(props: { onOpen(): void }): UIElement;
// ```
/** Draws the Open action as a filled tonal button with the folder glyph and its label. */
@Composable
private fun openButton(onOpen: () -> Unit) {
    FilledTonalButton(
        onClick = onOpen,
        modifier = Modifier.defaultMinSize(minHeight = TOUCH_TARGET).focusRing(CircleShape),
    ) {
        Icon(
            imageVector = FOLDER_OPEN_ICON,
            contentDescription = null,
            modifier = Modifier.size(ButtonDefaults.IconSize),
        )
        Box(modifier = Modifier.width(ButtonDefaults.IconSpacing))
        Text(text = "Open")
    }
}

// What:     `@Composable private fun settingsButton(...)` draws the settings icon button.
// Why:      Every bar exposes settings, and one shared button keeps its target and label identical.
//
// In TS you'd write (pseudocode):
// ```ts
// function settingsButton(props: { onSettings(): void }): UIElement;
// ```
/** Draws the settings icon button with a 48dp target and the spoken label Settings. */
@Composable
private fun settingsButton(onSettings: () -> Unit) {
    IconButton(onClick = onSettings, modifier = Modifier.size(TOUCH_TARGET).focusRing(CircleShape)) {
        Icon(imageVector = SETTINGS_ICON, contentDescription = "Settings")
    }
}

// What:     `@Composable private fun searchButton(...)` draws the search icon button.
// Why:      Search appears only where a search page exists, so the caller passes its action or nothing.
//
// In TS you'd write (pseudocode):
// ```ts
// function searchButton(props: { onSearch(): void }): UIElement;
// ```
/** Draws the search icon button with a 48dp target and the spoken label Search music. */
@Composable
private fun searchButton(onSearch: () -> Unit) {
    IconButton(onClick = onSearch, modifier = Modifier.size(TOUCH_TARGET).focusRing(CircleShape)) {
        Icon(imageVector = SEARCH_ICON, contentDescription = "Search music")
    }
}

// What:     `@Composable fun coverTopRow(...)` draws the cover screen's app bar with the folder title trigger.
// Why:      The folder title is the picker trigger and carries the caret that shows whether the picker is open (D104).
//
// In TS you'd write (pseudocode):
// ```ts
// function coverTopRow(props: { folderTitle: string; pickerOpen: boolean; onToggleFolderPicker(): void;
//   onOpen(): void; onSettings(): void; onSearch?: () => void; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the cover screen's top row: the folder title trigger with its caret, an optional search action,
 * the Open button, and the settings action.
 *
 * The row pads for the status bar itself, so the caller places it at the top of the screen without extra inset.
 *
 * @param folderTitle name of the current folder, shown as the trigger label
 * @param pickerOpen true while the folder picker is open, which points the caret up
 * @param onToggleFolderPicker invoked when the title trigger is tapped
 * @param onOpen invoked when the Open button is tapped
 * @param onSettings invoked when the settings button is tapped
 * @param onSearch invoked when the search button is tapped, or null to hide the search button
 * @param modifier outer modifier applied to the row
 */
@Composable
fun coverTopRow(
    folderTitle: String,
    pickerOpen: Boolean,
    onToggleFolderPicker: () -> Unit,
    onOpen: () -> Unit,
    onSettings: () -> Unit,
    onSearch: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = TOP_BAR_MIN_HEIGHT)
            .windowInsetsPadding(WindowInsets.statusBars)
            .padding(start = BAR_START_PADDING, end = BAR_END_PADDING),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ACTION_SPACING),
    ) {
        folderTitleTrigger(
            folderTitle = folderTitle,
            pickerOpen = pickerOpen,
            onToggleFolderPicker = onToggleFolderPicker,
            modifier = Modifier.weight(1f),
        )
        if (onSearch != null) {
            searchButton(onSearch = onSearch)
        }
        openButton(onOpen = onOpen)
        settingsButton(onSettings = onSettings)
    }
}

// What:     `@Composable fun unfoldedFoldersHeader(...)` draws the left pane header with the Folders title and Open.
// Why:      The unfolded left pane names its list and offers Open, so the header carries both.
//
// In TS you'd write (pseudocode):
// ```ts
// function unfoldedFoldersHeader(props: { title: string; onOpen(): void; modifier?: Modifier }): UIElement;
// ```
/** Draws the unfolded left pane header: the opened folder's name [title] on the left, the Open button on the right. */
@Composable
fun unfoldedFoldersHeader(title: String, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = TOP_BAR_MIN_HEIGHT)
            .windowInsetsPadding(WindowInsets.statusBars)
            .padding(start = BAR_START_PADDING, end = BAR_END_PADDING),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ACTION_SPACING),
    ) {
        Text(
            text = title,
            modifier = Modifier
                .weight(1f)
                .semantics { heading() },
            style = MaterialTheme.typography.titleLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        openButton(onOpen = onOpen)
    }
}

// What:     `@Composable fun unfoldedTrackHeader(...)` draws the right pane header with the folder title and actions.
// Why:      The unfolded track pane names the folder whose tracks it lists, with search and settings beside it.
//
// In TS you'd write (pseudocode):
// ```ts
// function unfoldedTrackHeader(props: { folderTitle: string; onSettings(): void;
//   onSearch?: () => void; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the unfolded right pane header: the folder title, then the optional search and the settings action.
 *
 * @param folderTitle name of the folder whose tracks the pane lists
 * @param onSettings invoked when the settings button is tapped
 * @param onSearch invoked when the search button is tapped, or null to hide the search button
 * @param modifier outer modifier applied to the row
 */
@Composable
fun unfoldedTrackHeader(
    folderTitle: String,
    onSettings: () -> Unit,
    onSearch: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = TOP_BAR_MIN_HEIGHT)
            .windowInsetsPadding(WindowInsets.statusBars)
            .padding(start = paneStartInset(BAR_START_PADDING), end = BAR_END_PADDING),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ACTION_SPACING),
    ) {
        Text(
            text = folderTitle,
            modifier = Modifier
                .weight(1f)
                .semantics { heading() },
            style = MaterialTheme.typography.titleLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (onSearch != null) {
            searchButton(onSearch = onSearch)
        }
        settingsButton(onSettings = onSettings)
    }
}
