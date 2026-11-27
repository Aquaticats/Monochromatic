//!  Encodes and validates scanner-owned runtime cache envelopes.
//!
//!  Framing is explicit little-endian data around the engine's existing bincode
//!  bytes.
//!  Every declared size is checked against a fixed ceiling and remaining
//!  input before allocation or slicing.
//!  The decoder accepts no trailing bytes.
//!
//!  The parts,
//!  in the order they build on each other:
//!  `frame_layout` holds the marker,
//!  schema version,
//!  size ceilings and the section-name grammar both directions share;
//!  `error` holds the one rejection type and its warning-reason mapping;
//!  `encode` writes a frame;
//!  `decode` reads one back.
//!  The re-exports below keep this module's surface exactly what the cache's
//!  callers already import.

/// What:
///  `mod decode;` registers the reader and its checked cursor.
/// Why:
///  A corrupt artifact must be rejected,
///  never half-read.
mod decode;
/// What:
///  `mod encode;` registers the writer of one complete artifact.
/// Why:
///  Publication and the `compile-rules` command both go through it.
mod encode;
/// What:
///  `mod error;` registers the redacted rejection type and the `map_err`
///  adapter that names a failure without disclosing it.
/// Why:
///  A rejection ends up in a warning whose reason values are closed.
mod error;
/// What:
///  `mod frame_layout;` registers the frame's numbers and the grammar a
///  rule name must obey.
/// Why:
///  The encoder and the decoder must bound every size the same way.
mod frame_layout;

/// What:
///  `pub(super) use decode::decode;` re-exports the reader.
/// Why:
///  A cache read calls it as `envelope::decode`.
pub(super) use decode::decode;
/// What:
///  `pub(super) use encode::encode;` re-exports the writer.
/// Why:
///  Publication calls it as `envelope::encode`.
pub(super) use encode::encode;
/// What:
///  `pub(super) use error::EnvelopeError;` re-exports the rejection type for
///  the tests that assert each variant's reason;
///  the cache's own callers
///  only see the warning the mapping produces.
/// Why:
///  Moving the type into a part must not move its path.
#[cfg(test)]
pub(super) use error::EnvelopeError;
/// What:
///  `pub(super) use frame_layout::MAGIC;` re-exports the artifact marker for
///  the framing tests,
///  which address fields by offset;
///  only those tests
///  name it,
///  so the re-export exists only for them.
/// Why:
///  A fixture that cannot name the marker cannot compute an offset.
#[cfg(test)]
pub(super) use frame_layout::MAGIC;
/// What:
///  `pub(super) use frame_layout::MAX_ARTIFACT_BYTES;` re-exports the one
///  ceiling the cache's publisher needs before it reads a file at all.
/// Why:
///  A file over it is refused without being decoded.
pub(super) use frame_layout::MAX_ARTIFACT_BYTES;

/// Registers envelope framing and corruption tests.
#[cfg(test)]
#[path = "envelope_tests.rs"]
mod tests;
