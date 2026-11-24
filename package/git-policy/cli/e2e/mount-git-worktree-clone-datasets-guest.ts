#!/usr/bin/env node
/**
 Drive one end-to-end run of the root-run clone-mount handler's adoption mode, in a throwaway `mvm` guest.

 The arms write the store folder `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities` and open
 a checkout as root, so no control on a person's machine may run them. This driver borrows the ZFS binding's guest
 recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's `native_binary` test target inside it, runs the
 adoption and store controls there, runs the store's publication-fsync probe, and destroys the guest.

 Usage: node e2e/mount-git-worktree-clone-datasets-guest.ts <create|prepare|run|delegation|snapdir|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 `delegation` answers the owner's question of 2026-11-22 by a run: under the
 design's own grants, as `ubuntu`, it tries every route that could give a
 dataset of the checkout's leaf name a row in the host's mount table, and then
 runs root's `zfs mount -a` and reads the table.

 The evidence of one run is `adoption-arm.log` and `store-fsync.log`, pulled back into the log directory, and a run
 passes only when the first carries every `ADOPTION` and `STORE` line below, the second carries both `PROBE` lines, and
 neither holds a `SKIPPED` line, so an arm that returned without running fails the run instead of reporting a green
 suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

/** Where the crate is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/git-policy-cli-adoption';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the test target may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds the guest arm may take. */
const ARM_TIMEOUT_MS = 900_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The environment variable the adoption arm reads, so it never writes `/var/lib` on a machine somebody uses. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_ADOPTION_GUEST';
/** The lines a run must find in the guest's arm log, one per arm it performed. */
const EXPECTED_MARKERS = [
  'ADOPTION recorded device=',
  'ADOPTION refused=wrong-device',
  'ADOPTION refused=link',
  'ADOPTION refused=mode-widened',
  'ADOPTION refused=slot-differs',
  'ADOPTION done',
  'STORE published=measured folder-mode=0755 record-mode=0644',
  'STORE replaced-parent=one-rename',
  'STORE killed-writer staging-removed=true',
] as const;

/** The lines a run must find in the guest's `snapdir` log, one per value of the property. */
const EXPECTED_SNAPDIR_MARKERS = [
  'SNAPDIR visible listed=true entered=true mounts-grown=true',
  'SNAPDIR hidden listed=false entered=true mounts-grown=true',
  'SNAPDIR disabled listed=false entered=false mounts-grown=false',
  'SNAPDIR account-cannot-set-it-back=true',
  'SNAPDIR release has-disabled=true',
  'SNAPDIR done',
] as const;

/** The lines a run must find in the guest's delegation log, one per attempt the design's grants allow or refuse. */
const EXPECTED_DELEGATION_MARKERS = [
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
] as const;

/** The lines a run must find in the guest's sweep-arm log, one per arm plan step 8 runs. */
const EXPECTED_SWEEP_MARKERS = [
  'SWEEP attached item=1 slot=one origin=checkout-snapshot',
  'SWEEP first-attach outcome=attached slot=one',
  'SWEEP twice-swept receipt=done',
  'SWEEP refusal item=5 step=key-unavailable receipt=refused',
  'SWEEP attached item=5 slot=two',
  'SWEEP refusal item=2 step=intent',
  'SWEEP nothing inherited=true',
  'SWEEP refusal item=2 step=clone-origin',
  'SWEEP refusal item=2 step=clone-origin-foreign',
  'SWEEP rollback unmount=EBUSY',
  'SWEEP rollback unmount=ok',
  'SWEEP bound stopped=true',
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
] as const;

/** The checkout the holder scan's arms build their own design layout inside, under the guest's own root. */
const SCAN_CHECKOUT = '/root/holder-scan-checkout';

/** The pool the holder scan's arms create their clone parent in, which the driver prepared for the store's arms. */
const SCAN_POOL = 'git-policy-cli-store';

/** The lines a run must find in the guest's holder-scan log, one per arm plan step 9 runs. */
const EXPECTED_SCAN_MARKERS = [
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
] as const;

/** The measurements the holder-scan arm prints beside the numbers they size. */
const EXPECTED_SCAN_MEASUREMENTS = [
  'MEASUREMENT holder-scan-ms=',
] as const;

/** The measurements the sweep arm prints beside the numbers they size. */
const EXPECTED_SWEEP_MEASUREMENTS = [
  'MEASUREMENT sweep-attach-ms=',
  'MEASUREMENT child-bound-listing-ms=',
] as const;

/** The lines a run must find in the guest's unmount-arm log, one per staging arm plan step 9 runs. */
const EXPECTED_UNMOUNT_MARKERS = [
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
] as const;

/** The checkout the sweep arms walk inside the guest, under the guest's own root. */
const SWEEP_CHECKOUT = '/root/sweep-checkout';

/** The pool the sweep arms create their clone parents in, which the driver prepared for the store's arms. */
const SWEEP_POOL = 'git-policy-cli-store';

/** The lines a run must find in the guest's publication-fsync log. */
const EXPECTED_FSYNC_MARKERS = [
  'PROBE append with fsync of the file and the folder:',
  'PROBE publication of one slot record: stage, write, fsync, link, unlink, folder fsync:',
] as const;

/** Where the store's publication-fsync arm writes inside the guest, on a dataset of the store's own pool. */
const FSYNC_FOLDER = '/var/lib/monochromatic-git-policy-cli-probe';

/** The store's path inside the guest, which the arms write. */
const STORE_PATH = '/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities';

/** The folder inside the probe's dataset that the probe creates and removes for each run. */
const FSYNC_ROUND_FOLDER = `${FSYNC_FOLDER}/round`;

/** The pool the guest makes so the store is on a ZFS dataset, and the file-backed vdev it stands on. */
const STORE_POOL = 'git-policy-cli-store';
const STORE_POOL_IMAGE = '/home/git-policy-cli-store-fixture/store.img';
const STORE_DATASET = `${STORE_POOL}/store`;
const PROBE_DATASET = `${STORE_POOL}/probe`;

/** The pool the delegation arm makes, the image it stands on and the checkout it mounts. */
const DELEGATION_POOL = 'git-policy-cli-delegation';
const DELEGATION_IMAGE = '/home/git-policy-cli-delegation.img';
const DELEGATION_CHECKOUT = '/home/delegation-checkout';
/** The account the guest image creates, which the design delegates to. */
const DELEGATION_ACCOUNT = 'ubuntu';

/** The `PATH` the guest's OpenZFS 2.4.4 userspace needs. */
const OPENZFS_PATH = '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';

/** Seconds the publication-fsync arm runs, which is the record's own count of rounds. */
const FSYNC_ROUNDS = '60';

/** The step, the guest name, the log directory and whether `all` keeps the guest. */
interface Options {
  readonly step: string;
  readonly name: string;
  readonly logDir: string;
  readonly keepGuest: boolean;
}

/** The step and the values the command line gave it. */
function options(): Options {
  const [step = '', ...rest] = process.argv.slice(2);
  const value = (flag: string): string | undefined => {
    const at = rest.indexOf(flag);
    return at === -1 ? undefined : rest[at + 1];
  };
  return {
    step,
    name: value('--name') ?? 'git-policy-cli-adoption',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'adoption-guest-logs'),
    keepGuest: rest.includes('--keep-guest'),
  };
}

/** Quote one block of shell for the guest's `sh -ec`, quotes included. */
function shellQuote(script: string): string {
  return `'${script.replaceAll("'", `'\\''`)}'`;
}

/**
 One host command whose whole output goes to its own log.

 @param chosen - the run's options

 @param command - the program to run

 @param args - its arguments

 @param log - the log file's name in the log directory

 @throws Error naming the command and its status
 */
function host({ chosen, command, args, log, }: { readonly chosen: Options; readonly command: string; readonly args: readonly string[]; readonly log: string; }): void {
  const output = openSync(join(chosen.logDir, log), 'w', 0o600);
  try {
    const result = spawnSync(command, args, {
      stdio: ['ignore', output, output,],
      timeout: HOST_COMMAND_TIMEOUT_MS,
    });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
}

/**
 One guest script, its whole output kept in its own log.

 @param chosen - the run's options

 @param script - the shell script to run inside the guest

 @param log - the log file's name in the log directory

 @param timeout - milliseconds the guest may take

 @returns what the guest wrote

 @throws Error naming the script and the guest's status
 */
function guest({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): string {
  const output = openSync(join(chosen.logDir, log), 'w', 0o600);
  const command = `test "$(hostname)" = ${chosen.name} && timeout ${Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS} sh -ec ${shellQuote(script)}`;
  try {
    const result = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', command], {
      stdio: ['ignore', output, output,],
      timeout,
    });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`mvm exec exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
  return readFileSync(join(chosen.logDir, log), 'utf8');
}

/**
 Run one step of the ZFS binding's guest recipe, which owns the guest's creation, preparation and destruction.

 @param chosen - the run's options

 @param step - the recipe's step

 @throws Error naming the recipe step and its status
 */
function recipe({ chosen, step, }: { readonly chosen: Options; readonly step: string; }): void {
  const script = resolve(import.meta.dirname, '..', '..', '..', 'zfs', 'request-to-openzfs-kernel-device', 'e2e', 'run-fixture.ts');
  const result = spawnSync('node', [script, step, '--name', chosen.name, '--log-dir', chosen.logDir,], {
    stdio: 'inherit',
    timeout: HOST_COMMAND_TIMEOUT_MS * 60,
  });
  if ((result.error !== undefined) || (result.status !== 0))
    throw result.error ?? new Error(`the guest recipe's ${step} step exited ${String(result.status)}`);
}

/**
 Build the test target inside the guest and run the adoption control there.

 The snapshot is the same repository-shaped tree the container gate builds from
 (`bin/native-wrapper-snapshot.ts`), so the arm compiles against the path crates the wrapper links, the ZFS binding
 among them, rather than against a copy somebody remembered to add.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks
 */
async function run(chosen: Options): Promise<void> {
  const context = join(chosen.logDir, 'snapshot');
  rmSync(context, { recursive: true, force: true, });
  mkdirSync(context, { recursive: true, mode: 0o700, });
  await copyWrapperSnapshot({ context, });
  const tarball = join(chosen.logDir, 'subject.tar.gz');
  host({ chosen, command: 'tar', args: ['--create', '--gzip', '--file', tarball, '--exclude=target', '--directory', context, '.'], log: 'host-package.log', });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, tarball, '/root/subject.tar.gz'], log: 'guest-push.log', });
  const build = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked > /root/adoption-build.log 2>&1',
    'tail --lines=3 /root/adoption-build.log',
  ].join('\n');
  guest({ chosen, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  storeDataset({ chosen, });
  const arm = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      `--filterset "test(zfs_adoption_guest)" > /root/adoption-arm.log 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=40 /root/adoption-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the recipe for the binding's live suites also holds.
    `if grep --quiet '^SKIPPED' /root/adoption-arm.log; then echo 'the adoption arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/adoption-arm.log`;
    }),
    'cp /root/adoption-arm.log /mnt/shared/adoption-arm.log',
  ].join('\n');
  guest({ chosen, script: arm, log: 'guest-adoption.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/adoption-arm.log', join(chosen.logDir, 'adoption-arm.log')], log: 'host-pull-adoption.log', });
  const pulled = readFileSync(join(chosen.logDir, 'adoption-arm.log'), 'utf8');
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
  if (/(^|\n)SKIPPED/m.test(pulled))
    throw new Error('the pulled arm log holds a SKIPPED line, so the adoption arm did not run');
  publicationFsync({ chosen, });
  sweepArms({ chosen, });
  holderScanArms({ chosen, });
  unmountArms({ chosen, });
}

/**
 Drive plan step 9's holder-scan arms in the same guest: the built handler's scan against the design's layout.

 The arms build their own checkout, clone parent and clone, attach the clone with the built sweep, and then hold it
 from a second mount namespace, a namespace made after the attach (under both intents), a confined user namespace, a
 thread with a mount namespace of its own, a task whose leader has ended, a namespace whose lowest task is chrooted,
 and a namespace pinned by a handle while no task is left in it; one arm detaches a mount while a working directory
 pins it. Task churn runs inside the guest only, and one scan is timed with a small and a large number of live tasks.
 Every one of those reads `/proc` or mounts a dataset, so no arm may run on the host.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function holderScanArms({ chosen, }: { readonly chosen: Options; }): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `rm --recursive --force ${SCAN_CHECKOUT}`,
    `mkdir --parents ${SCAN_CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export CLI_GIT_ZFS_SCAN_GUEST=${chosen.name}`,
    `export CLI_GIT_ZFS_SCAN_POOL=${SCAN_POOL}`,
    'export CLI_GIT_ZFS_SCAN_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SCAN_CHECKOUT=${SCAN_CHECKOUT}`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_holder_scan_guest)" > /root/holder-scan-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=80 /root/holder-scan-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/holder-scan-arm.log; then echo 'the holder-scan arm returned without running'; exit 1; fi`,
    ...EXPECTED_SCAN_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/holder-scan-arm.log`;
    }),
    ...EXPECTED_SCAN_MEASUREMENTS.map(function measurementLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/holder-scan-arm.log`;
    }),
    'cp /root/holder-scan-arm.log /mnt/shared/holder-scan-arm.log',
  ].join('\n');
  guest({ chosen, script, log: 'guest-holder-scan.log', timeout: ARM_TIMEOUT_MS * 3, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/holder-scan-arm.log', join(chosen.logDir, 'holder-scan-arm.log'),], log: 'host-pull-holder-scan.log', });
  const pulled = readFileSync(join(chosen.logDir, 'holder-scan-arm.log'), 'utf8');
  for (const marker of EXPECTED_SCAN_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled holder-scan log lacks ${marker}; the guest ran the arm without performing it`);
  }
}

/**
 Make the ZFS pool and the two datasets the store's arms write on.

 The guest's own `/var/lib` is on its ext4 root, which no ZFS property reaches, so the store's path is
 given a dataset of the guest's own pool. That is what settles the design's assumption about the
 store's dataset: a hard link, an unlinking, a rename and a folder `fsync` on a ZFS dataset.

 @param chosen - the run's options

 @throws Error when any of the pool's commands refused
 */
function storeDataset({ chosen, }: { readonly chosen: Options; }): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    'mkdir --parents "$(dirname ' + STORE_POOL_IMAGE + ')"',
    `if ! zpool list -H -o name | grep --quiet '^${STORE_POOL}$'; then truncate --size 1G ${STORE_POOL_IMAGE}; zpool create -f ${STORE_POOL} ${STORE_POOL_IMAGE}; fi`,
    `zfs list -H -o name ${STORE_DATASET} > /dev/null 2>&1 || zfs create ${STORE_DATASET}`,
    `zfs set mountpoint=${STORE_PATH} ${STORE_DATASET}`,
    `zfs list -H -o name ${PROBE_DATASET} > /dev/null 2>&1 || zfs create ${PROBE_DATASET}`,
    `zfs set mountpoint=${FSYNC_FOLDER} ${PROBE_DATASET}`,
    `chmod 0755 ${STORE_PATH} ${FSYNC_FOLDER}`,
    `findmnt --target ${STORE_PATH} --output SOURCE --noheadings --first-only`,
  ].join('\n');
  guest({ chosen, script, log: 'guest-store-dataset.log', timeout: ARM_TIMEOUT_MS, });
}

/**
 Run the store's publication cost against a dataset of the store's own pool, inside the same guest.

 The probe is pushed rather than snapshot, because it lives in the package's `doc/` folder and the snapshot holds only
 what a build reads. It is run in the guest's own `git-policy-cli-store/probe` dataset, which lies on the same pool as the
 store and beside the store's own dataset rather than on it, so the arm is what replaced the lock-hold record's subject.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function publicationFsync({ chosen, }: { readonly chosen: Options; }): void {
  const probe = resolve(import.meta.dirname, '..', 'doc', 'slot-identity-store-write-cost', 'probe-reaches-storage.py');
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, probe, '/root/probe-reaches-storage.py',], log: 'guest-push-probe.log', });
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    // The probe reads its folder's dataset before it creates anything, and it removes and recreates its own folder,
    // so it is given a folder inside the dataset rather than the dataset's mount point.
    `mkdir --parents ${FSYNC_ROUND_FOLDER}`,
    // The block device the pool's own file-backed vdev lives on, so the counters the probe reads are the ones the
    // pool's writes reach; the probe reads the pool's own ZIL counters from its kstat folder.
    `pool_device=$(findmnt --target ${STORE_POOL_IMAGE} --output SOURCE --noheadings --first-only)`,
    'test -n "$pool_device"',
    'pool_disk=$(lsblk --noheadings --output PKNAME --paths "$pool_device")',
    `python3 -I /mnt/shared/probe-reaches-storage.py ${FSYNC_ROUND_FOLDER} ${FSYNC_ROUNDS} --pool-device "$(basename "$pool_device")" --pool-disk "$(basename "$pool_disk")" > /mnt/shared/store-fsync.log 2>&1`,
  ].join('\n');
  guest({ chosen, script, log: 'guest-store-fsync.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/store-fsync.log', join(chosen.logDir, 'store-fsync.log'),], log: 'host-pull-store-fsync.log', });
  const pulled = readFileSync(join(chosen.logDir, 'store-fsync.log'), 'utf8');
  for (const marker of EXPECTED_FSYNC_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled publication-fsync log lacks ${marker}; the guest ran the probe without that arm`);
  }
}

/**
 Drive plan step 8's arms in the same guest: the built handler's own sweep against the guest's pool.

 The arms make their own fixtures as root inside the guest, because the sweep attaches a real clone and writes
 `/var/lib` and `/run`: one unencrypted clone parent with a clone that asks for its mount, one encrypted parent whose
 key is unloaded and then loaded, and the handler's refusals. This host owns the pool and the checkout only, so a run
 whose guest cannot answer fails here rather than reporting a green suite.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function sweepArms({ chosen, }: { readonly chosen: Options; }): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    // A layout an earlier run left mounted is taken down first, so the arm runs again on the same guest.
    // Every mount under the checkout goes first, the slots an earlier run attached included, then the checkout itself;
    // the findmnt sweep is the one that also reaches a mount the rollback arm moved beside the checkout.
    `for slot in ${SWEEP_CHECKOUT}/.claude/worktrees/*; do umount --force "$slot" 2>/dev/null || true; done`,
    'for target in $(findmnt --list --noheadings --output TARGET | grep --fixed-strings ' + SWEEP_CHECKOUT + ' || true); do umount "$target" 2>/dev/null || true; done',
    `umount --force ${SWEEP_CHECKOUT} 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-bound 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-encrypted 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-foreign 2>/dev/null || true`,
    // The stoppage arm's container and the probe fixture this arm creates live in the same pool.
    `zfs destroy --recursive --force ${SWEEP_POOL}/account-clones 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/nosnap 2>/dev/null || true`,
    // Another driver's checkout dataset carries the same leaf name and would be a second instance for the arm's
    // no-configuration shapes; it is taken down for the length of the arm and mounted again afterwards.
    'others=$(findmnt --list --noheadings --output SOURCE --type zfs | grep --fixed-strings monochromatic-git-checkout | grep --invert-match "git-policy-cli-store/step8" | sort --unique || true)',
    'for other in $others; do zfs unmount "$other" 2>/dev/null || true; done',
    `rm --recursive --force ${SWEEP_CHECKOUT}`,
    `mkdir --parents ${SWEEP_CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export CLI_GIT_ZFS_SWEEP_GUEST=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${SWEEP_POOL}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${SWEEP_CHECKOUT}`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_sweep_guest)" > /root/sweep-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=60 /root/sweep-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/sweep-arm.log; then echo 'the sweep arm returned without running'; exit 1; fi`,
    ...EXPECTED_SWEEP_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/sweep-arm.log`;
    }),
    ...EXPECTED_SWEEP_MEASUREMENTS.map(function measurementLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/sweep-arm.log`;
    }),
    'cp /root/sweep-arm.log /mnt/shared/sweep-arm.log',
    'for other in $others; do zfs mount "$other" 2>/dev/null || true; done',
    // The arm's own layout goes, so a driver that runs afterwards finds one instance of the checkout's leaf name and
    // not two: the probe's no-configuration arm reads the whole mount table.
    `for target in $(findmnt --list --noheadings --output TARGET | grep --fixed-strings ${SWEEP_CHECKOUT} || true); do umount "$target" 2>/dev/null || true; done`,
    `umount ${SWEEP_CHECKOUT} 2>/dev/null || true`,
    `for layout in step8 step8-encrypted step8-foreign step8-bound account-clones nosnap; do zfs destroy -r -f ${SWEEP_POOL}/$layout 2>/dev/null || true; done`,
  ].join('\n');
  guest({ chosen, script, log: 'guest-sweep.log', timeout: ARM_TIMEOUT_MS * 2, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/sweep-arm.log', join(chosen.logDir, 'sweep-arm.log'),], log: 'host-pull-sweep.log', });
  const pulled = readFileSync(join(chosen.logDir, 'sweep-arm.log'), 'utf8');
  for (const marker of EXPECTED_SWEEP_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled sweep-arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
}


/**
 Answer the owner's question of 2026-11-22 by a run: as the account the design delegates to, try every route that could
 give a dataset named `monochromatic-git-checkout` a row in the host's own mount table.

 The design's grants are applied to the exact datasets it names
 (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
 "Scoped delegation"):
 `snapshot` on the checkout dataset,
 `clone` on its descendants,
 `create,mount` on the clone container,
 `mount,userprop` on the container's descendants,
 and `mount,destroy` on the checkout's descendants.
 Then the account tries to create,
 rename,
 mount and property-set a dataset of the design's leaf name,
 to make a pool of its own,
 and to import one;
 root then runs `zfs mount -a`,
 which is what a boot's `zfs-mount.service` does,
 and the arm reads the mount table afterwards.
 Nothing here runs on the host.

 @param chosen - the run's options

 @throws Error naming the attempt and the guest's status
 */
function delegation({ chosen, }: { readonly chosen: Options; }): void {
  const sbin = '/opt/openzfs-2.4.4/sbin';
  const library = '/opt/openzfs-2.4.4/lib';
  const attempts = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `export LD_LIBRARY_PATH=${library}`,
    `ACCOUNT=${DELEGATION_ACCOUNT}`,
    `POOL=${DELEGATION_POOL}`,
    `IMAGE=${DELEGATION_IMAGE}`,
    'IMAGE_SECOND=/home/git-policy-cli-delegation-second.img',
    'POOL_SECOND="$POOL-second"',
    'CHECKOUT=' + DELEGATION_CHECKOUT,
    'HOME_DS="$POOL/ROOT/cos/home"',
    'CHECKOUT_DS="$HOME_DS/monochromatic-git-checkout"',
    'CONTAINER="$HOME_DS/monochromatic-git-worktree-clone-datasets"',
    `as_account() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${library} ${sbin}/zfs "$@"; }`,
    'as_account_ls() { runuser -u "$ACCOUNT" -- ls -a "$1"; }',
    `as_account_zpool() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${library} ${sbin}/zpool "$@"; }`,
    'rows() { findmnt --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings monochromatic-git | sort | tr "\\n" ";"; }',
    'say() { echo "DELEGATION $1=$2 rows=$(rows)"; }',
    // The design's layout and its grants.
    'zpool destroy -f "$POOL" > /dev/null 2>&1 || true',
    'rm --force "$IMAGE"',
    'truncate --size 512M "$IMAGE"',
    'zpool create -f -O mountpoint=none "$POOL" "$IMAGE"',
    'zfs create -o mountpoint=none "$POOL/ROOT"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos"',
    'zfs create -o mountpoint=none "$HOME_DS"',
    'zfs create -o mountpoint="$CHECKOUT" "$CHECKOUT_DS"',
    'mount --make-private "$CHECKOUT"',
    'zfs create -o mountpoint=legacy -o canmount=off -o setuid=off -o devices=off "$CONTAINER"',
    // The design's grants exactly as "Scoped delegation" names them: local where it says local, descendants where it
    // says descendants. The integration pass found this arm's earlier form granted everything to descendants, which is
    // wider than the design.
    'zfs allow -l -u "$ACCOUNT" snapshot "$CHECKOUT_DS"',
    'zfs allow -d -u "$ACCOUNT" clone,destroy,mount "$CHECKOUT_DS"',
    'zfs allow -l -u "$ACCOUNT" create,mount "$CONTAINER"',
    'zfs allow -d -u "$ACCOUNT" mount,destroy,userprop "$CONTAINER"',
    'echo "DELEGATION grants:"; zfs allow "$CHECKOUT_DS"; zfs allow "$CONTAINER"',
    // A: create under the container, which `create` on it allows.
    'if as_account create "$CONTAINER/plain" > /root/delegation-create.log 2>&1; then say create-under-container allowed; else say create-under-container refused; sed "s/^/  /" /root/delegation-create.log; fi',
    // B: create the design's leaf name with a mountpoint the account chooses.
    'if as_account create -o mountpoint=/home/account-second "$CONTAINER/monochromatic-git-checkout" > /root/delegation-create-mountpoint.log 2>&1; then say create-with-mountpoint allowed; else say create-with-mountpoint refused; sed "s/^/  /" /root/delegation-create-mountpoint.log; fi',
    // C: snapshot the checkout, which it is delegated.
    'if as_account snapshot "$CHECKOUT_DS@account-origin" > /root/delegation-snapshot.log 2>&1; then say snapshot-of-checkout allowed; else say snapshot-of-checkout refused; sed "s/^/  /" /root/delegation-snapshot.log; fi',
    // D: clone it to the design's leaf name, which `clone` and `create` allow.
    'if as_account clone "$CHECKOUT_DS@account-origin" "$CONTAINER/monochromatic-git-checkout" > /root/delegation-clone.log 2>&1; then say clone-named-the-leaf allowed; else say clone-named-the-leaf refused; sed "s/^/  /" /root/delegation-clone.log; fi',
    'zfs get -H -o property,value mountpoint,canmount,origin "$CONTAINER/monochromatic-git-checkout" 2>/dev/null || true',
    // E: rename the plain dataset onto the design's leaf name.
    'if as_account rename "$CONTAINER/plain" "$CONTAINER/monochromatic-git-checkout-second" > /root/delegation-rename.log 2>&1; then say rename-onto-the-leaf allowed; else say rename-onto-the-leaf refused; sed "s/^/  /" /root/delegation-rename.log; fi',
    // F: mount it into the host's namespace as the account.
    'if as_account mount "$CONTAINER/monochromatic-git-checkout" > /root/delegation-mount.log 2>&1; then say mount-as-account allowed; else say mount-as-account refused; sed "s/^/  /" /root/delegation-mount.log; fi',
    // G and H: the properties a mount needs, which it is not delegated.
    'if as_account set mountpoint=/home/account-third "$CONTAINER/monochromatic-git-checkout" > /root/delegation-set-mountpoint.log 2>&1; then say set-mountpoint-as-account allowed; else say set-mountpoint-as-account refused; sed "s/^/  /" /root/delegation-set-mountpoint.log; fi',
    'if as_account set canmount=on "$CONTAINER/monochromatic-git-checkout" > /root/delegation-set-canmount.log 2>&1; then say set-canmount-as-account allowed; else say set-canmount-as-account refused; sed "s/^/  /" /root/delegation-set-canmount.log; fi',
    // I and J: a pool of its own, and one to import.
    // The image a pool of the account's own would stand on is made readable and writable by it first,
    // so the attempt answers the capability rather than a missing file.
    'rm --force "$IMAGE_SECOND"',
    'truncate --size 256M "$IMAGE_SECOND"',
    'chmod 0666 "$IMAGE_SECOND"',
    'if as_account_zpool create -f "$POOL_SECOND" "$IMAGE_SECOND" > /root/delegation-pool.log 2>&1; then say create-pool-as-account allowed; else say create-pool-as-account refused; sed "s/^/  /" /root/delegation-pool.log; fi',
    // Root makes the same second pool and exports it, so the account's import attempt is a capability question rather than a name one.
    'zpool create -f -O mountpoint=none "$POOL_SECOND" "$IMAGE_SECOND"',
    'zpool export "$POOL_SECOND"',
    'if as_account_zpool import -d /home "$POOL_SECOND" > /root/delegation-import.log 2>&1; then say import-pool-as-account allowed; else say import-pool-as-account refused; sed "s/^/  /" /root/delegation-import.log; fi',
    'zpool import -d /home "$POOL_SECOND" || true',
    'zpool destroy -f "$POOL_SECOND" || true',
    // K: root mounts every dataset the pool holds, which is what a boot does.
    'echo "DELEGATION before-root-mount-a: $(rows)"',
    'zfs mount -a',
    'if rows | grep --quiet --fixed-strings "/home/account"; then say root-mount-a mounted-the-accounts; else say root-mount-a mounted-nothing-of-the-accounts; fi',
    'zfs list -H -o name,mountpoint,canmount -r "$POOL" | sed "s/^/  /"',
    'echo "DELEGATION done"',
  ].join('\n');
  guest({ chosen, script: attempts, log: 'guest-delegation.log', timeout: ARM_TIMEOUT_MS, });
  const pulled = readFileSync(join(chosen.logDir, 'guest-delegation.log'), 'utf8');
  for (const marker of EXPECTED_DELEGATION_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the delegation log lacks ${marker}; the guest ran the attempt without proving it`);
  }
}


/**
 Show what each value of the `snapdir` property does, which the project's third
 answer of 2026-11-22 asked for before any code relies on it.

 For each of `visible`, `hidden` and `disabled`, as the unprivileged account and
 under the design's own grants, the arm lists the checkout's `.zfs` directory,
 tries to enter a snapshot's directory by name, and reads the host's mount table
 before and after.
 Then it reads whether the account may set the property back,
 and which values this guest's OpenZFS release and the one the route's guests run
 accept.

 Nothing is set on the host: the pool is this guest's file-backed one.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function snapdirMatrix({ chosen, }: { readonly chosen: Options; }): void {
  const sbin = '/opt/openzfs-2.4.4/sbin';
  const library = '/opt/openzfs-2.4.4/lib';
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `export LD_LIBRARY_PATH=${library}`,
    'ACCOUNT=ubuntu',
    `POOL=${DELEGATION_POOL}`,
    'IMAGE=/home/snapdir-matrix.img',
    'CHECKOUT=/home/snapdir-checkout',
    'DS="$POOL/ROOT/cos/home/monochromatic-git-checkout"',
    `as_account() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${library} ${sbin}/zfs "$@"; }`,
    'as_account_ls() { runuser -u "$ACCOUNT" -- ls -a "$1"; }',
    'snapshot_rows() { findmnt --list --noheadings --output SOURCE --type zfs | grep --count "@" || true; }',
    'leaf_rows() { findmnt --list --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings "monochromatic-git-checkout" | tr "\\n" ";"; }',
    'snapdir_rows_cleanup() { findmnt --list --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings "@account-origin" | sed "s/ .*//" | xargs --no-run-if-empty umount || true; }',
    'zpool destroy -f "$POOL" > /dev/null 2>&1 || true',
    'rm --force "$IMAGE"',
    'truncate --size 512M "$IMAGE"',
    'zpool create -f -O mountpoint=none "$POOL" "$IMAGE"',
    'zfs create -o mountpoint=none "$POOL/ROOT"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos/home"',
    'zfs create -o mountpoint="$CHECKOUT" "$DS"',
    'mount --make-private "$CHECKOUT"',
    'zfs snapshot "$DS@account-origin"',
    'zfs allow -l -u "$ACCOUNT" snapshot "$DS"',
    'zfs allow -d -u "$ACCOUNT" clone,destroy,mount "$DS"',
    // The guest's own release accepts which values?
    'if zfs set snapdir=disabled "$DS" > /root/snapdir-disabled-set.log 2>&1; then echo "SNAPDIR release has-disabled=true"; else echo "SNAPDIR release has-disabled=false"; cat /root/snapdir-disabled-set.log; exit 1; fi',
    'zfs inherit snapdir "$DS"',
    // Each value is measured from a fresh mount, because a `.zfs` dentry the kernel already resolved keeps the
    // directory reachable until the dataset is mounted again, whatever the property says afterwards.
    'for value in visible hidden disabled; do',
    '  zfs unmount "$DS" > /dev/null 2>&1 || true',
    '  snapdir_rows_cleanup',
    '  zfs set "snapdir=$value" "$DS"',
    '  zfs mount "$DS"',
    '  mount --make-private "$CHECKOUT"',
    '  read_value=$(zfs get -H -o value snapdir "$DS")',
    '  test "$read_value" = "$value"',
    '  if as_account_ls "$CHECKOUT" > /root/snapdir-list.log 2>&1 && grep --quiet "^\.zfs$" /root/snapdir-list.log; then listed=true; else listed=false; fi',
    '  before=$(snapshot_rows)',
    '  if as_account_ls "$CHECKOUT/.zfs/snapshot/account-origin" > /root/snapdir-enter.log 2>&1; then entered=true; else entered=false; fi',
    '  after=$(snapshot_rows)',
    '  if test "$after" -gt "$before"; then grown=true; else grown=false; fi',
    '  echo "SNAPDIR $value listed=$listed entered=$entered mounts-grown=$grown enter-answer=$(head --lines=1 /root/snapdir-enter.log 2>/dev/null || true)"',
    '  # Entering a snapshot directory is what makes the kernel mount it, so the two answers must agree in every state.',
    '  test "$entered" = "$grown" || { echo "SNAPDIR $value entered=$entered grown=$grown disagree"; exit 1; }',
    '  snapdir_rows_cleanup',
    'done',
    // The account cannot set the property back: the route's grants do not name it.
    'if as_account set snapdir=visible "$DS" > /root/snapdir-account-set.log 2>&1; then echo "SNAPDIR account-cannot-set-it-back=false"; else echo "SNAPDIR account-cannot-set-it-back=true"; sed "s/^/  /" /root/snapdir-account-set.log; fi',
    'zfs get -H -o property,value snapdir "$DS"',
    // What the route's other release does: the distro module in this guest, if it is installed.
    'if test -f /usr/sbin/zfs; then /usr/sbin/zfs --version || true; else echo "SNAPDIR distro-userspace absent"; fi',
    'zpool destroy -f "$POOL"',
    'rm --force "$IMAGE"',
    'echo "SNAPDIR done"',
  ].join('\n');
  guest({ chosen, script, log: 'guest-snapdir.log', timeout: ARM_TIMEOUT_MS, });
  const pulled = readFileSync(join(chosen.logDir, 'guest-snapdir.log'), 'utf8');
  for (const marker of EXPECTED_SNAPDIR_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the snapdir log lacks ${marker}; the guest ran the arm without proving it`);
  }
}

/**
 Drive plan step 9's staging mechanics in the same guest: the record before the bind, the private bind, a mount moved onto it, the later sweep from the record's handle, a record from another boot, and the cleanup's allowed and rejected fixtures.

 The arm makes a tmpfs stand in for a clone, because the design's ZFS layout arms and M1 to M9 remain owed; it runs the built module's calls, the record and the sweep, and it prints one marker per arm.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function unmountArms({ chosen, }: { readonly chosen: Options; }): void {
  const script = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export CLI_GIT_ZFS_UNMOUNT_GUEST=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${SWEEP_POOL}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${SWEEP_CHECKOUT}`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      `--filterset "test(zfs_unmount_guest)" > /root/unmount-arm.log 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=60 /root/unmount-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/unmount-arm.log; then echo 'the unmount arm returned without running'; exit 1; fi`,
    ...EXPECTED_UNMOUNT_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/unmount-arm.log`;
    }),
    'cp /root/unmount-arm.log /mnt/shared/unmount-arm.log',
  ].join('\n');
  guest({ chosen, script, log: 'guest-unmount.log', timeout: ARM_TIMEOUT_MS * 2, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/unmount-arm.log', join(chosen.logDir, 'unmount-arm.log'),], log: 'host-pull-unmount.log', });
  const pulled = readFileSync(join(chosen.logDir, 'unmount-arm.log'), 'utf8');
  for (const marker of EXPECTED_UNMOUNT_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled unmount-arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
}

/** The steps by name, and the whole run. */
const steps: Record<string, (chosen: Options) => Promise<void> | void> = {
  create: (chosen) => recipe({ chosen, step: 'create', }),
  prepare: (chosen) => recipe({ chosen, step: 'prepare', }),
  run,
  delegation: (chosen) => delegation({ chosen, }),
  snapdir: (chosen) => snapdirMatrix({ chosen, }),
  destroy: (chosen) => recipe({ chosen, step: 'destroy', }),
  all: async (chosen: Options): Promise<void> => {
    let failure: unknown = null;
    let created = false;
    try {
      recipe({ chosen, step: 'create', });
      created = true;
      recipe({ chosen, step: 'prepare', });
      await run(chosen);
    } catch (thrown) {
      failure = thrown;
    }
    if (chosen.keepGuest && created)
      console.log(`git-policy adoption guest: --keep-guest kept '${chosen.name}' to inspect and destroy`);
    else if (!chosen.keepGuest && created) {
      try {
        recipe({ chosen, step: 'destroy', });
      } catch (destruction) {
        if (failure === null)
          throw destruction;
        console.error(`git-policy adoption guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
      }
    } else if (!chosen.keepGuest)
      console.error(`git-policy adoption guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
    if (failure !== null)
      throw failure;
  },
};

const chosen = options();
mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
const step = steps[chosen.step];
if (step === undefined)
  throw new Error(`unknown step '${chosen.step}'; use ${Object.keys(steps).join(', ')}`);
await step(chosen);
console.log(`git-policy adoption guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
