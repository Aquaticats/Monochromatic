//! The unit tests of the lockfile reading,
//! ported from the TypeScript tool's `lockfile-diff` cases.

use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

fn lock(entries: &[(&str, &str, &str)]) -> String {
    let mut text: String = "version = 4\n".to_string();
    for (name, version, source) in entries {
        text.push_str(&format!("\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n"));
        if !source.is_empty() {
            text.push_str(&format!("source = \"{source}\"\n"));
        }
    }
    return text;
}

fn version(name: &str, number: &str) -> LockedVersion {
    return LockedVersion { name: name.to_string(), version: number.to_string() };
}

#[test]
fn only_registry_packages_are_read() {
    let text: String = lock(&[("member", "0.1.0", ""), ("dep", "1.0.0", REGISTRY), ("fromgit", "2.0.0", "git+https://example.invalid/x")]);

    assert_eq!(parse_registry_versions(&text), Ok(vec![version("dep", "1.0.0")]));
}

#[test]
fn text_that_is_not_toml_is_an_error_not_an_empty_list() {
    assert!(parse_registry_versions("[[package\n").is_err());
}

#[test]
fn versions_are_deduplicated_across_lockfiles_and_sorted() {
    let first: String = lock(&[("b", "1.0.0", REGISTRY), ("a", "2.0.0", REGISTRY)]);
    let second: String = lock(&[("a", "2.0.0", REGISTRY)]);

    assert_eq!(registry_versions(&[first, second]), Ok(vec![version("a", "2.0.0"), version("b", "1.0.0")]));
}

#[test]
fn a_version_another_lockfile_already_held_is_not_added() {
    let before_one: String = lock(&[("a", "1.0.0", REGISTRY)]);
    let before_two: String = lock(&[("a", "2.0.0", REGISTRY)]);
    let after: String = lock(&[("a", "2.0.0", REGISTRY), ("b", "1.0.0", REGISTRY)]);

    assert_eq!(added_versions(&[before_one, before_two], &[after]), Ok(vec![version("b", "1.0.0")]));
}

#[test]
fn an_absent_base_lockfile_counts_every_version_as_new() {
    let after: String = lock(&[("a", "1.0.0", REGISTRY)]);

    assert_eq!(added_versions(&[String::new()], &[after]), Ok(vec![version("a", "1.0.0")]));
}

#[test]
fn direct_dependencies_are_the_names_the_sourceless_packages_depend_on() {
    let text: &str = "version = 4\n\n[[package]]\nname = \"member\"\nversion = \"0.1.0\"\ndependencies = [\"dep\", \"twice 1.0.0\"]\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = \"registry+x\"\ndependencies = [\"deep\"]\n";

    let names: Vec<String> = crate::lockfile::direct_dependency_names(text).expect("parses").into_iter().collect();

    assert_eq!(names, vec!["dep".to_string(), "twice".to_string()]);
}

/// Review finding (registry identity):
/// a package from any registry other than crates.io must be refused by name,
/// because
/// every lookup goes to crates.io and would approve an unrelated crate of the same name and version.
#[test]
fn a_package_from_another_registry_is_refused_naming_the_package_and_its_source() {
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
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
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let text: String = lock(&[("dep", "1.0.0", "registry+https://registry.example.invalid/index")]);

    // Borrow `&text`: lends the value without copying or giving it away.
    let result = parse_registry_versions(&text);

    // Typed binding: `message: String`.
    let message: String = result.expect_err("an unsupported registry must not be read as crates.io");
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert!(message.contains("dep") && message.contains("registry.example.invalid"), "{message}");
}

/// The sparse spelling of an alternative registry is the same case.
#[test]
fn a_sparse_registry_package_is_refused_too() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // Typed binding: `text: String`.
    let text: String = lock(&[("dep", "1.0.0", "sparse+https://registry.example.invalid/index/")]);

    // Borrow `&text`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_registry_versions(&text).is_err());
}
