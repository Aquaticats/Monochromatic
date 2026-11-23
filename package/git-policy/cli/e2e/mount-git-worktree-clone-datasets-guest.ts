#!/usr/bin/env node
/**
 Drive one end-to-end run of the root-run clone-mount handler's adoption mode, in a throwaway `mvm` guest.

 The arms write the store folder `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities` and open
 a checkout as root, so no control on a person's machine may run them. This driver borrows the ZFS binding's guest
 recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's `native_binary` test target inside it, runs the
 adoption and store controls there, runs the store's publication-fsync probe, and destroys the guest.

 Usage: node e2e/mount-git-worktree-clone-datasets-guest.ts <create|prepare|run|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

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
  'SWEEP done',
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
  unmountArms({ chosen, });
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
    `for slot in ${SWEEP_CHECKOUT}/.claude/worktrees/*; do umount --force "$slot" 2>/dev/null || true; done`,
    `umount --force ${SWEEP_CHECKOUT} 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-bound 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-encrypted 2>/dev/null || true`,
    `zfs destroy --recursive --force ${SWEEP_POOL}/step8-foreign 2>/dev/null || true`,
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
