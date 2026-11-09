//! What:
//!  The shipped policies as the engine sees them:
//!  one check per policy,
//!  built from
//!       the pure rule cores and the repository facts they ask for.
//! Why:
//!  The engine owns order,
//!  severity and stopping;
//!  the rule cores own decisions from
//!      arguments;
//!  this module joins them and fetches a repository fact only when a core
//!      asks.
//!  The content policies read the lifecycle's candidates:
//!  they report nothing
//!      where a lifecycle has none.
//!  The Markdown policy checks the candidates it is given
//!      with the native Markdown linter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await policy.check({ context });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Each check combines one rule core with the facts interface.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { decideRequireRoot, resolveRequireRoot } from './rule_require_root.ts';
/// ```
use crate::diagnostics::EngineFailureCode;
use crate::conflict_side_guard::{SideVerdict, judge_conflict_sides};
use crate::discarded_prune::Limits;
use crate::discard_audit::{AuditOutcome, record_audit, record_audit_with_copies};
/// Which worktree a command targets,
///  for the linked-worktree rule.
use crate::effective_target::{EffectiveTarget, classify_effective_target};
/// The Markdown autofix check,
///  its state and the linter the executable gives it.
use crate::markdown_linter::embedded_linter;
/// What the lifecycle offers content policies,
///  and its prepared candidates.
use crate::policy_content::{ContentState, LifecycleContent};
/// The dependent-version check over the candidate state.
use crate::policy_dependent_version::check_dependent_version;
/// The engine's check interface,
///  findings and outcomes.
use crate::policy_engine::{PolicyChecks, PolicyFinding, PolicyOutcome};
/// The built-in final-newline check over candidates.
use crate::policy_final_newline::check_final_newline;
/// The forbidden-strings check,
///  its settings and its scanner.
use crate::policy_forbidden_strings::{
    ScannerSettings, ScannerState, check_forbidden_strings, default_scanner_settings,
    unloaded_scanner,
};
use crate::policy_markdown::{MarkdownState, check_markdown, markdown_state};
/// The typed policy identities.
use crate::policy_registry::PolicyId;
/// The optional root-context check over candidates.
use crate::policy_root_context::check_root_context;
/// The optional catalog-specifier check over candidates.
use crate::policy_catalog_specifiers::check_catalog_specifiers;
/// The lifecycle points.
use crate::policy_trigger::Trigger;
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// Where a command runs,
///  and the directory Git works in.
use crate::repository_location::{RepositoryLocation, effective_directory};
/// The add-explicit rule core.
use crate::rule_add_explicit::{BULK_ADD_CODE, decide_add_explicit};
/// The branch-worktree rule core.
use crate::rule_branch_worktree::{
    BRANCH_CREATION_CODE, BranchWorktreeDecision, branch_creation_message, decide_branch_worktree,
};
/// The linked-worktree rule core.
use crate::rule_linked_worktree::{
    GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, LinkedWorktreeDecision, decide_linked_worktree,
    dirty_scope, dirty_worktree_message, resolve_linked_worktree,
};
/// Whether a Git directory has no index file yet.
use crate::rule_worktree_dirty::index_file_is_absent;
/// The require-root rule core.
use crate::rule_require_root::{
    NOT_AT_ROOT_CODE, RequireRootDecision, RequireRootFacts, RequireRootVerdict,
    decide_require_root, resolve_require_root,
};
/// The worktree's top level.
use crate::worktree_identity::worktree_root;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   The command's arguments are kept exactly as Git will receive them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw bytes.
use std::path::PathBuf;

/// What:
///  The shipped policies over one invocation.
///  `<F: RepositoryFacts>` says the record
///       works with any one type `F` that provides the facts interface,
///  chosen where the
///       record is built:
///  real Git in the executable,
///  a script in tests.
/// Why:
///   Holding the provider by value keeps its remembered answers and its query count
///       for the whole invocation,
///  and lets a test read them back afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShippedChecks<F extends RepositoryFacts> = { facts: F; arguments: string[]; candidates: LifecycleContent; allowedWorktreeDirs: string[]; content: ContentState; markdown: MarkdownState };
/// ```
pub struct ShippedChecks<F: RepositoryFacts> {
    /// Whether a pass may propose corrections that its caller applies;
    /// the commit path's convergence sets it.
    pub can_apply_patches: bool,
    /// Where repository facts come from.
    pub facts: F,
    /// The command's arguments without wrapper controls;
    ///  for a direct command,
    ///  the Git
    /// global options written before `monochromatic-git-policy-cli`.
    pub arguments: Vec<OsString>,
    /// What the lifecycle offers content policies.
    pub candidates: LifecycleContent,
    /// Tool-cache directories exempt from linked-worktree enforcement.
    pub allowed_worktree_dirs: Vec<PathBuf>,
    /// The limits of the folder of saved copies of conflicted files,
    /// from `discardedCopies` in `monochromatic-git-policy-cli.config.jsonc`.
    pub discarded_limits: Limits,
    /// The candidates once a content policy prepared them,
    ///  shared by every later one.
    pub content: ContentState,
    /// Where the forbidden-strings rules come from:
    ///  options and environment.
    pub scanner_settings: ScannerSettings,
    /// The forbidden-strings scanner once loaded,
    ///  shared by every later scan.
    pub scanner: ScannerState,
    /// The Markdown autofix policy's options,
    ///  linter and exclude patterns.
    pub markdown: MarkdownState,
    /// Plain notices the checks produced for the person (a saved copy of a conflicted file),
    /// shown before Git runs.
    pub notices: Vec<String>,
}

/// What:
///  The shipped checks for one invocation,
///  with no candidates yet prepared.
///       `<F: RepositoryFacts>` accepts any facts provider.
/// Why:
///   Every lifecycle builds its checks the same way;
///  it only chooses what the
///       content policies may read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shippedChecks(facts, args, candidates, allowedWorktreeDirs): ShippedChecks;
/// ```
pub fn shipped_checks<F: RepositoryFacts>(
    facts: F,
    arguments: Vec<OsString>,
    candidates: LifecycleContent,
    allowed_worktree_dirs: Vec<PathBuf>,
) -> ShippedChecks<F> {
    return ShippedChecks {
        can_apply_patches: false,
        facts,
        arguments,
        candidates,
        allowed_worktree_dirs,
        discarded_limits: Limits::default(),
        content: ContentState::new(),
        scanner_settings: default_scanner_settings(),
        scanner: unloaded_scanner(),
        // The linter is linked in, so it needs nothing from the invocation's environment and is ready at once.
        markdown: markdown_state(Box::new(embedded_linter())),
        notices: Vec::new(),
    };
}

/// What:
///  The outcome of a check whose repository fact could not be read.
///       `String` is the owned reason the facts provider gave.
/// Why:
///   Every fact these checks ask for is read from the repository,
///  so its absence is
///       `content-unavailable`,
///  never a failure of the policy itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const factUnavailable = (message: string): PolicyOutcome => ({ kind: 'failed', code: 'content-unavailable', message });
/// ```
fn fact_unavailable(message: String) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: EngineFailureCode::ContentUnavailable,
        message,
        // `None`: a repository fact is about no single file.
        path: None,
    };
}

/// What:
///  The outcome "nothing found".
///  `Vec::<PolicyFinding>::new()` is an empty owned list.
/// Why:
///   Most checks end this way;
///  one helper keeps them identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clean = (): PolicyOutcome => ({ kind: 'findings', findings: [] });
/// ```
fn clean() -> PolicyOutcome {
    return PolicyOutcome::Findings(Vec::<PolicyFinding>::new());
}

/// What:
///  The outcome "exactly one finding about the command".
///  `&'static str` is text
///       baked into the program;
///  `String` is owned text.
/// Why:
///   The four command policies each report at most one finding,
///  without a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rejected = (code: string, message: string): PolicyOutcome => ({ kind: 'findings', findings: [{ code, message }] });
/// ```
fn rejected(code: &'static str, message: String) -> PolicyOutcome {
    // `vec![...]` is a macro building an owned list from literal items.
    return PolicyOutcome::Findings(vec![PolicyFinding {
        code,
        message,
        // `None` is the "absent" case of `Option`.
        path: None,
        location: None,
        fix_available: false,
    }]);
}

/// What:
///  The require-root check.
///  `&mut dyn RepositoryFacts` lends "any facts provider" for
///       writing (`dyn` means the concrete type is chosen at run time,
///  like a TS interface
///       value);
///  `&[OsString]` borrows the arguments.
/// Why:
///   A forwarded command is first judged by its arguments,
///  because many commands are
///       exempt.
///  A direct check has no Git command whose exemption could apply.
///  The root
///       and the directory both come from Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkRequireRoot(facts: RepositoryFacts, args: string[], trigger: PolicyTrigger): Promise<PolicyOutcome>;
/// ```
fn check_require_root(
    facts: &mut dyn RepositoryFacts,
    arguments: &[OsString],
    trigger: Trigger,
) -> PolicyOutcome {
    // `&[]` is an empty flag list: wrapper controls were removed before this point.
    if trigger == Trigger::PreForward
        && decide_require_root(arguments, &[]) != RequireRootDecision::NeedsRepositoryRoot
    {
        return clean();
    }
    // `match` unpacks the answer; `Ok`/`Err` are the success/failure variants.
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => return fact_unavailable(message),
    };
    // `let Some(x) = ... else { ... };` unwraps the top level or returns: no worktree, no root.
    let Some(root) = worktree_root(&location.identity) else {
        return clean();
    };
    let measured: RequireRootFacts = RequireRootFacts {
        effective_directory: effective_directory(root, location.prefix.as_slice()),
        // `Some(x)` is the "present" case; `.to_path_buf()` copies the borrowed path.
        repository_root: Some(root.to_path_buf()),
    };
    // `&measured` lends the measurements read-only.
    match resolve_require_root(&measured) {
        RequireRootVerdict::Pass => return clean(),
        RequireRootVerdict::NotAtRoot(violation) => {
            return rejected(NOT_AT_ROOT_CODE, violation.message);
        }
    }
}

/// The clock reading the conflict copies are named with,
/// in milliseconds since the Unix epoch.
fn now_milliseconds() -> i64 {
    let elapsed: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    return i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
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
fn check_linked_worktree(
    facts: &mut dyn RepositoryFacts,
    arguments: &[OsString],
    allowed_worktree_dirs: &[PathBuf],
    discarded_limits: Limits,
    notices: &mut Vec<String>,
) -> PolicyOutcome {
    // `let Variant(x) = ... else { ... };` unwraps the guarded command or returns.
    let (command, moves_head_only): (GuardedCommand, bool) = match decide_linked_worktree(arguments) {
        LinkedWorktreeDecision::Pass => return clean(),
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
        match judge_conflict_sides(&facts.transaction_context(), arguments, now_milliseconds(), discarded_limits) {
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
                record_audit_with_copies(&location.identity, command, AuditOutcome::SavedConflict, arguments, None, places.as_slice());
                notices.push(notice);
                return clean();
            }
            SideVerdict::CopyFailed(message) => {
                record_audit(&location.identity, command, AuditOutcome::Rejected, arguments, None);
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
        record_audit(&location.identity, command, AuditOutcome::Rejected, arguments, None);
        return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{message}{conflict_note}"));
    }
    if target == EffectiveTarget::Allowlisted {
        record_audit(&location.identity, command, AuditOutcome::Exempt, arguments, None);
        return clean();
    }
    // A checkout that only moves `HEAD` in a Git directory with no index file is Git's own initial checkout (a clone made with
    // `--no-checkout`): there is no tracked work to carry over, so the status, which lists every file as a staged deletion, is not read.
    if moves_head_only && index_file_is_absent(facts.known_git_dir().as_deref(), std::env::var_os("GIT_INDEX_FILE").as_deref()) {
        record_audit(&location.identity, command, AuditOutcome::Allowed, arguments, Some(false));
        return clean();
    }
    match facts.worktree_dirty(dirty_scope(command)) {
        Ok(true) => {
            record_audit(&location.identity, command, AuditOutcome::Rejected, arguments, Some(true));
            return rejected(LINKED_WORKTREE_REQUIRED_CODE, format!("{}{conflict_note}", dirty_worktree_message(command)));
        }
        Ok(false) => {
            record_audit(&location.identity, command, AuditOutcome::Allowed, arguments, Some(false));
            return clean();
        }
        Err(message) => return fact_unavailable(message),
    }
}

/// What:
///  The branch-worktree-only check.
/// Why:
///   Explicit creation is rejected from the arguments;
///  a bare name is rejected only
///       when Git would create a branch for it from the one remote that has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkBranchWorktree(facts, args): Promise<PolicyOutcome>;
/// ```
fn check_branch_worktree(facts: &mut dyn RepositoryFacts, arguments: &[OsString]) -> PolicyOutcome {
    match decide_branch_worktree(arguments) {
        BranchWorktreeDecision::Pass => return clean(),
        BranchWorktreeDecision::Creates(command) => {
            return rejected(BRANCH_CREATION_CODE, branch_creation_message(command, None));
        }
        BranchWorktreeDecision::NeedsRemoteGuess { command, target } => {
            // `.as_os_str()` lends the owned name as a borrowed one.
            match facts.remote_guess_creates_branch(target.as_os_str()) {
                Err(message) => return fact_unavailable(message),
                Ok(false) => return clean(),
                Ok(true) => {
                    return rejected(
                        BRANCH_CREATION_CODE,
                        branch_creation_message(command, Some(target.as_os_str())),
                    );
                }
            }
        }
    }
}

/// What:
///  The add-explicit check.
/// Why:
///   The decision needs the arguments only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkAddExplicit(args: string[]): PolicyOutcome;
/// ```
fn check_add_explicit(arguments: &[OsString]) -> PolicyOutcome {
    match decide_add_explicit(arguments) {
        Some(message) => return rejected(BULK_ADD_CODE, message),
        None => return clean(),
    }
}

/// What:
///  `impl<F: RepositoryFacts> PolicyChecks for ShippedChecks<F>` provides the engine's
///       interface for the shipped policies,
///  for any facts provider `F`.
/// Why:
///   One `match` over the typed policy identity is the whole registry of behavior:
///       the compiler refuses a policy without a check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// check(policy: PolicyId, trigger: PolicyTrigger): Promise<PolicyOutcome> { switch (policy) { /* ... */ } }
/// ```
impl<F: RepositoryFacts> PolicyChecks for ShippedChecks<F> {
    /// What:
    ///  The outcome of one policy at one lifecycle point.
    /// Why:
    ///   Each arm calls the one check that owns the policy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// check(policy: PolicyId, trigger: PolicyTrigger): PolicyOutcome;
    /// ```
    fn check(&mut self, policy: PolicyId, trigger: Trigger) -> PolicyOutcome {
        // `&mut self.facts` lends the provider for writing; `.as_slice()` lends a list.
        match policy {
            PolicyId::RequireRoot => {
                return check_require_root(&mut self.facts, self.arguments.as_slice(), trigger);
            }
            PolicyId::LinkedWorktreeOnly => {
                return check_linked_worktree(
                    &mut self.facts,
                    self.arguments.as_slice(),
                    self.allowed_worktree_dirs.as_slice(),
                    self.discarded_limits,
                    &mut self.notices,
                );
            }
            PolicyId::BranchWorktreeOnly => {
                return check_branch_worktree(&mut self.facts, self.arguments.as_slice());
            }
            PolicyId::AddExplicit => return check_add_explicit(self.arguments.as_slice()),
            PolicyId::FinalNewline => {
                return check_final_newline(
                    &mut self.content,
                    &self.candidates,
                    &mut self.facts,
                    trigger,
                    self.can_apply_patches,
                );
            }
            PolicyId::ForbiddenRootContext => {
                return check_root_context(&mut self.content, &self.candidates, &mut self.facts);
            }
            PolicyId::MarkdownAutofix => {
                return check_markdown(
                    &mut self.content,
                    &self.candidates,
                    &mut self.facts,
                    &mut self.markdown,
                    trigger,
                    self.can_apply_patches,
                );
            }
            PolicyId::DependentVersionBump => {
                return check_dependent_version(
                    &mut self.content,
                    &self.candidates,
                    &mut self.facts,
                    trigger,
                );
            }
            PolicyId::CatalogSpecifiers => {
                return check_catalog_specifiers(
                    &mut self.content,
                    &self.candidates,
                    &mut self.facts,
                );
            }
            PolicyId::ForbiddenStrings => {
                return check_forbidden_strings(
                    &mut self.content,
                    &self.candidates,
                    &mut self.facts,
                    &self.scanner_settings,
                    &mut self.scanner,
                );
            }
        }
    }
}

/// Per-policy outcomes and lazy fact use stay out of the release executable.
#[cfg(test)]
#[path = "policy_checks_tests.rs"]
mod tests;
