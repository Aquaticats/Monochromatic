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
/// The tool-cache entries exempt from the linked-worktree enforcement.
use crate::effective_target::ToolCacheDirectory;
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
/// The code a check reports when a repository fact could not be read.
use crate::diagnostics::EngineFailureCode;
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
/// The add-explicit rule core,
///  its question and the measurement it reads.
use crate::rule_add_explicit::{
    BULK_ADD_CODE, StagedSelection, add_explicit_question, decide_add_explicit, pathspec_spellings,
};
/// The branch-worktree-only check,
/// one policy of its own file.
use crate::policy_branch_worktree::check_branch_worktree;
/// The guarded decision about the command Git will really run,
/// alias expansion included.
use crate::guarded_forwarded_command::{GuardedForwarded, decide_guarded_forwarded};
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
    pub allowed_worktree_dirs: Vec<ToolCacheDirectory>,
    /// The limits of the folder of saved copies of conflicted files,
    /// from `discardedCopies` in `monochromatic-git-policy-cli.config.jsonc`.
    pub discarded_limits: crate::discarded_prune::Limits,
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
    /// The guarded decision about the command Git will run,
    ///  once it has been asked for;
    ///  the guard,
    ///  the escape record and the fixed transforms read this one answer,
    /// so the
    ///  expansion an alias names is the command all three see.
    pub forwarded: Option<GuardedForwarded>,
    /// The environment the invocation was started with;
    ///  the guard needs its `PATH` to see that Git runs an installed
    ///  external command instead of an alias of that name.
    pub environment: Vec<(OsString, OsString)>,
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
    allowed_worktree_dirs: Vec<ToolCacheDirectory>,
) -> ShippedChecks<F> {
    return ShippedChecks {
        can_apply_patches: false,
        facts,
        arguments,
        candidates,
        allowed_worktree_dirs,
        discarded_limits: crate::discarded_prune::Limits::default(),
        content: ContentState::new(),
        scanner_settings: default_scanner_settings(),
        scanner: unloaded_scanner(),
        // The linter is linked in, so it needs nothing from the invocation's environment and is ready at once.
        markdown: markdown_state(Box::new(embedded_linter())),
        notices: Vec::new(),
        forwarded: None,
        environment: Vec::new(),
    };
}

/// What:
///  The guarded decision about the command Git will run,
///  asked once per invocation.
///  `<F: RepositoryFacts>` accepts any facts provider.
/// Why:
///   An alias makes the word on the command line differ from the command Git runs;
///  the guard refuses
///  on the expansion,
///  the fixed transforms add their tokens to the same expansion,
///  and that expansion is what
///  the wrapper forwards.
///  Resolving once keeps one reading of the alias for all three,
///  and a built-in
///  word needs no configuration read and no process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function guardedForwarded(checks): { decision; args } {
///   return checks.forwarded ??= decideGuardedForwarded(checks.facts.transactionContext(), checks.arguments);
/// }
/// ```
pub fn guarded_forwarded<F: RepositoryFacts>(checks: &mut ShippedChecks<F>) -> GuardedForwarded {
    // `if let Some(known) = &checks.forwarded` borrows the remembered answer when there is one.
    if let Some(known) = &checks.forwarded {
        return known.clone();
    }
    let decided: GuardedForwarded = decide_guarded_forwarded(
        &checks.facts.transaction_context(),
        checks.environment.as_slice(),
        checks.arguments.as_slice(),
    );
    checks.forwarded = Some(decided.clone());
    return decided;
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
pub(crate) fn fact_unavailable(message: String) -> PolicyOutcome {
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
pub(crate) fn clean() -> PolicyOutcome {
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
pub(crate) fn rejected(code: &'static str, message: String) -> PolicyOutcome {
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

/// What:
///  The add-explicit check.
/// Why:
///   A flag stages every changed path whatever the repository holds,
///  while a pathspec is judged by what Git says it selects,
///  so the question is asked first and the measurement only when one is needed.
///  A measurement that cannot be made lets the command through with one line,
///  because the guard may never stop a command it could not judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkAddExplicit(facts, args, notices): PolicyOutcome;
/// ```
fn check_add_explicit(
    facts: &mut dyn RepositoryFacts,
    arguments: &[OsString],
    notices: &mut Vec<String>,
) -> PolicyOutcome {
    // `let Some(x) = ... else { ... };` unwraps the question or returns when nothing stages in bulk.
    let Some(question) = add_explicit_question(arguments) else {
        return clean();
    };
    let spellings: Vec<OsString> = pathspec_spellings(&question);
    // `Option<StagedSelection>` is "the measurement, or nothing when it could not be made".
    let mut measured: Option<StagedSelection> = None;
    if !spellings.is_empty() {
        // `match` unpacks the answer; `Ok`/`Err` are the success/failure variants.
        match facts.staged_selection(spellings) {
            Ok(selection) => measured = Some(selection),
            Err(reason) => {
                notices.push(format!(
                    "monochromatic-git-policy-cli: could not check whether this git add stages the whole \
                     repository, so the command runs unchanged: {reason}\n"
                ));
            }
        }
    }
    // `.as_ref()` lends the measurement as an optional borrowed one.
    match decide_add_explicit(&question, measured.as_ref()) {
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
                let forwarded: GuardedForwarded = guarded_forwarded(self);
                return crate::policy_linked_worktree::check_linked_worktree(
                    &mut self.facts,
                    &forwarded,
                    self.allowed_worktree_dirs.as_slice(),
                    self.discarded_limits,
                    &mut self.notices,
                );
            }
            PolicyId::BranchWorktreeOnly => {
                return check_branch_worktree(&mut self.facts, self.arguments.as_slice());
            }
            PolicyId::AddExplicit => {
                return check_add_explicit(&mut self.facts, self.arguments.as_slice(), &mut self.notices);
            }
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
