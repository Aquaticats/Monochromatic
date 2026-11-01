//! What:
//!  Unit tests of the ownership decision for source files.
//! Why:
//!  The decision has four inputs (relative or absolute path,
//! inside or outside the package directory,
//! inside or outside the
//!  build script's output directory,
//! environment present or absent),
//! and the UI fixture covers only the compiler-facing path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test.each(cases)('excludes %s', (path, expected) => expect(roots.excludes(path)).toBe(expected));
//! ```

/// Import the type under test.
use crate::own_source::OwnSource;
/// Import the borrowed path type.
use std::path::Path;

/// The roots of a package with a build script whose output directory sits below the package.
fn package_with_output_inside() -> OwnSource {
    return OwnSource::from_values(
        Some("/work/package".into()),
        Some("/work/package/target/out".into()),
    );
}

/// A relative path is the package's own source.
#[test]
fn keeps_a_relative_path() {
    assert!(!package_with_output_inside().excludes(Path::new("src/lib.rs")));
}

/// An absolute path below the package directory is the package's own source.
#[test]
fn keeps_an_absolute_path_inside_the_package() {
    assert!(!package_with_output_inside().excludes(Path::new("/work/package/src/lib.rs")));
}

/// An absolute path outside the package directory is foreign.
#[test]
fn excludes_an_absolute_path_outside_the_package() {
    assert!(package_with_output_inside().excludes(Path::new("/var/build/out/app.rs")));
}

/// A sibling directory whose name starts with the package directory's name is outside it.
#[test]
fn excludes_a_sibling_with_a_common_name_prefix() {
    assert!(package_with_output_inside().excludes(Path::new("/work/package-two/src/lib.rs")));
}

/// Build-script output below the package directory is generated although the prefix test alone would keep it.
#[test]
fn excludes_the_output_directory_inside_the_package() {
    assert!(package_with_output_inside().excludes(Path::new("/work/package/target/out/app.rs")));
}

/// A `..` component that leaves the package directory makes the path foreign.
#[test]
fn excludes_a_parent_component_that_leaves_the_package() {
    assert!(package_with_output_inside().excludes(Path::new("/work/package/../shared/x.rs")));
}

/// A `..` component that stays inside the package directory keeps the path.
#[test]
fn keeps_a_parent_component_that_stays_inside_the_package() {
    assert!(!package_with_output_inside().excludes(Path::new("/work/package/src/../lib.rs")));
}

/// A `..` component that climbs out of the output directory returns the path to the package's own source.
#[test]
fn keeps_a_path_that_leaves_the_output_directory_for_the_package() {
    assert!(
        !package_with_output_inside()
            .excludes(Path::new("/work/package/target/out/../../src/lib.rs"))
    );
}

/// Without the package directory (the compiler was not started by cargo) only the output directory excludes.
#[test]
fn keeps_every_path_without_a_manifest_directory() {
    let roots: OwnSource = OwnSource::from_values(None, None);
    assert!(!roots.excludes(Path::new("/anywhere/x.rs")));
    assert!(!roots.excludes(Path::new("x.rs")));
}

/// A relative output directory never matches an absolute path,
/// and a relative path below it is excluded.
#[test]
fn excludes_below_a_relative_output_directory() {
    let roots: OwnSource = OwnSource::from_values(Some("/work/package".into()), Some("out".into()));
    assert!(roots.excludes(Path::new("out/app.rs")));
    assert!(!roots.excludes(Path::new("src/app.rs")));
}
