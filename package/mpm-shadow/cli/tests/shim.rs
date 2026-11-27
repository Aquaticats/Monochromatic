//! What:
//! End-to-end tests of the built `mpm-shadow` binary,
//! in six parts,
//! each in a file of its own beside this one.
//! This file holds how every part starts the binary and the disposable cargo home it starts it against:
//! the built binary is linked as `mpm`,
//! a
//!           fake "real" `mpm` script answers from `PATH`,
//! and the toolchain roots point into the fixture.
//! Why:
//! The unit tests prove the pieces;
//! only a real process shows that the arguments arrive untouched,
//! the exit
//!           status passes through,
//! and the stale copies are gone afterwards.
//! A part that started its own copy of this fixture would let the parts disagree about what the binary sees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.test.ts
//! import { fixtureRoot, writeFakeMpm, startShadow } from './shim/fixture.ts';
//! await import('./shim/arguments_and_the_exit_code_pass_through.ts');
//! // ...one import per part...
//! ```

/// Imports the file-system module (writes the fake mpm,
/// makes links and fixtures).
use std::fs;

/// Imports the borrowed path type.
use std::path::Path;
use std::process::{
    Command,
    Output,
};

/// The writer of the program files these tests start,
/// shared with the crate's other test binaries.
#[path = "written_programs/through_a_child_process.rs"]
mod through_a_child_process;

/// What:
/// `fn fixture_root() -> tempfile::TempDir` declares a helper returning a disposable directory that is
///           removed when the returned guard drops.
/// Why:
/// `env!("CARGO_TARGET_TMPDIR")` is a compile-time macro (the `!`) Cargo sets for integration tests:
/// a
///           directory in the build output,
/// so no test writes under the system temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = fs.mkdtempSync(path.join(TARGET_TMPDIR, 'mpm-shadow-'));
/// ```
fn fixture_root() -> tempfile::TempDir {
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
}

/// What:
/// `fn write_fake_mpm(directory: &Path, script: &str)` declares a helper writing an executable `mpm` script.
/// Why:
/// The shadow must find and run a program named `mpm` that is not itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeFakeMpm(directory: string, script: string): void
/// ```
fn write_fake_mpm(
    directory: &Path,
    script: &str,
) {
    // What:     The shadow starts this script as the real mpm.
    // Why:      A write descriptor this process held on it would be inherited by a child of
    //           another test thread, and the start would be refused with `ETXTBSY`
    //           (issue 6527), so a child process writes it.
    through_a_child_process::write_program(&directory.join("mpm"), script.as_bytes());
}

/// What:
/// `fn start_shadow(root: &Path, arguments: &[&str]) -> Output` declares a helper that links the built binary
///           as `mpm` and runs it with a PATH holding the link directory then the fake mpm's directory.
/// Why:
/// This is the exact shape of the real setup:
/// the link's directory first on PATH.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startShadow(root: string, args: string[]): SpawnSyncReturns
/// ```
fn start_shadow(
    root: &Path,
    arguments: &[&str],
) -> Output {
    return start_shadow_with(root, arguments, &[]);
}

/// Like `start_shadow`,
/// with extra environment variables for the shadow.
fn start_shadow_with(
    root: &Path,
    arguments: &[&str],
    extra_env: &[(&str, &str)],
) -> Output {
    let links: std::path::PathBuf = root.join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    // `env!("CARGO_BIN_EXE_mpm-shadow")` is a compile-time macro Cargo sets to the built binary's path.
    let _ = fs::remove_file(links.join("mpm"));
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_mpm-shadow"), links.join("mpm"))
        .expect("link the shadow as mpm");
    let path_variable: String = format!(
        "{}:{}:/usr/bin:/bin",
        links.display(),
        root.join("real").display()
    );
    return Command::new(links.join("mpm"))
        .args(arguments)
        .env("PATH", path_variable)
        .env("CARGO_HOME", root.join("cargo"))
        // The shadow finds the toolchain roots through these two, so they point into the fixture:
        // the machine's own install records must never decide what a test prunes.
        .env("HOME", root.join("home"))
        .env("XDG_DATA_HOME", root.join("data"))
        .envs(extra_env.iter().copied())
        .output()
        .expect("run the shadow");
}

/// What:
/// `fn write_stale_copies(root: &Path)` declares a helper creating a cargo home with an install record and
///           two cached versions of one of our crates.
/// Why:
/// The record says 0.4.0 is installed,
/// so the 0.1.0 archive and source must disappear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeStaleCopies(root: string): void
/// ```
fn write_stale_copies(root: &Path) {
    let cargo: std::path::PathBuf = root.join("cargo");
    let cache: std::path::PathBuf = cargo.join("registry/cache/index.crates.io-fixture");
    let source: std::path::PathBuf = cargo.join("registry/src/index.crates.io-fixture");
    fs::create_dir_all(&cache).expect("make the cache");
    fs::create_dir_all(source.join("monochromatic-lint-0.1.0")).expect("make the old source");
    fs::create_dir_all(source.join("monochromatic-lint-0.4.0")).expect("make the current source");
    fs::write(cache.join("monochromatic-lint-0.1.0.crate"), "").expect("write the old archive");
    fs::write(cache.join("monochromatic-lint-0.4.0.crate"), "").expect("write the current archive");
    fs::write(
        cargo.join(".crates.toml"),
        "[v1]\n\"monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n",
    )
    .expect("write the install record");
}

/// The tests of the arguments and the exit status a run gets back.
#[path = "shim/arguments_and_the_exit_code_pass_through.rs"]
mod arguments_and_the_exit_code_pass_through;

/// The tests of what the post-run cleanup reads and what it prints.
#[path = "shim/what_the_post_run_cleanup_reads_and_says.rs"]
mod what_the_post_run_cleanup_reads_and_says;

/// The tests of the lines the shadow adds after mpm's own version and help.
#[path = "shim/the_notice_after_mpms_own_output.rs"]
mod the_notice_after_mpms_own_output;

/// The tests of a start that fails and of the nesting bound.
#[path = "shim/a_start_that_fails_and_the_nesting_bound.rs"]
mod a_start_that_fails_and_the_nesting_bound;

/// The tests of pnpm's verbs on `mpm` and of the per-tree launcher.
#[path = "shim/pnpm_verbs_and_the_per_tree_launcher.rs"]
mod pnpm_verbs_and_the_per_tree_launcher;
