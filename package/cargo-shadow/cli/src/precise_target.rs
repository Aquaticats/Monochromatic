//! What:
//! The packages and the release a `cargo update --precise` names,
//! read from the arguments,
//! and the locked entries that match them.
//! Why:
//! `cargo update -p foo --precise 1.0.1` leaves the lockfile unchanged when `foo@1.0.1` is already locked,
//! so a check of only what the command added never sees the release the user asked for.
//! The named release is held to the policy whether or not the command changed the lockfile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // precise-target.ts
//! export function preciseTargets(args: { cargoArgs: string[]; locked: LockedVersion[] }): LockedVersion[]
//! ```

/// What:
/// `use crate::lockfile::LockedVersion;` imports the name and version pair of a locked registry entry.
/// Why:
/// The matching entries are returned in the same shape the policy checks take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;

/// What:
/// `fn package_name(spec: &str) -> &str` takes the crate name out of a cargo package spec
/// (`name`,
/// `name@1.2.3`,
/// `https://host/path#name@1.2.3`).
/// Why:
/// Only the name selects which locked entries the `--precise` release applies to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packageName = (spec: string): string => (spec.split('#').pop() ?? spec).split('@')[0];
/// ```
fn package_name(spec: &str) -> &str {
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
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
    let after_url: &str = spec.rsplit('#').next().unwrap_or(spec);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return after_url.split('@').next().unwrap_or(after_url);
}

/// What:
/// `fn option_values<'a>(arguments: &'a [String], long: &str, short: Option<&str>) -> Vec<&'a str>` collects every value given to one
/// option in the forms `--long value`,
/// `--long=value`,
/// `-s value`,
/// `-s=value` and `-svalue`.
/// Why:
/// `-p` may be given more than once and cargo accepts all of these spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionValues(args: string[], long: string, short?: string): string[]
/// ```
fn option_values<'a>(arguments: &'a [String], long: &str, short: Option<&str>) -> Vec<&'a str> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `values: Vec<&'a str>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut values: Vec<&'a str> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut index: usize = 0;
    while index < arguments.len() {
        // Typed binding: `argument: &str`.
        let argument: &str = arguments[index].as_str();
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        if argument == long || Some(argument) == short {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
            // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (value !== undefined) { const inner = value; ... }
            // ```
            if let Some(next) = arguments.get(index + 1) {
                values.push(next.as_str());
            }
            index += 2;
            continue;
        }
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
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
        if let Some(attached) = argument.strip_prefix(&format!("{long}=")) {
            values.push(attached);
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        } else if let Some(letter) = short {
            // `Some`: the present or the absent value of an `Option`.
            // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
            if let Some(attached) = argument.strip_prefix(letter) {
                // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
                values.push(attached.strip_prefix('=').unwrap_or(attached));
            }
        }
        index += 1;
    }
    return values;
}

/// What:
/// `pub fn preciseTargets(cargo_args, locked) -> Vec<LockedVersion>` returns the locked entries that a
/// `cargo update --package NAME --precise VERSION` names.
/// Why:
/// An empty result means the arguments name no precise release (or no locked entry has it),
/// so nothing extra is checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function preciseTargets(cargoArgs: string[], locked: LockedVersion[]): LockedVersion[]
/// ```
pub fn precise_targets(cargo_args: &[String], locked: &[LockedVersion]) -> Vec<LockedVersion> {
    // `None`: the present or the absent value of an `Option`.
    // Typed binding: `precise: Vec<&str>`.
    let precise: Vec<&str> = option_values(cargo_args, "--precise", None);
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    let Some(version) = precise.first() else {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Vec::new();
    };
    // `Some`: the present or the absent value of an `Option`.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `names: Vec<&str>`.
    let names: Vec<&str> = option_values(cargo_args, "--package", Some("-p")).into_iter().map(package_name).collect();
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // Borrow `&entry.name.as_str`: lends the value without copying or giving it away.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.filter(`: transforms or selects items with a closure.
    return locked.iter().filter(|entry| { return entry.version == *version && names.contains(&entry.name.as_str()); }).cloned().collect();
}
