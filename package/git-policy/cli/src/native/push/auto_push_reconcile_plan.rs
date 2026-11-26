//! What:
//!  What one reconcile run integrates,
//! where it publishes the merge,
//! how it words every line it prints,
//! and how it ended.
//! Why:
//!  The loop that does the work is awaited (`auto_push_reconcile_awaiting.rs`),
//! and the entry a command outside the runtime calls bridges to that one loop
//! (`auto_push_reconcile.rs`);
//! both read the plan and the words from here,
//! so neither can describe a merge the other does not make,
//! and the two files that use them need not depend on each other.
//!  The branch the merge is into is named the way Git's own message names it:
//! the branch whose tip is the merge's first parent,
//! and the tracked ref's short name for a detached HEAD,
//! which has no branch to name.
//! A branch whose upstream has another name (`branch.feature.merge = refs/heads/main`) is merged into by the same rule,
//! so naming the tracked ref would name a branch nobody merged into.
//!  The merge is published to the ref the rejected push updated,
//! which the branch's name does not always spell.
//! That ref is not always the branch's upstream
//! (an explicit `feature:main` push sends a branch that tracks `origin/feature`),
//! so the line says where the merge went and nothing about what the branch tracks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Reconcile = { destination: Destination; local: string; branchRef?: string; label: string; explicit: boolean };
//! ```

/// Where the plain push goes,
/// as `auto_push_destination` decides it.
use crate::auto_push_destination::Destination;
/// The push's own status and text,
/// which a reconcile run answers with,
/// and the failures a run can stop on.
use crate::auto_push_single_flight::{
    PushAttempt,
    SingleFlightError,
};

/// What:
///  What one reconcile run integrates,
/// where,
/// and how it words and limits itself.
///  `Option<&str>` is "the branch,
/// or nothing for a detached HEAD".
/// Why:
///   Auto-push and an explicit push share the loop;
/// they differ in the line prefix they print,
/// in whether the branch
///  follows the published merge,
/// and in how much of a failure they keep quiet about.
///   An explicit push already printed Git's own refusal,
/// so a remote that cannot be fetched,
/// a remote that did not
///  diverge,
/// and a failure that is not a stale update are left exactly as the push reported them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reconcile = { destination: Destination; local: string; branchRef?: string; label: string; explicit: boolean };
/// ```
pub struct Reconcile<'a> {
    /// The remote and the ref the rejected push updated.
    pub destination: Destination,
    /// The commit the rejected push sent.
    pub local: String,
    /// The branch that follows the published merge when it can,
    /// or nothing for a detached HEAD.
    pub branch_ref: Option<&'a str>,
    /// The word after `monochromatic-git-policy-cli:` in every line the run adds.
    pub label: &'a str,
    /// Whether the rejected push was typed by a person and has already shown its own output.
    pub explicit: bool,
}

/// How a reconcile run ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Integrated = { kind: 'not-divergent'; attempt } | { kind: 'finished'; attempt };
/// ```
pub enum Integrated {
    /// The remote did not diverge,
    /// the run could not tell,
    /// or the run could not run its merge push at all;
    /// the rejected attempt stands as it was.
    NotDivergent(PushAttempt),
    /// The run published the merge,
    /// or says why it did not.
    Finished(PushAttempt),
}

/// What:
///  The words a reconcile run prints and the message of its merge commit,
/// from the plan alone.
/// Why:
///   The loop publishes one merge and words it once,
/// so the prefixes,
/// the names and the commit message are read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { linePrefix, failedPrefix, short, mergedInto, message } = wordingOf(plan);
/// ```
pub(crate) struct Wording {
    /// What every line the run adds begins with.
    pub(crate) line_prefix: String,
    /// What a line saying nothing was published begins with.
    pub(crate) failed_prefix: String,
    /// The merge's destination ref without `refs/heads/`.
    pub(crate) short: String,
    /// The branch the merge is into.
    pub(crate) merged_into: String,
    /// The message of the merge commit.
    pub(crate) message: String,
}

/// What:
///  The words of one run.
/// Why:
///   The run reads them here,
/// so a merge cannot be worded two ways.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordingOf(plan: Reconcile): Wording;
/// ```
pub(crate) fn wording_of(plan: &Reconcile<'_>) -> Wording {
    let label: &str = plan.label;
    let line_prefix: String = crate::auto_push_reconcile_basics_and_limits::run_line_prefix(label);
    let failed_prefix: String = format!("{line_prefix}not published: ");
    let short: String = plan
        .destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&plan.destination.merge_ref)
        .to_owned();
    let merged_into: String = match plan.branch_ref {
        Some(reference) => reference
            .strip_prefix("refs/heads/")
            .unwrap_or(reference)
            .to_owned(),
        None => short.clone(),
    };
    let published: String = if merged_into == short {
        String::new()
    } else {
        format!(
            "\nPublished to {remote}/{short}.",
            remote = plan.destination.remote
        )
    };
    let message: String = format!(
        "Merge {remote}/{short} into {merged_into}\n\nCreated by monochromatic-git-policy-cli {label} because {remote}/{short} advanced after the commit landed.{published}",
        remote = plan.destination.remote
    );
    return Wording {
        line_prefix,
        failed_prefix,
        short,
        merged_into,
        message,
    };
}

/// What:
///  The plan of one automatic push's reconcile,
///  built from the destination its branch was read to have.
/// Why:
///   Both faces enter the loop with the auto-push's label and with `explicit` false,
///  and the commit they merge is the one the rejected push was allowed to send;
///  building it here is what keeps those decisions from being written on each face,
///  where a change to one would leave the other behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const autoPushPlan = (destination, local, branchRef): Reconcile => ({ destination, local, branchRef, label: 'auto-push', explicit: false });
/// ```
pub fn auto_push_plan<'a>(
    destination: Destination,
    local: &'a str,
    branch_ref: &'a str,
) -> Reconcile<'a> {
    // What:     `Reconcile { .. }` builds the plan record; `local.to_owned()` copies the caller's borrowed text into the owned
    //           `String` the plan keeps, and `Some(branch_ref)` keeps the branch the merge is into.
    // Why:      The run outlives the caller's borrow of the branch's name for `branch_ref` only while the caller's frame lives,
    //           which is the length of the run, and the plan needs no copy of a name it only reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { destination, local: local.slice(), branchRef: branchRef, label: 'auto-push', explicit: false };
    // ```
    return Reconcile {
        destination,
        local: local.to_owned(),
        branch_ref: Some(branch_ref),
        label: "auto-push",
        explicit: false,
    };
}

/// What:
///  The attempt a reconcile run answers the rejected push with.
/// Why:
///   Both faces finish the same way:
///  a run that did not diverge and one that finished both leave one attempt,
///  and a run a command stopped hands its own failure on;
///  reading that here means a face cannot turn one of those outcomes into another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attemptOf = (integrated) => integrated.kind === 'error' ? integrated : integrated.attempt;
/// ```
pub fn attempt_of(
    integrated: Result<Integrated, SingleFlightError>
) -> Result<PushAttempt, SingleFlightError> {
    // What:     `match` opens the run's answer: `Ok(..)` holds an `Integrated`, and `Err(stopped)` the failure that ended it.
    // Why:      The two kinds of `Integrated` each carry the attempt the push is answered with, and nothing else may be reported.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (integrated.kind === 'error') return integrated;
    // return integrated.attempt;
    // ```
    return match integrated {
        // What:     The alternation binds either kind of finished run to one name, `attempt`.
        // Why:      Both kinds answer with the attempt they carry, and no caller tells them apart afterwards.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return integrated.attempt;
        // ```
        Ok(Integrated::NotDivergent(attempt) | Integrated::Finished(attempt)) => Ok(attempt),
        // What:     `Err(stopped)` is the failure a Git step of the run reported, handed on unchanged.
        // Why:      The words of a stopped run are its own, and a face must not replace them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return integrated;
        // ```
        Err(stopped) => Err(stopped),
    };
}
