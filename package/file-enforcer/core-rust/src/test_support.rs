//! What:
//!  Disposable worktrees and small helpers shared by the engine tests.
//! Why:
//!  Tests must never touch a real worktree,
//! and scratch must live under the build directory,
//! not `/tmp`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scratchWorktree(files: Record<string, string>): { root: string }
//! ```

/// Imports `CollectingSink` from `crate::event`.
use crate::event::CollectingSink;
/// Imports `Event` from `crate::event`.
use crate::event::Event;
/// Imports `RuleRegistry` from `crate::registry`.
use crate::registry::RuleRegistry;
/// Imports `RunMode`,
/// `RunRequest`,
/// `run` from `crate::run`.
use crate::run::{RunMode, RunRequest, run};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `TempDir` from `tempfile`.
use tempfile::TempDir;

/// What:
///  The parent directory of every scratch tree:
/// `target/test-scratch` under this package.
/// Why:
///  `clippy.toml` bans `std::env::temp_dir`;
/// the build directory is the sanctioned place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratchBase(): string
/// ```
pub fn scratch_base() -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("test-scratch");
    fs::create_dir_all(&base).expect("the scratch base is creatable");
    return base;
}

/// What:
///  Create a scratch directory,
/// deleted when the returned guard drops.
/// Why:
///  Each test gets its own tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratchDir(): TempDir
/// ```
pub fn scratch_dir() -> TempDir {
    return TempDir::new_in(scratch_base()).expect("a scratch directory is creatable");
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
///  Run a config text against a root in the given mode with the default registry;
/// return the exit code and events.
/// Why:
///  The tests drive the same entry point the server uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runConfig(root: string, config: string, mode: RunMode): [number, Event[]]
/// ```
pub fn run_config(root: &Path, config: &str, mode: RunMode) -> (i32, Vec<Event>) {
    let config_path = root.join("file-enforcer.config.jsonc");
    fs::write(&config_path, config).expect("the config is writable");
    let request = RunRequest { root: root.to_path_buf(), config_path, scope: crate::config_decode::ConfigScope::Repository, mode, outputs: crate::output_scope::OutputScope::All };
    let mut sink = CollectingSink::default();
    let code = run(&request, &RuleRegistry::with_defaults(), &mut sink);
    return (code, sink.events);
}

/// What:
///  Count events with a given code.
/// Why:
///  Assertions on drift and warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countCode(events: Event[], code: string): number
/// ```
pub fn count_code(events: &[Event], code: &str) -> usize {
    return events.iter().filter(|event| return event.code == code).count();
}
