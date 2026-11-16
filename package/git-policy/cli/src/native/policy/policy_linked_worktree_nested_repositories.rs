//! What:
//!  What a clean in a linked worktree takes when it deletes a nested Git repository,
//!  and the one audit line that names it,
//!  beside the one place a discarding form is refused or let through.
//! Why:
//!  Git's clean removes a nested repository whole,
//!  with its own history and its uncommitted files,
//!  at a second `-f` for a repository with a `.git` entry and at one force for a bare one,
//!  and the outer worktree reports it as one ignored directory,
//!  so nothing else the policy reads can see the work inside it.
//!  The owner decided on 2026-11-15 that such a clean deletes the repository as real Git does
//!  and that the wrapper logs the command and what went,
//!  with no saved copy
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "Owner decisions in force",
//!  "Answers on waking, third round"):
//!  a copy is a thing the person makes,
//!  and the line is what a later reader uses to attribute the loss.
//!  The reading below therefore asks Git what the clean would delete at the force count the person
//!  typed,
//!  looks inside those paths and nowhere else,
//!  and writes the repositories it finds into the record before Git starts.
//!  The file also holds the two points a discarding form is let through,
//!  because both must ask this question before Git starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const taken = nestedRepositoriesACleanTakes(facts, location, region);
//! // recordAudit({ outcome: 'logged', facts: [{ kind: 'list', name: 'nestedRepositories', value: taken }] });
//! ```

/// The builder of the clean's own dry run,
///  and the parsed facts that decide whether the question applies.
use crate::command_clean::{clean_dry_run_arguments, parse_clean_region};
/// Which nested Git repositories that answer says the clean would delete.
use crate::command_clean_nested_repositories::nested_repositories_the_clean_deletes;
/// The directory a command runs in,
///  which a relative path of a clean is read from.
use crate::command_run_directory::command_run_directory;
/// The record of every decision,
///  and the named facts a line carries.
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_facts};
use crate::discard_audit_facts::AuditFact;
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// Every worktree entry a form would write over or delete,
///  which nothing else holds.
use crate::policy_linked_worktree_refused_overwrite::{
    EntriesInTheWay, WrittenOverRequest, entries_a_form_would_write_over,
};
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
use crate::repository_location::RepositoryLocation;
use crate::policy_checks::{clean, rejected};
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE};
/// The refusal that names those entries and the option that runs the command anyway.
use crate::rule_linked_worktree_refused_overwrite_messages::{
    Overwritten, overwritten_refusal_message, submodule_changes_refusal_message, tree_reading_failed_message,
};
/// The Git directories of an identity,
///  for naming the worktree the paths are below.
use crate::worktree_identity::worktree_root;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// What:
///  The name the record gives the repositories a clean takes.
/// Why:
///  A reader looking for what a clean deleted finds one list under one name,
///  and the line says in its own field that it was written before Git ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NESTED_REPOSITORIES = 'nestedRepositories';
/// ```
pub const NESTED_REPOSITORIES: &str = "nestedRepositories";

/// What:
///  Every repository one clean in this worktree takes,
///  as Git named it below the worktree root,
///  or the sentence saying why the reading could not name them.
///  `&[OsString]` borrows the clean's own tokens after the subcommand.
/// Why:
///  Git's own dry run is what says which paths the clean deletes,
///  and the walk inside them is what finds the repositories Git's answer cannot name
///  (`is_git_directory`'s bare shape included).
///  A clean that deletes nothing,
///  one Git refuses itself,
///  and an interactive clean whose menu is answered after Git starts are all no list:
///  the first two delete nothing and the third's outcome is the person's.
///  A clean Git may refuse for want of a force is asked about `clean.requireForce` first,
///  because the wrapper must not name repositories a command Git never runs deletes.
///  A dry run that did not answer,
///  a directory the look could not read and a name that could not be read are failures and never
///  "nothing goes":
///  a quiet line about a repository that went is worse than a stopped command,
///  and the sentence says which reading failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedRepositoriesACleanTakes(facts, location, ['-f', '-f', '-d', '-X']) === ['nested'];
/// ```
fn nested_repositories_a_clean_takes(
    facts: &mut dyn RepositoryFacts,
    location: &RepositoryLocation,
    region: &[OsString],
) -> Result<Vec<Vec<u8>>, String> {
    // What:
    //  The forms that delete nothing, or whose deletion is the person's own choice in a menu.
    // Why:
    //  `-x` together with `-X` is Git's own refusal before it reads anything (`cmd_clean`),
    //  and an interactive clean answers its menu after Git starts,
    //  so neither has a set of repositories this reading can name before the command runs.
    let Ok(parsed) = parse_clean_region(region, &[]) else {
        return Ok(Vec::<Vec<u8>>::new());
    };
    if !parsed.deletes_ignored || parsed.both_ignored_modes || parsed.interactive {
        return Ok(Vec::<Vec<u8>>::new());
    }
    // What:
    //  A clean that cannot delete a directory takes no repository, so Git is not asked.
    // Why:
    //  Without `-d` Git removes no directory at all:
    //  `git clean -f -X` in a worktree whose only ignored entry is the directory `ignored/` printed nothing and
    //  left the directory and its file in place (measured on this host with native Git 2.56.0, 2026-11-15).
    //  A repository is a directory, so the dry run's answer would name nothing and the record needs it not,
    //  and that start is the whole cost this reading adds to a clean over `main`
    //  (`keep/cost-a-clean-by-its-steps.ts`, this run's scratch folder).
    //  A positional path is still asked: Git removes a directory a pathspec names without `-d`
    //  (`git clean -ffX -- vendor`, measured earlier and recorded in "Discarding commands").
    if !parsed.descends && parsed.paths.is_empty() {
        return Ok(Vec::<Vec<u8>>::new());
    }
    // What:
    //  A clean with no force under the default `clean.requireForce` is Git's own refusal.
    // Why:
    //  `cmd_clean` refuses it before it deletes anything
    //  (`if (!interactive && !dry_run && force == 0 && clean_require_force) die(...)`),
    //  so the dry run that follows cannot tell this refusal from a clean that runs:
    //  `git clean -Xd` exits 128 and keeps the bare repository, while `git clean -n -X` lists it
    //  without a word (measured on this host with native Git 2.56.0, 2026-11-14).
    if parsed.forces == 0 {
        match facts.clean_require_force() {
            Ok(true) => return Ok(Vec::<Vec<u8>>::new()),
            Ok(false) => {}
            // The answer that was not read is no proof that the clean runs, and a line about
            //  repositories the command never reached would be a lie either way.
            Err(message) => return Err(message),
        }
    }
    let Some(root) = worktree_root(&location.identity) else {
        return Ok(Vec::<Vec<u8>>::new());
    };
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    // A directory that cannot be named stops the reading: a dry run started anywhere else could hide
    //  a repository, and the line would then be silent about it.
    let run_in: PathBuf = command_run_directory(prefix.as_slice())?;
    // The names are printed relative to the directory the dry run started in,
    //  so that directory's own prefix is the one read here.
    let command_prefix: Vec<u8> =
        crate::command_run_directory::run_directory_prefix(root, run_in.as_path(), location.prefix.as_slice());
    let Some(as_typed) = clean_dry_run_arguments(prefix.as_slice(), &parsed, run_in.as_path()) else {
        // A region Git would refuse deletes nothing, and Git refuses it in its own words when it runs.
        return Ok(Vec::<Vec<u8>>::new());
    };
    let deleted: Vec<Vec<u8>> = facts.clean_deleted_paths(as_typed)?;
    return nested_repositories_the_clean_deletes(deleted.as_slice(), root, command_prefix.as_slice());
}

/// What:
///  Record what one clean takes,
///  naming each repository Git will delete below the worktree root,
///  or refuse the clean when the reading that would name them failed.
///  `&[OsString]` borrows the command as Git receives it.
/// Why:
///  The line is written before Git runs,
///  so a reader sees what was about to go even when the command is killed half-way,
///  and its `writtenBeforeGitRuns` field says so in words a reader cannot take for the result.
///  A reading that failed stops the command instead:
///  a clean whose line cannot name the repositories it takes would delete them silently,
///  and nothing else the policy reads can see inside a nested repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordWhatACleanTakes(facts, location, form, seen, ['nested']) -> clean();
/// ```
fn record_what_a_clean_takes(
    form: &AllowedForm<'_>,
    taken: &[Vec<u8>],
    seen: &[OsString],
) -> PolicyOutcome {
    let named: Vec<String> = taken.iter().map(|path| return String::from_utf8_lossy(path).into_owned()).collect();
    let facts: [AuditFact; 1] = [AuditFact::List(NESTED_REPOSITORIES, named)];
    record_audit_with_facts(
        &form.location.identity,
        form.command,
        AuditOutcome::Logged,
        seen,
        form.dirty,
        facts.as_slice(),
    );
    return clean();
}

/// What:
///  Every part of the decision to let one form run.
///  A `struct` is a record with named fields.
/// Why:
///   Both points a discarding form is let through hold the same things;
///  one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowedForm = { facts; command; location; region; seen; shellDirectory; dirty };
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
///  Refuse a form that writes over an entry nothing else holds,
///  record what a clean takes,
///  and leave the one audit line that says what happened.
/// Why:
///  The entries a forced tree write or a path form would lose are refused before Git starts,
///  and the refusal names them and the option that runs the command as Git runs it;
///  the clean forms are the only ones whose second force removes another repository,
///  and that one is named in the record and the clean runs as typed,
///  so every other form reaches the same allowed line it always left,
///  and a refusal stops the command with the worktree exactly as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// allowADiscardingForm(form): PolicyOutcome;
/// ```
pub(crate) fn allow_a_discarding_form(form: AllowedForm<'_>) -> PolicyOutcome {
    // Every form that writes a tree or a path of one is refused while an entry nothing else holds stands in its way.
    let in_the_way: EntriesInTheWay = match entries_a_form_would_write_over(WrittenOverRequest {
        facts: form.facts,
        command: form.command,
        region: form.region,
    }) {
        Ok(found) => found,
        // A listing Git could not read says nothing about what the command writes over, so it stops the command.
        //  The sentence names what could not be read and prints the command the opt-out runs.
        Err(cause) => {
            record_audit(&form.location.identity, form.command, AuditOutcome::Rejected, form.seen, form.dirty);
            return rejected(
                LINKED_WORKTREE_REQUIRED_CODE,
                tree_reading_failed_message(form.command, form.region, cause.as_str()),
            );
        }
    };
    if !in_the_way.inside_submodules.is_empty() {
        record_audit(&form.location.identity, form.command, AuditOutcome::Rejected, form.seen, form.dirty);
        return rejected(
            LINKED_WORKTREE_REQUIRED_CODE,
            submodule_changes_refusal_message(Overwritten {
                command: form.command,
                region: form.region,
                paths: in_the_way.inside_submodules.as_slice(),
            }),
        );
    }
    if !in_the_way.entries.is_empty() {
        record_audit(&form.location.identity, form.command, AuditOutcome::Rejected, form.seen, form.dirty);
        return rejected(
            LINKED_WORKTREE_REQUIRED_CODE,
            overwritten_refusal_message(Overwritten {
                command: form.command,
                region: form.region,
                paths: in_the_way.entries.as_slice(),
            }),
        );
    }
    if matches!(form.command, GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly) {
        match nested_repositories_a_clean_takes(form.facts, form.location, form.region) {
            Ok(taken) if taken.is_empty() => {}
            Ok(taken) => return record_what_a_clean_takes(&form, taken.as_slice(), form.seen),
            Err(message) => {
                record_audit(&form.location.identity, form.command, AuditOutcome::Rejected, form.seen, None);
                return rejected(LINKED_WORKTREE_REQUIRED_CODE, message);
            }
        }
    }
    record_audit(&form.location.identity, form.command, AuditOutcome::Allowed, form.seen, form.dirty);
    return clean();
}

/// What a clean takes, and the line that names it.
#[cfg(test)]
#[path = "policy_linked_worktree_nested_repositories_tests.rs"]
mod tests;
