//! What:
//!  Refusing a clean that would delete a nested Git repository or worktree in a linked worktree,
//!  and naming every one it would destroy.
//! Why:
//!  Git's clean with a second `-f` removes a nested repository whole,
//!  with its own history and its uncommitted files,
//!  and the outer worktree reports it as one ignored directory,
//!  so nothing else the policy reads can see the work inside it.
//!  The owner's rule (2026-11-12) is that a command which could destroy a person's data is refused,
//!  naming what it would destroy,
//!  and that the person who means it passes the guard's opt-out and the command then runs as real Git
//!  runs it;
//!  the wrapper keeps no copy of the work.
//!  Git itself says which paths the clean removes,
//!  and the repositories are the directories below those paths that are a repository of their own.
//!  The file also holds the two points a discarding form is let through,
//!  because both must ask this question before Git starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const refusal = refuseTheNestedRepositoriesACleanWouldDelete(facts, location, region);
//! ```

/// The builder of the clean's own dry run,
///  and the parsed facts that decide whether the question applies.
use crate::command_clean::{clean_dry_run_arguments, parse_clean_region};
/// Which nested Git repositories that answer says the clean would delete.
use crate::command_clean_nested_repositories::nested_repositories_the_clean_deletes;
/// The record of every decision,
///  and the named fact that says which repositories the refusal named.
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_facts};
use crate::discard_audit_facts::AuditFact;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
use crate::policy_checks::{clean, rejected};
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE};
/// The refusal that names the repositories and the opt-out that runs the clean as Git runs it.
use crate::rule_linked_worktree_nested_repository_messages::nested_repositories_would_be_deleted_message;
/// The worktree the paths Git printed are below.
use crate::worktree_identity::worktree_root;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  What the question about a clean's nested repositories left to do.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The two answers are handled differently:
///  nothing to refuse lets the command run as it always did,
///  and a refused command stops with nothing changed and one line saying which repositories it named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NestedRepositories = { kind: 'nothing' } | { kind: 'refused'; message; repositories };
/// ```
enum NestedRepositories {
    /// The clean deletes no nested repository,
    /// or Git deletes none of what it names.
    Nothing,
    /// A repository would be deleted,
    ///  or what the clean would delete could not be read,
    ///  and the sentence says why and what to do.
    Refused {
        /// What the person reads instead of the clean running.
        message: String,
        /// The repositories the refusal named,
        ///  for the audit line;
        ///  empty when the reading itself failed and named none.
        repositories: Vec<Vec<u8>>,
    },
}

/// What:
///  Refuse a clean that would delete a nested Git repository,
/// naming each one,
///  before it runs.
///  `&RepositoryLocation` borrows where the command runs and which worktree it runs in.
/// Why:
///   A second `-f` removes a nested repository with its own history and its uncommitted files,
///  which no other copy of the surrounding repository holds;
///  the owner's rule is to refuse such a command and name what it would destroy,
///  and to run it as real Git runs it once the person passes `--no-enforce-worktree`.
///  Git itself says which paths the clean removes,
///  and the repository the clean deletes is the one below one of those paths,
///  found by looking inside the names Git printed and nowhere else.
///  The one clean Git refuses itself,
///  `-x` written with `-X`,
///  is known from the region before any reading:
///  it deletes nothing,
///  so Git's own refusal is the answer the person gets.
///  A dry run that did not answer,
///  and a nonzero exit,
///  leave nothing to name and no proof that the clean which runs afterwards refuses,
///  so that clean stops instead of running on a guess.
///  An interactive clean is refused here too:
///  its menu is answered after Git starts,
///  and nothing the wrapper does beforehand stops the person from picking a repository
///  it would then delete with no copy anywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseTheNestedRepositoriesACleanWouldDelete(facts, location, region): NestedRepositories;
/// ```
fn refuse_the_nested_repositories_a_clean_would_delete(
    facts: &mut dyn RepositoryFacts,
    location: &RepositoryLocation,
    region: &[OsString],
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
    // Git refuses `-x` together with `-X` before it reads anything (`cmd_clean`), so this clean deletes nothing and
    //  Git's own refusal is the answer the person gets. A dry run would only say so with a nonzero exit, which is no
    //  proof about the clean that runs afterwards.
    if parsed.both_ignored_modes {
        return NestedRepositories::Nothing;
    }
    let Some(root) = worktree_root(&location.identity) else {
        return NestedRepositories::Nothing;
    };
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    let Some(as_typed) = clean_dry_run_arguments(prefix.as_slice(), region) else {
        return NestedRepositories::Nothing;
    };
    let deleted: Vec<Vec<u8>> = match facts.clean_deleted_paths(as_typed) {
        Ok(answer) => answer,
        // What the clean would delete was not read, so a repository it would delete was not named; a nonzero exit is
        //  no proof that the later clean refuses, and running on it deleted a repository nobody was told about.
        Err(message) => {
            return NestedRepositories::Refused { message, repositories: Vec::<Vec<u8>>::new() };
        }
    };
    let repositories: Vec<Vec<u8>> = match nested_repositories_the_clean_deletes(deleted.as_slice(), root, location.prefix.as_slice()) {
        Ok(found) => found,
        Err(message) => {
            return NestedRepositories::Refused { message, repositories: Vec::<Vec<u8>>::new() };
        }
    };
    if repositories.is_empty() {
        return NestedRepositories::Nothing;
    }
    return NestedRepositories::Refused {
        message: nested_repositories_would_be_deleted_message(repositories.as_slice()),
        repositories,
    };
}

/// What:
///  Every part of the decision to let one form run.
///  A `struct` is a record with named fields.
/// Why:
///   The question about a clean's nested repositories is asked at the one point a form is let through,
///  and both such points hold the same five things;
///  one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowedForm = { facts; command; location; region; seen; dirty };
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
}

/// What:
///  Let one form run,
/// refusing a clean that would delete a nested Git repository,
///  and leave the one audit line that says what happened.
/// Why:
///   The clean forms are the only ones whose second force removes another repository;
///  every other form reaches the same allowed line it always left,
///  and a refused clean stops the command before Git starts,
///  with the worktree exactly as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function allowADiscardingForm(form: AllowedForm): PolicyOutcome;
/// ```
pub(crate) fn allow_a_discarding_form(form: AllowedForm<'_>) -> PolicyOutcome {
    if matches!(form.command, GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly) {
        match refuse_the_nested_repositories_a_clean_would_delete(form.facts, form.location, form.region) {
            NestedRepositories::Nothing => {}
            NestedRepositories::Refused { message, repositories } => {
                // The line names the repositories the refusal named, so a later reader sees what the command would
                //  have deleted without reading the person's terminal.
                let mut why: Vec<AuditFact> = Vec::<AuditFact>::new();
                if !repositories.is_empty() {
                    let mut named: Vec<String> = Vec::<String>::new();
                    for path in &repositories {
                        named.push(String::from_utf8_lossy(path).into_owned());
                    }
                    why.push(AuditFact::Text("nestedRepositories", named.join(" ")));
                }
                record_audit_with_facts(
                    &form.location.identity,
                    form.command,
                    AuditOutcome::Rejected,
                    form.seen,
                    form.dirty,
                    why.as_slice(),
                );
                return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
            }
        }
    }
    record_audit(&form.location.identity, form.command, AuditOutcome::Allowed, form.seen, form.dirty);
    return clean();
}
