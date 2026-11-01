// The geometry of the keyboard focus ring (D117). The ring is the pair of solid rings that Firefox paints for
// `:focus-visible { outline: 1px auto }`, hugging the element instead of its box: a near ring of 2 dp flush against the
// element and an outer ring of 1 dp around it, each following the element's own outline. An entry that sits flush in a
// clipped list draws the same pair just inside its own edge instead (D120). The numbers live here, without Compose
// types, as a spec with the widths, the gap and the square-corner radius as parameters, so the JVM tests can state the
// rule exactly and the drawing code only converts the result to pixels. Whether the ring shows at all is decided here
// too: only for keyboard input.

// What:     `package dev.monochromatic.musicplayer.core` places the geometry beside the other pure rules.
// Why:      The geometry has no Android or Compose types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `data class FocusRingSpec(...)` holds every number that shapes a focus ring.
// Why:      The drawing routine takes the numbers as parameters, so no width or radius is fixed inside it and the tests
//           can try a gap or other widths.
//
// In TS you'd write (pseudocode):
// ```ts
// type FocusRingSpec = { nearWidth: number; outerWidth: number; gap: number; squareCornerRadius: number };
// ```
/** The widths, gap and square-corner radius of a focus ring, in dp. */
data class FocusRingSpec(
    /** Width of the near ring, the one next to the element. */
    val nearWidth: Float,
    /** Width of the outer ring around the near ring. */
    val outerWidth: Float,
    /** Gap between the element and the near ring; zero is flush. */
    val gap: Float,
    /** Outer radius that the near ring gets at a corner where the element is square. */
    val squareCornerRadius: Float,
) {
    /** Distance from the element's edge to the outer edge of the outer ring: the room a clipping parent must leave. */
    val outset: Float get() = gap + nearWidth + outerWidth
}

/** Firefox: a 2 dp near ring and a 1 dp outer ring, flush, with a 2 dp outer radius at square corners. */
val FIREFOX_RING_SPEC: FocusRingSpec =
    FocusRingSpec(nearWidth = 2f, outerWidth = 1f, gap = 0f, squareCornerRadius = 2f)


// What:     `data class RingCorners(...)` holds the four corner radii of one outline, in the unit of the caller.
// Why:      Segments of the mode control round only their outer corners, so one radius cannot describe every element.
//
// In TS you'd write (pseudocode):
// ```ts
// type RingCorners = { topLeft: number; topRight: number; bottomRight: number; bottomLeft: number };
// ```
/** The four corner radii of an outline, clockwise from the top left. */
data class RingCorners(
    /** Radius of the top left corner. */
    val topLeft: Float,
    /** Radius of the top right corner. */
    val topRight: Float,
    /** Radius of the bottom right corner. */
    val bottomRight: Float,
    /** Radius of the bottom left corner. */
    val bottomLeft: Float,
) {
    /** The corners after every rounded corner grows by [amount] and every square corner becomes [squareRadius]. */
    fun grownBy(amount: Float, squareRadius: Float): RingCorners = RingCorners(
        topLeft = grownRadius(topLeft, amount, squareRadius),
        topRight = grownRadius(topRight, amount, squareRadius),
        bottomRight = grownRadius(bottomRight, amount, squareRadius),
        bottomLeft = grownRadius(bottomLeft, amount, squareRadius),
    )

    /** Corner radii shared by the constructors below. */
    companion object {
        /** All four corners with the same radius. */
        fun uniform(radius: Float): RingCorners = RingCorners(radius, radius, radius, radius)
    }
}

// What:     `private fun grownRadius(radius, amount, squareRadius)` grows one corner.
// Why:      A rounded corner grows by the offset so the ring stays concentric with the element; a square corner has no
//           arc to grow, so it takes the spec's square-corner radius (Firefox gives it 2 dp).
//
// In TS you'd write (pseudocode):
// ```ts
// const grownRadius = (r: number, amount: number, square: number) => (r > 0 ? r + amount : square);
// ```
/** Returns [radius] plus [amount] for a rounded corner and [squareRadius] for a square one. */
private fun grownRadius(radius: Float, amount: Float, squareRadius: Float): Float =
    if (radius > 0f) radius + amount else squareRadius

// What:     `data class RingRect(...)` is a rectangle with rounded corners, in coordinates whose origin is the top left
//           corner of the element.
// Why:      The inner edge, the outer edge and the stroke centerline of the ring are three such rectangles.
//
// In TS you'd write (pseudocode):
// ```ts
// type RingRect = { left: number; top: number; right: number; bottom: number; corners: RingCorners };
// ```
/** A rounded rectangle positioned relative to the top left corner of an element. */
data class RingRect(
    /** Left edge, negative when the rectangle reaches outside the element. */
    val left: Float,
    /** Top edge, negative when the rectangle reaches outside the element. */
    val top: Float,
    /** Right edge. */
    val right: Float,
    /** Bottom edge. */
    val bottom: Float,
    /** Corner radii. */
    val corners: RingCorners,
) {
    /** Width of the rectangle. */
    val width: Float get() = right - left

    /** Height of the rectangle. */
    val height: Float get() = bottom - top
}

// What:     `fun ringRectGrownBy(...)` grows an element's outline by an offset on every side.
// Why:      Every edge of the ring is the element's outline grown by some amount: the gap for the near ring's inner
//           edge, the gap plus the near width for the edge between the rings, and the full outset for the outer edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const ringRectGrownBy = (w: number, h: number, corners: RingCorners, amount: number, square: number) =>
//   ({ left: -amount, top: -amount, right: w + amount, bottom: h + amount, corners: grow(corners, amount, square) });
// ```
/**
 * Returns the outline of an element of [width] by [height] with [corners], grown by [amount] on every side.
 *
 * Each rounded corner grows by [amount] too, so the grown outline stays concentric with the element, and each square
 * corner becomes [squareRadius].
 */
fun ringRectGrownBy(width: Float, height: Float, corners: RingCorners, amount: Float, squareRadius: Float): RingRect =
    RingRect(
        left = -amount,
        top = -amount,
        right = width + amount,
        bottom = height + amount,
        corners = corners.grownBy(amount = amount, squareRadius = squareRadius),
    )

// What:     `data class FocusRingBands(...)` is the two rings as three outlines.
// Why:      A solid ring is the area between two outlines, so the near ring lies between [inner] and [middle] and the
//           outer ring between [middle] and [outer], and neither needs a stroke whose corners would be guessed.
//
// In TS you'd write (pseudocode):
// ```ts
// type FocusRingBands = { inner: RingRect; middle: RingRect; outer: RingRect };
// ```
/** The three outlines that bound the near ring and the outer ring. */
data class FocusRingBands(
    /** Inner edge of the near ring: the element grown by the gap. */
    val inner: RingRect,
    /** Edge shared by the near ring and the outer ring. */
    val middle: RingRect,
    /** Outer edge of the outer ring. */
    val outer: RingRect,
)

// What:     `fun focusRingBands(width, height, corners, spec)` computes both rings for one element.
// Why:      One function states the rule for every element: bounds grown by the gap, then by the near width, then by
//           the outer width, with radii growing by the same amounts.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRingBands = (w, h, c, spec) => ({ inner: grow(gap), middle: grow(gap + near), outer: grow(outset) });
// ```
/**
 * Returns the outlines that bound the rings of an element of [width] by [height] with [corners].
 *
 * The inner edge keeps a square corner square, and a square corner's outer radius is the spec's square-corner radius
 * at the near ring and that radius plus the outer width at the outer ring, as Firefox paints it.
 */
fun focusRingBands(width: Float, height: Float, corners: RingCorners, spec: FocusRingSpec): FocusRingBands {
    /** Edge between the rings. */
    val middle: RingRect = ringRectGrownBy(
        width = width,
        height = height,
        corners = corners,
        amount = spec.gap + spec.nearWidth,
        squareRadius = spec.squareCornerRadius,
    )
    return FocusRingBands(
        inner = ringRectGrownBy(
            width = width,
            height = height,
            corners = corners,
            amount = spec.gap,
            squareRadius = 0f,
        ),
        middle = middle,
        outer = ringRectGrownBy(
            width = width,
            height = height,
            corners = corners,
            amount = spec.outset,
            squareRadius = spec.squareCornerRadius + spec.outerWidth,
        ),
    )
}

// What:     `enum class FocusRingPlacement` says on which side of the element's edge the ring pair is drawn.
// Why:      D117 draws the pair outside the element; D120 draws the same pair just inside the edge of an entry that
//           sits flush in a clipped list, where an outside ring would be cut off.
//
// In TS you'd write (pseudocode):
// ```ts
// type FocusRingPlacement = "outside" | "inside";
// ```
/** Where the focus ring pair sits relative to the element's own edge. */
enum class FocusRingPlacement {
    /** Around the element, reaching [FocusRingSpec.outset] beyond its edge (D117). */
    OUTSIDE,

    /** Within the element, its outer edge on the element's edge, so a clipping parent cannot cut it (D120). */
    INSIDE,
}

// What:     `private fun RingCorners.shrunkBy(amount)` takes every corner radius in by [amount], not below square.
// Why:      An inside ring is the outside ring of the element taken in by the outset; a rounded corner of radius r
//           becomes r - outset, so the ring's outer edge meets the element's edge with the element's own radius.
//
// In TS you'd write (pseudocode):
// ```ts
// const shrunkBy = (c: RingCorners, a: number) => mapCorners(c, (r) => Math.max(0, r - a));
// ```
/** Returns these corners with every radius reduced by [amount] and never below zero. */
private fun RingCorners.shrunkBy(amount: Float): RingCorners = RingCorners(
    topLeft = (topLeft - amount).coerceAtLeast(0f),
    topRight = (topRight - amount).coerceAtLeast(0f),
    bottomRight = (bottomRight - amount).coerceAtLeast(0f),
    bottomLeft = (bottomLeft - amount).coerceAtLeast(0f),
)

/** Returns this outline moved by [dx] and [dy] with its corners unchanged. */
private fun RingRect.movedBy(dx: Float, dy: Float): RingRect =
    RingRect(left = left + dx, top = top + dy, right = right + dx, bottom = bottom + dy, corners = corners)

// What:     `fun focusRingBandsInside(width, height, corners, spec)` computes both rings just inside an element's edge.
// Why:      D120: an entry flush in a clipped list draws the same two rings, the 2 dp near ring and the 1 dp outer
//           ring, inside its own edge. The pair keeps its exact shape, so it is the outside pair of the element
//           taken in by the outset, which leaves the outer ring's outer edge on the element's edge and the content
//           side of the near ring [FocusRingSpec.outset] inside it.
//
// In TS you'd write (pseudocode):
// ```ts
// const inside = (w, h, c, spec) => translate(focusRingBands(w - 2 * o, h - 2 * o, shrink(c, o), spec), o, o);
// ```
/**
 * Returns the outlines that bound the rings of an element of [width] by [height] with [corners], drawn inside it.
 *
 * Coordinates stay relative to the element's top left corner, so the outer edge of the result is the element's own
 * bounds and nothing reaches beyond them. An element too small to hold the rings gets empty inner outlines.
 */
fun focusRingBandsInside(width: Float, height: Float, corners: RingCorners, spec: FocusRingSpec): FocusRingBands {
    /** How far the pair is taken in from the element's edge. */
    val inset: Float = spec.outset
    /** The outside pair of the element taken in by [inset]. */
    val taken: FocusRingBands = focusRingBands(
        width = (width - 2f * inset).coerceAtLeast(0f),
        height = (height - 2f * inset).coerceAtLeast(0f),
        corners = corners.shrunkBy(inset),
        spec = spec,
    )
    return FocusRingBands(
        inner = taken.inner.movedBy(dx = inset, dy = inset),
        middle = taken.middle.movedBy(dx = inset, dy = inset),
        outer = taken.outer.movedBy(dx = inset, dy = inset),
    )
}

// What:     `fun focusRingBandsAt(placement, width, height, corners, spec)` picks the outside or the inside pair.
// Why:      The drawing routine takes the placement as a parameter, so one function serves every element.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRingBandsAt = (p, w, h, c, spec) => (p === "inside" ? inside(w, h, c, spec) : outside(w, h, c, spec));
// ```
/** Returns the ring outlines of an element for [placement]: around it (D117) or just inside its edge (D120). */
fun focusRingBandsAt(
    placement: FocusRingPlacement,
    width: Float,
    height: Float,
    corners: RingCorners,
    spec: FocusRingSpec,
): FocusRingBands = when (placement) {
    FocusRingPlacement.OUTSIDE -> focusRingBands(width = width, height = height, corners = corners, spec = spec)
    FocusRingPlacement.INSIDE -> focusRingBandsInside(width = width, height = height, corners = corners, spec = spec)
}

// What:     `fun focusRingShows(focused, lastInputWasKeyboard)` is the one rule for whether a ring is drawn.
// Why:      D117: a ring appears for keyboard focus only (CSS `:focus-visible`). Focus from a tap or a touch, or a
//           list that kept focus while the user went back to touching, shows nothing.
//
// In TS you'd write (pseudocode):
// ```ts
// const focusRingShows = (focused: boolean, lastInputWasKeyboard: boolean) => focused && lastInputWasKeyboard;
// ```
/** Returns true when an element that [focused] draws its ring, which needs the last input to be the keyboard. */
fun focusRingShows(focused: Boolean, lastInputWasKeyboard: Boolean): Boolean = focused && lastInputWasKeyboard
