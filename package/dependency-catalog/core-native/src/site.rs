//! What:     The use-site model: one dependency entry found in a crate manifest, and how it refers to its source.
//! Why:      Adoption, resolution and editing all start from the same list of sites, in the order Cargo reads them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // types.ts (inputs half)
//! export type UseSiteKind = 'catalog' | 'literal' | 'path';
//! export type UseSite = { manifestPath: string; table: string[]; name: string; kind: UseSiteKind; literal: RustCatalogEntry; path: string; disablesDefaultFeatures: boolean };
//! ```

use crate::types::Entry;

/// What:     `pub enum SiteKind { ... }` says how a dependency entry refers to its source: through the catalog
///           (`workspace = true`), through an internal crate `path`, or by a literal version or git pin to adopt.
/// Why:      The sync treats each kind differently: literals are adopted and rewritten, paths get an exact version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseSiteKind = 'catalog' | 'literal' | 'path';
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteKind {
    /// `workspace = true`, resolved through the catalog.
    Catalog,
    /// A registry version or git pin written in the manifest.
    Literal,
    /// An internal crate by `path`.
    Path,
}

/// What:     `pub struct UseSite { ... }` is one dependency entry found in a manifest.
/// Why:      `table` is the key path of the dependency table (`["target", "cfg(unix)", "dependencies"]`); `literal`
///           is meaningful only for `Literal` sites and `path` only for `Path` sites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseSite = { manifestPath: string; table: readonly string[]; name: string; kind: UseSiteKind; literal: RustCatalogEntry; path: string; disablesDefaultFeatures: boolean };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseSite {
    /// Manifest the entry is in.
    pub manifest_path: String,
    /// Key path of the dependency table.
    pub table: Vec<String>,
    /// Dependency key.
    pub name: String,
    /// How the entry refers to its source.
    pub kind: SiteKind,
    /// Literal requirement or git pin; meaningful only for `Literal` sites.
    pub literal: Entry,
    /// Relative path of an internal dependency; empty for other kinds.
    pub path: String,
    /// Whether the entry writes `default-features = false`.
    pub disables_default_features: bool,
}

impl UseSite {
    /// Dotted location for diagnostics, such as `target.cfg(unix).dependencies.libc`.
    pub fn location(&self) -> String {
        let mut parts: Vec<&str> = self.table.iter().map(String::as_str).collect();
        parts.push(&self.name);
        return parts.join(".");
    }
}

/// What:     `pub struct GeneratedEntry { ... }` is one entry of a crate's generated `[workspace.dependencies]`.
/// Why:      `default_features` is false when every site in the crate disables default features, which Cargo before
///           1.99 requires before it accepts `default-features = false` beside `workspace = true`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedEntry = { entry: RustCatalogEntry; defaultFeatures: boolean };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedEntry {
    /// The catalog entry the crate resolved.
    pub entry: Entry,
    /// Whether the crate keeps default features.
    pub default_features: bool,
}
