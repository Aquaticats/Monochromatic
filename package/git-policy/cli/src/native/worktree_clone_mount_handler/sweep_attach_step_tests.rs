//! What:
//!  Controls of the attach step's seam:
//!  the request a sweep hands in,
//!  and the four answers a caller must tell apart.
//! Why:
//!  The real step mounts a real clone and only the throwaway guest may run it,
//!  so what this host proves is that a control's step is driven with the
//!  request the sweep built,
//!  and that a retained mount and a rollback are different answers rather than
//!  one "failure".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(step.attach(request)).toEqual({ kind: 'attached', mount });
//! ```

use super::{AttachRequest, AttachStepOutcome, CloneAttachStep};
use crate::worktree_clone_mount::MountIdentity;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// A step that answers from a script and remembers what it was asked,
/// which is how a control sees the sweep's own request.
struct ScriptedStep {
    /// The answer it gives.
    answer: AttachStepOutcome,
    /// The requests it was given.
    asked: RefCell<Vec<(PathBuf, String, String, PathBuf)>>,
}

impl CloneAttachStep for ScriptedStep {
    fn attach(&self, request: &AttachRequest<'_>) -> AttachStepOutcome {
        self.asked.borrow_mut().push((request.checkout.to_path_buf(), String::from(request.slot), String::from(request.clone), request.store.to_path_buf()));
        return self.answer.clone();
    }
}

/// The mount a control's step says it made.
fn mounted() -> MountIdentity {
    return MountIdentity {
        mount_id: 41,
        parent_mount_id: 40,
        mount_point: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"),
        source: String::from("zpcachyos/nosnap/clones/one"),
        file_system_type: String::from("zfs"),
        device: 57,
        inode: 1_337,
        observed_path: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"),
    };
}

/// The four answers one step can give stay four answers,
/// and each reads as what happened rather than as a summary.
#[test]
fn the_four_answers_stay_four() {
    let answers: [AttachStepOutcome; 3] = [
        AttachStepOutcome::RolledBack { said: String::from("the mount 41 left the kernel through the descriptor the attach acted on") },
        AttachStepOutcome::Retained { said: String::from("the mount 41 is still attached at /tmp/elsewhere") },
        AttachStepOutcome::Refused { said: String::from("the walk refused the slot") },
    ];
    for answer in answers {
        assert_ne!(answer, AttachStepOutcome::Attached { mount: mounted() }, "only a proved place is an attach: {answer:?}");
    }
    assert_ne!(
        AttachStepOutcome::RolledBack { said: String::from("left") },
        AttachStepOutcome::Retained { said: String::from("left") },
        "a rolled-back mount and a retained one are different answers even when their sentences start alike"
    );
}

/// The step is handed the checkout and the slot and the clone and the store the sweep
/// built,
/// and nothing else.
#[test]
fn the_step_is_handed_the_four_names_of_one_clone() {
    let step: ScriptedStep = ScriptedStep { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: RefCell::new(Vec::new()) };
    let request: AttachRequest<'_> = AttachRequest {
        checkout: Path::new("/home/user/Monochromatic"),
        slot: "one",
        clone: "zpcachyos/nosnap/clones/one",
        store: Path::new("/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities"),
    };
    let outcome: AttachStepOutcome = step.attach(&request);
    assert_eq!(outcome, AttachStepOutcome::Attached { mount: mounted() });
    let asked = step.asked.borrow();
    assert_eq!(asked.len(), 1);
    let (checkout, slot, clone, store) = &asked[0];
    assert_eq!(checkout, Path::new("/home/user/Monochromatic"));
    assert_eq!(slot, "one");
    assert_eq!(clone, "zpcachyos/nosnap/clones/one");
    assert_eq!(store, Path::new("/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities"));
}
