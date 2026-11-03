//! The unit tests of the shadow's install step: the fingerprint fast path, the authoritative check, the CI behavior.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::http::RegistryError;
use crate::install_step::{StepOutcome, StepRequest, run_install_step};
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const SATISFIED: &str = "# satisfied\nversion = 4\n";
const STALE: &str = "# stale\nversion = 4\n";
const CRATE_DIR: &str = "package/x/y";

/// A fake cargo: `update --locked` passes on a lock starting `# satisfied`; plain `update` writes `after`; `metadata` prints
/// the manifest list of a one-crate workspace.
struct FakeCargo {
    commands: RefCell<Vec<String>>,
    after: String,
}

impl CargoRunner for FakeCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        self.commands.borrow_mut().push(args.join(" "));
        let lock: std::path::PathBuf = cwd.join("Cargo.lock");
        if args[0] == "update" && args.iter().any(|argument| { return argument == "--locked"; }) {
            let satisfied: bool = fs::read_to_string(&lock).unwrap_or_default().starts_with("# satisfied");
            return Ok(CargoOutcome { ok: satisfied, output: "needs update".to_string(), stdout: String::new() });
        }
        if args[0] == "update" {
            fs::write(&lock, &self.after).map_err(|error| { return error.to_string(); })?;
        }
        if args[0] == "metadata" {
            let manifest: String = cwd.join("Cargo.toml").display().to_string();
            let body: String = format!("{{\"workspace_root\":\"{}\",\"packages\":[{{\"source\":null,\"manifest_path\":\"{manifest}\"}}]}}", cwd.display());
            return Ok(CargoOutcome { ok: true, output: body.clone(), stdout: body });
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    }
}

struct FakeRegistry {
    histories: BTreeMap<String, Vec<VersionRecord>>,
}

impl Registry for FakeRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
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

/// A catalog repository with one crate whose lock text is given.
fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::write(root.path().join("rust-catalog.toml"), "[catalog]\n").expect("catalog");
    fs::create_dir_all(root.path().join(CRATE_DIR).join("src")).expect("crate dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "[package]\nname = \"y\"\nversion = \"0.1.0\"\n\n[workspace]\n").expect("manifest");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

fn cargo(after: &str) -> FakeCargo {
    return FakeCargo { commands: RefCell::new(Vec::new()), after: after.to_string() };
}

fn registry() -> FakeRegistry {
    return FakeRegistry { histories: BTreeMap::new() };
}

fn step(start: &Path, frozen: bool, runner: &FakeCargo, registry: &FakeRegistry) -> StepOutcome {
    return run_install_step(&StepRequest { start_dir: start, frozen, now: 0, runner, registry });
}

#[test]
fn an_up_to_date_workspace_is_checked_once_and_then_costs_no_cargo_at_all() {
    let root = repository(SATISFIED);
    let runner: FakeCargo = cargo(SATISFIED);
    let start = root.path().join(CRATE_DIR).join("src");

    let first = step(&start, false, &runner, &registry());
    let commands_after_first: usize = runner.commands.borrow().len();
    let second = step(&start, false, &runner, &registry());

    assert_eq!(first, StepOutcome::Passed);
    assert_eq!(second, StepOutcome::FastPath);
    assert!(commands_after_first > 0);
    assert_eq!(runner.commands.borrow().len(), commands_after_first, "the second call started cargo");
}

#[test]
fn editing_a_manifest_after_a_pass_makes_the_next_call_check_again() {
    let root = repository(SATISFIED);
    let runner: FakeCargo = cargo(SATISFIED);
    let start = root.path().join(CRATE_DIR);
    assert_eq!(step(&start, false, &runner, &registry()), StepOutcome::Passed);
    fs::write(start.join("Cargo.toml"), "[package]\nname = \"y\"\nversion = \"0.2.0\"\n\n[workspace]\n# edited\n").expect("edit");
    let before: usize = runner.commands.borrow().len();

    let outcome = step(&start, false, &runner, &registry());

    assert_eq!(outcome, StepOutcome::Passed);
    assert!(runner.commands.borrow().len() > before);
}

#[test]
fn changing_the_lockfile_after_a_pass_makes_the_next_call_check_again() {
    let root = repository(SATISFIED);
    let runner: FakeCargo = cargo(SATISFIED);
    let start = root.path().join(CRATE_DIR);
    assert_eq!(step(&start, false, &runner, &registry()), StepOutcome::Passed);
    fs::write(start.join("Cargo.lock"), STALE).expect("make stale");

    let outcome = step(&start, false, &runner, &registry());

    assert!(matches!(outcome, StepOutcome::Updated { .. }), "{outcome:?}");
}

#[test]
fn a_stale_lockfile_is_updated_outside_ci_and_the_fixed_lock_then_takes_the_fast_path() {
    let root = repository(STALE);
    let runner: FakeCargo = cargo(SATISFIED);
    let start = root.path().join(CRATE_DIR);

    let outcome = step(&start, false, &runner, &registry());

    assert_eq!(outcome, StepOutcome::Updated { crate_dir: CRATE_DIR.to_string(), added: Vec::new() });
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), SATISFIED);
    assert_eq!(step(&start, false, &runner, &registry()), StepOutcome::FastPath);
}

#[test]
fn a_frozen_step_fails_naming_the_stale_lock_and_writes_nothing() {
    let root = repository(STALE);
    let runner: FakeCargo = cargo(SATISFIED);
    let start = root.path().join(CRATE_DIR);

    let outcome = step(&start, true, &runner, &registry());

    match outcome {
        StepOutcome::Failed(message) => assert!(message.contains(CRATE_DIR), "{message}"),
        other => panic!("expected a failure, got {other:?}"),
    }
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), STALE);
    assert!(runner.commands.borrow().iter().all(|command| { return command.contains("--locked"); }));
}

#[test]
fn a_refused_new_version_fails_the_step_and_restores_the_lock() {
    let root = repository(STALE);
    let after: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let runner: FakeCargo = cargo(&after);
    let history: Vec<VersionRecord> = vec![
        VersionRecord {
            num: "1.0.0".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            publisher_id: String::new(),
            trustpub_repository: "o/r".to_string(),
            repository: String::new(),
        },
        VersionRecord {
            num: "1.0.1".to_string(),
            created_at: "2026-02-01T00:00:00Z".to_string(),
            publisher_id: "9".to_string(),
            trustpub_repository: String::new(),
            repository: String::new(),
        },
    ];
    let fake: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
    let start = root.path().join(CRATE_DIR);

    let outcome = step(&start, false, &runner, &fake);

    match outcome {
        StepOutcome::Failed(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), STALE);
}

#[test]
fn outside_a_catalog_repository_and_without_a_lockfile_the_step_does_nothing() {
    let outside = scratch();
    fs::write(outside.path().join("Cargo.toml"), "[package]\nname = \"z\"\nversion = \"0.1.0\"\n\n[workspace]\n").expect("manifest");
    let runner: FakeCargo = cargo(SATISFIED);
    assert!(matches!(step(outside.path(), false, &runner, &registry()), StepOutcome::Skipped(_)));

    let root = repository(SATISFIED);
    fs::remove_file(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("remove the lock");
    assert!(matches!(step(&root.path().join(CRATE_DIR), false, &runner, &registry()), StepOutcome::Skipped(_)));
    assert!(runner.commands.borrow().is_empty());
}
