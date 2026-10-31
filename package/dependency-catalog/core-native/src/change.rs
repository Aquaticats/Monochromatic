//! What:     The change report: what a sync did (or, in check mode, would do) to each file.
//! Why:      Check mode must name why a file drifts, not just that it does; the same records explain a write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Change =
//!   | { kind: 'catalog-entry-added'; name: string }
//!   | { kind: 'catalog-floor-raised'; name: string; requirement: string }
//!   | ...;
//! ```

use std::fmt;

/// What:     `pub enum Change { ... }` is one edit the sync makes to one file.
/// Why:      The variants cover every edit kind: adoption (added entry, raised floor), pruning, rewriting a literal
///           site, writing an internal path version, and the generated table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { kind: 'catalog-entry-added'; name: string } | ...;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// A literal version or git pin joined `[catalog]`.
    CatalogEntryAdded {
        /// Dependency name.
        name: String,
    },
    /// An existing `>=` floor in `[catalog]` rose to satisfy a literal.
    CatalogFloorRaised {
        /// Dependency name.
        name: String,
        /// The new requirement text.
        requirement: String,
    },
    /// An entry no crate uses was removed (`catalogPrune = true`).
    CatalogEntryPruned {
        /// Dotted path such as `catalog.anyhow`.
        path: String,
    },
    /// A literal use site became `workspace = true`.
    SiteInherited {
        /// Dotted location such as `dependencies.anyhow`.
        location: String,
    },
    /// An internal path dependency got or changed its exact `version`.
    PathVersionWritten {
        /// Dotted location such as `dependencies.lib`.
        location: String,
        /// The version written, including the leading `=`.
        version: String,
    },
    /// The generated `[workspace.dependencies]` table was created or its entries replaced.
    GeneratedTableWritten,
    /// The generated `[workspace.dependencies]` table was removed.
    GeneratedTableRemoved,
}

impl fmt::Display for Change {
    /// Prints one short human-readable line per change.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Change::CatalogEntryAdded { name } => write!(formatter, "adopt {name} into [catalog]"),
            Change::CatalogFloorRaised { name, requirement } => write!(formatter, "raise the [catalog] floor of {name} to {requirement}"),
            Change::CatalogEntryPruned { path } => write!(formatter, "prune the unused entry {path}"),
            Change::SiteInherited { location } => write!(formatter, "rewrite {location} to `workspace = true`"),
            Change::PathVersionWritten { location, version } => write!(formatter, "write version \"{version}\" on {location}"),
            Change::GeneratedTableWritten => write!(formatter, "write the generated [workspace.dependencies] table"),
            Change::GeneratedTableRemoved => write!(formatter, "remove the generated [workspace.dependencies] table"),
        };
    }
}
