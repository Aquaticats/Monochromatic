//! What:
//!  The crate declares exactly the executable that the release archive holds.
//! Why:
//!  cargo-binstall requires every `[[bin]]` of a crate to be in the archive (`[package.metadata.binstall]`), and the release job
//!  packages one file, `monochromatic-git-policy-cli`. A second `[[bin]]` made the `catalog-specifiers` workflow fail from 2026-11-01 (`When
//!  resolving monochromatic-git-policy-cli bin cli-git-service is not found. This binary is not optional`) while local installs,
//!  which compile from source, worked. The service is a mode of the one executable (`service_main.rs`). The release job, the
//!  release list and the toolchain lock are checked against the same manifest by `bin/native-release-bins.unit.test.ts`, which
//!  reads files outside this crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(manifestBins(readFileSync('Cargo.toml', 'utf8'))).toEqual(['monochromatic-git-policy-cli']);
//! ```

use std::path::Path;

/// The one executable the release archive holds, which `.github/workflows/cargo-publish.yml` builds with `--bin` and copies.
const RELEASED_BINS: [&str; 1] = ["monochromatic-git-policy-cli"];

/// The names of the `[[bin]]` tables of a manifest: a table header starts a table, and the first `name = "..."` line of a
///  `[[bin]]` table names the executable.
fn manifest_bins(manifest: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut in_bin: bool = false;
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            in_bin = trimmed == "[[bin]]";
            continue;
        }
        if in_bin && let Some(rest) = trimmed.strip_prefix("name = \"") {
            names.push(rest.split('"').next().unwrap_or_default().to_string());
            in_bin = false;
        }
    }
    return names;
}

/// The crate's declared executables are the released one and no other.
#[test]
fn the_crate_declares_only_the_executable_the_release_archive_holds() {
    let manifest: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("the manifest");
    let mut declared: Vec<String> = manifest_bins(manifest.as_str());
    declared.sort();
    assert_eq!(
        declared,
        RELEASED_BINS.iter().map(|name| { return (*name).to_string(); }).collect::<Vec<String>>(),
        "every [[bin]] must be in the release archive, or cargo-binstall refuses the archive; make a second program a mode of monochromatic-git-policy-cli, or package it in .github/workflows/cargo-publish.yml and add it here and in releasable-crates.json"
    );
}

/// The reader sees a second table, whatever else the manifest holds around it.
#[test]
fn a_second_bin_table_is_read() {
    let planted: &str = "[package]\nname = \"x\"\n\n[[bin]]\nname = \"monochromatic-git-policy-cli\"\npath = \"a.rs\"\n\n[[test]]\nname = \"not-a-bin\"\n\n[[bin]]\npath = \"b.rs\"\nname = \"second\"\n";
    assert_eq!(manifest_bins(planted), vec![String::from("monochromatic-git-policy-cli"), String::from("second")]);
}
