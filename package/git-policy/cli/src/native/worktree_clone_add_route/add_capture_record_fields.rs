//! What:
//!  One field of the capture's grammar and the refusal that names it:
//! a
//!  hexadecimal path,
//! a decimal count,
//! a time,
//! the index's digest,
//! a commit's
//!  word,
//! one entry's line and one digest's line.
//! Why:
//!  A record whose values do not fit their types is damage or a later grammar,
//!  and neither is read under this one,
//!  so every field refuses in words a person acts on and the refusal names the
//!  line or the key
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What 'the captured source' is").
//!  `add_capture_record.rs` reads the whole record and would go over the
//!  file-size limit carrying the fields it reads,
//!  so the fields and their words live here and the record's order is that
//!  file's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entry = parseEntry(words); // throws with the field it refused
//! const digest = parseDigest(words); // throws with the field it refused
//! ```

use super::add_capture::{
    CapturedDigest,
    CapturedEntry,
    CapturedKind,
    MAXIMUM_PATH_BYTES,
};
use super::add_capture_record_bound::{
    WIDEST_COMMIT,
    WIDEST_NANOS,
};
use crate::worktree_clone_binding_record::binding_encoding::hexadecimal_as_bytes;

/// What:
///  Why a capture's bytes are not this grammar's,
/// in words a person reads.
/// Why:
///  The refusal names the line or field,
/// and every refusal leaves the record as
///  it stands,
/// which is the route's rule for a record it cannot read.
pub type CaptureParseRefusal = String;

/// A hexadecimal field's bytes,
/// refused when empty or wider than its limit.
pub(super) fn hexadecimal_bytes_named(
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
pub(super) fn decimal_named(
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

/// A time field,
/// `seconds.nanoseconds`.
pub(super) fn time_named(
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
pub(super) fn digest_named(value: &str) -> Result<[u8; 32], CaptureParseRefusal> {
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

/// A commit's word:
/// its name or `-`.
pub(super) fn commit_named(
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
pub(super) fn parse_entry(words: &str) -> Result<CapturedEntry, CaptureParseRefusal> {
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
pub(super) fn parse_digest(words: &str) -> Result<CapturedDigest, CaptureParseRefusal> {
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
