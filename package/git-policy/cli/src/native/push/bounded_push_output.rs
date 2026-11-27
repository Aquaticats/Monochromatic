//! What:
//!  The output a push keeps:
//!  the text read back from the file its streams wrote,
//!  whole lines only,
//!  and the line that says only the head of it is kept.
//! Why:
//!   A cap applied to raw bytes cuts a URL in half,
//!  and half a URL hides no secret,
//!  so what is kept stops at the last whole line that fits;
//!  the flag that says the run printed more comes from the read itself,
//!  so a run that ended between two looks of the wait still says its output is the head of it.
//!  These reads and words sit in a file of their own,
//!  because the run that uses them is more than the package's length limit allows one file
//!  (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! { text: readKeptOutput(await readFile(outputPath, 'utf8')), cut }
//! ```

/// The failure a run's own read is reported with.
use crate::one_push_attempt_and_its_failure::SingleFlightError;
/// The removal of the one trailing line break a captured run leaves.
use crate::push_deadline_and_patience_settings::strip_one_line_break;
/// The cap of the output a push keeps.
use crate::push_wait_until_done::MAX_PUSH_OUTPUT;
/// The anonymous output file the run's streams wrote.
use std::fs::File;
/// The reading of that file back from its start.
use std::io::{
    Read,
    Seek,
    SeekFrom,
};

/// What:
///  The output of a run,
///  read back from the file's start,
///  and whether the cap kept only part of it.
/// Why:
///   The flag comes from the read itself,
///  so a run that ended between two looks of the wait still says that its output is the head of it;
///  the size the wait looked at cannot answer for what was read later.
///  What is kept is whole lines,
///  because a cap applied to the raw bytes cuts a URL in half and half a URL hides no secret:
///  a cut before the `@` of a userinfo keeps the front of a password whichever mask runs afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { text: await readFile(outputPath, 'utf8'), cut }
/// ```
pub(crate) fn read_output(file: &mut File) -> Result<(String, bool), SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    // One byte past the cap is read, so the read itself says whether more was printed than is kept.
    file.take(MAX_PUSH_OUTPUT + 1)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    let limit: usize = usize::try_from(MAX_PUSH_OUTPUT).unwrap_or(usize::MAX);
    if bytes.len() <= limit {
        return Ok((String::from_utf8_lossy(&bytes).into_owned(), false));
    }
    let kept: usize = bytes
        .get(..limit)
        .and_then(|head| {
            return head.iter().rposition(is_line_feed);
        })
        .map_or(0, |at| {
            return at + 1;
        });
    return Ok((
        String::from_utf8_lossy(bytes.get(..kept).unwrap_or_default()).into_owned(),
        true,
    ));
}

/// Whether a byte ends a line.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The output an attempt keeps from a captured file:
///  the text with the one trailing line break removed,
///  and,
/// when the run ended with more printed than the cap,
///  the line that says only the head of it is kept.
/// Why:
///   The head of a run that ended is its own status's output,
///  not a stopped push's,
///  so the cut line is added from the read and from whether the run ended,
///  never from the size the wait last looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = stripOneTrailingBreak(captured); const text = shortened ? `${head}\n${cutLine(label)}` : head
/// ```
pub(crate) fn kept_output_text(
    captured: &str,
    shortened: bool,
    label: &str,
) -> String {
    let mut text: String = strip_one_line_break(captured).to_owned();
    if !shortened {
        return text;
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(shortened_output_line(label).as_str());
    return text;
}

/// What:
///  The line a push's output gains when more of it was printed than is kept.
/// Why:
///   The cap bounds what is held in memory,
/// and a person reading an attempt's output must know that what they see is the
///  head of it.
/// The push itself was not stopped,
/// so the line says what was kept and points at the program that printed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${label} wrote more than ${MAX} bytes of output, so only the whole lines that fit in the first ${MAX} bytes are kept here; the run itself was not stopped. Hint: if a hook or the remote prints this much, read the hook's own output by running it directly.`
/// ```
fn shortened_output_line(label: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {label} wrote more than {MAX_PUSH_OUTPUT} bytes of output, so only the whole lines that fit in \
         the first {MAX_PUSH_OUTPUT} bytes are kept here; the run itself was not stopped. Hint: if a hook or the remote prints this \
         much, read the hook's own output by running it directly."
    );
}
