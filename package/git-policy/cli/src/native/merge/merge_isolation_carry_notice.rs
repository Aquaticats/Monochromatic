//! What:
//!  The words a merge prints when it carries a person's own work across it:
//! the files it merged by word,
//! the
//!  files it kept both sides of between markers,
//! the files it left whole,
//! and the files it did not touch because they
//!  changed under it.
//! Why:
//!  A conflict the person must judge still exits 0,
//! so the line is the only way they learn what to do with the file;
//! it
//!  names the path and says what stands there now
//! (`SPEC.md`,
//! "Merges and commits never fail").
//!  A line about a path the carry kept whole names what stands at it of the person's work and what
//!  the merge made of it,
//! so a deleted path,
//! a symbolic link and a mode the word pass could not take are each named
//!  for what they are (`merge_isolation_carry_kept_path.rs`).
//! The words live in a file of their own so the
//!  file that writes the checkout stays readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const line of reportLines('feature', report)) stderr.write(`${line}\n`);
//! ```

use crate::merge_isolation_carry_kept_path::{Kept, kept_line};

/// What:
///  Everything one carry did,
/// for the lines it prints.
/// Why:
///  A path the word pass merged is worth a line,
/// a path holding markers is worth a line that says what to do,
/// and a
///  path another program changed is worth the one line that says it was left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Report = { merged: Uint8Array[]; conflicts: Uint8Array[]; kept: Kept[]; changed: Uint8Array[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    /// Paths the word pass merged,
    /// whose regions are named by the second pass's own lines.
    pub merged: Vec<Vec<u8>>,
    /// Paths that hold both sides between conflict markers.
    pub conflicts: Vec<Vec<u8>>,
    /// Paths left whole,
    /// with the reason the merge could not take them and what stands at each of them now.
    pub kept: Vec<Kept>,
    /// Paths that changed after the merge captured them,
    /// which nothing was written to.
    pub changed: Vec<Vec<u8>>,
}

impl Report {
    /// What:
    ///  Name the paths a write left alone,
    /// and take back every line a path's own step built before that write.
    /// Why:
    ///   A line is built from what stood at the path when its step judged it,
    /// and a path another program wrote after that holds the
    ///  program's bytes:
    /// the merge wrote nothing to it,
    /// so the earlier line is false about the checkout this command leaves,
    ///  and beside the line that names the change it contradicts it.
    /// The path keeps one line,
    /// the one that says the merge wrote nothing to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// report.aPathWrittenAgain(written.changed);
    /// ```
    pub(crate) fn a_path_written_again(&mut self, paths: Vec<Vec<u8>>) {
        for path in paths {
            self.merged.retain(|kept| *kept != path);
            self.conflicts.retain(|kept| *kept != path);
            self.kept.retain(|kept| kept.path != path);
            self.changed.push(path);
        }
    }
}

/// What:
///  The lines a carried merge adds to standard error,
/// after Git's own.
/// Why:
///   Each line names one file and what is in it,
/// and the conflict line says what to do with the markers,
/// because a
///  command that exits 0 has no other way to say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportLines(reference: string, report: Report): string[];
/// ```
pub fn report_lines(reference: &str, report: &Report) -> Vec<String> {
    // A command that finished a merge an earlier one began is not running that merge, so its lines name the merge the person
    //  started rather than a command they did not type.
    let command: String = if reference.is_empty() {
        String::from("the merge an earlier command began and this one finished")
    } else {
        format!("git merge {reference}")
    };
    let mut lines: Vec<String> = Vec::new();
    for path in &report.conflicts {
        lines.push(format!(
            "monochromatic-git-policy-cli: {command}: your changes to {} touch the lines the merge changes, so the file \
             holds both sides between conflict markers and the merge was made. Resolve the markers in that file, then run `git add -- {}`.",
            text(path),
            quoted(path)
        ));
    }
    for path in &report.merged {
        lines.push(format!(
            "monochromatic-git-policy-cli: {command}: your changes to {} were merged with the merge's by word, and both sides are in the file.",
            text(path)
        ));
    }
    for kept in &report.kept {
        lines.push(kept_line(command.as_str(), kept));
    }
    for path in &report.changed {
        lines.push(format!(
            "monochromatic-git-policy-cli: {command}: {} changed while the merge ran, so this merge wrote nothing to it and \
             left that change in place. The merge's version is in the merge commit; `git diff HEAD -- {}` shows the difference.",
            text(path),
            quoted(path)
        ));
    }
    return lines;
}

/// What:
///  Say that a path the captured work does not explain was left as it is,
/// with no claim about what stands there.
/// Why:
///   A merge that does not land puts the person's own layers back,
/// and a path written after the capture
///  holds bytes newer than those,
/// so nothing is written there and the person is told which file was left alone.
///  What that path holds now is theirs to see:
/// a deletion made after the capture leaves nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// say(`${path} changed after this merge captured it, so it was left as it is and the change that came after stands`);
/// ```
pub(crate) fn note_left_alone<W: std::io::Write + ?Sized>(out: &mut W, path: &[u8]) {
    let named: String = text(path);
    let _noted = out.write_all(
        format!(
            "monochromatic-git-policy-cli: {named} changed after this merge captured it, so it was left as it is and the change that \
             came after stands; the merge wrote nothing to it.\n"
        )
        .as_bytes(),
    );
}

/// The text of one path for a line the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = (path: Uint8Array) => pathMessageText(path);
/// ```
fn text(path: &[u8]) -> String {
    return crate::path_bytes::path_message_text(path);
}

/// What:
///  The same path spelled for a shell,
/// which every command a line prints takes.
/// Why:
///   The person copies the command out of the line,
/// and a name with a space reaches the program as several arguments otherwise
///  (`crate::shell_quote`,
/// the one owner of that spelling).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoted = (path: Uint8Array) => shellQuote(path);
/// ```
fn quoted(path: &[u8]) -> String {
    return crate::shell_quote::shell_quote(path);
}

/// The words a carry adds.
#[cfg(test)]
#[path = "merge_isolation_carry_notice_tests.rs"]
mod tests;
