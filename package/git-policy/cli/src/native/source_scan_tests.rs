//! The shared base of the checks that read this crate's own source text.
//! A check that lists sources by looking at the immediate entries of one directory, or that tells test support from
//! release code by file name, stops being right the day modules move into subdirectories. These helpers fix the
//! two questions once: which files exist (a strict walk that cannot return less than the tree holds) and which of them
//! the build compiles for tests only (the module tree from the manifest's targets). The identity of a source is its
//! `/`-separated path relative to `src/native`.

use std::path::{Path, PathBuf};

/// The declarations of one file: the `mod name;` lines with their `path` and `cfg(test)` attributes.
#[path = "source_scan_declaration_tests.rs"]
pub(crate) mod declaration;

/// The module tree and the classified listing built from it.
#[path = "source_scan_tree_tests.rs"]
pub(crate) mod tree;

/// The strict walk of a source root.
#[path = "source_scan_walk_tests.rs"]
pub(crate) mod walk;

/// Controls of the helpers themselves, on filesystem fixtures.
#[path = "source_scan_checks_tests.rs"]
mod checks;

pub(crate) use declaration::module_name;
pub(crate) use tree::{CrateRoot, SourceFile, classified_sources, manifest_roots, module_tree};

/// The source root every identity is relative to, as the manifest spells it.
pub(crate) const SOURCE_PREFIX: &str = "src/native/";

/// What:
///  `pub(crate) fn source_root() -> PathBuf` is the directory of this crate's own sources, `src/native`.
///  `env!("CARGO_MANIFEST_DIR")` is a compile-time text the build fills in with the crate's directory.
/// Why:
///  Every check of the real crate starts from the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function sourceRoot(): string { return join(manifestDir, 'src', 'native') }
/// ```
pub(crate) fn source_root() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
}

/// What:
///  `pub(crate) fn real_roots() -> Vec<CrateRoot>` are the roots of this crate's targets as `Cargo.toml` lists them.
/// Why:
///  A new binary or test target is covered the moment the manifest has it; there is no second list to forget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function realRoots(): CrateRoot[]
/// ```
pub(crate) fn real_roots() -> Vec<CrateRoot> {
    let manifest: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("the crate manifest");
    return manifest_roots(manifest.as_str(), SOURCE_PREFIX).expect("the manifest's targets under src/native");
}

/// What:
///  `pub(crate) fn write_tree(root: &Path, files: &[(&str, &str)])` writes each `(relative name, text)` pair as a
///  file under `root`, creating the directories between.
/// Why:
///  Fixture trees for the scans are built from a few lines of text, so each control shows its layout in the test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeTree(root: string, files: [string, string][]): void
/// ```
pub(crate) fn write_tree(root: &Path, files: &[(&str, &str)]) {
    for (name, text) in files {
        let path: PathBuf = root.join(name);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("fixture directory");
        std::fs::write(&path, text).expect("fixture file");
    }
}

/// What:
///  `pub(crate) fn release_root(path: &str) -> CrateRoot` is a release-build root at `path`.
/// Why:
///  Fixtures name their one root without spelling out the record each time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const releaseRoot = (path: string): CrateRoot => ({ path, testOnly: false })
/// ```
pub(crate) fn release_root(path: &str) -> CrateRoot {
    return CrateRoot { path: path.to_owned(), test_only: false };
}
