//! What:
//!  The immutable covered entry of the checkout's underlay:
//!  what the kernel says about the entry at the checkout path,
//!  why a reading of it is refused,
//!  and the verdict a boot takes before it attaches a clone or trusts a
//!  reimported or rolled-back checkout.
//! Why:
//!  The account can replace an unmounted checkout path on its own dataset,
//!  so the route decided an ordinary checkout mountpoint plus an immutable
//!  covered entry,
//! and the flag must be proved before an automated attach,
//!  including after a reimport or a rollback,
//! and not merely at installation
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "One-time move and checkout boot protection",
//!  and "Checkout boot mount protection" under "Project decisions").
//!  The measured facts the verdict rests on:
//!  an immutable checkout underlay refused the account's rename and removal
//!  while unmounted,
//!  allowed ordinary mounting,
//!  kept the flag across a reboot with a manual import and key load,
//!  and lost it when the parent was rolled back to a snapshot taken before the
//!  flag (VM item 8).
//!  A check that cannot prove the flag is reported as unproven rather than as
//!  protected or as a failure,
//!  which is how a check with no answer is reported everywhere else on this
//!  route.
//!  The entry read is a kernel read of the path while the checkout is unmounted:
//!  a mounted checkout makes the path another filesystem,
//!  whose device differs from its parent's,
//!  and the verdict names that rather than believing the flag off the wrong
//!  object.
//!
//! Files in this folder,
//! each using only the files listed before it:
//!
//! - `underlay_entry_read.rs`:
//!   the kernel's facts about the entry and its parent,
//!   the refusal,
//!   and the live read.
//! - `underlay_entry_verdict.rs`:
//!   the three answers a boot takes,
//!   the reasons and the line each prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = verify(reader.read(checkout), account); // Protected | Unprotected | Unproven
//! ```

pub mod underlay_entry_read;
pub mod underlay_entry_verdict;

pub use underlay_entry_read::{CoveredEntryFacts, EntryReadRefusal, EntryReader, LiveEntryReader};
pub use underlay_entry_verdict::{UnderlayVerdict, UnprotectedReason, UnprovenReason, verify};
