//! What:
//!  The finish pass's controls over one plan's tail:
//!  the slot's claim,
//!  the gate,
//!  the pool's own proof of absence,
//!  the reused name,
//!  the registration,
//!  the binding and its files,
//!  the plan,
//!  and the claim's release.
//! Why:
//!  Each arm is one state the design names for the pass,
//!  planted on a disposable repository so no arm opens `/dev/zfs`
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass's live read and its tail").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = thePass(fixture, liveFolder, gate, probe, settle);
//! expect(settle.calls).toBe(0);
//! ```

use crate::process::process_identity::current_birth_identity;
use crate::single_owner_claim::plant_claim_for_tests;
use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    claim_prefix,
    clone_inventory_name,
    origin_inventory_name,
};
use crate::worktree_clone_removal_finish_pass::pass_fixtures::{
    AGate,
    ANOTHER_CLONE_GUID,
    AProbe,
    ASettle,
    SLOT,
    a_finished_removal,
    a_plan_for,
    stands,
    the_home,
    the_pass,
    the_plan_folder,
    write_the_plan,
};
use crate::worktree_clone_removal_finish_pass::the_plans_folder::TheLivePlansFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use std::path::{
    Path,
    PathBuf,
};

/// The name of the plan's own file in the plan folder.
fn the_plan_path(
    fixture: &RemovalFixture,
    plan: &DestructionPlan,
) -> PathBuf {
    return the_plan_folder(fixture).join(plan.facts.clone_guid.file_name());
}

/// Rewrite one binding line's value,
/// which is how an arm changes exactly one field.
fn rewrite_the_binding(
    fixture: &RemovalFixture,
    key: &str,
    value: &str,
) {
    let path: PathBuf = the_home(fixture).join(binding_name(SLOT));
    let text: String =
        std::fs::read_to_string(path.as_path()).expect("the fixture's binding reads");
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        lines.push(match line.starts_with(format!("{key}=").as_str()) {
            true => format!("{key}={value}"),
            false => String::from(line),
        });
    }
    std::fs::write(path.as_path(), format!("{}\n", lines.join("\n")))
        .expect("the binding is rewritten with one field changed");
}

/// What every arm must still find standing after a pass that changed nothing.
fn everything_stands(
    fixture: &RemovalFixture,
    plan: &DestructionPlan,
    said: &str,
) {
    let home: PathBuf = the_home(fixture);
    for name in [
        origin_inventory_name(SLOT),
        clone_inventory_name(SLOT),
        binding_name(SLOT),
    ] {
        assert!(
            stands(home.join(name.as_str()).as_path()),
            "{said}: `{name}` stands"
        );
    }
    assert!(
        stands(the_plan_path(fixture, plan).as_path()),
        "{said}: the plan stands"
    );
    assert!(
        stands(fixture.registration.as_path()),
        "{said}: the registration stands"
    );
}

#[test]
fn the_whole_tail_runs_in_the_lists_order_under_the_slots_claim() {
    let (fixture, plan) = a_finished_removal("finish-pass-tail");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(settle.calls.get(), 1, "the registration is settled once");
    assert_eq!(
        settle.slots.borrow().as_slice(),
        &[String::from(SLOT)],
        "the settle is asked about the plan's own slot"
    );
    assert_eq!(
        lines.len(),
        1,
        "one line says what the finish removed, and this run printed {lines:?}"
    );
    let home: PathBuf = the_home(&fixture);
    for name in [
        origin_inventory_name(SLOT),
        clone_inventory_name(SLOT),
        binding_name(SLOT),
    ] {
        assert!(
            !stands(home.join(name.as_str()).as_path()),
            "`{name}` is gone"
        );
    }
    assert!(
        !stands(the_plan_path(&fixture, &plan).as_path()),
        "the plan went last"
    );
    assert!(
        !stands(the_plan_folder(&fixture).as_path()),
        "the empty plan folder went with it"
    );
    let claim: PathBuf = home.join(format!("{SLOT}.claim.1"));
    let text: String =
        std::fs::read_to_string(claim.as_path()).expect("the claim the pass took stands");
    assert!(
        text.contains("released"),
        "the claim was given back after the plan's removal: {text}"
    );
}

#[test]
fn a_live_owner_of_the_claim_ends_the_pass_with_nothing_touched() {
    let (fixture, plan) = a_finished_removal("finish-pass-busy");
    let home: PathBuf = the_home(&fixture);
    let birth: String = current_birth_identity().expect("this process's own birth identity reads");
    plant_claim_for_tests(
        home.as_path(),
        claim_prefix(SLOT).as_os_str(),
        1,
        i64::from(std::process::id()),
        birth.as_str(),
    );
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert!(
        lines.is_empty(),
        "a live removal finishes this slot, so the pass says nothing, and this run printed {lines:?}"
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "nothing is settled while another process holds the slot"
    );
    everything_stands(&fixture, &plan, "a claim of a live owner");
}

#[test]
fn a_refusing_gate_settles_nothing_and_names_the_plan() {
    let (fixture, plan) = a_finished_removal("finish-pass-gate");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::refusing("the device cannot be opened"),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line names the refusal");
    assert!(
        lines[0].contains("compatibility gate refused"),
        "the line names the gate: {}",
        lines[0]
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "a refused gate makes no request and settles nothing"
    );
    everything_stands(&fixture, &plan, "a refusing gate");
}

#[test]
fn a_pool_that_cannot_prove_the_absence_settles_nothing() {
    let (fixture, plan) = a_finished_removal("finish-pass-pool");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent().refusing("the pool refused the read"),
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "one line names the read that proved nothing"
    );
    assert!(
        lines[0].contains("cannot prove this plan's clone gone"),
        "the line says what is unproved: {}",
        lines[0]
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "nothing is settled on a read that proved nothing"
    );
    everything_stands(&fixture, &plan, "a pool that could not prove");
}

#[test]
fn a_live_clone_contradicts_the_plans_head_and_nothing_settles() {
    let (fixture, plan) = a_finished_removal("finish-pass-live-clone");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::standing(),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line names the live clone");
    assert!(
        lines[0].contains("contradicted"),
        "the line says the plan's word is contradicted: {}",
        lines[0]
    );
    assert_eq!(settle.calls.get(), 0);
    everything_stands(&fixture, &plan, "a live clone under the recorded name");
}

#[test]
fn a_child_carrying_the_recorded_guid_is_the_same_clone_under_another_name() {
    let (fixture, plan) = a_finished_removal("finish-pass-renamed");
    let settle: ASettle = ASettle::succeeding();
    let probe: AProbe = AProbe::renamed_to("tank/wt/fix-flaky-test-renamed");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &probe,
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "one line names the clone that still answers"
    );
    assert!(
        lines[0].contains("fix-flaky-test-renamed"),
        "the line names the live clone: {}",
        lines[0]
    );
    assert_eq!(settle.calls.get(), 0);
    everything_stands(
        &fixture,
        &plan,
        "a renamed clone carrying the recorded GUID",
    );
}

#[test]
fn a_reused_name_beside_a_registration_of_this_plans_refuses_the_reuse() {
    let (fixture, plan) = a_finished_removal("finish-pass-reused");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::reused(ANOTHER_CLONE_GUID),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line names the reuse");
    assert!(
        lines[0].contains(format!("the name now answers GUID {ANOTHER_CLONE_GUID}").as_str()),
        "the line names both GUIDs: {}",
        lines[0]
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "a registration of this plan's still names work to settle"
    );
    everything_stands(
        &fixture,
        &plan,
        "a reused name beside a registration of this plan's",
    );
}

#[test]
fn a_reused_name_beside_no_registration_and_another_clones_binding_removes_the_plan() {
    let (fixture, plan) = a_finished_removal("finish-pass-reused-gone");
    std::fs::remove_dir_all(fixture.registration.as_path()).expect("the registration is gone");
    rewrite_the_binding(
        &fixture,
        "cloneGuid",
        ANOTHER_CLONE_GUID.to_string().as_str(),
    );
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::reused(ANOTHER_CLONE_GUID),
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "one line says what the pass removed and why"
    );
    assert!(
        lines[0].contains("Removed the last removal plan"),
        "the line says the plan went: {}",
        lines[0]
    );
    assert!(
        !stands(the_plan_path(&fixture, &plan).as_path()),
        "the plan went"
    );
    let home: PathBuf = the_home(&fixture);
    assert!(
        stands(home.join(binding_name(SLOT)).as_path()),
        "the binding names another clone and is left"
    );
    assert!(
        stands(home.join(origin_inventory_name(SLOT)).as_path()),
        "its inventories are left with it"
    );
    assert_eq!(settle.calls.get(), 0, "no registration of this plan stood");
}

#[test]
fn a_binding_whose_origin_fields_differ_leaves_both_records() {
    let (fixture, plan) = a_finished_removal("finish-pass-binding-differs");
    rewrite_the_binding(&fixture, "originSnapshotGuid", "1");
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(lines.len(), 1, "one line names both records");
    assert!(
        lines[0].contains("originSnapshotGuid"),
        "the line names the field that differs: {}",
        lines[0]
    );
    assert_eq!(
        settle.calls.get(),
        1,
        "the registration is settled before the binding is read"
    );
    everything_stands(&fixture, &plan, "a binding whose origin fields differ");
}

#[test]
fn a_binding_that_names_another_slot_leaves_both_records() {
    let (fixture, plan) = a_finished_removal("finish-pass-binding-slot");
    rewrite_the_binding(
        &fixture,
        "slot",
        path_as_hexadecimal(Path::new("/elsewhere/fix-flaky-test")).as_str(),
    );
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert_eq!(lines.len(), 1, "one line names both records");
    assert!(
        lines[0].contains("`slot`"),
        "the line names the field: {}",
        lines[0]
    );
    everything_stands(
        &fixture,
        &plan,
        "a binding filed under this slot's name that names another path",
    );
}

#[test]
fn a_binding_that_cannot_be_read_leaves_both_records() {
    let (fixture, plan) = a_finished_removal("finish-pass-binding-unreadable");
    // The registration went in an earlier run, so the pass reaches step 12 with no settle to make and
    // the binding's own read is what it meets.
    std::fs::remove_dir_all(fixture.registration.as_path())
        .expect("the registration was settled already");
    let binding: PathBuf = the_home(&fixture).join(binding_name(SLOT));
    let mut permissions: std::fs::Permissions = std::fs::metadata(binding.as_path())
        .expect("the binding's metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o666);
    std::fs::set_permissions(binding.as_path(), permissions)
        .expect("the binding is made writable by others");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert_eq!(lines.len(), 1, "one line names the unreadable record");
    assert!(
        lines[0].contains("cannot be read"),
        "the line says the binding cannot be read: {}",
        lines[0]
    );
    assert!(stands(binding.as_path()), "the binding stands");
    assert!(
        stands(
            the_home(&fixture)
                .join(origin_inventory_name(SLOT))
                .as_path()
        ),
        "its inventories stand"
    );
    assert!(
        stands(the_plan_path(&fixture, &plan).as_path()),
        "the plan stands"
    );
}

#[test]
fn a_binding_of_another_clone_is_left_and_the_plan_goes() {
    let (fixture, plan) = a_finished_removal("finish-pass-binding-other-clone");
    std::fs::remove_dir_all(fixture.registration.as_path()).expect("the registration is gone");
    rewrite_the_binding(
        &fixture,
        "cloneGuid",
        ANOTHER_CLONE_GUID.to_string().as_str(),
    );
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert_eq!(lines.len(), 1, "one line says what went and what stayed");
    assert!(
        lines[0].contains("the binding at the slot names another clone"),
        "the line names the new binding: {}",
        lines[0]
    );
    assert!(
        !stands(the_plan_path(&fixture, &plan).as_path()),
        "the plan went"
    );
    assert!(
        stands(the_home(&fixture).join(binding_name(SLOT)).as_path()),
        "the new slot's binding is left"
    );
}

#[test]
fn a_binding_already_gone_leaves_the_plan_for_this_pass() {
    let (fixture, plan) = a_finished_removal("finish-pass-binding-gone");
    let home: PathBuf = the_home(&fixture);
    std::fs::write(
        home.join(clone_inventory_name(SLOT)),
        b"# a clone inventory\n",
    )
    .expect("a clone inventory a kill left");
    std::fs::remove_file(home.join(binding_name(SLOT)).as_path())
        .expect("the binding went in an earlier run");
    std::fs::remove_dir_all(fixture.registration.as_path())
        .expect("the registration was settled before it");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &ASettle::succeeding(),
    );
    assert_eq!(lines.len(), 1, "one line says what the pass removed");
    assert!(
        !stands(the_plan_path(&fixture, &plan).as_path()),
        "the plan was the last witness and went"
    );
    assert!(
        !stands(home.join(clone_inventory_name(SLOT)).as_path()),
        "the inventory a kill left was removed as the same step's leftover"
    );
    assert!(
        !stands(home.join(origin_inventory_name(SLOT)).as_path()),
        "and the other inventory with it"
    );
}

#[test]
fn a_settle_that_refused_leaves_every_record_standing() {
    let (fixture, plan) = a_finished_removal("finish-pass-settle-refused");
    let settle: ASettle = ASettle::refusing("the lock is held by another process");
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "one line names the step and the settle's words"
    );
    assert!(
        lines[0].contains("the lock is held by another process"),
        "the line carries the settle's words: {}",
        lines[0]
    );
    everything_stands(&fixture, &plan, "a settle that refused");
}

#[test]
fn a_plan_whose_target_names_no_slot_is_left_and_named() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-no-slot");
    let mut plan = a_plan_for(&fixture, HeadWord::Gone);
    plan.facts.target = PathBuf::from("/");
    let path: PathBuf = write_the_plan(&fixture, &plan);
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "one line names the plan and the slot it cannot key"
    );
    assert_eq!(settle.calls.get(), 0);
    assert!(stands(path.as_path()), "the plan stands");
}

#[test]
fn the_claim_is_taken_under_the_plans_target_and_never_its_dataset_name() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-claim-key");
    let mut plan = a_plan_for(&fixture, HeadWord::Gone);
    plan.facts.clone_dataset = String::from("tank/wt/another-leaf");
    for attempt in &mut plan.attempts {
        attempt.snapshot.name = format!(
            "tank/wt/another-leaf@{}",
            attempt
                .snapshot
                .name
                .split('@')
                .next_back()
                .unwrap_or("own")
        );
    }
    let path: PathBuf = write_the_plan(&fixture, &plan);
    let home: PathBuf = the_home(&fixture);
    let birth: String = current_birth_identity().expect("this process's own birth identity reads");
    plant_claim_for_tests(
        home.as_path(),
        claim_prefix("another-leaf").as_os_str(),
        1,
        i64::from(std::process::id()),
        birth.as_str(),
    );
    let settle: ASettle = ASettle::succeeding();
    let lines: Vec<String> = the_pass(
        &fixture,
        &TheLivePlansFolder,
        &AGate::passing(),
        &AProbe::absent(),
        &settle,
    );
    assert_eq!(
        lines.len(),
        1,
        "the pass acts, because the slot's own claim is free, and this run printed {lines:?}"
    );
    assert!(!stands(path.as_path()), "the plan went");
    assert!(
        stands(home.join(format!("{SLOT}.claim.1")).as_path()),
        "the claim was taken under the plan's target slot"
    );
    assert_eq!(settle.calls.get(), 1);
}
