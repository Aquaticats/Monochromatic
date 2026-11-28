//! What:
//!  Plan step 21's guest arms in one throwaway guest:
//!  the dependency preparation after an add,
//!  the watch rebinding after a clone's fill,
//!  and the queued repair beside the inline one.
//! Why:
//!  The plan's acceptance asks for "real dependency preparation after add and
//!  after migration",
//!  for "queued and inline repair,
//!  Cargo prewarm and cross-filesystem lock recovery,
//!  mise and launcher repair,
//!  and watch rebinding in the editor"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 21),
//!  and every piece of it acts on a clone of the design's datasets.
//!  `SPEC.md` says the comparison of the watches is reached by no command,
//!  and
//!  the skip of the separate ignored-file copy is not built,
//!  so the arms drive what `main` holds and the wiring stays the wiring step's.
//!  "After migration" is not runnable here:
//!  the owner's migration has not run,
//!  so the guest makes the checkout dataset the step 19 design describes rather
//!  than finding one.
//!  Each arm clears the slot it made,
//!  so one guest serves the whole run and the driver destroys it afterwards.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = prepare(); pnpm(); cargo(); launcher(); watch(); queuedAndInline();
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::Fixture;
use super::zfs_step_21_guest_layout::{
    guest,
    mounted_at,
};
use super::zfs_step_21_launcher_arm::the_launcher_arm;
use super::zfs_step_21_pnpm_and_cargo_arms::{
    the_cargo_arm,
    the_pnpm_arm,
};
use super::zfs_step_21_queued_arm::the_queued_and_inline_arms;
use super::zfs_step_21_watch_arm::the_watch_arm;

/// What:
///  Run every arm of the plan's step 21 in the guest the driver prepared.
/// Why:
///  One test runs the arms in the order a reader reads the record:
///  the two installs,
///  the queued pair,
///  the launcher repair,
///  and the watch,
///  each clearing the slot it made so the next one starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theStep21GuestArms();
/// ```
#[test]
fn the_step_21_guest_arms() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    println!(
        "STEP21 fixture checkout={} container={} rounds=3",
        fixture.checkout, fixture.container
    );
    the_pnpm_arm(&fixture);
    the_queued_and_inline_arms(&fixture);
    the_cargo_arm(&fixture);
    the_launcher_arm(&fixture);
    the_watch_arm(&fixture);
    assert!(
        !mounted_at(fixture.root()),
        "the arms left a mount over the checkout root"
    );
    println!("STEP21 done");
}
