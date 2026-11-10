//! What:
//!  The linked-worktree-only policy as the engine runs it:
//!  the guarded forms,
//!  the two forms that are only
//!       recorded,
//!  and the facts their decision asks for.
//! Why:
//!  The check joins the pure rule core with the repository facts it asks for,
//!  and it is the
//!       one place that decides when a discarding command runs and what its audit line says;
//!  it sits in its own file so
//!       the shipped policies' registry stays readable
//!       (`src/native/policy/policy_checks.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = checkLinkedWorktree({ facts, forwarded, allowedDirs, limits, notices });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Each part of the decision and each fact it asks for lives in the file that owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { dirtyQuestion, targetTree } from '../command/rule_linked_worktree.ts';
/// ```
use crate::command_clean::clean_dry_run_arguments;
/// Which worktree a command targets,
///  for the linked-worktree rule.
use crate::effective_target::{
    EffectiveTarget, ToolCacheDirectory, classify_effective_target, matching_allowed_worktree_dir,
};
/// The guarded decision about the command Git will run,
/// alias expansion included.
use crate::guarded_forwarded_command::GuardedForwarded;
/// The named facts a record of an exemption carries.
use crate::discard_audit_facts::AuditFact;
/// The record of every decision.
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_copies, record_audit_with_facts};
/// The limits of the folder of saved copies of conflicted files.
use crate::discarded_prune::Limits;
/// The sides of a conflicted file,
///  judged before a `checkout` or `restore` takes one.
use crate::conflict_side_guard::{SideVerdict, judge_conflict_sides};
/// The two commands that are recorded and never refused.
use crate::policy_audited::record_audited;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// Where a command runs.
use crate::repository_location::RepositoryLocation;
use crate::policy_checks::{fact_unavailable, clean, rejected};
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{
    DirtyQuestion, GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, LinkedWorktreeDecision, TargetTree,
    dirty_question, dirty_worktree_message, exemption_notice, ignored_file_message,
    repository_stash_message, resolve_linked_worktree, target_tree,
};
/// Whether a `git remote` or `git config` command writes the settings every worktree shares.
use crate::rule_shared_remote::{SHARED_REMOTE_SETTINGS_CODE, SharedRemoteWrite, shared_remote_write};
/// The text that refusal prints.
use crate::rule_shared_remote_messages::shared_remote_settings_message;
/// Whether a Git directory has no index file yet.
use crate::rule_worktree_dirty::{DirtyScope, index_file_is_absent};
/// The Git directories of an identity,
///  for naming the exempted directory and the file every worktree shares.
use crate::worktree_identity::{common_directory, git_directory};
/// The tokens after the subcommand word.
use crate::global_arguments::command_tokens;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// The clock reading the conflict copies are named with,
/// in milliseconds since the Unix epoch.
fn now_milliseconds() -> i64 {
    let elapsed: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    return i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
}

/// What:
///  The refusal of a command that would change the remote settings every worktree of the
///  repository shares,
///  when the command runs in a linked worktree.
///  `&[OsString]` borrows the arguments Git will receive.
/// Why:
///   One `.git/config` is shared by the main checkout and every linked worktree,
///  so a write from a
///       linked worktree reaches the main checkout and every other worktree;
///  the refusal names that file and
///       the main checkout,
///  and changes nothing.
///  A main checkout,
///  a place outside a worktree and an
///       exempt tool cache keep today's behaviour,
///  and every other command is left to the policies below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseSharedRemoteSettings(facts, args, allowedDirs): PolicyOutcome | undefined;
/// ```
fn refuse_shared_remote_settings(
    facts: &mut dyn RepositoryFacts,
    arguments: &[OsString],
    allowed_worktree_dirs: &[ToolCacheDirectory],
) -> Option<PolicyOutcome> {
    // A `?` answers nothing for a command line Git runs no subcommand for.
    let (word, region) = command_tokens(arguments)?;
    // `?` answers nothing for every command that changes no shared remote setting.
    let write: SharedRemoteWrite = shared_remote_write(word.as_encoded_bytes(), region)?;
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return Some(fact_unavailable(message)),
    };
    let target: EffectiveTarget =
        classify_effective_target(&location.identity, allowed_worktree_dirs);
    // Only a linked worktree shares its configuration with another checkout.
    if target != EffectiveTarget::LinkedWorktree {
        return None;
    }
    let main_checkout: PathBuf = match facts.main_worktree_root() {
        Ok(found) => found,
        Err(message) => return Some(fact_unavailable(message)),
    };
    // The configuration file Git writes for the repository: `$GIT_COMMON_DIR/config`.
    let shared_file: PathBuf = match common_directory(&location.identity) {
        Some(directory) => directory.join("config"),
        None => {
            return Some(fact_unavailable(String::from(
                "monochromatic-git-policy-cli could not name the configuration file this repository's worktrees \
                 share, so the command was not judged.",
            )));
        }
    };
    return Some(rejected(
        SHARED_REMOTE_SETTINGS_CODE,
        shared_remote_settings_message(&write, shared_file.as_path(), main_checkout.as_path()),
    ));
}

/// What:
///  The linked-worktree-only check.
///  `&[PathBuf]` borrows the exempt tool caches.
/// Why:
///   A guarded form that overwrites is rejected at once in the main worktree and outside any worktree,
/// and in a
///  linked worktree only while uncommitted state exists;
/// a form that only moves `HEAD` is rejected in the main or
///  a linked worktree only while uncommitted state exists.
/// Only these need the worktree kind or the status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkLinkedWorktree(facts, args, allowedDirs): Promise<PolicyOutcome>;
/// ```
pub(crate) fn check_linked_worktree(
    facts: &mut dyn RepositoryFacts,
    forwarded: &GuardedForwarded,
    allowed_worktree_dirs: &[ToolCacheDirectory],
    discarded_limits: Limits,
    notices: &mut Vec<String>,
) -> PolicyOutcome {
    // An alias reaches the command Git really runs, so the guard reads the expansion and every record names it.
    let seen: &[OsString] = forwarded.arguments.as_slice();
    // The settings every worktree shares are refused before anything else: no guarded form is about them.
    if let Some(outcome) = refuse_shared_remote_settings(facts, seen, allowed_worktree_dirs) {
        return outcome;
    }
    // `let Variant(x) = ... else { ... };` unwraps the guarded command or returns.
    let (command, moves_head_only): (GuardedCommand, bool) = match forwarded.decision {
        LinkedWorktreeDecision::Pass => return clean(),
        // The two audited forms run as Git runs them and leave one line saying what they took; nothing is refused.
        LinkedWorktreeDecision::Audited(audited) => {
            // Outside a repository there is no audit file to write, and nothing to name.
            if let Ok(location) = facts.location() {
                record_audited(facts, seen, audited, &location);
            }
            return clean();
        }
        LinkedWorktreeDecision::NeedsEffectiveTarget(command) => (command, false),
        LinkedWorktreeDecision::WhenDirty(command) => (command, true),
    };
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    let target: EffectiveTarget =
        classify_effective_target(&location.identity, allowed_worktree_dirs);
    // During a merge, rebase, cherry-pick or revert, taking a side of conflicted files is let through after the files
    // are saved; the verdict also carries the sentence a rejection of the same line gets.
    let mut conflict_note: String = String::new();
    if matches!(command, GuardedCommand::CheckoutPaths | GuardedCommand::RestoreWorktree)
        && matches!(target, EffectiveTarget::MainWorktree | EffectiveTarget::LinkedWorktree)
    {
        match judge_conflict_sides(&facts.transaction_context(), seen, now_milliseconds(), discarded_limits) {
            SideVerdict::NotApplicable => {}
            SideVerdict::Refused(note) => conflict_note = note,
            SideVerdict::Allowed { notice, copies } => {
                let mut places: Vec<String> = Vec::new();
                for copy in &copies {
                    let saved_to: String = match copy.saved_to.as_ref() {
                        Some(place) => place.display().to_string(),
                        None => String::from("none"),
                    };
                    places.push(format!("{} => {}", copy.path, saved_to));
                }
                record_audit_with_copies(&location.identity, command, AuditOutcome::SavedConflict, seen, None, places.as_slice());
                notices.push(notice);
                return clean();
            }
            SideVerdict::CopyFailed(message) => {
                record_audit(&location.identity, command, AuditOutcome::Rejected, seen, None);
                return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
            }
        }
    }
    if moves_head_only {
        // A `HEAD` move outside a worktree or in an exempt cache is not this policy's concern.
        if target != EffectiveTarget::MainWorktree && target != EffectiveTarget::LinkedWorktree {
            return clean();
        }
    } else if let Some(message) = resolve_linked_worktree(command, target) {
        record_audit(&location.identity, command, AuditOutcome::Rejected, seen, None);
        return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{message}{conflict_note}"));
    }
    if target == EffectiveTarget::Allowlisted {
        // The environment chose this exemption, so the person is told which variable did it and where it points,
        //  before Git runs and before anything could be lost.
        let exemption: Option<&ToolCacheDirectory> =
            matching_allowed_worktree_dir(git_directory(&location.identity).unwrap_or(Path::new("")), allowed_worktree_dirs);
        let mut why: Vec<AuditFact> = Vec::<AuditFact>::new();
        if let Some(found) = exemption {
            why.push(AuditFact::Text("exemptedBy", found.named_by.to_string()));
            notices.push(exemption_notice(found.named_by, found.path.as_path()));
        }
        record_audit_with_facts(&location.identity, command, AuditOutcome::Exempt, seen, None, why.as_slice());
        return clean();
    }
    // A checkout that only moves `HEAD` in a Git directory with no index file is Git's own initial checkout (a clone made with
    // `--no-checkout`): there is no tracked work to carry over, so the status, which lists every file as a staged deletion, is not read.
    if moves_head_only && index_file_is_absent(facts.known_git_dir().as_deref(), std::env::var_os("GIT_INDEX_FILE").as_deref()) {
        record_audit(&location.identity, command, AuditOutcome::Allowed, seen, Some(false));
        return clean();
    }
    // The tokens after the subcommand, read once: which tree a forced form writes and which question a clean asks
    //  both come from them.
    let region: &[OsString] = match command_tokens(seen) {
        Some((_word, tokens)) => tokens,
        None => &[],
    };
    // The tracked state being clean is the question these forms ask; the tree they write is the second question.
    let tree: Option<TargetTree> = target_tree(command, region);
    // A stash drop deletes an entry of the one stash the repository shares, so the running worktree's status says
    //  nothing about it and the repository's own stash is asked instead.
    let (measured, rejection): (Result<bool, String>, String) = match dirty_question(command, region) {
        DirtyQuestion::Worktree(scope) => (facts.worktree_dirty(scope), dirty_worktree_message(command)),
        // A clean that names its own exclusions is measured by what Git would delete, which is the one reading that
        //  applies those patterns; a region Git refuses deletes nothing and stays with the status.
        DirtyQuestion::CleanDryRun => (
            match clean_dry_run_arguments(&facts.transaction_context().global_prefix, region) {
                Some(query) => facts.clean_would_delete(query),
                None => facts.worktree_dirty(DirtyScope::IgnoredOnly),
            },
            dirty_worktree_message(command),
        ),
        DirtyQuestion::RepositoryStash => (facts.repository_holds_a_stash(), repository_stash_message(command)),
    };
    match measured {
        Ok(true) => {
            record_audit(&location.identity, command, AuditOutcome::Rejected, seen, Some(true));
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{rejection}{conflict_note}"));
        }
        Ok(false) => {
            // The tracked state being clean says nothing about an ignored file the target tree tracks: a forced
            //  form overwrites that file whatever `--no-overwrite-ignore` says, and the ignored file is where a person
            //  keeps their environment and their local secrets. The rejected form is not confirmed clean: a tree that
            //  cannot be read leaves the command unread, never allowed.
            if let Some(written) = tree.as_ref() {
                let shared: Vec<String> = match facts.ignored_files_the_tree_tracks(written.revision.as_deref()) {
                    Ok(found) => found,
                    Err(message) => return fact_unavailable(message),
                };
                if !shared.is_empty() {
                    let why: Vec<AuditFact> = vec![AuditFact::Text("ignoredAndTracked", shared.join(" "))];
                    record_audit_with_facts(
                        &location.identity,
                        command,
                        AuditOutcome::Rejected,
                        seen,
                        Some(false),
                        why.as_slice(),
                    );
                    return rejected(
                        LINKED_WORKTREE_REQUIRED_CODE,
                        format!("{}{conflict_note}", ignored_file_message(command, written.revision.as_deref(), shared.as_slice())),
                    );
                }
            }
            record_audit(&location.identity, command, AuditOutcome::Allowed, seen, Some(false));
            return clean();
        }
        Err(message) => return fact_unavailable(message),
    }
}
