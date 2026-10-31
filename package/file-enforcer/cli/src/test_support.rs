//! What:
//!  Disposable trees and run helpers shared by the rule-kind tests.
//! Why:
//!  The rule kinds are tested through the same `run` entry point the server uses,
//! in scratch worktrees under the build directory
//!  (`clippy.toml` bans the system temporary directory),
//! with a registry the test controls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function runConfig(root: string, config: string, mode: RunMode, registry: RuleRegistry): [number, Event[]]
//! ```

/// Imports `ConfigScope` from `monochromatic_file_enforcer::config_decode`.
use monochromatic_file_enforcer::config_decode::ConfigScope;
/// Imports `CollectingSink`,
/// `Event` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::{CollectingSink, Event};
/// Imports `OutputScope` from `monochromatic_file_enforcer::output_scope`.
use monochromatic_file_enforcer::output_scope::OutputScope;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;
/// Imports `RunMode`,
/// `RunRequest`,
/// `run` from `monochromatic_file_enforcer::run`.
use monochromatic_file_enforcer::run::{RunMode, RunRequest, run};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `TempDir` from `tempfile`.
use tempfile::TempDir;

/// What:
///  Create a scratch directory under `target/test-scratch`,
/// deleted when the guard drops.
/// Why:
///  Each test gets its own tree on a known device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratchDir(): TempDir
/// ```
pub fn scratch_dir() -> TempDir {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("test-scratch");
    fs::create_dir_all(&base).expect("the scratch base is creatable");
    return TempDir::new_in(base).expect("a scratch directory is creatable");
}

/// What:
///  Write files (path to text) under a root,
/// creating directories.
/// Why:
///  Compact fixtures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeFiles(root: string, files: [string, string][]): void
/// ```
pub fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let target = root.join(path);
        fs::create_dir_all(target.parent().expect("a file has a parent")).expect("directories are creatable");
        fs::write(&target, text).expect("fixture files are writable");
    }
}

/// What:
///  Read a file under a root as text.
/// Why:
///  Assertions read published files back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readText(root: string, path: string): string
/// ```
pub fn read_text(root: &Path, path: &str) -> String {
    return fs::read_to_string(root.join(path)).expect("the fixture file is readable");
}

/// What:
///  Run a config text against a root in the given scope and mode with a chosen registry;
/// return the exit code and events.
/// Why:
///  The tests drive the entry point the server uses,
/// with the real builders or with a test renderer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runConfig(options: { root: string; config: string; scope: ConfigScope; mode: RunMode; registry: RuleRegistry }): [number, Event[]]
/// ```
pub fn run_config(root: &Path, config: &str, scope: ConfigScope, mode: RunMode, registry: &RuleRegistry) -> (i32, Vec<Event>) {
    let config_path: PathBuf = root.join("file-enforcer.config.jsonc");
    fs::write(&config_path, config).expect("the config is writable");
    let request = RunRequest { root: root.to_path_buf(), config_path, scope, mode, outputs: OutputScope::All };
    let mut sink = CollectingSink::default();
    let code = run(&request, registry, &mut sink);
    return (code, sink.events);
}

/// What:
///  Count events with a given code.
/// Why:
///  Assertions on drift,
/// warnings and errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countCode(events: Event[], code: string): number
/// ```
pub fn count_code(events: &[Event], code: &str) -> usize {
    return events.iter().filter(|event| return event.code == code).count();
}
