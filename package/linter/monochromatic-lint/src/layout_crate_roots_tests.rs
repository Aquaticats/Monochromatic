//! What:
//!  Controls of crate root discovery.
//! Why:
//!  A root that is missed leaves a whole part of the package unchecked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('finds lib, main and bin roots', () => expect(crateRootFiles(manifest)).toEqual([...]));
//! ```

use super::{crate_root_files, explicit_root_paths, path_value};
use crate::layout_test_support::package_with;
use std::path::PathBuf;

/// The default library,
/// executable and bin roots are found,
/// sorted,
/// and a missing default is no error.
#[test]
fn default_roots_are_found_in_order() {
    let fixture = package_with(&[
        ("src/lib.rs", ""),
        ("src/main.rs", ""),
        ("src/bin/tool.rs", ""),
        ("src/bin/other/main.rs", ""),
        ("src/bin/other/helper.rs", ""),
    ]);
    let roots: Vec<PathBuf> = crate_root_files(&fixture.path.join("Cargo.toml")).expect("roots");
    let expected: Vec<PathBuf> = [
        "src/bin/other/main.rs",
        "src/bin/tool.rs",
        "src/lib.rs",
        "src/main.rs",
    ]
    .iter()
    .map(|name| return fixture.path.join(name))
    .collect::<Vec<PathBuf>>();
    assert_eq!(roots, expected);
}

/// Paths from `[lib]` and `[[bin]]` are roots,
/// tables of other names are not,
/// and a path that is also a default root is listed once.
#[test]
fn explicit_paths_of_lib_and_bin_tables_are_roots() {
    let fixture = package_with(&[
        ("src/lib.rs", ""),
        ("engine/start.rs", ""),
        ("cli/run.rs", ""),
        ("extra/other.rs", ""),
    ]);
    std::fs::write(
        fixture.path.join("Cargo.toml"),
        "[package]\nname = \"x\"\n[lib]\npath = \"engine/start.rs\" # comment\n[[bin]]\nname = 'run'\npath = 'cli/run.rs'\n[[test]]\npath = \"extra/other.rs\"\n[[bin]]\npath = \"src/lib.rs\"\n",
    )
    .expect("manifest");
    let roots: Vec<PathBuf> = crate_root_files(&fixture.path.join("Cargo.toml")).expect("roots");
    let expected: Vec<PathBuf> = ["cli/run.rs", "engine/start.rs", "src/lib.rs"]
        .iter()
        .map(|name| return fixture.path.join(name))
        .collect::<Vec<PathBuf>>();
    assert_eq!(roots, expected);
}

/// Both string forms are read,
/// and other keys and shapes are not.
#[test]
fn path_values_read_both_string_forms_only() {
    assert_eq!(
        path_value("path = \"a/b.rs\""),
        Some(String::from("a/b.rs"))
    );
    assert_eq!(
        path_value("  path='a/b.rs' # note"),
        Some(String::from("a/b.rs"))
    );
    assert_eq!(path_value("pathology = \"x\""), None);
    assert_eq!(path_value("path = 5"), None);
    assert_eq!(path_value("name = \"path\""), None);
    assert_eq!(
        explicit_root_paths("[dependencies]\npath = \"x\"\n"),
        Vec::<String>::new()
    );
}

/// A manifest that cannot be read is an error naming it.
#[test]
fn an_unreadable_manifest_is_an_error_naming_it() {
    let fixture = package_with(&[]);
    let message: String =
        crate_root_files(&fixture.path.join("missing.toml")).expect_err("no such manifest");
    assert!(message.contains("missing.toml"), "{message}");
}
