//! What:
//!  The sentences a pull prints about the untracked files its branch move would have overwritten:
//!  where each copy is,
//!  what nothing was copied from,
//!  and the whole failure a person reads,
//!  which keeps Git's own recovery words and adds what Git cannot say.
//! Why:
//!  A person who follows Git's words runs a reset that deletes the very file that made the pull stop,
//!  so the wrapper copies those files first and names every copy here.
//!  The worktree guard's hatch appears exactly once,
//!  in the sentence that offers the reset,
//!  and only when nothing the move would overwrite was left uncopied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = recoveryWords(origHead, inTheWay); // Git's words + the guard's line + where every copy is
//! ```

use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use crate::path_bytes::path_bytes_of;
use crate::pull_untracked_paths_in_the_way_kept::{
    KeptPath,
    PathLeftInTheWay,
    UntrackedInTheWay,
};
use crate::shell_quote::shell_quote;
use std::path::Path;
/// How many paths one sentence names before it says how many the folder holds besides.
/// Five is the bound the wrapper's other messages use for a list of paths.
const NAMED_PATHS_LIMIT: usize = 5;

/// What:
///  A path as the shell reads it back,
///  between the backticks a message puts around a command word.
/// Why:
///   The sentence names paths a person copies back with their shell,
///  so a space or a quote in one is quoted rather than left to split the word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `\`${shellQuote(path)}\``
/// ```
fn quoted(path: &Path) -> String {
    return format!("`{}`", shell_quote(path_bytes_of(path)));
}

/// What:
///  The sentence that says where the copies of the kept files are.
/// Why:
///   The person has to reach the copies,
///  so each kept path is named with the copy beside the Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The wrapper copied ${kept.map((one) => `${one.path} to ${one.copy}`).join(', ')} first`
/// ```
fn kept_sentence(
    kept: &[KeptPath],
    folder: Option<&Path>,
) -> String {
    let mut items: Vec<String> = Vec::new();
    for entry in kept.iter().take(NAMED_PATHS_LIMIT) {
        items.push(format!(
            "{} to {}",
            quoted(entry.in_the_worktree.as_path()),
            quoted(entry.in_the_copy.as_path())
        ));
    }
    let mut words: String = format!("The wrapper copied {} first", items.join(", "));
    if kept.len() > NAMED_PATHS_LIMIT {
        let beside: String = match folder {
            Some(made) => format!(" under {}", quoted(made)),
            None => String::new(),
        };
        words.push_str(format!(", and {} more{beside}", kept.len() - NAMED_PATHS_LIMIT).as_str());
    }
    return format!(
        "{words}; copy them back from there when you want them, and delete those copies when you do not.\n"
    );
}

/// What:
///  The sentence that names what nothing was copied from and says to move it first.
/// Why:
///   The reset deletes what stands there,
///  and the hatch is offered in this sentence and not before it,
///  so a reader who follows the words in order moves the path before the reset runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `nothing was copied from ${items}, and git diff ${orig} cannot hold it because Git does not track it; ...`
/// ```
fn left_sentence(
    left: &[PathLeftInTheWay],
    orig_head: &str,
) -> String {
    let mut items: Vec<String> = Vec::new();
    for entry in left.iter().take(NAMED_PATHS_LIMIT) {
        items.push(format!(
            "{} ({})",
            quoted(entry.in_the_worktree.as_path()),
            entry.why
        ));
    }
    let mut words: String = format!(
        "monochromatic-git-policy-cli: nothing was copied from {}",
        items.join(", ")
    );
    if left.len() > NAMED_PATHS_LIMIT {
        words.push_str(format!(" and {} more", left.len() - NAMED_PATHS_LIMIT).as_str());
    }
    words.push_str(format!(", and `git diff {orig_head}` cannot hold what stands there because Git does not track it; the reset deletes it. Move what stands there out of the worktree first, then run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard` if you mean it.\n").as_str());
    return words;
}

/// What:
///  The sentence for a listing that could not run,
///  which keeps nothing and offers the reset only after the untracked files are saved.
/// Why:
///   The wrapper cannot say the reset keeps nothing of the person's,
///  so it says what it does not know and what to do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the files this move would overwrite could not be listed (${why}), so nothing could be kept. Save what Git does not track (git ls-files --others) before ...`
/// ```
fn unlisted_sentence(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the files this move would overwrite could not be listed ({why}), so nothing could be kept; save what Git does not track (`git ls-files --others`) before you run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard`.\n"
    );
}

/// What:
///  The whole failure a pull prints when its move could not run:
///  Git's own recovery words,
///  the wrapper's line about the worktree guard,
///  and what it kept.
/// Why:
///   Git's words stay as Git prints them (`SPEC.md`,
///  "Pull");
///  the wrapper adds what Git cannot say,
///  that its guard refuses the reset,
///  and that the untracked files Git's diff omits are kept.
///  The hatch appears exactly once:
///  in the sentence that offers the reset,
///  which is the guard's own sentence only when nothing the move would overwrite was left
///  uncopied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitsRecoveryWords(orig) + guardSentence(inTheWay) + (kept.length ? keptSentence(kept) : '') + (left.length ? leftSentence(left) : '') + (listingFailure ? unlistedSentence(listingFailure) : '');
/// ```
pub fn recovery_words(
    orig_head: &str,
    in_the_way: &UntrackedInTheWay,
) -> String {
    let mut words: String = format!(
        "fatal: Cannot fast-forward your working tree.\nAfter making sure that you saved anything precious from\n$ git diff {orig_head}\noutput, run\n$ git reset --hard\nto recover.\nmonochromatic-git-policy-cli: that reset discards whatever this worktree holds, and the wrapper's worktree guard refuses a hard reset here without {WORKTREE_ENFORCEMENT_ESCAPE_HATCH}, which is what made this fast-forward fail"
    );
    let offering: bool = in_the_way.listing_failure.is_none() && in_the_way.left.is_empty();
    if offering {
        words.push_str(
            format!(
                "; run `git reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard` if you mean it.\n"
            )
            .as_str(),
        );
    } else {
        words.push_str(".\n");
    }
    if !in_the_way.kept.is_empty() {
        words.push_str("monochromatic-git-policy-cli: the move would have overwritten files Git does not track, which `git diff ");
        words.push_str(orig_head);
        words.push_str("` does not hold. ");
        words.push_str(
            kept_sentence(in_the_way.kept.as_slice(), in_the_way.folder.as_deref()).as_str(),
        );
    }
    if !in_the_way.left.is_empty() {
        words.push_str(left_sentence(in_the_way.left.as_slice(), orig_head).as_str());
    }
    if let Some(why) = &in_the_way.listing_failure {
        words.push_str(unlisted_sentence(why).as_str());
    }
    return words;
}
