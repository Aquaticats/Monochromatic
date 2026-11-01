//! Display-path helpers:
//!  one provider-supplied name becomes one path segment.
//!
//! Ported from `core/DisplayPath.kt` (`sanitizeComponent` and `joinDisplayPath`).
//! A `/` inside a name becomes a look-alike so depth accounting stays right,
//!  and
//! control characters become spaces so a path stays on one line.
//! No I/O,
//!  so it is fully unit-tested.

/// What:
///      `const SEPARATOR: char = '/';`.
///  The path separator that `join_display_path` glues segments with.
/// Why:
///       Naming the separator keeps the bare `'/'` literal out of the logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEPARATOR = "/";
/// ```
const SEPARATOR: char = '/';

/// What:
///      `const SEPARATOR_REPLACEMENT: char = '∕';`.
///  The glyph is U+2215
///  DIVISION SLASH,
///  which looks like `/` but is not the separator.
/// Why:
///       A real separator inside a name would fake an extra folder level,
///  so it
///  is swapped for this look-alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEPARATOR_REPLACEMENT = "∕"; // U+2215, not "/"
/// ```
const SEPARATOR_REPLACEMENT: char = '∕';

/// What:
///      `const CONTROL_REPLACEMENT: char = ' ';`.
///  One ASCII space.
/// Why:
///       Newlines,
///  carriage returns and tabs would break one-line display.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONTROL_REPLACEMENT = " ";
/// ```
const CONTROL_REPLACEMENT: char = ' ';

/// What:
///      `fn sanitize_char(character: char) -> char`.
///  Maps one character to
///  its safe replacement,
///  or returns it unchanged.
/// Why:
///       Keeping the per-character rule separate lets `sanitize_component`
///  stay a single chained expression.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sanitizeChar = (c: string) => (c === "/" ? "∕" : isControl(c) ? " " : c);
/// ```
fn sanitize_char(character: char) -> char {
    // Separator first, so that no later arm can leave a real `/` in place.
    if character == SEPARATOR {
        return SEPARATOR_REPLACEMENT;
    }
    // Control characters collapse to one space each.
    if character.is_control() {
        return CONTROL_REPLACEMENT;
    }
    return character;
}

/// What:
///      `pub fn sanitize_component(name: &str) -> String`.
///  Neutralizes the
///  separator and control characters of one provider-supplied name.
///  `..` and every other character pass through unchanged.
/// Why:
///       One name must contribute exactly one segment and no stray separators,
///  and must render on one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sanitizeComponent(name: string): string {
///   return [...name].map(sanitizeChar).join("");
/// }
/// ```
pub fn sanitize_component(name: &str) -> String {
    return name.chars().map(sanitize_char).collect();
}

/// What:
///      `pub fn join_display_path(prefix: &str, name: &str) -> String`.
///  Sanitizes `name` and appends it to `prefix` with one `/`.
///  An empty
///  prefix returns the bare sanitized segment.
/// Why:
///       A file directly in the chosen root has no prefix,
///  so no leading
///  separator is added.
///  The prefix is already sanitized by earlier joins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joinDisplayPath(prefix: string, name: string): string {
///   const segment = sanitizeComponent(name);
///   return prefix === "" ? segment : `${prefix}/${segment}`;
/// }
/// ```
pub fn join_display_path(prefix: &str, name: &str) -> String {
    let segment = sanitize_component(name);
    if prefix.is_empty() {
        return segment;
    }
    return format!("{prefix}{SEPARATOR}{segment}");
}

/// What:
///      `#[cfg(test)] #[path = "display_path_tests.rs"] mod tests;` declares a test-only submodule
///  whose code lives in the sibling file `display_path_tests.rs`.
/// Why:
///       Keep `display_path.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // display_path.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "display_path_tests.rs"]
mod tests;
