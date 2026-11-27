//! What:
//!  Every failure the client can meet,
//!  and the facts its text names:
//!  the platform this command runs on,
//!  what the service manager last said about the unit this command asked about,
//!  what this command asked the manager to do about that unit,
//!  and what the branch showed once the service went silent after a commit was prepared.
//! Why:
//!  The set is closed,
//!  so a test renders every member of it,
//!  and each sentence a failure prints is built from these facts alone.
//!  The two facts about a start are declared here rather than beside the words that read them
//!  (`service_messages_of_a_start_that_was_asked_and_did_not_come.rs`),
//!  because a failure's own record holds them,
//!  and that record and those words would otherwise use each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesTheFailuresAPersonCanMeet.ts
//! type ManagerLastWords = { kind: 'state', text: string } | { kind: 'not-held' } | { kind: 'silent' };
//! type RequestedStart = 'nothing' | 'start' | 'reset-then-start' | 'restart';
//! type ClientFailure = { kind: 'did-not-start', unit?: string, answer: ManagerLastWords, asked: RequestedStart } | ...;
//! ```

use std::time::Duration;
/// What:
///  What the service manager last said about the unit this command asked after.
/// Why:
///  A state word and the two answers that are not states are three different next steps,
///  and folding them into one made a manager that answered every poll read as one that answered none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerLastWords = { kind: 'state', text: string } | { kind: 'not-held' } | { kind: 'silent' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagerLastWords {
    /// The manager named the unit's state,
    /// in its own word (`active`,
    /// `activating`,
    /// `inactive`,
    /// `failed`,
    /// ...).
    State(String),
    /// The manager answered that it holds no such unit,
    /// which is what a unit that stopped and was unloaded leaves.
    NotHeld,
    /// The manager gave no answer:
    /// the exchange with it failed,
    /// or it was never asked.
    Silent,
}

/// What:
///  What this command asked the service manager to do about the unit during its wait.
/// Why:
///  A person reads whether anything of this command asked for a start,
///  which is not the same as whether a start is under way:
///  a connection to the socket of a socket-activated unit asks for one,
///  and the client cannot tell which connection asked,
///  so the words say what this command asked and leave who started the unit to the manager's own state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestedStart = 'nothing' | 'start' | 'reset-then-start' | 'restart';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedStart {
    /// No call of this command asked the manager for anything.
    Nothing,
    /// This command asked the manager to start the unit,
    /// which a unit it holds no such unit for needs to be loaded again.
    Start,
    /// This command asked the manager to forget that the unit failed and then to start it.
    ResetAndStart,
    /// This command asked the manager to stop the unit and start it again.
    Restart,
}
/// What:
///  Where the client cannot get a service from the operating system.
/// Why:
///   Each platform has its own remedy,
/// and none has a second way to commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = 'windows' | 'bsd' | 'linux-without-user-services' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Platform {
    /// Windows,
    /// where activation is designed but not built.
    Windows,
    /// A BSD system.
    Bsd,
    /// Linux with no user service manager (minimal containers,
    /// WSL 1,
    /// other init systems).
    LinuxWithoutUserServices,
    /// Any other system.
    Other,
}
/// What:
///  Every failure that reaches the user,
/// with the facts its text names.
/// Why:
///   The set is closed,
/// so a test renders all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClientFailure = { kind: 'not-installed' } | { kind: 'did-not-start' } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientFailure {
    /// The wait for the commit service ended with nothing greeting it,
    /// and what it learned about the service manager's own answer is what says why:
    /// a start that was never asked for leaves the service's log empty.
    DidNotStart {
        /// The unit this command asked the service manager about,
        /// or nothing where no manager can be asked to start a service at all.
        unit: Option<String>,
        /// What the service manager last said about that unit.
        answer: ManagerLastWords,
        /// What this command asked the manager to do about it.
        asked: RequestedStart,
    },
    /// Something accepted the connection at the service's address and nothing answered;
    /// no service manager could be asked to start or restart it.
    NothingAnswered {
        /// The address the service was asked at.
        address: String,
        /// How long the wait lasted.
        waited: Duration,
    },
    /// The platform cannot start the service on demand.
    NoActivation(Platform),
    /// An exchange with the user service manager ran out of the time it was given,
    /// which says nothing about whether the manager is there.
    ManagerDidNotAnswer {
        /// The time that exchange was given.
        waited: Duration,
    },
    /// The service on the socket belongs to another account.
    OtherAccount,
    /// The service stopped before it confirmed the commit.
    MayNotHaveLanded,
    /// The installed service speaks no protocol this wrapper speaks,
    /// after a replacement attempt.
    NoCommonVersion,
    /// A service started from another release of the wrapper is still running after it was asked to retire and its unit was
    /// restarted,
    /// so the units start a program that is not the release this command runs.
    OtherRelease,
    /// The repository has no room left.
    RepositoryFull(String),
    /// The repository's filesystem is read-only.
    RepositoryReadOnly(String),
    /// This account may not write the repository.
    PermissionDenied(String),
    /// A repository move has frozen the repository.
    RepositoryFrozen(String),
    /// The branch changed while the commit was prepared,
    /// `times` times.
    BranchKeepsChanging { branch: String, times: usize },
    /// The branch was no longer where the amended commit was prepared,
    /// so the amend was not made.
    BranchMoved { branch: String },
    /// The merge,
    /// cherry-pick or revert this commit concludes changed while the commit was prepared,
    /// so there was no longer an operation to conclude.
    ConclusionStateChanged { branch: String },
    /// The branch moved while the person wrote a message;
    /// the message is saved in `file` for them to reuse.
    BranchMovedMessageSaved {
        branch: String,
        file: String,
        amend: bool,
    },
    /// The worktree switched from `branch` to `now` while the commit was prepared;
    /// the message the person wrote,
    /// if any,
    /// is
    ///  saved in `file`.
    BranchSwitched {
        branch: String,
        now: String,
        file: Option<String>,
    },
    /// The service,
    /// once it greeted,
    /// said nothing for longer than it ever stays silent while it works,
    /// so the request is given up rather than asked again.
    ServiceSilent,
    /// The service took no part of the request,
    /// or not the whole of it in time,
    /// so the request never reached it as one line and the commit was not made.
    ServiceNotReading {
        /// Whether the service took no byte of the request at all.
        took_nothing: bool,
    },
    /// The same silence after a commit was prepared,
    /// with the branch read afterwards so the message can state what it shows.
    ServiceSilentAfterPreparing {
        /// What the branch showed.
        shows: WhatTheBranchShows,
    },
    /// The service kept the request for `minutes` minutes without answering,
    /// and the client withdrew it:
    /// it will not land.
    StillWithService {
        minutes: u64,
        blocker: Option<String>,
    },
    /// The client stopped waiting and could not confirm the service withdrew the request;
    /// `blocker` is what the service last said it
    ///  was stuck in,
    /// such as a branch write that has not returned.
    WithdrawalUnconfirmed { blocker: Option<String> },
    /// The person interrupted the command with `signal` while it waited,
    /// and the service confirmed that the request will never land.
    Interrupted {
        /// The signal the person sent.
        signal: i32,
        /// The process that sent it,
        /// where the kernel named one;
        /// nothing for the terminal's own interrupt.
        sender: Option<i32>,
        /// What the service last said it was waiting for.
        blocker: Option<String>,
    },
    /// The person interrupted the command with `signal`,
    /// and the service did not confirm that it gave the request up,
    /// so whether the commit was made is not known.
    InterruptedNotKnown {
        /// The signal the person sent.
        signal: i32,
        /// The process that sent it,
        /// where the kernel named one.
        sender: Option<i32>,
        /// What the service last said it was waiting for.
        blocker: Option<String>,
    },
    /// A Git step of the landing ran longer than the wrapper allows and was ended;
    /// the commit was withdrawn and kept.
    StepTooSlow,
    /// The request was refused as malformed or unsafe.
    Refused,
    /// Another program holds the repository's index.
    IndexInUse(String),
    /// A reference lock stands in the way of the branch write,
    /// and nothing proves that a Git process still holds it;
    /// `words` names the lock and the command that clears it.
    ReferenceLocked { branch: String, words: String },
    /// The service failed in a way it could not resolve.
    Unavailable,
    /// The service program is not next to the wrapper.
    ProgramMissing(String),
    /// Setting up the service failed for a reason the person can act on.
    CouldNotStart(String),
    /// The commit service's address is longer than this system's address of that kind can be,
    /// so no service could listen there.
    SocketPathTooLong {
        /// The address the service would have listened at.
        path: String,
        /// How many bytes that address is.
        length: usize,
        /// The longest address this system can hold.
        longest: usize,
        /// The longest runtime directory,
        /// which is the part of the address this command's setting decides,
        /// that still leaves room for the rest.
        longest_runtime: usize,
    },
    /// The commit service was started and listens at an address this command does not look at,
    /// because the service manager and this command use different runtime directories.
    ServiceElsewhere {
        /// The address the manager made the service listen at.
        made: String,
        /// The address this command waited at.
        looked_for: String,
        /// The runtime directory the manager runs with.
        manager_runtime: String,
        /// The runtime directory this command was given.
        client_runtime: String,
    },
    /// A unit file of the same name that something else wrote stands in the way.
    UnitInTheWay(String),
    /// The commit is saved but the backup push and upkeep could not be handed over.
    WorkNotHandedOver(String),
    /// The content prepared for the commit was removed from the repository before the commit landed,
    /// twice.
    ContentPruned,
}
/// What:
///  What the branch shows once the service went silent after a commit was prepared.
/// Why:
///   "It is not known whether the commit was made" was printed while the service still
/// held the commit and landed it 73 s later (2026-11-09T07:44:20Z).
/// One bounded read of the branch turns that into a fact,
/// and a branch that cannot be read stays the honest "not known".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheBranchShows = 'made' | 'not-made' | 'unread';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatTheBranchShows {
    /// The branch is at the commit this command prepared.
    Made,
    /// The branch is not at that commit.
    NotMade,
    /// The branch could not be read.
    Unread,
}
