//! What:
//!  The names another crate can reach through a library root,
//!  read from the source text:
//!  every bare `pub` item whose module the `pub mod` chain from the library root reaches,
//!  every bare `pub` field or method of a type that chain publishes,
//!  and every name a `pub use` carries out of a module that chain does not reach.
//! Why:
//!  A `mod` line and a `use` line decide reachability in two different ways,
//!  and a reader of one file sees neither:
//!  the surface scan states both once,
//!  so a check of the published surface reads them from one place.
//! Gotcha:
//!  Every name is keyed by the module path that publishes it,
//!  `<module path> <kind> <name>`,
//!  so two items of one kind and name in two modules are two published names.
//!  A field or a method is keyed by the type it belongs to,
//!  `<module path> field <Type>::<field>`,
//!  because a member is nameable through its type.
//!
//! What this reading cannot see, and why:
//!  - A `pub` field inside a tuple-like struct's parentheses
//!    (`pub struct PlanError(pub String);`, and five more fields written one to a line).
//!    Such a field is reached as `.0`,
//!    and this scan keys a member by the name the declaration writes,
//!    which a paren body does not write.
//!  - A member of a type published only through a `pub use` facade:
//!    the type's own `pub` declaration sits in a module no `pub mod` chain reaches,
//!    and this scan reads a type as published only where such a chain reaches its declaration.
//!  - A `pub trait`'s methods and a `pub enum`'s variants:
//!    both are published with no `pub` of their own,
//!    so a new one changes the published surface without a line here.
//!    The `trait` and the `enum` take a line.
//!  - An item inside an inline `mod name { ... }` body:
//!    this crate declares no inline module that is `pub`.
//!  - A `pub use ...::*` over a module no `pub mod` chain reaches:
//!    the glob is listed as `pub use <module>::*` and carries every name that module holds.
//!    Every glob of this crate today names a module the chain reaches, so it carries nothing.
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
use std::collections::{
    BTreeMap,
    BTreeSet,
};
use std::path::Path;

/// The kinds a bare `pub` declaration can name,
/// which are the item kinds this crate declares.
const ITEM_KINDS: &[&str] = &[
    "fn", "struct", "enum", "union", "trait", "const", "static", "type",
];

/// The words a declaration may put between its `pub` and its kind,
/// which qualify the item and never name it.
const QUALIFIERS: &[&str] = &["async", "unsafe", "extern", "default"];

/// The kinds whose body publishes a field.
const KINDS_WITH_FIELDS: &[&str] = &["struct", "union"];

/// What:
///  `pub(crate) struct NameableItem` is one item another crate can name:
///  the entry the list beside this file writes for it,
///  and the file it is declared in.
/// Why:
///  A finding names the entry and the file a reader opens to narrow the item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NameableItem = { key: string; file: string }
/// ```
pub(crate) struct NameableItem {
    /// The list's entry, `<module path> <kind> <name>`.
    pub(crate) key: String,
    /// The file that declares it,
    /// relative to the source root.
    pub(crate) file: String,
}

/// What:
///  `pub(crate) struct CarriedName` is one name a `pub use` statement hands out of a module
///  no `pub mod` chain reaches:
///  the entry the list writes for it,
///  and the file holding the statement.
/// Why:
///  A `pub use` publishes one name per name it lists,
///  so a second name added to a listed statement is a second published name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarriedName = { key: string; file: string }
/// ```
pub(crate) struct CarriedName {
    /// The list's entry, `<module path> pub use <target>::<name>`.
    pub(crate) key: String,
    /// The file holding the `pub use` statement.
    pub(crate) file: String,
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
    /// Every `pub` item the `pub mod` chain reaches,
    /// and every `pub` member of a type it publishes.
    pub(crate) items: Vec<NameableItem>,
    /// Every name a `pub use` carries out of a module that chain does not reach.
    pub(crate) carried: Vec<CarriedName>,
}

/// What:
///  `fn words_of(line: &str) -> Vec<&str>` reads a line's identifier runs in order,
///  skipping a string literal whole.
/// Why:
///  A declaration is read by the words it writes:
///  the qualifiers before a kind, the kind, and the name after it.
///  A word inside a string literal is no word of the declaration,
///  and `extern "C"` puts one there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordsOf = (line: string): string[] => line.match(/[A-Za-z0-9_]+|"(?:\\.|[^"])*"/g) ?? []
/// ```
fn words_of(line: &str) -> Vec<&str> {
    let mut words: Vec<&str> = Vec::new();
    let bytes: &[u8] = line.as_bytes();
    let mut index: usize = 0;
    while index < bytes.len() {
        let character: u8 = bytes[index];
        if character == b'"' {
            index += 1;
            while index < bytes.len() && bytes[index] != b'"' {
                if bytes[index] == b'\\' {
                    index += 1;
                }
                index += 1;
            }
            index += 1;
            continue;
        }
        if character.is_ascii_alphanumeric() || character == b'_' {
            let start: usize = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            words.push(&line[start..index]);
            continue;
        }
        index += 1;
    }
    return words;
}

/// What:
///  `fn declaration_after_pub(rest: &str) -> Option<(String, String)>` answers the kind and the name
///  a declaration writes after its `pub `,
///  whatever qualifiers stand in between.
/// Why:
///  `pub async fn`, `pub const fn`, `pub unsafe fn` and `pub extern "C" fn` publish a function,
///  and a reader that took the word after `pub` for the kind read none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declarationAfterPub(rest: string): [string, string] | undefined
/// ```
fn declaration_after_pub(rest: &str) -> Option<(String, String)> {
    let words: Vec<&str> = words_of(rest);
    let first: &&str = words.first()?;
    if *first == "const" {
        // What: `const` names a kind of its own unless `fn` follows it, where it qualifies one.
        // Why: `pub const fn` publishes a function and `pub const NAME: T` publishes a constant.
        if words.get(1) == Some(&"fn") {
            return Some((String::from("fn"), String::from(*words.get(2)?)));
        }
        return Some((String::from("const"), String::from(*words.get(1)?)));
    }
    let mut index: usize = 0;
    while index < words.len() && QUALIFIERS.contains(&words[index]) {
        index += 1;
    }
    let kind: &str = *words.get(index)?;
    if !ITEM_KINDS.contains(&kind) {
        return None;
    }
    let mut name_index: usize = index + 1;
    if kind == "static" && words.get(name_index) == Some(&"mut") {
        name_index += 1;
    }
    return Some((String::from(kind), String::from(*words.get(name_index)?)));
}

/// What:
///  `fn is_a_test_only_attribute(attribute: &str) -> bool` answers whether an attribute keeps the
///  declaration below it out of a release build.
/// Why:
///  A `#[cfg(test)]` item is compiled for the tests alone and publishes nothing,
///  so the list must not hold a line for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isATestOnlyAttribute = (attribute: string): boolean =>
///   attribute === '#[cfg(test)]' || (attribute.startsWith('#[cfg(all(') && attribute.includes('test'))
/// ```
fn is_a_test_only_attribute(attribute: &str) -> bool {
    let Some(predicate) = attribute
        .strip_prefix("#[cfg(")
        .and_then(|rest| return rest.strip_suffix(")]"))
    else {
        return false;
    };
    return predicate == "test" || (predicate.starts_with("all(") && predicate.contains("test"));
}

/// What:
///  `fn first_column_declarations(text: &str) -> Vec<(String, String)>` lists the `(kind, name)` of every
///  declaration the text writes as a bare `pub` item at the first column,
///  and of every `#[macro_export] macro_rules!` name.
/// Why:
///  Exactly `pub` publishes a name to another crate:
/// `pub(crate)` or `pub(super)` or `pub(in path)` do not,
///  and a `bare pub` written one level in is a field or a method, which `members_in` reads.
///  A `#[cfg(test)]` attribute above a declaration keeps it out of the release build,
///  so it publishes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstColumnDeclarations(text: string): [string, string][]
/// ```
fn first_column_declarations(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut attributes: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        if line.starts_with("#[") {
            attributes.push(line.trim_end());
            continue;
        }
        let exported: bool = attributes.iter().any(|attribute| {
            return *attribute == "#[macro_export]";
        });
        let only_for_tests: bool = attributes.iter().any(|attribute| {
            return is_a_test_only_attribute(attribute);
        });
        attributes.clear();
        if only_for_tests {
            continue;
        }
        if exported {
            let exported_name: Option<&str> = line.strip_prefix("macro_rules! ").and_then(|rest| {
                return rest
                    .split(|character: char| return character == ' ' || character == '{')
                    .next();
            });
            if let Some(name) = exported_name {
                found.push((String::from("macro_rules"), name.to_owned()));
            }
            continue;
        }
        let Some(rest) = line.strip_prefix("pub ") else {
            continue;
        };
        if let Some(declared) = declaration_after_pub(rest) {
            found.push(declared);
        }
    }
    return found;
}

/// What:
///  `fn self_type_of(line: &str) -> Option<String>` answers the type an `impl` line implements for:
///  the name after `for` when the line writes one,
///  and the name after the `impl`'s own generic parameters otherwise.
/// Why:
///  A `pub fn` inside an `impl` is nameable through that type,
///  so the reader keys it by the type's name and asks whether the type is published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selfTypeOf(line: string): string | undefined
/// ```
fn self_type_of(line: &str) -> Option<String> {
    let rest: &str = line.strip_prefix("impl")?.trim_start();
    let mut offset: usize = 0;
    if rest.starts_with('<') {
        let mut depth: i32 = 0;
        for (position, character) in rest.char_indices() {
            if character == '<' {
                depth += 1;
            }
            if character == '>' {
                depth -= 1;
                if depth == 0 {
                    offset = position + 1;
                    break;
                }
            }
        }
    }
    let mut tail: &str = rest[offset..].trim_start();
    if let Some((_, after)) = tail.split_once(" for ") {
        tail = after.trim_start();
    }
    let tail: &str = tail.split('{').next()?.trim();
    let tail: &str = tail.split('<').next()?.trim();
    let name: &str = tail.rsplit("::").next()?.trim();
    if name.is_empty() {
        return None;
    }
    return Some(name.to_owned());
}

/// What:
///  `fn brace_delta(text: &str) -> i32` counts the braces a line opens,
///  once its comment and its string and character literals are out of it.
/// Why:
///  A brace body ends where its braces balance,
///  and a brace inside a string or a comment opens nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const braceDelta = (text: string): number => [...code(text)].reduce((n, c) => n + (c === '{') - (c === '}'), 0)
/// ```
fn brace_delta(text: &str) -> i32 {
    let code: &str = text.split("//").next().unwrap_or(text);
    let mut delta: i32 = 0;
    let mut characters = code.chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                while let Some(inside) = characters.next() {
                    if inside == '\\' {
                        characters.next();
                        continue;
                    }
                    if inside == '"' {
                        break;
                    }
                }
            }
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }
    return delta;
}

/// What:
///  `fn attribute_only_for_tests(line: &str) -> bool` answers whether an indented attribute line keeps the
///  member below it out of a release build.
/// Why:
///  A `#[cfg(test)]` field of a published type exists in the test build alone,
///  and a line in the list for it would name something no consumer can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributeOnlyForTests = (line: string): boolean =>
///   line.trimStart().startsWith('#[') && isATestOnlyAttribute(line.trim())
/// ```
fn attribute_only_for_tests(line: &str) -> bool {
    return line.trim_start().starts_with("#[") && is_a_test_only_attribute(line.trim());
}

/// What:
///  `fn member_of_body(trimmed: &str, methods: bool) -> Option<(String, String)>` answers the kind and the
///  name a member line writes:
///  `field` and the name before its `:` in a struct's body,
///  and `fn` and the name after its `fn` in an `impl`'s body.
/// Why:
///  A field and a method are both written `pub` one level in,
///  and only the body they stand in says which they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const memberOfBody = (trimmed: string, methods: boolean): [string, string] | undefined
/// ```
fn member_of_body(
    trimmed: &str,
    methods: bool,
) -> Option<(String, String)> {
    let rest: &str = trimmed.strip_prefix("pub ")?;
    if !methods {
        let name: &str = rest.split(':').next()?.trim();
        let named: bool = !name.is_empty()
            && name
                .chars()
                .all(|character| return character.is_alphanumeric() || character == '_');
        if !named {
            return None;
        }
        return Some((String::from("field"), name.to_owned()));
    }
    let (kind, name) = declaration_after_pub(rest)?;
    if kind != "fn" {
        return None;
    }
    return Some((kind, name));
}

/// What:
///  `fn members_of(text: &str, published_types: &BTreeSet<String>) -> Vec<(String, String)>` lists the
///  `(kind, member)` of every bare `pub` field and method of a type `published_types` holds.
/// Why:
///  A member is nameable through its type,
///  so a `pub` field of a published struct and a `pub fn` of an `impl` of one publish a name too,
///  and a split that writes one publishes without touching any line the first-column scan reads.
/// Gotcha:
///  The scan reads a member one level inside the body its declaration opened,
///  so a member of a tuple-like struct's parentheses is read by nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function membersOf(text: string, publishedTypes: Set<string>): [string, string][]
/// ```
fn members_of(
    text: &str,
    published_types: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut body: Option<(String, bool)> = None;
    let mut remaining: i32 = 0;
    let mut only_for_tests: bool = false;
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let at_first_column: bool = !line.is_empty() && !line.starts_with(char::is_whitespace);
        if at_first_column {
            body = None;
            only_for_tests = false;
            if line.starts_with('#') {
                continue;
            }
            let declared: Option<(String, String)> = line
                .strip_prefix("pub ")
                .and_then(|rest| return declaration_after_pub(rest));
            let opened: Option<(String, bool)> = match declared {
                Some((kind, name)) if KINDS_WITH_FIELDS.contains(&kind.as_str()) => {
                    Some((name, false))
                }
                _ => self_type_of(line).map(|owner| return (owner, true)),
            };
            if let (Some((owner, methods)), true) = (opened, line.contains('{')) {
                if published_types.contains(owner.as_str()) {
                    body = Some((owner, methods));
                    remaining = brace_delta(line);
                    if remaining <= 0 {
                        body = None;
                    }
                }
            }
            continue;
        }
        if attribute_only_for_tests(line) {
            only_for_tests = true;
            continue;
        }
        let Some((owner, methods)) = body.as_ref() else {
            continue;
        };
        if !only_for_tests {
            if let Some((kind, member)) = member_of_body(line.trim(), *methods) {
                found.push((kind, format!("{owner}::{member}")));
            }
        }
        only_for_tests = false;
        remaining += brace_delta(line);
        if remaining <= 0 {
            body = None;
        }
    }
    return found;
}

/// What:
///  `fn use_statements(text: &str) -> Vec<String>` reads every whole `pub use` statement,
///  however many lines it spans,
///  as the text between `pub use ` and its `;`.
/// Why:
///  This crate writes most of its re-exports over several lines,
///  and a reader that kept only the statements a line ends on read none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const useStatements = (text: string): string[] =>
///   text.split('pub use ').slice(1).map((rest) => rest.slice(0, rest.indexOf(';')))
/// ```
fn use_statements(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut collected: Option<String> = None;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        let Some(statement) = collected.as_mut() else {
            if trimmed.starts_with("//") {
                continue;
            }
            let Some(rest) = trimmed.strip_prefix("pub use ") else {
                continue;
            };
            collected = Some(rest.to_owned());
            if rest.ends_with(';') {
                let finished: String = collected.take().expect("the statement just collected");
                found.push(finished[..finished.len() - 1].trim().to_owned());
            }
            continue;
        };
        statement.push(' ');
        statement.push_str(trimmed);
        if trimmed.ends_with(';') {
            let finished: String = collected.take().expect("the statement just collected");
            found.push(finished[..finished.len() - 1].trim().to_owned());
        }
    }
    return found;
}

/// What:
///  `fn carried_of_statement(statement: &str) -> Vec<(String, String)>` lists the `(module, name)` pairs one
///  `pub use` statement names,
///  with every `as` rename applied and the module written as the statement writes it.
/// Why:
///  The name a statement publishes is the name after its `as`, not the one it started from,
///  and a glob publishes every name of the module it names,
///  which is one entry with no name of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carriedOfStatement(statement: string): [string, string][]
/// ```
fn carried_of_statement(statement: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut prefixes: Vec<String> = vec![String::new()];
    let mut piece: String = String::new();
    let mut depth: usize = 0;
    let mut collected: Vec<(String, String)> = Vec::new();
    for character in statement.chars() {
        match character {
            '{' => {
                let parent: &String = prefixes.last().expect("the outermost prefix");
                prefixes.push(format!("{parent}{}", piece.trim()));
                piece.clear();
                depth += 1;
            }
            '}' => {
                collected.push((
                    prefixes.last().expect("the innermost prefix").clone(),
                    piece.clone(),
                ));
                prefixes.pop();
                piece.clear();
                depth = depth.saturating_sub(1);
            }
            ',' if depth > 0 => {
                collected.push((
                    prefixes.last().expect("the innermost prefix").clone(),
                    piece.clone(),
                ));
                piece.clear();
            }
            other => piece.push(other),
        }
    }
    collected.push((String::new(), piece));
    for (prefix, raw) in collected {
        let entry: String = format!("{prefix}{}", raw.trim());
        let entry: &str = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (path, name): (&str, &str) = match entry.split_once(" as ") {
            Some((path, alias)) => (path.trim(), alias.trim()),
            None => (entry, entry.rsplit("::").next().unwrap_or(entry).trim()),
        };
        let (module, last): (&str, &str) = match path.rsplit_once("::") {
            Some((module, last)) => (module, last),
            None => continue,
        };
        let published: &str = name_of(last, name);
        if published.is_empty() {
            continue;
        }
        found.push((module.to_owned(), published.to_owned()));
    }
    return found;
}

/// What:
///  `fn name_of(last: &str, named: &str) -> &str` answers the name a path segment publishes.
/// Why:
///  A `{self}` entry re-exports the module itself,
///  which publishes the module's own name rather than a name inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOf = (last: string, named: string): string => (last === 'self' ? '*' : named)
/// ```
fn name_of<'a>(
    last: &'a str,
    named: &'a str,
) -> &'a str {
    if last == "self" || named == "self" {
        return "*";
    }
    return named;
}

/// What:
///  `fn carried_names(text: &str) -> Vec<(String, String)>` lists every `(module, name)` pair the text's
///  `pub use` statements carry,
///  however many lines each statement spans.
/// Why:
///  The caller decides which of them leave a module the `pub mod` chain does not reach;
///  this reads the text once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriedNames = (text: string): [string, string][] =>
///   useStatements(text).flatMap(carriedOfStatement)
/// ```
fn carried_names(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for statement in use_statements(text) {
        found.extend(carried_of_statement(statement.as_str()));
    }
    return found;
}

/// What:
///  `fn module_key(chain: &str) -> String` is the module path a list entry starts with,
///  `crate` for the library root.
/// Why:
///  Every entry starts with where the name is published,
///  so the root needs a word of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moduleKey = (chain: string): string => chain || 'crate'
/// ```
fn module_key(chain: &str) -> String {
    if chain.is_empty() {
        return String::from("crate");
    }
    return chain.to_owned();
}

/// What:
///  `pub(crate) fn surface_of(root: &Path) -> Result<Surface, String>` reads the whole published surface of the
///  library root under `root`.
/// Why:
///  One reading serves the check and the list beside it,
/// and the limits of the reading are stated in this file's own doc rather than in each caller.
/// Gotcha:
///  The types a public module publishes are read before any member is,
///  because a member publishes a name only through its type;
///  so every file of the tree is read once for its declarations and once for its members.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function surfaceOf(root: string): Surface  // throws Error
/// ```
pub(crate) fn surface_of(root: &Path) -> Result<Surface, String> {
    let modules: BTreeMap<String, ModuleOfTheTree> = read_the_tree(root)?;
    let mut published_types: BTreeSet<String> = BTreeSet::new();
    for (chain, module) in &modules {
        if !module.public || module.test_only {
            continue;
        }
        let text: String = read_module(root, module)?;
        for (kind, name) in first_column_declarations(text.as_str()) {
            if KINDS_WITH_FIELDS.contains(&kind.as_str()) {
                published_types.insert(name);
            }
        }
        let _ = chain;
    }
    let mut items: Vec<NameableItem> = Vec::new();
    let mut carried: Vec<CarriedName> = Vec::new();
    for (chain, module) in &modules {
        if module.test_only {
            continue;
        }
        let text: String = read_module(root, module)?;
        let written: String = module_key(chain.as_str());
        for (kind, member) in members_of(text.as_str(), &published_types) {
            items.push(NameableItem {
                key: format!("{written} {kind} {member}"),
                file: module.file.clone(),
            });
        }
        if !module.public {
            continue;
        }
        for (kind, name) in first_column_declarations(text.as_str()) {
            items.push(NameableItem {
                key: format!("{written} {kind} {name}"),
                file: module.file.clone(),
            });
        }
        for (target, name) in carried_names(text.as_str()) {
            let reached: Option<String> =
                chain_the_target_reaches(&modules, chain, target.as_str());
            let public: bool = reached
                .and_then(|found| return modules.get(found.as_str()))
                .is_some_and(|found| return found.public && !found.test_only);
            if !public {
                carried.push(CarriedName {
                    key: format!("{written} pub use {target}::{name}"),
                    file: module.file.clone(),
                });
            }
        }
    }
    return Ok(Surface {
        items,
        carried,
    });
}

/// What:
///  `fn read_module(root: &Path, module: &ModuleOfTheTree) -> Result<String, String>` reads one module's file.
/// Why:
///  A scan that cannot read a file must stop rather than read a surface short by that file's names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readModule(root: string, module: ModuleOfTheTree): string  // throws Error
/// ```
fn read_module(
    root: &Path,
    module: &ModuleOfTheTree,
) -> Result<String, String> {
    return std::fs::read_to_string(root.join(module.file.as_str()))
        .map_err(|error| return format!("cannot read {}: {error}", module.file));
}
