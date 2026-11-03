//! What:
//! The catalog update,
//! the counterpart of `pnpm update`:
//! moves lockfiles to newer releases under the release-age and trust
//! policy in `rust-catalog.toml`,
//! as one transaction.
//! Why:
//! pnpm's `update` is its own policy-aware command (<https://pnpm.io/cli/update>,
//! read 2026-11-03):
//! it moves dependencies
//! within their ranges and applies `minimumReleaseAge` and `trustPolicy` while it chooses.
//! Cargo's resolver applies neither,
//! so the shadow answers `cargo update` with this procedure instead of passing it to cargo's resolver.
//!
//! Two kinds.
//! The full kind re-resolves each crate root with
//! `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`,
//! which picks the newest releases
//! published before the cutoff (only `generate-lockfile` accepts `--publish-time`,
//! re-probed 2026-10-29),
//! then moves crates
//! the catalog exempts from the release age to their newest release.
//! The targeted kind runs the user's `cargo update` command
//! (named packages,
//! `--precise`,
//! `--breaking`) as given.
//! Both then collect the registry versions no lockfile held before,
//! set aside what the policy refuses,
//! check what is left,
//! and keep the new lockfiles only when nothing failed and it is not a
//! dry run;
//! otherwise every lockfile is restored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export async function updateRustLockfiles({ root, dryRun, now, runCargo, registry }): Promise<UpdateReport>
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fmt;
// Imports `std::fs`.
use std::fs;
// Imports `std::path::Path`.
use std::path::Path;

// Imports `monochromatic_dependency_catalog::Settings`.
use monochromatic_dependency_catalog::Settings;

// Imports `crate::cargo_run::{CargoOutcome, CargoRunne...`.
use crate::cargo_run::{CargoOutcome, CargoRunner};
// Imports `crate::checks::{CheckOutcome, check_release...`.
use crate::checks::{CheckOutcome, check_release_age, check_trust};
// Imports `crate::crates::{LockSnapshot, read_locks}`.
use crate::crates::{LockSnapshot, read_locks};
// Imports `crate::install::policy_findings`.
use crate::install::policy_findings;
// Imports `crate::journal::{begin, finish, process_exi...`.
use crate::journal::{begin, finish, process_exists, recover, restore_if_unchanged};
// Imports `crate::lockfile::{LockedVersion, added_vers...`.
use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};
// Imports `crate::prune::prune_excludes`.
use crate::prune::prune_excludes;
// Imports `crate::registry::Registry`.
use crate::registry::Registry;
// Imports `crate::selectors::{matches_pattern, parse_s...`.
use crate::selectors::{matches_pattern, parse_selector};
// Imports `crate::set_aside::{SetAsideRequest, set_asi...`.
use crate::set_aside::{SetAsideRequest, set_aside_until_clean};
// Imports `crate::tighten::{Syncer, raise_floors, rais...`.
use crate::tighten::{Syncer, raise_floors, raised_names};
// Imports `crate::time_text::publish_time_cutoff`.
use crate::time_text::publish_time_cutoff;

/// Which procedure the update runs.
pub enum UpdateKind<'a> {
    /// Re-resolve every listed crate root under the release-age cutoff (`pnpm update` with no names).
    Full,
    /// Run the user's `cargo update` arguments as given (`pnpm update <name>[@<version>]`).
    Targeted {
        /// The complete cargo arguments,
        /// starting with `update`.
        cargo_args: &'a [String],
    },
}

/// What an update needs.
pub struct UpdateRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to update,
    /// repo-relative.
    pub crate_dirs: &'a [String],
    /// Report without keeping anything.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The catalog file,
    /// edited to prune stale excludes after a kept full update.
    pub catalog_path: &'a Path,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Cargo options forwarded to every cargo call (`--offline`,
    /// `--config`,
    /// `-Z`,
    /// `--color`,
    /// `-q`,
    /// `-v`).
    pub forward: &'a [String],
    /// Renders the catalog into the manifests after `--breaking` raised floors.
    pub syncer: &'a dyn Syncer,
    /// Prune exclude selectors that no locked version needs.
    /// Only an update that covers every lockfile of the repository may
    /// say that nothing needs a selector;
    /// an update of some workspaces must leave the lists alone.
    pub prune_catalog: bool,
}

/// The outcome of an update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// The crate roots updated.
    pub crate_dirs: Vec<String>,
    /// Registry versions newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the update.
    pub problems: Vec<String>,
    /// Findings reported without refusing,
    /// including set-asides.
    pub warnings: Vec<String>,
    /// Roots resolved without the release-age cutoff (`minimumReleaseAgeStrict = false`).
    pub fell_back: Vec<String>,
    /// Whether the new lockfiles were kept.
    pub written: bool,
    /// Exclude selectors pruned from the catalog.
    pub pruned: Vec<String>,
}

/// Why an update did not complete;
/// every lockfile was restored first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// Cargo could not resolve a crate.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup,
    /// file operation or parse failed.
    Other {
        /// What failed.
        message: String,
    },
}

impl fmt::Display for UpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match self {
            UpdateError::Cargo { crate_dir, command, output } => {
                // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
                // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
                // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const value = await expr;   // throws to the caller on failure
                // ```
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      Text building and printing are written as one call instead of string concatenation.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile was restored.")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Fix the cause cargo names. When it is that no release old enough satisfies a requirement,")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "either wait, add the crate to minimumReleaseAgeExclude in rust-catalog.toml,")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "or set minimumReleaseAgeStrict = false to fall back to a younger release.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "When cargo says `-Z` needs the nightly channel, set RUSTUP_TOOLCHAIN to the nightly toolchain.lock.json pins.");
            }
            // Macro `write!`: builds or prints text (or a list) from a template.
            UpdateError::Other { message } => return write!(formatter, "{message}"),
        }
    }
}

/// What:
/// `fn run_or_fail(request, crate_dir, args) -> Result<CargoOutcome, UpdateError>` runs one cargo command in a crate root and turns a non-zero exit into an error naming the crate.
/// `Result<T, E>` is Rust's success-or-failure value (sibling:
/// `Option<T>`,
/// a value or nothing);
/// `?` on one returns early with the failure.
/// Why:
/// Every cargo step of an update fails the same way,
/// so the message is in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runOrFail(args): Promise<CargoOutcome>  // throws
/// ```
fn run_or_fail(request: &UpdateRequest, crate_dir: &str, args: &[String]) -> Result<CargoOutcome, UpdateError> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let outcome: CargoOutcome = request
        .runner
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        .run(&request.root.join(crate_dir), args)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        .map_err(|message| { return UpdateError::Other { message }; })?;
    if !outcome.ok {
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
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", args.join(" ")), output: outcome.output });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(outcome);
}

/// What:
/// `pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String>` lists `name@version` for
/// each locked registry crate that a name-only selector exempts from the release age.
/// Why:
/// Those crates move to their newest release with `cargo update`;
/// a selector naming exact versions only exempts those
/// versions from the checks and does not move the crate,
/// because the cutoff already chose its version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function excludedUpdateSpecs({ lock, selectors }): string[]
/// ```
pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `patterns: Vec<String>`.
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
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut patterns: Vec<String> = Vec::new();
    for selector in selectors {
        let parsed = parse_selector(selector);
        if parsed.versions.is_empty() {
            patterns.push(parsed.pattern);
        }
    }
    if patterns.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(Vec::new());
    }
    // `let mut`: a binding that may change.
    // Typed binding: `specs: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut specs: Vec<String> = Vec::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for locked in parse_registry_versions(lock)? {
        // Closure `|pattern|`: an anonymous function, like a TS arrow function.
        // Borrow `&locked.name`: lends the value without copying or giving it away.
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        if patterns.iter().any(|pattern| { return matches_pattern(pattern, &locked.name); }) {
            specs.push(locked.key());
        }
    }
    specs.sort();
    specs.dedup();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(specs);
}

/// What:
/// `fn resolve_crate(request, crate_dir, cutoff) -> Result<bool, UpdateError>` re-resolves one crate root under the cutoff and returns whether it had to fall back to ignoring the cutoff.
/// Why:
/// With `minimumReleaseAgeStrict` false a crate that cannot resolve under the cutoff resolves without it and is reported;
/// strict fails instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCrate(args): Promise<boolean>  // throws
/// ```
fn resolve_crate(request: &UpdateRequest, crate_dir: &str, cutoff: &str) -> Result<bool, UpdateError> {
    // `let mut`: a binding that may change.
    // Typed binding: `constrained: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut constrained: Vec<String> = Vec::new();
    if cutoff.is_empty() {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        constrained.push("generate-lockfile".to_string());
    } else {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        constrained.extend(["-Zunstable-options".to_string(), "generate-lockfile".to_string(), "--publish-time".to_string(), cutoff.to_string()]);
    }
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    constrained.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Borrow `&request.root.join`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `first: CargoOutcome`.
    let first: CargoOutcome = request.runner.run(&request.root.join(crate_dir), &constrained).map_err(|message| { return UpdateError::Other { message }; })?;
    if first.ok {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(false);
    }
    // Closure `||`: an anonymous function, like a TS arrow function.
    if request.settings.minimum_release_age_strict || cutoff.is_empty() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", constrained.join(" ")), output: first.output });
    }
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: {crate_dir}: no resolution satisfies the release-age cutoff {cutoff}; minimumReleaseAgeStrict is false, so resolving without it");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `fallback: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut fallback: Vec<String> = vec!["generate-lockfile".to_string()];
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    fallback.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&fallback`: lends the value without copying or giving it away.
    run_or_fail(request, crate_dir, &fallback)?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(true);
}

/// What:
/// `fn resolve_all(request, cutoff) -> Result<Vec<String>, UpdateError>` resolves every listed crate root and then moves the exempt crates to their newest release.
/// Why:
/// The exempt update reads the lockfile that the same root's resolution just wrote,
/// so the two steps stay together per root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveAll(args): Promise<string[]>
/// ```
fn resolve_all(request: &UpdateRequest, cutoff: &str) -> Result<Vec<String>, UpdateError> {
    // `let mut`: a binding that may change.
    // Typed binding: `fell_back: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut fell_back: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        eprintln!("cargo-shadow: resolving {crate_dir}{}", if cutoff.is_empty() { String::new() } else { format!(" with releases published before {cutoff}") });
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        if resolve_crate(request, crate_dir, cutoff)? {
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            fell_back.push(crate_dir.clone());
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `snapshots: Vec<LockSnapshot>`.
        let snapshots: Vec<LockSnapshot> = read_locks(request.root, std::slice::from_ref(crate_dir)).map_err(|error| { return UpdateError::Other { message: error.to_string() }; })?;
        // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // Typed binding: `source: &str`.
        let source: &str = snapshots.first().map(|snapshot| { return snapshot.source.as_str(); }).unwrap_or("");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|message|`: an anonymous function, like a TS arrow function.
        // Borrow `&request.settings.minimum_release_age_exclu...`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `specs: Vec<String>`.
        let specs: Vec<String> = excluded_update_specs(source, &request.settings.minimum_release_age_exclude).map_err(|message| { return UpdateError::Other { message }; })?;
        if !specs.is_empty() {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `let mut`: a binding that may change.
            // Typed binding: `args: Vec<String>`.
            // Macro `vec!`: builds or prints text (or a list) from a template.
            let mut args: Vec<String> = vec!["update".to_string()];
            args.extend(specs);
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            args.extend(request.forward.iter().cloned());
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&args`: lends the value without copying or giving it away.
            run_or_fail(request, crate_dir, &args)?;
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(fell_back);
}

/// What:
/// `fn sources_of(snapshots) -> Vec<String>` copies the lockfile texts out of snapshots;
/// `.clone()` makes an independent copy of owned text.
/// Why:
/// The lockfile diff takes plain texts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourcesOf = (snapshots) => snapshots.map((s) => s.source)
/// ```
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
    // `.clone()`: an independent owned copy.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    return snapshots.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
}

/// What:
/// `fn other(message) -> UpdateError` wraps a message in the catch-all update error.
/// Why:
/// `map_err(other)` turns a plain text failure into the error type of the function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = (message: string) => new UpdateError(message)
/// ```
fn other(message: String) -> UpdateError {
    return UpdateError::Other { message };
}

/// What:
/// `fn run_update(request, kind, before, produced) -> Result<UpdateReport, UpdateError>` is the transaction body:
/// resolve or run the user's update,
/// set refused versions aside,
/// check what is left,
/// prune and raise catalog entries when kept.
/// `&mut Option<...>` is a borrowed slot the function may fill.
/// Why:
/// The caller restores on an error or a refusal;
/// `produced` records what cargo wrote before the slow registry checks,
/// so the restore can tell a concurrent writer from this run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runUpdate(args): Promise<UpdateReport>
/// ```
fn run_update(request: &UpdateRequest, kind: &UpdateKind, before: &[LockSnapshot], produced: &mut Option<Vec<LockSnapshot>>) -> Result<UpdateReport, UpdateError> {
    // Borrow `&Settings`: lends the value without copying or giving it away.
    // Typed binding: `settings: &Settings`.
    let settings: &Settings = request.settings;
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Typed binding: `cutoff: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let cutoff: String = if settings.minimum_release_age == 0 { String::new() } else { publish_time_cutoff(request.now, settings.minimum_release_age) };
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Typed binding: `cutoff_ms: i64`.
    // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
    // Why:      Time arithmetic and sizes come in different integer widths and must be made equal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Number(value)  // JS has one number type, so no cast is needed
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let cutoff_ms: i64 = if settings.minimum_release_age == 0 { 0 } else { request.now - (settings.minimum_release_age as i64) * crate::time_text::MS_PER_MINUTE };
    // `let mut`: a binding that may change.
    // Typed binding: `fell_back: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut fell_back: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `set_aside_allowed: bool`.
    let mut set_aside_allowed: bool = true;
    // `match`: picks the arm whose pattern fits the value.
    match kind {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&cutoff`: lends the value without copying or giving it away.
        UpdateKind::Full => fell_back = resolve_all(request, &cutoff)?,
        UpdateKind::Targeted { cargo_args } => {
            // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
            // Why:      The result can be stored or modified independently of the slice it came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // [...slice]
            // ```
            // `let mut`: a binding that may change.
            // Typed binding: `args: Vec<String>`.
            let mut args: Vec<String> = cargo_args.to_vec();
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            args.extend(request.forward.iter().cloned());
            // An explicit `--precise` names the version the user wants: the policy decides whether it stands, and nothing is
            // set aside in its place.
            set_aside_allowed = !cargo_args.iter().any(|argument| { return argument == "--precise" || argument.starts_with("--precise="); });
            for crate_dir in request.crate_dirs {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&args`: lends the value without copying or giving it away.
                run_or_fail(request, crate_dir, &args)?;
            }
        }
    }
    // Typed binding: `before_sources: Vec<String>`.
    let before_sources: Vec<String> = sources_of(before);
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut warnings: Vec<String> = Vec::new();
    if set_aside_allowed {
        // Borrow `&SetAsideRequest`: lends the value without copying or giving it away.
        warnings = set_aside_until_clean(&SetAsideRequest {
            root: request.root,
            crate_dirs: request.crate_dirs,
            // Borrow `&before_sources`: lends the value without copying or giving it away.
            before: &before_sources,
            registry: request.registry,
            settings,
            now: request.now,
            cutoff_ms,
            runner: request.runner,
            forward: request.forward,
        })
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        .map_err(other)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `after_snapshots: Vec<LockSnapshot>`.
    let after_snapshots: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return other(error.to_string()); })?;
    // What cargo produced, taken before the slow registry checks: a restore later replaces a lockfile only if it still holds this.
    *produced = Some(after_snapshots.clone());
    // Borrow `&after_snapshots`: lends the value without copying or giving it away.
    // Typed binding: `after: Vec<String>`.
    let after: Vec<String> = sources_of(&after_snapshots);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&before_sources`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after).map_err(other)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&added`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `age: CheckOutcome`.
    let age: CheckOutcome = check_release_age(&added, settings, request.registry, request.now).map_err(|error| { return other(error.message); })?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&added`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `trust: Vec<String>`.
    let trust: Vec<String> = check_trust(&added, settings, request.registry, request.now).map_err(|error| { return other(error.message); })?;
    // `let mut`: a binding that may change.
    // Typed binding: `problems: Vec<String>`.
    let mut problems: Vec<String> = age.problems;
    problems.extend(trust);
    warnings.extend(age.warnings);
    // Typed binding: `keep: bool`.
    let keep: bool = problems.is_empty() && !request.dry_run;
    // `let mut`: a binding that may change.
    // Typed binding: `pruned: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut pruned: Vec<String> = Vec::new();
    // Macro `matches!`: builds or prints text (or a list) from a template.
    if keep && request.prune_catalog && matches!(kind, UpdateKind::Full) {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(|error| { return other(error.to_string()); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `locked: Vec<LockedVersion>`.
        let locked: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let result = prune_excludes(&source, settings, &locked).map_err(other)?;
        if !result.removed.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&result.source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(request.catalog_path, &result.source).map_err(|error| { return other(error.to_string()); })?;
        }
        pruned = result.removed;
    }
    // `--breaking` is `pnpm update --latest`: the new range goes into the catalog entry and the sync renders it (tighten.rs).
    if keep && let UpdateKind::Targeted { cargo_args } = kind && cargo_args.iter().any(|argument| { return argument == "--breaking"; }) {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&before_sources`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `before_versions: Vec<LockedVersion>`.
        let before_versions: Vec<LockedVersion> = registry_versions(&before_sources).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `after_versions: Vec<LockedVersion>`.
        let after_versions: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(|error| { return other(error.to_string()); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (raised_source, changes) = raise_floors(&source, &after_versions, &raised_names(&before_versions, &after_versions)).map_err(other)?;
        if !changes.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&raised_source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(request.catalog_path, &raised_source).map_err(|error| { return other(error.to_string()); })?;
            // Closure `|change|`: an anonymous function, like a TS arrow function.
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            // Adapter `.map(`: transforms or selects items with a closure.
            // Macro `format!`: builds or prints text (or a list) from a template.
            warnings.extend(changes.iter().map(|change| { return format!("Raised the catalog floor {change}"); }));
            // `match`: picks the arm whose pattern fits the value.
            match request.syncer.sync(request.root) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(done) => warnings.push(done),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(message) => warnings.push(message),
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.to_vec()`: copies the slice into a new owned `Vec`.
    return Ok(UpdateReport { crate_dirs: request.crate_dirs.to_vec(), added, problems, warnings, fell_back, written: keep, pruned });
}

/// What:
/// `pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError>` runs the
/// update over the listed crate roots.
/// Why:
/// It first finishes any run that was killed (`journal.rs`),
/// snapshots every lockfile,
/// and puts the snapshots back on a failure,
/// a refusal or a dry run,
/// but only into lockfiles that still hold what this run produced,
/// so a refused version never stays in
/// some crates and not others and a change written by another process is never discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function updateRustLockfiles(args): Promise<UpdateReport>
/// ```
pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError> {
    // Closure `|old, live|`: an anonymous function, like a TS arrow function.
    // Borrow `&process_exists`: lends the value without copying or giving it away.
    // Typed binding: `recovered: Vec<String>`.
    let recovered: Vec<String> = recover(request.root, &process_exists, &|old, live| {
        return policy_findings(request.settings, request.registry, request.now, old, live);
    });
    // Borrow `&recovered`: lends the value without copying or giving it away.
    for message in &recovered {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {message}");
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return other(error.to_string()); })?;
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `journaled: bool`.
    let journaled: bool = begin(request.root, &before).is_ok();
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `produced: Option<Vec<LockSnapshot>>`.
    let mut produced: Option<Vec<LockSnapshot>> = None;
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<UpdateReport, UpdateError>`.
    let outcome: Result<UpdateReport, UpdateError> = run_update(request, kind, &before, &mut produced);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&outcome`: lends the value without copying or giving it away.
    // Typed binding: `keep: bool`.
    // Macro `matches!`: builds or prints text (or a list) from a template.
    let keep: bool = matches!(&outcome, Ok(report) if report.written);
    // `let mut`: a binding that may change.
    // Typed binding: `restore_warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut restore_warnings: Vec<String> = Vec::new();
    if !keep {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: restoring every lockfile that this run changed");
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `state: Vec<LockSnapshot>`.
        let state: Vec<LockSnapshot> = match produced {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `None`: the present or the absent value of an `Option`.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            None => read_locks(request.root, request.crate_dirs).unwrap_or_default(),
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&before`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        restore_warnings = restore_if_unchanged(request.root, &before, &state).map_err(|error| { return other(error.to_string()); })?;
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    if journaled {
        finish(request.root);
    }
    // Closure `|mut report|`: an anonymous function, like a TS arrow function.
    // Adapter `.map(`: transforms or selects items with a closure.
    return outcome.map(|mut report| {
        report.warnings.extend(recovered);
        report.warnings.extend(restore_warnings);
        return report;
    });
}
