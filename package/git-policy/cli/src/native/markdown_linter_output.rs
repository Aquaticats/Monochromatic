//! What:
//!  Read what one `monochromatic-lint --stdin --fix` run produced:
//!  the fixed source on
//!       standard output,
//!  findings it could not fix as JSON Lines on standard error,
//!  and its
//!       exit status.
//!  The result is the fixed bytes and the remaining findings,
//!  or the reason
//!       the run cannot be trusted.
//! Why:
//!  The linter is a child process whose output is untrusted text.
//!  Its contract
//!      (`package/linter/monochromatic-lint/README.md`,
//!  `src/run_stdin.rs`,
//!  `src/run_output.rs`)
//!      is exit status 0 with the fixed source and warning records,
//!  because the policy's
//!      one-rule configuration gives the rule severity `warn`;
//!  1 only for error findings or a
//!      warning limit,
//!  neither of which that configuration can produce;
//!  and 2 for a run that
//!      could not finish.
//!  Anything else,
//!  and any record that is not a finding of a selected
//!      rule,
//!  means the run cannot be trusted,
//!  so the policy fails instead of passing an
//!      unchecked file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = interpretLinterRun({ input, rules, finished }); // throws LinterOutputError
//! ```

/// Import how a bounded child ended and what it wrote.
use super::bounded_child::{ChildExit, FinishedChild};
/// Import the selectable rules and their configuration spelling.
use super::config_schema::{MarkdownRule, markdown_rule_name};
/// What:
///  Import the JSONC parser,
///  its value model and the decoder of its text units.
/// Why:
///   Each JSON Lines record is one JSON value;
///  JSON is a subset of what it reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc } from '@monochromatic-dev/jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncEntry, JsoncValue, parse_jsonc, units_to_string};

/// The prefix of every line the linter writes to standard error that is not a record.
pub const LINTER_MESSAGE_PREFIX: &str = "monochromatic-lint: ";

/// The longest explanation quoted from the linter's standard error,
///  in characters.
pub const EXPLANATION_LIMIT: usize = 400;

/// What:
///  One finding the linter reported and could not fix.
///  `u64` holds the one-based
///       line and column the linter gives.
/// Why:
///   The policy reports it with the incumbent's words,
///  naming the rule as configured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemainingFinding = { rule: MarkdownRule; line: number; column: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemainingFinding {
    /// The rule that reported it.
    pub rule: MarkdownRule,
    /// One-based line of the finding's first label.
    pub line: u64,
    /// One-based column of that label,
    ///  in the linter's units.
    pub column: u64,
    /// The linter's explanation.
    pub message: String,
}

/// What:
///  A run the policy can use:
///  the source after every fix,
///  and what is left.
/// Why:
///   The fixed bytes equal the input exactly when nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LintRun = { fixed: Uint8Array; remaining: RemainingFinding[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintRun {
    /// The fixed source,
    ///  byte for byte as the linter printed it.
    pub fixed: Vec<u8>,
    /// Findings the fixes did not remove,
    ///  in the linter's order.
    pub remaining: Vec<RemainingFinding>,
}

/// What:
///  The one member named `key` of a record,
///  or nothing.
///  `&'a [JsoncEntry]` borrows
///       the members;
///  the result borrows the value for as long.
/// Why:
///   A record that names a field twice is ambiguous and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function member(entries: JsoncEntry[], key: string): JsoncValue | undefined; // throws on a repeat
/// ```
fn member<'a>(entries: &'a [JsoncEntry], key: &str) -> Result<Option<&'a JsoncValue>, String> {
    let mut found: Option<&'a JsoncValue> = None;
    for entry in entries {
        // A key whose units are not valid text cannot be the key sought.
        let matches: bool = match units_to_string(&entry.key.units) {
            Ok(text) => text == key,
            Err(_) => false,
        };
        if !matches {
            continue;
        }
        if found.is_some() {
            return Err(format!("it names \"{key}\" twice"));
        }
        found = Some(&entry.value);
    }
    return Ok(found);
}

/// What:
///  The text of the required member `key`.
/// Why:
///   `code`,
///  `message` and `severity` must be strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requiredText(entries: JsoncEntry[], key: string): string;
/// ```
fn required_text(entries: &[JsoncEntry], key: &str) -> Result<String, String> {
    let Some(value) = member(entries, key)? else {
        return Err(format!("it has no \"{key}\""));
    };
    let Some(units) = value.text_units() else {
        return Err(format!("its \"{key}\" is not a string"));
    };
    match units_to_string(units) {
        Ok(text) => return Ok(text),
        Err(_) => return Err(format!("its \"{key}\" is not valid text")),
    }
}

/// What:
///  The whole number of the required member `key`.
///  `str::parse::<u64>` accepts only
///       plain decimal digits.
/// Why:
///   A line or column that is negative,
///  fractional or exponential is not a position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requiredCount(entries: JsoncEntry[], key: string): number;
/// ```
fn required_count(entries: &[JsoncEntry], key: &str) -> Result<u64, String> {
    let Some(value) = member(entries, key)? else {
        return Err(format!("its span has no \"{key}\""));
    };
    let Some(token) = value.number_token() else {
        return Err(format!("its span's \"{key}\" is not a number"));
    };
    match token.parse::<u64>() {
        Ok(count) => return Ok(count),
        Err(_) => return Err(format!("its span's \"{key}\" is not a whole number")),
    }
}

/// What:
///  The members of the span of a record's first label.
/// Why:
///   The line and column of a finding come from there,
///  as the incumbent took them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstSpan(entries: JsoncEntry[]): JsoncEntry[];
/// ```
fn first_span(entries: &[JsoncEntry]) -> Result<&[JsoncEntry], String> {
    let Some(labels) = member(entries, "labels")? else {
        return Err(String::from("it has no \"labels\""));
    };
    let Some(first) = labels.elements().and_then(<[JsoncValue]>::first) else {
        return Err(String::from("its \"labels\" is not a list with a label"));
    };
    let Some(label) = first.entries() else {
        return Err(String::from("its first label is not an object"));
    };
    let Some(span) = member(label, "span")?.and_then(JsoncValue::entries) else {
        return Err(String::from("its first label has no \"span\" object"));
    };
    return Ok(span);
}

/// What:
///  The selected rule a finding code names.
///  `&[MarkdownRule]` borrows the rules the
///       configuration selected.
/// Why:
///   The linter prefixes every Markdown rule with `markdown/`;
///  a code of a rule the
///       one-rule configuration did not select means the linter ran something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectedRule = (code, rules) => rules.find(rule => code === `markdown/${ruleName(rule)}`);
/// ```
fn selected_rule(code: &str, rules: &[MarkdownRule]) -> Option<MarkdownRule> {
    for rule in rules {
        if code.strip_prefix("markdown/") == Some(markdown_rule_name(*rule)) {
            return Some(*rule);
        }
    }
    return None;
}

/// What:
///  Read one JSON Lines record of a successful run as a finding.
/// Why:
///   Every record must be a warning of a selected rule with a position;
///  anything else
///       is a contract the policy cannot rely on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remainingFinding(line: string, rules: MarkdownRule[]): RemainingFinding;
/// ```
fn remaining_finding(line: &str, rules: &[MarkdownRule]) -> Result<RemainingFinding, String> {
    let record: JsoncValue = match parse_jsonc(line) {
        Ok(parsed) => parsed,
        Err(_) => return Err(String::from("it is not JSON")),
    };
    let Some(entries) = record.entries() else {
        return Err(String::from("it is not a JSON object"));
    };
    let code: String = required_text(entries, "code")?;
    let Some(rule) = selected_rule(code.as_str(), rules) else {
        return Err(format!("its code {code:?} is not a selected rule"));
    };
    if required_text(entries, "severity")? != "warn" {
        return Err(String::from("its severity is not the configured \"warn\""));
    }
    let message: String = required_text(entries, "message")?;
    let span: &[JsoncEntry] = first_span(entries)?;
    return Ok(RemainingFinding {
        rule,
        line: required_count(span, "line")?,
        column: required_count(span, "column")?,
        message,
    });
}

/// What:
///  Read the standard error of a successful run as findings.
///  `&[u8]` borrows the bytes.
/// Why:
///   JSON Lines:
///  one record per LF-terminated line;
///  an empty stream is no findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remainingFindings(stderr: Uint8Array, rules: MarkdownRule[]): RemainingFinding[];
/// ```
fn remaining_findings(
    stderr: &[u8],
    rules: &[MarkdownRule],
) -> Result<Vec<RemainingFinding>, String> {
    let Ok(text) = std::str::from_utf8(stderr) else {
        return Err(String::from(
            "monochromatic-lint wrote standard error that is not UTF-8 text",
        ));
    };
    let mut findings: Vec<RemainingFinding> = Vec::new();
    if text.is_empty() {
        return Ok(findings);
    }
    // Every record ends with LF; a lone LF is an empty line, which is not a record.
    let Some(body) = text.strip_suffix('\n') else {
        return Err(String::from(
            "monochromatic-lint ended its standard error without a line feed after the last record",
        ));
    };
    for (index, line) in body.split('\n').enumerate() {
        match remaining_finding(line, rules) {
            Ok(finding) => findings.push(finding),
            Err(reason) => {
                return Err(format!(
                    "monochromatic-lint wrote a record cli-git cannot use on line {} of its standard error: {reason}",
                    index + 1
                ));
            }
        }
    }
    return Ok(findings);
}

/// What:
///  The linter's own explanation of a failed run:
///  its prefixed message lines and the
///       messages of its records,
///  joined,
///  at most `EXPLANATION_LIMIT` characters.
/// Why:
///   The person needs the linter's reason;
///  the records' file names are left out,
///  so the
///       explanation does not repeat the candidate's pathname the event already names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explanation(stderr: Uint8Array): string;
/// ```
pub fn explanation(stderr: &[u8]) -> String {
    let text: String = String::from_utf8_lossy(stderr).into_owned();
    let mut parts: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(message) = line.strip_prefix(LINTER_MESSAGE_PREFIX) {
            parts.push(String::from(message));
        } else if let Ok(record) = parse_jsonc(line)
            && let Some(entries) = record.entries()
            && let Ok(message) = required_text(entries, "message")
        {
            parts.push(message);
        }
    }
    if parts.is_empty() {
        return String::from("it printed no explanation");
    }
    let joined: String = parts.join("; ");
    if joined.chars().count() <= EXPLANATION_LIMIT {
        return joined;
    }
    let kept: String = joined.chars().take(EXPLANATION_LIMIT).collect();
    return format!("{kept}...");
}

/// What:
///  Interpret one finished linter run over `input`.
///  `Result<LintRun, String>` is the
///       usable run,
///  or the reason it cannot be used,
///  for the person who ran the command.
/// Why:
///   See the module comment.
///  A successful run must print text,
///  and must print some for
///       a non-empty input:
///  the linter refuses to empty a file,
///  so an empty fixed source
///       would mean it never read the input,
///  and using it would empty the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function interpretLinterRun({ input, rules, finished }): LintRun; // throws string
/// ```
pub fn interpret_linter_run(
    input: &[u8],
    rules: &[MarkdownRule],
    finished: &FinishedChild,
) -> Result<LintRun, String> {
    match finished.exit {
        ChildExit::Code(0) => {}
        ChildExit::Code(2) => {
            return Err(format!(
                "monochromatic-lint could not finish (exit status 2): {}",
                explanation(&finished.stderr)
            ));
        }
        ChildExit::Code(code) => {
            return Err(format!(
                "monochromatic-lint exited with status {code}, which its one-rule configuration never produces: {}",
                explanation(&finished.stderr)
            ));
        }
        ChildExit::Signal(signal) => {
            return Err(format!("monochromatic-lint was ended by signal {signal}"));
        }
        ChildExit::Unknown => {
            return Err(String::from(
                "monochromatic-lint ended without an exit status",
            ));
        }
    }
    if std::str::from_utf8(&finished.stdout).is_err() {
        return Err(String::from(
            "monochromatic-lint printed a fixed source that is not UTF-8",
        ));
    }
    if finished.stdout.is_empty() && !input.is_empty() {
        return Err(String::from(
            "monochromatic-lint printed an empty fixed source for a file that is not empty",
        ));
    }
    return Ok(LintRun {
        fixed: finished.stdout.clone(),
        remaining: remaining_findings(&finished.stderr, rules)?,
    });
}

/// Contract,
///  record and explanation controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_output_tests.rs"]
mod tests;
