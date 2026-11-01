//! Encode a native Bash hook rewrite without parsing or re-encoding shell source.

/// What:
/// `use` imports a name;
/// `::` separates module names.
/// Why:
/// Represent the host's JSON without generated structs or a CLI framework.
/// TypeScript equivalent:
/// `type Value = JsonValue;`
use serde_json::Value;
/// Import filesystem checks for the previously initialized worker group.
use std::fs;
/// Import the standard error and result types.
use std::io;
/// Import a borrowed filesystem path,
/// rather than its owning `PathBuf` sibling.
use std::path::Path;

/// What:
/// `fn` declares a function;
/// `&str` borrows text while `String` owns its result.
/// Why:
/// Only the cgroup filename crosses a shell-quotation boundary.
/// TypeScript equivalent:
/// `function shellQuote(value: string): string { ... }`
fn shell_quote(value: &str) -> String {
    // What: `mut` permits appending to an owned buffer; `String::from` copies the literal.
    // Why: Build the escaped path in one linear pass.
    // TypeScript equivalent: `let output = "'";`
    let mut output = String::from("'");
    for character in value.chars() {
        if character == '\'' {
            output.push_str("'\\''");
        } else {
            output.push(character);
        }
    }
    output.push('\'');
    return output;
}

/// What:
/// `pub` exposes this function to the entry point;
/// `Result` carries output or an I/O error.
/// Why:
/// Reject invalid hook events and resource controls before proposing a command.
/// TypeScript equivalent:
/// `export function response(input: string, parent: string): string { ... }`
pub fn response(input: &str, parent: &Path) -> io::Result<String> {
    // What: The `Value` annotation selects serde_json's general JSON representation.
    // `map_err` converts parser errors to the entry point's I/O error type; `?` propagates them.
    // Why: A malformed event must block rather than run an unmodified command.
    // TypeScript equivalent: `const event: JsonValue = JSON.parse(input);`
    let event: Value = serde_json::from_str(input).map_err(io::Error::other)?;
    // What: `Some` represents a present value, unlike `None` for absence.
    // Why: Only the expected host event is eligible for a command rewrite.
    // TypeScript equivalent: `if (event.hook_event_name !== 'PreToolUse' || event.tool_name !== 'Bash') { ... }`
    if event.get("hook_event_name").and_then(Value::as_str) != Some("PreToolUse")
        || event.get("tool_name").and_then(Value::as_str) != Some("Bash")
    {
        // What: `Err` returns the failure variant, paired with successful `Ok`.
        // Why: This hook applies only to the host's Bash pre-execution boundary.
        // TypeScript equivalent: `throw new Error('Expected a Bash PreToolUse event.');`
        return Err(io::Error::other("Expected a Bash PreToolUse event."));
    }
    // Require the command container without inventing a default command.
    let tool_input = event.get("tool_input").ok_or_else(|| {
        // Return this diagnostic from the zero-argument error factory (`||`).
        return io::Error::other("The Bash hook event has no tool_input.");
    })?;
    // Borrow the exact decoded command string supplied by Codex.
    let original = tool_input.get("command").and_then(Value::as_str).ok_or_else(|| {
        // Distinguish absent or non-string input from a legitimate empty command.
        return io::Error::other("The Bash hook event requires a command string.");
    })?;
    // Check the real parent controls before releasing any original command text.
    crate::limits::verify(parent)?;
    // Derive the only writable cgroup child used by the hook.
    let membership = parent.join("codex/cgroup.procs");
    // What: `&membership` borrows the path; `?` propagates a missing-file error.
    // Why: Missing session initialization cannot produce an unrestricted command.
    // TypeScript equivalent: `const metadata = statSync(membership);`
    let metadata = fs::metadata(&membership)?;
    if !metadata.is_file() {
        // Refuse a directory or other non-file membership target.
        return Err(io::Error::other("The worker cgroup membership target must be a file."));
    }
    // Require lossless path text before shell quoting.
    let membership_text = membership.to_str().ok_or_else(|| {
        // Reject unrepresentable path bytes instead of changing the target.
        return io::Error::other("The worker membership path must be UTF-8.");
    })?;
    // Own the prefix buffer; the original shell body is appended without shell parsing.
    let mut command = String::from("command printf '%s' \"$$\" > ");
    // Borrow the separately shell-encoded pathname for appending.
    command.push_str(&shell_quote(membership_text));
    command.push_str(" || exit 70\n");
    command.push_str(original);
    // Serialize only the JSON string boundary, preserving the decoded command verbatim.
    let encoded = serde_json::to_string(&command).map_err(io::Error::other)?;
    // Construct the fixed native hook envelope around the safely encoded string.
    let mut output = String::from("{\"hookSpecificOutput\":{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"allow\",\"updatedInput\":{\"command\":");
    // Append the JSON-encoded command rather than interpolating shell text into JSON syntax.
    output.push_str(&encoded);
    output.push_str("}}}");
    // What: `Ok` is successful output, unlike the error variant `Err`.
    // Why: Return a complete envelope only after every check succeeded.
    // TypeScript equivalent: `return output;`
    return Ok(output);
}
