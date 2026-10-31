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
use crate::jetbrains_discovery::{CONFIG_DIRECTORY, compare_versions, jetbrains_config_directory, latest_options_directory, version_parts};
/// Imports the scratch helpers.
use crate::test_support::scratch_dir;
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `OsStr` from `std::ffi`.
use std::ffi::OsStr;
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
    let found = latest_options_directory(&mut view, CONFIG_DIRECTORY, &prefixes(&["IntelliJIdea", "IdeaIC"])).expect("lists");
    assert_eq!(found.as_deref(), Some(".config/JetBrains/IdeaIC2026.10/options"));
    let only_intellij = latest_options_directory(&mut view, CONFIG_DIRECTORY, &prefixes(&["IntelliJIdea"])).expect("lists");
    assert_eq!(only_intellij.as_deref(), Some(".config/JetBrains/IntelliJIdea2026.9/options"));
}

/// Equal versions are broken by the larger directory name;
/// no match and no directory yield nothing.
#[test]
fn ties_and_absence_are_handled() {
    let dir = scratch_dir();
    let mut view = View::new(dir.path());
    assert_eq!(latest_options_directory(&mut view, CONFIG_DIRECTORY, &prefixes(&["IntelliJIdea"])).expect("lists"), None, "no JetBrains directory at all");
    for name in ["IntelliJIdea2026.2", "IdeaIC2026.2"] {
        std::fs::create_dir_all(dir.path().join(".config/JetBrains").join(name)).expect("product directory");
    }
    let mut second = View::new(dir.path());
    let tie = latest_options_directory(&mut second, CONFIG_DIRECTORY, &prefixes(&["IntelliJIdea", "IdeaIC"])).expect("lists");
    assert_eq!(tie.as_deref(), Some(".config/JetBrains/IntelliJIdea2026.2/options"));
    let mut third = View::new(dir.path());
    assert_eq!(latest_options_directory(&mut third, CONFIG_DIRECTORY, &prefixes(&["PyCharm"])).expect("lists"), None);
}

/// Allowed fixtures:
/// an unset, empty or relative `XDG_CONFIG_HOME` leaves the default under the home directory,
/// and an absolute one below the home directory names the `JetBrains` directory inside it.
#[test]
fn the_configuration_directory_follows_xdg_config_home() {
    let dir = scratch_dir();
    let root = dir.path();
    let default = CONFIG_DIRECTORY.to_string();
    assert_eq!(jetbrains_config_directory(root, None), Ok(default.clone()), "unset");
    assert_eq!(jetbrains_config_directory(root, Some(OsStr::new(""))), Ok(default.clone()), "empty is ignored");
    assert_eq!(jetbrains_config_directory(root, Some(OsStr::new("relative/config"))), Ok(default.clone()), "relative is ignored");
    assert_eq!(jetbrains_config_directory(root, Some(OsStr::new("./.config"))), Ok(default), "dot-relative is ignored");
    let configured = root.join("elsewhere/config");
    assert_eq!(jetbrains_config_directory(root, Some(configured.as_os_str())), Ok("elsewhere/config/JetBrains".to_string()));
    assert_eq!(jetbrains_config_directory(root, Some(root.as_os_str())), Ok("JetBrains".to_string()), "the home directory itself");
}

/// Rejected fixtures:
/// an absolute value outside the home directory, or one that climbs out of it, is an error naming the value,
/// because the view cannot read there and the default must not be edited in its place.
#[test]
fn a_configuration_directory_outside_the_home_is_refused() {
    let dir = scratch_dir();
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).expect("home");
    let outside = dir.path().join("other");
    let message = jetbrains_config_directory(&home, Some(outside.as_os_str())).expect_err("outside the home");
    assert!(message.contains(&outside.display().to_string()) && message.contains("outside the home directory"), "{message}");
    let climbing = home.join("..").join("other");
    let climbing_message = jetbrains_config_directory(&home, Some(climbing.as_os_str())).expect_err("climbs out");
    assert!(climbing_message.contains("path segment other than a plain name"), "{climbing_message}");
}

/// A home reached through a link spells the same directory differently;
/// the resolved comparison accepts it.
#[cfg(unix)]
#[test]
fn a_linked_spelling_of_the_home_is_accepted() {
    let dir = scratch_dir();
    let home = dir.path().join("home");
    std::fs::create_dir_all(home.join("config")).expect("config directory");
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&home, &alias).expect("link");
    let through_alias = alias.join("config");
    assert_eq!(jetbrains_config_directory(&home, Some(through_alias.as_os_str())), Ok("config/JetBrains".to_string()));
}
