//! What:
//!  The text of every failure the wrapper tells the user when it cannot get a commit landed by the resident service,
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
//! The failures themselves and the facts their text names are declared in
//! `service_messages_the_failures_a_person_can_meet.rs`,
//! and the commands and words a failure gives a person in
//! `service_messages_what_a_failure_gives_a_person.rs`,
//! so a test can check each rendering against a list of terms that must never reach a user,
//! and so the text a client prints cannot drift from the text the tests read.
//! Each of them is read from here,
//! so no caller names the file a name moved to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessages.ts
//! export const render = (failure: ClientFailure): string => `The commit service ...`;
//! ```

/// The notices a wait prints are declared beside this file
/// (`service_messages_of_a_wait_for_the_service.rs`),
/// because the rendering of every failure is at its line bound,
/// and the call sites read them from here.
pub use crate::service_messages_of_a_wait_for_the_service::{
    STARTING_NOTICE,
    UPDATING_NOTICE,
    position_notice,
    rebuild_notice,
    replacing_notice,
    restarting_notice,
    silent_notice,
    starting_notice,
    unplaced_notice,
};
/// The sentences about what the service last said it waited for are read by the failures below and by the two of an interrupt.
pub(crate) use crate::service_messages_of_a_wait_for_the_service::{
    it_had_said,
    waiting_for,
};

/// The name of the link the commit service's unit starts its program from,
/// which the failure for another release names when that link is what could not be pointed
/// (`service_instance.rs`,
/// `PROGRAM_LINK`).
pub use crate::service_instance::PROGRAM_LINK;

/// Every failure the client can meet,
/// and the facts its text names,
/// are declared beside this file
/// (`service_messages_the_failures_a_person_can_meet.rs`),
/// because the rendering of every failure is at its line bound.
pub use crate::service_messages_the_failures_a_person_can_meet::{
    ClientFailure,
    ManagerLastWords,
    Platform,
    RequestedStart,
    WhatTheBranchShows,
};

/// The commands and words a failure gives a person are declared beside this file
/// (`service_messages_what_a_failure_gives_a_person.rs`),
/// because the rendering of every failure is at its line bound.
pub use crate::service_messages_what_a_failure_gives_a_person::{
    CHECK_COMMANDS,
    DENYLIST,
    LINGER_COMMAND,
    LOG_COMMAND,
    NOTHING_CHANGED,
    NOTHING_CHANGED_AND_THE_STAGED_WORK,
    RESTART_COMMAND,
    RESTART_ONE_UNIT_COMMAND,
    STATUS_COMMAND,
    STOP_COMMAND,
    path_text,
};

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
        ClientFailure::DidNotStart {
            unit,
            answer,
            asked,
        } => format!(
            "The commit service did not start, so this commit was not made: {}. Its log shows why: `{LOG_COMMAND}`. \
             Run the commit again after fixing that. {NOTHING_CHANGED}",
            crate::service_messages_of_a_start_that_was_asked_and_did_not_come::did_not_start(
                unit.as_deref(),
                answer,
                *asked
            )
        ),
        ClientFailure::NothingAnswered {
            address,
            waited,
        } => crate::service_messages_of_a_service_that_stopped::nothing_answered(
            address.as_str(),
            *waited,
        ),
        ClientFailure::NoActivation(platform) => format!(
            "This computer cannot start the commit service on demand, so this commit was not made. {} {NOTHING_CHANGED}",
            platform_remedy(*platform)
        ),
        ClientFailure::ManagerDidNotAnswer {
            waited,
        } => crate::service_messages_of_a_manager_that_did_not_answer::did_not_answer(*waited),
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
            "The commit service that is running was started from another release of this git wrapper, and it did not step \
             aside when it was asked to retire nor when this worktree's unit was restarted, so this commit was not made. Run the \
             commit again: a service that was only still finishing the work it held is replaced by the next commit, which stops this \
             worktree's unit and starts the program the units install. If this message comes back, the program those units start \
             is not the release you are running: the log of the command that set the service up names the link \
             `{PROGRAM_LINK}` it could not point at this release, and a runtime directory that cannot hold that link keeps \
             every commit here from being made, so set `XDG_RUNTIME_DIR` to a runtime directory this account can write. A \
             service that still will not stop can be restarted with `{RESTART_COMMAND}`. {NOTHING_CHANGED}"
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
        ClientFailure::BranchKeepsChanging {
            branch,
            times,
        } => format!(
            "The branch {branch} changed {times} times while this commit was being prepared, so it was not made. Run \
             the commit again; if other people commit to the same branch at the same moment, it will succeed once they \
             pause. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchMoved {
            branch,
        } => format!(
            "The branch {branch} moved while this commit was being prepared, and a commit that replaces or rewrites another \
             commit, or one that was written or signed by hand, cannot be moved onto a different last commit, so it was not \
             made. Run the commit again to make it on the branch as it is now. {NOTHING_CHANGED}"
        ),
        ClientFailure::ConclusionStateChanged {
            branch,
        } => format!(
            "The merge, cherry-pick or revert this commit was concluding changed while it was being prepared, so it was \
             not made on the branch {branch}. Another command either finished that operation or abandoned it. Run `git \
             status` to see what the worktree holds now, then run the commit again. {NOTHING_CHANGED}"
        ),
        ClientFailure::BranchMovedMessageSaved {
            branch,
            file,
            amend,
        } => format!(
            "The branch {branch} moved while this commit was being prepared, so it was not made. The message you wrote is saved \
             in {file}. Run `git commit {}-F {file}` to make the commit on the branch as it is now. {NOTHING_CHANGED}",
            if *amend { "--amend " } else { "" }
        ),
        ClientFailure::BranchSwitched {
            branch,
            now,
            file,
        } => format!(
            "The worktree switched from {branch} to {now} while this commit was being prepared, so it was not made. {} Switch back \
             to {branch} and run the commit again, or run it here to commit on {now}. {NOTHING_CHANGED}",
            match file {
                Some(saved) => format!(
                    "The message you wrote is saved in {saved}; `git commit -F {saved}` uses it."
                ),
                None => String::new(),
            }
        ),
        ClientFailure::ServiceSilent => crate::service_messages_of_a_service_that_stopped::silent(),
        ClientFailure::ServiceNotReading {
            took_nothing,
        } => crate::service_messages_of_a_service_that_stopped::not_reading(*took_nothing),
        ClientFailure::ServiceSilentAfterPreparing {
            shows,
        } => crate::service_messages_of_a_service_that_stopped::silent_after_preparing(*shows),
        ClientFailure::StillWithService {
            minutes,
            blocker,
        } => format!(
            "This commit waited {minutes} minutes for the commit service and was withdrawn, so it will not land later by itself.{} \
             Your staged changes and your worktree are as they were. Run the commit again; if the service is stuck, look at why with \
             `{LOG_COMMAND}` and restart it with `{RESTART_COMMAND}`. {NOTHING_CHANGED}",
            waiting_for(blocker.as_deref())
        ),
        ClientFailure::WithdrawalUnconfirmed {
            blocker,
        } => format!(
            "This commit waited for the commit service until the longest wait the wrapper allows, and the service did not confirm \
             that it withdrew it, so it is not known whether the commit was made.{} Run `{}` to see whether it was made, and `{}` to \
             see what is still staged. If it was not made, run the commit again; if the service is stuck, look at why with \
             `{LOG_COMMAND}` and restart it with `{RESTART_COMMAND}`.",
            it_had_said(blocker.as_deref()),
            CHECK_COMMANDS[0],
            CHECK_COMMANDS[1]
        ),
        // The two ends of an interrupted commit are worded beside this file, because this one is at its line bound.
        ClientFailure::Interrupted {
            signal,
            blocker,
            ..
        } => crate::service_messages_of_an_interrupted_commit::interrupted(
            *signal,
            blocker.as_deref(),
        ),
        ClientFailure::InterruptedNotKnown {
            signal,
            blocker,
            ..
        } => {
            crate::service_messages_of_an_interrupted_commit::not_known(*signal, blocker.as_deref())
        }
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
        ClientFailure::ReferenceLocked {
            branch,
            words,
        } => format!(
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
        ClientFailure::SocketPathTooLong {
            path,
            length,
            longest,
            longest_runtime,
        } => crate::service_messages_of_a_start_that_cannot_come::socket_path_too_long(
            path,
            *length,
            *longest,
            *longest_runtime,
        ),
        ClientFailure::ServiceElsewhere {
            made,
            looked_for,
            manager_runtime,
            client_runtime,
        } => crate::service_messages_of_a_start_that_cannot_come::service_elsewhere(
            made,
            looked_for,
            manager_runtime,
            client_runtime,
        ),
        ClientFailure::Unavailable => format!(
            "The commit service could not finish this commit, so it was not made. Run the commit again; if it fails \
             again, look at the service's log with `{LOG_COMMAND}`. {NOTHING_CHANGED}"
        ),
    };
}

/// The remedy sentence for a platform is declared beside this file
/// (`service_messages_of_a_platform.rs`),
/// because the rendering of every failure is at its line bound.
pub use crate::service_messages_of_a_platform::platform_remedy;

/// The word a failure goes by in the timing record is declared beside this file
/// (`service_messages_of_a_failure.rs`),
/// because the rendering of every failure is at its line bound.
pub use crate::service_messages_of_a_failure::failure_code;

/// Message controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_tests.rs"]
mod tests;
