//! What:
//!  Hands a push to the commit service after a command other than `git commit` added commits to the branch:
//! a merge,
//! a
//!  pull's merge,
//! a cherry-pick,
//! a revert or an applied patch.
//! Why:
//!  A clean `git merge` used to stay local until the next commit pushed it.
//! The owner chose (2026-11-02) that any command
//!  adding commits queues the push as a commit does,
//! when the branch tip moved and the branch is now ahead of its upstream
//!  and not diverged from it,
//! and that a rebase does not,
//! because a rebased branch would need a forced push.
//!  The check reads the branch,
//! its upstream tracking ref and the commit parents from the files,
//! with no Git process:
//!  the tip is read before and after the forwarded command,
//! and the push is handed over only when the old tip and the
//!  upstream tip are both ancestors of the new tip.
//! The old tip being an ancestor is what leaves a rebase (a rewritten
//!  branch) out,
//! `pull --rebase` and `pull.rebase` included,
//! without reading the configuration for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchTip(worktree); await runGit(args); if (exitCode === 0) await watch.handOverIfAhead();
//! ```

#![cfg(unix)]

/// Publish the checks this file's hand-over decides with,
/// under the names a caller already reads them by.
pub use super::auto_push_trigger_ahead_of_upstream::{
    WALK_BUDGET,
    reaches,
    tip_to_push,
    tip_to_push_or_why,
};
/// Publish the watch a caller starts before its command runs,
/// under the names it already reads them by.
pub use super::auto_push_trigger_watched_branch::{
    Declined,
    TipWatch,
    try_watch_tip,
    watch_tip,
};
use crate::push_record_of_a_moved_branch::{
    hand_over_push,
    write_push_record,
};
/// Import the push record's shape,
/// how a hand-over ended,
/// and the log it is written to.
use crate::push_trigger_log::{
    Decision,
    Outcome,
    record_decision,
};
/// Import the object store the controls' own walk reads with.
#[cfg(test)]
use crate::repo_read_odb::ObjectDatabase;
/// Import the client a hand-over is made with,
/// and the hand-over of the commit service,
/// which writes the record and wakes the service.
use crate::service_client::{
    Activation,
    ClientConfig,
    manager_watch_for,
};
use crate::service_handoff::ServiceHandoff;
/// Import the words a person reads when the service cannot be reached.
use crate::service_messages::platform_remedy;
use crate::service_messages::render;
/// Import the silence a heartbeat may leave,
/// the words a platform without a manager is given,
/// and the route to the service.
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::platform_without_manager;
use crate::service_route::{
    Route,
    route_from_environment,
};
/// Import the bound the hand-over's greeting is waited under.
use crate::wait_ceilings::HAND_OVER_GREETING_WAIT;
/// Import the environment and path types the controls write their own calls in.
#[cfg(test)]
use std::ffi::OsString;
/// Import the stream the words a person reads are written to.
use std::io::Write;
#[cfg(test)]
use std::path::{
    Path,
    PathBuf,
};

/// Record how a hand-over ended,
/// with the reason,
/// for the watched branch and the command that ran.
fn record_ending(
    watch: &TipWatch,
    command: &str,
    outcome: Outcome,
    reason: &str,
    detail: &str,
) {
    record_decision(&Decision {
        common_dir: Some(watch.common_dir()),
        origin: Some(watch.origin()),
        command,
        outcome,
        reason,
        detail,
        branch: Some(watch.branch_ref()),
    });
}

/// What:
///  After a command that succeeded:
/// hand the push over when the branch grew and is ahead of its upstream.
/// Why:
///   The person's command already finished;
/// a service that cannot be reached is said plainly and changes nothing.
///   The record is the same push record a commit leaves,
/// so the service pushes the branch by name and coalesces it.
///   Every ending is recorded with its reason (`push_trigger_log.rs`),
/// so a push that did not happen can be explained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await watch.handOverIfAhead(command, stderr);
/// ```
pub fn hand_over_if_ahead(
    watch: &TipWatch,
    command: &str,
    stderr: &mut dyn Write,
) {
    let branch: &str = watch.branch_ref();
    let tip: String = match tip_to_push_or_why(watch) {
        Ok(tip) => tip,
        Err(Declined {
            reason,
            detail,
        }) => {
            record_ending(watch, command, Outcome::Declined, reason, detail.as_str());
            return;
        }
    };
    // The record is written before the route is read: a context with no route (a non-systemd session, a container, another
    //  `XDG_RUNTIME_DIR`) then leaves the push on disk for the next wake instead of losing it with the message.
    if let Err(failure) = write_push_record(
        watch.layout().worktree_root.as_path(),
        watch.layout().git_dir.as_path(),
        &crate::ref_name::RefName::from_text(branch),
        tip.as_str(),
        watch.tip_before(),
    ) {
        record_ending(
            watch,
            command,
            Outcome::Declined,
            "handover-failed",
            render(&failure).as_str(),
        );
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&failure));
        return;
    }
    let Some(Route {
        address,
        provision,
    }) = route_from_environment(watch.real_git(), watch.layout().git_dir.as_path())
    else {
        record_ending(
            watch,
            command,
            Outcome::Declined,
            "no-route",
            "no way to reach the commit service",
        );
        // Terminal output error: the message is the only report of a push that waits for a service.
        // This is a merge or a pull that already happened, so the words say what became of its push, never that a commit was not made.
        let _written = writeln!(
            stderr,
            "monochromatic-git-policy-cli: {branch} was queued to be pushed, and no way to reach the commit service was found now. {} The push stays queued: the next command that reaches the service pushes it.",
            platform_remedy(platform_without_manager())
        );
        return;
    };
    let handoff: ServiceHandoff = ServiceHandoff {
        config: ClientConfig {
            address,
            activation: Activation::Managed,
            // The push record is durable, so the wake waits the hand-over's own greeting bound.
            start_wait: HAND_OVER_GREETING_WAIT,
            heartbeat_silence: HEARTBEAT_SILENCE,
            start_watch: provision.as_ref().map(manager_watch_for),
            keep_fresh: None,
            answer_ceiling: crate::wait_ceilings::ANSWER_CEILING,
            withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING,
            extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT,
            start_ceiling: HAND_OVER_GREETING_WAIT,
            waiting_notice_after: crate::wait_ceilings::waiting_notice_after(),
        },
        repository: watch.layout().worktree_root.clone(),
        git_dir: watch.layout().git_dir.clone(),
        reference: crate::ref_name::RefName::from_text(watch.branch_ref()),
        parent: watch.tip_before().to_owned(),
    };
    if let Err(failure) = hand_over_push(&handoff, tip.as_str(), stderr) {
        record_ending(
            watch,
            command,
            Outcome::Declined,
            "handover-failed",
            render(&failure).as_str(),
        );
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&failure));
        return;
    }
    record_ending(
        watch,
        command,
        Outcome::HandedOver,
        "ahead-of-upstream",
        format!("pushed {tip} of {branch}").as_str(),
    );
}

/// The trigger's controls stay out of the release executable.
#[cfg(test)]
#[path = "auto_push_trigger_tests.rs"]
mod tests;
