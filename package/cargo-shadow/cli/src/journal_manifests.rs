//! What:
//! The manifest side of a recovery:
//! the journaled manifests of a killed `--breaking` run,
//! the decision each root's lockfile
//! reached,
//! and putting back the manifests of the roots whose lockfile was restored.
//! Why:
//! `journal_recover.rs` holds the walk over journals and the lockfile decisions;
//! the manifest side was split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-manifests.ts
//! export function recoverManifests(manifests: JournaledManifest[], decisions: Map<string, RootDecision>, messages: string[]): boolean
//! ```

/// Imports `fs` to read the journaled manifests.
use std::fs;
/// Imports `io` for the file-system error type.
use std::io;
/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports the ownership evidence reader.
use crate::file_owner::observe;
/// Imports the manifest index name,
/// the kept-file naming and the replace outcome.
use crate::journal::{KeepAs, MANIFEST_INDEX_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, replace_keeping_with_hook};

/// One manifest recovered from a journal.
pub(crate) struct JournaledManifest {
    /// The crate root the manifest belongs to.
    pub(crate) crate_dir: String,
    /// The manifest's path,
    /// byte for byte.
    pub(crate) path: PathBuf,
    /// The manifest's text before the run.
    pub(crate) before: String,
}

/// What:
/// `fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>>` reads the manifest records of a journal.
/// `Ok` with an empty list means the transaction journaled no manifests (the file does not exist);
/// every other failure,
/// including a missing `end` line or a bad count,
/// is an error.
/// Why:
/// An unreadable index must keep the journal:
/// it is the only copy of the original manifests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestIndex(directory: string): JournaledManifest[]  // throws unless the record is complete
/// ```
pub(crate) fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    // Review finding (4b2 6): this is called only for a journal whose ready record promises manifests, so a missing index is
    // damage and never "no manifests".
    let text: String = fs::read_to_string(directory.join(MANIFEST_INDEX_FILE))?;
    // Macro `format!`: builds or prints text (or a list) from a template.
    /// What:
    /// `fn corrupt(why: &str) -> io::Error` is the error for a record that is damaged: "the manifest index is corrupt: " and the reason.
    /// Why:
    /// A function replaces the closure that every refusal of this reader called.
    fn corrupt(why: &str) -> io::Error {
        return io::Error::new(io::ErrorKind::InvalidData, format!("the manifest index is corrupt: {why}"));
    }
    // `let mut`: a binding that may change.
    let mut lines = text.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v1") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v1 header"));
    }
    // `let mut`: a binding that may change.
    // Typed binding: `found: Vec<JournaledManifest>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut found: Vec<JournaledManifest> = Vec::new();
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `ended: Option<usize>`.
    let mut ended: Option<usize> = None;
    for line in lines {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(count) = line.strip_prefix("end ") {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|_|`: an anonymous function, like a TS arrow function.
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            let Ok(end_count) = count.trim().parse() else {
                return Err(corrupt("the end count is not a number"));
            };
            ended = Some(end_count);
            continue;
        }
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        let Some((crate_dir, encoded)) = line.split_once('\t') else {
            return Err(corrupt("a line has no tab"));
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        // Typed binding: `path: PathBuf`.
        let Some(path) = decode_path(encoded) else {
            return Err(corrupt("a path is not hexadecimal"));
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `before: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let before: String = fs::read_to_string(directory.join(format!("m{}.toml", found.len())))?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        found.push(JournaledManifest { crate_dir: crate_dir.to_string(), path, before });
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(found.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("the end count does not match the lines"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(found);
}

/// What one root's recovery decided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootDecision {
    /// The leftover lockfile passed the policy and was kept;
    /// the root's manifest edits stay.
    Kept,
    /// The snapshot is live (or nothing differed);
    /// the root's manifests go back.
    Restored,
    /// Another writer's file is live;
    /// the root's manifests are left alone too.
    LeftAlone,
    /// The root could not be settled (an error,
    /// a live transaction);
    /// nothing of the root is touched and the journal is kept.
    Failed,
}

/// What:
/// `pub(crate) fn recover_manifests(manifests, decisions, between, messages) -> bool` puts back the manifests of the roots whose
/// lockfile was restored.
/// Why:
/// A root with no decision,
/// or a failed one,
/// is never restored:
/// the pair lockfile and manifest is not split.
/// Returns whether every manifest was settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(manifests: JournaledManifest[], decisions: Map<string, RootDecision>, messages: string[]): boolean
/// ```
pub(crate) fn recover_manifests(manifests: &[JournaledManifest], decisions: &[(String, RootDecision)], between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    let mut all_settled: bool = true;
    for manifest in manifests {
        // Closure `|(crate_dir, _)|`: an anonymous function, like a TS arrow function.
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // Typed binding: `decision: RootDecision`.
        let mut decision: RootDecision = RootDecision::Failed;
        for (crate_dir, found) in decisions {
            if *crate_dir == manifest.crate_dir {
                decision = *found;
                break;
            }
        }
        if decision == RootDecision::Failed {
            all_settled = false;
            continue;
        }
        if decision != RootDecision::Restored {
            continue;
        }
        let shown: String = manifest.path.display().to_string();
        let live: String = match observe(&manifest.path) {
            Ok(Some(found)) => found.text,
            Ok(None) => {
                messages.push(format!("{shown} is missing; the manifest of a killed run was not put back"));
                continue;
            }
            Err(error) => {
                all_settled = false;
                messages.push(format!("{shown} could not be read ({error}); its journal was kept"));
                continue;
            }
        };
        if live == manifest.before {
            continue;
        }
        let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
        match replace_keeping_with_hook(&manifest.path, &live, &manifest.before, &keep, between) {
            Ok(ReplaceOutcome::Replaced(copy)) => messages.push(format!(
                "{shown} was rewritten by a run that was killed and its lockfile was not kept; the earlier manifest was put back and the rewritten text is kept as {}",
                match copy {
                    Some(found) => found.display().to_string(),
                    None => "nothing (it had vanished)".to_string(),
                }
            )),
            Ok(ReplaceOutcome::LeftAlone) => messages.push(format!("{shown} was changed by another process while a killed run was being recovered; it was left as that process wrote it")),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok(ReplaceOutcome::Displaced(kept)) => messages.push(format!("{shown} was put back to its text before the run; the file the exchange displaced is kept as {}", kept.display())),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok(ReplaceOutcome::Undone(kept)) => messages.push(format!("{shown} was changed again during the recovery; the undo put the file it first displaced back, and the file the undo displaced is kept as {}", kept.display())),
            Err(error) => {
                all_settled = false;
                messages.push(format!("{shown}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
}
