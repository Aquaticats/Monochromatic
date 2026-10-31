//! What:
//!  Tests for JetBrains product discovery:
//! version parsing and ordering,
//! and the choice of the newest product directory through the view.
//! Why:
//!  A newer install must win over an older one,
//! backup copies and unversioned names must never match,
//! and the choice must not depend on listing order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('latestJetbrainsOptionsDirectory', () => { it('picks the newest', () => {}); });
//! ```

/// Imports the helpers under test.
use crate::jetbrains_discovery::{compare_versions, latest_options_directory, version_parts};
/// Imports the scratch helpers.
use crate::test_support::scratch_dir;
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;

/// Owned prefixes from literals.
fn prefixes(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| return (*name).to_string()).collect();
}

/// A name parses only when a prefix is followed by dot-separated decimal segments.
#[test]
fn version_parts_accepts_only_numeric_suffixes() {
    let wanted = prefixes(&["IntelliJIdea", "IdeaIC"]);
    assert_eq!(version_parts("IntelliJIdea2026.2", &wanted), Some(vec![2026, 2]));
    assert_eq!(version_parts("IdeaIC2025.3.1", &wanted), Some(vec![2025, 3, 1]));
    for name in ["IntelliJIdea2026.2-backup", "IntelliJIdea", "IntelliJIdea2026.", "IntelliJIdea.2", "IntelliJIdeaCE2026.2", "Air", "IntelliJIdea99999999999999999999999"] {
        assert_eq!(version_parts(name, &wanted), None, "{name}");
    }
}

/// Comparison pads the shorter tuple with zeros.
#[test]
fn versions_compare_with_zero_padding() {
    assert_eq!(compare_versions(&[2026, 2], &[2026, 2, 0]), Ordering::Equal);
    assert_eq!(compare_versions(&[2026, 10], &[2026, 9]), Ordering::Greater);
    assert_eq!(compare_versions(&[2025, 3, 1], &[2025, 3, 2]), Ordering::Less);
    assert_eq!(compare_versions(&[], &[]), Ordering::Equal);
}

/// The newest matching product wins whatever the listing order,
/// and the options directory is home-relative.
#[test]
fn the_newest_matching_product_wins() {
    let dir = scratch_dir();
    for name in ["IntelliJIdea2025.3", "IntelliJIdea2026.2", "IntelliJIdea2026.2-backup", "IdeaIC2026.10", "Air", "IntelliJIdea2026.9"] {
        std::fs::create_dir_all(dir.path().join(".config/JetBrains").join(name)).expect("product directory");
    }
    std::fs::write(dir.path().join(".config/JetBrains/IntelliJIdea2027.1"), "a file, not a product").expect("a plain file");
    let mut view = View::new(dir.path());
    let found = latest_options_directory(&mut view, &prefixes(&["IntelliJIdea", "IdeaIC"])).expect("lists");
    assert_eq!(found.as_deref(), Some(".config/JetBrains/IdeaIC2026.10/options"));
    let only_intellij = latest_options_directory(&mut view, &prefixes(&["IntelliJIdea"])).expect("lists");
    assert_eq!(only_intellij.as_deref(), Some(".config/JetBrains/IntelliJIdea2026.9/options"));
}

/// Equal versions are broken by the larger directory name; no match and no directory yield nothing.
#[test]
fn ties_and_absence_are_handled() {
    let dir = scratch_dir();
    let mut view = View::new(dir.path());
    assert_eq!(latest_options_directory(&mut view, &prefixes(&["IntelliJIdea"])).expect("lists"), None, "no JetBrains directory at all");
    for name in ["IntelliJIdea2026.2", "IdeaIC2026.2"] {
        std::fs::create_dir_all(dir.path().join(".config/JetBrains").join(name)).expect("product directory");
    }
    let mut second = View::new(dir.path());
    let tie = latest_options_directory(&mut second, &prefixes(&["IntelliJIdea", "IdeaIC"])).expect("lists");
    assert_eq!(tie.as_deref(), Some(".config/JetBrains/IntelliJIdea2026.2/options"));
    let mut third = View::new(dir.path());
    assert_eq!(latest_options_directory(&mut third, &prefixes(&["PyCharm"])).expect("lists"), None);
}
