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

fn run(world: &World, catalog_settings: &Settings, cargo: &FakeCargo, registry: &dyn Registry, dry_run: bool, kind: &UpdateKind) -> Result<UpdateReport, UpdateError> {
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

/// The older release (`0.9.0`) passes the policy, but cargo refuses to move to it (it is outside what the manifests allow,
/// which only cargo knows), so the finding stands and every lockfile is restored.
#[test]
fn a_refused_version_whose_older_releases_cargo_refuses_restores_every_lockfile() {
    let world = world(&lock_with("0.9.0"), "");
    let lock_path: PathBuf = world.root.path().join(CRATE_DIR).join("Cargo.lock");
    let cargo = move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        if args.iter().any(|argument| { return argument == "--precise"; }) {
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
        }
        fs::write(&lock_path, lock_with("1.0.1")).map_err(|error| { return error.to_string(); })?;
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
    let first: Vec<VersionRecord> = vec![record("0.9.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), first)]) };
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };

    let report = update_rust_lockfiles(&request, &UpdateKind::Full).expect("reports");

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

/// Review finding (early-error rollback): cargo fails before anything of this run was recorded as produced, and another
/// writer published the lockfile meanwhile. The rollback must leave that lockfile alone.
#[test]
fn a_cargo_failure_before_anything_was_produced_does_not_restore_over_another_writers_lock() {
    let world = world(&lock_with("0.9.0"), "");
    let lock_path: PathBuf = world.root.path().join(CRATE_DIR).join("Cargo.lock");
    let cargo = move |_cwd: &Path, _args: &[String]| -> Result<CargoOutcome, String> {
        fs::write(&lock_path, "published by another writer meanwhile").map_err(|error| { return error.to_string(); })?;
        return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
    };
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };

    let result = update_rust_lockfiles(&request, &UpdateKind::Full);

    assert!(result.is_err());
    assert_eq!(lock_of(&world), "published by another writer meanwhile");
}

fn token_publish(num: &str, created_at: &str) -> VersionRecord {
    return record(num, created_at, "9", "");
}

fn trusted_publish(num: &str, created_at: &str) -> VersionRecord {
    return record(num, created_at, "", "o/r");
}

fn set_aside_outcome(history: Vec<VersionRecord>, resolved_version: &str, catalog_settings: &Settings) -> (UpdateReport, String) {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with(resolved_version));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };

    let report = run(&world, catalog_settings, &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    return (report, lock_of(&world));
}

/// Review finding (caret class as boundary): the manifest requires `>=1.0.0`, the selected `2.0.0` is refused, and `1.9.9`
/// passes. The replacement boundary is what the manifest allows, which cargo checks on `--precise`, not the caret class of
/// the refused version.
#[test]
fn a_refused_release_may_be_replaced_by_an_older_release_of_an_earlier_major_version() {
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.9.9", "2026-02-01T00:00:00Z"),
        token_publish("2.0.0", "2026-03-01T00:00:00Z"),
    ];

    let (report, lock) = set_aside_outcome(history, "2.0.0", &settings());

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock, lock_with("1.9.9"));
}

/// Review finding (cutoff applied unconditionally): a crate exempt from the release age may be replaced by a young release.
#[test]
fn a_candidate_younger_than_the_cutoff_is_allowed_for_a_crate_exempt_from_the_release_age() {
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "2026-10-29T01:00:00Z"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];
    let mut exempt: Settings = settings();
    exempt.minimum_release_age_exclude = vec!["dep".to_string()];

    let (report, lock) = set_aside_outcome(history, "1.0.2", &exempt);

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock, lock_with("1.0.1"));
}

/// Review finding: with `minimumReleaseAgeStrict = false` the age is not a condition of the choice either.
#[test]
fn a_candidate_younger_than_the_cutoff_is_allowed_when_the_release_age_is_not_strict() {
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "2026-10-29T01:00:00Z"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];
    let mut relaxed: Settings = settings();
    relaxed.minimum_release_age_strict = false;

    let (report, lock) = set_aside_outcome(history, "1.0.2", &relaxed);

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock, lock_with("1.0.1"));
}

/// Review finding (unknown timestamps): a candidate whose publish time cannot be read does not pass the age condition.
#[test]
fn a_candidate_with_an_unreadable_publish_time_is_skipped_under_a_strict_release_age() {
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "garbled"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];

    let (report, lock) = set_aside_outcome(history, "1.0.2", &settings());

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock, lock_with("1.0.0"));
}

/// A registry with owners, so the candidate trust evaluation can be compared with the final one.
struct RegistryWithOwners {
    inner: FakeRegistry,
    owners: BTreeSet<String>,
}

impl Registry for RegistryWithOwners {
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return self.inner.pubtimes(name);
    }
    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return self.inner.yanked(name);
    }
    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        return self.inner.history(name, versions);
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(self.owners.clone());
    }
}

/// Review finding (owners omitted from candidate trust): `1.0.1` by a stranger ranks below `1.0.0` by a listed owner once the
/// owners are known, so it must not be chosen as the replacement; the final check would refuse it.
#[test]
fn a_candidate_that_the_final_check_would_refuse_for_its_owners_is_not_chosen() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.2"));
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "1", ""),
        record("1.0.1", "2026-02-01T00:00:00Z", "2", ""),
        record("1.0.2", "2026-03-01T00:00:00Z", "9", ""),
    ];
    let registry: RegistryWithOwners = RegistryWithOwners {
        inner: FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) },
        owners: ["1".to_string()].into_iter().collect(),
    };

    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
    let tried: Vec<String> = cargo.commands.borrow().clone();
    assert!(!tried.iter().any(|command| { return command.ends_with("--precise 1.0.1"); }), "a candidate the final check refuses was tried: {tried:?}");
}

/// A cargo whose resolution under a publish-time cutoff fails (the manifest needs a young release) and whose unconstrained
/// resolution locks that young release.
fn cargo_needing_a_young_release(lock_path: PathBuf, young_lock: String) -> impl Fn(&Path, &[String]) -> Result<CargoOutcome, String> {
    return move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        if args.iter().any(|argument| { return argument == "--publish-time"; }) {
            return Ok(CargoOutcome { ok: false, output: "failed to select a version for `dep`".to_string(), stdout: String::new() });
        }
        fs::write(&lock_path, &young_lock).map_err(|error| { return error.to_string(); })?;
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
}

/// Review finding (the global cutoff defeats exemptions): the manifest needs a young release of a crate listed in
/// `minimumReleaseAgeExclude`. Resolving under the strict cutoff cannot succeed, but the exemption means the young release is
/// allowed, so the resolution must not fail before the exemption is applied.
#[test]
fn a_young_release_that_a_manifest_needs_resolves_when_the_crate_is_exempt_from_the_release_age() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    let history: Vec<VersionRecord> = vec![trusted_publish("1.0.0", "2026-10-29T10:00:00Z")];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
    let mut exempt: Settings = settings();
    exempt.minimum_release_age_exclude = vec!["dep".to_string()];
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &exempt,
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };

    let report = update_rust_lockfiles(&request, &UpdateKind::Full).expect("the exempt young release resolves");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
}

/// Without an exemption the strict cutoff still stops the resolution, as before.
#[test]
fn a_young_release_that_a_manifest_needs_still_fails_a_strict_resolution_without_an_exemption() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };

    let result = update_rust_lockfiles(&request, &UpdateKind::Full);

    assert!(matches!(result, Err(UpdateError::Cargo { .. })), "{result:?}");
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

/// The unconstrained resolution that an exemption allows does not let a crate that is not exempt through: the final check
/// still refuses its young release and the lockfile is restored.
#[test]
fn a_crate_that_is_not_exempt_is_still_held_to_the_cutoff_after_the_unconstrained_resolution() {
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    let history: Vec<VersionRecord> = vec![trusted_publish("1.0.0", "2026-10-29T10:00:00Z")];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
    let mut exempt_other: Settings = settings();
    exempt_other.minimum_release_age_exclude = vec!["other".to_string()];
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &exempt_other,
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };

    let report = update_rust_lockfiles(&request, &UpdateKind::Full).expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

const OLD_REQUIREMENT_MANIFEST: &str = "[package]\nname = \"u\"\nversion = \"0.1.0\"\n\n[dependencies]\ndep = \">=0.9.0\"\n";
const NEW_REQUIREMENT_MANIFEST: &str = "[package]\nname = \"u\"\nversion = \"0.1.0\"\n\n[dependencies]\ndep = \">=1.0.1\"\n";

/// A cargo whose `update --breaking` edits the manifest's requirement and locks `dep@1.0.1`, and whose `metadata` lists the
/// manifest, the way cargo's own breaking update rewrites `Cargo.toml`.
fn cargo_that_edits_the_manifest(directory: PathBuf) -> impl Fn(&Path, &[String]) -> Result<CargoOutcome, String> {
    return move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        if args[0] == "metadata" {
            let body: String = format!(
                "{{\"workspace_root\":\"{}\",\"packages\":[{{\"source\":null,\"manifest_path\":\"{}\"}}]}}",
                directory.display(),
                directory.join("Cargo.toml").display()
            );
            return Ok(CargoOutcome { ok: true, output: body.clone(), stdout: body });
        }
        if args.iter().any(|argument| { return argument == "--precise"; }) {
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
        }
        fs::write(directory.join("Cargo.toml"), NEW_REQUIREMENT_MANIFEST).map_err(|error| { return error.to_string(); })?;
        fs::write(directory.join("Cargo.lock"), lock_with("1.0.1")).map_err(|error| { return error.to_string(); })?;
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
}

fn breaking_update(dry_run: bool) -> (World, Result<UpdateReport, UpdateError>) {
    let world = world(&lock_with("0.9.0"), "");
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };
    let result = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args });
    return (world, result);
}

fn manifest_of(world: &World) -> String {
    return fs::read_to_string(world.root.path().join(CRATE_DIR).join("Cargo.toml")).expect("read the manifest");
}

/// Review finding (`--breaking` edits outside the transaction): cargo's breaking update rewrites the manifests. When the
/// policy then refuses the release and nothing can be set aside, the manifests go back with the lockfiles.
#[test]
fn a_refused_breaking_update_puts_the_edited_manifests_back_with_the_lockfiles() {
    let (world, result) = breaking_update_without_a_way_out();

    let report = result.expect("reports");

    assert!(!report.written);
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST, "the manifest kept cargo's edit although the update was refused");
}

/// A dry run changes nothing either.
#[test]
fn a_dry_run_of_a_breaking_update_puts_the_edited_manifests_back() {
    let (world, result) = breaking_update(true);

    let report = result.expect("reports");

    assert!(!report.written);
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST);
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

/// A breaking update the policy accepts keeps cargo's manifest edit.
#[test]
fn an_accepted_breaking_update_keeps_the_manifest_edit() {
    let (world, result) = breaking_update_that_passes();

    let report = result.expect("reports");

    assert!(report.written, "{:?}", report.problems);
    assert_eq!(manifest_of(&world), NEW_REQUIREMENT_MANIFEST);
}

fn breaking_update_without_a_way_out() -> (World, Result<UpdateReport, UpdateError>) {
    return breaking_update(false);
}

fn breaking_update_that_passes() -> (World, Result<UpdateReport, UpdateError>) {
    let world = world(&lock_with("0.9.0"), "");
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
    };
    let result = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args });
    return (world, result);
}
