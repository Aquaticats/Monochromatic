//! Latest-value description of what language support the displayed file has,
//!  and why not.

/// Each status row names the server process it describes.
use super::identity::ServerIdentity;
/// What:
///  `PathBuf` is an owned filesystem path (sibling:
///  borrowed `&Path`).
/// Why:
///  A refused root is reported with the directory that was refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// What:
///  A tagged union of every state one configured server can be in for the displayed file.
///       `String` fields own their text (sibling:
///  borrowed `&str`).
/// Why:
///  Each state needs a different response from the reader:
///  wait,
///  show a reason,
///  or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerState =
///   | { kind: 'missingExecutable'; reason: string } | { kind: 'launchRefused'; reason: string }
///   | { kind: 'starting' } | { kind: 'ready' }
///   | { kind: 'failedToStart'; reason: string } | { kind: 'unsynchronized' } | { kind: 'exited' }
///   | { kind: 'rootOutsideProject'; root: string } | { kind: 'wrongWorkingDirectory'; directory: string }
///   | { kind: 'notStarted'; reason: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerState {
    /// The server's real program is not installed,
    ///  or the project has no usable copy of it.
    MissingExecutable {
        /// Which program was looked for and where.
        reason: String,
    },
    /// The launch policy could not produce a safe launch;
    ///  nothing was spawned in its place.
    LaunchRefused {
        /// What the policy or the launch preparation reported.
        reason: String,
    },
    /// The process runs but has not answered `initialize`;
    ///  no request may be sent yet.
    Starting,
    /// Initialized;
    ///  requests are answered or report an unsupported feature individually.
    Ready,
    /// The process could not be started,
    ///  ended before initializing,
    ///  or never answered `initialize`.
    FailedToStart {
        /// What was observed.
        reason: String,
    },
    /// Running,
    ///  but it takes no change notifications and cannot be resynchronized for this file.
    Unsynchronized,
    /// The process ended after initializing;
    ///  the next open or position request starts a new one.
    Exited,
    /// Helix would root this server at a directory outside the project,
    ///  so it was not started.
    RootOutsideProject {
        /// The directory that would have become the server's root.
        root: PathBuf,
    },
    /// The process working directory does not contain the project,
    ///  so no root can be computed.
    WrongWorkingDirectory {
        /// The directory Helix resolves roots from.
        directory: PathBuf,
    },
    /// The server's definition did not apply to this file.
    NotStarted {
        /// Why Helix declined to start it.
        reason: String,
    },
}

/// What:
///  A record of five booleans.
///  `Copy` lets it be passed like a number.
/// Why:
///  A ready server may still lack a feature;
///  the reader can hide what cannot work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Features = { definition: boolean; references: boolean; hover: boolean;
///                   inlayHints: boolean; pullDiagnostics: boolean };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Features {
    /// Go-to-definition is offered and enabled for this language.
    pub definition: bool,
    /// Find-references is offered and enabled for this language.
    pub references: bool,
    /// Hover is offered and enabled for this language.
    pub hover: bool,
    /// Inlay hints are offered and enabled for this language.
    pub inlay_hints: bool,
    /// Pull diagnostics are offered and enabled for this language.
    pub pull_diagnostics: bool,
}

/// One configured server as it relates to the displayed file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerStatus = { server: ServerIdentity; state: ServerState; features?: Features; progress?: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerStatus {
    /// Server name and process generation;
    ///  generation zero means it was never started.
    pub server: ServerIdentity,
    /// Current state.
    pub state: ServerState,
    /// What:
    ///  `Option<Features>` is "features,
    ///  or nothing".
    /// Why:
    ///  Capabilities are known only after the server initialized.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// features?: Features;
    /// ```
    pub features: Option<Features>,
    /// Title of work the server reports as running,
    ///  coalesced to the latest one.
    pub progress: Option<String>,
}

/// Why a displayed file has,
///  or does not have,
///  servers at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocumentState = 'closed' | 'noLanguage' | 'noServerConfigured' | 'outsideProject' | 'attached';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentState {
    /// No file is displayed.
    Closed,
    /// Helix recognizes no language for this file name.
    NoLanguage,
    /// The language has no language server in Helix's configuration.
    NoServerConfigured,
    /// The file is outside the project root;
    ///  only servers that already run are asked about it.
    OutsideProject,
    /// The rows in `servers` describe the language's servers.
    Attached,
}

/// The whole latest-value status the interface thread polls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LanguageStatus = { file?: number; language?: string; document: DocumentState; servers: ServerStatus[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageStatus {
    /// File-open generation this status describes;
    ///  absent when nothing is displayed.
    pub file: Option<u64>,
    /// Helix language name,
    ///  for example `rust`.
    pub language: Option<String>,
    /// Whether servers apply to the displayed file at all.
    pub document: DocumentState,
    /// What:
    ///  `Vec<ServerStatus>` is a growable list (siblings:
    ///  fixed `[T; N]`,
    ///  borrowed `&[T]`).
    /// Why:
    ///  The number of configured servers differs per language.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// servers: ServerStatus[];
    /// ```
    pub servers: Vec<ServerStatus>,
}

/// The status before any file is displayed.
impl LanguageStatus {
    /// Build the "nothing displayed" status.
    pub fn closed() -> Self {
        return Self {
            file: None,
            language: None,
            document: DocumentState::Closed,
            servers: Vec::new(),
        };
    }
}
