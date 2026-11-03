//! What:
//! The fetch refspecs of a remote (`+refs/heads/*:refs/remotes/origin/*`) and the mapping of a branch's `merge` reference through
//! them to the remote-tracking reference,
//! as `@{upstream}` does.
//! Why:
//! Git rejects the whole remote when any of its refspecs is invalid,
//! so every refspec is checked before one is used;
//! anything this reader does not know to be valid and plain (negative refspecs,
//! several wildcards,
//! unusual characters)
//! is an error and the caller asks Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refspec.ts
//! export function mapThroughRefspecs(specs: string[], merge: string): string | undefined  // throws on a spec it cannot judge
//! export function isPlainRefName(name: string): boolean
//! ```

/// What:
/// `fn is_plain_character(character: char) -> bool` says whether a character may appear in a plain reference name or pattern.
/// Why:
/// Only letters,
/// digits and `_ - . + / *` are judged by this reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainCharacter = (character: string): boolean => /[A-Za-z0-9_.+\/*-]/.test(character);
/// ```
fn is_plain_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || "_-.+/*".contains(character);
}

/// What:
/// `fn is_plain_component(part: &str) -> bool` says whether a path component of a reference name is non-empty,
/// not dot-leading
/// and not ending in `.lock`.
/// Why:
/// Git refuses such components in a reference name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainComponent = (part: string): boolean => part !== '' && !part.startsWith('.') && !part.endsWith('.lock');
/// ```
fn is_plain_component(part: &str) -> bool {
    return !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock");
}

/// What:
/// `pub fn is_plain_ref_name(name: &str) -> bool` says whether a reference name or pattern consists only of characters and
/// shapes that are certainly valid in Git (letters,
/// digits,
/// `_ - . + /`,
/// an optional `*`),
/// with no empty,
/// dot-leading or
/// `.lock` component and no `..`.
/// Why:
/// The reader judges only names it is sure about;
/// anything else (a space,
/// a bracket,
/// Unicode) is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isPlainRefName(name: string): boolean
/// ```
pub fn is_plain_ref_name(name: &str) -> bool {
    if name.is_empty() || name.contains("..") || name.ends_with('.') || name.ends_with('/') || name.starts_with('/') {
        return false;
    }
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    if !name.chars().all(is_plain_character) {
        return false;
    }
    // Closure `|part|`: an anonymous function, like a TS arrow function.
    return name.split('/').all(is_plain_component);
}

/// One side of a refspec that is checked and split at its single wildcard,
/// if any.
struct Side<'a> {
    /// The text before the wildcard (the whole text when there is none).
    prefix: &'a str,
    /// The text after the wildcard,
    /// or `None` when the side has no wildcard.
    suffix: Option<&'a str>,
}

/// What:
/// `fn side(text: &str) -> Result<Side, String>` checks one side of a refspec and splits it at the wildcard.
/// Why:
/// More than one wildcard,
/// or a name that is not plainly valid,
/// is what Git rejects or this reader cannot judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function side(text: string): { prefix: string; suffix?: string }  // throws
/// ```
fn side(text: &str) -> Result<Side<'_>, String> {
    if !is_plain_ref_name(text) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
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
        return Err(format!("the fetch refspec side {text:?} is not a plain reference name or pattern"));
    }
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match text.split_once('*') {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        None => Ok(Side { prefix: text, suffix: None }),
        // `Some`: the present or the absent value of an `Option`.
        Some((prefix, suffix)) => {
            if suffix.contains('*') {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                return Err(format!("the fetch refspec side {text:?} has more than one wildcard"));
            }
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            Ok(Side { prefix, suffix: Some(suffix) })
        }
    };
}

/// A fetch refspec that was checked.
struct Refspec<'a> {
    /// The remote side.
    source: Side<'a>,
    /// The local side,
    /// or `None` for a refspec without one (it maps nothing).
    destination: Option<Side<'a>>,
}

/// What:
/// `fn parse_refspec(spec: &str) -> Result<Refspec, String>` checks one fetch refspec.
/// Why:
/// Both sides must be present and plain,
/// both with a wildcard or both without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRefspec(spec: string): Refspec  // throws on a spec it cannot judge
/// ```
fn parse_refspec(spec: &str) -> Result<Refspec<'_>, String> {
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
    let without_force: &str = spec.strip_prefix('+').unwrap_or(spec);
    if without_force.starts_with('^') {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        return Err("a negative fetch refspec is applied by Git".to_string());
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    let Some((source_text, destination_text)) = without_force.split_once(':') else {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        return Ok(Refspec { source: side(without_force)?, destination: None });
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `source: Side`.
    let source: Side = side(source_text)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `destination: Side`.
    let destination: Side = side(destination_text)?;
    if source.suffix.is_some() != destination.suffix.is_some() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("the fetch refspec {spec:?} has a pattern on one side only"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    return Ok(Refspec { source, destination: Some(destination) });
}

/// What:
/// `pub fn map_through_refspecs(specs: &[&str], merge: &str) -> Result<Option<String>, String>` returns the local reference that
/// the first matching fetch refspec stores `merge` in.
/// Why:
/// Every refspec is checked first (Git rejects the remote otherwise),
/// then the first match in order wins,
/// as in Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function mapThroughRefspecs(specs: string[], merge: string): string | undefined  // throws on a spec it cannot judge
/// ```
pub fn map_through_refspecs(specs: &[&str], merge: &str) -> Result<Option<String>, String> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `parsed: Vec<Refspec>`.
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
    let mut parsed: Vec<Refspec> = Vec::new();
    for spec in specs {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        parsed.push(parse_refspec(spec)?);
    }
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    for refspec in &parsed {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Borrow `&refspec.destination`: lends the value without copying or giving it away.
        let Some(destination) = &refspec.destination else {
            continue;
        };
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&refspec.source.suffix`: lends the value without copying or giving it away.
        match (&refspec.source.suffix, &destination.suffix) {
            // `Some`: the present or the absent value of an `Option`.
            (Some(source_suffix), Some(destination_suffix)) => {
                // `Some`: the present or the absent value of an `Option`.
                // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
                if let Some(rest) = merge.strip_prefix(refspec.source.prefix)
                    // `Some`: the present or the absent value of an `Option`.
                    && let Some(middle) = rest.strip_suffix(source_suffix)
                {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    // `Some`: the present or the absent value of an `Option`.
                    // Macro `format!`: builds or prints text (or a list) from a template.
                    return Ok(Some(format!("{}{middle}{destination_suffix}", destination.prefix)));
                }
            }
            _ => {
                if refspec.source.prefix == merge {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    // `Some`: the present or the absent value of an `Option`.
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    return Ok(Some(destination.prefix.to_string()));
                }
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    return Ok(None);
}

/// What:
/// `pub fn check_refspec(spec: &str) -> Result<(), String>` says whether a fetch refspec is one this reader judges to be valid.
/// Why:
/// Git rejects the whole remote configuration when any refspec is invalid,
/// whether or not that remote is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkRefspec(spec: string): void  // throws on a spec it cannot judge
/// ```
pub fn check_refspec(spec: &str) -> Result<(), String> {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return parse_refspec(spec).map(drop);
}
