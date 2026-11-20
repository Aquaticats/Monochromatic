//! What:
//!  The refusal of a mode whose step has not landed yet:
//!  the mode's own name,
//!  what it will do,
//!  and the plan step that builds it.
//! Why:
//!  The plan lands one handler mode per step and refuses the rest until then
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 5),
//!  so a build that answered a mode with a partial sweep would be a root program
//!  doing half of an operation nobody designed.
//!  A refusal states that nothing was opened or changed,
//!  so an operator's boot unit fails visibly rather than silently doing nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! throw new AwaitingItsStep({ mode: 'boot-reconcile', step: 6, does: 'verifies the start gate' });
//! ```

/// What:
///  One mode this build answers with a refusal,
///  the plan step that lands it,
///  and the work it performs once that step is in.
/// Why:
///   The three fields are everything the sentence needs,
///  and keeping them apart lets a control read the step without matching prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Awaiting = { mode: string; step: number; does: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AwaitingItsStep {
    /// The mode as a person types it,
    /// or the default invocation's name.
    pub mode: &'static str,
    /// The plan step that builds it.
    pub step: u16,
    /// What it does once that step is in.
    pub does: &'static str,
}

impl AwaitingItsStep {
    /// The refusal in words:
    /// the mode,
    /// what it will do,
    /// the step that lands it,
    /// and that nothing was touched.
    pub fn message(&self) -> String {
        return format!(
            "the {} mode of mount-git-worktree-clone-datasets is not built yet: it {}, and plan step {} lands it. Nothing was opened, mounted, unmounted or recorded, so a boot or shutdown unit that runs this build leaves its own action undone rather than half done. Ways forward: run the same command again with a release whose plan steps have landed, or run the mode only after that step's code is in; `mount-git-worktree-clone-datasets --help` lists the modes this build has.",
            self.mode, self.does, self.step
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mode_awaiting_its_step_tests.rs"]
mod tests;
