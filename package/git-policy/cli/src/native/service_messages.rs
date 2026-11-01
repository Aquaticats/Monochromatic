//! What:
//!  Everything the wrapper tells the user when it cannot get a commit landed by the resident service, and the notices
//!  it prints while it waits.
//! Why:
//!  A failure the system can resolve is resolved without the user; the rest ends the command with plain words, the
//!  affected input named, every remedy given, and no implementation term (`doc/planning/cli-git-resident-service.md`,
//!  "Failures the client can meet"). The texts live here as constants so a test can check each against a list of terms
//!  that must never reach a user, and so the text a client prints cannot drift from the text the tests read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stderr.write(render({ kind: 'not-installed' }));
//! ```

use std::path::Path;

/// The command that shows why the service did not start (Linux with systemd).
pub const LOG_COMMAND: &str = "journalctl --user --unit cli-git.service";

/// The command that turns on user services for an account that is not logged in (Linux with systemd).
pub const LINGER_COMMAND: &str = "loginctl enable-linger";

/// The commands that show whether the commit was made.
pub const CHECK_COMMANDS: [&str; 2] = ["git log -1", "git status"];

/// Printed once when the service takes longer than the grace to answer.
pub const STARTING_NOTICE: &str = "Starting the commit service...";

/// Printed once when the service is being replaced by a newer one.
pub const UPDATING_NOTICE: &str = "Updating the commit service...";

/// What:
///  Where the client cannot get a service from the operating system.
/// Why:
///   Each platform has its own remedy, and none has a second way to commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = 'windows' | 'bsd' | 'linux-without-user-services' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Platform {
    /// Windows, where activation is designed but not built.
    Windows,
    /// A BSD system.
    Bsd,
    /// Linux with no user service manager (minimal containers, WSL 1, other init systems).
    LinuxWithoutUserServices,
    /// Any other system.
    Other,
}

/// What:
///  Every failure that reaches the user, with the facts its text names.
/// Why:
///   The set is closed, so a test renders all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClientFailure = { kind: 'not-installed' } | { kind: 'did-not-start' } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientFailure {
    /// The service was started and did not come up.
    DidNotStart,
    /// The platform cannot start the service on demand.
    NoActivation(Platform),
    /// The service on the socket belongs to another account.
    OtherAccount,
    /// The service stopped before it confirmed the commit.
    MayNotHaveLanded,
    /// The installed service speaks no protocol this wrapper speaks, after a replacement attempt.
    NoCommonVersion,
    /// The repository has no room left.
    RepositoryFull(String),
    /// The repository cannot be written.
    RepositoryReadOnly(String),
    /// The branch changed while the commit was prepared, `times` times.
    BranchKeepsChanging { branch: String, times: usize },
    /// The commit waited `minutes` minutes behind others.
    WaitedTooLong { minutes: u64 },
    /// The request was refused as malformed or unsafe.
    Refused,
    /// Another program holds the repository's index.
    IndexInUse(String),
    /// The service failed in a way it could not resolve.
    Unavailable,
    /// The service program is not next to the wrapper.
    ProgramMissing(String),
    /// Setting up the service failed for a reason the person can act on.
    CouldNotStart(String),
    /// A unit file of the same name that something else wrote stands in the way.
    UnitInTheWay(String),
    /// The commit is saved but the backup push and upkeep could not be handed over.
    WorkNotHandedOver(String),
}

/// The sentence every failure ends with when nothing was changed.
const NOTHING_CHANGED: &str = "Nothing in the repository changed.";

/// What:
///  The text for a failure.
/// Why:
///   Plain words, the input named, every remedy; nothing about how the service works.
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
            "This git is older than the installed commit service and cannot work with it, so this commit was not made. \
             Update the git wrapper with the tool that installed it, and run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::RepositoryFull(path) => format!(
            "There is no space left for {path}, so this commit was not made. Free some space and run the commit again. \
             {NOTHING_CHANGED}"
        ),
        ClientFailure::RepositoryReadOnly(path) => format!(
            "{path} cannot be written to, so this commit was not made. Make the repository writable (check its \
             permissions and whether the disk is mounted read-only) and run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchKeepsChanging { branch, times } => format!(
            "The branch {branch} changed {times} times while this commit was being prepared, so it was not made. Run \
             the commit again; if other people commit to the same branch at the same moment, it will succeed once they \
             pause. {NOTHING_CHANGED}"
        ),
        ClientFailure::WaitedTooLong { minutes } => format!(
            "This commit waited {minutes} minutes behind other commits and was not made. Run it again later, or raise \
             the waiting limit in the settings for the wrapper. {NOTHING_CHANGED}"
        ),
        ClientFailure::Refused => format!(
            "The commit service refused this commit because it names a path or a value that cannot be committed, so it \
             was not made. Check the paths you passed to the commit. {NOTHING_CHANGED}"
        ),
        ClientFailure::IndexInUse(holder) => format!(
            "{holder} is using this repository's index, so this commit was not made. Wait for it to finish, or close \
             it, and run the commit again. {NOTHING_CHANGED}"
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
        ClientFailure::CouldNotStart(reason) => format!(
            "The commit service could not be set up, so this commit was not made: {reason}. Fix that and run the commit \
             again. {NOTHING_CHANGED}"
        ),
        ClientFailure::Unavailable => format!(
            "The commit service could not finish this commit, so it was not made. Run the commit again; if it fails \
             again, look at the service's log with `{LOG_COMMAND}`. {NOTHING_CHANGED}"
        ),
    };
}

/// What:
///  The remedy sentence for a platform.
/// Why:
///   Windows and the BSD systems have no activation yet; Linux without user services can turn them on.
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

/// What:
///  The notice for a commit that waits behind others.
/// Why:
///   The position is the one fact the person needs: how many commits go first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positionNotice(number, of)
/// ```
pub fn position_notice(number: usize, of: usize) -> String {
    let ahead: usize = number.saturating_sub(1);
    let _ = of;
    return match ahead {
        0 => String::from("This commit is next."),
        1 => String::from("Waiting for 1 earlier commit."),
        many => format!("Waiting for {many} earlier commits."),
    };
}

/// What:
///  The notice for a backup push that failed after its commit was saved.
/// Why:
///   The push runs after the commit returned, so the person learns of a failure at their next commit. The service has
///  its own environment, which is the first thing to check when a push that works in a terminal fails here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushFailedNotice(commit, said)
/// ```
pub fn push_failed_notice(commit: &str, said: &str) -> String {
    let short: &str = commit.get(..7).unwrap_or(commit);
    let detail: String = if said.is_empty() { String::from("Git gave no reason.") } else { format!("Git said: {said}") };
    return format!(
        "The backup push of your commit {short} failed, so it exists only on this computer for now. {detail} The commit \
         service runs with its own settings, not your terminal's, so a push that works in your terminal can fail here \
         when it needs a variable your terminal has. Run `git push` when ready."
    );
}

/// The texts that must never reach a user: implementation terms the system should have resolved itself.
///  Matched case-insensitively against whole words after the quoted commands are removed.
pub const DENYLIST: [&str; 22] = [
    "protocol", "journal", "request id", "daemon", "socket", "errno", "epipe", "econnrefused", "eagain", "flock",
    "lock file", "peer", "frame", "handshake", "batch", "oid", "fast-import", "unix domain", "abstract", "pid",
    "uid", "os error",
];

/// The repository path a failure names, as text.
pub fn path_text(path: &Path) -> String {
    return path.display().to_string();
}

/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_tests.rs"]
mod tests;
