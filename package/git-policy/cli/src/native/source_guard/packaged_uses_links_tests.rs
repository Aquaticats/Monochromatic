//! What:
//!  The published package holds no second copy of the tree under a `uses/` link.
//! Why:
//!  A `uses/` link is a folder-tree aid (`AGENTS.md`,
//!  rule `LYD`).
//!  `cargo package --list` follows every link and packs the files below it,
//!  so the two links in `worktree_clone_mount/uses` alone add 195 files and 2056871 bytes
//!  that the package already holds under their real paths.
//!  The crate is releasable (`package/crate-release/cli/releasable-crates.json`)
//!  and the release runs `cargo publish --dry-run`,
//!  so the cost would land on the next publish.
//!  `Cargo.toml`'s `include` list therefore negates `src/**/uses/**`,
//!  the pattern the ZFS binding crate carries since step 1 of the worktree plan.
//!  This control reads the list cargo itself prints,
//!  so removing that negation fails here rather than at a release.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listed = execFileSync('cargo', ['package', '--list', '--offline', '--locked', '--allow-dirty']);
//! expect(listed.split('\n').filter((path) => path.split('/').includes('uses'))).toEqual([]);
//! ```

use std::path::Path;
use std::process::Command;

/// The listed package paths that lie under a folder named `uses`,
/// which is where a `uses/` link's target lands in the archive.
fn uses_entries(listing: &str) -> Vec<String> {
    return listing.lines().filter(|line| { return line.split('/').any(|segment| return segment == "uses"); }).map(String::from).collect();
}

/// What:
///  Every path `cargo package --list` prints for this crate,
///  which is what `cargo publish` would upload.
/// Why:
///   The list is cargo's own answer,
///  so the control fails on the package cargo would build rather than on this module's reading of `include`.
fn package_listing() -> Vec<String> {
    let manifest: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output: std::process::Output = Command::new("cargo")
        .args(["package", "--list", "--offline", "--locked", "--allow-dirty", "--manifest-path"])
        .arg(&manifest)
        .output()
        .expect("cargo runs: the package list is the answer this control judges");
    assert!(
        output.status.success(),
        "cargo package --list failed for {}: {}",
        manifest.display(),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    return String::from_utf8(output.stdout).expect("cargo prints UTF-8 paths").lines().map(String::from).collect();
}

/// What:
///  The package cargo would publish holds no path under a `uses` folder,
///  and the list it was read from holds the crate's own sources.
/// Why:
///  A link packed below its name duplicates 195 files,
///  and an empty list would pass the first assertion without proving cargo answered.
#[test]
fn the_package_carries_no_copy_of_the_tree_under_a_uses_link() {
    let listed: Vec<String> = package_listing();
    assert!(
        listed.iter().any(|path| { return path == "src/native/lib.rs"; }),
        "positive control: the list holds the crate's own sources, and it holds {} paths",
        listed.len()
    );
    let found: Vec<String> = uses_entries(listed.join("\n").as_str());
    assert!(
        found.is_empty(),
        "cargo would publish {} copies below a `uses/` link ({:?}); keep `!src/**/uses/**` in Cargo.toml's include list",
        found.len(),
        found.first()
    );
}

/// A planted list holding a link's copy is read as one,
/// so the control above is able to fail.
#[test]
fn a_listed_path_under_a_uses_folder_is_found() {
    let planted: &str = "Cargo.toml\nsrc/native/lib.rs\nsrc/native/worktree_clone_mount/uses/foundation/diagnostic_log.rs\nsrc/native/worktree_clone_mount/uses/subvolume/fs_beneath.rs\n";
    assert_eq!(
        uses_entries(planted),
        vec![
            String::from("src/native/worktree_clone_mount/uses/foundation/diagnostic_log.rs"),
            String::from("src/native/worktree_clone_mount/uses/subvolume/fs_beneath.rs"),
        ]
    );
}

/// A planted list of real sources holds nothing under a `uses` folder,
/// including a file whose own name carries the word.
#[test]
fn a_listed_path_outside_a_uses_folder_is_not_found() {
    let planted: &str = "Cargo.toml\nsrc/native/lib.rs\nsrc/native/source_guard/packaged_uses_links_tests.rs\n";
    assert!(uses_entries(planted).is_empty(), "{}", uses_entries(planted).join("\n"));
}
