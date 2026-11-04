//! What: Which repository paths the dependent-version planner reads: workspace manifests,
//!       the registry configuration, and a package's non-test source files.
//! Why: The planner selects these from the tracked-path list itself, so the provider needs
//!      no pathspec support. The rules are the incumbent's: `isWorkspaceManifestPath`
//!      (`dependent-version-bump-policy.ts:66-73`), the manifest pathspec
//!      `:(glob)package/*/*/package.json`, `PNPR_CONFIG_PATH`
//!      (`publishable-names.ts:10`) and `isNonTestSourcePath` (`source-imports.ts:38-55`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // isWorkspaceManifestPath(path); packageDirectory(manifestPath); isNonTestSourcePath({ directory, path });
//! ```

/// What: The generated registry configuration whose OIDC package list is the publish set.
///       `&[u8]` is a borrowed list of bytes baked into the program.
/// Why:  Only packages listed there receive a dependent bump.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PNPR_CONFIG_PATH = 'package/config/pnpr/config.yaml';
/// ```
pub const PNPR_CONFIG_PATH: &[u8] = b"package/config/pnpr/config.yaml";

/// What: The file name every workspace manifest has, with the separator before it.
/// Why:  A package's directory is its manifest path without this suffix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MANIFEST_SUFFIX = '/package.json';
/// ```
const MANIFEST_SUFFIX: &[u8] = b"/package.json";

/// What: Extensions of source files a bundler follows imports from.
/// Why:  A development dependency counts as bundled only when such a file imports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SOURCE_EXTENSIONS = ['.ts', '.tsx', '.mts', '.cts', '.js', '.jsx', '.mjs', '.cjs'];
/// ```
const SOURCE_EXTENSIONS: &[&[u8]] = &[
    b".ts", b".tsx", b".mts", b".cts", b".js", b".jsx", b".mjs", b".cjs",
];

/// What: The file-name segment that marks a test file.
/// Why:  Test files are not bundled, so their imports decide nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEST_MARKER = '.test.';
/// ```
const TEST_MARKER: &[u8] = b".test.";

/// What: Whether a path is `package/<category>/<name>/package.json`. `.split(...)` yields the
///       `/`-separated segments; `Vec<&[u8]>` collects borrowed views of them.
/// Why:  Only such a manifest is a workspace package; the incumbent's candidate test and
///       its tracked-file pathspec both select exactly these paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const segments = path.split('/'); return segments.length === 4 && segments[0] === 'package' && segments[3] === 'package.json';
/// ```
pub fn is_workspace_manifest_path(path: &[u8]) -> bool {
    let segments: Vec<&[u8]> = path.split(|byte: &u8| return *byte == b'/').collect();
    // `matches!` tests a value against a pattern; the slice pattern names all four segments.
    return matches!(segments.as_slice(), [b"package", _, _, b"package.json"]);
}

/// What: The package directory of a workspace manifest path. `.strip_suffix(...)` returns
///       the bytes before the suffix, or nothing when the path does not end with it.
/// Why:  Source files are looked up under `<directory>/src/`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = manifestPath.slice(0, -'/package.json'.length);
/// ```
pub fn package_directory(manifest_path: &[u8]) -> &[u8] {
    return manifest_path
        .strip_suffix(MANIFEST_SUFFIX)
        .unwrap_or(manifest_path);
}

/// What: Whether a path is non-test source of the package in `directory`.
/// Why:  The file must sit under the package's `src/`, end with a source extension, and
///       its name must not contain `.test.`. A sibling directory sharing a prefix
///       (`package/module/ab` for `package/module/a`) does not count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return path.startsWith(`${directory}/src/`) && SOURCE_EXTENSIONS.some(e => name.endsWith(e)) && !name.includes('.test.');
/// ```
pub fn is_non_test_source_path(directory: &[u8], path: &[u8]) -> bool {
    // `[a, b].concat()` joins two byte lists into one owned list.
    let prefix: Vec<u8> = [directory, b"/src/"].concat();
    if !path.starts_with(&prefix) {
        return false;
    }
    // `.rsplit(...)` yields segments from the end, so its first item is the file name.
    let name: &[u8] = path
        .rsplit(|byte: &u8| return *byte == b'/')
        .next()
        .unwrap_or(path);
    let has_extension: bool = SOURCE_EXTENSIONS
        .iter()
        .any(|extension: &&[u8]| return name.ends_with(extension));
    // `.windows(n)` yields every run of `n` consecutive bytes.
    let is_test: bool = name
        .windows(TEST_MARKER.len())
        .any(|window: &[u8]| return window == TEST_MARKER);
    return has_extension && !is_test;
}

/// Manifest, configuration and source path selection.
#[cfg(test)]
#[path = "dependent_version_paths_tests.rs"]
mod tests;
