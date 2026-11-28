//! What:
//!  The management grammar's two ZFS route words:
//!  `cancel-worktree-removal <slot>` and
//!  `settle-worktree-clone-record <slot> --confirm-removal-of <slot>
//!  --confirm-clone-guid <digits>`,
//!  and the answer each prints.
//! Why:
//!  Both commands were built ahead of the grammar that reaches them
//!  (`worktree_clone_removal_cancel::mod`,
//!  `worktree_clone_record_settling::mod`),
//!  and the plan's step 17 owes the words that reach them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 17").
//!  The grammar's own files are `invocation/`,
//!  a group folder.
//!  A new file there is a finding no allow-list line may record,
//!  and those files are already over the line budget,
//!  so the words are parsed there and everything a run needs lives here:
//!  the place the command runs in,
//!  the pool it reads when a read needs one,
//!  the request a cancellation files,
//!  and the streams and statuses each answer carries.
//!  The grammar turns one answer into this process's exit,
//!  and nothing here names the process's own action type,
//!  because that sits in the group folder this one is reached from.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `the_settling_commands_word.rs`:
//!   the word the grammar reads for the settling command,
//!   proved against the invocation the add's records print.
//! - `the_status_a_settling_stop_ends_in.rs`:
//!   the status each way the settling command stops ends in.
//! - `the_answer_one_word_prints.rs`:
//!   one answer's three fields,
//!   and the line feed a text a person reads ends with.
//! - `the_place_the_two_commands_run_in.rs`:
//!   the checkout,
//!   the common Git directory and the account's home one invocation runs in,
//!   and the refusals of an invocation that names none.
//! - `the_pool_a_command_reads.rs`:
//!   the pool both commands read,
//!   opened at the first read that needs it.
//! - `the_mount_request_a_cancellation_files.rs`:
//!   the reversing request a cancellation files,
//!   opened from the slot's record when the command first asks for one.
//! - `a_cancel_command_answered.rs`:
//!   what the `cancel-worktree-removal` word prints,
//!   over the release's own seams.
//! - `a_settling_command_answered.rs`:
//!   what the `settle-worktree-clone-record` word prints,
//!   over the release's own reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = word === 'cancel-worktree-removal'
//!   ? aCancelCommandAnswered(parseCancelArguments(rest), route)
//!   : aSettlingCommandAnswered(parseSettlingArguments(rest), route);
//! ```

/// The word the grammar reads for the settling command.
pub mod the_settling_commands_word;

/// The status each way the settling command stops ends in.
pub mod the_status_a_settling_stop_ends_in;

/// One finished invocation's status and the text each stream carries.
pub mod the_answer_one_word_prints;

/// The place the two commands run in.
pub mod the_place_the_two_commands_run_in;

/// The pool both commands read,
/// opened at the first read that needs it.
pub mod the_pool_a_command_reads;

/// The reversing request a cancellation files.
pub mod the_mount_request_a_cancellation_files;

/// What the `cancel-worktree-removal` word prints.
pub mod a_cancel_command_answered;

/// What the `settle-worktree-clone-record` word prints.
pub mod a_settling_command_answered;

pub use a_cancel_command_answered::a_cancel_command_answered;
pub use a_settling_command_answered::a_settling_command_answered;
pub use the_answer_one_word_prints::AnAnswer;
