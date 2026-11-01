//! What: Generators and invariants for the boundaries `markdown/autofix` added: the
//!       linter's output (exit status, fixed source and JSON Lines findings) the policy
//!       reads, the event form of a pathname that is not UTF-8, and the one-rule linter
//!       configuration built from repository-chosen exclude patterns.
//! Why: The linter is a child process whose output is untrusted text; pathnames and
//!      exclude patterns are chosen by whoever writes the repository and cross into JSON.
//!      Each check states its property without calling the code under test a second time:
//!      accepted output is re-read field by field with the JSONC parser, an event line is
//!      parsed back and its base64 decoded by a decoder written here, and a configuration
//!      is parsed back and compared with the patterns that built it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkLinterRun(data); checkGeneratedRecords(data); checkEventPath(data); checkLinterConfig(data);
//! ```

/// Import how a bounded child ended and what it wrote.
use git_policy_cli::bounded_child::{ChildExit, FinishedChild};
/// Import the selectable rules.
use git_policy_cli::config_schema::MarkdownRule;
/// Import the event form of a pathname and its encoder.
use git_policy_cli::event_path::{EventPath, base64_standard};
/// Import the one-rule configuration builder.
use git_policy_cli::markdown_linter_config::{MARKDOWN_FILE_PATTERNS, one_rule_configuration};
/// Import the reading of one linter run.
use git_policy_cli::markdown_linter_output::{LintRun, RemainingFinding, interpret_linter_run};
/// Import the finding event and its renderer.
use git_policy_cli::policy_events::{FindingEvent, PolicyEvent, render_policy_event};
/// Import the policy and lifecycle names a finding event carries.
use git_policy_cli::policy_registry::{PolicyId, Severity};
use git_policy_cli::policy_trigger::Trigger;
/// Import the JSONC parser and value model, used as an independent reader.
use monochromatic_jsonc_edit::{
    JsoncEntry, JsoncKey, JsoncValue, emit_jsonc_value, parse_jsonc, units_to_string,
};

/// The rules every check selects: the shipped one.
const RULES: &[MarkdownRule] = &[MarkdownRule::LfsImageUrl];

/// The standard base64 alphabet, restated for the decoder below.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The exit the first input byte selects; three quarters of inputs exit 0, so the record
/// reader is reached most of the time.
fn exit_of(selector: u8) -> ChildExit {
    match selector {
        0..=191 => return ChildExit::Code(0),
        192..=207 => return ChildExit::Code(1),
        208..=223 => return ChildExit::Code(2),
        224..=231 => return ChildExit::Code(i32::from(selector)),
        232..=247 => return ChildExit::Signal(i32::from(selector - 232)),
        _ => return ChildExit::Unknown,
    }
}

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

/// The one member named `key`, when there is exactly one.
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

/// A plain decimal whole number.
fn count(value: &JsoncValue) -> Option<u64> {
    return value.number_token()?.parse::<u64>().ok();
}

/// What: The finding one record states under the documented contract, or nothing when the
///       record breaks it: one object; exactly one `code`, `markdown/lfs-image-url`; one
///       `severity`, `warn`; one valid `message`; one `labels` list whose first element is an
///       object with one `span` object holding one whole `line` and one whole `column`.
/// Why:  Restated from `markdown_linter_output.rs`'s module comment, not from its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contractFinding(line: string): RemainingFinding | undefined;
/// ```
fn contract_finding(line: &str) -> Option<RemainingFinding> {
    let record: JsoncValue = parse_jsonc(line).ok()?;
    let entries: &[JsoncEntry] = record.entries()?;
    if text(only(entries, "code")?)? != "markdown/lfs-image-url" {
        return None;
    }
    if text(only(entries, "severity")?)? != "warn" {
        return None;
    }
    let message: String = text(only(entries, "message")?)?;
    let first: &JsoncValue = only(entries, "labels")?.elements()?.first()?;
    let span: &[JsoncEntry] = only(first.entries()?, "span")?.entries()?;
    return Some(RemainingFinding {
        rule: MarkdownRule::LfsImageUrl,
        line: count(only(span, "line")?)?,
        column: count(only(span, "column")?)?,
        message,
    });
}

/// The findings a whole standard error states under the contract: empty, or UTF-8 lines
/// each ended by LF and each a contract finding.
fn contract_findings(stderr: &[u8]) -> Option<Vec<RemainingFinding>> {
    let text: &str = std::str::from_utf8(stderr).ok()?;
    if text.is_empty() {
        return Some(Vec::new());
    }
    let body: &str = text.strip_suffix('\n')?;
    let mut findings: Vec<RemainingFinding> = Vec::new();
    for line in body.split('\n') {
        findings.push(contract_finding(line)?);
    }
    return Some(findings);
}

/// What: Assert the reader's verdict on one finished run against the contract.
/// Why:  Only exit 0 with a UTF-8 fixed source, not empty for a non-empty input, and a
///       standard error the contract accepts is usable, and then exactly as stated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRun(input: Buffer, finished: FinishedChild): void;
/// ```
fn check_run(input: &[u8], finished: &FinishedChild) {
    let verdict: Result<LintRun, String> = interpret_linter_run(input, RULES, finished);
    let usable_exit: bool = finished.exit == ChildExit::Code(0);
    let usable_source: bool = std::str::from_utf8(&finished.stdout).is_ok()
        && !(finished.stdout.is_empty() && !input.is_empty());
    let expected: Option<Vec<RemainingFinding>> = if usable_exit && usable_source {
        contract_findings(&finished.stderr)
    } else {
        None
    };
    match (verdict, expected) {
        (Ok(run), Some(findings)) => {
            assert_eq!(run.fixed, finished.stdout, "the fixed source is the output");
            assert_eq!(run.remaining, findings, "the findings are the records");
        }
        (Err(reason), None) => assert!(!reason.is_empty(), "a refusal has a reason"),
        (Ok(run), None) => panic!("accepted a run the contract refuses: {finished:?} -> {run:?}"),
        (Err(reason), Some(_)) => {
            panic!("refused a run the contract accepts: {finished:?}: {reason}")
        }
    }
}

/// What: Assert the reader's invariants over raw bytes: the first byte picks the exit,
///       the second where standard output ends, the rest is standard error.
/// Why:  Arbitrary output must never panic the reader or be accepted against the contract.
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
    let finished: FinishedChild = FinishedChild {
        exit: exit_of(selector),
        stdout: streams[..cut].to_vec(),
        stderr: streams[cut..].to_vec(),
    };
    // The input is the output itself, or empty, so both source rules are reached.
    check_run(&finished.stdout, &finished);
    check_run(b"", &finished);
}

/// A JSON string value holding `value`.
fn string(value: &str) -> JsoncValue {
    return JsoncValue::text_from_units(value.encode_utf16().collect());
}

/// A record member.
fn member(key: &str, value: JsoncValue) -> JsoncEntry {
    return JsoncEntry {
        key: JsoncKey::from_text(key),
        value,
    };
}

/// A number value.
fn number(value: u64) -> JsoncValue {
    return JsoncValue::number_from_token(value.to_string().as_str())
        .expect("a decimal is a number");
}

/// The ways a generated record can break the contract, by the byte that chooses one.
pub const CORRUPTIONS: [&str; 8] = [
    "no code",
    "other rule",
    "severity note",
    "line as text",
    "no labels",
    "two codes",
    "not json",
    "no final line feed",
];

/// What: One generated record: the linter's field order, the message from fuzz bytes, and
///       one corruption applied when `corruption` names one.
/// Why:  Messages carry quotes, backslashes, control and astral characters; positions are
///       any whole numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedRecord(message: string, line: number, column: number, corruption?: string): string;
/// ```
fn generated_record(message: &str, line: u64, column: u64, corruption: Option<&str>) -> String {
    let mut entries: Vec<JsoncEntry> = Vec::new();
    entries.push(member("message", string(message)));
    match corruption {
        Some("no code") => {}
        Some("other rule") => entries.push(member("code", string("markdown/no-bare-urls"))),
        Some("two codes") => {
            entries.push(member("code", string("markdown/lfs-image-url")));
            entries.push(member("code", string("markdown/lfs-image-url")));
        }
        _ => entries.push(member("code", string("markdown/lfs-image-url"))),
    }
    let severity: &str = if corruption == Some("severity note") {
        "note"
    } else {
        "warn"
    };
    entries.push(member("severity", string(severity)));
    entries.push(member("causes", JsoncValue::array(Vec::new())));
    entries.push(member("filename", string("pkg/README.md")));
    let line_value: JsoncValue = if corruption == Some("line as text") {
        string(line.to_string().as_str())
    } else {
        number(line)
    };
    let span: JsoncValue = JsoncValue::record(vec![
        member("offset", number(0)),
        member("length", number(1)),
        member("line", line_value),
        member("column", number(column)),
    ]);
    if corruption != Some("no labels") {
        entries.push(member(
            "labels",
            JsoncValue::array(vec![JsoncValue::record(vec![member("span", span)])]),
        ));
    }
    entries.push(member("related", JsoncValue::array(Vec::new())));
    // The emitter may break the record over lines; a JSON Lines record is one line.
    let rendered: String = emit_jsonc_value(&JsoncValue::record(entries)).replace('\n', "");
    if corruption == Some("not json") {
        return format!("{rendered}}}\n");
    }
    return format!("{rendered}\n");
}

/// What: Records generated from fuzz bytes, the standard error they make, the findings it
///       states, and the corruption planted, if any.
/// Why:  Raw bytes almost never form a record; this reaches every field and every refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedRecords(data: Buffer): { stderr: string; findings: RemainingFinding[]; corrupted?: [number, string] };
/// ```
pub fn generated_records(
    data: &[u8],
) -> (String, Vec<RemainingFinding>, Option<(usize, &'static str)>) {
    let mut stderr: String = String::new();
    let mut findings: Vec<RemainingFinding> = Vec::new();
    let mut corrupted: Option<(usize, &'static str)> = None;
    let chunks: Vec<&[u8]> = data.chunks(9).collect();
    for (index, chunk) in chunks.iter().enumerate().take(6) {
        let line: u64 = u64::from(*chunk.first().unwrap_or(&1));
        let column: u64 = u64::from(*chunk.get(1).unwrap_or(&1)) << 20;
        let message: String =
            String::from_utf8_lossy(chunk.get(3..).unwrap_or(&[])).into_owned() + "\"\\\u{1}😀";
        let chooser: u8 = *chunk.get(2).unwrap_or(&0);
        let corruption: Option<&'static str> = if corrupted.is_none() && chooser >= 224 {
            Some(CORRUPTIONS[usize::from(chooser - 224) % CORRUPTIONS.len()])
        } else {
            None
        };
        let mut record: String = generated_record(message.as_str(), line, column, corruption);
        if corruption == Some("no final line feed") {
            // Only the last record can lack its line feed; earlier ones are cut there.
            record.pop();
            stderr.push_str(record.as_str());
            corrupted = Some((index, "no final line feed"));
            break;
        }
        stderr.push_str(record.as_str());
        if let Some(kind) = corruption {
            corrupted = Some((index, kind));
        } else {
            findings.push(RemainingFinding {
                rule: MarkdownRule::LfsImageUrl,
                line,
                column,
                message,
            });
        }
    }
    return (stderr, findings, corrupted);
}

/// What: Assert the reader's verdict on generated records.
/// Why:  Clean records come back exactly, in order; a corrupted one is refused, and the
///       refusal names its line, or the missing line feed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedRecords(data: Buffer): void;
/// ```
pub fn check_generated_records(data: &[u8]) {
    let (stderr, findings, corrupted) = generated_records(data);
    let finished: FinishedChild = FinishedChild {
        exit: ChildExit::Code(0),
        stdout: b"x".to_vec(),
        stderr: stderr.into_bytes(),
    };
    check_run(b"x", &finished);
    match (interpret_linter_run(b"x", RULES, &finished), corrupted) {
        (Ok(run), None) => assert_eq!(run.remaining, findings),
        (Err(reason), Some((_, "no final line feed"))) => assert_eq!(
            reason,
            "monochromatic-lint ended its standard error without a line feed after the last record"
        ),
        (Err(reason), Some((index, kind))) => assert!(
            reason.contains(format!("on line {} of its standard error", index + 1).as_str()),
            "{kind}: {reason}"
        ),
        (verdict, expected) => panic!("{verdict:?} for {expected:?}"),
    }
}

/// The six-bit value of one base64 alphabet character, or nothing for any other byte.
fn alphabet_value(character: u8) -> Option<u32> {
    for (value, letter) in ALPHABET.iter().enumerate() {
        if *letter == character {
            return u32::try_from(value).ok();
        }
    }
    return None;
}

/// The bytes a base64 text decodes to, as a named function for `and_then`.
fn decoded_entry(entry: String) -> Option<Vec<u8>> {
    return decode_base64(entry.as_str());
}

/// Whether one byte separates two generated patterns.
fn is_separator(byte: &u8) -> bool {
    return *byte == 0xff;
}

/// What: Decode standard padded base64, restated from RFC 4648 section 4.
/// Why:  The encoder is checked against a decoder that shares none of its code.
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

/// The members of a parsed event line, by key.
fn event_members(line: &str) -> Vec<JsoncEntry> {
    let parsed: JsoncValue = parse_jsonc(line).expect("every event line is JSON");
    return parsed.entries().expect("every event is an object").to_vec();
}

/// What: Assert the event invariants of a pathname of any bytes.
/// Why:  `path` is always the replaced text, `pathBytes` is present exactly when that text
///       is not the name, and its base64 decodes back to the name; a fix summary's byte list
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

/// What: Exclude patterns from fuzz bytes: each `0xff`-separated piece, read as text.
/// Why:  Patterns reach the configuration as repository-chosen text of any characters.
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

/// What: Assert that a configuration built from any patterns is one block that selects the
///       Markdown files and the LFS rule at `warn` with exactly those patterns.
/// Why:  No pattern text may change the document's structure.
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
