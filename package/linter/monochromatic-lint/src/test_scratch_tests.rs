//! What:
//!  Controls of the unit-test scratch root.
//! Why:
//!  A fixture file that no `Cargo.toml` owns,
//! and a fixture directory outside the system temporary directory,
//!  are what the other unit tests assume of the root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the root is outside every manifest', () => expect(hasOwningManifest(scratchRoot())).toBe(false));
//! ```

use super::{has_owning_manifest, scratch_root};
use std::path::PathBuf;

/// No Cargo.toml owns the scratch root,
/// so a fixture file without a manifest of its own has no owning manifest.
#[test]
fn the_scratch_root_is_not_below_any_manifest() {
    let created: PathBuf = std::fs::canonicalize(scratch_root()).expect("scratch root");
    assert!(
        !has_owning_manifest(&created),
        "{} is below a Cargo.toml",
        created.display()
    );
}

/// The test reads the system temporary directory only to prove the scratch root is not inside it.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "compares against the banned location without writing there"
)]
fn the_scratch_root_is_not_under_the_system_temporary_directory() {
    let temporary: PathBuf =
        std::fs::canonicalize(std::env::temp_dir()).expect("system temporary directory");
    let created: PathBuf = std::fs::canonicalize(scratch_root()).expect("scratch root");
    assert!(
        !created.starts_with(&temporary),
        "{} is under {}",
        created.display(),
        temporary.display()
    );
}
