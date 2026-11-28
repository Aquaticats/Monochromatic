//! What:
//!  The names another crate can reach through a library root,
//!  read from the source text:
//!  every bare `pub` item whose module the `pub mod` chain from the library root reaches,
//!  every bare `pub` field,
//!  every method and every constant of a type that chain publishes,
//!  and every name a `pub use` publishes that the chain does not already reach:
//!  a name carried out of a module it does not reach,
//!  and a name a re-export spells differently from the name it re-exports.
//! Why:
//!  A `mod` line and a `use` line decide reachability in two different ways,
//!  and a reader of one file sees neither:
//!  the surface scan states both once,
//!  so a check of the published surface reads them from one place.
//! Gotcha:
//!  Every name is keyed by the module path that publishes it,
//!  `<module path> <kind> <name>`,
//!  so two items of one kind and name in two modules are two published names.
//!  A member is keyed by the type it belongs to,
//!  `<module path> field <Type>::<field>`,
//!  `<module path> fn <Type>::<method>` or `<module path> const <Type>::<NAME>`,
//!  because a member is nameable through its type.
//!
//! What this reading cannot see,
//! and why:
//!  - A `pub` field inside a tuple-like struct's parentheses
//!    (`pub struct PlanError(pub String);` and `pub PathBuf,` one to a line).
//!    Such a field is reached as `.0`,
//!    and this scan keys a member by the name the declaration writes,
//!    which a paren body does not write.
//!  - A member of a type whose only public path is a `pub use` facade is read,
//!    at the facade's path and under the name the facade gives it,
//!    from the module the statement names.
//!    Nothing of it is read where that statement names an inline `mod name { ... }` body
//!    (`conflict/discarded_store.rs` and `worktree/worktree_ladder.rs` re-export such a body,
//!    which exists only where the crate is built for another platform),
//!    where it names a module no `pub mod` chain and no glob of the crate root reaches,
//!    where two modules a root glob names both hold the path's next segment,
//!    or where the module it names declares no type of that name (a function or a constant takes its own line
//!    and has no members).
//!  - A `pub trait`'s methods and a `pub enum`'s variants:
//!    both are published with no `pub` of their own,
//!    so a new one changes the published surface without a line here.
//!    The `trait` and the `enum` take a line.
//!  - An item inside an inline `mod name { ... }` body:
//!    this crate declares no inline module that is `pub`.
//!  - A `pub use ...::*` over a module no `pub mod` chain reaches:
//!    the glob is listed as `pub use <module>::*` and carries every name that module holds.
//!    Every glob of this crate today names a module the chain reaches,
//!    so it carries nothing.
//!  - A second path to a name a `pub use` re-exports under the name it already has,
//!    where the target's own module is one the chain reaches:
//!    the name is published where its target is declared,
//!    so no entry names the path the re-export adds.
//!    A re-export that spells the name differently publishes a new name and takes a line.
//!  - A signature or a value that changes under a name that stays:
//!    the entry names the name and nothing of its shape or its value.
//!  - A name behind a feature gate, as `#[cfg(feature = "measurement-routes")] pub fn f`,
//!    counts as published, because a gate keeps an item out only where it holds only when testing.
//!    The build that turns the feature on publishes the name,
//!    and the list holds every such name the tree writes today.
//!  - Members are matched by a type's bare name and by nothing else,
//!    so a type declared `pub` in a module nothing publishes has its `pub` members read as published whenever a
//!    published type elsewhere writes the same name.
//!    Such a name takes a line a consumer does not need, and a control beside this file pins that it does.
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
use crate::source_scan_tests::declaration::is_test_gate;
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

/// The kinds an `impl` body publishes a name under,
/// which are the ones an inherent `impl` may write with a `pub` of its own.
const KINDS_AN_IMPL_PUBLISHES: &[&str] = &["fn", "const"];

/// The kinds whose name another crate can reach,
/// so that a `pub` method of an `impl` of one publishes a name through it.
const KINDS_THAT_OWN_MEMBERS: &[&str] = &["struct", "enum", "union", "trait", "type"];

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
    /// The list's entry,
    /// `<module path> <kind> <name>`.
    pub(crate) key: String,
    /// The file that declares it,
    /// relative to the source root.
    pub(crate) file: String,
}

/// What:
///  `pub(crate) struct CarriedName` is one name a `pub use` statement publishes
///  that the `pub mod` chain does not already reach:
///  a name carried out of a module the chain does not reach,
///  or a name the statement spells differently from the name it re-exports.
/// Why:
///  A `pub use` publishes one name per name it lists,
///  so a second name added to a listed statement is a second published name,
///  and an `as` rename publishes a name the target's own module does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarriedName = { key: string; file: string }
/// ```
pub(crate) struct CarriedName {
    /// The list's entry,
    /// `<module path> pub use <target>::<name>`.
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
    /// Every name a `pub use` publishes that the chain does not already reach.
    pub(crate) carried: Vec<CarriedName>,
}

/// What:
///  `fn words_of(line: &str) -> Vec<&str>` reads a line's identifier runs in order,
///  skipping a string literal whole.
/// Why:
///  A declaration is read by the words it writes:
///  the qualifiers before a kind,
/// the kind,
/// and the name after it.
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
///  `fn fn_follows(words: &[&str], from: usize) -> bool` answers whether `fn` is the next word after the
///  qualifiers that start at `from`.
/// Why:
///  `pub const fn f` publishes a function while `pub const NAME` publishes a constant,
///  and the qualifiers between the two may be more than one,
///  as `const unsafe` and `const extern "C"` are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fnFollows = (words: string[], from: number): boolean =>
///   words.slice(from).filter((w) => QUALIFIERS.includes(w))[0] === 'fn' || words[from] === 'fn'
/// ```
fn fn_follows(
    words: &[&str],
    from: usize,
) -> bool {
    let mut index: usize = from;
    while index < words.len() && QUALIFIERS.contains(&words[index]) {
        index += 1;
    }
    return words.get(index) == Some(&"fn");
}

/// What:
///  `fn declaration_after_pub(rest: &str) -> Option<(String, String)>` answers the kind and the name
///  a declaration writes after its `pub `,
///  whatever qualifiers stand in between.
/// Why:
///  `pub async fn`,
/// `pub const fn`,
/// `pub unsafe fn` and `pub extern "C" fn` publish a function,
///  and a reader that took the word after `pub` for the kind read none of them.
///  `const` is the one word that is both a kind and a qualifier,
///  so it names a kind only where no `fn` follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declarationAfterPub(rest: string): [string, string] | undefined
/// ```
fn declaration_after_pub(rest: &str) -> Option<(String, String)> {
    let words: Vec<&str> = words_of(rest);
    let mut index: usize = 0;
    while index < words.len() {
        let word: &str = words[index];
        // What: `const` names a kind of its own unless `fn` follows the qualifiers after it.
        // Why: `pub const NAME: T` publishes a constant and `pub const unsafe fn f` publishes a function.
        if word == "const" && !fn_follows(&words, index + 1) {
            return Some((String::from("const"), String::from(*words.get(index + 1)?)));
        }
        if !QUALIFIERS.contains(&word) && word != "const" {
            break;
        }
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
/// Gotcha:
///  A predicate that names `test` inside a negation (`all(not(test), unix)`) keeps the item out of the
///  test build alone,
///  so it publishes one;
///  the one reading of a test gate decides that for this scan and the `mod` line alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isATestOnlyAttribute = (attribute: string): boolean =>
///   attribute.startsWith('#[cfg(') && attribute.endsWith(')]') && isTestGate(attribute.slice(6, -2))
/// ```
fn is_a_test_only_attribute(attribute: &str) -> bool {
    let Some(predicate) = attribute
        .strip_prefix("#[cfg(")
        .and_then(|rest| return rest.strip_suffix(")]"))
    else {
        return false;
    };
    return is_test_gate(predicate);
}

/// What:
///  `fn first_column_declarations(text: &str) -> Vec<(String, String)>` lists the `(kind, name)` of every
///  declaration the text writes as a bare `pub` item at the first column,
///  and of every `#[macro_export] macro_rules!` name.
/// Why:
///  Exactly `pub` publishes a name to another crate:
/// `pub(crate)` or `pub(super)` or `pub(in path)` do not,
///  and a `bare pub` written one level in is a field or a method,
/// which `members_in` reads.
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
    let mut implemented: &str = rest[offset..].trim_start();
    if let Some((_, after)) = implemented.split_once(" for ") {
        implemented = after.trim_start();
    }
    let up_to_the_body: &str = implemented.split('{').next()?.trim();
    let without_generics: &str = up_to_the_body.split('<').next()?.trim();
    let name: &str = without_generics.rsplit("::").next()?.trim();
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
///  and the kind and the name of a declaration in an `impl`'s body,
///  which is a function or a constant.
/// Why:
///  A field,
///  a method and an associated constant are all written `pub` one level in,
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
    if !KINDS_AN_IMPL_PUBLISHES.contains(&kind.as_str()) {
        return None;
    }
    return Some((kind, name));
}

/// What:
///  `fn members_of(text: &str, published_types: &BTreeSet<String>) -> Vec<(String, String)>` lists the
///  `(kind, member)` of every bare `pub` field and every bare `pub` declaration of an `impl` of a type
///  `published_types` holds.
/// Why:
///  A member is nameable through its type,
///  so a `pub` field of a published struct,
///  a `pub fn` of an `impl` of one and a `pub const` of one publish a name too,
///  and a split that writes one publishes without touching any line the first-column scan reads.
/// Gotcha:
///  The scan reads a member one level inside the body its declaration opened,
///  so a member of a tuple-like struct's parentheses is read by nothing here.
///  A declaration whose body opens on a later line is held as the opening it still waits for,
///  because this crate writes the generics of a type or an `impl` over several lines.
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
    let mut opening: Option<(String, bool)> = None;
    let mut remaining: i32 = 0;
    let mut only_for_tests: bool = false;
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        // What: A declaration whose body has not opened yet takes this line as its header, even at the
        // first column, because a type's or an `impl`'s generics and `where` clause are written there.
        // Why: Only the line holding the body's `{` opens it, and a header that ends in `;` has no body.
        if let Some((owner, methods)) = opening.take() {
            remaining += brace_delta(line);
            if line.contains('{') {
                body = Some((owner, methods));
                if remaining <= 0 {
                    body = None;
                }
            } else if !(line.contains(';') || remaining < 0) {
                opening = Some((owner, methods));
            }
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
            let declared_body: Option<(String, bool)> = match declared {
                Some((kind, name)) if KINDS_WITH_FIELDS.contains(&kind.as_str()) => {
                    Some((name, false))
                }
                _ => self_type_of(line).map(|owner| return (owner, true)),
            };
            match declared_body {
                Some((owner, methods)) if published_types.contains(owner.as_str()) => {
                    if line.contains('{') {
                        remaining = brace_delta(line);
                        if remaining > 0 {
                            body = Some((owner, methods));
                        }
                    } else if !line.contains(';') {
                        // What: The declaration names its type but writes no `;` and no `{` yet, so its
                        // body waits for a later line.
                        // Why: `pub struct Unit;` writes a `;` and has no body at all.
                        remaining = 0;
                        opening = Some((owner, methods));
                    }
                }
                _ => {}
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
        let named: Option<(String, String)> = if only_for_tests {
            None
        } else {
            member_of_body(line.trim(), *methods)
        };
        if let Some((kind, member)) = named {
            found.push((kind, format!("{owner}::{member}")));
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
///  `struct CarriedOfAStatement` is one name a `pub use` statement publishes:
///  the module the statement names,
///  the name that module declares,
///  the name the statement publishes,
///  and whether those two differ.
/// Why:
///  A facade publishes a type at a path of its own,
///  so a member of that type is read where the type is declared and keyed by the name the facade gives it.
struct CarriedOfAStatement {
    /// The module path the statement writes before the name it publishes.
    module: String,
    /// The name the module the statement names declares.
    declared: String,
    /// The name the statement publishes,
    /// which is the name after an `as`.
    published: String,
    /// Whether the statement spells the name differently from the name it re-exports.
    renamed: bool,
}

/// What:
///  `fn carried_of_statement(statement: &str) -> Vec<CarriedOfAStatement>` lists one record per name one
///  `pub use` statement publishes,
///  with every `as` rename applied and the module written as the statement writes it.
/// Why:
///  The name a statement publishes is the name after its `as`,
/// not the one it started from,
///  so an `as` publishes a name its target's module does not hold;
///  and a glob publishes every name of the module it names,
///  which is one entry with no name of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carriedOfStatement(statement: string): CarriedOfAStatement[]
/// ```
fn carried_of_statement(statement: &str) -> Vec<CarriedOfAStatement> {
    let mut found: Vec<CarriedOfAStatement> = Vec::new();
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
        let whole: String = format!("{prefix}{}", raw.trim());
        let entry: &str = whole.trim();
        if entry.is_empty() {
            continue;
        }
        let (path, named): (&str, &str) = match entry.split_once(" as ") {
            Some((path, alias)) => (path.trim(), alias.trim()),
            None => (entry, entry.rsplit("::").next().unwrap_or(entry).trim()),
        };
        let (module, last): (&str, &str) = match path.rsplit_once("::") {
            Some((module, last)) => (module, last),
            None => continue,
        };
        let published: &str = name_of(last, named);
        if published.is_empty() {
            continue;
        }
        found.push(CarriedOfAStatement {
            module: module.to_owned(),
            declared: last.to_owned(),
            published: published.to_owned(),
            renamed: entry.contains(" as "),
        });
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
///  `fn carried_names(text: &str) -> Vec<CarriedOfAStatement>` lists every name the text's `pub use` statements
///  publish,
///  however many lines each statement spans.
/// Why:
///  The caller decides which of them the `pub mod` chain does not already reach;
///  this reads the text once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriedNames = (text: string): CarriedOfAStatement[] =>
///   useStatements(text).flatMap(carriedOfStatement)
/// ```
fn carried_names(text: &str) -> Vec<CarriedOfAStatement> {
    let mut found: Vec<CarriedOfAStatement> = Vec::new();
    for statement in use_statements(text) {
        found.extend(carried_of_statement(statement.as_str()));
    }
    return found;
}

/// What:
///  `fn glob_targets(text: &str) -> Vec<String>` lists the module paths a text's `pub use <module>::*` statements
///  name.
/// Why:
///  The library root re-exports each of its folders with a glob,
///  and a facade may write a path that names a module through one of them
///  (`crate::the_guarded_commands` for `command::the_guarded_commands`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const globTargets = (text: string): string[] =>
///   useStatements(text).filter((s) => s.endsWith('::*')).map((s) => s.slice(0, -3).trim())
/// ```
fn glob_targets(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for statement in use_statements(text) {
        let Some(target) = statement.strip_suffix("::*") else {
            continue;
        };
        found.push(target.trim().to_owned());
    }
    return found;
}

/// What:
///  `fn through_the_globs(modules, globbed, path) -> Option<String>` follows the rest of a root path from the
///  modules the root's globs name,
///  and answers the one chain every candidate agrees on.
/// Why:
///  A glob re-export is what makes such a path compile,
///  and a segment two globbed modules both hold is no answer this reading can give,
///  so it answers nothing rather than guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function throughTheGlobs(modules, globbed: string[], path: string): string | undefined
/// ```
fn through_the_globs(
    modules: &BTreeMap<String, ModuleOfTheTree>,
    globbed: &[String],
    path: &str,
) -> Option<String> {
    let mut candidates: Vec<String> = globbed.to_vec();
    for segment in path
        .split("::")
        .filter(|segment| return !segment.is_empty())
    {
        let mut next: Vec<String> = Vec::new();
        for chain in &candidates {
            let Some(child) = modules
                .get(chain.as_str())
                .and_then(|module| return module.children.get(segment))
            else {
                continue;
            };
            next.push(child.clone());
        }
        candidates = next;
    }
    if candidates.len() == 1 {
        return candidates.into_iter().next();
    }
    return None;
}

/// What:
///  `fn the_module_a_facade_names(modules, from, target, globbed) -> Option<String>` answers the chain of the
///  module a `pub use` statement names,
///  which is where the names it publishes are declared.
/// Why:
///  A type a facade publishes sits in a module no `pub mod` chain reaches,
///  so its declaration is found through the statement's own path;
///  and a `crate::` path the root holds as no module may still name one the root's globs re-export.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theModuleAFacadeNames(modules, from: string, target: string, globbed: string[]): string | undefined
/// ```
fn the_module_a_facade_names(
    modules: &BTreeMap<String, ModuleOfTheTree>,
    from: &str,
    target: &str,
    globbed: &[String],
) -> Option<String> {
    if let Some(found) = chain_the_target_reaches(modules, from, target) {
        return Some(found);
    }
    let rest: &str = target.strip_prefix("crate::")?;
    return through_the_globs(modules, globbed, rest);
}

/// What:
///  `fn the_members_a_facade_publishes(modules, root, from, carried, globbed)` lists the `pub` members of the type
///  a statement publishes,
///  each as its kind and the member's own name.
/// Why:
///  A consumer reaches the type's fields, methods and associated constants through the facade,
///  so they publish names of the facade's path even though the type's declaration sits in a module nothing
///  else reaches.
///  Only a declaration of the name the statement publishes is read,
///  so a function, a constant or another module's name of that spelling reads nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMembersAFacadePublishes(modules, root: string, from: string, carried: CarriedOfAStatement, globbed: string[]): [string, string][]  // throws Error
/// ```
fn the_members_a_facade_publishes(
    modules: &BTreeMap<String, ModuleOfTheTree>,
    root: &Path,
    from: &str,
    carried: &CarriedOfAStatement,
    globbed: &[String],
) -> Result<Vec<(String, String)>, String> {
    let Some(declaring) =
        the_module_a_facade_names(modules, from, carried.module.as_str(), globbed)
    else {
        return Ok(Vec::new());
    };
    let Some(module) = modules.get(declaring.as_str()) else {
        return Ok(Vec::new());
    };
    if module.test_only {
        return Ok(Vec::new());
    }
    let text: String = read_module(root, module)?;
    let declares_a_type: bool =
        first_column_declarations(text.as_str())
            .iter()
            .any(|(kind, name)| {
                return KINDS_THAT_OWN_MEMBERS.contains(&kind.as_str())
                    && *name == carried.declared;
            });
    if !declares_a_type {
        return Ok(Vec::new());
    }
    let mut published_types: BTreeSet<String> = BTreeSet::new();
    published_types.insert(carried.declared.clone());
    let mut found: Vec<(String, String)> = Vec::new();
    for (kind, member) in members_of(text.as_str(), &published_types) {
        let bare: &str = member
            .split_once("::")
            .map_or(member.as_str(), |(_owner, rest)| return rest);
        found.push((kind, bare.to_owned()));
    }
    return Ok(found);
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
///  so every file of the tree is read once for its declarations and once for its members,
///  and the file a facade names is read once more for the members that facade publishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function surfaceOf(root: string): Surface  // throws Error
/// ```
pub(crate) fn surface_of(root: &Path) -> Result<Surface, String> {
    let modules: BTreeMap<String, ModuleOfTheTree> = read_the_tree(root)?;
    let mut published_types: BTreeSet<String> = BTreeSet::new();
    let mut globbed: Vec<String> = Vec::new();
    for (chain, module) in &modules {
        if !module.public || module.test_only {
            continue;
        }
        let text: String = read_module(root, module)?;
        if chain.is_empty() {
            for target in glob_targets(text.as_str()) {
                let Some(found) = chain_the_target_reaches(&modules, chain, target.as_str()) else {
                    continue;
                };
                globbed.push(found);
            }
        }
        for (kind, name) in first_column_declarations(text.as_str()) {
            if KINDS_THAT_OWN_MEMBERS.contains(&kind.as_str()) {
                published_types.insert(name);
            }
        }
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
        for site in carried_names(text.as_str()) {
            let target: &str = site.module.as_str();
            let reached: Option<String> = chain_the_target_reaches(&modules, chain, target);
            let public: bool = reached
                .and_then(|found| return modules.get(found.as_str()))
                .is_some_and(|found| return found.public && !found.test_only);
            if !public || site.renamed {
                carried.push(CarriedName {
                    key: format!("{written} pub use {target}::{}", site.published),
                    file: module.file.clone(),
                });
                // What: the members a consumer reaches through this facade, keyed by the path and the name the
                //       facade gives the type.
                // Why: the statement is the only thing that says which type a consumer can name here, because
                //      the type's own declaration sits in a module the `pub mod` chain does not reach.
                for (kind, member) in the_members_a_facade_publishes(
                    &modules,
                    root,
                    chain,
                    &site,
                    globbed.as_slice(),
                )? {
                    items.push(NameableItem {
                        key: format!("{written} {kind} {}::{member}", site.published),
                        file: module.file.clone(),
                    });
                }
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
