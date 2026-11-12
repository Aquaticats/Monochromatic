//! What:
//!  The rule an exempt repository keeps around a form that would destroy work:
//!  the form is refused,
//!  and the refusal names every untracked entry and nested Git repository it would lose.
//! Why:
//!  The exemption was decided for the worktree rule
//!  (the owner's answer of 2026-11-11,
//!  `#3024`:
//!  a vendored clone under the Cargo build directory must not be refused),
//!  and it never decided that a command may destroy a file that exists nowhere else.
//!  The owner's rule
//!  (2026-11-12)
//!  is that such a command is refused with the guard's opt-out as the way through:
//!  `--no-enforce-worktree` runs it as Git runs it,
//!  and the wrapper keeps no copy.
//!  The exemption still covers the build step's own uncommitted changes,
//!  which is the case it was made for;
//!  it does not cover work Git did not put there,
//!  which no copy of the repository holds.
//!  The refusals are the ones an ordinary repository already makes
//!  (`ignored_file_message`,
//!  the clean's own dry run),
//!  and the one new reading is the untracked worktree entry that stands where the form writes
//!  (`repository_facts_paths_the_form_writes`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = refuseOrAllow({ facts, command, location, region, question, tree, exemption, dirty });
//! // clean() when nothing this guard protects is at risk, rejected(...) otherwise
//! ```

/// The builder of a clean's dry run and the sentence for a failed one.
use crate::command_clean::{UNREADABLE_CLEAN_DELETION, clean_dry_run_arguments, parse_clean_region};
/// The nested repositories a clean's dry run says it would delete,
///  and the paths that dry run names.
use crate::command_clean_nested_repositories::{nested_repositories_the_clean_deletes, removed_paths_from_dry_run};
/// The named facts a record of an exemption carries.
use crate::discard_audit_facts::AuditFact;
/// The record of every decision.
use crate::discard_audit::{AuditOutcome, record_audit_with_facts};
/// The engine's findings and outcomes.
use crate::policy_engine::PolicyOutcome;
/// The paths one form writes,
///  and the worktree entries that stand where they go.
use crate::policy_linked_worktree_exempt_written_paths::{Written, written_paths};
use crate::policy_checks::{clean, rejected};
/// The facts every question is asked.
use crate::repository_facts::RepositoryFacts;
/// The exempt directory the command's own environment named,
///  for the notice and the record.
use crate::effective_target::{ToolCacheDirectory, matching_allowed_worktree_dir_for_repository};
use crate::repository_facts_paths_the_form_writes::entries_in_the_way;
/// Where the command runs.
use crate::repository_location::RepositoryLocation;
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{
    DirtyQuestion, GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, TargetTree,
};
/// The notices an exempt decision can print,
///  and the refusals it reuses.
use crate::rule_linked_worktree_messages::{
    exempt_at_risk_message, exempt_unreadable_message, ignored_file_message, repository_stash_message,
};
/// The scope of a worktree status.
use crate::worktree_identity::{WorktreeIdentity, worktree_root};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  What one form would destroy that exists nowhere else.
///  A `struct` is a record with named fields.
/// Why:
///   The three classes are refused by different sentences:
///  an untracked entry was never in Git,
///  a nested repository holds its own history,
///  and an ignored file is one a tracked tree replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AtRisk = { untracked: string[]; nested: string[]; ignored: string[] };
/// ```
#[derive(Default)]
struct AtRisk {
    /// The untracked worktree entries that stand where the form writes,
    ///  or the ones a clean would delete.
    untracked: Vec<String>,
    /// The nested Git repositories a clean would delete.
    nested: Vec<String>,
    /// The ignored files whose paths the tree the form writes also tracks.
    ignored: Vec<String>,
}

impl AtRisk {
    /// Whether the form would destroy nothing this guard protects.
    fn is_empty(&self) -> bool {
        return self.untracked.is_empty() && self.nested.is_empty() && self.ignored.is_empty();
    }

    /// Every path this answer would lose,
    ///  for the record.
    fn all(&self) -> String {
        let mut every: Vec<String> = Vec::<String>::new();
        every.extend(self.untracked.iter().cloned());
        every.extend(self.nested.iter().cloned());
        every.extend(self.ignored.iter().cloned());
        return every.join(" ");
    }
}

/// What:
///  One exempt repository's discarding form,
///  with the facts it asks and the record it leaves.
///  A `struct` is a record with named fields.
/// Why:
///  Every guarded form in an exempt repository reaches this one place,
///  whether the worktree measurement found changes or not,
///  so the record and the refusal cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExemptForm = { facts; command; location; region; seen; question; tree; exemption; dirty; conflictNote };
/// ```
pub(crate) struct ExemptForm<'a> {
    /// The facts every question is asked.
    pub(crate) facts: &'a mut dyn RepositoryFacts,
    /// The guarded form about to run.
    pub(crate) command: GuardedCommand,
    /// Where the command runs.
    pub(crate) location: &'a RepositoryLocation,
    /// The tokens after the subcommand.
    pub(crate) region: &'a [OsString],
    /// The command as Git will run it,
    ///  aliases included.
    pub(crate) seen: &'a [OsString],
    /// The question the worktree measurement asked,
    ///  or nothing when the form asks none.
    pub(crate) question: Option<DirtyQuestion>,
    /// The tree the form writes,
    ///  whose tracked paths an ignored file is replaced at.
    pub(crate) tree: Option<&'a TargetTree>,
    /// The exemption the record names.
    pub(crate) exemption: &'a [AuditFact],
    /// What the worktree measurement answered,
    ///  for the record.
    pub(crate) dirty: Option<bool>,
    /// The sentence a rejection of the same line gets beside the exemption's own text.
    pub(crate) conflict_note: &'a str,
}

/// What:
///  Refuse an exempt repository's form when it would destroy work that exists nowhere else,
///  and let it run as Git runs it when it would not.
/// Why:
///   The refusal is the whole protection now that the wrapper keeps no copy,
///  so a reading that failed refuses too and never reads as nothing to protect;
///  a stash entry stays its own refusal,
///  because no copy of the repository holds it;
///  and a form with nothing at risk records the exemption and runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOrAllow(form: ExemptForm): PolicyOutcome;
/// ```
pub(crate) fn refuse_or_allow(form: ExemptForm<'_>) -> PolicyOutcome {
    let command: GuardedCommand = form.command;
    if matches!(form.question, Some(DirtyQuestion::RepositoryStash)) {
        record_audit_with_facts(&form.location.identity, command, AuditOutcome::Rejected, form.seen, form.dirty, form.exemption);
        return rejected(
            LINKED_WORKTREE_REQUIRED_CODE,
            format!("{}{}", repository_stash_message(command), form.conflict_note),
        );
    }
    let at_risk: AtRisk = match what_the_form_would_destroy(form.facts, form.command, form.region, form.tree, form.location) {
        Ok(found) => found,
        Err(message) => {
            record_audit_with_facts(&form.location.identity, command, AuditOutcome::Rejected, form.seen, form.dirty, form.exemption);
            return rejected(
                LINKED_WORKTREE_REQUIRED_CODE,
                format!("{}{}", exempt_unreadable_message(command, message.as_str()), form.conflict_note),
            );
        }
    };
    if at_risk.is_empty() {
        // Nothing this guard protects: the exemption lets the form run as Git runs it, and no copy is made.
        record_audit_with_facts(&form.location.identity, command, AuditOutcome::Exempt, form.seen, form.dirty, form.exemption);
        return clean();
    }
    let mut facts: Vec<AuditFact> = form.exemption.to_vec();
    facts.push(AuditFact::Text("wouldDestroy", at_risk.all()));
    record_audit_with_facts(&form.location.identity, command, AuditOutcome::Rejected, form.seen, form.dirty, facts.as_slice());
    let named: Option<&std::ffi::OsStr> = match form.tree {
        Some(tree) => tree.revision.as_deref(),
        None => None,
    };
    let message: String = match (at_risk.untracked.is_empty() && at_risk.nested.is_empty(), at_risk.ignored.is_empty()) {
        (false, _) => exempt_at_risk_message(command, at_risk.untracked.as_slice(), at_risk.nested.as_slice(), at_risk.ignored.as_slice()),
        (true, false) => ignored_file_message(command, named, at_risk.ignored.as_slice()),
        (true, true) => String::new(),
    };
    return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{message}{}", form.conflict_note));
}

/// What:
///  What one form would destroy that exists nowhere else.
///  `Result<AtRisk, String>` is "the answer,
///  or why it could not be read".
/// Why:
///  A clean is read by its own dry run,
///  which is the one reading that applies its flags,
///  patterns and pathspecs,
///  while a tree-writing form is read by the paths it writes and the entries that stand there;
///  a form with neither writes no untracked entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatTheFormWouldDestroy(facts, command, region, tree, location): Promise<AtRisk>; // throws
/// ```
fn what_the_form_would_destroy(
    facts: &mut dyn RepositoryFacts,
    command: GuardedCommand,
    region: &[OsString],
    tree: Option<&TargetTree>,
    location: &RepositoryLocation,
) -> Result<AtRisk, String> {
    return match command {
        GuardedCommand::Clean | GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly => {
            what_a_clean_would_destroy(facts, region, location)
        }
        _ => what_a_tree_form_would_destroy(facts, command, region, tree, location),
    };
}

/// What:
///  The untracked entries and nested repositories one clean would delete.
/// Why:
///   Git's own dry run names every path the clean removes,
///  including the directories it removes whole;
///  the untracked listing says which of those Git never tracked,
///  and the nested repositories inside the removed directories are found by looking there.
///  An ignored file alone is not refused:
///  the owner decided on 2026-11-12 that `clean -fX` and `clean -fx` delete,
///  exit 0 and are logged.
///  A clean Git refuses deletes nothing,
///  while a reading that never answered is a failure and refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatACleanWouldDestroy(facts, region, location): Promise<AtRisk>; // throws
/// ```
fn what_a_clean_would_destroy(
    facts: &mut dyn RepositoryFacts,
    region: &[OsString],
    location: &RepositoryLocation,
) -> Result<AtRisk, String> {
    let Ok(_parsed) = parse_clean_region(region, &[]) else {
        return Ok(AtRisk::default());
    };
    let prefix: Vec<OsString> = facts.transaction_context().global_prefix;
    let Some(arguments) = clean_dry_run_arguments(prefix.as_slice(), region) else {
        return Ok(AtRisk::default());
    };
    let deleted: Vec<Vec<u8>> = match facts.clean_deleted_paths(arguments) {
        Ok(answer) => answer,
        // Git refuses this clean itself, so it deletes nothing and Git's own refusal is the answer the person gets.
        Err(message) if message.starts_with(UNREADABLE_CLEAN_DELETION) => return Ok(AtRisk::default()),
        // A reading that never answered could hide a file or a repository, so the clean is refused.
        Err(message) => return Err(message),
    };
    let removed: Vec<Vec<u8>> = removed_paths_from_dry_run(deleted.as_slice());
    let untracked: Vec<Vec<u8>> = facts.untracked_paths()?;
    let mut nested: Vec<String> = Vec::<String>::new();
    let repositories: Vec<Vec<u8>> = match worktree_root(&location.identity) {
        Some(root) => nested_repositories_the_clean_deletes(deleted.as_slice(), root)?,
        None => Vec::<Vec<u8>>::new(),
    };
    for path in &repositories {
        nested.push(String::from_utf8_lossy(path.as_slice()).into_owned());
    }
    let candidates: Vec<String> = entries_in_the_way(removed.as_slice(), untracked.as_slice());
    // A nested repository arrives in the untracked listing as its own directory entry,
    //  which the repository's own refusal already names.
    let mut untracked_in_the_way: Vec<String> = Vec::<String>::new();
    for entry in candidates {
        if named_among(&nested, entry.as_str()) {
            continue;
        }
        untracked_in_the_way.push(entry);
    }
    return Ok(AtRisk {
        untracked: untracked_in_the_way,
        nested,
        ignored: Vec::<String>::new(),
    });
}

/// What:
///  The untracked entries that stand where a tree-writing form writes,
///  and the ignored files the tree it writes also tracks.
/// Why:
///  The written paths come from the form's own words and Git's own tree or index;
///  an untracked entry at one of them exists nowhere else and is refused,
///  and an ignored file at a path the target tree holds is replaced whatever `--no-overwrite-ignore` says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatATreeFormWouldDestroy(facts, command, region, tree, location): Promise<AtRisk>; // throws
/// ```
fn what_a_tree_form_would_destroy(
    facts: &mut dyn RepositoryFacts,
    command: GuardedCommand,
    region: &[OsString],
    tree: Option<&TargetTree>,
    location: &RepositoryLocation,
) -> Result<AtRisk, String> {
    let written: Option<Written> = match worktree_root(&location.identity) {
        Some(root) => written_paths(command, region, root)?,
        None => None,
    };
    let Some(form_writes) = written else {
        return Ok(AtRisk::default());
    };
    let paths: Vec<Vec<u8>> = match &form_writes {
        Written::Tree { revision, pathspecs, prefix } => {
            let tree_paths: Vec<Vec<u8>> = facts.tree_paths(revision.as_deref(), pathspecs.as_slice())?;
            crate::repository_facts_paths_the_form_writes::with_prefix(prefix.as_slice(), tree_paths.as_slice())
        }
        Written::Index { pathspecs, prefix } => {
            let index_paths: Vec<Vec<u8>> = facts.index_paths(pathspecs.as_slice())?;
            crate::repository_facts_paths_the_form_writes::with_prefix(prefix.as_slice(), index_paths.as_slice())
        }
        Written::Paths { paths } => paths.clone(),
    };
    let untracked: Vec<Vec<u8>> = facts.untracked_paths()?;
    // An ignored file whose path the tree the form writes holds is replaced whatever `--no-overwrite-ignore` says;
    //  the reading is the ordinary guard's for the same forms, and it names a file Git may keep for a path form,
    //  which the person sees before the opt-out runs the command as Git would.
    let ignored: Vec<String> = match tree {
        Some(target) => facts.ignored_files_the_tree_tracks(target.revision.as_deref())?,
        None => Vec::<String>::new(),
    };
    return Ok(AtRisk {
        untracked: entries_in_the_way(paths.as_slice(), untracked.as_slice()),
        nested: Vec::<String>::new(),
        ignored,
    });
}

/// What:
///  Whether one path is named among `named`,
///  with or without the separator Git prints on a directory.
/// Why:
///  A nested repository is named without the trailing separator,
///  while the untracked listing prints its directory entry with one,
///  and the two names are one loss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedAmong = (named, path) => named.some((one) => one.replace(/\/$/, '') === path.replace(/\/$/, ''));
/// ```
fn named_among(named: &[String], path: &str) -> bool {
    let bare: &str = path.strip_suffix('/').unwrap_or(path);
    for one in named {
        if one.strip_suffix('/').unwrap_or(one.as_str()) == bare {
            return true;
        }
    }
    return false;
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
    notices.push(crate::rule_linked_worktree_messages::exemption_notice(found.named_by, found.path.as_path()));
    return vec![AuditFact::Text("exemptedBy", found.named_by.to_string())];
}
