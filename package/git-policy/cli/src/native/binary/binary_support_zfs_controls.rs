//! What:
//!  The one place where a binary-level control states that it needs a ZFS fixture root,
//! and the ZFS fixture the step 3 arms run on:
//! a directory on a second dataset,
//! one under a different encryption key,
//! the kernel's block-cloning switch,
//! a mounted clone of the checkout dataset,
//! and the pool's block-clone counters.
//! Why:
//!  The container gate's fixture root is a tmpfs the container makes
//! (`bin/native-gate-fixture-root.ts`),
//! which is never a ZFS dataset,
//! so none of the dataset or block-cloning controls can run there.
//! A control there must not pass as coverage it does not have:
//! it fails with the filesystem it
//!  found unless the run states that the absence is known,
//! and then it prints one line naming itself and the reason it did not
//!  run.
//! The guest recipe of `package/zfs/request-to-openzfs-kernel-device`
//! (`e2e/README.md`, the step 3 arm) is the one place that sets the variables
//!  below,
//! and it runs the arms one at a time,
//! because the pool's block-clone counters count the whole pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // if (!onZfsOrAcknowledged(fixture, 'the_control')) return;
//! ```

use super::support::{Fixture, Observed, observe, wrapped};
use git_policy_cli::fs_kind::{FilesystemKind, filesystem_kind};
use git_policy_cli::fs_mount_options::read_mount_facts;
use std::path::{Path, PathBuf};

/// The hostname of the throwaway guest whose datasets the arms may change.
/// The guest recipe of `package/zfs/request-to-openzfs-kernel-device` sets it,
/// and the helper below refuses to let any arm touch a
///  ZFS path on this host,
/// whose pool is the repository's own.
pub const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_FIXTURE_GUEST";

/// The environment variable that acknowledges "this run has no ZFS fixture root".
/// The container gate image sets it,
/// and the btrfs host task sets it too,
/// so neither claims the ZFS coverage;
/// the guest recipe leaves it unset,
/// so a control there fails instead of
///  printing that it did not run.
pub const ZFS_ABSENT_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED";

/// The directory of the OpenZFS 2.4.4 userspace the guest builds:
/// `zfs` and `zpool`, which make the datasets,
/// the mounted clone and the counter readings.
pub const TOOL_DIRECTORY_VARIABLE: &str = "CLI_GIT_ZFS_TOOL_DIRECTORY";

/// A directory on a second dataset of the same pool,
/// under the same encryption key as the checkout dataset.
pub const SECOND_DATASET_VARIABLE: &str = "CLI_GIT_ZFS_SECOND_DATASET";

/// A directory on a dataset of the same pool that is an encryption root of its own,
/// so its key differs from the checkout dataset's.
pub const DIFFERENT_KEY_DATASET_VARIABLE: &str = "CLI_GIT_ZFS_DIFFERENT_KEY_DATASET";

/// The snapshot of the checkout dataset the controls clone,
/// as `pool/dataset@snapshot`.
pub const CLONE_SNAPSHOT_VARIABLE: &str = "CLI_GIT_ZFS_CLONE_SNAPSHOT";

/// What:
///  Whether the fixture root is on ZFS,
/// or `false` after one line naming the control and the filesystem it found.
/// Why:
///   A control that makes datasets or reads the pool's counters returns early on `false`,
/// and a run that never said it has no
///  ZFS fixture fails the control with both ways forward instead of reporting a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onZfsOrAcknowledged(fixture: Fixture, control: string): boolean;
/// ```
pub fn on_zfs_or_acknowledged(fixture: &Fixture, control: &str) -> bool {
    let kind: Result<FilesystemKind, _> = filesystem_kind(fixture.root.as_path());
    if let Some(named) = std::env::var_os(GUEST_VARIABLE) {
        // The guest recipe sets this, so a ZFS path may be changed: everything ZFS-changing runs in a throwaway guest and
        //  never on this host, whose pool holds the repository. A run against another machine is the one failure a
        //  variable can catch here.
        assert_eq!(machine_hostname(), named.to_string_lossy(), "the ZFS controls change datasets, so they run only in the guest {GUEST_VARIABLE} names; this machine is another one");
        assert_eq!(
            kind,
            Ok(FilesystemKind::Zfs),
            "the guest's fixture root {} must be on a ZFS dataset when {GUEST_VARIABLE} is set, and it is on {kind:?}",
            fixture.root.display()
        );
        return true;
    }
    let acknowledged: bool = std::env::var_os(ZFS_ABSENT_ACKNOWLEDGED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
    assert!(
        acknowledged,
        "the control {control} needs a ZFS fixture root in a throwaway guest, but {GUEST_VARIABLE} is unset and {} is on \
         {kind:?}; run the guest recipe of package/zfs/request-to-openzfs-kernel-device, or set \
         {ZFS_ABSENT_ACKNOWLEDGED_VARIABLE}=1 to state that this run has no ZFS fixture",
        fixture.root.display()
    );
    eprintln!("ZFS CONTROL NOT RUN: {control} (no guest is named by {GUEST_VARIABLE}, and fixture root {} is on {kind:?}; the guest recipe of package/zfs/request-to-openzfs-kernel-device runs it)", fixture.root.display());
    return false;
}

/// What:
///  The name this machine answers with.
/// Why:
///   The guest recipe names its guest,
/// and the arms may change a ZFS path only where that name matches,
/// so the name is read
///  rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hostname = readFile('/proc/sys/kernel/hostname').trim();
/// ```
fn machine_hostname() -> String {
    let read: String = std::fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_else(|error| { panic!("this machine's hostname cannot be read: {error}") });
    return read.trim().to_string();
}

/// What:
///  The directory one of the fixture's variables names.
/// Why:
///   A ZFS control that cannot find its second dataset must fail with the variable and the recipe that sets it,
/// never
///  run against a directory on the wrong dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureDirectory(variable: string): string; // throws when unset
/// ```
fn fixture_value(variable: &str) -> std::ffi::OsString {
    let value: Option<std::ffi::OsString> = std::env::var_os(variable);
    return value.unwrap_or_else(|| {
        panic!("the ZFS control needs {variable}, which the guest recipe of package/zfs/request-to-openzfs-kernel-device sets; no run of this control may guess a dataset");
    });
}

/// What:
///  The directory one of the fixture's variables names,
/// which must exist.
/// Why:
///   A variable that names no directory would let a control write to whatever path it holds,
/// so the absence fails
///  here with the variable's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureDirectory(variable: string): string;
/// ```
fn fixture_directory(variable: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(fixture_value(variable));
    assert!(directory.is_dir(), "{variable} names {}, which is not a directory", directory.display());
    return directory;
}

/// The directory on the second dataset of the fixture's pool.
pub fn second_dataset() -> PathBuf {
    return fixture_directory(SECOND_DATASET_VARIABLE);
}

/// The directory under the fixture pool's dataset that carries its own encryption key.
pub fn different_key_dataset() -> PathBuf {
    return fixture_directory(DIFFERENT_KEY_DATASET_VARIABLE);
}

/// The snapshot the controls clone into a mounted dataset of their own.
pub fn clone_snapshot() -> String {
    return fixture_value(CLONE_SNAPSHOT_VARIABLE).to_string_lossy().into_owned();
}

/// What:
///  Run one command of the OpenZFS userspace in the guest,
/// and answer with its standard output.
/// Why:
///   The recipe installs that userspace outside the system paths,
/// so the environment the recipe gives the suite is inherited here
/// (`LD_LIBRARY_PATH`),
///  and a
///  failed command must fail the control with what it printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTool(tool: string, args: string[]): string; // throws with stderr
/// ```
pub fn run_tool(tool: &str, arguments: &[String]) -> String {
    let directory: PathBuf = fixture_directory(TOOL_DIRECTORY_VARIABLE);
    let program: PathBuf = directory.join(tool);
    let output: std::process::Output = std::process::Command::new(program.as_path())
        .args(arguments)
        .output()
        .unwrap_or_else(|error| { panic!("{tool} could not be started from {}: {error}", directory.display()) });
    assert!(
        output.status.success(),
        "{tool} {} failed with {:?}: {}{}",
        arguments.join(" "),
        output.status.code(),
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// What:
///  The name of the pool that holds the fixture root's dataset.
/// Why:
///   The counters are pool properties and the recipe names the pool nowhere the control can read,
/// so the pool is
///  derived from the dataset the mount table reports for the fixture root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = mountSource(fixtureRoot).split('/')[0];
/// ```
pub fn pool_name(fixture: &Fixture) -> String {
    let source: Option<String> = read_mount_facts(fixture.root.as_path()).ok().flatten().map(|facts| { return facts.source; });
    let source: String = source.unwrap_or_else(|| { panic!("the mount table names no source for the fixture root {}", fixture.root.display()) });
    let pool: &str = source.split('/').next().unwrap_or(source.as_str());
    assert!(!pool.is_empty(), "the mount source {source} names no pool");
    return pool.to_string();
}

/// What:
///  What the pool reports about cloned blocks and about the space it holds,
/// at one moment.
/// Why:
///   A clone of an eligible file adds its blocks to `bcloneused` and its size to `bclonesaved`
/// without writing new
///  data,
/// a byte copy adds nothing to either,
/// and `allocated` is what any arm costs the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BlockCloneCounters = { bcloneused: number; bclonesaved: number; bcloneratio: string; allocated: number };
/// ```
#[derive(Clone, Debug)]
pub struct BlockCloneCounters {
    /// Bytes the pool holds as cloned blocks.
    pub bcloneused: u64,
    /// Bytes the pool would have used had those blocks not been cloned.
    pub bclonesaved: u64,
    /// The pool's own ratio of the two,
    /// as it prints it (`1.00x` before any clone).
    pub bcloneratio: String,
    /// Bytes the pool holds in all.
    pub allocated: u64,
}

/// What:
///  Read one property of the pool as a number.
/// Why:
///   `-p` prints the exact value instead of one rounded for people,
/// so two readings of one pool can be subtracted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = Number(zpool('get', '-H', '-p', '-o', 'value', property, pool));
/// ```
fn pool_number(pool: &str, property: &str) -> u64 {
    let printed: String = run_tool("zpool", &[String::from("get"), String::from("-H"), String::from("-p"), String::from("-o"), String::from("value"), String::from(property), String::from(pool)]);
    return printed.trim().parse::<u64>().unwrap_or_else(|error| { panic!("the pool {pool} reports {property} as {printed:?}, which is no number: {error}") });
}

/// What:
///  The pool's cloned-block counters and its allocated bytes.
/// Why:
///   The counters are per transaction group,
/// so a reading taken before the group that holds an arm's writes is committed reads the
///  pool as it was;
/// `zpool sync` commits that group first,
/// and then one reading of all four numbers says what the arm cost and
///  whether it shared blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = counters(fixture); /* the arm */ const after = counters(fixture);
/// ```
pub fn counters(fixture: &Fixture) -> BlockCloneCounters {
    let pool: String = pool_name(fixture);
    run_tool("zpool", &[String::from("sync"), pool.clone()]);
    let ratio: String = run_tool("zpool", &[String::from("get"), String::from("-H"), String::from("-o"), String::from("value"), String::from("bcloneratio"), pool.clone()]);
    return BlockCloneCounters {
        bcloneused: pool_number(pool.as_str(), "bcloneused"),
        bclonesaved: pool_number(pool.as_str(), "bclonesaved"),
        bcloneratio: ratio.trim().to_string(),
        allocated: pool_number(pool.as_str(), "allocated"),
    };
}

/// What:
///  The kernel's block-cloning switch,
/// turned off for one arm and put back when the arm ends however it ends.
/// Why:
///   Block cloning is one module parameter in OpenZFS 2.4.4 and no dataset property,
/// so the arm that must end in Git's checkout turns the pool's
///  cloning off,
/// and a panic must not leave it off for the arms after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const switch = blockCloningOff(); // writes 0
/// // ... the arm ...
/// switch.restore(); // writes 1 again
/// ```
pub struct BlockCloningSwitch {
    /// The value the parameter held before the arm,
    /// which the switch puts back.
    previous: String,
}

/// Where the kernel exposes the block-cloning switch.
const BLOCK_CLONING_PARAMETER: &str = "/sys/module/zfs/parameters/zfs_bclone_enabled";

/// What:
///  Turn the kernel's block cloning off and answer with the switch that puts it back.
/// Why:
///   The arm that must end in Git's checkout needs `FICLONE` to fail,
/// and the pool's counters only show a clone that happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockCloningOff(): BlockCloningSwitch; // throws unless the switch was on
/// ```
pub fn block_cloning_off() -> BlockCloningSwitch {
    let previous: String = std::fs::read_to_string(BLOCK_CLONING_PARAMETER).unwrap_or_else(|error| { panic!("the block-cloning switch {BLOCK_CLONING_PARAMETER} cannot be read: {error}") });
    assert_eq!(previous.trim(), "1", "the arm needs block cloning on before it turns it off, and {BLOCK_CLONING_PARAMETER} holds {previous:?}");
    std::fs::write(BLOCK_CLONING_PARAMETER, b"0\n").unwrap_or_else(|error| { panic!("the block-cloning switch {BLOCK_CLONING_PARAMETER} cannot be written: {error}") });
    return BlockCloningSwitch { previous: previous.trim().to_string() };
}

impl Drop for BlockCloningSwitch {
    fn drop(&mut self) {
        std::fs::write(BLOCK_CLONING_PARAMETER, format!("{}\n", self.previous).as_bytes())
            .unwrap_or_else(|error| { panic!("the block-cloning switch {BLOCK_CLONING_PARAMETER} cannot be put back to {}: {error}", self.previous) });
    }
}

/// What:
///  A mounted clone of the fixture's snapshot,
/// mounted over `slot`,
/// and the dataset name it was given.
/// Why:
///   A mounted clone is a dataset of its own,
/// which is what rung 2 must refuse both as the destination of an add
/// and as the source of
///  a slot in the checkout dataset;
/// the mount over the slot is what Git and the wrapper meet there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mount_cloneOver(fixture: Fixture, slot: string, name: string): string;
/// ```
pub fn mount_clone_over(fixture: &Fixture, slot: &Path, name: &str) -> String {
    let dataset: String = format!("{}/{}", pool_name(fixture), name);
    std::fs::create_dir_all(slot).expect("the slot of a mounted clone");
    run_tool(
        "zfs",
        &[String::from("clone"), String::from("-o"), format!("mountpoint={}", slot.display()), clone_snapshot(), dataset.clone()],
    );
    return dataset;
}

/// What:
///  Destroy a dataset a control made,
/// its snapshots included.
/// Why:
///   The fixture's own removal cannot remove a mounted dataset's files,
/// and a clone left mounted would keep the pool's counters
///  and the next arm's readings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// zfs(['destroy', '--recursive', dataset]);
/// ```
pub fn destroy_dataset(dataset: &str) {
    // `zfs destroy` in OpenZFS 2.4.4 takes no long form of this flag, so the short one is what it meets.
    run_tool("zfs", &[String::from("destroy"), String::from("-r"), String::from(dataset)]);
}

/// What:
///  Run a wrapped command with its diagnostic log on standard error,
/// so a control can read which rung served the add and why it did not.
/// Why:
///   The refused rungs are silent to the person by design (nothing they do changes a dataset),
/// so the reason is only in the
///  diagnostic log,
/// and that is what a refusal control asserts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // MONOCHROMATIC_VERBOSE=true git worktree add ../topic
/// ```
pub fn run_wrapped_verbose(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).env("MONOCHROMATIC_VERBOSE", "true").current_dir(directory).args(arguments), b"");
}
