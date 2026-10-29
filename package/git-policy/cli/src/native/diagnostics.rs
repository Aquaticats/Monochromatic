//! What:
//!  Render wrapper diagnostics as the JSONL events callers already parse.
//! Why:
//!  Agents and hooks read cli-git's event stream line by line;
//!  every wrapper-made
//!      event must be one compact JSON object per line with stable field order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process.stderr.write(JSON.stringify({ schemaVersion: 1, sequence: 0, type: 'engine-failure', ... }) + '\n');
//! ```

/// The hint that follows an engine failure line.
use super::event_hints::{engine_failure_hint, push_hint_field};

/// What:
///  The event schema version every line carries.
///       `u32` is an unsigned 32-bit integer (siblings `u8`,
///  `u64`,
///  `usize`).
/// Why:
///   Consumers branch on this number;
///  `u32` is ample for a version counter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCHEMA_VERSION = 1;
/// ```
pub const SCHEMA_VERSION: u32 = 1;

/// Code of the warning for a legacy executable configuration left beside a JSONC file.
pub const LEGACY_CONFIG_IGNORED_CODE: &str = "legacy-config-ignored";

/// What:
///  The stable causes of an engine failure this foundation and its successors report.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   A closed set keeps the wire spelling in one place.
///  Codes that described
///       plugins and trust are not carried over.
///  A shipped policy that could not finish
///       is reported by cause:
///  `ContentUnavailable` when something it had to read could
///       not be read,
///  `PolicyIncomplete` when its own machinery failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EngineFailureCode = 'config-invalid' | 'core-incomplete' | ... | 'index-lock-unproven-owner';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineFailureCode {
    /// Configuration could not be read or validated,
    ///  including legacy files needing migration.
    ConfigInvalid,
    /// A fixed transform failed unexpectedly.
    CoreIncomplete,
    /// Something a policy or lifecycle had to read could not be read:
    ///  a candidate's bytes,
    /// a repository fact asked of Git,
    ///  or transaction state.
    ///  The policy itself is intact;
    /// its input is missing.
    ContentUnavailable,
    /// A shipped policy's own machinery failed,
    ///  so the policy could not finish although
    /// its input was readable:
    ///  its rules file could not be loaded,
    ///  its linter could not
    /// start,
    ///  or it hit an internal error.
    ///  An unreadable candidate or repository fact is
    /// `ContentUnavailable` instead.
    PolicyIncomplete,
    /// A proposed patch was not valid.
    PatchInvalid,
    /// A proposed patch no longer applied.
    PatchConflict,
    /// Fixing revisited an earlier state.
    FixCycle,
    /// Fixing did not settle within the pass limit.
    FixPassLimit,
    /// A commit transaction could not complete.
    TransactionFailed,
    /// A foreign `index.lock` with an unproven owner outlasted the wait budget.
    IndexLockUnprovenOwner,
}

/// What:
///  The wire spelling of a failure code.
///       `&'static str` borrows text compiled into the executable for its whole run.
/// Why:
///   Events print exactly the spellings the incumbent emitted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function engineFailureCodeName(code: EngineFailureCode): string;
/// ```
pub fn engine_failure_code_name(code: EngineFailureCode) -> &'static str {
    // What: `match` picks the arm for the variant; the compiler rejects a missing one.
    // Why:  A new code cannot be added without giving it a wire spelling.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (code) { case 'ConfigInvalid': return 'config-invalid'; ... }
    // ```
    match code {
        EngineFailureCode::ConfigInvalid => return "config-invalid",
        EngineFailureCode::CoreIncomplete => return "core-incomplete",
        EngineFailureCode::ContentUnavailable => return "content-unavailable",
        EngineFailureCode::PolicyIncomplete => return "policy-incomplete",
        EngineFailureCode::PatchInvalid => return "patch-invalid",
        EngineFailureCode::PatchConflict => return "patch-conflict",
        EngineFailureCode::FixCycle => return "fix-cycle",
        EngineFailureCode::FixPassLimit => return "fix-pass-limit",
        EngineFailureCode::TransactionFailed => return "transaction-failed",
        EngineFailureCode::IndexLockUnprovenOwner => return "index-lock-unproven-owner",
    }
}

/// What:
///  Encode text as a JSON string literal,
///  exactly as `JSON.stringify` would.
///       `&str` borrows the text;
///  `String` is the owned,
///  quoted result.
/// Why:
///   Messages contain file paths,
///  quotes and user-written keys.
///  Each character is
///       encoded at this final step so no value can break out of its JSON string or
///       its line:
///  quotes,
///  backslashes and every control character are escaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsonString(text: string): string { return JSON.stringify(text); }
/// ```
pub fn json_string(text: &str) -> String {
    // What: `String::with_capacity(n)` is empty owned text with room for `n` bytes;
    //       `mut` allows appending.
    // Why:  Most text needs only its two surrounding quotes added.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let result = '"';
    // ```
    let mut result: String = String::with_capacity(text.len() + 2);
    result.push('"');
    // `.chars()` yields each Unicode character of the text in order.
    for character in text.chars() {
        if character == '"' {
            result.push_str("\\\"");
        } else if character == '\\' {
            result.push_str("\\\\");
        } else if character == '\n' {
            result.push_str("\\n");
        } else if character == '\r' {
            result.push_str("\\r");
        } else if character == '\t' {
            result.push_str("\\t");
        } else if character == '\u{8}' {
            result.push_str("\\b");
        } else if character == '\u{c}' {
            result.push_str("\\f");
        } else if character < ' ' {
            // What: `u32::from(character)` is the character's code point number;
            //       `{:04x}` prints it as four lowercase hexadecimal digits.
            // Why:  The remaining control characters have no short escape.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // result += '\\u' + code.toString(16).padStart(4, '0');
            // ```
            result.push_str(format!("\\u{:04x}", u32::from(character)).as_str());
        } else {
            result.push(character);
        }
    }
    result.push('"');
    return result;
}

/// What:
///  Render one engine-failure event as a line-terminated JSON object.
///       `u64` is an unsigned 64-bit integer for the per-invocation event number.
/// Why:
///   Field order and spelling match the incumbent's `JSON.stringify` output,
///  so
///       existing consumers parse native events unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderEngineFailure(sequence: number, code: EngineFailureCode, message: string): string;
/// ```
pub fn render_engine_failure(sequence: u64, code: EngineFailureCode, message: &str) -> String {
    return render_engine_failure_with_hint(sequence, code, message, engine_failure_hint(engine_failure_code_name(code)));
}

/// What:
///  Render one engine-failure event with the hint the caller chooses,
///  or with none.
/// Why:
///   A hint is a guess about the goal that depends on the code alone.
///  A message that already says what the person can do (a retained transaction names the file to remove or restore,
///  and what the wrapper then does) must not carry the code's generic hint, which may not apply to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderEngineFailureWithHint(sequence: number, code: EngineFailureCode, message: string, hint?: string): string;
/// ```
pub fn render_engine_failure_with_hint(
    sequence: u64,
    code: EngineFailureCode,
    message: &str,
    hint: Option<&str>,
) -> String {
    // `format!` builds the owned line; `{{` and `}}` are literal braces.
    let name: &str = engine_failure_code_name(code);
    let mut line: String = format!(
        "{{\"schemaVersion\":{SCHEMA_VERSION},\"sequence\":{sequence},\"type\":\"engine-failure\",\"code\":{},\"message\":{}",
        json_string(name),
        json_string(message)
    );
    push_hint_field(&mut line, hint);
    line.push_str("}\n");
    return line;
}

/// What:
///  Render one configuration-warning event as a line-terminated JSON object.
/// Why:
///   A warning about configuration itself (a legacy file left beside the JSONC
///       file) belongs on the machine stream as an event,
///  never as free prose there.
///       Unlike the incumbent's per-policy `warn-unsafe` warning it has no policy or
///       trigger,
///  so it carries a path instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderConfigurationWarning(sequence: number, code: string, message: string, path: string): string;
/// ```
pub fn render_configuration_warning(
    sequence: u64,
    code: &str,
    message: &str,
    path: &str,
) -> String {
    let mut line: String = format!(
        "{{\"schemaVersion\":{SCHEMA_VERSION},\"sequence\":{sequence},\"type\":\"configuration-warning\",\"code\":{},\"message\":{},\"path\":{}",
        json_string(code),
        json_string(message),
        json_string(path)
    );
    push_hint_field(&mut line, engine_failure_hint(code));
    line.push_str("}\n");
    return line;
}

/// Encoding and field-order controls stay out of the release executable.
#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
