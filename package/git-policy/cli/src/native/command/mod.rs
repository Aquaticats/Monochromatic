//! What:
//!  Per-command handling:
//!  the option grammar of each Git command the wrapper understands,
//!  and the rules that rewrite,
//!  refuse or hint at a command.
//! Why:
//!  The wrapper judges commands one by one,
//!  so everything that is about a single Git command lives together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command/index.ts: one `export * from './file'` per module below.
//! ```

/// Git 2.56.0 `parse-options` tokenization of one command's argument region.
pub mod command_options;

/// Long-option matching:
///  exact names,
///  unique abbreviations and `no-` negation.
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

/// The transform that stops a `checkout` or `switch` replacing an ignored file.
pub mod rule_keep_ignored_files;

/// The one-query question whether the repository holds a stash entry.
pub mod rule_repository_stash;

/// Pure decision of the branch-worktree-only policy and its one-query remote guess.
pub mod rule_branch_worktree;

/// Pure decision of the add-explicit policy.
pub mod rule_add_explicit;
