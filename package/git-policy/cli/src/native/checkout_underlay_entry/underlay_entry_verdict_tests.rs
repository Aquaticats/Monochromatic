//! What:
//!  The verdict's controls:
//!  one protected shape,
//!  one test per reason,
//!  the order that keeps each message true,
//!  and the line a journal keeps.
//! Why:
//!  each reason is a different repair,
//!  so a reading that answered "not protected" without the reason would leave a
//!  person without the command to run.
//!  The order is asserted on its own, because a mounted checkout reported as a
//!  missing flag would send a person to `chattr` on a dataset they cannot protect
//!  that way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verify({ ...facts, immutable: false }, 1000)).toEqual({ kind: 'unprotected', reason: 'immutable-clear' });
//! ```

use crate::checkout_underlay_entry::underlay_entry_read::EntryFacts;
use crate::checkout_underlay_entry::underlay_entry_verdict::{UnderlayVerdict, UnprotectedReason, UnprovenReason, verify};

/// The account the route requires the covered entry to belong to.
const WANTED_OWNER: u32 = 1_000;

/// The reading of an unmounted, immutable, account-owned covered entry.
fn protected() -> EntryFacts {
    return EntryFacts {
        device: 2_097_152,
        inode: 42,
        owner: WANTED_OWNER,
        mode: 0o040_755,
        directory: true,
        immutable: true,
        attributes_reported: true,
        same_device_as_parent: true,
    };
}

#[test]
fn an_unmounted_immutable_entry_of_the_account_is_protected() {
    assert_eq!(verify(&protected(), WANTED_OWNER), UnderlayVerdict::Protected);
    assert!(verify(&protected(), WANTED_OWNER).is_protected());
}

#[test]
fn a_file_or_a_link_at_the_path_is_no_covered_entry() {
    let facts: EntryFacts = EntryFacts { directory: false, ..protected() };
    assert_eq!(verify(&facts, WANTED_OWNER), UnderlayVerdict::Unprotected(UnprotectedReason::NotADirectory));
    assert!(!verify(&facts, WANTED_OWNER).is_protected());
}

#[test]
fn a_mount_at_the_path_is_named_before_the_flag_is_read() {
    let mounted: EntryFacts = EntryFacts { same_device_as_parent: false, immutable: false, ..protected() };
    assert_eq!(
        verify(&mounted, WANTED_OWNER),
        UnderlayVerdict::Unprotected(UnprotectedReason::MountedOver),
        "the flag read off a mounted dataset says nothing about the underlay"
    );
    assert!(
        UnderlayVerdict::Unprotected(UnprotectedReason::MountedOver).message(&protected()).contains("unmounted"),
        "the repair names the state the check needs"
    );
}

#[test]
fn a_filesystem_that_reports_no_attribute_is_unproven_and_not_unprotected() {
    let facts: EntryFacts = EntryFacts { attributes_reported: false, immutable: false, ..protected() };
    assert_eq!(verify(&facts, WANTED_OWNER), UnderlayVerdict::Unproven(UnprovenReason::AttributesNotReported));
    assert!(!verify(&facts, WANTED_OWNER).is_protected());
    assert!(
        UnderlayVerdict::Unproven(UnprovenReason::AttributesNotReported).message(&protected()).contains("lsattr"),
        "a person is given the read that answers it elsewhere"
    );
}

#[test]
fn a_clear_flag_is_unprotected_and_names_the_chattr_command() {
    let facts: EntryFacts = EntryFacts { immutable: false, ..protected() };
    let verdict: UnderlayVerdict = verify(&facts, WANTED_OWNER);
    assert_eq!(verdict, UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear));
    assert!(verdict.message(&facts).contains("chattr +i"), "the repair is the setup's own step: {}", verdict.message(&facts));
    assert!(verdict.message(&facts).contains("rollback"), "and a rollback is named as the way it is lost: {}", verdict.message(&facts));
}

#[test]
fn another_account_s_owner_is_reported_after_the_flag() {
    let facts: EntryFacts = EntryFacts { owner: 0, ..protected() };
    let verdict: UnderlayVerdict = verify(&facts, WANTED_OWNER);
    assert_eq!(verdict, UnderlayVerdict::Unprotected(UnprotectedReason::Owner { found: 0, wanted: WANTED_OWNER }));
    assert!(verdict.message(&facts).contains("chown 1000"), "the repair names the owner to restore: {}", verdict.message(&facts));
}

#[test]
fn every_verdict_states_the_numbers_it_compared() {
    let facts: EntryFacts = protected();
    let line: String = UnderlayVerdict::Protected.line(&facts);
    assert!(line.starts_with("UNDERLAY entry=protected"), "the line names its subject: {line}");
    for number in ["device=2097152", "inode=42", "owner=1000", "mode=040755", "immutable=true", "attributes-reported=true", "same-device-as-parent=true"] {
        assert!(line.contains(number), "the line carries `{number}`: {line}");
    }
    let unprotected: String = UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear).line(&facts);
    assert!(unprotected.contains("entry=unprotected"), "the answer is one word a reader greps: {unprotected}");
    let unproven: String = UnderlayVerdict::Unproven(UnprovenReason::AttributesNotReported).line(&facts);
    assert!(unproven.contains("entry=unproven"), "unknown is not a pass or a failure: {unproven}");
}

#[test]
fn the_sentences_say_nothing_was_attached_or_changed() {
    let facts: EntryFacts = protected();
    for verdict in [
        UnderlayVerdict::Unprotected(UnprotectedReason::NotADirectory),
        UnderlayVerdict::Unprotected(UnprotectedReason::MountedOver),
        UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear),
        UnderlayVerdict::Unprotected(UnprotectedReason::Owner { found: 0, wanted: WANTED_OWNER }),
        UnderlayVerdict::Unproven(UnprovenReason::AttributesNotReported),
    ] {
        let message: String = verdict.message(&facts);
        assert!(message.contains("Nothing was attached or changed"), "each answer says nothing happened: {message}");
        assert!(message.contains("Ways forward:"), "and each names a way forward: {message}");
    }
}
