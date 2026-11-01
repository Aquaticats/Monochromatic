//! Severity vocabulary of annotations:
//!  the worst-first rank,
//!  the word a reader sees,
//!  the default for an omitted severity,
//!  and the one-line text of a problem.

/// The problem record whose text `describe` spells out.
use super::Problem;
/// The severities as the Language module reports them.
use crate::language::diagnostics::Severity;

/// What:
///  Order of severity from worst to mildest;
///  `u8` is an unsigned byte (siblings `u32`,
///  `usize`).
/// Why:
///  Overlapping marks are drawn mildest first,
///  and messages above a line are listed worst first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rank(severity: Severity): number { return ['error', 'warning', 'information', 'hint'].indexOf(severity); }
/// ```
pub fn rank(severity: Severity) -> u8 {
    if severity == Severity::Error {
        return 0;
    }
    if severity == Severity::Warning {
        return 1;
    }
    if severity == Severity::Information {
        return 2;
    }
    return 3;
}

/// What:
///  The word a reader sees for a severity;
///  `&'static str` is text baked into the binary (sibling `String`).
/// Why:
///  Every message row starts with its severity in words,
///  so it does not depend on color.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function severityName(severity: Severity): string;
/// ```
pub fn severity_name(severity: Severity) -> &'static str {
    if severity == Severity::Error {
        return "Error";
    }
    if severity == Severity::Warning {
        return "Warning";
    }
    if severity == Severity::Information {
        return "Information";
    }
    return "Hint";
}

/// What:
///  Severity a diagnostic is drawn with;
///  `Option<Severity>` is a severity or nothing.
/// Why:
///  The protocol leaves an omitted severity to the client;
///  Helix shows it as a warning
///      (`helix-term/src/ui/editor.rs`,
///  `Some(Severity::Warning) | None => warning`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shown(severity?: Severity): Severity { return severity ?? 'warning'; }
/// ```
pub(super) fn shown(severity: Option<Severity>) -> Severity {
    // `unwrap_or` takes the present severity or substitutes the warning.
    return severity.unwrap_or(Severity::Warning);
}

/// What:
///  The text of one problem,
///  for example `Error E0308 (rustc): mismatched types`.
/// Why:
///  The severity is spelled out,
///  so a message row does not rely on its ink;
///  the code and source say who
///      reported it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(problem: Problem): string;
/// ```
pub fn describe(problem: &Problem) -> String {
    let mut text = severity_name(problem.mark.severity).to_string();
    if let Some(code) = &problem.code {
        text.push(' ');
        text.push_str(code);
    }
    if !problem.source.is_empty() {
        // `format!` builds a new `String` from a template, like a TS template literal.
        text.push_str(&format!(" ({})", problem.source));
    }
    text.push_str(": ");
    text.push_str(problem.message.trim());
    return text;
}
