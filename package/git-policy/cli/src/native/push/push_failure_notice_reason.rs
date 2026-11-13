//! What:
//!  Git's words as a record of a push keeps them:
//! the user name and password of every URL masked,
//! control characters dropped,
//!  and the length bounded.
//!  The last line stays whole when the text is cut.
//! Why:
//!  The text came from a remote and from Git and is printed on a terminal,
//! and the last line of a report is the wrapper's own sentence:
//!  the statement of what a reconcile published and where the branch is.
//! A cut at the end left a person who reads only that line with the cut marker instead
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "A typed push of a branch that was not checked out published a merge labelled with the checked-out branch").
//!  It lives apart from the records that hold these words
//!  (`push_failure_notice.rs`) to keep each file within the repository's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = cleanReason(gitsWords); // the first and the last line, and a marker where the rest went
//! ```

use crate::lfs_output_mask::mask_url_secrets_text;

/// The most bytes of Git's own words a record keeps.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
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
    let mut text: String = kept.join("\n");
    if text.len() > LONGEST_REASON {
        text = keeping_the_ends(&kept, LONGEST_REASON);
    }
    return text;
}

/// What:
///  The longest end of a text that fits a budget,
/// with the lines in the middle cut out and a marker where they were.
/// Why:
///   The last line is the wrapper's own sentence,
/// and the first lines are Git's:
/// a notice told by the last line alone lost the statement of what was published and where the branch is
/// when the text was cut at its end,
/// and a failure notice cut at its end lost Git's words as well.
/// So the two ends are kept and only the middle goes,
/// and a single line that needs the whole budget is cut and marked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepingTheEnds(lines: string[], budget: number): string;
/// ```
fn keeping_the_ends(lines: &[String], budget: usize) -> String {
    const MARKER: &str = "(Git's output is cut here.)";
    let Some((last, earlier)) = lines.split_last() else {
        return String::new();
    };
    if last.len() + MARKER.len() + 2 > budget {
        let mut cut: usize = budget;
        while !last.is_char_boundary(cut) {
            cut -= 1;
        }
        return format!("{}\n{MARKER}", &last[..cut]);
    }
    // The last line, the marker, and the two line feeds between them are already spent.
    let mut used: usize = last.len() + MARKER.len() + 2;
    let mut head: Vec<&str> = Vec::new();
    for line in earlier {
        if used + line.len() + 1 > budget {
            break;
        }
        used += line.len() + 1;
        head.push(line.as_str());
    }
    head.push(MARKER);
    head.push(last.as_str());
    return head.join("\n");
}
