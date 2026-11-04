//! The unit tests of the `cargo update` to `pnpm update` mapping.

use std::ffi::OsString;

use crate::update_plan::{UpdatePlan, plan_update};

fn plan(words: &[&str]) -> Result<crate::update_plan::Planned, String> {
    let arguments: Vec<OsString> = words.iter().map(OsString::from).collect();
    return plan_update(&arguments);
}

#[test]
fn no_names_is_pnpm_update_of_everything() {
    assert_eq!(plan(&["update"]).expect("plans").plan, UpdatePlan::Full { dry_run: false });
    assert_eq!(plan(&["+nightly", "update", "--dry-run"]).expect("plans").plan, UpdatePlan::Full { dry_run: true });
    assert_eq!(plan(&["update", "-n"]).expect("plans").plan, UpdatePlan::Full { dry_run: true });
}

#[test]
fn a_named_package_is_pnpm_update_of_that_name_run_as_given() {
    let planned = plan(&["update", "serde", "-p", "tokio@1.2.3"]).expect("plans");

    assert_eq!(
        planned.plan,
        UpdatePlan::Targeted {
            dry_run: false,
            cargo_args: ["update", "serde", "-p", "tokio@1.2.3"].iter().map(|word| { return word.to_string(); }).collect(),
            names: vec!["serde".to_string(), "tokio".to_string()],
            precise: false,
        }
    );
}

#[test]
fn precise_is_pnpm_update_name_at_version_and_needs_exactly_one_name() {
    let planned = plan(&["update", "serde", "--precise", "1.0.200"]).expect("plans");
    match planned.plan {
        UpdatePlan::Targeted { precise, cargo_args, .. } => {
            assert!(precise);
            assert!(cargo_args.contains(&"--precise".to_string()) && cargo_args.contains(&"1.0.200".to_string()));
        }
        other => panic!("expected targeted, got {other:?}"),
    }
    assert!(plan(&["update", "--precise", "1.0.0"]).is_err());
    assert!(plan(&["update", "a", "b", "--precise", "1.0.0"]).is_err());
}

#[test]
fn breaking_is_pnpm_update_latest_and_runs_cargos_own_breaking_update() {
    match plan(&["update", "--breaking"]).expect("plans").plan {
        UpdatePlan::Targeted { cargo_args, .. } => assert!(cargo_args.contains(&"--breaking".to_string())),
        other => panic!("expected targeted, got {other:?}"),
    }
}

#[test]
fn workspace_is_the_install_step_and_locked_or_frozen_is_the_frozen_check() {
    assert_eq!(plan(&["update", "--workspace"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
    assert_eq!(plan(&["update", "-w"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
    assert_eq!(plan(&["update", "--locked"]).expect("plans").plan, UpdatePlan::FrozenCheck);
    assert_eq!(plan(&["update", "serde", "--frozen"]).expect("plans").plan, UpdatePlan::FrozenCheck);
}

#[test]
fn generate_lockfile_is_pnpm_install_lockfile_only() {
    assert_eq!(plan(&["generate-lockfile"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
}

#[test]
fn cargo_only_options_are_forwarded_with_their_values_and_the_manifest_path_is_dropped() {
    let planned = plan(&["update", "--offline", "--color", "never", "--config", "net.retry=1", "-Zunstable-options", "-m", "a/Cargo.toml", "-q"])
        .expect("plans");

    assert_eq!(
        planned.forward,
        ["--offline", "--color", "never", "--config", "net.retry=1", "-Zunstable-options", "-q"].iter().map(|word| { return word.to_string(); }).collect::<Vec<String>>()
    );
    assert_eq!(planned.plan, UpdatePlan::Full { dry_run: false });
}

fn names_of(words: &[&str]) -> Vec<String> {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    match plan(words).expect("plans").plan {
        UpdatePlan::Targeted { names, .. } => return names,
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        other => panic!("expected targeted, got {other:?}"),
    }
}

/// Review finding (`--package=foo`):
/// the attached spelling names the package like `--package foo` does,
/// and the plan must not
/// widen into a full update.
#[test]
fn the_attached_and_short_spellings_of_package_name_the_package() {
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    assert_eq!(names_of(&["update", "--package=foo"]), vec!["foo".to_string()]);
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(names_of(&["update", "--package=foo@1.2.3"]), vec!["foo".to_string()]);
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(names_of(&["update", "-pfoo"]), vec!["foo".to_string()]);
}

/// Review finding (global options before the verb):
/// `--locked` before the verb asks for the frozen check,
/// as after it.
#[test]
fn a_locked_or_frozen_before_the_verb_is_the_frozen_check() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(plan(&["--locked", "update"]).expect("plans").plan, UpdatePlan::FrozenCheck);
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(plan(&["+nightly", "--frozen", "update"]).expect("plans").plan, UpdatePlan::FrozenCheck);
}

/// Review finding:
/// cargo-only options before the verb are forwarded to every cargo call the procedure makes.
#[test]
fn cargo_only_options_before_the_verb_are_forwarded_too() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let planned = plan(&["--offline", "--color", "never", "update"]).expect("plans");

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(planned.forward, vec!["--offline".to_string(), "--color".to_string(), "never".to_string()]);
}

/// Review finding (reject,
/// do not broaden):
/// an option the shadow cannot map to pnpm update is refused by name.
#[test]
fn an_option_the_shadow_cannot_answer_is_refused_by_name_and_not_dropped() {
    // Borrow `&[`: lends the value without copying or giving it away.
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
    let message: String = plan(&["update", "--bogus"]).expect_err("an unknown option must be refused");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("--bogus"), "{message}");
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "serde", "--lockfile-path", "x"]).is_err());
}

/// The options of `cargo update --help` stay accepted.
#[test]
fn every_option_of_cargo_update_help_is_still_accepted() {
    for words in [
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec!["update", "--recursive", "serde"],
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec!["update", "-vv"],
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec!["update", "--ignore-rust-version"],
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec!["update", "--manifest-path=a/Cargo.toml"],
        // Macro `vec!`: builds or prints text (or a list) from a template.
        vec!["-C", "dir", "update"],
    ] {
        // Borrow `&words`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(plan(&words).is_ok(), "{words:?}");
    }
}

/// Review finding (2c 9):
/// an option that needs a value and has none is cargo's argument error.
/// The wrapper must not turn it into a
/// full update of the current workspace.
#[test]
fn an_option_without_its_value_is_refused_and_not_dropped() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "-m"]).is_err());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "--manifest-path"]).is_err());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "--precise"]).is_err());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["-C"]).is_err());
}

/// The attached short form of the manifest path selects a workspace like the separate one.
#[test]
fn the_attached_short_manifest_path_is_understood() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(plan(&["update", "-m../b/Cargo.toml"]).expect("plans").plan, UpdatePlan::Full { dry_run: false });
}

/// Review finding (2c 10):
/// an option value that equals the command word is not the command word.
#[test]
fn an_option_value_equal_to_the_verb_is_not_the_verb() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let planned = plan(&["--config", "update", "update"]).expect("plans");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(planned.plan, UpdatePlan::Full { dry_run: false });
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(planned.forward, vec!["--config".to_string(), "update".to_string()]);
}

/// Review finding (3c 9):
/// an attached option value that is empty is a missing value,
/// not a path.
#[test]
fn an_empty_attached_manifest_path_is_refused() {
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "--manifest-path="]).is_err());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "-m="]).is_err());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(plan(&["update", "--precise="]).is_err());
}
