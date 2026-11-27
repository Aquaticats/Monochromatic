//! What:
//! The sentence that says a scope runs a newer repository-published tool than its lock pins,
//! with the remedy for that scope,
//! and the shell quoting of the tool id that sentence carries.
//! Why:
//! One text for `install`,
//! `list`,
//! `update`,
//! `install --lockfile-only`,
//! `bin` and every run of the tool,
//! so a person searching for it finds all of them,
//! and the sentence is pure text over a tool,
//! two versions and a lock path,
//! so its wording is stated once and its tests need no file system.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function shellWord(word: string): string
//! export function olderLockMessageFor(kind: 'repo' | 'global', toolId: string, locked: string, used: string, lockFile: string): string
//! export function olderLockMessage(toolId: string, locked: string, used: string, lockFile: string): string
//! export function isNewerThanLocked(used: string, locked: string): boolean
//! export function olderLockWarnings(scope: Scope, tools: LockedTool[], family?: OsFamily): string[]
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `winning_version` from `super::super::across_roots`.
use super::super::across_roots::winning_version;
/// Imports `applies_to_family` from `super::super::layout`.
use super::super::layout::applies_to_family;
/// Imports what this module uses from `super::super::model`.
use super::super::model::{
    LockedTool,
    OsFamily,
};
/// Imports what this module uses from `super::super::scope`.
use super::super::scope::{
    Scope,
    ScopeKind,
};
/// Imports `is_repository_published` from this module's parent.
use super::is_repository_published;

/// Whether a character may appear in a word that needs no quoting.
fn is_plain_word_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || "@%+=:,./_-".contains(character);
}

/// What:
/// `shell_word(word)` gives `word` as one word that a POSIX shell reads back unchanged:
/// bare when every character is an ASCII letter,
/// a digit or one of `@%+=:,./_-`,
/// otherwise inside single quotes,
/// with each single quote of the word written as `'\''` (close the quotes,
/// an escaped quote,
/// reopen them).
/// An empty word becomes `''`.
/// Why:
/// The older-lock hints print a command for the person to paste into a shell;
/// a tool id holding a space,
/// a quote,
/// a `;` or a `$` must still name exactly that tool and run nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function shellWord(word: string): string {
///   return /^[A-Za-z0-9@%+=:,.\/_-]+$/.test(word) ? word : `'${word.replaceAll("'", "'\\''")}'`;
/// }
/// ```
pub fn shell_word(word: &str) -> String {
    // What: `word.chars()` walks the text one Unicode character at a time;
    //  `.all(|character| { ... })` is true only when the closure `|character| { ... }` (TS: `(character) => { ... }`)
    //  returns true for every character.
    //  `is_ascii_alphanumeric` is true for `A-Z`, `a-z` and `0-9` only.
    // Why: such a word needs no quoting, and leaving it bare keeps the common hint (`cargo:monochromatic-...`) readable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plain = word !== '' && [...word].every((character) => /[A-Za-z0-9]/.test(character) || '@%+=:,./_-'.includes(character));
    // ```
    let plain: bool = !word.is_empty() && word.chars().all(is_plain_word_character);
    if plain {
        // What: `.to_string()` copies the borrowed text `&str` into a new owned `String` (TS strings need no such step).
        // Why: the function hands back text the caller keeps after `word` is gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return word;
        // ```
        return word.to_string();
    }
    // What: `word.replace('\'', "'\\''")` swaps every single quote (the char literal `'\''`) for the four characters
    //  `'\''`; `format!("'{}'", ...)` wraps the result in one more pair of single quotes.
    // Why: inside single quotes a shell treats every character literally except the closing quote itself,
    //  so only the quotes need this close-escape-reopen form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `'${word.replaceAll("'", "'\\''")}'`;
    // ```
    return format!("'{}'", word.replace('\'', "'\\''"));
}

/// What:
/// The sentence that says a scope runs a newer repository-published tool than its lock pins,
/// naming that lock file,
/// with the remedy for that scope as mpm's own pnpm-style verb naming the tool:
/// `mpm update <tool id>` for a tree's lock,
/// `mpm-shadow update -g <tool id>` for the machine-global lock,
/// the id written as one shell word (`shell_word`).
/// Why:
/// One text for `install`,
/// `list`,
/// `update`,
/// `install --lockfile-only`,
/// `bin` and every run of the tool,
/// so a person searching for it finds all of them.
/// The file is named because a launcher and a shell read whichever lock their own record names,
/// so a warning from another checkout's launcher looks stale until the file it read is visible.
/// The remedy names the tool so it moves only that row of the lock,
/// and names a program rather than a mise task because the repository is moving away from mise.
/// The machine-global sentence names `mpm-shadow`,
/// never `mpm`:
/// only a worktree's `PATH` makes `mpm` this tool (the plan's postinstall link),
/// and the machine-wide `mpm` is the real mpm,
/// which reads `update -g` as its own `upgrade` and refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockMessageFor(kind: 'repo' | 'global', toolId: string, locked: string, used: string, lockFile: string): string
/// ```
pub fn older_lock_message_for(
    kind: ScopeKind,
    tool_id: &str,
    locked: &str,
    used: &str,
    lock_file: &Path,
) -> String {
    // The tool id as one word the person can paste into a shell.
    let word: String = shell_word(tool_id);
    let lock: String = lock_file.display().to_string();
    return match kind {
        ScopeKind::Repo => format!(
            "{tool_id} is locked at {locked} in this tree's lock, {lock}, but {used} runs, because older repository tools are never kept \
             once a newer one is installed; upgrade this tree's lock with `mpm update {word}` or merge main"
        ),
        ScopeKind::Global => format!(
            "{tool_id} is locked at {locked} in this machine's global lock, {lock}, but {used} runs, because older repository tools are \
             never kept once a newer one is installed; upgrade the global lock with `mpm-shadow update -g {word}`"
        ),
    };
}

/// The older-lock sentence for a tree's lock (the repository scope).
pub fn older_lock_message(
    tool_id: &str,
    locked: &str,
    used: &str,
    lock_file: &Path,
) -> String {
    return older_lock_message_for(ScopeKind::Repo, tool_id, locked, used, lock_file);
}

/// Whether `used` is newer than `locked`.
pub fn is_newer_than_locked(
    used: &str,
    locked: &str,
) -> bool {
    return compare_versions(used, locked) == Ordering::Greater;
}

/// What:
/// A warning for every repository-published tool of this host whose installed version is newer than the lock's.
/// Why:
/// Commands that resolve tools for a tree report what will run in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockWarnings(scope: Scope, tools: LockedTool[], family?: OsFamily): string[]
/// ```
pub fn older_lock_warnings(
    scope: &Scope,
    tools: &[LockedTool],
    family: Option<OsFamily>,
) -> Vec<String> {
    let mut warnings: Vec<String> = Vec::new();
    for tool in tools {
        if !is_repository_published(tool) || !applies_to_family(&tool.declaration.os, family) {
            continue;
        }
        let Some(used) = winning_version(scope, tool) else {
            continue;
        };
        if is_newer_than_locked(&used, &tool.version) {
            warnings.push(older_lock_message_for(
                scope.kind,
                &tool.declaration.id,
                &tool.version,
                &used,
                &scope.lock_file,
            ));
        }
    }
    return warnings;
}
