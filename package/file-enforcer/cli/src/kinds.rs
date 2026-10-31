//! What:
//!  The rule registry of the `file-enforcer` command line:
//! the engine's built-in kinds plus the three kinds that other crates implement.
//! Why:
//!  The engine crate does not link the dependency-catalog library,
//! the browserslist crate or the XML edit crate,
//! so meow and other embedders choose their own kinds;
//! the command line links all three and replaces the engine's `kind-not-wired` placeholders.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function productionRegistry(): RuleRegistry
//! ```

/// Imports `BrowserslistBuilder` from `crate::kind_browserslist`.
use crate::kind_browserslist::BrowserslistBuilder;
/// Imports `JetbrainsHarperBuilder` from `crate::kind_jetbrains_harper`.
use crate::kind_jetbrains_harper::JetbrainsHarperBuilder;
/// Imports `RustCatalogBuilder` from `crate::kind_rust_catalog`.
use crate::kind_rust_catalog::RustCatalogBuilder;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;

/// What:
///  The registry every `file-enforcer` command and server uses.
/// Why:
///  One place decides which library implements which kind,
/// so the server and the tests cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function productionRegistry(): RuleRegistry
/// ```
pub fn production_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::with_defaults();
    registry.register("rust-catalog", Box::new(RustCatalogBuilder));
    registry.register("browserslist", Box::new(BrowserslistBuilder::production()));
    registry.register("jetbrains-harper", Box::new(JetbrainsHarperBuilder::from_environment()));
    return registry;
}
