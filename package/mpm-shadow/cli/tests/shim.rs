//! What:
//! End-to-end tests of the built `mpm-shadow` binary:
//! it is started through a symlink named `mpm`,
//! with a
//!           fake "real" `mpm` script on PATH and a disposable cargo home,
//! exactly as the toolchain will run it.
//! Why:
//! The unit tests prove the pieces;
//! only a real process shows that the arguments arrive untouched,
//! the exit
//!           status passes through,
//! and the stale copies are gone afterwards.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.integration.test.ts
//! const result = spawnSync(path.join(shadowDirectory, 'mpm'), ['list'], { env: { PATH, CARGO_HOME } });
//! ```

/// Imports the file-system module (writes the fake mpm,
/// makes links and fixtures).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks the fake mpm executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// Imports `Command` (start a program) and `Output` (its result).
use std::process::{Command, Output};

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
fn write_fake_mpm(directory: &Path, script: &str) {
    fs::create_dir_all(directory).expect("make the fake mpm directory");
    fs::write(directory.join("mpm"), script).expect("write the fake mpm");
    fs::set_permissions(directory.join("mpm"), fs::Permissions::from_mode(0o755)).expect("make it executable");
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
fn start_shadow(root: &Path, arguments: &[&str]) -> Output {
    let links: std::path::PathBuf = root.join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    // `env!("CARGO_BIN_EXE_mpm-shadow")` is a compile-time macro Cargo sets to the built binary's path.
    let _ = fs::remove_file(links.join("mpm"));
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_mpm-shadow"), links.join("mpm")).expect("link the shadow as mpm");
    let path_variable: String = format!("{}:{}:/usr/bin:/bin", links.display(), root.join("real").display());
    return Command::new(links.join("mpm"))
        .args(arguments)
        .env("PATH", path_variable)
        .env("CARGO_HOME", root.join("cargo"))
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

/// The arguments arrive untouched,
/// the child's exit code passes through,
/// and the stale copies are gone afterwards.
#[test]
fn passes_arguments_and_exit_code_through_then_prunes() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nfor a in \"$@\"; do echo \"[$a]\"; done\nexit 3\n");
    write_stale_copies(root.path());

    let output: Output = start_shadow(root.path(), &["--output-format", "json", "a b"]);

    // `String::from_utf8_lossy(&bytes)` reads captured output bytes as text, replacing invalid UTF-8.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[--output-format]\n[json]\n[a b]\n");
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("deleted stale"));
    let cargo: std::path::PathBuf = root.path().join("cargo");
    assert!(!cargo.join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
    assert!(!cargo.join("registry/src/index.crates.io-fixture/monochromatic-lint-0.1.0").exists());
    assert!(cargo.join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.4.0.crate").exists());
}

/// Standard output stays exactly mpm's when there is nothing to prune.
#[test]
fn adds_nothing_to_standard_output_or_error_when_nothing_is_stale() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\necho hello\n");

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
}

/// With no real mpm on PATH the shadow reports it and exits 127.
#[test]
fn exits_127_when_no_real_mpm_is_on_path() {
    let root = fixture_root();

    let output: Output = start_shadow(root.path(), &[]);

    assert_eq!(output.status.code(), Some(127));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no real mpm found on PATH"));
}

/// Started as `mpm-shadow`,
/// the `prune` subcommand deletes the stale copies and says which.
#[test]
fn the_prune_subcommand_deletes_and_reports() {
    let root = fixture_root();
    write_stale_copies(root.path());

    let output: Output = Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
        .arg("prune")
        .env("CARGO_HOME", root.path().join("cargo"))
        .output()
        .expect("run mpm-shadow prune");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("deleted stale"));
    assert!(!root.path().join("cargo/registry/cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
}
