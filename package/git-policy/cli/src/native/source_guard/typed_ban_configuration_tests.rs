//! What:
//!  The pin of the typed ban's own configuration:
//!  the crate's Clippy level table,
//!  and every disallowed call's path in `clippy.toml`.
//! Why:
//!  `disallowed_methods = "deny"` is what makes the two `std::env::args` rows a
//!  ban,
//!  and at `allow` the same source compiles with no call refused and no pinned
//!  line changed,
//!  so the configuration is held to the reviewed set rather than trusted
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "The typed half,
//! and the configuration it rests on").
//!  A reason's prose is free:
//! only the level table and the paths are compared.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(squeezed(clippyLevels)).toBe(CLIPPY_LEVELS);
//! expect(disallowedPaths).toEqual(DISALLOWED_CALLS);
//! ```

use super::squeezed;
use std::path::Path;

/// The crate's Clippy level table,
/// squeezed,
/// as the review fixed it.
/// `disallowed_methods` at `deny` is what makes the two `std::env::args` rows a ban;
/// at `allow` the same source compiles with no call refused and no pinned line changed.
const CLIPPY_LEVELS: &str = "complexity={level=\"allow\",priority=-1}disallowed_methods=\"deny\"implicit_return=\"deny\"needless_return=\"allow\"shadow_reuse=\"deny\"shadow_same=\"deny\"shadow_unrelated=\"deny\"";

/// Every disallowed call's path,
/// in the configuration's own order.
const DISALLOWED_CALLS: &[&str] = &[
    "std::result::Result::unwrap",
    "std::env::temp_dir",
    "tempfile::tempdir",
    "tempfile::tempfile",
    "tempfile::TempDir::new",
    "tempfile::NamedTempFile::new",
    "std::env::current_exe",
    "std::env::args",
    "std::env::args_os",
];

/// The typed ban is configured as this design names it.
#[test]
fn the_typed_ban_is_configured_as_this_design_names_it() {
    let crate_root: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: String =
        std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("the manifest");
    let mut levels: Vec<String> = Vec::new();
    let mut in_table: bool = false;
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            in_table = trimmed == "[lints.clippy]";
            continue;
        }
        if !in_table || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        levels.push(String::from(trimmed));
    }
    assert_eq!(
        squeezed(levels.join("\n").as_str()),
        CLIPPY_LEVELS,
        "the crate's Clippy level table is not the reviewed one, and a level changed here silences every pinned call without changing a line of source"
    );
    let configuration: String =
        std::fs::read_to_string(crate_root.join("clippy.toml")).expect("the Clippy configuration");
    let calls: Vec<&str> = configuration
        .lines()
        .filter_map(|line| return line.trim().strip_prefix("{ path = \""))
        .filter_map(|rest| {
            return rest.split('"').next();
        })
        .collect();
    assert_eq!(
        calls, DISALLOWED_CALLS,
        "the disallowed calls are not the reviewed set, in the reviewed order"
    );
}

/// The two rows the root modes add are each read by the list above,
/// and neither is the call a control may make.
#[test]
fn the_two_argument_vector_rows_are_both_named() {
    let configuration: String =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("clippy.toml"))
            .expect("the Clippy configuration");
    for call in ["std::env::args", "std::env::args_os"] {
        assert!(
            configuration.contains(&format!("{{ path = \"{call}\"")),
            "`{call}` is one of the two calls the root-mode dispatch rests on: an alias or a direct call would read the argument vector anywhere else"
        );
    }
    assert!(
        configuration.contains("disallowed-methods = ["),
        "the rows sit in the one table Clippy reads"
    );
}
