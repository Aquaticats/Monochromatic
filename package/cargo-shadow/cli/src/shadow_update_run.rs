//! What:
//! Runs one `cargo update` of either kind for the shadow,
//! and holds the stand-in lockfile that a full update writes while a crate root has none.
//! Why:
//! The command in `shadow_update.rs` decides which update to run and how to report a policy finding;
//! this file is the run itself and the lockfile it stands in with,
//! so each file is one thing a reader can follow.
//! `run_full_update` and `run_update` are here for the command to call,
//! and the stand-in lockfile is here because only a full update writes one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-update-run.ts
//! export function runFullUpdate(args): number
//! export interface PlaceholderBody { run(): number }
//! export function runWithPlaceholderLock(lockPath: string, keepDir: string, dryRun: boolean, body: PlaceholderBody): number
//! export function runWithPlaceholderLockHooked(lockPath: string, keepDir: string, dryRun: boolean, body: PlaceholderBody, beforeCleanup: (path: string) => void): number
//! ```

/// What:
/// `use rustix::fs::{CWD, RenameFlags, renameat_with};` imports the exclusive rename the stand-in is taken with.
/// Why:
/// `remove_placeholder` takes the stand-in away from its path under a name no other writer can have,
/// and only this call does that without replacing a file that appeared meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { renameatWith, CWD, RenameFlags } from 'rustix/fs';
/// ```
use rustix::fs::{
    CWD,
    RenameFlags,
    renameat_with,
};
/// What:
/// `use rustix::io::Errno;` imports the error code of a name that exists or a path that does not.
/// Why:
/// `remove_placeholder` reads those two answers of the exclusive rename as "try another name" and "nothing left to remove".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Errno } from 'rustix/io';
/// ```
use rustix::io::Errno;
/// What:
/// `use std::io::Write;` imports the `Write` trait, the borrowed writer the update reports into.
/// Why:
/// `run_full_update` and `run_update` take a `&mut dyn Write` sink, which a caller passes as standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
/// What:
/// `use std::path::Path;` imports the borrowed view of a file path (the sibling `PathBuf` owns one).
/// Why:
/// The lockfile and the kept folder are lent to the stand-in helpers, never stored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use crate::commands::{print_findings, print_update_report};` imports the printer of an update's report and of its findings.
/// Why:
/// `run_update` turns one update outcome into text and an exit code with exactly these two printers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { printFindings, printUpdateReport } from 'crate/commands';
/// ```
use crate::commands::{
    print_findings,
    print_update_report,
};
/// What:
/// `use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME};` imports the catalog marker, the failure code and the lockfile name.
/// Why:
/// `run_update` names the catalog marker and returns the failure code;
/// `run_full_update` names the lockfile a crate root may be missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::{
    CATALOG_MARKER,
    INSTALL_FAILED_EXIT_CODE,
    LOCKFILE_NAME,
};
/// What:
/// `use crate::shadow_step::{Prepared, now_millis};` imports the prepared catalog and the clock reading.
/// Why:
/// A full update works on the catalog that `prepare` read, and the update stamps its record with the current time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Prepared, nowMillis } from 'crate/shadow_step';
/// ```
use crate::shadow_step::{
    Prepared,
    now_millis,
};
/// What:
/// `use crate::tighten::FileEnforcer;` imports the syncer an update report is written through.
/// Why:
/// `run_update` hands it to the update so the record it writes is forced to disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FileEnforcer } from 'crate/tighten';
/// ```
use crate::tighten::FileEnforcer;
/// What:
/// `use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};` imports the update call and its types.
/// Why:
/// `run_update` builds the request, runs it, and prints the report it returns;
/// `UpdateKind` says whether it is the full update or a targeted one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateKind, UpdateReport, UpdateRequest, updateRustLockfiles } from 'crate/update';
/// ```
use crate::update::{
    UpdateKind,
    UpdateReport,
    UpdateRequest,
    update_rust_lockfiles,
};

/// The lockfile text that stands in for a missing lockfile while a full update creates it.
const EMPTY_LOCK: &str = "version = 4\n";

/// What:
/// `pub(crate) fn run_full_update(prepared, crate_dirs, dry_run, forward, sink) -> i32` runs the full catalog update for one crate root.
/// Why:
/// When no lockfile exists a stand-in is written so the transaction has a snapshot,
/// and it is removed again if the update is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runFullUpdate(args): number
/// ```
pub(crate) fn run_full_update(
    prepared: &Prepared,
    crate_dirs: &[String],
    dry_run: bool,
    forward: &[String],
    sink: &mut dyn Write,
) -> i32 {
    let lock_path = prepared
        .catalog_root
        .join(&crate_dirs[0])
        .join(LOCKFILE_NAME);
    // Borrow `&lock_path`: lends the value without copying or giving it away.
    return run_with_placeholder_lock(
        &lock_path,
        &prepared.catalog_root.join(crate::journal::QUARANTINE_DIR),
        dry_run,
        &mut FullUpdate {
            prepared,
            crate_dirs,
            dry_run,
            forward,
            sink,
        },
    );
}

/// What:
/// `pub trait PlaceholderBody` is the work that runs while a stand-in lockfile may be in place:
/// `run(&mut self)` does it and returns the exit status.
/// Why:
/// `run_with_placeholder_lock` takes the work as a value instead of a closure,
/// so the work can be a struct that holds what it needs.
/// Every `FnMut() -> i32` is a `PlaceholderBody` too,
/// so a test passes a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface PlaceholderBody { run(): number; }
/// ```
pub trait PlaceholderBody {
    /// What:
    /// `run(&mut self)` does the work once.
    /// Why:
    /// The exit status decides whether the stand-in is kept or removed.
    fn run(&mut self) -> i32;
}

/// What:
/// The implementation of `PlaceholderBody` for every function that takes nothing and returns an exit status.
/// Why:
/// Tests and callers that already hold a closure need no wrapper.
impl<F: FnMut() -> i32> PlaceholderBody for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the work.
    fn run(&mut self) -> i32 {
        return self();
    }
}

/// What:
/// `struct FullUpdate<'a>` is the work of one full update of the crate roots.
/// Why:
/// It holds what the closure that stood in `run_full_update` captured.
struct FullUpdate<'a> {
    /// The prepared catalog and its settings.
    prepared: &'a Prepared,
    /// The crate roots to update.
    crate_dirs: &'a [String],
    /// Whether the update is a dry run.
    dry_run: bool,
    /// The arguments forwarded to cargo.
    forward: &'a [String],
    /// Where the update writes its report.
    sink: &'a mut dyn Write,
}

/// What:
/// The implementation of `PlaceholderBody` for `FullUpdate`.
/// Why:
/// `run_with_placeholder_lock` runs it with the stand-in lockfile in place.
impl PlaceholderBody for FullUpdate<'_> {
    /// What:
    /// Runs the full update.
    /// Why:
    /// The exit status is the update's.
    fn run(&mut self) -> i32 {
        return run_update(
            self.prepared,
            self.crate_dirs,
            self.dry_run,
            &UpdateKind::Full,
            self.forward,
            self.sink,
        );
    }
}

/// What:
/// `pub fn run_with_placeholder_lock(lock_path, dry_run, body) -> i32` runs `body` with a stand-in lockfile in place when the
/// crate root has none,
/// and removes the stand-in again if the update is refused or was a dry run.
/// Why:
/// The transaction needs a lockfile to snapshot;
/// a root without one is created by the update itself.
/// The stand-in is created exclusively (`create_new`),
/// so a lockfile another writer published first is never truncated.
/// At the end it is taken by an atomic rename to a unique name and only then read:
/// a stand-in is deleted,
/// anything else is linked back to the path (or kept under the taken name when the path was
/// taken again),
/// so a lockfile another writer published during the update is never deleted.
/// The path is briefly empty between the take and the link back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runWithPlaceholderLock(lockPath: string, dryRun: boolean, body: () => number): number
/// ```
pub fn run_with_placeholder_lock(
    lock_path: &Path,
    keep_dir: &Path,
    dry_run: bool,
    body: &mut dyn PlaceholderBody,
) -> i32 {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    return run_with_placeholder_lock_hooked(
        lock_path,
        keep_dir,
        dry_run,
        body,
        &crate::journal_replace::no_hook,
    );
}

/// What:
/// `fn remove_placeholder(lock_path: &Path, keep_dir: &Path)` takes the stand-in lockfile away from its path without ever
/// unlinking an inode that something may still have open.
/// Why:
/// Review finding (3c P1-3):
/// reading the taken file and then unlinking it still loses the data of a writer that opened the
/// stand-in earlier and writes after the read.
/// The file is taken by an exclusive rename to a unique name (an existing name is never overwritten),
/// read,
/// and then:
/// a stand-in is given a surviving name in `keep_dir` before the taken name is removed;
/// anything else is linked back to the path (a link never replaces something published meanwhile),
/// or stays under its taken
/// name,
/// reported on stderr.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removePlaceholder(lockPath: string, keepDir: string): void  // rename (no replace), read, keep, then unlink the taken name
/// ```
fn remove_placeholder(
    lock_path: &Path,
    keep_dir: &Path,
) {
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `taken: Option<std::path::PathBuf>`.
    let mut taken_name: Option<std::path::PathBuf> = None;
    for _attempt in 0..MAX_TAKEN_NAMES {
        // `let mut`: a binding that may change.
        // Typed binding: `name: std::ffi::OsString`.
        // What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
        // Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let mut name: std::ffi::OsString = lock_path.as_os_str().to_os_string();
        // Macro `format!`: builds or prints text (or a list) from a template.
        name.push(format!(".taken.{}", crate::journal::unique_suffix()));
        // Typed binding: `candidate: std::path::PathBuf`.
        let candidate: std::path::PathBuf = std::path::PathBuf::from(name);
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&candidate`: lends the value without copying or giving it away.
        match renameat_with(CWD, lock_path, CWD, &candidate, RenameFlags::NOREPLACE) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(()) => {
                // `Some`: the present or the absent value of an `Option`.
                taken_name = Some(candidate);
                break;
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(code) if code == Errno::EXIST => continue,
            // Nothing is at the path any more: nothing to remove.
            Err(code) if code == Errno::NOENT => return,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(code) => {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!(
                    "cargo-shadow: warning: the stand-in {} could not be taken for removal: {code}",
                    lock_path.display()
                );
                return;
            }
        }
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    let Some(taken) = taken_name else {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: warning: no free name to take the stand-in {} under; it was left in place",
            lock_path.display()
        );
        return;
    };
    let is_placeholder: bool = match crate::file_owner::observe(&taken) {
        Ok(Some(seen)) => seen.text == EMPTY_LOCK,
        Ok(None) => return,
        Err(error) => {
            eprintln!(
                "cargo-shadow: warning: {} could not be read, so it is kept: {error}",
                taken.display()
            );
            false
        }
    };
    // One rule for both branches (review finding 4c F3): the taken inode gets a surviving name in the kept directory before its
    // taken name is removed; linking it back to the live path is not a preservation name, because the path is writable by others.
    let keep: crate::journal::KeepAs = crate::journal::KeepAs::in_directory(
        keep_dir,
        if is_placeholder {
            ".placeholder"
        } else {
            ".lockfile"
        },
    );
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&taken`: lends the value without copying or giving it away.
    let kept = match crate::journal_replace::keep_name(&taken, lock_path, &keep) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!(
                "cargo-shadow: warning: the lockfile taken from {} is kept as {} because it could not be given a kept name ({error})",
                lock_path.display(),
                taken.display()
            );
            if !is_placeholder {
                // Borrow `&taken`: lends the value without copying or giving it away.
                // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
                // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // void expr;   // or try { expr } catch {}
                // ```
                let _ = std::fs::hard_link(&taken, lock_path);
            }
            return;
        }
    };
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&taken`: lends the value without copying or giving it away.
    if !is_placeholder && let Err(error) = std::fs::hard_link(&taken, lock_path) {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: warning: a lockfile written during the run is kept as {} ({error})",
            kept.display()
        );
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&taken`: lends the value without copying or giving it away.
    if let Err(error) = std::fs::remove_file(&taken) {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: warning: {} could not be removed after it was kept as {}: {error}",
            taken.display(),
            kept.display()
        );
    }
}

/// The most names tried to take the stand-in under.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// Like the other name searches:
/// one try is the normal case and the collision count was not counted.
const MAX_TAKEN_NAMES: u32 = 100;

/// What:
/// `pub fn run_with_placeholder_lock_hooked(lock_path, dry_run, body, before_cleanup) -> i32` is `run_with_placeholder_lock` with a
/// hook that runs just before the cleanup takes the file.
/// Why:
/// That gap is where another writer can publish a real lockfile;
/// the hook lets a test publish one exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runWithPlaceholderLockHooked(lockPath: string, dryRun: boolean, body: () => number, beforeCleanup: (path: string) => void): number
/// ```
pub fn run_with_placeholder_lock_hooked(
    lock_path: &Path,
    keep_dir: &Path,
    dry_run: bool,
    body: &mut dyn PlaceholderBody,
    before_cleanup: &dyn Fn(&Path),
) -> i32 {
    // `create_new(true)` fails with `AlreadyExists` instead of truncating (the exclusive-create flag `wx` of Node), so the
    // check and the creation are one step another writer cannot slip into.
    let created_placeholder: bool = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(mut file) => file.write_all(EMPTY_LOCK.as_bytes()).is_ok(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => false,
    };
    // Typed binding: `code: i32`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let code: i32 = body.run();
    if created_placeholder && (code != 0 || dry_run) {
        before_cleanup(lock_path);
        remove_placeholder(lock_path, keep_dir);
    }
    return code;
}

/// What:
/// `pub(crate) fn run_update(prepared, crate_dirs, dry_run, kind, forward, sink) -> i32` runs an update of either kind and prints its report.
/// Why:
/// One place turns an update outcome into text and an exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runUpdate(args): number
/// ```
pub(crate) fn run_update(
    prepared: &Prepared,
    crate_dirs: &[String],
    dry_run: bool,
    kind: &UpdateKind,
    forward: &[String],
    sink: &mut dyn Write,
) -> i32 {
    let catalog_path = prepared.catalog_root.join(CATALOG_MARKER);
    let outcome = update_rust_lockfiles(
        // Borrow `&UpdateRequest`: lends the value without copying or giving it away.
        &UpdateRequest {
            // Borrow `&prepared.catalog_root`: lends the value without copying or giving it away.
            root: &prepared.catalog_root,
            crate_dirs,
            dry_run,
            now: now_millis(),
            // Borrow `&prepared.document.settings`: lends the value without copying or giving it away.
            settings: &prepared.document.settings,
            // Borrow `&catalog_path`: lends the value without copying or giving it away.
            catalog_path: &catalog_path,
            // Borrow `&prepared.runner`: lends the value without copying or giving it away.
            runner: &prepared.runner,
            // Borrow `&prepared.registry`: lends the value without copying or giving it away.
            registry: &prepared.registry,
            forward,
            // Borrow `&FileEnforcer`: lends the value without copying or giving it away.
            syncer: &FileEnforcer,
            prune_catalog: false,
            lock_base: prepared.lock_base.as_deref(),
        },
        kind,
    );
    // `match`: picks the arm whose pattern fits the value.
    return match outcome {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(report) => {
            // Typed binding: `finished: UpdateReport`.
            let finished: UpdateReport = report;
            // Borrow `&finished`: lends the value without copying or giving it away.
            print_update_report(&finished, dry_run, sink);
            if finished.problems.is_empty() {
                0
            } else {
                // Borrow `&finished.problems`: lends the value without copying or giving it away.
                print_findings(
                    "Update refused; every Cargo.lock was restored",
                    &finished.problems,
                );
                INSTALL_FAILED_EXIT_CODE
            }
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("{error}");
            INSTALL_FAILED_EXIT_CODE
        }
    };
}
