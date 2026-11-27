//! What:
//!  The names another crate can reach through a library root, read from the source text:
//!  every bare `pub` item whose module the `pub mod` chain from the library root reaches,
//!  and every `pub use` line that carries a name out of a module that chain does not reach.
//! Why:
//!  A `mod` line and a `use` line decide reachability in two different ways,
//!  and a reader of one file sees neither:
//!  the surface scan states both once,
//!  so a check of the published surface reads them from one place.
//! Gotcha:
//!  The names are read from lines,
//!  so the scan sees a `pub fn` written the way the crate writes it,
//!  at the first column of its own line.
//!  An item inside an inline `mod name { ... }` body is not read:
//!  this crate declares no inline module that is `pub`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const surface = surfaceOf(srcRoot); // { items: NameableItem[], carried: CarriedName[] }
//! ```

use super::modules_a_library_root_reaches_tests::{
    ModuleOfTheTree,
    chain_the_target_reaches,
    read_the_tree,
};
use std::collections::BTreeMap;
use std::path::Path;

/// The words a `pub` declaration can put between its visibility and its name,
/// which are the item kinds this crate declares.
const ITEM_KINDS: &[&str] = &[
    "fn", "struct", "enum", "union", "trait", "const", "static", "type",
];

/// What:
///  `pub(crate) struct NameableItem` is one item another crate can name:
///  its kind (`fn`, `struct`, ...),
///  its name,
///  and the file it is written in.
/// Why:
///  A finding names the item and the file a reader opens to narrow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NameableItem = { kind: string; name: string; file: string }
/// ```
pub(crate) struct NameableItem {
    /// The item's kind, as its declaration writes it.
    pub(crate) kind: String,
    /// The item's name.
    pub(crate) name: String,
    /// The file, relative to the source root, that declares it.
    pub(crate) file: String,
}

/// What:
///  `pub(crate) struct CarriedName` is one `pub use` site whose named module is not reachable
///  through `pub mod` lines:
///  the public re-export is what makes the name reachable,
/// and the site is what a reader has to review.
/// Why:
///  `pub use` can publish a `pub` item of a private module,
///  so a surface read from `pub mod` chains alone would miss it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarriedName = { file: string; target: string }
/// ```
pub(crate) struct CarriedName {
    /// The file holding the `pub use` line.
    pub(crate) file: String,
    /// The module part of the re-exported path, as the line writes it.
    pub(crate) target: String,
}

/// What:
///  `pub(crate) struct Surface` is everything another crate can name through one library root.
/// Why:
///  The check compares the two halves against one list each,
///  and a reader of a run's answer sees both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Surface = { items: NameableItem[]; carried: CarriedName[] }
/// ```
pub(crate) struct Surface {
    /// Every `pub` item the `pub mod` chain reaches.
    pub(crate) items: Vec<NameableItem>,
    /// Every `pub use` whose named module that chain does not reach.
    pub(crate) carried: Vec<CarriedName>,
}

/// What:
///  `fn word_after(line: &str) -> Option<&str>` reads the first word of a declaration's tail:
///  the name, or the `fn` of a `pub const fn`.
/// Why:
///  `pub const fn` puts two keywords before the name,
/// and every other kind puts one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordAfter = (line: string): string | undefined => line.match(/^[A-Za-z_]\w*/)?.[0];
/// ```
fn word_after(line: &str) -> Option<&str> {
    let end: usize = line
        .find(|character: char| return !(character.is_alphanumeric() || character == '_'))
        .unwrap_or(line.len());
    if end == 0 {
        return None;
    }
    return Some(&line[..end]);
}

/// What:
///  `fn nameable_declarations(text: &str) -> Vec<(String, String)>` lists the `(kind, name)` of every declaration
///  the text writes as a bare `pub` item at the first column,
///  and of every `#[macro_export] macro_rules!` name.
/// Why:
///  Exactly `pub` publishes a name to another crate:
/// `pub(crate)`, `pub(super)` and `pub(in path)` do not.
///  A `#[cfg(test)]` attribute above a declaration keeps it out of the release build,
///  so it publishes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nameableDeclarations(text: string): [string, string][]
/// ```
fn nameable_declarations(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut attributes: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        if line.starts_with("#[") {
            attributes.push(line);
            continue;
        }
        let exported: bool = attributes
            .iter()
            .any(|attribute| return *attribute == "#[macro_export]");
        let only_for_tests: bool = attributes.iter().any(|attribute| {
            let Some(predicate) = attribute
                .strip_prefix("#[cfg(")
                .and_then(|rest| return rest.strip_suffix(")]"))
            else {
                return false;
            };
            return predicate == "test"
                || (predicate.starts_with("all(") && predicate.contains("test"));
        });
        attributes.clear();
        if only_for_tests {
            continue;
        }
        if exported {
            let name: Option<&str> = line.strip_prefix("macro_rules! ").and_then(|rest| {
                return rest
                    .split(|character: char| return character == ' ' || character == '{')
                    .next();
            });
            if let Some(name) = name {
                found.push((String::from("macro_rules"), name.to_owned()));
            }
            continue;
        }
        let Some(rest) = line.strip_prefix("pub ") else {
            continue;
        };
        let Some(word) = word_after(rest) else {
            continue;
        };
        if !ITEM_KINDS.contains(&word) {
            continue;
        }
        let mut kind: &str = word;
        let mut tail: &str = rest[word.len()..].trim_start();
        if kind == "const" && tail.starts_with("fn ") {
            kind = "fn";
            tail = tail["fn ".len()..].trim_start();
        }
        let Some(name) = word_after(tail) else {
            continue;
        };
        found.push((kind.to_owned(), name.to_owned()));
    }
    return found;
}

/// What:
///  `fn carried_targets(text: &str) -> Vec<String>` lists the module part of every `pub use` line:
///  the text before the last `::` that is outside braces.
/// Why:
///  `pub use a::b::{C, D as E}` publishes `C` and `E` out of `a::b`,
///  so `a::b` is the module whose reachability decides it;
///  a line with no `::` re-exports a name of its own module and carries nothing out of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carriedTargets(text: string): string[]
/// ```
fn carried_targets(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        let Some(body) = trimmed.strip_prefix("pub use ") else {
            continue;
        };
        let Some(statement) = body.strip_suffix(';') else {
            continue;
        };
        let mut depth: usize = 0;
        let mut cut: Option<usize> = None;
        for (offset, character) in statement.char_indices() {
            if character == '{' {
                depth += 1;
                continue;
            }
            if character == '}' {
                depth = depth.saturating_sub(1);
                continue;
            }
            if character == ':' && statement[offset..].starts_with("::") && depth == 0 {
                cut = Some(offset);
            }
        }
        let Some(offset) = cut else {
            continue;
        };
        let target: &str = statement[..offset].trim();
        if !target.is_empty() {
            found.push(target.to_owned());
        }
    }
    return found;
}

/// What:
///  `pub(crate) fn surface_of(root: &Path) -> Result<Surface, String>` reads the whole published surface of the
///  library root under `root`.
/// Why:
///  One reading serves the check and the list beside it,
/// and the limits of the reading are stated in this file's own doc rather than in each caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function surfaceOf(root: string): Surface  // throws Error
/// ```
pub(crate) fn surface_of(root: &Path) -> Result<Surface, String> {
    let modules: BTreeMap<String, ModuleOfTheTree> = read_the_tree(root)?;
    let mut items: Vec<NameableItem> = Vec::new();
    let mut carried: Vec<CarriedName> = Vec::new();
    for (chain, module) in &modules {
        if !module.public || module.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(module.file.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", module.file))?;
        for (kind, name) in nameable_declarations(text.as_str()) {
            items.push(NameableItem {
                kind,
                name,
                file: module.file.clone(),
            });
        }
        for target in carried_targets(text.as_str()) {
            let reached: Option<String> =
                chain_the_target_reaches(&modules, chain, target.as_str());
            let public: bool = reached
                .and_then(|found| return modules.get(found.as_str()))
                .is_some_and(|found| return found.public && !found.test_only);
            if !public {
                carried.push(CarriedName {
                    file: module.file.clone(),
                    target,
                });
            }
        }
    }
    return Ok(Surface {
        items,
        carried,
    });
}
