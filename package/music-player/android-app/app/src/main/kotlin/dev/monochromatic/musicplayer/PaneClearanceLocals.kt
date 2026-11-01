// Composition locals that carry the crease clearance of the pane a composable sits in (E2, D110). The unfolded player
// provides the clearance of each pane once, and every text row, header and deck reads it, so no composable needs a new
// parameter and the two panes cannot disagree.

// What:     `package dev.monochromatic.musicplayer` places the locals beside the composables that read them.
// Why:      The rows, headers, rail and deck of the player share this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in the composition local types and the unit types.
// Why:      The locals hold Dp values and are read in composition.
//
// In TS you'd write (pseudocode):
// ```ts
// import { createContext, useContext } from "react";
// ```
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.monochromatic.musicplayer.core.HostKeyLayout
import dev.monochromatic.musicplayer.core.PaneClearance
import dev.monochromatic.musicplayer.core.leftPaneClearance
import dev.monochromatic.musicplayer.core.paneTextInsetDp
import dev.monochromatic.musicplayer.core.rightPaneClearance

// What:     `internal val LocalPaneStartClearance: ProvidableCompositionLocal<Dp>` is the clearance at a pane's start.
// Why:      Text in the right pane starts at least this far from the pane's edge, and zero means no crease is there.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalPaneStartClearance = createContext(0);
// ```
/** Distance that text keeps clear at the start edge of the pane it is drawn in. */
internal val LocalPaneStartClearance: ProvidableCompositionLocal<Dp> = compositionLocalOf { 0.dp }

// What:     `internal val LocalPaneEndClearance: ProvidableCompositionLocal<Dp>` is the clearance at a pane's end.
// Why:      Text in the left pane ends at least this far from the pane's edge, and zero means no crease is there.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalPaneEndClearance = createContext(0);
// ```
/** Distance that text keeps clear at the end edge of the pane it is drawn in. */
internal val LocalPaneEndClearance: ProvidableCompositionLocal<Dp> = compositionLocalOf { 0.dp }

// What:     `@Composable internal fun paneClearanceProvider(clearance, content)` gives a pane its clearance.
// Why:      One call per pane provides both edges, so a pane never provides only one of them by accident.
//
// In TS you'd write (pseudocode):
// ```ts
// const PaneClearanceProvider = ({ clearance, children }) => <Ctx value={clearance}>{children}</Ctx>;
// ```
/** Provides [content] with the start and end clearance of one pane. */
@Composable
internal fun paneClearanceProvider(clearance: PaneClearance, content: @Composable () -> Unit) {
    CompositionLocalProvider(
        LocalPaneStartClearance provides clearance.startDp.dp,
        LocalPaneEndClearance provides clearance.endDp.dp,
        content = content,
    )
}

// What:     `@Composable internal fun paneStartInset(ordinary)` is the start inset of a text in the current pane.
// Why:      The ordinary inset applies unless the crease needs more, and the larger value wins.
//
// In TS you'd write (pseudocode):
// ```ts
// const paneStartInset = (ordinary: number) => Math.max(ordinary, useContext(StartClearance));
// ```
/** Returns the larger of [ordinary] and the clearance at the start edge of the current pane. */
@Composable
internal fun paneStartInset(ordinary: Dp): Dp =
    paneTextInsetDp(ordinaryDp = ordinary.value, clearanceDp = LocalPaneStartClearance.current.value).dp

// What:     `@Composable internal fun paneEndInset(ordinary)` is the end inset of a text in the current pane.
// Why:      The ordinary inset applies unless the crease needs more, and the larger value wins.
//
// In TS you'd write (pseudocode):
// ```ts
// const paneEndInset = (ordinary: number) => Math.max(ordinary, useContext(EndClearance));
// ```
/** Returns the larger of [ordinary] and the clearance at the end edge of the current pane. */
@Composable
internal fun paneEndInset(ordinary: Dp): Dp =
    paneTextInsetDp(ordinaryDp = ordinary.value, clearanceDp = LocalPaneEndClearance.current.value).dp

// What:     `@Composable internal fun Modifier.feedbackOverlayWidth(...)` sizes the message overlay.
// Why:      The overlay floats along the bottom edge across the whole window, so on the unfolded screen with a crease
//           its text would cross the fold. It spans the left pane and ends at that pane's clearance instead; on the
//           cover and on a screen without a crease it keeps the full width.
//
// In TS you'd write (pseudocode):
// ```ts
// const feedbackOverlayWidth = (layout, creaseWidthPx) => unfoldedWithCrease ? leftPaneOnly : fullWidth;
// ```
/** Returns this modifier sized to the full width, or to the left pane before the crease when there is one. */
@Composable
internal fun Modifier.feedbackOverlayWidth(keyLayout: HostKeyLayout, creaseWidthPx: Float?): Modifier {
    if (keyLayout != HostKeyLayout.UNFOLDED || creaseWidthPx == null) {
        return this.fillMaxWidth()
    }
    /** Clearance of the left pane at the current density, which the overlay keeps free at its end. */
    val clearance: PaneClearance = leftPaneClearance(
        creaseWidthPx = creaseWidthPx,
        density = LocalDensity.current.density,
    )
    return this.fillMaxWidth(LEFT_PANE_SHARE).padding(end = clearance.endDp.dp)
}

// What:     `private const val LEFT_PANE_SHARE: Float = 0.5f` is the share of the window the left pane takes.
// Why:      D110 splits a creased window into exact halves.
//
// In TS you'd write (pseudocode):
// ```ts
// const LEFT_PANE_SHARE = 0.5;
// ```
/** Share of the window width that the left pane takes on a creased unfolded screen. */
private const val LEFT_PANE_SHARE: Float = 0.5f

// What:     `@Composable internal fun rightPaneOnCreasedScreen(...)` draws a notice in the right pane only.
// Why:      A notice centered over the whole window would sit on the crease, so on a creased unfolded screen it is
//           centered in the right pane instead, with that pane's start clearance, and elsewhere it keeps the window.
//
// In TS you'd write (pseudocode):
// ```ts
// const RightPaneOnCreasedScreen = ({ layout, crease, children }) =>
//   unfoldedWithCrease ? <Row><Gap/><Pane/></Row> : children;
// ```
/** Draws [content] over the right half of a creased unfolded screen, or over the whole window otherwise. */
@Composable
internal fun rightPaneOnCreasedScreen(
    keyLayout: HostKeyLayout,
    creaseWidthPx: Float?,
    content: @Composable () -> Unit,
) {
    if (keyLayout != HostKeyLayout.UNFOLDED || creaseWidthPx == null) {
        content()
        return
    }
    /** Clearance of the right pane at the current density. */
    val clearance: PaneClearance = rightPaneClearance(
        creaseWidthPx = creaseWidthPx,
        density = LocalDensity.current.density,
    )
    Row(modifier = Modifier.fillMaxSize()) {
        Spacer(modifier = Modifier.weight(1f))
        Box(modifier = Modifier.weight(1f).fillMaxHeight().padding(start = clearance.startDp.dp)) {
            content()
        }
    }
}
