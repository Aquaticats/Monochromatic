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
use crate::service_review_key::ReviewKey;
use std::path::{Path, PathBuf};

/// The protocol version this build speaks; a service and a client agree when their versions are equal.
pub const PROTOCOL_VERSION: u32 = 7;

/// The release of this build: the crate version, which the wrapper and the service program share because they are built
///  and installed together. A service from another release is retired even when it speaks the same protocol, since the
///  program its unit names, and the behavior behind the same messages, may differ.
pub const RELEASE: &str = env!("CARGO_PKG_VERSION");

/// How often the service tells each waiting client that it is alive, while the client's request is queued or landing.
///  One second is the pace of the other waiting notices (`service_client::START_NOTICE_GRACE`), so a client that waits sees
///  proof of life as often as it sees any other line. Measured on 2026-11-03 on this host: 300 timers of one second fired a
///  median of 1.0 ms and at most 2.2 ms late (`doc/planning/cli-git-shipped-limits-audit.md`, "Waits on the commit service
///  itself"), so a beat is never missed by the clock alone.
pub const HEARTBEAT_EVERY: std::time::Duration = std::time::Duration::from_secs(1);

/// How long a client hears nothing from the service, heartbeats included, before it takes the service for stopped.
///  Five beats: the measured lateness of a beat is a few milliseconds, so five missing beats are a stopped or frozen
///  process and not a late timer, and a wrong call costs one resend of the same request, which the service recognizes by
///  its identity and answers without landing it twice.
pub const HEARTBEAT_SILENCE: std::time::Duration = std::time::Duration::from_secs(5);

/// The 99th percentile of the lengths of the paths changed by the 21,696 non-merge commits of this repository since 2026-09-01
///  (176,042 changed paths read with `git log --no-merges --format=@%H --name-only --all`): 118 characters. The median was 58, the
///  90th percentile 86, the 99.9th percentile 159 and the longest 189.
pub const P99_CHANGED_PATH_CHARACTERS: usize = 118;

/// The longest line either side accepts, in bytes: one mebibyte, the size at which a request of `MAX_CHANGES` changes of the
///  99th-percentile path length (`P99_CHANGED_PATH_CHARACTERS`) still fits (a control in `service_protocol_tests.rs` encodes
///  one). A request with 4,096 paths of the longest length seen (189) would need about
///  1.06 MB and is refused as a decode failure. The coordinator's pick under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Sizes and ages of the service's own housekeeping").
pub const MAX_LINE_BYTES: usize = 1 << 20;

/// The most paths one request may change: 4,096. Of the 21,696 non-merge commits of this repository since 2026-09-01 (median 2
///  changed paths, 99th percentile 38, 99.9th percentile 377) 10 changed more than 4,096 paths (0.05 percent; the largest 8,178),
///  and those commits take the native path instead of the service. The coordinator's pick under the owner's delegation, open to
///  the owner's veto.
pub const MAX_CHANGES: usize = 4096;

/// The longest request identity either side accepts, in bytes (a random identity is 36 bytes). A display and input cap.
pub const MAX_ID_BYTES: usize = 128;

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
///  What the landing of a merge, cherry-pick or revert conclusion removes from the worktree's Git directory once the
///  commit is on the branch.
/// Why:
///   Native `git commit` removes `MERGE_HEAD` and its siblings when it concludes; the client built the commit with real
///  Git in a scratch repository that holds the state it was given, so the entries real Git removed there are the ones the
///  landing removes here, each only while it still holds the bytes copied at preparation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cleanup = { gitDir: string; scratch: string; reftable: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cleanup {
    /// The Git directory of the worktree the commit is made in.
    pub git_dir: PathBuf,
    /// The commit's scratch directory, which holds the conclusion copies and the scratch repository Git committed in.
    pub scratch: PathBuf,
    /// Whether the repository stores references in reftable tables, which hold three of the entries in the ref store.
    pub reftable: bool,
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
    /// Common git directory shared by the worktrees.
    pub common_dir: PathBuf,
    /// The worktree's own Git directory (`.git` of the main worktree, `.git/worktrees/<id>` of a linked one): the key of the
    ///  service instance that lands the commit, which refuses a request that names another worktree's.
    pub git_dir: PathBuf,
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
    /// The state files a conclusion removes once the commit has landed; only with a commit built elsewhere.
    pub cleanup: Option<Cleanup>,
    /// The real index of the worktree, whose lock the landing holds from before the branch moves until the new index is in place.
    pub index: PathBuf,
    /// When the client finished capturing the worktree's bytes: nanoseconds of `CLOCK_MONOTONIC`, which one clock gives every
    ///  process of the host, so two captures of one worktree compare whichever process made them; zero when unknown. The
    ///  service keeps it with the commit it lands, and a retry uses it to tell which of two overlapping captures is the later.
    pub captured: u64,
}

/// A commit the service landed and when its bytes were captured, as the service tells a client whose commit it sends back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedCapture {
    /// The landed commit.
    pub commit: String,
    /// The `captured` stamp of the request that made it.
    pub captured: u64,
}

/// Whether `name` is the all-zero object name, which stands for a branch that does not exist yet.
pub fn is_zero_object(name: &str) -> bool {
    return is_object_name(name) && name.bytes().all(|byte| { return byte == b'0'; });
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
    /// A Git step of the landing ran longer than the wrapper lets one step run and was ended; the batch was withdrawn.
    StepTooSlow,
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
            Self::StepTooSlow => "step-too-slow",
            Self::Unavailable => "unavailable",
        };
    }

    /// Every code, so a test reaches all of them.
    pub const ALL: [Self; 8] = [
        Self::RepositoryFull,
        Self::RepositoryReadOnly,
        Self::IndexInUse,
        Self::Rejected,
        Self::BranchKeepsChanging,
        Self::BranchMoved,
        Self::StepTooSlow,
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
    /// The sender stopped waiting for the commit of this identity (its ceiling passed): the service answers `Landed` when the
    ///  commit is on the branch, and otherwise withdraws it so it can never land, and answers `Withdrawn`.
    Withdraw {
        /// The identity of the request, as the client sent it.
        id: String,
    },
    /// The sender keeps this scratch directory of a commit in preparation for as long as the connection stays open; the
    ///  service removes the directory when the connection closes, whatever closed it.
    Scratch {
        /// The worktree's own Git directory, which holds the directory and names the instance that removes it.
        git_dir: PathBuf,
        /// The scratch directory (`cli-git-service-<id>` in a Git directory of that repository).
        path: PathBuf,
    },
    /// Work items were written for this repository; run what is pending.
    Work {
        /// The worktree's own Git directory, which holds the work directory.
        git_dir: PathBuf,
        /// The gateway credential for the standards review of the commits this message announces. It lives only in this
        /// message and in the service's memory, never in a work item or on disk.
        review_key: Option<ReviewKey>,
    },
}

/// What the service sends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceMessage {
    /// First line of every connection: the service's protocol version and the release it was built from. Services of
    ///  releases before the field existed send none, which reads as a release that is not this one.
    Welcome { version: u32, release: Option<String> },
    /// The request is queued behind others.
    Position { number: usize, of: usize },
    /// The commit is on the branch; `already` when an earlier request with the same identity landed it.
    Landed { id: String, commit: String, already: bool },
    /// A named path changed on the branch since the base; prepare again from `base`.
    RetryPrepare { id: String, base: String, landed: Vec<LandedCapture> },
    /// The request was not landed.
    Failed { id: String, code: FailureCode },
    /// The landing waits for another program that holds the repository's index: a line for the person's terminal.
    Waiting { text: String },
    /// The request of this identity was withdrawn at the client's asking and will never land.
    Withdrawn { id: String },
    /// Who held the index when the wait ran out, in words that fit "... is using this repository's index".
    Holder { who: String },
    /// The service is alive and the request is still waiting for its answer: sent on a fixed beat while a request is queued
    ///  or landing, so the client can wait for the answer with no clock of its own and end the wait on silence alone.
    Heartbeat,
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

/// The name prefix of a commit's scratch directory.
pub const SCRATCH_PREFIX: &str = "cli-git-service-";

/// What:
///  Whether `path` is the scratch directory of a commit in the worktree whose own Git directory is `git_dir`.
/// Why:
///   The service removes whatever a client names when the client's connection closes, so the name must be one only the
///  wrapper makes (the prefix and an identity of letters, digits and dashes) and the place must be directly inside the
///  worktree's own Git directory, with no relative part that could lead elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isScratchPath(path, gitDir)
/// ```
pub fn is_scratch_path(path: &Path, git_dir: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| { return name.to_str(); }) else { return false };
    let identity: &str = name.strip_prefix(SCRATCH_PREFIX).unwrap_or("");
    return git_dir.is_absolute()
        && path.parent() == Some(git_dir)
        && !identity.is_empty()
        && identity.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || byte == b'-'; })
        && git_dir.components().all(|part| { return !matches!(part, std::path::Component::ParentDir | std::path::Component::CurDir); })
        && !path.to_string_lossy().contains('\0');
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
///  The line that asks a service to retire, written in the protocol version that service speaks.
/// Why:
///   A service reads a line only in its own version, except that services built with this check read a retire at any
///  version. A service of an earlier release (the ones that sent no release in their greeting) refuses a retire written in
///  a newer version as unsupported and stays, so the line must carry the version the service announced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ type: 'retire', version: announcedVersion })
/// ```
pub fn encode_retire(version: u32) -> String {
    return object(vec![("type", Value::from("retire")), ("version", Value::from(version))]);
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
        ClientMessage::Retire => encode_retire(PROTOCOL_VERSION),
        ClientMessage::Withdraw { id } => object(vec![
            ("type", Value::from("withdraw")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("id", Value::from(id.as_str())),
        ]),
        ClientMessage::Scratch { git_dir, path } => object(vec![
            ("type", Value::from("scratch")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("gitDir", Value::from(git_dir.to_string_lossy().as_ref())),
            ("path", Value::from(path.to_string_lossy().as_ref())),
        ]),
        ClientMessage::Work { git_dir, review_key } => {
            let mut entries: Vec<(&str, Value)> = vec![
                ("type", Value::from("work")),
                ("version", Value::from(PROTOCOL_VERSION)),
                ("gitDir", Value::from(git_dir.to_string_lossy().as_ref())),
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
            ("gitDir", Value::from(request.git_dir.to_string_lossy().as_ref())),
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
            ("index", Value::from(request.index.to_string_lossy().as_ref())),
            ("captured", Value::from(request.captured)),
            (
                "cleanup",
                request.cleanup.as_ref().map_or(Value::Null, |cleanup| {
                    return Value::Object(Map::from_iter([
                        (String::from("gitDir"), Value::from(cleanup.git_dir.to_string_lossy().as_ref())),
                        (String::from("scratch"), Value::from(cleanup.scratch.to_string_lossy().as_ref())),
                        (String::from("reftable"), Value::from(cleanup.reftable)),
                    ]));
                }),
            ),
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
    // A request to retire carries no data a version could change the meaning of, and it is the one message a client of
    //  another protocol version must still be able to send, so that service can be replaced.
    if field(&value, "type")? == "retire" {
        return Ok(ClientMessage::Retire);
    }
    if value.get("version").and_then(Value::as_u64) != Some(u64::from(PROTOCOL_VERSION)) {
        return Err(String::from("unsupported version"));
    }
    match field(&value, "type")? {
        "retire" => return Ok(ClientMessage::Retire),
        "withdraw" => {
            let id: &str = field(&value, "id")?;
            if id.is_empty() || id.len() > MAX_ID_BYTES {
                return Err(String::from("withdraw request is not acceptable"));
            }
            return Ok(ClientMessage::Withdraw { id: id.to_string() });
        }
        "scratch" => {
            let git_dir: PathBuf = PathBuf::from(field(&value, "gitDir")?);
            let path: PathBuf = PathBuf::from(field(&value, "path")?);
            if !is_scratch_path(path.as_path(), git_dir.as_path()) {
                return Err(String::from("scratch directory is not acceptable"));
            }
            return Ok(ClientMessage::Scratch { git_dir, path });
        }
        "work" => {
            let git_dir: PathBuf = PathBuf::from(field(&value, "gitDir")?);
            if !git_dir.is_absolute() || git_dir.to_string_lossy().contains('\0') {
                return Err(String::from("work request is not acceptable"));
            }
            let review_key: Option<ReviewKey> = match value.get("reviewKey") {
                None | Some(Value::Null) => None,
                Some(Value::String(text)) => ReviewKey::new(text.as_str()),
                Some(_) => return Err(String::from("work request is not acceptable")),
            };
            return Ok(ClientMessage::Work { git_dir, review_key });
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
        git_dir: PathBuf::from(field(&value, "gitDir")?),
        reference: field(&value, "reference")?.to_string(),
        base: field(&value, "base")?.to_string(),
        changes,
        message: field(&value, "message")?.to_string(),
        author: value.get("author").and_then(Value::as_str).map(str::to_string),
        committer: field(&value, "committer")?.to_string(),
        amend: value.get("amend").and_then(Value::as_bool).unwrap_or(false),
        tree: value.get("tree").and_then(Value::as_str).map(str::to_string),
        commit: value.get("commit").and_then(Value::as_str).map(str::to_string),
        index: PathBuf::from(field(&value, "index")?),
        captured: value.get("captured").and_then(Value::as_u64).unwrap_or(0),
        cleanup: match value.get("cleanup").filter(|entry| { return !entry.is_null(); }) {
            None => None,
            Some(entry) => Some(Cleanup {
                git_dir: PathBuf::from(field(entry, "gitDir")?),
                scratch: PathBuf::from(field(entry, "scratch")?),
                reftable: entry.get("reftable").and_then(Value::as_bool).unwrap_or(false),
            }),
        },
    };
    let well_formed: bool = !request.id.is_empty()
        && request.id.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || byte == b'-'; })
        // A detached head is moved as `HEAD` itself.
        && (request.reference.starts_with("refs/") || request.reference == "HEAD")
        && !request.reference.contains(['\0', '\n', ' '])
        && is_object_name(&request.base)
        // A branch that does not exist yet has no commit to build the tree from, so only a commit built elsewhere names it.
        && (!is_zero_object(&request.base) || request.commit.is_some())
        && request.repository.is_absolute()
        && request.common_dir.is_absolute()
        && request.git_dir.is_absolute()
        && !request.git_dir.to_string_lossy().contains('\0')
        && request.index.is_absolute()
        && !request.index.to_string_lossy().contains('\0')
        && !request.committer.contains(['\0', '\n'])
        && request.author.as_deref().is_none_or(|author| { return !author.contains(['\0', '\n']); })
        // An amend the service builds names the tree it makes; a commit built elsewhere is named whole; any other request
        //  names neither, so the three cannot be mixed up.
        && (request.amend && request.commit.is_none()) == request.tree.is_some()
        && request.tree.as_deref().is_none_or(is_object_name)
        && request.commit.as_deref().is_none_or(is_object_name)
        && request.cleanup.as_ref().is_none_or(|cleanup| {
            return request.commit.is_some()
                && cleanup.git_dir.is_absolute()
                && cleanup.scratch.is_absolute()
                && cleanup.scratch.starts_with(&cleanup.git_dir)
                && !cleanup.git_dir.to_string_lossy().contains('\0')
                && !cleanup.scratch.to_string_lossy().contains('\0');
        });
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
        ServiceMessage::Welcome { version, release } => {
            let mut entries: Vec<(&str, Value)> = vec![("type", Value::from("welcome")), ("version", Value::from(*version))];
            if let Some(text) = release {
                entries.push(("release", Value::from(text.as_str())));
            }
            object(entries)
        }
        ServiceMessage::Position { number, of } => {
            object(vec![("type", Value::from("position")), ("number", Value::from(*number)), ("of", Value::from(*of))])
        }
        ServiceMessage::Landed { id, commit, already } => object(vec![
            ("type", Value::from("landed")),
            ("id", Value::from(id.as_str())),
            ("commit", Value::from(commit.as_str())),
            ("already", Value::from(*already)),
        ]),
        ServiceMessage::RetryPrepare { id, base, landed } => object(vec![
            ("type", Value::from("retry-prepare")),
            ("id", Value::from(id.as_str())),
            ("base", Value::from(base.as_str())),
            (
                "landed",
                Value::Array(
                    landed
                        .iter()
                        .map(|entry| {
                            return Value::Object(Map::from_iter([
                                (String::from("commit"), Value::from(entry.commit.as_str())),
                                (String::from("captured"), Value::from(entry.captured)),
                            ]));
                        })
                        .collect(),
                ),
            ),
        ]),
        ServiceMessage::Waiting { text } => object(vec![("type", Value::from("waiting")), ("text", Value::from(text.as_str()))]),
        ServiceMessage::Holder { who } => object(vec![("type", Value::from("holder")), ("who", Value::from(who.as_str()))]),
        ServiceMessage::Heartbeat => object(vec![("type", Value::from("heartbeat"))]),
        ServiceMessage::Withdrawn { id } => object(vec![("type", Value::from("withdrawn")), ("id", Value::from(id.as_str()))]),
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
            release: value.get("release").and_then(Value::as_str).map(String::from),
        }),
        "position" => Ok(ServiceMessage::Position { number: number("number")?, of: number("of")? }),
        "landed" => Ok(ServiceMessage::Landed {
            id: field(&value, "id")?.to_string(),
            commit: field(&value, "commit")?.to_string(),
            already: value.get("already").and_then(Value::as_bool).unwrap_or(false),
        }),
        "retry-prepare" => Ok(ServiceMessage::RetryPrepare {
            id: field(&value, "id")?.to_string(),
            base: field(&value, "base")?.to_string(),
            // A service that sends no list tells nothing about the order of captures.
            landed: value
                .get("landed")
                .and_then(Value::as_array)
                .map(|entries| {
                    return entries
                        .iter()
                        .filter_map(|entry| {
                            return Some(LandedCapture {
                                commit: entry.get("commit").and_then(Value::as_str).filter(|name| { return is_object_name(name); })?.to_string(),
                                captured: entry.get("captured").and_then(Value::as_u64)?,
                            });
                        })
                        .collect();
                })
                .unwrap_or_default(),
        }),
        "waiting" => Ok(ServiceMessage::Waiting { text: field(&value, "text")?.to_string() }),
        "holder" => Ok(ServiceMessage::Holder { who: field(&value, "who")?.to_string() }),
        "heartbeat" => Ok(ServiceMessage::Heartbeat),
        "withdrawn" => Ok(ServiceMessage::Withdrawn { id: field(&value, "id")?.to_string() }),
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
