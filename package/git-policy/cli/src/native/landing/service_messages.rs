//! What:
//!  Everything the wrapper tells the user when it cannot get a commit landed by the resident service,
//! and the notices
//!  it prints while it waits.
//! Why:
//!  A failure the system can resolve is resolved without the user;
//! the rest ends the command with plain words,
//! the
//!  affected input named,
//! every remedy given,
//! and no implementation term (`doc/planning/cli-git-resident-service.md`,
//!  "Failures the client can meet").
//! The notices it prints while it waits are declared in
//! `service_messages_of_a_wait_for_the_service.rs`
//! and read from here.
//! The texts live here as constants so a test can check each against a list of terms
//!  that must never reach a user,
//! and so the text a client prints cannot drift from the text the tests read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stderr.write(render({ kind: 'not-installed' }));
//! ```

use std::path::Path;
use std::time::Duration;

/// The notices a wait prints are declared beside this file
/// (`service_messages_of_a_wait_for_the_service.rs`),
/// because the rendering of every failure is at its line bound,
/// and the call sites read them from here.
pub use crate::service_messages_of_a_wait_for_the_service::{
    SILENT_NOTICE, STARTING_NOTICE, UPDATING_NOTICE, position_notice, rebuild_notice, restarting_notice, starting_notice, unplaced_notice,
};

/// The command that shows why the service did not start (Linux with systemd).
pub const LOG_COMMAND: &str = "journalctl --user --unit 'monochromatic-git-policy-cli@*.service'";

/// The command that turns on user services for an account that is not logged in (Linux with systemd).
pub const LINGER_COMMAND: &str = "loginctl enable-linger";

/// The command that restarts the commit service,
/// so the manager starts the one the installed units name (Linux with systemd).
pub const RESTART_COMMAND: &str = "systemctl --user restart 'monochromatic-git-policy-cli@*.service'";

/// The command that stops the commit service,
/// so the manager starts the one the installed units name on the next connection (Linux with systemd).
/// Why:
///   A restart starts the program the manager's unit names,
/// which after a rollback is still the newer release's,
/// so stopping the
///  service is the one command that lets the older wrapper's own service answer.
pub const STOP_COMMAND: &str = "systemctl --user stop 'monochromatic-git-policy-cli@*.service'";

/// The commands that show whether the commit was made.
pub const CHECK_COMMANDS: [&str; 2] = ["git log -1", "git status"];



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
    /// The service was started and did not come up.
    DidNotStart,
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
    /// The service on the socket belongs to another account.
    OtherAccount,
    /// The service stopped before it confirmed the commit.
    MayNotHaveLanded,
    /// The installed service speaks no protocol this wrapper speaks,
    /// after a replacement attempt.
    NoCommonVersion,
    /// A service started from another release of the wrapper is still running after it was asked to exit.
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
    BranchMovedMessageSaved { branch: String, file: String, amend: bool },
    /// The worktree switched from `branch` to `now` while the commit was prepared;
    /// the message the person wrote,
    /// if any,
    /// is
    ///  saved in `file`.
    BranchSwitched { branch: String, now: String, file: Option<String> },
    /// The service sent nothing for longer than it ever stays silent while it works,
    /// on every try to ask it again.
    ServiceSilent,
    /// The same silence after a commit was prepared,
    /// with the branch read afterwards so the message can state what it shows.
    ServiceSilentAfterPreparing {
        /// What the branch showed.
        shows: WhatTheBranchShows,
    },
    /// The service kept the request for `minutes` minutes without answering,
    /// and the client withdrew it:
    /// it will not land.
    StillWithService { minutes: u64, blocker: Option<String> },
    /// The client stopped waiting and could not confirm the service withdrew the request;
    /// `blocker` is what the service last said it
    ///  was stuck in,
    /// such as a branch write that has not returned.
    WithdrawalUnconfirmed { blocker: Option<String> },
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

/// The sentence every failure ends with when nothing was changed.
pub const NOTHING_CHANGED: &str = "Nothing in the repository changed.";

/// What:
///  The text for a failure.
/// Why:
///   Plain words,
/// the input named,
/// every remedy;
/// nothing about how the service works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// render(failure): string
/// ```
pub fn render(failure: &ClientFailure) -> String {
    return match failure {
        ClientFailure::DidNotStart => format!(
            "The commit service did not start, so this commit was not made. Its log shows why: `{LOG_COMMAND}`. \
             Run the commit again after fixing that. {NOTHING_CHANGED}"
        ),
        ClientFailure::NothingAnswered { address, waited } => format!(
            "Nothing has answered at the commit service's address for this worktree ({address}) for {} seconds, so this \
             commit was not made. A service that is stopped inside its own process (a frozen control group, a debugger, a \
             suspended machine, or a wait in the kernel that never ends) looks like that, and this computer has no user \
             service manager this git can ask to start or restart it. Check what the service of this worktree is doing with \
             `git monochromatic-git-policy-cli service status`, end the service if it is stopped inside its own process, and \
             run the commit again. {NOTHING_CHANGED}",
            waited.as_secs().max(1)
        ),
        ClientFailure::NoActivation(platform) => format!(
            "This computer cannot start the commit service on demand, so this commit was not made. {} {NOTHING_CHANGED}",
            platform_remedy(*platform)
        ),
        ClientFailure::OtherAccount => format!(
            "Another account runs the commit service here, so this commit was not made. Use your own account, or ask \
             the other account to stop its service, then run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::MayNotHaveLanded => format!(
            "The commit service stopped before it confirmed this commit. Run `{}` to see whether it was made, and `{}` \
             to see what is still staged. If it was not made, run the commit again.",
            CHECK_COMMANDS[0], CHECK_COMMANDS[1]
        ),
        ClientFailure::NoCommonVersion => format!(
            "This git is older than the commit service that is running, which was started from a newer release of this git \
             wrapper, so this commit was not made. Update this git with the tool that installed it, or install the release it \
             belongs to and stop the newer service with `{STOP_COMMAND}`; then run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::OtherRelease => format!(
            "The commit service that is running was started from another release of this git wrapper and did not stop when \
             asked, so this commit was not made. Run the commit again: the next commit sets the service up from the units it \
             installs and asks the manager to reload them. A service that still will not stop can be restarted with \
             `{RESTART_COMMAND}`. {NOTHING_CHANGED}"
        ),
        ClientFailure::RepositoryFull(path) => format!(
            "There is no space left for {path}, so this commit was not made. Free some space and run the commit again. \
             {NOTHING_CHANGED}"
        ),
        ClientFailure::RepositoryReadOnly(path) => format!(
            "{path} cannot be written to, so this commit was not made. Make the repository writable (check its \
             permissions and whether the disk is mounted read-only) and run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::PermissionDenied(path) => format!(
            "This account has no permission to write {path}, so this commit was not made. Check who owns the repository's files and \
             folders and their permissions, and run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::RepositoryFrozen(path) => format!(
            "{path} is read-only because a repository move has frozen the repository, so this commit was not made. A repository move \
             has frozen the repository; it thaws when the move's thaw step runs. Run the commit again after the move finishes. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchKeepsChanging { branch, times } => format!(
            "The branch {branch} changed {times} times while this commit was being prepared, so it was not made. Run \
             the commit again; if other people commit to the same branch at the same moment, it will succeed once they \
             pause. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchMoved { branch } => format!(
            "The branch {branch} moved while this commit was being prepared, and a commit that replaces or rewrites another \
             commit, or one that was written or signed by hand, cannot be moved onto a different last commit, so it was not \
             made. Run the commit again to make it on the branch as it is now. {NOTHING_CHANGED}"
        ),
        ClientFailure::ConclusionStateChanged { branch } => format!(
            "The merge, cherry-pick or revert this commit was concluding changed while it was being prepared, so it was \
             not made on the branch {branch}. Another command either finished that operation or abandoned it. Run `git \
             status` to see what the worktree holds now, then run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchMovedMessageSaved { branch, file, amend } => format!(
            "The branch {branch} moved while this commit was being prepared, so it was not made. The message you wrote is saved \
             in {file}. Run `git commit {}-F {file}` to make the commit on the branch as it is now. {NOTHING_CHANGED}",
            if *amend { "--amend " } else { "" }
        ),
        ClientFailure::BranchSwitched { branch, now, file } => format!(
            "The worktree switched from {branch} to {now} while this commit was being prepared, so it was not made. {} Switch back \
             to {branch} and run the commit again, or run it here to commit on {now}. {NOTHING_CHANGED}",
            match file {
                Some(saved) => format!("The message you wrote is saved in {saved}; `git commit -F {saved}` uses it."),
                None => String::new(),
            }
        ),
        ClientFailure::ServiceSilent => format!(
            "The commit service stopped answering while this commit waited, and it did not answer when asked again, so it is \
             not known whether the commit was made. Run `{}` to see whether it was made, and `{}` to see what is still \
             staged. If it was not made, look at why the service is stuck with `{LOG_COMMAND}`, restart it with \
             `{RESTART_COMMAND}`, and run the commit again.",
            CHECK_COMMANDS[0], CHECK_COMMANDS[1]
        ),
        ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Made } => String::from(
            "The commit service stopped answering, and the commit was made: the branch is at the commit this command \
             prepared. There is nothing to run.",
        ),
        ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::NotMade } => format!(
            "The commit service stopped answering while this commit waited, and it did not answer when asked again. The \
             branch is not at the commit this command prepared, so it was not made by this command; the service may still \
             land it, so run `{}` to see. If it was not made, look at why the service is stuck with `{LOG_COMMAND}`, \
             restart it with `{RESTART_COMMAND}`, and run the commit again. `{}` shows what is still staged.",
            CHECK_COMMANDS[0], CHECK_COMMANDS[1]
        ),
        ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Unread } => format!(
            "The commit service stopped answering while this commit waited, and the branch could not be read afterwards, so \
             it is not known whether the commit was made. Run `{}` to see whether it was made, and `{}` to see what is still \
             staged. If it was not made, look at why the service is stuck with `{LOG_COMMAND}`, restart it with \
             `{RESTART_COMMAND}`, and run the commit again.",
            CHECK_COMMANDS[0], CHECK_COMMANDS[1]
        ),
        ClientFailure::StillWithService { minutes, blocker } => format!(
            "This commit waited {minutes} minutes for the commit service and was withdrawn, so it will not land later by itself.{} \
             Your staged changes and your worktree are as they were. Run the commit again; if the service is stuck, look at why with \
             `{LOG_COMMAND}` and restart it with `{RESTART_COMMAND}`. {NOTHING_CHANGED}",
            match blocker.as_ref() {
                Some(text) => format!(" It was waiting for: {text}."),
                None => String::new(),
            }
        ),
        ClientFailure::WithdrawalUnconfirmed { blocker } => format!(
            "This commit waited for the commit service until the longest wait the wrapper allows, and the service did not confirm \
             that it withdrew it, so it is not known whether the commit was made.{} Run `{}` to see whether it was made, and `{}` to \
             see what is still staged. If it was not made, run the commit again; if the service is stuck, look at why with \
             `{LOG_COMMAND}` and restart it with `{RESTART_COMMAND}`.",
            match blocker.as_ref() {
                Some(text) => format!(" The service had said: {}", text.trim()),
                None => String::new(),
            },
            CHECK_COMMANDS[0], CHECK_COMMANDS[1]
        ),
        ClientFailure::StepTooSlow => format!(
            "A Git step of this commit ran longer than the commit service allows and was ended, so this commit was not made and will not \
             land later by itself. The step is named above, and the commit built for you is kept on the side ref named there when one \
             is. Run the commit again; if it happens again, look at why with `{LOG_COMMAND}`. {NOTHING_CHANGED}"
        ),
        ClientFailure::Refused => format!(
            "The commit service refused this commit because it names a path or a value that cannot be committed, so it \
             was not made. Check the paths you passed to the commit. {NOTHING_CHANGED}"
        ),
        ClientFailure::IndexInUse(holder) => format!(
            "{holder} is using this repository's index, so this commit was not made. Wait for it to finish, or close \
             it, and run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::ReferenceLocked { branch, words } => format!(
            "A lock stands in the way of writing the branch {branch}, so this commit was not made. {words} Run the commit again \
             once it is clear. {NOTHING_CHANGED}"
        ),
        ClientFailure::ProgramMissing(path) => format!(
            "The commit service program was not found at {path}, so this commit was not made. Reinstall the git wrapper \
             with the tool that installed it, so that the program sits next to it, then run the commit again. \
             {NOTHING_CHANGED}"
        ),
        ClientFailure::UnitInTheWay(path) => format!(
            "The file `{path}` has the name of a settings file of the commit service but was not written by the git wrapper, \
             so the commit service could not be set up and this commit was not made. Move or delete that file, then run the \
             commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::WorkNotHandedOver(reason) => format!(
            "Your commit is saved. Its backup push and upkeep could not be handed to the commit service: {reason}. \
             Run `git push` when ready."
        ),
        ClientFailure::ContentPruned => format!(
            "The files of this commit were removed from the repository while the commit was being made, so it was not made. \
             Git removes content that no branch or index refers to yet when `git gc --prune=now` or `git prune` runs in the \
             same repository, and Git's documentation says that `--prune=now` risks failures and corruption when another \
             process writes to the repository at the same moment. Run the commit again once the cleanup has finished, and avoid `--prune=now` while commits are \
             being made (the default `git gc` keeps new content for two weeks). If nothing like that was running, run \
             `git fsck` to check the repository. {NOTHING_CHANGED}"
        ),
        ClientFailure::CouldNotStart(reason) => format!(
            "The commit service could not be set up, so this commit was not made: {reason}. Fix that and run the commit \
             again. {NOTHING_CHANGED}"
        ),
        ClientFailure::SocketPathTooLong { path, length, longest, longest_runtime } => {
            crate::service_messages_of_a_start_that_cannot_come::socket_path_too_long(path, *length, *longest, *longest_runtime)
        }
        ClientFailure::ServiceElsewhere { made, looked_for, manager_runtime, client_runtime } => {
            crate::service_messages_of_a_start_that_cannot_come::service_elsewhere(made, looked_for, manager_runtime, client_runtime)
        }
        ClientFailure::Unavailable => format!(
            "The commit service could not finish this commit, so it was not made. Run the commit again; if it fails \
             again, look at the service's log with `{LOG_COMMAND}`. {NOTHING_CHANGED}"
        ),
    };
}

/// What:
///  The remedy sentence for a platform.
/// Why:
///   Windows and the BSD systems have no activation yet;
/// Linux without user services can turn them on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// platformRemedy(platform)
/// ```
pub fn platform_remedy(platform: Platform) -> String {
    return match platform {
        Platform::Windows => String::from(
            "Committing through this wrapper is not supported on Windows yet. Use the Linux or macOS machine for commits \
             to this repository.",
        ),
        Platform::Bsd => String::from(
            "Committing through this wrapper is not supported on this system yet. Use a Linux or macOS machine for \
             commits to this repository.",
        ),
        Platform::LinuxWithoutUserServices => format!(
            "This account has no user service manager. Turn it on with `{LINGER_COMMAND}` (the system must run \
             systemd), or commit from a system that does."
        ),
        Platform::Other => String::from(
            "Committing through this wrapper is not supported on this system. Use a Linux or macOS machine for \
             commits to this repository.",
        ),
    };
}



/// The texts that must never reach a user:
/// implementation terms the system should have resolved itself.
///  Matched case-insensitively against whole words after the quoted commands are removed.
pub const DENYLIST: [&str; 22] = [
    "protocol", "journal", "request id", "daemon", "socket", "errno", "epipe", "econnrefused", "eagain", "flock",
    "lock file", "peer", "frame", "handshake", "batch", "oid", "fast-import", "unix domain", "abstract", "pid",
    "uid", "os error",
];

/// The repository path a failure names,
/// as text.
pub fn path_text(path: &Path) -> String {
    return path.display().to_string();
}


/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_tests.rs"]
mod tests;
