#!/usr/bin/env node
/**
 Drive plan step 13's managed-Git-lock arms in one throwaway `mvm` guest, on a file-backed pool.

 The arms need a real ZFS clone mounted over a Git worktree slot and a file-backed pool, so nothing here may run on a
 person's machine: the driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`), makes the pool the arms clone on, builds this
 crate's `native_binary` test target inside the guest, runs the lock arm there, and destroys the guest.

 The arm is plan step 13's "VM measurements": item 6's lock arms, extended to installing the lock after pointer loss,
 and measurement `M2`'s mounted arm of the two-fact proof.

 Usage: node e2e/managed-git-lock-guest.ts <create|prepare|pool|run|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `lock-arm.log`, pulled back into the log directory, and a run passes only when it carries
 every `LOCK` line below and no `SKIP` line, so an arm that returned without running fails the run instead of
 reporting a green suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import { join, resolve, } from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

/** Where the crate is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/git-policy-cli-lock';
/** The checkout the arm's Git repository and its worktree live in, under the guest's own root. */
const CHECKOUT = '/root/lock-checkout';
/** The pool the arm's clone parent and clone live in, and the file it stands on. */
const POOL = 'git-policy-cli-store';
const POOL_IMAGE = '/home/git-policy-cli-lock-fixture/lock.img';
/** The `PATH` the guest's OpenZFS 2.4.4 userspace needs. */
const OPENZFS_PATH = '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';
/** The variable the arm reads, so it never runs outside the guest. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_LOCK_GUEST';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the test target may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds the arm may take. */
const ARM_TIMEOUT_MS = 1_800_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The lines a run must find in the guest's arm log, one per arm it performed. */
const EXPECTED_MARKERS = [
  'LOCK mounted slot-identity-answers=no',
  'LOCK mounted classified managed=yes',
  'LOCK classified managed-nonce=1 pointer=absent',
  'LOCK prune-kept-the-locked-registration=true',
  'LOCK operator class=an-operators-lock',
  'LOCK operator git-removal-refused=yes single-force-refused=yes',
  'LOCK put-back wrote=yes line=yes',
  'LOCK installed-lock-kept-the-registration=true',
  'LOCK release refused-with-another-clones-proof=yes',
  'LOCK released=yes registration-still-stands=true',
  'LOCK unlock-control-lost-the-registration=true',
  'LOCK done',
] as const;

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
    name: value('--name') ?? 'git-policy-lock',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'lock-guest-logs'),
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
 */
function guest({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): string {
  const output = openSync(join(chosen.logDir, log), 'w', 0o600);
  const command = `test "$(hostname)" = ${chosen.name} && timeout ${Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS} sh -ec ${shellQuote(script)}`;
  try {
    const result = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', command,], {
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
 Make the file-backed pool the arm's datasets live in, and the folder its image stands in.

 The pool is made without a mountpoint so the guest's own root keeps no mount from it; the arm's checkout dataset
 carries its own.

 @param chosen - the run's options
 */
function pool({ chosen, }: { readonly chosen: Options; }): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `mkdir --parents "$(dirname ${POOL_IMAGE})"`,
    `if ! zpool list -H -o name | grep --quiet '^${POOL}$'; then truncate --size 2G ${POOL_IMAGE}; zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE}; fi`,
    `zpool list -H -o name ${POOL}`,
  ].join('\n');
  guest({ chosen, script, log: 'guest-pool.log', timeout: ARM_TIMEOUT_MS, });
}

/**
 Build the test target inside the guest and run the managed-lock arm there.

 @param chosen - the run's options
 */
async function run(chosen: Options): Promise<void> {
  const context = join(chosen.logDir, 'snapshot');
  rmSync(context, { recursive: true, force: true, });
  mkdirSync(context, { recursive: true, mode: 0o700, });
  await copyWrapperSnapshot({ context, });
  const tarball = join(chosen.logDir, 'subject.tar.gz');
  host({ chosen, command: 'tar', args: ['--create', '--gzip', '--file', tarball, '--exclude=target', '--directory', context, '.',], log: 'host-package.log', });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, tarball, '/root/subject.tar.gz',], log: 'guest-push.log', });
  const build = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked > /root/lock-build.log 2>&1',
    'tail --lines=3 /root/lock-build.log',
  ].join('\n');
  guest({ chosen, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  pool({ chosen, });
  const arm = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${POOL}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${CHECKOUT}`,
    // The arm's own layout call destroys the pool's tree and its mounts first, so no stale mount is left to refuse.
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      `--filterset "test(zfs_managed_lock_guest)" > /root/lock-arm.log 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=60 /root/lock-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which every guest arm of this package holds.
    `if grep --quiet '^SKIP' /root/lock-arm.log; then echo 'the lock arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/lock-arm.log`;
    }),
    'cp /root/lock-arm.log /mnt/shared/lock-arm.log',
  ].join('\n');
  guest({ chosen, script: arm, log: 'guest-lock-arm.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/lock-arm.log', join(chosen.logDir, 'lock-arm.log'),], log: 'host-pull-lock-arm.log', });
  const pulled = readFileSync(join(chosen.logDir, 'lock-arm.log'), 'utf8');
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
  if (/(^|\n)SKIP/m.test(pulled))
    throw new Error('the pulled arm log holds a SKIP line, so the lock arm did not run');
}

/** The steps by name, and the whole run. */
const steps: Record<string, (chosen: Options) => Promise<void> | void> = {
  create: (chosen) => recipe({ chosen, step: 'create', }),
  prepare: (chosen) => recipe({ chosen, step: 'prepare', }),
  pool: (chosen) => pool({ chosen, }),
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
      console.log(`git-policy lock guest: --keep-guest kept '${chosen.name}' to inspect and destroy`);
    else if (!chosen.keepGuest && created) {
      try {
        recipe({ chosen, step: 'destroy', });
      } catch (destruction) {
        if (failure === null)
          throw destruction;
        console.error(`git-policy lock guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
      }
    } else if (!chosen.keepGuest)
      console.error(`git-policy lock guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
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
console.log(`git-policy lock guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
