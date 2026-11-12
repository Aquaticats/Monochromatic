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
use crate::command_clean::{clean_dry_run_arguments, clean_dry_run_arguments_without_exclusions, paths_only_the_exclusions_add};
/// Which worktree a command targets,
///  for the linked-worktree rule.
use crate::effective_target::{EffectiveTarget, ToolCacheDirectory, classify_effective_target};
/// The guarded decision about the command Git will run,
/// alias expansion included.
use crate::guarded_forwarded_command::GuardedForwarded;
/// The named facts a record of an exemption carries.
use crate::discard_audit_facts::AuditFact;
/// The record of every decision.
use crate::discard_audit::{
    AuditOutcome, record_audit, record_audit_with_copies_and_facts, record_audit_with_facts,
};
/// The limits of the folder of saved copies of conflicted files.
use crate::discarded_prune::Limits;
/// The sides of a conflicted file,
///  judged before a `checkout` or `restore` takes one.
use crate::conflict_side_guard::{SideVerdict, judge_conflict_sides};
/// The two commands that are recorded and never refused.
use crate::policy_audited::record_audited;
/// The abort carve-out:
/// the operation's own state is what an abort lets go,
/// the person's own work is saved first.
use crate::abort_discard_guard::{AbortVerdict, judge_abort_discard};
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
    dirty_question, dirty_worktree_message, ignored_file_message,
    repository_stash_message, resolve_linked_worktree, target_tree,
};
/// The refusal of a command that would change the settings every worktree shares.
use crate::policy_linked_worktree_shared_remote::refuse_shared_remote_settings;
/// Whether a Git directory has no index file yet.
use crate::rule_worktree_dirty::index_file_is_absent;
/// The one place a form is let through,
/// which keeps a clean's nested repositories first.
use crate::policy_linked_worktree_nested_repositories::{AllowedForm, allow_a_discarding_form, now_milliseconds};
/// The save-first guard an exempt repository keeps around a form that discards work.
use crate::policy_linked_worktree_exempt_save::{
    ExemptDiscard, ExemptForm, exemption_facts, save_and_allow,
};
/// The list of copies a record names.
use crate::conflict_sides::copy_places;
/// The tokens after the subcommand word.
use crate::global_arguments::command_tokens;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Whether a `git clean -X` that carries its own `-e`/`--exclude` patterns would delete a path those patterns
///  alone call ignored.
///  `&[OsString]` borrows the clean's own tokens after the subcommand.
/// Why:
///   A pattern can pull two different things into a clean:
///  a path the worktree itself ignores is an ignored file,
///  which the owner decided is not a reason to refuse this clean in a linked worktree,
///  while a path only the patterns call ignored is an untracked file this clean would lose,
///  which still is.
///  Git answers both sets itself:
///  the dry run of the clean as typed and the dry run of the same clean without the patterns,
///  and what only the first answer names is what the patterns add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await cleanDeletesAPathOnlyItsExclusionsIgnore(facts, ['-fX', '-e', 'notes.txt'])
/// ```
fn clean_deletes_a_path_only_its_exclusions_ignore(
    facts: &mut dyn RepositoryFacts,
    region: &[OsString],
) -> Result<bool, String> {
    // The global options come before every start of Git, so they are read once for both dry runs.
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    // A region Git would refuse deletes nothing, and Git refuses it in its own words when the command runs.
    let Some(with_patterns) = clean_dry_run_arguments(prefix.as_slice(), region) else {
        return Ok(false);
    };
    let deleted: Vec<Vec<u8>> = facts.clean_deleted_paths(with_patterns)?;
    if deleted.is_empty() {
        return Ok(false);
    }
    // The same clean without them deletes what the worktree itself ignores; both answers name a path the same way,
    //  so they are compared as Git printed them.
    let Some(without_patterns) = clean_dry_run_arguments_without_exclusions(prefix.as_slice(), region) else {
        return Ok(true);
    };
    let otherwise_deleted: Vec<Vec<u8>> = facts.clean_deleted_paths(without_patterns)?;
    return Ok(!paths_only_the_exclusions_add(deleted.as_slice(), otherwise_deleted.as_slice()).is_empty());
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
    // The settings every worktree shares are decided first: no guarded form below is about them,
    // and the guard read them from the command Git will run, a shell alias included.
    if let LinkedWorktreeDecision::SharedRemote(change) = &forwarded.decision {
        return refuse_shared_remote_settings(facts, change, allowed_worktree_dirs);
    }
    // `let Variant(x) = ... else { ... };` unwraps the guarded command or returns.
    let (command, moves_head_only): (GuardedCommand, bool) = match &forwarded.decision {
        LinkedWorktreeDecision::Pass => return clean(),
        // The two audited forms run as Git runs them and leave one line saying what they took; nothing is refused.
        LinkedWorktreeDecision::Audited(audited) => {
            // Outside a repository there is no audit file to write, and nothing to name.
            if let Ok(location) = facts.location() {
                record_audited(facts, seen, *audited, &location);
            }
            return clean();
        }
        LinkedWorktreeDecision::NeedsEffectiveTarget(command) => (*command, false),
        LinkedWorktreeDecision::WhenDirty(command) => (*command, true),
        // Refused above, before any guarded form was measured.
        LinkedWorktreeDecision::SharedRemote(_) => return clean(),
    };
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    let target: EffectiveTarget =
        classify_effective_target(&location.identity, allowed_worktree_dirs);
    // A form that only moves `HEAD` in an exempt cache asks nothing, so nothing is measured, saved, announced or
    //  recorded for it.
    let exempt: bool = target == EffectiveTarget::Allowlisted;
    if moves_head_only && !matches!(target, EffectiveTarget::MainWorktree | EffectiveTarget::LinkedWorktree) {
        return clean();
    }
    // The environment chose this exemption, so the person is told which variable did it and where it points,
    //  before Git runs and before anything could be lost;
    //  every record this decision writes carries the same name.
    // The exemption is judged on the repository the command runs in, so the notice names the entry that held both
    //  its Git directory and its worktree top.
    let exemption: Vec<AuditFact> = if exempt {
        exemption_facts(&location.identity, allowed_worktree_dirs, notices)
    } else {
        Vec::<AuditFact>::new()
    };
    // During a merge, rebase, cherry-pick or revert, taking a side of conflicted files is let through after the files
    // are saved; the verdict also carries the sentence a rejection of the same line gets.
    let mut conflict_note: String = String::new();
    if matches!(command, GuardedCommand::CheckoutPaths | GuardedCommand::RestoreWorktree)
        && matches!(
            target,
            EffectiveTarget::MainWorktree | EffectiveTarget::LinkedWorktree | EffectiveTarget::Allowlisted
        )
    {
        match judge_conflict_sides(&facts.transaction_context(), seen, now_milliseconds(), discarded_limits) {
            SideVerdict::NotApplicable => {}
            SideVerdict::Refused(note) => conflict_note = note,
            SideVerdict::Allowed { notice, copies } => {
                let places: Vec<String> = copy_places(copies.as_slice());
                record_audit_with_copies_and_facts(
                    &location.identity,
                    command,
                    AuditOutcome::SavedConflict,
                    seen,
                    None,
                    places.as_slice(),
                    exemption.as_slice(),
                );
                notices.push(notice);
                return clean();
            }
            SideVerdict::CopyFailed(message) => {
                record_audit_with_facts(&location.identity, command, AuditOutcome::Rejected, seen, None, exemption.as_slice());
                return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
            }
        }
    }
    if !moves_head_only
        && let Some(message) = resolve_linked_worktree(command, target)
    {
        record_audit(&location.identity, command, AuditOutcome::Rejected, seen, None);
        return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{message}{conflict_note}"));
    }
    // A merge, a rebase, a cherry-pick, a revert or an `am` that waits for its conflicts is undone by its own abort,
    //  and the state it left makes `git status` unclean by itself: the operation's own work is what the abort is for, and
    //  the person's own work is copied beside the audit log before the command runs. Where nothing waits, Git refuses the
    //  abort itself and discards nothing, so the command is left to Git. An exempt repository reaches this carve-out too:
    //  the exemption is from the worktree rule, which refused no abort here, and it never turned off the copy.
    match judge_abort_discard(facts, command, &location.identity, now_milliseconds(), discarded_limits) {
        AbortVerdict::NotApplicable => {}
        AbortVerdict::LeftToGit => {
            // The command discards nothing this policy knows of, so it runs, and the line says the guard left it alone.
            record_audit_with_facts(&location.identity, command, AuditOutcome::Allowed, seen, None, exemption.as_slice());
            return clean();
        }
        AbortVerdict::Allowed { notice, copies } => {
            if copies.is_empty() {
                record_audit_with_facts(&location.identity, command, AuditOutcome::Allowed, seen, None, exemption.as_slice());
            } else {
                let places: Vec<String> = copy_places(copies.as_slice());
                record_audit_with_copies_and_facts(
                    &location.identity,
                    command,
                    AuditOutcome::SavedAbort,
                    seen,
                    None,
                    places.as_slice(),
                    exemption.as_slice(),
                );
            }
            if !notice.is_empty() {
                notices.push(notice);
            }
            return clean();
        }
        AbortVerdict::CopyFailed(message) => {
            record_audit_with_facts(&location.identity, command, AuditOutcome::Rejected, seen, None, exemption.as_slice());
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
        }
    }
    // A checkout that only moves `HEAD` in a Git directory with no index file is Git's own initial checkout (a clone made with
    // `--no-checkout`): there is no tracked work to carry over, so the status, which lists every file as a staged deletion, is not read.
    if moves_head_only && index_file_is_absent(facts.known_git_dir().as_deref(), std::env::var_os("GIT_INDEX_FILE").as_deref()) {
        record_audit_with_facts(&location.identity, command, AuditOutcome::Allowed, seen, Some(false), exemption.as_slice());
        return clean();
    }
    // The tokens after the subcommand, read once: which tree a forced form writes and which question a clean asks
    //  both come from them.
    let region: &[OsString] = match command_tokens(seen) {
        Some((_word, tokens)) => tokens,
        None => &[],
    };
    // Which question this form asks, and the tree a forced form writes, both come from the tokens after it.
    let asked: Option<DirtyQuestion> = dirty_question(command, region);
    // A form that loses nothing this policy protects leaves its one line and runs: an ignored file is not a reason
    //  to refuse a clean in a linked worktree (the owner's answer of 2026-11-10, "Allow and log").
    let Some(question) = asked else {
        return allow_a_discarding_form(AllowedForm {
            facts,
            command,
            location: &location,
            region,
            seen,
            dirty: None,
            discarded_limits,
            notices,
            exemption: exemption.as_slice(),
            saved_by_exemption: &[],
        });
    };
    let tree: Option<TargetTree> = target_tree(command, region);
    // A stash drop deletes an entry of the one stash the repository shares, so the running worktree's status says
    //  nothing about it and the repository's own stash is asked instead.
    let (measured, rejection): (Result<bool, String>, String) = match question {
        DirtyQuestion::Worktree(scope) => (facts.worktree_dirty(scope), dirty_worktree_message(command)),
        // A clean that names its own exclusions is measured by what Git would delete only because of them, which is
        //  the one reading that applies those patterns.
        DirtyQuestion::CleanDryRun => (
            clean_deletes_a_path_only_its_exclusions_ignore(facts, region),
            dirty_worktree_message(command),
        ),
        DirtyQuestion::RepositoryStash => (facts.repository_holds_a_stash(), repository_stash_message(command)),
    };
    match measured {
        Ok(true) => {
            if exempt {
                // The exemption runs the form, and "save, then allow" keeps what it would discard. The repository's own
                //  stash is the one entry no copy here holds, so that form keeps its refusal and the hatch it names.
                if matches!(question, DirtyQuestion::RepositoryStash) {
                    record_audit_with_facts(&location.identity, command, AuditOutcome::Rejected, seen, Some(true), exemption.as_slice());
                    return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{rejection}{conflict_note}"));
                }
                return save_and_allow(ExemptForm {
                    facts,
                    discard: ExemptDiscard {
                        command,
                        identity: &location.identity,
                        question: Some(question),
                        tree: tree.as_ref(),
                        ignored_paths: None,
                        limits: discarded_limits,
                        milliseconds: now_milliseconds(),
                    },
                    location: &location,
                    region,
                    seen,
                    exemption: exemption.as_slice(),
                    dirty: Some(true),
                    notices,
                    conflict_note: &conflict_note,
                });
            }
            record_audit(&location.identity, command, AuditOutcome::Rejected, seen, Some(true));
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{rejection}{conflict_note}"));
        }
        Ok(false) => {
            // The tracked state being clean says nothing about an ignored file the target tree tracks: a forced
            //  form overwrites that file whatever `--no-overwrite-ignore` says, and the ignored file is where a person
            //  keeps their environment and their local secrets. A tree that cannot be read leaves the command unread,
            //  never allowed.
            let shared: Vec<String> = match tree.as_ref() {
                Some(written) => match facts.ignored_files_the_tree_tracks(written.revision.as_deref()) {
                    Ok(found) => found,
                    Err(message) => return fact_unavailable(message),
                },
                None => Vec::<String>::new(),
            };
            if exempt {
                // The exemption runs the form, and an ignored file the written tree tracks is copied first: the
                //  ignored file is where a person keeps their environment and their local secrets.
                return save_and_allow(ExemptForm {
                    facts,
                    discard: ExemptDiscard {
                        command,
                        identity: &location.identity,
                        question: None,
                        tree: tree.as_ref(),
                        ignored_paths: Some(shared),
                        limits: discarded_limits,
                        milliseconds: now_milliseconds(),
                    },
                    location: &location,
                    region,
                    seen,
                    exemption: exemption.as_slice(),
                    dirty: Some(false),
                    notices,
                    conflict_note: &conflict_note,
                });
            }
            if !shared.is_empty() {
                let written: &TargetTree = tree.as_ref().expect("only a form that writes a tree names one");
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
            return allow_a_discarding_form(AllowedForm {
                facts,
                command,
                location: &location,
                region,
                seen,
                dirty: Some(false),
                discarded_limits,
                notices,
                exemption: exemption.as_slice(),
                saved_by_exemption: &[],
            });
        }
        Err(message) => return fact_unavailable(message),
    }
}
