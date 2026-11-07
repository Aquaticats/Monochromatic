//! What:
//! The check that the four shadow crates carry one copy of the rule "a shadow never starts a shadow".
//! Why:
//! `cargo-shadow`, `mpm-shadow`, `pnpm-shadow` and `search-shadow` share no dependency,
//! so `src/shadow_identity.rs` and its tests are byte-identical copies, one per crate.
//! A copy that changed alone would make one shadow judge another by a different identity text,
//! and a shadow that the others skip is a shadow that can be started as a real tool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const crate of SHADOW_CRATES) assert(readFileSync(join(crate, 'src/shadow-identity.rs')).equals(own));
//! ```

use std::fs;
use std::path::PathBuf;

/// The files that must be the same in every shadow crate.
const COPIED_FILES: &[&str] = &["src/shadow_identity.rs", "src/shadow_identity_tests.rs"];

/// The other shadow crates, as folders beside this package's folder (`package/<name>/cli`).
const OTHER_SHADOW_CRATES: &[&str] = &["cargo-shadow", "mpm-shadow", "pnpm-shadow"];

#[test]
fn every_shadow_crate_carries_the_same_copy_of_the_identity_rule() {
    let own_crate: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let packages: PathBuf = own_crate.parent().and_then(|shadow_folder| { return shadow_folder.parent(); }).expect("the package folder").to_path_buf();
    for file in COPIED_FILES {
        let own: String = fs::read_to_string(own_crate.join(file)).expect("read this crate's copy");
        for other in OTHER_SHADOW_CRATES {
            let path: PathBuf = packages.join(other).join("cli").join(file);
            let copy: String = fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            assert_eq!(own, copy, "{} differs from the copy in search-shadow: change both, byte for byte", path.display());
        }
    }
}
