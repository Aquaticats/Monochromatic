//! What:
//!  Validate a single atomic processor group,
//!  including virtual-to-empty rewrites.
//! Why:
//!  Empty virtual content is allowed;
//!  the existing host edit engine still refuses an empty real file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Validate sorted byte intervals, then append original slices and replacements once.
//! ```

/// Import exact byte edits and immutable processor failures.
use crate::edits::Fix;
/// Map failures to the original host before returning.
use crate::processors_model::{Mapping, ProcessorError};

/// Return a borrowed edit's byte sort key without capturing a function-local variable.
fn edit_key(edit: &&crate::edits::Edit) -> (usize, usize) {
    return (edit.start, edit.end);
}

/// Refuse malformed,
///  internally overlapping or CRLF-bisecting virtual edits.
pub(crate) fn rewrite(mapping: &Mapping, fix: &Fix) -> Result<String, ProcessorError> {
    // Retain references rather than copying replacement strings; sorting never changes caller order.
    let mut ordered: Vec<&crate::edits::Edit> = fix.edits.iter().collect();
    ordered.sort_by_key(edit_key);
    let mut previous_start: Option<usize> = None;
    let mut cursor: usize = 0;
    let mut output: String = String::new();
    for edit in ordered {
        if edit.start > edit.end
            || edit.end > mapping.text.len()
            || !mapping.text.is_char_boundary(edit.start)
            || !mapping.text.is_char_boundary(edit.end)
        {
            return Err(mapping
                .error("Processor fix has an invalid byte range or splits a UTF-8 character."));
        }
        if previous_start == Some(edit.start) || edit.start < cursor {
            return Err(
                mapping.error("Processor fix contains overlapping edits in one atomic group.")
            );
        }
        for boundary in [edit.start, edit.end] {
            if boundary > 0
                && mapping.text.as_bytes().get(boundary) == Some(&b'\n')
                && mapping.text.as_bytes()[boundary - 1] == b'\r'
            {
                return Err(mapping.error("Processor fix splits an authored CRLF newline."));
            }
        }
        output.push_str(&mapping.text[cursor..edit.start]);
        output.push_str(edit.replacement.as_str());
        cursor = edit.end;
        previous_start = Some(edit.start);
    }
    output.push_str(&mapping.text[cursor..]);
    return Ok(output);
}
