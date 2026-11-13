//! What:
//!  The text `git monochromatic-git-policy-cli kept-references` prints:
//!  every kept reference the repository holds,
//!  its commit,
//!  how old it is,
//!  the worktree and batch it came from,
//!  and what the next pass will do with it.
//! Why:
//!  The pruning removes the only handle on a commit no branch reaches,
//!  so a person has to be able to see what is kept and what is about to go before it goes.
//!  The listing runs the same judgement the pass runs
//!  (`kept_reference_pass.rs`),
//!  with the removals left out,
//!  so what it says the next pass will take is what that pass takes,
//!  and a control asserts exactly that.
//!  A listing dates a reference no release recorded
//!  and keeps that date,
//!  as a pass does:
//!  the record is the one thing both write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(theListing(pass, limits));
//! ```

use crate::iso_time::format_iso_milliseconds;
use crate::kept_reference_limits::KeptReferenceLimits;
use crate::kept_reference_log::why_words;
use crate::kept_reference_pass_answers::{ThePass, WhyNotPruned};
use crate::kept_reference_words::{age_words, span_words};

/// The sentence a listing prints for a reference the next pass prunes,
/// which a control matches to know what that pass will take.
pub const PRUNED_NEXT: &str = "the next pass prunes it:";

/// The sentence a listing prints for a reference the next pass keeps.
pub const KEPT_NEXT: &str = "the next pass keeps it:";

/// What:
///  Why a reference the pass keeps is still there,
///  in the words a person reads.
/// Why:
///  The three rules that spare a reference are three different reasons,
///  and a person who wonders why a commit they gave up on is still held reads which one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptWords = (why, limits) => ({...})[why];
/// ```
pub fn kept_words(why: WhyNotPruned, limits: KeptReferenceLimits) -> String {
    return match why {
        WhyNotPruned::YoungerThanTheLimit => format!("it is younger than the age limit of {}", span_words(limits.max_age)),
        WhyNotPruned::UnderTheSizeLimit => format!("the objects only kept references reach weigh less than the {} bytes the size limit allows", limits.max_bytes),
        WhyNotPruned::ThisPassReachedItsCap => String::from("the pass that read it had reached the most removals one pass makes, so a later pass takes it"),
        WhyNotPruned::TheRemovalWasRefused => String::from("this pass planned to remove it and Git refused, because the reference moved or another pass removed it first"),
    };
}

/// What:
///  The worktree a kept reference came from,
///  in the words the record allows.
/// Why:
///   A reference written by a release that recorded no worktree has none,
///  and saying so is what keeps a person from believing the wrapper knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeWords = (worktree) => worktree ?? 'not recorded';
/// ```
pub fn worktree_words(worktree: Option<&str>) -> String {
    return match worktree {
        Some(path) => String::from(path),
        None => String::from(NOT_RECORDED),
    };
}

/// What a listing says about a reference no release recorded a worktree for.
pub const NOT_RECORDED: &str = "not recorded";

/// What:
///  The lines of one kept reference.
/// Why:
///  Every field a person needs to act is one line,
///  so a listing of a repository with many references stays readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesOf = (one, pruned, limits) => [name, `  commit ...`, ..., verdict];
/// ```
fn lines_of_one(reference: &str, commit: &str, rescued: bool, kept_at_ms: u128, age_ms: u128, batch: u64, worktree: Option<&str>, verdict: String) -> String {
    let own: &str = if rescued { ", the name beside the batch's own" } else { "" };
    return format!(
        "{reference}\n  commit {commit}\n  kept {} ({} ago), batch {batch}{own}\n  worktree {}\n  {verdict}\n",
        format_iso_milliseconds(i64::try_from(kept_at_ms).unwrap_or(i64::MAX)),
        age_words(age_ms),
        worktree_words(worktree)
    );
}

/// What:
///  The paragraph the listing opens with:
///  how many references the repository holds,
///  what the objects only they reach weigh,
///  and the three limits they are judged by.
/// Why:
///  The figures and the limits are what a person compares,
///  so they come before the references they are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const summary = (pass, limits) => `${pass.kept.length + pass.pruned.length} kept references ...`;
/// ```
pub fn summary_of(pass: &ThePass, limits: KeptReferenceLimits) -> String {
    let count: usize = pass.kept.len() + pass.pruned.len();
    let references: String = if count == 1 { String::from("1 kept reference") } else { format!("{count} kept references") };
    return format!(
        "{references} hold objects only they reach, {bytes} bytes on disk.\nThe limits are {age} of age and {max_bytes} bytes, and a reference younger than {floor} is never pruned for its size.\n",
        bytes = pass.bytes_before,
        age = span_words(limits.max_age),
        max_bytes = limits.max_bytes,
        floor = span_words(limits.floor)
    );
}

/// What:
///  The whole listing:
///  the summary,
///  one block per kept reference with what the next pass does,
///  and the line that says what that pass would leave.
/// Why:
///  A person reads the blocks to find the commit they want,
///  and the closing line to know how much the pruning would give back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theListing = (pass, limits) => summary + blocks + closing;
/// ```
pub fn the_listing(pass: &ThePass, limits: KeptReferenceLimits) -> String {
    let mut text: String = summary_of(pass, limits);
    for one in &pass.kept {
        text.push_str(lines_of_one(
            one.reference.as_str(),
            one.commit.as_str(),
            one.rescued,
            one.kept_at_ms,
            one.age_ms,
            one.batch,
            one.worktree.as_deref(),
            format!("{KEPT_NEXT} {}", kept_words(one.why_not, limits)),
        ).as_str());
    }
    for one in &pass.pruned {
        let kept_at_ms: u128 = one.kept_at_ms;
        text.push_str(
            lines_of_one(
                one.reference.as_str(),
                one.commit.as_str(),
                one.rescued,
                kept_at_ms,
                one.age_ms,
                one.batch,
                one.worktree.as_deref(),
                format!("{PRUNED_NEXT} {}", why_words(one.why, limits)),
            )
            .as_str(),
        );
    }
    text.push_str(closing_of(pass).as_str());
    return text;
}

/// What:
///  The line that says how many references the next pass would prune and what would be left.
/// Why:
///  It is the answer to the question the command exists for,
///  and it is the line a person reads when they only want the figure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const closing = (pass) => pass.pruned.length === 0 ? `The next pass prunes nothing; ${pass.bytesBefore} bytes stay.\n` : `The next pass prunes ${n}, leaving ${pass.bytesAfter} bytes.\n`;
/// ```
pub fn closing_of(pass: &ThePass) -> String {
    let count: usize = pass.pruned.len();
    if count == 0 {
        return format!("The next pass prunes nothing, so the objects only kept references reach stay at {} bytes.\n", pass.bytes_before);
    }
    let references: String = if count == 1 { String::from("1 of them") } else { format!("{count} of them") };
    let cap: &str = if pass.stopped_early {
        " It stopped at the most removals one pass makes; a later pass takes the rest."
    } else {
        ""
    };
    return format!("The next pass prunes {references}, leaving {} bytes.{cap}\n", pass.bytes_after);
}

/// What:
///  The lines `--prune` prints about one pass that has run:
///  each reference that went,
///  with its commit and why,
///  and what the pass left.
/// Why:
///  A person who asked for the pass needs to know what it took and where the record is,
///  and a reference Git refused to remove is said as it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pruningReport = (pass, limits, logPath) => lines + closing;
/// ```
pub fn the_pruning_report(pass: &ThePass, limits: KeptReferenceLimits, log: &std::path::Path) -> String {
    let mut text: String = String::new();
    if pass.pruned.is_empty() {
        text.push_str(&format!("No kept reference was pruned, so nothing was given back; the objects only kept references reach weigh {} bytes.\n", pass.bytes_before));
        return text;
    }
    for one in &pass.pruned {
        text.push_str(&format!("pruned {} (commit {}, {} ago): {}\n", one.reference, one.commit, age_words(one.age_ms), why_words(one.why, limits)));
    }
    text.push_str(&format!("The objects only kept references reach weigh {} bytes now.\nThe line about each one is in {}.\n", pass.bytes_after, log.display()));
    return text;
}

/// Controls for the listing stay out of the release executable.
#[cfg(test)]
#[path = "kept_reference_listing_tests.rs"]
mod tests;
