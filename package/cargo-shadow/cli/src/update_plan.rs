//! What:
//! Reads a `cargo update` or `cargo generate-lockfile` command line and decides which pnpm command it stands for.
//! Why:
//! The owner's rule (2026-11-03, "follow pnpm semantics"): `cargo update` under the shadow behaves like `pnpm update`, which
//! is pnpm's own policy-aware command (https://pnpm.io/cli/update, read 2026-11-03), and `cargo generate-lockfile` like
//! `pnpm install --lockfile-only` (https://pnpm.io/cli/install). The mapping, flag by flag:
//!
//! - no names: `pnpm update`, every dependency within its range: the full catalog update of this workspace.
//! - `[SPEC]...` or `-p SPEC`: `pnpm update <name>`, the named dependencies within their ranges.
//! - `--recursive`: `pnpm update <name> --depth Infinity` (the dependencies of the named ones too); cargo's own resolver
//!   decides which move, then the policy checks the result. Without it, only the named ones.
//! - `--precise V`: `pnpm update <name>@<version>`, still subject to the policy; pnpm refuses it for a package that is not a
//!   direct dependency (`ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP`), and so does this.
//! - `--breaking`: `pnpm update --latest`, run by cargo and then checked; set aside to an older release when refused.
//! - `-n`, `--dry-run`: pnpm update has no dry run; the nearest is this tool's own: report, then restore every lockfile.
//! - `-w`, `--workspace`: `pnpm install` (relink the workspace's own packages): the install step.
//! - `--locked`, `--frozen`: `pnpm install --frozen-lockfile`: check and fail naming the lock, change nothing.
//! - `--offline`, `--ignore-rust-version`, `--config`, `-Z`, `--color`, `-q`, `-v`: cargo-only; forwarded to every cargo call.
//! - `-m`, `--manifest-path`: pnpm's `--dir`; it selects the workspace.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-plan.ts
//! export function planUpdate(args: string[]): UpdatePlan
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
// What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
// Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::ffi::OsString;

// Imports `crate::verb::{find_verb, words_before_doubl...`.
use crate::verb::{find_verb, words_before_double_dash};

/// What the command line stands for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdatePlan {
    /// `pnpm install --frozen-lockfile`: check only.
    FrozenCheck,
    /// `pnpm install` (and `--lockfile-only`): the install step.
    Install {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update`: the full catalog update.
    Full {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update <name>[@<version>]`: run the user's update, then the policy.
    Targeted {
        /// Report without keeping anything.
        dry_run: bool,
        /// The cargo arguments, starting with `update`, without the forwarded options.
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
const VALUE_OPTIONS: [&str; 8] = ["--precise", "-m", "--manifest-path", "--color", "--config", "-Z", "-p", "--package"];

/// Value options that are forwarded to every cargo call together with their value.
const FORWARDED_VALUE_OPTIONS: [&str; 3] = ["--color", "--config", "-Z"];

/// Flags that are forwarded to every cargo call.
const FORWARDED_FLAGS: [&str; 6] = ["--offline", "--ignore-rust-version", "-q", "--quiet", "-v", "--verbose"];

/// What:
/// `pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String>` maps a `cargo update` or
/// `cargo generate-lockfile` command line to the pnpm command it stands for.
/// Why:
/// `Err` carries the message of a refusal that mirrors pnpm's own (a `--precise` without one name, or for an indirect
/// dependency is checked later, where the lockfile is known).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planUpdate(args: string[]): Planned  // throws
/// ```
pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
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
    let words: Vec<String> = words_before_double_dash(arguments);
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `verb: String`.
    let verb: String = find_verb(arguments).unwrap_or_default();
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
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `start: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let start: usize = words.iter().position(|word| { return *word == verb; }).map(|index| { return index + 1; }).unwrap_or(words.len());
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `dry_run: bool`.
    let mut dry_run: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `frozen: bool`.
    let mut frozen: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `workspace: bool`.
    let mut workspace: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `precise: bool`.
    let mut precise: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `breaking: bool`.
    let mut breaking: bool = false;
    // `let mut`: a binding that may change.
    // Typed binding: `names: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut names: Vec<String> = Vec::new();
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `cargo_args: Vec<String>`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let mut cargo_args: Vec<String> = vec!["update".to_string()];
    // `let mut`: a binding that may change.
    // Typed binding: `forward: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut forward: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    let mut index: usize = start;
    while index < words.len() {
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // Typed binding: `word: &str`.
        let word: &str = &words[index];
        // Borrow `&word`: lends the value without copying or giving it away.
        // Typed binding: `takes_value: bool`.
        let takes_value: bool = VALUE_OPTIONS.contains(&word);
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `value: String`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        let value: String = if takes_value { words.get(index + 1).cloned().unwrap_or_default() } else { String::new() };
        // Closure `||`: an anonymous function, like a TS arrow function.
        if word == "-n" || word == "--dry-run" {
            dry_run = true;
        // Closure `||`: an anonymous function, like a TS arrow function.
        } else if word == "--locked" || word == "--frozen" {
            frozen = true;
        // Closure `||`: an anonymous function, like a TS arrow function.
        } else if word == "-w" || word == "--workspace" {
            workspace = true;
        } else if word == "--precise" {
            precise = true;
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.extend(["--precise".to_string(), value.clone()]);
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        } else if let Some(attached) = word.strip_prefix("--precise=") {
            precise = true;
            // Macro `format!`: builds or prints text (or a list) from a template.
            cargo_args.push(format!("--precise={attached}"));
        // Closure `||`: an anonymous function, like a TS arrow function.
        } else if word == "-m" || word == "--manifest-path" {
            // Dropped: the working directory of each cargo call is the selected workspace.
        } else if word.starts_with("--manifest-path=") {
            // Dropped, as above.
        } else if FORWARDED_VALUE_OPTIONS.contains(&word) {
            // `.clone()`: an independent owned copy.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            forward.extend([word.to_string(), value.clone()]);
        // Closure `||`: an anonymous function, like a TS arrow function.
        // Borrow `&word`: lends the value without copying or giving it away.
        } else if FORWARDED_FLAGS.contains(&word) || word.starts_with("--color=") || word.starts_with("--config=") || (word.starts_with("-Z") && word.len() > 2) {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            forward.push(word.to_string());
        // Closure `||`: an anonymous function, like a TS arrow function.
        } else if word == "-p" || word == "--package" {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            names.push(value.split('@').next().unwrap_or("").to_string());
            // `.clone()`: an independent owned copy.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.extend([word.to_string(), value.clone()]);
        } else if word.starts_with('-') {
            if word == "--breaking" {
                breaking = true;
            }
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.push(word.to_string());
        } else {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            names.push(word.split('@').next().unwrap_or("").to_string());
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            cargo_args.push(word.to_string());
        }
        index += if takes_value { 2 } else { 1 };
    }
    if precise && names.len() != 1 {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("--precise needs exactly one package to update (pnpm update <name>@<version> names one dependency)".to_string());
    }
    // Typed binding: `plan: UpdatePlan`.
    let plan: UpdatePlan = if frozen {
        UpdatePlan::FrozenCheck
    // Closure `||`: an anonymous function, like a TS arrow function.
    } else if verb == "generate-lockfile" || (workspace && names.is_empty()) {
        UpdatePlan::Install { dry_run }
    // Closure `||`: an anonymous function, like a TS arrow function.
    } else if !names.is_empty() || precise || breaking {
        UpdatePlan::Targeted { dry_run, cargo_args, names, precise }
    } else {
        UpdatePlan::Full { dry_run }
    };
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Planned { plan, forward });
}
