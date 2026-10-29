//! What:
//!  Extraction of virtual shell files from the steps of a workflow or composite action.
//! Why:
//!  Each shell step becomes `<workflow>/<n>.<suffix>`,
//!  the same virtual-file model as fenced Rust in Markdown,
//!  so configuration selects steps by glob and findings map back to the workflow's own bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function steps(parent: Mapping): Mapping[]; // throws ProcessorError
//! ```

/// Import the exact line copier.
use crate::processors_lines::copy_line;
/// Import the immutable mapping records.
use crate::processors_model::{Guard, MappedLine, Mapping, ProcessorError, ProcessorLanguage};
/// Import the dialect classification.
use crate::shell_dialect::{ShellClass, ShellDialect, classify_shell};
/// Import the records the walk returns.
use crate::workflow_model::{RunBody, Step, WalkError, Workflow};
/// Import the walk.
use crate::workflow_walk::walk;
/// Share exact parent snapshots between sibling steps.
use std::sync::Arc;

/// What:
///  Decide the dialect a step's script is written in.
/// Why:
///  The step's own `shell:` wins,
///  then the job's `defaults.run.shell`,
///  then the workflow's.
///  With none, a Windows runner means PowerShell and every other runner means a POSIX shell.
///  `Ok(None)` is a non-shell interpreter, which is out of scope;
///  a shell the rules cannot classify is an error, so the ban fails closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dialectFor(workflow: Workflow, step: Step): ShellDialect | undefined; // throws WalkError
/// ```
fn dialect_for(workflow: &Workflow, step: &Step) -> Result<Option<ShellDialect>, WalkError> {
    let mut chosen = step.shell.as_ref();
    let mut windows: bool = false;
    if let Some(index) = step.job {
        let job = &workflow.jobs[index];
        windows = job.runs_on.contains("windows");
        if chosen.is_none() {
            chosen = job.shell.as_ref();
        }
    }
    if chosen.is_none() {
        chosen = workflow.shell.as_ref();
    }
    let Some(shell) = chosen else {
        if windows {
            return Ok(Some(ShellDialect::PowerShell));
        }
        return Ok(Some(ShellDialect::Posix));
    };
    match classify_shell(shell.value.as_str()) {
        ShellClass::Dialect(dialect) => return Ok(Some(dialect)),
        ShellClass::OutOfScope => return Ok(None),
        ShellClass::Unknown(reason) => {
            return Err(WalkError {
                offset: shell.offset,
                message: format!("cannot tell which shell runs this step: {reason}"),
            });
        }
    }
}

/// Copy a step's script into an empty child mapping,
///  keeping each removed indentation as the line prefix.
fn fill(child: &mut Mapping, parent: &Mapping, body: &RunBody) {
    match body {
        RunBody::Plain { start, end } => {
            child.text.push_str(&parent.text[*start..*end]);
            child.lines.push(MappedLine {
                start: 0,
                end: child.text.len(),
                parent_start: *start,
                envelope: *start,
                prefix: String::new(),
            });
        }
        RunBody::Block { lines } => {
            for line in lines {
                let prefix: String = String::from(&parent.text[line.envelope..line.payload]);
                copy_line(
                    child,
                    parent.text.as_str(),
                    line.envelope,
                    line.payload,
                    line.end,
                    prefix,
                );
            }
        }
    }
}

/// What:
///  Extract one virtual shell file for every in-scope step that has a script.
/// Why:
///  A form the walk cannot read is a refusal for the whole file:
///  a skipped step would hide a long script from the rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(parent: Mapping): Mapping[]; // throws ProcessorError
/// ```
pub(crate) fn steps(parent: &Arc<Mapping>) -> Result<Vec<Mapping>, ProcessorError> {
    let workflow: Workflow = match walk(parent.text.as_str()) {
        Ok(found) => found,
        Err(error) => return Err(parent.error_at(error.offset, error.message.as_str())),
    };
    let mut result: Vec<Mapping> = Vec::new();
    for step in &workflow.steps {
        let Some(body) = &step.body else {
            continue;
        };
        let dialect: ShellDialect = match dialect_for(&workflow, step) {
            Ok(Some(found)) => found,
            Ok(None) => continue,
            Err(error) => return Err(parent.error_at(error.offset, error.message.as_str())),
        };
        let name: String = format!("{}/{}{}", parent.filename, result.len(), dialect.suffix());
        let mut child: Mapping = crate::processors::child(
            parent,
            name,
            ProcessorLanguage::Shell,
            Guard::Step,
            step.anchor,
        );
        fill(&mut child, parent, body);
        result.push(child);
    }
    return Ok(result);
}

/// Step extraction controls stay outside release artifacts.
#[cfg(test)]
#[path = "processors_workflow_tests.rs"]
mod tests;
