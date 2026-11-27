//! What:
//!  The workflow's own `run` bodies for the two steps that select and scan
//!  changed entries,
//!  extracted from `.github/workflows/forbidden-strings.yml` and handed to
//!  `bash` exactly as the Actions runner would run them.
//! Why:
//!  Testing a re-implementation would prove nothing about what CI runs;
//!  the workflow file is the single source of the scope logic,
//!  so the headings the extraction keys on and the extraction live together
//!  here,
//!  and a renamed step fails loudly instead of silently testing nothing.
//!  The logic sits in two steps because the repository's linter caps one
//!  shell step at 58 code lines;
//!  `main`'s single step already used all 58 (`shell/max-lines`),
//!  so the tests run both bodies in order,
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

// What:     `fn step_body(name)` returns one workflow step's shell body
//           exactly as the Actions runner executes it.
// Why:      Testing a re-implementation would prove nothing about what CI
//           runs; the workflow file is the single source of the logic.
pub(super) fn step_body(name: &str) -> String {
    let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.github/workflows/forbidden-strings.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("read workflow file");
    let step_start = workflow
        .find(name)
        .unwrap_or_else(|| panic!("workflow must keep its `{}` step", name.trim()));
    let from_step = &workflow[step_start..];
    let step_end = match from_step.find("\n    - name: ") {
        Some(index) => index + 1,
        None => from_step.len(),
    };
    let step = &from_step[..step_end];
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
        // `line` borrows from `step`, which borrows from `workflow`; copying
        // each stripped line into an owned `String` (the sibling `&str` is a
        // borrow that cannot outlive `body`) lets the caller join them freely.
        lines.push(stripped.to_string());
    }
    while !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }
    return lines.join("\n");
}
