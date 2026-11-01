// What:     `package dev.monochromatic.musicplayer` places the template editor beside the other player composables.
// Why:      The editor is a stateless composable the host screen calls without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the editor state, its edit value, its preview and field records, and help.
// Why:      The composable only draws what the state yields and reports edits back in the same types.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TemplateEditorState, TemplateEditorEdit, TemplateHelp } from "core/TemplateEditorState";
// ```
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.graphics.RectangleShape
import dev.monochromatic.musicplayer.core.FocusRingPlacement
import dev.monochromatic.musicplayer.core.TemplateEditorEdit
import dev.monochromatic.musicplayer.core.TemplateEditorFieldEntry
import dev.monochromatic.musicplayer.core.TemplateEditorState
import dev.monochromatic.musicplayer.core.TemplateHelp
import dev.monochromatic.musicplayer.core.TemplatePreviewRow

// What:     Imports from `androidx.compose.foundation` bring in backgrounds, clicks, scrolling and the text
//           keyboard options.
// Why:      The editor scrolls its whole page so the lines under the field can be reached with the keyboard open.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clickable, verticalScroll } from "compose/foundation";
// ```
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.material3` bring in the Material 3 parts the page is built from.
// Why:      The page uses the platform's own list row, outlined text field, buttons and dividers,
//           so it follows the theme.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ListItem, OutlinedTextField, TextButton, HorizontalDivider, Icon, IconButton, Text } from "material3";
// ```
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable

// What:     Imports from `androidx.compose.ui` bring in modifiers, alignment, focus, colour and semantics.
// Why:      The page marks headings and buttons for assistive technology and reads focus from the field.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier, Color, heading, Role } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `internal val DESTINATION_SIDE_INSET: Dp = 16.dp` is the inset between the page edge and its text.
// Why:      Every row, heading and field in the page starts at the same inset, and Settings shares it.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_SIDE_INSET = 16;
// ```
/** Horizontal inset between the page edge and its text, fields and headings. */
internal val DESTINATION_SIDE_INSET: Dp = 16.dp

// What:     `internal val DESTINATION_ROW_INSET: Dp = 16.dp` is the side padding a Material list item already adds.
// Why:      Rows subtract it from the page inset so their text lines up with the page's text.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_ROW_INSET = 16;
// ```
/** Side padding a Material list item adds inside its own edges. */
internal val DESTINATION_ROW_INSET: Dp = 16.dp

// What:     `private val HEADER_HEIGHT: Dp = 72.dp` is the height of the destination header.
// Why:      The Settings and editor headers share the height of the accepted header band.
//
// In TS you'd write (pseudocode):
// ```ts
// const HEADER_HEIGHT = 72;
// ```
/** Height of the header band that holds the Back target and the title. */
private val HEADER_HEIGHT: Dp = 72.dp

// What:     `internal val DESTINATION_TOUCH_TARGET: Dp = 48.dp` is the smallest touch target of any control.
// Why:      Every tappable row, the Back target and the reset button keep at least this size.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_TOUCH_TARGET = 48;
// ```
/** Smallest height or width of any tappable control on the page. */
internal val DESTINATION_TOUCH_TARGET: Dp = 48.dp

// What:     `private val ROW_MIN_HEIGHT: Dp = 56.dp` is the minimum height of one preview row.
// Why:      A preview row keeps the height of a two-line track row and grows with larger text.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_MIN_HEIGHT = 56;
// ```
/** Minimum height of one preview row, the height of a two-line track row. */
private val ROW_MIN_HEIGHT: Dp = 56.dp

// What:     `private val ROW_VERTICAL_PADDING: Dp = 8.dp` is the inset above and below one preview row's text.
// Why:      Eight dp above and below the two lines makes exactly the minimum row height at the default text size.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROW_VERTICAL_PADDING = 8;
// ```
/** Vertical inset above and below the two text lines of a preview row. */
private val ROW_VERTICAL_PADDING: Dp = 8.dp

// What:     `internal val DESTINATION_SECTION_GAP: Dp = 16.dp` is the space above each section heading.
// Why:      Headings start a new group with the same space everywhere on the page and in Settings.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_SECTION_GAP = 16;
// ```
/** Space above each section heading. */
internal val DESTINATION_SECTION_GAP: Dp = 16.dp

// What:     `internal val DESTINATION_SMALL_GAP: Dp = 8.dp` is the space between a heading or note and its neighbour.
// Why:      Notes, buttons and headings sit close together so they read as one group.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_SMALL_GAP = 8;
// ```
/** Space between a heading or note and the element next to it. */
internal val DESTINATION_SMALL_GAP: Dp = 8.dp

// What:     `internal val DESTINATION_HEADING_GAP: Dp = 4.dp` is the space below a section heading.
// Why:      The heading sits close to the content it names.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_HEADING_GAP = 4;
// ```
/** Space between a section heading and the content under it. */
internal val DESTINATION_HEADING_GAP: Dp = 4.dp

// What:     `internal val DESTINATION_HAIRLINE: Dp = 1.dp` is the thickness of every divider on the page.
// Why:      The page and Settings use the same single-pixel-class divider between rows and below the header.
//
// In TS you'd write (pseudocode):
// ```ts
// const DESTINATION_HAIRLINE = 1;
// ```
/** Thickness of each divider between rows and below the header. */
internal val DESTINATION_HAIRLINE: Dp = 1.dp

// What:     `private val ERROR_ICON_SIZE: Dp = 16.dp` is the square size of the mistake icon.
// Why:      The icon sits beside the first text line of its mistake without outgrowing it.
//
// In TS you'd write (pseudocode):
// ```ts
// const ERROR_ICON_SIZE = 16;
// ```
/** Square size of the icon beside each mistake line. */
private val ERROR_ICON_SIZE: Dp = 16.dp

// What:     `private val ICON_TEXT_GAP: Dp = 4.dp` is the space between a mistake icon and its text.
// Why:      The mistake text sits close to its icon so the pair reads as one line.
//
// In TS you'd write (pseudocode):
// ```ts
// const ICON_TEXT_GAP = 4;
// ```
/** Space between a mistake icon and the text beside it. */
private val ICON_TEXT_GAP: Dp = 4.dp

// What:     `private val BUTTON_LABEL_INSET: Dp = 12.dp` is the padding a text button adds around its label.
// Why:      The reset button's label lines up with the page text, so the button starts this far before the page inset.
//
// In TS you'd write (pseudocode):
// ```ts
// const BUTTON_LABEL_INSET = 12;
// ```
/** Padding a Material text button adds on each side of its label. */
private val BUTTON_LABEL_INSET: Dp = 12.dp

// What:     `data class TemplateEditorActions(...)` holds the callbacks the editor reports to its host.
// Why:      The host applies each report to its state, so the composable itself stays stateless.
//
// In TS you'd write (pseudocode):
// ```ts
// type TemplateEditorActions = { onEdit(edit): void; onFocusChanged(focused): void; onInsertField(mode): void;
//   onReset(): void; onBack(): void };
// ```
/** Callbacks the template editor reports to the host that owns the state. */
data class TemplateEditorActions(
    /** Reports one edit of the template field with its text and selection. */
    val onEdit: (TemplateEditorEdit) -> Unit,
    /** Reports whether the template field gained or lost keyboard focus. */
    val onFocusChanged: (Boolean) -> Unit,
    /** Reports the mode word of the field tapped in the field list. */
    val onInsertField: (String) -> Unit,
    /** Reports a tap on the way back to the default text. */
    val onReset: () -> Unit,
    /** Reports a tap on the Back target that returns to Settings. */
    val onBack: () -> Unit,
)

// What:     `internal fun templateEditorTextValue(state: TemplateEditorState): TextFieldValue` converts the state
//           into the field's value.
// Why:      The field needs the text and selection together, and the state keeps them as plain values.
//
// In TS you'd write (pseudocode):
// ```ts
// function templateEditorTextValue(state) { return { text, selectionStart, selectionEnd }; }
// ```
/** The field value for the state's open template text and selection. */
internal fun templateEditorTextValue(state: TemplateEditorState): TextFieldValue = TextFieldValue(
    text = state.text,
    selection = TextRange(state.selectionStart, state.selectionEnd),
)

// What:     `internal fun templateEditorEditOf(value: TextFieldValue): TemplateEditorEdit` converts a field report.
// Why:      The state takes plain text and offsets, so the field's selection object is unpacked here.
//
// In TS you'd write (pseudocode):
// ```ts
// function templateEditorEditOf(value) { return { text: value.text, selectionStart: value.start }; }
// ```
/** The edit the state takes for one value the field reported. */
internal fun templateEditorEditOf(value: TextFieldValue): TemplateEditorEdit = TemplateEditorEdit(
    text = value.text,
    selectionStart = value.selection.start,
    selectionEnd = value.selection.end,
)

// What:     `internal fun templateEditorFieldValueText(value: String): String` names a field with no value.
// Why:      A blank value in the list would read as a missing row, so the list says so in words.
//
// In TS you'd write (pseudocode):
// ```ts
// const fieldValueText = (value: string) => value === "" ? "No value yet" : value;
// ```
/** The line the field list shows for a field's value, naming a field that has none. */
internal fun templateEditorFieldValueText(value: String): String = if (value.isEmpty()) {
    "No value yet"
} else {
    value
}

// What:     `internal fun templateEditorParameterSpan(help: TemplateHelp): IntRange?` finds the argument name.
// Why:      The argument the caret is in is set bold inside the signature, so its place must be known.
//
// In TS you'd write (pseudocode):
// ```ts
// function parameterSpan(help) { const start = help.signature.indexOf(help.parameter); return ...; }
// ```
/** The character range of the argument name inside the signature, or null when the signature does not name it. */
internal fun templateEditorParameterSpan(help: TemplateHelp): IntRange? {
    /** Offset of the argument name inside the signature, or -1 when it is absent. */
    val start: Int = help.signature.indexOf(help.parameter)
    if (start < 0) {
        return null
    }
    return start until start + help.parameter.length
}

// What:     `@Composable fun templateEditor(...)` draws the whole editor page for one open template.
// Why:      The page stacks the preview, the template field, the field list and the reset, and it scrolls as one.
//
// In TS you'd write (pseudocode):
// ```ts
// function TemplateEditor(props: { state: State; actions: Actions; modifier?: Modifier }): UIElement;
// ```
/**
 * Draws the template editor for the open template: header, preview, template field, field list and reset,
 * on the player's canvas color.
 * [embedded] draws it as the right pane of the wide Settings page: its header has no Back target (Back closes
 * Settings) and matches the Settings header's height, and [startInset] moves every text start away from the
 * pane's left edge so text keeps clear of the crease.
 */
@Composable
fun templateEditor(
    state: TemplateEditorState,
    actions: TemplateEditorActions,
    modifier: Modifier = Modifier,
    embedded: Boolean = false,
    startInset: Dp = 0.dp,
) {
    Column(modifier = modifier.fillMaxSize().background(canvasColor(dark = isSystemInDarkTheme()))) {
        // What:     `Box` with `windowInsetsTopHeight` is an empty block as tall as the status bar.
        // Why:      The page draws edge to edge, so its header starts below the clock and icons.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <div style={{ height: 'env(safe-area-inset-top)' }}/>
        // ```
        Box(modifier = Modifier.fillMaxWidth().windowInsetsTopHeight(WindowInsets.statusBars))
        destinationHeader(
            title = state.kind.label,
            backDescription = "Back to Settings",
            onBack = if (embedded) null else actions.onBack,
            height = if (embedded) SETTINGS_HEADER_HEIGHT else HEADER_HEIGHT,
            startInset = startInset,
        )
        if (!embedded) {
            HorizontalDivider(thickness = DESTINATION_HAIRLINE, color = MaterialTheme.colorScheme.outline)
        }
        // What:     `imePadding()` before `verticalScroll` shrinks the scrolling window while the keyboard is open.
        // Why:      The platform then scrolls a focused field back into view, so the lines under it stay reachable.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <div style={{ flex: 1, marginBottom: keyboardHeight, overflowY: 'auto' }}>{body}</div>
        // ```
        Column(
            modifier = Modifier.weight(1f).fillMaxWidth().imePadding().verticalScroll(rememberScrollState())
                .windowInsetsPadding(WindowInsets.navigationBars),
        ) {
            destinationSectionHeading(text = "Preview", bottom = DESTINATION_HEADING_GAP, startInset = startInset)
            /** The preview rows the state yields from the last template that applied. */
            val rows: List<TemplatePreviewRow> = state.previewRows
            for (row in rows) {
                templateEditorPreviewRowView(row = row, startInset = startInset)
                HorizontalDivider(thickness = DESTINATION_HAIRLINE, color = MaterialTheme.colorScheme.outlineVariant)
            }
            for (note in state.previewNotes) {
                Text(
                    text = note,
                    modifier = Modifier.padding(
                        start = DESTINATION_SIDE_INSET + startInset,
                        end = DESTINATION_SIDE_INSET,
                        top = DESTINATION_SMALL_GAP,
                    ),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            templateEditorTemplateField(state = state, actions = actions, startInset = startInset)
            destinationSectionHeading(text = "Fields", bottom = DESTINATION_HEADING_GAP, startInset = startInset)
            /** The insertable fields of the open template with their values. */
            val entries: List<TemplateEditorFieldEntry> = state.fields
            for (entry in entries) {
                templateEditorFieldRow(
                    entry = entry,
                    startInset = startInset,
                    onInsert = { actions.onInsertField(entry.mode) },
                )
                HorizontalDivider(thickness = DESTINATION_HAIRLINE, color = MaterialTheme.colorScheme.outlineVariant)
            }
            if (state.canReset) {
                TextButton(
                    onClick = actions.onReset,
                    modifier = Modifier.padding(
                        start = DESTINATION_SIDE_INSET - BUTTON_LABEL_INSET + startInset,
                        top = DESTINATION_SMALL_GAP,
                        bottom = DESTINATION_SIDE_INSET,
                    ).defaultMinSize(minHeight = DESTINATION_TOUCH_TARGET).focusRing(CircleShape),
                ) {
                    Text(text = "Reset to default")
                }
            }
        }
    }
}

// What:     `@Composable internal fun destinationHeader(...)` draws the editor's header row.
// Why:      The editor page returns to Settings through its Back target, while the editor pane of the wide Settings
//           page draws the same row without one.
//
// In TS you'd write (pseudocode):
// ```ts
// function DestinationHeader(props: { title: string; backDescription: string; onBack(): void }): UIElement;
// ```
/**
 * Draws the header row on the canvas, with no band fill: a Back target when [onBack] is not null, then the
 * destination's title. [startInset] is the extra start inset that keeps the content clear of the crease.
 */
@Composable
internal fun destinationHeader(
    title: String,
    backDescription: String,
    onBack: (() -> Unit)?,
    height: Dp,
    startInset: Dp,
) {
    Row(
        modifier = Modifier.fillMaxWidth().height(height)
            .padding(start = DESTINATION_SIDE_INSET + startInset, end = DESTINATION_SIDE_INSET),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onBack != null) {
            IconButton(onClick = onBack, modifier = Modifier.size(DESTINATION_TOUCH_TARGET).focusRing(CircleShape)) {
                Icon(
                    imageVector = DESTINATION_BACK_ICON,
                    contentDescription = backDescription,
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
        Text(
            text = title,
            modifier = Modifier.weight(1f).padding(start = if (onBack != null) DESTINATION_SMALL_GAP else 0.dp)
                .semantics { heading() },
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.titleLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

// What:     `@Composable internal fun destinationSectionHeading(...)` draws one section heading.
// Why:      `Templates`, `Preview` and `Fields` share one style, marked as headings for assistive technology.
//
// In TS you'd write (pseudocode):
// ```ts
// function SectionHeading(props: { text: string; bottom: number }): UIElement;
// ```
/** Draws one section heading in the label role, marked as a heading. */
@Composable
internal fun destinationSectionHeading(text: String, bottom: Dp, startInset: Dp = 0.dp) {
    Text(
        text = text,
        modifier = Modifier.padding(
            start = DESTINATION_SIDE_INSET + startInset,
            end = DESTINATION_SIDE_INSET,
            top = DESTINATION_SECTION_GAP,
            bottom = bottom,
        )
            .semantics { heading() },
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        style = MaterialTheme.typography.labelLarge,
    )
}

// What:     `@Composable private fun templateEditorPreviewRowView(...)` draws one preview row like a track row.
// Why:      A preview row is a picture of a track row, not a control, so it has no click handler.
//
// In TS you'd write (pseudocode):
// ```ts
// function PreviewRow(props: { row: PreviewRow }): UIElement;
// ```
/** Draws one preview row: the title, and under it the supporting line when the template yields one. */
@Composable
private fun templateEditorPreviewRowView(row: TemplatePreviewRow, startInset: Dp) {
    Column(
        modifier = Modifier.fillMaxWidth().heightIn(min = ROW_MIN_HEIGHT)
            .padding(
                start = DESTINATION_SIDE_INSET + startInset,
                end = DESTINATION_SIDE_INSET,
                top = ROW_VERTICAL_PADDING,
                bottom = ROW_VERTICAL_PADDING,
            ),
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = row.title,
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodyLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (row.supporting.isNotEmpty()) {
            Text(
                text = row.supporting,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

// What:     `@Composable private fun templateEditorTemplateField(...)` draws the outlined template field.
// Why:      The field is where the user types, and its supporting slot shows mistakes or typing help.
//
// In TS you'd write (pseudocode):
// ```ts
// function TemplateField(props: { state: State; actions: Actions }): UIElement;
// ```
/** Draws the template field with its monospace text, its focus reporting, and its mistakes or help. */
@Composable
private fun templateEditorTemplateField(
    state: TemplateEditorState,
    actions: TemplateEditorActions,
    startInset: Dp,
) {
    /** The mistake lines of the field's current text, empty when it applies. */
    val mistakes: List<String> = state.mistakeLines
    OutlinedTextField(
        value = templateEditorTextValue(state),
        onValueChange = { value -> actions.onEdit(templateEditorEditOf(value)) },
        modifier = Modifier.fillMaxWidth()
            .padding(start = DESTINATION_SIDE_INSET + startInset, end = DESTINATION_SIDE_INSET)
            .onFocusChanged { focusState -> actions.onFocusChanged(focusState.isFocused) },
        textStyle = MaterialTheme.typography.bodyLarge.copy(fontFamily = FontFamily.Monospace),
        label = { Text(text = "Template") },
        supportingText = templateEditorSupporting(mistakes = mistakes, help = state.help),
        isError = mistakes.isNotEmpty(),
        keyboardOptions = KeyboardOptions(
            capitalization = KeyboardCapitalization.None,
            autoCorrectEnabled = false,
            keyboardType = KeyboardType.Ascii,
        ),
    )
}

// What:     `private fun templateEditorSupporting(...)` picks what the field draws under itself.
// Why:      Mistakes take the place of typing help, and help shows only while the template applies.
//
// In TS you'd write (pseudocode):
// ```ts
// function supporting(mistakes, help) { return mistakes.length ? Errors : help ? Help : undefined; }
// ```
/** The content under the template field: its mistake lines, else its typing help, else nothing. */
private fun templateEditorSupporting(mistakes: List<String>, help: TemplateHelp?): (@Composable () -> Unit)? {
    if (mistakes.isNotEmpty()) {
        return { templateEditorErrorLines(lines = mistakes) }
    }
    /** The typing help for the call around the caret, or null when there is none. */
    val current: TemplateHelp? = help
    if (current != null) {
        return { templateEditorHelpLines(help = current) }
    }
    return null
}

// What:     `@Composable private fun templateEditorErrorLines(...)` draws one row per mistake.
// Why:      Each mistake is marked by an icon, the error colour and its own words, not by colour alone.
//
// In TS you'd write (pseudocode):
// ```ts
// function ErrorLines(props: { lines: string[] }): UIElement;
// ```
/** Draws each mistake line with the error icon beside it. */
@Composable
private fun templateEditorErrorLines(lines: List<String>) {
    Column {
        for (line in lines) {
            Row {
                Icon(
                    imageVector = TEMPLATE_ERROR_ICON,
                    contentDescription = null,
                    modifier = Modifier.size(ERROR_ICON_SIZE),
                    tint = MaterialTheme.colorScheme.error,
                )
                Text(
                    text = line,
                    modifier = Modifier.padding(start = ICON_TEXT_GAP),
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

// What:     `@Composable private fun templateEditorHelpLines(...)` draws the signature and the argument's sentence.
// Why:      The argument the caret is in is set bold in the signature, so the weight shows it as well as the sentence.
//
// In TS you'd write (pseudocode):
// ```ts
// function HelpLines(props: { help: TemplateHelp }): UIElement;
// ```
/** Draws the call's signature with the current argument in bold, and that argument's sentence under it. */
@Composable
private fun templateEditorHelpLines(help: TemplateHelp) {
    /** The character range of the current argument inside the signature, or null when it is not named there. */
    val span: IntRange? = templateEditorParameterSpan(help)
    /** The signature text with the current argument set bold. */
    val signature = buildAnnotatedString {
        if (span == null) {
            append(help.signature)
        } else {
            append(help.signature.substring(0, span.first))
            withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
                append(help.parameter)
            }
            append(help.signature.substring(span.last + 1))
        }
    }
    Column {
        Text(
            text = signature,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontFamily = FontFamily.Monospace,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = help.description,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}

// What:     `@Composable private fun templateEditorFieldRow(...)` draws one insertable field as a list row.
// Why:      The whole row inserts the field, and it shows the field's value beside its name. The ring lies inside the
//           row's edge (D120) and adds no padding, so the row's text keeps the column of the section headings (D130).
//
// In TS you'd write (pseudocode):
// ```ts
// function FieldRow(props: { entry: FieldEntry; onInsert(): void }): UIElement;
// ```
/** Draws one field row: its name, its value for the first preview track, and the call a tap inserts. */
@Composable
private fun templateEditorFieldRow(entry: TemplateEditorFieldEntry, startInset: Dp, onInsert: () -> Unit) {
    ListItem(
        modifier = Modifier.fillMaxWidth()
            .padding(
                start = DESTINATION_SIDE_INSET - DESTINATION_ROW_INSET,
                end = DESTINATION_SIDE_INSET - DESTINATION_ROW_INSET,
            )
            .defaultMinSize(minHeight = DESTINATION_TOUCH_TARGET)
            .focusRing(RectangleShape, placement = FocusRingPlacement.INSIDE)
            .clickable(role = Role.Button, onClick = onInsert)
            // The crease moves the text only: the row's box, state layer and inside ring keep the pane edge (D130).
            .padding(start = startInset),
        supportingContent = {
            Text(
                text = templateEditorFieldValueText(entry.value),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        trailingContent = {
            Text(
                text = entry.insert,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontFamily = FontFamily.Monospace,
                style = MaterialTheme.typography.labelLarge,
            )
        },
        colors = ListItemDefaults.colors(containerColor = Color.Transparent),
    ) {
        Text(text = entry.label)
    }
}
