//! What: Render the native planner's results as the canonical texts the incumbent's driver
//!       writes: failures by class, plans with their exact manifest bytes, and findings.
//! Why: Equal texts mean equal plans, findings, patched bytes and failure kinds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const text = canonical(result); // the driver's TypeScript twin of these functions
//! ```

/// The failure kinds the canonical failure classifies.
use crate::dependent_version_content::{PlanError, PolicyIncomplete};
/// Canonical JSON writers shared with the fixture reader.
use crate::dependent_version_fixture_json::{array, object, quote, quote_hex};
/// The planned bump and the plan outcome.
use crate::dependent_version_plan::{ManifestBump, PlanOutcome};
/// The policy's findings.
use crate::dependent_version_policy::DependentFinding;
/// The incumbent's message for an unsupported version.
use crate::dependent_version_release::unsupported_message;

/// The canonical failure: its class and, for shape problems, the incumbent's message.
pub fn failed(error: &PlanError) -> String {
    let (class, detail): (&str, String) = match error {
        PlanError::ContentUnavailable(_) => ("unavailable", String::new()),
        PlanError::PolicyIncomplete(PolicyIncomplete::NotUtf8 { .. }) => ("decode", String::new()),
        PlanError::PolicyIncomplete(PolicyIncomplete::ManifestSyntax { .. }) => {
            ("syntax", String::new())
        }
        PlanError::PolicyIncomplete(PolicyIncomplete::ManifestShape { problem, .. }) => {
            ("shape", problem.clone())
        }
        PlanError::PolicyIncomplete(PolicyIncomplete::DuplicateName { .. }) => {
            ("graph", String::new())
        }
    };
    return object(&[
        ("kind", quote("failed")),
        ("error", quote(class)),
        ("detail", quote(&detail)),
    ]);
}

/// The canonical plan result.
pub fn plan_output(outcome: &Result<PlanOutcome, PlanError>) -> String {
    match outcome {
        Err(error) => return failed(error),
        Ok(PlanOutcome::Unsupported(unsupported)) => {
            return object(&[
                ("kind", quote("unsupported")),
                ("message", quote(&unsupported_message(unsupported))),
            ]);
        }
        Ok(PlanOutcome::Planned(plan)) => {
            let bumps: Vec<String> = plan.bumps.iter().map(bump_output).collect();
            let names: Vec<String> = plan
                .bumped_names
                .iter()
                .map(|name| return quote(name))
                .collect();
            return object(&[
                ("kind", quote("planned")),
                ("bumpedNames", array(&names)),
                ("bumps", array(&bumps)),
            ]);
        }
    }
}

/// The canonical form of one planned manifest bump.
pub fn bump_output(bump: &ManifestBump) -> String {
    return object(&[
        ("name", quote(&bump.planned.name)),
        ("directory", quote_hex(&bump.planned.directory)),
        ("from", quote(&bump.planned.bump.from)),
        ("to", quote(&bump.planned.bump.to)),
        ("path", quote_hex(&bump.path)),
        ("original", quote_hex(&bump.original)),
        ("replacement", quote_hex(&bump.replacement)),
    ]);
}

/// The canonical policy result.
pub fn policy_output(outcome: &Result<Vec<DependentFinding>, PlanError>) -> String {
    let findings: &Vec<DependentFinding> = match outcome {
        Err(error) => return failed(error),
        Ok(found) => found,
    };
    let rendered: Vec<String> = findings
        .iter()
        .map(|finding| {
            let patch: String = finding.patch.as_ref().map_or_else(
                || return String::from("null"),
                |bump| {
                    return object(&[
                        ("path", quote_hex(&bump.path)),
                        ("original", quote_hex(&bump.original)),
                        ("replacement", quote_hex(&bump.replacement)),
                    ]);
                },
            );
            let path: String = finding
                .path
                .as_deref()
                .map_or_else(|| return String::from("null"), quote_hex);
            return object(&[
                ("code", quote(finding.code)),
                ("message", quote(&finding.message)),
                ("path", path),
                ("patch", patch),
            ]);
        })
        .collect();
    return object(&[("kind", quote("findings")), ("findings", array(&rendered))]);
}
