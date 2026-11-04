//! The `mod name;` declarations of one source file, with the attributes that decide where the module's file is and
//! whether the module is compiled for tests only.
//! The reading is textual and strict: a declaration it cannot place exactly is an error, because a guess here would
//! decide which files a scan treats as release code.

/// What:
///  `pub(crate) struct Declaration { ... }` is a record type (TS: an object type) with three fields:
///  `name: String` (owned text; sibling `&str`, a borrowed view, would not outlive the line it was cut from),
///  `path_attribute: Option<String>` (`Some(text)` when `#[path = "text"]` was written, `None` when not; TS:
///  `string | undefined`), and `test_only: bool`.
/// Why:
///  The module tree resolver needs exactly these three facts per declaration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declaration = { name: string; pathAttribute: string | undefined; testOnly: boolean }
/// ```
pub(crate) struct Declaration {
    /// The module name after `mod`.
    pub(crate) name: String,
    /// The text of a `#[path = "..."]` attribute on the declaration.
    pub(crate) path_attribute: Option<String>,
    /// Whether a `cfg` attribute on the declaration compiles the module only when testing.
    pub(crate) test_only: bool,
}

/// What:
///  `fn split_top_level(text: &str) -> Vec<&str>` cuts `text` at the commas that are not inside parentheses, so
///  `test, any(unix, windows)` gives two pieces. `&str` is a borrowed view of text (sibling: `String`, owned).
///  `Vec<&str>` is a growable list of those views.
/// Why:
///  `cfg(all(test, target_os = "linux"))` lists conditions that must all hold, and the check needs each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitTopLevel(text: string): string[]
/// ```
fn split_top_level(text: &str) -> Vec<&str> {
    let mut pieces: Vec<&str> = Vec::new();
    // `usize` is the unsigned integer wide enough for any text offset (siblings `u32`, `u64`, `i32`); every
    // standard-library text offset is a `usize`, so no casts are needed.
    let mut depth: usize = 0;
    let mut start: usize = 0;
    // `char_indices()` yields each character with its byte offset; `(offset, character)` is a tuple destructured
    // in the loop header (TS: `for (const [offset, character] of ...)`).
    for (offset, character) in text.char_indices() {
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth = depth.saturating_sub(1);
        } else if character == ',' && depth == 0 {
            // `&text[start..offset]` borrows the slice from `start` up to, not including, `offset`.
            pieces.push(text[start..offset].trim());
            start = offset + 1;
        }
    }
    pieces.push(text[start..].trim());
    // `pieces` is the tail expression, so it is returned; it is a plain list, not wrapped in `Ok`/`Some`.
    return pieces;
}

/// What:
///  `fn is_test_gate(predicate: &str) -> bool` says whether a `cfg` condition holds only when testing: it is
///  exactly `test`, or an `all(...)` that lists `test` among its conditions (recursively).
/// Why:
///  Such a module is absent from the release build. A condition like `any(test, unix)` or `not(test)` is compiled in
///  release, so it is not a test gate; reading it as release is the safe direction (the scan reads more, not less).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTestGate(predicate: string): boolean
/// ```
fn is_test_gate(predicate: &str) -> bool {
    let trimmed: &str = predicate.trim();
    if trimmed == "test" {
        return true;
    }
    // `if let Some(inner) = ...` runs the block only when the value is `Some`, binding the inside to `inner`.
    // `.and_then(...)` continues only when the previous step gave `Some`.
    if let Some(inner) = trimmed.strip_prefix("all(").and_then(|rest| return rest.strip_suffix(')')) {
        for condition in split_top_level(inner) {
            if is_test_gate(condition) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  `fn module_name(line: &str) -> Option<&str>` reads one trimmed source line as `[visibility] mod name;` and gives
///  `Some(name)`, or `None` when the line is anything else (including an inline `mod name {`).
/// Why:
///  Both the resolver and the boundary scan need to recognise a module declaration line the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moduleName(line: string): string | undefined
/// ```
pub(crate) fn module_name(line: &str) -> Option<&str> {
    let mut rest: &str = line.trim();
    // Visibility prefixes, longest first so `pub(crate) ` is not cut short by `pub `.
    for prefix in ["pub(crate) ", "pub(super) ", "pub "] {
        if let Some(after) = rest.strip_prefix(prefix) {
            rest = after;
            break;
        }
    }
    let after_keyword: &str = rest.strip_prefix("mod ")?;
    let name: &str = after_keyword.strip_suffix(';')?;
    if name.is_empty() || !name.chars().all(|character| return character.is_alphanumeric() || character == '_') {
        return None;
    }
    // `Some(name)` is the present variant of `Option`, wrapping the borrowed name; it is the returned tail.
    return Some(name);
}

/// What:
///  `pub(crate) fn declarations(text: &str) -> Result<Vec<Declaration>, String>` lists the file-module declarations
///  of one source text in order, each with the attributes written directly above it (comments and blank lines
///  between do not detach them).
/// Why:
///  The resolver follows these to find every file the compiler would read. It fails on an indented declaration (an
///  inline module's child, which resolves differently) and on a `path` attribute it cannot read, instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function declarations(text: string): Declaration[]  // throws Error
/// ```
pub(crate) fn declarations(text: &str) -> Result<Vec<Declaration>, String> {
    let mut found: Vec<Declaration> = Vec::new();
    let mut attributes: Vec<&str> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") {
            attributes.push(trimmed);
            continue;
        }
        let Some(name) = module_name(trimmed) else {
            // Any other line (an item, a use, a statement) detaches the attributes above it.
            attributes.clear();
            continue;
        };
        if line.starts_with(char::is_whitespace) {
            return Err(format!("`mod {name};` is indented; a module declared inside an inline module is not resolved"));
        }
        let mut path_attribute: Option<String> = None;
        let mut test_only: bool = false;
        for attribute in attributes.drain(..) {
            if attribute.starts_with("#[path") {
                // `#[path = "x.rs"]` is the only spelling read; `strip_prefix` and `strip_suffix` cut the wrapper off.
                let value: Option<&str> =
                    attribute.strip_prefix("#[path = \"").and_then(|rest| return rest.strip_suffix("\"]"));
                let Some(written) = value else {
                    return Err(format!("`{attribute}` above `mod {name};` is not in the `#[path = \"...\"]` form"));
                };
                path_attribute = Some(written.to_owned());
            } else if attribute.starts_with("#[cfg_attr(") && attribute.contains("path") {
                return Err(format!("`{attribute}` above `mod {name};` chooses a file by condition; not resolved"));
            } else if let Some(predicate) = attribute.strip_prefix("#[cfg(").and_then(|rest| return rest.strip_suffix(")]")) {
                test_only = test_only || is_test_gate(predicate);
            }
        }
        found.push(Declaration { name: name.to_owned(), path_attribute, test_only });
    }
    // `Ok(found)` is the success variant of `Result`; the tail expression is the return value.
    return Ok(found);
}
