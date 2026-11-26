//! What:
//!  The report of a damaged link between a worktree and its Git registration:
//!  what was found,
//!  which copy of the pointer was read,
//!  what is kept,
//!  what a repair can and cannot restore,
//!  and the one next safe command,
//!  printed live for one slot.
//! Why:
//!  Plan step 12 builds the report that says what a damaged link can and cannot
//!  restore
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 12";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees").
//!  A surviving registration can have its pointer rebuilt by
//!  `git worktree repair`,
//!  while a registration Git already pruned takes HEAD,
//!  the index and the worktree's lock with it,
//!  and a report that hid the difference would invite a person to trust a
//!  rebuilt pointer that brings nothing back.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `repair_report.rs`:
//!   the four states of one link and the lines a person reads.
//! - `live_link_report.rs`:
//!   the same report read from the record and the machine,
//!   and the paths one `git worktree repair` names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { reportOf, theLinksReport } from '@monochromatic-dev/git-policy-cli/worktree-clone-link-repair';
//! ```

/// The four states of one link and the lines a person reads.
pub mod repair_report;

/// The report read from the record and the machine.
pub mod live_link_report;

pub use live_link_report::{
    SlotAtTheCommand,
    repair_paths,
    the_links_report,
    the_slot_at,
    the_slot_at_with,
};
pub use repair_report::{
    LinkRepair,
    LinkRepairReport,
    class_of,
    link_report_lines,
    link_report_of,
};
