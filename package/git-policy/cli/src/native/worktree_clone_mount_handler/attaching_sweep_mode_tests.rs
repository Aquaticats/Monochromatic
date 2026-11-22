//! What:
//!  Controls of the sweep's refusal:
//!  its mode name,
//!  its plan step,
//!  that running it answers the refusal rather than a sweep's line,
//!  and that a run lists no store name and opens no store folder.
//! Why:
//!  The default invocation is what the installed ZED event starts,
//!  so a build that answered it with success would report a sweep nobody ran
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 8).
//!  A sweep that removed a killed writer's staging file would list the store folder once per run,
//!  and the project answered that plan step 18's `doctor` reports and removes those forms
//!  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
//!  "Who removes a killed writer's staging file").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING).toEqual({ mode: 'sweep (no argument)', step: 8, does: expect.any(String) });
//! ```

use super::{AWAITING, run};
use crate::worktree_clone_mount::WHEN_THE_STORE_IS_LISTED;
use crate::worktree_clone_mount::slot_identity_store::BEFORE_THE_STORE_OPENS;
use std::sync::{Arc, Mutex};

/// The refusal names the default invocation and step 8.
#[test]
fn the_refusal_names_the_default_invocation_and_its_step() {
    assert_eq!(AWAITING.mode, "sweep (no argument)");
    assert_eq!(AWAITING.step, 8);
    assert!(AWAITING.message().contains("plan step 8"), "{}", AWAITING.message());
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}

/// What:
///  A sweep run lists no store name,
///  opens no store folder and reads no record,
///  and still answers its step's refusal.
/// Why:
///  The store's listing is the adoption's and `doctor`'s,
///  because a run that removes a staging name must list the folder to find it,
///  and the project answered that step 18's `doctor` is where that report and removal belong.
///  The listing hook stands where either listing call starts,
///  so a run that listed a store folder is caught whatever the listing then answers;
///  the open hook stands at the store's own open,
///  because this host has no store folder at the design's path,
///  and a plant that put the removal back would reach that open and stop there,
///  passing a control that watched the listing alone.
#[test]
fn a_sweep_run_lists_nothing() {
    let reached: Arc<Mutex<Vec<&'static str>>> = Arc::default();
    let listed: Arc<Mutex<Vec<&'static str>>> = Arc::clone(&reached);
    WHEN_THE_STORE_IS_LISTED.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            listed.lock().expect("the control's own list").push("listed");
        }));
    });
    let opened: Arc<Mutex<Vec<&'static str>>> = Arc::clone(&reached);
    BEFORE_THE_STORE_OPENS.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            opened.lock().expect("the control's own list").push("opened");
        }));
    });
    let answered: Result<String, super::AwaitingItsStep> = run();
    WHEN_THE_STORE_IS_LISTED.with(|hook| { *hook.borrow_mut() = None });
    BEFORE_THE_STORE_OPENS.with(|hook| { *hook.borrow_mut() = None });
    let touched: Vec<&'static str> = reached.lock().expect("the control's own list").clone();
    assert_eq!(answered, Err(AWAITING), "the sweep still refuses its own step");
    assert!(touched.is_empty(), "a sweep run reaches no store folder and lists no name, and this one did: {touched:?}");
}
