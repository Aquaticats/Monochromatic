//! What:
//!  The immutable commit request record: a strict line-oriented text with hexadecimal fields, an end line carrying the
//!  SHA-256 of everything before it, validation, and a no-replace publication into `inbox/`.
//! Why:
//!  A request is accepted when this record is durable (`doc/planning/cli-git-commits-never-fail.md`, "Durable
//!  acceptance"). Every free-form field (branch, message, path) is hex-encoded, so no character in it can break the
//!  line grammar; decoding is strict, so one request has one spelling and one hash.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Request = { version: 1; id: string; branch: string; base: string; message: string; changes: Change[] };
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{create_dir_all_durable, sync_directory, temporary_name};
use super::commit_integrity::{hex, is_identity, is_object_name, sha256_hex, unhex};

/// The record format version this module reads and writes.
pub const REQUEST_VERSION: u32 = 2;

/// What:
///  `struct Change { ... }`: one path of a request: its mode (`000000` for a deletion), and the SHA-256 of its captured
///  bytes in the spool (`-` for a deletion).
/// Why:
///  The record names what the spool must hold, so a spool can be checked against the record that promised it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { path: string; mode: string; sha256: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// The repository-relative path, `/` separated, no empty, `.` or `..` component.
    pub path: String,
    /// The mode: `100644`, `100755`, `120000`, `160000`, or `000000` for a deletion.
    pub mode: String,
    /// The SHA-256 of the captured bytes, or `-` for a deletion.
    pub sha256: String,
    /// The mode of the path at the captured base (`000000` when the base has no such path).
    pub old_mode: String,
    /// The SHA-256 of the path's bytes at the captured base, or `-` when the base has no such path. The spool holds those bytes,
    /// so a replay needs no object of the base.
    pub old_sha256: String,
}

/// What:
///  `struct Request { ... }`: the whole immutable request.
/// Why:
///  Everything the lane needs to replay a commit from the spool: identity, branch, captured base, message, changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { id: string; branch: string; base: string; message: string; changes: Change[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// The request identity.
    pub id: String,
    /// The branch the request was captured on.
    pub branch: String,
    /// The captured base commit (or the empty tree for an unborn branch), as an object name.
    pub base: String,
    /// The commit message.
    pub message: String,
    /// The changes, in path order.
    pub changes: Vec<Change>,
}

/// What:
///  `enum RequestError { ... }`: why a request cannot be encoded, decoded or published.
/// Why:
///  A damaged or foreign record is reported with the line that failed, and is never repaired in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RequestError extends Error { kind: 'invalid' | 'damaged' | 'conflict' | 'io'; }
/// ```
#[derive(Debug)]
pub enum RequestError {
    /// A field value is not valid (the name is given).
    Invalid(&'static str),
    /// The bytes are not a record this module wrote (or its end hash fails).
    Damaged(&'static str),
    /// The identity already has a record with other content.
    Conflict,
    /// A file system call failed.
    Io(io::Error),
}

impl From<io::Error> for RequestError {
    /// Lets `?` turn an `io::Error` into `RequestError::Io`.
    fn from(error: io::Error) -> RequestError {
        return RequestError::Io(error);
    }
}

/// Whether `path` is a safe repository-relative path: non-empty, no NUL, no leading or trailing `/`, no empty, `.`,
/// `..` or `.git` component.
pub fn valid_path(path: &str) -> bool {
    if path.is_empty() || path.contains('\0') || path.starts_with('/') || path.ends_with('/') {
        return false;
    }
    return path.split('/').all(|part| { return !part.is_empty() && part != "." && part != ".." && part != ".git"; });
}

/// Checks every field of `request`.
pub fn validate(request: &Request) -> Result<(), RequestError> {
    if !is_identity(&request.id) {
        return Err(RequestError::Invalid("id"));
    }
    if request.branch.is_empty() || request.branch.contains('\0') {
        return Err(RequestError::Invalid("branch"));
    }
    if !is_object_name(&request.base) {
        return Err(RequestError::Invalid("base"));
    }
    if request.message.trim().is_empty() || request.message.contains('\0') {
        return Err(RequestError::Invalid("message"));
    }
    let mut previous: Option<&str> = None;
    for change in &request.changes {
        if !valid_path(&change.path) {
            return Err(RequestError::Invalid("path"));
        }
        if previous.is_some_and(|before| { return before >= change.path.as_str(); }) {
            return Err(RequestError::Invalid("path order"));
        }
        previous = Some(change.path.as_str());
        let deletion: bool = change.mode == "000000";
        let mode_ok: bool = deletion || ["100644", "100755", "120000", "160000"].contains(&change.mode.as_str());
        if !mode_ok {
            return Err(RequestError::Invalid("mode"));
        }
        let hash_ok: bool = if deletion { change.sha256 == "-" } else { change.sha256.len() == 64 && unhex(&change.sha256).is_some() };
        if !hash_ok {
            return Err(RequestError::Invalid("sha256"));
        }
        let absent: bool = change.old_mode == "000000";
        let old_mode_ok: bool = absent || ["100644", "100755", "120000", "160000"].contains(&change.old_mode.as_str());
        let old_hash_ok: bool = if absent { change.old_sha256 == "-" } else { change.old_sha256.len() == 64 && unhex(&change.old_sha256).is_some() };
        if !old_mode_ok || !old_hash_ok {
            return Err(RequestError::Invalid("old entry"));
        }
    }
    return Ok(());
}

/// What:
///  The bytes of `request`: the body lines, then `end <sha256 of the body>`.
/// Why:
///  The end line makes the record self-checking; the hash of the body is the request's content hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encode(request: Request): Uint8Array;
/// ```
pub fn encode(request: &Request) -> Result<Vec<u8>, RequestError> {
    validate(request)?;
    let mut body: String = format!("cli-git-request {}\nid {}\n", REQUEST_VERSION, request.id);
    body.push_str(&format!("branch {}\nbase {}\n", hex(request.branch.as_bytes()), request.base));
    body.push_str(&format!("message {}\n", hex(request.message.as_bytes())));
    for change in &request.changes {
        body.push_str(&format!("change {} {} {} {} {}\n", change.mode, change.sha256, change.old_mode, change.old_sha256, hex(change.path.as_bytes())));
    }
    let end: String = format!("end {}\n", sha256_hex(body.as_bytes()));
    body.push_str(&end);
    return Ok(body.into_bytes());
}

/// The content hash of `request`: the SHA-256 of its body, as the end line carries it.
pub fn content_hash(request: &Request) -> Result<String, RequestError> {
    let bytes: Vec<u8> = encode(request)?;
    let text: String = String::from_utf8(bytes).map_err(|_| { return RequestError::Damaged("utf-8"); })?;
    let end_at: usize = text.rfind("end ").ok_or(RequestError::Damaged("end"))?;
    return Ok(sha256_hex(&text.as_bytes()[..end_at]));
}

/// The text after `prefix` on `line`, or a damaged-record error naming `what`.
fn field<'a>(line: Option<&'a str>, prefix: &str, what: &'static str) -> Result<&'a str, RequestError> {
    let text: &str = line.ok_or(RequestError::Damaged(what))?;
    return text.strip_prefix(prefix).ok_or(RequestError::Damaged(what));
}

/// What:
///  The request `bytes` spell. Strict: exact line order, lowercase hexadecimal, the end hash must match, nothing may
///  follow the end line, and the decoded request must re-encode to exactly these bytes.
/// Why:
///  One spelling per request: a record cannot be altered in a way that keeps its hash and changes its meaning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decode(bytes: Uint8Array): Request;
/// ```
pub fn decode(bytes: &[u8]) -> Result<Request, RequestError> {
    let text: &str = std::str::from_utf8(bytes).map_err(|_| { return RequestError::Damaged("utf-8"); })?;
    let mut lines = text.split_inclusive('\n');
    let header: &str = field(lines.next(), "cli-git-request ", "header")?;
    if header != format!("{}\n", REQUEST_VERSION) {
        return Err(RequestError::Damaged("version"));
    }
    let id: String = field(lines.next(), "id ", "id")?.trim_end().to_string();
    let branch_hex: &str = field(lines.next(), "branch ", "branch")?.trim_end();
    let base: String = field(lines.next(), "base ", "base")?.trim_end().to_string();
    let message_hex: &str = field(lines.next(), "message ", "message")?.trim_end();
    let mut changes: Vec<Change> = Vec::new();
    let mut end: Option<String> = None;
    for line in lines {
        if let Some(rest) = line.strip_prefix("change ") {
            let parts: Vec<&str> = rest.trim_end().split(' ').collect();
            if parts.len() != 5 || end.is_some() {
                return Err(RequestError::Damaged("change"));
            }
            let path: String = String::from_utf8(unhex(parts[4]).ok_or(RequestError::Damaged("path"))?)
                .map_err(|_| { return RequestError::Damaged("path"); })?;
            changes.push(Change {
                path,
                mode: parts[0].to_string(),
                sha256: parts[1].to_string(),
                old_mode: parts[2].to_string(),
                old_sha256: parts[3].to_string(),
            });
        } else if let Some(rest) = line.strip_prefix("end ") {
            if end.is_some() {
                return Err(RequestError::Damaged("end"));
            }
            end = Some(rest.trim_end().to_string());
        } else {
            return Err(RequestError::Damaged("line"));
        }
    }
    let recorded: String = end.ok_or(RequestError::Damaged("end"))?;
    let request: Request = Request {
        id,
        branch: String::from_utf8(unhex(branch_hex).ok_or(RequestError::Damaged("branch"))?).map_err(|_| { return RequestError::Damaged("branch"); })?,
        base,
        message: String::from_utf8(unhex(message_hex).ok_or(RequestError::Damaged("message"))?).map_err(|_| { return RequestError::Damaged("message"); })?,
        changes,
    };
    let again: Vec<u8> = encode(&request).map_err(|_| { return RequestError::Damaged("fields"); })?;
    if again != bytes {
        return Err(RequestError::Damaged("spelling"));
    }
    if content_hash(&request)? != recorded {
        return Err(RequestError::Damaged("end hash"));
    }
    return Ok(request);
}

/// What:
///  The result of publishing: this call created `inbox/<id>`, or an identical record was already there.
/// Why:
///  A resend of the same request is not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Published = 'created' | 'already-present';
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Published {
    /// This call linked the record into place.
    Created,
    /// A record with the same bytes was already there.
    AlreadyPresent,
}

/// What:
///  Publishes `request` as `inbox/<id>`: a synced temporary file linked into place without replacing, the inbox
///  directory synced. If the name exists, the existing record must have identical bytes, else `Conflict`.
/// Why:
///  A rename over `inbox/<id>` would replace an acknowledged request; a link cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function publish(inbox: string, request: Request): Published;
/// ```
pub fn publish(inbox: &Path, request: &Request) -> Result<Published, RequestError> {
    let bytes: Vec<u8> = encode(request)?;
    create_dir_all_durable(inbox)?;
    let target: PathBuf = inbox.join(&request.id);
    let temporary: PathBuf = temporary_name(inbox)?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    let linked: io::Result<()> = fs::hard_link(&temporary, &target);
    fs::remove_file(&temporary)?;
    match linked {
        Ok(()) => {
            sync_directory(inbox)?;
            return Ok(Published::Created);
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // A resend finds the name linked by a writer that may have died before syncing the directory.
            sync_directory(inbox)?;
            if let Some(parent) = inbox.parent() {
                sync_directory(parent)?;
            }
            let existing: Vec<u8> = fs::read(&target)?;
            if existing == bytes {
                return Ok(Published::AlreadyPresent);
            }
            return Err(RequestError::Conflict);
        }
        Err(error) => return Err(RequestError::Io(error)),
    }
}
