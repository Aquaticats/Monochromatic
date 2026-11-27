//! What:
//!  How serious an entry is,
//!  and the words of one entry:
//!  the text cut to a bound with the mark that says it was cut,
//!  and the text made one line for standard error.
//! Why:
//!  The native protocol carries any bytes (a value with a newline in the length-prefixed form),
//!  so a value is cut and not changed;
//!  only a line on standard error needs every control character replaced,
//!  because a newline in a message would otherwise start a second line in the stream.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! text.length > limit ? text.slice(0, limit) + ' [cut]' : text
//! ```

/// What is appended to a cut message.
pub(crate) const CUT_MARK: &str = " [cut]";

/// How serious an entry is,
/// as journald's `PRIORITY` (syslog levels).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    /// Something the service needed failed;
    /// `journalctl --priority=err` shows it.
    Error,
    /// Something did not go as intended and the service went on.
    Warning,
    /// An event of the service's ordinary life.
    Info,
}

impl Priority {
    /// The syslog number journald reads.
    pub fn number(self) -> u8 {
        return match self {
            Priority::Error => 3,
            Priority::Warning => 4,
            Priority::Info => 6,
        };
    }
}

/// What:
///  The text cut to `limit` characters,
/// with the mark appended when it was cut;
/// control characters and newlines are kept.
/// Why:
///   The native protocol carries any bytes (a value with a newline in the length-prefixed form),
/// so a value is cut and not
///  changed;
/// only the line on standard error needs `one_line`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.length > limit ? text.slice(0, limit) + ' [cut]' : text
/// ```
pub fn cut(
    text: &str,
    limit: usize,
) -> String {
    return match text.char_indices().nth(limit) {
        Some((end, _)) => format!("{}{CUT_MARK}", &text[..end]),
        None => text.to_string(),
    };
}

/// What:
///  The text with every control character (a newline included) replaced by a space,
/// and cut to `limit` characters.
/// Why:
///   A line on standard error is one line whatever a client or an error message held:
/// a newline in a message could
///  otherwise start a second line in the stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/[\x00-\x1f\x7f]/g, ' ').slice(0, limit)
/// ```
pub fn one_line(
    text: &str,
    limit: usize,
) -> String {
    let mut single: String = String::with_capacity(text.len().min(limit + CUT_MARK.len()));
    for (count, character) in text.chars().enumerate() {
        if count >= limit {
            single.push_str(CUT_MARK);
            break;
        }
        single.push(if character.is_control() {
            ' '
        } else {
            character
        });
    }
    return single;
}
