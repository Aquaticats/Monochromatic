//! What:
//! Unit tests of the prune rule in `prune.rs`,
//! the registry scan in `registry.rs` and the install record
//!           reader in `installed.rs`.
//! Why:
//! The rule deletes files,
//! so each of its decisions (which version stays,
//! which crates it never touches) has
//!           a test,
//! and the deletion itself runs against a disposable cargo home.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune.unit.test.ts
//! describe('findStale', () => { it('keeps the installed version', () => { ... }); });
//! ```

/// What:
/// `use std::collections::BTreeMap;` imports the sorted key-value map (see `installed.rs`).
/// Why:
/// `find_stale` takes the installed versions as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // A TS Map; there is no built-in sorted map.
/// ```
use std::collections::BTreeMap;

/// Imports the file-system module (writes fixtures,
/// checks what survived).
use std::fs;

/// Imports the owned path type.
use std::path::{Path, PathBuf};

use crate::installed::installed_versions;
use crate::prune::{find_stale, installed_everywhere, prune_stale_own_crates};
use crate::registry::RegistryEntry;
use crate::toolchain::cargo_installs::repository_versions_installed;
use crate::toolchain::scope::{EnvMap, toolchain_roots};
use crate::test_scratch::scratch;

/// What:
/// `fn entry(crate_name: &str, version: &str) -> RegistryEntry` declares a helper that builds one registry
///           entry with a made-up path.
/// Why:
/// The pure-rule tests need lists of entries but no files.
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

/// What:
/// `fn versions_of(entries: &[RegistryEntry]) -> Vec<String>` declares a helper listing `crate-version` texts.
/// Why:
/// Assertions compare short readable lists instead of whole records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionsOf = (entries) => entries.map((e) => `${e.crateName}-${e.version}`);
/// ```
fn versions_of(entries: &[RegistryEntry]) -> Vec<String> {
    // `.iter().map(|e| ...).collect()` walks the list, builds a text per element with a closure, and gathers them.
    return entries.iter().map(|e| return format!("{}-{}", e.crate_name, e.version)).collect();
}

/// The installed version stays;
/// every other cached version of that crate goes.
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

/// A crate with no install record keeps its newest cached version,
/// compared as numbers.
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

/// What:
/// `fn write_record(cargo_home: &Path, key: &str)` declares a helper that writes cargo's install record with
///           one installed crate.
/// Why:
/// The reader and the deletion tests need a record on disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeRecord(cargoHome: string, key: string): void { fs.writeFileSync(path.join(cargoHome, '.crates.toml'), JSON.stringify({ installs: { [key]: {} } })); }
/// ```
fn write_record(cargo_home: &Path, key: &str) {
    // `.expect("...")` unwraps a `Result` and, on error, stops the test with that message (tests may panic on purpose).
    fs::write(cargo_home.join(".crates.toml"), format!("[v1]\n\"{key}\" = [\"bin\"]\n")).expect("write the record");
}

/// What:
/// `fn write_entry(cargo_home: &Path, kind: &str, name: &str)` declares a helper that creates one cached copy:
///           a file under `cache` (kind `cache`) or a directory under `src`.
/// Why:
/// The deletion tests need real files and directories to delete.
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

/// The record names the installed version;
/// stale archives and sources go,
/// the rest stay.
#[test]
fn deletes_stale_archives_and_sources_and_keeps_the_installed_version() {
    let directory = scratch();
    let home: &Path = directory.path();
    write_record(home, "monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)");
    for name in ["monochromatic-lint-0.1.0", "monochromatic-lint-0.4.0"] {
        write_entry(home, "cache", name);
        write_entry(home, "src", name);
    }

    let deleted = prune_stale_own_crates(home, &[]);

    let index: PathBuf = home.join("registry");
    assert_eq!(deleted.len(), 2);
    assert!(!index.join("cache/index.crates.io-fixture/monochromatic-lint-0.1.0.crate").exists());
    assert!(!index.join("src/index.crates.io-fixture/monochromatic-lint-0.1.0").exists());
    assert!(index.join("cache/index.crates.io-fixture/monochromatic-lint-0.4.0.crate").exists());
    assert!(index.join("src/index.crates.io-fixture/monochromatic-lint-0.4.0").exists());
}

/// Crates this repository does not publish are never touched,
/// whatever their versions.
#[test]
fn never_touches_third_party_crates() {
    let directory = scratch();
    let home: &Path = directory.path();
    write_entry(home, "cache", "serde-1.0.1");
    write_entry(home, "cache", "serde-1.0.2");

    assert!(prune_stale_own_crates(home, &[]).is_empty());
    assert!(home.join("registry/cache/index.crates.io-fixture/serde-1.0.1.crate").exists());
}

/// A cargo home with no registry has nothing to delete and is not an error.
#[test]
fn deletes_nothing_when_the_registry_does_not_exist() {
    let directory = scratch();
    assert!(prune_stale_own_crates(directory.path(), &[]).is_empty());
}

/// An unreadable record means nothing is known as installed,
/// so the newest cached version stays.
#[test]
fn falls_back_to_the_newest_version_when_the_record_is_malformed() {
    let directory = scratch();
    let home: &Path = directory.path();
    fs::write(home.join(".crates.toml"), "not json").expect("write the bad record");
    assert!(installed_versions(home).is_empty());
    write_entry(home, "cache", "monochromatic-lint-0.1.0");
    write_entry(home, "cache", "monochromatic-lint-0.2.0");

    assert_eq!(prune_stale_own_crates(home, &[]).len(), 1);
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

//region version-keyed install records

/// Writes the record `mpm-shadow install` leaves for a finished install:
/// `<root>/cargo/<crate>/<version>/.crates.toml` naming the crate and version.
fn finish_version(root: &Path, crate_name: &str, version: &str) {
    let directory: PathBuf = root.join("cargo").join(crate_name).join(version);
    fs::create_dir_all(&directory).expect("make the version directory");
    fs::write(
        directory.join(".crates.toml"),
        format!("[v1]\n\"{crate_name} {version} (registry+https://github.com/rust-lang/crates.io-index)\" = [\"{crate_name}\"]\n"),
    )
    .expect("write the version's record");
}

/// A version directory that was started and never finished:
/// no cargo record inside.
fn start_version(root: &Path, crate_name: &str, version: &str) {
    fs::create_dir_all(root.join("cargo").join(crate_name).join(version)).expect("make the unfinished directory");
}

fn cache_has(home: &Path, name: &str) -> bool {
    return home.join("registry/cache/index.crates.io-fixture").join(format!("{name}.crate")).exists();
}

fn source_has(home: &Path, name: &str) -> bool {
    return home.join("registry/src/index.crates.io-fixture").join(name).exists();
}

/// The reported bug:
/// a stale `.crates.toml` names 0.4.0 while the version-keyed record names 0.5.0,
/// and the prune deleted the newer cached archive.
#[test]
fn a_stale_legacy_record_no_longer_deletes_the_newer_cached_archive() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("cargo-home");
    let root: PathBuf = directory.path().join("toolchain");
    fs::create_dir_all(&home).expect("make the cargo home");
    write_record(&home, "monochromatic-lint 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)");
    finish_version(&root, "monochromatic-lint", "0.5.0");
    for name in ["monochromatic-lint-0.4.0", "monochromatic-lint-0.5.0"] {
        write_entry(&home, "cache", name);
        write_entry(&home, "src", name);
    }

    // Control: without the toolchain root the old reading is reproduced, so the fixture shows the bug.
    let legacy_only: Vec<PathBuf> = prune_stale_own_crates(&home, &[]);
    assert_eq!(legacy_only.len(), 2, "the legacy record alone keeps 0.4.0 and deletes 0.5.0");
    assert!(!cache_has(&home, "monochromatic-lint-0.5.0"));
    for name in ["monochromatic-lint-0.4.0", "monochromatic-lint-0.5.0"] {
        write_entry(&home, "cache", name);
        write_entry(&home, "src", name);
    }

    let deleted: Vec<PathBuf> = prune_stale_own_crates(&home, &[root]);

    assert_eq!(deleted.len(), 2, "{deleted:?}");
    assert!(cache_has(&home, "monochromatic-lint-0.5.0") && source_has(&home, "monochromatic-lint-0.5.0"));
    assert!(!cache_has(&home, "monochromatic-lint-0.4.0") && !source_has(&home, "monochromatic-lint-0.4.0"));
}

/// Trees still on the TypeScript toolchain keep installing into `$CARGO_HOME`,
/// so a legacy record newer than every version-keyed one is kept.
#[test]
fn a_legacy_record_newer_than_the_version_keyed_ones_is_still_kept() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("cargo-home");
    let root: PathBuf = directory.path().join("toolchain");
    fs::create_dir_all(&home).expect("make the cargo home");
    write_record(&home, "monochromatic-lint 0.6.0 (registry+https://github.com/rust-lang/crates.io-index)");
    finish_version(&root, "monochromatic-lint", "0.5.0");
    for name in ["monochromatic-lint-0.5.0", "monochromatic-lint-0.6.0"] {
        write_entry(&home, "cache", name);
    }

    let deleted: Vec<PathBuf> = prune_stale_own_crates(&home, &[root]);

    assert_eq!(deleted.len(), 1, "{deleted:?}");
    assert!(cache_has(&home, "monochromatic-lint-0.6.0"));
    assert!(!cache_has(&home, "monochromatic-lint-0.5.0"));
}

/// With no legacy record at all (after the cutover removes it) the version-keyed one decides.
#[test]
fn the_version_keyed_record_decides_when_there_is_no_legacy_record() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("cargo-home");
    let root: PathBuf = directory.path().join("toolchain");
    fs::create_dir_all(&home).expect("make the cargo home");
    finish_version(&root, "monochromatic-lint", "0.4.0");
    for name in ["monochromatic-lint-0.3.0", "monochromatic-lint-0.4.0", "monochromatic-lint-0.5.0"] {
        write_entry(&home, "cache", name);
    }

    let deleted: Vec<PathBuf> = prune_stale_own_crates(&home, &[root]);

    assert_eq!(deleted.len(), 2, "installed 0.4.0 is kept even though 0.5.0 is cached, as with the legacy record");
    assert!(cache_has(&home, "monochromatic-lint-0.4.0"));
}

/// Both roots are read and the newest recorded version across them wins.
#[test]
fn the_repository_root_and_the_global_root_are_both_read_and_the_newest_wins() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("cargo-home");
    let repository_root: PathBuf = directory.path().join("toolchain");
    let global_root: PathBuf = directory.path().join("toolchain-global");
    fs::create_dir_all(&home).expect("make the cargo home");
    finish_version(&repository_root, "monochromatic-lint", "0.4.0");
    finish_version(&global_root, "monochromatic-lint", "0.5.0");
    finish_version(&global_root, "monochromatic-mpm-shadow", "0.2.0");

    let installed: BTreeMap<String, String> = installed_everywhere(&home, &[repository_root, global_root]);

    assert_eq!(installed.get("monochromatic-lint").map(String::as_str), Some("0.5.0"));
    assert_eq!(installed.get("monochromatic-mpm-shadow").map(String::as_str), Some("0.2.0"));
}

/// Rejected fixtures:
/// an unfinished install,
/// a third-party crate,
/// a stray file,
/// and a missing root are not records of an installed repository crate.
#[test]
fn unfinished_installs_third_party_crates_and_missing_roots_are_not_installed_records() {
    let directory = scratch();
    let root: PathBuf = directory.path().join("toolchain");
    finish_version(&root, "monochromatic-lint", "0.4.0");
    start_version(&root, "monochromatic-lint", "0.5.0");
    start_version(&root, "monochromatic-unfinished", "0.1.0");
    finish_version(&root, "fd-find", "10.5.0");
    fs::write(root.join("cargo").join("monochromatic-stray"), "a file, not a crate directory").expect("write a stray file");

    let found: BTreeMap<String, String> = repository_versions_installed(&root);

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found.get("monochromatic-lint").map(String::as_str), Some("0.4.0"), "the unfinished 0.5.0 is ignored");
    assert!(repository_versions_installed(&directory.path().join("no-such-root")).is_empty());
    assert!(installed_everywhere(directory.path(), &[directory.path().join("no-such-root")]).is_empty());
}

/// The shim learns the roots from the environment alone.
#[test]
fn the_roots_come_from_xdg_data_home_then_home_and_are_empty_without_either() {
    let with_xdg = toolchain_roots(&EnvMap::from_pairs(&[("HOME", "/h"), ("XDG_DATA_HOME", "/x")]));
    assert_eq!(with_xdg, vec![PathBuf::from("/x/monochromatic-dev/toolchain"), PathBuf::from("/x/monochromatic-dev/toolchain-global")]);
    let with_home = toolchain_roots(&EnvMap::from_pairs(&[("HOME", "/h")]));
    assert_eq!(
        with_home,
        vec![PathBuf::from("/h/.local/share/monochromatic-dev/toolchain"), PathBuf::from("/h/.local/share/monochromatic-dev/toolchain-global")]
    );
    assert!(toolchain_roots(&EnvMap::from_pairs(&[("XDG_DATA_HOME", "/x")])).is_empty(), "no home, no roots");
}

//endregion
