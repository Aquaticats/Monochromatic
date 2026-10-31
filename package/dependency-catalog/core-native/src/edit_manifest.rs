//! What:     Format-preserving edits that bring one crate manifest in line with the catalog: literal use sites
//!           become `workspace = true`, internal path dependencies get an exact `version`, and the generated
//!           `[workspace.dependencies]` table holds exactly the resolved entries.
//! Why:      All edits go through the parsed `toml_edit::DocumentMut`, so every byte outside the edited entries is
//!           printed back unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // edit-manifest.ts
//! export function inheritSplices(...): Splice[]
//! export function pathVersionSplices(...): Splice[]
//! export function generatedTableSplices(...): Splice[]
//! ```

use toml_edit::{DocumentMut, InlineTable, Item, Key, RawString, Table, TableLike, Value};

use crate::edit_catalog::{end_position, replace_string};
use crate::quote::quote;
use crate::render::{GENERATED_TABLE_COMMENT, parse_value, render_generated_value};
use crate::site::GeneratedEntry;

/// Keys a literal carries that move into the catalog when the site becomes `workspace = true`.
const MOVED_KEYS: [&str; 4] = ["git", "package", "rev", "version"];

/// What:     `pub fn item_mut<'a>(...)` walks a key path of tables and returns the entry named `name` in the last
///           one, mutably.
/// Why:      Sites are recorded as `(table path, name)`, and edits need the live item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// table.reduce((current, segment) => current[segment], manifest)[name]
/// ```
pub fn item_mut<'a>(root: &'a mut dyn TableLike, table: &[String], name: &str) -> Option<&'a mut Item> {
    let mut current: &'a mut dyn TableLike = root;
    for segment in table {
        current = current.get_mut(segment)?.as_table_like_mut()?;
    }
    return current.get_mut(name);
}

/// Text of an inline-table member without its surrounding whitespace, for example `features = ["derive"]`.
fn member_text(key: &Key, value: &Value) -> String {
    let mut bare: Value = value.clone();
    bare.decor_mut().clear();
    return format!("{} = {}", key.display_repr(), bare);
}

/// What:     `fn inherit_inline(table: &toml_edit::InlineTable) -> Value` builds `{ workspace = true, <kept members> }`
///           from an inline-table dependency, dropping `git`, `package`, `rev` and `version`.
/// Why:      The kept members (`features`, `optional`, `default-features`) are copied as text, so their own
///           formatting survives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `{ ${['workspace = true', ...kept].join(', ')} }`
/// ```
fn inherit_inline(table: &toml_edit::InlineTable) -> Value {
    let mut parts: Vec<String> = vec!["workspace = true".to_string()];
    for (name, value) in table.iter() {
        if MOVED_KEYS.contains(&name) {
            continue;
        }
        if let Some((key, _)) = table.get_key_value(name) {
            parts.push(member_text(key, value));
        }
    }
    return parse_value(&format!("{{ {} }}", parts.join(", ")));
}

/// Rewrites a section-form or dotted dependency: the first moved key becomes `workspace = true` in place, later
/// moved keys are dropped.
fn inherit_table(table: &mut Table) {
    let names: Vec<String> = table.iter().map(|pair| return pair.0.to_string()).collect();
    let mut rebuilt: Vec<(Key, Item)> = Vec::new();
    let mut replaced: bool = false;
    for name in &names {
        let Some((key, item)) = table.remove_entry(name) else {
            continue;
        };
        if !MOVED_KEYS.contains(&name.as_str()) {
            rebuilt.push((key, item));
            continue;
        }
        if replaced {
            continue;
        }
        replaced = true;
        let mut workspace_key: Key = Key::new("workspace");
        *workspace_key.leaf_decor_mut() = key.leaf_decor().clone();
        let mut value: Value = Value::from(true);
        if let Item::Value(old) = &item {
            *value.decor_mut() = old.decor().clone();
        }
        rebuilt.push((workspace_key, Item::Value(value)));
    }
    for (key, item) in rebuilt {
        table.insert_formatted(&key, item);
    }
}

/// What:     `pub fn inherit_site(item: &mut Item)` turns a literal dependency into `workspace = true`, keeping its
///           `features`, `optional` and `default-features`.
/// Why:      Once the version lives in the catalog, the use site only says what is crate-local.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function inheritSplices(args: { source; located; site }): Splice[]
/// ```
pub fn inherit_site(item: &mut Item) {
    match item {
        Item::Value(value) => {
            let decor: toml_edit::Decor = value.decor().clone();
            let mut replacement: Option<Value> = None;
            if value.is_str() {
                replacement = Some(parse_value("{ workspace = true }"));
            } else if let Value::InlineTable(table) = &*value {
                replacement = Some(inherit_inline(table));
            }
            if let Some(mut new_value) = replacement {
                *new_value.decor_mut() = decor;
                *value = new_value;
            }
        }
        Item::Table(table) => inherit_table(table),
        _ => {}
    }
}

/// What:     `pub fn set_path_version(item: &mut Item, version: &str) -> bool` makes an internal path dependency
///           carry `version = "=<version>"`, the counterpart of `workspace:*`. Returns whether anything changed.
/// Why:      A dependency with no members at all is left alone, as the TypeScript tool did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pathVersionSplices(args: { source; located; site; version }): Splice[]
/// ```
pub fn set_path_version(item: &mut Item, version: &str) -> bool {
    let wanted: String = format!("={version}");
    let wanted_text: String = quote(&wanted);
    let Some(members) = item.as_table_like_mut() else {
        return false;
    };
    if members.is_empty() {
        return false;
    }
    if let Some(Item::Value(current)) = members.get_mut("version") {
        let mut bare: Value = current.clone();
        bare.decor_mut().clear();
        if bare.to_string() == wanted_text {
            return false;
        }
        replace_string(current, &wanted);
        return true;
    }
    if let Item::Value(Value::InlineTable(inline)) = item {
        append_inline_version(inline, &wanted_text);
        return true;
    }
    if let Some(table) = item.as_table_like_mut() {
        table.insert("version", Item::Value(parse_value(&wanted_text)));
    }
    return true;
}

/// What:     `fn append_inline_version(...)` adds `version = "<wanted>"` after the last member of an inline table
///           so it prints as `, version = "=0.1.1" }`.
/// Why:      The last member's trailing space (before `}`) moves to the new value; without that, `toml_edit` would
///           print a stray space before the comma.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// source.slice(0, last.range.end) + ', version = "=0.1.1"' + source.slice(last.range.end)
/// ```
fn append_inline_version(inline: &mut InlineTable, wanted_text: &str) {
    let last_name: Option<String> = inline.iter().last().map(|pair| return pair.0.to_string());
    let mut last_suffix: Option<RawString> = None;
    if let Some(name) = last_name
        && let Some(last) = inline.get_mut(&name)
    {
        last_suffix = last.decor().suffix().cloned();
        last.decor_mut().set_suffix("");
    }
    let mut key: Key = Key::new("version");
    key.leaf_decor_mut().set_prefix(" ");
    key.leaf_decor_mut().set_suffix(" ");
    let mut value: Value = parse_value(wanted_text);
    value.decor_mut().set_prefix(" ");
    if let Some(suffix) = last_suffix {
        value.decor_mut().set_suffix(suffix);
    }
    inline.insert_formatted(&key, value);
}

/// Builds the table of generated entries in name order.
fn generated_table(entries: &[(String, GeneratedEntry)]) -> Table {
    let mut sorted: Vec<&(String, GeneratedEntry)> = entries.iter().collect();
    sorted.sort_by(|left: &&(String, GeneratedEntry), right: &&(String, GeneratedEntry)| return left.0.cmp(&right.0));
    let mut table: Table = Table::new();
    for (name, generated) in sorted {
        table.insert(name, Item::Value(parse_value(&render_generated_value(generated))));
    }
    return table;
}

/// Printed form of a `[workspace.dependencies]` table without its leading comment, for comparison.
fn body_text(table: &Table) -> String {
    let mut copy: Table = table.clone();
    copy.decor_mut().clear();
    let mut workspace: Table = Table::new();
    workspace.set_implicit(true);
    workspace.insert("dependencies", Item::Table(copy));
    let mut document: DocumentMut = DocumentMut::new();
    document.insert("workspace", Item::Table(workspace));
    return document.to_string();
}

/// What the generated-table step did to the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableChange {
    /// The table already held exactly the wanted entries.
    Unchanged,
    /// The table was created or its entries replaced.
    Written,
    /// The table was removed because the crate uses no catalog entry.
    Removed,
}

/// What:     `pub fn sync_generated_table(...) -> TableChange` makes `[workspace.dependencies]` hold exactly
///           `entries`, sorted by name. A table that already prints the same entries is left untouched (including a
///           comment on its header line); a new table gets the generated-by comment above it.
/// Why:      `source_ends_with_newline` decides the blank-line separator, as the TypeScript appender did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function generatedTableSplices(args: { source; located; entries }): Splice[]
/// ```
pub fn sync_generated_table(document: &mut DocumentMut, source_ends_with_newline: bool, entries: &[(String, GeneratedEntry)]) -> TableChange {
    let wanted: Table = generated_table(entries);
    let end: isize = end_position(document);
    let root: &mut Table = document.as_table_mut();
    let existing: Option<Table> = match root.get("workspace").and_then(Item::as_table_like).and_then(|workspace| return workspace.get("dependencies")) {
        Some(Item::Table(table)) => Some(table.clone()),
        _ => None,
    };
    let workspace_is_table: bool = root.get("workspace").is_none_or(|item: &Item| return item.as_table().is_some());
    if !workspace_is_table {
        return TableChange::Unchanged;
    }
    let Some(current) = existing else {
        if entries.is_empty() {
            return TableChange::Unchanged;
        }
        let mut fresh: Table = wanted;
        fresh.set_position(Some(end));
        let separator: &str = if source_ends_with_newline { "\n" } else { "\n\n" };
        fresh.decor_mut().set_prefix(format!("{separator}{GENERATED_TABLE_COMMENT}\n"));
        if root.get("workspace").is_none() {
            let mut workspace: Table = Table::new();
            workspace.set_implicit(true);
            root.insert("workspace", Item::Table(workspace));
        }
        if let Some(workspace) = root.get_mut("workspace").and_then(Item::as_table_mut) {
            workspace.insert("dependencies", Item::Table(fresh));
        }
        return TableChange::Written;
    };
    let Some(workspace) = root.get_mut("workspace").and_then(Item::as_table_mut) else {
        return TableChange::Unchanged;
    };
    if entries.is_empty() {
        workspace.remove("dependencies");
        return TableChange::Removed;
    }
    let mut candidate: Table = wanted;
    if body_text(&candidate) == body_text(&current) {
        return TableChange::Unchanged;
    }
    *candidate.decor_mut() = current.decor().clone();
    candidate.set_position(current.position());
    workspace.insert("dependencies", Item::Table(candidate));
    return TableChange::Written;
}
