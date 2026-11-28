//! What:
//!  Controls of the checkout's boot protection:
//!  the protected entry passes,
//!  each unprotected reason answers its own field with the verdict's repair,
//!  and a read nobody could make or a filesystem without the attribute stays
//!  unknown.
//! Why:
//!  the three answers decide whether an automated attach may act at all,
//!  so a reason that read as a pass would attach a clone onto a mount another
//!  process can rename,
//!  and each reason's field is what sends a person to one call
//!  (`AGENTS.md` rules PKG and TCV).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = { entry: { ...covered, immutable: false }, wantedOwner: 1000 };
//! expect(checkBootProtection(facts)).toMatchObject({ kind: 'finding', field: 'immutable' });
//! ```

use super::{
    BootProtectionFacts,
    check_boot_protection,
};
use crate::checkout_underlay_entry::underlay_entry_read::{
    CoveredEntryFacts,
    EntryReadRefusal,
};
use crate::health::doctor_zfs_route::check_outcome::CheckOutcome;
use std::path::PathBuf;

/// The account the route requires the covered entry to belong to.
const WANTED_OWNER: u32 = 1000;

/// The entry an unmounted,
/// immutable,
/// account-owned checkout has.
fn covered() -> CoveredEntryFacts {
    return CoveredEntryFacts {
        device: 42,
        inode: 1001,
        owner: WANTED_OWNER,
        mode: 0o040755,
        directory: true,
        immutable: true,
        attributes_reported: true,
        same_device_as_parent: true,
    };
}

/// The facts around one reading of the covered entry.
fn facts(read: CoveredEntryFacts) -> BootProtectionFacts {
    return BootProtectionFacts {
        entry: Ok(read),
        wanted_owner: WANTED_OWNER,
    };
}

/// The answer as the line a report prints.
fn line(outcome: &CheckOutcome) -> String {
    let report = crate::health::doctor_zfs_route::check_outcome::CheckReport::new(
        crate::health::doctor_zfs_route::check_outcome::Check::BootProtection,
        outcome.clone(),
    );
    return report.line();
}

/// The protected entry passes and says what was read.
#[test]
fn a_protected_entry_passes() {
    let outcome: CheckOutcome = check_boot_protection(&facts(covered()));
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Pass { .. }), "{text}");
    assert!(text.contains("is protected"), "{text}");
    assert!(text.contains("immutable"), "{text}");
}

/// A path that is not a directory is a finding about the entry,
/// with the repair the verdict states.
#[test]
fn a_path_that_is_not_a_directory_is_a_finding_about_the_entry() {
    let mut read: CoveredEntryFacts = covered();
    read.directory = false;
    let outcome: CheckOutcome = check_boot_protection(&facts(read));
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "entry"),
        "{text}"
    );
    assert!(text.contains("is not a directory"), "{text}");
}

/// A mount standing at the path is a finding about the mount,
/// so a person unmounts rather than setting a flag on a dataset.
#[test]
fn a_mount_at_the_path_is_a_finding_about_the_mount() {
    let mut read: CoveredEntryFacts = covered();
    read.same_device_as_parent = false;
    let outcome: CheckOutcome = check_boot_protection(&facts(read));
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "mount"),
        "{text}"
    );
    assert!(text.contains("is a mount of its own"), "{text}");
}

/// A clear immutable flag is a finding about the flag,
/// with the `chattr +i` the verdict names.
#[test]
fn a_clear_immutable_flag_is_a_finding_about_the_flag() {
    let mut read: CoveredEntryFacts = covered();
    read.immutable = false;
    let outcome: CheckOutcome = check_boot_protection(&facts(read));
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "immutable"),
        "{text}"
    );
    assert!(
        text.contains("does not carry the immutable attribute"),
        "{text}"
    );
    assert!(text.contains("chattr +i"), "{text}");
}

/// Another account's entry is a finding about the owner,
/// naming both numbers.
#[test]
fn another_accounts_entry_is_a_finding_about_the_owner() {
    let mut read: CoveredEntryFacts = covered();
    read.owner = WANTED_OWNER + 1;
    let outcome: CheckOutcome = check_boot_protection(&facts(read));
    let text: String = line(&outcome);
    assert!(
        matches!(&outcome, CheckOutcome::Finding { field, .. } if field == "owner"),
        "{text}"
    );
    assert!(text.contains("owned by user 1001"), "{text}");
    assert!(text.contains("requires user 1000"), "{text}");
}

/// A filesystem that reports no immutable attribute proves neither answer,
/// so the check stays unknown and says what to inspect.
#[test]
fn a_filesystem_without_the_attribute_is_unknown() {
    let mut read: CoveredEntryFacts = covered();
    read.attributes_reported = false;
    let outcome: CheckOutcome = check_boot_protection(&facts(read));
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Unknown { .. }), "{text}");
    assert!(text.contains("reports no immutable attribute"), "{text}");
}

/// A read the kernel refused is unknown with the call and the reason,
/// because a made-up shape would turn "nobody looked" into a verdict.
#[test]
fn a_read_the_kernel_refused_is_unknown() {
    let refused: BootProtectionFacts = BootProtectionFacts {
        entry: Err(EntryReadRefusal::Call {
            what: "the `statx` read",
            path: PathBuf::from("/home/user/Monochromatic"),
            reason: String::from("Permission denied"),
        }),
        wanted_owner: WANTED_OWNER,
    };
    let outcome: CheckOutcome = check_boot_protection(&refused);
    let text: String = line(&outcome);
    assert!(matches!(outcome, CheckOutcome::Unknown { .. }), "{text}");
    assert!(text.contains("/home/user/Monochromatic"), "{text}");
    assert!(text.contains("Permission denied"), "{text}");
}
