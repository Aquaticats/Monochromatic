//! What:
//!  The guest arm of the settling command,
//!  on a file-backed pool inside one throwaway `mvm` guest.
//! Why:
//!  The absence proof opens `/dev/zfs`,
//!  so no control on this machine may judge it against a pool it did not build
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16's "VM measurements").
//!  The arm measures the three states the design gives the command:
//!  a clone that stands refuses and changes nothing,
//!  a clone a person destroyed outside the wrapper is settled with its
//!  registration and its records,
//!  and a slot whose checkout has begun keeps its registration and its files
//!  while the records go.
//!  A run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the settling command guest arms', () => { /* stands, gone, checkout */ });
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use crate::worktree_clone_record_settling::settling_command::{
    SettlingDone,
    SettlingReads,
    settle_the_records_of_a_slot,
};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;
use crate::worktree_clone_record_settling::settling_steps::SettlingStep;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
    LivePoolFixture,
    PoolVariables,
    SLOT,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SETTLING_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_SETTLING_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_SETTLING_PREFIX";
/// The variable naming the folder the pool's image and the repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_SETTLING_ROOT";

/// The pool the arm builds when the driver names none,
/// which is no other arm's.
const DEFAULT_POOL: &str = "zfsstep16settling";
/// The folder the arm's image and repository live under when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-16-settling-fixture";

/// The design's own line for the fixture's clone,
/// whose GUID's last four digits are read from the record the fixture wrote.
fn the_request(fixture: &LivePoolFixture) -> SettlingRequest {
    let recorded = fixture.recorded();
    let whole: String = recorded.clone_guid.file_name();
    let digits: String = String::from(&whole[whole.len() - 4..]);
    return SettlingRequest {
        slot: String::from(SLOT),
        confirmed_slot: String::from(SLOT),
        digits,
    };
}

/// The `.git` pointer the recipe's first step removes,
/// which the fixture's record names and the slot must hold.
fn write_the_pointer(fixture: &LivePoolFixture) {
    std::fs::write(
        fixture.slot_path().join(".git"),
        format!("gitdir: {}\n", fixture.registration().display()),
    )
    .expect("the pointer writes");
}

/// Whether one dataset answers the guest's own `zfs list`,
/// which is the pool's answer and not this arm's reading of it.
fn the_pool_holds(
    fixture: &LivePoolFixture,
    name: &str,
) -> bool {
    return fixture.exists(name);
}

/// What:
///  The arm's one test.
/// Why:
///  Three states are three facts a person reads,
///  and each prints one marker line so a driver can refuse a run that
///  returned without performing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the settling command guest arms', () => { ... });
/// ```
#[test]
#[ignore = "runs only inside the throwaway guest through e2e/settling-command-guest.ts"]
fn the_settling_command_guest_arms() {
    let variables: PoolVariables = PoolVariables {
        guest: GUEST_VARIABLE,
        pool: POOL_VARIABLE,
        prefix: PREFIX_VARIABLE,
        root: ROOT_VARIABLE,
        default_root: DEFAULT_ROOT,
        default_pool: DEFAULT_POOL,
    };
    let Some(fixture) =
        LivePoolFixture::from_environment("the_settling_command_guest_arms", &variables)
    else {
        return;
    };
    let probe: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
    let mounts: crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets =
        crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
    let reads: SettlingReads<'_> = SettlingReads {
        probe: &probe,
        mounts: &mounts,
    };
    fixture.build(&[]);
    let recorded = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    write_the_pointer(&fixture);
    let request: SettlingRequest = the_request(&fixture);

    // The clone stands, so the command refuses and changes nothing.
    let refused = settle_the_records_of_a_slot(fixture.common().as_path(), &request, &reads)
        .expect_err("a standing clone refuses");
    assert!(
        matches!(refused, SettlingStop::TheCloneStillStands { .. }),
        "{refused:?}"
    );
    assert!(
        the_pool_holds(&fixture, fixture.clone().as_str()),
        "the clone still answers"
    );
    assert!(fixture.registration().exists(), "the registration stands");
    assert!(
        crate::worktree_clone_binding_record::registration_home::home_folder(
            fixture.common().as_path()
        )
        .join(format!("{SLOT}.binding"))
        .exists(),
        "the binding stands"
    );
    println!("SETTLING stands=refused");

    // A person destroys the clone outside the wrapper, which is the state the command exists for.
    fixture.run("zfs", &["destroy", "-r", fixture.clone().as_str()]);
    let settled: SettlingDone =
        settle_the_records_of_a_slot(fixture.common().as_path(), &request, &reads)
            .expect("a gone clone settles");
    assert!(
        !the_pool_holds(&fixture, fixture.clone().as_str()),
        "the clone is gone from the pool"
    );
    assert!(!fixture.registration().exists(), "the registration went");
    assert!(
        !crate::worktree_clone_binding_record::registration_home::home_folder(
            fixture.common().as_path()
        )
        .join(format!("{SLOT}.binding"))
        .exists(),
        "the binding went"
    );
    assert!(
        settled
            .lines
            .iter()
            .any(|line| return line.contains("the clone's absence is proved")),
        "{:?}",
        settled.lines
    );
    println!("SETTLING gone=settled records={}", settled.files.len());

    // A slot whose checkout has begun keeps its registration and its files.
    fixture.build(&[]);
    let checked_out = fixture.recorded();
    fixture.write_the_adds_records(&checked_out);
    write_the_pointer(&fixture);
    std::fs::write(
        fixture.slot_path().join("README.md"),
        b"# the person's file\n",
    )
    .expect("the file writes");
    // A rebuilt pool makes a new clone, so its GUID is not the first arm's and the confirmation is read again.
    let rebuilt: SettlingRequest = the_request(&fixture);
    fixture.run("zfs", &["destroy", "-r", fixture.clone().as_str()]);
    let with_a_checkout: SettlingDone =
        settle_the_records_of_a_slot(fixture.common().as_path(), &rebuilt, &reads)
            .expect("a checkout's records settle");
    assert!(with_a_checkout.checkout_begun, "a checkout had begun");
    assert_eq!(
        with_a_checkout.steps,
        vec![
            SettlingStep::TheOriginInventory,
            SettlingStep::TheCloneInventory,
            SettlingStep::TheBinding,
            SettlingStep::TheClaim
        ]
    );
    assert!(
        fixture.registration().exists(),
        "the registration stays for git worktree remove"
    );
    assert!(
        fixture.slot_path().join(".git").exists(),
        "the pointer stays"
    );
    assert!(
        fixture.slot_path().join("README.md").exists(),
        "the person's file stays"
    );
    println!("SETTLING checkout=records-only registration=kept");
    fixture.teardown();
    println!("SETTLING done");
    let _: &dyn MountedDatasets = &mounts;
}
