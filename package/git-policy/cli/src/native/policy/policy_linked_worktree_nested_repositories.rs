//! What:
//!  Keeping every nested Git repository a clean in a linked worktree would delete,
//!  and the one place a discarding form is let through or stopped.
//! Why:
//!  Git's clean with a second `-f` removes a nested repository whole,
//!  with its own history and its uncommitted files,
//!  and the outer worktree reports it as one ignored directory,
//!  so nothing else the policy reads can see the work inside it.
//!  The owner's answer for uncommitted work is "save,
//! then allow" (`conflict_side_guard.rs`),
//!  so the repository is moved beside the audit log and the clean runs as typed;
//!  a move that fails stops the command with the worktree as it was.
//!  The file also holds the two points a discarding form is let through,
//!  because both must ask this question before Git starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const saved = keepTheNestedRepositoriesACleanWouldDelete(facts, location, region, limits);
//! ```

/// The builder of the clean's own dry run,
///  and the parsed facts that decide whether the question applies.
use crate::command_clean::{UNREADABLE_CLEAN_DELETION, clean_dry_run_arguments, parse_clean_region};
/// Which nested Git repositories that answer says the clean would delete.
use crate::command_clean_nested_repositories::nested_repositories_the_clean_deletes;
/// The bound on the folder of saved copies applied after a save,
///  and the place saved copies live.
use crate::conflict_side_guard::prune_after_save;
use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy};
/// The record of every decision.
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_copies_and_facts, record_audit_with_facts};
/// The fact that names the exemption a record ran under.
use crate::discard_audit_facts::AuditFact;
/// The copies a record names, as the one list every saving ending writes.
use crate::conflict_sides::copy_places;
/// The limits of the folder of saved copies.
use crate::discarded_prune::Limits;
/// The move of a nested repository the clean would delete.
use crate::discarded_nested_repository::set_nested_repositories_aside;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
use crate::policy_checks::{clean, rejected};
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE};
/// The notice that names where a saved nested repository went,
///  and the rejection when it could not be moved there.
use crate::rule_linked_worktree_nested_repository_messages::{
    NotSaved, nested_repository_not_saved_message, saved_nested_repository_notice,
};
/// The Git directories of an identity,
///  for naming the place saved copies live and the worktree the paths are below.
use crate::worktree_identity::{common_directory, worktree_root};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::PathBuf;

/// The clock reading the conflict copies are named with,
/// in milliseconds since the Unix epoch.
pub(crate) fn now_milliseconds() -> i64 {
    let elapsed: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    return i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
}

/// What:
///  What keeping the nested repositories of a clean left to do.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The three answers are handled differently:
///  nothing to keep lets the command run as it always did,
///  a save lets it run with a notice and an audit line naming the places,
///  and a refusal stops it with nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NestedRepositories = { kind: 'nothing' } | { kind: 'saved'; copies; notice } | { kind: 'refused'; message };
/// ```
enum NestedRepositories {
    /// The clean deletes no nested repository,
    /// or Git deletes none of what it names.
    Nothing,
    /// Each repository is below the saved copies,
    /// and the notice names where.
    Saved {
        /// Where each repository went,
        /// for the audit line.
        copies: Vec<SavedCopy>,
        /// What the person reads before Git runs.
        notice: String,
    },
    /// A repository could not be kept,
    ///  the sentence says why and what to do,
    ///  and `copies` names every repository a stopped save left below the saved copies.
    Refused {
        /// What the person reads instead of the clean running.
        message: String,
        /// The repositories the stopped save moved and could not put back,
        ///  each with the place it is.
        copies: Vec<SavedCopy>,
    },
}

/// What:
///  Keep every nested Git repository the clean would delete,
/// whole,
/// before it runs.
///  `&RepositoryLocation` borrows where the command runs and which worktree it runs in.
/// Why:
///   A second `-f` removes a nested repository with its own history and its uncommitted files,
///  which no other copy of the surrounding repository holds;
///  the owner's answer for uncommitted work is to save it and let the command run
///  (`conflict_side_guard.rs`),
///  and refusing would only move the same loss to `rm -rf`.
///  Git itself says which paths the clean removes,
///  and the repository the clean deletes is the one below one of those paths,
///  found by looking inside the names Git printed and nowhere else.
///  A clean whose dry run Git refuses deletes nothing,
///  and Git then refuses the command itself in its own words.
///  A clean whose dry run never answered was read by nobody,
///  so it stops instead of running on a guess.
///  An interactive clean is not moved here at all:
///  its menu is answered after Git starts,
///  so the fixed transform keeps the repositories out of that menu by one force
///  (`rule_clean_interactive_nested_repositories.rs`),
///  and a repository moved before a menu is one the person never chose to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepTheNestedRepositoriesACleanWouldDelete(facts, location, region, limits): NestedRepositories;
/// ```
fn keep_the_nested_repositories_a_clean_would_delete(
    facts: &mut dyn RepositoryFacts,
    location: &RepositoryLocation,
    region: &[OsString],
    discarded_limits: Limits,
) -> NestedRepositories {
    // A clean removes a nested repository only with a second `-f` (`builtin/clean.c`: `if (force > 1) rm_flags = 0`),
    //  and only when it descends into the directories it deletes ignored files from. Every other clean is Git's own
    //  business, and one force is Git's own protection of the repository.
    let Ok(parsed) = parse_clean_region(region, &[]) else {
        return NestedRepositories::Nothing;
    };
    if !parsed.deletes_ignored || !parsed.descends || parsed.forces < 2 {
        return NestedRepositories::Nothing;
    }
    // A menu is answered after Git starts, so nothing here can ask the person first; the transform that runs the clean
    //  with one force keeps the repositories where they are instead of moving them before a decision.
    if parsed.interactive {
        return NestedRepositories::Nothing;
    }
    let Some(root) = worktree_root(&location.identity) else {
        return NestedRepositories::Nothing;
    };
    let Some(common_dir) = common_directory(&location.identity) else {
        return NestedRepositories::Nothing;
    };
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    let Some(as_typed) = clean_dry_run_arguments(prefix.as_slice(), region) else {
        return NestedRepositories::Nothing;
    };
    let deleted: Vec<Vec<u8>> = match facts.clean_deleted_paths(as_typed) {
        Ok(answer) => answer,
        // Git refuses this clean itself, so it deletes nothing and Git's own refusal is the answer the person gets.
        Err(message) if message.starts_with(UNREADABLE_CLEAN_DELETION) => return NestedRepositories::Nothing,
        // A reading that never answered could hide a repository, so the clean stops here.
        Err(message) => return NestedRepositories::Refused { message, copies: Vec::<SavedCopy>::new() },
    };
    let repositories: Vec<Vec<u8>> = match nested_repositories_the_clean_deletes(deleted.as_slice(), root) {
        Ok(found) => found,
        Err(message) => return NestedRepositories::Refused { message, copies: Vec::<SavedCopy>::new() },
    };
    if repositories.is_empty() {
        return NestedRepositories::Nothing;
    }
    let milliseconds: i64 = now_milliseconds();
    match set_nested_repositories_aside(common_dir, root, repositories.as_slice(), milliseconds) {
        Ok(copies) => {
            let unpruned: String = prune_after_save(common_dir, copies.as_slice(), milliseconds, discarded_limits);
            return NestedRepositories::Saved {
                notice: format!("{}{unpruned}", saved_nested_repository_notice(copies.as_slice())),
                copies,
            };
        }
        Err(stopped) => {
            let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
            return NestedRepositories::Refused {
                message: nested_repository_not_saved_message(NotSaved {
                    repositories: repositories.as_slice(),
                    failure: &stopped.failure,
                    saved_root: saved_root.as_path(),
                    still_moved: stopped.still_moved.as_slice(),
                }),
                copies: stopped.still_moved,
            };
        }
    }
}

/// What:
///  Every part of the decision to let one form run.
///  A `struct` is a record with named fields.
/// Why:
///   The saved nested repositories of a clean are kept at the one point a form is let through,
///  and both such points hold the same seven things;
///  one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowedForm = { facts; command; location; region; seen; dirty; discardedLimits; notices };
/// ```
pub(crate) struct AllowedForm<'a> {
    /// The facts every question is asked.
    pub(crate) facts: &'a mut dyn RepositoryFacts,
    /// The guarded form being let through.
    pub(crate) command: GuardedCommand,
    /// Where the command runs.
    pub(crate) location: &'a RepositoryLocation,
    /// The tokens after the subcommand.
    pub(crate) region: &'a [OsString],
    /// The command as Git will run it,
    /// aliases included.
    pub(crate) seen: &'a [OsString],
    /// What the policy measured,
    ///  or nothing when it asked no question.
    pub(crate) dirty: Option<bool>,
    /// The age and size limits of the folder of saved copies.
    pub(crate) discarded_limits: Limits,
    /// The notices the person reads before Git runs.
    pub(crate) notices: &'a mut Vec<String>,
    /// The facts a record carries for a repository the allowlist exempts,
    ///  empty for every other repository.
    pub(crate) exemption: &'a [AuditFact],
    /// The copies an exempt repository's own save made before this form,
    ///  empty when there was none or when the repository is not exempt.
    pub(crate) saved_by_exemption: &'a [SavedCopy],
}

/// What:
///  Let one form run,
/// keeping every nested repository a clean would delete first,
///  and leave the one audit line that says what happened.
/// Why:
///   The clean forms are the only ones whose second force removes another repository;
///  every other form reaches the same allowed line it always left,
///  and a refused save stops the command before Git starts,
///  with the worktree exactly as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function allowADiscardingForm(form: AllowedForm): PolicyOutcome;
/// ```
pub(crate) fn allow_a_discarding_form(form: AllowedForm<'_>) -> PolicyOutcome {
    if matches!(form.command, GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly) {
        match keep_the_nested_repositories_a_clean_would_delete(form.facts, form.location, form.region, form.discarded_limits) {
            NestedRepositories::Nothing => {}
            NestedRepositories::Saved { copies, notice } => {
                // Every copy this decision made is named in its one line, the exemption's own among them.
                let mut places: Vec<String> = copy_places(copies.as_slice());
                places.extend(copy_places(form.saved_by_exemption));
                form.notices.push(notice);
                record_audit_with_copies_and_facts(
                    &form.location.identity,
                    form.command,
                    AuditOutcome::SavedNestedRepository,
                    form.seen,
                    form.dirty,
                    places.as_slice(),
                    form.exemption,
                );
                return clean();
            }
            NestedRepositories::Refused { message, copies } => {
                let places: Vec<String> = copy_places(copies.as_slice());
                // A rejection that left a repository under the saved copies names where it is;
                //  one that left none is a worktree nobody can tell apart from the one before the command.
                if places.is_empty() {
                    record_audit_with_facts(&form.location.identity, form.command, AuditOutcome::Rejected, form.seen, None, form.exemption);
                } else {
                    record_audit_with_copies_and_facts(
                        &form.location.identity,
                        form.command,
                        AuditOutcome::Rejected,
                        form.seen,
                        None,
                        places.as_slice(),
                        form.exemption,
                    );
                }
                return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
            }
        }
    }
    // An exempt repository's line says the exemption let the form through, and names the copies it made:
    //  nothing was at risk, or what was at risk is on disk beside the audit log.
    if !form.saved_by_exemption.is_empty() {
        let places: Vec<String> = copy_places(form.saved_by_exemption);
        record_audit_with_copies_and_facts(
            &form.location.identity,
            form.command,
            AuditOutcome::SavedExempt,
            form.seen,
            form.dirty,
            places.as_slice(),
            form.exemption,
        );
        return clean();
    }
    if !form.exemption.is_empty() {
        record_audit_with_facts(&form.location.identity, form.command, AuditOutcome::Exempt, form.seen, form.dirty, form.exemption);
        return clean();
    }
    record_audit(&form.location.identity, form.command, AuditOutcome::Allowed, form.seen, form.dirty);
    return clean();
}
