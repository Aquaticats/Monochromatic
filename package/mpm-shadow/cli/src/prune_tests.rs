//! What:     Unit tests of the prune rule in `prune.rs`, the registry scan in `registry.rs` and the install record
//!           reader in `installed.rs`.
//! Why:      The rule deletes files, so each of its decisions (which version stays, which crates it never touches) has
//!           a test, and the deletion itself runs against a disposable cargo home.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune.unit.test.ts
//! describe('findStale', () => { it('keeps the installed version', () => { ... }); });
//! ```

/// What:     `use std::collections::BTreeMap;` imports the sorted key-value map (see `installed.rs`).
/// Why:      `find_stale` takes the installed versions as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // A TS Map; there is no built-in sorted map.
/// ```
use std::collections::BTreeMap;

/// Imports the file-system module (writes fixtures, checks what survived).
use std::fs;

/// Imports the owned path type.
use std::path::{Path, PathBuf};

use crate::installed::installed_versions;
use crate::prune::{find_stale, prune_stale_own_crates};
use crate::registry::RegistryEntry;
use crate::test_scratch::scratch;

/// What:     `fn entry(crate_name: &str, version: &str) -> RegistryEntry` declares a helper that builds one registry
///           entry with a made-up path.
/// Why:      The pure-rule tests need lists of entries but no files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entry(crateName: string, version: string): RegistryEntry { return { crateName, version, path: `/x/${crateName}-${version}` }; }
/// ```
fn entry(crate_name: &str, version: &str) -> RegistryEntry {
    // `format!` builds an owned text from a template, like a TS template literal; `PathBuf::from` makes it a path.
    return RegistryEntry {
        crate_name: crate_name.to_string(),
        version: version.to_string(),
        path: PathBuf::from(format!("/x/{crate_name}-{version}")),
    };
}

/// What:     `fn versions_of(entries: &[RegistryEntry]) -> Vec<String>` declares a helper listing `crate-version` texts.
/// Why:      Assertions compare short readable lists instead of whole records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionsOf = (entries) => entries.map((e) => `${e.crateName}-${e.version}`);
/// ```
fn versions_of(entries: &[RegistryEntry]) -> Vec<String> {
    // `.iter().map(|e| ...).collect()` walks the list, builds a text per element with a closure, and gathers them.
    return entries.iter().map(|e| return format!("{}-{}", e.crate_name, e.version)).collect();
}

/// The installed version stays; every other cached version of that crate goes.
#[test]
fn keeps_the_installed_version() {
    let mut installed: BTreeMap<String, String> = BTreeMap::new();
    installed.insert("monochromatic-lint".to_string(), "0.4.0".to_string());
    let stale = find_stale(
        vec![
            entry("monochromatic-lint", "0.1.0"),
            entry("monochromatic-lint", "0.4.0"),
            entry("monochromatic-lint", "0.5.0"),
        ],
        &installed,
    );
    assert_eq!(versions_of(&stale), vec!["monochromatic-lint-0.1.0", "monochromatic-lint-0.5.0"]);
}

/// A crate with no install record keeps its newest cached version, compared as numbers.
#[test]
fn keeps_the_newest_version_without_an_install_record() {
    let stale = find_stale(
        vec![
            entry("monochromatic-jsonc-edit", "0.1.0"),
            entry("monochromatic-jsonc-edit", "0.1.9"),
            entry("monochromatic-jsonc-edit", "0.1.10"),
        ],
        &BTreeMap::new(),
    );
    assert_eq!(versions_of(&stale), vec!["monochromatic-jsonc-edit-0.1.0", "monochromatic-jsonc-edit-0.1.9"]);
}

/// A crate whose name extends another's is a different crate and keeps its own version.
#[test]
fn judges_a_crate_sharing_a_name_prefix_on_its_own_versions() {
    let mut installed: BTreeMap<String, String> = BTreeMap::new();
    installed.insert("monochromatic-lint".to_string(), "0.4.0".to_string());
    let stale = find_stale(
        vec![entry("monochromatic-lint", "0.4.0"), entry("monochromatic-lint-closure", "0.1.0")],
        &installed,
    );
    assert!(stale.is_empty());
}

/// What:     `fn write_record(cargo_home: &Path, key: &str)` declares a helper that writes cargo's install record with
///           one installed crate.
/// Why:      The reader and the deletion tests need a record on disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeRecord(cargoHome: string, key: string): void { fs.writeFileSync(path.join(cargoHome, '.crates2.json'), JSON.stringify({ installs: { [key]: {} } })); }
/// ```
fn write_record(cargo_home: &Path, key: &str) {
    // `.expect("...")` unwraps a `Result` and, on error, stops the test with that message (tests may panic on purpose).
    fs::write(cargo_home.join(".crates2.json"), format!("{{\"installs\":{{\"{key}\":{{}}}}}}")).expect("write the record");
}

/// What:     `fn write_entry(cargo_home: &Path, kind: &str, name: &str)` declares a helper that creates one cached copy:
///           a file under `cache` (kind `cache`) or a directory under `src`.
/// Why:      The deletion tests need real files and directories to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeEntry(cargoHome: string, kind: 'cache' | 'src', name: string): void
/// ```
fn write_entry(cargo_home: &Path, kind: &str, name: &str) {
    let directory: PathBuf = cargo_home.join("registry").join(kind).join("index.crates.io-fixture");
    fs::create_dir_all(&directory).expect("make the index directory");
    if kind == "cache" {
        fs::write(directory.join(format!("{name}.crate")), "").expect("write the archive");
    } else {
        fs::create_dir_all(directory.join(name)).expect("make the source directory");
    }
}

/// The record names the installed version; stale archives and sources go, the rest stay.
#[test]
fn deletes_stale_archives_and_sources_and_keeps_the_installed_version() {
    let directory = scratch();
    let home: &Path = directory.path();
    write_record(home, "monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)");
    for name in ["monochromatic-lint-0.1.0", "monochromatic-lint-0.4.0"] {
        write_entry(home, "cache", name);
        write_entry(home, "src", name);
    }

    let deleted = prune_stale_own_crates(home);

    let index: PathBuf = home.join("registry");
    assert_eq!(deleted.len(), 2);
    assert!(!index.join("cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
    assert!(!index.join("src/index.crates.io-fixture/monochromatic-lint-0.1.0").exists());
    assert!(index.join("cache/index.crates.io-fixture/monochromatic-lint-0.4.0.crate").exists());
    assert!(index.join("src/index.crates.io-fixture/monochromatic-lint-0.4.0").exists());
}

/// Crates this repository does not publish are never touched, whatever their versions.
#[test]
fn never_touches_third_party_crates() {
    let directory = scratch();
    let home: &Path = directory.path();
    write_entry(home, "cache", "serde-1.0.1");
    write_entry(home, "cache", "serde-1.0.2");

    assert!(prune_stale_own_crates(home).is_empty());
    assert!(home.join("registry/cache/index.crates.io-fixture/serde-1.0.1.crate").exists());
}

/// A cargo home with no registry has nothing to delete and is not an error.
#[test]
fn deletes_nothing_when_the_registry_does_not_exist() {
    let directory = scratch();
    assert!(prune_stale_own_crates(directory.path()).is_empty());
}

/// An unreadable record means nothing is known as installed, so the newest cached version stays.
#[test]
fn falls_back_to_the_newest_version_when_the_record_is_malformed() {
    let directory = scratch();
    let home: &Path = directory.path();
    fs::write(home.join(".crates2.json"), "not json").expect("write the bad record");
    assert!(installed_versions(home).is_empty());
    write_entry(home, "cache", "monochromatic-lint-0.1.0");
    write_entry(home, "cache", "monochromatic-lint-0.2.0");

    assert_eq!(prune_stale_own_crates(home).len(), 1);
    assert!(home.join("registry/cache/index.crates.io-fixture/monochromatic-lint-0.2.0.crate").exists());
}

/// The reader takes the crate name and version out of cargo's key.
#[test]
fn reads_the_installed_version_from_the_record() {
    let directory = scratch();
    write_record(directory.path(), "monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)");
    let installed = installed_versions(directory.path());
    assert_eq!(installed.get("monochromatic-lint"), Some(&"0.4.0".to_string()));
}
