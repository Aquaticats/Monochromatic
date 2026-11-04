//! What:
//!  The immutable commit request record: a strict line-oriented text with hexadecimal fields, an end line carrying the
//!  SHA-256 of everything before it, and validation. Publication is the inbox's `accept` (`commit_inbox.rs`).
//! Why:
//!  A request is accepted when this record is durable (`doc/planning/cli-git-commits-never-fail.md`, "Durable
//!  acceptance"). Every free-form field (branch, message, path) is hex-encoded, so no character in it can break the
//!  line grammar; decoding is strict, so one request has one spelling and one hash.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Request = { version: 1; id: string; branch: string; base: string; message: string; changes: Change[] };
//! ```

use std::io;

use super::commit_integrity::{hex, is_identity, is_object_name, sha256_hex, unhex};

/// The record format version this module reads and writes. These components are unreleased and no request of version 1 or 2 was ever
/// accepted anywhere, so earlier versions are rejected rather than migrated (version 3 adds the mandatory `operation` line and the
/// amend parents); a later format change would need a reader of the older one.
pub const REQUEST_VERSION: u32 = 4;

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
    /// The repository-relative path as bytes, `/` separated, no empty, `.`, `..` or `.git` component (Git paths need not be UTF-8).
    pub path: Vec<u8>,
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
///  `enum Operation { ... }`: the kind of commit a request asks for. `Amend` names the commit it amends.
/// Why:
///  A request with no changes is an empty commit or a message-only amend, which need different parent and history treatment;
///  the record must say which before it is accepted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Operation = { kind: 'commit' } | { kind: 'empty-commit' } | { kind: 'amend'; target: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    /// An ordinary commit; it has at least one change.
    Commit,
    /// `git commit --allow-empty`: no change, a new commit on the base.
    EmptyCommit,
    /// An amendment of the commit with this object name (possibly message-only).
    Amend {
        /// The object name of the commit being amended.
        target: String,
        /// The target's parents, so the amendment can be rebuilt after the target is pruned (retention by spooling).
        parents: Vec<String>,
    },
}

/// What:
///  `enum Eligibility { ... }`: whether a request can be accepted by the lane or goes the way of real Git before acceptance.
/// Why:
///  An amendment cannot be rebuilt from this record once its target is pruned (its author, committer and tree are not here), and the
///  spool has no representation of a submodule commit; both are refused before acceptance and run natively, so the commit never fails
///  for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Eligibility = { kind: 'accept' } | { kind: 'native-fallback'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Eligibility {
    /// The lane can accept the request.
    Accept,
    /// The request is run by real Git, with the reason.
    NativeFallback(&'static str),
}

/// The eligibility of `request`, decided from its operation and modes alone (so it works on a request that validation would refuse).
pub fn eligibility(request: &Request) -> Eligibility {
    if matches!(request.operation, Operation::Amend { .. }) {
        return Eligibility::NativeFallback("an amendment is not recoverable from the record");
    }
    let gitlink = |mode: &str| -> bool { return mode == "160000"; };
    if request.changes.iter().any(|change| { return gitlink(&change.mode) || gitlink(&change.old_mode); }) {
        return Eligibility::NativeFallback("a submodule commit has no representation in the spool");
    }
    return Eligibility::Accept;
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
    /// The branch the request was captured on, as bytes.
    pub branch: Vec<u8>,
    /// The captured base commit (or the empty tree for an unborn branch), as an object name.
    pub base: String,
    /// The commit message, as bytes.
    pub message: Vec<u8>,
    /// What the request asks for, so an empty commit and a message-only amend differ.
    pub operation: Operation,
    /// A fresh 16-byte invocation nonce, generated once per logical request and retained across retries: two independent invocations with
    /// equal payloads get different identities, a resend of one invocation the same.
    pub nonce: Vec<u8>,
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

// What:     `impl From<A> for B` teaches the compiler to turn an `A` into a `B`, which is what `?` uses to convert an error on the way out.
// Why:      Let every file call below propagate its I/O error as this module's own error with `?`.
//
// In TS you'd write (pseudocode):
// ```ts
// class B { static from(a: A): B { ... } }
// ```
impl From<io::Error> for RequestError {
    /// Lets `?` turn an `io::Error` into `RequestError::Io`.
    fn from(error: io::Error) -> RequestError {
        return RequestError::Io(error);
    }
}

/// Whether `path` is a safe repository-relative path: non-empty, no NUL, no leading or trailing `/`, no empty, `.`,
/// `..` or `.git` component.
pub fn valid_path(path: &[u8]) -> bool {
    // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
    // Why:      Say in one place what to do for each element or error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (x) => expr
    // ```
    if path.is_empty() || path.contains(&0) || path.starts_with(b"/") || path.ends_with(b"/") {
        return false;
    }
    return path.split(|byte| { return *byte == b'/'; }).all(|part| { return !part.is_empty() && part != b"." && part != b".." && part != b".git"; });
}

/// Checks every field of `request`.
pub fn validate(request: &Request) -> Result<(), RequestError> {
    if !is_identity(&request.id) {
        // What:     `Err(e)` is the failure case of `Result` (TS: throwing `e`).
        // Why:      Report a failure to the caller as a value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw e;
        // ```
        return Err(RequestError::Invalid("id"));
    }
    if request.branch.is_empty() || request.branch.contains(&0) {
        // What:     `return Ok(x)` / `return Err(e)` ends the function with a success or a failure value.
        // Why:      Hand the result to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return x; // or throw
        // ```
        return Err(RequestError::Invalid("branch"));
    }
    if !is_object_name(&request.base) {
        return Err(RequestError::Invalid("base"));
    }
    // What:     `.iter()` walks a list by reference; `.iter_mut()` lends each entry for change; `.into_iter()` consumes the list. The chained method (`any`, `all`, `map`, `find`) takes a closure.
    // Why:      Ask a question of every entry in one expression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.some((x) => ...)
    // ```
    if request.message.iter().all(|byte| { return byte.is_ascii_whitespace(); }) || request.message.contains(&0) {
        return Err(RequestError::Invalid("message"));
    }
    // What:     `match value { pattern => ..., }` compares a value against patterns top to bottom and runs the arm that fits; the compiler checks that every case is covered.
    // Why:      Branch on the outcome of a call that can succeed or fail, or on which variant a value is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (value.kind) { case ...: ...; }
    // ```
    if request.nonce.len() != 16 {
        return Err(RequestError::Invalid("nonce"));
    }
    match &request.operation {
        Operation::Commit if request.changes.is_empty() => return Err(RequestError::Invalid("changes")),
        Operation::EmptyCommit if !request.changes.is_empty() => return Err(RequestError::Invalid("changes")),
        Operation::Amend { .. } => return Err(RequestError::Invalid("amend")),
        _ => {}
    }
    // What:     `None` is the absent case of `Option` (TS: `undefined`).
    // Why:      Say there is no value, or no failure, without a sentinel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    let mut previous: Option<&[u8]> = None;
    for change in &request.changes {
        if !valid_path(&change.path) {
            return Err(RequestError::Invalid("path"));
        }
        if previous.is_some_and(|before| { return before >= change.path.as_slice(); }) {
            return Err(RequestError::Invalid("path order"));
        }
        // What:     `Some(x)` is the present case of `Option` (TS: the value itself, as opposed to `undefined`); `None` is the absent case.
        // Why:      Return or store a value that may be missing without using a sentinel.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return x; // or undefined
        // ```
        previous = Some(change.path.as_slice());
        let deletion: bool = change.mode == "000000";
        // Gitlinks (mode 160000) are unsupported: the spool has no representation of a submodule commit, and the replay cannot decide
        // its ancestry; such a request is refused before acceptance and goes the way of Git itself.
        let mode_ok: bool = deletion || ["100644", "100755", "120000"].contains(&change.mode.as_str());
        if !mode_ok {
            return Err(RequestError::Invalid("mode"));
        }
        let hash_ok: bool = if deletion { change.sha256 == "-" } else { change.sha256.len() == 64 && unhex(&change.sha256).is_some() };
        if !hash_ok {
            return Err(RequestError::Invalid("sha256"));
        }
        let absent: bool = change.old_mode == "000000";
        let old_mode_ok: bool = absent || ["100644", "100755", "120000"].contains(&change.old_mode.as_str());
        let old_hash_ok: bool = if absent { change.old_sha256 == "-" } else { change.old_sha256.len() == 64 && unhex(&change.old_sha256).is_some() };
        if !old_mode_ok || !old_hash_ok {
            return Err(RequestError::Invalid("old entry"));
        }
    }
    // What:     `Ok(x)` is the success case of `Result` (TS: returning `x`); `Err(e)` is the failure case (TS: throwing `e`).
    // Why:      Return a value from a function that can also fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return x;
    // ```
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
    body.push_str(&format!("branch {}\nbase {}\n", hex(&request.branch), request.base));
    body.push_str(&format!("message {}\n", hex(&request.message)));
    match &request.operation {
        Operation::Commit => body.push_str("operation commit\n"),
        Operation::EmptyCommit => body.push_str("operation empty-commit\n"),
        Operation::Amend { target, parents } => {
            let listed: String = if parents.is_empty() { "-".to_string() } else { parents.join(",") };
            body.push_str(&format!("operation amend {} {}\n", target, listed));
        }
    }
    body.push_str(&format!("nonce {}\n", hex(&request.nonce)));
    for change in &request.changes {
        body.push_str(&format!("change {} {} {} {} {}\n", change.mode, change.sha256, change.old_mode, change.old_sha256, hex(&change.path)));
    }
    let end: String = format!("end {}\n", sha256_hex(body.as_bytes()));
    body.push_str(&end);
    return Ok(body.into_bytes());
}

/// The content hash of `request`: the SHA-256 of its body, as the end line carries it.
pub fn content_hash(request: &Request) -> Result<String, RequestError> {
    let bytes: Vec<u8> = encode(request)?;
    // What:     `.map_err(|e| ...)` converts the error of a `Result` into another error type and leaves a success alone; `|e| ...` is a closure (TS: `(e) => ...`).
    // Why:      Make a library error fit this module's own error type before propagating it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (e) { throw new OwnError(e); }
    // ```
    let text: String = String::from_utf8(bytes).map_err(|_| { return RequestError::Damaged("utf-8"); })?;
    // What:     `.ok_or(error)` turns an `Option` into a `Result`: the present value becomes `Ok(value)` and `None` becomes `Err(error)`.
    // Why:      Name the failure when a value that must exist is absent, so `?` can propagate it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value === undefined) throw error;
    // ```
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
    let operation_text: &str = field(lines.next(), "operation ", "operation")?.trim_end();
    let operation: Operation = match operation_text {
        "commit" => Operation::Commit,
        "empty-commit" => Operation::EmptyCommit,
        other => {
            let rest: &str = other.strip_prefix("amend ").ok_or(RequestError::Damaged("operation"))?;
            let (target, listed) = rest.split_once(' ').ok_or(RequestError::Damaged("operation"))?;
            let parents: Vec<String> = if listed == "-" { Vec::new() } else { listed.split(',').map(|parent| { return parent.to_string(); }).collect() };
            Operation::Amend { target: target.to_string(), parents }
        }
    };
    // What:     `Vec::new()` makes an empty growable list (TS: `[]`); the type on the left says what it will hold. Siblings: `[T; N]`, a fixed-size array, and `&[T]`, a borrowed view.
    // Why:      Collect results whose number is not known beforehand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: T[] = [];
    // ```
    let nonce: Vec<u8> = unhex(field(lines.next(), "nonce ", "nonce")?.trim_end()).ok_or(RequestError::Damaged("nonce"))?;
    let mut changes: Vec<Change> = Vec::new();
    let mut end: Option<String> = None;
    for line in lines {
        // What:     `if let Some(x) = value { ... }` runs the block only when the `Option` (or `Result`) holds a value, naming it `x` inside; otherwise it is skipped.
        // Why:      Handle the present case without a full `match`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const x = value; ... }
        // ```
        if let Some(rest) = line.strip_prefix("change ") {
            let parts: Vec<&str> = rest.trim_end().split(' ').collect();
            if parts.len() != 5 || end.is_some() {
                return Err(RequestError::Damaged("change"));
            }
            let path: Vec<u8> = unhex(parts[4]).ok_or(RequestError::Damaged("path"))?;
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
        branch: unhex(branch_hex).ok_or(RequestError::Damaged("branch"))?,
        base,
        message: unhex(message_hex).ok_or(RequestError::Damaged("message"))?,
        operation,
        nonce,
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
