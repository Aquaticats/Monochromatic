//! What:
//!  Which forms of `git commit` real Git refuses before it makes any commit,
//!  decided from the options the person wrote.
//! Why:
//!  Native preparation removes the tokens Git validates before it commits:
//!  `-a` goes,
//!  the selection controls and the pathspecs already reached the private index,
//!  so a mistyped command reaches the private `git commit` as an ordinary one and
//!  makes a commit real Git would have refused
//!  (finding 8 of the audit of 0.6.1-alpha.14,
//!  and the review of the commit-argument fixes,
//!  which found `-i` with `-o`).
//!  The owner's answer of 2026-11-12 is "Refuse as Git does":
//!  every form real Git refuses is refused with Git's own message
//!  (`doc/handover/cli-git-rust-implementation.md`).
//!  So this file only decides which form is one Git refuses,
//!  and never words the refusal:
//!  Git is asked for its own words
//!  (`commit_form_refusal_from_git.rs`),
//!  which stays true when a release rewords a message.
//!  Every rule below is one Git applies before it writes an object or moves a
//!  reference,
//!  and each is stated as exactly as Git states it,
//!  so a command that is fine asks Git nothing:
//!  reading the parsed region costs no process and no file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const form = formGitRefuses(region); // 'selection-conflict' | 'reword-with-path' | undefined
//! ```

/// The commit options Git's table read,
/// as final states after every option in order.
use crate::command_commit::{CommitRegion, FixupKind};

/// One form of `git commit` real Git refuses before it makes a commit.
/// `#[derive(...)]` generates copying,
/// comparison and debug printing.
/// Why:
///   The name is what a debug line and a test say the wrapper asked Git about,
/// and the shape alone never decides the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitRefusedForm = 'selection-conflict' | 'reword-with-path' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitRefusedForm {
    /// Two or more of `-i/--include`,
    ///  `-o/--only`,
    ///  `-a/--all` and `--interactive/-p/--patch`.
    SelectionConflict,
    /// `--fixup=reword:` with a pathspec.
    RewordWithPath,
    /// `--fixup=reword:` with a selection option.
    RewordWithSelection,
    /// `--pathspec-from-file` with pathspec arguments.
    PathspecFromFileWithPaths,
    /// `--pathspec-from-file` with `-a/--all`.
    PathspecFromFileWithAll,
    /// `--pathspec-from-file` with `--interactive/-p/--patch`.
    PathspecFromFileWithInteractive,
    /// `-i/--include` or `-o/--only` with no paths at all.
    NoPathsWithSelection,
    /// `-a/--all` with pathspec arguments.
    AllWithPaths,
    /// `--pathspec-file-nul` without `--pathspec-from-file`.
    PathspecFileNulWithoutFromFile,
}

/// The name of each form,
/// for a debug line and for the test that names the row it covers.
impl GitRefusedForm {
    /// What:
    ///  The stable name of this form.
    /// Why:
    ///   A debug line and a test name the form without naming a message,
    /// which Git owns.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// form.name() // 'selection-conflict'
    /// ```
    pub const fn name(&self) -> &'static str {
        // `match` on `&self` reads the variant without taking it.
        match self {
            Self::SelectionConflict => "selection-conflict",
            Self::RewordWithPath => "reword-with-path",
            Self::RewordWithSelection => "reword-with-selection",
            Self::PathspecFromFileWithPaths => "pathspec-from-file-with-paths",
            Self::PathspecFromFileWithAll => "pathspec-from-file-with-all",
            Self::PathspecFromFileWithInteractive => "pathspec-from-file-with-interactive",
            Self::NoPathsWithSelection => "no-paths-with-selection",
            Self::AllWithPaths => "all-with-paths",
            Self::PathspecFileNulWithoutFromFile => "pathspec-file-nul-without-from-file",
        }
    }
}

/// What:
///  How many of the four options Git's own compatibility check counts are on.
/// Why:
///   Git counts `--patch` as `--interactive`
/// (`if (patch_interactive) interactive = 1`),
///  and refuses two or more of the four whatever their spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectionCount = Number(include) + Number(only) + Number(all) + Number(interactive || patch);
/// ```
fn selection_count(region: &CommitRegion) -> usize {
    let mut count: usize = 0;
    if region.include {
        count += 1;
    }
    if region.only == Some(true) {
        count += 1;
    }
    if region.all {
        count += 1;
    }
    // Git's `interactive` is one option however it was spelled, so both spellings count once.
    if region.interactive || region.patch {
        count += 1;
    }
    return count;
}

/// What:
///  Whether the final `--fixup` value is the `reword:` form,
///  which turns on `--only` itself and refuses paths and every selection.
/// Why:
///   Git checks that form apart from every other,
/// with messages of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rewording = region.fixup === 'reword';
/// ```
fn is_rewording(region: &CommitRegion) -> bool {
    return region.fixup == Some(FixupKind::Reword);
}

/// What:
///  Whether a pathless `-i`/`-o` is one Git refuses,
/// from Git's own condition.
/// Why:
///   Git refuses it unless an empty commit was allowed or the commit amends
/// (`-o` with `--allow-empty`,
///  `-o` with `--amend`,
///  and `-o` with `--fixup=amend:`,
///  which allows an empty change too).
///  A pathspec file decides the count itself,
/// so with one the count is Git's to read and this file stays silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = !fromFile && !paths.length && (include || (only && !allowEmpty && (!amend || (fixup && !amendFixup))));
/// ```
fn no_paths_with_selection_is_refused(region: &CommitRegion) -> bool {
    if region.pathspec_from_file.is_some() || !region.pathspecs.is_empty() {
        return false;
    }
    if region.include {
        return true;
    }
    if region.only != Some(true) || region.allow_empty {
        return false;
    }
    if !region.amend {
        return true;
    }
    // `--fixup=amend:` allows an empty change, so its pathless `-o` is fine; `--fixup=reword:` does not, and Git's own
    //  `strcmp(fixup_prefix, "amend")` reads exactly that difference.
    return region.fixup.is_some() && region.fixup != Some(FixupKind::Amend);
}

/// What:
///  The form of `git commit` real Git refuses,
///  or nothing when Git would accept this command's options.
/// Why:
///   A form Git accepts must reach the transaction unchanged,
///  so every rule is as wide as Git's and no wider.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const form = formGitRefuses(region);
/// ```
pub fn form_git_refuses(region: &CommitRegion) -> Option<GitRefusedForm> {
    // Selection conflicts come first because Git's own parse refuses them before it reads anything else.
    if selection_count(region) > 1 {
        return Some(GitRefusedForm::SelectionConflict);
    }
    if is_rewording(region) {
        if !region.pathspecs.is_empty() {
            return Some(GitRefusedForm::RewordWithPath);
        }
        if selection_count(region) > 0 {
            return Some(GitRefusedForm::RewordWithSelection);
        }
    }
    if region.pathspec_from_file.is_some() {
        if !region.pathspecs.is_empty() {
            return Some(GitRefusedForm::PathspecFromFileWithPaths);
        }
        if region.all {
            return Some(GitRefusedForm::PathspecFromFileWithAll);
        }
        if region.interactive || region.patch {
            return Some(GitRefusedForm::PathspecFromFileWithInteractive);
        }
    }
    if region.all && !region.pathspecs.is_empty() {
        return Some(GitRefusedForm::AllWithPaths);
    }
    if region.pathspec_file_nul && region.pathspec_from_file.is_none() {
        return Some(GitRefusedForm::PathspecFileNulWithoutFromFile);
    }
    if no_paths_with_selection_is_refused(region) {
        return Some(GitRefusedForm::NoPathsWithSelection);
    }
    return None;
}

/// The forms,
/// the count of the options Git's own check reads,
/// and the pathless selection.
#[cfg(test)]
#[path = "rule_commit_region_git_refuses_tests.rs"]
mod tests;
