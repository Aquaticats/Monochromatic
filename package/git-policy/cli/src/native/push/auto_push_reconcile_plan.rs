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
/// which a reconcile run answers with.
use crate::auto_push_single_flight::PushAttempt;

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
    /// or the run could not tell;
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
    let line_prefix: String = format!("monochromatic-git-policy-cli: {label}: ");
    let failed_prefix: String = format!("{line_prefix}not published: ");
    let short: String = plan
        .destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&plan.destination.merge_ref)
        .to_owned();
    let merged_into: String = match plan.branch_ref {
        Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference).to_owned(),
        None => short.clone(),
    };
    let published: String = if merged_into == short {
        String::new()
    } else {
        format!("\nPublished to {remote}/{short}.", remote = plan.destination.remote)
    };
    let message: String = format!(
        "Merge {remote}/{short} into {merged_into}\n\nCreated by monochromatic-git-policy-cli {label} because {remote}/{short} advanced after the commit landed.{published}",
        remote = plan.destination.remote
    );
    return Wording { line_prefix, failed_prefix, short, merged_into, message };
}
