//! The unit tests of the install step,
//! ported from `install.unit.test.ts` against a fake cargo and a fake registry.

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
        trust_lockfile: false,
    };
}

/// Lock text that holds one registry package.
fn lock_with(name: &str, version: &str) -> String {
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

/// A fake cargo over files:
/// `update --locked` passes when the lock text is satisfied;
/// a plain `update` writes `after`.
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
        // A move to an older release (`--precise`) is one this fake cargo cannot make, so the refusal it answers stands.
        if args.iter().any(|argument| { return argument == "--precise"; }) {
            // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
            // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Ok(value)  is  return value;
            // // Err(error) is  throw error;   (the caller sees it as a returned value)
            // ```
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
            // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const list: string[] = [];   // or new Set() / new Map()
            // ```
            // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
            // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS has one type for this: string
            // ```
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
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

/// A repository with crate roots whose lockfile texts are given,
/// in the order listed.
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
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
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

/// Sixth pass,
/// group 7 (K1):
/// a frozen run starts no transaction,
/// and it still announces the kept files past the count,
/// once.
#[test]
fn a_frozen_install_announces_the_kept_files_past_the_count() {
    let (root, dirs) = repository(&[("package/a/one", SATISFIED)]);
    let kept = root.path().join(crate::journal::QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    for index in 0..=crate::journal::KEPT_FILES_WARNING_COUNT {
        fs::write(kept.join(format!("Cargo.lock.displaced.{index}")), "kept").expect("a kept file");
    }
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);
    let _ = crate::bounded_run::take_trace_for_test();

    run_install(root.path(), &dirs, &cargo, &empty_registry(), true, false).expect("passes");

    let announced: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("kept notice "); }).collect();
    assert_eq!(announced.len(), 1, "{announced:?}");
    assert!(announced[0].contains(&format!("{} files", crate::journal::KEPT_FILES_WARNING_COUNT + 1)), "{announced:?}");
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

/// A path-only workspace whose lockfile lacks the dependency `a` declares on `b`:
/// stale without any registry.
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
            crate::test_scratch::skip_or_fail("no cargo on PATH, so the real-cargo install case cannot run");
            return;
        }
    };
    let root = scratch();
    write_stale_path_workspace(root.path());
    let dirs: Vec<String> = vec!["package/real/ws".to_string()];
    // Typed binding: `cargo: crate::cargo_run::RealCargo`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
    // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let cargo: crate::cargo_run::RealCargo = crate::cargo_run::RealCargo { program, toolchain: String::new(), bound: std::time::Duration::from_secs(crate::cargo_run::CARGO_CALL_SECONDS) };
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
            lock_base: None,
            trust_base: "HEAD",
            known_consistent: false,
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
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    })
    .expect("passes");

    assert_eq!(*cargo.commands.borrow(), vec!["one :: update --workspace --locked".to_string()]);
}

/// What the other writer publishes:
/// a lockfile that still holds the refused release,
/// so the checks still refuse it.
const OTHER_WRITERS_LOCK: &str = "# written by another process\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n";

/// A registry that,
/// like a second process,
/// rewrites the lockfile while the policy check runs.
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
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        fs::write(&self.lock, OTHER_WRITERS_LOCK).expect("the other writer");
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
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    })
    .expect("reports");

    assert!(!report.written);
    assert_eq!(report.problems.len(), 1);
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(lock_text(root.path(), "package/a/stale"), OTHER_WRITERS_LOCK);
    assert!(report.warnings.iter().any(|warning| { return warning.contains("another process"); }), "{:?}", report.warnings);
}

#[test]
fn a_run_killed_after_cargo_wrote_is_finished_by_the_next_install() {
    let (root, dirs) = repository(&[("package/a/one", &lock_with("dep", "1.0.1"))]);
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    crate::journal_support::plant(root.path(), &[crate::crates::LockSnapshot { crate_dir: "package/a/one".to_string(), source: SATISFIED.to_string() }], &[]);
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

#[test]
fn a_frozen_install_also_checks_every_locked_version_against_the_policy_like_pnpm() {
    let (root, dirs) = repository(&[("package/a/one", &lock_with("dep", "1.0.1"))]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);
    let registry: FakeRegistry = FakeRegistry {
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")],
        )]),
    };

    let report = run_install(root.path(), &dirs, &cargo, &registry, true, false).expect("reports");

    assert!(report.stale.is_empty());
    assert_eq!(report.problems.len(), 1);
    assert!(report.problems[0].contains("dep@1.0.1 has trust level 1"));
    assert_eq!(lock_text(root.path(), "package/a/one"), lock_with("dep", "1.0.1"), "a frozen install writes nothing");
}

/// A frozen install of a satisfied lockfile that holds `dep@1.0.1`,
/// whose history the trust policy refuses,
/// under `trustLockfile`
/// with the given lockfile text at the base revision.
fn frozen_under_trust_lockfile(base_lock: &str, with_reader: bool) -> Result<InstallReport, InstallError> {
    let (root, dirs) = repository(&[("package/a/one", &lock_with("dep", "1.0.1"))]);
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);
    let registry: FakeRegistry = FakeRegistry {
        histories: BTreeMap::from([(
            "dep".to_string(),
            vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")],
        )]),
    };
    let mut catalog_settings: Settings = settings();
    catalog_settings.trust_lockfile = true;
    let base: String = base_lock.to_string();
    let factory = move || -> Result<crate::install::LockBaseReader, String> {
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        let text: String = base.clone();
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.clone()`: an independent owned copy.
        // Constructor `Box::new(`: an empty owned collection or a boxed value.
        return Ok(Box::new(move |_crate_dir: &str| -> String { return text.clone(); }));
    };
    return install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: true,
        offline: true,
        dry_run: false,
        now: 0,
        settings: &catalog_settings,
        runner: &cargo,
        registry: &registry,
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // Borrow `&factory`: lends the value without copying or giving it away.
        lock_base: if with_reader { Some(&factory) } else { None },
        trust_base: "HEAD",
        known_consistent: false,
    });
}

#[test]
fn under_trust_lockfile_a_version_the_base_already_held_is_not_checked() {
    let report = frozen_under_trust_lockfile(&lock_with("dep", "1.0.1"), true).expect("reports");

    assert!(report.problems.is_empty(), "{:?}", report.problems);
}

#[test]
fn under_trust_lockfile_a_version_new_since_the_base_is_checked() {
    let report = frozen_under_trust_lockfile(&lock_with("dep", "1.0.0"), true).expect("reports");

    assert_eq!(report.problems.len(), 1);
    assert!(report.problems[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn under_trust_lockfile_a_lockfile_absent_at_the_base_counts_as_new_in_full() {
    let report = frozen_under_trust_lockfile("", true).expect("reports");

    assert_eq!(report.problems.len(), 1);
}

#[test]
fn under_trust_lockfile_without_a_base_reader_the_install_refuses_instead_of_trusting_everything() {
    let result = frozen_under_trust_lockfile(&lock_with("dep", "1.0.1"), false);

    match result {
        Err(InstallError::Io { message }) => assert!(message.contains("trustLockfile"), "{message}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A fake cargo that fails in `package/a/one` after another writer published `package/a/two`.
struct CargoWithAConcurrentWriter {
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    root: PathBuf,
}

impl CargoRunner for CargoWithAConcurrentWriter {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if args.iter().any(|argument| { return argument == "--locked"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "the lock file needs to be updated".to_string(), stdout: String::new() });
        }
        if args[0] == "update" && cwd.ends_with("one") {
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            fs::write(self.root.join("package/a/two/Cargo.lock"), "published by another writer meanwhile").map_err(|error| { return error.to_string(); })?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    }
}

/// Review finding (early-error rollback):
/// cargo fails in the first root before anything was produced,
/// and another writer
/// updates the second root meanwhile.
/// The rollback must not read the second root's new content and call it its own output.
#[test]
fn a_failure_before_anything_was_produced_does_not_restore_over_another_writers_lock() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let (root, dirs) = repository(&[("package/a/one", STALE), ("package/a/two", SATISFIED)]);
    // Typed binding: `cargo: CargoWithAConcurrentWriter`.
    let cargo: CargoWithAConcurrentWriter = CargoWithAConcurrentWriter { root: root.path().to_path_buf() };
    // Typed binding: `settings_value: Settings`.
    let settings_value: Settings = settings();

    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
    let result = install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: false,
        offline: true,
        dry_run: false,
        now: 0,
        settings: &settings_value,
        runner: &cargo,
        registry: &empty_registry(),
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path(), "package/a/two"), "published by another writer meanwhile");
}

/// Review finding (failed begin):
/// without a complete recovery record nothing may be mutated,
/// so a journal that cannot be
/// started stops the install before cargo runs.
#[test]
fn an_install_whose_journal_cannot_be_started_runs_no_cargo_and_changes_nothing() {
    // Borrow `&[`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&[("package/a/one", STALE)]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(".cache"), "a file where the cache directory must go").expect("block the cache directory");
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_that_writes(SATISFIED);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    let result = run_install(root.path(), &dirs, &cargo, &empty_registry(), false, false);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err(), "{result:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(cargo.commands.borrow().is_empty(), "{:?}", cargo.commands);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path(), "package/a/one"), STALE);
}
