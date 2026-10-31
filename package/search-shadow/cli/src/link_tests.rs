//! What:
//! Unit tests of the link creation in `link.rs`.
//! Why:
//! The links are what switch the binary into shadow mode,
//! so each present tool must get a link that points at the executable,
//! and an absent tool must get none (and lose a stale one).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // link.unit.test.ts
//! describe('linkShadows', () => { ... });
//! ```

/// Imports the function under test.
use crate::link::link_shadows;

/// Imports the file-system module (read links / write files).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks a fixture tool executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `fn write_tool(directory: &Path, name: &str)` declares a helper that writes an executable one-line script.
/// Why:
/// The links are made only for names that have a real runnable tool on PATH.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTool(directory: string, name: string): void { fs.writeFileSync(...); fs.chmodSync(..., 0o755); }
/// ```
fn write_tool(directory: &Path, name: &str) {
    // `.expect("...")` unwraps a `Result` and, on error, stops the test with that message (tests may panic on purpose).
    fs::write(directory.join(name), "#!/bin/sh\n").expect("write the fixture tool");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o755)).expect("make the fixture tool executable");
}

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('links only the names that have a real tool', () => { ... });
/// ```
#[test]
fn links_only_the_names_that_have_a_real_tool() {
    let world = crate::test_scratch::scratch();
    let tools = world.path().join("tools");
    fs::create_dir(&tools).expect("make the tools directory");
    write_tool(&tools, "rg");
    write_tool(&tools, "grep");
    write_tool(&tools, "find");
    let executable = world.path().join("search-shadow");
    write_tool(world.path(), "search-shadow");
    let links_directory = world.path().join("bin");
    let path_variable = tools.to_string_lossy().to_string();

    let report = link_shadows(&links_directory, &executable, &path_variable).expect("create the links");

    // `assert_eq!(a, b)` stops the test with a message when `a != b` (a macro, hence `!`).
    assert_eq!(report.linked.len(), 3);
    assert!(report.removed.is_empty());
    for name in ["rg", "grep", "find"] {
        // `fs::read_link` gives the path a symlink points at.
        assert_eq!(fs::read_link(links_directory.join(name)).expect("read the link"), executable);
    }
    // Absent tools get no link and are reported as skipped.
    assert!(report.skipped.contains(&"ag".to_string()));
    assert!(report.skipped.contains(&"fdfind".to_string()));
    assert!(links_directory.join("ag").symlink_metadata().is_err());
}

/// A link whose real tool has gone must be removed,
/// so `command -v` stops reporting it.
#[test]
fn removes_a_stale_link_when_the_real_tool_is_gone() {
    let world = crate::test_scratch::scratch();
    let tools = world.path().join("tools");
    fs::create_dir(&tools).expect("make the tools directory");
    write_tool(&tools, "rg");
    let executable = world.path().join("search-shadow");
    write_tool(world.path(), "search-shadow");
    let links_directory = world.path().join("bin");
    let path_variable = tools.to_string_lossy().to_string();
    link_shadows(&links_directory, &executable, &path_variable).expect("create the links");

    // The tool disappears; linking again removes its link.
    fs::remove_file(tools.join("rg")).expect("remove the real tool");
    let report = link_shadows(&links_directory, &executable, &path_variable).expect("link again");

    assert_eq!(report.removed, vec![links_directory.join("rg")]);
    assert!(links_directory.join("rg").symlink_metadata().is_err());
}

/// Linking again is idempotent and replaces what is in the way
/// and never takes a shadow link for the real tool.
#[test]
fn linking_again_replaces_what_is_there_and_skips_its_own_links() {
    let world = crate::test_scratch::scratch();
    let tools = world.path().join("tools");
    fs::create_dir(&tools).expect("make the tools directory");
    write_tool(&tools, "rg");
    let executable = world.path().join("search-shadow");
    write_tool(world.path(), "search-shadow");
    let links_directory = world.path().join("bin");
    fs::create_dir(&links_directory).expect("make the links directory");
    fs::write(links_directory.join("rg"), "old regular file").expect("write an old file in the way");
    // The links directory comes first on PATH, as in real use; its links must not count as real tools.
    let path_variable = format!("{}:{}", links_directory.display(), tools.display());

    link_shadows(&links_directory, &executable, &path_variable).expect("create the links");
    let second = link_shadows(&links_directory, &executable, &path_variable).expect("create the links again");

    assert_eq!(fs::read_link(links_directory.join("rg")).expect("read the link"), executable);
    assert_eq!(second.linked, vec![links_directory.join("rg")]);
    assert!(second.removed.is_empty());
}
