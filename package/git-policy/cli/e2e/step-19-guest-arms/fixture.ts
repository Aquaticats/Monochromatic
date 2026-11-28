/**
 What plan step 19's guest run names in one place: the crate's folder inside the guest, the pools and datasets each arm
 builds, the paths on the guest's own disks, the environment each Rust arm reads, the lines a pulled log must carry, and
 the bound every step gets.

 The layout is the design's own: a checkout dataset mounted at the checkout path, the unmounted clone container beside it
 with `mountpoint=legacy`, `canmount=off`, `setuid=off` and `devices=off`, and one dataset whose mountpoint lies inside the
 checkout. Everything lives under one fixture folder on the guest's root disk, because a reboot has to keep it.

 @example
 ```ts
 const { checkoutPath, pool } = fixture;
 ```
 */

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
export const crateDirectory = '/opt/git-policy-cli-step19';

/**
 The environment variable every arm of this driver reads, so an arm that reached a person's machine refuses instead of
 writing `/etc/systemd/system` there.
 */
export const guestVariable = 'CLI_GIT_ZFS_STEP19_GUEST';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsPath: string =
  '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin';

/**
 The folder the guest's OpenZFS 2.4.4 userspace answers from, which the Rust arm that clones a dataset starts `zfs` by.
 */
export const openZfsSbin = '/opt/openzfs-2.4.4/sbin';

/**
 The library folder the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsLibrary = '/opt/openzfs-2.4.4/lib';

/**
 The folder OpenZFS 2.4.4 was built in, whose `etc/systemd/system` holds the provider text a boot needs.
 */
export const openZfsSource = '/root/zfs-2.4.4';

/**
 The folder every arm of this run builds inside the guest's own root disk, which a reboot keeps.
 */
export const fixtureRoot = '/home/zfs-step19-fixture';

/**
 The pool the boot arms build, whose image is a file on the guest's root disk.
 */
export const pool = 'zfsstep19';

/**
 The checkout dataset, mounted at the checkout path.
 */
export const checkoutDataset: string = `${pool}/ROOT/home/monochromatic-git-checkout`;

/**
 The unmounted clone container beside it, which every clone inherits `mountpoint=legacy` from.
 */
export const containerDataset: string =
  `${pool}/ROOT/home/monochromatic-git-worktree-clone-datasets`;

/**
 The dataset whose mountpoint lies inside the checkout, which one unit of the plan mounts after the gate.
 */
export const nestedDataset: string = `${pool}/ROOT/home/vm-builder-output`;

/**
 The checkout path the pool serves, whose dashes the mount unit's name escapes.
 */
export const checkoutPath: string = `${fixtureRoot}/home/monochromatic-git-checkout`;

/**
 The folder the checkout path falls through to while the checkout dataset is not mounted, which stands for the home
 underlay the immutable flag is set on.
 */
export const underlay: string = `${fixtureRoot}/home`;

/**
 The nested dataset's mountpoint, inside the checkout.
 */
export const nestedMountpoint: string = `${checkoutPath}/nested-output`;

/**
 The short word the nested dataset's unit name carries.
 */
export const nestedSlug: string = 'output';

/**
 The slot one managed clone stands at.
 */
export const slot: string = 'one';

/**
 The clone container's requests a boot leaves behind, and the first attach's record the sweep files.
 */
export const zedHistoryEventName: string = 'history_event-mount_git_worktree_clone_datasets';

/**
 The root-owned program every unit line starts, which the runbook installs from the release build.
 */
export const installedProgram: string =
  `/usr/libexec/monochromatic-git-policy-cli/${zedHistoryEventName}`;

/**
 The release binary inside the guest's build.
 */
export const releaseBinary: string = `${crateDirectory}/package/git-policy/cli/target/release/monochromatic-git-policy-cli`;

/**
 The library test binary's own working folder, which the crate's own tests run from.
 */
export const cratePackage: string = `${crateDirectory}/package/git-policy/cli`;

/**
 The variable the units' and the boot arm's tests read, so an arm that ran elsewhere printed `SKIPPED` and the driver
 refuses a log that lacks the line it expects.
 */
export const bootTest: string = 'checkout_boot_units::guest_tests::the_plan_parses_and_reports_the_owner_s_order_in_a_guest';

/**
 The boot arm's test, whose phases a reboot splits.
 */
export const rebootTest: string =
  'checkout_boot_units::guest_tests::reboot_tests::the_plan_the_setup_writes_orders_a_real_boot_in_a_guest';

/**
 The immutable entry's arm.
 */
export const underlayTest: string =
  'checkout_underlay_entry::underlay_entry_verdict::guest_tests::the_verdict_reads_a_real_immutable_folder_in_a_guest';

/**
 The lines a run must find in the units' own log, one per claim the arm printed.
 */
export const expectedUnitMarkers: readonly string[] = [
  'STEP19-GUEST verified=/etc/systemd/system/reconcile-git-worktree-clone-mounts.service',
  'STEP19-GUEST propagation-pair-offline-limit=',
  'STEP19-GUEST verified=/etc/systemd/system/checkout-propagation.service',
  'STEP19-GUEST verified=/etc/systemd/system/git-worktree-nested-output-mount.service',
  'STEP19-GUEST gate-orders-and-bounds-read',
  'STEP19-GUEST drop-ins-read',
  'STEP19-GUEST nested-unit-orders-after-the-gate',
  'STEP19-GUEST plan-written-parsed-and-removed',
];

/**
 The lines the immutable entry's arm must print.
 */
export const expectedUnderlayMarkers: readonly string[] = [
  'STEP19-UNDERLAY plain verdict=unprotected same-device-as-parent=true immutable=false attributes-reported=true',
  'STEP19-UNDERLAY flagged verdict=protected same-device-as-parent=true immutable=true attributes-reported=true',
  'STEP19-UNDERLAY mounted verdict=unprotected same-device-as-parent=false',
  'STEP19-UNDERLAY unmounted-again verdict=protected',
  'STEP19-UNDERLAY other-owner verdict=unprotected',
  'STEP19-UNDERLAY rolled-back verdict=unprotected',
  'STEP19-UNDERLAY done',
];

/**
 The lines the `snapdir` arm must print: the value set on both datasets and the clone that inherited it.
 */
export const expectedSnapdirMarkers: readonly string[] = [
  'SNAPDIR-INHERITANCE checkout=disabled local',
  'SNAPDIR-INHERITANCE container=disabled local',
  'SNAPDIR-INHERITANCE clone=disabled inherited-from=',
  'SNAPDIR-INHERITANCE clone-hides-the-directory=true',
  'SNAPDIR-INHERITANCE done',
];

/**
 The lines a healthy boot prints after its reboot.
 */
export const expectedHealthyBootMarkers: readonly string[] = [
  'STEP19-BOOT healthy-boot read',
  'STEP19-BOOT clone-attached=',
];

/**
 The lines a boot that failed import, key load or the checkout mount prints.
 */
export const expectedFailedBootMarkers: readonly string[] = ['STEP19-BOOT failed-boot read'];

/**
 Milliseconds the guest's build of the test target and the release binary may take.
 */
export const crateBuildTimeoutMilliseconds: number = 2_700_000;

/**
 Milliseconds one guest arm may take.
 */
export const armTimeoutMilliseconds: number = 900_000;

/**
 Milliseconds the guest's boot arms get beyond their own steps, which a reboot's own minute needs.
 */
export const rebootTimeoutMilliseconds: number = 600_000;

/**
 Milliseconds the host gives a detached script past its own bound: the pull after it costs the answer's own seconds.
 */
export const finishMarginMilliseconds: number = 180_000;

/**
 The migration's own pool, its image and the checkout its home dataset holds a tree at, which the runbook moves.
 */
export const migrationPool = 'zfsstep19mig';

/**
 The file-backed image the migration's pool stands on.
 */
export const migrationImage: string = `${fixtureRoot}/migration.img`;

/**
 The checkout path the migration's home dataset holds its tree at.
 */
export const migrationCheckout = '/home/zfs-step19-migration/Monochromatic';

/**
 The folder the migration mounts the checkout dataset at while it is filled and verified, which the runbook leaves to
 the operator and this run names.
 */
export const migrationStaging = '/mnt/checkout-migration';

/**
 The snapshot the migration keeps until every check passes, whose name carries the day it was taken.
 */
export const migrationSnapshotPrefix: string = 'before-checkout-migration-';
