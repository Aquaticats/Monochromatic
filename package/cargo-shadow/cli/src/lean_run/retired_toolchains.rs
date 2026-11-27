//! What:
//! The dated nightly toolchains whose keyed folders exist and that no checkout pins.
//! Why:
//! Retiring a toolchain is removing its folder,
//! and a folder a checkout pins is still in use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function retiredToolchainNames(trees: Tree[], pinned: Set<string>): Set<string>
//! ```
/// Imports the tree and the key its toolchain is named by.
use crate::lean_build_trees::{
    Tree,
    TreeKind,
};
/// Imports the check that a keyed folder names a dated nightly.
use crate::lean_rustup_toolchains::dated_nightly_name;
/// Imports the set the pinned names are held in and answered with.
use std::collections::HashSet;
/// What:
/// `fn retired_toolchain_names(trees: &[Tree], pinned: &HashSet<String>) -> HashSet<String>` lists the dated nightlies whose keyed
/// folders exist and that no checkout pins.
/// Why:
/// Retiring a toolchain is removing its folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retiredToolchainNames(trees: Tree[], pinned: Set<string>): Set<string>
/// ```
pub(crate) fn retired_toolchain_names(
    trees: &[Tree],
    pinned: &HashSet<String>,
) -> HashSet<String> {
    let mut retired: HashSet<String> = HashSet::new();
    for tree in trees {
        if let TreeKind::Keyed(name) = &tree.kind
            && dated_nightly_name(name).is_some()
            && !pinned.contains(name)
        {
            retired.insert(name.clone());
        }
    }
    return retired;
}
