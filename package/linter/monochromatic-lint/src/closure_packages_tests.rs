//! What:
//!  Controls of the grouping of selected files by owning package.
//! Why:
//!  One package must be linted once however many of its files are selected,
//! sidecar crates and unowned files must be
//!  left out with a reason,
//! and the library path must resolve against the configuration that named it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('groups by manifest', () => expect(groupPackages(plans, cwd).packages.map(p => p.manifest)).toEqual([...]));
//! ```

use super::{group_packages, is_sidecar_crate};
use crate::{
    run_plan::FilePlan,
    run_test_support::{CONFIG, plan, write},
    test_fs::Fixture,
};
use std::path::Path;

/// The rule at warning level with a relative library,
/// and off for tests.
const CONFIGURATION: &str = r#"[
  { "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "tools/closure lib" } } },
  { "files": ["**/tests/**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "off" } } }
]"#;

/// Plan files of a fixture.
fn plans(root: &Path, files: &[&str]) -> Vec<FilePlan> {
    let mut found: Vec<FilePlan> = Vec::<FilePlan>::new();
    for file in files {
        found.push(plan(root, file));
    }
    return found;
}

/// Several files of one package make one entry,
/// and the library resolves against the configuration directory.
#[test]
fn several_files_of_one_package_make_one_entry() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "a package/Cargo.toml", "[package]\n");
    write(&fixture.path, "a package/src/lib.rs", "");
    write(&fixture.path, "a package/src/other.rs", "");
    write(&fixture.path, "a package/src/deep/mod.rs", "");
    let selected = plans(
        &fixture.path,
        &[
            "a package/src/lib.rs",
            "a package/src/other.rs",
            "a package/src/deep/mod.rs",
        ],
    );
    let grouping = group_packages(selected.as_slice(), &fixture.path);
    assert_eq!(grouping.packages.len(), 1);
    assert_eq!(
        grouping.packages[0].manifest,
        fixture.path.join("a package/Cargo.toml")
    );
    assert_eq!(
        grouping.packages[0].library,
        fixture.path.join("tools/closure lib")
    );
    assert_eq!(grouping.packages[0].configuration, CONFIG);
    assert!(grouping.notes.is_empty());
}

/// Two packages make two entries,
/// ordered by manifest path.
#[test]
fn two_packages_make_two_ordered_entries() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "b/Cargo.toml", "");
    write(&fixture.path, "a/Cargo.toml", "");
    write(&fixture.path, "b/src/lib.rs", "");
    write(&fixture.path, "a/src/lib.rs", "");
    let grouping = group_packages(
        plans(&fixture.path, &["b/src/lib.rs", "a/src/lib.rs"]).as_slice(),
        &fixture.path,
    );
    assert_eq!(
        grouping.packages[0].manifest,
        fixture.path.join("a/Cargo.toml")
    );
    assert_eq!(
        grouping.packages[1].manifest,
        fixture.path.join("b/Cargo.toml")
    );
}

/// Fuzz and bench crates are left out with a note,
/// even though their files select the rule.
#[test]
fn fuzz_and_bench_crates_are_left_out() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    for directory in ["x.fuzz", "x.bench", "x"] {
        write(
            &fixture.path,
            format!("{directory}/Cargo.toml").as_str(),
            "",
        );
        write(
            &fixture.path,
            format!("{directory}/src/lib.rs").as_str(),
            "",
        );
    }
    let selected = plans(
        &fixture.path,
        &["x.fuzz/src/lib.rs", "x.bench/src/lib.rs", "x/src/lib.rs"],
    );
    let grouping = group_packages(selected.as_slice(), &fixture.path);
    assert_eq!(grouping.packages.len(), 1);
    assert_eq!(
        grouping.packages[0].manifest,
        fixture.path.join("x/Cargo.toml")
    );
    assert_eq!(grouping.notes.len(), 2);
    assert!(
        grouping.notes[0].contains("fuzz or bench crate"),
        "{:?}",
        grouping.notes
    );
}

/// The suffix rule looks at the directory name only.
#[test]
fn the_sidecar_suffix_is_read_from_the_directory_name() {
    assert!(is_sidecar_crate(Path::new("/p/cli/forbidden-strings.fuzz")));
    assert!(is_sidecar_crate(Path::new("/p/music/truepeak-core.bench")));
    assert!(!is_sidecar_crate(Path::new("/p/fuzz/real-crate")));
    assert!(!is_sidecar_crate(Path::new("/p/crate.fuzzy")));
    assert!(!is_sidecar_crate(Path::new("/p/benchmark")));
    assert!(!is_sidecar_crate(Path::new("/")));
}

/// A file that no Cargo.toml owns is left out with a note naming it,
/// without failing.
#[test]
fn a_file_without_an_owning_manifest_is_left_out_with_a_note() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "loose/script.rs", "");
    let grouping = group_packages(
        plans(&fixture.path, &["loose/script.rs"]).as_slice(),
        &fixture.path,
    );
    assert!(grouping.packages.is_empty());
    assert_eq!(grouping.notes.len(), 1);
    assert!(
        grouping.notes[0].starts_with("loose/script.rs: not checked by rust/avoidable-closure"),
        "{:?}",
        grouping.notes
    );
}

/// A file whose rule is off selects nothing,
/// and the nearest manifest owns a nested file.
#[test]
fn files_with_the_rule_off_select_nothing_and_the_nearest_manifest_owns() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "outer/Cargo.toml", "");
    write(&fixture.path, "outer/inner/Cargo.toml", "");
    write(&fixture.path, "outer/inner/src/lib.rs", "");
    write(&fixture.path, "outer/tests/it.rs", "");
    let grouping = group_packages(
        plans(
            &fixture.path,
            &["outer/tests/it.rs", "outer/inner/src/lib.rs"],
        )
        .as_slice(),
        &fixture.path,
    );
    assert_eq!(grouping.packages.len(), 1);
    assert_eq!(
        grouping.packages[0].manifest,
        fixture.path.join("outer/inner/Cargo.toml")
    );
}

/// A Markdown plan is never grouped.
#[test]
fn markdown_is_not_grouped() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "a/Cargo.toml", "");
    write(&fixture.path, "a/README.md", "# a\n");
    // The README has no matching block, so it is planned with no root rules.
    let grouping = group_packages(
        plans(&fixture.path, &["a/README.md"]).as_slice(),
        &fixture.path,
    );
    assert!(grouping.packages.is_empty() && grouping.notes.is_empty());
}
