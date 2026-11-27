//! What:
//!  The decoder:
//!  one complete artifact frame in,
//!  one hybrid rule bundle out,
//!  read through a checked forward-only cursor over the borrowed bytes.
//! Why:
//!  This is the only path by which a file under the user's cache root
//!  reaches a scan,
//!  so every read is bounded by the declared ceiling and by the bytes that
//!  remain,
//!  the trailing payload and engine digests are re-derived and compared,
//!  and no trailing byte is tolerated.
//!  The cursor lives here because nothing else reads the frame.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function decode(bytes: Uint8Array, expected: SourceDigest): RuntimeRules
//! ```

/// Imports the rejection type and the adapter that names a failure without
///  disclosing it.
use super::error::{
    EnvelopeError,
    invalid_envelope,
};
/// Imports the frame's marker,
///  version,
///  ceilings,
///  name markers and the name grammar the decoder re-checks.
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
/// Imports the hybrid rule bundle and its literal groups,
///  which the frame rebuilds.
use crate::runtime_matcher::{
    LiteralGroup,
    RuntimeRules,
};

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
        return Self {
            bytes,
            position: 0,
        };
    }

    /// Borrows exact next byte count or rejects truncation and overflow.
    fn take(
        &mut self,
        count: usize,
    ) -> Result<&'a [u8], EnvelopeError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(EnvelopeError::Invalid)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(EnvelopeError::Invalid)?;
        self.position = end;
        return Ok(value);
    }

    /// Reads one byte.
    fn read_u8(&mut self) -> Result<u8, EnvelopeError> {
        let bytes = self.take(1)?;
        return Ok(bytes[0]);
    }

    /// Reads little-endian 16-bit integer.
    fn read_u16(&mut self) -> Result<u16, EnvelopeError> {
        let bytes: [u8; 2] = self.take(2)?.try_into().map_err(invalid_envelope)?;
        return Ok(u16::from_le_bytes(bytes));
    }

    /// Reads little-endian 32-bit integer.
    fn read_u32(&mut self) -> Result<u32, EnvelopeError> {
        let bytes: [u8; 4] = self.take(4)?.try_into().map_err(invalid_envelope)?;
        return Ok(u32::from_le_bytes(bytes));
    }

    /// Reads little-endian 64-bit integer.
    fn read_u64(&mut self) -> Result<u64, EnvelopeError> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(invalid_envelope)?;
        return Ok(u64::from_le_bytes(bytes));
    }

    /// Reads bounded UTF-8 identity string.
    fn read_identity(&mut self) -> Result<&'a str, EnvelopeError> {
        let length = self.read_u16()?;
        if length == 0 || length > MAX_IDENTITY_BYTES {
            return Err(EnvelopeError::Invalid);
        }
        return std::str::from_utf8(self.take(usize::from(length))?).map_err(invalid_envelope);
    }

    /// Reports whether all artifact bytes were consumed.
    fn is_finished(&self) -> bool {
        return self.position == self.bytes.len();
    }
}

/// Decodes and validates one complete hybrid envelope against source identity.
pub(crate) fn decode(
    bytes: &[u8],
    expected_digest: SourceDigest,
) -> Result<RuntimeRules, EnvelopeError> {
    if u64::try_from(bytes.len()).map_err(invalid_envelope)? > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    let mut header_cursor = Cursor::new(bytes);
    if header_cursor.take(MAGIC.len())? != MAGIC {
        return Err(EnvelopeError::Invalid);
    }
    if header_cursor.read_u32()? != SCHEMA_VERSION {
        return Err(EnvelopeError::Incompatible);
    }
    if header_cursor.read_identity()? != scanner_version()
        || header_cursor.read_identity()? != platform_identity()
    {
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
    let expected_payload_digest: [u8; 32] =
        bytes[payload_end..].try_into().map_err(invalid_envelope)?;
    let actual_payload_digest =
        source_digest(&bytes[payload_start..payload_end]).map_err(invalid_envelope)?;
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
        let name_bytes = cursor.take(usize::try_from(name_length).map_err(invalid_envelope)?)?;
        let name = std::str::from_utf8(name_bytes).map_err(invalid_envelope)?;
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
            .take(usize::try_from(literal_length).map_err(invalid_envelope)?)?
            .to_vec();
        let id_count = cursor.read_u32()?;
        if id_count == 0 || id_count > rule_count {
            return Err(EnvelopeError::Invalid);
        }
        let mut rule_ids = Vec::new();
        for _ in 0..id_count {
            rule_ids.push(usize::try_from(cursor.read_u32()?).map_err(invalid_envelope)?);
        }
        literal_groups.push(LiteralGroup {
            bytes: literal,
            rule_ids,
        });
    }

    let regex_id_count = cursor.read_u32()?;
    if regex_id_count > rule_count {
        return Err(EnvelopeError::Invalid);
    }
    let mut regex_rule_ids = Vec::new();
    for _ in 0..regex_id_count {
        regex_rule_ids.push(usize::try_from(cursor.read_u32()?).map_err(invalid_envelope)?);
    }
    let expected_engine_digest: [u8; 32] = cursor.take(32)?.try_into().map_err(invalid_envelope)?;
    let engine_length = cursor.read_u64()?;
    if engine_length > MAX_ARTIFACT_BYTES {
        return Err(EnvelopeError::Invalid);
    }
    let engine_bytes = cursor.take(usize::try_from(engine_length).map_err(invalid_envelope)?)?;
    if !cursor.is_finished() {
        return Err(EnvelopeError::Invalid);
    }
    let actual_engine_digest = source_digest(engine_bytes).map_err(invalid_envelope)?;
    if digest_bytes(actual_engine_digest) != expected_engine_digest {
        return Err(EnvelopeError::Invalid);
    }
    let regex_set = if engine_bytes.is_empty() {
        None
    } else {
        Some(crate::load_precompiled(engine_bytes).map_err(invalid_envelope)?)
    };
    return RuntimeRules::from_artifact(names, literal_groups, regex_set, regex_rule_ids)
        .map_err(invalid_envelope);
}
