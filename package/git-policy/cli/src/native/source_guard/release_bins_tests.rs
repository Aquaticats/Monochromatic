//! What:
//!  The crate declares exactly the executable the release archive holds,
//!  and that executable's `[[bin]]` compiles the file the entry pin reads.
//! Why:
//!  cargo-binstall requires every `[[bin]]` of a crate to be in the archive (`[package.metadata.binstall]`),
//!  and the release job packages that one file, `monochromatic-git-policy-cli`.
//!  A second `[[bin]]` once made the `catalog-specifiers` workflow fail from 2026-11-01
//!  (`When resolving monochromatic-git-policy-cli bin cli-git-service is not found. This binary is not optional`)
//!  while local installs,
//!  which compile from source,
//!  worked;
//!  the resident service is a mode of the first executable (`service_main.rs`),
//!  and after the owner's answer the ZFS route's root modes are modes of it too
//!  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`).
//!  The release job,
//!  the release list,
//!  the toolchain lock and this list are checked against the same manifest by
//!  `bin/native-release-bins.unit.test.ts`,
//!  which reads files outside this crate.
//!  A `path` that moved would leave the entry pin reading a file the release does not build,
//!  which no reader of the names caught before this assertion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(manifestBins(readFileSync('Cargo.toml', 'utf8'))).toEqual(['monochromatic-git-policy-cli']);
//! ```

use std::path::Path;

/// The executable the release archive holds,
/// which `.github/workflows/cargo-publish.yml` builds with one `--bin` and copies into one archive.
const RELEASED_BINS: [&str; 1] = ["monochromatic-git-policy-cli"];

/// What:
///  The `[[bin]]` tables of a manifest as `(name, path)` pairs, in the order they are written.
/// Why:
///  The release builds one file per declared name,
///  and the entry pin reads the file the manifest's `path` names,
///  so the pair is what a reader must compare.
fn manifest_bin_tables(manifest: &str) -> Vec<(String, String)> {
    let mut tables: Vec<(String, String)> = Vec::new();
    let mut name: Option<String> = None;
    let mut path: Option<String> = None;
    let mut in_bin: bool = false;
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            if in_bin {
                tables.push((name.take().unwrap_or_default(), path.take().unwrap_or_default()));
            }
            in_bin = trimmed == "[[bin]]";
            continue;
        }
        if !in_bin {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            name = Some(rest.split('"').next().unwrap_or_default().to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("path = \"") {
            path = Some(rest.split('"').next().unwrap_or_default().to_string());
        }
    }
    if in_bin {
        tables.push((name.take().unwrap_or_default(), path.take().unwrap_or_default()));
    }
    return tables;
}

/// The names of the `[[bin]]` tables of a manifest:
/// the name of each pair the reader above answers.
fn manifest_bins(manifest: &str) -> Vec<String> {
    return manifest_bin_tables(manifest).into_iter().map(|(name, _path)| { return name; }).collect();
}

/// The crate's declared executables are the one released one and no other.
#[test]
fn the_crate_declares_only_the_executables_the_release_archive_holds() {
    let manifest: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("the manifest");
    let mut declared: Vec<String> = manifest_bins(manifest.as_str());
    declared.sort();
    assert_eq!(
        declared,
        RELEASED_BINS.iter().map(|name| { return (*name).to_string(); }).collect::<Vec<String>>(),
        "every [[bin]] must be in the release archive, or cargo-binstall refuses the archive; make a second program a mode of monochromatic-git-policy-cli, or package it in .github/workflows/cargo-publish.yml and add it here, in releasable-crates.json and in toolchain.lock.json"
    );
}

/// The reader sees every table,
/// whatever else the manifest holds around and between them.
#[test]
fn every_bin_table_is_read() {
    let planted: &str = "[package]\nname = \"x\"\n\n[[bin]]\nname = \"monochromatic-git-policy-cli\"\npath = \"a.rs\"\n\n[[test]]\nname = \"not-a-bin\"\n\n[[bin]]\npath = \"b.rs\"\nname = \"second\"\n";
    assert_eq!(manifest_bins(planted), vec![String::from("monochromatic-git-policy-cli"), String::from("second")]);
    assert_eq!(
        manifest_bin_tables(planted),
        vec![(String::from("monochromatic-git-policy-cli"), String::from("a.rs")), (String::from("second"), String::from("b.rs"))]
    );
}

/// The wrapper's `[[bin]]` compiles the file the entry pin holds.
#[test]
fn the_wrappers_bin_compiles_the_pinned_main() {
    let manifest: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("the manifest");
    let wrapper: Vec<(String, String)> = manifest_bin_tables(manifest.as_str()).into_iter().filter(|(name, _path)| { return name == "monochromatic-git-policy-cli"; }).collect();
    assert_eq!(wrapper, vec![(String::from("monochromatic-git-policy-cli"), String::from("src/native/main.rs"))], "the wrapper's `[[bin]]` must compile src/native/main.rs, the one file the entry pin reads; a `path` that moves makes the pinned text guard a file the release does not build");
}
