//! What:
//!  One drift the coverage comparison found:
//!  the field that failed,
//!  what was measured against what the decision says,
//!  and the concrete correction when one exists,
//!  plus the check's answer and the lines a report prints.
//! Why:
//!  `doctor` prints one line per finding and the correction beside it,
//!  so a person edits one thing per line
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints",
//!  which asks for the concrete INI correction or `zfs set` for confirmed
//!  drift).
//!  A finding without a correction says why none exists:
//!  a folder inside another dataset cannot be excluded,
//!  and a store outside recursive coverage is not moved by an INI line.
//!  The field names are stable handles,
//!  so a log line from one release still says which fact spoke in another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CoverageFinding = { field: string; detail: string; correction?: string };
//! answer.lines(); // ['finding: section-order: ...', 'correction: move the [a] section below [b]']
//! ```

use crate::doctor_zfs_route::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};

/// The configuration carries no section for the recursive ancestor.
pub const FIELD_ROOT_SECTION: &str = "root-section";

/// The recursive ancestor's section does not cover its children by a word
/// sanoid reads as true.
pub const FIELD_RECURSION: &str = "recursion";

/// The recursive ancestor's section keeps no template,
/// or another one.
pub const FIELD_TEMPLATE: &str = "template";

/// The measured tree does not hold one excluded dataset.
pub const FIELD_EXCLUSION_DATASET: &str = "exclusion-dataset";

/// One excluded dataset's mount point is not the kind the decision gives it.
pub const FIELD_EXCLUSION_MOUNT: &str = "exclusion-mount";

/// The configuration carries no section for one excluded dataset.
pub const FIELD_EXCLUSION_SECTION: &str = "exclusion-section";

/// One excluded dataset's section stands before a section whose recursion
/// reaches it.
pub const FIELD_SECTION_ORDER: &str = "section-order";

/// One excluded dataset's section does not turn its snapshots off.
pub const FIELD_EXCLUSION_AUTOSNAP: &str = "exclusion-autosnap";

/// One excluded dataset's `com.sun:auto-snapshot` is not `false`.
pub const FIELD_EXCLUSION_MARKER: &str = "exclusion-marker";

/// The store is not held by a dataset recursive coverage reaches.
pub const FIELD_STORE_COVERAGE: &str = "store-coverage";

/// What:
///  One drift of a person's configuration from the decided coverage.
/// Why:
///  The three parts a person acts on stay together:
///  what failed,
///  what the two sides say,
///  and the line that ends the drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoverageFinding = { field: string; detail: string; correction?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageFinding {
    /// The stable name of the fact that failed.
    pub field: String,
    /// What the check measured and what the decision says.
    pub detail: String,
    /// The text a person applies,
    /// or nothing when no configuration line or property ends the drift.
    pub correction: Option<String>,
}

impl CoverageFinding {
    /// One finding,
    /// with the correction that ends it or nothing.
    pub fn new(
        field: impl Into<String>,
        detail: impl Into<String>,
        correction: Option<String>,
    ) -> Self {
        return CoverageFinding {
            field: field.into(),
            detail: detail.into(),
            correction,
        };
    }
}

/// What:
///  What the coverage check answered:
///  the report's answer,
///  every finding,
///  and the fact the check could not prove when one stopped the comparison.
/// Why:
///  An unproved fact outranks a finding,
///  because a comparison that could not read the whole configuration cannot
///  claim its list of drifts is complete,
///  and the answer still carries the findings it did make so the unproved
///  fact never hides them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoverageAnswer = { outcome: CheckOutcome; findings: CoverageFinding[]; unproved?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageAnswer {
    /// The report's answer for this check.
    pub outcome: CheckOutcome,
    /// Every drift the comparison found.
    pub findings: Vec<CoverageFinding>,
    /// The fact that stopped the comparison,
    /// when one did.
    pub unproved: Option<String>,
}

impl CoverageAnswer {
    /// What:
    ///  The answer for one comparison.
    /// Why:
    ///  The precedence of unknown over finding over pass stands in one place,
    ///  so no caller can report a complete pass over a text nobody read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// coverageAnswer(findings, unproved, passDetail);
    /// ```
    pub fn new(
        findings: Vec<CoverageFinding>,
        unproved: Option<String>,
        pass_detail: String,
    ) -> Self {
        let outcome: CheckOutcome = match (&unproved, findings.first()) {
            (Some(reason), _) => unknown(reason.clone()),
            (None, Some(first)) => {
                let fields: Vec<&str> = findings
                    .iter()
                    .map(|entry| return entry.field.as_str())
                    .collect();
                finding(
                    first.field.clone(),
                    format!("{} finding(s): {}", findings.len(), fields.join(", ")),
                )
            }
            (None, None) => pass(pass_detail),
        };
        return CoverageAnswer {
            outcome,
            findings,
            unproved,
        };
    }

    /// What:
    ///  The lines a report prints below the check's own line:
    ///  one for each finding,
    ///  its correction after it,
    ///  and the unproved fact last.
    /// Why:
    ///  The report's one-line answer cannot hold a correction,
    ///  and setup findings go to standard error without changing `doctor`'s
    ///  exit status,
    ///  so every line a person acts on is printed here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answer.lines(); // ['finding: recursion: ...', 'correction: ...', 'unknown: ...']
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for entry in &self.findings {
            lines.push(format!("finding: {}: {}", entry.field, entry.detail));
            if let Some(correction) = &entry.correction {
                lines.push(format!("correction: {correction}"));
            }
        }
        if let Some(reason) = &self.unproved {
            lines.push(format!("unknown: {reason}"));
        }
        return lines;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "coverage_finding_tests.rs"]
mod tests;
