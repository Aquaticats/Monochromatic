//! Encodes and validates scanner-owned runtime cache envelopes.
//!
//! Framing is explicit little-endian data around the engine's existing bincode
//! bytes.
//!  Every declared size is checked against a fixed ceiling and remaining
//! input before allocation or slicing.
//!  The decoder accepts no trailing bytes.

/// Imports hybrid runtime rule bundle reconstructed by cache decoder.
use crate::runtime_matcher::{LiteralGroup, RuntimeRules};
/// Imports regex-only engine type for artifact serialization.
use forbidden_regex::RegexSet;

/// Imports content digest and compile compatibility identities.
use super::path::{
    digest_bytes, platform_identity, scanner_version, source_digest, SourceDigest,
};
/// Imports redacted warning reason mapping.
use super::warning::CacheWarningReason;

/// Exact eight-byte artifact marker.
const MAGIC: &[u8; 8] = b"FSRULES\0";
/// Scanner-owned outer envelope schema version.
const SCHEMA_VERSION: u32 = 2;
/// Maximum complete artifact accepted before reading or decoding:
///  512 MiB.
pub(super) const MAX_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
/// Maximum rule identities decoded into memory.
const MAX_RULE_COUNT: u32 = 1_000_000;
/// Maximum UTF-8 bytes in one section name.
const MAX_RULE_NAME_BYTES: u32 = 4_096;
/// Maximum exact-literal bytes in one cached group.
const MAX_LITERAL_BYTES: u32 = 16 * 1024 * 1024;
/// Maximum UTF-8 bytes in version or platform identity.
const MAX_IDENTITY_BYTES: u16 = 128;
/// Marker for unnamed legacy rule identity.
const NAME_ABSENT: u8 = 0;
/// Marker for named tail-format rule identity.
const NAME_PRESENT: u8 = 1;

/// Redacted envelope rejection category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnvelopeError {
    /// Schema,
    ///  scanner version,
    ///  or platform identity differs.
    Incompatible,
    /// Embedded source digest disagrees with current source.
    SourceMismatch,
    /// Framing,
    ///  names,
    ///  counts,
    ///  or engine bytes are malformed.
    Invalid,
}

/// Converts envelope rejection into closed warning protocol reason.
impl EnvelopeError {
    /// Returns corresponding scanner cache-warning reason.
    pub(super) fn warning_reason(self) -> CacheWarningReason {
        if self == EnvelopeError::Incompatible {
            return CacheWarningReason::Incompatible;
        }
        if self == EnvelopeError::SourceMismatch {
            return CacheWarningReason::SourceMismatch;
        }
        return CacheWarningReason::Invalid
    }
}

/// Cursor over borrowed artifact bytes with checked forward-only reads.
struct Cursor<'a> {
    /// Complete immutable artifact bytes.
    bytes: &'a [u8],
    /// Next unread byte offset.
    position: usize,
}

/// Checked cursor operations used by envelope decoder.
impl<'a> Cursor<'a> {
    /// Creates cursor at beginning of artifact bytes.
    fn new(bytes: &'a [u8]) -> Self {
        return Self { bytes, position: 0 }
    }

    /// Borrows exact next byte count or rejects truncation and overflow.
    fn take(&mut self, count: usize) -> Result<&'a [u8], EnvelopeError> {
        let end = self.position.checked_add(count).ok_or(EnvelopeError::Invalid)?;
        let value = self.bytes.get(self.position..end).ok_or(EnvelopeError::Invalid)?;
        self.position = end;
        return Ok(value)
    }

    /// Reads one byte.
    fn read_u8(&mut self) -> Result<u8, EnvelopeError> {
        return self.take(1).map(|bytes| return bytes[0])
    }

    /// Reads little-endian 16-bit integer.
    fn read_u16(&mut self) -> Result<u16, EnvelopeError> {
        let bytes: [u8; 2] = self.take(2)?.try_into().map_err(|_| return EnvelopeError::Invalid)?;
        return Ok(u16::from_le_bytes(bytes))
    }

    /// Reads little-endian 32-bit integer.
    fn read_u32(&mut self) -> Result<u32, EnvelopeError> {
        let bytes: [u8; 4] = self.take(4)?.try_into().map_err(|_| return EnvelopeError::Invalid)?;
        return Ok(u32::from_le_bytes(bytes))
    }

    /// Reads little-endian 64-bit integer.
    fn read_u64(&mut self) -> Result<u64, EnvelopeError> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| return EnvelopeError::Invalid)?;
        return Ok(u64::from_le_bytes(bytes))
    }

    /// Reads bounded UTF-8 identity string.
    fn read_identity(&mut self) -> Result<&'a str, EnvelopeError> {
        let length = self.read_u16()?;
        if length == 0 || length > MAX_IDENTITY_BYTES {
            return Err(EnvelopeError::Invalid);
        }
        return std::str::from_utf8(self.take(usize::from(length))?)
            .map_err(|_| return EnvelopeError::Invalid)
    }

    /// Reports whether all artifact bytes were consumed.
    fn is_finished(&self) -> bool {
        return self.position == self.bytes.len()
    }
}

/// Appends length-prefixed compatibility identity.
fn append_identity(output: &mut Vec<u8>, value: &str) -> Result<(), EnvelopeError> {
    let length = u16::try_from(value.len()).map_err(|_| return EnvelopeError::Invalid)?;
    if length == 0 || length > MAX_IDENTITY_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value.as_bytes());
    return Ok(())
}

/// Reports whether decoded name obeys strict tail-format section-name grammar.
fn valid_rule_name(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    return characters.all(|character| {
        return character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '.'
            || character == '-'
    })
}

/// Encodes hybrid literal data and regex-only engine bytes into one artifact.
pub(super) fn encode(
    compiled: &RuntimeRules,
    digest: SourceDigest,
) -> Result<Vec<u8>, EnvelopeError> {
    let rule_count = u32::try_from(compiled.len()).map_err(|_| return EnvelopeError::Invalid)?;
    if rule_count > MAX_RULE_COUNT {
        return Err(EnvelopeError::Invalid);
    }
    let engine_bytes = compiled
        .regex_set()
        .map(RegexSet::to_bytes)
        .transpose()
        .map_err(|_| return EnvelopeError::Invalid)?
        .unwrap_or_default();
    let engine_digest = source_digest(&engine_bytes).map_err(|_| return EnvelopeError::Invalid)?;

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
            let name_length = u32::try_from(name.len()).map_err(|_| return EnvelopeError::Invalid)?;
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

    let literal_count = u32::try_from(compiled.literal_groups().len())
        .map_err(|_| return EnvelopeError::Invalid)?;
    output.extend_from_slice(&literal_count.to_le_bytes());
    for group in compiled.literal_groups() {
        let literal_length = u32::try_from(group.bytes.len()).map_err(|_| return EnvelopeError::Invalid)?;
        let id_count = u32::try_from(group.rule_ids.len()).map_err(|_| return EnvelopeError::Invalid)?;
        if literal_length == 0 || literal_length > MAX_LITERAL_BYTES || id_count == 0 {
            return Err(EnvelopeError::Invalid);
        }
        output.extend_from_slice(&literal_length.to_le_bytes());
        output.extend_from_slice(&group.bytes);
        output.extend_from_slice(&id_count.to_le_bytes());
        for &rule_id in &group.rule_ids {
            let encoded_id = u32::try_from(rule_id).map_err(|_| return EnvelopeError::Invalid)?;
            output.extend_from_slice(&encoded_id.to_le_bytes());
        }
    }

    let regex_id_count = u32::try_from(compiled.regex_rule_ids().len())
        .map_err(|_| return EnvelopeError::Invalid)?;
    output.extend_from_slice(&regex_id_count.to_le_bytes());
    for &rule_id in compiled.regex_rule_ids() {
        let encoded_id = u32::try_from(rule_id).map_err(|_| return EnvelopeError::Invalid)?;
        output.extend_from_slice(&encoded_id.to_le_bytes());
    }
    output.extend_from_slice(&digest_bytes(engine_digest));
    output.extend_from_slice(
        &u64::try_from(engine_bytes.len())
            .map_err(|_| return EnvelopeError::Invalid)?
            .to_le_bytes(),
    );
    output.extend_from_slice(&engine_bytes);
    let payload_digest = source_digest(&output[payload_start..])
        .map_err(|_| return EnvelopeError::Invalid)?;
    output.extend_from_slice(&digest_bytes(payload_digest));
    if u64::try_from(output.len()).map_err(|_| return EnvelopeError::Invalid)? > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    return Ok(output)
}

/// Decodes and validates one complete hybrid envelope against source identity.
pub(super) fn decode(
    bytes: &[u8],
    expected_digest: SourceDigest,
) -> Result<RuntimeRules, EnvelopeError> {
    if u64::try_from(bytes.len()).map_err(|_| return EnvelopeError::Invalid)? > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    let mut header_cursor = Cursor::new(bytes);
    if header_cursor.take(MAGIC.len())? != MAGIC {
        return Err(EnvelopeError::Invalid);
    }
    if header_cursor.read_u32()? != SCHEMA_VERSION {
        return Err(EnvelopeError::Incompatible);
    }
    if header_cursor.read_identity()? != scanner_version() || header_cursor.read_identity()? != platform_identity() {
        return Err(EnvelopeError::Incompatible);
    }
    if header_cursor.take(32)? != digest_bytes(expected_digest) {
        return Err(EnvelopeError::SourceMismatch);
    }
    let payload_start = header_cursor.position;
    let payload_end = bytes.len().checked_sub(32).ok_or(EnvelopeError::Invalid)?;
    if payload_end < payload_start {
        return Err(EnvelopeError::Invalid);
    }
    let expected_payload_digest: [u8; 32] = bytes[payload_end..]
        .try_into()
        .map_err(|_| return EnvelopeError::Invalid)?;
    let actual_payload_digest = source_digest(&bytes[payload_start..payload_end])
        .map_err(|_| return EnvelopeError::Invalid)?;
    if digest_bytes(actual_payload_digest) != expected_payload_digest {
        return Err(EnvelopeError::Invalid);
    }
    let mut cursor = Cursor::new(&bytes[payload_start..payload_end]);

    let rule_count = cursor.read_u32()?;
    if rule_count > MAX_RULE_COUNT {
        return Err(EnvelopeError::Invalid);
    }
    let mut names = Vec::new();
    for _ in 0..rule_count {
        let marker = cursor.read_u8()?;
        if marker == NAME_ABSENT {
            names.push(None);
            continue;
        }
        if marker != NAME_PRESENT {
            return Err(EnvelopeError::Invalid);
        }
        let name_length = cursor.read_u32()?;
        if name_length == 0 || name_length > MAX_RULE_NAME_BYTES {
            return Err(EnvelopeError::Invalid);
        }
        let name_bytes = cursor.take(usize::try_from(name_length).map_err(|_| return EnvelopeError::Invalid)?)?;
        let name = std::str::from_utf8(name_bytes).map_err(|_| return EnvelopeError::Invalid)?;
        if !valid_rule_name(name) {
            return Err(EnvelopeError::Invalid);
        }
        names.push(Some(name.to_string()));
    }

    let literal_count = cursor.read_u32()?;
    if literal_count > rule_count {
        return Err(EnvelopeError::Invalid);
    }
    let mut literal_groups = Vec::new();
    for _ in 0..literal_count {
        let literal_length = cursor.read_u32()?;
        if literal_length == 0 || literal_length > MAX_LITERAL_BYTES {
            return Err(EnvelopeError::Invalid);
        }
        let literal = cursor
            .take(usize::try_from(literal_length).map_err(|_| return EnvelopeError::Invalid)?)?
            .to_vec();
        let id_count = cursor.read_u32()?;
        if id_count == 0 || id_count > rule_count {
            return Err(EnvelopeError::Invalid);
        }
        let mut rule_ids = Vec::new();
        for _ in 0..id_count {
            rule_ids.push(usize::try_from(cursor.read_u32()?).map_err(|_| return EnvelopeError::Invalid)?);
        }
        literal_groups.push(LiteralGroup { bytes: literal, rule_ids });
    }

    let regex_id_count = cursor.read_u32()?;
    if regex_id_count > rule_count {
        return Err(EnvelopeError::Invalid);
    }
    let mut regex_rule_ids = Vec::new();
    for _ in 0..regex_id_count {
        regex_rule_ids.push(usize::try_from(cursor.read_u32()?).map_err(|_| return EnvelopeError::Invalid)?);
    }
    let expected_engine_digest: [u8; 32] = cursor
        .take(32)?
        .try_into()
        .map_err(|_| return EnvelopeError::Invalid)?;
    let engine_length = cursor.read_u64()?;
    if engine_length > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    let engine_bytes = cursor.take(usize::try_from(engine_length).map_err(|_| return EnvelopeError::Invalid)?)?;
    if !cursor.is_finished() {
        return Err(EnvelopeError::Invalid);
    }
    let actual_engine_digest = source_digest(engine_bytes).map_err(|_| return EnvelopeError::Invalid)?;
    if digest_bytes(actual_engine_digest) != expected_engine_digest {
        return Err(EnvelopeError::Invalid);
    }
    let regex_set = if engine_bytes.is_empty() {
        None
    } else {
        Some(crate::load_precompiled(engine_bytes).map_err(|_| return EnvelopeError::Invalid)?)
    };
    return RuntimeRules::from_artifact(names, literal_groups, regex_set, regex_rule_ids)
        .map_err(|_| return EnvelopeError::Invalid)
}

/// Registers envelope framing and corruption tests.
#[cfg(test)]
#[path = "envelope_tests.rs"]
mod tests;
