//! What:
//! Decides from cargo's arguments what the shadow does before cargo runs.
//! Why:
//! Only a command that resolves dependencies can rewrite `Cargo.lock` as a side effect,
//! so only those get the install
//! step;
//! every other command goes straight to cargo,
//! and a help or version request never triggers anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verb.ts
//! export function treatmentOf(args: string[]): Treatment { ... }
//! ```

/// What:
/// `use std::ffi::OsString;` imports `OsString` from the standard library so this file can name it unqualified.
/// Why:
/// `OsString` is used by `words_before_double_dash`,
/// `find_verb`,
/// `treatment_of`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OsString } from 'std/ffi';
/// ```
use std::ffi::OsString;

/// What the shadow does before it runs cargo.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Treatment {
    /// Make the workspace's lockfile satisfy its manifests first.
    InstallStep,
    /// Run cargo,
    /// then warn about dependencies that do not use the workspace catalog.
    CatalogWarning,
    /// `update` and `generate-lockfile`:
    /// answered as the pnpm commands they stand for (`shadow_update.rs`).
    PnpmUpdate,
    /// Run cargo at once.
    PassThrough,
}

/// What:
/// `pub const INSTALL_STEP_VERBS: [&str; 32] = [...]` is the fixed list of cargo commands,
/// with their aliases,
/// that resolve
/// dependencies and so rewrite a stale `Cargo.lock` as a side effect.
/// A fixed-size array (sibling:
/// `Vec<&str>`) because the
/// list never grows at run time.
/// Why:
/// Measured on 2026-11-03 with cargo 1.101.0-nightly (3960fa69b 2026-10-12):
/// in a workspace whose lock lacked a
/// dependency its manifest declares,
/// each of `check`,
/// `build`,
/// `metadata`,
/// `tree`,
/// `fetch`,
/// `clippy`,
/// `doc`,
/// `vendor`,
/// `package`,
/// `rustc`,
/// `rustdoc`,
/// `test`,
/// `bench`,
/// `info`,
/// `nextest`,
/// `dylint` and `ndk` rewrote the lock.
/// `run` and `fix`
/// stopped on their own errors in that fixture (no binary target,
/// no lint to fix) before a write could be observed;
/// they
/// run the same build path,
/// so `run` (alias `r`) and `fix` are listed with them,
/// as are `publish` (it builds the package to verify it) and the
/// installed cargo plugins that compile the workspace:
/// `miri`,
/// `mutants`,
/// `fuzz`,
/// `ndk`,
/// `bloat`.
/// `fmt`,
/// `clean`,
/// `locate-project`,
/// `pkgid`,
/// `read-manifest`,
/// `verify-project`,
/// `new`,
/// `search`,
/// `config` and `metadata --no-deps` kept
/// the lock and are not listed.
/// Aliases (`b`,
/// `c`,
/// `d`,
/// `r`,
/// `t`) are read from `cargo --list`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const INSTALL_STEP_VERBS = ['build', 'b', 'check', 'c', ...] as const;
/// ```
pub const INSTALL_STEP_VERBS: [&str; 32] = [
    "build",
    "b",
    "check",
    "c",
    "clippy",
    "doc",
    "d",
    "run",
    "r",
    "test",
    "t",
    "bench",
    "fix",
    "fetch",
    "metadata",
    "tree",
    "vendor",
    "rustc",
    "rustdoc",
    "package",
    "publish",
    "info",
    "nextest",
    "dylint",
    "miri",
    "mutants",
    "fuzz",
    "ndk",
    "ndk-test",
    "bloat",
    "mutants-parallel",
    "mutants-gate-slot",
];

/// The commands that add or remove a dependency (`rm` is the alias of `remove` in `cargo --list`):
/// cargo runs first,
/// then
/// the shadow warns about a dependency that does not use the workspace catalog.
pub const CATALOG_WARNING_VERBS: [&str; 3] = ["add", "remove", "rm"];

/// What:
/// `pub const VALUE_OPTIONS: [&str; 5] = [...]` lists the options before the command that take the next argument as
/// their value.
/// Why:
/// The verb is the first word that is not an option,
/// so the value of `-C dir` must not be mistaken for it.
/// Read from
/// `cargo --help` of cargo 1.101.0-nightly:
/// `-C <DIRECTORY>`,
/// `--color <WHEN>`,
/// `--config <KEY=VALUE|PATH>` and `-Z <FLAG>`,
/// plus `--explain <CODE>`.
/// An option written `--name=value` or `-Zflag` is one word and needs no entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const VALUE_OPTIONS = ['-C', '--color', '--config', '-Z', '--explain'] as const;
/// ```
pub const VALUE_OPTIONS: [&str; 5] = ["-C", "--color", "--config", "-Z", "--explain"];

/// A word anywhere before `--` that makes cargo print help or its version and stop,
/// so nothing resolves.
pub const HELP_OR_VERSION_WORDS: [&str; 4] = ["--help", "-h", "--version", "-V"];

/// The command whose own help lives under a word:
/// `cargo help build` prints help and resolves nothing.
pub const HELP_VERB: &str = "help";

/// The option that makes `cargo metadata` skip dependency resolution.
pub const NO_DEPS_OPTION: &str = "--no-deps";

/// What:
/// `pub fn words_before_double_dash(arguments: &[OsString]) -> Vec<String>` turns the arguments into text and stops at a
/// literal `--`.
/// Why:
/// Everything after `--` belongs to the program cargo runs,
/// never to cargo's own grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function wordsBeforeDoubleDash(args: string[]): string[] { const end = args.indexOf('--'); return end === -1 ? args : args.slice(0, end); }
/// ```
pub fn words_before_double_dash(arguments: &[OsString]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for argument in arguments {
        // `to_string_lossy().to_string()` copies the argument as owned text, replacing invalid UTF-8 with a placeholder.
        let word: String = argument.to_string_lossy().to_string();
        if word == "--" {
            break;
        }
        words.push(word);
    }
    return words;
}

/// What:
/// `pub fn find_verb(arguments: &[OsString]) -> Option<String>` returns the command word:
/// the first word that is not the
/// leading `+toolchain` word,
/// an option or an option's value.
/// Why:
/// `cargo +nightly --color never check` has the verb after a toolchain word and options;
/// `None` means there is none.
/// rustup,
/// not cargo,
/// consumes the `+toolchain` word,
/// but only as the very first argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findVerb(args: string[]): string | undefined
/// ```
pub fn find_verb(arguments: &[OsString]) -> Option<String> {
    let words: Vec<String> = words_before_double_dash(arguments);
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    let position: usize = find_verb_position(&words)?;
    return Some(words[position].clone());
}

/// What:
/// `pub fn find_verb_position(words: &[String]) -> Option<usize>` returns the index of the command word in the words before `--`.
/// Why:
/// Review finding (2c 10):
/// matching the verb by its text finds an option value that spells the same word
/// (`cargo --config update update`);
/// the position skips each value option's value,
/// so the real command word is the one found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findVerbPosition(words: string[]): number | undefined
/// ```
pub fn find_verb_position(words: &[String]) -> Option<usize> {
    let mut index: usize = 0;
    if !words.is_empty() && words[0].starts_with('+') {
        index = 1;
    }
    while index < words.len() {
        let word: &String = &words[index];
        if word.starts_with('-') {
            if VALUE_OPTIONS.contains(&word.as_str()) {
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        return Some(index);
    }
    return None;
}

/// What:
/// `fn is_no_deps_option(word: &str) -> bool` tells whether one command-line word is `--no-deps`.
/// Why:
/// `cargo metadata --no-deps` reads no registry data,
/// so it passes through without the install step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNoDepsOption = (word: string) => word === '--no-deps';
/// ```
fn is_no_deps_option(word: &str) -> bool {
    return word == NO_DEPS_OPTION;
}

/// What:
/// `pub fn treatment_of(arguments: &[OsString]) -> Treatment` says what the shadow does before cargo runs.
/// Why:
/// A help or version word before `--` cancels everything,
/// because cargo then resolves nothing.
/// `metadata --no-deps` resolves
/// nothing either.
/// An unrecognised command line passes through:
/// a missed install step costs less than a stop nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function treatmentOf(args: string[]): Treatment
/// ```
pub fn treatment_of(arguments: &[OsString]) -> Treatment {
    let words: Vec<String> = words_before_double_dash(arguments);
    for word in &words {
        if HELP_OR_VERSION_WORDS.contains(&word.as_str()) {
            return Treatment::PassThrough;
        }
    }
    let verb: String = match find_verb(arguments) {
        Some(found) => found,
        None => return Treatment::PassThrough,
    };
    if verb == HELP_VERB {
        return Treatment::PassThrough;
    }
    if CATALOG_WARNING_VERBS.contains(&verb.as_str()) {
        return Treatment::CatalogWarning;
    }
    if verb == "update" || verb == "generate-lockfile" {
        return Treatment::PnpmUpdate;
    }
    if verb == "metadata" && words.iter().map(String::as_str).any(is_no_deps_option) {
        return Treatment::PassThrough;
    }
    if INSTALL_STEP_VERBS.contains(&verb.as_str()) {
        return Treatment::InstallStep;
    }
    return Treatment::PassThrough;
}
