//! What:
//!  The merge of one candidate file:
//! the line merge Git makes with `--zdiff3`,
//! then the word pass on every region it left.
//! Why:
//!   Git's line merge is the one the person's own Git would have made,
//! and `zdiff3` keeps the lines both sides wrote identically out of a conflict,
//! so the word pass sees only the lines that are a choice.
//! The pass is all-or-nothing still:
//! a file is resolved only when every region resolves,
//! so a file Git left conflicted and this cannot settle whole stays exactly as Git left it.
//!  The markers of the answer carry labels made unguessable for every merge,
//! so a line of the person's file that already looks like a marker is the file's own text:
//! the labels are what tells Git's delimiters from what the file held
//! (cross-vendor review of round 133,
//! finding 2).
//! The split marker carries no label,
//! so a line of the file that is exactly seven equals signs is told from Git's split by what the three stages
//! hold,
//! and a region no single split explains is left to the person
//! (`Cut::place`;
//! cross-vendor review of round 198,
//! finding 1).
//! Measured on 723 files that both sides of a two-parent merge changed and that plain line merging
//! conflicts on,
//! from 3,194 merges of this repository,
//! this resolved 130 of them where the pass's own line merge resolved 108 under the same refusals,
//! with no line no side holds in a region that spans lines
//! (`doc/planning/cli-git-merge-auto-resolution.md`,
//! "Measured again on 2026-11-10").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function mergeWithGitLines(runner, conflict, [base, ours, theirs]): Promise<WordMerge>;
//! ```

use crate::merge_second_pass::{Conflict, GitReply, GitRunner};
use crate::merge_second_pass_line_regions::{Cut, MarkerLabels, Region, cut, lines_between};
use crate::word_merge::{LineSpan, MergedPart, WordMerge, merge_by_word};

/// How many conflicts Git counts in its exit status at most:
/// a merge with more conflicts exits with this status as well,
/// so a status of `CONFLICT_STATUS_CEILING` means that many or more.
const CONFLICT_STATUS_CEILING: i32 = 127;

/// What:
///  The merge of one candidate file:
/// the line merge Git makes with `--zdiff3`,
/// then the word pass on every region it left.
/// Why:
///   Git's line merge is the one the person's own Git would have made,
/// and `zdiff3` keeps the lines both sides wrote identically out of a conflict,
/// so the word pass sees only the lines that are a choice;
/// measured on 723 files that both sides of a two-parent merge changed and that plain line merging conflicts on,
/// from this repository's history,
/// this resolved 130 of them where the pass's own line merge resolved 108,
/// with no line no side holds in a region that spans lines and the same 21 lines no side holds overall
/// (`doc/planning/cli-git-merge-auto-resolution.md`).
/// A file is still resolved only when every region resolves,
/// so a file Git left conflicted and the pass cannot settle whole stays exactly as Git left it.
///  A marker line is Git's own only when it carries the label this call gave the merge,
/// and those labels are a fresh random value per call,
/// so a block already in the person's file that looks like a conflict is left as text:
/// Git does not escape the markers it writes,
/// and the count in Git's exit status cannot tell the two apart at its ceiling.
/// The split marker carries no label,
/// so a split-shaped line of the file is told from Git's split by the stages the region's sides are found in,
/// and a region no single split explains is left to the person rather than merged on a guess.
/// `Err` means Git could not be started or answered with something that is not a merge;
/// the caller then leaves every conflict as Git left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeWithGitLines(runner, conflict, [base, ours, theirs]): Promise<WordMerge>;
/// ```
pub(super) async fn merge_with_git_lines(runner: &dyn GitRunner, conflict: &Conflict, sides: [&[u8]; 3]) -> Result<WordMerge, String> {
    let (Some(base_stage), Some(ours_stage), Some(theirs_stage)) = (&conflict.base, &conflict.ours, &conflict.theirs) else {
        return Ok(WordMerge::Conflict { unresolved: 1 });
    };
    // The labels are made unguessable for every merge, because Git does not escape the markers it writes: a line of the
    // file that looks like a marker of its own cannot be told from a region by the count alone, and Git's status stops
    // counting at `CONFLICT_STATUS_CEILING`.
    let nonce: String = match crate::random_id::random_uuid() {
        Ok(made) => made,
        Err(error) => return Err(format!("no label for the line merge of this file: {}", error.0)),
    };
    let ours_label: String = format!("ours-{nonce}");
    let base_label: String = format!("base-{nonce}");
    let theirs_label: String = format!("theirs-{nonce}");
    let labels: MarkerLabels<'_> = MarkerLabels { ours: ours_label.as_bytes(), base: base_label.as_bytes(), theirs: theirs_label.as_bytes() };
    let reply: GitReply = runner
        .run(
            &[
                "merge-file",
                "--stdout",
                "--object-id",
                "--zdiff3",
                "-L",
                ours_label.as_str(),
                "-L",
                base_label.as_str(),
                "-L",
                theirs_label.as_str(),
                ours_stage.oid.as_str(),
                base_stage.oid.as_str(),
                theirs_stage.oid.as_str(),
            ],
            None,
            None,
        )
        .await?;
    let conflicts: usize = match reply.code {
        // A merge with no conflict left is Git's clean answer, and there is no region to word-merge.
        Some(0) => return Ok(WordMerge::Resolved { merged: reply.stdout, parts: Vec::new() }),
        // The exit status of `git merge-file` is the number of conflicts, truncated to 127.
        Some(count) if (1..=CONFLICT_STATUS_CEILING).contains(&count) => usize::try_from(count).expect("a conflict count is positive"),
        other => return Err(format!("git merge-file answered with {other:?}: {}", String::from_utf8_lossy(&reply.stderr).trim())),
    };
    let cut: Cut = cut(reply.stdout.as_slice(), &labels)?;
    let ceiling: usize = usize::try_from(CONFLICT_STATUS_CEILING).expect("the conflict status ceiling is positive");
    // Git reports the number of conflicts as its exit status, so a cut that finds another number read the answer wrongly.
    // The status stops counting at its ceiling, so at that count the answer may hold more regions than Git counted, never
    // fewer.
    if conflicts < ceiling && cut.regions.len() != conflicts {
        return Err(format!("git merge-file reported {conflicts} conflicts and its answer holds {}", cut.regions.len()));
    }
    if conflicts == ceiling && cut.regions.len() < conflicts {
        return Err(format!("git merge-file reported {conflicts} or more conflicts and its answer holds {}", cut.regions.len()));
    }
    // The split of each region is chosen by the stages, and a region no single split explains is still a region the
    // person must judge; the refusal names it by the region count rather than by a wrong line.
    let Some(regions): Option<Vec<Region>> = cut.place(sides) else {
        return Ok(WordMerge::Conflict { unresolved: cut.regions.len() });
    };
    let mut merged: Vec<u8> = Vec::new();
    let mut parts: Vec<MergedPart> = Vec::new();
    // What: each side's line count up to the place it reached, carried from region to region.
    // Why: a region's place never moves backwards, so counting the lines from the last place to this one costs the stage's
    // lines once for the whole file; counting them from the top of the stage for every part cost the stage per part, and
    // the pass over a hundred and three thousand one-line regions against stages of 7.6 MB took about seven hundred seconds
    // of a push that took 889 seconds,
    // whose merge-tree took about 190
    // (measured 2026-11-11 on this host with the release build).
    let mut counted_to: [usize; 3] = [0, 0, 0];
    let mut counted_lines: [usize; 3] = [0, 0, 0];
    for (at, region) in regions.iter().enumerate() {
        merged.extend_from_slice(cut.settled[at].as_slice());
        let WordMerge::Resolved { merged: text, parts: inner } = merge_by_word(region.base.as_slice(), region.ours.as_slice(), region.theirs.as_slice()) else {
            return Ok(WordMerge::Conflict { unresolved: cut.regions.len() });
        };
        merged.extend_from_slice(text.as_slice());
        let base_above: usize = count_lines_to(&mut counted_to[0], &mut counted_lines[0], sides[0], region.starts[0]);
        let ours_above: usize = count_lines_to(&mut counted_to[1], &mut counted_lines[1], sides[1], region.starts[1]);
        let theirs_above: usize = count_lines_to(&mut counted_to[2], &mut counted_lines[2], sides[2], region.starts[2]);
        for part in inner {
            parts.push(MergedPart {
                base: span_in_the_file(part.base, base_above),
                ours: span_in_the_file(part.ours, ours_above),
                theirs: span_in_the_file(part.theirs, theirs_above),
                merged: part.merged,
            });
        }
    }
    merged.extend_from_slice(cut.settled.last().expect("a piece after the last region").as_slice());
    return Ok(WordMerge::Resolved { merged, parts });
}

/// What:
///  How many lines of `text` come before `place`,
/// counted from the place this side last reached.
/// Why:
///   A region's place never moves backwards,
/// so each stage's lines are counted once for the whole file instead of once per part.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const countLinesTo = (reached, counted, text, place) => { counted.value += linesBetween(text, reached.at, place); reached.at = place; return counted.value; };
/// ```
fn count_lines_to(reached: &mut usize, counted: &mut usize, text: &[u8], place: usize) -> usize {
    *counted += lines_between(text, *reached, place);
    *reached = place;
    return *counted;
}

/// What:
///  A line span of one region,
/// moved to its place in the file that region came from.
/// Why:
///   The report names a part by the line of the file a person opens,
/// and the word pass numbers the lines of the region it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spanInTheFile = (span, linesAbove) => ({ first: span.first + linesAbove, count: span.count });
/// ```
fn span_in_the_file(span: LineSpan, lines_above: usize) -> LineSpan {
    return LineSpan { first: span.first + lines_above, count: span.count };
}
