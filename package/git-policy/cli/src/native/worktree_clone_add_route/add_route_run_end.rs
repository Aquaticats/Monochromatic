//! What:
//!  The end of one add:
//! the repair of the new worktree,
//! the hand-over of its
//!  trash to the service,
//! and the record's last phase.
//! Why:
//!  A filled worktree is whole before either runs,
//! so neither fails the add:
//!  a trash the service cannot take stays named by its record,
//! and a repair
//!  that finds nothing reports nothing
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//! A12).
//!  `add_route_run.rs` would go over the file-size limit carrying the end beside
//!  the runner,
//! and the end is the one step that must not change the add's
//!  answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! finishTheAdd(steps, trash, record); // repair, hand over, mark filled
//! ```

use super::add_route_steps::{
    StepRefusal,
    Steps,
    write_the_record,
};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use std::io::Write as _;

/// Repair the new worktree and hand its trash over;
/// neither fails the add.
pub(super) fn finish_the_add(
    steps: &Steps<'_>,
    trash: Option<crate::worktree_trash::PendingTrash>,
    record: &mut BindingRecord,
) -> Result<(), StepRefusal> {
    if let Some(pending) = trash {
        let mut notices = std::io::stderr();
        if let Err(reason) =
            crate::worktree_trash::hand_over_to_the_service(steps.context, &pending, &mut notices)
        {
            let _written: std::io::Result<()> = writeln!(
                std::io::stderr(),
                "monochromatic-git-policy-cli: the trash of {} could not be handed to the service ({reason}); its record still names it",
                steps.route.destination.display()
            );
        }
    }
    #[cfg(target_os = "linux")]
    {
        let environment: crate::package_repair::RepairEnvironment =
            crate::package_repair::RepairEnvironment::from_process();
        let request = crate::package_repair::RepairRequest {
            git: steps.context,
            source: steps.route.source_root.as_path(),
            destination: steps.route.destination.as_path(),
            switch: crate::package_repair::RepairSwitch::from_arguments(steps.arguments),
            environment: &environment,
        };
        let mut findings = std::io::stderr();
        crate::package_repair::repair_worktree(&request, &mut findings);
    }
    record.phase = Phase::Filled;
    return write_the_record(steps, record);
}
