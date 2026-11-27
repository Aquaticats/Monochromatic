//! What:
//!  The binary-boundary integration suite of `forbidden-strings`.
//!  Each test spawns the built binary as a subprocess and asserts on its
//!  stdout,
//!  stderr and exit code;
//!  Cargo's `tests/*.rs` layout compiles this one file as one test
//!  executable that links the binary beside it,
//!  and the compile-time `CARGO_BIN_EXE_forbidden-strings` names that copy.
//! Why:
//!  Several defects live in `main.rs` (read-error handling,
//!  path skip logic,
//!  the `--all` walker integration) and only show at the binary boundary;
//!  unit tests on internal helpers would miss the wiring.
//!  The suite outgrew one readable file,
//!  so the tests sit in the role-named modules of this folder,
//!  each a plain `mod` of this crate rather than its own test executable,
//!  which keeps one process,
//!  one harness and one cache root per run.
//!  This file holds the harness every part spawns through,
//!  then lists the parts in the order they build on it.
//!  Each part needs `#[path]` because a crate root always looks for its
//!  modules beside itself,
//!  in `tests/`,
//!  not in this folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // integration.test.ts: the harness, then the role modules below
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

// What:     `#[path = "..."] mod blocked_root;` compiles the named file as
//           a private module of this test crate.
// Why:      The embedding-warning suite includes the same file, so both
//           assert one platform-specific reason for a cache root blocked
//           by a regular file.
//
// In TS you'd write (pseudocode):
// ```ts
// import * as blockedRoot from './support/blocked_root.ts';
// ```
#[path = "support/blocked_root.rs"]
mod blocked_root;

// What:     `const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");`
//           uses the compile-time env var Cargo sets for integration
//           tests of a binary crate. The value is the absolute path to
//           the freshly built binary; `env!` panics at compile time if
//           the env var is absent (it never is for this crate's tests).
// Why:      Avoid hardcoding `target/release/forbidden-strings` -- that
//           breaks under `cargo test` (debug profile) and on platforms
//           where the path layout differs.
//
// In TS you'd write (pseudocode):
// ```ts
// const BIN = process.env.CARGO_BIN_EXE_forbidden_strings!;
// ```
const BIN: &str = env!("CARGO_BIN_EXE_forbidden-strings");

/// Test command factory injecting one disposable process-scoped cache root.
struct Command;

/// Builds subprocess commands without touching developer's real user cache.
impl Command {
    /// Creates process command with hermetic cache-root override.
    fn configured(program: impl AsRef<std::ffi::OsStr>) -> ProcessCommand {
        static CACHE_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        let cache_root = CACHE_ROOT.get_or_init(|| {
            let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
                "forbidden-strings-integration-cache-{}",
                std::process::id(),
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create integration cache root");
            return path;
        });
        let mut command = ProcessCommand::new(program);
        command.env("FORBIDDEN_STRINGS_CACHE_DIR", cache_root);
        return command;
    }
}

// What:     `fn unique_tmp(label) -> PathBuf` returns a fresh empty
//           directory under `CARGO_TARGET_TMPDIR`. Uses PID + label so
//           parallel test runs do not collide.
// Why:      Tests create rules files and target files; sharing a dir
//           across tests is a flake source.
//
// In TS you'd write (pseudocode):
// ```ts
// function uniqueTmp(label: string): string {
//   return path.join(os.tmpdir(), `${label}-${process.pid}`);
// }
// ```
fn unique_tmp(label: &str) -> PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "fs-int-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tmp dir");
    return dir;
}

// What:     `#[path = "integration/read_error_tests.rs"] mod read_error_tests;`
//           declares the module asserting that a file the scanner cannot read
//           is a finding, not a silent pass.
// Why:      A secret-scanning gate that exits clean on an unreadable file is a
//           false-clean result.
#[path = "integration/read_error_tests.rs"]
mod read_error_tests;
// What:     `#[path = "integration/binary_probe_tests.rs"]
//           mod binary_probe_tests;` declares the module that pins the 8 KiB
//           probe cap for files holding a NUL byte.
// Why:      Those two rows of the design table (scan the probe, skip the tail)
//           have no other test at this boundary.
#[path = "integration/binary_probe_tests.rs"]
mod binary_probe_tests;
// What:     `#[path = "integration/file_selection_tests.rs"]
//           mod file_selection_tests;` declares the module deciding which paths
//           a run scans and which it skips.
// Why:      A skip that is too broad hides real findings, and one that is too
//           narrow makes a rules file match itself.
#[path = "integration/file_selection_tests.rs"]
mod file_selection_tests;
// What:     `#[path = "integration/finding_shape_tests.rs"]
//           mod finding_shape_tests;` declares the module that pins the
//           columnless `PATH:LINE rule=N` finding text and rule attribution.
// Why:      CI consumers parse that shape, and the matched text must never
//           surface.
#[path = "integration/finding_shape_tests.rs"]
mod finding_shape_tests;
// What:     `#[path = "integration/help_and_version_tests.rs"]
//           mod help_and_version_tests;` declares the module covering the
//           `--help` and `--version` output channels and exit codes.
// Why:      Wrappers and package managers branch on them.
#[path = "integration/help_and_version_tests.rs"]
mod help_and_version_tests;
// What:     `#[path = "integration/rules_flag_tests.rs"] mod rules_flag_tests;`
//           declares the module covering where the rules list comes from and
//           how a bad invocation is refused.
// Why:      The precedence and the exit-2 usage channel are documented
//           behavior that CI depends on.
#[path = "integration/rules_flag_tests.rs"]
mod rules_flag_tests;
// What:     `#[path = "integration/builtin_baseline_tests.rs"]
//           mod builtin_baseline_tests;` declares the module covering the
//           embedded betterleaks baseline and the `curl-auth-user` rule.
// Why:      The baseline is pure opt-in, and its rule ids must not shift the
//           user's own numbering.
#[path = "integration/builtin_baseline_tests.rs"]
mod builtin_baseline_tests;
// What:     `#[path = "integration/runtime_cache_and_compile_rules_tests.rs"]
//           mod runtime_cache_and_compile_rules_tests;` declares the module
//           covering the content-addressed artifact cache and the
//           `compile-rules` subcommand that fills it.
// Why:      A cache that reports a false clean, or a warning that leaks a
//           path, is a product defect; both only show at this boundary.
#[path = "integration/runtime_cache_and_compile_rules_tests.rs"]
mod runtime_cache_and_compile_rules_tests;
