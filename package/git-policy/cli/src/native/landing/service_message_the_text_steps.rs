//! What:
//!  The text steps of a commit's message,
//!  in the order Git applies them:
//!  the line breaks at its end,
//!  the comment lines the clean-up removes,
//!  the encoding header it may carry,
//!  and the author line of a commit an amend reuses.
//! Why:
//!   Everything here is text arithmetic Git itself defines,
//!  so the wrapper embeds it instead of starting `git commit`,
//!  and each step is compared field by field with the native commit
//!  (`binary_service_options_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const body = withFinalLineBreak(stripCommentLines(text));
//! ```

/// What:
///  Whether a header line of a raw commit names an encoding.
/// Why:
///  A commit in another encoding is Git's to reuse,
/// so `reused_commit` gives it up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEncodingHeader = (line: string) => line.startsWith('encoding ');
/// ```
pub(super) fn is_encoding_header(line: &str) -> bool {
    return line.starts_with("encoding ");
}

/// What:
///  The author line a raw commit header line carries,
/// nothing for any other line.
/// Why:
///  `reused_commit` takes the first such line as the author.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const authorOfHeaderLine = (line: string) => line.startsWith('author ') ? line.slice(7) : undefined;
/// ```
pub(super) fn author_of_header_line(line: &str) -> Option<&str> {
    return line.strip_prefix("author ");
}

/// What:
///  A text without the line breaks at its end.
/// Why:
///  The subject Git prints after `fixup! ` ends in a newline that is not part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutFinalLineBreaks = (text: string) => text.replace(/\n+$/, '');
/// ```
pub(super) fn without_final_line_breaks(text: String) -> String {
    return text.trim_end_matches('\n').to_string();
}

/// What:
///  Whether a line is not a comment:
/// it does not start with `#`.
/// Why:
///  `strip_comment_lines` keeps these lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotCommentLine = (line: string) => !line.startsWith('#');
/// ```
pub(super) fn is_not_comment_line(line: &&str) -> bool {
    return !line.starts_with('#');
}

/// What:
///  A text with a line break at its end,
/// added when it has none.
/// Why:
///  A verbatim message keeps every text as given,
/// each ending in a line break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withFinalLineBreak = (text: string) => text.endsWith('\n') ? text : text + '\n';
/// ```
pub(super) fn with_final_line_break(text: &String) -> String {
    return if text.ends_with('\n') {
        text.clone()
    } else {
        format!("{text}\n")
    };
}

/// The lines of a message without those that start with the comment character,
/// as `--cleanup=strip` drops them.
pub fn strip_comment_lines(text: &str) -> String {
    return text
        .split('\n')
        .filter(is_not_comment_line)
        .collect::<Vec<&str>>()
        .join("\n");
}
