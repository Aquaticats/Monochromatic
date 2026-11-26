#!/usr/bin/env node
/**
 Drive plan step 12's slot classes, its move and removal refusals, its prune pointer and its repair report end to
 end in a throwaway `mvm` guest.

 The arm needs a real pool, a real Git registration and a real clone: it reads one slot's class from the binding
 record, the mount intent and the mount table while the clone is mounted and again while it is unmounted, and it
 shows the wrapper's refusal beside Git's own answer, which is a busy mount while mounted and success while
 unmounted. The same run drives the wrapper's own command path: `git worktree move` and `git worktree remove` are
 refused where Git's own move succeeded, the prune pass writes the recorded pointer back before Git would drop the
 registration, and `git worktree repair` prints what it can and cannot restore. Every one of those is a ZFS or Git
 fact no host control can make, so the arm runs only inside the guest the ZFS binding's recipe creates
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`).

 Usage: node e2e/slot-state-and-move-refusal-guest.ts <create|prepare|run|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `slot-state-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `SLOT-STATE`, `MOVE-REFUSAL`, `REMOVAL-REFUSAL`, `PRUNE-POINTER` and `REPAIR` line below and no `SKIP`
 line, so an arm that returned without running fails the run instead of reporting a green suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

/** Where the crate is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/git-policy-cli-slot-state';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the test target may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds the guest arm may take. */
const ARM_TIMEOUT_MS = 900_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The environment variable the arm reads, so it never writes a guest fixture on a machine somebody uses. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_SLOT_STATE_GUEST';
/** The checkout the arm registers, clones and moves, under the guest's own root. */
const CHECKOUT = '/root/slot-state-checkout';
/** The OpenZFS userspace the guest built, which every `zfs` the arm runs comes from. */
const SBIN = '/opt/openzfs-2.4.4/sbin';
/** The pool this driver makes for the arm, and the file-backed vdev it stands on. */
const POOL = 'git-policy-cli-slot-state';
const POOL_IMAGE = '/home/git-policy-cli-slot-state-fixture/pool.img';

/** The lines a run must find in the guest's arm log, one per class and per move arm plan step 12 runs. */
const EXPECTED_MARKERS = [
  'SLOT-STATE mounted class=mounted managed clone dataset=',
  'SLOT-STATE record-behind-mount class=cannot be proved',
  'SLOT-STATE unmounted class=known unmounted clone intent=mount-requested',
  'SLOT-STATE pending-add class=pending add',
  'SLOT-STATE pending-removal class=pending removal',
  'MOVE-REFUSAL mounted cause=bound',
  'MOVE-REFUSAL mounted git=refused-busy',
  'MOVE-REFUSAL wrapper-mounted cause=bound status=refused',
  'MOVE-REFUSAL unmounted cause=bound',
  'MOVE-REFUSAL wrapper-unmounted cause=bound status=refused',
  'MOVE-REFUSAL unmounted git=succeeded',
  'REMOVAL-REFUSAL wrapper class=known-unmounted-clone status=refused',
  'PRUNE-POINTER intact registration=kept',
  'PRUNE-POINTER restored registration=kept',
  'REPAIR damaged registration=stands',
  'REPAIR pruned registration=gone',
  'SLOT-STATE done',
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
    name: value('--name') ?? 'git-policy-cli-slot-state',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'slot-state-guest-logs'),
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
 Build the test target inside the guest and run plan step 12's arm there.

 The snapshot is the same repository-shaped tree the container gate builds from (`bin/native-wrapper-snapshot.ts`), so
 the arm compiles against the path crates the wrapper links, the ZFS binding among them.

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
  const prepare = [
    'set -eu',
    // `mvm push` lands the tarball in the guest's shared folder, not at the path it was given.
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    // The arm needs a pool of its own; the checkout dataset and its clone are made on it by the arm.
    `mkdir --parents ${POOL_IMAGE.slice(0, POOL_IMAGE.lastIndexOf('/'))}`,
    `if ! ${SBIN}/zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${POOL_IMAGE}`,
    `  ${SBIN}/zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE}`,
    'fi',
    `mkdir --parents ${CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked --features test-only-switches > /root/slot-state-build.log 2>&1',
    'tail --lines=3 /root/slot-state-build.log',
  ].join('\n');
  guest({ chosen, script: prepare, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  const arm = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${POOL}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${SBIN}`,
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${CHECKOUT}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all ' +
      `--filterset "test(zfs_slot_state_guest)" > /root/slot-state-arm.log 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=60 /root/slot-state-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/slot-state-arm.log; then echo 'the slot-state arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/slot-state-arm.log`;
    }),
    'cp /root/slot-state-arm.log /mnt/shared/slot-state-arm.log',
  ].join('\n');
  guest({ chosen, script: arm, log: 'guest-slot-state.log', timeout: ARM_TIMEOUT_MS * 2, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/slot-state-arm.log', join(chosen.logDir, 'slot-state-arm.log'),], log: 'host-pull-slot-state.log', });
  const pulled = readFileSync(join(chosen.logDir, 'slot-state-arm.log'), 'utf8');
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled slot-state log lacks ${marker}; the guest ran the arm without performing it`);
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
      console.log(`git-policy slot-state guest: --keep-guest kept '${chosen.name}' to inspect and destroy`);
    else if (!chosen.keepGuest && created) {
      try {
        recipe({ chosen, step: 'destroy', });
      } catch (destruction) {
        if (failure === null)
          throw destruction;
        console.error(`git-policy slot-state guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
      }
    } else if (!chosen.keepGuest)
      console.error(`git-policy slot-state guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
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
console.log(`git-policy slot-state guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
