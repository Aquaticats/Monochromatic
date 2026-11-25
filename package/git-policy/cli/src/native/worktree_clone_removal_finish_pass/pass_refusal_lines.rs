//! What:
//!  Every sentence the finish pass prints about a plan it left standing,
//!  and the one line it prints about a record that is not a plan.
//! Why:
//!  A person reads these lines between their own commands,
//!  so each one names the plan,
//!  what the pass read that stopped it,
//!  and that nothing was removed
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  Keeping them here means one file holds every refusal's words,
//!  and a control reads the sentence from the same function the release prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(theGateRefused(planPath, said));
//! ```

/// What:
///  The line for a gate that refused before the pass read the pool.
/// Why:
///  A refusing gate makes no request at all,
///  so the registration, the binding and the plan stay,
///  and the next forwarded command reads the pool again
///  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
///  "When the route's gate refuses, or the device cannot be opened").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theGateRefused(plan, refusal);
/// ```
pub fn the_gate_refused(plan: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} is not finished: the ZFS compatibility gate refused before any pool request ({said}), so this pass made no request, and the plan, its registration and its binding stand. Ways forward: read the gate's own line, fix the module or the device it names, and run a command in this repository again.",
        plan.display()
    );
}

/// What:
///  The line for a pool that could not prove the clone gone.
/// Why:
///  A plan whose head is `gone` waits for this pass to read the pool again,
///  and a read that failed proves nothing either way,
///  so nothing is settled on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePoolCouldNotProve(plan, said);
/// ```
pub fn the_pool_could_not_prove(plan: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} is not finished: the pool cannot prove this plan's clone gone ({said}), so the plan, its registration and its binding stand, and nothing was settled.",
        plan.display()
    );
}

/// What:
///  The line for a plan whose `gone` head the live clone contradicts.
/// Why:
///  The plan's word alone settles nothing,
///  and a child of the pinned parent that carries the recorded GUID is the same clone however it is named,
///  so the pass never rewrites the plan and never settles on a contradicted record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theCloneStands(plan, liveName, recordedGuid);
/// ```
pub fn the_clone_stands(plan: &std::path::Path, live: &str, recorded_guid: &str) -> String {
    return format!(
        "The removal plan {} records its head as gone, and `{live}` carries this plan's clone GUID {recorded_guid}, so the plan's word is contradicted: the plan, its registration and its binding stand, nothing was settled, and the plan was not rewritten.",
        plan.display()
    );
}

/// What:
///  The line for a reused clone name beside a registration of this plan's.
/// Why:
///  By GUID this plan's clone is gone,
///  and a live registration of this plan's still names work to settle,
///  so the pass refuses the reuse and leaves every record for a removal or a person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theNameWasReusedWithARegistration(plan, clone, recorded, live);
/// ```
pub fn the_name_was_reused_with_a_registration(plan: &std::path::Path, clone: &str, recorded_guid: &str, live_guid: &str) -> String {
    return format!(
        "The removal plan {} records `{clone}` with clone GUID {recorded_guid}, the name now answers GUID {live_guid}, and a registration of this plan's still stands: nothing was settled, and the plan, its registration and its binding stay. Ways forward: `monochromatic-git-policy-cli doctor` names this plan and its registration.",
        plan.display()
    );
}

/// What:
///  The line for a binding that could not be read.
/// Why:
///  The binding is the last proof that this plan's own records were this clone's,
///  so an unreadable one leaves both records for a person
///  (the project's answer on a record that cannot be read).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theBindingCouldNotBeRead(plan, binding, said);
/// ```
pub fn the_binding_could_not_be_read(plan: &std::path::Path, binding: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} and its binding {} stay: the binding cannot be read ({said}), so nothing proves this plan's binding is finished. Nothing was removed, and a person reads the two records before acting.",
        plan.display(),
        binding.display()
    );
}

/// What:
///  The line for a binding whose recorded slot is another path than the plan's.
/// Why:
///  A binding filed under this slot's name that names another path is a record a person must read,
///  and removing the plan beside it would leave that record with no witness.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theBindingNamesAnotherSlot(plan, binding, planSlot, bindingSlot);
/// ```
pub fn the_binding_names_another_slot(plan: &std::path::Path, binding: &std::path::Path, plan_slot: &str, binding_slot: &str) -> String {
    return format!(
        "The removal plan {} and its binding {} stay: the plan's slot path is `{plan_slot}` and the binding's `slot=` is `{binding_slot}`, so the two records do not name one slot. Nothing was removed.",
        plan.display(),
        binding.display()
    );
}

/// What:
///  The line for a binding of this plan's own clone whose origin fields disagree.
/// Why:
///  The pass's condition is one clone and one origin,
///  so a disagreement leaves both records rather than removing a witness
///  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
///  "The condition a finisher meets").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theBindingDisagrees(plan, binding, field, bindingValue, planValue);
/// ```
pub fn the_binding_disagrees(plan: &std::path::Path, binding: &std::path::Path, field: &str, binding_value: &str, plan_value: &str) -> String {
    return format!(
        "The removal plan {} and its binding {} stay: the binding's `{field}` is `{binding_value}` and the plan's is `{plan_value}`, so the two records do not name one clone and one origin. Nothing was removed.",
        plan.display(),
        binding.display()
    );
}

/// What:
///  The line for a registration the settle refused.
/// Why:
///  A settle that refused leaves the registration, the binding and the plan,
///  and the next command may try again,
///  so the line names the step and the settle's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theRegistrationCouldNotBeSettled(plan, step, said);
/// ```
pub fn the_registration_could_not_be_settled(plan: &std::path::Path, step: &str, said: &str) -> String {
    return format!(
        "The removal plan {} is not finished: the Git registration could not be settled ({step}: {said}), so the plan, its registration and its binding stand. Ways forward: read the registration and the managed lock the line names, and run a command in this repository again.",
        plan.display()
    );
}

/// What:
///  The line for a registration whose own record could not be read.
/// Why:
///  A registration directory the pass cannot read is no proof of whose it is,
///  so nothing is settled and the plan stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theRegistrationCouldNotBeRead(plan, registration, said);
/// ```
pub fn the_registration_could_not_be_read(plan: &std::path::Path, registration: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} is not finished: the registration {} cannot be read ({said}), so the plan, its registration and its binding stand. Nothing was removed.",
        plan.display(),
        registration.display()
    );
}

/// What:
///  The line for a plan whose `targetHex` names no plain slot name.
/// Why:
///  The slot is the key of the claim and of the binding,
///  so a plan that names no plain slot name is left for a person rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theSlotIsNotAName(plan, target, said);
/// ```
pub fn the_slot_is_not_a_name(plan: &std::path::Path, target: &str, said: &str) -> String {
    return format!(
        "The removal plan {} names the slot path `{target}`, which keys no slot ({said}), so this pass took no claim and removed nothing. Ways forward: read the plan's `targetHex` field, and `monochromatic-git-policy-cli doctor` reports the record beside it.",
        plan.display()
    );
}

/// What:
///  The line for a claim the pass could not take.
/// Why:
///  A claim that could not be written proves nothing about who owns the slot,
///  so the pass touches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theClaimCouldNotBeTaken(plan, said);
/// ```
pub fn the_claim_could_not_be_taken(plan: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} is not finished: the slot's claim could not be taken ({said}), so the plan, its registration and its binding stand. Ways forward: read the claim folder the line names, and run a command in this repository again.",
        plan.display()
    );
}

/// What:
///  The line for a plan whose own file could not be removed.
/// Why:
///  The plan is the last witness,
///  and it stays when the unlink failed,
///  so a person reads what stands and what the call answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanCouldNotBeRemoved(plan, said);
/// ```
pub fn the_plan_could_not_be_removed(plan: &std::path::Path, said: &str) -> String {
    return format!(
        "The removal plan {} stays: its own file could not be removed ({said}), so nothing else was removed. The plan is the last witness of this removal, and the next command reads it again.",
        plan.display()
    );
}

/// What:
///  The line for an entry in the plan folder that does not read as a plan.
/// Why:
///  A plan that cannot be read is never moved or removed,
///  and what it holds may be the only record of a destroyed snapshot,
///  so the line names the entry and says that reading it destroyed nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanWasNotRead(entry, said);
/// ```
pub fn the_plan_was_not_read(entry: &std::path::Path, said: &str) -> String {
    return format!(
        "The entry {} in the destruction plan folder is not a plan this build reads ({said}), so it was left alone and nothing was destroyed by reading it. Ways forward: `monochromatic-git-policy-cli doctor` reports the record and a person decides what stands there.",
        entry.display()
    );
}

/// What:
///  The line for an entry whose name is no plan's name.
/// Why:
///  A foreign entry is reported and never read,
///  because a reader that guessed at one would read a person's file as a removal's record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theEntryIsNoPlanName(entry);
/// ```
pub fn the_entry_is_no_plan_name(entry: &std::path::Path) -> String {
    return format!(
        "The entry {} in the destruction plan folder carries no clone GUID's name, so this pass left it alone and read nothing in it. Ways forward: `monochromatic-git-policy-cli doctor` reports every foreign entry by path.",
        entry.display()
    );
}

/// What:
///  The line for a plan folder that is not a folder, or that could not be listed.
/// Why:
///  Reading no plan is the whole answer,
///  and the sentence names the path so a person can look at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theFolderCouldNotBeListed(folder, said);
/// ```
pub fn the_folder_could_not_be_listed(folder: &std::path::Path, said: &str) -> String {
    return format!(
        "The destruction plan folder {} could not be listed ({said}), so this pass read no plan and removed nothing. Ways forward: read that path, and `monochromatic-git-policy-cli doctor` reports the records it holds.",
        folder.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pass_refusal_lines_tests.rs"]
mod tests;
