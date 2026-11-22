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
//!  and the GUID and nonce are read so a caller can compare them with its own
//!  request
//!  (`crate::clone_holder_scan::holder_scan_acceptance`).
//!  The bound comes first so a huge file is refused without being split into
//!  lines.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = parseHolderScan(text); // throws HolderScanRefusal
//! ```

use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
use crate::clone_holder_scan::holder_scan_text::{
    ABSENT_WORD, EXCLUDED_KEY, HOLDER_KEY, INCOMPLETE_KEY, MAXIMUM_NAMESPACE_BYTES, MAXIMUM_RESULT_BYTES, MAXIMUM_SOURCE_BYTES, OWN_MOUNT_KEY, RESULT_KIND, RESULT_KEYS, TIME_KEY, VERSION_KEY,
    is_a_result_time, unescape_path, unescape_text,
};
use crate::clone_holder_scan::incomplete_reads::{
    DETAILS_PAST_THE_BOUND_WORD, IncompleteRead, LISTING_WORD, NAMESPACES_PAST_THE_BOUND_WORD, NO_LIVE_MEMBER_WORD, TABLE_UNREADABLE_WORD, TASK_ENDED_WORD,
};
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// One text read line by line,
/// with the place the next line is expected at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = { lines, at: 0 }; const value = keyed(cursor, 'dataset');
/// ```
struct Cursor<'a> {
    /// The text's lines.
    lines: &'a [&'a str],
    /// How many lines have been read.
    at: usize,
}

impl<'a> Cursor<'a> {
    /// The value of the line whose key is `key`,
    /// refusing a line that is not there or begins another key.
    fn keyed(&mut self, key: &'static str) -> Result<&'a str, HolderScanRefusal> {
        let Some(line) = self.lines.get(self.at).copied() else {
            return Err(HolderScanRefusal::MissingLine { key });
        };
        self.at += 1;
        let Some((found, value)) = line.split_once(' ') else {
            return Err(HolderScanRefusal::Key { number: self.at, expected: key, found: String::from(line) });
        };
        if found != key {
            return Err(HolderScanRefusal::Key { number: self.at, expected: key, found: String::from(found) });
        }
        if value.is_empty() {
            return Err(HolderScanRefusal::Value { key, shown: String::from("<empty>") });
        }
        return Ok(value);
    }

    /// One canonical decimal count.
    fn count(&mut self, key: &'static str) -> Result<usize, HolderScanRefusal> {
        let value: &'a str = self.keyed(key)?;
        let count: usize = decimal_count(key, value)?;
        let remaining: usize = self.lines.len().saturating_sub(self.at);
        if count > remaining {
            return Err(HolderScanRefusal::Count { key, shown: String::from(value) });
        }
        return Ok(count);
    }

    /// Nothing may follow the last field.
    fn finish(self) -> Result<(), HolderScanRefusal> {
        let Some(line) = self.lines.get(self.at).copied() else {
            return Ok(());
        };
        return Err(HolderScanRefusal::TrailingLine { found: String::from(line) });
    }
}

/// One canonical decimal count,
/// which is one or more digits with no sign and no leading zero.
fn decimal_count(key: &'static str, value: &str) -> Result<usize, HolderScanRefusal> {
    let canonical: bool = !value.is_empty() && value.bytes().all(|byte| return byte.is_ascii_digit()) && !(value.len() > 1 && value.starts_with('0'));
    let Ok(count) = value.parse::<usize>() else {
        return Err(HolderScanRefusal::Count { key, shown: String::from(value) });
    };
    if !canonical {
        return Err(HolderScanRefusal::Count { key, shown: String::from(value) });
    }
    return Ok(count);
}

/// One mount id,
/// its root and its mount point,
/// as the last three fields of an own-mount line.
fn mount_after_the_key(key: &'static str, value: &str) -> Result<HeldMount, HolderScanRefusal> {
    let parts: Vec<&str> = value.splitn(3, ' ').collect();
    let [number, root, mount_point] = parts.as_slice() else {
        return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
    };
    let Ok(mount_id) = number.parse::<u64>() else {
        return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
    };
    return Ok(HeldMount { mount_id, root: unescape_path(root), mount_point: unescape_path(mount_point) });
}

/// What one holder line names:
/// its namespace,
/// its task and every mount of the clone that namespace holds.
fn holder_fields(key: &'static str, value: &str) -> Result<NamespaceHolder, HolderScanRefusal> {
    let parts: Vec<&str> = value.split(' ').collect();
    let [namespace, task, mounts @ ..] = parts.as_slice() else {
        return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
    };
    if mounts.is_empty() || mounts.len() % 3 != 0 {
        return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
    }
    let mut held: Vec<HeldMount> = Vec::with_capacity(mounts.len() / 3);
    for triple in mounts.chunks(3) {
        let [number, root, mount_point] = triple else {
            return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
        };
        let Ok(mount_id) = number.parse::<u64>() else {
            return Err(HolderScanRefusal::Value { key, shown: String::from(value) });
        };
        held.push(HeldMount { mount_id, root: unescape_path(root), mount_point: unescape_path(mount_point) });
    }
    return Ok(NamespaceHolder { namespace: unescape_text(namespace), task: unescape_path(task), mounts: held });
}

/// One section of holder lines,
/// one line per namespace.
fn holder_section(cursor: &mut Cursor<'_>, count_key: &'static str, line_key: &'static str) -> Result<Vec<NamespaceHolder>, HolderScanRefusal> {
    let count: usize = cursor.count(count_key)?;
    let mut holders: Vec<NamespaceHolder> = Vec::with_capacity(count);
    for _line in 0..count {
        holders.push(holder_fields(line_key, cursor.keyed(line_key)?)?);
    }
    return Ok(holders);
}

/// One section of the reader's own mounts.
fn own_mount_section(cursor: &mut Cursor<'_>) -> Result<Vec<HeldMount>, HolderScanRefusal> {
    let count: usize = cursor.count(RESULT_KEYS[6])?;
    let mut mounts: Vec<HeldMount> = Vec::with_capacity(count);
    for _line in 0..count {
        mounts.push(mount_after_the_key(OWN_MOUNT_KEY, cursor.keyed(OWN_MOUNT_KEY)?)?);
    }
    return Ok(mounts);
}

/// The refusal one incomplete read's line makes when it does not fit its
/// cause.
fn incomplete_refusal(value: &str) -> HolderScanRefusal {
    return HolderScanRefusal::Value { key: INCOMPLETE_KEY, shown: String::from(value) };
}

/// One incomplete read,
/// read by the cause its own word names.
fn incomplete_of(value: &str) -> Result<IncompleteRead, HolderScanRefusal> {
    let Some((kind, rest)) = value.split_once(' ') else {
        return Err(incomplete_refusal(value));
    };
    return match kind {
        LISTING_WORD => {
            let Some((proc_root, reason)) = rest.split_once(' ') else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::Listing { proc_root: unescape_path(proc_root), reason: String::from(reason) })
        }
        TASK_ENDED_WORD | TABLE_UNREADABLE_WORD => {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let [namespace, task, reason] = parts.as_slice() else {
                return Err(incomplete_refusal(value));
            };
            let (namespace_text, task_path, reason_text) = (unescape_text(namespace), unescape_path(task), String::from(*reason));
            if kind == TASK_ENDED_WORD {
                return Ok(IncompleteRead::TaskEnded { namespace: namespace_text, task: task_path, reason: reason_text });
            }
            return Ok(IncompleteRead::TableUnreadable { namespace: namespace_text, task: task_path, reason: reason_text });
        }
        NO_LIVE_MEMBER_WORD => {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let [namespace, tasks_tried, reason] = parts.as_slice() else {
                return Err(incomplete_refusal(value));
            };
            let Ok(tried) = tasks_tried.parse::<usize>() else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::NoLiveMember { namespace: unescape_text(namespace), tasks_tried: tried, reason: String::from(*reason) })
        }
        NAMESPACES_PAST_THE_BOUND_WORD => Ok(IncompleteRead::NamespacesPastTheBound { bound: decimal_count(INCOMPLETE_KEY, rest)? }),
        DETAILS_PAST_THE_BOUND_WORD => Ok(IncompleteRead::DetailsPastTheBound { bound: decimal_count(INCOMPLETE_KEY, rest)? }),
        _ => Err(incomplete_refusal(value)),
    };
}

/// One section of the reads the scan could not complete.
fn incomplete_section(cursor: &mut Cursor<'_>) -> Result<Vec<IncompleteRead>, HolderScanRefusal> {
    let count: usize = cursor.count(RESULT_KEYS[9])?;
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
        return Err(HolderScanRefusal::TooLarge { bytes: text.len(), bound: MAXIMUM_RESULT_BYTES });
    }
    let lines: Vec<&str> = text.lines().collect();
    let Some(version) = lines.first().copied() else {
        return Err(HolderScanRefusal::MissingLine { key: VERSION_KEY });
    };
    if version != VERSION_KEY {
        return Err(HolderScanRefusal::Version { found: String::from(version) });
    }
    let mut cursor: Cursor<'_> = Cursor { lines: lines.as_slice(), at: 1 };
    let kind: &str = cursor.keyed(RESULT_KEYS[0])?;
    if kind != RESULT_KIND {
        return Err(HolderScanRefusal::Kind { found: String::from(kind) });
    }
    let dataset: DatasetGuid = DatasetGuid::parse(cursor.keyed(RESULT_KEYS[1])?).map_err(HolderScanRefusal::Dataset)?;
    let nonce: RequestNonce = RequestNonce::parse(cursor.keyed(RESULT_KEYS[2])?).map_err(HolderScanRefusal::Nonce)?;
    let source: &str = cursor.keyed(RESULT_KEYS[3])?;
    if source.len() > MAXIMUM_SOURCE_BYTES {
        return Err(HolderScanRefusal::Value { key: RESULT_KEYS[3], shown: String::from(source) });
    }
    let own_namespace: &str = cursor.keyed(RESULT_KEYS[4])?;
    if own_namespace.len() > MAXIMUM_NAMESPACE_BYTES {
        return Err(HolderScanRefusal::Value { key: RESULT_KEYS[4], shown: String::from(own_namespace) });
    }
    let namespaces_scanned: usize = cursor.count(RESULT_KEYS[5])?;
    let own_mounts: Vec<HeldMount> = own_mount_section(&mut cursor)?;
    let holders: Vec<NamespaceHolder> = holder_section(&mut cursor, RESULT_KEYS[7], HOLDER_KEY)?;
    let excluded: Vec<NamespaceHolder> = holder_section(&mut cursor, RESULT_KEYS[8], EXCLUDED_KEY)?;
    let incomplete: Vec<IncompleteRead> = incomplete_section(&mut cursor)?;
    let time: &str = cursor.keyed(TIME_KEY)?;
    if !is_a_result_time(time) {
        return Err(HolderScanRefusal::Time { value: String::from(time) });
    }
    cursor.finish()?;
    return Ok(HolderScan {
        dataset,
        source: unescape_text(source),
        nonce,
        own_namespace: if own_namespace == ABSENT_WORD { None } else { Some(unescape_text(own_namespace)) },
        namespaces_scanned,
        own_mounts,
        holders,
        excluded,
        incomplete,
        time: String::from(time),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_parse_tests.rs"]
mod tests;
