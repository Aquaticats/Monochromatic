/**
 What the adoption guest's arms name: the crate's folder inside the guest, the
 store's path and the pool it is on, the pools and checkouts each arm builds, and
 the bound each step gets.

 The store's arms write `/var/lib/monochromatic-git-policy-cli`, which the guest's
 own ext4 root cannot carry a ZFS property for, so the driver gives that path a
 dataset of the guest's own pool: a hard link, an unlinking, a rename and a
 folder `fsync` on a ZFS dataset are what the design's assumption rests on.

 @example
 ```ts
 const { name, logDir } = options();
 ```
 */

/**
 Where the crate is unpacked and built inside the guest, which every account may
 traverse.
 */
export const crateDirectory = '/opt/git-policy-cli-adoption';

/**
 The environment variable the adoption arm reads, so it never writes `/var/lib` on
 a machine somebody uses.
 */
export const adoptionGuestVariable = 'CLI_GIT_ZFS_ADOPTION_GUEST';

/**
 The environment the sweep arm reads.
 */
export const sweepGuestVariable = 'CLI_GIT_ZFS_SWEEP_GUEST';

/**
 The environment the holder-scan arm reads.
 */
export const scanGuestVariable = 'CLI_GIT_ZFS_SCAN_GUEST';

/**
 The environment the unmount arm reads.
 */
export const unmountGuestVariable = 'CLI_GIT_ZFS_UNMOUNT_GUEST';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsPath: string = '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';

/**
 The `sbin` folder the guest's OpenZFS 2.4.4 tools live in, which the arms that
 name a tool's folder read.
 */
export const openZfsSbin = '/opt/openzfs-2.4.4/sbin';

/**
 The store's path inside the guest, which the arms write.
 */
export const storePath = '/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities';

/**
 Where the store's publication-fsync arm writes inside the guest, on a dataset of
 the store's own pool.
 */
export const fsyncFolder = '/var/lib/monochromatic-git-policy-cli-probe';

/**
 The folder inside the probe's dataset that the probe creates and removes for each
 run.
 */
export const fsyncRoundFolder: string = `${fsyncFolder}/round`;

/**
 The pool the guest makes so the store is on a ZFS dataset.
 */
export const storePool = 'git-policy-cli-store';

/**
 The file-backed vdev the store's pool stands on.
 */
export const storePoolImage = '/home/git-policy-cli-store-fixture/store.img';

/**
 The dataset the store's path is mounted from.
 */
export const storeDataset: string = `${storePool}/store`;

/**
 The dataset beside the store's own, where the publication-fsync probe runs.
 */
export const probeDataset: string = `${storePool}/probe`;

/**
 The checkout the holder scan's arms build their own design layout inside, under
 the guest's own root.
 */
export const scanCheckout = '/root/holder-scan-checkout';

/**
 The checkout the sweep arms walk inside the guest, under the guest's own root.
 */
export const sweepCheckout = '/root/sweep-checkout';

/**
 The second checkout the stale sub-case mounts its own layout at, which a run that
 stopped inside it leaves standing.
 */
export const sweepStaleCheckout = '/root/sweep-stale-checkout';

/**
 The pool the delegation arm makes.
 */
export const delegationPool = 'git-policy-cli-delegation';

/**
 The image the delegation arm's pool stands on.
 */
export const delegationImage = '/home/git-policy-cli-delegation.img';

/**
 The checkout the delegation arm mounts its own dataset at.
 */
export const delegationCheckout = '/home/delegation-checkout';

/**
 The account the guest image creates, which the design delegates to.
 */
export const delegationAccount = 'ubuntu';

/**
 Seconds the publication-fsync arm runs, which is the record's own count of
 rounds.
 */
export const fsyncRounds = '60';

/**
 Milliseconds the guest's build of the test target may take.
 */
export const crateBuildTimeoutMilliseconds = 2_400_000;

/**
 Milliseconds one guest arm may take.
 */
export const armTimeoutMilliseconds = 900_000;

/**
 Milliseconds the host gives a detached script past its own bound: the pull after
 it costs the answer's own seconds.
 */
export const finishMarginMilliseconds = 120_000;

/**
 Milliseconds the host gives the crate's build past its own bound: the pull of a
 log this size, and the guest's own `timeout` margin, cost more than one arm's.
 */
export const buildFinishMarginMilliseconds = 180_000;

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
export const expectedMarkers: readonly string[] = [
  'ADOPTION recorded device=',
  'ADOPTION refused=wrong-device',
  'ADOPTION refused=link',
  'ADOPTION refused=mode-widened',
  'ADOPTION refused=slot-differs',
  'ADOPTION done',
  'STORE published=measured folder-mode=0755 record-mode=0644',
  'STORE replaced-parent=one-rename',
  'STORE killed-writer staging-removed=true',
];

/**
 The lines a run must find in the guest's `snapdir` log, one per value of the
 property.
 */
export const expectedSnapdirMarkers: readonly string[] = [
  'SNAPDIR visible listed=true entered=true mounts-grown=true',
  'SNAPDIR hidden listed=false entered=true mounts-grown=true',
  'SNAPDIR disabled listed=false entered=false mounts-grown=false',
  'SNAPDIR account-cannot-set-it-back=true',
  'SNAPDIR release has-disabled=true',
  'SNAPDIR done',
];

/**
 The lines a run must find in the guest's delegation log, one per attempt the
 design's grants allow or refuse.
 */
export const expectedDelegationMarkers: readonly string[] = [
  'DELEGATION create-under-container=allowed',
  'DELEGATION create-with-mountpoint=refused',
  'DELEGATION snapshot-of-checkout=allowed',
  'DELEGATION clone-named-the-leaf=allowed',
  'DELEGATION rename-onto-the-leaf=refused',
  'DELEGATION mount-as-account=refused',
  'DELEGATION set-mountpoint-as-account=refused',
  'DELEGATION set-canmount-as-account=refused',
  'DELEGATION create-pool-as-account=refused',
  'DELEGATION import-pool-as-account=refused',
  'DELEGATION root-mount-a=mounted-nothing-of-the-accounts',
  'DELEGATION done',
];

/**
 The lines a run must find in the guest's sweep-arm log, one per arm plan step 8
 runs.
 */
export const expectedSweepMarkers: readonly string[] = [
  'SWEEP attached item=1 slot=one origin=checkout-snapshot',
  'SWEEP first-attach outcome=attached slot=one',
  'SWEEP twice-swept receipt=done',
  'SWEEP second-request answered=done mounts=1',
  'SWEEP refusal item=5 step=key-unavailable receipt=refused',
  'SWEEP attached item=5 slot=two',
  'SWEEP refusal item=2 step=intent',
  'SWEEP nothing inherited=true',
  'SWEEP refusal item=2 step=clone-origin',
  'SWEEP refusal item=2 step=clone-origin-foreign',
  'SWEEP rollback unmount=EBUSY',
  'SWEEP rollback unmount=ok',
  'SWEEP bound stopped=true',
  'SWEEP stale flags-changed=refused',
  'SWEEP stale replaced=refused',
  'SWEEP stale swapped=refused',
  'SWEEP no-configuration shape=no-file attached slot=one',
  'SWEEP no-configuration shape=empty-object attached slot=one',
  'SWEEP no-configuration shape=only-start-probe attached slot=one',
  'SWEEP no-configuration shape=dataset-names attached slot=one',
  'SWEEP no-configuration several=two-pools refused slot=one mounted=false',
  'STOPPAGE account-clone attached slot=monochromatic-git-checkout container=git-policy-cli-store/account-clones',
  'STOPPAGE probe passed container=git-policy-cli-store/account-clones',
  'PROBE listing several names-every-row=true container=git-policy-cli-store/account-clones',
  'PROBE listing none names-what-was-found=true',
  'STOPPAGE cleanup done container=git-policy-cli-store/account-clones',
  'SNAPDIR-REPLY local-visible=1 inherited-disabled=2 default=absent',
  'SWEEP done',
];

/**
 The lines a run must find in the guest's holder-scan log, one per arm plan step 9
 runs.
 */
export const expectedScanMarkers: readonly string[] = [
  'SCAN baseline own=1 holders=0',
  'SCAN HOLDER other-namespace=1',
  'SCAN H1 intent=mount holders=',
  'SCAN H1 intent=unmount holders=',
  'SCAN nonces differ=true',
  'SCAN unreadable-link',
  'SCAN thread-namespace found=',
  'SCAN leader-ended found=',
  'SCAN chrooted-lowest found=',
  'SCAN confined-consumer=',
  'SCAN churn scans=20',
  'SCAN detached-mount invisible=true',
  'SCAN pinned-namespace invisible=true',
  'SCAN done',
];

/**
 The measurements the holder-scan arm prints beside the numbers they size.
 */
export const expectedScanMeasurements: readonly string[] = [
  'MEASUREMENT holder-scan-ms=',
];

/**
 The measurements the sweep arm prints beside the numbers they size.
 */
export const expectedSweepMeasurements: readonly string[] = [
  'MEASUREMENT sweep-attach-ms=',
  'MEASUREMENT child-bound-listing-ms=',
];

/**
 The lines a run must find in the guest's unmount-arm log, one per staging arm
 plan step 9 runs.
 */
export const expectedUnmountMarkers: readonly string[] = [
  'UNMOUNT attached clone=',
  'UNMOUNT plain=unmounted receipt=done destroy=ok',
  'UNMOUNT busy=reattached holder=ended',
  'UNMOUNT reattach-then-unmounted=true',
  'UNMOUNT two-at-once=both-unmounted',
  'UNMOUNT handle-reopened=true',
  'UNMOUNT kill=staged finished=unmounted',
  'UNMOUNT kill=between-unmounts finished=unmounted',
  'UNMOUNT kill=bind-over-slot finished=unmounted',
  'UNMOUNT kill=none finished=unmounted',
  'UNMOUNT kill-points=done',
  'UNMOUNT done',
];

/**
 The lines a run must find in the guest's publication-fsync log.
 */
export const expectedFsyncMarkers: readonly string[] = [
  'PROBE append with fsync of the file and the folder:',
  'PROBE publication of one slot record: stage, write, fsync, link, unlink, folder fsync:',
];
