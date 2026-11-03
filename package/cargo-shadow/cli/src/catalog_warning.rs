//! What:
//! Finds dependencies in a `Cargo.toml` that do not use the workspace catalog:
//! a literal version,
//! or a table with neither
//! `workspace = true`,
//! `path` nor `git`.
//! Why:
//! `cargo add` writes a literal version,
//! and the repository's rule is that every registry version lives in
//! `rust-catalog.toml` (<https://pnpm.io/catalogs> is the model:
//! `catalog:` references instead of ranges).
//! `mise run sync:files`
//! adopts the literal into the catalog,
//! so the warning names that command.
//! pnpm-shadow warns the same way about specifiers
//! that are not `catalog:`;
//! like it,
//! this never blocks cargo.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // catalog-warning.ts
//! export function literalDependencies(manifestText: string): { table: string; name: string }[]
//! ```

/// What:
/// `use std::io::Write;` imports `Write` from the standard library so this file can name it unqualified.
/// Why:
/// `Write` is used by `warn_about_manifest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `warn_about_manifest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use toml_edit::{DocumentMut, Item, Table};` imports `DocumentMut`,
/// `Item`,
/// `Table` from the external crate `toml_edit` so this file can name them unqualified.
/// Why:
/// `DocumentMut` is used by `literal_dependencies`;
/// `Item` is used by `is_catalog_or_local`;
/// `Table` is used by `collect`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DocumentMut, Item, Table } from 'toml_edit';
/// ```
use toml_edit::{DocumentMut, Item, Table};

/// The dependency tables of a package manifest.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// One dependency that does not use the workspace catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Literal {
    /// The table it is in,
    /// such as `dependencies` or `target.'cfg(unix)'.dependencies`.
    pub table: String,
    /// The dependency name as written.
    pub name: String,
}

/// Whether one dependency value uses the catalog or needs none.
fn is_catalog_or_local(item: &Item) -> bool {
    if let Some(table) = item.as_table_like() {
        return table.get("workspace").and_then(Item::as_bool) == Some(true)
            || table.contains_key("path")
            || table.contains_key("git");
    }
    return false;
}

/// Collects the offenders of one dependency table.
fn collect(table_name: &str, table: &Table, found: &mut Vec<Literal>) {
    for (name, item) in table.iter() {
        if !is_catalog_or_local(item) {
            found.push(Literal { table: table_name.to_string(), name: name.to_string() });
        }
    }
}

/// What:
/// `pub fn literal_dependencies(manifest_text: &str) -> Vec<Literal>` lists the offenders in the package's own dependency
/// tables,
/// including `[target.<cfg>.*dependencies]`;
/// `[workspace.dependencies]` is the catalog's generated table and is
/// skipped.
/// Why:
/// Text that is not valid TOML yields nothing:
/// cargo reports that itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function literalDependencies(text: string): Literal[]
/// ```
pub fn literal_dependencies(manifest_text: &str) -> Vec<Literal> {
    let document: DocumentMut = match manifest_text.parse::<DocumentMut>() {
        Ok(found) => found,
        Err(_) => return Vec::new(),
    };
    let mut found: Vec<Literal> = Vec::new();
    for name in DEPENDENCY_TABLES {
        if let Some(table) = document.get(name).and_then(Item::as_table) {
            collect(name, table, &mut found);
        }
    }
    if let Some(targets) = document.get("target").and_then(Item::as_table) {
        for (target, target_item) in targets.iter() {
            for name in DEPENDENCY_TABLES {
                if let Some(table) = target_item.get(name).and_then(Item::as_table) {
                    collect(&format!("target.'{target}'.{name}"), table, &mut found);
                }
            }
        }
    }
    return found;
}

/// Writes the warning for a manifest to `sink`;
/// nothing when every dependency uses the catalog or the file is unreadable.
pub fn warn_about_manifest(manifest: &Path, sink: &mut dyn Write) {
    let text: String = match std::fs::read_to_string(manifest) {
        Ok(found) => found,
        Err(_) => return,
    };
    for literal in literal_dependencies(&text) {
        let line: String = format!(
            "cargo-shadow: warning: {}: [{}] {} does not use the workspace catalog; run `mise run sync:files` to move its version into rust-catalog.toml and write `{} = {{ workspace = true }}`\n",
            manifest.display(),
            literal.table,
            literal.name,
            literal.name
        );
        let _ = sink.write_all(line.as_bytes());
    }
}
