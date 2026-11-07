//! What:
//!  Controls of the libghostty offline-input preparation.
//! Why:
//!  A package whose offline inputs are absent or half written must skip with the task that makes them,
//! and a complete set must reach the build as the two variables that `libghostty-vt-sys` reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('absent inputs skip', () => expect(() => ghosttySetup(emptyDirectory)).toThrow(/fetch:ghostty-offline-inputs/));
//! ```

use super::{GHOSTTY_SOURCE_VARIABLE, GHOSTTY_ZIG_SYSTEM_VARIABLE, ghostty_setup};
use crate::test_fs::Fixture;
use std::path::{Path, PathBuf};

/// Write the inputs as the package's fetch task leaves them,
/// the ready marker last.
fn write_inputs(package: &Path, marker: bool) -> PathBuf {
    let root: PathBuf = package.join("target").join("ghostty-offline");
    std::fs::create_dir_all(root.join("source")).expect("source directory");
    std::fs::write(root.join("source").join("build.zig"), "").expect("build.zig");
    std::fs::create_dir_all(root.join("packages")).expect("packages directory");
    if marker {
        std::fs::write(root.join("ready"), "").expect("ready marker");
    }
    return root;
}

/// Without any input the package is skipped with a reason that names the fetch task and the marker.
#[test]
fn absent_inputs_are_skipped_with_the_task_that_makes_them() {
    let fixture: Fixture = Fixture::new();
    let reason: String = ghostty_setup(&fixture.path).expect_err("nothing is fetched");
    assert!(
        reason.contains("//package/terminal/app:fetch:ghostty-offline-inputs"),
        "{reason}"
    );
    assert!(reason.contains("ready"), "{reason}");
}

/// Inputs without the ready marker are a fetch that did not finish,
/// so the package is skipped.
#[test]
fn inputs_without_the_ready_marker_are_skipped() {
    let fixture: Fixture = Fixture::new();
    write_inputs(&fixture.path, false);
    assert!(ghostty_setup(&fixture.path).is_err());
}

/// A marker beside a source tree without `build.zig` is skipped too,
/// because `libghostty-vt-sys` asserts on it.
#[test]
fn a_source_tree_without_build_zig_is_skipped() {
    let fixture: Fixture = Fixture::new();
    let root: PathBuf = write_inputs(&fixture.path, true);
    std::fs::remove_file(root.join("source").join("build.zig")).expect("remove build.zig");
    assert!(ghostty_setup(&fixture.path).is_err());
}

/// Complete inputs become the two variables,
/// each naming an absolute directory below the package.
#[test]
fn complete_inputs_become_the_two_build_variables() {
    let fixture: Fixture = Fixture::new();
    let package: PathBuf = fixture.path.join("terminal package");
    let root: PathBuf = write_inputs(&package, true);
    let variables: Vec<(String, String)> = ghostty_setup(&package).expect("prepared");
    assert_eq!(
        variables,
        vec![
            (
                String::from(GHOSTTY_SOURCE_VARIABLE),
                root.join("source").to_string_lossy().into_owned()
            ),
            (
                String::from(GHOSTTY_ZIG_SYSTEM_VARIABLE),
                root.join("packages").to_string_lossy().into_owned()
            ),
        ]
    );
}
