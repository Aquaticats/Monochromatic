// Keyboard focus indicator of the player (D117): a ring and nothing else, painted the way Firefox paints
// `:focus-visible { outline: 1px auto }` but hugging the element: a solid near ring of 2 dp in the primary role, flush
// against the element, and a solid outer ring of 1 dp in the on-primary role around it, which keeps the ring readable
// on any surface. Both follow the element's own outline, are drawn outside it and move nothing, show for keyboard
// focus only (CSS `:focus-visible`), and focus adds no fill, tint, state layer, bold or size change to the element. An
// entry that sits flush in a clipped list draws the same pair just inside its own edge instead (D120), because an
// outside ring would be cut off by the list. The geometry is in `core/FocusRingGeometry.kt`; the drawing takes the
// widths, the gap, the square-corner radius, the placement and both colors as parameters.

// What:     `package dev.monochromatic.musicplayer` places the indicator beside the theme that provides it.
// Why:      The theme and the list composables share this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in Compose drawing, focus observation, the input mode and the ripple configuration.
// Why:      The ring is a stroked rounded path drawn over the element, shown while focus is inside it and the last
//           input was the keyboard.
//
// In TS you'd write (pseudocode):
// ```ts
// import { drawWithContent, onFocusChanged, inputMode, RippleThemeConfiguration } from "compose";
// ```
import androidx.compose.material3.LocalRippleThemeConfiguration
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RippleThemeConfiguration
import androidx.compose.foundation.interaction.InteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import dev.monochromatic.musicplayer.core.FIREFOX_RING_SPEC
import dev.monochromatic.musicplayer.core.FocusRingBands
import dev.monochromatic.musicplayer.core.FocusRingPlacement
import dev.monochromatic.musicplayer.core.FocusRingSpec
import dev.monochromatic.musicplayer.core.RingCorners
import dev.monochromatic.musicplayer.core.RingRect
import dev.monochromatic.musicplayer.core.focusRingBandsAt
import dev.monochromatic.musicplayer.core.focusRingShows

// What:     `internal val FOCUS_RING_OUTSET: Dp` is how far the rings reach outside their element.
// Why:      A parent that clips its children (a menu surface, a toast) must leave this much room around an element
//           that draws the outside ring, or the ring would be cut off. Entries of a clipped list draw the ring
//           inside their own edge instead (D120) and leave no room.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOCUS_RING_OUTSET = "3dp";
// ```
/** Distance from an element's edge to the outer edge of its outside focus ring: the room a clipping parent leaves. */
internal val FOCUS_RING_OUTSET: Dp = FIREFOX_RING_SPEC.outset.dp

/** The geometry and the two colors of one focus ring, given to the drawing modifier as parameters. */
internal class FocusRingStyle(
    /** Widths, gap and square-corner radius. */
    val spec: FocusRingSpec,
    /** Color of the near ring. */
    val nearColor: Color,
    /** Color of the outer ring. */
    val outerColor: Color,
)

/** Returns the Firefox ring style, with its two colors taken from the current theme: primary, then on-primary. */
@Composable
private fun currentFocusRingStyle(): FocusRingStyle = FocusRingStyle(
    spec = FIREFOX_RING_SPEC,
    nearColor = MaterialTheme.colorScheme.primary,
    outerColor = MaterialTheme.colorScheme.onPrimary,
)

// What:     `internal fun Modifier.keyboardFocusRing(visible, shape, placement)` draws the ring for a focused list.
// Why:      An entry of a one-stop list (a row, a rail cell, a folder name) cannot be focused by itself, so the list
//           marks the entry under its own ring with the same outline a focused button shows, and only while the last
//           input was the keyboard, like every other ring.
//
// In TS you'd write (pseudocode):
// ```ts
// const keyboardFocusRing = (visible: boolean, shape: Shape, placement = "outside") =>
//   (el) => el.outlineWhen(visible && keyboard, shape, placement);
// ```
/**
 * Draws the focus ring on the entry of a focused list when [visible] is true and the last input was the keyboard.
 *
 * @param visible whether the entry holds the ring of a list that has focus.
 * @param shape the entry's own shape, which the ring hugs.
 * @param placement outside the entry (D117), or just inside its edge for an entry flush in a clipped list (D120).
 */
@Composable
internal fun Modifier.keyboardFocusRing(
    visible: Boolean,
    shape: Shape,
    placement: FocusRingPlacement = FocusRingPlacement.OUTSIDE,
): Modifier = ringWhen(focused = { visible }, shape = shape, placement = placement)

// What:     `internal fun Modifier.focusRing(shape, placement)` draws the ring while focus is inside the element.
// Why:      Buttons, icon buttons, text targets and menu rows take focus themselves; observing the focus below this
//           modifier in the chain needs no interaction source and is not clipped by the Material surface inside.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRing = (shape: Shape, placement = "outside") =>
//   (el) => el.onFocus((f) => (focused = f)).outlineWhen(focused && keyboard, shape, placement);
// ```
/**
 * Draws the focus ring around the element while it or a descendant has focus and the last input was the keyboard.
 *
 * Put it after the element's size and before any padding, so the ring hugs the element's own bounds.
 *
 * @param shape the element's own shape, which the ring hugs.
 * @param placement outside the element (D117), or just inside its edge for an entry flush in a clipped list (D120).
 */
@Composable
internal fun Modifier.focusRing(shape: Shape, placement: FocusRingPlacement = FocusRingPlacement.OUTSIDE): Modifier {
    /** Whether focus is inside the element. */
    var focused: Boolean by remember { mutableStateOf(false) }
    return this
        .onFocusChanged { state -> focused = state.hasFocus }
        .ringWhen(focused = { focused }, shape = shape, placement = placement)
}

// What:     `internal fun Modifier.focusRingOf(interactionSource, shape)` draws the ring while a source reports focus.
// Why:      A part that is not the focus target itself, such as the thumb of a slider, learns about focus only from
//           the interaction source it shares with the control.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRingOf = (source: Source, shape: Shape) => (el) => el.outlineWhen(source.focused && keyboard, shape);
// ```
/**
 * Draws the focus ring while [interactionSource] reports focus and the last input was the keyboard.
 *
 * @param interactionSource the source that the focusable control emits focus interactions to.
 * @param shape the element's own shape, which the ring hugs.
 */
@Composable
internal fun Modifier.focusRingOf(interactionSource: InteractionSource, shape: Shape): Modifier {
    /** Whether the source reports focus. */
    val focused: Boolean by interactionSource.collectIsFocusedAsState()
    return ringWhen(focused = { focused }, shape = shape, placement = FocusRingPlacement.OUTSIDE)
}

/**
 * Raises the element above its siblings while it is ringed, so a neighbor drawn later cannot cover the ring, and draws
 * the ring after the element's own content, outside it, with no clip.
 */
@Composable
private fun Modifier.ringWhen(focused: () -> Boolean, shape: Shape, placement: FocusRingPlacement): Modifier {
    /** Input mode, read when drawing so a switch from touch to keyboard repaints the ring. */
    val inputMode = LocalInputModeManager.current
    /** Widths and colors of the ring. */
    val style: FocusRingStyle = currentFocusRingStyle()
    /** Whether the ring shows now: focus is there and the last input was the keyboard (D117), for every element. */
    val shows: () -> Boolean = {
        focusRingShows(focused = focused(), lastInputWasKeyboard = inputMode.inputMode == InputMode.Keyboard)
    }
    return focusRingDrawn(shows = shows, shape = shape, style = style, placement = placement)
}

// What:     `internal fun Modifier.focusRingDrawn(shows, shape, style, placement)` is the one drawing routine.
// Why:      Every kind of element goes through it, with the widths, the gap, the square-corner radius and the two
//           colors as parameters (D117) and the placement, outside or inside the edge (D120), so none is fixed
//           inside the drawing.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRingDrawn = (shows: () => boolean, shape: Shape, style: Style, placement: Placement) =>
//   (el) => el.over(ringsOf(shape, style, placement));
// ```
/**
 * Draws the two solid rings of [style] while [shows] is true, over the element, outside it for
 * [FocusRingPlacement.OUTSIDE] and just inside its edge for [FocusRingPlacement.INSIDE].
 *
 * @param shows read when drawing, so a state change repaints without recomposing.
 * @param shape the element's own shape, which the rings hug.
 * @param style the widths, the gap, the square-corner radius and the two ring colors.
 * @param placement which side of the element's edge the rings are on.
 */
internal fun Modifier.focusRingDrawn(
    shows: () -> Boolean,
    shape: Shape,
    style: FocusRingStyle,
    placement: FocusRingPlacement = FocusRingPlacement.OUTSIDE,
): Modifier =
    this
        .zIndex(if (shows()) 1f else 0f)
        .drawWithContent {
            drawContent()
            if (shows()) {
                drawFocusRings(style = style, shape = shape, placement = placement)
            }
        }

// What:     `private fun DrawScope.drawFocusRings(style, shape, placement)` paints both rings on the current element.
// Why:      The element's outline gives the corner radii; the pure geometry grows them into three outlines, and each
//           ring is the area between two of them, filled with the even-odd rule so the inner radius is the element's.
//
// In TS you'd write (pseudocode):
// ```ts
// function drawFocusRings(ctx, style, shape) { const b = focusRingBands(size, cornersOf(shape), style.spec); ... }
// ```
/** Fills the near ring and the outer ring of [style] on the element being drawn, at [placement]. */
private fun DrawScope.drawFocusRings(style: FocusRingStyle, shape: Shape, placement: FocusRingPlacement) {
    /** The element's own corner radii in pixels. */
    val corners: RingCorners = cornersOf(shape.createOutline(size, layoutDirection, this), size)
    /** Spec converted from dp to pixels. */
    val spec = FocusRingSpec(
        nearWidth = style.spec.nearWidth.dp.toPx(),
        outerWidth = style.spec.outerWidth.dp.toPx(),
        gap = style.spec.gap.dp.toPx(),
        squareCornerRadius = style.spec.squareCornerRadius.dp.toPx(),
    )
    /** The three outlines that bound the two rings. */
    val bands: FocusRingBands = focusRingBandsAt(
        placement = placement,
        width = size.width,
        height = size.height,
        corners = corners,
        spec = spec,
    )
    /** Seam overlap, needed only where the near ring meets the element's anti-aliased edge, which is outside only. */
    val overlap: Float = if (placement == FocusRingPlacement.OUTSIDE) RING_SEAM_OVERLAP_PX else 0f
    drawPath(path = ringBetween(bands.middle, bands.inner, overlap = overlap), color = style.nearColor)
    drawPath(path = ringBetween(bands.outer, bands.middle), color = style.outerColor)
}

/**
 * Pixels by which the near ring reaches under the element's own edge. Two shapes that meet on an anti-aliased edge each
 * cover part of the edge pixels, so a ring exactly flush with the element leaves a faint seam of the background; half a
 * pixel of overlap closes it and changes no pixel more than 2 px inside the element.
 */
private const val RING_SEAM_OVERLAP_PX: Float = 0.5f

/** Returns the area between [outer] and [inner] moved [overlap] pixels inward, which is one solid ring. */
private fun ringBetween(outer: RingRect, inner: RingRect, overlap: Float = 0f): Path = Path().apply {
    fillType = PathFillType.EvenOdd
    addRoundRect(roundRectOf(outer))
    addRoundRect(roundRectOf(inner.deflatedBy(overlap)))
}

/** Returns this outline moved [amount] pixels inward on every side, with each rounded corner shrunk by the same. */
private fun RingRect.deflatedBy(amount: Float): RingRect = RingRect(
    left = left + amount,
    top = top + amount,
    right = right - amount,
    bottom = bottom - amount,
    corners = RingCorners(
        topLeft = (corners.topLeft - amount).coerceAtLeast(0f),
        topRight = (corners.topRight - amount).coerceAtLeast(0f),
        bottomRight = (corners.bottomRight - amount).coerceAtLeast(0f),
        bottomLeft = (corners.bottomLeft - amount).coerceAtLeast(0f),
    ),
)

/** Converts one outline of the geometry to a Compose rounded rectangle. */
private fun roundRectOf(rect: RingRect): RoundRect = RoundRect(
    left = rect.left,
    top = rect.top,
    right = rect.right,
    bottom = rect.bottom,
    topLeftCornerRadius = CornerRadius(rect.corners.topLeft),
    topRightCornerRadius = CornerRadius(rect.corners.topRight),
    bottomRightCornerRadius = CornerRadius(rect.corners.bottomRight),
    bottomLeftCornerRadius = CornerRadius(rect.corners.bottomLeft),
)

/** Reads the corner radii of an outline in pixels; a rectangle or a free-form outline counts as square. */
private fun cornersOf(outline: Outline, size: Size): RingCorners = when (outline) {
    is Outline.Rounded -> RingCorners(
        topLeft = minOf(outline.roundRect.topLeftCornerRadius.x, size.minDimension / 2f),
        topRight = minOf(outline.roundRect.topRightCornerRadius.x, size.minDimension / 2f),
        bottomRight = minOf(outline.roundRect.bottomRightCornerRadius.x, size.minDimension / 2f),
        bottomLeft = minOf(outline.roundRect.bottomLeftCornerRadius.x, size.minDimension / 2f),
    )
    else -> RingCorners.uniform(0f)
}

// What:     `@Composable internal fun focusRingTheme(content)` removes the Material focus appearance.
// Why:      Material 3 draws a ten percent fill on a focused control and, configured otherwise, a ring inside its
//           bounds. D117 allows neither, so the ripple's focus ring is given no width and the app draws its own ring.
//
// In TS you'd write (pseudocode):
// ```ts
// const FocusRingTheme = ({ children }) => <RippleConfig focus={noRing}>{children}</RippleConfig>;
// ```
/** Provides [content] with a Material ripple configuration that draws neither a focus fill nor a focus ring. */
@Composable
internal fun focusRingTheme(content: @Composable () -> Unit) {
    CompositionLocalProvider(
        LocalRippleThemeConfiguration provides RippleThemeConfiguration(
            focus = RippleThemeConfiguration.Focus.InsetRing(
                outerStrokeInset = 0.dp,
                outerStrokeWidth = 0.dp,
                innerStrokeInset = 0.dp,
                innerStrokeWidth = 0.dp,
            ),
        ),
        content = content,
    )
}
