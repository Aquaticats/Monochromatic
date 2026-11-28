//! What:
//!  Controls of this boot's readiness:
//!  the withdrawal happens before anything is judged,
//!  a boot that published nothing proves nothing,
//!  a record of another boot proves nothing,
//!  a pool or a checkout that was not ready proves nothing,
//!  and a record whose pool or checkout disagrees with the configuration is a
//!  refusal.
//! Why:
//!  The pass unmounts only what this boot's own record proves,
//!  so each arm here is a machine whose mounts stay standing and a line a
//!  person reads
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  The record is a file of the test's fixture and the configuration a value,
//!  so no control reads `/run` or `/etc`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const readiness = withdrawBeforeDraining(places);
//! whatTheRecordVerified(readiness, configuration); // 'everything' or 'nothing, and why'
//! ```

use super::{
    BootsReadiness,
    Verified,
    what_the_record_verified,
    withdraw_before_draining,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount::slot_identity_store::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file_withdraw::RecordStood;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_places::ShutdownUnmountPlaces;
use std::path::{
    Path,
    PathBuf,
};

/// The boot every control runs in.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The pool and checkout every control's configuration names.
const CONTAINER: &str = "zpcachyos/nosnap/git-worktree-clones";

/// The checkout the configuration names.
const CHECKOUT: &str = "/home/user/Mono";

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// One control's places,
/// all of them under the test's fixture root,
/// with this boot's identifier written.
fn places(name: &str) -> ShutdownUnmountPlaces {
    let root: PathBuf = fixture(name);
    std::fs::write(root.join("boot_id"), format!("{BOOT}\n")).expect("the boot is written");
    return ShutdownUnmountPlaces {
        readiness: ReadinessFile::at(root.join("readiness"), this_uid()),
        configuration: root.join("configuration.jsonc"),
        claims: SlotClaimDirectory::at(root.join("claims"), this_uid()),
        store: PathBuf::from(SLOT_IDENTITY_STORE_PATH),
        boot_identity: root.join("boot_id"),
        proc_root: root.join("proc"),
        self_task: root.join("proc").join("self"),
    };
}

/// A ready record for this boot.
fn record() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: String::from(CHECKOUT),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("2".repeat(32).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-27T20:00:00-05:00"),
    };
}

/// The configuration the boot unit read.
fn configuration() -> SweepConfiguration {
    return SweepConfiguration {
        parent_dataset: String::from(CONTAINER),
        checkout: PathBuf::from(CHECKOUT),
        checkout_dataset: String::from("zpcachyos/nosnap/git-worktree-checkout"),
    };
}

/// This boot's record is withdrawn before the judgement,
/// which finds the pool and the checkout ready.
#[test]
fn a_ready_record_of_this_boot_proves_everything_and_is_removed() {
    let places: ShutdownUnmountPlaces = places("readiness-ready");
    places
        .readiness
        .publish(&record())
        .expect("the record is published");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("the record of this boot is withdrawn");
    assert_eq!(readiness.boot, BOOT);
    assert_eq!(
        readiness.withdrawal.stood.clone(),
        RecordStood::Record(Box::new(record()))
    );
    assert_eq!(
        places.readiness.read().expect("the directory reads"),
        None,
        "the withdrawal came before the judgement, so admission is closed by the time it happens"
    );
    assert_eq!(
        what_the_record_verified(&readiness, &configuration()).expect("the names agree"),
        Verified::Everything
    );
}

/// A boot that published no record proves nothing.
#[test]
fn a_boot_that_published_nothing_proves_nothing() {
    let places: ShutdownUnmountPlaces = places("readiness-none");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("a boot that published nothing is withdrawn");
    match what_the_record_verified(&readiness, &configuration()).expect("an absence is no refusal")
    {
        Verified::Nothing {
            said,
        } => assert!(
            said.contains("published no readiness record"),
            "the sentence says what was missing, and this one says {said}"
        ),
        other => panic!("a boot that published nothing proves nothing, and this one {other:?}"),
    }
}

/// Another boot's record proves nothing about this one.
#[test]
fn a_record_of_another_boot_proves_nothing() {
    let places: ShutdownUnmountPlaces = places("readiness-another-boot");
    let mut another_boots_record: ReadinessRecord = record();
    another_boots_record.boot = String::from("11111111-2222-4333-8444-555555555555");
    places
        .readiness
        .publish(&another_boots_record)
        .expect("the record is published");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("another boot's record is withdrawn too");
    match what_the_record_verified(&readiness, &configuration()).expect("an absence is no refusal")
    {
        Verified::Nothing {
            said,
        } => assert!(
            said.contains(
                "belonged to boot 11111111-2222-4333-8444-555555555555, and this is boot"
            ),
            "the sentence names both boots, and this one says {said}"
        ),
        proved => panic!("another boot's record proves nothing, and this one {proved:?}"),
    }
}

/// A pool or a checkout that was not ready proves nothing.
#[test]
fn a_pool_or_a_checkout_that_was_not_ready_proves_nothing() {
    let cases: Vec<(ReadinessRecord, &str)> = vec![
        (
            ReadinessRecord {
                pool_state: ReadyState::NotReady,
                ..record()
            },
            "says the pool was `not-ready` and not ready",
        ),
        (
            ReadinessRecord {
                checkout_state: ReadyState::Unknown,
                ..record()
            },
            "says the checkout was `unknown` and not ready",
        ),
    ];
    for (index, (published, wanted)) in cases.iter().enumerate() {
        let places: ShutdownUnmountPlaces = places(format!("readiness-not-ready-{index}").as_str());
        places
            .readiness
            .publish(published)
            .expect("the record is published");
        let readiness: BootsReadiness =
            withdraw_before_draining(&places).expect("the record is withdrawn");
        match what_the_record_verified(&readiness, &configuration())
            .expect("an absence is no refusal")
        {
            Verified::Nothing {
                said,
            } => assert!(
                said.contains(wanted),
                "the sentence should say {wanted:?}, and this one says {said}"
            ),
            other => panic!("an unready provider proves nothing, and this one {other:?}"),
        }
    }
}

/// A record and a configuration that name different pools refuse.
#[test]
fn a_record_and_a_configuration_naming_different_pools_refuse() {
    let places: ShutdownUnmountPlaces = places("readiness-pool-mismatch");
    let mut published: ReadinessRecord = record();
    published.pool_name = String::from("anotherpool");
    places
        .readiness
        .publish(&published)
        .expect("the record is published");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("the record is withdrawn");
    let said: String =
        what_the_record_verified(&readiness, &configuration()).expect_err("two pools is a refusal");
    assert!(
        said.contains("names pool `anotherpool` and the configuration names `zpcachyos`"),
        "the refusal names both pools, and this one says {said}"
    );
    assert!(
        said.contains("Ways forward:"),
        "a refusal says what to do next, and this one says {said}"
    );
}

/// A record and a configuration that name different checkouts refuse.
#[test]
fn a_record_and_a_configuration_naming_different_checkouts_refuse() {
    let places: ShutdownUnmountPlaces = places("readiness-checkout-mismatch");
    let mut published: ReadinessRecord = record();
    published.checkout_path = String::from("/home/user/elsewhere");
    places
        .readiness
        .publish(&published)
        .expect("the record is published");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("the record is withdrawn");
    let said: String = what_the_record_verified(&readiness, &configuration())
        .expect_err("two checkouts is a refusal");
    assert!(
        said.contains(
            "names checkout /home/user/elsewhere and the configuration names /home/user/Mono"
        ),
        "the refusal names both paths, and this one says {said}"
    );
}

/// A record this route's reader refuses proves nothing and is named.
#[test]
fn a_record_the_reader_refuses_proves_nothing_and_is_named() {
    let places: ShutdownUnmountPlaces = places("readiness-unreadable");
    places.readiness.prepare().expect("the directory is made");
    std::fs::write(places.readiness.record().as_path(), "not a record at all\n")
        .expect("the planted file is written");
    let readiness: BootsReadiness =
        withdraw_before_draining(&places).expect("a file the reader refuses is removed");
    assert!(
        matches!(readiness.withdrawal.stood, RecordStood::Unreadable { .. }),
        "the withdrawal answers what stood, and this one says {:?}",
        readiness.withdrawal.stood
    );
    match what_the_record_verified(&readiness, &configuration()).expect("an absence is no refusal")
    {
        Verified::Nothing {
            said,
        } => assert!(
            said.contains("is one this route's reader refuses"),
            "the sentence says the reader refused it, and this one says {said}"
        ),
        other => panic!("an unreadable record proves nothing, and this one {other:?}"),
    }
}

/// A readiness folder this route did not make refuses the whole pass.
#[test]
fn a_folder_this_route_did_not_make_refuses_the_pass() {
    let places: ShutdownUnmountPlaces = places("readiness-wrong-folder");
    places
        .readiness
        .publish(&record())
        .expect("the record is published");
    std::fs::set_permissions(
        places.readiness.directory(),
        std::os::unix::fs::PermissionsExt::from_mode(0o777),
    )
    .expect("the fixture takes another mode");
    let said: String =
        withdraw_before_draining(&places).expect_err("a folder at another mode is not the route's");
    assert!(
        said.contains("Nothing was unmounted and no durable intent was changed"),
        "the refusal says what it cost, and this one says {said}"
    );
    assert!(
        Path::new(places.readiness.record().as_path()).exists(),
        "the record of a folder the pass did not prove is left exactly as it was"
    );
}
