//! What:
//!  Plan step 22's pinned-output arm in the throwaway guest:
//!  incompressible bytes written into the checkout's ignored folder,
//!  pinned by an hourly snapshot,
//!  overwritten after that snapshot,
//!  and then what a worktree clone's removal and what the snapshot's expiry free,
//!  read from ZFS's own accounting once the transaction group has settled,
//!  beside the same sequence over one repeated byte.
//! Why:
//!  The record of 2026-11-25
//!  (`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`)
//!  wrote twelve mebibytes of one repeated byte,
//!  so the pool stored the output at almost no cost,
//!  and the figures it printed were a snapshot's own accounting rather than retained build output.
//!  The correcting probe never ran there.
//!  So this arm states the bytes it writes,
//!  the snapshots it takes and the clone it removes before it runs,
//!  prints ZFS's own figures for each stage,
//!  and reads every free as a difference of two readings taken after their own
//!  transaction group settled.
//!  The repeated-byte case is the positive control of that reading:
//!  the same probe must report a large pinned figure for bytes that do not
//!  compress and a small one for bytes that do,
//!  or a reading of zero would say nothing.
//!  The arm drives the datasets as root in a guest of the driver's own,
//!  never on a machine somebody uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pinnedOutputCase(fixture, 'incompressible'); // 64 MiB of a seeded stream, one hourly snapshot, one clone
//! pinnedOutputCase(fixture, 'repeated-byte');  // the same sequence over one repeated byte, the control
//! ```

#![cfg(target_os = "linux")]

use std::path::{
    Path,
    PathBuf,
};

use super::zfs_step_22_measurement::Fixture;

/// The folder under the checkout one case writes,
/// which the checkout's own ignore rule hides.
const OUTPUT_FOLDER: &str = "build-output-pinned";

/// The clone dataset's leaf name inside the route's container,
/// which the removal takes away.
const REMOVAL_CLONE_LEAF: &str = "pinned-output-removal";

/// The slot folder the removal's clone stands at while the case writes into it.
const REMOVAL_SLOT: &str = "pinned-output-removal";

/// How many files one case writes.
const OUTPUT_FILES: usize = 8;

/// How many mebibytes each of those files holds,
/// so one case writes `OUTPUT_FILES * OUTPUT_FILE_MIB` mebibytes.
const OUTPUT_FILE_MIB: usize = 8;

/// How many transaction groups the arm forces while it waits for one set of figures to settle.
/// Why:
///  A reading taken before the transaction group that holds the write or the
///  destroy has closed reports the state before it,
///  and the record's own arm printed such a reading as a free.
const SETTLE_ROUNDS: u32 = 64;

/// The stamp the incompressible case's hourly snapshot carries,
/// in the shape Sanoid writes and the guard's own pattern reads.
const INCOMPRESSIBLE_HOURLY: &str = "autosnap_2026-11-27_00:00:00_hourly";

/// The stamp the control case's hourly snapshot carries.
const REPEATED_HOURLY: &str = "autosnap_2026-11-27_01:00:00_hourly";

/// The seed the first pass over the output generates from.
const FIRST_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// The seed the overwrite generates from,
/// which is a different stream so the pinned blocks are not the live ones.
const SECOND_SEED: u64 = 0xD1B5_4A32_D192_ED03;

/// The seed the removal clone's own output generates from.
const CLONE_SEED: u64 = 0xA24B_AED4_963E_E407;

/// The multiplier of the xorshift64* stream the incompressible case writes.
const STREAM_MULTIPLIER: u64 = 0x2545_F491_4F6C_DD1D;

/// What one case writes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Content {
    /// Bytes a seeded stream produces,
    /// which the pool's compressor cannot shrink.
    Incompressible,
    /// One byte repeated,
    /// which is the record's own output and this arm's positive control.
    RepeatedByte,
}

impl Content {
    /// The word this case's printed lines carry.
    fn name(self) -> &'static str {
        return match self {
            Content::Incompressible => "incompressible",
            Content::RepeatedByte => "repeated-byte",
        };
    }

    /// The hourly snapshot's stamp for this case.
    fn hourly(self) -> &'static str {
        return match self {
            Content::Incompressible => INCOMPRESSIBLE_HOURLY,
            Content::RepeatedByte => REPEATED_HOURLY,
        };
    }
}

/// One settled reading of ZFS's own accounting:
/// the checkout dataset's figures and the pool's allocated bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Figures {
    /// The dataset's `used`.
    used: u64,
    /// The dataset's `usedbydataset`.
    dataset: u64,
    /// The dataset's `usedbysnapshots`.
    snapshots: u64,
    /// The pool's `allocated`.
    allocated: u64,
}

/// One settled reading beside the number of transaction groups it took.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Settled {
    /// The figures both readings agreed on.
    figures: Figures,
    /// How many groups the arm forced before they did.
    rounds: u32,
}

/// What:
///  Each case's whole sequence,
///  the incompressible measurement and then the control that proves the probe can move.
/// Why:
///  The control runs through the same code,
///  on the same pool and the same checkout dataset,
///  so the only fact that differs between the two lines is the bytes.
pub(super) fn pinned_output_arm(fixture: &Fixture) {
    pinned_output_case(fixture, Content::Incompressible);
    pinned_output_case(fixture, Content::RepeatedByte);
}

/// What:
///  One case:
///  write the bytes,
///  pin them with an hourly snapshot,
///  overwrite them,
///  write the removal clone's own output,
///  then remove the clone,
///  expire the snapshot and remove the live copy,
///  reading the settled figures at every stage.
/// Why:
///  The plan asks what expiration,
///  removal and the guard can free,
///  and each of those is one of the last three steps here.
fn pinned_output_case(
    fixture: &Fixture,
    content: Content,
) {
    let case: &str = content.name();
    let checkout: &str = fixture.checkout.as_str();
    let snapshot: String = format!("{checkout}@{}", content.hourly());
    let clone: String = format!("{}/{}", fixture.container, REMOVAL_CLONE_LEAF);
    let folder: PathBuf = fixture.root.join(".claude").join(OUTPUT_FOLDER);
    let slot: PathBuf = fixture.slot(REMOVAL_SLOT);
    // A killed run leaves the case's own clone, snapshot and folders behind, and a case replaces everything it made
    // rather than reading a predecessor's leftovers.
    let _left_clone: (bool, String, String) = fixture.zfs(&["destroy", "-r", "-f", clone.as_str()]);
    let _left_snapshot: (bool, String, String) = fixture.zfs(&["destroy", "-f", snapshot.as_str()]);
    let _left_folder: std::io::Result<()> = std::fs::remove_dir_all(folder.as_path());
    let _left_slot: std::io::Result<()> = std::fs::remove_dir_all(slot.as_path());

    let before: Settled = settle(fixture);
    stage(case, "before", before);
    let compression: String = property(fixture, "compression", checkout);
    let ratio_before: String = property(fixture, "compressratio", checkout);
    let written: u64 = write_the_output(folder.as_path(), content, FIRST_SEED);
    let after_write: Settled = settle(fixture);
    let ratio_after: String = property(fixture, "compressratio", checkout);
    stage(case, "written", after_write);
    println!(
        "STEP22 pinned case={case} written-bytes={written} files={OUTPUT_FILES} file-mib={OUTPUT_FILE_MIB} compression={compression} compressratio-before={ratio_before} compressratio-after-write={ratio_after}"
    );

    fixture.zfs_ok(&["snapshot", snapshot.as_str()]);
    let pinned: Settled = settle(fixture);
    stage(case, "pinned", pinned);

    let rewritten: u64 = write_the_output(folder.as_path(), content, SECOND_SEED);
    let overwritten: Settled = settle(fixture);
    stage(case, "overwritten", overwritten);
    println!("STEP22 pinned case={case} rewritten-bytes={rewritten}");

    // The clone a removal takes away, mounted at its slot so the case writes the output a worktree's build leaves.
    std::fs::create_dir_all(slot.as_path()).expect("the removal clone's slot folder");
    fixture.zfs_ok(&[
        "clone",
        "-o",
        "mountpoint=legacy",
        snapshot.as_str(),
        clone.as_str(),
    ]);
    super::zfs_sweep_guest::command_ok(
        "mount",
        &[
            "-t",
            "zfs",
            clone.as_str(),
            slot.to_str().expect("a text path"),
        ],
    );
    let clone_written: u64 = write_the_output(slot.as_path(), content, CLONE_SEED);
    let clone_alive: Settled = settle(fixture);
    let clone_used: u64 = fixture.bytes("used", clone.as_str());
    stage(case, "clone-written", clone_alive);
    println!(
        "STEP22 pinned case={case} clone-written-bytes={clone_written} clone-used={clone_used}"
    );

    // The removal: the clone goes, and with it the output it wrote. The snapshot that pins the checkout's own output
    // is still there, so this reading says what a removal alone frees.
    super::zfs_step_22_measurement::unmount_quiet(slot.as_path());
    let gone: (bool, String, String) = fixture.zfs(&["destroy", "-r", "-f", clone.as_str()]);
    assert!(gone.0, "the removal's clone is destroyed: {}", gone.2);
    let after_removal: Settled = settle(fixture);
    stage(case, "after-removal", after_removal);
    let _slot: std::io::Result<()> = std::fs::remove_dir_all(slot.as_path());

    // The expiry of the snapshot that pinned the overwritten output.
    let expired: (bool, String, String) = fixture.zfs(&["destroy", "-f", snapshot.as_str()]);
    assert!(expired.0, "the hourly snapshot is destroyed: {}", expired.2);
    let after_expiry: Settled = settle(fixture);
    stage(case, "after-expiry", after_expiry);

    // The live copy's own removal, which is what a person's cleanup or the guard's pruning of the dataset's own
    // snapshots leaves behind.
    std::fs::remove_dir_all(folder.as_path()).expect("the case takes its own output away");
    let after_output: Settled = settle(fixture);
    stage(case, "after-output-removed", after_output);

    println!(
        "STEP22 pinned case={case} snapshot-used-at-snapshot={} snapshot-used-after-overwrite={} live-at-overwrite={} freed-by-removal-clone-used={clone_used} freed-by-removal-pool={} freed-by-expiry-dataset={} freed-by-expiry-pool={} freed-by-output-removal-dataset={} freed-by-output-removal-pool={} untouched-by-removal-dataset={}",
        pinned.figures.snapshots,
        overwritten.figures.snapshots,
        overwritten.figures.dataset,
        clone_alive
            .figures
            .allocated
            .saturating_sub(after_removal.figures.allocated),
        after_removal
            .figures
            .used
            .saturating_sub(after_expiry.figures.used),
        after_removal
            .figures
            .allocated
            .saturating_sub(after_expiry.figures.allocated),
        after_expiry
            .figures
            .used
            .saturating_sub(after_output.figures.used),
        after_expiry
            .figures
            .allocated
            .saturating_sub(after_output.figures.allocated),
        before.figures.used.abs_diff(after_removal.figures.used),
    );
}

/// Print one stage's settled figures.
fn stage(
    case: &str,
    name: &str,
    settled: Settled,
) {
    println!(
        "STEP22 pinned case={case} stage={name} used={} usedbydataset={} usedbysnapshots={} pool-allocated={} settle-rounds={} settled={}",
        settled.figures.used,
        settled.figures.dataset,
        settled.figures.snapshots,
        settled.figures.allocated,
        settled.rounds,
        settled.rounds < SETTLE_ROUNDS
    );
}

/// One of the checkout dataset's properties,
/// as text.
fn property(
    fixture: &Fixture,
    name: &str,
    dataset: &str,
) -> String {
    let (_ok, said, _error): (bool, String, String) =
        fixture.zfs(&["get", "-H", "-p", "-o", "value", name, dataset]);
    return said.trim().to_string();
}

/// What:
///  Write every file of one case's output into `folder`,
///  replacing whatever a killed run left,
///  and answer the bytes written.
/// Why:
///  The bytes are the arm's stated count:
///  `OUTPUT_FILES` files of `OUTPUT_FILE_MIB` mebibytes each,
///  so a reader checks the record's sentence against this figure rather
///  than against a count of whatever the folder happened to hold.
fn write_the_output(
    folder: &Path,
    content: Content,
    seed: u64,
) -> u64 {
    std::fs::create_dir_all(folder).expect("the output's folder");
    let mut written: u64 = 0;
    for index in 0..OUTPUT_FILES {
        let block: Vec<u8> = block_of(content, seed.wrapping_add(index as u64));
        written += block.len() as u64;
        std::fs::write(folder.join(format!("block-{index}")), block.as_slice())
            .expect("one write of the case's output");
    }
    return written;
}

/// One file of the case's output.
fn block_of(
    content: Content,
    seed: u64,
) -> Vec<u8> {
    let bytes: usize = OUTPUT_FILE_MIB * 1024 * 1024;
    let mut block: Vec<u8> = Vec::with_capacity(bytes);
    if content == Content::RepeatedByte {
        block.resize(bytes, b'x');
        return block;
    }
    let mut state: u64 = seed | 1;
    while block.len() < bytes {
        let word: u64 = next_word(&mut state);
        block.extend_from_slice(word.to_le_bytes().as_slice());
    }
    return block;
}

/// One word of a xorshift64* stream,
/// which no compressor in the pool shrinks at this length.
fn next_word(state: &mut u64) -> u64 {
    let mut word: u64 = *state;
    word ^= word >> 12;
    word ^= word << 25;
    word ^= word >> 27;
    *state = word;
    return word.wrapping_mul(STREAM_MULTIPLIER);
}

/// What:
///  Force a transaction group and read the figures until two readings taken
///  after their own group agree,
///  and answer the last reading with the rounds it took.
/// Why:
///  A figure read before the group holding a write or a destroy has closed is
///  the figure from before it,
///  and `zpool sync` is what makes the pool answer from the group it closed.
///  The bound is `SETTLE_ROUNDS`,
///  so a reading that took every round is answered at the bound and is not a
///  settled one:
///  the stage line says which of the two it is.
fn settle(fixture: &Fixture) -> Settled {
    sync_the_pool(fixture);
    let mut previous: Figures = figures(fixture);
    for round in 1..=SETTLE_ROUNDS {
        sync_the_pool(fixture);
        let now: Figures = figures(fixture);
        if now == previous {
            return Settled {
                figures: now,
                rounds: round,
            };
        }
        previous = now;
    }
    return Settled {
        figures: previous,
        rounds: SETTLE_ROUNDS,
    };
}

/// Force the current transaction group to close.
fn sync_the_pool(fixture: &Fixture) {
    let (ok, _said, error): (bool, String, String) = super::zfs_sweep_guest::zfs(
        fixture.sbin.as_str(),
        "zpool",
        &["sync", fixture.pool.as_str()],
        None,
    );
    assert!(ok, "zpool sync of {} was refused: {error}", fixture.pool);
}

/// The checkout dataset's own figures and the pool's allocated bytes.
fn figures(fixture: &Fixture) -> Figures {
    let checkout: &str = fixture.checkout.as_str();
    return Figures {
        used: fixture.bytes("used", checkout),
        dataset: fixture.bytes("usedbydataset", checkout),
        snapshots: fixture.bytes("usedbysnapshots", checkout),
        allocated: pool_allocated(fixture),
    };
}

/// The bytes the pool says it has allocated.
fn pool_allocated(fixture: &Fixture) -> u64 {
    let (_ok, said, _error): (bool, String, String) = super::zfs_sweep_guest::zfs(
        fixture.sbin.as_str(),
        "zpool",
        &["list", "-H", "-p", "-o", "allocated", fixture.pool.as_str()],
        None,
    );
    return said.trim().parse().unwrap_or(0);
}
