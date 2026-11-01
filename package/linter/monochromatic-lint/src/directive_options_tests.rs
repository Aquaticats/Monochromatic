//! What:
//!  Controls for the unused-directive settings.
//! Why:
//!  Each spelling must map to the severity oxlint gives it,
//!  and the default must report nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('DirectiveOptions', () => { /* default, flag, each severity spelling */ });
//! ```

/// Import the resolver and the accepted severity spellings.
use super::{DirectiveOptions, UnusedDirectiveSeverity};
use crate::diagnostic::Severity;

/// Without either option nothing is reported.
#[test]
fn unused_directives_are_not_reported_by_default() {
    assert_eq!(DirectiveOptions::from_flags(false, None).unused, None);
    assert_eq!(DirectiveOptions::default().unused, None);
}

/// The plain flag reports at warning severity,
///  as oxlint does (measured with `oxlint --report-unused-disable-directives`).
#[test]
fn the_plain_flag_reports_warnings() {
    assert_eq!(
        DirectiveOptions::from_flags(true, None).unused,
        Some(Severity::Warn)
    );
}

/// Every severity spelling maps to its level,
///  and `allow` and `off` report nothing even beside the plain flag's absence.
#[test]
fn each_severity_spelling_maps_to_its_level() {
    let cases: [(UnusedDirectiveSeverity, Option<Severity>); 5] = [
        (UnusedDirectiveSeverity::Allow, None),
        (UnusedDirectiveSeverity::Off, None),
        (UnusedDirectiveSeverity::Warn, Some(Severity::Warn)),
        (UnusedDirectiveSeverity::Deny, Some(Severity::Error)),
        (UnusedDirectiveSeverity::Error, Some(Severity::Error)),
    ];
    for (spelling, expected) in cases {
        assert_eq!(
            DirectiveOptions::from_flags(false, Some(spelling)).unused,
            expected,
            "{spelling:?}"
        );
    }
}
