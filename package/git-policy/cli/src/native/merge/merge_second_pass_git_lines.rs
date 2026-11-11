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
use crate::merge_second_pass_line_regions::{cut, lines_before, offset_from_line};
use crate::word_merge::{LineSpan, MergedPart, WordMerge, merge_by_word};

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
    let reply: GitReply = runner
        .run(
            &[
                "merge-file",
                "--stdout",
                "--object-id",
                "--zdiff3",
                "-L",
                "ours",
                "-L",
                "base",
                "-L",
                "theirs",
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
        Some(count) if (1..=127).contains(&count) => usize::try_from(count).expect("a conflict count is positive"),
        other => return Err(format!("git merge-file answered with {other:?}: {}", String::from_utf8_lossy(&reply.stderr).trim())),
    };
    let cut = cut(reply.stdout.as_slice())?;
    // Git reports the number of conflicts as its exit status, so a cut that finds another number read
    // the answer wrongly. The status is truncated to 127, so the comparison is only exact below that.
    if conflicts < 127 && cut.regions.len() != conflicts {
        return Err(format!("git merge-file reported {conflicts} conflicts and its answer holds {}", cut.regions.len()));
    }
    let mut merged: Vec<u8> = Vec::new();
    let mut parts: Vec<MergedPart> = Vec::new();
    let mut cursors: [usize; 3] = [0, 0, 0];
    for (at, region) in cut.regions.iter().enumerate() {
        merged.extend_from_slice(cut.settled[at].as_slice());
        let region_sides: [&[u8]; 3] = [region.base.as_slice(), region.ours.as_slice(), region.theirs.as_slice()];
        let mut starts: [usize; 3] = [0, 0, 0];
        for (side, text) in sides.iter().enumerate() {
            match offset_from_line(*text, region_sides[side], cursors[side]) {
                Some(found) => {
                    starts[side] = found;
                    cursors[side] = found + region_sides[side].len();
                }
                None => {
                    // A region whose place cannot be found is still a region the person must judge;
                    // the refusal names it by the region count rather than by a wrong line.
                    return Ok(WordMerge::Conflict { unresolved: cut.regions.len() });
                }
            }
        }
        let WordMerge::Resolved { merged: text, parts: inner } = merge_by_word(region.base.as_slice(), region.ours.as_slice(), region.theirs.as_slice()) else {
            return Ok(WordMerge::Conflict { unresolved: cut.regions.len() });
        };
        merged.extend_from_slice(text.as_slice());
        for part in inner {
            parts.push(MergedPart {
                base: span_in_the_file(part.base, sides[0], starts[0]),
                ours: span_in_the_file(part.ours, sides[1], starts[1]),
                theirs: span_in_the_file(part.theirs, sides[2], starts[2]),
                merged: part.merged,
            });
        }
    }
    merged.extend_from_slice(cut.settled.last().expect("a piece after the last region").as_slice());
    return Ok(WordMerge::Resolved { merged, parts });
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
/// const spanInTheFile = (span, stage, regionStart) => ({ first: span.first + linesBefore(stage, regionStart), count: span.count });
/// ```
fn span_in_the_file(span: LineSpan, stage: &[u8], region_start: usize) -> LineSpan {
    return LineSpan { first: span.first + lines_before(stage, region_start), count: span.count };
}
