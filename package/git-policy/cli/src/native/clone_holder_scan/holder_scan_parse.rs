//! What:
//!  Reading one holder scan's result back:
//!  the byte bound before any line,
//!  the version and kind,
//!  each key at its own place,
//!  every count against the lines that follow it,
//!  and each path and sentence decoded with the writer's own rules.
//! Why:
//!  A later step carries this answer across a privilege boundary,
//!  so the reader checks every field rather than trusting the file:
//!  a truncated,
//!  reordered,
//!  padded or foreign file is refused,
//!  and the GUID and the scan's nonce are read so a caller can compare them
//!  with the scan it asked for
//!  (`crate::clone_holder_scan::holder_scan_acceptance`).
//!  The bound comes first so a huge file is refused without being split into
//!  lines.
//!  The place a line is read at is `crate::clone_holder_scan::holder_scan_parse_cursor`'s,
//!  the fields of one line are `crate::clone_holder_scan::holder_scan_parse_line_fields`'s,
//!  and one incomplete read is `crate::clone_holder_scan::holder_scan_parse_incomplete_reads`'s;
//!  this file places the lines,
//!  counts each section and answers the whole result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = parseHolderScan(text); // throws HolderScanRefusal
//! ```

/// One mount of the dataset as a report names it.
use crate::clone_holder_scan::held_mounts::HeldMount;
/// The scan and the value it answers.
use crate::clone_holder_scan::holder_scan::HolderScan;
/// The place the next line is expected at.
use crate::clone_holder_scan::holder_scan_parse_cursor::Cursor;
/// One incomplete read,
/// read by the cause its own word names.
use crate::clone_holder_scan::holder_scan_parse_incomplete_reads::incomplete_of;
/// The fields a mount line and a holder line carry.
use crate::clone_holder_scan::holder_scan_parse_line_fields::{
    holder_fields,
    mount_after_the_key,
};
/// Why a line that is not the field the reader expected refuses the whole result.
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
/// The result's words,
/// its bound,
/// the keys in the order it writes them and the time's shape.
use crate::clone_holder_scan::holder_scan_text::{
    ABSENT_WORD,
    HOLDER_KEY,
    INCOMPLETE_KEY,
    MAXIMUM_NAMESPACE_BYTES,
    MAXIMUM_RESULT_BYTES,
    MAXIMUM_SOURCE_BYTES,
    OWN_MOUNT_KEY,
    RESULT_KEYS,
    RESULT_KIND,
    TIME_KEY,
    VERSION_KEY,
    is_a_result_time,
    unescape_text,
};
/// One incomplete read.
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
/// One namespace and the mounts of the dataset it holds.
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
/// The dataset's GUID and the request's nonce,
/// each parsed where it is read.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
/// One section of holder lines,
/// one line per namespace.
fn holder_section(
    cursor: &mut Cursor<'_>,
    count_key: &'static str,
) -> Result<Vec<NamespaceHolder>, HolderScanRefusal> {
    let count: usize = cursor.section_count(count_key)?;
    let mut holders: Vec<NamespaceHolder> = Vec::with_capacity(count);
    for _line in 0..count {
        holders.push(holder_fields(HOLDER_KEY, cursor.keyed(HOLDER_KEY)?)?);
    }
    return Ok(holders);
}

/// One section of the reader's own mounts.
fn own_mount_section(cursor: &mut Cursor<'_>) -> Result<Vec<HeldMount>, HolderScanRefusal> {
    let count: usize = cursor.section_count(RESULT_KEYS[6])?;
    let mut mounts: Vec<HeldMount> = Vec::with_capacity(count);
    for _line in 0..count {
        mounts.push(mount_after_the_key(
            OWN_MOUNT_KEY,
            cursor.keyed(OWN_MOUNT_KEY)?,
        )?);
    }
    return Ok(mounts);
}

/// One section of the reads the scan could not complete.
fn incomplete_section(cursor: &mut Cursor<'_>) -> Result<Vec<IncompleteRead>, HolderScanRefusal> {
    let count: usize = cursor.section_count(RESULT_KEYS[8])?;
    let mut reads: Vec<IncompleteRead> = Vec::with_capacity(count);
    for _line in 0..count {
        reads.push(incomplete_of(cursor.keyed(INCOMPLETE_KEY)?)?);
    }
    return Ok(reads);
}

/// Read one holder scan's result.
///
/// # Errors
/// The text is past the bound,
/// not text,
/// or any line's key,
/// count,
/// value or order does not fit a result this route writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = parseHolderScan(text);
/// ```
pub fn parse_holder_scan(text: &str) -> Result<HolderScan, HolderScanRefusal> {
    if text.len() > MAXIMUM_RESULT_BYTES {
        return Err(HolderScanRefusal::TooLarge {
            bytes: text.len(),
            bound: MAXIMUM_RESULT_BYTES,
        });
    }
    let lines: Vec<&str> = text.lines().collect();
    let Some(version) = lines.first().copied() else {
        return Err(HolderScanRefusal::MissingLine {
            key: VERSION_KEY,
        });
    };
    if version != VERSION_KEY {
        return Err(HolderScanRefusal::Version {
            found: String::from(version),
        });
    }
    let mut cursor: Cursor<'_> = Cursor::after_the_version_line(lines.as_slice());
    let kind: &str = cursor.keyed(RESULT_KEYS[0])?;
    if kind != RESULT_KIND {
        return Err(HolderScanRefusal::Kind {
            found: String::from(kind),
        });
    }
    let dataset: DatasetGuid =
        DatasetGuid::parse(cursor.keyed(RESULT_KEYS[1])?).map_err(HolderScanRefusal::Dataset)?;
    let nonce: RequestNonce =
        RequestNonce::parse(cursor.keyed(RESULT_KEYS[2])?).map_err(HolderScanRefusal::Nonce)?;
    let source: &str = cursor.keyed(RESULT_KEYS[3])?;
    if source.len() > MAXIMUM_SOURCE_BYTES {
        return Err(HolderScanRefusal::Value {
            key: RESULT_KEYS[3],
            shown: String::from(source),
        });
    }
    let own_namespace: &str = cursor.keyed(RESULT_KEYS[4])?;
    if own_namespace.len() > MAXIMUM_NAMESPACE_BYTES {
        return Err(HolderScanRefusal::Value {
            key: RESULT_KEYS[4],
            shown: String::from(own_namespace),
        });
    }
    let namespaces_scanned: usize = cursor.bare_count(RESULT_KEYS[5])?;
    let own_mounts: Vec<HeldMount> = own_mount_section(&mut cursor)?;
    let holders: Vec<NamespaceHolder> = holder_section(&mut cursor, RESULT_KEYS[7])?;
    let incomplete: Vec<IncompleteRead> = incomplete_section(&mut cursor)?;
    let time: &str = cursor.keyed(TIME_KEY)?;
    if !is_a_result_time(time) {
        return Err(HolderScanRefusal::Time {
            value: String::from(time),
        });
    }
    cursor.finish()?;
    return Ok(HolderScan {
        dataset,
        source: unescape_text(source),
        nonce,
        own_namespace: if own_namespace == ABSENT_WORD {
            None
        } else {
            Some(unescape_text(own_namespace))
        },
        namespaces_scanned,
        own_mounts,
        holders,
        incomplete,
        time: String::from(time),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_parse_tests.rs"]
mod tests;
