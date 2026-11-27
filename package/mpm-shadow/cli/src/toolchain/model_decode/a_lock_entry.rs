//! What:
//! One lock entry:
//! the declaration fields the plan and the lock share,
//! the version the lock pins,
//! and the artifacts of that version.
//! Why:
//! The lock is read by `install --lockfile-only --frozen-lockfile`,
//! `install`,
//! `list` and `update`,
//! so a malformed entry stops them all with the field named;
//! and the version becomes the name of the install directory,
//! which is why it is checked here.
//! A caller imports from `model_decode` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool;
//! ```

/// Imports the JSON value one lock entry is read from.
use serde_json::Value;

/// Imports the model types a lock entry is built from.
use super::super::model::{
    Artifact,
    Declaration,
    LockedTool,
};
/// Imports the decoding checks this file shares with its siblings.
use super::super::model_decode_support::{
    DecodeResult,
    as_nonempty_text,
    as_object,
    reject_unknown_keys,
};
/// Imports the check that the version is one plain directory name.
use super::super::segment::check_version;

/// Imports the declaration fields the plan and the lock share.
use super::a_declaration::decode_declaration_fields;
/// Imports the artifact rows of the pinned version.
use super::the_artifacts_and_their_digests::decode_artifacts;
/// What:
/// Decodes one lock entry (`tools.<id>` in `toolchain.lock.json`):
/// declaration fields,
/// `version`,
/// `artifacts`.
/// Why:
/// The lock is read by `install --lockfile-only --frozen-lockfile`,
/// `install`,
/// `list` and `update`;
/// a malformed entry stops them all with
///           the field named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool
/// ```
pub fn decode_lock_entry(
    id: &str,
    value: &Value,
    path: &str,
) -> DecodeResult<LockedTool> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &[
            "request",
            "version",
            "route",
            "manager",
            "packageId",
            "bins",
            "os",
            "env",
            "options",
            "artifacts",
        ],
        path,
    )?;
    let declaration: Declaration = decode_declaration_fields(id, map, path)?;
    let Some(version_value) = map.get("version") else {
        return Err(format!("{path}.version is required"));
    };
    let version: String = as_nonempty_text(version_value, &format!("{path}.version"))?;
    // The version becomes the name of the install directory, and a version such as `../x` would leave the tool's own.
    if let Err(message) = check_version(&version) {
        return Err(format!("{path}.version: {message}"));
    }
    let artifacts: Vec<(String, Artifact)> = match map.get("artifacts") {
        Some(rows) => decode_artifacts(rows, &format!("{path}.artifacts"))?,
        None => Vec::new(),
    };
    return Ok(LockedTool {
        declaration,
        version,
        artifacts,
    });
}
