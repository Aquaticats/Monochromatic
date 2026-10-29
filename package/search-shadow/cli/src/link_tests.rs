//! What:     Unit tests of the link creation in `link.rs`.
//! Why:      The links are what switch the binary into shadow mode, so each name must exist and point at the executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // link.unit.test.ts
//! describe('linkShadows', () => { ... });
//! ```

/// Imports the function under test.
use crate::link::link_shadows;

/// Imports the file-system module (read links, write files).
use std::fs;

/// What:     `#[test]` marks the next function as a test case.
/// Why:      The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('creates one link per shadowed name', () => { ... });
/// ```
#[test]
fn creates_one_link_per_shadowed_name_pointing_at_the_executable() {
    let directory = tempfile::tempdir().expect("make a temporary directory");
    let executable = directory.path().join("search-shadow");
    fs::write(&executable, "binary").expect("write the fixture executable");
    let links_directory = directory.path().join("bin");

    // `link_shadows(...)` returns a `Result`; `.expect("...")` unwraps the success case or stops the test.
    let created = link_shadows(&links_directory, &executable).expect("create the links");

    assert_eq!(created.len(), 3);
    for name in ["rg", "grep", "find"] {
        // `fs::read_link` gives the path a symlink points at.
        let target = fs::read_link(links_directory.join(name)).expect("read the link");
        assert_eq!(target, executable);
    }
}

/// Linking again must replace what is there, not fail on it.
#[test]
fn replaces_existing_links_and_files() {
    let directory = tempfile::tempdir().expect("make a temporary directory");
    let executable = directory.path().join("search-shadow");
    fs::write(&executable, "binary").expect("write the fixture executable");
    let links_directory = directory.path().join("bin");
    fs::create_dir(&links_directory).expect("make the links directory");
    fs::write(links_directory.join("rg"), "old regular file").expect("write an old file in the way");

    link_shadows(&links_directory, &executable).expect("create the links");
    link_shadows(&links_directory, &executable).expect("create the links again");

    assert_eq!(fs::read_link(links_directory.join("rg")).expect("read the link"), executable);
}
