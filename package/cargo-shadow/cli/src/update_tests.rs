//! The unit tests of the catalog update,
//! ported from `update.unit.test.ts` against a fake cargo and a fake registry.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::http::RegistryError;
use crate::lockfile::LockedVersion;
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;
use crate::time_text::parse_rfc3339_millis;
use crate::tighten::Syncer;
use crate::update::{UpdateError, UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const CRATE_DIR: &str = "package/u/one";

fn now() -> i64 {
    return parse_rfc3339_millis("2026-10-29T12:00:00Z").expect("a fixed time");
}

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 1_440,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: false,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

fn lock_with(version: &str) -> String {
    return format!("version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

/// A fake cargo:
/// `generate-lockfile` writes the configured lock;
/// `update --package n@v --precise c` swaps the version;
/// a
/// bare `update` with names is accepted;
/// a crate named `ghost` fails.
struct FakeCargo {
    commands: RefCell<Vec<String>>,
    resolved: String,
    fail: bool,
}

impl CargoRunner for FakeCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        self.commands.borrow_mut().push(args.join(" "));
        let lock: PathBuf = cwd.join("Cargo.lock");
        if self.fail {
            return Ok(CargoOutcome { ok: false, output: "failed to select a version for `ghost`".to_string(), stdout: String::new() });
        }
        let is_plain_update: bool = args[0] == "update" && !args.iter().any(|argument| { return argument == "--precise"; });
        if (args.iter().any(|argument| { return argument == "generate-lockfile"; }) || is_plain_update) && !self.resolved.is_empty() {
            fs::write(&lock, &self.resolved).map_err(|error| { return error.to_string(); })?;
        }
        if let Some(position) = args.iter().position(|argument| { return argument == "--precise"; }) {
            let package: &String = &args[position - 1];
            let from: &str = package.split('@').nth(1).unwrap_or("");
            let text: String = fs::read_to_string(&lock).map_err(|error| { return error.to_string(); })?;
            fs::write(&lock, text.replace(&format!("version = \"{from}\""), &format!("version = \"{}\"", args[position + 1])))
                .map_err(|error| { return error.to_string(); })?;
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    }
}

struct FakeRegistry {
    histories: BTreeMap<String, Vec<VersionRecord>>,
}

impl Registry for FakeRegistry {
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        let mut times: BTreeMap<String, String> = BTreeMap::new();
        for record in self.histories.get(name).cloned().unwrap_or_default() {
            times.insert(record.num.clone(), record.created_at.clone());
        }
        return Ok(times);
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
    fn history(&self, name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        return Ok(self.histories.get(name).cloned().unwrap_or_default());
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
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

/// Trusted `1.0.0`,
/// then a token publish `1.0.1` that the trust policy refuses.
fn downgrading_history() -> Vec<VersionRecord> {
    return vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")];
}

/// A sync that records the call instead of running file-enforcer.
struct RecordingSyncer {
    calls: RefCell<u32>,
}

impl Syncer for RecordingSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        *self.calls.borrow_mut() += 1;
        return Ok("synced".to_string());
    }
}

struct World {
    root: tempfile::TempDir,
    dirs: Vec<String>,
    catalog: PathBuf,
}

fn world(before: &str, catalog_text: &str) -> World {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("crate dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), before).expect("lock");
    let catalog: PathBuf = root.path().join("rust-catalog.toml");
    fs::write(&catalog, catalog_text).expect("catalog");
    return World { root, dirs: vec![CRATE_DIR.to_string()], catalog };
}

fn lock_of(world: &World) -> String {
    return fs::read_to_string(world.root.path().join(CRATE_DIR).join("Cargo.lock")).expect("read the lock");
}

fn run(world: &World, catalog_settings: &Settings, cargo: &FakeCargo, registry: &FakeRegistry, dry_run: bool, kind: &UpdateKind) -> Result<UpdateReport, UpdateError> {
    return update_rust_lockfiles(
        &UpdateRequest {
            root: world.root.path(),
            crate_dirs: &world.dirs,
            dry_run,
            now: now(),
            settings: catalog_settings,
            catalog_path: &world.catalog,
            runner: cargo,
            registry,
            forward: &[],
            syncer: &RecordingSyncer { calls: RefCell::new(0) },
            prune_catalog: true,
        },
        kind,
    );
}

fn cargo_resolving(lock: &str) -> FakeCargo {
    return FakeCargo { commands: RefCell::new(Vec::new()), resolved: lock.to_string(), fail: false };
}

fn dep(version: &str) -> LockedVersion {
    return LockedVersion { name: "dep".to_string(), version: version.to_string() };
}

#[test]
fn a_passing_update_keeps_the_lockfiles_uses_the_publish_time_cutoff_and_prunes_stale_excludes() {
    let world = world(&lock_with("1.0.0"), "trustPolicyExclude = [\"gone@1.0.0\"]\n");
    let mut catalog_settings: Settings = settings();
    catalog_settings.trust_policy_exclude = vec!["gone@1.0.0".to_string()];
    catalog_settings.trust_policy_exclude_prune = true;
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.0"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };

    let report = run(&world, &catalog_settings, &cargo, &registry, false, &UpdateKind::Full).expect("updates");

    assert!(report.written);
    assert_eq!(report.pruned, vec!["trustPolicyExclude: gone@1.0.0".to_string()]);
    assert!(!fs::read_to_string(&world.catalog).expect("catalog").contains("gone@1.0.0"));
    assert_eq!(
        *cargo.commands.borrow(),
        vec!["-Zunstable-options generate-lockfile --publish-time 2026-10-28T12:00:00Z".to_string()]
    );
}

#[test]
fn a_refused_version_with_no_older_release_restores_every_lockfile() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let only_refused: Vec<VersionRecord> = vec![record("1.0.1", "2026-02-01T00:00:00Z", "9", "")];
    let first: Vec<VersionRecord> = vec![record("0.9.0", "2026-01-01T00:00:00Z", "", "o/r"), only_refused[0].clone()];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), first)]) };

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

#[test]
fn a_refused_version_is_set_aside_for_the_newest_older_release_that_passes() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("sets aside");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(report.added, vec![dep("1.0.0")]);
    assert!(report.warnings.iter().any(|warning| { return warning.contains("Set aside dep@1.0.1") && warning.contains("1.0.0"); }));
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
}

#[test]
fn a_candidate_published_after_the_release_age_cutoff_is_skipped() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.2"));
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"),
        record("1.0.1", "2026-10-29T01:00:00Z", "", "o/r"),
        record("1.0.2", "2026-02-01T00:00:00Z", "9", ""),
    ];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("sets aside");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock_of(&world), lock_with("1.0.0"), "1.0.1 is younger than the cutoff");
}

#[test]
fn a_dry_run_restores_every_lockfile_and_still_reports_what_would_be_added() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.0"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };

    let report = run(&world, &settings(), &cargo, &registry, true, &UpdateKind::Full).expect("dry run");

    assert!(!report.written);
    assert_eq!(report.added, vec![dep("1.0.0")]);
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

#[test]
fn a_cargo_failure_restores_every_lockfile_and_is_returned_naming_the_crate() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = FakeCargo { commands: RefCell::new(Vec::new()), resolved: String::new(), fail: true };
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };

    let result = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full);

    match result {
        Err(UpdateError::Cargo { crate_dir, output, .. }) => {
            assert_eq!(crate_dir, CRATE_DIR);
            assert!(output.contains("ghost"));
        }
        other => panic!("expected a cargo failure, got {other:?}"),
    }
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

#[test]
fn a_precise_version_the_policy_refuses_fails_and_nothing_is_set_aside_in_its_place() {
    let world = world(&lock_with("1.0.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.0"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    let args: Vec<String> = ["update", "--package", "dep@1.0.0", "--precise", "1.0.1"].iter().map(|argument| { return argument.to_string(); }).collect();

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(*cargo.commands.borrow(), vec![args.join(" ")], "only the user's own command ran");
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
}

#[test]
fn a_named_package_update_is_run_as_given_and_set_aside_when_the_newest_release_is_refused() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    let args: Vec<String> = ["update", "--package", "dep"].iter().map(|argument| { return argument.to_string(); }).collect();

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Targeted { cargo_args: &args }).expect("sets aside");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(cargo.commands.borrow()[0], args.join(" "));
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
}

fn run_with_syncer(world: &World, cargo: &FakeCargo, registry: &FakeRegistry, syncer: &RecordingSyncer, args: &[String]) -> UpdateReport {
    let catalog_settings: Settings = settings();
    return update_rust_lockfiles(
        &UpdateRequest {
            root: world.root.path(),
            crate_dirs: &world.dirs,
            dry_run: false,
            now: now(),
            settings: &catalog_settings,
            catalog_path: &world.catalog,
            runner: cargo,
            registry,
            forward: &[],
            syncer,
            prune_catalog: true,
        },
        &UpdateKind::Targeted { cargo_args: args },
    )
    .expect("updates");
}

fn passing_history() -> BTreeMap<String, Vec<VersionRecord>> {
    return BTreeMap::from([(
        "dep".to_string(),
        vec![record("1.0.0", "2026-01-01T00:00:00Z", "7", ""), record("1.0.1", "2026-02-01T00:00:00Z", "7", "")],
    )]);
}

#[test]
fn breaking_is_pnpm_update_latest_it_raises_the_catalog_floor_and_runs_the_sync() {
    let world = world(&lock_with("1.0.0"), "[catalog]\ndep = \">=1.0.0\"\n");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let registry: FakeRegistry = FakeRegistry { histories: passing_history() };
    let syncer: RecordingSyncer = RecordingSyncer { calls: RefCell::new(0) };
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();

    let report = run_with_syncer(&world, &cargo, &registry, &syncer, &args);

    assert!(report.written, "{:?}", report.problems);
    assert!(fs::read_to_string(&world.catalog).expect("catalog").contains("dep = \">=1.0.1\""));
    assert_eq!(*syncer.calls.borrow(), 1);
    assert!(report.warnings.iter().any(|line| { return line.contains("dep: >=1.0.0 to >=1.0.1"); }));
}

#[test]
fn a_plain_update_is_pnpm_update_no_save_and_leaves_the_catalog_alone() {
    let world = world(&lock_with("1.0.0"), "[catalog]\ndep = \">=1.0.0\"\n");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let registry: FakeRegistry = FakeRegistry { histories: passing_history() };
    let syncer: RecordingSyncer = RecordingSyncer { calls: RefCell::new(0) };
    let args: Vec<String> = ["update", "--package", "dep"].iter().map(|argument| { return argument.to_string(); }).collect();

    run_with_syncer(&world, &cargo, &registry, &syncer, &args);

    assert!(fs::read_to_string(&world.catalog).expect("catalog").contains("dep = \">=1.0.0\""));
    assert_eq!(*syncer.calls.borrow(), 0);
}

#[test]
fn a_refused_breaking_update_changes_neither_the_lock_nor_the_catalog() {
    let world = world(&lock_with("1.0.0"), "[catalog]\ndep = \">=1.0.0\"\n");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    let syncer: RecordingSyncer = RecordingSyncer { calls: RefCell::new(0) };
    let args: Vec<String> = ["update", "--breaking", "--package", "dep@1.0.0", "--precise", "1.0.1"].iter().map(|argument| { return argument.to_string(); }).collect();

    let report = run_with_syncer(&world, &cargo, &registry, &syncer, &args);

    assert!(!report.written);
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
    assert!(fs::read_to_string(&world.catalog).expect("catalog").contains(">=1.0.0"));
    assert_eq!(*syncer.calls.borrow(), 0);
}

#[test]
fn an_update_of_some_workspaces_never_prunes_the_catalog_because_other_lockfiles_may_need_its_excludes() {
    let world = world(&lock_with("1.0.0"), "trustPolicyExclude = [\"needed-elsewhere@1.0.0\"]\n");
    let mut catalog_settings: Settings = settings();
    catalog_settings.trust_policy_exclude = vec!["needed-elsewhere@1.0.0".to_string()];
    catalog_settings.trust_policy_exclude_prune = true;
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.0"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };

    let report = update_rust_lockfiles(
        &UpdateRequest {
            root: world.root.path(),
            crate_dirs: &world.dirs,
            dry_run: false,
            now: now(),
            settings: &catalog_settings,
            catalog_path: &world.catalog,
            runner: &cargo,
            registry: &registry,
            forward: &[],
            syncer: &RecordingSyncer { calls: RefCell::new(0) },
            prune_catalog: false,
        },
        &UpdateKind::Full,
    )
    .expect("updates");

    assert!(report.written);
    assert!(report.pruned.is_empty());
    assert!(fs::read_to_string(&world.catalog).expect("catalog").contains("needed-elsewhere@1.0.0"));
}
