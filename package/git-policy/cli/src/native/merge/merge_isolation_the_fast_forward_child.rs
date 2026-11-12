//! What:
//!  The Git child the wrapper runs to move the branch in an isolated merge,
//!  and the words the person reads about it:
//!  a merge
//!  that fast-forwards is that step itself,
//!  so Git's own line about it is the merge's line.
//! Why:
//!  A merge the wrapper completes for a merge commit moves the checkout with
//! `git merge --ff-only --no-edit <merge>`
//!  and prints the words of a merge commit itself,
//!  because Git's own line about that step ("Updating a..b",
//! "Fast-forward")
//!  is about
//!  the wrapper's step and not about the merge the person asked for.
//!  A merge that fast-forwards is that step:
//!  native Git writes `Updating a..b`,
//! `Fast-forward`,
//!  and the
//!  diffstat with its create,
//!  delete and mode lines,
//!  and with a message typed (`-m`) it says that no commit was created and the
//!  message was ignored,
//!  in that line and in the reflog entry.
//!  Measured with Git 2.56.0:
//! `git merge --ff-only --no-edit -m <text> <commit>` under the reflog action the wrapper
//!  sets writes the same line and the same reflog entry as `git merge --no-edit -m <text> <branch>`,
//!  so the person's
//!  message goes to that child.
//!  The person's `-q`,
//! `--stat` and `--no-stat` are not handed to it:
//! no Git run the wrapper starts
//!  carries `--quiet` or another option that only hides what Git prints (`README.md`,
//! "No `--quiet` on the
//!  wrapper's own Git runs"),
//!  and the diffstat is hidden here instead,
//!  by printing the first two lines of what Git said
//!  (`doc/troubleshooting/cli-git-merge-fast-forward-carries.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = theArguments({ shape, newOid }); // ['merge', '--ff-only', '--no-edit', ['-m', message], oid]
//! const shown = theOutput({ shape, config, printed }); // the header, the whole thing, or nothing under -q
//! ```

/// The command line and the settings of the merge.
use crate::merge_isolation_run_types::MergeConfig;
use crate::merge_options::MergeShape;
/// One argument of raw bytes,
/// which a typed message may hold.
use crate::merge_isolation_carry_reads::path_argument;
/// Owned operating-system text.
use std::ffi::OsString;

/// How many lines of Git's own words about a fast-forward come before its diffstat:
/// `Updating a..b`,
/// then `Fast-forward` with the note about an ignored `-m`.
const THE_HEADER_LINES: usize = 2;

/// What:
///  The arguments of the child that fast-forwards the branch,
///  with the message the person typed.
/// Why:
///   The message decides what Git says and what it writes to the reflog
///  (`Fast-forward (no commit created; -m option
///  ignored)`),
///  so handing it over is what makes the carried fast-forward indistinguishable from the one native Git
///  would have run.
///  A typed `-m` is never read for a fast-forward;
///  Git only reports that it was ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theArguments({ shape, newOid }): OsString[]; // merge --ff-only --no-edit [-m message] newOid
/// ```
pub fn the_arguments(shape: &MergeShape, new_oid: &str) -> Vec<OsString> {
    let mut words: Vec<OsString> = Vec::with_capacity(6);
    for token in ["merge", "--ff-only", "--no-edit"] {
        words.push(OsString::from(token));
    }
    if let Some(message) = shape.message.as_ref() {
        words.push(OsString::from("--message"));
        words.push(path_argument(message.as_slice()));
    }
    words.push(OsString::from(new_oid));
    return words;
}

/// What:
///  What the person reads about a carried fast-forward,
///  from what Git printed about it.
/// Why:
///   `-q` hides everything,
///  `--no-stat`,
///  `-n` and `merge.stat=false` hide the diffstat and keep the two lines that
///  name the step,
///  and every other shape is Git's own output whole.
///  The two lines are read by their place and not by their text,
///  so a Git whose words are translated says what this program says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theOutput({ shape, config, printed }): string; // '' | the header | printed
/// ```
pub fn the_output(shape: &MergeShape, config: &MergeConfig, printed: &[u8]) -> String {
    if shape.quiet {
        return String::new();
    }
    let said: String = String::from_utf8_lossy(printed).into_owned();
    if shape.stat.unwrap_or(config.stat) {
        return said;
    }
    let mut lines: usize = 0;
    for (index, byte) in said.as_bytes().iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        lines += 1;
        if lines == THE_HEADER_LINES {
            return String::from(&said[..index + 1]);
        }
    }
    return said;
}

/// What:
///  The arguments of the child that applies a merge commit to the checkout,
///  whose output the wrapper replaces with the words of the merge it made.
/// Why:
///   Git's own line about that step is `Updating a..b` and `Fast-forward`,
///  which is true of the step and not of
///  the merge,
///  so the wrapper prints `Merge made by the 'ort' strategy.` and the merge's diffstat itself
///  (`merge_isolation_apply.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = ['merge', '--ff-only', '--no-edit', mergeOid];
/// ```
pub fn the_merge_commit_arguments(new_oid: &str) -> Vec<OsString> {
    let mut words: Vec<OsString> = Vec::with_capacity(4);
    for token in ["merge", "--ff-only", "--no-edit"] {
        words.push(OsString::from(token));
    }
    words.push(OsString::from(new_oid));
    return words;
}

/// The words one child is given,
/// and the output the person reads.
#[cfg(test)]
#[path = "merge_isolation_the_fast_forward_child_tests.rs"]
mod tests;
