//! What:
//! `toolchain.lock.json`:
//! building it from a list of locked tools,
//! rendering it as strict JSON,
//! reading it
//!           back,
//! and the digest of its contents that `install --lockfile-only --frozen-lockfile` recomputes.
//! Why:
//! mpm-shadow is the only writer of the file and the only checker.
//! The file stays strict JSON because
//!           `jq` readers in CI and Dockerfiles select `.tools.<id>.version`,
//! `.artifacts.<platform>.{url,sha256}` and
//!           `.options.androidPackages`;
//! the digest makes a hand edit of a resolved version fail the gate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lock.ts
//! export function renderLock(lock: Lock): string { ... }       // pretty JSON, trailing newline, digest last
//! export function parseLock(source: string, label: string): ParsedLock { ... }
//! export function lockDigest(lock: Lock): string { ... }
//! ```

/// Imports `fs` from `std`, for the read of the lock the scope holds now.
use std::fs;

/// Imports the JSON value types (see `model.rs`).
use serde_json::{Map, Value};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `sha256_hex` from `super::hash`.
use super::hash::sha256_hex;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, decode_lock_entry, lock_entry_value};
/// Imports the one pretty-JSON renderer the lock and `bin` share.
use super::model_encode::render_pretty_json;
/// Imports the shared named helpers.
use super::util::find_tool;

/// The file-format version the lock records in `schemaVersion`.
pub const LOCK_SCHEMA_VERSION: u64 = 1;

/// The plan file the lock says it was projected from (`generatedFrom`).
pub const LOCK_GENERATED_FROM: &str = "toolchain.plan.jsonc";

/// The prefix of the digest text,
/// naming the hash so a later change of hash is visible in the file.
pub const DIGEST_PREFIX: &str = "sha256:";

/// What:
/// The lock:
/// its tools in plan order.
/// Why:
/// A `Vec` and not a map because order is part of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Lock = { tools: LockedTool[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lock {
    /// Locked tools in written order,
    /// ids unique.
    pub tools: Vec<LockedTool>,
}

/// What:
/// A lock read from disk together with the digest text the file recorded.
/// Why:
/// The recorded digest is compared with the recomputed one by the validator,
/// not by the reader,
/// so a command
///           that only needs the contents can say why it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedLock = { lock: Lock; recordedDigest: string | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedLock {
    /// The decoded contents.
    pub lock: Lock,
    /// The `digest` field as written,
    /// or `None` when the file has none.
    pub recorded_digest: Option<String>,
}

/// The methods of `Lock`.
impl Lock {
    /// The tool with this id,
    /// if the lock holds it.
    pub fn tool(&self, id: &str) -> Option<&LockedTool> {
        return find_tool(&self.tools, id);
    }
}

/// What:
/// The lock document without its digest:
/// `schemaVersion`,
/// `generatedFrom`,
/// `tools`,
/// in that order.
/// Why:
/// The digest covers exactly this value,
/// so the writer and the checker share the construction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bodyValue(lock: Lock): Record<string, unknown>
/// ```
fn body_value(lock: &Lock) -> Map<String, Value> {
    let mut tools: Map<String, Value> = Map::new();
    for tool in &lock.tools {
        tools.insert(tool.declaration.id.clone(), lock_entry_value(tool));
    }
    let mut body: Map<String, Value> = Map::new();
    body.insert("schemaVersion".to_string(), Value::from(LOCK_SCHEMA_VERSION));
    body.insert("generatedFrom".to_string(), Value::String(LOCK_GENERATED_FROM.to_string()));
    body.insert("tools".to_string(), Value::Object(tools));
    return body;
}

/// What:
/// The digest of the lock's contents:
/// `sha256:` plus the hex SHA-256 of the compact JSON of the document
///           without its `digest` field.
/// Why:
/// Compact output has no whitespace to vary,
/// and the contents are re-encoded from the decoded lock,
/// so a
///           reformatted file keeps its digest while any changed version,
/// URL,
/// hash,
/// bin or option changes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lockDigest(lock: Lock): string {
///   return 'sha256:' + sha256Hex(JSON.stringify(bodyValue(lock)));
/// }
/// ```
pub fn lock_digest(lock: &Lock) -> String {
    let compact: String = Value::Object(body_value(lock)).to_string();
    return format!("{DIGEST_PREFIX}{}", sha256_hex(compact.as_bytes()));
}

/// What:
/// The text of `toolchain.lock.json`:
/// the document with its digest last,
/// pretty-printed with two-space
///           indentation and one trailing newline.
/// Why:
/// Byte-stable output keeps commits small and lets the revision check compare file hashes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function renderLock(lock: Lock): string { return JSON.stringify(withDigest(lock), null, 2) + '\n'; }
/// ```
pub fn render_lock(lock: &Lock) -> String {
    let mut document: Map<String, Value> = body_value(lock);
    document.insert("digest".to_string(), Value::String(lock_digest(lock)));
    let mut text: String = render_pretty_json(&Value::Object(document));
    text.push('\n');
    return text;
}

/// What:
/// The lock the scope's file holds at this instant,
/// or `None` when it cannot be read or does not decode.
/// Why:
/// A removal must be decided against the lock as it is when the removal happens:
/// a lock loaded earlier is a guess by the time it is acted on,
/// and another command may have published a newer one in between.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function currentLock(scope: Scope): Lock | undefined
/// ```
pub fn current_lock(scope: &Scope) -> Option<Lock> {
    let source: String = fs::read_to_string(&scope.lock_file).ok()?;
    return parse_lock(&source, &scope.lock_file.display().to_string()).ok().map(|parsed| return parsed.lock);
}

/// What:
/// Decodes lock text.
/// Why:
/// `label` is the file path shown in every message.
/// The `digest` field is optional here so a lock written by
///           hand (without one) is reported as a digest problem by the validator rather than as a syntax problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseLock(source: string, label: string): ParsedLock
/// ```
pub fn parse_lock(source: &str, label: &str) -> Result<ParsedLock, ToolchainError> {
    let value: Value = match serde_json::from_str(source) {
        Ok(parsed) => parsed,
        Err(error) => return Err(ToolchainError::config(format!("{label} is not valid JSON: {error}"))),
    };
    let Some(root) = value.as_object() else {
        return Err(ToolchainError::config(format!("{label} must hold a JSON object at its top level")));
    };
    for key in root.keys() {
        if !["schemaVersion", "generatedFrom", "tools", "digest"].contains(&key.as_str()) {
            return Err(ToolchainError::config(format!(
                "{label}: $.{key} is not a known field; the known fields are schemaVersion, generatedFrom, tools, digest"
            )));
        }
    }
    if root.get("schemaVersion").and_then(Value::as_u64) != Some(LOCK_SCHEMA_VERSION) {
        return Err(ToolchainError::config(format!("{label}: $.schemaVersion must be {LOCK_SCHEMA_VERSION}")));
    }
    let Some(tools_value) = root.get("tools").and_then(Value::as_object) else {
        return Err(ToolchainError::config(format!("{label}: $.tools must be an object keyed by tool id")));
    };
    let mut tools: Vec<LockedTool> = Vec::new();
    for (id, entry) in tools_value {
        match decode_lock_entry(id, entry, &format!("tools.{id}")) {
            Ok(tool) => tools.push(tool),
            Err(message) => return Err(ToolchainError::config(format!("{label}: {message}"))),
        }
    }
    let recorded_digest: Option<String> = root.get("digest").and_then(Value::as_str).map(str::to_string);
    return Ok(ParsedLock { lock: Lock { tools }, recorded_digest });
}
