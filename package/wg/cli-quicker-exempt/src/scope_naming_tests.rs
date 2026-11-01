//! Verifies the scope names made from origin cgroup paths:
//! the escaping,
//! the uniqueness of distinct origins,
//! and the cut of a name that would pass the unit name limit.

/// Names under test.
use crate::scope_naming::{escape_unit_name, origin_scope_name, scope_cgroup_directory, scope_cgroup_path};
/// Fixture paths.
use std::path::Path;

/// The origin the owner's option preview names.
const CLAUDE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";

/// The origin of the cap daemon's cgroup gives the name the README shows,
/// with the path below the manager escaped the way `systemd-escape --path` does.
#[test]
fn the_claude_origin_is_named_from_its_escaped_path() {
    assert_eq!(origin_scope_name(CLAUDE, 1000), "wg-quicker-update-app.slice-claude\\x2dcode\\x2dbash.scope");
}

/// Slashes become dashes,
/// dashes and backslashes become hex escapes,
/// letters,
/// digits,
/// colons,
/// underscores and inner dots stay,
/// a leading dot is escaped,
/// and every byte of a multi-byte character is escaped.
#[test]
fn the_escaping_follows_systemd_escape() {
    assert_eq!(escape_unit_name("a/b-c"), "a-b\\x2dc");
    assert_eq!(escape_unit_name("a\\b"), "a\\x5cb");
    assert_eq!(escape_unit_name("x:y_z.scope"), "x:y_z.scope");
    assert_eq!(escape_unit_name(".hidden"), "\\x2ehidden");
    assert_eq!(escape_unit_name("a b"), "a\\x20b");
    assert_eq!(escape_unit_name("\u{e9}"), "\\xc3\\xa9");
    assert_eq!(escape_unit_name("a\nb"), "a\\x0ab");
}

/// Distinct origins get distinct names:
/// the same last component under different parents,
/// a dash against a slash,
/// and an escape spelled out against the character it stands for.
#[test]
fn distinct_origins_get_distinct_names() {
    let prefix = "/user.slice/user-1000.slice/user@1000.service";
    let names: Vec<String> = [
        format!("{prefix}/app.slice/x.scope"),
        format!("{prefix}/app.slice/y.slice/x.scope"),
        format!("{prefix}/app.slice/a-b"),
        format!("{prefix}/app.slice/a/b"),
        format!("{prefix}/app.slice/a\\x2db"),
    ]
    .iter()
    .map(|origin| return origin_scope_name(origin, 1000))
    .collect();
    for (index, name) in names.iter().enumerate() {
        assert!(!names[..index].contains(name), "{name} repeats");
    }
}

/// A name uses only the characters a unit name allows,
/// starts with the watcher's prefix and ends with `.scope`.
#[test]
fn every_name_is_a_valid_unit_name() {
    for origin in [CLAUDE, "/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty-surface-transient-1.scope", "/user.slice/user-1000.slice/user@1000.service/app.slice/we ird\u{e9}\n"] {
        let name = origin_scope_name(origin, 1000);
        assert!(name.starts_with("wg-quicker-update-") && name.ends_with(".scope"), "{name}");
        assert!(name.bytes().all(|byte| return byte.is_ascii_alphanumeric() || b":_.\\-".contains(&byte)), "{name}");
    }
}

/// A path outside the user manager's subtree is still named,
/// from the whole path without its leading slash.
#[test]
fn an_origin_outside_the_manager_subtree_is_named_from_its_whole_path() {
    assert_eq!(origin_scope_name("/system.slice/x.service", 1000), "wg-quicker-update-system.slice-x.service.scope");
    assert_eq!(origin_scope_name("/user.slice/user-2000.slice/user@2000.service/app.slice/x", 1000), "wg-quicker-update-user.slice-user\\x2d2000.slice-user\\x402000.service-app.slice-x.scope");
}

/// A name that would pass the limit of 255 bytes is cut and ends in a hash of the whole path,
/// is the same every time,
/// differs for two paths that agree up to the cut,
/// and never ends inside a four-byte escape.
#[test]
fn a_long_origin_is_cut_and_hashed_and_stays_within_the_limit() {
    let prefix = "/user.slice/user-1000.slice/user@1000.service/app.slice/";
    let dashes = "-".repeat(300);
    let first = origin_scope_name(&format!("{prefix}{dashes}a"), 1000);
    let second = origin_scope_name(&format!("{prefix}{dashes}b"), 1000);
    assert!(first.len() <= 255 && second.len() <= 255, "{} {}", first.len(), second.len());
    assert_ne!(first, second);
    assert_eq!(first, origin_scope_name(&format!("{prefix}{dashes}a"), 1000));
    for name in [&first, &second] {
        assert!(name.ends_with(".scope"));
        let stem = name.trim_end_matches(".scope");
        let (kept, hash) = stem.rsplit_once('-').expect("a hash follows the cut");
        assert_eq!(hash.len(), 16, "{name}");
        assert!(hash.bytes().all(|byte| return byte.is_ascii_hexdigit()), "{name}");
        assert!(!kept.ends_with("\\x") && !kept.ends_with("\\x2") && !kept.ends_with('\\'), "cut inside an escape: {kept}");
    }
}

/// The directory and the path of a scope follow the manager's layout.
#[test]
fn a_scope_is_found_below_the_updates_slice() {
    let path = scope_cgroup_path(1000, "app-monochromatic\\x2dupdates.slice", "wg-quicker-update-x.scope");
    assert_eq!(path, "/user.slice/user-1000.slice/user@1000.service/app.slice/app-monochromatic\\x2dupdates.slice/wg-quicker-update-x.scope");
    let directory = scope_cgroup_directory(Path::new("/sys/fs/cgroup"), 1000, "app-monochromatic\\x2dupdates.slice", "wg-quicker-update-x.scope");
    assert_eq!(directory, Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/')));
}
