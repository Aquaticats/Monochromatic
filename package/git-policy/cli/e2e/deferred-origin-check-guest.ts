#!/usr/bin/env node
/**
 Drive plan step 15's guest arm in a throwaway `mvm` guest: the deferred-origin check against a real pool.

 The check reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This
 driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate inside it, runs the arm, and destroys the guest.

 The four arms, as the design's "Measurements owed" asks (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
 M1 and control 28):
 - the planned case: an unmarked origin goes on, and the mark reads `0` through the same read the wrapper uses;
 - a marked origin: `zfs destroy -d` marks the clone's origin snapshot, the mark reads `1`, and the check refuses;
 - the reap: destroying the clone's own snapshots and then the head takes the marked snapshot in the same transaction,
   which is the loss the refusal avoids, so an arm that cannot see it cannot show the refusal matters;
 - a moved origin: `zfs promote` moves the lineage under the tuple the removal recorded, and the check refuses on an
   identity.

 Usage: node e2e/deferred-origin-check-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `deferred-origin-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `DEFERRED` line below and no `SKIPPED` line, so an arm that returned without running fails the run
 instead of reporting a green suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

/** Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/deferred-origin-check';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the library and its tests may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds the guest arm may take. */
const ARM_TIMEOUT_MS = 900_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/** The environment the arm reads, so it never opens the host's device. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_DEFERRED_ORIGIN_GUEST';
/** The pool the arm builds and destroys. */
const POOL = 'zfsstep15';
/** The folder the pool's image and the fixtures live under. */
const FIXTURE_ROOT = '/home/zfs-step-15-fixture';
/** The OpenZFS 2.4.4 prefix the guest's recipe installs. */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/** The `PATH` the guest's OpenZFS userspace needs. */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/** The lines a run must find in the guest's arm log, one per arm it performed. */
const EXPECTED_MARKERS = [
  'DEFERRED planned=proceed mark=0',
  'DEFERRED marked counts clones=1 holds=Counted(0)',
  'DEFERRED marked=refused step=origin-mark',
  'DEFERRED reaped=gone origin=',
  'DEFERRED promoted=refused',
  'DEFERRED done',
] as const;

/** The step, the guest name, the log directory and whether `all` keeps the guest. */
interface Options {
  readonly step: string;
  readonly name: string;
  readonly logDir: string;
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
    name: value('--name') ?? 'deferred-origin-check',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'deferred-origin-guest-logs'),
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

/** Create the guest, refusing a name another run holds. */
function create(chosen: Options): void {
  recipe({ chosen, step: 'create', });
}

/** Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest. */
function prepare(chosen: Options): void {
  recipe({ chosen, step: 'prepare', });
}

/**
 Push a repository-shaped snapshot of the wrapper, build it in the guest, and run the arm.

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
    'if test -f /root/subject.tar.gz; then :; else cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked > /root/deferred-origin-build.log 2>&1',
    'tail --lines=3 /root/deferred-origin-build.log',
  ].join('\n');
  guest({ chosen, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  const arm = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_DEFERRED_ORIGIN_POOL=${POOL}`,
    'export CLI_GIT_ZFS_DEFERRED_ORIGIN_PREFIX=' + OPENZFS_PREFIX,
    `export CLI_GIT_ZFS_DEFERRED_ORIGIN_ROOT=${FIXTURE_ROOT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --lib --run-ignored all --no-capture --status-level all ' +
      '--filterset "test(the_deferred_origin_guest_arms)" > /root/deferred-origin-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=40 /root/deferred-origin-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^SKIPPED' /root/deferred-origin-arm.log; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/deferred-origin-arm.log`;
    }),
    'cp /root/deferred-origin-arm.log /mnt/shared/deferred-origin-arm.log',
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n');
  guest({ chosen, script: arm, log: 'guest-deferred-origin.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/deferred-origin-arm.log', join(chosen.logDir, 'deferred-origin-arm.log'),], log: 'host-pull-arm.log', });
  const pulled = readFileSync(join(chosen.logDir, 'deferred-origin-arm.log'), 'utf8');
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
  if (/(^|\n)SKIPPED/m.test(pulled))
    throw new Error('the pulled arm log holds a SKIPPED line, so the arm did not run');
}

/** Destroy the guest and keep the list of what is left. */
function destroy(chosen: Options): void {
  recipe({ chosen, step: 'destroy', });
}

/**
 Create, prepare, run and destroy, in that order.

 @param chosen - the run's options

 @throws Error naming the step that refused
 */
async function all(chosen: Options): Promise<void> {
  create(chosen);
  prepare(chosen);
  await run(chosen);
  destroy(chosen);
}

/** The steps this driver knows, and nothing else. */
const steps: Record<string, (chosen: Options) => void | Promise<void>> = {
  create,
  prepare,
  run,
  destroy,
  all,
};

/** Run the step the command line names. */
async function main(): Promise<void> {
  const chosen: Options = options();
  const step = steps[chosen.step];
  if (step === undefined)
    throw new Error(`unknown step '${chosen.step}'; use create, prepare, run, destroy or all`);
  mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
  await step(chosen);
}

await main();
