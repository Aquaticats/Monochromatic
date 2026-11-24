//! What:
//!  Controls of the handler's installation,
//!  configuration,
//!  folders,
//!  ZED wiring,
//!  readiness and unresolved requests.
//! Why:
//!  Each of these is a fact about a root-owned path or a record a caller
//!  waits for,
//!  and a pass that survived a wrong owner or a record from another boot would
//!  let a root program act on an account's choice,
//!  so each control changes exactly one of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_handler_folders({ readiness: ownedBy(1000, 0o755), results })).toMatchObject({ kind: 'finding', field: 'owner' });
//! ```

use super::{
    FolderFacts, HandlerConfigurationFacts, HandlerInstallationFacts, HandlerWiringFacts, PLACEHOLDER_EXECUTABLE_PATH, PLACEHOLDER_ZEDLET_PATH, PathFacts, PathState, REQUIRED_ROOT_MODES, RESULTS_DIRECTORY_MODE, ReadinessFacts, RequestFacts,
    READINESS_DIRECTORY_MODE, ROOT_OWNER, check_handler_configuration, check_handler_folders, check_handler_installation, check_modes, check_readiness, check_root_modes, check_unresolved_requests, check_zed_wiring,
};
use crate::health::doctor_zfs_route::route_test_facts::{healthy_configuration, healthy_folders, healthy_installation, healthy_readiness, healthy_request, healthy_wiring, path_facts, root_file, LIVE_BOOT};
use crate::worktree_clone_mount_protocol::ReadyState;

/// A file owned by another account.
fn owned_by(path: &str, owner: u32, mode: u32) -> PathFacts {
    return path_facts(path, PathState::File { owner, mode });
}

//region The installation

/// The two installed halves at root's ownership pass.
#[test]
fn the_installed_halves_at_root_ownership_pass() {
    assert!(matches!(check_handler_installation(&healthy_installation()), super::CheckOutcome::Pass { .. }));
}

/// An executable owned by another account is a finding naming the owner.
#[test]
fn an_executable_owned_by_another_account_is_a_finding() {
    let facts: HandlerInstallationFacts = HandlerInstallationFacts { executable: owned_by(PLACEHOLDER_EXECUTABLE_PATH, 1000, 0o755), zedlet: root_file(PLACEHOLDER_ZEDLET_PATH) };
    let super::CheckOutcome::Finding { field, detail } = check_handler_installation(&facts) else {
        panic!("another owner is a finding");
    };
    assert_eq!(field, "owner");
    assert!(detail.contains("1000"), "{detail}");
}

/// An installed file without an executable bit is a finding naming the mode.
#[test]
fn a_file_without_an_executable_bit_is_a_finding() {
    let facts: HandlerInstallationFacts = HandlerInstallationFacts { executable: owned_by(PLACEHOLDER_EXECUTABLE_PATH, ROOT_OWNER, 0o644), zedlet: root_file(PLACEHOLDER_ZEDLET_PATH) };
    let super::CheckOutcome::Finding { field, .. } = check_handler_installation(&facts) else {
        panic!("a file no one may run is a finding");
    };
    assert_eq!(field, "mode");
}

/// A missing half is a finding naming the installation.
#[test]
fn a_missing_half_is_a_finding_naming_the_installation() {
    let facts: HandlerInstallationFacts = HandlerInstallationFacts { executable: root_file(PLACEHOLDER_EXECUTABLE_PATH), zedlet: path_facts(PLACEHOLDER_ZEDLET_PATH, PathState::Absent) };
    let super::CheckOutcome::Finding { field, detail } = check_handler_installation(&facts) else {
        panic!("a missing zedlet is a finding");
    };
    assert_eq!(field, "installation");
    assert!(detail.contains(PLACEHOLDER_ZEDLET_PATH), "{detail}");
}

/// The one executable answers every root mode the route needs.
#[test]
fn the_one_executable_answers_every_root_mode_the_route_needs() {
    let outcome: super::CheckOutcome = check_root_modes();
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("this build's grammar holds every required mode: {outcome:?}");
    };
    assert!(detail.contains('5'), "{detail}");
}

/// A mode word this build does not answer is a finding naming the mode.
#[test]
fn a_mode_the_build_does_not_answer_is_a_finding() {
    let outcome: super::CheckOutcome = check_modes(&["not-a-mode"]);
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a word the grammar refuses is a finding: {outcome:?}");
    };
    assert_eq!(field, "mode");
    assert!(detail.contains("not-a-mode"), "{detail}");
}

/// The required modes are the five the owner's answer names,
/// and the sweep is the empty word.
#[test]
fn the_required_modes_are_the_four_the_tree_holds() {
    assert_eq!(REQUIRED_ROOT_MODES, ["", "adopt-worktree-parent", "boot-reconcile", "shutdown-unmount", "start-probe"]);
}

/// A look this account was refused leaves the installation unknown.
#[test]
fn an_unreadable_installation_is_unknown() {
    let facts: HandlerInstallationFacts = HandlerInstallationFacts { executable: path_facts(PLACEHOLDER_EXECUTABLE_PATH, PathState::Unreadable { why: String::from("permission denied") }), zedlet: root_file(PLACEHOLDER_ZEDLET_PATH) };
    assert!(matches!(check_handler_installation(&facts), super::CheckOutcome::Unknown { .. }));
}

//endregion

//region The configuration

/// A root-owned configuration that parses passes.
#[test]
fn a_root_owned_configuration_that_parses_passes() {
    assert!(matches!(check_handler_configuration(&healthy_configuration()), super::CheckOutcome::Pass { .. }));
}

/// An absent configuration passes,
/// because the design's fallback finds the datasets without one.
#[test]
fn an_absent_configuration_falls_back() {
    let facts: HandlerConfigurationFacts = HandlerConfigurationFacts { file: path_facts("/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc", PathState::Absent), parse_error: None };
    assert!(matches!(check_handler_configuration(&facts), super::CheckOutcome::Pass { .. }));
}

/// A configuration another account owns is a finding naming the owner.
#[test]
fn a_configuration_another_account_owns_is_a_finding() {
    let facts: HandlerConfigurationFacts = HandlerConfigurationFacts { file: owned_by("/etc/x.jsonc", 1000, 0o644), parse_error: None };
    let super::CheckOutcome::Finding { field, .. } = check_handler_configuration(&facts) else {
        panic!("another owner is a finding");
    };
    assert_eq!(field, "owner");
}

/// A configuration root's group could write is a finding naming the mode.
#[test]
fn a_configuration_its_group_could_write_is_a_finding() {
    let facts: HandlerConfigurationFacts = HandlerConfigurationFacts { file: owned_by("/etc/x.jsonc", ROOT_OWNER, 0o664), parse_error: None };
    let super::CheckOutcome::Finding { field, .. } = check_handler_configuration(&facts) else {
        panic!("a writable configuration is a finding");
    };
    assert_eq!(field, "mode");
}

/// A configuration that does not parse is a finding naming the parse.
#[test]
fn a_configuration_that_does_not_parse_is_a_finding() {
    let facts: HandlerConfigurationFacts = HandlerConfigurationFacts { file: owned_by("/etc/x.jsonc", ROOT_OWNER, 0o644), parse_error: Some(String::from("the object has a duplicate key")) };
    let super::CheckOutcome::Finding { field, .. } = check_handler_configuration(&facts) else {
        panic!("a bad parse is a finding");
    };
    assert_eq!(field, "parse");
}

//endregion

//region The folders

/// Both folders at the design's modes pass.
#[test]
fn both_folders_at_the_designs_modes_pass() {
    assert!(matches!(check_handler_folders(&healthy_folders()), super::CheckOutcome::Pass { .. }));
}

/// A readiness folder at another mode is a finding naming the mode.
#[test]
fn a_folder_at_another_mode_is_a_finding() {
    let mut facts: FolderFacts = healthy_folders();
    facts.readiness = path_facts("/run/git-worktree-clone-mount-readiness", PathState::Directory { owner: ROOT_OWNER, mode: 0o700 });
    let super::CheckOutcome::Finding { field, detail } = check_handler_folders(&facts) else {
        panic!("a closed folder is a finding");
    };
    assert_eq!(field, "mode");
    assert!(detail.contains("0755"), "{detail}");
}

/// A results folder owned by the account is a finding naming the owner.
#[test]
fn a_folder_the_account_owns_is_a_finding() {
    let mut facts: FolderFacts = healthy_folders();
    facts.results = path_facts("/run/git-worktree-clone-mount-results", PathState::Directory { owner: 1000, mode: RESULTS_DIRECTORY_MODE });
    let super::CheckOutcome::Finding { field, .. } = check_handler_folders(&facts) else {
        panic!("another owner is a finding");
    };
    assert_eq!(field, "owner");
}

/// A missing folder is a finding naming the folder.
#[test]
fn a_missing_folder_is_a_finding() {
    let mut facts: FolderFacts = healthy_folders();
    facts.readiness = path_facts("/run/git-worktree-clone-mount-readiness", PathState::Absent);
    let super::CheckOutcome::Finding { field, .. } = check_handler_folders(&facts) else {
        panic!("a missing folder is a finding");
    };
    assert_eq!(field, "folder");
}

/// The readiness folder's decided mode is 0755.
#[test]
fn the_readiness_folder_mode_is_0755() {
    assert_eq!(READINESS_DIRECTORY_MODE, 0o755);
}

//endregion

//region ZED

/// ZED installed and enabled passes and lists the other zedlets.
#[test]
fn zed_installed_and_enabled_passes_with_its_other_zedlets() {
    let outcome: super::CheckOutcome = check_zed_wiring(&healthy_wiring());
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("ZED installed and enabled passes");
    };
    assert!(detail.contains("history_event-zfs-list-cacher.sh"), "{detail}");
    assert!(detail.contains("not asked of the service manager"), "{detail}");
}

/// An installed ZED nobody enabled is a finding naming the enablement.
#[test]
fn zed_installed_without_enablement_is_a_finding() {
    let mut facts: HandlerWiringFacts = healthy_wiring();
    facts.zed_enabled = false;
    let super::CheckOutcome::Finding { field, detail } = check_zed_wiring(&facts) else {
        panic!("an unenabled ZED is a finding");
    };
    assert_eq!(field, "enablement");
    assert!(detail.contains("history_event-zfs-list-cacher.sh"), "{detail}");
}

/// A missing ZED unit is a finding naming the unit.
#[test]
fn a_missing_zed_unit_is_a_finding() {
    let mut facts: HandlerWiringFacts = healthy_wiring();
    facts.zed_unit = path_facts("/usr/lib/systemd/system/zfs-zed.service", PathState::Absent);
    let super::CheckOutcome::Finding { field, .. } = check_zed_wiring(&facts) else {
        panic!("a missing unit is a finding");
    };
    assert_eq!(field, "unit");
}

//endregion

//region The readiness record

/// This boot's ready record passes.
#[test]
fn this_boots_ready_record_passes() {
    assert!(matches!(check_readiness(&healthy_readiness(), LIVE_BOOT), super::CheckOutcome::Pass { .. }));
}

/// A record from another boot is a finding naming the boot.
#[test]
fn a_record_from_another_boot_is_a_finding() {
    let outcome: super::CheckOutcome = check_readiness(&healthy_readiness(), "another-boot");
    let super::CheckOutcome::Finding { field, .. } = outcome else {
        panic!("another boot's record is a finding");
    };
    assert_eq!(field, "boot");
}

/// A pool that is not ready is a finding naming the pool.
#[test]
fn a_pool_that_is_not_ready_is_a_finding() {
    let facts: ReadinessFacts = ReadinessFacts::Read { boot: String::from(LIVE_BOOT), pool: ReadyState::NotReady, checkout: ReadyState::Ready, sweep: String::from("never") };
    let super::CheckOutcome::Finding { field, .. } = check_readiness(&facts, LIVE_BOOT) else {
        panic!("an unready pool is a finding");
    };
    assert_eq!(field, "pool");
}

/// A checkout the handler could not read leaves the answer unknown.
#[test]
fn a_checkout_the_handler_could_not_read_is_unknown() {
    let facts: ReadinessFacts = ReadinessFacts::Read { boot: String::from(LIVE_BOOT), pool: ReadyState::Ready, checkout: ReadyState::Unknown, sweep: String::from("never") };
    assert!(matches!(check_readiness(&facts, LIVE_BOOT), super::CheckOutcome::Unknown { .. }));
}

/// No record at all is a finding: a caller waits for nothing.
#[test]
fn no_record_is_a_finding() {
    let facts: ReadinessFacts = ReadinessFacts::Absent { why: String::from("/run/git-worktree-clone-mount-readiness/result is not there") };
    let super::CheckOutcome::Finding { field, .. } = check_readiness(&facts, LIVE_BOOT) else {
        panic!("no record is a finding");
    };
    assert_eq!(field, "readiness");
}

//endregion

//region The unresolved requests

/// A request answered by this boot's receipt passes.
#[test]
fn an_answered_request_passes() {
    assert!(matches!(check_unresolved_requests(&[healthy_request()], LIVE_BOOT), super::CheckOutcome::Pass { .. }));
}

/// A request with no receipt is a finding naming the clone.
#[test]
fn a_request_with_no_receipt_is_a_finding() {
    let facts: RequestFacts = RequestFacts { clone: String::from("pool/one"), intent_nonce: Some(String::from("0123456789abcdef0123456789abcdef")), receipt_nonce: None, receipt_boot: None };
    let super::CheckOutcome::Finding { field, detail } = check_unresolved_requests(&[facts], LIVE_BOOT) else {
        panic!("a request with no receipt is a finding");
    };
    assert_eq!(field, "request");
    assert!(detail.contains("pool/one"), "{detail}");
}

/// A receipt that answers another nonce does not answer this request.
#[test]
fn a_receipt_for_another_nonce_does_not_answer() {
    let facts: RequestFacts = RequestFacts { clone: String::from("pool/one"), intent_nonce: Some(String::from("0123456789abcdef0123456789abcdef")), receipt_nonce: Some(String::from("fedcba9876543210fedcba9876543210")), receipt_boot: Some(String::from(LIVE_BOOT)) };
    assert!(matches!(check_unresolved_requests(&[facts], LIVE_BOOT), super::CheckOutcome::Finding { .. }));
}

/// A receipt from another boot does not answer this boot's request.
#[test]
fn a_receipt_from_another_boot_does_not_answer() {
    let mut facts: RequestFacts = healthy_request();
    facts.receipt_boot = Some(String::from("another-boot"));
    assert!(matches!(check_unresolved_requests(&[facts], LIVE_BOOT), super::CheckOutcome::Finding { .. }));
}

/// A clone asking for no work is not a request.
#[test]
fn a_clone_asking_for_nothing_is_not_a_request() {
    let facts: RequestFacts = RequestFacts { clone: String::from("pool/one"), intent_nonce: None, receipt_nonce: None, receipt_boot: None };
    let outcome: super::CheckOutcome = check_unresolved_requests(&[facts], LIVE_BOOT);
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("no request passes");
    };
    assert!(detail.contains('0'), "{detail}");
}

//endregion
