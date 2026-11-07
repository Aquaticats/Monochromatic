//! What:
//! Generators and invariants for the boundaries `markdown/autofix` added:
//! the
//!       in-process linter run (its failure status and explanation,
//! its fixed source and its
//!       findings) the policy reads,
//! the event form of a pathname that is not UTF-8,
//! and the
//!       one-rule linter configuration built from repository-chosen exclude patterns.
//! Why:
//!  The linter's run crosses into the policy as typed values,
//! but the policy must not trust
//!       them:
//! a failure status,
//! an empty fixed source for a non-empty file,
//! or a finding that
//!       is not a warning of the selected rule with a position must be refused,
//! and pathnames and
//!       exclude patterns are chosen by whoever writes the repository and cross into JSON.
//!       Each check states its property without calling the code under test a second time:
//!       a run's verdict is restated from the contract in `markdown_linter_output.rs`'s module
//!       comment,
//! an event line is parsed back and its base64 decoded by a decoder written here,
//!       and a configuration is parsed back and compared with the patterns that built it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkLinterRun(data); checkGeneratedRecords(data); checkEventPath(data); checkLinterConfig(data);
//! ```

/// Import the selectable rules.
use git_policy_cli::config_schema::MarkdownRule;
/// Import the event form of a pathname and its encoder.
use git_policy_cli::event_path::{EventPath, base64_standard};
/// Import the one-rule configuration builder.
use git_policy_cli::markdown_linter_config::{MARKDOWN_FILE_PATTERNS, one_rule_configuration};
/// Import the reading of one in-process linter run,
/// its explanation bound and its size bounds.
use git_policy_cli::markdown_linter_output::{
    EXPLANATION_LIMIT, LintRun, MAX_FINDINGS_BYTES, MAX_FIXED_BYTES, RemainingFinding,
    interpret_embedded_run,
};
/// Import the finding event and its renderer.
use git_policy_cli::policy_events::{FindingEvent, PolicyEvent, render_policy_event};
/// Import the policy and lifecycle names a finding event carries.
use git_policy_cli::policy_registry::{PolicyId, Severity};
use git_policy_cli::policy_trigger::Trigger;
/// Import the JSONC parser and value model,
/// used as an independent reader.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncValue, parse_jsonc, units_to_string};
/// Import the linter's finding model and the two results of one in-process run.
use monochromatic_lint::diagnostic::{Diagnostic, Label, Severity as LintSeverity, Span};
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRun};

/// The rules every check selects:
/// the shipped one.
const RULES: &[MarkdownRule] = &[MarkdownRule::LfsImageUrl];

/// The finding code of the selected rule,
/// restated from the linter's `markdown/` prefix and the rule's name.
const SELECTED_CODE: &str = "markdown/lfs-image-url";

/// The standard base64 alphabet,
/// restated for the decoder below.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The members of a record named `key`.
fn members<'a>(entries: &'a [JsoncEntry], key: &str) -> Vec<&'a JsoncValue> {
    let mut found: Vec<&'a JsoncValue> = Vec::new();
    for entry in entries {
        if let Ok(name) = units_to_string(&entry.key.units)
            && name == key
        {
            found.push(&entry.value);
        }
    }
    return found;
}

/// The one member named `key`,
/// when there is exactly one.
fn only<'a>(entries: &'a [JsoncEntry], key: &str) -> Option<&'a JsoncValue> {
    let found: Vec<&'a JsoncValue> = members(entries, key);
    if found.len() == 1 {
        return Some(found[0]);
    }
    return None;
}

/// The valid text of a string value.
fn text(value: &JsoncValue) -> Option<String> {
    return units_to_string(value.text_units()?).ok();
}

/// The ways a generated finding can break the contract,
/// by the byte that chooses one.
pub const CORRUPTIONS: [&str; 6] = [
    "other rule",
    "no code",
    "severity error",
    "no labels",
    "unprefixed rule",
    "rule with a suffix",
];

/// What:
/// One finding of a generated run and the corruption planted in it,
/// if any.
/// Why:
///  The oracle of the generated view is the corruption the generator planted,
/// not a reading of the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedFinding = { finding: Diagnostic; corruption?: string };
/// ```
#[derive(Clone, Debug)]
pub struct GeneratedFinding {
    /// The finding as the linter would report it.
    pub finding: Diagnostic,
    /// The corruption that makes the policy refuse it,
    /// or nothing for a usable warning.
    pub corruption: Option<&'static str>,
}

/// What:
/// The finding one 9-byte chunk of fuzz bytes describes.
/// Why:
///  Line and column are any whole numbers (the largest `usize` included),
/// the message carries quotes,
///       backslashes,
/// control and astral characters,
/// a chooser at or above 224 plants one corruption,
/// and an odd
///       line adds a second label,
/// which the policy must not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedFinding(chunk: Buffer): GeneratedFinding;
/// ```
fn generated_finding(chunk: &[u8]) -> GeneratedFinding {
    let first: u8 = *chunk.first().unwrap_or(&1);
    let line: usize = if first == 255 { usize::MAX } else { usize::from(first) };
    let column: usize = usize::from(*chunk.get(1).unwrap_or(&1)) << 20;
    let chooser: u8 = *chunk.get(2).unwrap_or(&0);
    let message: String =
        String::from_utf8_lossy(chunk.get(3..).unwrap_or(&[])).into_owned() + "\"\\\u{1}😀";
    let corruption: Option<&'static str> = if chooser >= 224 {
        Some(CORRUPTIONS[usize::from(chooser - 224) % CORRUPTIONS.len()])
    } else {
        None
    };
    let code: &str = match corruption {
        Some("other rule") => "markdown/no-bare-urls",
        Some("no code") => "",
        Some("unprefixed rule") => "lfs-image-url",
        Some("rule with a suffix") => "markdown/lfs-image-url-extra",
        _ => SELECTED_CODE,
    };
    let severity: LintSeverity = if corruption == Some("severity error") {
        LintSeverity::Error
    } else {
        LintSeverity::Warn
    };
    let mut finding: Diagnostic = Diagnostic::new(
        code,
        severity,
        message,
        String::from("pkg/README.md"),
        Span { offset: 0, length: 1, line, column },
    );
    if corruption == Some("no labels") {
        finding.labels.clear();
    } else if line % 2 == 1 {
        finding.labels.push(Label {
            span: Span { offset: 5, length: 2, line: 7, column: 9 },
        });
    }
    return GeneratedFinding { finding, corruption };
}

/// The finding of a generated finding,
/// as a named function for `map`.
fn finding_of(generated: GeneratedFinding) -> Diagnostic {
    return generated.finding;
}

/// What:
/// The findings of up to six chunks of fuzz bytes.
/// Why:
///  Raw bytes almost never form a finding the policy accepts;
/// this reaches every field and every refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedFindings(data: Buffer): GeneratedFinding[];
/// ```
pub fn generated_findings(data: &[u8]) -> Vec<GeneratedFinding> {
    let mut found: Vec<GeneratedFinding> = Vec::new();
    for chunk in data.chunks(9).take(6) {
        found.push(generated_finding(chunk));
    }
    return found;
}

/// What:
/// The finding a usable run may carry,
/// restated from the contract:
/// a warning of the selected rule,
/// with a
///       position taken from its first label.
/// Why:
///  Restated from `markdown_linter_output.rs`'s module comment,
/// not from its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contractFinding(finding: Diagnostic): RemainingFinding | undefined;
/// ```
fn contract_finding(finding: &Diagnostic) -> Option<RemainingFinding> {
    if finding.code != SELECTED_CODE || finding.severity != LintSeverity::Warn {
        return None;
    }
    let label: &Label = finding.labels.first()?;
    return Some(RemainingFinding {
        rule: MarkdownRule::LfsImageUrl,
        line: u64::try_from(label.span.line).ok()?,
        column: u64::try_from(label.span.column).ok()?,
        message: finding.message.clone(),
    });
}

/// What:
/// What one in-process run produced,
/// as plain fields:
/// a failure status,
/// or a fixed source and findings.
/// Why:
///  The outcome type owns its parts and is consumed by the reader,
/// so the expectation is computed from this copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunSpec = { failure?: { status: number; explanation: string }; fixed: string; findings: Diagnostic[] };
/// ```
#[derive(Clone, Debug)]
pub struct RunSpec {
    /// The status of a run that did not end well,
    /// or nothing for a run the executable would end with status 0.
    pub failure: Option<u8>,
    /// The linter's explanation of a failed run.
    pub explanation: String,
    /// The source after every fix.
    pub fixed: String,
    /// The findings the fixes left.
    pub findings: Vec<Diagnostic>,
}

impl RunSpec {
    /// The result the linter library would return for these fields.
    fn outcome(&self) -> Result<EmbeddedRun, EmbeddedFailure> {
        if let Some(exit_status) = self.failure {
            return Err(EmbeddedFailure { exit_status, explanation: self.explanation.clone() });
        }
        return Ok(EmbeddedRun { fixed: self.fixed.clone(), findings: self.findings.clone() });
    }
}

/// The explanation as the policy quotes it:
/// whole up to the bound,
/// else the first characters and an ellipsis.
fn quoted_explanation(explanation: &str) -> String {
    if explanation.chars().count() <= EXPLANATION_LIMIT {
        return String::from(explanation);
    }
    let kept: String = explanation.chars().take(EXPLANATION_LIMIT).collect();
    return format!("{kept}...");
}

/// What:
/// Assert the reader's verdict on one in-process run against the contract.
/// Why:
///  Only a status-0 run with a source that is not empty for a non-empty input,
/// and findings that are all warnings
///       of the selected rule with a position,
/// is usable,
/// and then exactly as stated;
/// every other run is refused with
///       a reason that names what is wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRun(input: Buffer, spec: RunSpec): void;
/// ```
pub fn check_run(input: &[u8], spec: &RunSpec) {
    let verdict: Result<LintRun, String> = interpret_embedded_run(input, RULES, spec.outcome());
    if let Some(status) = spec.failure {
        let reason: String = verdict.expect_err("a failed run is refused");
        let quoted: String = quoted_explanation(spec.explanation.as_str());
        assert!(reason.ends_with(quoted.as_str()), "the reason quotes the explanation: {reason}");
        if status == 2 {
            assert!(reason.contains("could not finish (exit status 2)"), "{reason}");
        } else {
            assert!(reason.contains(format!("exited with status {status}").as_str()), "{reason}");
        }
        return;
    }
    if spec.fixed.is_empty() && !input.is_empty() {
        let reason: String = verdict.expect_err("an empty fixed source for a non-empty file is refused");
        assert!(reason.contains("empty fixed source"), "{reason}");
        return;
    }
    let mut expected: Vec<RemainingFinding> = Vec::new();
    for (index, finding) in spec.findings.iter().enumerate() {
        match contract_finding(finding) {
            Some(usable) => expected.push(usable),
            None => {
                let reason: String =
                    verdict.expect_err("a finding the contract refuses refuses the run");
                assert!(reason.contains(format!("finding {} ", index + 1).as_str()), "{reason}");
                return;
            }
        }
    }
    let run: LintRun = verdict.expect("a run the contract accepts is used");
    assert_eq!(run.fixed, spec.fixed.as_bytes(), "the fixed source is the output");
    assert_eq!(run.remaining, expected, "the findings are the warnings, in order");
}

/// What:
/// Assert the reader's invariants over raw bytes:
/// the first byte picks how the run ended,
/// the second where the
///       fixed source ends,
/// the rest is the explanation of a failure or the findings of a run.
/// Why:
///  Any outcome must never panic the reader or be accepted against the contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkLinterRun(data: Buffer): void;
/// ```
pub fn check_linter_run(data: &[u8]) {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let Some((&split, streams)) = rest.split_first() else {
        return;
    };
    let cut: usize = usize::from(split).min(streams.len());
    let tail: &[u8] = &streams[cut..];
    // Three quarters of inputs end well, so the findings are reached most of the time.
    let failure: Option<u8> = match selector {
        0..=191 => None,
        192..=207 => Some(1),
        208..=223 => Some(2),
        _ => Some(selector),
    };
    let spec: RunSpec = RunSpec {
        failure,
        explanation: String::from_utf8_lossy(tail).into_owned(),
        fixed: String::from_utf8_lossy(&streams[..cut]).into_owned(),
        findings: generated_findings(tail)
            .into_iter()
            .map(finding_of)
            .collect(),
    };
    // The input is the output itself, or empty, so both source rules are reached.
    check_run(spec.fixed.as_bytes(), &spec);
    check_run(b"", &spec);
}

/// What:
/// Assert the reader's verdict on generated findings.
/// Why:
///  Clean findings come back exactly,
/// in order;
/// a corrupted one is refused,
/// and the refusal names its position.
///       The expectation is the corruption the generator planted,
/// not a reading of the finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedRecords(data: Buffer): void;
/// ```
pub fn check_generated_records(data: &[u8]) {
    let generated: Vec<GeneratedFinding> = generated_findings(data);
    let spec: RunSpec = RunSpec {
        failure: None,
        explanation: String::new(),
        fixed: String::from("x"),
        findings: generated.iter().cloned().map(finding_of).collect(),
    };
    check_run(b"x", &spec);
    let verdict: Result<LintRun, String> = interpret_embedded_run(b"x", RULES, spec.outcome());
    let mut first_corrupted: Option<(usize, &'static str)> = None;
    for (index, entry) in generated.iter().enumerate() {
        if let Some(kind) = entry.corruption {
            first_corrupted = Some((index, kind));
            break;
        }
    }
    match (verdict, first_corrupted) {
        (Ok(run), None) => {
            assert_eq!(run.remaining.len(), generated.len());
            for (usable, entry) in run.remaining.iter().zip(generated.iter()) {
                assert_eq!(usable.message, entry.finding.message);
            }
        }
        (Err(reason), Some((index, kind))) => assert!(
            reason.contains(format!("finding {} ", index + 1).as_str()),
            "{kind}: {reason}"
        ),
        (outcome, expected) => panic!("{outcome:?} for {expected:?}"),
    }
}

/// What:
/// Assert that a run over the size bounds is refused whatever else it holds.
/// Why:
///  The bounds are the former child's output caps;
/// fuzz inputs are far too small to reach them,
/// so a control
///       builds the two oversized runs the bounds describe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOversizedRuns(): void;
/// ```
pub fn check_oversized_runs() {
    let fixed_too_large: RunSpec = RunSpec {
        failure: None,
        explanation: String::new(),
        fixed: "a".repeat(MAX_FIXED_BYTES + 1),
        findings: Vec::new(),
    };
    let reason: String = interpret_embedded_run(b"a", RULES, fixed_too_large.outcome())
        .expect_err("a fixed source over the bound is refused");
    assert!(reason.contains(MAX_FIXED_BYTES.to_string().as_str()), "{reason}");
    let mut large_finding: GeneratedFinding = generated_finding(&[1, 1, 0]);
    large_finding.finding.message = "m".repeat(MAX_FINDINGS_BYTES + 1);
    let findings_too_large: RunSpec = RunSpec {
        failure: None,
        explanation: String::new(),
        fixed: String::from("x"),
        findings: vec![large_finding.finding],
    };
    let findings_reason: String = interpret_embedded_run(b"x", RULES, findings_too_large.outcome())
        .expect_err("findings over the bound are refused");
    assert!(
        findings_reason.contains(MAX_FINDINGS_BYTES.to_string().as_str()),
        "{findings_reason}"
    );
}

/// The six-bit value of one base64 alphabet character,
/// or nothing for any other byte.
fn alphabet_value(character: u8) -> Option<u32> {
    for (value, letter) in ALPHABET.iter().enumerate() {
        if *letter == character {
            return u32::try_from(value).ok();
        }
    }
    return None;
}

/// The bytes a base64 text decodes to,
/// as a named function for `and_then`.
fn decoded_entry(entry: String) -> Option<Vec<u8>> {
    return decode_base64(entry.as_str());
}

/// Whether one byte separates two generated patterns.
fn is_separator(byte: &u8) -> bool {
    return *byte == 0xff;
}

/// What:
/// Decode standard padded base64,
/// restated from RFC 4648 section 4.
/// Why:
///  The encoder is checked against a decoder that shares none of its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decode = (text: string) => Buffer.from(text, 'base64');
/// ```
pub fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let bytes: &[u8] = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut decoded: Vec<u8> = Vec::new();
    for group in bytes.chunks(4) {
        let mut bits: u32 = 0;
        let mut padding: usize = 0;
        for (position, character) in group.iter().enumerate() {
            let value: u32 = if *character == b'=' && position >= 2 {
                padding += 1;
                0
            } else {
                if padding > 0 {
                    return None;
                }
                alphabet_value(*character)?
            };
            bits = (bits << 6) | value;
        }
        let full: [u8; 3] = [(bits >> 16) as u8, (bits >> 8) as u8, bits as u8];
        decoded.extend_from_slice(&full[..3 - padding]);
    }
    return Some(decoded);
}

/// The members of a parsed event line,
/// by key.
fn event_members(line: &str) -> Vec<JsoncEntry> {
    let parsed: JsoncValue = parse_jsonc(line).expect("every event line is JSON");
    return parsed.entries().expect("every event is an object").to_vec();
}

/// What:
/// Assert the event invariants of a pathname of any bytes.
/// Why:
///  `path` is always the replaced text,
/// `pathBytes` is present exactly when that text
///       is not the name,
/// and its base64 decodes back to the name;
/// a fix summary's byte list
///       lines up with its text list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkEventPath(data: Buffer): void;
/// ```
pub fn check_event_path(data: &[u8]) {
    let path: EventPath = EventPath::from_git_bytes(data);
    let lossy: String = String::from_utf8_lossy(data).into_owned();
    let not_text: bool = std::str::from_utf8(data).is_err();
    assert_eq!(path.text(), lossy.as_str());
    assert_eq!(path.exact().is_some(), not_text);
    assert_eq!(path.bytes(), data);
    let encoded: String = base64_standard(data);
    assert_eq!(encoded.len(), data.len().div_ceil(3) * 4);
    assert_eq!(decode_base64(encoded.as_str()).as_deref(), Some(data));
    let line: String = render_policy_event(
        0,
        &PolicyEvent::Finding(FindingEvent {
            trigger: Trigger::DirectCheck,
            policy: PolicyId::MarkdownAutofix,
            severity: Severity::Warn,
            code: "markdown-autofix",
            message: String::from("m"),
            path: Some(path.clone()),
            location: None,
            fix_available: false,
        }),
    );
    assert!(
        line.ends_with("}\n") && line.matches('\n').count() == 1,
        "{line:?}"
    );
    let entries: Vec<JsoncEntry> = event_members(line.trim_end());
    assert_eq!(
        only(&entries, "path").and_then(text).as_deref(),
        Some(lossy.as_str())
    );
    let bytes: Option<String> = only(&entries, "pathBytes").and_then(text);
    assert_eq!(bytes.is_some(), not_text);
    if let Some(field) = bytes {
        assert_eq!(decode_base64(field.as_str()).as_deref(), Some(data));
    }
    let summary: String = render_policy_event(
        0,
        &PolicyEvent::FixSummary {
            trigger: Trigger::DirectFix,
            passes: 1,
            changed_paths: vec![EventPath::from_git_bytes(b"a.txt"), path],
        },
    );
    let summary_entries: Vec<JsoncEntry> = event_members(summary.trim_end());
    match only(&summary_entries, "changedPathBytes").and_then(JsoncValue::elements) {
        Some(listed) => {
            assert!(not_text);
            assert_eq!(listed.len(), 2);
            assert_eq!(text(&listed[0]).as_deref(), Some("YS50eHQ="));
            assert_eq!(
                text(&listed[1]).and_then(decoded_entry).as_deref(),
                Some(data)
            );
        }
        None => assert!(!not_text),
    }
}

/// What:
/// Exclude patterns from fuzz bytes:
/// each `0xff`-separated piece,
/// read as text.
/// Why:
///  Patterns reach the configuration as repository-chosen text of any characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const patterns = (data: Buffer) => data.toString().split('�');
/// ```
pub fn generated_patterns(data: &[u8]) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::new();
    for piece in data.split(is_separator) {
        patterns.push(String::from_utf8_lossy(piece).into_owned());
    }
    return patterns;
}

/// What:
/// Assert that a configuration built from any patterns is one block that selects the
///       Markdown files and the LFS rule at `warn` with exactly those patterns.
/// Why:
///  No pattern text may change the document's structure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkLinterConfig(data: Buffer): void;
/// ```
pub fn check_linter_config(data: &[u8]) {
    let patterns: Vec<String> = generated_patterns(data);
    let document: String = one_rule_configuration(RULES, patterns.as_slice());
    let parsed: JsoncValue = parse_jsonc(document.as_str()).expect("the configuration is JSON");
    let blocks: &[JsoncValue] = parsed.elements().expect("a list of blocks");
    assert_eq!(blocks.len(), 1);
    let block: &[JsoncEntry] = blocks[0].entries().expect("a block object");
    assert_eq!(block.len(), 2);
    let mut files: Vec<String> = Vec::new();
    for value in only(block, "files")
        .and_then(JsoncValue::elements)
        .expect("files")
    {
        files.push(text(value).expect("a file pattern"));
    }
    assert_eq!(files, MARKDOWN_FILE_PATTERNS);
    let rules: &[JsoncEntry] = only(block, "rules")
        .and_then(JsoncValue::entries)
        .expect("rules");
    assert_eq!(rules.len(), 1);
    let lfs: &[JsoncEntry] = only(rules, "markdown/lfs-image-url")
        .and_then(JsoncValue::entries)
        .expect("the LFS rule");
    assert_eq!(lfs.len(), 2);
    assert_eq!(
        only(lfs, "severity").and_then(text).as_deref(),
        Some("warn")
    );
    let mut excluded: Vec<String> = Vec::new();
    for value in only(lfs, "exclude")
        .and_then(JsoncValue::elements)
        .expect("exclude")
    {
        excluded.push(text(value).expect("a pattern"));
    }
    assert_eq!(excluded, patterns);
}

/// Generator reach and fixed hard cases stay out of the fuzz targets.
#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;
