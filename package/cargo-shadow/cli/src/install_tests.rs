//! The unit tests of the install step, ported from `install.unit.test.ts` against a fake cargo and a fake registry.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::http::RegistryError;
use crate::install::{InstallError, InstallReport, InstallRequest, install_rust_lockfiles};
use crate::lockfile::LockedVersion;
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// The text of a lockfile the fake cargo calls satisfied.
const SATISFIED: &str = "# satisfied\nversion = 4\n";

/// The text of a lockfile the fake cargo calls stale.
const STALE: &str = "# stale\nversion = 4\n";

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: true,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
    };
}

/// Lock text that holds one registry package.
fn lock_with(name: &str, version: &str) -> String {
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

/// A fake cargo over files: `update --locked` passes when the lock text is satisfied; a plain `update` writes `after`.
struct FakeCargo {
    commands: RefCell<Vec<String>>,
    after: String,
    fail_update_in: Option<String>,
}

impl CargoRunner for FakeCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        let line: String = format!("{} :: {}", cwd.file_name().unwrap_or_default().to_string_lossy(), args.join(" "));
        self.commands.borrow_mut().push(line);
        let lock_path: PathBuf = cwd.join("Cargo.lock");
        if args.iter().any(|argument| { return argument == "--locked"; }) && args[0] == "update" {
            let satisfied: bool = fs::read_to_string(&lock_path).unwrap_or_default().starts_with("# satisfied");
            return Ok(CargoOutcome { ok: satisfied, output: "the lock file needs to be updated".to_string(), stdout: String::new() });
        }
        if args[0] == "update" {
            if let Some(failing) = &self.fail_update_in
                && cwd.ends_with(failing)
            {
                return Ok(CargoOutcome { ok: false, output: "no matching package named `ghost` found".to_string(), stdout: String::new() });
            }
            fs::write(&lock_path, &self.after).map_err(|error| { return error.to_string(); })?;
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    }
}

/// A registry whose histories are canned and whose publish times are empty.
struct FakeRegistry {
    histories: BTreeMap<String, Vec<VersionRecord>>,
}

impl Registry for FakeRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
    fn history(&self, name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        return Ok(self.histories.get(name).cloned().unwrap_or_default());
    }
    fn owners(&self, _name: &str) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
}

fn record(num: &str, created_at: &str, publisher_id: &str, trustpub_repository: &str) -> VersionRecord {
    return VersionRecord {
        num: num.to_string(),
        created_at: created_at.to_string(),
        publisher_id: publisher_id.to_string(),
        trustpub_repository: trustpub_repository.to_string(),
        repository: String::new(),
    };
}

/// A repository with crate roots whose lockfile texts are given, in the order listed.
fn repository(locks: &[(&str, &str)]) -> (tempfile::TempDir, Vec<String>) {
    let root = scratch();
    let mut dirs: Vec<String> = Vec::new();
    for (dir, text) in locks {
        fs::create_dir_all(root.path().join(dir)).expect("make the crate dir");
        fs::write(root.path().join(dir).join("Cargo.lock"), text).expect("write the lock");
        dirs.push(dir.to_string());
    }
    return (root, dirs);
}

fn lock_text(root: &Path, dir: &str) -> String {
    return fs::read_to_string(root.join(dir).join("Cargo.lock")).expect("read the lock");
}

fn run_install(root: &Path, dirs: &[String], cargo: &FakeCargo, registry: &FakeRegistry, frozen: bool, dry_run: bool) -> Result<InstallReport, InstallError> {
    let catalog_settings: Settings = settings();
    return install_rust_lockfiles(&InstallRequest {
        root,
        crate_dirs: dirs,
        frozen,
        offline: true,
        dry_run,
        now: 0,
        settings: &catalog_settings,
        runner: cargo,
        registry,
    });
}

fn cargo_that_writes(after: &str) -> FakeCargo {
    return FakeCargo { commands: RefCell::new(Vec::new()), after: after.to_string(), fail_update_in: None };
}

fn empty_registry() -> FakeRegistry {
    return FakeRegistry { histories: BTreeMap::new() };
}

#[test]
fn a_frozen_install_fails_naming_every_stale_root_and_writes_nothing() {
    let (root, dirs) = repository(&[("package/a/one", STALE), ("package/a/two", SATISFIED), ("package/a/three", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);

    let result = run_install(root.path(), &dirs, &cargo, &empty_registry(), true, false);

    assert_eq!(result, Err(InstallError::Stale { stale_dirs: vec!["package/a/one".to_string(), "package/a/three".to_string()] }));
    assert_eq!(lock_text(root.path(), "package/a/one"), STALE);
    assert!(cargo.commands.borrow().iter().all(|command| { return command.contains("--locked"); }), "{:?}", cargo.commands);
}

#[test]
fn a_frozen_install_passes_running_only_read_only_checks_when_every_lockfile_is_satisfied() {
    let (root, dirs) = repository(&[("package/a/one", SATISFIED), ("package/a/two", SATISFIED)]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);

    let report = run_install(root.path(), &dirs, &cargo, &empty_registry(), true, false).expect("passes");

    assert!(report.stale.is_empty());
    assert!(!report.written);
    assert_eq!(cargo.commands.borrow().len(), 2);
    assert!(cargo.commands.borrow().iter().all(|command| { return command.ends_with("update --workspace --locked --offline"); }));
}

#[test]
fn an_update_runs_in_the_stale_crate_only_offline_then_verifies_with_cargo_metadata() {
    let (root, dirs) = repository(&[("package/a/fresh", SATISFIED), ("package/a/stale", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);

    let report = run_install(root.path(), &dirs, &cargo, &empty_registry(), false, false).expect("updates");

    assert_eq!(report.stale, vec!["package/a/stale".to_string()]);
    assert!(report.written);
    assert_eq!(
        *cargo.commands.borrow(),
        vec![
            "fresh :: update --workspace --locked --offline".to_string(),
            "stale :: update --workspace --locked --offline".to_string(),
            "stale :: update --workspace --offline".to_string(),
            "stale :: metadata --locked --offline --format-version 1".to_string(),
        ]
    );
    assert_eq!(lock_text(root.path(), "package/a/stale"), SATISFIED);
}

#[test]
fn a_dry_run_restores_every_lockfile_and_still_reports_what_it_would_update() {
    let (root, dirs) = repository(&[("package/a/stale", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);

    let report = run_install(root.path(), &dirs, &cargo, &empty_registry(), false, true).expect("dry run");

    assert_eq!(report.stale, vec!["package/a/stale".to_string()]);
    assert!(!report.written);
    assert_eq!(lock_text(root.path(), "package/a/stale"), STALE);
}

#[test]
fn a_cargo_failure_restores_every_lockfile_and_names_the_crate() {
    let (root, dirs) = repository(&[("package/a/one", STALE), ("package/a/two", STALE)]);
    let mut cargo: FakeCargo = cargo_that_writes(SATISFIED);
    cargo.fail_update_in = Some("package/a/two".to_string());

    let result = run_install(root.path(), &dirs, &cargo, &empty_registry(), false, false);

    match result {
        Err(InstallError::Cargo { crate_dir, command, output }) => {
            assert_eq!(crate_dir, "package/a/two");
            assert_eq!(command, "cargo update --workspace --offline");
            assert!(output.contains("ghost"));
        }
        other => panic!("expected a cargo failure, got {other:?}"),
    }
    assert_eq!(lock_text(root.path(), "package/a/one"), STALE);
    assert_eq!(lock_text(root.path(), "package/a/two"), STALE);
}

#[test]
fn a_newly_locked_version_that_fails_the_trust_policy_restores_every_lockfile() {
    let (root, dirs) = repository(&[("package/a/stale", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(&lock_with("dep", "1.0.1"));
    let registry: FakeRegistry = FakeRegistry {
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")],
        )]),
    };

    let report = run_install(root.path(), &dirs, &cargo, &registry, false, false).expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1);
    assert!(report.problems[0].contains("dep@1.0.1 has trust level 1"));
    assert_eq!(lock_text(root.path(), "package/a/stale"), STALE);
}

#[test]
fn a_newly_locked_version_that_passes_is_kept_and_reported() {
    let (root, dirs) = repository(&[("package/a/stale", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(&lock_with("dep", "1.0.1"));
    let registry: FakeRegistry = FakeRegistry {
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "7", ""), record("1.0.1", "2026-02-01T00:00:00Z", "7", "")],
        )]),
    };

    let report = run_install(root.path(), &dirs, &cargo, &registry, false, false).expect("keeps");

    assert!(report.written);
    assert_eq!(report.added, vec![LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }]);
    assert_eq!(lock_text(root.path(), "package/a/stale"), lock_with("dep", "1.0.1"));
}

#[test]
fn ci_is_any_value_but_empty_zero_or_false() {
    for value in ["true", "1", "yes", "TRUE"] {
        assert!(crate::install::is_continuous_integration(value), "{value}");
    }
    for value in ["", "0", "false", "False"] {
        assert!(!crate::install::is_continuous_integration(value), "{value}");
    }
}

/// A path-only workspace whose lockfile lacks the dependency `a` declares on `b`: stale without any registry.
fn write_stale_path_workspace(root: &Path) {
    let dir: PathBuf = root.join("package/real/ws");
    fs::create_dir_all(dir.join("a/src")).expect("make a");
    fs::create_dir_all(dir.join("b/src")).expect("make b");
    fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"3\"\n").expect("root manifest");
    fs::write(
        dir.join("a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
    )
    .expect("a manifest");
    fs::write(dir.join("a/src/lib.rs"), "pub fn f() {}\n").expect("a source");
    fs::write(dir.join("b/Cargo.toml"), "[package]\nname = \"b\"\nversion = \"0.1.0\"\nedition = \"2024\"\n").expect("b manifest");
    fs::write(dir.join("b/src/lib.rs"), "pub fn g() {}\n").expect("b source");
    fs::write(
        dir.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .expect("stale lock");
}

#[test]
fn real_cargo_finds_the_path_workspace_stale_updates_it_offline_and_then_finds_it_satisfied() {
    let program: PathBuf = match crate::real_binary::find_real_cargo() {
        Some(found) => found,
        None => {
            eprintln!("skipped: no cargo on PATH, so the real-cargo install case cannot run");
            return;
        }
    };
    let root = scratch();
    write_stale_path_workspace(root.path());
    let dirs: Vec<String> = vec!["package/real/ws".to_string()];
    let cargo: crate::cargo_run::RealCargo = crate::cargo_run::RealCargo { program, toolchain: String::new() };
    let catalog_settings: Settings = settings();
    let registry: FakeRegistry = empty_registry();
    let request = |frozen: bool| {
        return InstallRequest {
            root: root.path(),
            crate_dirs: &dirs,
            frozen,
            offline: true,
            dry_run: false,
            now: 0,
            settings: &catalog_settings,
            runner: &cargo,
            registry: &registry,
        };
    };

    let frozen = install_rust_lockfiles(&request(true));
    assert_eq!(frozen, Err(InstallError::Stale { stale_dirs: dirs.clone() }));

    let report = install_rust_lockfiles(&request(false)).expect("updates");
    assert_eq!(report.stale, dirs);
    assert!(report.written);
    assert!(lock_text(root.path(), "package/real/ws").contains("dependencies"));

    let again = install_rust_lockfiles(&request(true)).expect("now satisfied");
    assert!(again.stale.is_empty());
}

#[test]
fn a_check_that_may_use_the_network_does_not_pass_offline() {
    let (root, dirs) = repository(&[("package/a/one", SATISFIED)]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);
    let catalog_settings: Settings = settings();
    let registry: FakeRegistry = empty_registry();

    install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: true,
        offline: false,
        dry_run: false,
        now: 0,
        settings: &catalog_settings,
        runner: &cargo,
        registry: &registry,
    })
    .expect("passes");

    assert_eq!(*cargo.commands.borrow(), vec!["one :: update --workspace --locked".to_string()]);
}

/// A registry that, like a second process, rewrites the lockfile while the policy check runs.
struct InterferingRegistry {
    lock: PathBuf,
    histories: BTreeMap<String, Vec<VersionRecord>>,
}

impl Registry for InterferingRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
    fn history(&self, name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        fs::write(&self.lock, "# written by another process\nversion = 4\n").expect("the other writer");
        return Ok(self.histories.get(name).cloned().unwrap_or_default());
    }
    fn owners(&self, _name: &str) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
}

#[test]
fn a_lockfile_another_process_wrote_during_the_checks_is_not_discarded_by_a_refusal() {
    let (root, dirs) = repository(&[("package/a/stale", STALE)]);
    let cargo: FakeCargo = cargo_that_writes(&lock_with("dep", "1.0.1"));
    let registry: InterferingRegistry = InterferingRegistry {
        lock: root.path().join("package/a/stale/Cargo.lock"),
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")],
        )]),
    };
    let catalog_settings: Settings = settings();

    let report = install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: false,
        offline: true,
        dry_run: false,
        now: 0,
        settings: &catalog_settings,
        runner: &cargo,
        registry: &registry,
    })
    .expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(lock_text(root.path(), "package/a/stale"), "# written by another process\nversion = 4\n");
    assert!(report.warnings.iter().any(|warning| { return warning.contains("another process"); }), "{:?}", report.warnings);
}

#[test]
fn a_run_killed_after_cargo_wrote_is_finished_by_the_next_install() {
    let (root, dirs) = repository(&[("package/a/one", &lock_with("dep", "1.0.1"))]);
    crate::journal::begin(
        root.path(),
        &[crate::crates::LockSnapshot { crate_dir: "package/a/one".to_string(), source: SATISFIED.to_string() }],
    )
    .expect("journal");
    let ours = root.path().join(crate::journal::JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.path().join(crate::journal::JOURNAL_DIR).join("4194305")).expect("pose as a killed process");
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);
    let registry: FakeRegistry = FakeRegistry {
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")],
        )]),
    };

    let report = run_install(root.path(), &dirs, &cargo, &registry, false, false).expect("recovers");

    assert_eq!(lock_text(root.path(), "package/a/one"), SATISFIED);
    assert_eq!(
        fs::read_to_string(root.path().join("package/a/one/Cargo.lock.refused")).expect("saved"),
        lock_with("dep", "1.0.1")
    );
    assert!(report.warnings.iter().any(|warning| { return warning.contains("killed"); }), "{:?}", report.warnings);
}
