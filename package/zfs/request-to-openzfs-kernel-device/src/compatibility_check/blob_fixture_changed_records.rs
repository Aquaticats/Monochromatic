//! What:
//!  The changes a test makes to an entry a blob already wrote:
//! the kind flag set on every record,
//! one member whose name no reader can read,
//! and one byte moved.
//! Why:
//!  A test that refuses a changed module has to hand the check a file no writer
//!  would produce,
//! so the change is stated as one write into the entry's own bytes rather than
//!  as a second table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusable: Blob = written.with_kind_flag_on_records(); const moved: number[] = Blob.with_byte(bytes, 8, 0x02);
//! ```

/// The blob an entry is changed in.
use super::blob_fixture::Blob;
/// The kind numbers and the name offset the changes write.
use super::blob_fixture::{
    KIND_STRUCT,
    KIND_UNION,
    UNREADABLE_NAME,
    VLEN_MASK,
};

/// One little-endian 32-bit word at `at`,
/// and nothing for a file too short to hold it.
fn word_in(
    bytes: &[u8],
    at: usize,
) -> u32 {
    let mut field: [u8; 4] = [0; 4];
    let Some(source): Option<&[u8]> = bytes.get(at..at + 4) else {
        return 0;
    };
    field.copy_from_slice(source);
    return u32::from_le_bytes(field);
}

/// Write one little-endian 32-bit word at `at`,
/// which every writer of this file does through this one place.
fn set_word(
    bytes: &mut [u8],
    at: usize,
    value: u32,
) {
    let Some(field): Option<&mut [u8]> = bytes.get_mut(at..at + 4) else {
        return;
    };
    field.copy_from_slice(&value.to_le_bytes());
}

/// The changes one test makes to an entry it already wrote.
impl Blob {
    /// A copy of the file whose every record carries the kind flag,
    /// with every member's offset and width as written.
    pub fn with_kind_flag_on_records(mut self) -> Self {
        for entry in &mut self.entries {
            let info: u32 = word_in(entry, 4);
            let kind: u32 = (info >> 24) & 0x7f;
            if kind == KIND_STRUCT || kind == KIND_UNION {
                let flagged: u32 = info | (1 << 31);
                set_word(entry, 4, flagged);
            }
        }
        return self;
    }

    /// One more member in an entry the writer already holds,
    /// whose name offset points past this file's string section.
    ///
    /// A reader cannot read that name,
    /// so the member is the one a reader must refuse rather than drop.
    pub fn member_with_unreadable_name(
        &mut self,
        of: u32,
        type_id: u32,
        bit_offset: u32,
    ) {
        let Some(entry): Option<&mut Vec<u8>> =
            self.entries.get_mut((of as usize).saturating_sub(1))
        else {
            panic!("entry {of} is not one this writer wrote");
        };
        let info: u32 = word_in(entry, 4);
        let members: u32 = info & VLEN_MASK;
        entry.extend_from_slice(&UNREADABLE_NAME.to_le_bytes());
        entry.extend_from_slice(&type_id.to_le_bytes());
        entry.extend_from_slice(&bit_offset.to_le_bytes());
        set_word(entry, 4, (info & !VLEN_MASK) | (members + 1));
    }

    /// The same file with its bytes changed at `at`,
    /// so a test can move one field.
    pub fn with_byte(
        mut file: Vec<u8>,
        at: usize,
        byte: u8,
    ) -> Vec<u8> {
        if let Some(slot) = file.get_mut(at) {
            *slot = byte;
        }
        return file;
    }
}
