//! What:
//!  Git's words as a record of a push keeps them:
//! the user name and password of every URL masked,
//! control characters dropped,
//!  and, for a push that will be tried again,
//! the length bounded.
//!  A cut keeps the first and the last line whole.
//! Why:
//!  The text came from a remote and from Git and is printed on a terminal,
//! and the last line of a report is the wrapper's own sentence:
//!  the statement of what a reconcile published and where the branch is.
//! A cut at the end left a person who reads only that line with the cut marker instead
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "A typed push of a branch that was not checked out published a merge labelled with the checked-out branch").
//!  The account of a push that landed keeps every line,
//! because nothing makes such a push say it again;
//! only the display cap of a push that did not finish cuts anything.
//!  It lives apart from the records that hold these words
//!  (`push_failure_notice.rs`) to keep each file within the repository's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = cleanReason(gitsWords); // masked, unbounded for a landed push, bounded for a failure
//! ```

use crate::lfs_output_mask::mask_url_secrets_text;

/// What:
///  The line that says where the middle of a long text went.
/// Why:
///   The marker is appended after a cut,
/// so the cut is sized against it:
/// a bound that forgot it produced a record some bytes longer than the limit it names
/// (a review of commit `98de70959d`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CUT_MARKER = "(Git's output is cut here.)";
/// ```
const CUT_MARKER: &str = "(Git's output is cut here.)";

/// The most bytes of Git's own words a record of a push that did not land keeps.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
/// It bounds no account of a push that landed,
/// which nothing can make say the same thing again
/// (`clean_words`).
pub(crate) const LONGEST_REASON: usize = 4000;

/// What:
///  Git's words for the person:
/// the user name and password of every URL masked,
/// control characters dropped,
/// and the
///  length bounded.
/// Why:
///   The text came from a remote and from Git,
/// and is printed on a terminal;
/// a URL with a user name and password,
/// or an
///  escape sequence,
/// must not reach the screen or a log of it.
/// Only the credentials are masked,
/// never the line around them,
///  because the line often says why the push failed (the wrapper's own Git LFS failure line names the endpoint setting and
///  carries the exit status).
/// The bound is the notice's own display cap and belongs to a push that will say the same thing again on its next try;
/// the account of a push that landed keeps every line (`clean_words`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
    return bounded(clean_words(said));
}

/// What:
///  Git's words for the person with nothing cut:
/// every URL credential masked,
/// control characters dropped,
/// and every line kept.
/// Why:
///   The account of a push that landed is the only copy of what a server and the reconcile said,
/// and nothing can make an
///  accepted push say it again:
/// a push of what the remote already holds prints `Everything up-to-date` and runs no hook
///  (`SPEC.md`,
/// "Reconcile and bounds").
/// That account therefore keeps every line the push chose to show,
/// while a failure record
///  keeps the display cap,
/// because a failure is tried again and says the same thing then.
/// Only the credentials are masked,
/// never the line around them,
/// as `clean_reason`
///  says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanWords(said)
/// ```
pub fn clean_words(said: &str) -> String {
    let masked: String = mask_url_secrets_text(said);
    let mut kept: Vec<String> = Vec::new();
    for line in masked.lines() {
        let mut plain: String = String::new();
        for character in line.chars() {
            if character == '\t' || !character.is_control() {
                plain.push(character);
            }
        }
        let trimmed: &str = plain.trim_end();
        if !trimmed.is_empty() {
            kept.push(trimmed.to_string());
        }
    }
    return kept.join("\n");
}

/// What:
///  The words cut to the most bytes a notice of a push that did not land keeps,
/// with a line that says so,
/// and the last line whole.
/// Why:
///   `LONGEST_REASON` is a display cap of the person's notice,
/// and a push Git refused can print any
///  amount of text.
///   The last line is the wrapper's own sentence,
/// the statement of what a reconcile published and where the branch is,
/// and a cut at the end costs a person who reads only that line the very words they need;
/// the first lines are Git's,
///  which name what to repair.
/// So the two ends stay and only the middle goes,
/// and a text that is one long line keeps its beginning and says where it was cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bounded(words) // the first and the last line, and a marker where the rest went
/// ```
fn bounded(text: String) -> String {
    if text.len() <= LONGEST_REASON {
        return text;
    }
    let lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    return keeping_the_ends(&lines, LONGEST_REASON);
}

/// What:
///  The longest end of a text that fits a budget,
/// with the lines in the middle cut out and a marker where they were.
/// Why:
///   Every byte the answer holds is counted against the budget,
/// the marker included,
///  so the record never exceeds the cap it names;
/// the earlier version cut one long line at the whole budget and then appended the marker,
///  which made the record longer than its own limit
/// (a review of commit `98de70959d`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepingTheEnds(lines: string[], budget: number): string;
/// ```
fn keeping_the_ends(lines: &[String], budget: usize) -> String {
    let Some((last, earlier)) = lines.split_last() else {
        return String::new();
    };
    // The marker and the line feed before it are spent whatever else the answer holds.
    let spent: usize = CUT_MARKER.len() + 1;
    if spent + last.len() > budget {
        // One line needs the whole budget: keep its beginning, cut where the marker still fits.
        let room: usize = budget.saturating_sub(spent);
        let mut cut: usize = room.min(last.len());
        while !last.is_char_boundary(cut) {
            cut -= 1;
        }
        if cut == 0 {
            return String::from(CUT_MARKER);
        }
        return format!("{}\n{CUT_MARKER}", &last[..cut]);
    }
    // The last line, the marker, and the line feeds between the kept lines are spent; a line more costs its own feed.
    let mut used: usize = spent + last.len();
    let mut head: Vec<&str> = Vec::new();
    for line in earlier {
        if used + line.len() + 1 > budget {
            break;
        }
        used += line.len() + 1;
        head.push(line.as_str());
    }
    head.push(CUT_MARKER);
    head.push(last.as_str());
    return head.join("\n");
}
