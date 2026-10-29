//! Tab stops are pixel positions,
//!  so a tab ends at the same place whatever script precedes it.
//!
//! This follows the CSS Text Level 3 rule that editord's `tab-size: 2` used:
//! stops lie at multiples of the tab size from the line's start edge,
//! the tab size is a multiple of the space character's advance,
//! and a tab that would be narrower than half a space runs on to the following stop.
//! Counting character columns instead would put a tab after a CJK glyph,
//!  whose fallback-font advance
//! is not two Latin advances,
//!  at a different pixel position than a tab after two Latin letters.

/// What:
///  `pub const` exports a compile-time value;
///  `f32` is a 32-bit float (sibling `f64`).
/// Why:
///  Two space advances per stop match the visible indentation of editord's `tab-size: 2`.
/// `f32` is the unit of every glyph advance in the shaping engine,
///  so no conversion is needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TAB_SPACES = 2;
/// ```
pub const TAB_SPACES: f32 = 2.0;

/// Width of a tab that starts `position` pixels from the line's start edge.
///
/// `space` is the measured advance of one space character in the same pixel unit.
/// A space advance that is not a positive number has no stops;
///  the tab then keeps the width of one space,
/// so a font without a usable space glyph cannot produce an infinite or undefined width.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tabAdvance(position: number, space: number): number {
///   if (!(space > 0)) return space;
///   const size = space * TAB_SPACES;
///   let distance = size - (((position % size) + size) % size);
///   if (distance < space / 2) distance += size;
///   return distance;
/// }
/// ```
pub fn tab_advance(position: f32, space: f32) -> f32 {
    // A not-a-number space fails `space > 0.0`, so it is rejected together with zero and negative advances.
    let usable = space > 0.0 && space.is_finite() && position.is_finite();
    if !usable {
        return space;
    }
    let size = space * TAB_SPACES;
    // `rem_euclid` is the remainder that is never negative, even for a negative position.
    let mut distance = size - position.rem_euclid(size);
    if distance < space / 2.0 {
        distance += size;
    }
    return distance;
}

/// Extra advance to add to each tab of one line so every tab ends on a stop.
///
/// `naturals` holds,
///  in reading order,
///  the start position of each tab when every tab is drawn as one space.
/// Widening an earlier tab moves every later one,
///  so the correction accumulates from left to right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tabSpacings(naturals: number[], space: number): number[] {
///   let shift = 0;
///   return naturals.map(natural => {
///     const extra = tabAdvance(natural + shift, space) - space;
///     shift += extra;
///     return extra;
///   });
/// }
/// ```
pub fn tab_spacings(naturals: &[f32], space: f32) -> Vec<f32> {
    // What: `Vec::new()` creates an empty growable list (siblings: fixed `[f32; N]`, borrowed `&[f32]`).
    // Why: The number of tabs differs per line, and the caller keeps the result after this call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spacings: number[] = [];
    // ```
    let mut spacings = Vec::new();
    let mut shift = 0.0;
    // What: `for natural in naturals` lends each element as `&f32`; `*natural` reads the number behind it.
    // Why: The slice stays owned by the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const natural of naturals) { ... }
    // ```
    for natural in naturals {
        let extra = tab_advance(*natural + shift, space) - space;
        spacings.push(extra);
        shift += extra;
    }
    return spacings;
}
