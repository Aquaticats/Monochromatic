//! What:
//! Adding the entries the arguments described to the plan's text,
//! under the scope's OS lock,
//! and the report of what each argument named.
//! Why:
//! This is the write `mpm i -g <tool>` does before the install:
//! the plan is read once,
//! the entries are appended as text with everything else kept,
//! and the file is published only if no other mpm-shadow wrote it meanwhile.
//! A request that cannot be described,
//! or a run that may not write,
//! refuses here,
//! before the plan is touched.
//! A caller imports from `install_argument` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function nothingStopsTheRun(): void;
//! export function addNamedTools(context: Context, arguments: string[], refusal?: AddRefusal, afterPlanWrite: () => void): AddReport;
//! ```

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the model type an entry is.
use super::super::model::Declaration;
/// Imports the plan the arguments are looked up in,
/// and the reader that decodes the plan text.
use super::super::plan::{
    Plan,
    parse_plan,
};
/// Imports the splice into the plan's text.
use super::super::plan_insert::plan_with_entries;
/// Imports the guarded write of the plan.
use super::super::publish::{
    Revision,
    publish_plan,
    read_optional,
    revision_of,
};
/// Imports the scope whose plan file is written.
use super::super::scope::Scope;

/// Imports the route a request's prefix selects.
use super::the_routes_a_request_can_add::addable_route;
/// Imports why a new entry cannot be added at all,
/// and the sentence for a plan that lacks the installer.
use super::the_words_a_refusal_says::{
    AddRefusal,
    missing_installer,
};
/// Imports the entry a request describes,
/// and what an argument names.
use super::what_an_argument_names::{
    Target,
    entry_from_request,
    target,
};
/// What:
/// What adding the arguments did.
/// Why:
/// `install` prints one line per added entry,
/// and a frozen run resolves the entries its own command line added,
/// so the ids travel back to the caller.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AddReport {
    /// The entries this run added to the plan,
    /// in the order given.
    pub added: Vec<Declaration>,
    /// The ids the plan already held,
    /// in the order given.
    pub existing: Vec<String>,
}

/// The methods of `AddReport`.
impl AddReport {
    /// The ids of the entries this run added.
    pub fn added_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for declaration in &self.added {
            ids.push(declaration.id.clone());
        }
        return ids;
    }
}

/// The name a plan entry declares twice,
/// or `None`:
/// the tool id every entry shares,
/// and each binary name.
fn collision(
    plan: &Plan,
    pending: &[Declaration],
) -> Option<String> {
    for declaration in pending {
        if plan
            .tools
            .iter()
            .any(|existing| return existing.id == declaration.id)
        {
            return Some(declaration.id.clone());
        }
        if let Some(existing) = plan.tools.iter().find(|existing| {
            return existing
                .bins
                .iter()
                .any(|bin| return declaration.bins.contains(bin));
        }) {
            return Some(format!(
                "{} (the binary names overlap with those of {})",
                declaration.id, existing.id
            ));
        }
    }
    for (index, declaration) in pending.iter().enumerate() {
        for other in &pending[index + 1..] {
            if declaration
                .bins
                .iter()
                .any(|bin| return other.bins.contains(bin))
            {
                return Some(format!(
                    "{} (the binary names overlap with those of {})",
                    other.id, declaration.id
                ));
            }
        }
    }
    return None;
}

/// What:
/// The fault point the real command passes:
/// nothing stops the run between the plan's write and the lock's.
/// Why:
/// `add_named_tools` takes that step as a parameter so a test can stop the command exactly where a kill would,
/// and a run that carries on says so by naming this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const nothingStopsTheRun = () => undefined;
/// ```
pub fn nothing_stops_the_run() -> Result<(), ToolchainError> {
    return Ok(());
}

/// What:
/// Adds every argument the plan does not hold yet to the plan file,
/// and returns what each one named.
/// Why:
/// This is the write `mpm i -g <tool>` does before the install:
/// the plan is read once,
/// the entries are appended as text with everything else kept,
/// and the file is published under the scope's OS lock only if no other mpm-shadow wrote it meanwhile.
/// A request that cannot be described,
/// or a run that may not write,
/// refuses here,
/// before the plan is touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addNamedTools(context: Context, arguments: string[], refusal: AddRefusal | undefined, afterPlanWrite: () => void): AddReport
/// ```
pub fn add_named_tools<F>(
    context: &Context,
    arguments: &[String],
    refusal: Option<AddRefusal>,
    after_plan_write: F,
) -> Result<AddReport, ToolchainError>
where
    F: FnOnce() -> Result<(), ToolchainError>,
{
    let scope: &Scope = context.scope;
    let label: String = scope.plan_file.display().to_string();
    let bytes: Option<Vec<u8>> = read_optional(&scope.plan_file)?;
    let Some(content) = bytes else {
        return Err(ToolchainError::config(format!(
            "the toolchain plan {label} does not exist"
        )));
    };
    let text: String = match String::from_utf8(content.clone()) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Err(ToolchainError::config(format!(
                "{label} is not valid UTF-8: {error}"
            )));
        }
    };
    let plan: Plan = parse_plan(&text, &label)?;
    let mut report: AddReport = AddReport::default();
    for argument in arguments {
        match target(&plan, argument, &label)? {
            Target::Existing(id) => {
                if !report.existing.contains(&id) {
                    report.existing.push(id);
                }
            }
            Target::Request(request) => {
                if let Some(reason) = refusal {
                    return Err(ToolchainError::config(format!(
                        "{} was not added: {}",
                        request.text(),
                        reason.reason()
                    )));
                }
                if report
                    .added
                    .iter()
                    .any(|added| return added.id == request.source)
                {
                    continue;
                }
                if let Some(route) = addable_route(&request.source)
                    && let Some(installer) = route.installs_with
                    && !plan.tools.iter().any(|tool| return tool.id == installer)
                {
                    return Err(missing_installer(argument, installer, &label));
                }
                report.added.push(entry_from_request(context, request)?);
            }
        }
    }
    if report.added.is_empty() {
        return Ok(report);
    }
    if let Some(name) = collision(&plan, &report.added) {
        return Err(ToolchainError::config(format!(
            "{label}: a plan holds each tool id and each binary name once, and {name} is already there, so nothing was added"
        )));
    }
    let updated: String = plan_with_entries(&text, &label, &report.added)?;
    let expected: Revision = revision_of(&Some(content));
    publish_plan(scope, &expected, &updated)?;
    after_plan_write()?;
    return Ok(report);
}
