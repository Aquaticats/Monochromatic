//! What:
//!  The largest answer one command of the second pass may print:
//!  the push report's own cap,
//!  one stage per object name a `cat-file --batch` asks for,
//!  and the whole of a `merge-file` answer,
//!  whose marker lines are bounded by the lines of its stages and not by their bytes.
//! Why:
//!  These children are not a person's:
//!  the reconcile runs them in the object store of a repository that another program may have wedged,
//!  so an answer is refused at a ceiling instead of held whole,
//!  and the ceiling of each command is what says whether a merge the pass resolves is refused as a flood.
//!  The measurement that set the marker ceiling is in the doc of `MERGE_ANSWER_MARKER_BYTES`.
//!
//! Files beside this one,
//!  and why the merge commit is not one file:
//!  `auto_push_reconcile_merge_commit.rs` runs the commands and commits the merged tree,
//!  and asks this file for the ceiling each answer is bounded by.
//!  `rust/max-lines` splits the merge commit by role,
//!  and the ceilings are the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cap = (args: string[], input?: Uint8Array): number => { /* ... */ };
//! ```

/// The push report's own cap,
///  which every command of the pass but a whole `merge-file` answer keeps.
use crate::bounded_push::MAX_PUSH_OUTPUT;
/// The largest stage one command of the pass takes.
use crate::merge_second_pass::LARGEST_STAGE_BYTES;

/// The bytes one `git cat-file --batch` header may take:
/// the object name,
/// its kind,
/// the size and the line feed.
pub(crate) const CAT_FILE_HEADER_BYTES: u64 = 128;

/// The most bytes the content of one `merge-file` answer of the second pass may take:
/// the three stages it is given,
/// and the text Git settled around its regions.
/// Why:
///   A stage the pass takes is at most `LARGEST_STAGE_BYTES`,
/// and its answer holds the text Git settled plus the three sides of every region;
/// each line of a stage lands in the answer once,
/// either in the settled text or in a region's side,
/// so three stages hold the content and the fourth is the room a line can shift between them.
pub(crate) const MERGE_ANSWER_CONTENT_BYTES: u64 = 4 * LARGEST_STAGE_BYTES;

/// The most bytes the marker lines of one `merge-file` answer of the second pass may take:
/// eight stages,
/// which is about four hundred thousand regions.
/// Why:
///   Each region adds four marker lines,
/// which with the pass's labels are fifty,
/// fifty,
/// fifty-one and eight bytes for `ours-` and a 36-character UUID;
/// the number of regions is bounded by the lines of the stages and never by a margin of their bytes,
/// so a triple of near-limit stages with a conflict every other line holds a hundred and three thousand regions
/// and prints a 37.9 MB answer although its content is under four stages
/// (measured 2026-11-11 with Git 2.56.0:
/// stages of 7,605,777 bytes each,
/// 103,000 regions,
/// 37,871,557 bytes of answer;
/// cross-vendor review of round 198,
/// finding 2).
/// An answer is held whole and then cut,
/// so the answer of a merge at this ceiling costs its own bytes twice,
/// about 190 MB,
/// plus the merged file and its parts,
/// under the 352 MB the pass's own word merge of three 8 MiB texts already costs
/// (`merge_second_pass.rs`).
/// This ceiling is a memory backstop and not the bound of what eligible stages can print:
/// marker bytes grow with the lines of the stages and not with their bytes,
/// so a stage filled with two-byte lines holds more regions,
/// and more marker lines,
/// than any ceiling can take,
/// and such an answer is refused and the path stays as Git left it,
/// for the person to resolve.
/// A merge this ceiling newly admits costs the merge its own time in Git before the pass reads anything:
/// `merge-tree` took 192 seconds on the fixture above in one run under a 3-CPU scope on a host at load 66 to 75,
/// so the 300-second ceiling of the merge step is what ends the slowest of them.
pub(crate) const MERGE_ANSWER_MARKER_BYTES: u64 = 8 * LARGEST_STAGE_BYTES;

/// The most bytes one `merge-file` of the second pass may print:
/// its content and its marker lines.
pub(crate) const MERGE_FILE_ANSWER_BYTES: u64 =
    MERGE_ANSWER_CONTENT_BYTES + MERGE_ANSWER_MARKER_BYTES;

/// What:
///  The largest answer one command of the second pass may print.
/// Why:
///   `cat-file --batch` prints one blob per name it was given and a header for each,
/// and no stage the pass merges is larger than its own limit,
/// so the
///  answer for the names it asks for is bounded and a larger one would be refused after being read:
/// the ceiling stops the command
///  instead of holding a blob the pass would not take.
/// `merge-file --stdout` prints the merge of three stages,
/// so its answer is larger than each of them
/// and the push report's cap would refuse a merge the pass resolves.
/// Every other command of the pass prints a line or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cap = (args, input) => args[0] === 'cat-file' ? Math.max(MAX_PUSH_OUTPUT, names * (LARGEST_STAGE_BYTES + 128)) : args[0] === 'merge-file' ? MERGE_FILE_ANSWER_BYTES : MAX_PUSH_OUTPUT;
/// ```
pub(crate) fn answer_cap(
    arguments: &[&str],
    input: Option<&[u8]>,
) -> u64 {
    match arguments.first() {
        // The names `cat-file` asks for are counted from its input; every other command gets a fixed ceiling.
        Some(&"cat-file") => {
            let mut asked: u64 = 0;
            for byte in input.unwrap_or_default() {
                if *byte == b'\n' {
                    asked += 1;
                }
            }
            return asked
                .saturating_mul(LARGEST_STAGE_BYTES.saturating_add(CAT_FILE_HEADER_BYTES))
                .max(MAX_PUSH_OUTPUT);
        }
        Some(&"merge-file") => return MERGE_FILE_ANSWER_BYTES,
        _ => return MAX_PUSH_OUTPUT,
    }
}
