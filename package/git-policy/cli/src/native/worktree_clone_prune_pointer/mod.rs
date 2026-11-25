//! What:
//!  The underlay pointer the add keeps for `git worktree prune`,
//!  independent of the clone's own pointer:
//!  what it says against the bytes the record holds,
//!  the one write that puts it back,
//!  the facts a damaged link is judged from,
//!  and the pass a prune runs before Git starts.
//! Why:
//!  Plan step 12 builds the pointer half of prune protection
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 12";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees",
//!  and "Prune protection while unmounted").
//!  The registration names the slot,
//!  Git's prune reads the slot's `.git`,
//!  and the add records those exact bytes,
//!  so a pointer an account deleted or changed is restorable rather than
//!  guessed,
//!  and the prune stops when nothing can restore it.
//!  A mounted clone shows its own copy of the pointer at the same path and an
//!  unmounted one shows the underlay's,
//!  so every reading says which copy it took.
//!
//! Files in this folder,
//! each using only the files listed before it:
//!
//! - `pointer_state.rs`:
//!   the bytes at the slot against the recorded ones,
//!   and which copy of the pointer the slot shows.
//! - `pointer_restore.rs`:
//!   the one write that puts the recorded bytes back,
//!   by a staging name and one rename.
//! - `link_facts.rs`:
//!   every fact a damaged link is judged from,
//!   read once.
//! - `prune_guard.rs`:
//!   the pass over every bound slot before a prune,
//!   its restores and the slots nothing restores.
//! - `prune_removes_nothing.rs`:
//!   whether Git's own parser reads one prune as a command that prunes
//!   nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { readPointerStateAt, restoreTheSlotPointer } from '@monochromatic-dev/git-policy-cli/worktree-clone-prune-pointer';
//! import { linkFactsOf, guardThePrune } from '@monochromatic-dev/git-policy-cli/worktree-clone-prune-pointer';
//! ```

/// The bytes at the slot against the recorded ones,
/// and which copy they are.
pub mod pointer_state;

/// The one write that puts the recorded pointer back.
pub mod pointer_restore;

/// Every fact a damaged link is judged from.
pub mod link_facts;

/// The pass over every bound slot before a prune.
pub mod prune_guard;

/// Whether Git's own parser reads one prune as a command that prunes nothing.
pub mod prune_removes_nothing;

pub use prune_removes_nothing::a_prune_that_removes_nothing;

pub use link_facts::{LinkFacts, RecordedLink, link_facts_of, recorded_link_of, registration_names_the_slot, the_registration_stands};
pub use pointer_restore::{POINTER_FILE_MODE, RestoreOutcome, restore_the_slot_pointer};
pub use pointer_state::{PointerSite, PointerState, pointer_site_of, pointer_state_of, read_pointer_state_at};
pub use prune_guard::{PruneGuard, PruneRefusal, guard_the_prune};
