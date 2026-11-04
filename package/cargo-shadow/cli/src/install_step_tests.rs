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
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
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
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
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
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let history: Vec<VersionRecord> = vec![
        VersionRecord {
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            num: "1.0.0".to_string(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            created_at: "2026-01-01T00:00:00Z".to_string(),
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
            publisher_id: String::new(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            trustpub_repository: "o/r".to_string(),
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            repository: String::new(),
        },
        VersionRecord {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            num: "1.0.1".to_string(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            created_at: "2026-02-01T00:00:00Z".to_string(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            publisher_id: "9".to_string(),
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            trustpub_repository: String::new(),
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            repository: String::new(),
        },
    ];
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
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
    // Typed binding: `lock: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let lock: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    // Borrow `&lock`: lends the value without copying or giving it away.
    let root = repository(&lock);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock);
    let start = root.path().join(CRATE_DIR);
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"off\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("lenient catalog");
    // Borrow `&start`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(step(&start, true, &runner, &downgrading_registry()), StepOutcome::Passed);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("strict catalog");

    // Borrow `&start`: lends the value without copying or giving it away.
    let outcome = step(&start, true, &runner, &downgrading_registry());

    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    match outcome {
        // Macro `assert!`: builds or prints text (or a list) from a template.
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        // Macro `panic!`: builds or prints text (or a list) from a template.
        other => panic!("the stricter policy was never applied: {other:?}"),
    }
}

/// The lockfile used by the base tests:
/// it holds the refused `dep@1.0.1`.
fn refused_lock() -> String {
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
}

fn step_with_base(start: &Path, frozen: bool, runner: &FakeCargo, registry: &FakeRegistry, base_text: &str, trust_base: &str) -> StepOutcome {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `text: String`.
    let text: String = base_text.to_string();
    let factory = move || -> Result<crate::install::LockBaseReader, String> {
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // Typed binding: `held: String`.
        let held: String = text.clone();
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // `.clone()`: an independent owned copy.
        // Constructor `Box::new(`: an empty owned collection or a boxed value.
        return Ok(Box::new(move |_dir: &str| -> String { return held.clone(); }));
    };
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    return run_install_step(&StepRequest { start_dir: start, frozen, offline: true, now: 0, runner, registry, lock_base: Some(&factory), trust_base });
}

/// Review finding (the stamp bypasses the base-revision check):
/// a pass against one base must not let a frozen call against
/// another base skip `verify_added_since_base`.
#[test]
fn a_pass_against_one_trust_base_does_not_stand_for_another_base() {
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    let root = repository(&refused_lock());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    // Borrow `&start`: lends the value without copying or giving it away.
    let against_the_old_base = step_with_base(&start, true, &runner, &downgrading_registry(), &refused_lock(), "base-one");
    // Borrow `&start`: lends the value without copying or giving it away.
    let against_a_base_without_it = step_with_base(&start, true, &runner, &downgrading_registry(), "", "base-two");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(against_the_old_base, StepOutcome::Passed);
    // `match`: picks the arm whose pattern fits the value.
    match against_a_base_without_it {
        // Macro `assert!`: builds or prints text (or a list) from a template.
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        // Macro `panic!`: builds or prints text (or a list) from a template.
        other => panic!("the new base was never applied: {other:?}"),
    }
}

/// The same base and settings do take the fast path,
/// with no cargo call and no git:
/// the saving the stamp exists for.
#[test]
fn the_same_base_and_settings_take_the_fast_path() {
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    let root = repository(&refused_lock());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);
    // Borrow `&start`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(step_with_base(&start, true, &runner, &downgrading_registry(), &refused_lock(), "base-one"), StepOutcome::Passed);
    // Typed binding: `before: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let before: usize = runner.commands.borrow().len();

    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let again = run_install_step(&StepRequest {
        start_dir: &start,
        frozen: true,
        offline: true,
        now: 0,
        runner: &runner,
        registry: &downgrading_registry(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        lock_base: Some(&|| { return Err("git must not be started on the fast path".to_string()); }),
        trust_base: "base-one",
    });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(again, StepOutcome::FastPath);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(runner.commands.borrow().len(), before);
}

/// A local install under `trustLockfile = true` holds only the versions new since the base to the policy:
/// an entry the base
/// already held is trusted,
/// one it did not is set aside (cargo here refuses every move,
/// so the install stops).
#[test]
fn a_local_install_under_trust_lockfile_checks_only_what_is_new_since_the_base() {
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    let root = repository(&refused_lock());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\ntrustLockfile = true\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    // Borrow `&start`: lends the value without copying or giving it away.
    let trusted = step_with_base(&start, false, &runner, &downgrading_registry(), &refused_lock(), "base-one");
    // Borrow `&start`: lends the value without copying or giving it away.
    let new_since_the_base = step_with_base(&start, false, &runner, &downgrading_registry(), "", "base-two");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(trusted, StepOutcome::Passed);
    // `match`: picks the arm whose pattern fits the value.
    match new_since_the_base {
        // Macro `assert!`: builds or prints text (or a list) from a template.
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        // Macro `panic!`: builds or prints text (or a list) from a template.
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
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    let root = repository(&refused_lock());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    // Borrow `&refused_lock`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&refused_lock());
    let start = root.path().join(CRATE_DIR);

    // Borrow `&start`: lends the value without copying or giving it away.
    let refused = step(&start, true, &runner, &downgrading_registry());

    // `match`: picks the arm whose pattern fits the value.
    match refused {
        // Macro `assert!`: builds or prints text (or a list) from a template.
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1"), "{message}"),
        // Macro `panic!`: builds or prints text (or a list) from a template.
        other => panic!("a frozen refusal must stop the command: {other:?}"),
    }
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(start.join("Cargo.lock"), STALE).expect("make stale");
    // Borrow `&start`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(step(&start, true, &runner, &downgrading_registry()), StepOutcome::Failed(_)));
}

/// Review finding (2c 1):
/// a cargo that wrote a lockfile and then failed leaves the lockfile behind.
/// The rollback must put the
/// original back;
/// when it cannot,
/// the step must stop the command,
/// because `--locked` would build the unapproved output.
#[test]
fn a_restore_that_cannot_complete_stops_the_command_instead_of_building_the_unrestored_lockfile() {
    use std::os::unix::fs::PermissionsExt;
    if ignores_permissions() {
        return;
    }
    // Typed binding: `young: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let young: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    // `.clone()`: an independent owned copy.
    let directory = start.clone();
    // `.clone()`: an independent owned copy.
    let written = young.clone();
    // A cargo that writes the new lockfile, makes the directory unwritable (so no restore can be made) and exits with an error.
    let runner = move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
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
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        if arguments[0] == "update" {
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&written`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            fs::write(cwd.join("Cargo.lock"), &written).map_err(|error| { return error.to_string(); })?;
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&directory`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).map_err(|error| { return error.to_string(); })?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "failed after writing".to_string(), stdout: String::new() });
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });
    // Borrow `&start`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&start, fs::Permissions::from_mode(0o755)).expect("make the directory writable again");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, StepOutcome::Refused(_)), "the command would build the unrestored lockfile: {outcome:?}");
}

/// Review finding (2c 1,
/// the restorable case):
/// the original is back and the failure is operational.
#[test]
fn a_cargo_that_wrote_a_lockfile_and_failed_has_its_output_rolled_back() {
    // Typed binding: `young: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let young: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        if arguments[0] == "update" {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&young`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(cwd.join("Cargo.lock"), &young).map_err(|error| { return error.to_string(); })?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "failed after writing".to_string(), stdout: String::new() });
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, StepOutcome::Failed(_)), "{outcome:?}");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), STALE);
}

/// Review finding (2c 2,
/// end to end):
/// a lockfile from a registry other than crates.io is a refusal that stops the command,
/// not a parse problem that the shim answers with a build.
#[test]
fn a_package_from_an_unsupported_registry_stops_the_command() {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `other: String`.
    let other: String = "# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = \"registry+https://registry.example.invalid/index\"\n".to_string();
    // Borrow `&other`: lends the value without copying or giving it away.
    let root = repository(&other);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    // Borrow `&other`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&other);
    let start = root.path().join(CRATE_DIR);

    // Borrow `&start`: lends the value without copying or giving it away.
    let local = step(&start, false, &runner, &registry());
    // Borrow `&start`: lends the value without copying or giving it away.
    let frozen = step(&start, true, &runner, &registry());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(local, StepOutcome::Refused(_)), "local: {local:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(frozen, StepOutcome::Refused(_)), "frozen: {frozen:?}");
}

/// Whether the process may ignore directory permissions,
/// which makes a read-only directory no obstacle.
fn ignores_permissions() -> bool {
    use std::os::unix::fs::MetadataExt;
    // Closure `|metadata|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return fs::metadata("/proc/self").map(|metadata| { return metadata.uid() == 0; }).unwrap_or(true);
}

/// Review finding (3c P1-1):
/// cargo succeeds,
/// the policy refuses what it wrote,
/// and the rollback cannot complete.
/// A policy report is a result,
/// not an error,
/// so the rollback failure must not turn it into a warn-and-run fallback.
#[test]
fn a_policy_refusal_whose_rollback_cannot_complete_stops_the_command() {
    use std::os::unix::fs::PermissionsExt;
    if ignores_permissions() {
        return;
    }
    // Typed binding: `young: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let young: String = format!("# refused\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    // `.clone()`: an independent owned copy.
    let directory = start.clone();
    let runner = move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if arguments[0] == "update" && !arguments.iter().any(|argument| { return argument == "--precise"; }) {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&young`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(cwd.join("Cargo.lock"), &young).map_err(|error| { return error.to_string(); })?;
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&directory`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).map_err(|error| { return error.to_string(); })?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &downgrading_registry(), lock_base: None, trust_base: "HEAD" });
    // Borrow `&start`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&start, fs::Permissions::from_mode(0o755)).expect("make the directory writable again");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, StepOutcome::Refused(_)), "the command would build the refused lockfile: {outcome:?}");
}

/// A cargo that writes a lockfile with a refused release and succeeds,
/// then answers every other call without changing anything.
fn cargo_that_writes_a_refused_release() -> impl Fn(&Path, &[String]) -> Result<CargoOutcome, String> {
    // Typed binding: `young: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let young: String = format!("# refused\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    return move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if arguments[0] == "update" && !arguments.iter().any(|argument| { return argument == "--precise"; }) {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&young`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(cwd.join("Cargo.lock"), &young).map_err(|error| { return error.to_string(); })?;
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
}

/// Review finding (4c F1):
/// the policy refused the output,
/// the rollback worked,
/// and finishing the journal failed;
/// the refusal still
/// stops the command (an operational error would let the locked fallback build the consistent,
/// refused lockfile).
#[test]
fn a_refusal_survives_a_journal_that_cannot_be_finished() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = cargo_that_writes_a_refused_release();
    crate::failpoints::arm(crate::failpoints::Point::FinishDurability);

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &downgrading_registry(), lock_base: None, trust_base: "HEAD" });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, StepOutcome::Refused(_)), "the refusal was replaced: {outcome:?}");
}

/// Review finding (4c,
/// regression coverage):
/// the rollback failure of the policy-refusal case,
/// provoked without permissions so that
/// it also runs for a root user.
/// Round 6 (5c N1):
/// when the policy accepted the run but the outputs could not be forced to disk,
/// the command must not run on a lockfile that is not durable.
#[test]
fn an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::FinishDurability);

    let outcome = step(&start, false, &runner, &registry());

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("disk"), "{message}"),
        other => panic!("a failed durability barrier read as {other:?}"),
    }
}

/// Round 6 (5c N1):
/// a journal that cannot be removed after its completion marker was made durable is only a warning.
#[test]
fn an_accepted_run_whose_journal_cannot_be_removed_afterwards_still_passes() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::FinishCleanup);

    let outcome = step(&start, false, &runner, &registry());

    assert_eq!(outcome, StepOutcome::Updated { crate_dir: CRATE_DIR.to_string(), added: Vec::new() });
}

/// Round 6 (5c N1):
/// a refusal survives a failed durability barrier and the message says the barrier failed too.
#[test]
fn a_refusal_with_a_failed_durability_barrier_keeps_the_refusal_and_names_the_barrier() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = cargo_that_writes_a_refused_release();
    crate::failpoints::arm(crate::failpoints::Point::FinishDurability);

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &downgrading_registry(), lock_base: None, trust_base: "HEAD" });

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1") && message.contains("disk"), "{message}"),
        other => panic!("the refusal was replaced: {other:?}"),
    }
}

#[test]
fn a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = cargo_that_writes_a_refused_release();
    crate::failpoints::arm(crate::failpoints::Point::Restore);

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &downgrading_registry(), lock_base: None, trust_base: "HEAD" });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, StepOutcome::Refused(_)), "the command would build the refused lockfile: {outcome:?}");
}

/// A cargo whose update writes the given lockfile and leaves a process that could not be stopped.
struct CargoThatLeavesWriters {
    /// The lockfile text its update writes.
    after: String,
}

/// How the fake runs.
impl CargoRunner for CargoThatLeavesWriters {
    /// What:
    /// `fn run(&self, cwd, args)` writes the lockfile on `update` and reports that its descendants could not be stopped.
    /// Why:
    /// The command must stop when writers may remain,
    /// whatever the policy says about the bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(cwd: string, args: string[]): CargoOutcome
    /// ```
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        if args[0] == "update" && !args.iter().any(|argument| { return argument == "--locked"; }) {
            fs::write(cwd.join("Cargo.lock"), &self.after).map_err(|error| { return error.to_string(); })?;
            crate::bounded_run::mark_scope_lost_for_test();
            return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
        }
        return cargo(&self.after).run(cwd, args);
    }
}

/// Round 6 (5c N2):
/// when a descendant of cargo could not be stopped,
/// the lockfiles are neither rolled back nor accepted,
/// the journal is kept,
/// and the command does not run.
#[test]
fn a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = CargoThatLeavesWriters { after: SATISFIED.to_string() };

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("may still") && message.contains("journal"), "{message}"),
        other => panic!("a run with writers that may remain read as {other:?}"),
    }
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), SATISFIED, "the lockfile was rolled back under a writer that may still write it");
    let journals = crate::journal_support::journal_dirs(root.path());
    assert_eq!(journals.len(), 1, "the journal was not kept: {journals:?}");
}

/// Round 7 (6c R1):
/// cargo failed (an operational error) and the mandatory durability barrier failed too:
/// the command stops,
/// because an operational error alone lets the shim run the command again with `--locked`.
#[test]
fn an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = |_cwd: &Path, _arguments: &[String]| -> Result<CargoOutcome, String> {
        return Ok(CargoOutcome { ok: false, output: "cargo broke".to_string(), stdout: String::new() });
    };
    crate::failpoints::arm(crate::failpoints::Point::FinishDurability);

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("disk") && message.contains("cargo broke"), "{message}"),
        other => panic!("an operational error hid a failed barrier and the shim would fall back: {other:?}"),
    }
}

/// Round 7 (6b2 5):
/// a completion marker that could not be published is not a cleanup problem:
/// a later recovery would still see an active journal and could judge the run again.
#[test]
fn a_completion_marker_that_cannot_be_published_stops_the_command() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::FinishCompletion);

    let outcome = step(&start, false, &runner, &registry());

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("completion"), "{message}"),
        other => panic!("a failed completion marker read as {other:?}"),
    }
}

/// Round 7 (6b2 5):
/// the cleanup failure point is after the durable marker:
/// the journal is already named `.done` when the removal fails.
#[test]
fn the_cleanup_failure_happens_after_the_completion_marker_was_published() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::FinishCleanup);

    let outcome = step(&start, false, &runner, &registry());

    assert_eq!(outcome, StepOutcome::Updated { crate_dir: CRATE_DIR.to_string(), added: Vec::new() });
    let journals = crate::journal_support::journal_dirs(root.path());
    assert!(journals.iter().any(|journal| { return journal.to_string_lossy().ends_with(".done"); }), "no completion marker at the point of the cleanup failure: {journals:?}");
}

/// Acceptance row A3, A4 (the actual completion rename):
/// an accepted run whose completion rename fails stops the command.
#[test]
fn an_accepted_run_whose_completion_rename_fails_stops_the_command() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::CompletionRename);

    let outcome = step(&start, false, &runner, &registry());

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("completion"), "{message}"),
        other => panic!("a failed completion rename read as {other:?}"),
    }
}

/// Acceptance row A3, A4 (the actual completion-parent sync):
/// an accepted run whose sync of the marker's directory fails stops the command.
#[test]
fn an_accepted_run_whose_completion_directory_sync_fails_stops_the_command() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner: FakeCargo = cargo(SATISFIED);
    crate::failpoints::arm(crate::failpoints::Point::CompletionSync);

    let outcome = step(&start, false, &runner, &registry());

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("completion"), "{message}"),
        other => panic!("a failed completion directory sync read as {other:?}"),
    }
}

/// Acceptance row A3 (refused outcome):
/// a refusal survives a failed completion marker and the message names the marker too.
#[test]
fn a_refusal_survives_a_failed_completion_marker_and_names_it() {
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let runner = cargo_that_writes_a_refused_release();
    crate::failpoints::arm(crate::failpoints::Point::CompletionRename);

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &downgrading_registry(), lock_base: None, trust_base: "HEAD" });

    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("dep@1.0.1") && message.contains("completion"), "{message}"),
        other => panic!("the refusal was replaced: {other:?}"),
    }
}

/// Acceptance row R3:
/// cargo failed after writing a lockfile and the rollback fails too:
/// the step stops the command (so the `--locked` fallback never builds the unrestored lockfile),
/// without depending on permissions.
#[test]
fn a_cargo_failure_followed_by_a_failed_rollback_stops_the_command_instead_of_falling_back() {
    let young: String = format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.1\"\nsource = \"{REGISTRY}\"\n");
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let unlocked_update_ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let seen_by_runner = std::sync::Arc::clone(&unlocked_update_ran);
    let written = young.clone();
    let runner = move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        if arguments[0] == "update" {
            fs::write(cwd.join("Cargo.lock"), &written).map_err(|error| { return error.to_string(); })?;
            seen_by_runner.store(true, std::sync::atomic::Ordering::Relaxed);
            return Ok(CargoOutcome { ok: false, output: "failed after writing".to_string(), stdout: String::new() });
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };
    crate::failpoints::arm(crate::failpoints::Point::Restore);

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });

    // Both faults were reached: cargo failed after it wrote the lockfile, and the restore fault was consumed by the rollback.
    assert!(unlocked_update_ran.load(std::sync::atomic::Ordering::Relaxed), "cargo never wrote the lockfile, so the test shows nothing");
    assert!(!crate::failpoints::fires(crate::failpoints::Point::Restore), "the rollback never reached the restore");
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), young, "the unrestored lockfile is not the one cargo wrote");
    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("could not be put back"), "the refusal is not the rollback failure: {message}"),
        other => panic!("the command would fall back onto the unrestored lockfile: {other:?}"),
    }
}

/// Acceptance row A2, R4 (joined, 7a F3):
/// a real run through the bounded runner that cannot show its scope empty,
/// as cargo's update inside the install step:
/// the step stops, the lockfile cargo wrote is neither rolled back nor accepted, the journal is kept,
/// and nothing runs after it (the user's command is not started).
#[test]
fn a_real_run_that_cannot_show_quiescence_stops_the_install_with_the_lockfile_kept_and_nothing_after_it() {
    if crate::cgroup_test_support::delegated_route().is_none() {
        return;
    }
    let root = repository(STALE);
    let start = root.path().join(CRATE_DIR);
    let ready = root.path().join("writer-ready");
    let calls: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::clone(&calls);
    let script: String = format!(
        "printf '# satisfied\\nversion = 4\\n' > Cargo.lock; setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; exit 0",
        ready = ready.display()
    );
    let runner = move |cwd: &Path, arguments: &[String]| -> Result<CargoOutcome, String> {
        seen.lock().expect("the call log").push(arguments.join(" "));
        if arguments[0] == "update" && arguments.iter().any(|argument| { return argument == "--locked"; }) {
            return Ok(CargoOutcome { ok: false, output: "needs update".to_string(), stdout: String::new() });
        }
        if arguments[0] == "update" {
            crate::failpoints::arm(crate::failpoints::Point::ContainmentStopStuck);
            let bound = crate::bounded_run::Bound { limit: std::time::Duration::from_secs(30), what: "cargo".to_string(), rerun: "cargo".to_string() };
            let mut command = std::process::Command::new("sh");
            command.current_dir(cwd).args(["-c", &script]);
            return match crate::bounded_run::output_within(&mut command, None, &bound) {
                Ok(_) => Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() }),
                Err(error) => Err(error.to_string()),
            };
        }
        return Ok(CargoOutcome { ok: true, output: String::new(), stdout: String::new() });
    };

    let outcome = run_install_step(&StepRequest { start_dir: &start, frozen: false, offline: true, now: 0, runner: &runner, registry: &registry(), lock_base: None, trust_base: "HEAD" });

    let journals = crate::journal_support::journal_dirs(root.path());
    let scope_path = journals.first().map(|journal| { return crate::cgroup_test_support::scope_directory_of(&journal.join("groups").display().to_string()); });
    if let Some(scope) = &scope_path {
        let _ = fs::write(scope.join("cgroup.kill"), "1");
    }
    match outcome {
        StepOutcome::Refused(message) => assert!(message.contains("may still") && message.contains("journal"), "{message}"),
        other => panic!("a run that could not show quiescence read as {other:?}"),
    }
    assert_eq!(fs::read_to_string(start.join("Cargo.lock")).expect("lock"), SATISFIED, "the lockfile cargo wrote was rolled back or replaced");
    assert_eq!(journals.len(), 1, "the journal was not kept: {journals:?}");
    let log: Vec<String> = calls.lock().expect("the call log").clone();
    assert!(log.last().is_some_and(|last| { return last.starts_with("update") && !last.contains("--locked"); }), "something ran after the run that could not be stopped: {log:?}");
}
