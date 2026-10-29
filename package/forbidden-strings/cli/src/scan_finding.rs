//! What:
//!  Redacted structured findings shared by embedded and standalone scans.
//! Why:
//!  A policy consumer must not recover candidate identity or finding kind by parsing terminal text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // A discriminated union carries rule identity and source location, never matched bytes.
//! ```

/// What:
///  One content/name result or explicit inability to complete a scan.
/// Why:
///  Enum variants keep source locations separate from operational failures without sentinel line numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanFinding = { kind: 'content'; line: number; rule: string }
///   | { kind: 'name'; component: number; rule: string }
///   | { kind: 'engine-error' } | { kind: 'pathname-line-break' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanFinding {
    /// A rule matched content on one physical source line.
    Content {
        /// One-based line index;
        ///  usize indexes an in-memory file rather than a fixed-width wire integer.
        line: usize,
        /// Owned non-secret configured name or opaque numeric rule token,
        ///  never the matched source.
        rule: String,
    },
    /// A rule matched an entire logical pathname component.
    Name {
        /// One-based component number after excluding navigation/volume prefixes.
        component: usize,
        /// Owned rule identity,
        ///  independent of the fully masked matching component.
        rule: String,
    },
    /// A matcher failed,
    ///  so absence of matches cannot be treated as a pass.
    EngineError,
    /// A pathname contains line endings that the line-based matcher cannot inspect faithfully.
    PathnameLineBreak,
}

/// Keep terminal rendering at the standalone consumer boundary.
impl ScanFinding {
    /// What:
    ///  Render the existing standalone protocol using an already sanitized display path.
    /// Why:
    ///  Structured consumers use the enum directly;
    ///  the executable preserves its current text contract.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function render(finding, safeDisplayPath): string;
    /// ```
    pub(crate) fn render(&self, safe_display: &str) -> String {
        // Match only the enum's actual variants; adding an operational failure requires an explicit rendering choice.
        match self {
            Self::Content { line, rule } => return format!("{safe_display}:{line} rule={rule}"),
            Self::Name { component, rule } => return format!("{safe_display}:name:{component} rule={rule}"),
            Self::EngineError => return format!("{safe_display}: engine error"),
            Self::PathnameLineBreak => return format!("{safe_display}: unsupported pathname line break"),
        }
    }
}
