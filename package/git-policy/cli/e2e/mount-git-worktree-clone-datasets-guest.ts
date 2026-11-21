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

/** The lines a run must find in the guest's publication-fsync log. */
const EXPECTED_FSYNC_MARKERS = [
  'PROBE append with fsync of the file and the folder:',
  'PROBE publication of one slot record: stage, write, fsync, link, unlink, folder fsync:',
] as const;

/** Where the store's publication-fsync arm writes inside the guest, on the dataset the store lives on. */
const FSYNC_FOLDER = '/var/lib/monochromatic-git-policy-cli-fsync-probe';

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
}

/**
 Run the store's publication cost against the pool the store lives on, inside the same guest.

 The probe is pushed rather than snapshot, because it lives in the package's `doc/` folder and the snapshot holds only
 what a build reads. It is run on a folder under `/var/lib`, which is the dataset the store itself is on, so the arm is
 what replaced the lock-hold record's subject.

 @param chosen - the run's options

 @throws Error naming the marker the pulled log lacks
 */
function publicationFsync({ chosen, }: { readonly chosen: Options; }): void {
  const probe = resolve(import.meta.dirname, '..', 'doc', 'slot-identity-store-write-cost', 'probe-reaches-storage.py');
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, probe, '/root/probe-reaches-storage.py',], log: 'guest-push-probe.log', });
  const script = [
    'set -eu',
    `mkdir --parents ${FSYNC_FOLDER}`,
    // `/sbin` is where Ubuntu puts the pool commands and an `mvm exec` shell does not always carry it.
    'export PATH=/sbin:/usr/sbin:$PATH',
    // The pool's own vdev, read inside the guest, so the counters the probe reads are the ones the store's writes reach.
    'pool_device=$(zpool status -P | grep --only-matching "/dev/[^ ]*" | head --lines=1)',
    'test -n "$pool_device"',
    'pool_disk=$(basename "$(readlink --canonicalize "$(lsblk --noheadings --output PKNAME --paths "$pool_device")")")',
    `python3 -I /root/probe-reaches-storage.py ${FSYNC_FOLDER} ${FSYNC_ROUNDS} --pool-device "$(basename "$pool_device")" --pool-disk "$pool_disk" > /mnt/shared/store-fsync.log 2>&1`,
    'rmdir ' + FSYNC_FOLDER,
  ].join('\n');
  guest({ chosen, script, log: 'guest-store-fsync.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/store-fsync.log', join(chosen.logDir, 'store-fsync.log'),], log: 'host-pull-store-fsync.log', });
  const pulled = readFileSync(join(chosen.logDir, 'store-fsync.log'), 'utf8');
  for (const marker of EXPECTED_FSYNC_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled publication-fsync log lacks ${marker}; the guest ran the probe without that arm`);
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
