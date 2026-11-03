//! The unit tests of compare-before-replace and of recovery from a killed run.

use std::fs;
use std::path::Path;

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin, finish, recover, restore_if_unchanged};
use crate::test_scratch::scratch;

const CRATE_DIR: &str = "package/j/one";

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

#[test]
fn a_lock_that_still_holds_what_the_run_produced_is_replaced_by_the_snapshot() {
    let root = repository("produced");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    assert_eq!(lock(root.path()), "original");
}

#[test]
fn a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning() {
    let root = repository("written by someone else");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert_eq!(lock(root.path()), "written by someone else");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains(CRATE_DIR) && warnings[0].contains("another process"));
}

#[test]
fn a_lock_that_already_equals_the_snapshot_needs_nothing_and_no_temporary_file_is_left() {
    let root = repository("original");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    let leftovers: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR))
        .expect("list")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    assert_eq!(leftovers, vec!["Cargo.lock".to_string()]);
}

/// A process id that cannot exist on Linux (above the kernel's maximum).
const DEAD_PID: u32 = 4_194_305;

fn plant_journal(root: &Path, original: &str) {
    begin(root, &[snapshot(original)]).expect("begin");
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
}

#[test]
fn a_killed_run_whose_lock_holds_refused_versions_is_saved_aside_and_the_snapshot_goes_back() {
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(lock(root.path()), "original");
    assert_eq!(
        fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the saved lock"),
        "unchecked output of the killed run"
    );
    assert_eq!(messages.len(), 1);
    assert!(!root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn a_killed_run_whose_lock_passes_the_policy_is_kept() {
    let root = repository("good output");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert_eq!(lock(root.path()), "good output");
    assert!(!root.path().join(CRATE_DIR).join("Cargo.lock.refused").exists());
    assert_eq!(messages.len(), 1);
}

#[test]
fn a_journal_of_a_running_process_and_our_own_journal_are_not_touched() {
    let root = repository("live lock");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return true; }, &|_before, _live| { return vec!["never asked".to_string()]; });

    assert!(messages.is_empty());
    assert_eq!(lock(root.path()), "live lock");
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn finish_removes_this_processs_journal() {
    let root = repository("x");
    begin(root.path(), &[snapshot("x")]).expect("begin");

    finish(root.path());

    assert!(!root.path().join(JOURNAL_DIR).join(std::process::id().to_string()).exists());
}

/// Review finding (recovery destroys the remaining recovery copies):
/// a second refusal must not overwrite the first saved
/// lockfile,
/// so every saved copy gets its own name.
#[test]
fn a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives() {
    let root = repository("first unchecked output");
    plant_journal(root.path(), "original");
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
    recover(root.path(), &|_pid| { return false; }, &refuse);
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "second unchecked output").expect("the second killed run");
    plant_journal(root.path(), "original");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Borrow `&refuse`: lends the value without copying or giving it away.
    recover(root.path(), &|_pid| { return false; }, &refuse);

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
    plant_journal(root.path(), "original");
    let crate_dir = root.path().join(CRATE_DIR);
    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the directory read-only");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("make the directory writable again");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(messages.len(), 1, "{messages:?}");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "the journal of a failed recovery was deleted");
}

/// Review finding:
/// a live lockfile that cannot be read keeps the journal.
#[test]
fn an_unreadable_live_lock_keeps_the_journal() {
    let root = repository("whatever");
    plant_journal(root.path(), "original");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("remove the live lock");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "the journal was deleted with the live lock unread");
}

/// Review finding:
/// a journal that cannot be read in full keeps its directory.
#[test]
fn an_unreadable_journal_is_kept() {
    let root = repository("whatever");
    plant_journal(root.path(), "original");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).join("0.lock")).expect("damage the journal");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists(), "a damaged journal was deleted");
}

/// Review finding (compare-before-replace):
/// the policy judgement is slow,
/// and another writer can publish a lockfile while
/// it runs.
/// Recovery must compare again right before it replaces the file.
#[test]
fn recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged() {
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| {
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
    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[]).expect("restores");

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
    let warnings = restore_if_unchanged_with(root.path(), &[snapshot("original")], &[snapshot("produced")], &publish).expect("restores");

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
    plant_journal(root.path(), "original");
    let lock_path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Closure `|_path: &Path|`: an anonymous function, like a TS arrow function.
    // Borrow `&lock_path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let publish = move |_path: &Path| { fs::write(&lock_path, "published by another writer in the gap").expect("the other writer"); };

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Borrow `&publish`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover_with(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; }, &publish);

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
    assert!(messages.iter().any(|message| { return message.contains("another process"); }), "{messages:?}");
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
    assert_eq!(outcome, ReplaceOutcome::Replaced);
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
    assert_eq!(names, vec!["Cargo.lock".to_string()]);
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

fn plant_breaking_journal(root: &Path, original_lock: &str, original_manifest: &str) {
    use crate::manifest_edit::ManifestEdit;
    let manifest = root.join(CRATE_DIR).join("Cargo.toml");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    begin(root, &[snapshot(original_lock)]).expect("begin");
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    crate::journal::add_manifests(root, &[ManifestEdit { path: manifest, before: original_manifest.to_string(), produced: None }]).expect("journals the manifest");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    // Borrow `&ours`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
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
    plant_breaking_journal(root.path(), "original", "manifest before");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

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
    assert!(!root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

/// When the lockfile it left passes the policy,
/// the manifest edit that goes with it is kept.
#[test]
fn a_killed_breaking_run_whose_lock_passes_keeps_its_manifest_edit() {
    let root = repository("good output");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), "edited by cargo").expect("the edit");
    plant_breaking_journal(root.path(), "original", "manifest before");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

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
    plant_breaking_journal(root.path(), "original", "manifest before");

    // Closure `|_pid|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["never asked".to_string()]; });

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
    assert_eq!(outcome, ReplaceOutcome::Preserved(kept.clone()));
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
    assert_eq!(outcome, ReplaceOutcome::Preserved(kept.clone()));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.displaced")).expect("kept"), "an earlier displaced file");
    // Borrow `&kept`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&kept).expect("kept"), "second writer");
}
