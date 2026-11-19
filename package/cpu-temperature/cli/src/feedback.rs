//! Chooses a ceiling within the available frequency range from one sample.

/// Percentage points in the whole adjustable frequency range.
pub const FULL_RANGE: u32 = 100;

/// Reduce the ceiling faster when the sample reaches the target.
const STEP_AT_TARGET: u32 = 20;

/// Reduce the ceiling as the sample approaches the target.
const STEP_NEAR_TARGET: u32 = 10;

/// Return frequency gradually after cooling.
const RESTORE_STEP: u32 = 5;

/// What
/// A function whose arguments are integers with explicit units. Rust's `i32`
/// is signed, while `u32`, `i64` and `usize` offer other sign, width or indexing choices.
/// Why
/// Signed millidegrees accommodate below-freezing readings. An unsigned percentage
/// represents a position in the frequency interval and fits within `u32`.
///
/// TypeScript pseudocode
/// ```ts
/// function adjustCeiling(previous: number, sample: number, target: number, margin: number): number
/// ```
pub fn adjust_ceiling(previous: u32, sample: i32, target: i32, margin: i32) -> u32 {
    if sample >= target {
        // What
        // Saturating subtraction clamps at zero when an unsigned difference would be negative.
        // Why
        // A hot reading at the frequency floor still gives the floor.
        // TypeScript pseudocode
        // return Math.max(0, previous - STEP_AT_TARGET);
        return previous.saturating_sub(STEP_AT_TARGET);
    }
    if sample >= target - margin {
        // Lower the cap while the reading is within the approach margin.
        return previous.saturating_sub(STEP_NEAR_TARGET);
    }
    if sample <= target - margin * 2 {
        // What
        // `min` caps a calculated value at the end of an interval.
        // Why
        // Repeated cool samples can restore the full range.
        // TypeScript pseudocode
        // return Math.min(previous + RESTORE_STEP, FULL_RANGE);
        return (previous + RESTORE_STEP).min(FULL_RANGE);
    }
    return previous;
}
