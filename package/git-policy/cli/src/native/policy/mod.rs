//! What:
//!  The policy engine:
//!  the registry,
//!  triggers,
//!  events,
//!  the passes over candidates and direct fixes.
//! Why:
//!  Policies are the reason the wrapper exists,
//!  so the engine and its shared pieces have one home.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `diagnostics.rs`:
//!   JSONL event rendering for wrapper-made diagnostics.
//! - `direct_fix_install_attributes.rs`:
//!   the extended attributes a replacement carries with it.
//! - `direct_fix_install_attributes_reading.rs`:
//!   the answers direct fix install attributes reading holds.
//! - `direct_fix_install_writes.rs`:
//!   the swap and the replacement write an install uses.
//! - `direct_fix_install_journal.rs`:
//!   the journal an install is recorded in,
//!   and the sweep that finishes what a kill interrupted.
//! - `direct_fix_install.rs`:
//!   installing a direct fix's corrections into the worktree,
//!   all or none.
//! - `direct_fix_install_judging.rs`:
//!   what a name holds,
//!   and whether it is exactly what an install put there.
//! - `direct_fix_install_journal_sweep.rs`:
//!   the sweep of the journals an interrupted fix left,
//!   and its sentences.
//! - `direct_fix_install_messages.rs`:
//!   the sentences an install reports.
//! - `event_hints.rs`:
//!   the intent-guessing hint that follows a finding,
//!   an engine failure or a notice.
//! - `event_path.rs`:
//!   a pathname as events report it:
//!   text,
//!   plus base64 `pathBytes` when it is not UTF-8.
//! - `path_bytes.rs`:
//!   exact repository path bytes as the operating system takes them,
//!   and as text for messages.
//! - `policy_audited.rs`:
//!   the two commands the wrapper records and never refuses.
//! - `policy_convergence.rs`:
//!   the bounded loop that repeats a pass after corrections changed candidate content.
//! - `policy_registry.rs`:
//!   the fixed registry of shipped policies.
//! - `policy_trigger.rs`:
//!   lifecycle triggers and the trigger set of every shipped policy.
//! - `policy_events.rs`:
//!   policy events and their JSON Lines rendering.
//! - `policy_engine.rs`:
//!   ordered policy execution for one lifecycle point.
//! - `policy_branch_worktree.rs`:
//!   the branch-worktree-only check.
//! - `policy_content.rs`:
//!   what a lifecycle offers content policies,
//!   and its candidates once prepared.
//! - `policy_catalog_specifiers.rs`:
//!   the optional catalog-specifier policy over the shared rule crate:
//!   every dependency in a tracked `package.json` is a catalog or workspace reference.
//! - `policy_final_newline.rs`:
//!   the built-in final-newline policy:
//!   canonical final LF,
//!   preserved paths,
//!   findings.
//! - `policy_forbidden_strings.rs`:
//!   the optional forbidden-strings policy over the linked scanner,
//!   with redacted findings.
//! - `policy_linked_worktree.rs`:
//!   the linked-worktree check,
//!   in its own file so the registry of shipped policies stays readable.
//! - `policy_linked_worktree_nested_repositories.rs`:
//!   keeping the nested Git repositories a clean would delete,
//!   and the one place a form is let through.
//! - `policy_root_context.rs`:
//!   the optional root-context policy:
//!   no top-level `CONTEXT.md` enters the index.
//! - `policy_checks.rs`:
//!   the shipped policies as checks over rule cores and repository facts.
//! - `policy_transforms.rs`:
//!   the fixed argument transforms of a forwarded command,
//!   in order.
//! - `policy_pass.rs`:
//!   one complete pass:
//!   built-in policies,
//!   fixed transforms,
//!   optional policies.
//! - `direct_fix.rs`:
//!   the direct fix:
//!   converge corrections in memory,
//!   then install them.
//! - `unported.rs`:
//!   the work of the installed monochromatic-git-policy-cli that is not ported,
//!   and the refusal notice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy/index.ts: one `export * from './file'` per module below.
//! ```

/// JSONL event rendering for wrapper-made diagnostics.
pub mod diagnostics;

/// The fixed registry of shipped policies.
pub mod policy_registry;

/// Lifecycle triggers and the trigger set of every shipped policy.
pub mod policy_trigger;

/// Policy events and their JSON Lines rendering.
pub mod policy_events;

/// A pathname as events report it:
///  text,
///  plus base64 `pathBytes` when it is not UTF-8.
pub mod event_path;

/// Exact repository path bytes as the operating system takes them,
/// and as text for messages.
pub mod path_bytes;

/// Ordered policy execution for one lifecycle point.
pub mod policy_engine;

/// The bounded loop that repeats a pass after corrections changed candidate content.
pub mod policy_convergence;

/// The intent-guessing hint that follows a finding,
/// an engine failure or a notice.
pub mod event_hints;

/// The shipped policies as checks over rule cores and repository facts.
pub mod policy_checks;

/// The linked-worktree check,
/// in its own file so the registry of shipped policies stays readable.
pub mod policy_linked_worktree;

/// Keeping the nested Git repositories a clean would delete,
///  and the one place a form is let through.
pub mod policy_linked_worktree_nested_repositories;

/// What a lifecycle offers content policies,
///  and its candidates once prepared.
pub mod policy_content;

/// The built-in final-newline policy:
///  canonical final LF,
///  preserved paths,
///  findings.
pub mod policy_final_newline;

/// The optional root-context policy:
///  no top-level `CONTEXT.md` enters the index.
pub mod policy_root_context;

/// The optional catalog-specifier policy over the shared rule crate:
///  every dependency in a tracked `package.json` is a catalog or workspace reference.
pub mod policy_catalog_specifiers;

/// The branch-worktree-only check.
pub mod policy_branch_worktree;

/// The two commands the wrapper records and never refuses.
pub mod policy_audited;

/// The optional forbidden-strings policy over the linked scanner,
///  with redacted findings.
pub mod policy_forbidden_strings;

/// Installing a direct fix's corrections into the worktree,
///  all or none.
pub mod direct_fix_install;

/// The extended attributes a replacement carries with it.
pub mod direct_fix_install_attributes;

/// The swap and the replacement write an install uses.
pub mod direct_fix_install_writes;

/// The journal an install is recorded in,
/// and the sweep that finishes what a kill interrupted.
pub mod direct_fix_install_journal;

/// The sweep of the journals an interrupted fix left,
/// and its sentences.
pub mod direct_fix_install_journal_sweep;

/// The sentences an install reports.
pub mod direct_fix_install_messages;

/// What a name holds,
/// and whether it is exactly what an install put there.
pub mod direct_fix_install_judging;

/// The direct fix:
///  converge corrections in memory,
///  then install them.
pub mod direct_fix;

/// The fixed argument transforms of a forwarded command,
///  in order.
pub mod policy_transforms;

/// One complete pass:
///  built-in policies,
///  fixed transforms,
///  optional policies.
pub mod policy_pass;

/// The work of the installed monochromatic-git-policy-cli that is not ported,
///  and the refusal notice.
pub mod unported;

/// A scripted repository-facts provider for unit tests;
///  never in the release build.
#[cfg(test)]
pub(crate) mod policy_test_support;
