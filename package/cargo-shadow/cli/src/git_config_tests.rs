//! The unit tests of the Git configuration reader.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use crate::git_config::{ConfigEntry, parse_config};

fn entry(section: &str, subsection: Option<&str>, key: &str, value: &str) -> ConfigEntry {
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return ConfigEntry { section: section.to_string(), subsection: subsection.map(str::to_string), key: key.to_string(), value: value.to_string() };
}

#[test]
fn sections_subsections_keys_and_values_are_read_with_git_case_rules() {
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
    let text: &str = "[Core]\n\tBare = false\n[branch \"Feature/One\"]\n\tremote = origin\n\tMerge = refs/heads/main\n";

    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    let parsed = parse_config(text).expect("parses");

    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(
        parsed.entries,
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec![
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            entry("core", None, "bare", "false"),
            // `Some`: the present or the absent value of an `Option`.
            entry("branch", Some("Feature/One"), "remote", "origin"),
            // `Some`: the present or the absent value of an `Option`.
            entry("branch", Some("Feature/One"), "merge", "refs/heads/main"),
        ]
    );
    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("BRANCH", Some("Feature/One"), "MERGE"), Some("refs/heads/main"));
    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("branch", Some("feature/one"), "merge"), None, "subsection names keep their case");
}

#[test]
fn the_older_dotted_header_lowercases_its_subsection() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let parsed = parse_config("[branch.Main]\n\tremote = origin\n").expect("parses");

    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("branch", Some("main"), "remote"), Some("origin"));
}

#[test]
fn a_key_may_repeat_and_the_last_value_wins_for_a_single_valued_key() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let parsed = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n\tfetch = ^refs/heads/skip\n[core]\n\tx = 1\n\tx = 2\n").expect("parses");

    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.values("remote", Some("origin"), "fetch").len(), 2);
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("core", None, "x"), Some("2"));
}

#[test]
fn quotes_escapes_comments_and_continuations_are_applied() {
    // Typed binding: `text: &str`.
    let text: &str = "[a]\n\tq = \"x ; y\"  ; trailing comment\n\te = a\\tb\n\tc = one \\\ntwo\n\tbare\n# whole line\n";

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let parsed = parse_config(text).expect("parses");

    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("a", None, "q"), Some("x ; y"));
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("a", None, "e"), Some("a\tb"));
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("a", None, "c"), Some("one two"));
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("a", None, "bare"), Some("true"));
}

#[test]
fn a_key_on_the_header_line_is_read() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let parsed = parse_config("[core] bare = true\n").expect("parses");

    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(parsed.last("core", None, "bare"), Some("true"));
}

#[test]
fn include_and_include_if_sections_are_flagged_and_not_followed() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[include]\n\tpath = x\n").expect("parses").has_include);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[includeIf \"gitdir:/x/\"]\n\tpath = y\n").expect("parses").has_include);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!parse_config("[core]\n\tbare = false\n").expect("parses").has_include);
}

#[test]
fn text_that_cannot_be_read_exactly_is_an_error() {
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("key = value\n").is_err(), "a line before any header");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[core\n").is_err(), "an unclosed header");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[core]\n\tx = \"open\n").is_err(), "an unclosed quote");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[core]\n\tx = a\\qb\n").is_err(), "an unknown escape");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[core]\n\t1x = y\n").is_err(), "a key starting with a digit");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(parse_config("[branch main]\n").is_err(), "a subsection without quotes");
}
