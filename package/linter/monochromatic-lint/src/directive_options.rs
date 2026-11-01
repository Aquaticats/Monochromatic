//! What:
//!  Whether and how loudly unused suppression directives are reported.
//! Why:
//!  The command line spells this like oxlint does:
//!  `--report-unused-disable-directives` reports at warning severity,
//!  and `--report-unused-disable-directives-severity` names the severity,
//!  where `allow` and `off` report nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DirectiveOptions = { unused?: 'warn' | 'error' };
//! ```

/// Import the finding severity that an unused-directive finding carries.
use crate::diagnostic::Severity;
/// Import clap's derive for a closed set of accepted spellings.
use clap::ValueEnum;

/// What:
///  The severity spellings oxlint accepts for unused directives.
/// Why:
///  Matching the incumbent's spellings lets a repository move its flag between the two tools.
///  `deny` is the same level as `error`,
///  and `off` is the same level as `allow`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnusedDirectiveSeverity = 'allow' | 'off' | 'warn' | 'deny' | 'error';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum UnusedDirectiveSeverity {
    /// Report nothing.
    Allow,
    /// Report nothing.
    Off,
    /// Report as a warning.
    Warn,
    /// Report as an error.
    Deny,
    /// Report as an error.
    Error,
}

/// What:
///  The directive engine's run-wide settings.
/// Why:
///  They are fixed by the command line for a whole run,
///  so every file plan carries the same copy.
///  Absence of a severity means unused directives are not reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirectiveOptions = { unused?: Severity };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectiveOptions {
    /// Severity of an unused-directive finding,
    ///  or `None` when they are not reported.
    pub unused: Option<Severity>,
}

/// What:
///  Resolve the two command-line spellings into one setting.
/// Why:
///  The command grammar rejects giving both,
///  so at most one is present here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// DirectiveOptions.fromFlags(report, severity)
/// ```
impl DirectiveOptions {
    /// The plain flag means warning severity;
    ///  the severity option names its own level.
    pub fn from_flags(report: bool, severity: Option<UnusedDirectiveSeverity>) -> DirectiveOptions {
        let unused: Option<Severity> = match severity {
            Some(UnusedDirectiveSeverity::Allow) | Some(UnusedDirectiveSeverity::Off) => None,
            Some(UnusedDirectiveSeverity::Warn) => Some(Severity::Warn),
            Some(UnusedDirectiveSeverity::Deny) | Some(UnusedDirectiveSeverity::Error) => {
                Some(Severity::Error)
            }
            None if report => Some(Severity::Warn),
            None => None,
        };
        return DirectiveOptions { unused };
    }
}

/// Resolution controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_options_tests.rs"]
mod tests;
