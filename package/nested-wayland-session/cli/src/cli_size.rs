//! Size validation shared by clap and parser tests.

/// What:
///  Import application errors and context for numeric parsing.
/// Why:
///  Keep dimension failures explicit instead of accepting malformed geometry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Throw contextual Error objects on malformed input.
/// ```
use anyhow::{bail, Context, Result};

/// Parse a `WIDTHxHEIGHT` string into a positive `(width, height)` pair.
///
/// What:
///      `pub(super) fn parse_size(spec: &str) -> Result<(i32, i32)>`.
///  Private helper
///           shared only with the parser module.
///  Borrows the spec string and returns two `i32`s or
///           an error.
///  `(i32, i32)` is an anonymous two-field tuple.
/// Why:
///       Isolate the split-and-validate logic so both parsing and tests can
///           exercise it directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSize(spec: string): [number, number] { ... }
/// ```
///
/// @example
/// ```ts
/// parseSize("800x600"); // => [800, 600]
/// ```
pub(super) fn parse_size(spec: &str) -> Result<(i32, i32)> {
    // What:     `let mut parts = spec.split(['x', 'X']);`. `.split(pattern)` returns
    //           a lazy iterator of substrings between separators; the pattern is a
    //           two-element array of `char`s, so it splits on either lowercase or
    //           uppercase `x`. `let mut` because iterating with `.next()` mutates the
    //           iterator's cursor.
    // Why:      Accept both `1280x720` and `1280X720` spellings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts = spec.split(/[xX]/)[Symbol.iterator]();
    // ```
    let mut parts = spec.split(['x', 'X']);

    // What:     `let width_text = parts.next().context(...)?;`. `.next()` yields the
    //           first substring as `Option<&str>`; `.context(msg)?` maps `None` to an
    //           error and unwraps `Some`.
    // Why:      Extract the width portion.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const widthText = parts.next().value ?? throwError("...");
    // ```
    let width_text = parts.next().context("--size must be WIDTHxHEIGHT")?;

    // What:     `let height_text = parts.next().context(...)?;`. Second substring.
    // Why:      Extract the height portion.
    let height_text = parts.next().context("--size must be WIDTHxHEIGHT")?;

    // What:     `if parts.next().is_some()`. A third element means an extra `x`, e.g.
    //           `1x2x3`. `.is_some()` is `true` when the `Option` holds a value.
    // Why:      Reject malformed specs with too many separators.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (parts.next().value !== undefined) throw new Error("...");
    // ```
    if parts.next().is_some() {
        bail!("--size must be exactly WIDTHxHEIGHT");
    }

    // What:     `let width: i32 = width_text.trim().parse().context(...)?;`.
    //           `.trim()` drops surrounding whitespace; `.parse()` reads the string
    //           as the target type inferred from the `: i32` annotation, returning
    //           `Result<i32, ParseIntError>`; `.context(msg)?` attaches a message and
    //           unwraps.
    // Why:      Convert text to a number, failing cleanly on non-numeric input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const width = Number.parseInt(widthText.trim(), 10);
    // if (Number.isNaN(width)) throw new Error("...");
    // ```
    let width: i32 = width_text
        .trim()
        .parse()
        .context("--size width is not a number")?;

    // What:     `let height: i32 = height_text.trim().parse().context(...)?;`. Same
    //           parse for the height field.
    // Why:      Convert the height text to a number.
    let height: i32 = height_text
        .trim()
        .parse()
        .context("--size height is not a number")?;

    // What:     `if width <= 0 || height <= 0`. Ordinary comparison and logical OR.
    // Why:      A zero or negative screen size cannot be created; reject it early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (width <= 0 || height <= 0) throw new Error("...");
    // ```
    if width <= 0 || height <= 0 {
        bail!("--size dimensions must be positive");
    }

    // What:     `Ok((width, height))`. Wrap the validated tuple as the success value;
    //           tail expression, so it is returned.
    // Why:      Hand the pair back to `parse_args`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [width, height];
    // ```
    return Ok((width, height))
}
