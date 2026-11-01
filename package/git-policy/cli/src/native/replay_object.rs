//! What:
//!  Raw commit objects as exact bytes:
//!       parsing, the unsigned replay rewrite, and the inputs a signed replay passes to `git commit-tree`.
//! Why:
//!  Replay keeps every byte of the prepared commit except its `tree` and `parent` lines,
//!       so headers and message stay bytes;
//!       a non-UTF-8 `encoding` commit round-trips unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! parseRawCommit(bytes); rewriteCommitObject({ commit, treeOid, parentOid });
//! ```

/// Owned byte and text values.
use std::fmt;

/// What:
///  One header with its continuation lines.
/// Why:
///   The rewrite copies each header's bytes unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawCommitHeader = { name: string; bytes: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawCommitHeader {
    /// Header name, Latin-1 decoded, before the first space.
    pub name: String,
    /// Complete header bytes, continuation lines and final newline included.
    pub bytes: Vec<u8>,
}

/// What:
///  A parsed raw commit.
/// Why:
///   Replay reads the tree, the encoding, whether the object is signed,
///       and the identities; the message and headers are copied back as bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawCommit = { headers; message: Uint8Array; treeOid: string; encoding?: string; signed: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawCommit {
    /// Headers in object order.
    pub headers: Vec<RawCommitHeader>,
    /// Message bytes after the blank line.
    pub message: Vec<u8>,
    /// Tree header value.
    pub tree_oid: String,
    /// `encoding` header value, absent for UTF-8.
    pub encoding: Option<String>,
    /// Whether a `gpgsig` or `gpgsig-sha256` header is present.
    pub signed: bool,
}

/// What:
///  A raw commit that cannot be replayed:
///       no `tree` header, or an identity header that is missing or malformed.
/// Why:
///   Replay fails the transaction rather than guess at identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReplayObjectError extends TypeError {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayObjectError(pub String);

impl fmt::Display for ReplayObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// Headers a signed replay reproduces through `git commit-tree`; every other header is dropped there.
const REBUILT_HEADERS: [&str; 7] = [
    "tree",
    "parent",
    "author",
    "committer",
    "encoding",
    "gpgsig",
    "gpgsig-sha256",
];

/// What:
///  The text value of a single-line header,
///       without its name and trailing newline, decoded as UTF-8 with replacement.
/// Why:
///   The incumbent decodes header values with `toString('utf8')`,
///       which replaces invalid sequences rather than failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(header.bytes).toString('utf8', name.length + 1, bytes.length - 1)
/// ```
fn header_value(header: &RawCommitHeader) -> String {
    let start: usize = (header.name.len() + 1).min(header.bytes.len());
    let stop: usize = header.bytes.len().saturating_sub(1).max(start);
    return String::from_utf8_lossy(&header.bytes[start..stop]).into_owned();
}

/// What:
///  Splits the header section into headers,
///       joining each continuation line (one starting with a space) to the header it continues.
/// Why:
///   A signature spans many lines; each header must keep all of them to round-trip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitHeaders(headerBytes);
/// ```
fn split_headers(header_bytes: &[u8]) -> Vec<RawCommitHeader> {
    if header_bytes.is_empty() {
        return Vec::new();
    }
    // Every line start, the first byte included, except after a final newline.
    let mut line_starts: Vec<usize> = vec![0];
    for (index, byte) in header_bytes.iter().enumerate() {
        if *byte == b'\n' && index + 1 < header_bytes.len() {
            line_starts.push(index + 1);
        }
    }
    let header_starts: Vec<usize> = line_starts
        .into_iter()
        .filter(|start| { return header_bytes[*start] != b' '; })
        .collect();
    let mut headers: Vec<RawCommitHeader> = Vec::new();
    for (index, start) in header_starts.iter().enumerate() {
        let stop: usize = header_starts.get(index + 1).copied().unwrap_or(header_bytes.len());
        let bytes: Vec<u8> = header_bytes[*start..stop].to_vec();
        let name_end: usize = match bytes.iter().position(|byte| { return *byte == b' '; }) {
            Some(space) => space,
            None => bytes.len() - 1,
        };
        headers.push(RawCommitHeader {
            name: bytes[..name_end].iter().map(|byte| { return char::from(*byte); }).collect(),
            bytes,
        });
    }
    return headers;
}

/// What:
///  Parses a raw commit object from `git cat-file commit` output.
/// Why:
///   Replay reads the tree, encoding, and signature state, and keeps the rest as bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRawCommit(bytes); // throws TypeError without a tree header
/// ```
pub fn parse_raw_commit(bytes: &[u8]) -> Result<RawCommit, ReplayObjectError> {
    let blank: Option<usize> = bytes.windows(2).position(|pair| { return pair == b"\n\n"; });
    let (header_bytes, message): (&[u8], Vec<u8>) = match blank {
        Some(index) => (&bytes[..index + 1], bytes[index + 2..].to_vec()),
        None => (bytes, Vec::new()),
    };
    let headers: Vec<RawCommitHeader> = split_headers(header_bytes);
    let tree: &RawCommitHeader = match headers.iter().find(|header| { return header.name == "tree"; }) {
        Some(found) => found,
        None => return Err(ReplayObjectError(String::from("Commit object has no tree header."))),
    };
    let tree_oid: String = header_value(tree);
    let encoding: Option<String> = headers
        .iter()
        .find(|header| { return header.name == "encoding"; })
        .map(header_value);
    let signed: bool = headers
        .iter()
        .any(|header| { return header.name == "gpgsig" || header.name == "gpgsig-sha256"; });
    return Ok(RawCommit {
        headers,
        message,
        tree_oid,
        encoding,
        signed,
    });
}

/// What:
///  Rewrites an unsigned commit onto a new tree and parent, keeping every other byte.
/// Why:
///   The replayed object must differ from the prepared one only in `tree` and `parent`,
///       so the message, encoding, identities, and custom headers stay byte-identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rewriteCommitObject({ commit, treeOid, parentOid });
/// ```
pub fn rewrite_commit_object(commit: &RawCommit, tree_oid: &str, parent_oid: &str) -> Vec<u8> {
    // `mut` lets the output be assembled in header order.
    let mut output: Vec<u8> = Vec::new();
    for header in &commit.headers {
        match header.name.as_str() {
            "tree" => output.extend_from_slice(format!("tree {tree_oid}\nparent {parent_oid}\n").as_bytes()),
            "parent" => {}
            _ => output.extend_from_slice(&header.bytes),
        }
    }
    output.push(b'\n');
    output.extend_from_slice(&commit.message);
    return output;
}

/// What:
///  The names of headers a signed replay drops, each once in object order.
/// Why:
///   Replay reports them with `replay-headers-dropped`,
///       so the operator learns a hook or tool added headers that `git commit-tree` cannot reproduce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// droppedReplayHeaders(commit); // ['x-custom']
/// ```
pub fn dropped_replay_headers(commit: &RawCommit) -> Vec<String> {
    // `mut` lets the names be collected once each, in object order.
    let mut names: Vec<String> = Vec::new();
    for header in &commit.headers {
        if !REBUILT_HEADERS.contains(&header.name.as_str()) && !names.contains(&header.name) {
            names.push(header.name.clone());
        }
    }
    return names;
}

/// What:
///  The Git identity environment of one `author` or `committer` header, as `git commit-tree` reads it:
///       the name, the email, and the date as `@<seconds> <zone>`.
/// Why:
///   A signed replay reproduces the prepared commit's identities and dates through these variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityEnvironment({ commit, role: 'author' }); // GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL, GIT_AUTHOR_DATE
/// ```
pub fn identity_environment(
    commit: &RawCommit,
    role: &str,
) -> Result<Vec<(String, String)>, ReplayObjectError> {
    let header: &RawCommitHeader = match commit.headers.iter().find(|candidate| { return candidate.name == role; }) {
        Some(found) => found,
        None => {
            return Err(ReplayObjectError(format!("Commit object has no {role} header.")));
        }
    };
    let value: String = header_value(header);
    let malformed = || {
        return ReplayObjectError(format!(
            "Commit {role} header is malformed: {}",
            serde_json::to_string(&value).unwrap_or_default()
        ));
    };
    let open: usize = value.rfind('<').ok_or_else(malformed)?;
    let close: usize = open + value[open..].find('>').ok_or_else(malformed)?;
    let prefix: String = format!("GIT_{}", role.to_uppercase());
    return Ok(vec![
        (format!("{prefix}_NAME"), value[..open].trim_end().to_string()),
        (format!("{prefix}_EMAIL"), value[open + 1..close].to_string()),
        (format!("{prefix}_DATE"), format!("@{}", value[close + 1..].trim())),
    ]);
}

#[cfg(test)]
#[path = "replay_object_tests.rs"]
mod tests;
