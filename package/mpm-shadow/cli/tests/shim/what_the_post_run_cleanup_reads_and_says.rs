//! What:
//! The part of the shim tests that asks what the post-run cleanup reads and what it prints:
//! which toolchain roots it reads (`data home` of the fixture, never the machine's own),
//! a newer version recorded where `install` records one,
//! one summary line by default and one line per removed path under `MPM_SHADOW_VERBOSE`,
//! and silence when nothing is stale.
//! Why:
//! The cleanup reads install records written by other tools,
//! so a test that planted a version must see that version kept:
//! a run that read the machine's own records would delete the wrong archives,
//! and the summary line must not read as part of the mpm command that ran.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/what_the_post_run_cleanup_reads_and_says.test.ts
//! test('many stale copies are one line', () => { expect(stderr.split('\n')).toHaveLength(2); });
//! ```

use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

use crate::{
    fixture_root,
    start_shadow,
    start_shadow_with,
    write_fake_mpm,
    write_stale_copies,
};

/// A version no real machine has installed,
/// so a test that sees it can only have read the disposable data directory it planted it in.
const PLANTED_NEWER_VERSION: &str = "9.9.9";

/// A newer version of the stale crate recorded the way `install` records it
/// (a version directory with its own `.crates.toml` under the toolchain root of the disposable data home),
/// and its archive cached.
fn write_toolchain_record(root: &Path) {
    let version_dir: std::path::PathBuf = root
        .join("data/monochromatic-dev/toolchain/cargo/monochromatic-lint")
        .join(PLANTED_NEWER_VERSION);
    fs::create_dir_all(&version_dir).expect("make the version directory");
    fs::write(
        version_dir.join(".crates.toml"),
        format!(
            "[v1]\n\"monochromatic-lint {PLANTED_NEWER_VERSION} (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n"
        ),
    )
    .expect("write the version-keyed record");
    fs::write(
        root.join("cargo/registry/cache/index.crates.io-fixture")
            .join(format!("monochromatic-lint-{PLANTED_NEWER_VERSION}.crate")),
        "",
    )
    .expect("write the newer archive");
}

/// The shadow reads the toolchain roots under the data home it is given,
/// never the machine's own,
/// so a newer recorded version there is the one kept and the older caches go.
#[test]
fn reads_the_toolchain_roots_of_the_data_home_it_is_given_and_never_the_machines_own() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_stale_copies(root.path());
    write_toolchain_record(root.path());

    let output: Output = start_shadow(root.path(), &["installed"]);

    assert_eq!(output.status.code(), Some(0));
    let cache: std::path::PathBuf = root
        .path()
        .join("cargo/registry/cache/index.crates.io-fixture");
    assert!(
        cache
            .join(format!("monochromatic-lint-{PLANTED_NEWER_VERSION}.crate"))
            .exists()
    );
    assert!(!cache.join("monochromatic-lint-0.4.0.crate").exists());
    assert!(!cache.join("monochromatic-lint-0.1.0.crate").exists());
}

/// How many stale copies the many-paths tests plant.
const MANY_STALE: usize = 18;

/// Many stale archives of one crate next to the one the install record names.
fn write_many_stale_copies(root: &Path) {
    let cargo: std::path::PathBuf = root.join("cargo");
    let cache: std::path::PathBuf = cargo.join("registry/cache/index.crates.io-fixture");
    fs::create_dir_all(&cache).expect("make the cache");
    fs::write(cache.join("monochromatic-lint-0.4.0.crate"), "").expect("write the current archive");
    for index in 0..MANY_STALE {
        fs::write(
            cache.join(format!("monochromatic-lint-0.1.{index}.crate")),
            "",
        )
        .expect("write a stale archive");
    }
    fs::write(
        cargo.join(".crates.toml"),
        "[v1]\n\"monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)\" = [\"monochromatic-lint\"]\n",
    )
    .expect("write the install record");
}

/// Many removals are one line by default,
/// and the line says it is cleanup after the mpm command.
#[test]
fn many_stale_copies_are_one_post_run_cleanup_line_by_default() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_many_stale_copies(root.path());

    let output: Output = start_shadow(root.path(), &["installed"]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(
        stderr.contains("post-run cleanup") && stderr.contains(&format!("{MANY_STALE} ")),
        "{stderr}"
    );
    assert!(stderr.contains("MPM_SHADOW_VERBOSE"), "{stderr}");
}

/// The variable named in that line lists every path.
#[test]
fn the_verbose_variable_lists_every_removed_path_after_the_summary() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");
    write_many_stale_copies(root.path());

    let output: Output =
        start_shadow_with(root.path(), &["installed"], &[("MPM_SHADOW_VERBOSE", "1")]);

    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(stderr.lines().count(), 1 + MANY_STALE, "{stderr}");
    assert_eq!(
        stderr.matches("deleted stale").count(),
        MANY_STALE,
        "{stderr}"
    );
}

/// Nothing stale,
/// nothing printed.
#[test]
fn a_run_with_nothing_stale_prints_no_cleanup_line_even_when_verbose() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), "#!/bin/sh\nexit 0\n");

    let output: Output =
        start_shadow_with(root.path(), &["installed"], &[("MPM_SHADOW_VERBOSE", "1")]);

    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
