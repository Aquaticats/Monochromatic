//! What:
//!  Per-command handling:
//!  the option grammar of each Git command the wrapper understands,
//!  and the rules that rewrite,
//!  refuse or hint at a command.
//! Why:
//!  The wrapper judges commands one by one,
//!  so everything that is about a single Git command lives together.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `branch_delete_audit.rs`:
//!   the facts one audit line of a deleted branch carries.
//! - `command_clean_nested_repositories.rs`:
//!   which nested Git repositories a clean's two dry runs say it would delete.
//! - `command_options_types.rs`:
//!   what one argument region is read into:
//!   a row of Git's option table,
//!   the parse switches,
//!   every accepted use and the refusal kinds.
//! - `command_branch_table.rs`:
//!   the complete Git 2.56.0 option table of `git branch`.
//! - `command_checkout_table.rs`:
//!   the complete Git 2.56.0 option tables of `git checkout` and `git switch`.
//! - `command_commit_table.rs`:
//!   the complete Git 2.56.0 option table of `git commit`.
//! - `command_config_table.rs`:
//!   the Git 2.56.0 option groups of `git config`.
//! - `command_options_long_resolution.rs`:
//!   which row one long token names:
//!   exact names,
//!   unique abbreviations and `no-` negation.
//! - `command_options_value.rs`:
//!   the shared decision of which bytes are an option's value.
//! - `command_options_long.rs`:
//!   long-option matching:
//!   one token read into the tokenizer's record of the options Git applied.
//! - `command_options_short.rs`:
//!   short-option clusters and their attached or following values.
//! - `command_options.rs`:
//!   git 2.56.0 `parse-options` tokenization of one command's argument region.
//! - `command_options_query.rs`:
//!   final-state and position questions over a tokenized region.
//! - `command_add.rs`:
//!   the Git 2.56.0 option table of `git add` and its bulk-staging facts.
//! - `command_branch_delete.rs`:
//!   which branches a `git branch` region deletes.
//! - `command_branch_mode.rs`:
//!   which action a tokenized `git branch` region selects.
//! - `command_branch_target.rs`:
//!   explicit creation and the remote-guess candidate of `git checkout` and `git switch`.
//! - `command_branch_create.rs`:
//!   branch-creation facts of `git branch`,
//!   `git checkout` and `git switch`.
//! - `command_clean.rs`:
//!   the Git 2.56.0 option table of `git clean` and its deletion facts.
//! - `command_commit.rs`:
//!   facts about the arguments after `git commit`.
//! - `command_config.rs`:
//!   the form and file scope of a `git config` region.
//! - `command_discard.rs`:
//!   the commands that can overwrite or delete uncommitted state,
//!   read as Git reads them.
//! - `command_push.rs`:
//!   the Git 2.56.0 option table of `git push` and its facts.
//! - `command_remote_table.rs`:
//!   the one option `git remote` reads before a subcommand.
//! - `command_remote.rs`:
//!   the subcommand of a `git remote` region and where it starts.
//! - `command_reset.rs`:
//!   the Git 2.56.0 option table of `git reset` and its mode.
//! - `command_stash_table.rs`:
//!   the Git 2.56.0 option tables of every `git stash` subcommand.
//! - `command_stash.rs`:
//!   `git stash` subcommand dispatch and wrapper-flag positions.
//! - `command_status.rs`:
//!   the Git 2.56.0 option table of `git status`,
//!   its facts and the advice-key reading.
//! - `command_worktree.rs`:
//!   whether a `git worktree` invocation changes worktree registrations (an add or a move),
//!   and which directory a removal takes.
//! - `rule_add_explicit.rs`:
//!   pure decision of the add-explicit policy.
//! - `rule_argument_rewrite.rs`:
//!   the shared result and builder of transforms that add tokens to a command line.
//! - `rule_atomic_push.rs`:
//!   pure decision of the atomic-push transform.
//! - `rule_branch_worktree.rs`:
//!   pure decision of the branch-worktree-only policy and its one-query remote guess.
//! - `rule_clean_interactive_nested_repositories.rs`:
//!   the transform that keeps a nested Git repository out of an interactive clean's menu.
//! - `rule_keep_ignored_files.rs`:
//!   the transform that stops a `checkout` or `switch` replacing an ignored file.
//! - `rule_repository_stash.rs`:
//!   the one-query question whether the repository holds a stash entry.
//! - `rule_require_root.rs`:
//!   pure decision of the require-root policy.
//! - `rule_shared_remote.rs`:
//!   pure decision of the shared-remote-settings refusal:
//!   which `git remote` and `git config` commands write the remote settings every worktree shares.
//! - `rule_shared_remote_messages.rs`:
//!   the rejection texts of the shared-remote-settings refusal.
//! - `rule_status_hints.rs`:
//!   pure decision of the status-hints transform.
//! - `rule_worktree_dirty.rs`:
//!   the query and reading of "does this worktree hold uncommitted state?".
//! - `rule_linked_worktree.rs`:
//!   pure decision of the linked-worktree-only policy.
//! - `rule_abort_operation.rs`:
//!   the operation each abort form undoes,
//!   and whether that operation waits.
//! - `rule_abort_messages.rs`:
//!   the plain-language texts of the abort carve-out.
//! - `rule_linked_worktree_messages.rs`:
//!   the rejection texts of the linked-worktree-only policy.
//! - `rule_linked_worktree_nested_repository_messages.rs`:
//!   the notice that names where a clean's nested repository went,
//!   and the rejection when it could not be moved there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command/index.ts: one `export * from './file'` per module below.
//! ```

/// What one argument region is read into:
///  a row of Git's option table,
///  the parse switches,
///  every accepted use and the refusal kinds.
pub mod command_options_types;

/// Git 2.56.0 `parse-options` tokenization of one command's argument region.
pub mod command_options;

/// Which row one long token names:
///  exact names,
///  unique abbreviations and `no-` negation.
pub(crate) mod command_options_long_resolution;

/// Long-option matching:
///  one token read into the tokenizer's record of the options Git applied.
pub(crate) mod command_options_long;

/// Final-state and position questions over a tokenized region.
pub mod command_options_query;

/// Short-option clusters and their attached or following values.
pub(crate) mod command_options_short;

/// The shared decision of which bytes are an option's value.
pub(crate) mod command_options_value;

/// Real-Git fixtures and table oracles for command tests;
///  never in the release build.
#[cfg(test)]
pub(crate) mod command_test_support;

/// Per-case copies of a pristine fixture and worker threads for independent test cases;
///  never in the release build.
#[cfg(test)]
pub(crate) mod command_test_cases;

/// Differential harness over `git rev-parse --parseopt`;
///  never in the release build.
#[cfg(test)]
pub(crate) mod command_test_parseopt;

/// Table oracle over `--git-completion-helper-all`;
///  never in the release build.
#[cfg(test)]
pub(crate) mod command_test_completion;

/// Facts about the arguments after `git commit`.
pub mod command_commit;

/// The complete Git 2.56.0 option table of `git commit`.
pub mod command_commit_table;

/// The shared result and builder of transforms that add tokens to a command line.
pub mod rule_argument_rewrite;

/// The query and reading of "does this worktree hold uncommitted state?".
pub mod rule_worktree_dirty;

/// The Git 2.56.0 option table of `git push` and its facts.
pub mod command_push;

/// The Git 2.56.0 option table of `git status`,
///  its facts and the advice-key reading.
pub mod command_status;

/// Pure decision of the atomic-push transform.
pub mod rule_atomic_push;

/// Pure decision of the status-hints transform.
pub mod rule_status_hints;

/// The Git 2.56.0 option table of `git add` and its bulk-staging facts.
pub mod command_add;

/// The Git 2.56.0 option table of `git clean` and its deletion facts.
pub mod command_clean;

/// Which nested Git repositories a clean's two dry runs say it would delete.
pub mod command_clean_nested_repositories;

/// The Git 2.56.0 option table of `git reset` and its mode.
pub mod command_reset;

/// `git stash` subcommand dispatch and wrapper-flag positions.
pub mod command_stash;

/// The commands that can overwrite or delete uncommitted state,
/// read as Git reads them.
pub mod command_discard;

/// The Git 2.56.0 option tables of every `git stash` subcommand.
pub mod command_stash_table;

/// Branch-creation facts of `git branch`,
///  `git checkout` and `git switch`.
pub mod command_branch_create;

/// Which action a tokenized `git branch` region selects.
pub(crate) mod command_branch_mode;

/// The complete Git 2.56.0 option table of `git branch`.
pub mod command_branch_table;

/// Explicit creation and the remote-guess candidate of `git checkout` and `git switch`.
pub(crate) mod command_branch_target;

/// The complete Git 2.56.0 option tables of `git checkout` and `git switch`.
pub mod command_checkout_table;

/// The Git 2.56.0 option groups of `git config`.
pub mod command_config_table;

/// The form and file scope of a `git config` region.
pub mod command_config;

/// The one option `git remote` reads before a subcommand.
pub mod command_remote_table;

/// The subcommand of a `git remote` region and where it starts.
pub mod command_remote;

/// Pure decision of the require-root policy.
pub mod rule_require_root;

/// Whether a `git worktree` invocation changes worktree registrations (an add or a move),
///  and which directory a removal takes.
pub mod command_worktree;

/// Which branches a `git branch` region deletes.
pub mod command_branch_delete;

/// The facts one audit line of a deleted branch carries.
pub mod branch_delete_audit;

/// Pure decision of the linked-worktree-only policy.
pub mod rule_linked_worktree;

/// The rejection texts of the linked-worktree-only policy.
pub mod rule_linked_worktree_messages;

/// The notice that names where a clean's nested repository went,
/// and the rejection when it could not be moved there.
pub mod rule_linked_worktree_nested_repository_messages;

/// Pure decision of the shared-remote-settings refusal:
///  which `git remote` and `git config` commands write the remote settings every worktree shares.
pub mod rule_shared_remote;

/// The rejection texts of the shared-remote-settings refusal.
pub mod rule_shared_remote_messages;

/// The operation each abort form undoes,
/// and whether that operation waits.
pub mod rule_abort_operation;

/// The plain-language texts of the abort carve-out.
pub mod rule_abort_messages;

/// The transform that stops a `checkout` or `switch` replacing an ignored file.
pub mod rule_keep_ignored_files;

/// The transform that keeps a nested Git repository out of an interactive clean's menu.
pub mod rule_clean_interactive_nested_repositories;

/// The one-query question whether the repository holds a stash entry.
pub mod rule_repository_stash;

/// Pure decision of the branch-worktree-only policy and its one-query remote guess.
pub mod rule_branch_worktree;

/// Pure decision of the add-explicit policy.
pub mod rule_add_explicit;
