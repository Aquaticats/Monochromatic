//! What:
//!  The plain-language notices of a conflict-side command:
//! the saved copies,
//! and the clean paths of a refused line.
//! Why:
//!  The person is told where each copy is before Git runs,
//! and which paths to leave out when a line is refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! savedNotice(copies); cleanPathsNote(['b.rs'], 'merge');
//! ```

use crate::conflict_side_state::Operation;
use crate::conflict_sides::SavedCopy;
use std::ffi::OsString;

/// What:
///  The plain-language notice for the copies,
/// one block per path.
/// Why:
///   The person is told where each copy is before Git runs,
/// so a file they edited by hand can be found again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedNotice(copies: SavedCopy[]): string;
/// ```
pub fn saved_notice(copies: &[SavedCopy]) -> String {
    let mut text: String = String::new();
    for copy in copies {
        match &copy.saved_to {
            Some(place) => text.push_str(
                format!(
                    "monochromatic-git-policy-cli: saved your copy of {} to\n  {}\n  before this command replaces it ({} was conflicted).\n",
                    copy.path,
                    place.display(),
                    copy.path
                )
                .as_str(),
            ),
            None => text.push_str(
                format!(
                    "monochromatic-git-policy-cli: {} was conflicted and is deleted in your worktree, so there is no copy to save; this command brings a file back.\n",
                    copy.path
                )
                .as_str(),
            ),
        }
    }
    return text;
}

/// What:
///  The sentence that says which copies hold what stood when the wrapper opened the file,
///  because another program was writing it.
/// Why:
///   A copy of a file a writer is still adding to can never be the whole of it,
///  so the wrapper copies what the file held when it opened it and lets the command run;
///  a person who needs the later bytes must be told they are not in the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stillBeingWrittenNote(['a.rs']);
/// ```
pub fn still_being_written_note(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let named: String = paths.join(", ");
    let (verb, pronoun): (&str, &str) = if paths.len() == 1 {
        ("was still being written while the wrapper copied it", "it")
    } else {
        ("were still being written while the wrapper copied them", "them")
    };
    return format!(
        "monochromatic-git-policy-cli: {named} {verb}, so the copy holds what stood when the wrapper opened the file and nothing written \
         after that. This command runs now, and a write that lands after the copy is not saved. Keep {pronoun} another way if the later \
         bytes matter.\n"
    );
}

/// What:
///  The sentence a rejection gets when the command qualified for a conflict side but some paths were not conflicted.
/// Why:
///   The person is told what is allowed and which paths to leave out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanPathsNote(others: string[], operation: Operation): string;
/// ```
pub fn clean_paths_note(others: &[OsString], operation: Operation) -> String {
    let mut named: Vec<String> = Vec::new();
    for each in others {
        named.push(format!("`{}`", each.to_string_lossy()));
    }
    let word: &str = match operation {
        Operation::Merge => "merge",
        Operation::Rebase => "rebase",
        Operation::CherryPick => "cherry-pick",
        Operation::Revert => "revert",
    };
    return format!(
        " This {word} is in progress, and a command that names only conflicted files is let through after your copy is saved, \
         but these are not conflicted files: {}. Leave them out, or name each conflicted file itself (no directory, wildcard or \
         pathspec magic).",
        named.join(", ")
    );
}
