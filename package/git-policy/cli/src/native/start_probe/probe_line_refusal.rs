//! What:
//!  Why a status line on a probe pipe could not be read.
//! Why:
//!   A child that wrote a line is not a child whose line can be trusted,
//!  and each cause is a different answer:
//!  another program's line on the pipe,
//!  a step this build does not know,
//!  an answer it does not know,
//!  a field given twice or missing,
//!  a word that is no errno name,
//!  an arm that disagrees with its step,
//!  or a value that is not one word.
//!  The sentence names the line and what to change,
//!  and no cause is decoded by matching prose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LineRefusal = { kind: 'not-a-status-line' | 'unknown-step' | ...; word: string };
//! ```

use super::probe_status_prefix::STATUS_PREFIX;

/// What:
///  Why a status line could not be read.
/// Why:
///   A child that wrote a line is not a child whose line can be trusted,
///  and each cause is a different answer for the person reading the record:
///  another program's line on the pipe,
///  a step this build does not know,
///  an answer it does not know,
///  a field given twice,
///  a field missing,
///  or a word that is no errno name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineRefusal = { kind: 'not-a-status-line' | 'unknown-step' | ...; word: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineRefusal {
    /// The line does not begin with the status prefix.
    NotAStatusLine {
        /// The line as read.
        text: String,
    },
    /// A field's key is not one this build writes.
    UnknownKey {
        /// The key as read.
        word: String,
    },
    /// A field was given more than once.
    RepeatedKey {
        /// The key.
        key: String,
    },
    /// A field the answer requires is absent.
    MissingKey {
        /// The key.
        key: String,
    },
    /// A step word is none of this build's steps.
    UnknownStep {
        /// The word as read.
        word: String,
    },
    /// An answer word is none of `passed`,
    /// `failed` or `not-exercised`.
    UnknownAnswer {
        /// The word as read.
        word: String,
    },
    /// A word is no errno name this build knows,
    /// and is not `none`.
    UnknownErrno {
        /// The word as read.
        word: String,
    },
    /// A reason word is none this build writes.
    UnknownReason {
        /// The word as read.
        word: String,
    },
    /// The arm field disagrees with the arm the step belongs to.
    WrongArm {
        /// The step's own arm.
        expected: &'static str,
        /// The arm the line carries.
        word: String,
    },
    /// A value holds a space or is empty,
    /// so the fixed-key line cannot carry it.
    ValueNotOneWord {
        /// The key whose value it is.
        key: String,
    },
}

impl LineRefusal {
    /// The refusal in words,
    /// naming the line and what to change.
    pub fn message(&self) -> String {
        return match self {
            LineRefusal::NotAStatusLine {
                text,
            } => format!(
                "the line `{text}` on a probe status pipe does not begin with `{STATUS_PREFIX}`, so it is not a step outcome and the child's answer cannot be read. The add refuses the managed attach as unproven."
            ),
            LineRefusal::UnknownKey {
                word,
            } => format!(
                "the status line's field `{word}` is not one this build writes. Ways forward: nothing on this host changes it; report the line and the build that wrote it, because a child of another build wrote it."
            ),
            LineRefusal::RepeatedKey {
                key,
            } => format!(
                "the status line's field `{key}` was given more than once, so two values would compete. The child's answer cannot be read."
            ),
            LineRefusal::MissingKey {
                key,
            } => format!(
                "the status line has no `{key}` field, which its answer needs. The child's answer cannot be read."
            ),
            LineRefusal::UnknownStep {
                word,
            } => format!(
                "the status line's step `{word}` is none of this build's steps. Ways forward: nothing on this host changes it; report the word and the build that wrote it."
            ),
            LineRefusal::UnknownAnswer {
                word,
            } => format!(
                "the status line's answer `{word}` is none of `passed`, `failed` and `not-exercised`. The child's answer cannot be read."
            ),
            LineRefusal::UnknownErrno {
                word,
            } => format!(
                "the status line's errno `{word}` is no name this build knows. The child's answer cannot be read."
            ),
            LineRefusal::UnknownReason {
                word,
            } => format!(
                "the status line's reason `{word}` is no cause this build writes. The child's answer cannot be read."
            ),
            LineRefusal::WrongArm {
                expected,
                word,
            } => format!(
                "the status line's arm `{word}` is not the arm `{expected}` its step belongs to, so the line disagrees with this build. The child's answer cannot be read."
            ),
            LineRefusal::ValueNotOneWord {
                key,
            } => format!(
                "the status line's `{key}` value is empty or holds a space, and this line carries words only."
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_line_refusal_tests.rs"]
mod tests;
