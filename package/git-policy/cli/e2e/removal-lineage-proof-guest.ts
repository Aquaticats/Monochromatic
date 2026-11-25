#!/usr/bin/env node
/**
 Drive plan step 14's guest arm in a throwaway `mvm` guest: the removal's lineage proof, bookmark rule, intent and plan against a real pool.

 The proof reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This driver
 borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the arm, and destroys
 the guest.

 The arms, as the plan's step 14 asks (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 14"):
 - the healthy removal enters, writes its plan and prints the one own snapshot it loses;
 - VM item 11's promote: the proof refuses a promoted clone before the first delete, every protected snapshot and
   bookmark tuple survives, and reverse promotion restores the recorded lineage;
 - VM item 13's bookmarks: a bookmark a live snapshot explains passes, and one whose snapshot went refuses and is named;
 - VM item 12's replacement and grants: the decided grant set carries no lineage-changing permission, and a same-name
   replacement clone refuses on its GUID.

 Usage: node e2e/removal-lineage-proof-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `removal-lineage-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `REMOVAL` line below and no `SKIPPED` line, so an arm that returned without running fails the run instead
 of reporting a green suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

/** Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/removal-lineage-proof';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the library and its tests may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds the guest arm may take. */
const ARM_TIMEOUT_MS = 1_200_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/** The environment the arm reads, so it never opens the host's device. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST';
/** The environment the refusal arm reads, which drives the built wrapper's `git worktree remove`. */
const REFUSAL_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_REFUSAL_GUEST';
/** The pool the refusal arm builds its own datasets in, beside the proof arm's pool. */
const REFUSAL_POOL = 'removal-refusal';
/** The folder the refusal arm's file-backed pool image lives in. */
const REFUSAL_ROOT = '/home/removal-refusal-fixture';
/** The checkout the refusal arm registers a real worktree in. */
const REFUSAL_CHECKOUT = '/root/removal-refusal-checkout';
/** The pool the arm builds and destroys. */
const POOL = 'zfsstep14';
/** The folder the pool's image and the fixture repository live under. */
const FIXTURE_ROOT = '/home/zfs-step-14-fixture';
/** The OpenZFS 2.4.4 prefix the guest's recipe installs. */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/** The `PATH` the guest's OpenZFS userspace needs. */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/** The lines a run must find in the guest's arm log, one per arm it performed. */
const EXPECTED_MARKERS = [
  'REMOVAL healthy=entered',
  'REMOVAL promoted=refused',
  'REMOVAL reverse-promotion=proved',
  'REMOVAL bookmark-live=passed',
  'REMOVAL bookmark-stale=refused',
  'REMOVAL grants=lineage-changing-absent',
  'REMOVAL replacement=refused',
  'REMOVAL done',
  'REMOVAL-ARMS healthy=proved wrote=nothing',
  'REMOVAL-ARMS promoted=refused',
  'REMOVAL-ARMS inherited-tuple=measured',
  'REMOVAL-ARMS reverse-promotion=proved',
  'REMOVAL-ARMS bookmark-live=printed',
  'REMOVAL-ARMS bookmark-stale=refused',
  'REMOVAL-ARMS replacement=refused',
  'REMOVAL-ARMS done',
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
    name: value('--name') ?? 'removal-lineage-proof',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'removal-lineage-guest-logs'),
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
    // The shared copy is the tree this run made; a guest reused by an earlier run still holds that run's tarball at
    // /root, so the shared copy always wins and a missing one refuses rather than building the older tree.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    // The refusal arm's own pool, made from a file so nothing of the guest's root disk is a vdev.
    `mkdir --parents ${REFUSAL_ROOT}`,
    `mkdir --parents ${REFUSAL_CHECKOUT}`,
    `if ! ${OPENZFS_PREFIX}/sbin/zpool list -H -o name ${REFUSAL_POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${REFUSAL_ROOT}/pool.img`,
    `  ${OPENZFS_PREFIX}/sbin/zpool create -f -O mountpoint=none ${REFUSAL_POOL} ${REFUSAL_ROOT}/pool.img`,
    'fi',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free --mebytes',
    // Two compiling jobs: the guest has four cores, and a linking test binary of this crate needs more than a
    // gigabyte, so an out-of-memory kill here would look like a compile failure. The log is copied to the shared
    // folder whatever the build answered, so a guest's death still leaves the reason on the host.
    'if cargo build --release --tests --locked --jobs 2 > /root/removal-lineage-build.log 2>&1; then status=0; else status=$?; fi',
    'cp /root/removal-lineage-build.log /mnt/shared/removal-lineage-build.log',
    'tail --lines=5 /root/removal-lineage-build.log',
    'test "$status" = 0',
  ].join('\n');
  guest({ chosen, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/removal-lineage-build.log', join(chosen.logDir, 'removal-lineage-build.log'),], log: 'host-pull-build.log', });
  const arm = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_POOL=${POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_ROOT=${FIXTURE_ROOT}`,
    `export ${REFUSAL_GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${REFUSAL_POOL}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${OPENZFS_PREFIX}/sbin`,
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${REFUSAL_CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --lib --run-ignored all --no-capture --status-level all ' +
      '--filterset "test(the_removal_lineage_guest_arms)" > /root/removal-lineage-arm.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=40 /root/removal-lineage-arm.log',
    // The refusal arms run through the built wrapper's own `git worktree remove`, in the same log.
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_removal_refusal_guest)" >> /root/removal-lineage-arm.log 2>&1; then refusal=0; else refusal=$?; fi',
    'tail --lines=60 /root/removal-lineage-arm.log',
    'test "$status" = 0',
    'test "$refusal" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^SKIPPED' /root/removal-lineage-arm.log; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker)} /root/removal-lineage-arm.log`;
    }),
    'cp /root/removal-lineage-arm.log /mnt/shared/removal-lineage-arm.log',
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n');
  guest({ chosen, script: arm, log: 'guest-removal-lineage.log', timeout: ARM_TIMEOUT_MS, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, '/mnt/shared/removal-lineage-arm.log', join(chosen.logDir, 'removal-lineage-arm.log'),], log: 'host-pull-arm.log', });
  const pulled = readFileSync(join(chosen.logDir, 'removal-lineage-arm.log'), 'utf8');
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
 Create, prepare, run and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone
 */
async function all(chosen: Options): Promise<void> {
  let failure: unknown = null;
  let created = false;
  try {
    create(chosen);
    created = true;
    prepare(chosen);
    await run(chosen);
  } catch (thrown) {
    failure = thrown;
  }
  if (!created) {
    console.error(`git-policy removal-lineage guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
    if (failure !== null)
      throw failure;
    return;
  }
  try {
    destroy(chosen);
  } catch (destruction) {
    if (failure === null)
      throw destruction;
    console.error(`git-policy removal-lineage guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
  }
  if (failure !== null)
    throw failure;
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
