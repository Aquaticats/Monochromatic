// What:     `package dev.monochromatic.musicplayer` places the feedback overlay beside the other player composables.
// Why:      The overlay is a stateless composable that the player places inside its own overlay container.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `androidx.compose.foundation` bring in the stacking column, row, and sizing helpers.
// Why:      The overlay stacks messages at their content width and sizes the dismiss target to 48dp.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Column, Row, size } from "compose/foundation/layout";
// ```
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.draw.shadow
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape

// What:     Imports from `androidx.compose.material3` bring in the surface, text, icon, and button controls.
// Why:      The overlay uses Material 3 roles so each message follows the app theme.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Surface, Text, Icon, IconButton, TextButton, MaterialTheme } from "material3";
// ```
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton

// What:     Imports from `androidx.compose.runtime` bring in the composable marker and the keyed scope.
// Why:      Each message is keyed by its id so Compose keeps its state when the list changes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Composable, key } from "compose/runtime";
// ```
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key

// What:     Imports from `androidx.compose.ui` bring in alignment, modifiers, colors, and semantics.
// Why:      The overlay aligns text with icons and publishes each message as a polite live region.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Alignment, Modifier, Color } from "compose/ui";
// ```
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     Imports from `dev.monochromatic.musicplayer.core` bring in the message model and its kinds.
// Why:      The overlay draws the messages that the pure queue produced.
//
// In TS you'd write (pseudocode):
// ```ts
// import { FeedbackMessage, FeedbackKind } from "core/FeedbackQueue";
// ```
import dev.monochromatic.musicplayer.core.FeedbackKind
import dev.monochromatic.musicplayer.core.FeedbackMessage

// What:     `private val FEEDBACK_OWNER_GAP: Dp = 16.dp` names the gap between stacked messages.
// Why:      D83 and D29 require 16dp between feedback owners, and the gap is only drawn between messages.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_OWNER_GAP = 16;
// ```
/** Vertical gap between stacked messages. */
private val FEEDBACK_OWNER_GAP: Dp = 16.dp

// What:     `private val FEEDBACK_CORNER_RADIUS: Dp = 24.dp` rounds each message surface.
// Why:      The accepted study uses the same 24dp radius for every floating message.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_CORNER_RADIUS = 24;
// ```
/** Corner radius of each message surface. */
private val FEEDBACK_CORNER_RADIUS: Dp = 24.dp

// What:     `private val FEEDBACK_SHADOW_ELEVATION: Dp = 3.dp` lifts each message from the content beneath.
// Why:      The accepted study uses a 3dp shadow so the floating message reads as separate from the list.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_SHADOW_ELEVATION = 3;
// ```
/** Shadow elevation that separates each message from the content beneath it. */
private val FEEDBACK_SHADOW_ELEVATION: Dp = 3.dp

// What:     `private val FEEDBACK_START_INSET: Dp = 16.dp` insets the leading icon or text.
// Why:      The accepted study keeps 16dp between the surface edge and its first element.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_START_INSET = 16;
// ```
/** Inset between the leading surface edge and the first element. */
private val FEEDBACK_START_INSET: Dp = 16.dp

// What:     `private val FEEDBACK_END_INSET: Dp = 8.dp` insets the trailing dismiss target.
// Why:      The dismiss target sits close to the trailing edge, as in the accepted study.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_END_INSET = 8;
// ```
/** Inset between the trailing surface edge and the dismiss target. */
private val FEEDBACK_END_INSET: Dp = 8.dp

// What:     `private val FEEDBACK_ICON_GAP: Dp = 12.dp` separates the warning glyph from the text.
// Why:      The accepted study spaces the warning glyph from the message text by 12dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_ICON_GAP = 12;
// ```
/** Gap between the warning glyph and the message text. */
private val FEEDBACK_ICON_GAP: Dp = 12.dp

// What:     `private val FEEDBACK_ICON_SIZE: Dp = 24.dp` sets the edge length of the warning glyph.
// Why:      The glyph matches the 24dp grid of the feedback icon set.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_ICON_SIZE = 24;
// ```
/** Edge length of the warning glyph. */
private val FEEDBACK_ICON_SIZE: Dp = 24.dp

// What:     `private val FEEDBACK_TOUCH_TARGET: Dp = 48.dp` is the minimum size of every tappable control.
// Why:      D83 and the accessibility floor require a 48dp dismiss target and action target.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_TOUCH_TARGET = 48;
// ```
/** Minimum width and height of the dismiss control and the action control. */
private val FEEDBACK_TOUCH_TARGET: Dp = 48.dp

// What:     `private val FEEDBACK_TEXT_VERTICAL_INSET: Dp = 8.dp` pads the message text above and below.
// Why:      Two lines of body text need this vertical room to stay clear of the surface edges.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_TEXT_VERTICAL_INSET = 8;
// ```
/** Vertical padding above and below the message text. */
private val FEEDBACK_TEXT_VERTICAL_INSET: Dp = 8.dp

// What:     `internal const val FEEDBACK_TEXT_MAX_LINES: Int = 2` is the visible line limit of message text.
// Why:      D83 asks messages to fit two visible lines, and longer detail goes to the Android logs.
//
// In TS you'd write (pseudocode):
// ```ts
// const FEEDBACK_TEXT_MAX_LINES = 2;
// ```
/** Maximum visible lines of message text before the text ends with an ellipsis. */
internal const val FEEDBACK_TEXT_MAX_LINES: Int = 2

// What:     `internal fun feedbackDismissDescription(kind: FeedbackKind): String` names the dismiss control for a kind.
// Why:      Screen readers hear which message the dismiss control closes, so the two kinds read differently.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackDismissDescription(kind: FeedbackKind): string { ... }
// ```
/** Returns the accessibility label of the dismiss control for [kind]. */
internal fun feedbackDismissDescription(kind: FeedbackKind): String = when (kind) {
    FeedbackKind.ERROR -> "Dismiss message"
    FeedbackKind.UNDO -> "Dismiss Undo message"
    FeedbackKind.NOTICE -> "Dismiss notice"
}

// What:     `internal fun feedbackExceedsTextLimit(lineCount, hasVisualOverflow): Boolean` applies the D83 limit.
// Why:      A caller that measures message text uses this rule to decide whether to direct the user to Android logs.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackExceedsTextLimit(lineCount: number, hasVisualOverflow: boolean): boolean {
//   if (lineCount < 1) throw new Error("Measured feedback must have at least one line.");
//   return lineCount > 2 || hasVisualOverflow;
// }
// ```
/**
 * Reports whether measured message text needs more room than the two-line limit allows.
 * Throws [IllegalArgumentException] when [lineCount] is below one, because an empty layout is not measured text.
 */
internal fun feedbackExceedsTextLimit(lineCount: Int, hasVisualOverflow: Boolean): Boolean {
    // What:     `require(lineCount >= 1)` rejects a layout that reports no lines.
    // Why:      A zero line count means the text was not measured, so the rule cannot give an answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (lineCount < 1) throw new Error("Measured feedback must have at least one line.");
    // ```
    require(lineCount >= 1) { "Measured feedback must have at least one line, was $lineCount." }
    return lineCount > FEEDBACK_TEXT_MAX_LINES || hasVisualOverflow
}

// What:     `fun feedbackOverlay(...)` draws the visible messages as floating content-width surfaces.
// Why:      The caller places this in an overlay container, so the content beneath never changes size.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackOverlay(input: {
//   messages: FeedbackMessage[]; onDismiss: (id: string) => void; onAction: (id: string) => void;
// }): UIElement;
// ```
/**
 * Draws [messages] top to bottom, each at its content width and left aligned.
 * The caller bounds the width through [modifier] (for example to the track viewport minus the edge insets),
 * and the overlay never reserves space in the content beneath it.
 * [onAction] receives the message id; the caller decides whether that action also dismisses the message.
 */
@Composable
fun feedbackOverlay(
    messages: List<FeedbackMessage>,
    onDismiss: (String) -> Unit,
    onAction: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    // What:     `Column(modifier = modifier, ...)` stacks the messages with the 16dp owner gap.
    // Why:      Column without a fill modifier wraps its widest child, which gives content width and left alignment.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // column({ gap: 16, align: "start" }, messages.map(renderMessage));
    // ```
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(FEEDBACK_OWNER_GAP)) {
        messages.forEach { message ->
            key(message.id) {
                feedbackMessageSurface(message = message, onDismiss = onDismiss, onAction = onAction)
            }
        }
    }
}

// What:     `private fun feedbackMessageSurface(...)` draws one message with its glyph, text, action, and dismiss.
// Why:      Each message keeps its own surface so its live region and touch targets stay independent.
//
// In TS you'd write (pseudocode):
// ```ts
// function feedbackMessageSurface(
//   message: FeedbackMessage, onDismiss: (id: string) => void, onAction: (id: string) => void,
// ): UIElement;
// ```
/** Draws one message surface at its content width, announced politely when it appears. */
@Composable
private fun feedbackMessageSurface(
    message: FeedbackMessage,
    onDismiss: (String) -> Unit,
    onAction: (String) -> Unit,
) {
    // What:     `val container` selects the surface color for the message kind.
    // Why:      Error state uses the error container and Undo uses the neutral high surface,
    //           so the two kinds differ by color.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const container = message.kind === "ERROR" ? errorContainer : surfaceContainerHigh;
    // ```
    /** Surface color that marks the message kind. */
    val container = when (message.kind) {
        FeedbackKind.ERROR -> MaterialTheme.colorScheme.errorContainer
        FeedbackKind.UNDO -> MaterialTheme.colorScheme.surfaceContainerHigh
        FeedbackKind.NOTICE -> MaterialTheme.colorScheme.surfaceContainerHigh
    }
    // What:     `val content` selects the text and glyph color that reads on the container.
    // Why:      Each content color is paired with its container from the same Material role family.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const content = message.kind === "ERROR" ? onErrorContainer : onSurface;
    // ```
    /** Text and glyph color that reads on the message container. */
    val content = when (message.kind) {
        FeedbackKind.ERROR -> MaterialTheme.colorScheme.onErrorContainer
        FeedbackKind.UNDO -> MaterialTheme.colorScheme.onSurface
        FeedbackKind.NOTICE -> MaterialTheme.colorScheme.onSurface
    }
    /** Shape of the message container. */
    val shape = RoundedCornerShape(FEEDBACK_CORNER_RADIUS)
    // A Surface would clip its children to the shape and cut off the focus ring of its buttons (D117), so the
    // container is a shadow and a fill; every child draws its own content color.
    Box(
        modifier = Modifier
            .semantics { liveRegion = LiveRegionMode.Polite }
            .shadow(elevation = FEEDBACK_SHADOW_ELEVATION, shape = shape)
            .background(color = container, shape = shape),
    ) {
        // What:     `Row` places the optional glyph, text, optional action, and dismiss control in one line.
        // Why:      Controls share the surface's vertical center, and the text takes only the width it needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // row({ align: "center", padding: { start: 16, end: 8 } }, [glyph?, text, action?, dismiss]);
        // ```
        Row(
            modifier = Modifier.padding(start = FEEDBACK_START_INSET, end = FEEDBACK_END_INSET),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (message.kind == FeedbackKind.ERROR) {
                Icon(
                    imageVector = FEEDBACK_WARNING_ICON,
                    contentDescription = null,
                    modifier = Modifier.size(FEEDBACK_ICON_SIZE).padding(end = FEEDBACK_ICON_GAP),
                    tint = content,
                )
            }
            Text(
                text = message.text,
                color = content,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = FEEDBACK_TEXT_MAX_LINES,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f, fill = false).padding(vertical = FEEDBACK_TEXT_VERTICAL_INSET),
            )
            // What:     `message.actionLabel?.let` draws the action only when the message carries a label.
            // Why:      The Undo message offers its action through this label, and a null label means no action.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (message.actionLabel !== null) renderAction(message.actionLabel);
            // ```
            message.actionLabel?.let { label ->
                TextButton(
                    onClick = { onAction(message.id) },
                    colors = ButtonDefaults.textButtonColors(contentColor = content),
                    modifier = Modifier.defaultMinSize(
                        minWidth = FEEDBACK_TOUCH_TARGET,
                        minHeight = FEEDBACK_TOUCH_TARGET,
                    ).focusRing(CircleShape),
                ) {
                    Text(text = label, fontWeight = FontWeight.Bold)
                }
            }
            IconButton(
                onClick = { onDismiss(message.id) },
                modifier = Modifier.size(FEEDBACK_TOUCH_TARGET).focusRing(CircleShape),
            ) {
                Icon(
                    imageVector = FEEDBACK_CLOSE_ICON,
                    contentDescription = feedbackDismissDescription(message.kind),
                    tint = content,
                )
            }
        }
    }
}
