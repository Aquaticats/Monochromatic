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
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            lock_base: None,
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

/// The older release (`0.9.0`) passes the policy,
/// but cargo refuses to move to it (it is outside what the manifests allow,
/// which only cargo knows),
/// so the finding stands and every lockfile is restored.
#[test]
fn a_refused_version_whose_older_releases_cargo_refuses_restores_every_lockfile() {
    let world = world(&lock_with("0.9.0"), "");
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let lock_path: PathBuf = world.root.path().join(CRATE_DIR).join("Cargo.lock");
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let cargo = move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
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
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
        }
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        fs::write(&lock_path, lock_with("1.0.1")).map_err(|error| { return error.to_string(); })?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
    // Typed binding: `first: Vec<VersionRecord>`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let first: Vec<VersionRecord> = vec![record("0.9.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")];
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), first)]) };
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
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
            // `None`: the present or the absent value of an `Option`.
            lock_base: None,
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
            // `None`: the present or the absent value of an `Option`.
            lock_base: None,
        },
        &UpdateKind::Full,
    )
    .expect("updates");

    assert!(report.written);
    assert!(report.pruned.is_empty());
    assert!(fs::read_to_string(&world.catalog).expect("catalog").contains("needed-elsewhere@1.0.0"));
}

/// Review finding (early-error rollback):
/// cargo fails before anything of this run was recorded as produced,
/// and another
/// writer published the lockfile meanwhile.
/// The rollback must leave that lockfile alone.
#[test]
fn a_cargo_failure_before_anything_was_produced_does_not_restore_over_another_writers_lock() {
    // Two roots: the call in the first fails without writing, and another writer publishes the second root's lockfile meanwhile.
    // (A writer that publishes inside the very window of a child's own run cannot be told from the child by any observation after
    // the fact; `file_owner_tests.rs` covers the attribution rule itself.)
    let world = world(&lock_with("0.9.0"), "");
    // Typed binding: `second_dir: PathBuf`.
    let second_dir: PathBuf = world.root.path().join("package/u/two");
    // Borrow `&second_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&second_dir).expect("second crate dir");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(second_dir.join("Cargo.lock"), lock_with("0.9.0")).expect("second lock");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `dirs: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let dirs: Vec<String> = vec![CRATE_DIR.to_string(), "package/u/two".to_string()];
    // Typed binding: `foreign: PathBuf`.
    let foreign: PathBuf = second_dir.join("Cargo.lock");
    let cargo = move |_cwd: &Path, _args: &[String]| -> Result<CargoOutcome, String> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&foreign`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&foreign, "published by another writer meanwhile").map_err(|error| { return error.to_string(); })?;
        return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
    };
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    let result = update_rust_lockfiles(&request, &UpdateKind::Full);

    assert!(result.is_err());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(second_dir.join("Cargo.lock")).expect("read"), "published by another writer meanwhile");
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

fn token_publish(num: &str, created_at: &str) -> VersionRecord {
    return record(num, created_at, "9", "");
}

fn trusted_publish(num: &str, created_at: &str) -> VersionRecord {
    return record(num, created_at, "", "o/r");
}

fn set_aside_outcome(history: Vec<VersionRecord>, resolved_version: &str, catalog_settings: &Settings) -> (UpdateReport, String) {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with(resolved_version));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };

    // Borrow `&world`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = run(&world, catalog_settings, &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    // Borrow `&world`: lends the value without copying or giving it away.
    return (report, lock_of(&world));
}

/// Review finding (caret class as boundary):
/// the manifest requires `>=1.0.0`,
/// the selected `2.0.0` is refused,
/// and `1.9.9`
/// passes.
/// The replacement boundary is what the manifest allows,
/// which cargo checks on `--precise`,
/// not the caret class of
/// the refused version.
#[test]
fn a_refused_release_may_be_replaced_by_an_older_release_of_an_earlier_major_version() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.9.9", "2026-02-01T00:00:00Z"),
        token_publish("2.0.0", "2026-03-01T00:00:00Z"),
    ];

    // Borrow `&settings`: lends the value without copying or giving it away.
    let (report, lock) = set_aside_outcome(history, "2.0.0", &settings());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock, lock_with("1.9.9"));
}

/// Review finding (cutoff applied unconditionally):
/// a crate exempt from the release age may be replaced by a young release.
#[test]
fn a_candidate_younger_than_the_cutoff_is_allowed_for_a_crate_exempt_from_the_release_age() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "2026-10-29T01:00:00Z"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `exempt: Settings`.
    let mut exempt: Settings = settings();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    exempt.minimum_release_age_exclude = vec!["dep".to_string()];

    // Borrow `&exempt`: lends the value without copying or giving it away.
    let (report, lock) = set_aside_outcome(history, "1.0.2", &exempt);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock, lock_with("1.0.1"));
}

/// Review finding:
/// with `minimumReleaseAgeStrict = false` the age is not a condition of the choice either.
#[test]
fn a_candidate_younger_than_the_cutoff_is_allowed_when_the_release_age_is_not_strict() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "2026-10-29T01:00:00Z"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];
    // `let mut`: a binding that may change.
    // Typed binding: `relaxed: Settings`.
    let mut relaxed: Settings = settings();
    relaxed.minimum_release_age_strict = false;

    // Borrow `&relaxed`: lends the value without copying or giving it away.
    let (report, lock) = set_aside_outcome(history, "1.0.2", &relaxed);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock, lock_with("1.0.1"));
}

/// Review finding (unknown timestamps):
/// a candidate whose publish time cannot be read does not pass the age condition.
#[test]
fn a_candidate_with_an_unreadable_publish_time_is_skipped_under_a_strict_release_age() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        trusted_publish("1.0.0", "2026-01-01T00:00:00Z"),
        trusted_publish("1.0.1", "garbled"),
        token_publish("1.0.2", "2026-02-01T00:00:00Z"),
    ];

    // Borrow `&settings`: lends the value without copying or giving it away.
    let (report, lock) = set_aside_outcome(history, "1.0.2", &settings());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock, lock_with("1.0.0"));
}

/// A registry with owners,
/// so the candidate trust evaluation can be compared with the final one.
struct RegistryWithOwners {
    inner: FakeRegistry,
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
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
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        return Ok(self.owners.clone());
    }
}

/// Review finding (owners omitted from candidate trust):
/// `1.0.1` by a stranger ranks below `1.0.0` by a listed owner once the
/// owners are known,
/// so it must not be chosen as the replacement;
/// the final check would refuse it.
#[test]
fn a_candidate_that_the_final_check_would_refuse_for_its_owners_is_not_chosen() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.2"));
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "1", ""),
        record("1.0.1", "2026-02-01T00:00:00Z", "2", ""),
        record("1.0.2", "2026-03-01T00:00:00Z", "9", ""),
    ];
    // Typed binding: `registry: RegistryWithOwners`.
    let registry: RegistryWithOwners = RegistryWithOwners {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        inner: FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) },
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        owners: ["1".to_string()].into_iter().collect(),
    };

    // Borrow `&world`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
    // `.clone()`: an independent owned copy.
    // Typed binding: `tried: Vec<String>`.
    let tried: Vec<String> = cargo.commands.borrow().clone();
    // Closure `|command|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!tried.iter().any(|command| { return command.ends_with("--precise 1.0.1"); }), "a candidate the final check refuses was tried: {tried:?}");
}

/// A cargo whose resolution under a publish-time cutoff fails (the manifest needs a young release) and whose unconstrained
/// resolution locks that young release.
fn cargo_needing_a_young_release(lock_path: PathBuf, young_lock: String) -> impl Fn(&Path, &[String]) -> Result<CargoOutcome, String> {
    return move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if args.iter().any(|argument| { return argument == "--publish-time"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "failed to select a version for `dep`".to_string(), stdout: String::new() });
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&lock_path`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&lock_path, &young_lock).map_err(|error| { return error.to_string(); })?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
}

/// Review finding (the global cutoff defeats exemptions):
/// the manifest needs a young release of a crate listed in
/// `minimumReleaseAgeExclude`.
/// Resolving under the strict cutoff cannot succeed,
/// but the exemption means the young release is
/// allowed,
/// so the resolution must not fail before the exemption is applied.
#[test]
fn a_young_release_that_a_manifest_needs_resolves_when_the_crate_is_exempt_from_the_release_age() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![trusted_publish("1.0.0", "2026-10-29T10:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
    // `let mut`: a binding that may change.
    // Typed binding: `exempt: Settings`.
    let mut exempt: Settings = settings();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    exempt.minimum_release_age_exclude = vec!["dep".to_string()];
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Full).expect("the exempt young release resolves");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("1.0.0"));
}

/// Without an exemption the strict cutoff still stops the resolution,
/// as before.
#[test]
fn a_young_release_that_a_manifest_needs_still_fails_a_strict_resolution_without_an_exemption() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    // Typed binding: `registry: FakeRegistry`.
    // Constructor `BTreeMap::new()`: an empty owned collection or a boxed value.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    let result = update_rust_lockfiles(&request, &UpdateKind::Full);

    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(result, Err(UpdateError::Cargo { .. })), "{result:?}");
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

/// The unconstrained resolution that an exemption allows does not let a crate that is not exempt through:
/// the final check
/// still refuses its young release and the lockfile is restored.
#[test]
fn a_crate_that_is_not_exempt_is_still_held_to_the_cutoff_after_the_unconstrained_resolution() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    let cargo = cargo_needing_a_young_release(world.root.path().join(CRATE_DIR).join("Cargo.lock"), lock_with("1.0.0"));
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![trusted_publish("1.0.0", "2026-10-29T10:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
    // `let mut`: a binding that may change.
    // Typed binding: `exempt_other: Settings`.
    let mut exempt_other: Settings = settings();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    exempt_other.minimum_release_age_exclude = vec!["other".to_string()];
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Full).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

const OLD_REQUIREMENT_MANIFEST: &str = "[package]\nname = \"u\"\nversion = \"0.1.0\"\n\n[dependencies]\ndep = \">=0.9.0\"\n";
const NEW_REQUIREMENT_MANIFEST: &str = "[package]\nname = \"u\"\nversion = \"0.1.0\"\n\n[dependencies]\ndep = \">=1.0.1\"\n";

/// A cargo whose `update --breaking` edits the manifest's requirement and locks `dep@1.0.1`,
/// and whose `metadata` lists the
/// manifest,
/// the way cargo's own breaking update rewrites `Cargo.toml`.
fn cargo_that_edits_the_manifest(directory: PathBuf) -> impl Fn(&Path, &[String]) -> Result<CargoOutcome, String> {
    return move |_cwd: &Path, args: &[String]| -> Result<CargoOutcome, String> {
        if args[0] == "metadata" {
            // Typed binding: `body: String`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            let body: String = format!(
                "{{\"workspace_root\":\"{}\",\"packages\":[{{\"source\":null,\"manifest_path\":\"{}\"}}]}}",
                directory.display(),
                directory.join("Cargo.toml").display()
            );
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            return Ok(CargoOutcome { ok: true, output: body.clone(), stdout: body });
        }
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if args.iter().any(|argument| { return argument == "--precise"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(directory.join("Cargo.toml"), NEW_REQUIREMENT_MANIFEST).map_err(|error| { return error.to_string(); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(directory.join("Cargo.lock"), lock_with("1.0.1")).map_err(|error| { return error.to_string(); })?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
}

fn breaking_update(dry_run: bool) -> (World, Result<UpdateReport, UpdateError>) {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };
    // Borrow `&request`: lends the value without copying or giving it away.
    let result = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args });
    return (world, result);
}

fn manifest_of(world: &World) -> String {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return fs::read_to_string(world.root.path().join(CRATE_DIR).join("Cargo.toml")).expect("read the manifest");
}

/// Review finding (`--breaking` edits outside the transaction):
/// cargo's breaking update rewrites the manifests.
/// When the
/// policy then refuses the release and nothing can be set aside,
/// the manifests go back with the lockfiles.
#[test]
fn a_refused_breaking_update_puts_the_edited_manifests_back_with_the_lockfiles() {
    let (world, result) = breaking_update_without_a_way_out();

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = result.expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST, "the manifest kept cargo's edit although the update was refused");
}

/// A dry run changes nothing either.
#[test]
fn a_dry_run_of_a_breaking_update_puts_the_edited_manifests_back() {
    let (world, result) = breaking_update(true);

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = result.expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

/// A breaking update the policy accepts keeps cargo's manifest edit.
#[test]
fn an_accepted_breaking_update_keeps_the_manifest_edit() {
    let (world, result) = breaking_update_that_passes();

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = result.expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_of(&world), NEW_REQUIREMENT_MANIFEST);
}

fn breaking_update_without_a_way_out() -> (World, Result<UpdateReport, UpdateError>) {
    return breaking_update(false);
}

fn breaking_update_that_passes() -> (World, Result<UpdateReport, UpdateError>) {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "");
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    // Typed binding: `trusted: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `request: UpdateRequest`.
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
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };
    // Borrow `&request`: lends the value without copying or giving it away.
    let result = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args });
    return (world, result);
}

/// A sync that fails,
/// the way `file-enforcer write` fails when it is not installed.
struct FailingSyncer;

impl Syncer for FailingSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("could not start file-enforcer".to_string());
    }
}

/// Delegated pick 7 (2026-11-03):
/// a sync failure after the catalog write makes the breaking update transactional,
/// so the
/// catalog goes back with the manifests and the lockfiles instead of being left raised and unrendered.
#[test]
fn a_sync_failure_after_the_catalog_write_restores_the_catalog_the_manifests_and_the_lockfiles() {
    // Typed binding: `catalog_text: &str`.
    let catalog_text: &str = "[catalog]\ndep = \">=0.9.0\"\n";
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), catalog_text);
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    // Typed binding: `trusted: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `request: UpdateRequest`.
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
        syncer: &FailingSyncer,
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Closure `|problem|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.iter().any(|problem| { return problem.contains("file-enforcer"); }), "{:?}", report.problems);
    // Borrow `&world.catalog`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&world.catalog).expect("catalog"), catalog_text);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_of(&world), lock_with("0.9.0"));
}

/// Review finding (2c 3):
/// `cargo update -p foo --precise 1.0.1` leaves the lockfile unchanged when `foo@1.0.1` is already locked.
/// The requested release is still held to the policy;
/// the old check looked only at what the command added.
#[test]
fn an_unchanged_precise_target_that_the_policy_refuses_is_still_refused() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("1.0.1"), "");
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving("");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--package", "dep@1.0.1", "--precise", "1.0.1"].iter().map(|argument| { return argument.to_string(); }).collect();

    // Borrow `&world`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written, "an unchanged but refused release was accepted");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
}

/// A syncer that edits the catalog behind the transaction's back and then fails.
struct ConcurrentCatalogSyncer {
    catalog: PathBuf,
}

impl Syncer for ConcurrentCatalogSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&self.catalog`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&self.catalog, "# edited by a person while the sync ran\n").map_err(|error| { return error.to_string(); })?;
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("could not start file-enforcer".to_string());
    }
}

/// Review finding (2c 6):
/// the rollback of a failed sync must not write the old catalog over an edit made meanwhile.
#[test]
fn a_catalog_edited_during_a_failing_sync_keeps_the_edit() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "[catalog]\ndep = \">=0.9.0\"\n");
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    let cargo = cargo_that_edits_the_manifest(directory);
    // Typed binding: `trusted: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // `.clone()`: an independent owned copy.
    // Typed binding: `syncer: ConcurrentCatalogSyncer`.
    let syncer: ConcurrentCatalogSyncer = ConcurrentCatalogSyncer { catalog: world.catalog.clone() };
    // Typed binding: `request: UpdateRequest`.
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
        syncer: &syncer,
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Borrow `&world.catalog`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&world.catalog).expect("catalog"), "# edited by a person while the sync ran\n");
}

/// A syncer that rewrites a manifest and then fails,
/// the way a partial `file-enforcer write` can.
struct PartialSyncer {
    manifest: PathBuf,
}

impl Syncer for PartialSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&self.manifest`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&self.manifest, "# rendered by a sync that then failed\n").map_err(|error| { return error.to_string(); })?;
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("file-enforcer write failed halfway".to_string());
    }
}

/// Review finding (2c 11):
/// what a failing sync wrote to the manifests is part of the transaction and goes back with it.
#[test]
fn a_manifest_a_failing_sync_rewrote_goes_back_with_the_transaction() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), "[catalog]\ndep = \">=0.9.0\"\n");
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    // `.clone()`: an independent owned copy.
    let cargo = cargo_that_edits_the_manifest(directory.clone());
    // Typed binding: `trusted: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `syncer: PartialSyncer`.
    let syncer: PartialSyncer = PartialSyncer { manifest: directory.join("Cargo.toml") };
    // Typed binding: `request: UpdateRequest`.
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
        syncer: &syncer,
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Borrow `&world`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_of(&world), OLD_REQUIREMENT_MANIFEST, "the manifest kept what the failed sync wrote");
}

/// A lockfile that holds two versions of the crate `dep`.
fn lock_with_two(first: &str, second: &str) -> String {
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{first}\"\nsource = \"{REGISTRY}\"\n\n[[package]]\nname = \"dep\"\nversion = \"{second}\"\nsource = \"{REGISTRY}\"\n");
}

/// Review finding (3c P1-2):
/// a full update that changes nothing still ends with a lockfile that holds a refused version;
/// the update
/// checks the lockfiles it leaves,
/// not only what it added.
#[test]
fn an_unchanged_full_update_still_refuses_a_refused_locked_version() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("1.0.1"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };

    // Borrow `&world`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Full).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written, "an update that leaves a refused version locked was kept");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.problems.is_empty());
}

/// Review finding (3c P1-2):
/// `cargo update -p dep@1.0.0` moves an edge to `1.0.1`,
/// which was already in the lockfile for another
/// edge;
/// the version pair is not new,
/// but the selection is.
#[test]
fn an_edge_moved_to_an_already_locked_refused_version_is_refused() {
    // Borrow `&lock_with_two`: lends the value without copying or giving it away.
    let world = world(&lock_with_two("1.0.0", "1.0.1"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "-p", "dep@1.0.0"].iter().map(|argument| { return argument.to_string(); }).collect();

    // Borrow `&world`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = run(&world, &settings(), &cargo, &registry, false, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written, "the refused selection was kept");
}

/// A reader of the base lockfile that gives the same text for every crate root.
fn base_holding(text: &str) -> impl Fn() -> Result<crate::install::LockBaseReader, String> + use<> {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `owned: String`.
    let owned: String = text.to_string();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Closure `|_crate_dir: &str|`: an anonymous function, like a TS arrow function.
    // `.clone()`: an independent owned copy.
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    return move || { return Ok(Box::new({ let held = owned.clone(); move |_crate_dir: &str| { return held.clone(); } }) as crate::install::LockBaseReader); };
}

/// Review finding (3c P1-2):
/// under `trustLockfile` the scope is "not at the base",
/// so a version the working lockfile already held
/// but the base did not is judged.
#[test]
fn under_trust_lockfile_a_version_the_base_did_not_hold_is_judged_even_if_the_working_lockfile_held_it() {
    // Borrow `&lock_with_two`: lends the value without copying or giving it away.
    let world = world(&lock_with_two("1.0.0", "1.0.1"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // `let mut`: a binding that may change.
    // Typed binding: `trusting: Settings`.
    let mut trusting: Settings = settings();
    trusting.trust_lockfile = true;
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let base = base_holding(&lock_with("1.0.0"));
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "-p", "dep@1.0.0"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let syncer = RecordingSyncer { calls: RefCell::new(0) };
    // Typed binding: `request: UpdateRequest`.
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &trusting,
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &syncer,
        prune_catalog: false,
        // `Some`: the present or the absent value of an `Option`.
        // Borrow `&base`: lends the value without copying or giving it away.
        lock_base: Some(&base),
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written, "a version new since the base was kept unchecked");
}

/// Under `trustLockfile` a version the base held is trusted:
/// the same update is kept.
#[test]
fn under_trust_lockfile_a_version_the_base_held_is_trusted() {
    // Borrow `&lock_with_two`: lends the value without copying or giving it away.
    let world = world(&lock_with_two("1.0.0", "1.0.1"), "");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.1"));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // `let mut`: a binding that may change.
    // Typed binding: `trusting: Settings`.
    let mut trusting: Settings = settings();
    trusting.trust_lockfile = true;
    // Borrow `&lock_with_two`: lends the value without copying or giving it away.
    let base = base_holding(&lock_with_two("1.0.0", "1.0.1"));
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "-p", "dep@1.0.0"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let syncer = RecordingSyncer { calls: RefCell::new(0) };
    // Typed binding: `request: UpdateRequest`.
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &trusting,
        catalog_path: &world.catalog,
        runner: &cargo,
        registry: &registry,
        forward: &[],
        syncer: &syncer,
        prune_catalog: false,
        // `Some`: the present or the absent value of an `Option`.
        // Borrow `&base`: lends the value without copying or giving it away.
        lock_base: Some(&base),
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.written, "{:?}", report.problems);
}

/// A syncer that writes bytes that are not UTF-8 into a manifest,
/// so reading the manifest back to record it fails.
struct UnreadableSyncer {
    manifest: PathBuf,
}

impl Syncer for UnreadableSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&self.manifest`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&self.manifest, [0xff_u8, 0xfe_u8]).map_err(|error| { return error.to_string(); })?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok("synced".to_string());
    }
}

/// Review finding (3c P2-3):
/// an error from recording the manifests around the sync must still put the raised catalog back.
#[test]
fn a_recording_error_around_the_sync_still_puts_the_catalog_back() {
    // Typed binding: `original_catalog: &str`.
    let original_catalog: &str = "[catalog]\ndep = \">=0.9.0\"\n";
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("0.9.0"), original_catalog);
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = world.root.path().join(CRATE_DIR);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(directory.join("Cargo.toml"), OLD_REQUIREMENT_MANIFEST).expect("manifest");
    // `.clone()`: an independent owned copy.
    let cargo = cargo_that_edits_the_manifest(directory.clone());
    // Typed binding: `trusted: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let trusted: Vec<VersionRecord> = vec![trusted_publish("0.9.0", "2026-01-01T00:00:00Z"), trusted_publish("1.0.1", "2026-02-01T00:00:00Z")];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), trusted)]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "--breaking"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `syncer: UnreadableSyncer`.
    let syncer: UnreadableSyncer = UnreadableSyncer { manifest: directory.join("Cargo.toml") };
    // Typed binding: `request: UpdateRequest`.
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
        syncer: &syncer,
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    let _outcome = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args });

    // Borrow `&world.catalog`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&world.catalog).expect("catalog"), original_catalog, "the raised catalog was left behind");
}

/// A syncer that counts its calls.
struct CountingSyncer {
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    calls: RefCell<u32>,
}

impl Syncer for CountingSyncer {
    fn sync(&self, _root: &Path) -> Result<String, String> {
        *self.calls.borrow_mut() += 1;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok("synced".to_string());
    }
}

/// Review finding (3c P2-2):
/// the manifest sync belongs to `--breaking` alone,
/// where the manifests are journaled before cargo runs;
/// an update without it never calls the sync,
/// so there is no manifest edit to snapshot.
#[test]
fn an_update_without_breaking_never_calls_the_manifest_sync() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let world = world(&lock_with("1.0.0"), "[catalog]\ndep = \">=0.9.0\"\n");
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `cargo: FakeCargo`.
    let cargo: FakeCargo = cargo_resolving(&lock_with("1.0.0"));
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `registry: FakeRegistry`.
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::from([("dep".to_string(), downgrading_history())]) };
    // Closure `|argument|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `args: Vec<String>`.
    let args: Vec<String> = ["update", "-p", "dep"].iter().map(|argument| { return argument.to_string(); }).collect();
    // Typed binding: `syncer: CountingSyncer`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let syncer: CountingSyncer = CountingSyncer { calls: RefCell::new(0) };
    // Typed binding: `request: UpdateRequest`.
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
        syncer: &syncer,
        prune_catalog: false,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
    };

    // Borrow `&request`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let _report = update_rust_lockfiles(&request, &UpdateKind::Targeted { cargo_args: &args }).expect("reports");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(*syncer.calls.borrow(), 0);
}

/// A cargo whose update writes the given lockfile, reports that a descendant could not be stopped (the innermost run's scope is
/// marked lost, as a failed shutdown does), and counts every call so that a later cargo operation is visible.
struct CargoThatLeavesWriters {
    /// The lockfile text its update writes.
    after: String,
    /// Every call, as its arguments joined by spaces.
    calls: RefCell<Vec<String>>,
}

/// How the fake runs.
impl CargoRunner for CargoThatLeavesWriters {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        self.calls.borrow_mut().push(args.join(" "));
        let resolves: bool = args[0] == "update" || args.iter().any(|argument| { return argument == "generate-lockfile"; });
        if resolves && !args.iter().any(|argument| { return argument == "--locked"; }) {
            fs::write(cwd.join("Cargo.lock"), &self.after).map_err(|error| { return error.to_string(); })?;
            crate::bounded_run::mark_scope_lost_for_test();
            return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    }
}

/// Acceptance rows A2 and R4 (certification, fourth pass): the separate update path stops with the lockfiles neither rolled back nor
/// accepted when a run cannot show that its writers stopped. Cargo wrote a lockfile different from its snapshot and its run reports
/// that a descendant may remain: `update_rust_lockfiles` fails terminally, the produced bytes are still live (no rollback), the
/// journal is kept unfinished, and cargo is asked nothing after the failed update.
#[test]
fn an_update_whose_run_cannot_show_quiescence_stops_without_rollback_or_acceptance_and_keeps_the_journal() {
    let world = world(&lock_with("0.9.0"), "");
    let runner = CargoThatLeavesWriters { after: lock_with("1.0.0"), calls: RefCell::new(Vec::new()) };
    let registry: FakeRegistry = FakeRegistry { histories: BTreeMap::new() };
    let request: UpdateRequest = UpdateRequest {
        root: world.root.path(),
        crate_dirs: &world.dirs,
        dry_run: false,
        now: now(),
        settings: &settings(),
        catalog_path: &world.catalog,
        runner: &runner,
        registry: &registry,
        forward: &[],
        syncer: &RecordingSyncer { calls: RefCell::new(0) },
        prune_catalog: false,
        lock_base: None,
    };

    let result = update_rust_lockfiles(&request, &UpdateKind::Full);

    let error: UpdateError = match result {
        Err(found) => found,
        Ok(report) => panic!("an update whose writers may remain must fail, got {report:?}; cargo was asked {:?}", runner.calls.borrow()),
    };
    assert!(error.to_string().contains("could not be stopped") && error.to_string().contains("journal"), "{error}");
    assert_eq!(lock_of(&world), lock_with("1.0.0"), "the produced lockfile was rolled back or changed under a writer that may still write it");
    let journals = crate::journal_support::journal_dirs(world.root.path());
    assert_eq!(journals.len(), 1, "the journal was finished or removed: {journals:?}");
    assert!(!journals[0].to_string_lossy().ends_with(".done"), "the journal was marked done: {journals:?}");
    let calls: Vec<String> = runner.calls.borrow().clone();
    assert_eq!(calls.len(), 1, "cargo was asked something after the failed resolution, or never asked: {calls:?}");
    assert!(calls[0].contains("generate-lockfile") || calls[0].starts_with("update"), "the resolution never ran, so the test shows nothing: {calls:?}");
}
