//! What:
//!  The controls of what the add recorded:
//!  a healthy binding read by the slot's name,
//!  a phase that never reached the clone,
//!  a field the removal needs and the record lacks,
//!  an origin-side inventory that disagrees with the binding,
//!  and a registration whose identity is another worktree's.
//! Why:
//!  A removal copies the binding's identities and never guesses one,
//!  and a registration of another identity is another removal's
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  section 6).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readTheRecordedBinding(common, 'fix-flaky-test').cloneGuid).toBe(7828944690444299989n);
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_SNAPSHOT, RemovalFixture, SLOT};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{RecordedBinding, prove_the_adds_registration, read_the_origin_inventory, read_the_recorded_binding};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

#[test]
fn a_filled_binding_is_read_by_the_slots_name() {
    let fixture: RemovalFixture = RemovalFixture::healthy("binding-read");
    let recorded: RecordedBinding = read_the_recorded_binding(fixture.common(), SLOT).expect("the fixture's binding reads");
    assert_eq!(recorded.slot, SLOT);
    assert_eq!(recorded.clone_dataset, CLONE);
    assert_eq!(recorded.clone_guid.value(), CLONE_GUID);
    assert_eq!(recorded.clone_createtxg, CLONE_CREATETXG);
    assert_eq!(recorded.origin_snapshot, ORIGIN_SNAPSHOT);
    assert_eq!(recorded.slot_path, fixture.slot_path);
    assert_eq!(recorded.registration_id, Vec::from("fix-flaky-test".as_bytes()));
    assert_eq!(recorded.origin_fields().len(), 5);
}

#[test]
fn a_slot_the_add_never_wrote_refuses_as_absent() {
    let fixture: RemovalFixture = RemovalFixture::healthy("binding-absent");
    match read_the_recorded_binding(fixture.common(), "no-such-slot").expect_err("a slot the add never wrote refuses") {
        RemovalStop::NoBinding { slot, .. } => assert_eq!(slot, "no-such-slot"),
        other => panic!("an absent binding refuses as absent, not {other:?}"),
    }
}

#[test]
fn a_binding_whose_clone_fields_are_missing_refuses_on_the_field_it_names() {
    for (line, field) in [("cloneGuid", "cloneGuid"), ("registrationIdentity", "registrationIdentity"), ("poolGuid", "poolGuid"), ("slot", "slot")] {
        let mut fixture: RemovalFixture = RemovalFixture::healthy(&format!("binding-without-{field}"));
        fixture.drop_binding_line(line);
        fixture.write_the_binding();
        let refusal: RemovalStop = match read_the_recorded_binding(fixture.common(), SLOT) {
            Ok(_) => panic!("a record without `{line}` must not read as one the removal can use"),
            Err(refusal) => refusal,
        };
        match refusal {
            RemovalStop::NoBinding { .. } => {}
            RemovalStop::BindingIncomplete { field: named, .. } => assert_eq!(named, field),
            other => panic!("a missing `{line}` refuses on its own field or as an unreadable record, not {other:?}"),
        }
    }
}

#[test]
fn the_origin_side_inventory_is_read_and_compared_with_the_binding() {
    let fixture: RemovalFixture = RemovalFixture::healthy("origin-inventory");
    let recorded: RecordedBinding = fixture.recorded();
    let inventory = read_the_origin_inventory(fixture.common(), &recorded).expect("the fixture's inventory reads");
    assert_eq!(inventory.bounds, ORIGIN_SNAPSHOT);
    let disagreeing: RemovalFixture = RemovalFixture::healthy("origin-inventory-differs");
    disagreeing.write_the_binding();
    let mut record = disagreeing.the_origin_inventory();
    record.bounds_guid = crate::worktree_clone_removal_proof_and_plan::removal_fixtures::guid(4_242);
    std::fs::write(
        crate::worktree_clone_binding_record::registration_home::home_folder(disagreeing.common()).join(crate::worktree_clone_binding_record::registration_home::origin_inventory_name(SLOT)),
        record.encode(),
    )
    .expect("the disagreeing inventory");
    match read_the_origin_inventory(disagreeing.common(), &disagreeing.recorded()).expect_err("a disagreeing inventory refuses") {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "originSnapshotGuid"),
        other => panic!("a disagreeing inventory refuses on its field, not {other:?}"),
    }
}

#[test]
fn the_registration_standing_there_is_proved_by_its_identity() {
    let fixture: RemovalFixture = RemovalFixture::healthy("registration");
    let recorded: RecordedBinding = fixture.recorded();
    let identity = prove_the_adds_registration(fixture.common(), &recorded).expect("the fixture's registration is the recorded one");
    assert_eq!(identity.inode, recorded.registration_identity.inode());
    std::fs::remove_dir_all(fixture.registration.as_path()).expect("the registration goes");
    match prove_the_adds_registration(fixture.common(), &recorded).expect_err("an absent registration refuses") {
        RemovalStop::DoesNotAnswer { object, .. } => assert!(object.contains("worktrees"), "the refusal names the registration: {object}"),
        other => panic!("an absent registration refuses as unreadable, not {other:?}"),
    }
}
