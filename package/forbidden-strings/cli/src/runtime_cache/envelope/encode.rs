//! What:
//!  The encoder:
//!  one hybrid rule bundle in,
//!  one complete artifact frame out,
//!  with every count,
//!  length and ceiling checked before a byte is written.
//! Why:
//!  The artifact is what a later scan trusts,
//!  so it must be impossible to publish one the decoder would reject:
//!  the same ceilings are applied here as there,
//!  and the payload and engine digests are computed from the bytes this
//!  function is about to return.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function encode(compiled: RuntimeRules, digest: SourceDigest): Uint8Array
//! ```

/// Imports the engine type whose serialized bytes travel in the artifact.
use forbidden_regex::RegexSet;

/// Imports the rejection type and the adapter that names a failure without
///  disclosing it.
use super::error::{
    EnvelopeError,
    invalid_envelope,
};
/// Imports the frame's marker,
///  version,
///  ceilings,
///  name markers and the name grammar the encoder enforces.
use super::frame_layout::{
    MAGIC,
    MAX_ARTIFACT_BYTES,
    MAX_IDENTITY_BYTES,
    MAX_LITERAL_BYTES,
    MAX_RULE_COUNT,
    MAX_RULE_NAME_BYTES,
    NAME_ABSENT,
    NAME_PRESENT,
    SCHEMA_VERSION,
    valid_rule_name,
};
/// Imports the content digest and the compile compatibility identities.
use crate::runtime_cache::path::{
    SourceDigest,
    digest_bytes,
    platform_identity,
    scanner_version,
    source_digest,
};
/// Imports the hybrid rule bundle the artifact encodes.
use crate::runtime_matcher::RuntimeRules;

/// Appends length-prefixed compatibility identity.
fn append_identity(
    output: &mut Vec<u8>,
    value: &str,
) -> Result<(), EnvelopeError> {
    let length = u16::try_from(value.len()).map_err(invalid_envelope)?;
    if length == 0 || length > MAX_IDENTITY_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value.as_bytes());
    return Ok(());
}

/// Encodes hybrid literal data and regex-only engine bytes into one artifact.
pub(crate) fn encode(
    compiled: &RuntimeRules,
    digest: SourceDigest,
) -> Result<Vec<u8>, EnvelopeError> {
    let rule_count = u32::try_from(compiled.len()).map_err(invalid_envelope)?;
    if rule_count > MAX_RULE_COUNT {
        return Err(EnvelopeError::Invalid);
    }
    let engine_bytes = compiled
        .regex_set()
        .map(RegexSet::to_bytes)
        .transpose()
        .map_err(invalid_envelope)?
        .unwrap_or_default();
    let engine_digest = source_digest(&engine_bytes).map_err(invalid_envelope)?;

    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&SCHEMA_VERSION.to_le_bytes());
    append_identity(&mut output, scanner_version())?;
    append_identity(&mut output, &platform_identity())?;
    output.extend_from_slice(&digest_bytes(digest));
    let payload_start = output.len();
    output.extend_from_slice(&rule_count.to_le_bytes());
    for optional_name in compiled.names() {
        if let Some(name) = optional_name {
            let name_length = u32::try_from(name.len()).map_err(invalid_envelope)?;
            if name_length > MAX_RULE_NAME_BYTES || !valid_rule_name(name) {
                return Err(EnvelopeError::Invalid);
            }
            output.push(NAME_PRESENT);
            output.extend_from_slice(&name_length.to_le_bytes());
            output.extend_from_slice(name.as_bytes());
        } else {
            output.push(NAME_ABSENT);
        }
    }

    let literal_count = u32::try_from(compiled.literal_groups().len()).map_err(invalid_envelope)?;
    output.extend_from_slice(&literal_count.to_le_bytes());
    for group in compiled.literal_groups() {
        let literal_length = u32::try_from(group.bytes.len()).map_err(invalid_envelope)?;
        let id_count = u32::try_from(group.rule_ids.len()).map_err(invalid_envelope)?;
        if literal_length == 0 || literal_length > MAX_LITERAL_BYTES || id_count == 0 {
            return Err(EnvelopeError::Invalid);
        }
        output.extend_from_slice(&literal_length.to_le_bytes());
        output.extend_from_slice(&group.bytes);
        output.extend_from_slice(&id_count.to_le_bytes());
        for &rule_id in &group.rule_ids {
            let encoded_id = u32::try_from(rule_id).map_err(invalid_envelope)?;
            output.extend_from_slice(&encoded_id.to_le_bytes());
        }
    }

    let regex_id_count =
        u32::try_from(compiled.regex_rule_ids().len()).map_err(invalid_envelope)?;
    output.extend_from_slice(&regex_id_count.to_le_bytes());
    for &rule_id in compiled.regex_rule_ids() {
        let encoded_id = u32::try_from(rule_id).map_err(invalid_envelope)?;
        output.extend_from_slice(&encoded_id.to_le_bytes());
    }
    output.extend_from_slice(&digest_bytes(engine_digest));
    output.extend_from_slice(
        &u64::try_from(engine_bytes.len())
            .map_err(invalid_envelope)?
            .to_le_bytes(),
    );
    output.extend_from_slice(&engine_bytes);
    let payload_digest = source_digest(&output[payload_start..]).map_err(invalid_envelope)?;
    output.extend_from_slice(&digest_bytes(payload_digest));
    if u64::try_from(output.len()).map_err(invalid_envelope)? > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    return Ok(output);
}
