//! What:
//!  The header of one BTF file and the sections it names,
//! with the checks that run
//!  before any type entry is read:
//! the magic,
//! its byte order,
//! the version,
//!  the flags,
//! the header length,
//! and one sorted section list with no gap,
//! no overlap and
//!  the last section ending at the read length.
//! Why:
//!  A file whose sections overlap or stop short of its own length is a file
//! this
//!  process would read out of bounds or short,
//! and the kernel's own reader refuses that
//!  shape,
//! so this one does too:
//! the sections are sorted by offset,
//! compared with one
//!  another,
//! and the last of them must end where the file ends.
//!  Two header lengths exist:
//! the 24 bytes Linux 6.8 wrote,
//! whose string section must end
//!  the file,
//! and the 32 bytes a current tool writes,
//! whose layout section may sit
//!  between the types and the strings.
//! Both are read,
//! and nothing infers a kernel release from
//!  either.
//!  `btf-rs` has no section validation of its own,
//! so these checks are this
//!  crate's and run before it parses a byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sections = readHeader({ bytes, path, step }); // throws Refusal
//! ```

/// The step vocabulary and the refusal builders.
use super::refusal::{
    Refusal,
    Step,
    refusal_at,
};
/// The borrowed path type the refusal names.
use std::path::Path;

/// The two bytes every type-information file starts with,
/// read little-endian on this
/// target.
pub const MAGIC: u16 = 0xeb9f;
/// The version of the format this reader represents.
pub const VERSION: u8 = 1;
/// The flags byte every file this reader represents carries as zero.
pub const FLAGS: u8 = 0;
/// The header length of the format Linux 6.8 wrote,
/// with no layout section.
pub const OLDEST_HEADER_BYTES: usize = 24;
/// The header length that carries the layout section's two fields.
pub const LAYOUT_HEADER_BYTES: usize = 32;
/// The most sections one header names:
/// types,
/// strings and the layout description.
pub const SECTION_COUNT: usize = 3;

/// What:
///  What one file's header states:
/// its own length,
/// which every section offset is counted
///  from.
/// Why:
///   A caller prints the header length beside a pass,
/// and the sections themselves are
///  validated here rather than handed on,
/// so the header is the one fact a reader of this
///  file keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Sections = { headerBytes: number };
/// ```
pub struct Sections {
    /// The header's own length in bytes,
    /// which every section offset is counted from.
    pub header_bytes: usize,
}

/// What:
///  One named section before it is cut out of the file.
/// Why:
///   The order the sections sit in is not the order the header lists them,
/// so each is
///  read as a name,
/// an offset counted from the header's end and a length,
/// then sorted
///  by offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placed = { which: 'types' | 'strings' | 'layout'; offset: usize; length: usize };
/// ```
struct Placed {
    /// Which section this is.
    which: Which,
    /// Bytes from the end of the header to the section's first byte.
    offset: usize,
    /// The section's length in bytes.
    length: usize,
}

/// The section a header field names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Which {
    /// The type entries.
    Types,
    /// The name strings.
    Strings,
    /// The layout description current tools write.
    Layout,
}

/// Read one file's header and check the sections it names.
///
/// # Errors
/// A refusal at the header step naming the file and the check that failed.
pub fn read_sections(
    bytes: &[u8],
    path: &Path,
) -> Result<Sections, Refusal> {
    let step: Step = Step::Header;
    if bytes.len() < OLDEST_HEADER_BYTES {
        return Err(refusal_at(
            step,
            path,
            format!(
                "it holds {} bytes, fewer than the {OLDEST_HEADER_BYTES}-byte oldest header",
                bytes.len()
            ),
        ));
    }
    check_magic(bytes, path)?;
    if bytes[2] != VERSION {
        return Err(refusal_at(
            step,
            path,
            format!("its format version is {}, not {VERSION}", bytes[2]),
        ));
    }
    if bytes[3] != FLAGS {
        return Err(refusal_at(
            step,
            path,
            format!("its flags byte is {}, not {FLAGS}", bytes[3]),
        ));
    }
    let header_bytes: usize = read_u32(bytes, 4) as usize;
    if header_bytes < OLDEST_HEADER_BYTES || header_bytes > bytes.len() {
        return Err(refusal_at(
            step,
            path,
            format!(
                "its header length is {header_bytes} bytes, and this reader accepts {OLDEST_HEADER_BYTES} up to {} bytes",
                bytes.len()
            ),
        ));
    }
    if header_bytes != OLDEST_HEADER_BYTES && header_bytes < LAYOUT_HEADER_BYTES {
        return Err(refusal_at(
            step,
            path,
            format!(
                "its header is {header_bytes} bytes, between the {OLDEST_HEADER_BYTES}-byte oldest header and the {LAYOUT_HEADER_BYTES}-byte header whose layout fields this reader knows"
            ),
        ));
    }
    let places: Vec<Placed> = places_of(bytes, header_bytes);
    check_sections(bytes, header_bytes, &places, path, step)?;
    return Ok(Sections {
        header_bytes,
    });
}

/// Refuse a file whose first two bytes are not the magic in this target's byte order.
///
/// # Errors
/// A refusal naming whether the file is the other byte order or no type information at all.
fn check_magic(
    bytes: &[u8],
    path: &Path,
) -> Result<(), Refusal> {
    let step: Step = Step::Header;
    let little: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
    if little == MAGIC {
        return Ok(());
    }
    let big: u16 = u16::from_be_bytes([bytes[0], bytes[1]]);
    if big == MAGIC {
        return Err(refusal_at(
            step,
            path,
            "its type information is big-endian, and this target is little-endian".to_string(),
        ));
    }
    return Err(refusal_at(
        step,
        path,
        format!("its magic is 0x{little:04x}, not 0x{MAGIC:04x}"),
    ));
}

/// The sections the header names,
/// each with the offset it was given.
fn places_of(
    bytes: &[u8],
    header_bytes: usize,
) -> Vec<Placed> {
    let mut places: Vec<Placed> = Vec::<Placed>::with_capacity(SECTION_COUNT);
    places.push(Placed {
        which: Which::Types,
        offset: read_u32(bytes, 8) as usize,
        length: read_u32(bytes, 12) as usize,
    });
    places.push(Placed {
        which: Which::Strings,
        offset: read_u32(bytes, 16) as usize,
        length: read_u32(bytes, 20) as usize,
    });
    if header_bytes >= LAYOUT_HEADER_BYTES {
        let length: usize = read_u32(bytes, 28) as usize;
        if length > 0 {
            places.push(Placed {
                which: Which::Layout,
                offset: read_u32(bytes, 24) as usize,
                length,
            });
        }
    }
    places.sort_by_key(|place| {
        return place.offset;
    });
    return places;
}

/// Refuse a section list with a gap,
/// an overlap or an end before the file's.
///
/// # Errors
/// A refusal at the header step naming the two sections that do not add up.
fn check_sections(
    bytes: &[u8],
    header_bytes: usize,
    places: &[Placed],
    path: &Path,
    step: Step,
) -> Result<(), Refusal> {
    let mut expected_start: usize = 0;
    for place in places {
        if place.offset < expected_start {
            return Err(refusal_at(
                step,
                path,
                format!(
                    "its {:?} section starts at byte {} from the header's end, before the {} a section already ends at",
                    place.which, place.offset, expected_start
                ),
            ));
        }
        if place.offset > expected_start {
            return Err(refusal_at(
                step,
                path,
                format!(
                    "its {:?} section starts at byte {} from the header's end, leaving a gap after {}",
                    place.which, place.offset, expected_start
                ),
            ));
        }
        let end: usize = place.offset.saturating_add(place.length);
        let total: usize = header_bytes.saturating_add(end);
        if total > bytes.len() {
            return Err(refusal_at(
                step,
                path,
                format!(
                    "its {:?} section ends at byte {} of a {}-byte file",
                    place.which,
                    total,
                    bytes.len()
                ),
            ));
        }
        expected_start = end;
    }
    let total: usize = header_bytes.saturating_add(expected_start);
    if total != bytes.len() {
        return Err(refusal_at(
            step,
            path,
            format!(
                "its sections end at byte {total} of a {}-byte file",
                bytes.len()
            ),
        ));
    }
    let strings: &Placed = place_of(places, Which::Strings, path, step)?;
    if header_bytes == OLDEST_HEADER_BYTES {
        let strings_end: usize = header_bytes.saturating_add(strings.offset + strings.length);
        if strings_end != bytes.len() {
            return Err(refusal_at(
                step,
                path,
                format!(
                    "a {OLDEST_HEADER_BYTES}-byte header requires the string section to end the file, and it ends at byte {strings_end} of {}",
                    bytes.len()
                ),
            ));
        }
    }
    return Ok(());
}

/// The section of one kind,
/// or a refusal when the header names none.
///
/// # Errors
/// A refusal at the header step naming the section that is absent.
fn place_of<'a>(
    places: &'a [Placed],
    which: Which,
    path: &Path,
    step: Step,
) -> Result<&'a Placed, Refusal> {
    let found: Option<&Placed> = places.iter().find(|place| {
        return place.which == which;
    });
    let Some(place): Option<&Placed> = found else {
        return Err(refusal_at(
            step,
            path,
            format!("it names no {which:?} section"),
        ));
    };
    return Ok(place);
}

/// One little-endian 32-bit field of the header.
fn read_u32(
    bytes: &[u8],
    at: usize,
) -> u32 {
    let mut field: [u8; 4] = [0; 4];
    field.copy_from_slice(&bytes[at..at + 4]);
    return u32::from_le_bytes(field);
}
