/**
 Give the store's path a dataset of the guest's own pool, and run the store's
 publication cost against a dataset beside it.

 The guest's own `/var/lib` is on its ext4 root, which no ZFS property reaches, so
 the store's path is given a dataset of the guest's own pool. That is what settles
 the design's assumption about the store's dataset: a hard link, an unlinking, a
 rename and a folder `fsync` on a ZFS dataset.

 The probe is pushed rather than snapshotted, because it lives in the package's
 `doc/` folder and the snapshot holds only what a build reads. It is run in the
 guest's own `git-policy-cli-store/probe` dataset, which lies on the same pool as
 the store and beside the store's own dataset rather than on it.

 @example
 ```ts
 await storeDataset({ chosen });
 ```
 */

import {
  join,
  resolve,
} from 'node:path';

import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import {
  type GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  host,
} from '../../bin/mvm-guest-host-command.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';
import {
  readLog,
} from '../../bin/mvm-guest-step-command.ts';




import {
  armTimeoutMilliseconds,
  expectedFsyncMarkers,
  finishMarginMilliseconds,
  fsyncFolder,
  fsyncRoundFolder,
  fsyncRounds,
  openZfsPath,
  probeDataset,
  storeDataset as storeDatasetName,
  storePath,
  storePool,
  storePoolImage,
} from './adoption-fixture.ts';
import { detachedStep, } from './detached-step.ts';

/**
 The store's own dataset-creating step log.
 */
const storeDatasetLog = 'guest-store-dataset.log';

/**
 The log the publication-fsync step writes inside the guest's shared folder.
 */
const fsyncStepLog = 'guest-store-fsync.log';

/**
 The publication-fsync probe's own log, which the step writes and the host pulls.
 */
const fsyncLog = 'store-fsync.log';

/**
 The path of the probe inside the guest, which `mvm push` lands under its own
 name.
 */
const probeInGuest = '/mnt/shared/probe-reaches-storage.py';

/**
 Make the ZFS pool and the two datasets the store's arms write on.

 @param chosen - the run's guest and log folder

 @throws Error when any of the pool's commands refused

 @example
 ```ts
 await storeDataset({ chosen });
 ```
 */
export async function storeDataset({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The pool, the store's dataset and the probe's dataset, each created only when
   the guest does not already hold it.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `mkdir --parents "$(dirname ${storePoolImage})"`,
    `if ! zpool list -H -o name | grep --quiet '^${storePool}$'; then truncate --size 1G ${storePoolImage}; zpool create -f ${storePool} ${storePoolImage}; fi`,
    `zfs list -H -o name ${storeDatasetName} > /dev/null 2>&1 || zfs create ${storeDatasetName}`,
    `zfs set mountpoint=${storePath} ${storeDatasetName}`,
    `zfs list -H -o name ${probeDataset} > /dev/null 2>&1 || zfs create ${probeDataset}`,
    `zfs set mountpoint=${fsyncFolder} ${probeDataset}`,
    `chmod 0755 ${storePath} ${fsyncFolder}`,
    `findmnt --target ${storePath} --output SOURCE --noheadings --first-only`,
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: storeDatasetLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}

/**
 Run the store's publication cost against a dataset of the store's own pool,
 inside the same guest.

 @param chosen - the run's guest and log folder

 @returns the probe's log, pulled into the run's log folder

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await publicationFsync({ chosen });
 ```
 */
export async function publicationFsync({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The probe itself, in the package's own `doc` folder, which this arms folder
   sits two levels below.
   */
  const probe = resolve(
    import.meta.dirname,
    '..',
    '..',
    'doc',
    'slot-identity-store-write-cost',
    'probe-reaches-storage.py',
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      probe,
      '/root/probe-reaches-storage.py',
    ],
    log: 'guest-push-probe.log',
  },);
  /**
   The probe's run: its folder inside the store's pool, and the pool's own device
   so the counters it reads are the ones the pool's writes reach.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    // The probe reads its folder's dataset before it creates anything, and it
    // removes and recreates its own folder, so it is given a folder inside the
    // dataset rather than the dataset's mount point.
    `mkdir --parents ${fsyncRoundFolder}`,
    // The block device the pool's own file-backed vdev lives on, so the counters
    // the probe reads are the ones the pool's writes reach; the probe reads the
    // pool's own ZIL counters from its kstat folder.
    `pool_device=$(findmnt --target ${storePoolImage} --output SOURCE --noheadings --first-only)`,
    'test -n "$pool_device"',
    'pool_disk=$(lsblk --noheadings --output PKNAME --paths "$pool_device")',
    `python3 -I ${probeInGuest} ${fsyncRoundFolder} ${fsyncRounds} --pool-device "$(basename "$pool_device")" --pool-disk "$(basename "$pool_disk")" > /mnt/shared/${fsyncLog} 2>&1`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: fsyncStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
    finishTimeoutMilliseconds: armTimeoutMilliseconds + finishMarginMilliseconds,
  },);
  /**
   The probe's own log, which its step wrote beside the step's log.
   */
  const pulledPath = join(
    chosen.logDir,
    fsyncLog,
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      `/mnt/shared/${fsyncLog}`,
      pulledPath,
    ],
    log: 'host-pull-store-fsync.log',
  },);
  /**
   The probe's answers, as `mvm` copied them out.
   */
  const pulled = await readLog({ path: pulledPath, },);
  requireMarkers({
    text: pulled,
    markers: expectedFsyncMarkers,
    subject: 'the pulled publication-fsync log',
    because: 'the guest ran the probe without that arm',
  },);
  return pulled;
}
