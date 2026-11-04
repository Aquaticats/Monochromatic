//! What:
//!  Why a manual push's scope could not be established, as a cause the hint is chosen by.
//! Why:
//!  Every such failure stops the push with a `content-unavailable` engine failure, and that one code covers causes the
//!  person answers differently: a remote that moved is retried, a remote that could not be read is checked for network and
//!  credentials, and output the wrapper cannot read is a defect to report. A hint chosen by the code alone named missing
//!  content for all of them, which was true for one. The cause travels with the failure so the hint is chosen by it,
//!  never by matching the message text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ManualPushCause = 'content-missing' | 'remote-moved' | 'remote-unreachable' | 'git-not-started' | 'git-silent' | 'unexpected-output' | 'unsupported-object' | 'self-explained';
//! ```

/// What:
///  The cause of a manual-push discovery failure.
/// Why:
///   Each variant names what the person can do, because that is what separates the hints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManualPushCause = 'content-missing' | 'remote-moved';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManualPushCause {
    /// A Git read of the local repository failed for lack of an object, a ref or history: the content is not in this clone.
    ContentMissing,
    /// A destination ref changed between Git's dry run and the wrapper's confirmation: nothing is wrong, run it again.
    RemoteMoved,
    /// The remote's refs could not be read: network, address or credentials.
    RemoteUnreachable,
    /// Git could not be started for the check, so the Git the wrapper runs is missing or not executable.
    GitNotStarted,
    /// Git ended the check by a signal or with a failure status and no message, so there is nothing of Git's to relay.
    GitSilent,
    /// Git answered in a form the wrapper does not read: a defect of the wrapper, not of the repository.
    UnexpectedOutput,
    /// The push names an object of a kind the manual-push policies do not read (not a commit).
    UnsupportedObject,
    /// The message already says what to do (a deadline with its setting, a repository that cannot be located); no hint is added.
    SelfExplained,
}
