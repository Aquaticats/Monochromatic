//! What:
//!  Every answer the repository link check can give,
//! and the exit status of the Git wrapper's refusal.
//! Why:
//!  The answers and their texts equal the TypeScript check's (`package/repository-link/check/src/link-types.ts`),
//! and one shared table of fixture states proves it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LinkAnswer = { kind: AnswerKind; ready: boolean; message: string };
//! ```

/// Exit status of the Git wrapper's refusal:
/// Git's own fatal status (`doc/planning/repository-on-the-860-drive.md`,
/// "Work items per tool",
/// the exit status paragraph).
pub const EXIT_GIT_FATAL: i32 = 128;

/// What:
///  Which answer the check gave.
/// Why:
///  A caller and a test tell the answers apart without reading the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnswerKind = 'drive-did-not-answer' | 'drive-not-mounted' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnswerKind {
    /// The filesystem did not answer within the deadline.
    DriveDidNotAnswer,
    /// The target is missing and the mount point is not a mount.
    DriveNotMounted,
    /// No identity file was read.
    IdentityMissing,
    /// Nothing is at the link path.
    LinkMissing,
    /// The mount point is a mount and the target is missing.
    MountedFolderMissing,
    /// The path is not a repository folder.
    NotARepository,
    /// Phase `after`:
    /// the link is usable.
    ReadyAfter,
    /// Phase `before`:
    /// the real folder is usable.
    ReadyBefore,
    /// A real folder replaced the link.
    RealFolderReplacedLink,
    /// The folder is on another filesystem than recorded.
    WrongFilesystem,
    /// The link or the repository is not the recorded one.
    WrongRepository,
}

impl AnswerKind {
    /// What:
    ///  The kebab-case name the shared table and the TypeScript check use.
    /// Why:
    ///  One spelling of each kind in both languages.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answer.kind // 'drive-not-mounted'
    /// ```
    pub fn name(self) -> &'static str {
        return match self {
            Self::DriveDidNotAnswer => "drive-did-not-answer",
            Self::DriveNotMounted => "drive-not-mounted",
            Self::IdentityMissing => "identity-missing",
            Self::LinkMissing => "link-missing",
            Self::MountedFolderMissing => "mounted-folder-missing",
            Self::NotARepository => "not-a-repository",
            Self::ReadyAfter => "ready-after",
            Self::ReadyBefore => "ready-before",
            Self::RealFolderReplacedLink => "real-folder-replaced-link",
            Self::WrongFilesystem => "wrong-filesystem",
            Self::WrongRepository => "wrong-repository",
        };
    }
}

/// What:
///  The check's answer:
/// a kind,
/// whether the repository is ready,
/// and the plain message for a person.
/// Why:
///  The gate prints the message of an answer that is not ready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkAnswer = { kind: AnswerKind; ready: boolean; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkAnswer {
    /// Which answer.
    pub kind: AnswerKind,
    /// Whether the repository is usable.
    pub ready: bool,
    /// The plain message.
    pub message: String,
}

impl LinkAnswer {
    /// What:
    ///  An answer that is not ready.
    /// Why:
    ///  Every refusal has the same shape.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusal({ kind, message })
    /// ```
    pub fn refusal(kind: AnswerKind, message: String) -> LinkAnswer {
        return LinkAnswer { kind, ready: false, message };
    }

    /// What:
    ///  An answer that is ready.
    /// Why:
    ///  A ready answer carries a message too,
    /// so a diagnostic can print it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind, ready: true, message }
    /// ```
    pub fn ready(kind: AnswerKind, message: String) -> LinkAnswer {
        return LinkAnswer { kind, ready: true, message };
    }
}
