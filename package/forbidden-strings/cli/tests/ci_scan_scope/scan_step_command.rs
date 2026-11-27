//! What:
//!  The workflow's own `Scan changed files` shell body,
//!  extracted from `.github/workflows/forbidden-strings.yml` and handed to
//!  `bash` exactly as the Actions runner would run it.
//! Why:
//!  Testing a re-implementation would prove nothing about what CI runs;
//!  the workflow file is the single source of the scope logic,
//!  so the heading it keys on and the extraction live together here,
//!  and a renamed step fails loudly instead of silently testing nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function scanStepBody(): string  // the YAML block scalar, dedented
//! ```

use std::fs;
use std::path::PathBuf;

/// What:
///  `pub(super) const SCAN_STEP_NAME` is the step heading the extraction keys
///  on.
/// Why:
///  Keying on the human-facing step name keeps the extraction independent of
///  YAML formatting while failing loudly when the step is renamed.
pub(super) const SCAN_STEP_NAME: &str = "    - name: Scan changed files\n";

// What:     `fn scan_step_body()` returns the workflow's scan-step shell body
//           exactly as the Actions runner executes it.
// Why:      Testing a re-implementation would prove nothing about what CI
//           runs; the workflow file is the single source of the logic.
pub(super) fn scan_step_body() -> String {
    let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.github/workflows/forbidden-strings.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("read workflow file");
    let step_start = workflow
        .find(SCAN_STEP_NAME)
        .expect("workflow must keep its `Scan changed files` step");
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
        lines.push(stripped.to_string());
    }
    while !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }
    return lines.join("\n");
}
