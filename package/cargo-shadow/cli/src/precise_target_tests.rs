//! The unit tests of the release a `cargo update --precise` names.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use crate::lockfile::LockedVersion;
// Imports `crate::precise_target::precise_targets`.
use crate::precise_target::precise_targets;

fn locked(name: &str, version: &str) -> LockedVersion {
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return LockedVersion { name: name.to_string(), version: version.to_string() };
}

fn words(text: &str) -> Vec<String> {
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return text.split_whitespace().map(str::to_string).collect();
}

#[test]
fn every_spelling_of_the_package_option_names_the_same_entry() {
    let entries = [locked("dep", "1.0.1"), locked("other", "1.0.1"), locked("dep", "2.0.0")];
    for spelling in ["update -p dep --precise 1.0.1", "update --package dep --precise=1.0.1", "update -pdep --precise 1.0.1", "update -p=dep@1.0.1 --precise 1.0.1", "update --package=dep --precise 1.0.1"] {
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
        assert_eq!(precise_targets(&words(spelling), &entries), vec![locked("dep", "1.0.1")], "{spelling}");
    }
}

#[test]
fn a_source_url_spec_names_the_package_after_the_hash() {
    let entries = [locked("dep", "1.0.1")];
    let args = words("update --package https://example.invalid/index#dep@1.0.1 --precise 1.0.1");

    // Borrow `&args`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(precise_targets(&args, &entries), vec![locked("dep", "1.0.1")]);
}

#[test]
fn arguments_without_a_precise_release_name_nothing() {
    let entries = [locked("dep", "1.0.1")];

    // Borrow `&words`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(precise_targets(&words("update -p dep"), &entries).is_empty());
    // Borrow `&words`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(precise_targets(&words("update --precise"), &entries).is_empty());
}

#[test]
fn a_precise_release_that_is_not_locked_names_nothing() {
    let entries = [locked("dep", "1.0.0")];

    // Borrow `&words`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(precise_targets(&words("update -p dep --precise 1.0.1"), &entries).is_empty());
}
