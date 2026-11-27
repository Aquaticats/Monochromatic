//! What:
//!  The guest arm of plan step 15's piece:
//!  the planned removal going on,
//!  an origin snapshot marked with `zfs destroy -d` refusing,
//!  the reap that mark causes when a removal goes ahead,
//!  and an origin moved by `zfs promote` refusing.
//! Why:
//!  The mark is the kernel's own field and only a real module answers it,
//!  so the arm runs inside the throwaway `mvm` guest the package's e2e recipe drives
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Measurements owed",
//! M1 and control 28).
//!  The reap is the arm's positive control:
//!  a harness that cannot see the loss cannot show that the refusal avoids it
//!  (rule `QPC`).
//!  A run on this machine has no fixture and prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!  The tuple is read from the live objects here because step 1's lineage proof is step 14's and
//!  no removal of this route exists yet;
//!  the arm judges the check,
//! never the proof.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the deferred-origin guest arm', () => { /* zpool, clone, mark, promote */ });
//! ```

use super::super::origin_mark::{
    MARK_OFF,
    MARK_ON,
    OriginMark,
    origin_mark,
};
use super::super::origin_refusal::OriginRefusal;
use super::super::planned_origin::PlannedOrigin;
use super::{
    Verdict,
    check_planned_origin,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ZfsDatasetReader,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
};

/// The variable naming the guest a live arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_DEFERRED_ORIGIN_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_DEFERRED_ORIGIN_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_DEFERRED_ORIGIN_PREFIX";
/// The variable naming the folder the pool's image and mountpoints live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_DEFERRED_ORIGIN_ROOT";

/// The guest directory the fixture lives in when the recipe names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-15-fixture";
/// The guest's OpenZFS 2.4.4 prefix when the recipe names none.
const DEFAULT_PREFIX: &str = "/opt/openzfs-2.4.4";
/// The pool the arm builds when the recipe names none.
const DEFAULT_POOL: &str = "zfsstep15";
/// The image the pool stands on,
/// in gibibytes.
const POOL_IMAGE_SIZE: &str = "1G";
/// The slot the arm's clone takes.
const SLOT: &str = "fix-flaky-test";
/// The origin snapshot's own name,
/// in the form the add route gives it.
const ORIGIN_SNAPSHOT: &str = "git-worktree-clone-origin-0011223344556677";
/// The second origin snapshot the promotion arm makes.
const PROMOTED_SNAPSHOT: &str = "git-worktree-clone-origin-ffeeddccbbaa9988";

/// What:
///  The fixture one live arm may run in.
/// Why:
///  A live request opens `/dev/zfs`,
///  so the arm refuses to run anywhere but the guest the recipe names,
///  and it says by name why it skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = Fixture.fromEnvironment('the-deferred-origin-arm');
/// ```
struct Fixture {
    /// The guest's host name.
    guest: String,
    /// The pool the arm builds and destroys.
    pool: String,
    /// The OpenZFS 2.4.4 prefix whose `sbin` and `bin` the commands come from.
    prefix: PathBuf,
    /// The folder the pool's image lives under.
    root: PathBuf,
}

impl Fixture {
    /// The fixture the environment names,
    /// or nothing on a host run,
    /// where no pool command may be made.
    fn from_environment(test: &str) -> Option<Fixture> {
        let Some(named) = std::env::var_os(GUEST_VARIABLE) else {
            println!(
                "SKIPPED {test}: {GUEST_VARIABLE} is not set, so this machine has no guest fixture and no live arm ran"
            );
            return None;
        };
        let guest: String = named.to_string_lossy().into_owned();
        let host: String = Self::hostname();
        if host != guest {
            panic!(
                "{test}: {GUEST_VARIABLE} names the guest `{guest}` and this machine calls itself `{host}`, so a live arm would reach the wrong pool"
            );
        }
        return Some(Fixture {
            guest,
            pool: std::env::var(POOL_VARIABLE)
                .unwrap_or_else(|_| return String::from(DEFAULT_POOL)),
            prefix: PathBuf::from(
                std::env::var(PREFIX_VARIABLE)
                    .unwrap_or_else(|_| return String::from(DEFAULT_PREFIX)),
            ),
            root: PathBuf::from(
                std::env::var(ROOT_VARIABLE).unwrap_or_else(|_| return String::from(DEFAULT_ROOT)),
            ),
        });
    }

    /// This machine's own host name.
    fn hostname() -> String {
        let output: Output = Command::new("hostname")
            .output()
            .expect("the guest has `hostname`");
        return String::from_utf8_lossy(output.stdout.as_slice())
            .trim()
            .to_string();
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    fn search_path(&self) -> String {
        return format!(
            "{}:{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            self.prefix.join("sbin").display(),
            self.prefix.join("bin").display()
        );
    }

    /// One pool command,
    /// whose output the arm keeps for the log.
    fn run(
        &self,
        program: &str,
        arguments: &[&str],
    ) -> String {
        let output: Output = Command::new(program)
            .args(arguments)
            .env("PATH", self.search_path())
            .output()
            .unwrap_or_else(|error| {
                panic!(
                    "`{program} {}` could not start in the guest: {error}",
                    arguments.join(" ")
                )
            });
        assert!(
            output.status.success(),
            "`{program} {}` exited {}: {}{}",
            arguments.join(" "),
            output.status,
            String::from_utf8_lossy(output.stdout.as_slice()),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return String::from_utf8_lossy(output.stdout.as_slice())
            .trim()
            .to_string();
    }

    /// Whether the named dataset or snapshot exists in the pool.
    fn exists(
        &self,
        name: &str,
    ) -> bool {
        let output: Output = Command::new("zfs")
            .args(["list", "-H", "-o", "name", name])
            .env("PATH", self.search_path())
            .output()
            .expect("the guest has `zfs`");
        return output.status.success();
    }

    /// The full name of one dataset in this pool.
    fn named(
        &self,
        below: &str,
    ) -> String {
        return format!("{}/{below}", self.pool);
    }

    /// Build the pool,
    /// the checkout,
    /// the clone container and one clone,
    /// with the origin snapshot the check judges.
    fn build(
        &self,
        snapshot: &str,
        slot: &str,
    ) -> PlannedOrigin {
        self.run(
            "mkdir",
            &[
                "--parents",
                self.root.to_str().expect("the fixture root is text"),
            ],
        );
        let image: PathBuf = self.root.join("pool.img");
        let image_text: &str = image.to_str().expect("the image path is text");
        if Path::new(image_text).exists() {
            self.run("zpool", &["destroy", "-f", self.pool.as_str()]);
            std::fs::remove_file(image.as_path()).expect("the earlier arm's image goes");
        }
        self.run("truncate", &["--size", POOL_IMAGE_SIZE, image_text]);
        self.run("zpool", &["create", "-f", self.pool.as_str(), image_text]);
        let checkout: String = self.named("checkout");
        let clones: String = self.named("clones");
        self.run(
            "zfs",
            &["create", "-o", "mountpoint=legacy", checkout.as_str()],
        );
        self.run(
            "zfs",
            &[
                "create",
                "-o",
                "mountpoint=legacy",
                "-o",
                "canmount=off",
                clones.as_str(),
            ],
        );
        let origin: String = format!("{checkout}@{snapshot}");
        self.run("zfs", &["snapshot", origin.as_str()]);
        let clone: String = format!("{clones}/{slot}");
        self.run("zfs", &["clone", origin.as_str(), clone.as_str()]);
        self.run(
            "zfs",
            &["snapshot", format!("{clone}@manual-2026-11-21").as_str()],
        );
        return self.planned_for(&clone, &checkout, &origin, slot);
    }

    /// The tuple a removal would have recorded,
    /// read through the same seam the check reads.
    fn planned_for(
        &self,
        clone: &str,
        checkout: &str,
        origin: &str,
        slot: &str,
    ) -> PlannedOrigin {
        let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");
        let clone_reply: DatasetIdentity = reader.identity(clone).expect("the clone is read");
        let dataset_reply: DatasetIdentity = reader
            .identity(checkout)
            .expect("the origin dataset is read");
        let snapshot_reply: DatasetIdentity = reader
            .identity(origin)
            .expect("the origin snapshot is read");
        return PlannedOrigin {
            slot: String::from(slot),
            clone_name: String::from(clone),
            clone_guid: guid_of(clone_reply.stats.dds_guid),
            origin_dataset_name: String::from(checkout),
            origin_dataset_guid: guid_of(dataset_reply.stats.dds_guid),
            origin_snapshot_name: String::from(origin),
            origin_snapshot_guid: guid_of(snapshot_reply.stats.dds_guid),
            origin_snapshot_createtxg: snapshot_reply.stats.dds_creation_txg,
        };
    }

    /// Tear the fixture down,
    /// whatever the arms left.
    fn teardown(&self) {
        let _ = Command::new("zpool")
            .args(["destroy", "-f", self.pool.as_str()])
            .env("PATH", self.search_path())
            .output();
        let image: PathBuf = self.root.join("pool.img");
        let _ = std::fs::remove_file(image.as_path());
    }
}

/// One live GUID as a planned tuple holds it.
fn guid_of(value: u64) -> crate::worktree_clone_mount_protocol::DatasetGuid {
    return crate::worktree_clone_mount_protocol::DatasetGuid::parse(value.to_string().as_str())
        .expect("the kernel reports a canonical GUID");
}

/// The cause one verdict refused with,
/// and the run's own word for it.
fn cause_of(verdict: Verdict) -> OriginRefusal {
    return match verdict {
        Verdict::Proceed => panic!("the check proceeded over the state this arm planted"),
        Verdict::Refused(cause) => cause,
    };
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_deferred_origin_guest_arms() {
    let Some(fixture) = Fixture::from_environment("the_deferred_origin_guest_arms") else {
        return;
    };
    println!("DEFERRED guest={} pool={}", fixture.guest, fixture.pool);
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest has /dev/zfs");

    // The planned case: an unmarked origin goes on after exactly three reads.
    let planned: PlannedOrigin = fixture.build(ORIGIN_SNAPSHOT, SLOT);
    assert_eq!(
        origin_mark(&reader, planned.origin_snapshot_name.as_str()),
        OriginMark::Unmarked,
        "the fresh origin reads {MARK_OFF}"
    );
    assert_eq!(
        check_planned_origin(&reader, &planned),
        Verdict::Proceed,
        "an unmarked origin goes on"
    );
    println!("DEFERRED planned=proceed mark={MARK_OFF}");

    // The marked case: the account marks the origin snapshot, which marks because the clone exists.
    fixture.run(
        "zfs",
        &["destroy", "-d", planned.origin_snapshot_name.as_str()],
    );
    assert_eq!(
        origin_mark(&reader, planned.origin_snapshot_name.as_str()),
        OriginMark::Marked {
            clones: 1,
            holds: super::super::origin_mark::UserHolds::Counted(0)
        },
        "the mark reads {MARK_ON} with one clone and no hold"
    );
    let marked_cause: OriginRefusal = cause_of(check_planned_origin(&reader, &planned));
    match &marked_cause {
        OriginRefusal::OriginMarkedForDeferredDestroy {
            clones,
            holds,
            ..
        } => {
            println!("DEFERRED marked counts clones={clones} holds={holds:?}");
        }
        other => panic!("a marked origin must refuse with its own cause, not {other:?}"),
    }
    assert!(
        marked_cause
            .message(&super::super::removal_history::RemovalHistory::NothingDestroyed)
            .contains("so destroying this clone destroys that snapshot in the same transaction"),
        "the refusal says what the next destroy would do"
    );
    println!("DEFERRED marked=refused step={}", marked_cause.step());

    // The reap the refusal avoids: the clone's own snapshots, then the head, take the marked origin with them.
    fixture.run(
        "zfs",
        &[
            "destroy",
            format!("{}@manual-2026-11-21", planned.clone_name).as_str(),
        ],
    );
    fixture.run("zfs", &["destroy", planned.clone_name.as_str()]);
    assert!(
        !fixture.exists(planned.origin_snapshot_name.as_str()),
        "the head destroy reaped the marked origin snapshot in the same transaction"
    );
    println!(
        "DEFERRED reaped=gone origin={}",
        planned.origin_snapshot_name
    );

    // The promotion: a second clone of a second origin is promoted, so its origin moves under the plan.
    fixture.teardown();
    let promoted: PlannedOrigin = fixture.build(PROMOTED_SNAPSHOT, SLOT);
    assert_eq!(
        check_planned_origin(&reader, &promoted),
        Verdict::Proceed,
        "the second clone goes on before the promotion"
    );
    fixture.run("zfs", &["promote", promoted.clone_name.as_str()]);
    let promoted_cause: OriginRefusal = cause_of(check_planned_origin(&reader, &promoted));
    println!(
        "DEFERRED promoted=refused cause={} step={}",
        promoted_cause.cause_name(),
        promoted_cause.step()
    );
    assert_ne!(
        promoted_cause.step(),
        "origin-mark",
        "a moved origin refuses on an identity, not on the mark"
    );
    assert_eq!(
        promoted_cause.form(),
        super::super::origin_refusals::RefusalForm::NothingCompared,
        "no request follows a moved origin"
    );

    fixture.teardown();
    println!("DEFERRED done");
}
