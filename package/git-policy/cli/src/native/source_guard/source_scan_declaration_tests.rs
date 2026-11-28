//! The `mod name;` declarations of one source file,
//! with the attributes that decide where the module's file is and
//! whether the module is compiled for tests only.
//! The reading is textual and strict:
//! a declaration it cannot place exactly is an error,
//! because a guess here would
//! decide which files a scan treats as release code.
//! The visibilities it places are
//! `mod`,
//! `pub mod`,
//! `pub(crate) mod` and
//! `pub(super) mod`.
//! A `mod` statement at a file's first column written with any other visibility
//! (`pub(in crate::a) mod x;`,
//! `pub(self) mod x;`)
//! or with text after its `;` (`mod x; // a note`) is an error,
//! because the module it declares would be missing from every scan in silence.
//! An inline `mod name { ... }` declares no file of its own and is not read.
//! The reading is one line at a time and knows nothing of string literals or block comments,
//! so it reads a `mod` statement at a file's first column wherever it stands:
//! fixture text holding one is written indented,
//! which is why an indented line is skipped rather than refused.

/// What:
///  `pub(crate) struct Declaration { ... }` is a record type (TS:
/// an object type) with three fields:
///  `name: String` (owned text;
/// sibling `&str`,
/// a borrowed view,
/// would not outlive the line it was cut from),
///  `path_attribute: Option<String>` (`Some(text)` when `#[path = "text"]` was written,
/// `None` when not;
/// TS:
///  `string | undefined`),
/// `test_only: bool` and `public: bool`.
/// Why:
///  The module tree resolver needs the first three facts per declaration,
/// and the surface scan asks whether the declaration is exactly `pub mod`,
/// which is what lets another crate name the module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declaration = { name: string; pathAttribute: string | undefined; testOnly: boolean; public: boolean }
/// ```
pub(crate) struct Declaration {
    /// The module name after `mod`.
    pub(crate) name: String,
    /// The text of a `#[path = "..."]` attribute on the declaration.
    pub(crate) path_attribute: Option<String>,
    /// Whether a `cfg` attribute on the declaration compiles the module only when testing.
    pub(crate) test_only: bool,
    /// Whether the line reads exactly `pub mod`,
    /// which publishes the module and every name it exports.
    pub(crate) public: bool,
}

/// What:
///  `fn split_top_level(text: &str) -> Vec<&str>` cuts `text` at the commas that are not inside parentheses,
/// so
///  `test, any(unix, windows)` gives two pieces.
/// `&str` is a borrowed view of text (sibling:
/// `String`,
/// owned).
///  `Vec<&str>` is a growable list of those views.
/// Why:
///  `cfg(all(test, target_os = "linux"))` lists conditions that must all hold,
/// and the check needs each one.
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
///  `fn is_test_gate(predicate: &str) -> bool` says whether a `cfg` condition holds only when testing:
/// it is
///  exactly `test`,
/// or an `all(...)` that lists `test` among its conditions (recursively).
/// Why:
///  Such a module is absent from the release build.
/// A condition like `any(test, unix)` or `not(test)` is compiled in
///  release,
/// so it is not a test gate;
/// reading it as release is the safe direction (the scan reads more,
/// not less).
/// Gotcha:
///  This is the one reading of a test gate,
///  for a `mod` line and for an item alike,
///  so a predicate is never a test gate in one place and release code in another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTestGate(predicate: string): boolean
/// ```
pub(crate) fn is_test_gate(predicate: &str) -> bool {
    let trimmed: &str = predicate.trim();
    if trimmed == "test" {
        return true;
    }
    // `if let Some(inner) = ...` runs the block only when the value is `Some`, binding the inside to `inner`.
    // `.and_then(...)` continues only when the previous step gave `Some`.
    if let Some(inner) = trimmed
        .strip_prefix("all(")
        .and_then(|rest| return rest.strip_suffix(')'))
    {
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
///  `Some(name)`,
/// or `None` when the line is anything else (including an inline `mod name {`).
/// Why:
///  Both the resolver and the boundary scan need to recognise a module declaration line the same way.
///  The visibilities read are the ones this crate writes:
///  `mod`,
///  `pub mod`,
///  `pub(crate) mod` and
///  `pub(super) mod`.
///  `declarations` errors on a module statement written any other way rather than stepping over it.
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
    if name.is_empty()
        || !name
            .chars()
            .all(|character| return character.is_alphanumeric() || character == '_')
    {
        return None;
    }
    // `Some(name)` is the present variant of `Option`, wrapping the borrowed name; it is the returned tail.
    return Some(name);
}

/// What:
///  `fn starts_a_module_declaration(line: &str) -> bool` says whether that line is written as the start of a
///  file-module declaration,
///  whatever visibility stands before its `mod`:
///  `mod name`,
///  `pub mod name`,
///  `pub(crate) mod name`,
///  `pub(super) mod name`,
///  `pub(in path) mod name` and
///  `pub(self) mod name`.
/// Why:
///  `declarations` stops on a module statement it cannot place,
///  and a line that is no module statement at all is no error there:
///  an item,
///  a `use`,
///  a statement.
///  An inline `mod name {` starts one too,
///  and the caller reads its `{` to tell that shape apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startsAModuleDeclaration(line: string): boolean
/// ```
fn starts_a_module_declaration(line: &str) -> bool {
    let trimmed: &str = line.trim();
    let after_visibility: &str = if let Some(rest) = trimmed.strip_prefix("pub(") {
        let Some((_inside, after)) = rest.split_once(')') else {
            return false;
        };
        after.trim_start()
    } else if let Some(after) = trimmed.strip_prefix("pub ") {
        after
    } else {
        trimmed
    };
    return after_visibility.starts_with("mod ");
}

/// What:
///  `fn leading_attributes(line: &str) -> (Vec<&str>, &str)` splits the attributes a line starts with from the
///  declaration written after them on the same line.
///  `Vec<&str>` is a growable list of borrowed views of the line (sibling:
///  `String`,
///  owned).
/// Why:
///  An attribute may stand on the declaration's own line (`#[inline] pub fn inlined() {}`,
///  `#[cfg(test)] mod support;`),
///  and a reader that took such a line for an attribute alone placed no declaration below it
///  (the alpha.52 read's L1).
///  The module tree and the published surface both read a declaration line through this one split,
///  so a shape one of them places is a shape the other places too.
///  An attribute's `]` is found with its square brackets counted and its string literals skipped,
///  so `#[cfg(all(not(test), unix))]` and a value that holds a bracket end where the attribute ends.
/// Gotcha:
///  A line whose attribute is not closed answers the whole line as one attribute and no declaration,
///  which is the reading these scans gave a line starting with `#[` before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadingAttributes(line: string): [string[], string]
/// ```
pub(crate) fn leading_attributes(line: &str) -> (Vec<&str>, &str) {
    let bytes: &[u8] = line.as_bytes();
    let mut found: Vec<&str> = Vec::new();
    let mut at: usize = 0;
    while line[at..].starts_with("#[") {
        let start: usize = at;
        let mut depth: usize = 1;
        let mut index: usize = at + "#[".len();
        while index < bytes.len() && depth > 0 {
            match bytes[index] {
                b'"' => {
                    index += 1;
                    while index < bytes.len() && bytes[index] != b'"' {
                        if bytes[index] == b'\\' {
                            index += 1;
                        }
                        index += 1;
                    }
                }
                b'[' => depth += 1,
                b']' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth > 0 {
            found.push(line.trim_end());
            return (found, "");
        }
        found.push(&line[start..index]);
        at = index;
        while at < bytes.len() && bytes[at] == b' ' {
            at += 1;
        }
    }
    return (found, &line[at..]);
}

/// What:
///  `pub(crate) fn declarations(text: &str) -> Result<Vec<Declaration>, String>` lists the file-module declarations
///  of one source text in order,
/// each with the attributes written directly above it (comments and blank lines
///  between do not detach them)
///  and those written on its own line before it.
/// Why:
///  The resolver follows these to find every file the compiler would read.
/// It fails on an indented declaration (an
///  inline module's child,
/// which resolves differently),
/// on a module statement written with a visibility or an ending it does not place,
///  and on a `path` attribute it cannot read,
/// instead of guessing.
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
        // What: the attributes this line starts with, and the declaration written after them on it.
        // Why: `#[cfg(test)] mod support;` declares a module, and a reader that took the line for an
        //      attribute alone left that module out of every scan of the tree in silence.
        let (written_attributes, declaration): (Vec<&str>, &str) = if trimmed.starts_with("#[") {
            leading_attributes(trimmed)
        } else {
            (Vec::new(), trimmed)
        };
        attributes.extend(written_attributes);
        if declaration.trim().is_empty() {
            continue;
        }
        let Some(name) = module_name(declaration) else {
            // What: A `mod` statement at a file's first column that this reading cannot place stops the scan.
            // Why: Stepping over it would leave its module out of the tree, and the surface would be short by
            //      every name that module publishes, with nothing said.
            //      An indented one is inside an inline module or a fixture's own text, which is no file of the
            //      tree either way.
            if starts_a_module_declaration(declaration)
                && !declaration.contains('{')
                && !line.starts_with(char::is_whitespace)
            {
                return Err(format!(
                    "`{trimmed}` is no `mod` declaration this reading places, so the module it declares would be missing from the tree: write `mod`, `pub mod`, `pub(crate) mod` or `pub(super) mod`, and end the line at its `;`"
                ));
            }
            // Any other line (an item, a use, a statement) detaches the attributes above it.
            attributes.clear();
            continue;
        };
        if line.starts_with(char::is_whitespace) {
            return Err(format!(
                "`mod {name};` is indented; a module declared inside an inline module is not resolved"
            ));
        }
        let mut path_attribute: Option<String> = None;
        let mut test_only: bool = false;
        for attribute in attributes.drain(..) {
            if attribute.starts_with("#[path") {
                // `#[path = "x.rs"]` is the only spelling read; `strip_prefix` and `strip_suffix` cut the wrapper off.
                let value: Option<&str> = attribute
                    .strip_prefix("#[path = \"")
                    .and_then(|rest| return rest.strip_suffix("\"]"));
                let Some(written) = value else {
                    return Err(format!(
                        "`{attribute}` above `mod {name};` is not in the `#[path = \"...\"]` form"
                    ));
                };
                path_attribute = Some(written.to_owned());
            } else if attribute.starts_with("#[cfg_attr(") && attribute.contains("path") {
                return Err(format!(
                    "`{attribute}` above `mod {name};` chooses a file by condition; not resolved"
                ));
            } else if let Some(predicate) = attribute
                .strip_prefix("#[cfg(")
                .and_then(|rest| return rest.strip_suffix(")]"))
            {
                test_only = test_only || is_test_gate(predicate);
            }
        }
        found.push(Declaration {
            name: name.to_owned(),
            path_attribute,
            test_only,
            public: declaration.starts_with("pub mod "),
        });
    }
    // `Ok(found)` is the success variant of `Result`; the tail expression is the return value.
    return Ok(found);
}
