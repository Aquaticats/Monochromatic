//! The unit tests of the policy checks of a local (not frozen) install:
//! a committed lockfile that holds a release under the age limit or a trust downgrade is changed to the newest older release
//! that passes, and the install stops when none does.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::http::RegistryError;
use crate::install::{InstallError, InstallReport, InstallRequest, install_rust_lockfiles};
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;
use crate::time_text::parse_rfc3339_millis;

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const CRATE_DIR: &str = "package/p/one";
const STALE: &str = "# stale\nversion = 4\n";

fn now() -> i64 {
    return parse_rfc3339_millis("2026-10-29T12:00:00Z").expect("a fixed time");
}

/// One day of release age, strict, trust policy on, every locked entry checked.
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
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

/// A fake cargo:
/// `update --locked` is satisfied by a lock whose first line says so;
/// a plain `update` writes `after`;
/// `update --package n@v --precise c` swaps the version unless `refuse_precise` is set.
struct FakeCargo {
    after: String,
    refuse_precise: bool,
    commands: RefCell<Vec<String>>,
}

impl CargoRunner for FakeCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        self.commands.borrow_mut().push(args.join(" "));
        let lock: PathBuf = cwd.join("Cargo.lock");
        let ok = CargoOutcome { ok: true, output: String::new(), stdout: String::new() };
        if args[0] == "metadata" {
            return Ok(ok);
        }
        if args[0] != "update" {
            return Ok(ok);
        }
        if args.iter().any(|argument| { return argument == "--locked"; }) {
            let satisfied: bool = fs::read_to_string(&lock).unwrap_or_default().starts_with("# satisfied");
            return Ok(CargoOutcome { ok: satisfied, output: "needs update".to_string(), stdout: String::new() });
        }
        if let Some(position) = args.iter().position(|argument| { return argument == "--precise"; }) {
            if self.refuse_precise {
                return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
            }
            let from: String = args[position - 1].split('@').nth(1).unwrap_or("").to_string();
            let text: String = fs::read_to_string(&lock).map_err(|error| { return error.to_string(); })?;
            fs::write(&lock, text.replace(&format!("version = \"{from}\""), &format!("version = \"{}\"", args[position + 1])))
                .map_err(|error| { return error.to_string(); })?;
            return Ok(ok);
        }
        fs::write(&lock, &self.after).map_err(|error| { return error.to_string(); })?;
        return Ok(ok);
    }
}

fn cargo(after: &str, refuse_precise: bool) -> FakeCargo {
    return FakeCargo { after: after.to_string(), refuse_precise, commands: RefCell::new(Vec::new()) };
}

/// A registry that serves the canned histories and derives the publish times from them.
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

fn registry_of(history: Vec<VersionRecord>) -> FakeRegistry {
    return FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
}

fn repository(lock: &str) -> (tempfile::TempDir, Vec<String>) {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("crate dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return (root, vec![CRATE_DIR.to_string()]);
}

fn lock_text(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("read the lock");
}

fn install(root: &Path, dirs: &[String], runner: &FakeCargo, registry: &FakeRegistry, frozen: bool) -> Result<InstallReport, InstallError> {
    let catalog_settings: Settings = settings();
    return install_rust_lockfiles(&InstallRequest {
        root,
        crate_dirs: dirs,
        frozen,
        offline: true,
        dry_run: false,
        now: now(),
        settings: &catalog_settings,
        runner,
        registry,
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    });
}

const OLD_ENOUGH: &str = "2026-10-20T00:00:00Z";
const THREE_HOURS_AGO: &str = "2026-10-29T09:00:00Z";

/// Owner case 1: a merge brings a lockfile holding a release published 3 hours ago, the limit is one day, the release before
/// it is nine days old and fits the range. The lockfile changes to the older release, one line names both, the install goes on.
#[test]
fn a_committed_lock_holding_a_release_under_the_age_limit_is_changed_to_the_newest_older_release() {
    let (root, dirs) = repository(&lock_with("1.0.1"));
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
    assert!(
        report.warnings.iter().any(|line| { return line.contains("dep@1.0.1") && line.contains("1.0.0"); }),
        "one line must name both releases: {:?}",
        report.warnings
    );
}

/// Owner case 1, when nothing older fits: the install stops with the finding and the lockfile is left as it was.
#[test]
fn a_committed_lock_holding_a_young_release_with_no_older_release_that_fits_stops_the_install() {
    let (root, dirs) = repository(&lock_with("1.0.1"));
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), true);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("reports");

    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    assert!(!report.written);
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
}

/// Owner case 2: the lockfile holds `1.0.1` from an account with no earlier releases while `1.0.0` came from the trusted
/// publisher. The lockfile changes to `1.0.0`.
#[test]
fn a_committed_lock_holding_a_trust_downgrade_is_changed_to_the_earlier_release_with_the_earlier_trust_level() {
    let (root, dirs) = repository(&lock_with("1.0.1"));
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", "2026-10-21T00:00:00Z", "9", "")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
    assert!(report.warnings.iter().any(|line| { return line.contains("dep@1.0.1") && line.contains("1.0.0"); }), "{:?}", report.warnings);
}

/// Owner case 2, when no earlier release fits the range (cargo refuses every move): the install stops.
#[test]
fn a_committed_trust_downgrade_with_no_earlier_release_that_fits_stops_the_install() {
    let (root, dirs) = repository(&lock_with("1.0.1"));
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), true);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", "2026-10-21T00:00:00Z", "9", "")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("reports");

    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
}

/// The adopted rule for a version the install itself newly needs: a stale lockfile is updated offline, the update picks a
/// release under the age limit, and that release is set aside for the newest older one that passes instead of being refused.
#[test]
fn a_release_the_update_newly_needs_under_the_age_limit_is_set_aside_instead_of_refused() {
    let (root, dirs) = repository(STALE);
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
}

/// CI keeps the frozen behavior: a frozen install never changes the lockfile, so a refused entry is a finding and no
/// `--precise` move is attempted.
#[test]
fn a_frozen_install_reports_the_refused_entry_and_changes_nothing() {
    let (root, dirs) = repository(&lock_with("1.0.1"));
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, true).expect("reports");

    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
    assert!(!runner.commands.borrow().iter().any(|command| { return command.contains("--precise"); }));
}
