//! What:
//!  The capture's own record grammar: its header, its fixed lines, one line per
//!  tracked path, and a reader that refuses every shape it did not write.
//! Why:
//!  A resume after a kill proves the clone against the capture only if the
//!  capture outlives the process, so it is a durable record of its own beside
//!  the binding (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What 'the captured source' is").
//!  It is keyed by the add's nonce as well as its slot, so a capture another add
//!  left is never read as this one's.
//!  Its bound is the index's own entry count times one line's widest form plus
//!  the fixed lines, and no separate total is shipped, exactly as the design
//!  says.
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
    CapturedKind,
    MAXIMUM_PATH_BYTES,
    digest_words,
    entry_words,
};
use crate::worktree_clone_binding_record::binding_encoding::{
    bytes_as_hexadecimal,
    hexadecimal_as_bytes,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The first line of a capture, naming the grammar's version.
/// Version 2 carries the C3 and C4 digests a dirty source needs; a version 1
/// record holds none, so it is refused rather than read as one that does.
pub const CAPTURE_HEADER: &str =
    "# monochromatic-git-policy-cli zfs worktree add capture, version 2";

/// The keys the fixed part writes, in the order it writes them.
pub const CAPTURE_FIXED_KEYS: [&str; 11] = [
    "nonce",
    "repository",
    "source",
    "checkoutGuid",
    "indexSize",
    "indexMtime",
    "indexDigest",
    "head",
    "sourceHead",
    "entries",
    "digests",
];

/// The widest decimal form of a 64-bit unsigned number.
const WIDEST_DECIMAL: usize = 20;

/// The widest nanosecond count, nine digits.
const WIDEST_NANOS: usize = 10;

/// The widest commit name, 64 hexadecimal digits for a SHA-256 repository.
const WIDEST_COMMIT: usize = 64;

/// One captured entry's widest line:
/// hex path, six fields, two times, the executable bit and a link's hex target.
pub const CAPTURE_ENTRY_LINE_MAXIMUM_BYTES: u64 = (2 * MAXIMUM_PATH_BYTES) as u64
    + 1
    + 7
    + 1
    + WIDEST_DECIMAL as u64
    + 1
    + (WIDEST_DECIMAL + 1 + WIDEST_NANOS) as u64 * 2
    + 1
    + 1
    + 1
    + (2 * MAXIMUM_PATH_BYTES) as u64
    + 1;

/// One digest's widest line: the hex path, the SHA-256 and their space.
pub const CAPTURE_DIGEST_LINE_MAXIMUM_BYTES: u64 = (2 * MAXIMUM_PATH_BYTES) as u64 + 1 + 64 + 1;

/// The capture's fixed lines' widest total.
pub const CAPTURE_FIXED_BYTES: u64 = CAPTURE_HEADER.len() as u64
    + 1
    + ("nonce=".len() + 32 + 1) as u64
    + ("repository=".len() + 2 * MAXIMUM_PATH_BYTES + 1) as u64
    + ("source=".len() + 2 * MAXIMUM_PATH_BYTES + 1) as u64
    + ("checkoutGuid=".len() + WIDEST_DECIMAL + 1) as u64
    + ("indexSize=".len() + WIDEST_DECIMAL + 1) as u64
    + ("indexMtime=".len() + WIDEST_DECIMAL + 1 + WIDEST_NANOS + 1) as u64
    + ("indexDigest=".len() + 64 + 1) as u64
    + ("head=".len() + WIDEST_COMMIT + 1) as u64
    + ("sourceHead=".len() + WIDEST_COMMIT + 1) as u64
    + ("entries=".len() + WIDEST_DECIMAL + 1) as u64
    + ("digests=".len() + WIDEST_DECIMAL + 1) as u64;

/// The largest capture one entry count and its digests can name.
pub fn capture_bound(entries: usize) -> u64 {
    return CAPTURE_FIXED_BYTES
        + entries as u64 * CAPTURE_ENTRY_LINE_MAXIMUM_BYTES
        + entries as u64 * CAPTURE_DIGEST_LINE_MAXIMUM_BYTES;
}

/// What:
///  A capture's bytes, in the grammar's order.
/// Why:
///  The writer puts every fixed key on its line and one line per entry, so the
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

/// One commit's word: its hexadecimal name, or `-` when the branch is unborn.
fn commit_word(commit: &str) -> &str {
    return if commit.is_empty() { "-" } else { commit };
}

/// What:
///  Why a capture's bytes are not this grammar's, in words a person reads.
/// Why:
///  The refusal names the line or field, and every refusal leaves the record as
///  it stands, which is the route's rule for a record it cannot read.
pub type CaptureParseRefusal = String;

/// What:
///  Read one capture's bytes.
/// Why:
///  A record whose count disagrees with its lines, whose keys arrive out of
///  order or twice, or whose values do not fit their types is damage or a later
///  grammar, and neither is read under this one.
///
/// # Errors
/// The header, a key, a value or the count does not fit the grammar.
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

/// A hexadecimal field's bytes, refused when empty or wider than its limit.
fn hexadecimal_bytes_named(
    value: &str,
    key: &str,
    widest: usize,
) -> Result<Vec<u8>, CaptureParseRefusal> {
    let Some(bytes) = hexadecimal_as_bytes(value) else {
        return Err(format!(
            "the capture's `{key}=` is not lowercase hexadecimal: `{value}`"
        ));
    };
    if bytes.len() > widest {
        return Err(format!(
            "the capture's `{key}=` names {} bytes, and a path is at most {widest}",
            bytes.len()
        ));
    }
    return Ok(bytes);
}

/// A decimal field.
fn decimal_named(
    value: &str,
    key: &str,
) -> Result<u64, CaptureParseRefusal> {
    if value.is_empty() || !value.bytes().all(|byte| return byte.is_ascii_digit()) {
        return Err(format!("the capture's `{key}=` is not decimal: `{value}`"));
    }
    return value.parse().map_err(|_overflow| {
        return format!("the capture's `{key}=` does not fit 64 bits: `{value}`");
    });
}

/// A time field, `seconds.nanoseconds`.
fn time_named(
    value: &str,
    key: &str,
) -> Result<(i64, u32), CaptureParseRefusal> {
    let Some((seconds, nanos)) = value.split_once('.') else {
        return Err(format!(
            "the capture's `{key}=` is not `<seconds>.<nanoseconds>`: `{value}`"
        ));
    };
    let parsed_seconds: i64 = seconds.parse().map_err(|_refused| {
        return format!("the capture's `{key}=` seconds do not fit: `{value}`");
    })?;
    if nanos.is_empty()
        || nanos.len() > WIDEST_NANOS
        || !nanos.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return Err(format!(
            "the capture's `{key}=` nanoseconds are not up to nine digits: `{value}`"
        ));
    }
    let parsed_nanos: u32 = nanos.parse().map_err(|_refused| {
        return format!("the capture's `{key}=` nanoseconds do not fit: `{value}`");
    })?;
    return Ok((parsed_seconds, parsed_nanos));
}

/// The index's SHA-256 digest.
fn digest_named(value: &str) -> Result<[u8; 32], CaptureParseRefusal> {
    let Some(bytes) = hexadecimal_as_bytes(value) else {
        return Err(format!(
            "the capture's `indexDigest=` is not lowercase hexadecimal: `{value}`"
        ));
    };
    let Ok(digest) = <[u8; 32]>::try_from(bytes.as_slice()) else {
        return Err(String::from("the capture's `indexDigest=` is not 32 bytes"));
    };
    return Ok(digest);
}

/// A commit's word: its name or `-`.
fn commit_named(
    value: &str,
    key: &str,
) -> Result<String, CaptureParseRefusal> {
    if value == "-" {
        return Ok(String::new());
    }
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "the capture's `{key}=` is not a lowercase hexadecimal commit: `{value}`"
        ));
    }
    if value.len() != 40 && value.len() != WIDEST_COMMIT {
        return Err(format!(
            "the capture's `{key}=` is {} digits, and a commit name is 40 or 64",
            value.len()
        ));
    }
    return Ok(String::from(value));
}

/// One entry's line.
fn parse_entry(words: &str) -> Result<CapturedEntry, CaptureParseRefusal> {
    let fields: Vec<&str> = words.split(' ').collect();
    if fields.len() != 7 {
        return Err(format!(
            "the capture's entry `{words}` does not hold its seven fields"
        ));
    }
    let path: Vec<u8> = hexadecimal_bytes_named(fields[0], "entry path", MAXIMUM_PATH_BYTES)?;
    let Some(kind) = CapturedKind::parse(fields[1]) else {
        return Err(format!(
            "the capture's entry kind `{}` is not `regular`, `link` or `absent`",
            fields[1]
        ));
    };
    let size: u64 = decimal_named(fields[2], "entry size")?;
    let mtime: (i64, u32) = time_named(fields[3], "entry mtime")?;
    let ctime: (i64, u32) = time_named(fields[4], "entry ctime")?;
    let executable: bool = match fields[5] {
        "0" => false,
        "1" => true,
        other => {
            return Err(format!(
                "the capture's entry execute bit `{other}` is neither 0 nor 1"
            ));
        }
    };
    let link_target: Option<Vec<u8>> = match (kind, fields[6]) {
        (CapturedKind::Link, "-") => {
            return Err(format!(
                "the capture's link `{}` names no target",
                fields[0]
            ));
        }
        (CapturedKind::Link, target) => Some(hexadecimal_as_bytes(target).ok_or_else(|| {
            return format!("the capture's link target `{target}` is not lowercase hexadecimal");
        })?),
        (_other, "-") => None,
        (other, target) => {
            return Err(format!(
                "the capture's {other:?} entry names a link target `{target}`"
            ));
        }
    };
    return Ok(CapturedEntry {
        path,
        kind,
        size,
        mtime,
        ctime,
        executable,
        link_target,
    });
}

/// One digest's line.
fn parse_digest(words: &str) -> Result<CapturedDigest, CaptureParseRefusal> {
    let fields: Vec<&str> = words.split(' ').collect();
    if fields.len() != 2 {
        return Err(format!(
            "the capture's digest `{words}` does not hold its two fields"
        ));
    }
    let path: Vec<u8> = hexadecimal_bytes_named(fields[0], "digest path", MAXIMUM_PATH_BYTES)?;
    let Some(bytes) = hexadecimal_as_bytes(fields[1]) else {
        return Err(format!(
            "the capture's digest `{}` is not lowercase hexadecimal",
            fields[1]
        ));
    };
    let Ok(digest) = <[u8; 32]>::try_from(bytes.as_slice()) else {
        return Err(format!(
            "the capture's digest `{}` is not 32 bytes",
            fields[1]
        ));
    };
    return Ok(CapturedDigest {
        path,
        digest,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_capture_record_tests.rs"]
mod tests;
