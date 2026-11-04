//! The module tree of a crate: which source files the compiler reads from the crate roots, and which of them are
//! compiled for tests only.
//! A file is test-only when every declaration that reaches it sits under a `cfg` that compiles only when testing, so
//! a nested helper inherits the gate of the module that declares it. Identity is the path relative to the source
//! root, never the file name, because two modules in different directories may share a file name.

use std::collections::BTreeMap;
use std::path::Path;

use super::declaration::{Declaration, declarations};
use super::walk::discover;

/// What:
///  `pub(crate) struct CrateRoot { ... }` names one file the compiler starts from, relative to the source root, and
///  whether it is built only for tests.
/// Why:
///  Roots come from the manifest (`lib`, `bin`, `test` targets); everything else is reached by `mod` declarations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CrateRoot = { path: string; testOnly: boolean }
/// ```
#[derive(Debug)]
pub(crate) struct CrateRoot {
    /// The root file, `/`-separated, relative to the source root.
    pub(crate) path: String,
    /// Whether the target is built only when testing.
    pub(crate) test_only: bool,
}

/// What:
///  `pub(crate) struct SourceFile { ... }` is one file of the tree: its relative identity and its classification.
/// Why:
///  The scans read the identity for findings and exceptions, and the classification to skip test-only support.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceFile = { path: string; testOnly: boolean }
/// ```
#[derive(Debug)]
pub(crate) struct SourceFile {
    /// The file, `/`-separated, relative to the source root.
    pub(crate) path: String,
    /// Whether the file is compiled only when testing.
    pub(crate) test_only: bool,
}

/// What:
///  `fn join_names(directory: &str, part: &str) -> Result<String, String>` joins a relative `directory` and a
///  relative `part` (as written in a `#[path]` attribute), folding `.` and `..` away.
/// Why:
///  Identities must be one spelling per file, and a path that climbs above the source root leaves the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joinNames(directory: string, part: string): string  // throws Error
/// ```
fn join_names(directory: &str, part: &str) -> Result<String, String> {
    let mut names: Vec<&str> = Vec::new();
    for piece in directory.split('/').chain(part.split('/')) {
        if piece.is_empty() || piece == "." {
            continue;
        }
        if piece == ".." {
            // `pop()` gives `None` on an empty list, which means the path climbs out of the root.
            if names.pop().is_none() {
                return Err(format!("`{part}` in `{directory}` leaves the source root"));
            }
            continue;
        }
        names.push(piece);
    }
    return Ok(names.join("/"));
}

/// What:
///  `fn parent_of(file: &str) -> &str` is the directory part of a `/`-separated name (`""` for a top-level file).
/// Why:
///  A `mod` declaration resolves relative to the directory of the file that holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parentOf(file: string): string
/// ```
fn parent_of(file: &str) -> &str {
    return file.rsplit_once('/').map_or("", |(parent, _)| return parent);
}

/// What:
///  `fn stem_of(file: &str) -> &str` is the file name without its directory and `.rs` suffix.
/// Why:
///  A file `engine.rs` that is not a `mod.rs` keeps its child modules in a directory named `engine`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stemOf(file: string): string
/// ```
fn stem_of(file: &str) -> &str {
    let name: &str = file.rsplit_once('/').map_or(file, |(_, name)| return name);
    return name.strip_suffix(".rs").unwrap_or(name);
}

/// What:
///  `fn locate(root: &Path, file: &str, owns_directory: bool, declaration: &Declaration)` finds the file a `mod`
///  declaration names, and whether that file keeps its own children beside it (`true`, like `mod.rs` or a file
///  loaded through `#[path]`) or in a directory named after it (`false`).
/// Why:
///  This is the compiler's own lookup rule, so a nested layout is resolved the way the build resolves it; a
///  declaration with no file, or with two candidate files, is an error because the tree would not be what compiles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(root: string, file: string, ownsDirectory: boolean, d: Declaration): { path: string; ownsDirectory: boolean }
/// ```
fn locate(root: &Path, file: &str, owns_directory: bool, declaration: &Declaration) -> Result<(String, bool), String> {
    let directory: &str = parent_of(file);
    if let Some(written) = &declaration.path_attribute {
        let target: String = join_names(directory, written.as_str())?;
        if !root.join(target.as_str()).is_file() {
            return Err(format!("{file}: `#[path = \"{written}\"]` names {target}, which is not a file"));
        }
        return Ok((target, true));
    }
    let module_directory: String = if owns_directory {
        directory.to_owned()
    } else {
        join_names(directory, stem_of(file))?
    };
    let plain: String = join_names(module_directory.as_str(), format!("{}.rs", declaration.name).as_str())?;
    let nested: String = join_names(module_directory.as_str(), format!("{}/mod.rs", declaration.name).as_str())?;
    let plain_exists: bool = root.join(plain.as_str()).is_file();
    let nested_exists: bool = root.join(nested.as_str()).is_file();
    if plain_exists && nested_exists {
        return Err(format!("{file}: `mod {};` matches both {plain} and {nested}", declaration.name));
    }
    if plain_exists {
        return Ok((plain, false));
    }
    if nested_exists {
        return Ok((nested, true));
    }
    return Err(format!("{file}: `mod {};` has neither {plain} nor {nested}", declaration.name));
}

/// What:
///  `pub(crate) fn module_tree(root, roots)` follows every `mod` declaration from the crate roots and returns each
///  reached file once, sorted, with its test-only classification.
/// Why:
///  A file reached by a release path and by a test-only path is release (the build compiles it either way, and the
///  scans must read it). A file reached only through gates is test-only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function moduleTree(root: string, roots: CrateRoot[]): SourceFile[]  // throws Error
/// ```
pub(crate) fn module_tree(root: &Path, roots: &[CrateRoot]) -> Result<Vec<SourceFile>, String> {
    // `BTreeMap<String, bool>` is a sorted map (TS: `Map`, but iterated in key order) from file to "test-only".
    let mut known: BTreeMap<String, bool> = BTreeMap::new();
    // The work list holds (file, test-only, owns-directory) for files still to read.
    let mut pending: Vec<(String, bool, bool)> = Vec::new();
    for crate_root in roots {
        pending.push((crate_root.path.clone(), crate_root.test_only, true));
    }
    while let Some((file, test_only, owns_directory)) = pending.pop() {
        // Read a file again only when it was first reached as test-only and is now reached as release.
        if known.get(file.as_str()).is_some_and(|earlier| return !*earlier || test_only) {
            continue;
        }
        known.insert(file.clone(), test_only);
        let text: String = std::fs::read_to_string(root.join(file.as_str()))
            .map_err(|error| return format!("cannot read {file}: {error}"))?;
        let found: Vec<Declaration> = declarations(text.as_str()).map_err(|error| return format!("{file}: {error}"))?;
        for declaration in found {
            let (child, child_owns): (String, bool) = locate(root, file.as_str(), owns_directory, &declaration)?;
            pending.push((child, test_only || declaration.test_only, child_owns));
        }
    }
    let mut tree: Vec<SourceFile> = Vec::new();
    for (path, test_only) in known {
        tree.push(SourceFile { path, test_only });
    }
    return Ok(tree);
}

/// What:
///  `pub(crate) fn classified_sources(root, roots)` is the whole classified listing of a source root: the module
///  tree, proven to cover every `.rs` file the strict walk finds.
/// Why:
///  A source file that no `mod` declaration reaches is either dead text or a registration that moved without its
///  file; either way a scan could not say whether it is release code, so the listing fails instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function classifiedSources(root: string, roots: CrateRoot[]): SourceFile[]  // throws Error
/// ```
pub(crate) fn classified_sources(root: &Path, roots: &[CrateRoot]) -> Result<Vec<SourceFile>, String> {
    let walked: Vec<String> = discover(root)?;
    let tree: Vec<SourceFile> = module_tree(root, roots)?;
    let mut unreached: Vec<String> = Vec::new();
    for path in walked {
        if !tree.iter().any(|file| return file.path == path) {
            unreached.push(path);
        }
    }
    if !unreached.is_empty() {
        return Err(format!("no `mod` declaration reaches: {}", unreached.join(", ")));
    }
    return Ok(tree);
}

/// What:
///  `pub(crate) fn manifest_roots(manifest: &str)` reads the crate roots out of `Cargo.toml` text: the `[lib]` and
///  `[[bin]]` paths are release roots, the `[[test]]`, `[[bench]]` and `[[example]]` paths are test-only roots. Only
///  paths under `src_prefix` count, and the prefix is cut off.
/// Why:
///  The scans start from the targets the build has, not from a list that could drift from the manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function manifestRoots(manifest: string, srcPrefix: string): CrateRoot[]  // throws Error
/// ```
pub(crate) fn manifest_roots(manifest: &str, src_prefix: &str) -> Result<Vec<CrateRoot>, String> {
    let mut roots: Vec<CrateRoot> = Vec::new();
    let mut section: &str = "";
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            section = trimmed;
            continue;
        }
        let Some(value) = trimmed.strip_prefix("path = \"").and_then(|rest| return rest.strip_suffix('"')) else {
            continue;
        };
        let test_only: bool = matches!(section, "[[test]]" | "[[bench]]" | "[[example]]");
        let release: bool = matches!(section, "[lib]" | "[[bin]]");
        let Some(relative) = value.strip_prefix(src_prefix) else {
            if test_only || release {
                return Err(format!("{section} path `{value}` is not under {src_prefix}"));
            }
            continue;
        };
        if test_only || release {
            roots.push(CrateRoot { path: relative.to_owned(), test_only });
        }
    }
    if roots.is_empty() {
        return Err(String::from("the manifest names no target under the source prefix"));
    }
    return Ok(roots);
}
