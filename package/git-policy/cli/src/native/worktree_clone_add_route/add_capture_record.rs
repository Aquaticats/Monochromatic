//! What:
//!  The capture's own record grammar:
//! its bytes in the grammar's order,
//! and a
//!  reader that refuses every shape it did not write.
//! Why:
//!  A resume after a kill proves the clone against the capture only if the
//!  capture outlives the process,
//! so it is a durable record of its own beside
//!  the binding
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What 'the captured source' is").
//!  It is keyed by the add's nonce as well as its slot,
//! so a capture another add
//!  left is never read as this one's.
//!  The widths its bound rests on are `add_capture_record_bound.rs`'s and the
//!  fields its reader refuses are `add_capture_record_fields.rs`'s,
//!  so this file is the order the writer puts its lines in and the reader's
//!  count and order,
//! which are the writer's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = encodeCapture(capture); // the header and one line per entry
//! const capture = parseCapture(text); // throws with the line it refused
//! ```

use super::add_capture::{
    Capture,
    CapturedDigest,
    CapturedEntry,
    CapturedIndex,
    MAXIMUM_PATH_BYTES,
    digest_words,
    entry_words,
};
use super::add_capture_record_fields::{
    CaptureParseRefusal,
    commit_named,
    decimal_named,
    digest_named,
    hexadecimal_bytes_named,
    parse_digest,
    parse_entry,
    time_named,
};
use crate::worktree_clone_binding_record::binding_encoding::bytes_as_hexadecimal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The first line of a capture,
/// naming the grammar's version.
pub use super::add_capture_record_bound::CAPTURE_HEADER;

/// The keys the fixed part writes,
/// in the order it writes them.
pub use super::add_capture_record_bound::CAPTURE_FIXED_KEYS;

/// The largest capture one entry count and its digests can name.
pub use super::add_capture_record_bound::capture_bound;

/// One captured entry's widest line,
/// which the bound counts per entry.
pub use super::add_capture_record_bound::CAPTURE_ENTRY_LINE_MAXIMUM_BYTES;

/// One digest's widest line,
/// which the bound counts per digest.
pub use super::add_capture_record_bound::CAPTURE_DIGEST_LINE_MAXIMUM_BYTES;

/// The capture's fixed lines' widest total.
pub use super::add_capture_record_bound::CAPTURE_FIXED_BYTES;

/// What:
///  A capture's bytes,
/// in the grammar's order.
/// Why:
///  The writer puts every fixed key on its line and one line per entry,
/// so the
///  reader's count and order are the writer's own.
pub fn encode_capture(capture: &Capture) -> Vec<u8> {
    let mut text: String = String::from(CAPTURE_HEADER);
    text.push('\n');
    text.push_str(format!("nonce={}\n", capture.nonce.as_str()).as_str());
    text.push_str(
        format!(
            "repository={}\n",
            bytes_as_hexadecimal(capture.repository.as_slice())
        )
        .as_str(),
    );
    text.push_str(
        format!(
            "source={}\n",
            bytes_as_hexadecimal(capture.source.as_slice())
        )
        .as_str(),
    );
    text.push_str(format!("checkoutGuid={}\n", capture.checkout_guid.file_name()).as_str());
    text.push_str(format!("indexSize={}\n", capture.index.size).as_str());
    text.push_str(
        format!(
            "indexMtime={}.{}\n",
            capture.index.mtime.0, capture.index.mtime.1
        )
        .as_str(),
    );
    text.push_str(
        format!(
            "indexDigest={}\n",
            bytes_as_hexadecimal(capture.index.digest.as_slice())
        )
        .as_str(),
    );
    text.push_str(format!("head={}\n", commit_word(capture.head.as_str())).as_str());
    text.push_str(format!("sourceHead={}\n", commit_word(capture.source_head.as_str())).as_str());
    text.push_str(format!("entries={}\n", capture.entries.len()).as_str());
    text.push_str(format!("digests={}\n", capture.digests.len()).as_str());
    for entry in &capture.entries {
        text.push_str(format!("entry={}\n", entry_words(entry)).as_str());
    }
    for digest in &capture.digests {
        text.push_str(format!("digest={}\n", digest_words(digest)).as_str());
    }
    return text.into_bytes();
}

/// One commit's word:
/// its hexadecimal name,
/// or `-` when the branch is unborn.
fn commit_word(commit: &str) -> &str {
    return if commit.is_empty() { "-" } else { commit };
}

/// What:
///  Read one capture's bytes.
/// Why:
///  A record whose count disagrees with its lines,
/// whose keys arrive out of
///  order or twice,
/// or whose values do not fit their types is damage or a later
///  grammar,
/// and neither is read under this one.
///
/// # Errors
/// The header,
/// a key,
/// a value or the count does not fit the grammar.
pub fn parse_capture(text: &str) -> Result<Capture, CaptureParseRefusal> {
    let mut lines = text.lines();
    match lines.next() {
        Some(first) if first == CAPTURE_HEADER => {}
        Some(first) => {
            return Err(format!(
                "the capture's first line is `{first}`, and this grammar's is `{CAPTURE_HEADER}`"
            ));
        }
        None => return Err(String::from("the capture is empty")),
    }
    let mut values: Vec<&str> = Vec::with_capacity(CAPTURE_FIXED_KEYS.len());
    for key in CAPTURE_FIXED_KEYS {
        let Some(line) = lines.next() else {
            return Err(format!("the capture ends before `{key}=`"));
        };
        match line
            .strip_prefix(key)
            .and_then(|rest| return rest.strip_prefix('='))
        {
            Some(value) => values.push(value),
            None => return Err(format!("the capture holds `{line}` where `{key}=` belongs")),
        }
    }
    let nonce: RequestNonce = RequestNonce::parse(values[0]).map_err(|refusal| {
        return format!("the capture's nonce is not 32 lowercase hexadecimal digits ({refusal:?})");
    })?;
    let repository: Vec<u8> = hexadecimal_bytes_named(values[1], "repository", MAXIMUM_PATH_BYTES)?;
    let source: Vec<u8> = hexadecimal_bytes_named(values[2], "source", MAXIMUM_PATH_BYTES)?;
    let checkout_guid: DatasetGuid = DatasetGuid::parse(values[3]).map_err(|refusal| {
        return format!(
            "the capture's checkout dataset GUID is not canonical decimal ({refusal:?})"
        );
    })?;
    let index_size: u64 = decimal_named(values[4], "indexSize")?;
    let index_mtime: (i64, u32) = time_named(values[5], "indexMtime")?;
    let digest: [u8; 32] = digest_named(values[6])?;
    let head: String = commit_named(values[7], "head")?;
    let source_head: String = commit_named(values[8], "sourceHead")?;
    let count: usize =
        usize::try_from(decimal_named(values[9], "entries")?).map_err(|_overflow| {
            return String::from("the capture's entry count does not fit this machine");
        })?;
    let digest_count: usize =
        usize::try_from(decimal_named(values[10], "digests")?).map_err(|_overflow| {
            return String::from("the capture's digest count does not fit this machine");
        })?;
    let mut entries: Vec<CapturedEntry> = Vec::with_capacity(count.min(1 << 16));
    for _ in 0..count {
        let Some(line) = lines.next() else {
            return Err(format!(
                "the capture says entries={count} and holds fewer entry lines"
            ));
        };
        let Some(words) = line.strip_prefix("entry=") else {
            return Err(format!("the capture holds `{line}` where `entry=` belongs"));
        };
        entries.push(parse_entry(words)?);
    }
    let mut digests: Vec<CapturedDigest> = Vec::with_capacity(digest_count.min(1 << 16));
    for _ in 0..digest_count {
        let Some(line) = lines.next() else {
            return Err(format!(
                "the capture says digests={digest_count} and holds fewer digest lines"
            ));
        };
        let Some(words) = line.strip_prefix("digest=") else {
            return Err(format!(
                "the capture holds `{line}` where `digest=` belongs"
            ));
        };
        digests.push(parse_digest(words)?);
    }
    if let Some(extra) = lines.next() {
        return Err(format!(
            "the capture holds a line after its `digests={digest_count}` lines: `{extra}`"
        ));
    }
    return Ok(Capture {
        nonce,
        repository,
        source,
        checkout_guid,
        index: CapturedIndex {
            size: index_size,
            mtime: index_mtime,
            digest,
        },
        head,
        source_head,
        entries,
        digests,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_record_tests.rs"]
mod tests;
