//! What:
//!  Deterministic configuration-discovery tests.
//! Why:
//!  A memory filesystem avoids depending on any host-wide configuration during ancestor lookup.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('config discovery', () => { /* nearest, explicit, absence, error controls */ });
//! ```

/// Import the same adapter interface and discovery entry used by production.
use super::{
    CONFIG_NAME, ConfigFilesystem, NativeConfigFilesystem, discover_configuration,
    discover_with_filesystem,
};
use crate::config_error::ConfigError;
use crate::test_fs::Fixture;
/// Import owned lookup storage and native path values.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What:
///  A test filesystem with explicit file contents and read failures.
/// Why:
///  Absent keys mean absence rather than a fallback to the host filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MemoryFilesystem { files = new Map<Path, string>(); failures = new Set<Path>(); }
/// ```
#[derive(Default)]
struct MemoryFilesystem {
    /// Source texts supplied by the test.
    files: BTreeMap<PathBuf, String>,
    /// Paths that fail instead of appearing absent.
    failures: BTreeSet<PathBuf>,
}

/// What:
///  Supply controlled reads for the production discovery algorithm.
/// Why:
///  Error-versus-absence behavior is observable without platform permission assumptions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// read(path: Path) { if (failures.has(path)) throw new Error('...'); return files.get(path); }
/// ```
impl ConfigFilesystem for MemoryFilesystem {
    /// Read an explicitly supplied fixture or preserve the missing-file signal.
    fn read(&self, path: &Path) -> Result<Option<String>, ConfigError> {
        if self.failures.contains(path) {
            return Err(ConfigError::new("fixture configuration read failed"));
        }
        return Ok(self.files.get(path).cloned());
    }
}

/// Native temporary-directory paths supply the platform's absolute-root grammar without reading files.
fn root() -> PathBuf {
    return crate::test_scratch::scratch_root().join("monochromatic-lint-memory-config");
}

/// The native adapter reads only owned fixture paths and classifies real I/O failures.
#[test]
fn native_adapter_distinguishes_file_absence_and_failures() {
    let fixture = Fixture::new();
    let adapter = NativeConfigFilesystem;
    let source = fixture.path.join("selected.jsonc");
    assert_eq!(adapter.read(&source).expect("missing file"), None);
    std::fs::write(&source, "[]").expect("write source");
    assert_eq!(
        adapter.read(&source).expect("read source").as_deref(),
        Some("[]")
    );
    assert_eq!(
        adapter
            .read(&source.join("child"))
            .expect("not a directory is absent"),
        None
    );
    let directory_error = adapter
        .read(&fixture.path)
        .expect_err("directory is not a file");
    assert!(directory_error.message.contains("not a regular file"));
    std::fs::write(&source, [0xff]).expect("write invalid UTF-8");
    let encoding_error = adapter.read(&source).expect_err("invalid source encoding");
    assert!(encoding_error.message.contains("Cannot read configuration"));
    let path_error = adapter
        .read(&fixture.path.join("invalid\0name"))
        .expect_err("NUL path is rejected");
    assert!(path_error.message.contains("Cannot inspect configuration"));
}

/// The production entry point uses the native adapter and preserves an explicit config's base.
#[test]
fn production_entry_reads_explicit_configuration() {
    let fixture = Fixture::new();
    let source = fixture.path.join("selected.jsonc");
    std::fs::write(&source, "[{\"files\":[\"**/*.md\"]}]").expect("write source");
    let selected = discover_configuration(Path::new("README.md"), &fixture.path, Some(&source))
        .expect("native lookup")
        .expect("selected config");
    assert_eq!(selected.path, source);
    assert_eq!(selected.base, fixture.path);
}

/// The nearest configuration is used alone and its directory becomes the pattern base.
#[test]
fn nearest_configuration_wins_without_merging_ancestors() {
    let base = root();
    let child = base.join("child");
    let mut filesystem = MemoryFilesystem::default();
    filesystem.files.insert(
        base.join(CONFIG_NAME),
        "[{\"name\":\"outer\",\"files\":[\"**/*.rs\"]}]".to_string(),
    );
    filesystem.files.insert(
        child.join(CONFIG_NAME),
        "[{\"name\":\"inner\",\"files\":[\"*.rs\"]}]".to_string(),
    );
    let selected = discover_with_filesystem(&child.join("file.rs"), &base, None, &filesystem)
        .expect("lookup")
        .expect("configuration");
    assert_eq!(selected.path, child.join(CONFIG_NAME));
    assert_eq!(selected.base, child);
    assert_eq!(selected.blocks.len(), 1);
    assert_eq!(selected.blocks[0].name.as_deref(), Some("inner"));
}

/// An explicit outside config bypasses discovery and resolves patterns against cwd.
#[test]
fn explicit_configuration_uses_working_directory_as_base() {
    let base = root();
    let outside = base.join("other").join("one-rule.jsonc");
    let mut filesystem = MemoryFilesystem::default();
    filesystem
        .files
        .insert(base.join(CONFIG_NAME), "not JSONC".to_string());
    filesystem
        .files
        .insert(outside.clone(), "[{\"files\":[\"**/*.md\"]}]".to_string());
    let selected =
        discover_with_filesystem(Path::new("doc/file.md"), &base, Some(&outside), &filesystem)
            .expect("override")
            .expect("configuration");
    assert_eq!(selected.path, outside);
    assert_eq!(selected.base, base);
}

/// Relative file and override paths resolve from the injected cwd,
///  not the process's cwd.
#[test]
fn relative_paths_resolve_from_injected_cwd() {
    let base = root();
    let config = base.join("selected.jsonc");
    let mut filesystem = MemoryFilesystem::default();
    filesystem.files.insert(config.clone(), "[]".to_string());
    let selected = discover_with_filesystem(
        Path::new("nested/file.rs"),
        &base,
        Some(Path::new("selected.jsonc")),
        &filesystem,
    )
    .expect("relative override")
    .expect("configuration");
    assert_eq!(selected.path, config);
    assert_eq!(selected.base, base);
}

/// Missing discovered config is absence;
///  a missing explicit config is a setup error.
#[test]
fn missing_and_explicit_missing_are_distinct() {
    let base = root();
    let filesystem = MemoryFilesystem::default();
    assert!(
        discover_with_filesystem(Path::new("src/file.rs"), &base, None, &filesystem)
            .expect("absence")
            .is_none()
    );
    let error = discover_with_filesystem(
        Path::new("src/file.rs"),
        &base,
        Some(Path::new("missing.jsonc")),
        &filesystem,
    )
    .expect_err("explicit absence");
    assert!(error.message.contains("does not exist"));
}

/// A malformed nearest config cannot silently fall back to a valid parent.
#[test]
fn invalid_nearest_configuration_does_not_fall_back() {
    let base = root();
    let child = base.join("child");
    let mut filesystem = MemoryFilesystem::default();
    filesystem
        .files
        .insert(base.join(CONFIG_NAME), "[]".to_string());
    filesystem
        .files
        .insert(child.join(CONFIG_NAME), "{}".to_string());
    let error = discover_with_filesystem(&child.join("file.rs"), &base, None, &filesystem)
        .expect_err("nearest schema failure");
    assert!(error.message.contains("ordered array"));
    assert!(error.message.contains("child"));
}

/// Read failures are not absence,
///  and relative cwd is rejected before filesystem access.
#[test]
fn read_failures_and_relative_cwd_are_errors() {
    let base = root();
    let mut filesystem = MemoryFilesystem::default();
    filesystem.failures.insert(base.join(CONFIG_NAME));
    let read_error = discover_with_filesystem(Path::new("file.rs"), &base, None, &filesystem)
        .expect_err("read failure");
    assert!(read_error.message.contains("read failed"));
    let cwd_error = discover_with_filesystem(
        Path::new("file.rs"),
        Path::new("relative"),
        None,
        &filesystem,
    )
    .expect_err("relative cwd");
    assert!(cwd_error.message.contains("absolute working directory"));
}
