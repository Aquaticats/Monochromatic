//! Formatting a duration for the window.
//!
//! What:
//! `format_time` writes seconds as "m:ss",
//! the one text the seek bar and the duration label show.
//! Why:
//! Slint's own number-to-string is awkward,
//! so one function here formats it and every caller passes text,
//! split from `src/main.rs` to stay within the line budget.

/// What:
///      `fn format_time(secs: f64) -> String`.
///  Format seconds as "m:ss".
///           `f64` is a 64-bit float (sibling:
///  `f32`);
///  `String` is an owned heap
///           string (sibling:
///  `&str`,
///  a borrowed view we could not return here
///           because it would point at this function's freed locals).
/// Why:
///       Slint number-to-string is awkward,
///  so we format here and pass strings;
///           the result is `String` (owned) so the caller can keep it past this call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatTime(secs: number): string { ... }
/// ```
pub(crate) fn format_time(secs: f64) -> String {
    // What:     `let whole = if secs > 0.0 { secs as u64 } else { 0 };`. `if/else`
    //           used as an EXPRESSION (both arms yield a value). `secs as u64` is a
    //           numeric CAST that truncates the float toward zero into a 64-bit
    //           unsigned integer (`u64`; siblings: `u32`, `usize`). The else arm
    //           clamps negatives and NaN to 0.
    // Why:      Avoid negative or garbage times; `u64` because a track length never
    //           needs a sign and easily fits a 64-bit count of seconds.
    // Gotcha:   `as u64` is a SATURATING/truncating cast, not TS's `Number` coercion:
    //           NaN becomes 0 and the fraction is dropped, with no exception.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const whole = secs > 0 ? Math.floor(secs) : 0;
    // ```
    let whole = if secs > 0.0 { secs as u64 } else { 0 };
    // What:     `format!("{}:{:02}", whole / 60, whole % 60)`. The `format!` MACRO
    //           builds an owned `String` from a template: `{}` interpolates minutes
    //           (`whole / 60`, integer division) and `{:02}` interpolates seconds
    //           (`whole % 60`) zero-padded to 2 digits. No trailing `;`, so this is
    //           the function's tail expression and is returned.
    // Why:      "3:07" style display.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, "0")}`;
    // ```
    return format!("{}:{:02}", whole / 60, whole % 60);
}
