//! What:
//! The transaction around the manifests that cargo's own `update --breaking` rewrites:
//! the text of every workspace manifest before the run,
//! the text after it,
//! and the restore that puts back only what this run wrote.
//! Why:
//! `cargo update --breaking` edits version requirements in `Cargo.toml` as well as the lockfile,
//! so a refusal or a dry run that restored only the lockfiles would leave the new requirements behind.
//! The restore follows the lockfile rule:
//! a manifest is replaced only while it still holds what this run's cargo wrote,
//! through a sibling file and an atomic rename,
//! and one that another process changed is left alone with a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // manifest-edit.ts
//! export type ManifestEdit = { path: string; before: string; produced?: string };
//! export function snapshotManifests(paths: string[]): ManifestEdit[]
//! export function recordManifestOutput(edits: ManifestEdit[]): void
//! export function restoreManifests(edits: ManifestEdit[]): string[]
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a list that can be changed through a shared reference.
/// Why:
/// The list of manifests grows while the transaction holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::io;` imports the file-system error module.
/// Why:
/// A manifest that cannot be read is an error,
/// not an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io;
/// What:
/// `use std::path::PathBuf;` imports the owned path type.
/// Why:
/// A manifest's path is stored,
/// so it is the owned type (sibling:
/// `&Path`,
/// the borrowed one).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::PathBuf;

/// What:
/// `use crate::file_owner::{Claim, Window, claim_output, observe, timed};` imports the ownership evidence.
/// Why:
/// Each manifest is attributed to the cargo call of its own root with the same rule the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim, Window, claimOutput, observe, timed } from 'crate/file_owner';
/// ```
use crate::file_owner::{Claim, Window, claim_output, observe, timed};
/// What:
/// `use crate::journal::{KeepAs, ReplaceOutcome, replace_keeping};` imports the replacement protocol.
/// Why:
/// A manifest goes back with the same protocol the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeepAs, ReplaceOutcome, replaceKeeping } from 'crate/journal';
/// ```
use crate::journal::{KeepAs, ReplaceOutcome, replace_keeping};

/// One manifest the run may rewrite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEdit {
    /// The crate root whose cargo call may rewrite the manifest.
    pub crate_dir: String,
    /// Where the manifest is.
    pub path: PathBuf,
    /// Its text before the run.
    pub before: String,
    /// Its text as this run's cargo call left it;
    /// `None` until the evidence says the call wrote it,
    /// and a manifest without it is never restored.
    pub produced: Option<String>,
}

/// What:
/// `pub fn snapshot_manifests(crate_dir, paths, edits) -> io::Result<()>` reads each manifest and adds it to `edits`.
/// Why:
/// The text must be taken before cargo runs;
/// a manifest that cannot be read (or is not UTF-8) stops the run before anything is mutated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function snapshotManifests(crateDir: string, paths: string[], edits: ManifestEdit[]): void  // throws when a file cannot be read
/// ```
pub fn snapshot_manifests(crate_dir: &str, paths: &[PathBuf], edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<()> {
    for path in paths {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
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
        let before: String = match observe(path)? {
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            Some(found) => found.text,
            // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
            // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Ok(value)  is  return value;
            // // Err(error) is  throw error;   (the caller sees it as a returned value)
            // ```
            // `None`: the present or the absent value of an `Option`.
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            None => return Err(io::Error::new(io::ErrorKind::NotFound, format!("{} does not exist", path.display()))),
        };
        // `None`: the present or the absent value of an `Option`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        edits.borrow_mut().push(ManifestEdit { crate_dir: crate_dir.to_string(), path: path.clone(), before, produced: None });
    }
    return Ok(());
}

/// What:
/// `pub fn run_recording_manifests<T>(crate_dir, edits, run) -> io::Result<T>` runs a child that may rewrite the manifests of one
/// crate root and records as its output each of those manifests that the evidence says it wrote.
/// Why:
/// Only the manifests of the root the child ran in are claimed,
/// so an edit to another root's member is never taken for this run's output.
/// The claim is made whether the child succeeded or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runRecordingManifests<T>(crateDir: string, edits: ManifestEdit[], run: () => T): T
/// ```
pub fn run_recording_manifests<T>(crate_dir: &str, edits: &RefCell<Vec<ManifestEdit>>, run: impl crate::file_owner::RecordedWork<T>) -> io::Result<T> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `priors: Vec<(usize, String)>`.
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
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut priors: Vec<(usize, String)> = Vec::new();
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for (position, edit) in edits.borrow().iter().enumerate() {
        if edit.crate_dir != crate_dir {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // Typed binding: `prior: String`.
        let prior: String = match &edit.produced {
            // `Some`: the present or the absent value of an `Option`.
            // `.clone()`: an independent owned copy.
            Some(known) => known.clone(),
            // `None`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&edit.path`: lends the value without copying or giving it away.
            None => match observe(&edit.path)? {
                // `Some`: the present or the absent value of an `Option`.
                Some(found) => found.text,
                // `None`: the present or the absent value of an `Option`.
                // `.clone()`: an independent owned copy.
                None => edit.before.clone(),
            },
        };
        priors.push((position, prior));
    }
    let (result, window): (T, Window) = timed(run);
    for (position, prior) in priors {
        // `.clone()`: an independent owned copy.
        // Typed binding: `path: PathBuf`.
        // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
        // Why:      Joined or stored paths are owned; paths passed in are borrowed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let path: PathBuf = edits.borrow()[position].path.clone();
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Borrow `&path`: lends the value without copying or giving it away.
        // Typed binding: `Claim: :Ours(text)`.
        if let Claim::Ours(text) = claim_output(&path, &prior, window)? {
            // `Some`: the present or the absent value of an `Option`.
            edits.borrow_mut()[position].produced = Some(text);
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(result);
}

/// What:
/// `pub fn restore_manifests(edits, keep) -> io::Result<Vec<String>>` puts each manifest back that still holds this run's output
/// and returns a warning for each one it left alone.
/// Why:
/// Unknown ownership never authorizes a restore:
/// a manifest with no recorded output,
/// or one that differs from it,
/// belongs to another process;
/// a manifest that cannot be read is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreManifests(edits: ManifestEdit[], keep: KeepAs): string[]  // throws when a read or write fails
/// ```
fn restore_manifests_into(edits: &RefCell<Vec<ManifestEdit>>, keep: &KeepAs, warnings: &mut Vec<String>) -> io::Result<()> {
    for edit in edits.borrow().iter() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&edit.path`: lends the value without copying or giving it away.
        // Typed binding: `live: String`.
        let live: String = match observe(&edit.path)? {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found.text,
            // `None`: the present or the absent value of an `Option`.
            None => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                warnings.push(format!("{} is missing; it was not restored", edit.path.display()));
                continue;
            }
        };
        if live == edit.before {
            continue;
        }
        match &edit.produced {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&edit.path`: lends the value without copying or giving it away.
            Some(produced) if *produced == live => match replace_keeping(&edit.path, produced, &edit.before, keep)? {
                // Review finding (3b 8): whoever wrote the text that is replaced, it is kept and named, never silently dropped.
                ReplaceOutcome::Replaced(kept) => warnings.push(format!(
                    "{} was put back to its text before the run; the text it held is kept as {}",
                    edit.path.display(),
                    // `match`: picks the arm whose pattern fits the value.
                    match kept {
                        // `Some`: the present or the absent value of an `Option`.
                        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                        Some(found) => found.display().to_string(),
                        // `None`: the present or the absent value of an `Option`.
                        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                        None => "nothing (it had vanished)".to_string(),
                    }
                )),
                ReplaceOutcome::LeftAlone => warnings.push(format!(
                    "{} was changed by another process while it was being restored; it was left as that process wrote it",
                    edit.path.display()
                )),
                // Macro `format!`: builds or prints text (or a list) from a template.
                ReplaceOutcome::Displaced(kept) => warnings.push(format!(
                    "{} was put back to its text before the run; the file the exchange displaced is kept as {}",
                    edit.path.display(),
                    kept.display()
                )),
                // Macro `format!`: builds or prints text (or a list) from a template.
                ReplaceOutcome::Undone(kept) => warnings.push(format!(
                    "{} was changed again during the restore; the undo put the file it first displaced back, and the file the undo displaced is kept as {}",
                    edit.path.display(),
                    kept.display()
                )),
            },
            _ => warnings.push(format!(
                "{} differs from its text before the run, but this run did not leave it that way, so another process did; it was left as it is, not restored",
                edit.path.display()
            )),
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `pub fn restore_manifests(edits, keep) -> (Vec<String>, Option<io::Error>)` puts the manifests back and returns every warning
/// it produced together with the first failure.
/// Why:
/// Review finding (4b 3):
/// the report of a manifest that was restored and kept must survive a later manifest's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreManifests(edits: ManifestEdit[], keep: KeepAs): { warnings: string[]; failure?: Error }
/// ```
pub fn restore_manifests(edits: &RefCell<Vec<ManifestEdit>>, keep: &KeepAs) -> (Vec<String>, Option<io::Error>) {
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut warnings: Vec<String> = Vec::new();
    // Borrow `&mut warnings`: lends the value without copying or giving it away.
    // Typed binding: `failure: Option<io::Error>`.
    let failure: Option<io::Error> = restore_manifests_into(edits, keep, &mut warnings).err();
    return (warnings, failure);
}
