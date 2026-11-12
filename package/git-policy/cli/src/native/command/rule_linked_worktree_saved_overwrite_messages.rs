//! What:
//!  The notice that names the copy of each entry a discarding form would overwrite,
//!  and the rejection when a copy could not be made.
//! Why:
//!  A person whose untracked file,
//!  ignored file or nested repository was moved or copied aside has only these words to find it:
//!  they name the entry,
//!  the place it is now,
//!  and the command that runs next,
//!  so nothing has to be guessed and nothing has to be looked for by hand.
//!  A rejection names what stopped the save,
//!  where the copies live,
//!  and every way forward.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // overwrittenSavedNotice([{ path: 'b.txt', savedTo: '/r/.git/.../discarded/2026-11-12T01-00-00/b.txt' }], 'other')
//! ```

/// The files a save made,
///  and what stopped one.
use crate::conflict_sides::{SavedCopy, SaveFailure};
/// The paths a message names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::Path;

/// What:
///  The plain-language notice for the entries a save kept,
///  one block per entry,
///  written before Git runs.
/// Why:
///   The place is a directory under the common Git directory,
///  which nobody finds by looking at the worktree;
///  `source` names what the command writes them over with,
///  so the person knows why their file is not where they left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overwrittenSavedNotice(copies: SavedCopy[], source: string): string;
/// ```
pub fn overwritten_saved_notice(copies: &[SavedCopy], source: &str) -> String {
    let mut text: String = String::new();
    for copy in copies {
        let Some(place) = &copy.saved_to else {
            text.push_str(
                format!(
                    "monochromatic-git-policy-cli: {} was gone when this command was about to write over it, so \
                     there was nothing to save.\n",
                    copy.path
                )
                .as_str(),
            );
            continue;
        };
        text.push_str(
            format!(
                "monochromatic-git-policy-cli: saved your copy of {} to\n  {}\n  because {source} writes over it, and Git \
                 did not put it there.\n",
                copy.path,
                place.display()
            )
            .as_str(),
        );
    }
    return text;
}

/// What:
///  Everything the rejection of one save names.
///  A `struct` is a record with named fields.
/// Why:
///   What stopped the save names whichever entry it reached,
///  and the person also has to know which entries the command would have written over,
///  where the copies live,
///  and how to run the command deliberately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotSaved = { failure: SaveFailure; savedRoot: string; requested: string[] };
/// ```
pub struct NotSaved<'a> {
    /// What stopped the save.
    pub failure: &'a SaveFailure,
    /// The directory the copies live under.
    pub saved_root: &'a Path,
    /// The entries the command would have written over,
    ///  as Git named them.
    pub requested: &'a [String],
}

/// What:
///  The rejection of a command whose copies could not all be made.
/// Why:
///   The command must not run while an entry it would overwrite has no copy,
///  and a person who has decided to lose it needs the one deliberate way through;
///  the text names the entries,
///  the place the copies go,
///  and the hatch,
///  in words that explain themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overwrittenNotSavedMessage(notSaved: NotSaved): string;
/// ```
pub fn overwritten_not_saved_message(not_saved: NotSaved<'_>) -> String {
    let mut named: Vec<String> = Vec::<String>::new();
    for path in not_saved.requested.iter().take(SHARED_PATHS_NAMED) {
        named.push(format!("`{path}`"));
    }
    let rest: usize = not_saved.requested.len().saturating_sub(named.len());
    let and_more: String = match rest {
        0 => String::new(),
        _ => format!(" and {rest} more"),
    };
    return format!(
        "monochromatic-git-policy-cli: this command would write over {}{and_more}, and a copy of every one of them could \
         not be made: {}. The copies go under {}, which the system would not write; the command was not run, so nothing \
         in your worktree changed. Fix what the message above names (a full disk, a permission, a directory that cannot \
         be written) and run the same command again. If you have decided to lose {} on purpose, pass \
         --no-enforce-worktree directly after the subcommand.",
        named.join(", "),
        describe(not_saved.failure),
        not_saved.saved_root.display(),
        named.join(", "),
    );
}

/// What:
///  One failure as a sentence a person can act on:
///  what the system refused,
///  and what it was reading or writing.
/// Why:
///  The save's failure carries the path it reached and the system's own words,
///  and a message needs both without a second copy of either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the system refused ${failure.subject}: ${failure.reason}`
/// ```
fn describe(failure: &SaveFailure) -> String {
    return format!("the system refused {}: {}", failure.subject, failure.reason);
}

/// The notice of a saved entry and the rejection of a save that failed.
#[cfg(test)]
#[path = "rule_linked_worktree_saved_overwrite_messages_tests.rs"]
mod tests;
