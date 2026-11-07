//! What:
//! Unit tests for the accepted-specifier grammar,
//! one test per rule edge.
//! Why:
//! The grammar is the heart of the contract;
//! each literal the README names must have a test that fails when it breaks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // grammar.unit.test.ts
//! describe('isAllowedSpecifier', () => { it('accepts bare catalog:', () => { /* ... */ }); });
//! ```

/// What:
/// `use super::{...};` imports names from the parent module (`super` is "the module above this one"),
/// here the crate root,
/// which re-exports the grammar.
/// Why:
/// Tests call the same public path a consumer would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isAllowedSpecifier, type SectionKind } from './grammar';
/// ```
use super::{SectionKind, is_allowed_specifier};

/// What:
/// `#[test]` marks the next function as a test the runner (`cargo nextest`) will call.
/// `fn name() { ... }` takes no inputs;
/// a failed `assert!` panics,
/// which the runner reports as a failure.
/// Why:
/// Bare `catalog:` selects the default catalog and must pass in both kinds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('accepts bare catalog: in both kinds', () => { expect(isAllowedSpecifier('install', 'catalog:')).toBe(true); });
/// ```
#[test]
fn bare_catalog_is_allowed_in_both_kinds() {
    assert!(is_allowed_specifier(SectionKind::Install, "catalog:"));
    assert!(is_allowed_specifier(SectionKind::Peer, "catalog:"));
}

/// Named catalogs with the full allowed character set pass in both kinds.
#[test]
fn named_catalog_with_every_allowed_character_class_is_allowed() {
    assert!(is_allowed_specifier(SectionKind::Install, "catalog:Team.a_b-9"));
    assert!(is_allowed_specifier(SectionKind::Peer, "catalog:Team.a_b-9"));
}

/// `workspace:*` passes only in install sections.
#[test]
fn workspace_star_is_allowed_only_in_install_sections() {
    assert!(is_allowed_specifier(SectionKind::Install, "workspace:*"));
    assert!(!is_allowed_specifier(SectionKind::Peer, "workspace:*"));
}

/// A bare `*` passes only in the peer section.
#[test]
fn bare_star_is_allowed_only_in_peers() {
    assert!(is_allowed_specifier(SectionKind::Peer, "*"));
    assert!(!is_allowed_specifier(SectionKind::Install, "*"));
}

/// Every literal-version shape is refused in both kinds.
#[test]
fn literal_versions_are_refused_in_both_kinds() {
    // What: `for specifier in ["a", "b"]` loops over a fixed array of `&str` values; `assert!` panics when its condition is false.
    // Why: One loop covers every shape without repeating the assertion.
    for specifier in ["4.6.7", "^4.6.7", "~1.2.3", ">=4.6.7", "1.0.0-beta.1", "latest", "x", ""] {
        assert!(!is_allowed_specifier(SectionKind::Install, specifier), "install accepted {specifier:?}");
        assert!(!is_allowed_specifier(SectionKind::Peer, specifier), "peer accepted {specifier:?}");
    }
}

/// Workspace forms other than exactly `workspace:*` are refused,
/// including the `*.0` trailing-text leak.
#[test]
fn other_workspace_forms_are_refused() {
    for specifier in ["workspace:", "workspace:^", "workspace:~", "workspace:^1.0.0", "workspace:*.0", "workspace:**"] {
        assert!(!is_allowed_specifier(SectionKind::Install, specifier), "install accepted {specifier:?}");
    }
}

/// Peer text that merely starts with `*` is refused.
#[test]
fn peers_starting_with_a_star_are_refused() {
    for specifier in ["*.0", "* ", "**", " *"] {
        assert!(!is_allowed_specifier(SectionKind::Peer, specifier), "peer accepted {specifier:?}");
    }
}

/// Catalog references with whitespace,
/// colons,
/// wildcards,
/// non-ASCII text or the wrong case are refused.
#[test]
fn malformed_catalog_references_are_refused() {
    for specifier in [
        "catalog: ", " catalog:", "catalog:team ", "catalog:te am", "catalog:te\tam", "catalog:team:other", "catalog:*",
        "catalog:\u{e9}", "Catalog:", "CATALOG:", "catalog", "catalog:\n",
    ] {
        assert!(!is_allowed_specifier(SectionKind::Install, specifier), "install accepted {specifier:?}");
        assert!(!is_allowed_specifier(SectionKind::Peer, specifier), "peer accepted {specifier:?}");
    }
}

/// Protocol and URL forms are refused.
#[test]
fn protocol_and_url_forms_are_refused() {
    for specifier in [
        "link:../x", "file:../x", "git+https://github.com/a/b.git", "github:a/b", "https://example.com/x.tgz",
        "npm:other@^1.0.0", "portal:../x", "jsr:@a/b@^1",
    ] {
        assert!(!is_allowed_specifier(SectionKind::Install, specifier), "install accepted {specifier:?}");
    }
}
