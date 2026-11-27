//! What:
//! The bytes of a compiled program, built by hand for the tests:
//! a header,
//! one program-header entry,
//! and the interpreter's path when the program names one.
//! Why:
//! A sentence in the message has to name the file a compiled program's start needs,
//! and the reader of the program's own file is what answers which file that is.
//! A copy of a program on this machine would test whatever that program happens to name;
//! bytes built here state exactly the entry under test.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function elf(layout: Layout, bigEndian: boolean, interpreter?: string): Uint8Array
//! ```

/// The two header layouts,
/// as the numbers that place their fields.
pub struct Layout {
    /// Bytes the header takes.
    pub header: usize,
    /// Bytes one program-header entry takes.
    pub entry: usize,
    /// Where the table's offset is written.
    pub offset_field: usize,
    /// Where an entry's size is written.
    pub size_field: usize,
    /// Where the entry count is written.
    pub count_field: usize,
    /// Where an entry's payload offset is written.
    pub entry_offset_field: usize,
    /// Where an entry's payload size is written.
    pub entry_size_field: usize,
    /// Bytes a payload offset takes.
    pub offset_width: usize,
}

/// The 64-bit layout of a compiled program's header.
pub const WIDE: Layout = Layout {
    header: 64,
    entry: 56,
    offset_field: 32,
    size_field: 54,
    count_field: 56,
    entry_offset_field: 8,
    entry_size_field: 32,
    offset_width: 8,
};

/// The 32-bit layout of a compiled program's header.
pub const NARROW: Layout = Layout {
    header: 52,
    entry: 32,
    offset_field: 28,
    size_field: 42,
    count_field: 44,
    entry_offset_field: 4,
    entry_size_field: 16,
    offset_width: 4,
};

/// What:
/// `fn put(buffer: &mut [u8], offset: usize, value: u64, width: usize, big_endian: bool)` writes `value` into
/// `buffer` at `offset` as `width` bytes in the order the file uses.
/// Why:
/// A compiled program's header is numbers at fixed offsets,
/// and both byte orders have to be buildable to test the reader that handles them.
fn put(
    buffer: &mut [u8],
    offset: usize,
    value: u64,
    width: usize,
    big_endian: bool,
) {
    for index in 0..width {
        let shift: u32 = 8 * u32::try_from(if big_endian { width - 1 - index } else { index })
            .expect("a shift inside the field");
        let byte: u8 = u8::try_from((value >> shift) & 0xff).expect("one byte");
        buffer[offset + index] = byte;
    }
}

/// What:
/// `pub fn elf(layout: &Layout, big_endian: bool, interpreter: Option<&str>) -> Vec<u8>` builds a compiled
/// program's bytes with the interpreter's path as its one entry.
/// `None` writes an entry of another kind, as a program that names no interpreter does.
/// Why:
/// One builder covers all four combinations of class and byte order,
/// so each test states only what it is about.
pub fn elf(
    layout: &Layout,
    big_endian: bool,
    interpreter: Option<&str>,
) -> Vec<u8> {
    let text: &str = interpreter.unwrap_or("");
    let text_offset: usize = layout.header + layout.entry;
    let mut buffer: Vec<u8> = vec![0; text_offset + text.len()];
    buffer[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
    buffer[4] = if layout.header == WIDE.header { 2 } else { 1 };
    buffer[5] = if big_endian { 2 } else { 1 };
    put(
        &mut buffer,
        layout.offset_field,
        u64::try_from(layout.header).expect("a small offset"),
        layout.offset_width,
        big_endian,
    );
    put(
        &mut buffer,
        layout.size_field,
        u64::try_from(layout.entry).expect("a small entry"),
        2,
        big_endian,
    );
    put(&mut buffer, layout.count_field, 1, 2, big_endian);
    let kind: u64 = if interpreter.is_some() { 3 } else { 1 };
    put(&mut buffer, layout.header, kind, 4, big_endian);
    put(
        &mut buffer,
        layout.header + layout.entry_offset_field,
        u64::try_from(text_offset).expect("a small offset"),
        layout.offset_width,
        big_endian,
    );
    put(
        &mut buffer,
        layout.header + layout.entry_size_field,
        u64::try_from(text.len()).expect("a short text"),
        layout.offset_width,
        big_endian,
    );
    buffer[text_offset..].copy_from_slice(text.as_bytes());
    return buffer;
}
