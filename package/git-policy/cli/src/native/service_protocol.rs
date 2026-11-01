//! What:
//!  The messages between the wrapper and the resident commit service, one JSON object per line.
//! Why:
//!  A client sends a prepared change with a request identity and waits; the service answers with a landing, a queue
//!  position, a request to prepare again, or a failure from a closed list. The line form needs no framing library and
//!  every field is checked on read, so a damaged or hostile line is refused rather than guessed
//!  (`doc/planning/cli-git-resident-service.md`, "Protocol and request identity").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socket.write(JSON.stringify(encodeRequest(request)) + '\n'); const reply = decodeResponse(await readLine(socket));
//! ```

use serde_json::{Map, Value};
use super::service_review_key::ReviewKey;
use std::path::PathBuf;

/// The protocol version this build speaks; a service and a client agree when their versions are equal.
pub const PROTOCOL_VERSION: u32 = 4;

/// The longest line either side accepts, in bytes.
pub const MAX_LINE_BYTES: usize = 1 << 20;

/// The most paths one request may change.
pub const MAX_CHANGES: usize = 4096;

/// Modes a change may give a file.
pub const ALLOWED_MODES: [u32; 3] = [0o100_644, 0o100_755, 0o120_000];

/// What:
///  One path a commit changes: a file to set, or a file to delete.
/// Why:
///   The client has already written the file's content into the repository's object store, so the service names the
///  object and never reads the user's worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { path: string; mode: number; blob?: string }; // no blob means delete
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// Repository-relative path with `/` separators.
    pub path: String,
    /// File mode, one of `ALLOWED_MODES`.
    pub mode: u32,
    /// Object name of the content, or nothing for a deletion.
    pub blob: Option<String>,
}

/// What:
///  A request to land one commit.
/// Why:
///   Every field the service needs to land it on the current branch without asking the client anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitRequest = { id; repository; commonDir; reference; base; changes; message; author?; committer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitRequest {
    /// Random identity of this invocation, kept across reconnects.
    pub id: String,
    /// Worktree root whose index is updated.
    pub repository: PathBuf,
    /// Common git directory shared by the worktrees; the queue key.
    pub common_dir: PathBuf,
    /// Full name of the branch the commit goes on.
    pub reference: String,
    /// The commit the client prepared against.
    pub base: String,
    /// The paths to change.
    pub changes: Vec<Change>,
    /// The commit message, exact bytes.
    pub message: String,
    /// `Name <email> seconds zone`, or nothing to use the committer.
    pub author: Option<String>,
    /// `Name <email> seconds zone`.
    pub committer: String,
    /// Whether the commit replaces the branch tip: its parents are the base commit's parents, and it lands only while
    ///  the branch tip is still the base.
    pub amend: bool,
    /// The tree of an amend, which the client wrote into the repository's object store; nothing for any other request,
    ///  whose tree the service makes from the base and the changes.
    pub tree: Option<String>,
    /// A commit Git built in a scratch repository (an editor, a signature), already in the repository's object store. The
    ///  service lands it as it is, only while the branch tip is still the base, and ignores the message, the identities
    ///  and the tree of the request.
    pub commit: Option<String>,
}

impl CommitRequest {
    /// Whether the request lands alone and only while the branch tip equals its base: an amend, or a commit built elsewhere.
    pub fn lands_alone(&self) -> bool {
        return self.amend || self.commit.is_some();
    }
}

/// The closed list of reasons the service gives for not landing a request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureCode {
    /// The repository has no room.
    RepositoryFull,
    /// The repository cannot be written.
    RepositoryReadOnly,
    /// Another program holds the repository's index.
    IndexInUse,
    /// The request is not one the service accepts.
    Rejected,
    /// The branch moved too often while the request waited.
    BranchKeepsChanging,
    /// The branch is no longer where the commit was prepared, and the commit cannot be made again on another tip.
    BranchMoved,
    /// Anything else the service could not resolve.
    Unavailable,
}

impl FailureCode {
    /// The word on the wire.
    pub fn word(self) -> &'static str {
        return match self {
            Self::RepositoryFull => "repository-full",
            Self::RepositoryReadOnly => "repository-read-only",
            Self::IndexInUse => "index-in-use",
            Self::Rejected => "rejected",
            Self::BranchKeepsChanging => "branch-keeps-changing",
            Self::BranchMoved => "branch-moved",
            Self::Unavailable => "unavailable",
        };
    }

    /// Every code, so a test reaches all of them.
    pub const ALL: [Self; 7] = [
        Self::RepositoryFull,
        Self::RepositoryReadOnly,
        Self::IndexInUse,
        Self::Rejected,
        Self::BranchKeepsChanging,
        Self::BranchMoved,
        Self::Unavailable,
    ];

    /// The code for a word, nothing for an unknown word.
    pub fn from_word(word: &str) -> Option<Self> {
        return Self::ALL.into_iter().find(|code| { return code.word() == word; });
    }
}

/// What a client sends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientMessage {
    /// Land this commit.
    Commit(Box<CommitRequest>),
    /// Finish the queue and exit, so the manager starts the current binary.
    Retire,
    /// Work items were written for this repository; run what is pending.
    Work {
        /// The repository's common Git directory, which holds the work directory.
        common_dir: PathBuf,
        /// The gateway credential for the standards review of the commits this message announces. It lives only in this
        /// message and in the service's memory, never in a work item or on disk.
        review_key: Option<ReviewKey>,
    },
}

/// What the service sends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceMessage {
    /// First line of every connection: the service's protocol version.
    Welcome { version: u32 },
    /// The request is queued behind others.
    Position { number: usize, of: usize },
    /// The commit is on the branch; `already` when an earlier request with the same identity landed it.
    Landed { id: String, commit: String, already: bool },
    /// A named path changed on the branch since the base; prepare again from `base`.
    RetryPrepare { id: String, base: String },
    /// The request was not landed.
    Failed { id: String, code: FailureCode },
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    return value.get(name).and_then(Value::as_str).ok_or_else(|| { return format!("field {name} is missing or not text"); });
}

/// Whether `text` is a full object name in hexadecimal (40 or 64 digits).
pub fn is_object_name(text: &str) -> bool {
    return (text.len() == 40 || text.len() == 64) && text.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Whether `path` is a plain repository-relative path: no leading slash, no empty, `.` or `..` part, no NUL or newline.
pub fn is_plain_path(path: &str) -> bool {
    return !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(['\0', '\n'])
        && path.split('/').all(|part| { return !part.is_empty() && part != "." && part != ".." && part != ".git"; });
}

/// One JSON object, written on one line from its entries.
fn object(entries: Vec<(&str, Value)>) -> String {
    let mut map: Map<String, Value> = Map::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    return Value::Object(map).to_string();
}

/// What:
///  The line for a client message.
/// Why:
///   One object per line, no newline inside any field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(message)
/// ```
pub fn encode_client(message: &ClientMessage) -> String {
    return match message {
        ClientMessage::Retire => object(vec![("type", Value::from("retire")), ("version", Value::from(PROTOCOL_VERSION))]),
        ClientMessage::Work { common_dir, review_key } => {
            let mut entries: Vec<(&str, Value)> = vec![
                ("type", Value::from("work")),
                ("version", Value::from(PROTOCOL_VERSION)),
                ("commonDir", Value::from(common_dir.to_string_lossy().as_ref())),
            ];
            if let Some(key) = review_key {
                entries.push(("reviewKey", Value::from(key.expose())));
            }
            object(entries)
        }
        ClientMessage::Commit(request) => object(vec![
            ("type", Value::from("commit")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("id", Value::from(request.id.as_str())),
            ("repository", Value::from(request.repository.to_string_lossy().as_ref())),
            ("commonDir", Value::from(request.common_dir.to_string_lossy().as_ref())),
            ("reference", Value::from(request.reference.as_str())),
            ("base", Value::from(request.base.as_str())),
            (
                "changes",
                Value::Array(
                    request
                        .changes
                        .iter()
                        .map(|change| {
                            return Value::Object(Map::from_iter([
                                (String::from("path"), Value::from(change.path.as_str())),
                                (String::from("mode"), Value::from(change.mode)),
                                (String::from("blob"), change.blob.as_deref().map_or(Value::Null, Value::from)),
                            ]));
                        })
                        .collect(),
                ),
            ),
            ("message", Value::from(request.message.as_str())),
            ("author", request.author.as_deref().map_or(Value::Null, Value::from)),
            ("committer", Value::from(request.committer.as_str())),
            ("amend", Value::from(request.amend)),
            ("tree", request.tree.as_deref().map_or(Value::Null, Value::from)),
            ("commit", request.commit.as_deref().map_or(Value::Null, Value::from)),
        ]),
    };
}

/// What:
///  Reads and checks one client line.
/// Why:
///   The service trusts nothing from a socket: sizes, names, paths, modes and the object names are all checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeClient(line) // throws on anything that is not a well-formed message
/// ```
pub fn decode_client(line: &str) -> Result<ClientMessage, String> {
    if line.len() > MAX_LINE_BYTES {
        return Err(String::from("line is too long"));
    }
    let value: Value = serde_json::from_str(line).map_err(|error| { return error.to_string(); })?;
    if value.get("version").and_then(Value::as_u64) != Some(u64::from(PROTOCOL_VERSION)) {
        return Err(String::from("unsupported version"));
    }
    match field(&value, "type")? {
        "retire" => return Ok(ClientMessage::Retire),
        "work" => {
            let common_dir: PathBuf = PathBuf::from(field(&value, "commonDir")?);
            if !common_dir.is_absolute() || common_dir.to_string_lossy().contains('\0') {
                return Err(String::from("work request is not acceptable"));
            }
            let review_key: Option<ReviewKey> = match value.get("reviewKey") {
                None | Some(Value::Null) => None,
                Some(Value::String(text)) => ReviewKey::new(text.as_str()),
                Some(_) => return Err(String::from("work request is not acceptable")),
            };
            return Ok(ClientMessage::Work { common_dir, review_key });
        }
        "commit" => {}
        other => return Err(format!("unknown message type {other}")),
    }
    let mut changes: Vec<Change> = Vec::new();
    for entry in value.get("changes").and_then(Value::as_array).ok_or("changes is missing")? {
        let path: &str = field(entry, "path")?;
        let mode: u32 = u32::try_from(entry.get("mode").and_then(Value::as_u64).ok_or("mode is missing")?)
            .map_err(|_| { return String::from("mode is out of range"); })?;
        let blob: Option<String> = entry.get("blob").and_then(Value::as_str).map(str::to_string);
        if !is_plain_path(path) || !ALLOWED_MODES.contains(&mode) || blob.as_deref().is_some_and(|name| { return !is_object_name(name); }) {
            return Err(format!("change for {path:?} is not acceptable"));
        }
        changes.push(Change { path: path.to_string(), mode, blob });
    }
    if changes.len() > MAX_CHANGES {
        return Err(String::from("number of changes is not acceptable"));
    }
    let request = CommitRequest {
        id: field(&value, "id")?.to_string(),
        repository: PathBuf::from(field(&value, "repository")?),
        common_dir: PathBuf::from(field(&value, "commonDir")?),
        reference: field(&value, "reference")?.to_string(),
        base: field(&value, "base")?.to_string(),
        changes,
        message: field(&value, "message")?.to_string(),
        author: value.get("author").and_then(Value::as_str).map(str::to_string),
        committer: field(&value, "committer")?.to_string(),
        amend: value.get("amend").and_then(Value::as_bool).unwrap_or(false),
        tree: value.get("tree").and_then(Value::as_str).map(str::to_string),
        commit: value.get("commit").and_then(Value::as_str).map(str::to_string),
    };
    let well_formed: bool = !request.id.is_empty()
        && request.id.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || byte == b'-'; })
        && request.reference.starts_with("refs/")
        && !request.reference.contains(['\0', '\n', ' '])
        && is_object_name(&request.base)
        && request.repository.is_absolute()
        && request.common_dir.is_absolute()
        && !request.committer.contains(['\0', '\n'])
        && request.author.as_deref().is_none_or(|author| { return !author.contains(['\0', '\n']); })
        // An amend the service builds names the tree it makes; a commit built elsewhere is named whole; any other request
        //  names neither, so the three cannot be mixed up.
        && (request.amend && request.commit.is_none()) == request.tree.is_some()
        && request.tree.as_deref().is_none_or(is_object_name)
        && request.commit.as_deref().is_none_or(is_object_name);
    if !well_formed {
        return Err(String::from("request is not acceptable"));
    }
    return Ok(ClientMessage::Commit(Box::new(request)));
}

/// What:
///  The line for a service message.
/// Why:
///   The same one-object form in the other direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(message)
/// ```
pub fn encode_service(message: &ServiceMessage) -> String {
    return match message {
        ServiceMessage::Welcome { version } => object(vec![("type", Value::from("welcome")), ("version", Value::from(*version))]),
        ServiceMessage::Position { number, of } => {
            object(vec![("type", Value::from("position")), ("number", Value::from(*number)), ("of", Value::from(*of))])
        }
        ServiceMessage::Landed { id, commit, already } => object(vec![
            ("type", Value::from("landed")),
            ("id", Value::from(id.as_str())),
            ("commit", Value::from(commit.as_str())),
            ("already", Value::from(*already)),
        ]),
        ServiceMessage::RetryPrepare { id, base } => {
            object(vec![("type", Value::from("retry-prepare")), ("id", Value::from(id.as_str())), ("base", Value::from(base.as_str()))])
        }
        ServiceMessage::Failed { id, code } => {
            object(vec![("type", Value::from("failed")), ("id", Value::from(id.as_str())), ("code", Value::from(code.word()))])
        }
    };
}

/// What:
///  Reads one service line.
/// Why:
///   The client checks what it is told as strictly as the service checks what it is sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeService(line)
/// ```
pub fn decode_service(line: &str) -> Result<ServiceMessage, String> {
    if line.len() > MAX_LINE_BYTES {
        return Err(String::from("line is too long"));
    }
    let value: Value = serde_json::from_str(line).map_err(|error| { return error.to_string(); })?;
    let number = |name: &str| -> Result<usize, String> {
        return value.get(name).and_then(Value::as_u64).and_then(|n| { return usize::try_from(n).ok(); })
            .ok_or_else(|| { return format!("field {name} is missing"); });
    };
    return match field(&value, "type")? {
        "welcome" => Ok(ServiceMessage::Welcome {
            version: u32::try_from(number("version")?).map_err(|_| { return String::from("version is out of range"); })?,
        }),
        "position" => Ok(ServiceMessage::Position { number: number("number")?, of: number("of")? }),
        "landed" => Ok(ServiceMessage::Landed {
            id: field(&value, "id")?.to_string(),
            commit: field(&value, "commit")?.to_string(),
            already: value.get("already").and_then(Value::as_bool).unwrap_or(false),
        }),
        "retry-prepare" => Ok(ServiceMessage::RetryPrepare { id: field(&value, "id")?.to_string(), base: field(&value, "base")?.to_string() }),
        "failed" => Ok(ServiceMessage::Failed {
            id: field(&value, "id")?.to_string(),
            code: FailureCode::from_word(field(&value, "code")?).ok_or("unknown failure code")?,
        }),
        other => Err(format!("unknown message type {other}")),
    };
}

/// Protocol controls stay out of the release executable.
#[cfg(test)]
#[path = "service_protocol_tests.rs"]
mod tests;
