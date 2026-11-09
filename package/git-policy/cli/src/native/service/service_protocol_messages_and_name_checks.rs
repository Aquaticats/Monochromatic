//! What:
//!  The messages between the wrapper and the resident commit service as typed values,
//! and the checks on the names they carry:
//!  object names,
//! plain paths,
//! identities and scratch directories.
//! Why:
//!  The encoder and the decoder of the one-JSON-object-per-line protocol (`doc/planning/cli-git-resident-service.md`,
//!  "Protocol and request identity") both speak in these types and use these checks,
//! so they sit in a file that uses neither.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // protocol/types.ts: export type CommitRequest, ClientMessage, ServiceMessage; export function isObjectName(), isPlainPath()
//! ```

use serde_json::Value;
use crate::path_bytes as path_bytes;
use crate::service_review_key::ReviewKey;
use crate::common_directory_of_a_git_directory::common_directory_of;
use crate::scratch_location::REPOSITORY_SCRATCH_NAME;
use crate::wrapper_names_old_and_new::SERVICE_SCRATCH_PREFIX;
use std::path::{Path, PathBuf};

/// The protocol version this build speaks;
/// a service and a client agree when their versions are equal.
/// It is 9 because a commit request names its scratch tree in its own field,
/// and a service of version 8 removes only the tree a request's cleanup names,
/// which the tree of every commit that concludes nothing is not:
/// the service would land the request and leave the tree to the age sweep.
/// It was 8 because a commit request's changed paths carry their bytes beside the readable spelling,
/// and a service of version 7 ignores that field and would commit the spelling instead:
/// the release text cannot keep such a service from being served,
/// because a rebuild of the same crate version names the same release,
/// so the version a service announces in its welcome is what is compared,
/// and a service of another version is asked to retire before any request is sent
/// (`service_client_reach_service.rs`).
pub const PROTOCOL_VERSION: u32 = 9;

/// The release of this build:
/// the crate version,
/// which the wrapper and the service program share because they are built
///  and installed together.
/// A service from another release is retired even when it speaks the same protocol,
/// since the
///  program its unit names,
/// and the behavior behind the same messages,
/// may differ.
pub const RELEASE: &str = env!("CARGO_PKG_VERSION");

/// How often the service tells each waiting client that it is alive,
/// while the client's request is queued or landing.
///  One second is the pace of the other waiting notices (`service_client::START_NOTICE_GRACE`),
/// so a client that waits sees
///  proof of life as often as it sees any other line.
/// Measured on 2026-11-03 on this host:
/// 300 timers of one second fired a
///  median of 1.0 ms and at most 2.2 ms late (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Waits on the commit service
///  itself"),
/// so a beat is never missed by the clock alone.
pub const HEARTBEAT_EVERY: std::time::Duration = std::time::Duration::from_secs(1);

/// How long a client hears nothing from the service,
/// heartbeats included,
/// before it takes the service for stopped.
///  Five beats:
/// the measured lateness of a beat is a few milliseconds,
/// so five missing beats are a stopped or frozen
///  process and not a late timer,
/// and a wrong call costs one resend of the same request,
/// which the service recognizes by
///  its identity and answers without landing it twice.
pub const HEARTBEAT_SILENCE: std::time::Duration = std::time::Duration::from_secs(5);

/// The 99th percentile of the lengths of the paths changed by the 21,696 non-merge commits of this repository since 2026-09-01
///  (176,042 changed paths read with `git log --no-merges --format=@%H --name-only --all`):
/// 118 characters.
/// The median was 58,
/// the
///  90th percentile 86,
/// the 99.9th percentile 159 and the longest 189.
pub const P99_CHANGED_PATH_CHARACTERS: usize = 118;

/// The longest line either side accepts,
/// in bytes:
/// one mebibyte,
/// the size at which a request of `MAX_CHANGES` changes of the
///  99th-percentile path length (`P99_CHANGED_PATH_CHARACTERS`) still fits (a control in `service_protocol_tests.rs` encodes
///  one).
/// A request with 4,096 paths of the longest length seen (189) would need about
///  1.06 MB and is refused as a decode failure.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const MAX_LINE_BYTES: usize = 1 << 20;

/// The most paths one request may change:
/// 4,096.
/// Of the 21,696 non-merge commits of this repository since 2026-09-01 (median 2
///  changed paths,
/// 99th percentile 38,
/// 99.9th percentile 377) 10 changed more than 4,096 paths (0.05 percent;
/// the largest 8,178),
///  and those commits take the native path instead of the service.
/// The coordinator's pick under the owner's delegation,
/// open to
///  the owner's veto.
pub const MAX_CHANGES: usize = 4096;

/// The longest request identity either side accepts,
/// in bytes (a random identity is 36 bytes).
/// A display and input cap.
pub const MAX_ID_BYTES: usize = 128;

/// Modes a change may give a file.
pub const ALLOWED_MODES: [u32; 3] = [0o100_644, 0o100_755, 0o120_000];

/// What:
///  One path a commit changes:
/// a file to set,
/// or a file to delete.
/// Why:
///   The client has already written the file's content into the repository's object store,
/// so the service names the
///  object and never reads the user's worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { path: Uint8Array; mode: number; blob?: string }; // no blob means delete
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// Repository-relative path with `/` separators,
    /// as the bytes Git listed:
    /// a file name that is not text is still a file name.
    pub path: PathBuf,
    /// File mode,
    /// one of `ALLOWED_MODES`.
    pub mode: u32,
    /// Object name of the content,
    /// or nothing for a deletion.
    pub blob: Option<String>,
}

/// What:
///  What the landing of a merge,
/// cherry-pick or revert conclusion removes from the worktree's Git directory once the
///  commit is on the branch.
/// Why:
///   Native `git commit` removes `MERGE_HEAD` and its siblings when it concludes;
/// the client built the commit with real
///  Git in a scratch repository that holds the state it was given,
/// so the entries real Git removed there are the ones the
///  landing removes here,
/// each only while it still holds the bytes copied at preparation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cleanup = { gitDir: string; scratch: string; reftable: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cleanup {
    /// The Git directory of the worktree the commit is made in.
    pub git_dir: PathBuf,
    /// The commit's scratch directory,
    /// which holds the conclusion copies and the scratch repository Git committed in.
    pub scratch: PathBuf,
    /// Whether the repository stores references in reftable tables,
    /// which hold three of the entries in the ref store.
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
    /// Random identity of this invocation,
    /// kept across reconnects.
    pub id: String,
    /// Worktree root whose index is updated.
    pub repository: PathBuf,
    /// Common git directory shared by the worktrees.
    pub common_dir: PathBuf,
    /// The worktree's own Git directory (`.git` of the main worktree,
    /// `.git/worktrees/<id>` of a linked one):
    /// the key of the
    ///  service instance that lands the commit,
    /// which refuses a request that names another worktree's.
    pub git_dir: PathBuf,
    /// Full name of the branch the commit goes on.
    pub reference: String,
    /// The commit the client prepared against.
    pub base: String,
    /// The paths to change.
    pub changes: Vec<Change>,
    /// The commit message,
    /// exact bytes.
    pub message: String,
    /// `Name <email> seconds zone`,
    /// or nothing to use the committer.
    pub author: Option<String>,
    /// `Name <email> seconds zone`.
    pub committer: String,
    /// Whether the commit replaces the branch tip:
    /// its parents are the base commit's parents,
    /// and it lands only while
    ///  the branch tip is still the base.
    pub amend: bool,
    /// The tree of an amend,
    /// which the client wrote into the repository's object store;
    /// nothing for any other request,
    ///  whose tree the service makes from the base and the changes.
    pub tree: Option<String>,
    /// A commit Git built in a scratch repository (an editor,
    /// a signature),
    /// already in the repository's object store.
    /// The
    ///  service lands it as it is,
    /// only while the branch tip is still the base,
    /// and ignores the message,
    /// the identities
    ///  and the tree of the request.
    pub commit: Option<String>,
    /// The state files a conclusion removes once the commit has landed;
    /// only with a commit built elsewhere.
    pub cleanup: Option<Cleanup>,
    /// The real index of the worktree,
    /// whose lock the landing holds from before the branch moves until the new index is in place.
    pub index: PathBuf,
    /// The commit's scratch tree,
    /// which holds the private index and,
    /// for a commit built elsewhere,
    /// the conclusion copies the landing reads.
    /// The service removes it once the landing of this request is over,
    /// so the request says where it is instead of the service working it out from a path.
    pub scratch: PathBuf,
    /// When the client finished capturing the worktree's bytes:
    /// nanoseconds of `CLOCK_MONOTONIC`,
    /// which one clock gives every
    ///  process of the host,
    /// so two captures of one worktree compare whichever process made them;
    /// zero when unknown.
    /// The
    ///  service keeps it with the commit it lands,
    /// and a retry uses it to tell which of two overlapping captures is the later.
    pub captured: u64,
}

/// A commit the service landed and when its bytes were captured,
/// as the service tells a client whose commit it sends back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedCapture {
    /// The landed commit.
    pub commit: String,
    /// The `captured` stamp of the request that made it.
    pub captured: u64,
}

/// Whether `name` is the all-zero object name,
/// which stands for a branch that does not exist yet.
pub fn is_zero_object(name: &str) -> bool {
    return is_object_name(name) && name.trim_start_matches('0').is_empty();
}

impl CommitRequest {
    /// Whether the request lands alone and only while the branch tip equals its base:
    /// an amend,
    /// or a commit built elsewhere.
    pub fn lands_alone(&self) -> bool {
        return self.amend || self.commit.is_some();
    }
}

/// The closed list of reasons the service gives for not landing a request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureCode {
    /// The repository has no room.
    RepositoryFull,
    /// The repository's filesystem is read-only (the operating system's `EROFS`).
    RepositoryReadOnly,
    /// This account may not write the repository (the operating system's `EACCES`).
    PermissionDenied,
    /// Another program holds the repository's index.
    IndexInUse,
    /// The request is not one the service accepts.
    Rejected,
    /// The branch moved too often while the request waited.
    BranchKeepsChanging,
    /// The branch is no longer where the commit was prepared,
    /// and the commit cannot be made again on another tip.
    BranchMoved,
    /// A Git step of the landing ran longer than the wrapper lets one step run and was ended;
    /// the batch was withdrawn.
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
            Self::PermissionDenied => "repository-permission-denied",
            Self::IndexInUse => "index-in-use",
            Self::Rejected => "rejected",
            Self::BranchKeepsChanging => "branch-keeps-changing",
            Self::BranchMoved => "branch-moved",
            Self::StepTooSlow => "step-too-slow",
            Self::Unavailable => "unavailable",
        };
    }

    /// Every code,
    /// so a test reaches all of them.
    pub const ALL: [Self; 9] = [
        Self::RepositoryFull,
        Self::RepositoryReadOnly,
        Self::PermissionDenied,
        Self::IndexInUse,
        Self::Rejected,
        Self::BranchKeepsChanging,
        Self::BranchMoved,
        Self::StepTooSlow,
        Self::Unavailable,
    ];

    /// The code for a word,
    /// nothing for an unknown word.
    pub fn from_word(word: &str) -> Option<Self> {
        for code in Self::ALL {
            if code.word() == word {
                return Some(code);
            }
        }
        return None;
    }
}

/// What a client sends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientMessage {
    /// Land this commit.
    Commit(Box<CommitRequest>),
    /// Finish the queue and exit,
    /// so the manager starts the current binary.
    Retire,
    /// The sender stopped waiting for the commit of this identity (its ceiling passed):
    /// the service answers `Landed` when the
    ///  commit is on the branch,
    /// and otherwise withdraws it so it can never land,
    /// and answers `Withdrawn`.
    Withdraw {
        /// The identity of the request,
        /// as the client sent it.
        id: String,
    },
    /// Work items were written for this repository;
    /// run what is pending.
    Work {
        /// The worktree's own Git directory,
        /// which holds the work directory.
        git_dir: PathBuf,
        /// The gateway credential for the standards review of the commits this message announces.
        /// It lives only in this
        /// message and in the service's memory,
        /// never in a work item or on disk.
        review_key: Option<ReviewKey>,
    },
    /// The rules text of this repository changed (a checkout,
    /// merge or pull of the person's own command);
    /// compile its forbidden-strings rules into the shared cache,
    /// on the service's own time.
    /// Nothing is answered:
    /// the connection is closed after the line is read.
    Warm {
        /// The worktree's own Git directory,
        /// which names the instance that serves it.
        git_dir: PathBuf,
        /// The repository's top level,
        /// whose configuration names the rules files.
        repository: PathBuf,
    },
}

/// What the service sends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceMessage {
    /// First line of every connection:
    /// the service's protocol version and the release it was built from.
    /// Services of
    ///  releases before the field existed send none,
    /// which reads as a release that is not this one.
    Welcome { version: u32, release: Option<String> },
    /// The request is queued behind others.
    Position { number: usize, of: usize },
    /// The commit is on the branch;
    /// `already` when an earlier request with the same identity landed it.
    Landed { id: String, commit: String, already: bool },
    /// A named path changed on the branch since the base;
    /// prepare again from `base`.
    RetryPrepare { id: String, base: String, landed: Vec<LandedCapture> },
    /// The request was not landed.
    Failed { id: String, code: FailureCode },
    /// The landing waits for another program that holds the repository's index:
    /// a line for the person's terminal.
    Waiting { text: String },
    /// A `waiting` line that also says this client's own request is inside the service's stuck branch write,
    /// so the commit may
    ///  already be on the branch.
    /// On the wire it is the `waiting` message with the extra field `inside_write` set to true,
    /// so a client
    ///  that does not know the field reads it as the `waiting` line it is (a notice,
    /// and the blocker its error names),
    /// and a service that
    ///  does not send it leaves a client that does on its older path:
    /// nothing in the protocol changes for either,
    /// and the version stays.
    InsideWrite { text: String },
    /// The request of this identity was withdrawn at the client's asking and will never land.
    Withdrawn { id: String },
    /// Who held the index when the wait ran out,
    /// in words that fit "... is using this repository's index".
    Holder { who: String },
    /// The service is alive and the request is still waiting for its answer:
    /// sent on a fixed beat while a request is queued
    ///  or landing,
    /// so the client can tell a working service from a stopped one and end the wait on silence.
    Heartbeat,
}

pub(crate) fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    return match value.get(name).and_then(Value::as_str) {
        Some(text) => Ok(text),
        None => Err(format!("field {name} is missing or not text")),
    };
}

/// Whether `text` is a full object name in hexadecimal (40 or 64 digits).
pub fn is_object_name(text: &str) -> bool {
    return (text.len() == 40 || text.len() == 64) && text.as_bytes().iter().all(u8::is_ascii_hexdigit);
}

/// What:
///  Whether one part of a repository-relative path is a plain name:
/// not empty,
/// not `.`,
/// `..` or `.git`.
/// Why:
///  `is_plain_path` checks every part,
/// so no path can leave the repository or reach Git's own directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainPart = (part: Uint8Array) => part.length > 0 && !['.', '..', '.git'].includes(text(part));
/// ```
fn is_plain_part(part: &[u8]) -> bool {
    return !part.is_empty() && part != b"." && part != b".." && part != b".git";
}

/// Whether a byte may appear in a request identity or a scratch directory identity.
/// The rule and its reason are in the foundation module that names what this program keeps on disk.
pub(crate) use crate::wrapper_names_old_and_new::is_identity_byte;

/// What:
///  Whether one component of a directory path is a normal step:
/// not `..` and not `.`.
/// Why:
///  `is_scratch_path` refuses a Git directory that climbs or stands still.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPlainComponent = (part: string) => part !== '..' && part !== '.';
/// ```
fn is_plain_component(part: std::path::Component<'_>) -> bool {
    return !matches!(part, std::path::Component::ParentDir | std::path::Component::CurDir);
}

/// Whether a byte separates the parts of a repository-relative path.
fn is_a_separator(byte: &u8) -> bool {
    return *byte == b'/';
}

/// Whether `path` is a plain repository-relative path:
/// no leading slash,
/// no empty,
/// `.` or `..` part,
/// no NUL or newline.
/// Why:
///  The bytes are checked rather than their text,
/// so a name that is not text is a name like any other and no name is judged by a spelling of it.
pub fn is_plain_path(path: &Path) -> bool {
    let bytes: &[u8] = path_bytes::path_bytes_of(path);
    return !bytes.is_empty()
        && !bytes.starts_with(b"/")
        && !bytes.contains(&b'\0')
        && !bytes.contains(&b'\n')
        && bytes.split(is_a_separator).all(is_plain_part);
}

/// The name prefix of a commit's scratch directory,
/// which the foundation module that names everything this program keeps on disk holds.
pub const SCRATCH_PREFIX: &str = SERVICE_SCRATCH_PREFIX;

/// What:
///  Whether `path` is the scratch directory of a commit in the worktree whose own Git directory is `git_dir`.
/// Why:
///   The service removes whatever a client names when the client's connection closes,
/// so the name must be one only the
///  wrapper makes (the prefix and an identity of letters,
/// digits and dashes) and the place must be directly inside the
///  repository scratch directory of the worktree's Git directory,
/// or directly inside that Git directory as releases before this one made it,
/// with no relative part that could lead elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isScratchPath(path, gitDir)
/// ```
pub fn is_scratch_path(path: &Path, git_dir: &Path) -> bool {
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else { return false };
    let identity: &str = name.strip_prefix(SCRATCH_PREFIX).unwrap_or("");
    // Where this release makes the tree: the repository scratch directory, which the sweep of a later command also covers.
    let swept: PathBuf = common_directory_of(git_dir).join(REPOSITORY_SCRATCH_NAME);
    let parent: Option<&Path> = path.parent();
    return git_dir.is_absolute()
        && (parent == Some(git_dir) || parent == Some(swept.as_path()))
        && !identity.is_empty()
        && identity.as_bytes().iter().all(is_identity_byte)
        && git_dir.components().all(is_plain_component)
        && !path.to_string_lossy().contains('\0');
}
