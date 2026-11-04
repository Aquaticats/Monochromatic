//! What:
//! Reads a `cargo update` or `cargo generate-lockfile` command line and decides which pnpm command it stands for.
//! Why:
//! The owner's rule (2026-11-03,
//! "follow pnpm semantics"):
//! `cargo update` under the shadow behaves like `pnpm update`,
//! which
//! is pnpm's own policy-aware command (<https://pnpm.io/cli/update>,
//! read 2026-11-03),
//! and `cargo generate-lockfile` like
//! `pnpm install --lockfile-only` (<https://pnpm.io/cli/install>).
//! The mapping,
//! flag by flag:
//!
//! - no names:
//!   `pnpm update`,
//!   every dependency within its range:
//!   the full catalog update of this workspace.
//! - `[SPEC]...` or `-p SPEC`:
//!   `pnpm update <name>`,
//!   the named dependencies within their ranges.
//! - `--recursive`:
//!   `pnpm update <name> --depth Infinity` (the dependencies of the named ones too);
//!   cargo's own resolver
//!   decides which move,
//!   then the policy checks the result.
//!   Without it,
//!   only the named ones.
//! - `--precise V`:
//!   `pnpm update <name>@<version>`,
//!   still subject to the policy;
//!   pnpm refuses it for a package that is not a
//!   direct dependency (`ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP`),
//!   and so does this.
//! - `--breaking`:
//!   `pnpm update --latest`,
//!   run by cargo and then checked;
//!   set aside to an older release when refused.
//! - `-n`,
//!   `--dry-run`:
//!   pnpm update has no dry run;
//!   the nearest is this tool's own:
//!   report,
//!   then restore every lockfile.
//! - `-w`,
//!   `--workspace`:
//!   `pnpm install` (relink the workspace's own packages):
//!   the install step.
//! - `--locked`,
//!   `--frozen`:
//!   `pnpm install --frozen-lockfile`:
//!   check and fail naming the lock,
//!   change nothing.
//! - `--offline`,
//!   `--ignore-rust-version`,
//!   `--config`,
//!   `-Z`,
//!   `--color`,
//!   `-q`,
//!   `-v`:
//!   cargo-only;
//!   forwarded to every cargo call.
//! - `-m`,
//!   `--manifest-path`:
//!   pnpm's `--dir`;
//!   it selects the workspace.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-plan.ts
//! export function planUpdate(args: string[]): UpdatePlan
//! ```

// What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
// Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::ffi::OsString;` imports `OsString` from the standard library so this file can name it unqualified.
/// Why:
/// `OsString` is used by `plan_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OsString } from 'std/ffi';
/// ```
use std::ffi::OsString;

/// What:
/// `use crate::verb::{find_verb, words_before_double_dash};` imports `find_verb`,
/// `words_before_double_dash` from another module of this crate so this file can name them unqualified.
/// Why:
/// `find_verb` is used by `plan_update`;
/// `words_before_double_dash` is used by `plan_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_verb, words_before_double_dash } from 'crate/verb';
/// ```
use crate::verb::{find_verb, words_before_double_dash, find_verb_position};

/// What the command line stands for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdatePlan {
    /// `pnpm install --frozen-lockfile`:
    /// check only.
    FrozenCheck,
    /// `pnpm install` (and `--lockfile-only`):
    /// the install step.
    Install {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update`:
    /// the full catalog update.
    Full {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update <name>[@<version>]`:
    /// run the user's update,
    /// then the policy.
    Targeted {
        /// Report without keeping anything.
        dry_run: bool,
        /// The cargo arguments,
        /// starting with `update`,
        /// without the forwarded options.
        cargo_args: Vec<String>,
        /// The package names named (without any `@version`).
        names: Vec<String>,
        /// Whether `--precise` was given.
        precise: bool,
    },
}

/// A plan with the cargo options to forward to every cargo call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Planned {
    /// What to do.
    pub plan: UpdatePlan,
    /// The options to forward.
    pub forward: Vec<String>,
}

/// Options of `cargo update` that take the next word as their value.
const VALUE_OPTIONS: [&str; 9] = ["--precise", "-m", "--manifest-path", "--color", "--config", "-Z", "-p", "--package", "-C"];

/// Value options that are forwarded to every cargo call together with their value.
const FORWARDED_VALUE_OPTIONS: [&str; 3] = ["--color", "--config", "-Z"];

/// Flags that are forwarded to every cargo call.
const FORWARDED_FLAGS: [&str; 6] = ["--offline", "--ignore-rust-version", "-q", "--quiet", "-v", "--verbose"];

/// What:
/// `pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String>` maps a `cargo update` or
/// `cargo generate-lockfile` command line to the pnpm command it stands for.
/// Why:
/// `Err` carries the message of a refusal that mirrors pnpm's own (a `--precise` without one name,
/// or for an indirect
/// dependency is checked later,
/// where the lockfile is known),
/// and the refusal of an option that has no mapping (it is never dropped or passed on silently).
/// Options before the command word are read like those after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planUpdate(args: string[]): Planned  // throws
/// ```
pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String> {
    let words: Vec<String> = words_before_double_dash(arguments);
    let verb: String = find_verb(arguments).unwrap_or_default();
    // The position of the command word itself: everything else is scanned, including the global options before it
    // (review finding: `cargo --locked update` and `cargo --offline update` must read like their later spellings).
    // The position of the real command word; an option value that spells the same word is not it.
    let verb_position: Option<usize> = find_verb_position(&words);
    let mut dry_run: bool = false;
    let mut frozen: bool = false;
    let mut workspace: bool = false;
    let mut precise: bool = false;
    let mut breaking: bool = false;
    let mut names: Vec<String> = Vec::new();
    let mut cargo_args: Vec<String> = vec!["update".to_string()];
    let mut forward: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    let mut index: usize = 0;
    while index < words.len() {
        let word: &str = &words[index];
        // The toolchain word and the command word are not options.
        if (index == 0 && word.starts_with('+')) || Some(index) == verb_position {
            index += 1;
            continue;
        }
        let takes_value: bool = VALUE_OPTIONS.contains(&word);
        // Review finding (2c 9): an option that needs a value and has none is cargo's argument error; it is refused here and never
        // dropped, which would turn the command into a full update.
        if takes_value && index + 1 >= words.len() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("`{word}` needs a value and none was given, so the command was not run"));
        }
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // Typed binding: `value: String`.
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
        let value: String = if takes_value { words[index + 1].clone() } else { String::new() };
        // Review finding (3c 9): an attached value that is empty (`--manifest-path=`, `-m=`, `--precise=`) is a missing value too.
        if matches!(word, "--manifest-path=" | "-m=" | "--precise=") {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("`{word}` needs a value and none was given, so the command was not run"));
        }
        if word == "-n" || word == "--dry-run" {
            dry_run = true;
        } else if word == "--locked" || word == "--frozen" {
            frozen = true;
        } else if word == "-w" || word == "--workspace" {
            workspace = true;
        } else if word == "--precise" {
            precise = true;
            cargo_args.extend(["--precise".to_string(), value.clone()]);
        } else if let Some(attached) = word.strip_prefix("--precise=") {
            precise = true;
            cargo_args.push(format!("--precise={attached}"));
        } else if word == "-m" || word == "--manifest-path" || word.starts_with("--manifest-path=") || word == "-C" || (word.starts_with("-m") && word.len() > 2) {
            // The workspace these select is found before the plan runs (`options.rs`, `shadow_step.rs`), and each cargo call
            // runs inside it, so the option itself is dropped.
        } else if FORWARDED_VALUE_OPTIONS.contains(&word) {
            forward.extend([word.to_string(), value.clone()]);
        // Borrow `&word`: lends the value without copying or giving it away.
        } else if FORWARDED_FLAGS.contains(&word) || word == "-vv" || word.starts_with("--color=") || word.starts_with("--config=") || (word.starts_with("-Z") && word.len() > 2) {
            forward.push(word.to_string());
        } else if word == "-p" || word == "--package" {
            // Borrow `&value`: lends the value without copying or giving it away.
            names.push(package_name(&value));
            cargo_args.extend([word.to_string(), value.clone()]);
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        } else if let Some(attached) = word.strip_prefix("--package=") {
            // Review finding: the attached spelling names the package like the separated one.
            names.push(package_name(attached));
            cargo_args.push(word.to_string());
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        } else if let Some(attached) = word.strip_prefix("-p")
            && !attached.is_empty()
            && !attached.starts_with('-')
        {
            names.push(package_name(attached));
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            cargo_args.push(word.to_string());
        } else if word == "--breaking" {
            breaking = true;
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.push(word.to_string());
        } else if word == "--recursive" {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.push(word.to_string());
        } else if word.starts_with('-') {
            // Review finding (reject, do not broaden): an option the shadow has no mapping for would otherwise be dropped
            // silently from a full update, or passed on to a targeted one, and change what the person asked for.
            return Err(format!(
                "`{word}` is not an option of `cargo update` that the shadow can answer as `pnpm update`, so it was not run; the options it maps are listed in the README of cargo-shadow (section \"Mapping `cargo update` to `pnpm update`\"). To run cargo's own command, start the real cargo directly"
            ));
        } else {
            names.push(package_name(word));
            cargo_args.push(word.to_string());
        }
        index += if takes_value { 2 } else { 1 };
    }
    if precise && names.len() != 1 {
        return Err("--precise needs exactly one package to update (pnpm update <name>@<version> names one dependency)".to_string());
    }
    let plan: UpdatePlan = if frozen {
        UpdatePlan::FrozenCheck
    } else if verb == "generate-lockfile" || (workspace && names.is_empty()) {
        UpdatePlan::Install { dry_run }
    } else if !names.is_empty() || precise || breaking {
        UpdatePlan::Targeted { dry_run, cargo_args, names, precise }
    } else {
        UpdatePlan::Full { dry_run }
    };
    return Ok(Planned { plan, forward });
}

/// What:
/// `fn package_name(spec) -> String` is the crate name of a package spec (`name` or `name@version`).
/// Why:
/// The plan counts and checks names;
/// the version part belongs to cargo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packageName = (spec: string) => spec.split('@')[0];
/// ```
fn package_name(spec: &str) -> String {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    return spec.split('@').next().unwrap_or("").to_string();
}
