//! What:
//! The parsed `rust-catalog.toml` document:
//! entries,
//! ordered entry maps and supply-chain settings.
//! Why:
//! Every other module (parser,
//! adoption,
//! sync) passes these plain values around instead of toml_edit nodes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // types.ts
//! export type RustCatalogEntry = { kind: 'registry'; version: string; package?: string } | { kind: 'git'; git: string; rev: string; package?: string };
//! export type RustCatalogDocument = { catalog: Map<string, RustCatalogEntry>; catalogs: Map<string, Map<string, RustCatalogEntry>>; settings: RustCatalogSettings };
//! ```

/// What:
/// `pub enum Entry { ... }` is a sum type (a TS discriminated union):
/// a registry requirement or a git pin,
/// each with an optional `package` rename.
/// `Option<String>` is "a `String` or nothing" (TS `string |
/// undefined`);
/// `String` and not `&str` because the entry owns its text.
/// Why:
/// Features,
/// optional and default-features never live here:
/// Cargo only adds use-site features on top of an
/// inherited entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { kind: 'registry'; version: string; package?: string } | { kind: 'git'; git: string; rev: string; package?: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A crates.io requirement:
    /// a `>=` floor in the default catalog,
    /// a bounded requirement in a named catalog.
    Registry {
        /// Version requirement text.
        version: String,
        /// Cargo `package` rename target.
        package: Option<String>,
    },
    /// A git dependency pinned to one revision.
    Git {
        /// Repository URL.
        git: String,
        /// Pinned commit.
        rev: String,
        /// Cargo `package` rename target.
        package: Option<String>,
    },
}

/// Methods of `Entry`.
impl Entry {
    /// What:
    /// Returns the `package` rename target as a borrowed `Option<&str>` (`as_deref` turns
    /// `&Option<String>` into `Option<&str>`).
    /// Why:
    /// Equality and rendering treat both variants alike.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entry.package // string | undefined
    /// ```
    pub fn package(&self) -> Option<&str> {
        return match self {
            Entry::Registry { package, .. } => package.as_deref(),
            Entry::Git { package, .. } => package.as_deref(),
        };
    }
}

/// What:
/// `pub struct EntryMap { ... }` is an insertion-ordered map from dependency name to entry,
/// backed by a
/// `Vec` of pairs.
/// Why:
/// Source order drives diagnostics and warnings,
/// and no ordered-map crate is approved;
/// catalogs hold a
/// few hundred entries,
/// so linear lookup is fine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryMap = Map<string, Entry>; // a JS Map is already insertion ordered
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryMap {
    /// Pairs in source order.
    pairs: Vec<(String, Entry)>,
}

/// Methods of `EntryMap`.
impl EntryMap {
    /// Creates an empty map.
    pub fn new() -> EntryMap {
        return EntryMap { pairs: Vec::new() };
    }

    /// Looks a name up,
    /// returning a borrowed entry or `None`.
    pub fn get(&self, name: &str) -> Option<&Entry> {
        for (key, entry) in &self.pairs {
            if key == name {
                return Some(entry);
            }
        }
        return None;
    }

    /// Whether the name is present.
    pub fn contains(&self, name: &str) -> bool {
        return self.get(name).is_some();
    }

    /// Inserts or replaces an entry,
    /// keeping the first position of an existing name.
    pub fn insert(&mut self, name: &str, entry: Entry) {
        for pair in &mut self.pairs {
            if pair.0 == name {
                pair.1 = entry;
                return;
            }
        }
        self.pairs.push((name.to_string(), entry));
    }

    /// Entries in source order.
    pub fn iter(&self) -> std::slice::Iter<'_, (String, Entry)> {
        return self.pairs.iter();
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        return self.pairs.len();
    }

    /// Whether the map has no entries.
    pub fn is_empty(&self) -> bool {
        return self.pairs.is_empty();
    }
}

/// What:
/// `pub enum TrustPolicy { ... }` is pnpm's `trustPolicy` setting.
/// Why:
/// Only these two modes exist in `rust-catalog.toml`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrustPolicy = 'no-downgrade' | 'off';
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustPolicy {
    /// Refuse a release published less trustworthily than an earlier one.
    NoDowngrade,
    /// No trust check.
    Off,
}

/// What:
/// `pub struct Settings { ... }` holds the supply-chain settings with pnpm's names (snake-cased) and pnpm's
/// documented defaults applied.
/// Why:
/// The update tooling reads these;
/// this library only validates them and reads `catalog_prune`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RustCatalogSettings = { catalogPrune: boolean; minimumReleaseAge: number; ... };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Remove entries no crate uses on every sync.
    pub catalog_prune: bool,
    /// Minutes a release must have existed.
    pub minimum_release_age: u64,
    /// Names,
    /// patterns or exact versions exempt from the age rule.
    pub minimum_release_age_exclude: Vec<String>,
    /// Remove age exemptions no locked version needs.
    pub minimum_release_age_exclude_prune: bool,
    /// Whether a missing publish time passes the age rule.
    pub minimum_release_age_ignore_missing_time: bool,
    /// Whether the age rule fails when no release is old enough.
    pub minimum_release_age_strict: bool,
    /// Trust policy mode.
    pub trust_policy: TrustPolicy,
    /// Selectors exempt from the trust policy.
    pub trust_policy_exclude: Vec<String>,
    /// Remove trust exemptions no locked version needs.
    pub trust_policy_exclude_prune: bool,
    /// Minutes after which a release skips the trust check.
    pub trust_policy_ignore_after: Option<u64>,
}

/// What:
/// `pub struct CatalogDocument { ... }` is the validated catalog file:
/// the default catalog,
/// the named
/// catalogs in source order,
/// and the settings.
/// Why:
/// Adoption and resolution read it;
/// it is the library's parse result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RustCatalogDocument = { catalog: Map<string, Entry>; catalogs: Map<string, Map<string, Entry>>; settings: Settings };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDocument {
    /// The default `[catalog]`.
    pub catalog: EntryMap,
    /// `[catalogs.<name>]` tables in source order.
    pub catalogs: Vec<(String, EntryMap)>,
    /// Settings with defaults applied.
    pub settings: Settings,
}

/// Methods of `CatalogDocument`.
impl CatalogDocument {
    /// Looks a named catalog up.
    pub fn named(&self, name: &str) -> Option<&EntryMap> {
        for (key, entries) in &self.catalogs {
            if key == name {
                return Some(entries);
            }
        }
        return None;
    }
}
