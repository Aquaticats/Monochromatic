//! What:
//!  The refusal of a mode whose step has not landed yet:
//!  the mode's own name,
//!  what it will do,
//!  the unit and plan step that run it,
//!  and that nothing was touched.
//! Why:
//!  The plan lands one handler mode per step and refuses the rest until then
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 5),
//!  so a build that answered a mode with a partial sweep would be a root program
//!  doing half of an operation nobody designed.
//!  A refusal states that nothing was opened or changed,
//!  so an operator's boot unit fails visibly rather than silently doing nothing.
//!  The sentence names what runs the mode rather than a step number alone:
//!  the shutdown unmount's own code landed with step 9,
//!  and what is missing is the unit whose `ExecStopPost` runs the mode,
//!  which step 19 installs,
//!  so a step number alone would send a reader to code that is already in the
//!  binary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! throw new AwaitingItsStep({ mode: 'shutdown-unmount', arrivesWith: 'plan step 19, whose boot unit runs it in its ExecStopPost', does: 'unmounts the mounts this boot verified' });
//! ```

/// What:
///  One mode this build answers with a refusal,
///  what runs it,
///  and the work it performs once that is in.
/// Why:
///   The three fields are everything the sentence needs,
///  and keeping them apart lets a control read the arrival without matching
///  prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Awaiting = { mode: string; arrivesWith: string; does: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AwaitingItsStep {
    /// The mode as a person types it,
    /// or the default invocation's name.
    pub mode: &'static str,
    /// The unit and the plan step that run the mode,
    /// naming what is missing rather than the code that already landed.
    pub arrives_with: &'static str,
    /// What it does once that is in.
    pub does: &'static str,
}

impl AwaitingItsStep {
    /// The refusal in words:
    /// the mode,
    /// what it will do,
    /// what runs it,
    /// and that nothing was touched.
    pub fn message(&self) -> String {
        return format!(
            "the {} mode is not built yet: it {}, and it arrives with {}. Nothing was opened, mounted, unmounted or recorded, so a boot or shutdown unit that runs this build leaves its own action undone rather than half done. Ways forward: run the mode again with a release that has it, or by hand once the unit is installed; `--help` lists the modes this build has.",
            self.mode, self.does, self.arrives_with
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mode_awaiting_its_step_tests.rs"]
mod tests;
