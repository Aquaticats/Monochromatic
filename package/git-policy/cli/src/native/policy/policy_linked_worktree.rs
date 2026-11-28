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

/// The abort and skip carve-out:
/// the operation's own state is what the command lets go,
/// and the person's own work is what it may not.
use crate::abort_discard_guard::{
    AbortVerdict,
    judge_abort_discard,
};
/// The second measurement of an abort or a skip,
/// owed immediately before Git starts.
use crate::abort_recheck::{
    AbortRecheckPlan,
    plan_the_second_look,
};
/// The record of every decision.
use crate::discard_audit::{
    AuditOutcome,
    record_audit,
    record_audit_with_facts,
};
/// The named facts a record of an exemption carries.
use crate::discard_audit_facts::AuditFact;
/// The limits of the folder of saved copies of conflicted files.
use crate::discarded_prune::Limits;
/// Which worktree a command targets,
///  for the linked-worktree rule.
use crate::effective_target::{
    EffectiveTarget,
    ToolCacheDirectory,
    classify_effective_target,
    matching_allowed_worktree_dir_for_repository,
};
/// The tokens after the subcommand word.
use crate::global_arguments::command_tokens;
/// The guarded decision about the command Git will run,
/// alias expansion included.
use crate::guarded_forwarded_command::GuardedForwarded;
/// The two commands that are recorded and never refused.
use crate::policy_audited::record_audited;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The refusal of a checkout or a restore that would replace a conflict's resolution,
///  asked of every command before the worktree's state is measured.
use crate::policy_linked_worktree_conflicted_file_refusal::{
    ConflictedFileForm,
    refuse_a_form_that_would_replace_a_conflicted_file,
};
/// The one place a form is let through,
/// which keeps a clean's nested repositories first.
use crate::policy_linked_worktree_nested_repositories::{
    AllowedForm,
    allow_a_discarding_form,
};
/// The refusal of a write to the settings every worktree of one repository shares.
use crate::policy_linked_worktree_shared_settings_refusal::refuse_shared_remote_settings;
/// The two places a dirty refusal names,
///  which are read from the identity the facts answered with.
use crate::policy_linked_worktree_the_places_it_judged::dirty_worktree;
/// The two readings of what work a form would take from the person.
use crate::policy_linked_worktree_work_a_form_would_discard::{
    clean_deletes_a_path_only_its_exclusions_ignore,
    ends_an_operation,
};
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// Where a command runs.
use crate::repository_location::RepositoryLocation;
use crate::rule_linked_worktree::{
    DirtyQuestion,
    GuardedCommand,
    LINKED_WORKTREE_REQUIRED_CODE,
    LinkedWorktreeDecision,
    TargetTree,
    dirty_question,
    exemption_notice,
    ignored_file_message,
    repository_stash_message,
    resolve_linked_worktree,
    target_tree,
};
/// The linked-worktree rule core.
use crate::rule_linked_worktree_dirty_refusal::{
    DirtyWorktree,
    dirty_worktree_message,
};
/// Whether a Git directory has no index file yet.
use crate::rule_worktree_dirty::index_file_is_absent;
/// The three endings every check shares.
use crate::the_answers_a_policy_gives::{
    clean,
    fact_unavailable,
    rejected,
};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path of raw bytes (siblings `PathBuf`,
///  the owned one,
///  and `&str` of text).
use std::path::Path;

/// What:
///  The linked-worktree-only check.
///  `&[ToolCacheDirectory]` borrows the tool caches the environment exempts.
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
    start: Option<&Path>,
) -> PolicyOutcome {
    // An alias reaches the command Git really runs, so the guard reads the expansion and every record names it.
    let seen: &[OsString] = forwarded.arguments.as_slice();
    // The settings every worktree shares are decided first: no guarded form below is about them,
    // and the guard read them from the command Git will run, a shell alias included.
    if let LinkedWorktreeDecision::SharedRemote(change) = &forwarded.decision {
        return refuse_shared_remote_settings(
            facts,
            change,
            forwarded.shell_directory.as_deref(),
            allowed_worktree_dirs,
            start,
        );
    }
    // `let Variant(x) = ... else { ... };` unwraps the guarded command or returns.
    let (command, moves_head_only): (GuardedCommand, bool) = match &forwarded.decision {
        LinkedWorktreeDecision::Pass => return clean(),
        // The two audited forms run as Git runs them and leave one line saying what they took; nothing is refused.
        LinkedWorktreeDecision::Audited(audited) => {
            // Outside a repository there is no audit file to write, and nothing to name.
            if let Ok(location) = facts.location() {
                record_audited(facts, seen, *audited, &location, start);
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
    // A checkout or a restore that would replace a conflict's resolution is refused before anything is measured;
    //  the refusal names every such file and keeps no copy (`policy_linked_worktree_conflicted_file_refusal.rs`).
    if let Some(outcome) = refuse_a_form_that_would_replace_a_conflicted_file(ConflictedFileForm {
        facts,
        identity: &location.identity,
        command,
        target,
        seen,
    }) {
        return outcome;
    }
    if moves_head_only {
        // A `HEAD` move outside a worktree or in an exempt cache is not this policy's concern.
        if target != EffectiveTarget::MainWorktree && target != EffectiveTarget::LinkedWorktree {
            return clean();
        }
    } else if !ends_an_operation(command)
        && let Some(message) = resolve_linked_worktree(command, target)
    {
        // Every abort and every skip is decided by the carve-out below, in the main worktree as in a linked one:
        //  the same reading answers both, and only the work it finds decides the command.
        record_audit(
            &location.identity,
            command,
            AuditOutcome::Rejected,
            seen,
            None,
        );
        return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
    }
    if target == EffectiveTarget::Allowlisted {
        // The environment chose this exemption, so the person is told which variable did it and where it points,
        //  before Git runs and before anything could be lost.
        // The exemption is judged on the repository the command runs in, so the notice names the entry that held both
        //  its Git directory and its worktree top.
        let exemption: Option<&ToolCacheDirectory> =
            matching_allowed_worktree_dir_for_repository(&location.identity, allowed_worktree_dirs);
        let mut why: Vec<AuditFact> = Vec::<AuditFact>::new();
        if let Some(found) = exemption {
            why.push(AuditFact::Text("exemptedBy", found.named_by.to_string()));
            notices.push(exemption_notice(found.named_by, found.path.as_path()));
        }
        record_audit_with_facts(
            &location.identity,
            command,
            AuditOutcome::Exempt,
            seen,
            None,
            why.as_slice(),
        );
        return clean();
    }
    // A merge, a rebase, a cherry-pick, a revert or an `am` that waits for its conflicts is undone by its own abort,
    //  and the state it left makes `git status` unclean by itself: the operation's own work is what the abort is for, while
    //  the person's own work is refused and never copied. Where nothing waits, Git refuses the abort itself and discards
    //  nothing, so the command is left to Git. The exemption above runs first, because an exempt repository is one no guard reads.
    match judge_abort_discard(facts, command) {
        AbortVerdict::NotApplicable => {}
        AbortVerdict::LeftToGit => {
            // The command discards nothing this policy knows of, so it runs, and the line says the guard left it alone.
            record_audit(
                &location.identity,
                command,
                AuditOutcome::Allowed,
                seen,
                None,
            );
            return clean();
        }
        AbortVerdict::Allowed => {
            // The operation's own state is all there is, so the command runs as Git runs it; one more measurement is owed
            //  just before Git starts, because an edit that lands in between would go with the abort unseen.
            plan_the_second_look(AbortRecheckPlan {
                context: facts.transaction_context(),
                command,
                identity: location.identity.clone(),
                seen: seen.to_vec(),
            });
            record_audit(
                &location.identity,
                command,
                AuditOutcome::Allowed,
                seen,
                None,
            );
            return clean();
        }
        AbortVerdict::Refused(message) => {
            record_audit(
                &location.identity,
                command,
                AuditOutcome::Rejected,
                seen,
                None,
            );
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
        }
    }
    // A checkout that only moves `HEAD` in a Git directory with no index file is Git's own initial checkout (a clone made with
    // `--no-checkout`): there is no tracked work to carry over, so the status, which lists every file as a staged deletion, is not read.
    if moves_head_only
        && index_file_is_absent(
            facts.known_git_dir().as_deref(),
            std::env::var_os("GIT_INDEX_FILE").as_deref(),
        )
    {
        record_audit(
            &location.identity,
            command,
            AuditOutcome::Allowed,
            seen,
            Some(false),
        );
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
        });
    };
    let tree: Option<TargetTree> = target_tree(command, region);
    // What: the two places a dirty refusal names, read from the identity the facts answered with.
    // Why: the index a nested Git keeps and the files its own `-C` moves can be two worktrees', and only a text that
    //  names both tells a person which of them to clean (the alpha.52 read's L2).
    let places: Option<DirtyWorktree<'_>> = dirty_worktree(command, &location);
    // A stash drop deletes an entry of the one stash the repository shares, so the running worktree's status says
    //  nothing about it and the repository's own stash is asked instead.
    let (measured, rejection): (Result<bool, String>, String) = match question {
        DirtyQuestion::Worktree(scope) => {
            let Some(reading) = places else {
                return fact_unavailable(
                    "the worktree this command's state would be read in cannot be named, so the command was not run"
                        .to_string(),
                );
            };
            (facts.worktree_dirty(scope), dirty_worktree_message(reading))
        }
        // A clean that names its own exclusions is measured by what Git would delete only because of them, which is
        //  the one reading that applies those patterns.
        DirtyQuestion::CleanDryRun => {
            let Some(reading) = places else {
                return fact_unavailable(
                    "the worktree this clean's state would be read in cannot be named, so the command was not run"
                        .to_string(),
                );
            };
            (
                clean_deletes_a_path_only_its_exclusions_ignore(facts, region),
                dirty_worktree_message(reading),
            )
        }
        DirtyQuestion::RepositoryStash => (
            facts.repository_holds_a_stash(),
            repository_stash_message(command),
        ),
    };
    match measured {
        Ok(true) => {
            record_audit(
                &location.identity,
                command,
                AuditOutcome::Rejected,
                seen,
                Some(true),
            );
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, rejection);
        }
        Ok(false) => {
            // The tracked state being clean says nothing about an ignored file the target tree tracks: a forced
            //  form overwrites that file whatever `--no-overwrite-ignore` says, and the ignored file is where a person
            //  keeps their environment and their local secrets. The rejected form is not confirmed clean: a tree that
            //  cannot be read leaves the command unread, never allowed.
            if let Some(written) = tree.as_ref() {
                let shared: Vec<String> =
                    match facts.ignored_files_the_tree_tracks(written.revision.as_deref()) {
                        Ok(found) => found,
                        Err(message) => return fact_unavailable(message),
                    };
                if !shared.is_empty() {
                    let why: Vec<AuditFact> =
                        vec![AuditFact::Text("ignoredAndTracked", shared.join(" "))];
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
                        ignored_file_message(
                            command,
                            written.revision.as_deref(),
                            shared.as_slice(),
                        ),
                    );
                }
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
            });
        }
        Err(message) => return fact_unavailable(message),
    }
}
