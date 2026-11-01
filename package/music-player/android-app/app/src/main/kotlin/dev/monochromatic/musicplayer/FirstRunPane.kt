// What:     `package dev.monochromatic.musicplayer` places the first-run pane beside the other
//           player composables of the app.
// Why:      The pane is stateless and screens import it without reaching into MainActivity.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `core` bring in the first-run states, copy, action labels, and analysis text.
// Why:      The pane draws the same strings and labels that the pure core tests check.
//
// In TS you'd write (pseudocode):
// ```ts
// import { firstRunCopy, FirstRunState } from "core/FirstRunState";
// ```
import androidx.compose.foundation.shape.CircleShape
import dev.monochromatic.musicplayer.core.ALLOW_ACCESS_LABEL
import dev.monochromatic.musicplayer.core.ANALYSIS_EXPLANATION
import dev.monochromatic.musicplayer.core.FirstRunCopy
import dev.monochromatic.musicplayer.core.FirstRunState
import dev.monochromatic.musicplayer.core.OPEN_FOLDER_LABEL
import dev.monochromatic.musicplayer.core.SETTINGS_LABEL
import dev.monochromatic.musicplayer.core.firstRunCopy

// What:     Imports from `androidx.compose.foundation` bring in layout, insets, and scrolling.
// Why:      The pane keeps large 200% text scrollable and clears the system bars.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Column, verticalScroll } from "compose/foundation";
// ```
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll

// What:     Imports from `androidx.compose.material3` bring in the filled and outlined buttons,
//           text, and the theme accessor.
// Why:      The pane uses Material 3 roles so it follows the app theme in light and dark.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Button, OutlinedButton, Text, MaterialTheme } from "material3";
// ```
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text

// What:     Imports from `androidx.compose.runtime` and `ui` bring in the composable marker,
//           the modifier, and the dp unit.
// Why:      The pane is a composable that takes a modifier and lays out Dp measurements.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Composable, Modifier, dp } from "compose";
// ```
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// What:     `private val MIN_TARGET: Dp = 48.dp` names the minimum touch target of every button.
// Why:      Each action keeps a 48dp minimum so it stays reachable at every text scale.
//
// In TS you'd write (pseudocode):
// ```ts
// const MIN_TARGET = 48;
// ```
/** Minimum width and height of every action button. */
private val MIN_TARGET: Dp = 48.dp

// What:     `private val EDGE_PADDING: Dp = 24.dp` names the outer inset of the scrolling column.
// Why:      The study keeps 24dp to the top, bottom, and right edges.
//
// In TS you'd write (pseudocode):
// ```ts
// const EDGE_PADDING = 24;
// ```
/** Outer inset on the top, bottom, and trailing edges of the pane. */
private val EDGE_PADDING: Dp = 24.dp

// What:     `private val SECTION_SPACING: Dp = 24.dp` names the gap between headline, body, actions, and analysis.
// Why:      The study spaces every section by 24dp so the copy reads as separate groups.
//
// In TS you'd write (pseudocode):
// ```ts
// const SECTION_SPACING = 24;
// ```
/** Vertical gap between the headline, body, actions, and analysis explanation. */
private val SECTION_SPACING: Dp = 24.dp

// What:     `private val WIDE_LAYOUT_MIN_WIDTH: Dp = 600.dp` names the width at which the blank left half appears.
// Why:      The study leaves the left half of the inner panel blank only on layouts at least this wide.
//
// In TS you'd write (pseudocode):
// ```ts
// const WIDE_LAYOUT_MIN_WIDTH = 600;
// ```
/** Available width at or above which the text starts past the centre of the pane. */
private val WIDE_LAYOUT_MIN_WIDTH: Dp = 600.dp

// What:     `private const val ANALYSIS_HEADING: String` names the heading above the analysis explanation.
// Why:      The heading is pane copy, while the explanation sentence lives in core.
//
// In TS you'd write (pseudocode):
// ```ts
// const ANALYSIS_HEADING = "About true-peak analysis";
// ```
/** Heading drawn above the true-peak explanation in the declined state. */
private const val ANALYSIS_HEADING: String = "About true-peak analysis"

// What:     `internal enum class FirstRunActionTarget` names the three callbacks a button can reach.
// Why:      Mapping labels to targets is a pure step the tests can check without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunActionTarget = "ALLOW_ACCESS" | "OPEN_FOLDER" | "SETTINGS";
// ```
/** Identifies which callback a first-run button invokes. */
internal enum class FirstRunActionTarget {
    // What:     `ALLOW_ACCESS` is the first entry of the enum.
    // Why:      The filled button in the declined state requests the Android music permission.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // ALLOW_ACCESS = "ALLOW_ACCESS",
    // ```
    /** Requests access to music on this device. */
    ALLOW_ACCESS,

    // What:     `OPEN_FOLDER` is the second entry of the enum.
    // Why:      Both no-audio states and the declined state offer a folder choice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // OPEN_FOLDER = "OPEN_FOLDER",
    // ```
    /** Opens the folder picker. */
    OPEN_FOLDER,

    // What:     `SETTINGS` is the third entry of the enum.
    // Why:      Settings opens the in-app Settings pane, not Android's permission page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // SETTINGS = "SETTINGS",
    // ```
    /** Opens the in-app Settings pane. */
    SETTINGS,
}

// What:     `data class FirstRunActions(...)` bundles the three callbacks the pane reports through.
// Why:      The pane never requests permission or opens a picker itself; the caller decides.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunActions = { onAllowAccess(): void; onOpenFolder(): void; onSettings(): void };
// ```
/** Callbacks invoked when the user chooses an action in the pane. */
data class FirstRunActions(
    // What:     `val onAllowAccess: () -> Unit` reports the Allow access button.
    // Why:      The caller owns the permission request and any Settings fallback after repeated refusal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onAllowAccess: () => void;
    // ```
    /** Requests access to music on this device. */
    val onAllowAccess: () -> Unit,
    // What:     `val onOpenFolder: () -> Unit` reports the Open a folder button.
    // Why:      The caller decides which folder picker opens and whether the source changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onOpenFolder: () => void;
    // ```
    /** Opens the folder picker. */
    val onOpenFolder: () -> Unit,
    // What:     `val onSettings: () -> Unit` reports the Settings button.
    // Why:      The caller decides what the in-app Settings pane shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // onSettings: () => void;
    // ```
    /** Opens the in-app Settings pane. */
    val onSettings: () -> Unit,
)

// What:     `internal fun firstRunStartPadding(maxWidth: Dp): Dp` computes the start inset of the text column.
// Why:      On wide layouts the text starts past the centre so the left half of the panel stays blank.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunStartPadding(maxWidth: number): number {
//   return maxWidth >= 600 ? maxWidth / 2 + 24 : 24;
// }
// ```
/**
 * Computes the start inset of the first-run text column for the available width.
 *
 * @param maxWidth available width of the pane
 * @return start inset in Dp
 */
internal fun firstRunStartPadding(maxWidth: Dp): Dp =
    if (maxWidth >= WIDE_LAYOUT_MIN_WIDTH) maxWidth / 2 + EDGE_PADDING else EDGE_PADDING

// What:     `internal fun firstRunActionTargetFor(label: String): FirstRunActionTarget` maps a label to its target.
// Why:      Labels come from the core copy, so an unknown label is a programming error and throws.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunActionTargetFor(label: string): FirstRunActionTarget {
//   switch (label) { ... default: throw new Error("Unknown first-run action label"); }
// }
// ```
/**
 * Maps one action label from the core copy to its callback target.
 *
 * @param label visible label of an action button
 * @return target whose callback the button invokes
 * @throws IllegalArgumentException when the label is not one of the first-run action labels
 */
internal fun firstRunActionTargetFor(label: String): FirstRunActionTarget = when (label) {
    ALLOW_ACCESS_LABEL -> FirstRunActionTarget.ALLOW_ACCESS
    OPEN_FOLDER_LABEL -> FirstRunActionTarget.OPEN_FOLDER
    SETTINGS_LABEL -> FirstRunActionTarget.SETTINGS
    else -> throw IllegalArgumentException("Unknown first-run action label: $label")
}

// What:     `private fun callbackFor(...)` returns the caller's callback for one target.
// Why:      The button reads one function value, so each target maps to exactly one callback.
//
// In TS you'd write (pseudocode):
// ```ts
// function callbackFor(target: FirstRunActionTarget, actions: FirstRunActions): () => void { ... }
// ```
/**
 * Returns the caller callback that a button target invokes.
 *
 * @param target button target chosen from the action label
 * @param actions callbacks supplied by the caller
 * @return the callback matching the target
 */
private fun callbackFor(target: FirstRunActionTarget, actions: FirstRunActions): () -> Unit = when (target) {
    FirstRunActionTarget.ALLOW_ACCESS -> actions.onAllowAccess
    FirstRunActionTarget.OPEN_FOLDER -> actions.onOpenFolder
    FirstRunActionTarget.SETTINGS -> actions.onSettings
}

// What:     `@Composable fun firstRunPane(...)` draws the first-run state from its copy.
// Why:      The pane shows the state's headline, body, actions, and analysis explanation, and reports taps only.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunPane(state, folderName, actions, modifier): UI { ... }
// ```
/**
 * Draws one first-run state as a scrollable column with its actions.
 *
 * The pane applies the system-bar insets itself, so the caller must not add them again.
 *
 * @param state first-run screen to draw
 * @param folderName name of the chosen folder, required for the folder no-audio state
 * @param actions callbacks invoked when the user chooses an action
 * @param modifier outer modifier applied before the insets
 */
@Composable
fun firstRunPane(
    state: FirstRunState,
    folderName: String?,
    actions: FirstRunActions,
    modifier: Modifier = Modifier,
) {
    /** Copy and action labels for the state, resolved once per composition. */
    val copy: FirstRunCopy = firstRunCopy(state = state, folderName = folderName)
    BoxWithConstraints(modifier = modifier.fillMaxSize().windowInsetsPadding(WindowInsets.systemBars)) {
        /** Start inset that keeps the text beside the blank left half on wide layouts. */
        val start: Dp = if (maxWidth >= WIDE_LAYOUT_MIN_WIDTH) {
            // On a creased screen the text also keeps the crease clearance of the right pane (E2, D110).
            firstRunStartPadding(maxWidth = maxWidth) - EDGE_PADDING + paneStartInset(EDGE_PADDING)
        } else {
            firstRunStartPadding(maxWidth = maxWidth)
        }
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(start = start, end = EDGE_PADDING)
                .verticalScroll(rememberScrollState())
                .padding(vertical = EDGE_PADDING),
            verticalArrangement = Arrangement.spacedBy(SECTION_SPACING),
        ) {
            Text(copy.title, style = MaterialTheme.typography.headlineSmall)
            Text(copy.body, style = MaterialTheme.typography.bodyLarge)
            /** Filled action label, or null when the state draws no filled action. */
            val primary: String? = copy.primaryAction
            if (primary != null) {
                /** Callback for the filled action. */
                val onPrimary: () -> Unit = callbackFor(firstRunActionTargetFor(primary), actions)
                Button(
                    onClick = onPrimary,
                    modifier = Modifier.defaultMinSize(minWidth = MIN_TARGET, minHeight = MIN_TARGET)
                        .focusRing(CircleShape),
                ) { Text(primary) }
            }
            copy.secondaryActions.forEach { label ->
                /** Callback for one outlined action. */
                val onSecondary: () -> Unit = callbackFor(firstRunActionTargetFor(label), actions)
                OutlinedButton(
                    onClick = onSecondary,
                    modifier = Modifier.defaultMinSize(minWidth = MIN_TARGET, minHeight = MIN_TARGET)
                        .focusRing(CircleShape),
                ) { Text(label) }
            }
            if (copy.explainsAnalysis) {
                Text(ANALYSIS_HEADING, style = MaterialTheme.typography.titleMedium)
                Text(ANALYSIS_EXPLANATION, style = MaterialTheme.typography.bodyLarge)
            }
        }
    }
}
