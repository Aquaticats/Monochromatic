//! What:
//! Finds the first-party crate roots that carry a `Cargo.lock`,
//! and snapshots and restores their lockfiles.
//! Why:
//! The catalog commands treat all lockfiles as one transaction:
//! any failure or refusal puts every lockfile back as it was.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // crates.ts
//! export async function lockedCrateDirs(root: string): Promise<string[]>
//! export async function readLocks({ root, crateDirs }): Promise<LockSnapshot[]>
//! export async function restoreLocks({ root, snapshots }): Promise<void>
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `collect_at_depth`,
/// `read_locks`,
/// `restore_locks`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports `io` from the standard library so this file can name it unqualified.
/// Why:
/// `io` is used by `collect_at_depth`,
/// `locked_crate_dirs`,
/// `read_locks`,
/// `restore_locks`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { io } from 'std';
/// ```
use std::io;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `collect_at_depth`,
/// `locked_crate_dirs`,
/// `read_locks`,
/// `restore_locks`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use crate::constants::{LOCKFILE_NAME, SKIPPED_DIRECTORIES};` imports `LOCKFILE_NAME`,
/// `SKIPPED_DIRECTORIES` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LOCKFILE_NAME` is used by `collect_at_depth`,
/// `read_locks`,
/// `restore_locks`;
/// `SKIPPED_DIRECTORIES` is used by `collect_at_depth`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME, SKIPPED_DIRECTORIES } from 'crate/constants';
/// ```
use crate::constants::{LOCKFILE_NAME, SKIPPED_DIRECTORIES};

/// How many directory levels under `package/` hold crate roots:
/// `package/<category>/<name>` and
/// `package/<category>/<name>/<sub>`,
/// the same two globs file-enforcer's Rust catalog sync uses.
const CRATE_ROOT_DEPTHS: [usize; 2] = [2, 3];

/// The top directory of the first-party packages.
const PACKAGE_DIRECTORY: &str = "package";

/// One crate's lockfile text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockSnapshot {
    /// Repo-relative crate directory,
    /// forward slashes.
    pub crate_dir: String,
    /// The lockfile's text.
    pub source: String,
}

/// What:
/// `impl LockSnapshot { ... }` attaches three read-only accessors to the snapshot struct (a TS class body).
/// Why:
/// Iterator steps name `LockSnapshot::source_text` instead of repeating a closure that reaches into the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LockSnapshot { sourceText(): string { return this.source; } }
/// ```
impl LockSnapshot {
    /// What:
    /// `pub fn source_text(&self) -> String` returns an owned copy of the lockfile text.
    /// Why:
    /// The lockfile diff and the journal take plain texts and keep them after the snapshots go out of scope.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sourceText(): string { return this.source; }
    /// ```
    pub fn source_text(&self) -> String {
        return self.source.clone();
    }

    /// What:
    /// `pub fn source_str(&self) -> &str` lends the lockfile text without copying it.
    /// Why:
    /// Reading one snapshot's text to parse it needs no ownership.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sourceStr(): string { return this.source; }
    /// ```
    pub fn source_str(&self) -> &str {
        return self.source.as_str();
    }

    /// What:
    /// `pub fn crate_dir_text(&self) -> String` returns an owned copy of the repo-relative crate directory.
    /// Why:
    /// Recovery builds the list of crate roots to re-read from the journal's snapshots.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// crateDirText(): string { return this.crateDir; }
    /// ```
    pub fn crate_dir_text(&self) -> String {
        return self.crate_dir.clone();
    }
}

/// Collects the repo-relative directories exactly `depth` levels below `base` that hold a lockfile.
fn collect_at_depth(root: &Path, relative: &str, depth: usize, found: &mut Vec<String>) -> io::Result<()> {
    if depth == 0 {
        if root.join(relative).join(LOCKFILE_NAME).is_file() {
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            found.push(relative.to_string());
        }
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        return Ok(());
    }
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    let entries = match fs::read_dir(root.join(relative)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(listing) => listing,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    };
    for listed in entries {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        let entry = listed?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
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
        let name: String = entry.file_name().to_string_lossy().to_string();
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        if SKIPPED_DIRECTORIES.contains(&name.as_str()) || !entry.file_type()?.is_dir() {
            continue;
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&format`: lends the value without copying or giving it away.
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        collect_at_depth(root, &format!("{relative}/{name}"), depth - 1, found)?;
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `pub fn locked_crate_dirs(root: &Path) -> io::Result<Vec<String>>` lists the repo-relative crate directories with a
/// `Cargo.lock`,
/// sorted.
/// Why:
/// Same discovery as the update tool's and file-enforcer's globs;
/// `target` and `node_modules` are never entered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function lockedCrateDirs(root: string): Promise<string[]>
/// ```
pub fn locked_crate_dirs(root: &Path) -> io::Result<Vec<String>> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `found: Vec<String>`.
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
    let mut found: Vec<String> = Vec::new();
    for depth in CRATE_ROOT_DEPTHS {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&mut found`: lends the value without copying or giving it away.
        collect_at_depth(root, PACKAGE_DIRECTORY, depth, &mut found)?;
    }
    found.sort();
    found.dedup();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(found);
}

/// Reads every listed crate's lockfile,
/// in order.
pub fn read_locks(root: &Path, crate_dirs: &[String]) -> io::Result<Vec<LockSnapshot>> {
    // `let mut`: a binding that may change.
    // Typed binding: `snapshots: Vec<LockSnapshot>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for crate_dir in crate_dirs {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(root.join(crate_dir).join(LOCKFILE_NAME))?;
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        snapshots.push(LockSnapshot { crate_dir: crate_dir.clone(), source });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(snapshots);
}

/// Writes the snapshots back,
/// so a refused or dry-run install leaves every lockfile as it was.
pub fn restore_locks(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    for snapshot in snapshots {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&snapshot.crate_dir`: lends the value without copying or giving it away.
        fs::write(root.join(&snapshot.crate_dir).join(LOCKFILE_NAME), &snapshot.source)?;
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}
