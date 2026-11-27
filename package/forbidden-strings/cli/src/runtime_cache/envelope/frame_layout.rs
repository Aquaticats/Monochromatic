//! What:
//!  The artifact's frame layout,
//!  as data:
//!  the eight-byte marker every artifact starts with,
//!  the outer schema version,
//!  every ceiling a declared size is checked against,
//!  the markers saying whether a rule has a name,
//!  and the grammar a decoded name must obey.
//! Why:
//!  The encoder and the decoder must agree on every number and every rule,
//!  and a ceiling that lives in only one of them is a bound the other one
//!  does not enforce.
//!  Keeping them in one file means a change to the frame is one edit in one
//!  place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const MAGIC = Buffer.from('FSRULES\0'); const MAX_ARTIFACT_BYTES = 512 * 1024 * 1024;
//! ```

/// Exact eight-byte artifact marker.
pub(crate) const MAGIC: &[u8; 8] = b"FSRULES\0";
/// Scanner-owned outer envelope schema version.
pub(super) const SCHEMA_VERSION: u32 = 2;
/// Maximum complete artifact accepted before reading or decoding:
///  512 MiB.
pub(crate) const MAX_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
/// Maximum rule identities decoded into memory.
pub(super) const MAX_RULE_COUNT: u32 = 1_000_000;
/// Maximum UTF-8 bytes in one section name.
pub(super) const MAX_RULE_NAME_BYTES: u32 = 4_096;
/// Maximum exact-literal bytes in one cached group.
pub(super) const MAX_LITERAL_BYTES: u32 = 16 * 1024 * 1024;
/// Maximum UTF-8 bytes in version or platform identity.
pub(super) const MAX_IDENTITY_BYTES: u16 = 128;
/// Marker for unnamed legacy rule identity.
pub(super) const NAME_ABSENT: u8 = 0;
/// Marker for named tail-format rule identity.
pub(super) const NAME_PRESENT: u8 = 1;

/// Reports whether decoded name obeys strict tail-format section-name grammar.
pub(super) fn valid_rule_name(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    return characters.all(is_rule_name_character);
}

/// Whether one character after the first may appear in a rule name.
pub(super) fn is_rule_name_character(character: char) -> bool {
    return character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || character == '.'
        || character == '-';
}
