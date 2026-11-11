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
use crate::command_clean::{
    clean_dry_run_arguments, clean_dry_run_arguments_without_exclusions, paths_only_the_exclusions_add,
};
/// Which worktree a command targets,
///  for the linked-worktree rule.
use crate::effective_target::{
    EffectiveTarget, ToolCacheDirectory, classify_effective_target,
    matching_allowed_worktree_dir_for_repository,
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
use crate::rule_shared_remote::{SHARED_REMOTE_SETTINGS_CODE, SharedRemoteChange};
/// The text that refusal prints.
use crate::rule_shared_remote_messages::shared_remote_settings_message;
/// Whether a Git directory has no index file yet.
use crate::rule_worktree_dirty::index_file_is_absent;
/// The common Git directory,
///  for naming the file every worktree shares,
///  and the worktree top level,
///  for the directory a command runs in.
use crate::worktree_identity::{common_directory, worktree_root};
/// The reader that turns the bytes Git printed into a path,
///  whose form differs per platform.
use crate::git_metadata::path_from_git_bytes;
/// The path with every link resolved as far as it exists,
///  for comparing two spellings of one file.
use crate::repository_link_paths::resolve_as_far_as_it_exists;
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
///  The directory the command runs in,
///  as Git resolved it:
///  the worktree top level plus the path Git
///  reported from it to the working directory.
///  `&RepositoryLocation` borrows the reading that names both.
/// Why:
///   A relative `--file` path is read from where the command runs,
///  and `git -C` moves that
///       directory before the command reads it;
///  Git's own `--show-prefix` already carries that move.
///  A command below the worktree root is
///       usually refused by the require-root policy before this rule reads it,
///  and its hatch is what lets such
///       a command reach here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commandDirectory = (location) => location.worktreeRoot ? join(location.worktreeRoot, location.prefix) : undefined;
/// ```
fn command_directory(location: &RepositoryLocation) -> Option<PathBuf> {
    let root: &Path = worktree_root(&location.identity)?;
    // A command at the top level reports no prefix at all, which is the root itself.
    return Some(match path_from_git_bytes(location.prefix.as_slice()) {
        Some(prefix) => root.join(prefix),
        None => root.to_path_buf(),
    });
}

/// What:
///  Whether the path a `--file` write names is the configuration file every worktree
///  shares.
///  `&Path` borrows the named path,
///  the directory the command runs in and the shared file.
/// Why:
///   The shared file is one of the paths `--file` takes,
///  and naming it is not naming another file;
///  a
///       relative path,
///  one through `..` and one through a link are the same file,
///  so both spellings are
///       resolved the way the filesystem does before they are compared.
///  A path that names neither the file nor a
///       link to it is a file of the command's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheSharedFile = (named, directory, shared) => resolve(named.startsWith('/') ? named : join(directory, named)) === resolve(shared);
/// ```
fn names_the_shared_file(named: &Path, directory: &Path, shared_file: &Path) -> bool {
    let spelled: PathBuf = if named.is_absolute() {
        named.to_path_buf()
    } else {
        directory.join(named)
    };
    return resolve_as_far_as_it_exists(spelled.as_path()) == resolve_as_far_as_it_exists(shared_file);
}

/// What:
///  The refusal of a command that would change the remote settings every worktree of the
///  repository shares,
///  when the command runs in a linked worktree.
///  `&SharedRemoteChange` borrows the change the guard decided about.
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
///  A
///       `--file` write is this rule's only when its path resolves to that same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseSharedRemoteSettings(facts, change, allowedDirs): PolicyOutcome;
/// ```
fn refuse_shared_remote_settings(
    facts: &mut dyn RepositoryFacts,
    change: &SharedRemoteChange,
    allowed_worktree_dirs: &[ToolCacheDirectory],
) -> PolicyOutcome {
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    let target: EffectiveTarget =
        classify_effective_target(&location.identity, allowed_worktree_dirs);
    // Only a linked worktree shares its configuration with another checkout.
    if target != EffectiveTarget::LinkedWorktree {
        return clean();
    }
    // The configuration file Git writes for the repository: `$GIT_COMMON_DIR/config`.
    let shared_file: PathBuf = match common_directory(&location.identity) {
        Some(directory) => directory.join("config"),
        None => {
            return fact_unavailable(String::from(
                "monochromatic-git-policy-cli could not name the configuration file this repository's worktrees \
                 share, so the command was not judged.",
            ));
        }
    };
    // A `--file` write names its own file: only the path that resolves to this repository's shared file is one.
    if let Some(named) = &change.named_file {
        let Some(directory) = command_directory(&location) else {
            return clean();
        };
        if !names_the_shared_file(named.as_path(), directory.as_path(), shared_file.as_path()) {
            return clean();
        }
    }
    let main_checkout: PathBuf = match facts.main_worktree_root() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    return rejected(
        SHARED_REMOTE_SETTINGS_CODE,
        shared_remote_settings_message(&change.setting, shared_file.as_path(), main_checkout.as_path()),
    );
}

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
        // The exemption is judged on the repository the command runs in, so the notice names the entry that held both
        //  its Git directory and its worktree top.
        let exemption: Option<&ToolCacheDirectory> =
            matching_allowed_worktree_dir_for_repository(&location.identity, allowed_worktree_dirs);
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
    // Which question this form asks, and the tree a forced form writes, both come from the tokens after it.
    let asked: Option<DirtyQuestion> = dirty_question(command, region);
    // A form that loses nothing this policy protects leaves its one line and runs: an ignored file is not a reason
    //  to refuse a clean in a linked worktree (the owner's answer of 2026-11-10, "Allow and log").
    let Some(question) = asked else {
        record_audit(&location.identity, command, AuditOutcome::Allowed, seen, None);
        return clean();
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
