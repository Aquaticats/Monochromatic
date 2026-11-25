#!/usr/bin/env node
/**
 Drive plan step 14's guest arm in a throwaway `mvm` guest: the removal's lineage proof, bookmark rule, intent and plan against a real pool.

 The proof reads `/dev/zfs`, so no control on a person's machine may run it against a pool it did not build. This driver
 borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the arm, and destroys
 the guest.

 The arms, as the plan's steps 14 and 15 ask
 (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 14" and "Step 15"):
 step 14's:
 - the healthy removal enters, writes its plan and prints the one own snapshot it loses;
 - VM item 11's promote: the proof refuses a promoted clone before the first delete, every protected snapshot and
   bookmark tuple survives, and reverse promotion restores the recorded lineage;
 - VM item 13's bookmarks: a bookmark a live snapshot explains passes, and one whose snapshot went refuses and is named;
 - VM item 12's replacement and grants: the decided grant set carries no lineage-changing permission, and a same-name
   replacement clone refuses on its GUID;

 and step 15's, through the library's writing path, because the removal's command path stays cut until step 16:
 - a healthy removal destroys the clone's own snapshots, records every answer and stops before the head destroy;
 - the look inside the entry refuses a mark that is already there, with nothing written;
 - the first check refuses a mark that appears after the look, with no request sent;
 - the owed measurement's second half: the mark appears after the batch and the head check refuses, with the plan
   keeping what the batch destroyed and the Git registration surviving;
 - a replacement clone whose different origin is marked refuses at the entry's own proof, with no request sent.

 Usage: node e2e/removal-lineage-proof-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `removal-lineage-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `REMOVAL` and `DESTRUCTION` line below and no `SKIPPED` line, so an arm that returned without running fails
 the run instead of reporting a green suite.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';
import {
  basename,
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
/** How often the host asks the guest whether a detached script has finished, in milliseconds. */
const POLL_INTERVAL_MS = 15_000;
/** The swap file the guest adds, in gibibytes, so one rustc at five gigabytes does not exhaust its memory. */
const SWAP_GIB = '6G';

/** The environment the arm reads, so it never opens the host's device. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST';
/** The environment step 15's destruction arm reads, so it never opens the host's device. */
const DESTRUCTION_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_DESTRUCTION_GUEST';
/** The environment the refusal arm reads, which drives the built wrapper's `git worktree remove`. */
const REFUSAL_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_REFUSAL_GUEST';
/** The pool the refusal arm builds its own datasets in, beside the proof arm's pool. */
const REFUSAL_POOL = 'removal-refusal';
/** The folder the refusal arm's file-backed pool image lives in. */
const REFUSAL_ROOT = '/home/removal-refusal-fixture';
/** The checkout the refusal arm registers a real worktree in. */
const REFUSAL_CHECKOUT = '/root/removal-refusal-checkout';
/** The pool step 14's arm builds and destroys. */
const POOL = 'zfsstep14';
/** The folder step 14's pool image and fixture repository live under. */
const FIXTURE_ROOT = '/home/zfs-step-14-fixture';
/** The pool step 15's destruction arm builds and destroys, so the two arms never share one. */
const DESTRUCTION_POOL = 'zfsstep15destruction';
/** The folder step 15's pool image and fixture repository live under. */
const DESTRUCTION_ROOT = '/home/zfs-step-15-destruction-fixture';
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
  'DESTRUCTION healthy=destroyed',
  'DESTRUCTION look=refused',
  'DESTRUCTION first-check=refused',
  'DESTRUCTION head-check=refused',
  'DESTRUCTION replacement=refused',
  'DESTRUCTION done',
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
 Start one long guest script detached, so it cannot lose its answer, and leave its log in the shared folder.

 The guest's crate build ran for ten minutes and the two compiling jobs of the first runs exhausted the guest's memory,
 where the agent lost the command's process and the reason with it. The script therefore runs under `setsid nohup`,
 writes its log where the host can read it, and ends by appending `DONE=<status>` to that log.

 @param chosen - the run's options

 @param script - the shell script

 @param log - the log's name inside the shared folder

 @param timeout - milliseconds the script may take inside the guest

 @throws Error when the guest cannot be asked to start it
 */
function guestStart({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): void {
  const seconds = Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS;
  // The step and the launcher are files, not one command line: `mvm exec` hands its argument to one shell, and a
  // detached job written inline nested that shell inside another and the ampersand ended up inside the inner quoting,
  // so the host waited for the whole build instead of its start.
  const step = join(chosen.logDir, `${log}.step.sh`);
  writeFileSync(step, [`timeout ${seconds} sh -ec ${shellQuote(script)} > /mnt/shared/${log} 2>&1`, 'status=$?', `echo "DONE=$status" >> /mnt/shared/${log}`, ''].join('\n'), { mode: 0o600, });
  const launch = join(chosen.logDir, `${log}.launch.sh`);
  writeFileSync(launch, [`setsid nohup sh /mnt/shared/${log}.step.sh < /dev/null > /dev/null 2>&1 &`, 'exit 0', ''].join('\n'), { mode: 0o600, });
  // `mvm push` lands each file in the guest's shared folder under the destination's own name, which is where the
  // launcher looks for the step.
  for (const from of [step, launch,]) {
    host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, from, `/shared/${basename(from)}`,], log: `guest-push-${log}.log`, });
  }
  const result = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', 'sh', `/mnt/shared/${log}.launch.sh`,], {
    stdio: 'ignore',
    timeout: HOST_COMMAND_TIMEOUT_MS,
  });
  if (result.error !== undefined)
    throw result.error;
}

/**
 Wait for one detached script's own `DONE=` line, pull its log, and refuse a status other than zero.

 @param chosen - the run's options

 @param log - the log's name inside the shared folder

 @param timeout - milliseconds the host waits before giving up

 @returns the pulled log

 @throws Error naming the status the script ended with, or the wait that ended
 */
function guestFinish({ chosen, log, timeout, }: { readonly chosen: Options; readonly log: string; readonly timeout: number; }): string {
  const path = `/mnt/shared/${log}`;
  const deadline = Date.now() + timeout;
  let finished = false;
  while (Date.now() < deadline) {
    const answer = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', 'tail', '--lines=1', path,], {
      encoding: 'utf8',
      timeout: HOST_COMMAND_TIMEOUT_MS,
    });
    if ((answer.stdout ?? '').trim().startsWith('DONE=')) {
      finished = true;
      break;
    }
    spawnSync('sleep', [String(POLL_INTERVAL_MS / 1000),], { timeout: POLL_INTERVAL_MS * 2, });
  }
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, path, join(chosen.logDir, log),], log: `host-pull-${log}`, });
  const pulled = readFileSync(join(chosen.logDir, log), 'utf8');
  if (!finished)
    throw new Error(`${log}: the guest script did not finish inside ${String(timeout)} ms; see ${log}`);
  const status = /^DONE=(?<status>\d+)$/m.exec(pulled)?.groups?.status;
  if (status !== '0')
    throw new Error(`${log}: the guest script ended ${status ?? 'without a status'}; see ${log}`);
  return pulled;
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
    // `free` with no option, because the guest's procps has no `--mebytes` and `set -e` would end the build there.
    'free',
    // One rustc for a crate this size wants more than five gigabytes, and the guest has eight: the first runs compiled
    // two at a time, both together exhausted the guest's memory, and the kernel killed one of them, which looked like
    // a compile failure. One job and a swap file are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIB} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'cargo build --release --tests --locked --jobs 1',
  ].join('\n');
  guestStart({ chosen, script: build, log: 'removal-lineage-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  guestFinish({ chosen, log: 'removal-lineage-build.log', timeout: CRATE_BUILD_TIMEOUT_MS + 180_000, });
  const arm = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_POOL=${POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_ROOT=${FIXTURE_ROOT}`,
    `export ${DESTRUCTION_GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_POOL=${DESTRUCTION_POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_ROOT=${DESTRUCTION_ROOT}`,
    `export ${REFUSAL_GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${REFUSAL_POOL}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${OPENZFS_PREFIX}/sbin`,
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${REFUSAL_CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --lib --run-ignored all --no-capture --status-level all ' +
      '--filterset "test(the_removal_lineage_guest_arms) | test(the_removal_destruction_guest_arms)" > /root/removal-lineage-arm.log 2>&1; then status=0; else status=$?; fi',
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
    `zpool destroy -f ${DESTRUCTION_POOL} 2>/dev/null || true`,
    `rm --force ${DESTRUCTION_ROOT}/pool.img 2>/dev/null || true`,
  ].join('\n');
  guestStart({ chosen, script: arm, log: 'removal-lineage-arm-shell.log', timeout: ARM_TIMEOUT_MS, });
  guestFinish({ chosen, log: 'removal-lineage-arm-shell.log', timeout: ARM_TIMEOUT_MS + 120_000, });
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
