//! What:
//!  What one reconcile run integrates,
//! the local side the push sent,
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
//!  Every name a message or a line carries is the name of the branch that was pushed,
//! never the name of the branch a worktree happened to have checked out:
//! a typed push of an absent branch used to publish a merge labelled with the checked-out branch,
//! a wrong statement in a published history that cannot be corrected without rewriting it
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "A typed push of a branch that was not checked out published a merge labelled with the checked-out branch").
//!  The branch the merge is into is named the way Git's own message names it:
//! the branch whose tip is the merge's first parent,
//! which is the branch the push sent, and
//! the ref the merge is published to when the push sent no branch (a detached `HEAD`, a commit expression),
//! because nothing local is merged into then.
//! A branch whose upstream has another name (`branch.feature.merge = refs/heads/main`) is named by the same rule,
//! so naming the tracked ref would name a branch nobody merged into.
//!  The merge is published to the ref the rejected push updated,
//! which the branch's name does not always spell.
//! That ref is not always the branch's upstream
//! (an explicit `feature:main` push sends a branch that tracks `origin/feature`),
//! so the line says where the merge went and nothing about what the branch tracks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Reconcile = { destination: Destination; local: string; pushed: PushedSide; label: string; explicit: boolean };
//! ```

/// Where the plain push goes,
/// as `auto_push_destination` decides it.
use crate::auto_push_destination::Destination;
/// The push's own status and text,
/// which a reconcile run answers with.
use crate::auto_push_single_flight::PushAttempt;

/// What:
///  The local side of the push whose merge the reconcile publishes.
/// Why:
///   A refspec names a ref or an expression (`HEAD`,
/// `feat/topic~1`),
/// and only a branch may follow the published merge.
///  The reconcile used to learn this side from the command's checkout,
/// so a push of `feat/topic` typed while `main` was checked out was represented by `main`:
/// the merge's subject, the report and the fast-forward all named a branch nobody pushed,
/// and a checkout whose tip happened to equal the pushed tip was moved over a merge of another branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushedSide = { kind: 'branch'; ref: string } | { kind: 'expression'; name: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PushedSide<'a> {
    /// The local branch the push sent, as `refs/heads/<name>`:
    /// the only branch the published merge may advance.
    Branch(&'a str),
    /// The local side as Git reported it when it names no branch:
    /// a detached `HEAD`, or a commit expression.
    Expression(&'a str),
}

impl PushedSide<'_> {
    /// What:
    ///  The branch the published merge may advance,
    /// or nothing when the push sent no branch.
    /// Why:
    ///   The fast-forward of a reconciliation moves only this branch,
    /// and a branch is advanced only from the tip the push sent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const advanceable = (pushed) => pushed.kind === 'branch' ? pushed.ref : undefined;
    /// ```
    pub fn branch(&self) -> Option<&str> {
        return match self {
            Self::Branch(reference) => Some(reference),
            Self::Expression(_) => None,
        };
    }

    /// What:
    ///  The local side's short name,
    /// which the merge's subject and the report name.
    /// Why:
    ///   The subject of a merge names the branch whose tip is its first parent,
    /// the way `git merge` names the branch it is on;
    /// an expression is shown as Git printed it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const source = (pushed) => (pushed.kind === 'branch' ? pushed.ref : pushed.name).replace(/^refs\/heads\//, '');
    /// ```
    pub fn source(&self) -> &str {
        return match self {
            Self::Branch(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference),
            Self::Expression(name) => name,
        };
    }
}

/// What:
///  What one reconcile run integrates,
/// where,
/// and how it words and limits itself.
/// Why:
///   Auto-push and an explicit push share the loop;
/// they differ in the line prefix they print,
/// in whether the push a person typed has already shown its own output,
/// and in what the merge commit says happened.
///   An explicit push already printed Git's own refusal,
/// so a remote that cannot be fetched,
/// a remote that did not
///  diverge,
/// and a failure that is not a stale update are left exactly as the push reported them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reconcile = { destination: Destination; local: string; pushed: PushedSide; label: string; explicit: boolean };
/// ```
pub struct Reconcile<'a> {
    /// The remote and the ref the rejected push updated.
    pub destination: Destination,
    /// The commit the rejected push sent.
    pub local: String,
    /// The local side the push sent:
    /// the only branch the merge may advance, and the name every line carries.
    pub pushed: PushedSide<'a>,
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
///  The words a reconcile run prints and the names of its merge commit,
/// from the plan alone.
/// Why:
///   The loop publishes one merge and words it once,
/// so the prefixes,
/// the names and the journal identifier are read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { linePrefix, failedPrefix, short, source, mergedInto } = wordingOf(plan);
/// ```
pub(crate) struct Wording {
    /// What every line the run adds begins with.
    pub(crate) line_prefix: String,
    /// What a line saying nothing was published begins with.
    pub(crate) failed_prefix: String,
    /// The merge's destination ref without `refs/heads/`.
    pub(crate) short: String,
    /// The push's local side,
    /// short: the branch that was pushed, or the expression Git reported.
    pub(crate) source: String,
    /// The branch the merge is into:
    /// the pushed branch, or the destination when the push named no branch.
    pub(crate) merged_into: String,
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
    let source: String = plan.pushed.source().to_owned();
    // The merge's first parent is the pushed tip, so the branch it is into is the branch that was pushed; a push of an
    //  expression has no local branch to name, and the ref it was published to is what it is into.
    let merged_into: String = match plan.pushed.branch() {
        Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference).to_owned(),
        None => short.clone(),
    };
    return Wording { line_prefix, failed_prefix, short, source, merged_into };
}

/// What:
///  The message of the merge commit,
/// from the plan and the remote tip it merges in.
/// Why:
///   A reader of the branch's history a year later meets this message without the run around it,
/// so it says which branch was merged with what,
/// why the commit exists,
/// which two tips it joins,
/// and how a person who did not want it undoes it.
///  Only the explicit push says the remote had commits the local side did not:
/// the automatic push is made because the remote advanced after the commit landed,
/// and no commit of the person's is involved there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeMessage(plan, remoteTip): string;
/// ```
pub(crate) fn merge_message(plan: &Reconcile<'_>, remote_tip: &str) -> String {
    let remote: &str = plan.destination.remote.as_str();
    let short: String = plan
        .destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&plan.destination.merge_ref)
        .to_owned();
    let source: &str = plan.pushed.source();
    let merged_into: String = match plan.pushed.branch() {
        Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference).to_owned(),
        None => short.clone(),
    };
    let local: &str = plan.local.as_str();
    let because: String = if plan.explicit {
        format!("{remote}/{short} had commits {source} did not have.")
    } else {
        format!("{remote}/{short} advanced after the commit landed.")
    };
    let undo: String = match plan.pushed.branch() {
        Some(_) => format!(
            "To undo it, `git revert -m 1 <commit>` on {merged_into} keeps both sides' commits in the history and removes \
             its effect; `git reset --hard {local}` on {merged_into} drops it from the local branch alone."
        ),
        None => String::from(
            "To undo it, `git revert -m 1 <commit>` on it keeps both sides' commits in the history and removes its effect.",
        ),
    };
    let published: String = if merged_into == short {
        String::new()
    } else {
        format!("\nPublished to {remote}/{short}.")
    };
    return format!(
        "Merge {remote}/{short} into {merged_into}\n\n\
         Created by monochromatic-git-policy-cli {label} because {because}\n\
         The first parent is {source} at {local} and the second {remote}/{short} at {remote_tip}; nothing was forced.\n\
         {undo}{published}",
        label = plan.label,
    );
}
