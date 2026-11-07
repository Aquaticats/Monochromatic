//! The unit tests of compare-before-replace and of recovery from a killed run.

use std::fs;
use std::path::Path;

use crate::crates::LockSnapshot;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use crate::journal::{ReplaceOutcome, recover, restore_if_unchanged};
// Imports `crate::journal_support::{journal_dir, journ...`.
use crate::journal_support::{journal_dir, journal_dirs, plant};
// Imports `crate::manifest_edit::ManifestEdit`.
use crate::manifest_edit::ManifestEdit;
use crate::test_scratch::scratch;

const CRATE_DIR: &str = "package/j/one";

/// The journaled edit of the manifest next to the lockfile of the case.
fn breaking_edit(root: &Path, before: &str) -> ManifestEdit {
    // `None`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    return ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: root.join(CRATE_DIR).join("Cargo.toml"), before: before.to_string(), produced: None };
}

fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

fn lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("read");
}

/// The warnings of a restore that must not have failed.
fn restored(result: (Vec<String>, Option<std::io::Error>)) -> Vec<String> {
    assert!(result.1.is_none(), "the restore failed: {:?}", result.1);
    return result.0;
}

/// Round 6 (5b 2):
/// a restore that fails on one root still restores the others and returns the warnings it collected,
/// beside the failure,
/// so the person is told about every retained path.
#[test]
fn a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others() {
    let root = scratch();
    let names = ["package/j/a", "package/j/b", "package/j/c", "package/j/d"];
    for name in names {
        fs::create_dir_all(root.path().join(name)).expect("dir");
    }
    fs::write(root.path().join(names[0]).join("Cargo.lock"), "written by someone else").expect("a: changed by another process");
    fs::create_dir(root.path().join(names[1]).join("Cargo.lock")).expect("b: a directory where the lockfile should be, so reading it fails");
    fs::write(root.path().join(names[2]).join("Cargo.lock"), "produced").expect("c: still what the run produced");
    fs::write(root.path().join(names[3]).join("Cargo.lock"), "written by yet another process").expect("d: after the failed root, changed by another process, so it needs a warning of its own");
    let snapshots: Vec<LockSnapshot> = names.iter().map(|name| { return LockSnapshot { crate_dir: name.to_string(), source: "original".to_string() }; }).collect();
    let produced: Vec<LockSnapshot> = names.iter().map(|name| { return LockSnapshot { crate_dir: name.to_string(), source: "produced".to_string() }; }).collect();

    let (warnings, failure) = restore_if_unchanged(root.path(), &snapshots, &produced);

    assert!(failure.is_some(), "the unreadable root was not reported as a failure");
    assert!(warnings.iter().any(|warning| { return warning.contains(names[0]) && warning.contains("another process"); }), "the warning of the first root was lost: {warnings:?}");
    assert_eq!(fs::read_to_string(root.path().join(names[2]).join("Cargo.lock")).expect("c"), "original", "the root after the failed one was not restored");
    // Certification, second pass (R5): a warning generated after the failed root is kept beside the earlier one, and that root's file
    // is left as the other process wrote it.
    assert!(warnings.iter().any(|warning| { return warning.contains(names[3]) && warning.contains("another process"); }), "the warning of a root after the failed one was lost: {warnings:?}");
    assert_eq!(fs::read_to_string(root.path().join(names[3]).join("Cargo.lock")).expect("d"), "written by yet another process");
}

#[test]
fn a_lock_that_still_holds_what_the_run_produced_is_replaced_by_the_snapshot() {
    let root = repository("produced");

    let warnings = restored(restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]));

    assert!(warnings.is_empty());
    assert_eq!(lock(root.path()), "original");
}

#[test]
fn a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning() {
    let root = repository("written by someone else");

    let warnings = restored(restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]));

    assert_eq!(lock(root.path()), "written by someone else");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains(CRATE_DIR) && warnings[0].contains("another process"));
}

#[test]
fn a_lock_that_already_equals_the_snapshot_needs_nothing_and_no_temporary_file_is_left() {
    let root = repository("original");

    let warnings = restored(restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]));

    assert!(warnings.is_empty());
    let leftovers: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR))
        .expect("list")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    assert_eq!(leftovers, vec!["Cargo.lock".to_string()]);
}

/// A process id that cannot exist on Linux (above the kernel's maximum).

#[test]
fn a_killed_run_whose_lock_holds_refused_versions_is_saved_aside_and_the_snapshot_goes_back() {
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);

    // Closure `|_pid, _identity|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(lock(root.path()), "original");
    assert_eq!(
        fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the saved lock"),
        "unchecked output of the killed run"
    );
    assert_eq!(messages.len(), 1);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal_dirs(root.path()).is_empty());
}

#[test]
fn a_killed_run_whose_lock_passes_the_policy_is_kept() {
    let root = repository("good output");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);

    // Closure `|_pid, _identity|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let messages = recover(root.path(), &|_before, _live| { return Vec::new(); });

    assert_eq!(lock(root.path()), "good output");
    assert!(!root.path().join(CRATE_DIR).join("Cargo.lock.refused").exists());
    assert_eq!(messages.len(), 1);
}

/// Review finding (recovery destroys the remaining recovery copies):
/// a second refusal must not overwrite the first saved
/// lockfile,
/// so every saved copy gets its own name.
#[test]
fn a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives() {
    let root = repository("first unchecked output");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
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
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let refuse = |_before: &[LockSnapshot], _live: &[LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; };
    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    recover(root.path(), &refuse);
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "second unchecked output").expect("the second killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Borrow `&refuse`: lends the value without copying or giving it away.
    recover(root.path(), &refuse);

    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
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
    let mut saved: Vec<String> = Vec::new();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    for entry in fs::read_dir(root.path().join(CRATE_DIR)).expect("list").flatten() {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Typed binding: `name: String`.
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("Cargo.lock.refused") {
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            saved.push(fs::read_to_string(entry.path()).expect("read a saved copy"));
        }
    }
    saved.sort();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(saved, vec!["first unchecked output".to_string(), "second unchecked output".to_string()]);
}

/// Review finding:
/// a recovery that could not finish keeps the journal,
/// because the journal holds the only copy of the
/// original lockfile.
/// Here the crate directory cannot be written,
/// so neither the refused copy nor the restore can happen.
#[test]
fn a_recovery_that_fails_keeps_the_journal() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    let crate_dir = root.path().join(CRATE_DIR);
    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the directory read-only");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("make the directory writable again");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(messages.len(), 1, "{messages:?}");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal_dirs(root.path()).len() == 1, "the journal of a failed recovery was deleted");
}

/// Review finding:
/// a live lockfile that cannot be read keeps the journal.
#[test]
fn an_unreadable_live_lock_keeps_the_journal() {
    let root = repository("whatever");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("remove the live lock");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_before, _live| { return Vec::new(); });

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal_dirs(root.path()).len() == 1, "the journal was deleted with the live lock unread");
}

/// Review finding:
/// a journal that cannot be read in full keeps its directory.
#[test]
fn an_unreadable_journal_is_kept() {
    let root = repository("whatever");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(journal_dir(root.path()).join("0.lock")).expect("damage the journal");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_before, _live| { return Vec::new(); });

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal_dirs(root.path()).len() == 1, "a damaged journal was deleted");
}

/// Review finding (compare-before-replace):
/// the policy judgement is slow,
/// and another writer can publish a lockfile while
/// it runs.
/// Recovery must compare again right before it replaces the file.
#[test]
fn recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged() {
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    let messages = recover(root.path(), &|_before, _live| {
        // Borrow `&lock_path`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(&lock_path, "published by another writer meanwhile").expect("the other writer");
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        return vec!["dep@1.0.1 refused".to_string()];
    });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "published by another writer meanwhile");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(messages.len(), 1, "{messages:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages[0].contains("another process"), "{messages:?}");
}

/// Review finding (unknown ownership):
/// a lockfile this run never produced is not restored.
#[test]
fn a_lock_the_run_never_produced_is_left_alone_even_when_it_differs_from_the_snapshot() {
    let root = repository("written by someone else");

    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(restore_if_unchanged(root.path(), &[snapshot("original")], &[]));

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "written by someone else");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 1);
}

/// Delegated pick 5 (2026-11-03):
/// a restore compares and replaces in one atomic step (`renameat2` with `RENAME_EXCHANGE`) and
/// undoes the swap when the file it displaced is not the one it expected.
/// Another writer publishes a lockfile in the gap
/// between the comparison and the replacement;
/// that lockfile must survive.
#[test]
fn a_lockfile_published_between_the_comparison_and_the_replacement_is_not_lost() {
    use crate::journal::restore_if_unchanged_with;
    let root = repository("produced");
    // Closure `|path: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let publish = |path: &Path| { fs::write(path, "published by another writer in the gap").expect("the other writer"); };

    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(restore_if_unchanged_with(root.path(), &[snapshot("original")], &[snapshot("produced")], &publish));

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "published by another writer in the gap");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

/// The recovery of a killed run replaces the same way:
/// a lockfile published after it was judged survives.
#[test]
fn recovery_does_not_replace_a_lock_published_in_the_gap_after_the_last_comparison() {
    use crate::journal::recover_with;
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Closure `|_path: &Path|`: an anonymous function, like a TS arrow function.
    // Borrow `&lock_path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let publish = move |_path: &Path| { fs::write(&lock_path, "published by another writer in the gap").expect("the other writer"); };

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Borrow `&publish`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover_with(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; }, &publish);

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "published by another writer in the gap");
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.iter().any(|message| { return message.contains("other process"); }), "{messages:?}");
}

#[test]
fn a_compare_and_replace_that_finds_the_expected_text_replaces_it_and_leaves_no_temporary_file() {
    use crate::journal::{ReplaceOutcome, replace_if_holds};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");

    // Closure `|_path|`: an anonymous function, like a TS arrow function.
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("replaces");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert!(matches!(outcome, ReplaceOutcome::Replaced(_)), "{outcome:?}");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "new text");
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `names: Vec<String>`.
    let names: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR)).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(names, vec!["Cargo.lock".to_string(), "Cargo.lock.displaced".to_string()], "the displaced file keeps a name; no temporary file is left");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.displaced")).expect("kept"), "expected");
}

#[test]
fn a_compare_and_replace_that_finds_other_text_or_no_file_leaves_things_as_they_are() {
    use crate::journal::{ReplaceOutcome, replace_if_holds};
    let root = repository("somebody else's");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");

    // Closure `|_path|`: an anonymous function, like a TS arrow function.
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("answers"), ReplaceOutcome::LeftAlone);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "somebody else's");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(&path).expect("remove");
    // Closure `|_path|`: an anonymous function, like a TS arrow function.
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(replace_if_holds(&path, "expected", "new text", &|_path| {}).expect("answers"), ReplaceOutcome::LeftAlone);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!path.exists());
}

fn manifest(root: &Path) -> String {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.toml")).expect("read the manifest");
}

/// Delegated pick 8 (2026-11-03):
/// a killed `--breaking` run is recovered for the manifests it rewrote.
/// When the lockfile it left
/// holds refused versions,
/// the lockfile and the manifest both go back,
/// and the text that was replaced is kept aside.
#[test]
fn a_killed_breaking_run_whose_lock_is_refused_gets_its_manifest_back_too() {
    let root = repository("unchecked output of the killed run");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[breaking_edit(root.path(), "manifest before")]);

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "original");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest(root.path()), "manifest before");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let kept = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.toml.refused")).expect("the replaced manifest is kept aside");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(kept, "edited by cargo");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal_dirs(root.path()).is_empty());
}

/// When the lockfile it left passes the policy,
/// the manifest edit that goes with it is kept.
#[test]
fn a_killed_breaking_run_whose_lock_passes_keeps_its_manifest_edit() {
    let root = repository("good output");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[breaking_edit(root.path(), "manifest before")]);

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_before, _live| { return Vec::new(); });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "good output");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest(root.path()), "edited by cargo");
}

/// A run killed before cargo wrote the lockfile but after it rewrote a manifest:
/// the lockfile is unchanged,
/// the manifest goes back.
#[test]
fn a_killed_breaking_run_that_only_rewrote_the_manifest_gets_it_back() {
    let root = repository("original");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[breaking_edit(root.path(), "manifest before")]);

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before, _live| { return vec!["never asked".to_string()]; });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest(root.path()), "manifest before");
}

/// A file displaced twice:
/// one writer publishes in the gap before the exchange,
/// another between the exchange and its undo.
/// Neither file is lost:
/// the first is back at the path,
/// the second is kept as `Cargo.lock.displaced`.
#[test]
fn a_file_displaced_twice_is_kept_as_displaced_and_the_first_writers_file_is_put_back() {
    use crate::journal::{ReplaceOutcome, replace_if_holds_with_hooks};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let second_writer = |target: &Path| { fs::write(target, "second writer").expect("the second writer"); };

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &second_writer).expect("answers");

    let kept = root.path().join(CRATE_DIR).join("Cargo.lock.displaced");
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(outcome, ReplaceOutcome::Undone(kept.clone()));
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "first writer");
    // Borrow `&kept`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&kept).expect("the displaced file is kept"), "second writer");
}

/// A second displaced file does not replace the first kept one.
#[test]
fn a_second_displaced_file_gets_its_own_name() {
    use crate::journal::{ReplaceOutcome, replace_if_holds_with_hooks};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock.displaced"), "an earlier displaced file").expect("earlier");
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let second_writer = |target: &Path| { fs::write(target, "second writer").expect("the second writer"); };

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &second_writer).expect("answers");

    let kept = root.path().join(CRATE_DIR).join("Cargo.lock.displaced.1");
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(outcome, ReplaceOutcome::Undone(kept.clone()));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.displaced")).expect("kept"), "an earlier displaced file");
    // Borrow `&kept`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&kept).expect("kept"), "second writer");
}

// ---- the replacement protocol, second review (2b)

fn names_in(directory: &Path) -> Vec<String> {
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // `let mut`: a binding that may change.
    // Typed binding: `names: Vec<String>`.
    let mut names: Vec<String> = fs::read_dir(directory).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
    names.sort();
    return names;
}

/// Review finding (2b 3):
/// a filesystem that cannot exchange must make the restore fail visibly.
/// The old fallback compared again and
/// renamed over the target,
/// which overwrites a file published after the comparison.
#[test]
fn a_filesystem_without_exchange_fails_the_restore_and_overwrites_nothing() {
    use crate::journal::{KeepAs, replace_with_swap};
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    let cannot = |_first: &Path, _second: &Path| -> std::io::Result<()> { return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "no exchange here")); };

    // Borrow `&path`: lends the value without copying or giving it away.
    let result = replace_with_swap(&path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &cannot);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "expected");
    // Borrow `&root.path`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(names_in(&root.path().join(CRATE_DIR)), vec!["Cargo.lock".to_string()], "the replacement file must not be left behind");
}

/// Review finding (2b 5):
/// a live file that is not UTF-8 text is an error;
/// it must not read as an empty file that matches an empty
/// expectation and gets deleted.
#[test]
fn a_live_file_that_cannot_be_read_as_text_is_an_error_and_is_left_alone() {
    use crate::journal::replace_if_holds;
    let root = repository("");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, [0xff, 0xfe, 0x00]).expect("a person's binary edit");

    // Closure `|_path|`: an anonymous function, like a TS arrow function.
    // Borrow `&path`: lends the value without copying or giving it away.
    let result = replace_if_holds(&path, "", "new text", &|_path| {});

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read(&path).expect("read"), vec![0xff, 0xfe, 0x00]);
}

/// Review finding (2b 2):
/// an editor holds the live file open across the exchange and writes afterwards.
/// The displaced inode keeps a
/// name,
/// so the write is not lost.
#[test]
fn a_write_through_a_descriptor_that_was_open_across_the_exchange_survives_in_the_kept_file() {
    use crate::journal::{KeepAs, replace_keeping};
    use std::io::Write;
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `let mut`: a binding that may change.
    let mut editor = fs::OpenOptions::new().append(true).open(&path).expect("the editor opens the file");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_keeping(&path, "expected", "new text", &KeepAs::next_to_target(".displaced")).expect("replaces");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    editor.write_all(b" and a late edit").expect("the late write goes to the old inode");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    editor.sync_all().expect("sync");

    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    let kept = match outcome {
        // `Some`: the present or the absent value of an `Option`.
        ReplaceOutcome::Replaced(Some(found)) => found,
        // Macro `panic!`: builds or prints text (or a list) from a template.
        other => panic!("expected the displaced file to be kept: {other:?}"),
    };
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(kept).expect("read"), "expected and a late edit");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lock(root.path()), "new text");
}

/// A replacement that is itself written to in place by a second writer between the exchange and its undo is not mistaken for the
/// run's own file:
/// its text differs,
/// so it is kept.
#[test]
fn a_replacement_edited_in_place_after_the_exchange_is_kept_not_deleted() {
    use crate::journal::replace_if_holds_with_hooks;
    use std::os::unix::fs::MetadataExt;
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let replacement_inode: std::cell::Cell<u64> = std::cell::Cell::new(0);
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    let in_place = |target: &Path| {
        // The replacement is what the live path names right after the exchange; its inode is captured before the second writer writes.
        replacement_inode.set(fs::metadata(target).expect("the replacement").ino());
        let mut file = fs::OpenOptions::new().write(true).truncate(true).open(target).expect("open in place");
        std::io::Write::write_all(&mut file, b"second writer, in place").expect("write");
    };

    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &in_place).expect("answers");

    assert!(matches!(outcome, ReplaceOutcome::Undone(_)), "{outcome:?}");
    assert_eq!(lock(root.path()), "first writer");
    let kept: Vec<String> = names_in(&root.path().join(CRATE_DIR)).into_iter().filter(|name| { return name.contains("displaced"); }).collect();
    assert_eq!(kept.len(), 1, "{kept:?}");
    // The kept name holds the in-place writer's bytes and is the captured replacement inode (certification group 3): a placeholder
    // with another content, or another file, would not satisfy this.
    let kept_path = root.path().join(CRATE_DIR).join(&kept[0]);
    assert_eq!(fs::read_to_string(&kept_path).expect("the kept file"), "second writer, in place", "the in-place writer's result was lost");
    assert_eq!(fs::metadata(&kept_path).expect("the kept file").ino(), replacement_inode.get(), "the kept name is not the replacement inode");
}

// ---- recovery, second review (2b)

fn two_root_repository(one: &str, two: &str) -> tempfile::TempDir {
    let root = scratch();
    for (dir, text) in [(CRATE_DIR, one), ("package/j/two", two)] {
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::create_dir_all(root.path().join(dir)).expect("dir");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(root.path().join(dir).join("Cargo.lock"), text).expect("lock");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(root.path().join(dir).join("Cargo.toml"), "edited by cargo").expect("manifest");
    }
    return root;
}

/// Review finding (2b 8):
/// root one's new lockfile passes the policy,
/// root two's does not.
/// Only root two's manifest goes back.
#[test]
fn a_kept_root_keeps_its_manifest_and_a_rejected_root_gets_its_own_back() {
    use crate::manifest_edit::ManifestEdit;
    let root = two_root_repository("good output", "refused output");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let second = LockSnapshot { crate_dir: "package/j/two".to_string(), source: "original two".to_string() };
    let edits = [
        ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: root.path().join(CRATE_DIR).join("Cargo.toml"), before: "one before".to_string(), produced: None },
        ManifestEdit { crate_dir: "package/j/two".to_string(), path: root.path().join("package/j/two").join("Cargo.toml"), before: "two before".to_string(), produced: None },
    ];
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original one"), second], &edits);

    // Closure `|before, _live|`: an anonymous function, like a TS arrow function.
    recover(root.path(), &|before, _live| {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return if before[0].crate_dir == CRATE_DIR { Vec::new() } else { vec!["dep@1.0.1 refused".to_string()] };
    });

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.toml")).expect("read"), "edited by cargo");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join("package/j/two").join("Cargo.toml")).expect("read"), "two before");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join("package/j/two").join("Cargo.lock")).expect("read"), "original two");
}

/// Review finding (2b 7):
/// a manifest index that exists but cannot be read keeps the journal,
/// which holds the only original manifests.
#[test]
fn an_unreadable_or_truncated_manifest_index_keeps_the_journal() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository("unchecked output of the killed run");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[breaking_edit(root.path(), "manifest before")]);
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let journal = journal_dir(root.path());
    let index = journal.join("manifests.txt");

    // Truncated: the `end` line is missing.
    let text = fs::read_to_string(&index).expect("index");
    // Borrow `&index`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&index, text.replace("end 1\n", "")).expect("truncate");
    // Closure `|_pid, _identity|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before, _live| { return vec!["refused".to_string()]; });
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal.exists(), "a truncated manifest index must keep the journal");

    // Unreadable: no permission to read it.
    fs::write(&index, text).expect("restore the index");
    // Borrow `&index`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&index, fs::Permissions::from_mode(0o000)).expect("remove all permissions");
    // Closure `|_pid, _identity|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before, _live| { return vec!["refused".to_string()]; });
    // Borrow `&index`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&index, fs::Permissions::from_mode(0o644)).expect("give the permissions back");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal.exists(), "an unreadable manifest index must keep the journal");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest(root.path()), "edited by cargo", "nothing may be restored from a journal that could not be read");
}

/// Review finding (2b 10):
/// a manifest path that is not UTF-8 survives the journal byte for byte.
#[test]
fn a_manifest_path_that_is_not_utf8_is_recovered_exactly() {
    use crate::manifest_edit::ManifestEdit;
    use std::os::unix::ffi::OsStringExt;
    let root = repository("unchecked output of the killed run");
    // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
    // Why:      The result can be stored or modified independently of the slice it came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // [...slice]
    // ```
    // What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
    // Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let odd = root.path().join(std::ffi::OsString::from_vec(b"odd-\xff-dir".to_vec()));
    // Borrow `&odd`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&odd).expect("a directory whose name is not UTF-8");
    let manifest_path = odd.join("Cargo.toml");
    // Borrow `&manifest_path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&manifest_path, "edited by cargo").expect("manifest");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    plant(root.path(), &[snapshot("original")], &[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest_path.clone(), before: "manifest before".to_string(), produced: None }]);

    // Closure `|_pid, _identity|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before, _live| { return vec!["refused".to_string()]; });

    // Borrow `&manifest_path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&manifest_path).expect("read"), "manifest before");
}

/// Acceptance row R1 (certification,
/// third pass):
/// a conflict found at the exchange boundary of a rollback (another process changes the
/// file in place between the comparison and the exchange,
/// so the exchange displaced a file that is not the expected one) leaves both
/// writers' bytes intact,
/// keeps the replacement,
/// and is reported as a warning that names the kept file.
#[test]
fn a_rollback_whose_exchange_finds_another_writers_change_keeps_both_files_and_warns() {
    use std::os::unix::fs::MetadataExt;
    let root = repository("produced");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let writers_inode: u64 = fs::metadata(&path).expect("examines").ino();
    let other_writer = |live: &Path| { fs::write(live, "another writer's text").expect("the other writer writes"); };

    let (warnings, failure) = crate::journal::restore_if_unchanged_with(root.path(), &[snapshot("original")], &[snapshot("produced")], &other_writer);

    assert!(failure.is_none(), "{failure:?}");
    let warning: &String = warnings.iter().find(|text| { return text.contains("was changed again during the restore"); }).unwrap_or_else(|| { panic!("the conflict at the exchange boundary was not reported: {warnings:?}"); });
    // The other writer's bytes are live again and its inode is the one that was there; the replacement (the snapshot text) is kept.
    assert_eq!(lock(root.path()), "another writer's text");
    assert_eq!(fs::metadata(&path).expect("examines").ino(), writers_inode, "the other writer's inode is not the live file");
    let kept: &str = warning.rsplit("kept as ").next().expect("a kept path").trim();
    assert_eq!(fs::read_to_string(kept).expect("the kept replacement"), "original", "the replacement was not kept under the reported name: {warning}");
}
