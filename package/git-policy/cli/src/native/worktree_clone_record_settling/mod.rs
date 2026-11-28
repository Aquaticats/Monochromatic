//! What:
//!  The settling command a state names,
//!  which this release's management grammar admits:
//!  `git monochromatic-git-policy-cli settle-worktree-clone-record <slot>
//!  --confirm-removal-of <slot> --confirm-clone-guid <the last four decimal
//!  digits of cloneGuid=>`.
//! Why:
//!  A binding whose recorded clone is gone and whose absence the pool cannot
//!  prove keeps its slot refusing the next add,
//!  and the project chose a command that removes the record once a person
//!  confirms it with two values the record supplies
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A slot whose recorded clone is gone and cannot be proved gone").
//!  The design that states the command,
//!  its six reads,
//!  its two modes and its refusal is
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command" and its veto entries V21 and V23.
//!  This release's management grammar admits the command,
//!  and the sentences that name it send a person to it.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `settling_steps.rs`:
//!   the two removal orders,
//!   as data.
//! - `settling_words.rs`:
//!   the two flags,
//!   the placeholders,
//!   and the GUID's last digits.
//! - `settling_refusal.rs`:
//!   every way the command stops.
//! - `settling_refusal_sentences.rs`:
//!   the sentence a person reads for each stop.
//! - `settling_absence.rs`:
//!   what the pool answers about the recorded clone's existence.
//! - `settling_removals.rs`:
//!   the removals themselves,
//!   proved gone one by one.
//! - `settling_request.rs`:
//!   the grammar's three values and the two checks that need no record.
//! - `settling_record.rs`:
//!   the binding record,
//!   its identities,
//!   and the two values the flags confirm.
//! - `settling_slot.rs`:
//!   the slot's `.git`,
//!   the registration's identity,
//!   the slot's entries and the mount table.
//! - `settling_standing_records.rs`:
//!   a standing origin cleanup record and a standing destruction plan,
//!   compared with the binding.
//! - `settling_lines.rs`:
//!   the lines a person reads.
//! - `settling_command.rs`:
//!   the entry that runs the six reads and the removals in the design's order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { settleTheRecordsOfASlot, parseSettlingArguments } from '@monochromatic-dev/git-policy-cli/worktree-clone-record-settling';
//! ```

pub mod settling_absence;
pub mod settling_command;
pub mod settling_lines;
pub mod settling_record;
pub mod settling_refusal;
pub mod settling_removals;
pub mod settling_request;
pub mod settling_slot;
pub mod settling_standing_records;
pub mod settling_steps;
pub mod settling_words;

pub use settling_command::{
    SettlingDone,
    SettlingReads,
    settle_the_records_of_a_slot,
};
pub use settling_refusal::SettlingStop;
pub use settling_request::{
    SettlingRequest,
    parse_settling_arguments,
};
pub use settling_steps::{
    SettlingStep,
    TheOrderFacts,
    WHEN_A_CHECKOUT_HAS_BEGUN,
    WHEN_NO_CHECKOUT_HAS_BEGUN,
    the_steps,
};
pub use settling_words::{
    CONFIRM_GUID_FLAG,
    CONFIRM_REMOVAL_FLAG,
    GUID_DIGITS_SHOWN,
    carries_a_placeholder,
    the_last_four_digits,
    the_usage_line,
};

/// The folder and record fixtures every control of this module starts from,
/// never part of the release executable.
#[cfg(test)]
#[path = "settling_fixtures_tests.rs"]
pub(crate) mod settling_fixtures;

/// The guest arm of this module's piece,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "settling_guest_arm_tests.rs"]
mod settling_guest_arm_tests;
