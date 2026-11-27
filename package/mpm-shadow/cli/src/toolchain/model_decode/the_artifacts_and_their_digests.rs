//! What:
//! The artifact rows of a lock entry:
//! one download address and SHA-256 per platform id,
//! and the check that a digest is 64 lowercase hex characters.
//! Why:
//! A row keyed by anything but a known platform id can never match the running machine,
//! so it is refused here with the row named rather than kept as an artifact nothing will use;
//! and the digest spelling is checked where the row is read,
//! because every checker of a downloaded file trusts it later.
//! A caller imports from `model_decode` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeArtifacts(value: unknown, path: string): [string, Artifact][];
//! export function isSha256Hex(text: string): boolean;
//! ```

/// Imports the JSON value the rows are read from.
use serde_json::Value;

/// Imports the decoding checks this file shares with its siblings.
use super::super::model_decode_support::{
    DecodeResult,
    as_nonempty_text,
    as_object,
    as_text,
    reject_unknown_keys,
};
/// Imports the platform ids an artifact row may be keyed by.
use super::super::update_resolve::LINUX_PLATFORMS;

/// Imports the model type one artifact row is.
use super::super::model::Artifact;
/// The artifact rows object at `path`.
pub(super) fn decode_artifacts(
    value: &Value,
    path: &str,
) -> DecodeResult<Vec<(String, Artifact)>> {
    let map = as_object(value, path)?;
    let mut rows: Vec<(String, Artifact)> = Vec::new();
    for (platform, row) in map {
        let row_path: String = format!("{path}.{platform}");
        // A row keyed by anything else can never match the running machine, so it is refused here, naming the row.
        if !LINUX_PLATFORMS.contains(&platform.as_str()) {
            return Err(format!(
                "{row_path}: {platform:?} is not a platform id; the known ids are {}",
                LINUX_PLATFORMS.join(", ")
            ));
        }
        let row_map = as_object(row, &row_path)?;
        reject_unknown_keys(row_map, &["url", "sha256"], &row_path)?;
        let Some(url) = row_map.get("url") else {
            return Err(format!("{row_path}.url is required"));
        };
        let Some(sha256) = row_map.get("sha256") else {
            return Err(format!("{row_path}.sha256 is required"));
        };
        let sha256_text: String = as_text(sha256, &format!("{row_path}.sha256"))?;
        if !is_sha256_hex(&sha256_text) {
            return Err(format!(
                "{row_path}.sha256 must be 64 lowercase hex characters"
            ));
        }
        rows.push((
            platform.clone(),
            Artifact {
                url: as_nonempty_text(url, &format!("{row_path}.url"))?,
                sha256: sha256_text,
            },
        ));
    }
    return Ok(rows);
}

/// Whether `text` is 64 lowercase hex characters.
pub fn is_sha256_hex(text: &str) -> bool {
    if text.len() != 64 {
        return false;
    }
    for byte in text.bytes() {
        if !(byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
            return false;
        }
    }
    return true;
}
