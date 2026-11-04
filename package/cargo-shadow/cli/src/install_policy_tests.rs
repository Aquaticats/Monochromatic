//! The unit tests of the policy checks of a local (not frozen) install:
//! a committed lockfile that holds a release under the age limit or a trust downgrade is changed to the newest older release
//! that passes,
//! and the install stops when none does.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::cell::RefCell;
// Imports `std::collections::{BTreeMap, BTreeSet}`.
use std::collections::{BTreeMap, BTreeSet};
// Imports `std::fs`.
use std::fs;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};

// Imports `monochromatic_dependency_catalog::{Settings...`.
use monochromatic_dependency_catalog::{Settings, TrustPolicy};

// Imports `crate::cargo_run::{CargoOutcome, CargoRunne...`.
use crate::cargo_run::{CargoOutcome, CargoRunner};
// Imports `crate::http::RegistryError`.
use crate::http::RegistryError;
// Imports `crate::install::{InstallError, InstallRepor...`.
use crate::install::{InstallError, InstallReport, InstallRequest, install_rust_lockfiles};
// Imports `crate::policy::VersionRecord`.
use crate::policy::VersionRecord;
// Imports `crate::registry::Registry`.
use crate::registry::Registry;
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;
// Imports `crate::time_text::parse_rfc3339_millis`.
use crate::time_text::parse_rfc3339_millis;

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const CRATE_DIR: &str = "package/p/one";
const STALE: &str = "# stale\nversion = 4\n";

fn now() -> i64 {
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    return parse_rfc3339_millis("2026-10-29T12:00:00Z").expect("a fixed time");
}

/// One day of release age,
/// strict,
/// trust policy on,
/// every locked entry checked.
fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 1_440,
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: false,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

fn lock_with(version: &str) -> String {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return format!("# satisfied\nversion = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n");
}

/// A fake cargo:
/// `update --locked` is satisfied by a lock whose first line says so;
/// a plain `update` writes `after`;
/// `update --package n@v --precise c` swaps the version unless `refuse_precise` is set.
struct FakeCargo {
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    after: String,
    refuse_precise: bool,
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    commands: RefCell<Vec<String>>,
}

impl CargoRunner for FakeCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        self.commands.borrow_mut().push(args.join(" "));
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        let lock: PathBuf = cwd.join("Cargo.lock");
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        let ok = CargoOutcome { ok: true, output: String::new(), stdout: String::new() };
        if args[0] == "metadata" {
            // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
            // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Ok(value)  is  return value;
            // // Err(error) is  throw error;   (the caller sees it as a returned value)
            // ```
            return Ok(ok);
        }
        if args[0] != "update" {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(ok);
        }
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
        if args.iter().any(|argument| { return argument == "--locked"; }) {
            // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
            // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
            // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
            // ```
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            // Typed binding: `satisfied: bool`.
            let satisfied: bool = fs::read_to_string(&lock).unwrap_or_default().starts_with("# satisfied");
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            return Ok(CargoOutcome { ok: satisfied, output: "needs update".to_string(), stdout: String::new() });
        }
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Closure `|argument|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if let Some(position) = args.iter().position(|argument| { return argument == "--precise"; }) {
            if self.refuse_precise {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                // Constructor `String::new()`: an empty owned collection or a boxed value.
                return Ok(CargoOutcome { ok: false, output: "failed to select a version".to_string(), stdout: String::new() });
            }
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            // Typed binding: `from: String`.
            let from: String = args[position - 1].split('@').nth(1).unwrap_or("").to_string();
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&lock`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            // Typed binding: `text: String`.
            let text: String = fs::read_to_string(&lock).map_err(|error| { return error.to_string(); })?;
            // Borrow `&lock`: lends the value without copying or giving it away.
            // Macro `format!`: builds or prints text (or a list) from a template.
            fs::write(&lock, text.replace(&format!("version = \"{from}\""), &format!("version = \"{}\"", args[position + 1])))
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Closure `|error|`: an anonymous function, like a TS arrow function.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                .map_err(|error| { return error.to_string(); })?;
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(ok);
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&lock`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        fs::write(&lock, &self.after).map_err(|error| { return error.to_string(); })?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(ok);
    }
}

fn cargo(after: &str, refuse_precise: bool) -> FakeCargo {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    return FakeCargo { after: after.to_string(), refuse_precise, commands: RefCell::new(Vec::new()) };
}

/// A registry that serves the canned histories and derives the publish times from them.
struct FakeRegistry {
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    histories: BTreeMap<String, Vec<VersionRecord>>,
}

impl Registry for FakeRegistry {
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `times: BTreeMap<String, String>`.
        // Constructor `BTreeMap::new()`: an empty owned collection or a boxed value.
        let mut times: BTreeMap<String, String> = BTreeMap::new();
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        for record in self.histories.get(name).cloned().unwrap_or_default() {
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            times.insert(record.num.clone(), record.created_at.clone());
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(times);
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        return Ok(BTreeSet::new());
    }
    fn history(&self, name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        return Ok(self.histories.get(name).cloned().unwrap_or_default());
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        return Ok(BTreeSet::new());
    }
}

fn record(num: &str, created_at: &str, publisher_id: &str, trustpub_repository: &str) -> VersionRecord {
    return VersionRecord {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        num: num.to_string(),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        created_at: created_at.to_string(),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        publisher_id: publisher_id.to_string(),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        trustpub_repository: trustpub_repository.to_string(),
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        repository: String::new(),
    };
}

fn registry_of(history: Vec<VersionRecord>) -> FakeRegistry {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    return FakeRegistry { histories: BTreeMap::from([("dep".to_string(), history)]) };
}

fn repository(lock: &str) -> (tempfile::TempDir, Vec<String>) {
    let root = scratch();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("crate dir");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    return (root, vec![CRATE_DIR.to_string()]);
}

fn lock_text(root: &Path) -> String {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("read the lock");
}

fn install(root: &Path, dirs: &[String], runner: &FakeCargo, registry: &FakeRegistry, frozen: bool) -> Result<InstallReport, InstallError> {
    // Typed binding: `catalog_settings: Settings`.
    let catalog_settings: Settings = settings();
    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
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
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    });
}

const OLD_ENOUGH: &str = "2026-10-20T00:00:00Z";
const THREE_HOURS_AGO: &str = "2026-10-29T09:00:00Z";

/// Owner case 1:
/// a merge brings a lockfile holding a release published 3 hours ago,
/// the limit is one day,
/// the release before
/// it is nine days old and fits the range.
/// The lockfile changes to the older release,
/// one line names both,
/// the install goes on.
#[test]
fn a_committed_lock_holding_a_release_under_the_age_limit_is_changed_to_the_newest_older_release() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(
        // Closure `|line|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        report.warnings.iter().any(|line| { return line.contains("dep@1.0.1") && line.contains("1.0.0"); }),
        "one line must name both releases: {:?}",
        report.warnings
    );
}

/// Owner case 1,
/// when nothing older fits:
/// the install stops with the finding and the lockfile is left as it was.
#[test]
fn a_committed_lock_holding_a_young_release_with_no_older_release_that_fits_stops_the_install() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), true);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("reports");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!report.written);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
}

/// Owner case 2:
/// the lockfile holds `1.0.1` from an account with no earlier releases while `1.0.0` came from the trusted
/// publisher.
/// The lockfile changes to `1.0.0`.
#[test]
fn a_committed_lock_holding_a_trust_downgrade_is_changed_to_the_earlier_release_with_the_earlier_trust_level() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", "2026-10-21T00:00:00Z", "9", "")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
    // Closure `|line|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.warnings.iter().any(|line| { return line.contains("dep@1.0.1") && line.contains("1.0.0"); }), "{:?}", report.warnings);
}

/// Owner case 2,
/// when no earlier release fits the range (cargo refuses every move):
/// the install stops.
#[test]
fn a_committed_trust_downgrade_with_no_earlier_release_that_fits_stops_the_install() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), true);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", "2026-10-21T00:00:00Z", "9", "")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("reports");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
}

/// The adopted rule for a version the install itself newly needs:
/// a stale lockfile is updated offline,
/// the update picks a
/// release under the age limit,
/// and that release is set aside for the newest older one that passes instead of being refused.
#[test]
fn a_release_the_update_newly_needs_under_the_age_limit_is_set_aside_instead_of_refused() {
    let (root, dirs) = repository(STALE);
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, false).expect("installs");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
}

/// CI keeps the frozen behavior:
/// a frozen install never changes the lockfile,
/// so a refused entry is a finding and no
/// `--precise` move is attempted.
#[test]
fn a_frozen_install_reports_the_refused_entry_and_changes_nothing() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);

    // Borrow `&dirs`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `report: InstallReport`.
    let report: InstallReport = install(root.path(), &dirs, &runner, &registry, true).expect("reports");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.1"));
    // Closure `|command|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!runner.commands.borrow().iter().any(|command| { return command.contains("--precise"); }));
}

/// A registry that,
/// the first time the policy check asks it anything,
/// runs a second install over the same root,
/// the way a
/// second shadow invocation would while the first one waits for crates.io.
struct RegistryThatRunsASecondInstall {
    inner: FakeRegistry,
    root: PathBuf,
    dirs: Vec<String>,
    started: std::cell::Cell<bool>,
    second: RefCell<Option<Result<InstallReport, InstallError>>>,
}

impl Registry for RegistryThatRunsASecondInstall {
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        if !self.started.replace(true) {
            // Typed binding: `runner: FakeCargo`.
            let runner: FakeCargo = cargo("", false);
            // `.clone()`: an independent owned copy.
            // Typed binding: `plain: FakeRegistry`.
            let plain: FakeRegistry = FakeRegistry { histories: self.inner.histories.clone() };
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&self.root`: lends the value without copying or giving it away.
            *self.second.borrow_mut() = Some(install(&self.root, &self.dirs, &runner, &plain, false));
        }
        return self.inner.pubtimes(name);
    }
    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return self.inner.yanked(name);
    }
    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        return self.inner.history(name, versions);
    }
    fn owners(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return self.inner.owners(name);
    }
}

/// Review finding (unapproved lockfiles are visible to concurrent consumers):
/// cargo writes the live lockfile before the
/// registry checks approve it,
/// so a second invocation can start from that intermediate lockfile.
/// It must not take it for
/// approved:
/// the only approval is a remembered pass of that exact text,
/// so the second invocation checks it itself and never
/// builds on the refused release.
#[test]
fn a_second_install_does_not_take_the_first_ones_unapproved_lockfile_for_approved() {
    let (root, dirs) = repository(STALE);
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: RegistryThatRunsASecondInstall`.
    let registry: RegistryThatRunsASecondInstall = RegistryThatRunsASecondInstall {
        // Macro `vec!`: builds or prints text (or a list) from a template.
        inner: registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]),
        root: root.path().to_path_buf(),
        // `.clone()`: an independent owned copy.
        dirs: dirs.clone(),
        // Constructor `Cell::new(`: an empty owned collection or a boxed value.
        started: std::cell::Cell::new(false),
        // `None`: the present or the absent value of an `Option`.
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        second: RefCell::new(None),
    };

    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
    let first = install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: false,
        offline: true,
        dry_run: false,
        now: now(),
        settings: &settings(),
        runner: &runner,
        registry: &registry,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    })
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    .expect("the first install reports");

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `second: InstallReport`.
    // Review finding (4b2 1): the workspace lock keeps a second transaction out while the first holds the unapproved output, so the
    // second install stops with an error naming the holder instead of taking that lockfile for approved.
    let second = registry.second.borrow_mut().take().expect("the second install ran");
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `message: String`.
    let message: String = second.err().map(|error| { return error.to_string(); }).unwrap_or_default();
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("another cargo-shadow run"), "{message}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(first.problems.is_empty(), "{:?}", first.problems);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock_text(root.path()), lock_with("1.0.0"));
}

/// A registry that serves the canned histories but fails the owner lookup,
/// the way a rate-limited or offline registry does.
struct RegistryWithoutOwners {
    inner: FakeRegistry,
}

impl Registry for RegistryWithoutOwners {
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
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err(RegistryError { message: "the owners request failed".to_string(), status: None, retry_after_seconds: 0 });
    }
}

/// Review finding (2c 2):
/// the release-age refusal is already known when the owners request fails.
/// The failure must not erase it
/// and turn a refusal into an operational problem that the shim answers with a build.
#[test]
fn a_lookup_failure_after_a_release_age_refusal_keeps_the_refusal() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `young_downgrade: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let young_downgrade: Vec<VersionRecord> = vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "9", "")];
    // Typed binding: `registry: RegistryWithoutOwners`.
    let registry: RegistryWithoutOwners = RegistryWithoutOwners { inner: registry_of(young_downgrade) };
    // Typed binding: `catalog_settings: Settings`.
    let catalog_settings: Settings = settings();

    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
    let report = install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: true,
        offline: true,
        dry_run: false,
        now: now(),
        settings: &catalog_settings,
        runner: &runner,
        registry: &registry,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    })
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    .expect("a known refusal is a report, not an operational error");

    // Closure `|problem|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.iter().any(|problem| { return problem.contains("minimumReleaseAge"); }), "{:?}", report.problems);
}

/// Review finding (2c 8):
/// a young release that is allowed because the release age is not strict still gets its warning,
/// on the
/// frozen path as on the local one,
/// and a pass with a warning is not remembered.
#[test]
fn the_non_strict_release_age_warning_is_reported_on_the_frozen_path() {
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    let (root, dirs) = repository(&lock_with("1.0.1"));
    // Borrow `&lock_with`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&lock_with("1.0.1"), false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);
    // `let mut`: a binding that may change.
    // Typed binding: `relaxed: Settings`.
    let mut relaxed: Settings = settings();
    relaxed.minimum_release_age_strict = false;

    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
    let report = install_rust_lockfiles(&InstallRequest {
        root: root.path(),
        crate_dirs: &dirs,
        frozen: true,
        offline: true,
        dry_run: false,
        now: now(),
        settings: &relaxed,
        runner: &runner,
        registry: &registry,
        // `None`: the present or the absent value of an `Option`.
        lock_base: None,
        trust_base: "HEAD",
        known_consistent: false,
    })
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    .expect("passes");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    // Closure `|warning|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(report.warnings.iter().any(|warning| { return warning.contains("minimumReleaseAge"); }), "the warning was dropped: {:?}", report.warnings);
}

/// Review finding (3c P2-1):
/// a trust finding that the owners of the crate could still clear is provisional;
/// when the owners request
/// fails it is an incomplete evaluation (an operational failure),
/// not a refusal.
#[test]
fn an_owners_lookup_failure_does_not_turn_a_provisional_trust_finding_into_a_refusal() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r"), record("1.0.1", "2026-02-01T00:00:00Z", "9", "")];
    // Typed binding: `registry: RegistryWithoutOwners`.
    let registry: RegistryWithoutOwners = RegistryWithoutOwners { inner: registry_of(history) };
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let versions = [crate::lockfile::LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }];

    // Borrow `&versions`: lends the value without copying or giving it away.
    let result = crate::checks::check_trust(&versions, &settings(), &registry, now());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err(), "a provisional finding was reported as a refusal: {result:?}");
}

/// Review finding (3c P2-4):
/// a local pass that carried a release-age warning is not remembered,
/// so the next call reports the
/// warning again instead of silently skipping the release.
#[test]
fn a_local_pass_that_carried_a_warning_is_not_remembered() {
    let (root, dirs) = repository(STALE);
    // Typed binding: `satisfied: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let satisfied: String = format!("# satisfied\n{}", lock_with("1.0.1"));
    // Borrow `&satisfied`: lends the value without copying or giving it away.
    // Typed binding: `runner: FakeCargo`.
    let runner: FakeCargo = cargo(&satisfied, false);
    // Typed binding: `registry: FakeRegistry`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let registry: FakeRegistry = registry_of(vec![record("1.0.0", OLD_ENOUGH, "", "o/r"), record("1.0.1", THREE_HOURS_AGO, "", "o/r")]);
    // `let mut`: a binding that may change.
    // Typed binding: `relaxed: Settings`.
    let mut relaxed: Settings = settings();
    relaxed.minimum_release_age_strict = false;
    let call = || {
        // Borrow `&InstallRequest`: lends the value without copying or giving it away.
        return install_rust_lockfiles(&InstallRequest {
            root: root.path(),
            crate_dirs: &dirs,
            frozen: false,
            offline: true,
            dry_run: false,
            now: now(),
            settings: &relaxed,
            runner: &runner,
            registry: &registry,
            // `None`: the present or the absent value of an `Option`.
            lock_base: None,
            trust_base: "HEAD",
            known_consistent: false,
        })
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        .expect("passes");
    };

    let first = call();
    let second = call();

    // Closure `|warning|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(first.warnings.iter().any(|warning| { return warning.contains("minimumReleaseAge"); }), "the first call must warn: {:?}", first.warnings);
    // Closure `|warning|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(second.warnings.iter().any(|warning| { return warning.contains("minimumReleaseAge"); }), "the pass with a warning was remembered: {:?}", second.warnings);
}
