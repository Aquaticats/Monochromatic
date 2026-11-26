//! What:
//!  Step 16's guest arms of the snapshot batch,
//! run inside the throwaway `mvm` guest the recipe in
//!  `package/zfs/request-to-openzfs-kernel-device/e2e/` makes:
//!  a verified set
//!  destroyed in one request,
//! a set whose one held snapshot the kernel names while
//!  every name stays,
//! and a set split across requests by the caller's budget.
//! Why:
//!  The request, its per-name answer and the split are the kernel's own
//!  behaviour,
//! so a host run proves only the code around them;
//! these arms destroy real
//!  snapshots of a real clone on a file-backed pool inside a guest,
//! and the guest is
//!  destroyed by the recipe that made it
//! (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 16,
//!  "VM measurements" items 14,
//! 17 and 18 repeated through the binding).
//!  The hold arm shows the kernel's own
//!  no-destruction-on-failure rule:
//! the per-name list names the held snapshot and
//!  every other name is intact,
//! because a refused request destroys none of its set
//!  (`module/zfs/dsl_destroy.c:618` to `:706`, OpenZFS `zfs-2.4.4`).
//!  A run outside the
//!  guest prints `SKIP` and returns,
//! which the recipe refuses to count as a pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guest = inTheGuest('worktree_clone_snapshot_batch::snapshot_batch_arm_tests::...');
//! const names = guest.snapshot(guest.clone('one-request'), ['s1', 's2']);
//! const plan = planRequests(guest.pool, names, budgetFor(names));
//! sendRequest(ZfsDevice.open(), guest.pool, plan.chunks()[0]);
//! ```

use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_outcome::BatchOutcome;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::{
    RequestPlan,
    plan_requests,
};
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::{
    RequestResult,
    send_request,
};
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes;
use crate::worktree_clone_snapshot_batch::snapshot_name_outcome::NameOutcome;
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::ElementError;
use std::path::PathBuf;
use std::process::Output;

/// The variable that names the guest the recipe made.
const GUEST_VARIABLE: &str = "MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_GUEST";

/// The variable that names the recipe's pool.
const POOL_VARIABLE: &str = "MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_POOL";

/// The variable that names the directory holding the guest's `zfs` and
/// `zpool`.
const SBIN_VARIABLE: &str = "MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_SBIN";

/// The tag one arm holds a snapshot with.
const ARM_HOLD_TAG: &str = "snapshot-batch-arm";

/// The guest arms' fixture:
/// the recipe's pool and the guest's OpenZFS programs.
struct Guest {
    /// The pool the recipe made and owns.
    pool: String,
    /// The directory the guest's `zfs` and `zpool` live in.
    sbin: PathBuf,
}

/// What:
///  The run's guest,
/// its pool and its OpenZFS userspace,
/// or `None` on a run that is not
///  inside the guest.
/// Why:
///  These arms destroy snapshots,
/// so a run anywhere else must return rather
///  than touch a machine nobody offered;
/// the `SKIP` line is what the recipe fails
///  on.
fn in_the_guest(test: &str) -> Option<Guest> {
    let Some(named) = std::env::var_os(GUEST_VARIABLE) else {
        println!("SKIP {test}: {GUEST_VARIABLE} is unset, so this is not the recipe's guest");
        return None;
    };
    let machine: String = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .expect("/proc/sys/kernel/hostname is read")
        .trim()
        .to_string();
    let expected: String = named.to_string_lossy().into_owned();
    assert_eq!(
        machine, expected,
        "these arms run in the guest {expected} and not on {machine}"
    );
    // SAFETY: `geteuid` reads this process's own credentials and touches no memory.
    let euid: libc::uid_t = unsafe { libc::geteuid() };
    assert_eq!(
        euid, 0,
        "these arms make and destroy snapshots of the recipe's pool, so they run as root and not as {euid}"
    );
    let pool: String = std::env::var(POOL_VARIABLE).expect("the pool is named");
    let sbin: PathBuf = PathBuf::from(
        std::env::var_os(SBIN_VARIABLE).expect("the OpenZFS userspace directory is named"),
    );
    return Some(Guest {
        pool,
        sbin,
    });
}

/// The methods of `Guest`.
impl Guest {
    /// Run one OpenZFS program and return its standard output.
    ///
    /// # Panics
    ///
    /// When the program fails,
    /// with its standard error in the message,
    /// because a
    ///  fixture step that did not happen would make the assertions meaningless.
    fn program(
        &self,
        program: &str,
        arguments: &[&str],
    ) -> String {
        let output: Output = std::process::Command::new(self.sbin.join(program))
            .args(arguments)
            .output()
            .expect("the guest's OpenZFS userspace runs");
        assert!(
            output.status.success(),
            "{program} {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    }

    /// Run one OpenZFS program and report whether it succeeded.
    fn program_succeeds(
        &self,
        program: &str,
        arguments: &[&str],
    ) -> bool {
        return std::process::Command::new(self.sbin.join(program))
            .args(arguments)
            .output()
            .is_ok_and(|output| {
                return output.status.success();
            });
    }

    /// What:
    ///  One arm's own clone of the recipe's base snapshot.
    /// Why:
    ///  Each arm destroys its own snapshot set,
    /// so an arm never reads another's
    ///  leftovers;
    /// a clone an interrupted run left is cleared first,
    /// which includes
    ///  releasing a hold the interrupted arm took.
    fn make_clone(
        &self,
        arm: &str,
    ) -> String {
        let parent: String = format!("{}/clones", self.pool);
        let clone: String = format!("{parent}/{arm}");
        self.clear(clone.as_str());
        let origin: String = format!("{}/checkout@batch-base", self.pool);
        self.program(
            "zfs",
            &[
                "clone",
                "-o",
                "mountpoint=legacy",
                origin.as_str(),
                clone.as_str(),
            ],
        );
        return clone;
    }

    /// Release every hold the clone's snapshots carry and destroy the clone.
    fn clear(
        &self,
        clone: &str,
    ) {
        if self.program_succeeds("zfs", &["holds", "-H", "-r", clone]) {
            let listed: String = self.program("zfs", &["holds", "-H", "-r", clone]);
            for line in listed.lines() {
                let mut columns = line.split('\t');
                let (Some(name), Some(tag)) = (columns.next(), columns.next()) else {
                    continue;
                };
                self.program_succeeds("zfs", &["release", tag, name]);
            }
        }
        self.program_succeeds("zfs", &["destroy", "-r", "-f", clone]);
    }

    /// Make the named snapshots of `dataset` and return their full names.
    fn snapshots(
        &self,
        dataset: &str,
        suffixes: &[&str],
    ) -> Vec<String> {
        return suffixes
            .iter()
            .map(|suffix| {
                let name: String = format!("{dataset}@{suffix}");
                self.program("zfs", &["snapshot", name.as_str()]);
                return name;
            })
            .collect();
    }

    /// The snapshot names `dataset` holds now.
    fn live_snapshots(
        &self,
        dataset: &str,
    ) -> Vec<String> {
        let printed: String = self.program(
            "zfs",
            &[
                "list", "-H", "-p", "-o", "name", "-t", "snapshot", "-r", dataset,
            ],
        );
        return printed
            .lines()
            .map(|line| {
                return line.trim().to_string();
            })
            .filter(|line| {
                return !line.is_empty();
            })
            .collect();
    }

    /// The budget that exactly fits the named set's own request.
    fn budget_for(
        &self,
        names: &[String],
    ) -> RequestBudget {
        return RequestBudget::of_input_bytes(
            encoded_request_bytes(self.pool.as_str(), names).expect("the names fit"),
        );
    }
}

/// The outcome of one answered request,
/// which every arm needs as a value.
fn answered(
    result: RequestResult,
    arm: &str,
) -> BatchOutcome {
    let RequestResult::Answered(outcome) = result else {
        panic!("{arm}: a non-empty set is sent, never skipped");
    };
    return outcome;
}

/// A verified set is destroyed in one request and the clone itself stands.
#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn snapshot_batch_arm_one_request_destroys_the_whole_set() {
    let Some(guest) = in_the_guest(
        "worktree_clone_snapshot_batch::snapshot_batch_arm_tests::snapshot_batch_arm_one_request_destroys_the_whole_set",
    ) else {
        return;
    };
    let clone: String = guest.make_clone("one-request");
    let names: Vec<String> = guest.snapshots(clone.as_str(), &["s1", "s2", "s3", "s4", "s5"]);
    let budget: RequestBudget = guest.budget_for(names.as_slice());
    let plan: RequestPlan =
        plan_requests(guest.pool.as_str(), names.as_slice(), budget).expect("the set is planable");
    assert_eq!(
        plan.chunks().len(),
        1,
        "the whole set fits the arm's budget, so it is one request"
    );
    let device: ZfsDevice = ZfsDevice::open().expect("the guest has the device");
    let result: RequestResult =
        send_request(&device, guest.pool.as_str(), plan.chunks()[0].as_slice())
            .expect("the request is built and sent");
    let outcome: BatchOutcome = answered(result, "one request");
    assert!(
        outcome.failures().is_empty(),
        "the set names only the clone's own snapshots: {:?}",
        outcome.failures()
    );
    for submitted in outcome.names() {
        assert_eq!(
            submitted.outcome,
            NameOutcome::Destroyed,
            "{} must be destroyed by the one request",
            submitted.name
        );
    }
    assert!(
        guest.live_snapshots(clone.as_str()).is_empty(),
        "no snapshot of the clone is left"
    );
    assert!(
        guest.program_succeeds("zfs", &["list", "-H", "-o", "name", clone.as_str()]),
        "the clone itself stands while its own snapshots are gone"
    );
}

/// A held snapshot is named in the per-name list and every name stays.
#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn snapshot_batch_arm_a_hold_names_one_and_keeps_every_name() {
    let Some(guest) = in_the_guest(
        "worktree_clone_snapshot_batch::snapshot_batch_arm_tests::snapshot_batch_arm_a_hold_names_one_and_keeps_every_name",
    ) else {
        return;
    };
    let clone: String = guest.make_clone("held");
    let names: Vec<String> = guest.snapshots(clone.as_str(), &["s1", "s2", "s3", "s4"]);
    let held: String = names[1].clone();
    guest.program("zfs", &["hold", ARM_HOLD_TAG, held.as_str()]);
    let budget: RequestBudget = guest.budget_for(names.as_slice());
    let plan: RequestPlan =
        plan_requests(guest.pool.as_str(), names.as_slice(), budget).expect("the set is planable");
    let device: ZfsDevice = ZfsDevice::open().expect("the guest has the device");
    let result: RequestResult =
        send_request(&device, guest.pool.as_str(), plan.chunks()[0].as_slice())
            .expect("the request is built and sent");
    let outcome: BatchOutcome = answered(result, "a hold");
    assert_eq!(
        outcome.errno(),
        Some(libc::EBUSY),
        "a hold makes the kernel refuse the whole request with EBUSY"
    );
    assert_eq!(
        outcome.failures(),
        [ElementError {
            name: held.clone(),
            errno: libc::EBUSY,
        }],
        "the per-name list names the held snapshot alone"
    );
    assert_eq!(
        outcome.outcome_of(held.as_str()),
        Some(&NameOutcome::Refused {
            errno: libc::EBUSY
        })
    );
    for name in &names {
        if *name == held {
            continue;
        }
        assert_eq!(
            outcome.outcome_of(name.as_str()),
            Some(&NameOutcome::Intact),
            "the kernel destroys none of a refused request, so {name} is intact"
        );
    }
    let mut still_there: Vec<String> = guest.live_snapshots(clone.as_str());
    still_there.sort();
    let mut wanted: Vec<String> = names.clone();
    wanted.sort();
    assert_eq!(
        still_there, wanted,
        "every name of the refused set is still there"
    );
    assert!(
        guest
            .program("zfs", &["holds", "-H", held.as_str()])
            .contains(ARM_HOLD_TAG),
        "the hold the refusal named is still the arm's own"
    );
    guest.program("zfs", &["release", ARM_HOLD_TAG, held.as_str()]);
}

/// A set over the caller's budget is destroyed by several requests.
#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn snapshot_batch_arm_a_budget_splits_the_set_across_requests() {
    let Some(guest) = in_the_guest(
        "worktree_clone_snapshot_batch::snapshot_batch_arm_tests::snapshot_batch_arm_a_budget_splits_the_set_across_requests",
    ) else {
        return;
    };
    let clone: String = guest.make_clone("split");
    let names: Vec<String> = guest.snapshots(clone.as_str(), &["s1", "s2", "s3", "s4", "s5", "s6"]);
    let mut sorted: Vec<String> = names.clone();
    sorted.sort();
    let two: Vec<String> = sorted[..2].to_vec();
    let budget: RequestBudget = guest.budget_for(two.as_slice());
    let plan: RequestPlan =
        plan_requests(guest.pool.as_str(), names.as_slice(), budget).expect("the set is planable");
    assert_eq!(
        plan.chunks().len(),
        3,
        "six names with a two-name budget make three requests"
    );
    assert!(plan.chunks().iter().all(|chunk| {
        return chunk.len() == 2;
    }));
    let device: ZfsDevice = ZfsDevice::open().expect("the guest has the device");
    let mut encoded: usize = 0;
    for (index, chunk) in plan.chunks().iter().enumerate() {
        let result: RequestResult = send_request(&device, guest.pool.as_str(), chunk.as_slice())
            .expect("each chunk is built and sent");
        let outcome: BatchOutcome = answered(result, "a split chunk");
        assert!(
            outcome.failures().is_empty(),
            "chunk {index} names only the clone's own snapshots: {:?}",
            outcome.failures()
        );
        assert!(outcome.names().iter().all(|submitted| {
            return submitted.outcome == NameOutcome::Destroyed;
        }));
        encoded += plan.encoded_sizes()[index];
    }
    assert!(
        guest.live_snapshots(clone.as_str()).is_empty(),
        "the requests together destroyed every name"
    );
    println!(
        "MEASUREMENT snapshot-batch requests={} names={} budget-bytes={} encoded-bytes={encoded}",
        plan.chunks().len(),
        plan.name_count(),
        budget.input_bytes()
    );
}
