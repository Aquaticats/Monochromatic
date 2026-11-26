//! What:
//!  The span of every line of each file a layout finding points at,
//!  each file read once and never whole.
//! Why:
//!  Findings carry a byte offset,
//!  a line and a column,
//!  and one file can carry many findings:
//!  a list near its cap full of dead lines makes one finding per line about that one file,
//!  so reading the file again for each finding would cost one read per finding.
//!  The step that reports the findings owns one `LineSpans`
//!  and asks it for one line at a time,
//!  so the table of every line stays here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class LineSpans { span(file: Path, line: number): Span }
//! ```

/// Import the diagnostic span and the file-start span a missing line reports.
use crate::{
    diagnostic::Span,
    run_failure::file_start,
};
/// Import the ordered table of files already read,
///  the bounded reader,
///  and the paths both are keyed by.
use std::{
    collections::BTreeMap,
    io::Read,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The most bytes of one file this reader reads to answer its lines:
///  eight mebibytes.
/// Why:
///  Lines are answered for the files a finding is reported in,
///  which are the Rust files of the package
///  and the layout rule's allow list:
///  this repository's largest tracked Rust file was 176,292 bytes on 2026-11-26,
///  and a list cannot be over four mebibytes (`layout_allow_list::MAX_BYTES`).
///  Eight mebibytes is about 47 times that largest file and twice that cap,
///  so every file this repository can hold keeps its finding's line,
///  while a path that turns out to be a device or a file that grew is never read whole.
///  A line the bound cuts off reports the file start,
///  because a span with a guessed length would point at the wrong bytes.
///  The siblings a reader might expect are an unbounded read and a memory map;
///  a bound on the bytes read is the one that also ends on a file that keeps growing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_SPAN_BYTES = 8 * 1024 * 1024;
/// ```
const MAX_SPAN_BYTES: u64 = 8 * 1024 * 1024;

/// What:
///  The line spans of each file already read,
///  keyed by absolute path.
/// Why:
///  A caller reports findings file by file and never needs the whole table,
///  so the table stays inside this type and only the span it asks for comes out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineSpans = Map<Path, Span[] | undefined>;
/// ```
#[derive(Debug, Default)]
pub(crate) struct LineSpans {
    /// The lines of each file already read,
    ///  or nothing when the file could not be read.
    by_file: BTreeMap<PathBuf, Option<Vec<Span>>>,
}

/// What:
///  The span of each line of a file in file order,
///  or nothing when the file cannot be read.
/// Why:
///  One pass over at most `MAX_SPAN_BYTES` of the file answers every line the read reaches,
///  and the bytes are split on the line break itself,
///  so no decoding stands between a finding's offset and the bytes of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLineSpans(file: Path): Span[] | undefined;
/// ```
fn read_line_spans(file: &Path) -> Option<Vec<Span>> {
    // What: the file opened for a bounded read, or nothing when it cannot be opened.
    // Why: a folder or a link that vanished between two findings must not fail the run.
    let mut opened: std::fs::File = std::fs::File::open(file).ok()?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    // What: at most the bound, read through a handle that stays open for the byte after it.
    // Why: the handle must still be usable to ask whether a byte follows, which a consumed reader could not answer.
    (&mut opened)
        .take(MAX_SPAN_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;
    // What: the one byte a file longer than the bound still holds, and whether it was there.
    // Why: a read of exactly the bound cannot tell a whole file of that size from a longer one.
    let mut past: [u8; 1] = [0; 1];
    let cut: bool = match opened.read(&mut past) {
        Ok(read) => read > 0,
        Err(_) => true,
    };
    let mut spans: Vec<Span> = Vec::<Span>::new();
    let mut start: usize = 0;
    for (index, part) in bytes
        .split_inclusive(|byte| return *byte == b'\n')
        .enumerate()
    {
        // A bound that cut this line left its true length unknown, so this line and the rest get no span.
        if cut && part.last() != Some(&b'\n') {
            break;
        }
        let mut length: usize = part.len();
        while length > 0 && (part[length - 1] == b'\n' || part[length - 1] == b'\r') {
            length -= 1;
        }
        spans.push(Span {
            offset: start,
            length,
            line: index + 1,
            column: 1,
        });
        start += part.len();
    }
    return Some(spans);
}

/// What:
///  The line spans of a file,
///  asked for one line at a time.
/// Why:
///  A finding names one line,
///  and the answer for it must not read the file again:
///  a file is read on the first question about it and the answer is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineSpans.span(file, line)
/// ```
impl LineSpans {
    /// What:
    ///  The span of one line of one file,
    ///  or the file start when the line is zero,
    ///  the file does not hold it,
    ///  or the file cannot be read.
    /// Why:
    ///  A finding about a folder or a link has no line,
    ///  and a file that vanished between two findings must not fail the run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// span(file: Path, line: number): Span
    /// ```
    pub(crate) fn span(
        &mut self,
        file: &Path,
        line: usize,
    ) -> Span {
        if line == 0 {
            return file_start();
        }
        if !self.by_file.contains_key(file) {
            self.by_file
                .insert(file.to_path_buf(), read_line_spans(file));
        }
        let Some(Some(spans)) = self.by_file.get(file) else {
            return file_start();
        };
        return match spans.get(line - 1) {
            Some(span) => span.clone(),
            None => file_start(),
        };
    }
}

/// Controls of the bounded line reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_line_spans_tests.rs"]
mod tests;
