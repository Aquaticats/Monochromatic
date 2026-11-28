//! What:
//!  The workflow's own `run` bodies for the steps that select and scan changed
//!  entries,
//!  extracted from `.github/workflows/forbidden-strings.yml` and handed to
//!  `bash` exactly as the Actions runner would run them.
//! Why:
//!  Testing a re-implementation would prove nothing about what CI runs;
//!  the workflow file is the single source of the scope logic,
//!  so the headings the extraction keys on and the extraction live together
//!  here,
//!  and a renamed step fails loudly instead of silently testing nothing.
//!  The logic sits in four steps because the repository's linter caps one
//!  shell step at 58 code lines;
//!  `main`'s single step already used all 58 (`shell/max-lines`),
//!  so the tests run the bodies in order,
//!  the way the runner does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function stepBody(name: string): string  // the YAML block scalar, dedented
//! ```

use std::fs;
use std::path::PathBuf;

/// What:
///  `pub(super) const SCOPE_STEP_NAME` is the heading of the step that picks
///  the scan scope and writes the raw changed-entry list.
/// Why:
///  Keying on the human-facing step names keeps the extraction independent of
///  YAML formatting while failing loudly when a step is renamed.
pub(super) const SCOPE_STEP_NAME: &str = "    - name: List changed files\n";

/// What:
///  `pub(super) const SCAN_STEP_NAME` is the heading of the step that leaves
///  out entries a content scan cannot open and runs the scanner.
/// Why:
///  This is the step the issue is about;
///  a rename must fail the extraction rather than test nothing.
pub(super) const SCAN_STEP_NAME: &str = "    - name: Scan changed files\n";

/// What:
///  `pub(super) const LINK_LIST_STEP_NAME` is the heading of the step that
///  writes each link's stored target text and each gitlink's name for the step
///  after it,
///  from the changed entries a push or pull request lists or,
///  when no baseline exists,
///  from every link and gitlink of the index.
/// Why:
///  Issue #6640:
///  a delta's link to a file stays in the file scan,
///  and the whole-tree arm has no changed-entry list at all,
///  so one step must bring both arms' links and gitlinks to the scanner;
///  a rename must fail the cases that pin it rather than test nothing.
pub(super) const LINK_LIST_STEP_NAME: &str =
    "    - name: Write each link's stored target text to a file\n";

/// What:
///  `pub(super) const LINK_SCAN_STEP_NAME` is the heading of the step that hands
///  each link and each gitlink to the scanner under its own name,
///  with the materialized target text or an empty file beside it.
/// Why:
///  Issue #6633 left a folder link and a submodule gitlink inspected by nothing
///  once the file scan stopped opening them;
///  issue #6640 adds every other link's stored target text and every link of a
///  whole-tree run.
///  A case must pin that this step still runs,
///  and a rename must fail it rather than test nothing.
pub(super) const LINK_SCAN_STEP_NAME: &str =
    "    - name: Scan each link's stored target text and each gitlink's name\n";

// What:     `fn step_body(name)` returns one workflow step's shell body
//           exactly as the Actions runner executes it,
//           and refuses a step the workflow does not declare.
// Why:      Testing a re-implementation would prove nothing about what CI
//           runs; the workflow file is the single source of the logic.
pub(super) fn step_body(name: &str) -> String {
    return declared_step_body(name)
        .unwrap_or_else(|| panic!("workflow must keep its `{}` step", name.trim()));
}

// What:     `fn step_source(name)` returns the workflow file's own text for
//           the step called `name`, or `None` when the workflow declares no
//           such step.
// Why:      The step's body and the status check beside it come from one slice
//           of the file, so a case can read the condition the workflow really
//           declares instead of a copy of it.
pub(super) fn step_source(name: &str) -> Option<String> {
    let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.github/workflows/forbidden-strings.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("read workflow file");
    let step_start = workflow.find(name)?;
    let from_step = &workflow[step_start..];
    let step_end = match from_step.find("\n    - name: ") {
        Some(index) => index + 1,
        None => from_step.len(),
    };
    return Some(from_step[..step_end].to_string());
}

// What:     `fn step_runs_after_failure(name)` reports whether the step called
//           `name` declares a status check that keeps it running after an
//           earlier step in the job failed.
// Why:      The runner skips a job's later steps once one fails unless the step
//           carries a check such as `!cancelled()` or `always()`, and the
//           fixture must follow the workflow's own declaration: a harness that
//           assumed the check would report a step running after a failure that
//           the workflow never runs, which is how a plant of the check's
//           removal once passed every case.
pub(super) fn step_runs_after_failure(name: &str) -> bool {
    let Some(step) = step_source(name) else {
        return false;
    };
    for line in step.lines() {
        let trimmed: &str = line.trim();
        let Some(condition) = trimmed.strip_prefix("if: ") else {
            continue;
        };
        return condition.contains("!cancelled()") || condition.contains("always()");
    }
    return false;
}

// What:     `fn declared_step_body(name)` returns the body of the step called
//           `name`, or `None` when the workflow declares no such step.
// Why:      The fixture runs every body the workflow declares, so a workflow
//           without one of the later steps still runs its other steps and a
//           case reports what those really hand over, instead of every case
//           failing with the harness's own panic.
pub(super) fn declared_step_body(name: &str) -> Option<String> {
    let step: String = step_source(name)?;
    let run_key = "      run: |\n";
    let run_start = step
        .find(run_key)
        .expect("scan step must keep a `run: |` body");
    let body = &step[run_start + run_key.len()..];
    let mut lines: Vec<String> = Vec::new();
    for line in body.split('\n') {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let Some(stripped) = line.strip_prefix("        ") else {
            panic!("unexpected scan-step line outside the run body: {}", line);
        };
        // `line` borrows from `body`, which borrows from the owned `step`;
        // copying each stripped line into an owned `String` lets the lines
        // outlive that borrow when the caller joins them.
        lines.push(stripped.to_string());
    }
    while !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }
    // What:     `return Some(lines.join("\n"))` wraps the joined body in the
    //           present variant of `Option` and hands it back as the step's
    //           body.
    // Why:      The caller gets the body, and an absent step answers `None`
    //           instead of aborting the whole test binary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return lines.join('\n');
    // ```
    return Some(lines.join("\n"));
}
