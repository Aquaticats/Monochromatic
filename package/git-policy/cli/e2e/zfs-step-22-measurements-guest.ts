#!/usr/bin/env node
/**
 Drive plan step 22's guest measurements in one throwaway `mvm` guest.

 The arms build a file-backed pool, clone datasets and mounts as root, so no control on a person's machine may run them.
 This driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, builds this crate's `native_binary` test target inside it, runs step 22's
 measurement arm there, pulls its log, and destroys the guest.

 The arms are the ones the plan's step 22 asks for
 (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 22"): the add against ordinary Git under the same
 workload with an interleaved control, the mount request's wait, the unmount's, the sweep's cost and the lost-event
 behaviour, what expiration and removal free, the forced arm's three waits, and the request budget.

 Usage: node e2e/zfs-step-22-measurements-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `step-22-measurements-arm.log`, pulled back into the log directory, and a run passes only
 when it carries the `STEP22` lines the arm performs and no `SKIP` line, so an arm that returned without running fails the
 run instead of reporting a green suite.
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
const CRATE_DIR = '/opt/step22-measurement';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's build of the test target may take. */
const CRATE_BUILD_TIMEOUT_MS = 5_400_000;
/** Milliseconds the guest arm may take. */
const ARM_TIMEOUT_MS = 3_600_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** How often the host asks the guest whether a detached script has finished, in milliseconds. */
const POLL_INTERVAL_MS = 15_000;
/** The swap file the guest adds, in gibibytes, so one rustc at five gigabytes does not exhaust its memory. */
const SWAP_GIB = '6G';
/** The environment the arm reads, so it never opens a device on a machine somebody uses. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_STEP22_GUEST';
/** The pool the arm's arms build their datasets in. */
const POOL = 'zfsstep22';
/** The guest folder the file-backed pool's image lives in. */
const FIXTURE_ROOT = '/home/step22-fixture';
/** The checkout the design's dataset is mounted at. */
const CHECKOUT = '/root/step22-checkout';
/** The arm's log inside the guest's shared folder. */
const ARM_LOG = 'step-22-measurements-arm.log';
/** The lines a run must find in the guest's arm log, one per arm it performed. */
const EXPECTED_MARKERS = [
  'STEP22 pool=',
  'STEP22 add arm=git round=0',
  'STEP22 add arm=clone round=0',
  'STEP22 add-total control-ms=',
  'STEP22 sweep clones=1',
  'STEP22 unmount holder=none',
  'STEP22 forced-arm t-wait-ms=',
  'STEP22 forced-arm t-stop-ms=',
  'STEP22 retention after-removal=',
  'STEP22 request-budget names=2048',
  'STEP22 done',
] as const;

/** The step, the guest name and the log directory. */
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
    name: value('--name') ?? 'zfs-step-22-measurements',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'step-22-guest-logs'),
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
 Start one long guest script detached, so it cannot lose its answer, and leave its log in the shared folder.

 @param chosen - the run's options

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the shared folder

 @param timeout - milliseconds the script may take inside the guest

 @throws Error when the guest cannot be asked to start it
 */
function guestStart({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): void {
  const seconds = Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS;
  const step = join(chosen.logDir, `${log}.step.sh`);
  writeFileSync(step, [`test "$(hostname)" = ${chosen.name}`, `timeout ${seconds} sh -ec ${shellQuote(script)} > /mnt/shared/${log} 2>&1`, 'status=$?', `echo "DONE=$status" >> /mnt/shared/${log}`, ''].join('\n'), { mode: 0o600, });
  const launch = join(chosen.logDir, `${log}.launch.sh`);
  writeFileSync(launch, [`setsid nohup sh /mnt/shared/${log}.step.sh < /dev/null > /dev/null 2>&1 &`, 'exit 0', ''].join('\n'), { mode: 0o600, });
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

/** Install OpenZFS 2.4.4 and the Rust toolchain in the guest. */
function prepare(chosen: Options): void {
  recipe({ chosen, step: 'prepare', });
}

/**
 Push a repository-shaped snapshot of the wrapper, build its test target in the guest, lay the file-backed pool, and run
 the measurement arm.

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
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, tarball, '/root/subject.tar.gz',], log: 'guest-push.log', });
  const build = [
    'set -eu',
    // `mvm push` lands every file in the guest's shared folder, so the copy the
    // build unpacks is taken from there and a missing one refuses.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    // The tree is unpacked over whatever the guest already holds, so a second
    // run of the arm reuses the build's target folder instead of compiling the
    // whole crate again; a renamed source file would linger, and this arm only
    // adds files.
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    // One rustc for a crate this size wants more than five gigabytes and the guest has eight, so one job and a swap
    // file are what make the build fit; the customer's own guest-arm drivers measure the same bound.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIB} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked > /root/step22-build.log 2>&1',
    'tail --lines=3 /root/step22-build.log',
  ].join('\n');
  guestStart({ chosen, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
  guestFinish({ chosen, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS + 180_000, });

  const pool = [
    'set -eu',
    'export PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
    `mkdir --parents ${FIXTURE_ROOT}`,
    `mkdir --parents ${CHECKOUT}`,
    `if ! zpool list -H -o name ${POOL} > /dev/null 2>&1; then truncate --size 2G ${FIXTURE_ROOT}/pool.img; zpool create -f -O mountpoint=none ${POOL} ${FIXTURE_ROOT}/pool.img; fi`,
    `zpool list -H -o name,size ${POOL}`,
  ].join('\n');
  guestStart({ chosen, script: pool, log: 'guest-pool.log', timeout: 300_000, });
  guestFinish({ chosen, log: 'guest-pool.log', timeout: 300_000 + 120_000, });

  const arm = [
    'set -eu',
    'export PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export CLI_GIT_ZFS_STEP22_POOL=${POOL}`,
    'export CLI_GIT_ZFS_STEP22_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_STEP22_ROOT=${CHECKOUT}`,
    `export CLI_GIT_ZFS_STEP22_WORKLOAD=${CRATE_DIR}/package/git-policy/cli/src`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      `--filterset "test(the_step_22_guest_measurements)" > /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=120 /root/${ARM_LOG}`,
    'test "$status" = 0',
  ].join('\n');
  guestStart({ chosen, script: arm, log: 'guest-arm.log', timeout: ARM_TIMEOUT_MS, });
  guestFinish({ chosen, log: 'guest-arm.log', timeout: ARM_TIMEOUT_MS + 180_000, });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'pull', chosen.name, `/mnt/shared/${ARM_LOG}`, join(chosen.logDir, ARM_LOG),], log: 'host-pull-arm.log', });
  const pulled = readFileSync(join(chosen.logDir, ARM_LOG), 'utf8');
  for (const marker of EXPECTED_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the pulled arm log lacks ${marker}; the guest ran the arm without performing it`);
  }
  if (/(^|\n)SKIP/m.test(pulled))
    throw new Error('the pulled arm log holds a SKIP line, so the arm did not run');
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
    console.error(`git-policy step 22 guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
    if (failure !== null)
      throw failure;
    return;
  }
  try {
    destroy(chosen);
  } catch (destruction) {
    if (failure === null)
      throw destruction;
    console.error(`git-policy step 22 guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
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
  const chosen: Options = await Promise.resolve(options());
  const step = steps[chosen.step];
  if (step === undefined)
    throw new Error(`unknown step '${chosen.step}'; use create, prepare, run, destroy or all`);
  mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
  await step(chosen);
}

await main();
