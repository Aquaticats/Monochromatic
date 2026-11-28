/**
 The three arms plan step 19 wrote and never ran, each run inside a guest and each judged by the lines it printed:

 - the units, written where systemd reads them, parsed by `systemd-analyze verify` and read back from the guest's own
   manager;
 - the immutable covered entry, read against a real folder that carries the kernel's flag on a real dataset, with the
   account's own attempts to move and remove it;
 - `snapdir=disabled`, set on the checkout and on the clone container, and the clone that inherits it.

 Every one of them is a library test of this crate, run by its exact name, and a run that printed no line of its own is a
 failed run: a test binary that skipped prints `SKIPPED` and exits zero, so the driver reads the log rather than the
 status (`AGENTS.md` rule `TRV`).

 @example
 ```ts
 await theUnitsReadBack({ chosen });
 ```
 */

import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';

import {
  installThePlan,
  removeThePlan,
} from './boot.ts';
import {
  runOneGuestTest,
} from './guest-test-run.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';

import {
  armTimeoutMilliseconds,
  checkoutPath,
  containerDataset,
  expectedSnapdirMarkers,
  expectedUnderlayMarkers,
  expectedUnitMarkers,
  fixtureRoot,
  keyUnit,
  nestedDataset,
  nestedMountpoint,
  nestedSlug,
  openZfsLibrary,
  openZfsPath,
  pool,
  underlayEntryFolder,
  underlayTest,
  bootTest,
} from './fixture.ts';

/**
 The units' arm: the plan written at the paths systemd reads, every unit file parsed by the guest's own
 `systemd-analyze verify`, and each unit's ordering, bounds and drop-ins read back from the guest's manager.

 @param chosen - the run's guest and log folder

 @returns the arm's own log

 @throws GuestDriverError naming the marker a pulled log lacks

 @example
 ```ts
 const pulled = await theUnitsReadBack({ chosen });
 ```
 */
export async function theUnitsReadBack({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm's own log, which the markers are read from.
   */
  const pulled = await runOneGuestTest({
    chosen,
    test: bootTest,
    environment: [
      `export CLI_GIT_ZFS_STEP19_CHECKOUT=${checkoutPath}`,
      `export CLI_GIT_ZFS_STEP19_NESTED=${nestedDataset}`,
      `export CLI_GIT_ZFS_STEP19_NESTED_MOUNTPOINT=${nestedMountpoint}`,
      `export CLI_GIT_ZFS_STEP19_NESTED_SLUG=${nestedSlug}`,
      `export CLI_GIT_ZFS_STEP19_KEY_UNIT=${keyUnit}`,
    ],
    log: 'guest-step19-units.log',
  },);
  requireMarkers({
    text: pulled,
    markers: expectedUnitMarkers,
    subject: 'the pulled units log',
    because: 'the guest ran the arm without proving what it is named for',
  },);
  return pulled;
}

/**
 The immutable entry's arm: the verdict against a real folder carrying the kernel's flag, a real mount over the same
 path, an entry another account owns, and the flag a rollback loses.

 @param chosen - the run's guest and log folder

 @returns the arm's own log

 @throws GuestDriverError naming the marker a pulled log lacks

 @example
 ```ts
 const pulled = await theImmutableEntry({ chosen });
 ```
 */
export async function theImmutableEntry({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  // The arm's own fixture, left as the last run left it: a run that stopped before its last line keeps the flag on the
  // entry, so the next reading would start from a protected folder.
  await guest({
    chosen,
    script: [
      'set -eu',
      `mkdir --parents ${underlayEntryFolder}`,
      `umount ${underlayEntryFolder}/monochromatic-git-checkout 2>/dev/null || true`,
      `chattr -i ${underlayEntryFolder}/monochromatic-git-checkout 2>/dev/null || true`,
      `chmod 0755 ${underlayEntryFolder}/monochromatic-git-checkout 2>/dev/null || true`,
      `rm --recursive --force ${underlayEntryFolder}/monochromatic-git-checkout`,
      'echo "STEP19-UNDERLAY-PRELUDE clean"',
    ].join('\n',),
    log: 'guest-step19-underlay-prelude.log',
    timeoutMilliseconds: 300_000,
  },);
  /**
   The arm's own log, which the markers are read from.
   */
  const pulled = await runOneGuestTest({
    chosen,
    test: underlayTest,
    environment: [
      `export CLI_GIT_ZFS_STEP19_UNDERLAY=${underlayEntryFolder}`,
    ],
    log: 'guest-step19-underlay.log',
  },);
  requireMarkers({
    text: pulled,
    markers: expectedUnderlayMarkers,
    subject: 'the pulled immutable entry log',
    because: 'the guest ran the arm without reading a real immutable folder',
  },);
  return pulled;
}

/**
 What:
  Set `snapdir=disabled` on the checkout dataset and on the clone container, and read the value back off a clone of the
  checkout's snapshot.
 Why:
  the plan's third answer of 2026-11-23 hides the snapshot directories on both datasets, and a clone inherits the
  container's value;
  the runbook listed the set and the inheritance as never repeated through its own page, and this arm is that
  repetition.
  It is a shell arm rather than a library test because the answer is the property's own `sourced` field, which only
  `zfs get` prints.

 @param chosen - the run's guest and log folder

 @returns the arm's own log

 @throws GuestDriverError naming the marker a pulled log lacks

 @example
 ```ts
 const pulled = await theSnapdirInheritance({ chosen });
 ```
 */
export async function theSnapdirInheritance({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The property set on both datasets, and the clone whose value names where it came from.
   */
  const script = [
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsLibrary}`,
    'set -eu',
    // Both datasets carry the value the project chose, and each says it set it itself.
    `zfs set snapdir=disabled ${pool}/ROOT/home/monochromatic-git-checkout`,
    `zfs set snapdir=disabled ${containerDataset}`,
    `checkout_value=$(zfs get -H -o value snapdir ${pool}/ROOT/home/monochromatic-git-checkout)`,
    `checkout_source=$(zfs get -H -o source snapdir ${pool}/ROOT/home/monochromatic-git-checkout)`,
    `container_value=$(zfs get -H -o value snapdir ${containerDataset})`,
    `container_source=$(zfs get -H -o source snapdir ${containerDataset})`,
    'echo "SNAPDIR-INHERITANCE checkout=$checkout_value $checkout_source"',
    'echo "SNAPDIR-INHERITANCE container=$container_value $container_source"',
    // A clone of the checkout's snapshot, made in the container, answers the container's value and names the container
    // as where it came from.
    // A run that stopped inside this arm leaves the clone and its snapshot behind, and a second snapshot of one name is
    // a refusal.
    `zfs destroy -r -f ${containerDataset}/snapdir-arm || true`,
    `zfs destroy ${pool}/ROOT/home/monochromatic-git-checkout@snapdir-arm || true`,
    `zfs snapshot ${pool}/ROOT/home/monochromatic-git-checkout@snapdir-arm`,
    `zfs clone ${pool}/ROOT/home/monochromatic-git-checkout@snapdir-arm ${containerDataset}/snapdir-arm`,
    `zfs set mountpoint=${fixtureRoot}/snapdir-clone ${containerDataset}/snapdir-arm`,
    `clone_value=$(zfs get -H -o value snapdir ${containerDataset}/snapdir-arm)`,
    `clone_source=$(zfs get -H -o source snapdir ${containerDataset}/snapdir-arm)`,
    'echo "SNAPDIR-INHERITANCE clone=$clone_value inherited-from=$clone_source"',
    // The value's own meaning, read the way step 8's snapdir arm reads it: `disabled` refuses the directory's name,
    // from a fresh mount, because a dentry the kernel already resolved stays reachable.
    `mountpoint=$(zfs get -H -o value mountpoint ${containerDataset}/snapdir-arm)`,
    // `zfs set mountpoint` already mounted it, so the listing starts from the mount the value is in force on.
    `mount --make-private "$mountpoint"`,
    `zfs snapshot ${containerDataset}/snapdir-arm@snapdir-arm`,
    `if ls -a "$mountpoint" | grep --quiet --extended-regexp "^[.]zfs$"; then listed=true; else listed=false; fi`,
    `if ls "$mountpoint/.zfs/snapshot/snapdir-arm" > /dev/null 2>&1; then entered=true; else entered=false; fi`,
    'test "$listed" = false',
    'test "$entered" = false',
    'echo "SNAPDIR-INHERITANCE clone-hides-the-directory=true"',
    `zfs unmount ${containerDataset}/snapdir-arm`,
    `zfs destroy -r ${containerDataset}/snapdir-arm`,
    `zfs destroy ${pool}/ROOT/home/monochromatic-git-checkout@snapdir-arm`,
    'echo "SNAPDIR-INHERITANCE done"',
  ].join('\n',);
  /**
   The arm's own log, which the markers are read from.
   */
  const pulled = await guest({
    chosen,
    script,
    log: 'guest-step19-snapdir.log',
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
  requireMarkers({
    text: pulled,
    markers: expectedSnapdirMarkers,
    subject: 'the pulled snapdir log',
    because: 'the guest ran the arm without setting the value or reading an inheritance',
  },);
  return pulled;
}
/**
 What:
  The two refusals a real manager's offline verifier gives this plan, captured from the plan's own files.
 Why:
  the units' arm was written to hand every file of the plan to `systemd-analyze verify`, and that verifier refuses a
  drop-in path and cannot resolve a mount unit the manager learns from the mount table.
  The arm was changed to read the drop-ins back from the manager instead, and this step keeps the refusals themselves as
  the evidence for that change, so a reader compares a real capture rather than a memory of one.
  It writes the plan, asks for the two refusals, and takes the plan away again, which leaves the guest as it found it.

 @param chosen - the run's guest and log folder

 @returns the step's own log, which holds both refusals

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 const pulled = await theRefusals({ chosen });
 ```
 */
export async function theRefusals({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  await installThePlan({ chosen, },);
  /**
   The two questions a person would ask the offline verifier, asked of this plan's own files.
   */
  const script = [
    'set -eu',
    'echo "STEP19-REFUSAL drop-in:"',
    'systemd-analyze verify /etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf 2>&1 || true',
    'echo "STEP19-REFUSAL propagation-service:"',
    'systemd-analyze verify /etc/systemd/system/checkout-propagation.service 2>&1 || true',
  ].join('\n',);
  /**
   The step's own log, which holds both refusals.
   */
  const pulled = await guest({
    chosen,
    script,
    log: 'guest-step19-refusals.log',
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
  await removeThePlan({ chosen, },);
  return pulled;
}
