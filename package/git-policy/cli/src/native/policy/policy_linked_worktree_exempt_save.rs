//! What:
//!  The save-first guard of a repository the allowlist exempts from the worktree rule:
//!  the work a guarded form would discard is copied beside the audit log,
//!  and the command then runs.
//! Why:
//!  The exemption was decided for the worktree rule
//!  (the owner's answer of 2026-11-11,
//!  `#3024`:
//!  a vendored clone under the Cargo build directory must not be refused),
//!  and it never decided that the rule "never discard user data" stops holding there.
//!  A refusal would fail a build that resets or cleans its own clone,
//!  which is the very case the exemption exists for,
//!  so the form runs with its save instead:
//!  the cost is the listing and the bytes of what is at risk,
//!  and a worktree holding nothing this form would lose costs one question
//!  (`policy_linked_worktree`).
//!  The copies are the ones the conflicted-file and abort carve-outs write,
//!  into the same folder and with the same bound,
//!  because a person reading one notice must not have to learn two places.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = saveTheExemptDiscard(facts, { command, identity, question, tree, limits, milliseconds });
//! // { kind: 'saved', notice, copies } | { kind: 'nothing' } | { kind: 'stopped', text }
//! ```

/// The bound on the folder of saved copies,
///  the pass that applies it after a save,
///  and the saved copies the record names.
use crate::conflict_side_guard::prune_after_save;
use crate::conflict_sides::{SavedCopy, save_copies};
/// The age and size limits that pass applies.
use crate::discarded_prune::Limits;
/// The named facts a record of an exemption carries.
use crate::discard_audit_facts::AuditFact;
/// The record of every decision.
use crate::discard_audit::{AuditOutcome, record_audit_with_facts};
/// The exempt directory the command's own environment named.
use crate::effective_target::{ToolCacheDirectory, matching_allowed_worktree_dir_for_repository};
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// Where the command runs.
use crate::repository_location::RepositoryLocation;
/// The facts every question is asked.
use crate::repository_facts::RepositoryFacts;
/// The one place a form is let through.
use crate::policy_linked_worktree_nested_repositories::{AllowedForm, allow_a_discarding_form};
use crate::policy_checks::rejected;
/// Which question the form's measurement asked,
///  because it says what the form would lose,
///  and the code a stopped command is rejected with.
use crate::rule_linked_worktree::{
    DirtyQuestion, GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, TargetTree,
};
/// The scope of a worktree status,
///  whose two answers name tracked changes and untracked files apart.
use crate::rule_worktree_dirty::DirtyScope;
/// The notices of a save the exemption made,
///  and the sentence that says which variable exempted the repository.
use crate::rule_linked_worktree_messages::{
    exempt_saved_notice, exempt_save_stopped_message, exemption_notice,
};
/// The directory the copies are read from,
///  and the directory they are written below.
use crate::worktree_identity::WorktreeIdentity;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The form the exemption is about to run,
///  and everything the copy of what it would lose needs.
///  A `struct` is a record with named fields.
/// Why:
///   The three callers (a form measured in a worktree that holds changes,
///  a form measured clean that still overwrites an ignored file,
///  and the tokens it must already have read) differ in one field,
///  so one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExemptDiscard = { command; identity; question?; tree?; ignoredPaths?; limits; milliseconds };
/// ```
pub(crate) struct ExemptDiscard<'a> {
    /// The guarded form that is about to run.
    pub(crate) command: GuardedCommand,
    /// The repository,
    ///  whose worktree the copies are read from and whose common directory they are written below.
    pub(crate) identity: &'a WorktreeIdentity,
    /// The question the measurement asked,
    ///  or nothing when the form was measured clean and still writes a tree.
    pub(crate) question: Option<DirtyQuestion>,
    /// The tree the form writes,
    ///  whose tracked paths an ignored file is overwritten at.
    pub(crate) tree: Option<&'a TargetTree>,
    /// The ignored files that tree also tracks,
    ///  when the caller already read them,
    ///  so one question is not asked twice.
    pub(crate) ignored_paths: Option<Vec<String>>,
    /// The age and size limits of the folder of saved copies.
    pub(crate) limits: Limits,
    /// The clock reading the copies are named with.
    pub(crate) milliseconds: i64,
}

/// What the exemption does with one form.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Three answers need three effects:
///  nothing was at risk and the form runs,
///  the work is on disk and the form runs with a notice,
///  and a copy that could not be written stops the form with the worktree as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExemptVerdict = { kind: 'nothing' } | { kind: 'saved'; notice; copies } | { kind: 'stopped'; text };
/// ```
pub(crate) enum ExemptVerdict {
    /// The repository holds nothing this form would lose:
    ///  the command runs as Git runs it.
    Nothing,
    /// What the form would lose is copied beside the audit log:
    ///  the command runs,
    ///  and the notice names each copy.
    Saved {
        /// What the person reads before Git runs.
        notice: String,
        /// What was saved,
        ///  for the audit record.
        copies: Vec<SavedCopy>,
    },
    /// A copy could not be written:
    ///  the command stops with this text,
    ///  because it would discard unsaved work.
    Stopped(String),
}

/// What:
///  The worktree root and the common directory of `identity`,
///  or nothing for an identity with no worktree.
/// Why:
///  A copy is written below the common Git directory and read from the worktree,
///  and a target the allowlist exempts is always one of the two worktree kinds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = (identity) => 'worktreeRoot' in identity ? [identity.worktreeRoot, identity.commonDir] : undefined;
/// ```
fn places(identity: &WorktreeIdentity) -> Option<(&Path, &Path)> {
    return match identity {
        WorktreeIdentity::MainWorktree { common_dir, worktree_root, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, worktree_root, .. } => {
            Some((worktree_root.as_path(), common_dir.as_path()))
        }
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => None,
    };
}

/// What:
///  The paths one question says this form would lose,
///  read from Git.
/// Why:
///   Each answer is the one listing of the thing that question measured:
///  tracked changes for a form that writes a tree or the index,
///  untracked files for a clean that deletes them,
///  and nothing for a clean that deletes ignored files only,
///  which the owner decided is not a reason to hold a clean back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pathsAtRisk(facts, question): Promise<Buffer[]>;
/// ```
fn paths_at_risk(facts: &mut dyn RepositoryFacts, question: Option<DirtyQuestion>) -> Result<Vec<Vec<u8>>, String> {
    return match question {
        Some(DirtyQuestion::Worktree(DirtyScope::TrackedAndUntracked)) | Some(DirtyQuestion::CleanDryRun) => {
            facts.untracked_paths()
        }
        Some(DirtyQuestion::Worktree(DirtyScope::Tracked)) => facts.uncommitted_paths(),
        // A clean that deletes ignored files and nothing else destroys no untracked file,
        //  and a stash entry belongs to the repository rather than to this worktree:
        //  its own guard refuses the form instead, and nothing is copied here.
        Some(DirtyQuestion::RepositoryStash) | None => Ok(Vec::<Vec<u8>>::new()),
    };
}

/// What:
///  Copy what the form `form` is about to discard,
/// now that the exemption means it will run,
///  and say what happened.
/// Why:
///   The order is the owner's "save,
///  then allow" (`conflict_side_guard.rs`):
///  the bytes are on disk and synced before Git starts,
///  a copy that could not be written stops the form,
///  and the folder's limits are applied after the save,
///  never before,
///  so a kill while copying costs no older copy.
///  An ignored file at a path the form's tree tracks is copied here as well,
///  because a forced form overwrites it whatever `--no-overwrite-ignore` says
///  and it is where a person keeps their environment and their local secrets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saveTheExemptDiscard(facts, form): ExemptVerdict;
/// ```
pub(crate) fn save_the_exempt_discard(facts: &mut dyn RepositoryFacts, form: ExemptDiscard<'_>) -> ExemptVerdict {
    let ExemptDiscard {
        command,
        identity,
        question,
        tree,
        ignored_paths,
        limits,
        milliseconds,
    } = form;
    let mut paths: Vec<Vec<u8>> = match paths_at_risk(facts, question) {
        Ok(found) => found,
        Err(message) => return ExemptVerdict::Stopped(exempt_save_stopped_message(command, "", &message)),
    };
    // The ignored files the written tree tracks are the ones the form overwrites whatever `--no-overwrite-ignore` says.
    let shared: Vec<String> = match ignored_paths {
        Some(found) => found,
        None => match tree {
            Some(written) => match facts.ignored_files_the_tree_tracks(written.revision.as_deref()) {
                Ok(found) => found,
                Err(message) => return ExemptVerdict::Stopped(exempt_save_stopped_message(command, "", &message)),
            },
            None => Vec::<String>::new(),
        },
    };
    for path in shared {
        paths.push(path.into_bytes());
    }
    if paths.is_empty() {
        return ExemptVerdict::Nothing;
    }
    let Some((root, common)) = places(identity) else {
        // A target the allowlist exempts always has a worktree, so this is unreachable rather than "nothing at risk".
        return ExemptVerdict::Stopped(exempt_save_stopped_message(
            command,
            "",
            "this repository names no worktree to read the copies from",
        ));
    };
    match save_copies(root, common, paths.as_slice(), milliseconds) {
        Ok(copies) => {
            let unpruned: String = prune_after_save(common, copies.as_slice(), milliseconds, limits);
            return ExemptVerdict::Saved {
                notice: format!("{}{unpruned}", exempt_saved_notice(command, copies.as_slice())),
                copies,
            };
        }
        Err(failure) => {
            return ExemptVerdict::Stopped(exempt_save_stopped_message(command, &failure.subject, &failure.reason));
        }
    }
}

/// What:
///  The facts a record carries for a repository the allowlist exempts,
///  and the notice the person reads,
///  or both empty for a repository no entry holds.
///  `&[ToolCacheDirectory]` borrows the exempt directories.
/// Why:
///   Every decision an exemption lets through names the variable that chose it,
///  in one place,
///  so no ending can record the exemption without saying which variable made it
///  and no ending can print the notice twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exemptionFacts(identity, allowedDirs, notices): AuditFact[];
/// ```
pub(crate) fn exemption_facts(
    identity: &WorktreeIdentity,
    allowed_worktree_dirs: &[ToolCacheDirectory],
    notices: &mut Vec<String>,
) -> Vec<AuditFact> {
    let Some(found) = matching_allowed_worktree_dir_for_repository(identity, allowed_worktree_dirs) else {
        return Vec::<AuditFact>::new();
    };
    notices.push(exemption_notice(found.named_by, found.path.as_path()));
    return vec![AuditFact::Text("exemptedBy", found.named_by.to_string())];
}

/// What:
///  One exempt repository's discarding form,
///  with the facts it still asks and the record it leaves.
///  A `struct` is a record with named fields.
/// Why:
///  The two places an exempt repository saves from (a worktree that holds changes,
///  and a tree that would overwrite an ignored file) pass the same things,
///  so one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExemptForm = { facts; discard; location; seen; exemption; dirty; notices; conflictNote };
/// ```
pub(crate) struct ExemptForm<'a> {
    /// The facts the save still asks.
    pub(crate) facts: &'a mut dyn RepositoryFacts,

    /// The form and what its copy needs,
    ///  whose own limits the let-through point applies to a nested repository too.
    pub(crate) discard: ExemptDiscard<'a>,

    /// Where the command runs.
    pub(crate) location: &'a RepositoryLocation,

    /// The tokens after the subcommand,
    ///  for the nested repository a clean would delete.
    pub(crate) region: &'a [OsString],

    /// The command as Git will run it,
    /// aliases included.
    pub(crate) seen: &'a [OsString],

    /// The exemption the record names.
    pub(crate) exemption: &'a [AuditFact],

    /// What the policy measured,
    ///  or nothing when it asked no question.
    pub(crate) dirty: Option<bool>,

    /// The notices the person reads before Git runs.
    pub(crate) notices: &'a mut Vec<String>,

    /// The sentence a rejection of the same line gets beside the exemption's own text.
    pub(crate) conflict_note: &'a str,

}

/// What:
///  Copy what an exempt repository's form would discard,
///  record the decision,
///  and let the form run.
/// Why:
///   The exemption is from the worktree rule,
///  and never from the rule that nothing a person wrote is discarded unsaved:
///  the form runs with its save,
///  a save that could not be made stops it with the worktree as it was,
///  and a form that would lose nothing records the exemption and runs as Git runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saveAndAllow(form: ExemptForm): PolicyOutcome;
/// ```
pub(crate) fn save_and_allow(form: ExemptForm<'_>) -> PolicyOutcome {
    let command: GuardedCommand = form.discard.command;
    let discarded_limits: Limits = form.discard.limits;
    let mut saved: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    match save_the_exempt_discard(form.facts, form.discard) {
        ExemptVerdict::Nothing => {}
        ExemptVerdict::Saved { notice, copies } => {
            form.notices.push(notice);
            saved = copies;
        }
        ExemptVerdict::Stopped(message) => {
            record_audit_with_facts(&form.location.identity, command, AuditOutcome::Rejected, form.seen, form.dirty, form.exemption);
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{message}{}", form.conflict_note));
        }
    }
    // The save is on disk, and the form is let through where every letting-through happens, so a clean that would
    //  delete a nested repository still moves it whole before Git runs.
    return allow_a_discarding_form(AllowedForm {
        facts: form.facts,
        command,
        location: form.location,
        region: form.region,
        seen: form.seen,
        dirty: form.dirty,
        discarded_limits,
        notices: form.notices,
        exemption: form.exemption,
        saved_by_exemption: saved.as_slice(),
    });
}
