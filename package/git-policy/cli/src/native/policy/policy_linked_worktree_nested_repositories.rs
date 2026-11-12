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
use crate::command_clean::{clean_dry_run_arguments, parse_clean_region};
/// Which nested Git repositories that answer says the clean would delete.
use crate::command_clean_nested_repositories::nested_repositories_the_clean_deletes;
/// The bound on the folder of saved copies applied after a save,
///  and the place saved copies live.
use crate::conflict_side_guard::prune_after_save;
use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy};
/// The record of every decision.
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_copies};
/// The limits of the folder of saved copies.
use crate::discarded_prune::Limits;
/// The move of a nested repository the clean would delete.
use crate::discarded_nested_repository::set_nested_repositories_aside;
/// Keeping every entry a form would overwrite,
///  and the one refusal such a save has.
use crate::policy_linked_worktree_saved_overwrite::{Overwritten, SaveRequest, keep_what_the_form_would_overwrite};
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// The linked-worktree check's answer to where a clean runs,
///  which both dry runs start in.
use crate::command_run_directory::command_run_directory;
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
use std::path::{Path, PathBuf};

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
///  The prefix Git writes before a name it prints,
///  as the path from the worktree root to the directory the reading started in,
///  or `location_prefix` when that directory is not below the root.
///  `&Path` borrows the root and the reading's directory,
///  and `&[u8]` the prefix the location query answered.
/// Why:
///   Git prints each name relative to the directory the command runs in,
///  and a clean a `!` alias runs starts at the top of the working tree,
///  so the prefix of the reading is the one that goes before those names,
///  which is not the caller's prefix when the caller stood below the top.
///  A reading whose directory is not below the worktree is one no name can be joined to,
///  so the prefix the invocation itself has is kept,
///  which is what every reading in the caller's own directory carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixOf(root: string, runIn: string, locationPrefix: Buffer): Buffer;
/// ```
fn prefix_of(root: &Path, run_in: &Path, location_prefix: &[u8]) -> Vec<u8> {
    let Ok(relative) = run_in.strip_prefix(root) else {
        return location_prefix.to_vec();
    };
    let mut prefix: Vec<u8> = Vec::<u8>::new();
    for component in relative.components() {
        prefix.extend_from_slice(component.as_os_str().as_encoded_bytes());
        prefix.push(b'/');
    }
    return prefix;
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
///  That reading carries the directory the clean runs in (`command_run_directory`)
///  and is started there,
///  so a repository above the directory the wrapper stands in is one it sees.
///  `Option<&Path>` is the directory Git runs the shell of a `!` alias in,
///  when the clean is the command that shell runs.
///  The one clean Git refuses itself,
///  `-x` written with `-X`,
///  is known from the region before any reading:
///  it deletes nothing,
///  so Git's own refusal is the answer the person gets.
///  A dry run that did not answer was read by nobody,
///  and a nonzero exit is no proof that the clean which runs afterwards refuses,
///  so that clean stops instead of running on a guess.
///  An interactive clean is not moved here at all:
///  its menu is answered after Git starts,
///  so the fixed transform keeps the repositories out of that menu by one force
///  (`rule_clean_interactive_nested_repositories.rs`),
///  and a repository moved before a menu is one the person never chose to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepTheNestedRepositoriesACleanWouldDelete(facts, location, region, shellDirectory, limits): NestedRepositories;
/// ```
fn keep_the_nested_repositories_a_clean_would_delete(
    facts: &mut dyn RepositoryFacts,
    location: &RepositoryLocation,
    region: &[OsString],
    shell_directory: Option<&Path>,
    discarded_limits: Limits,
) -> NestedRepositories {
    // A clean removes a nested repository only with a second `-f` (`builtin/clean.c`: `if (force > 1) rm_flags = 0`).
    //  Every other clean is Git's own business, and one force is Git's own protection of the repository.
    //  `-d` is not part of the question: it is how a clean reaches an untracked directory, while a pathspec names one
    //  Git deletes without it (`clean -ffX -- vendor`), so a reading that waited for `-d` let that clean delete a
    //  repository unsaved. What the clean really deletes is Git's own dry run, read below.
    let Ok(parsed) = parse_clean_region(region, &[]) else {
        return NestedRepositories::Nothing;
    };
    if !parsed.deletes_ignored || parsed.forces < 2 {
        return NestedRepositories::Nothing;
    }
    // Git refuses `-x` together with `-X` before it reads anything (`cmd_clean`), so this clean deletes nothing and
    //  Git's own refusal is the answer the person gets. A dry run would only say so with a nonzero exit, which is no
    //  proof about the clean that runs afterwards.
    if parsed.both_ignored_modes {
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
    // A directory that cannot be named stops the clean: a dry run started anywhere else could hide a repository.
    let run_in: PathBuf = match command_run_directory(prefix.as_slice(), shell_directory) {
        Ok(found) => found,
        Err(message) => return NestedRepositories::Refused { message, copies: Vec::<SavedCopy>::new() },
    };
    let Some(as_typed) = clean_dry_run_arguments(prefix.as_slice(), region, run_in.as_path()) else {
        return NestedRepositories::Nothing;
    };
    let deleted: Vec<Vec<u8>> = match facts.clean_deleted_paths(as_typed) {
        Ok(answer) => answer,
        // What the clean would delete was not read, so a repository it would delete was not seen; a nonzero exit is
        //  no proof that the later clean refuses, and running on it deleted a repository no copy held.
        Err(message) => return NestedRepositories::Refused { message, copies: Vec::<SavedCopy>::new() },
    };
    // The names are printed relative to the directory the dry run started in, which is not the caller's own directory
    //  when Git runs the clean inside a `!` alias's shell, so that directory's own prefix is the one read here.
    let command_prefix: Vec<u8> = prefix_of(&root, run_in.as_path(), location.prefix.as_slice());
    let repositories: Vec<Vec<u8>> = match nested_repositories_the_clean_deletes(deleted.as_slice(), root, command_prefix.as_slice()) {
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
///  and both such points hold the same eight things;
///  one record keeps them from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowedForm = { facts; command; location; region; seen; shellDirectory; dirty; discardedLimits; notices };
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
    /// The directory Git runs the shell of a `!` alias in,
    ///  when the clean is the command that shell runs;
    ///  nothing for a direct command and an ordinary alias.
    pub(crate) shell_directory: Option<&'a Path>,
    /// What the policy measured,
    ///  or nothing when it asked no question.
    pub(crate) dirty: Option<bool>,
    /// The age and size limits of the folder of saved copies.
    pub(crate) discarded_limits: Limits,
    /// The notices the person reads before Git runs.
    pub(crate) notices: &'a mut Vec<String>,
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
    let milliseconds: i64 = now_milliseconds();
    // Every form that writes a tree or a path of one keeps what stands in its way before Git runs.
    let overlap: Overwritten = keep_what_the_form_would_overwrite(SaveRequest {
        facts: form.facts,
        command: form.command,
        location: form.location,
        region: form.region,
        limits: form.discarded_limits,
        milliseconds,
    });
    match overlap {
        Overwritten::Nothing => {}
        Overwritten::Saved { copies, notice } => {
            form.notices.push(notice);
            record_audit_with_copies(
                &form.location.identity,
                form.command,
                AuditOutcome::SavedOverwritten,
                form.seen,
                form.dirty,
                places_of(copies.as_slice()).as_slice(),
            );
            return clean();
        }
        Overwritten::Refused { message, copies } => {
            return refuse_a_stopped_save(form.location, form.command, form.seen, &message, copies.as_slice());
        }
    }
    if matches!(form.command, GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly) {
        match keep_the_nested_repositories_a_clean_would_delete(
            form.facts,
            form.location,
            form.region,
            form.shell_directory,
            form.discarded_limits,
        ) {
            NestedRepositories::Nothing => {}
            NestedRepositories::Saved { copies, notice } => {
                form.notices.push(notice);
                record_audit_with_copies(
                    &form.location.identity,
                    form.command,
                    AuditOutcome::SavedNestedRepository,
                    form.seen,
                    form.dirty,
                    places_of(copies.as_slice()).as_slice(),
                );
                return clean();
            }
            NestedRepositories::Refused { message, copies } => {
                return refuse_a_stopped_save(form.location, form.command, form.seen, &message, copies.as_slice());
            }
        }
    }
    record_audit(&form.location.identity, form.command, AuditOutcome::Allowed, form.seen, form.dirty);
    return clean();
}

/// What:
///  Each saved copy as `<path> => <place>`,
///  which is how an audit line names where a copy is.
/// Why:
///   A copy with no place is one the save found nothing at,
///  and the record has to say that rather than leave the field to look like a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placesOf = (copies) => copies.map((c) => `${c.path} => ${c.savedTo ?? 'none'}`);
/// ```
fn places_of(copies: &[SavedCopy]) -> Vec<String> {
    let mut places: Vec<String> = Vec::<String>::new();
    for copy in copies {
        let saved_to: String = match copy.saved_to.as_ref() {
            Some(place) => place.display().to_string(),
            None => String::from("none"),
        };
        places.push(format!("{} => {}", copy.path, saved_to));
    }
    return places;
}

/// What:
///  Stop a command whose save could not keep everything,
///  and record what the save had already kept.
/// Why:
///  A rejection that left copies below the saved folder names where they are;
///  one that left none is a worktree nobody can tell apart from the one before the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseAStoppedSave(location, command, seen, message, copies): PolicyOutcome;
/// ```
fn refuse_a_stopped_save(
    location: &RepositoryLocation,
    command: GuardedCommand,
    seen: &[OsString],
    message: &str,
    copies: &[SavedCopy],
) -> PolicyOutcome {
    let places: Vec<String> = places_of(copies);
    if places.is_empty() {
        record_audit(&location.identity, command, AuditOutcome::Rejected, seen, None);
    } else {
        record_audit_with_copies(&location.identity, command, AuditOutcome::Rejected, seen, None, places.as_slice());
    }
    return rejected(LINKED_WORKTREE_REQUIRED_CODE, message.to_string());
}
