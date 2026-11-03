//! The unit tests of the shadow's install step:
//! the fingerprint fast path,
//! the authoritative check,
//! the CI behavior.

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

/// A fake cargo:
/// `update --locked` passes on a lock starting `# satisfied`;
/// plain `update` writes `after`;
/// `metadata` prints
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
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
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
    return run_install_step(&StepRequest { start_dir: start, frozen, offline: true, now: 0, runner, registry, lock_base: None, trust_base: "HEAD" });
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
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
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

/// A registry whose one crate has a trusted `1.0.0` and a token-published `1.0.1` that the trust policy refuses.
fn downgrading_registry() -> FakeRegistry {
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
    return FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
}

/// Review finding (the freshness stamp bypasses the policy checks):
/// the stamp certifies that the lockfile satisfies its
/// manifests,
/// nothing about the policy.
/// A pass under a lenient trust policy must not let a later frozen call under a stricter
/// policy skip the verification of the locked entries.
#[test]
fn a_stamp_recorded_under_a_lenient_policy_does_not_skip_the_verification_under_a_stricter_one() {
    let lock: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let root = repository(&lock);
    let runner: FakeCargo = cargo(&lock);
    let start = root.path().join(CRATE_DIR);
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"off\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("lenient catalog");
    assert_eq!(step(&start, true, &runner, &downgrading_registry()), StepOutcome::Passed);
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("strict catalog");

    let outcome = step(&start, true, &runner, &downgrading_registry());

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        other => panic!("the stricter policy was never applied: {other:?}"),
    }
}

/// The lockfile used by the base tests:
/// it holds the refused `dep@1.0.1`.
fn refused_lock() -> String {
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
}

fn step_with_base(start: &Path, frozen: bool, runner: &FakeCargo, registry: &FakeRegistry, base_text: &str, trust_base: &str) -> StepOutcome {
    let text: String = base_text.to_string();
    let factory = move || -> Result<crate::install::LockBaseReader, String> {
        let held: String = text.clone();
        return Ok(Box::new(move |_dir: &str| -> String { return held.clone(); }));
    };
    return run_install_step(&StepRequest { start_dir: start, frozen, offline: true, now: 0, runner, registry, lock_base: Some(&factory), trust_base });
}

/// Review finding (the stamp bypasses the base-revision check):
/// a pass against one base must not let a frozen call against
/// another base skip `verify_added_since_base`.
#[test]
fn a_pass_against_one_trust_base_does_not_stand_for_another_base() {
    let root = repository(&refused_lock());
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    let against_the_old_base = step_with_base(&start, true, &runner, &downgrading_registry(), &refused_lock(), "base-one");
    let against_a_base_without_it = step_with_base(&start, true, &runner, &downgrading_registry(), "", "base-two");

    assert_eq!(against_the_old_base, StepOutcome::Passed);
    match against_a_base_without_it {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        other => panic!("the new base was never applied: {other:?}"),
    }
}

/// The same base and settings do take the fast path,
/// with no cargo call and no git:
/// the saving the stamp exists for.
#[test]
fn the_same_base_and_settings_take_the_fast_path() {
    let root = repository(&refused_lock());
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);
    assert_eq!(step_with_base(&start, true, &runner, &downgrading_registry(), &refused_lock(), "base-one"), StepOutcome::Passed);
    let before: usize = runner.commands.borrow().len();

    let again = run_install_step(&StepRequest {
        start_dir: &start,
        frozen: true,
        offline: true,
        now: 0,
        runner: &runner,
        registry: &downgrading_registry(),
        lock_base: Some(&|| { return Err("git must not be started on the fast path".to_string()); }),
        trust_base: "base-one",
    });

    assert_eq!(again, StepOutcome::FastPath);
    assert_eq!(runner.commands.borrow().len(), before);
}

/// A local install under `trustLockfile = true` holds only the versions new since the base to the policy:
/// an entry the base
/// already held is trusted,
/// one it did not is set aside (cargo here refuses every move,
/// so the install stops).
#[test]
fn a_local_install_under_trust_lockfile_checks_only_what_is_new_since_the_base() {
    let root = repository(&refused_lock());
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    let trusted = step_with_base(&start, false, &runner, &downgrading_registry(), &refused_lock(), "base-one");
    let new_since_the_base = step_with_base(&start, false, &runner, &downgrading_registry(), "", "base-two");

    assert_eq!(trusted, StepOutcome::Passed);
    match new_since_the_base {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        other => panic!("expected a refusal: {other:?}"),
    }
}

/// Delegated pick 3 (2026-11-03):
/// a frozen check that finds a policy refusal stops the command,
/// as pnpm's frozen install fails
/// on a rejected lockfile entry before any fetch.
/// A stale lockfile in a frozen check stays a warning plus `--locked`.
#[test]
fn a_frozen_policy_refusal_stops_the_command_and_a_frozen_stale_lock_does_not() {
    let root = repository(&refused_lock());
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    let refused = step(&start, true, &runner, &downgrading_registry());

    match refused {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        other => panic!("a frozen refusal must stop the command: {other:?}"),
    }
    fs::write(start.join("Cargo.lock"), STALE).expect("make stale");
    assert!(matches!(step(&start, true, &runner, &downgrading_registry()), StepOutcome::Failed(_)));
}
