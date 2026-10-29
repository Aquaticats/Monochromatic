//! Requests the interface thread can make and the one-shot replies it polls for.

/// Every reply names the displayed text and the server process it answers.
use super::identity::{DocumentStamp, ServerIdentity};
/// Refused targets carry their reason instead of disappearing.
use super::target::TargetRefusal;
/// What:
///  `PathBuf` is an owned filesystem path (sibling:
///  borrowed `&Path`).
/// Why:
///  A navigation target outlives the server reply it was read from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// What:
///  A plain `enum` without data is a closed set of names.
///  `Copy` lets values be passed
///       around like numbers.
/// Why:
///  These are the three position requests of the accepted scope;
///  hints and diagnostics
///      are latest-value state,
///  not one-shot replies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestKind = 'definition' | 'references' | 'hover';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestKind {
    /// Where the symbol at the position is defined.
    Definition,
    /// Every use of the symbol at the position,
    ///  including its declaration.
    References,
    /// Documentation and type information for the symbol at the position.
    Hover,
}

/// One position request,
///  tagged with the text the position refers to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PositionRequest = { stamp: DocumentStamp; kind: RequestKind; position: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionRequest {
    /// Displayed file generation and revision the character offset belongs to.
    pub stamp: DocumentStamp,
    /// Which feature is asked for.
    pub kind: RequestKind,
    /// Character offset in the displayed text.
    ///  `usize` is the index type Helix ropes use.
    pub position: usize,
}

/// A file a definition or reference points at,
///  already validated and converted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OpenTarget = { path: string; outsideProject: boolean; sameDocument: boolean;
///                     line: number; range?: [number, number] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTarget {
    /// Canonical path of an existing regular file.
    pub path: PathBuf,
    /// True for a file outside the project root:
    ///  open it read-only,
    ///  without changing the root or the tree.
    pub outside_project: bool,
    /// True when the target is the file the request was made in.
    pub same_document: bool,
    /// Zero-based line the server named.
    pub line: usize,
    /// What:
    ///  `Option<(usize, usize)>` is "a pair of character offsets,
    ///  or nothing".
    /// Why:
    ///  Offsets exist only when the target text could be read and the server's line exists
    ///      in it;
    ///  the line number alone still allows navigation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// range?: [start: number, end: number];
    /// ```
    pub range: Option<(usize, usize)>,
}

/// A location the server returned:
///  either openable or reported as unavailable,
///  never dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { kind: 'open'; target: OpenTarget }
///             | { kind: 'unavailable'; uri: string; refusal: TargetRefusal };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// The reader may open this file.
    Open(
        /// The validated file and where in it the target is.
        OpenTarget,
    ),
    /// The reader must not open this address;
    ///  `uri` is shown with the reason.
    Unavailable {
        /// Address exactly as the server sent it.
        uri: String,
        /// Why the address cannot be opened.
        refusal: TargetRefusal,
    },
}

/// Hover content as delivered,
///  with the character range it describes when the server named one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HoverText = { text: string; markdown: boolean; range?: [number, number] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverText {
    /// Content parts joined by blank lines.
    pub text: String,
    /// True when the text is Markdown;
    ///  false for plain text.
    pub markdown: bool,
    /// Character offsets in the text of the reply's revision.
    pub range: Option<(usize, usize)>,
}

/// Why a request failed,
///  kept apart because each class leads to a different message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestFailure = { kind: 'rpc'; code: number; message: string }
///                     | { kind: 'timeout' } | { kind: 'streamClosed' } | { kind: 'other'; message: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestFailure {
    /// The server answered with a protocol error.
    ///  `i64` is a signed 64-bit integer;
    ///  protocol
    /// error codes are negative.
    Rpc {
        /// JSON-RPC error code,
        ///  for example `-32603`.
        code: i64,
        /// The server's own message.
        message: String,
    },
    /// No answer arrived within the server's configured request timeout.
    Timeout,
    /// The server process ended while the request was pending.
    StreamClosed,
    /// Anything else,
    ///  such as an answer that does not decode.
    Other(
        /// What went wrong,
        ///  in the client's or the decoder's words.
        String,
    ),
}

/// Every way one server can answer one position request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestOutcome =
///   | { kind: 'locations'; targets: Target[] } | { kind: 'hover'; hover: HoverText }
///   | { kind: 'empty' } | { kind: 'unsupported' } | { kind: 'starting' } | { kind: 'unsynchronized' }
///   | { kind: 'noServer' } | { kind: 'superseded' } | { kind: 'failed'; failure: RequestFailure };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestOutcome {
    /// Definition or reference targets;
    ///  never an empty list.
    Locations(
        /// One entry per location the server returned,
        ///  in the server's order.
        Vec<Target>,
    ),
    /// Hover content.
    Hover(
        /// The content and the range it describes.
        HoverText,
    ),
    /// The server answered successfully with nothing at this position.
    Empty,
    /// The server is running but does not offer this feature.
    Unsupported,
    /// The server has not finished starting;
    ///  ask again after its state becomes ready.
    Starting,
    /// The server could not be told about the latest reload,
    ///  so positions cannot be trusted.
    Unsynchronized,
    /// No running server serves this file for this feature.
    NoServer,
    /// The server gave up because its own state changed (`-32801` or `-32800`),
    ///  even after retries.
    Superseded,
    /// The request failed.
    Failed(
        /// The class of the failure.
        RequestFailure,
    ),
}

/// One server's answer to one position request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LanguageReply = { request: number; stamp: DocumentStamp; server?: ServerIdentity;
///                        kind: RequestKind; position: number; remaining: number; outcome: RequestOutcome };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageReply {
    /// Number `LanguageWorker::request` returned for this request.
    pub request: u64,
    /// File generation and revision the request was made for.
    pub stamp: DocumentStamp,
    /// Answering server process;
    ///  absent only when no server could be asked.
    pub server: Option<ServerIdentity>,
    /// Feature that was asked for.
    pub kind: RequestKind,
    /// Character offset that was asked about.
    pub position: usize,
    /// How many other servers have yet to answer this same request.
    pub remaining: usize,
    /// What the server said.
    pub outcome: RequestOutcome,
}
