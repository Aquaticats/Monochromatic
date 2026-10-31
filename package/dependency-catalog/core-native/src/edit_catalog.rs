//! What:
//! Format-preserving edits of `rust-catalog.toml`:
//! adopted entries,
//! raised floors and pruned entries.
//! Why:
//! Every edit goes through the parsed `toml_edit::DocumentMut`,
//! so comments,
//! spacing and key order outside
//! the edited entries come back byte for byte when the document is printed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // edit-catalog.ts
//! export function catalogSplices(args: { source; located; additions; floorRaises; prunes }): Splice[]
//! ```

/// Imports `toml_edit::{DocumentMut, Item, Key, Table, TableLike, Value}`.
use toml_edit::{DocumentMut, Item, Key, Table, TableLike, Value};

/// Imports `crate::adopt::AdoptionPlan`.
use crate::adopt::AdoptionPlan;
/// Imports `crate::quote::quote`.
use crate::quote::quote;
/// Imports `crate::render::{parse_value, render_entry_value}`.
use crate::render::{parse_value, render_entry_value};
/// Imports `crate::types::Entry`.
use crate::types::Entry;

/// What:
/// `pub fn replace_string(value: &mut Value, text: &str)` swaps a string value for a new quoted string,
/// keeping the old value's surrounding whitespace and trailing comment.
/// Why:
/// Raising a floor must not move or drop the comment that follows it on the same line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// source.slice(0, start) + JSON.stringify(text) + source.slice(end)
/// ```
pub fn replace_string(value: &mut Value, text: &str) {
    let decor: toml_edit::Decor = value.decor().clone();
    let mut replacement: Value = parse_value(&quote(text));
    *replacement.decor_mut() = decor;
    *value = replacement;
}

/// Raises the floor of one `[catalog]` entry,
/// whether written as a string or as a table with `version`.
fn raise_floor(table: &mut dyn TableLike, name: &str, requirement: &str) {
    let Some(item) = table.get_mut(name) else {
        return;
    };
    match item {
        Item::Value(Value::String(_)) => {
            if let Item::Value(value) = item {
                replace_string(value, requirement);
            }
        }
        _ => {
            let Some(members) = item.as_table_like_mut() else {
                return;
            };
            if let Some(Item::Value(value)) = members.get_mut("version")
                && value.is_str()
            {
                replace_string(value, requirement);
            }
        }
    }
}

/// Position after the last positioned table,
/// so a table created now prints at the end of the file.
pub fn end_position(document: &DocumentMut) -> isize {
    let mut last: isize = 0;
    let mut stack: Vec<&Table> = vec![document.as_table()];
    while let Some(table) = stack.pop() {
        if let Some(position) = table.position() {
            last = last.max(position);
        }
        for (_, item) in table.iter() {
            match item {
                Item::Table(child) => stack.push(child),
                Item::ArrayOfTables(children) => {
                    for child in children.iter() {
                        stack.push(child);
                    }
                }
                _ => {}
            }
        }
    }
    return last + 1;
}

/// Creates a missing `[catalog]` table at the end of the file,
/// one blank line after the previous text.
fn ensure_catalog_table(document: &mut DocumentMut, source_is_empty: bool) {
    if document.as_table().get("catalog").is_some() {
        return;
    }
    let position: isize = end_position(document);
    let mut table: Table = Table::new();
    table.set_position(Some(position));
    if !source_is_empty {
        table.decor_mut().set_prefix("\n");
    }
    document.as_table_mut().insert("catalog", Item::Table(table));
}

/// What:
/// `fn insert_sorted(table: &mut Table, name: &str, value: Value)` inserts a new entry before the first
/// existing entry whose name sorts after it (byte order),
/// else after the last.
/// Why:
/// The catalog is kept in name order;
/// an entry's leading comment lives in its key's decor,
/// so the new
/// entry lands above that comment block,
/// not between the comment and its entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const following = existing.find((pair) => pair.name > name); // insert before its leading comments, else after the last
/// ```
fn insert_sorted(table: &mut Table, name: &str, value: Value) {
    let names: Vec<String> = table.iter().map(|pair| return pair.0.to_string()).collect();
    let mut entries: Vec<(Key, Item)> = Vec::new();
    for existing in &names {
        if let Some(pair) = table.remove_entry(existing) {
            entries.push(pair);
        }
    }
    let mut index: usize = entries.len();
    for (position, (key, _)) in entries.iter().enumerate() {
        if key.get() > name {
            index = position;
            break;
        }
    }
    entries.insert(index, (Key::new(name), Item::Value(value)));
    for (key, item) in entries {
        table.insert_formatted(&key, item);
    }
}

/// What:
/// `pub fn edit_catalog(...)` applies adoption (new entries,
/// raised floors) and pruning to the catalog
/// document,
/// and returns the names it touched for the change report.
/// Why:
/// `prunes` are paths such as `["catalog", "anyhow"]` or `["catalogs", "pinned", "salsa"]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// applySplices({ source, splices: catalogSplices({ source, located, additions, floorRaises, prunes }) })
/// ```
pub fn edit_catalog(document: &mut DocumentMut, source: &str, plan: &AdoptionPlan, prunes: &[Vec<String>]) {
    if !plan.additions.is_empty() {
        ensure_catalog_table(document, source.is_empty());
        if let Some(Item::Table(table)) = document.as_table_mut().get_mut("catalog") {
            for (name, entry) in sorted_additions(&plan.additions) {
                insert_sorted(table, name, parse_value(&render_entry_value(entry)));
            }
        } else if let Some(table) = document.as_table_mut().get_mut("catalog").and_then(Item::as_table_like_mut) {
            for (name, entry) in sorted_additions(&plan.additions) {
                table.insert(name, Item::Value(parse_value(&render_entry_value(entry))));
            }
        }
    }
    if let Some(table) = document.as_table_mut().get_mut("catalog").and_then(Item::as_table_like_mut) {
        for (name, requirement) in &plan.floor_raises {
            raise_floor(table, name, requirement);
        }
    }
    for path in prunes {
        remove_path(document.as_table_mut(), path);
    }
}

/// Additions in name order,
/// the order TypeScript merged them at a shared insertion point.
fn sorted_additions(additions: &[(String, Entry)]) -> Vec<(&str, &Entry)> {
    let mut sorted: Vec<(&str, &Entry)> = additions.iter().map(|pair| return (pair.0.as_str(), &pair.1)).collect();
    sorted.sort_by(|left: &(&str, &Entry), right: &(&str, &Entry)| return left.0.cmp(right.0));
    return sorted;
}

/// Removes the entry at `path` (every segment but the last names a table).
fn remove_path(root: &mut Table, path: &[String]) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut current: &mut dyn TableLike = root;
    for segment in parents {
        let Some(next) = current.get_mut(segment).and_then(Item::as_table_like_mut) else {
            return;
        };
        current = next;
    }
    current.remove(last);
}
