//! What:
//! Where the lean-folder run looks:
//! the in-tree `target/` directories of every checkout,
//! their `verification` directories,
//! and the profile directories of everything it will sweep.
//! Why:
//! A package keeps its output in `<checkout>/package/<category>/<name>/target`
//! (the repository's package layout),
//! and the root of a checkout may have its own `target`.
//! The walk never descends into a nested worktree or a dependency directory:
//! every checkout is listed once by `lean_checkouts.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function targetDirectories(checkout: string): string[]
//! ```

/// Files and paths.
use std::{fs, path::{Path, PathBuf}};

/// What:
/// `pub fn target_directories(checkout: &Path) -> Vec<PathBuf>` lists the `target` directories of a checkout that exist:
/// the root one and `package/<category>/<name>/target`.
/// Why:
/// They are where in-tree builds and run records live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function targetDirectories(checkout: string): string[]
/// ```
pub fn target_directories(checkout: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let root: PathBuf = checkout.join("target");
    if root.is_dir() {
        found.push(root);
    }
    let Ok(categories) = fs::read_dir(checkout.join("package")) else { return found };
    for category in categories.flatten() {
        let Ok(names) = fs::read_dir(category.path()) else { continue };
        for name in names.flatten() {
            let target: PathBuf = name.path().join("target");
            if target.is_dir() {
                found.push(target);
            }
        }
    }
    found.sort();
    return found;
}

/// What:
/// `pub fn verification_directories(targets: &[PathBuf]) -> Vec<PathBuf>` lists the `verification` directories inside targets.
/// Why:
/// They hold the run records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const verificationDirectories = (targets: string[]): string[]
/// ```
pub fn verification_directories(targets: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for target in targets {
        let directory: PathBuf = target.join("verification");
        if directory.is_dir() {
            found.push(directory);
        }
    }
    return found;
}
